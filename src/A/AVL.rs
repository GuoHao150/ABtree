use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::FromIterator;
use std::mem;
use std::{marker::PhantomData, ptr::NonNull};

/// An AVL balanced tree with owned nodes.
pub struct AVL<K: Ord, V> {
    root_node: OpNode<K, V>,
    len: usize,
    _marker: PhantomData<Box<Node<K, V>>>,
}

type OpNode<K: Ord, V> = Option<NonNull<Node<K, V>>>;

/// Inner Node to store data
struct Node<K: Ord, V> {
    key: K,
    value: V,
    parent_node: OpNode<K, V>,
    left_node: OpNode<K, V>,
    right_node: OpNode<K, V>,
    height: isize,
}

impl<K: Ord, V> Node<K, V> {
    fn new(k: K, v: V) -> Self {
        Node {
            key: k,
            value: v,
            parent_node: None,
            left_node: None,
            right_node: None,
            height: 1,
        }
    }

    /// Take a boxed Node and return the key and value
    #[inline]
    fn into_element(n: Box<Node<K, V>>) -> (K, V) {
        (n.key, n.value)
    }

    /// get parent node
    #[inline]
    fn get_parent(node: OpNode<K, V>) -> OpNode<K, V> {
        node.as_ref()
            .and_then(|n| unsafe { (*n.as_ptr()).parent_node })
    }

    /// set the parent node for a child node
    #[inline]
    fn set_parent(child_node: OpNode<K, V>, parent_node: OpNode<K, V>) {
        if parent_node.is_none() {
            child_node.as_ref().map(|n| unsafe {
                (*n.as_ptr()).parent_node = None;
            });
            return;
        }
        let parent_k = parent_node.as_ref().map(|p| unsafe { &(*p.as_ptr()).key });
        let child_k = child_node.as_ref().map(|c| unsafe { &(*c.as_ptr()).key });
        let ordering = parent_k.and_then(|pk| child_k.map(|ck| pk.cmp(ck)));

        if let Some(o) = ordering {
            match o {
                Ordering::Equal => {}
                Ordering::Less => {
                    parent_node.as_ref().map(|p| unsafe {
                        (*p.as_ptr()).right_node = child_node;
                    });
                    child_node.as_ref().map(|c| unsafe {
                        (*c.as_ptr()).parent_node = parent_node;
                    });
                }
                Ordering::Greater => {
                    parent_node.as_ref().map(|p| unsafe {
                        (*p.as_ptr()).left_node = child_node;
                    });
                    child_node.as_ref().map(|c| unsafe {
                        (*c.as_ptr()).parent_node = parent_node;
                    });
                }
            }
        }
    }

    /// unlink a child node and parent node
    /// and set their linkage to None
    #[inline]
    fn unlink(child_node: OpNode<K, V>, parent_node: OpNode<K, V>) {
        let parent_k = parent_node.as_ref().map(|p| unsafe { &(*p.as_ptr()).key });
        let child_k = child_node.as_ref().map(|c| unsafe { &(*c.as_ptr()).key });
        let ordering = parent_k.and_then(|pk| child_k.map(|ck| pk.cmp(ck)));

        if let Some(o) = ordering {
            match o {
                Ordering::Equal => {}
                Ordering::Less => {
                    parent_node.as_ref().map(|p| unsafe {
                        (*p.as_ptr()).right_node = None;
                    });
                    child_node.as_ref().map(|c| unsafe {
                        (*c.as_ptr()).parent_node = None;
                    });
                }
                Ordering::Greater => {
                    parent_node.as_ref().map(|n| unsafe {
                        (*n.as_ptr()).left_node = None;
                    });
                    child_node.as_ref().map(|n| unsafe {
                        (*n.as_ptr()).parent_node = None;
                    });
                }
            }
        }
    }

    /// Set the left node of a given node
    #[inline]
    fn set_left(cur_node: OpNode<K, V>, left_node: OpNode<K, V>) {
        cur_node.as_ref().map(|cur| unsafe {
            (*cur.as_ptr()).left_node = left_node;
        });
        left_node.as_ref().map(|l| unsafe {
            (*l.as_ptr()).parent_node = cur_node;
        });
    }

    /// set the right node of a given node
    #[inline]
    fn set_right(cur_node: OpNode<K, V>, right_node: OpNode<K, V>) {
        cur_node.as_ref().map(|cur| unsafe {
            (*cur.as_ptr()).right_node = right_node;
        });
        right_node.as_ref().map(|l| unsafe {
            (*l.as_ptr()).parent_node = cur_node;
        });
    }

    /// get the left node
    #[inline]
    fn get_left(node: OpNode<K, V>) -> OpNode<K, V> {
        node.as_ref()
            .and_then(|n| unsafe { (*n.as_ptr()).left_node })
    }

    /// get the right node
    #[inline]
    fn get_right(node: OpNode<K, V>) -> OpNode<K, V> {
        node.as_ref()
            .and_then(|n| unsafe { (*n.as_ptr()).right_node })
    }

    /// get the height of a node
    #[inline]
    fn get_height(node: OpNode<K, V>) -> isize {
        if node.is_none() {
            0
        } else {
            node.as_ref()
                .map(|n| unsafe { (*n.as_ptr()).height })
                .unwrap()
        }
    }

    /// set the height of a node
    #[inline]
    fn set_height(node: OpNode<K, V>, h: isize) {
        node.as_ref().map(|n| unsafe {
            (*n.as_ptr()).height = h;
        });
    }

    /// update the height of a node
    /// and the heights of children must be updated
    #[inline]
    fn update_height(node: OpNode<K, V>) {
        let l_height = Node::get_height(Node::get_left(node));
        let r_height = Node::get_height(Node::get_right(node));
        let new_height = if l_height < r_height {
            r_height + 1
        } else {
            l_height + 1
        };
        node.as_ref().map(|n| unsafe {
            (*n.as_ptr()).height = new_height;
        });
    }

    /// give a node compare with some K
    #[inline]
    fn compare_key(node: OpNode<K, V>, k: &K) -> Option<Ordering> {
        node.as_ref().map(|n| unsafe { (*n.as_ptr()).key.cmp(k) })
    }

    /// Wrap a NonNull Node into a Box
    #[inline]
    fn boxed_node(node: OpNode<K, V>) -> Option<Box<Node<K, V>>> {
        node.map(|n| unsafe { Box::from_raw(n.as_ptr()) })
    }
}

impl<K: Ord, V> AVL<K, V> {
    /// For a given node this method will update all the heights of it's children using dynamic programming
    /// and then update all the height of it's upper nodes
    fn _update_nodes_height_down_up(&mut self, node: OpNode<K, V>) {
        let mut todo = vec![node];
        let mut updated = HashMap::<OpNode<K, V>, isize>::new();
        // updates all heights of lower nodes
        'outer: loop {
            let c = todo.pop();
            match c {
                None => {
                    if todo.is_empty() {
                        break 'outer;
                    } else {
                        continue 'outer;
                    }
                }
                Some(cur_node) => {
                    let cur_left = Node::get_left(cur_node);
                    let cur_right = Node::get_right(cur_node);
                    let adjs: Vec<OpNode<K, V>> = vec![cur_left, cur_right]
                        .into_iter()
                        .filter(|n| n.is_some())
                        .collect();

                    if cur_node.is_none() && adjs.is_empty() && todo.is_empty() {
                        break 'outer;
                    } else if cur_node.is_none() && !adjs.is_empty() && todo.is_empty() {
                        break 'outer;
                    } else if cur_node.is_none() && adjs.is_empty() && !todo.is_empty() {
                        continue 'outer;
                    } else if cur_node.is_none() && !adjs.is_empty() && !todo.is_empty() {
                        adjs.into_iter().for_each(|n| {
                            todo.push(n);
                        });
                        continue 'outer;
                    } else if cur_node.is_some() && adjs.is_empty() && todo.is_empty() {
                        Node::set_height(cur_node, 1);
                        break 'outer;
                    } else if cur_node.is_some() && adjs.is_empty() && !todo.is_empty() {
                        Node::set_height(cur_node, 1);
                        updated.insert(cur_node, 1);
                        continue 'outer;
                    } else {
                        //cur_node.is_some() && !adjs.is_empty() && (todo.is_empty() || !todo.is_empty()) {
                        let adjs_not_seen: Vec<OpNode<K, V>> = adjs
                            .iter()
                            .map(|n| *n)
                            .filter(|n| !updated.contains_key(n))
                            .collect();
                        if adjs_not_seen.is_empty() {
                            // all child nodes have been updated
                            let new_height =
                                (*adjs.iter().flat_map(|n| updated.get(n)).max().unwrap()) + 1;

                            cur_node.as_ref().map(|cur| unsafe {
                                (*cur.as_ptr()).height = new_height;
                            });
                            updated.insert(cur_node, new_height);
                            continue 'outer;
                        } else {
                            todo.push(cur_node);
                            adjs_not_seen.into_iter().for_each(|n| {
                                todo.push(n);
                            });
                            continue 'outer;
                        }
                    }
                }
            }
        }
        // the input node and it's children have been updated
        // now update it's upper nodes if possible
        self._update_all_upper_nodes(node);
    }

    /// Because every time we call add or remove only one node will changed
    /// so this method will assume the cur_node is the changed node
    /// and it's height has been updated but not it's parent's height
    fn _update_all_upper_nodes(&mut self, mut cur_node: OpNode<K, V>) {
        loop {
            let cur_parent = Node::get_parent(cur_node);
            if cur_parent.is_none() {
                break;
            }
            let parnet_l_height = Node::get_height(Node::get_left(cur_parent));
            let parnet_r_height = Node::get_height(Node::get_right(cur_parent));
            let new_p_height = if parnet_l_height < parnet_r_height {
                parnet_r_height + 1
            } else {
                parnet_l_height + 1
            };
            let old_p_height = Node::get_height(cur_parent);
            if new_p_height == old_p_height {
                break;
            }
            Node::set_height(cur_parent, new_p_height);
            cur_node = Node::get_parent(cur_parent);
            continue;
        }
    }

    /// Get balance factor
    fn _get_balance_factor(&self, node: OpNode<K, V>) -> isize {
        if node.is_none() {
            0
        } else {
            let left_height = Node::get_height(Node::get_left(node));
            let right_height = Node::get_height(Node::get_right(node));

            match (left_height, right_height) {
                (0, 0) => 0,
                (l, 0) => l,
                (0, r) => -r,
                (l, r) => l - r,
            }
        }
    }

    /// To tell if this tree is balanced
    fn _is_balanced_tree(&self) -> bool {
        let mut queue = VecDeque::new();
        queue.push_back(self.root_node);
        loop {
            let c = queue.pop_front();
            match c {
                None => {
                    break true;
                }
                Some(cur_node) => {
                    if self._get_balance_factor(cur_node).abs() > 1 {
                        break false;
                    }
                    let adjs = vec![Node::get_left(cur_node), Node::get_right(cur_node)];
                    adjs.into_iter().for_each(|n| {
                        if n.is_some() {
                            queue.push_back(n);
                        }
                    });
                    continue;
                }
            }
        }
    }

    /// Right rotate for node `y`
    ///        y                              x
    ///       / \                           /   \
    ///      x   T4                         z     y
    ///     / \       - - - - - - - ->    / \   / \
    ///    z   T3                       T1  T2 T3 T4
    ///   / \
    /// T1   T2
    fn _right_rotate(&mut self, y: OpNode<K, V>) {
        let y_parent = Node::get_parent(y);
        let x = Node::get_left(y);
        let t3 = Node::get_right(x);

        Node::set_parent(t3, y);
        Node::set_parent(y, x);
        Node::set_parent(x, y_parent);

        if y_parent.is_none() {
            self.root_node = x;
        }
        // node x and node y needs to update the height
        Node::update_height(y);
        Node::update_height(x);
        self._update_all_upper_nodes(x);
    }

    /// Left ratate for node `y`
    ///    y                             x
    ///  /  \                          /   \
    /// T1   x                        y     z
    ///     / \   - - - - - - - ->   / \   / \
    ///   T2  z                     T1 T2 T3 T4
    ///      / \
    ///     T3 T4
    fn _left_rotate(&mut self, y: OpNode<K, V>) {
        let y_parent = Node::get_parent(y);
        let x = Node::get_right(y);
        let t2 = Node::get_left(x);

        Node::set_right(y, t2);
        Node::set_left(x, y);
        Node::set_parent(x, y_parent);

        if y_parent.is_none() {
            self.root_node = x;
        }
        // node x and node y needs to update the height
        Node::update_height(y);
        Node::update_height(x);
        self._update_all_upper_nodes(x);
    }

    /// Private method for adding a key-value pair
    fn _add_loop(&mut self, k: K, v: V) {
        if self.root_node.is_none() {
            let new_node = Box::new(Node::new(k, v));
            let new_raw = NonNull::new(Box::into_raw(new_node));
            self.len += 1;
            self.root_node = new_raw;
            return;
        }
        let mut todo = vec![self.root_node];
        'outer: loop {
            let c = todo.pop();
            match c {
                None => {
                    break 'outer;
                }
                Some(cur_node) => {
                    let cur_left = Node::get_left(cur_node);
                    let cur_right = Node::get_right(cur_node);
                    let cmp = Node::compare_key(cur_node, &k);

                    match cmp {
                        None => {
                            break 'outer;
                        }
                        Some(Ordering::Equal) => {
                            cur_node.as_ref().map(|cur| unsafe {
                                (*cur.as_ptr()).value = v;
                            });
                            break 'outer;
                        }
                        Some(Ordering::Greater) => {
                            if cur_left.is_some() {
                                todo.push(cur_left);
                                continue 'outer;
                            } else {
                                self.len += 1;
                                let new_node = Box::new(Node::new(k, v));
                                let new_raw = NonNull::new(Box::into_raw(new_node));
                                Node::set_left(cur_node, new_raw);
                                // try to rebalance
                                self._update_nodes_height_down_up(self.root_node);
                                self._try_to_rebalancing(new_raw);
                                break 'outer;
                            }
                        }
                        Some(Ordering::Less) => {
                            if cur_right.is_some() {
                                todo.push(cur_right);
                                continue 'outer;
                            } else {
                                self.len += 1;
                                let new_node = Box::new(Node::new(k, v));
                                let new_raw = NonNull::new(Box::into_raw(new_node));
                                Node::set_right(cur_node, new_raw);
                                self._update_nodes_height_down_up(self.root_node);
                                self._try_to_rebalancing(new_raw);
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Pop out the minimun node
    fn _pop_min_loop(&mut self) -> OpNode<K, V> {
        let cur_min = self._find_min_child(self.root_node);
        cur_min
            .as_ref()
            .and_then(|n| unsafe { self._remove_node(&(*n.as_ptr()).key) })
    }

    /// Return a boxed node after pop out the minimum node
    fn _pop_min(&mut self) -> Option<Box<Node<K, V>>> {
        Node::boxed_node(self._pop_min_loop())
    }

    /// Pop out the maximum node
    fn _pop_max_loop(&mut self) -> OpNode<K, V> {
        let cur_max = self._find_max_child(self.root_node);
        cur_max
            .as_ref()
            .and_then(|n| unsafe { self._remove_node(&(*n.as_ptr()).key) })
    }

    /// Returnn a boxed node after pop out the maximum node
    fn _pop_max(&mut self) -> Option<Box<Node<K, V>>> {
        Node::boxed_node(self._pop_max_loop())
    }

    /// Given a ref key return the node
    fn _get_node(&self, k: &K) -> OpNode<K, V> {
        let mut cur_node = self.root_node;
        loop {
            let cmp = Node::compare_key(cur_node, k);
            match cmp {
                None => {
                    break None;
                }
                Some(Ordering::Equal) => {
                    break cur_node;
                }
                Some(Ordering::Greater) => {
                    cur_node = Node::get_left(cur_node);
                    continue;
                }
                Some(Ordering::Less) => {
                    cur_node = Node::get_right(cur_node);
                    continue;
                }
            }
        }
    }

    // When all heights have been updated call this methods to
    // find the first unbalanced node from bottom to top
    fn _get_unbalanced_node(&mut self, mut cur_node: OpNode<K, V>) -> OpNode<K, V> {
        loop {
            let cur_parent = Node::get_parent(cur_node);
            let cur_b_factor = self._get_balance_factor(cur_node);
            if cur_b_factor.abs() > 1 {
                break cur_node;
            } else {
                if cur_parent.is_some() {
                    cur_node = cur_parent;
                    continue;
                } else {
                    break None;
                }
            }
        }
    }

    /// Rebalancing
    fn _try_to_rebalancing(&mut self, cur_node: OpNode<K, V>) {
        let unbalanced = self._get_unbalanced_node(cur_node);
        if unbalanced.is_some() {
            self._rebalancing(unbalanced);
        }
    }

    /// Find maximun child node
    fn _find_max_child(&self, mut cur_node: OpNode<K, V>) -> OpNode<K, V> {
        loop {
            let cur_right = Node::get_right(cur_node);
            if cur_right.is_some() {
                cur_node = cur_right;
                continue;
            } else {
                break cur_node;
            }
        }
    }

    /// Find minimum child node
    fn _find_min_child(&self, mut cur_node: OpNode<K, V>) -> OpNode<K, V> {
        loop {
            let cur_left = Node::get_left(cur_node);
            if cur_left.is_some() {
                cur_node = cur_left;
                continue;
            } else {
                break cur_node;
            }
        }
    }

    /// remove node
    fn _remove_node(&mut self, k: &K) -> OpNode<K, V> {
        let target_node = self._get_node(k);
        match target_node {
            None => None,
            cur_node @ Some(_) => {
                self.len -= 1;
                let cur_parent = Node::get_parent(cur_node);
                let cur_left = Node::get_left(cur_node);
                let cur_right = Node::get_right(cur_node);

                if cur_left.is_some() && cur_right.is_some() && cur_parent.is_some() {
                    let cur_left_max = self._find_max_child(cur_left);
                    Node::unlink(cur_left_max, Node::get_parent(cur_left_max));
                    Node::set_parent(cur_left_max, cur_parent);
                    Node::set_right(cur_left_max, cur_right);
                    if !cur_left_max.eq(&cur_left) {
                        Node::set_left(cur_left_max, cur_left);
                    }
                    self._update_nodes_height_down_up(cur_left_max);
                    self._try_to_rebalancing(cur_left_max);
                    return cur_node;
                } else if cur_left.is_some() && cur_right.is_some() && cur_parent.is_none() {
                    let cur_left_max = self._find_max_child(cur_left);
                    Node::unlink(cur_left_max, Node::get_parent(cur_left_max));
                    self.root_node = cur_left_max;
                    Node::set_parent(cur_left_max, None);
                    Node::set_right(cur_left_max, cur_right);
                    if !cur_left_max.eq(&cur_left) {
                        Node::set_left(cur_left_max, cur_left);
                    }
                    self._update_nodes_height_down_up(cur_left_max);
                    self._try_to_rebalancing(cur_left_max);
                    return cur_node;
                } else if cur_left.is_some() && cur_right.is_none() && cur_parent.is_some() {
                    let cur_left_max = self._find_max_child(cur_left);
                    Node::unlink(cur_left_max, Node::get_parent(cur_left_max));
                    Node::set_parent(cur_left_max, cur_parent);
                    if !cur_left_max.eq(&cur_left) {
                        Node::set_left(cur_left_max, cur_left);
                    }
                    self._update_nodes_height_down_up(cur_left_max);
                    self._try_to_rebalancing(cur_left_max);
                    return cur_node;
                } else if cur_left.is_some() && cur_right.is_none() && cur_parent.is_none() {
                    let cur_left_max = self._find_max_child(cur_left);
                    Node::unlink(cur_left_max, Node::get_parent(cur_left_max));
                    self.root_node = cur_left_max;
                    Node::set_parent(cur_left_max, None);
                    if !cur_left_max.eq(&cur_left) {
                        Node::set_left(cur_left_max, cur_left);
                    }
                    self._update_nodes_height_down_up(cur_left_max);
                    self._try_to_rebalancing(cur_left_max);
                    return cur_node;
                } else if cur_left.is_none() && cur_right.is_some() && cur_parent.is_some() {
                    Node::set_parent(cur_right, cur_parent);
                    self._update_nodes_height_down_up(cur_right);
                    self._try_to_rebalancing(cur_right);
                    return cur_node;
                } else if cur_left.is_none() && cur_right.is_some() && cur_parent.is_none() {
                    Node::set_parent(cur_right, None);
                    self.root_node = cur_right;
                    self._update_nodes_height_down_up(cur_right);
                    self._try_to_rebalancing(cur_right);
                    return cur_node;
                } else if cur_left.is_none() && cur_right.is_none() && cur_parent.is_some() {
                    Node::unlink(cur_node, cur_parent);
                    self._update_nodes_height_down_up(cur_parent);
                    self._try_to_rebalancing(cur_parent);
                    return cur_node;
                } else {
                    // cur_left is none and cur_right is none and cur_parent is none
                    self.root_node = None;
                    return cur_node;
                }
            }
        }
    }

    // For the heights of nodes will be updated after rotate
    // the rebalanceing methods is also heights updated after calling
    fn _rebalancing(&mut self, cur_node: OpNode<K, V>) {
        let cur_left = Node::get_left(cur_node);
        let cur_right = Node::get_right(cur_node);
        let cur_b_factor = self._get_balance_factor(cur_node);
        let cur_left_b_factor = self._get_balance_factor(cur_left);
        let cur_right_b_factor = self._get_balance_factor(cur_right);
        if cur_b_factor > 1 && cur_left_b_factor >= 0 {
            self._right_rotate(cur_node);
        }

        if cur_b_factor < -1 && cur_right_b_factor <= 0 {
            self._left_rotate(cur_node);
        }

        if cur_b_factor > 1 && cur_left_b_factor < 0 {
            self._left_rotate(cur_left);
            self._right_rotate(cur_node);
        }

        if cur_b_factor < -1 && cur_right_b_factor > 0 {
            self._right_rotate(cur_right);
            self._left_rotate(cur_node);
        }
    }
}

/// Drop
impl<K: Ord, V> Drop for AVL<K, V> {
    fn drop(&mut self) {
        struct DropGuard<'a, K: Ord, V>(&'a mut AVL<K, V>);
        impl<'a, K: Ord, V> Drop for DropGuard<'a, K, V> {
            fn drop(&mut self) {
                while self.0._pop_min().is_some() {}
            }
        }

        while let Some(b) = self._pop_min() {
            let guard = DropGuard(self);
            drop(b);
            mem::forget(guard);
        }
    }
}

pub struct IntoIter<K: Ord, V>(AVL<K, V>);

impl<K: Ord, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0._pop_min().map(|n| Node::into_element(n))
    }
}

impl<K: Ord, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0._pop_max().map(|n| Node::into_element(n))
    }
}

impl<K: Ord, V> Drop for IntoIter<K, V> {
    fn drop(&mut self) {
        struct DropGuard<'a, K: Ord, V>(&'a mut IntoIter<K, V>);

        impl<'a, K: Ord, V> Drop for DropGuard<'a, K, V> {
            fn drop(&mut self) {
                while self.0.next().is_some() {}
            }
        }

        while let Some(d) = self.next() {
            let guard = DropGuard(self);
            drop(d);
            mem::forget(guard);
        }
    }
}

pub struct Iter<'a, K: Ord, V> {
    next_nodes: Vec<OpNode<K, V>>,
    seen: HashSet<NonNull<Node<K, V>>>,
    next_back_nodes: Vec<OpNode<K, V>>,
    seen_back: HashSet<NonNull<Node<K, V>>>,
    _marker: PhantomData<&'a Node<K, V>>,
}

impl<'a, K: Ord, V> Iter<'a, K, V> {
    fn next_ascending(&mut self) -> OpNode<K, V> {
        loop {
            let n = self.next_nodes.pop();
            match n {
                None => {
                    if self.next_nodes.len() == 0 {
                        break None;
                    } else {
                        continue;
                    }
                }
                Some(node) => unsafe {
                    let left = node
                        .as_ref()
                        .and_then(|n| (*n.as_ptr()).left_node)
                        .filter(|n| !self.seen.contains(n));
                    let right = node
                        .as_ref()
                        .and_then(|n| (*n.as_ptr()).right_node)
                        .filter(|n| !self.seen.contains(n));

                    if left.is_some() && right.is_some() {
                        self.next_nodes.push(node);
                        self.next_nodes.push(left);
                        continue;
                    } else if left.is_some() && right.is_none() {
                        self.next_nodes.push(node);
                        self.next_nodes.push(left);
                        continue;
                    } else if left.is_none() && right.is_some() {
                        self.next_nodes.push(right);
                        node.map(|n| {
                            self.seen.insert(n);
                        });
                        break node;
                    } else {
                        node.map(|n| {
                            self.seen.insert(n);
                        });
                        break node;
                    }
                },
            }
        }
    }

    fn next_descending(&mut self) -> OpNode<K, V> {
        loop {
            let n = self.next_back_nodes.pop();
            match n {
                None => {
                    if self.next_back_nodes.len() == 0 {
                        break None;
                    } else {
                        continue;
                    }
                }
                Some(node) => unsafe {
                    let left = node
                        .as_ref()
                        .and_then(|n| (*n.as_ptr()).left_node)
                        .filter(|n| !self.seen_back.contains(n));
                    let right = node
                        .as_ref()
                        .and_then(|n| (*n.as_ptr()).right_node)
                        .filter(|n| !self.seen_back.contains(n));

                    if left.is_some() && right.is_some() {
                        self.next_back_nodes.push(node);
                        self.next_back_nodes.push(right);
                        continue;
                    } else if left.is_some() && right.is_none() {
                        self.next_back_nodes.push(left);
                        node.map(|n| {
                            self.seen_back.insert(n);
                        });
                        break node;
                    } else if left.is_none() && right.is_some() {
                        self.next_back_nodes.push(node);
                        self.next_back_nodes.push(right);
                        continue;
                    } else {
                        // left is none and right is node
                        node.map(|n| {
                            self.seen.insert(n);
                        });
                        break node;
                    }
                },
            }
        }
    }
}

impl<'a, K: Ord, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.next_ascending()
            .as_ref()
            .map(|n| unsafe { (&(*n.as_ptr()).key, &(*n.as_ptr()).value) })
    }
}

impl<'a, K: Ord, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next_descending()
            .as_ref()
            .map(|n| unsafe { (&(*n.as_ptr()).key, &(*n.as_ptr()).value) })
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for AVL<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let inputs: Vec<_> = iter.into_iter().collect();
        if inputs.is_empty() {
            return AVL::<K, V>::new();
        }
        let mut out = AVL::<K, V>::new();
        for (k, v) in inputs {
            out.add(k, v);
        }
        out
    }
}

impl<K: Ord, V> IntoIterator for AVL<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<K: Ord + Copy, V: Copy> Clone for AVL<K, V> {
    fn clone(&self) -> Self {
        let mut out = AVL::<K, V>::new();
        for (k, v) in self.iter() {
            out.add(*k, *v);
        }
        out
    }
}

unsafe impl<K: Ord + Send, V: Send> Send for AVL<K, V> {}

unsafe impl<K: Ord + Sync, V: Sync> Sync for AVL<K, V> {}

unsafe impl<K: Ord + Send, V: Send> Send for Iter<'_, K, V> {}

unsafe impl<K: Ord + Sync, V: Sync> Sync for Iter<'_, K, V> {}

impl<K: Ord, V> AVL<K, V> {
    /// Create an empty AVL tree
    ///
    /// # Examples
    ///
    /// ```
    /// use ABtree::AVL;
    ///
    /// let t = AVL::<i32, i32>::new();
    /// ```
    pub fn new() -> Self {
        AVL {
            root_node: None,
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Adding key-value pair into the tree
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::AVL;
    /// let mut t = AVL::<i32, i32>::new();
    /// t.add(2, 3);
    /// assert_eq!(t.len(), 1);
    /// ```
    pub fn add(&mut self, k: K, v: V) {
        self._add_loop(k, v);
    }
    /// Adding key-value pair into the tree
    /// this method is an alias of method add
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::AVL;
    /// let mut t = AVL::<i32, i32>::new();
    /// t.insert(2, 3);
    /// assert_eq!(t.len(), 1);
    /// ```
    pub fn insert(&mut self, k: K, v: V) {
        self._add_loop(k, v);
    }

    /// Setting a key-value pair
    /// if the key exists it will update the value
    /// otherwise it will insert the key-value into the tree
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::AVL;
    /// let mut t = AVL::<i32, i32>::new();
    /// t.set(2, 2);
    /// t.set(2, 31);
    /// assert_eq!(t.get(&2), Some(&31));
    /// ```
    pub fn set(&mut self, k: K, v: V) {
        self.add(k, v)
    }

    /// Get the length of this tree
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::AVL;
    /// let mut t = AVL::<i32, i32>::new();
    /// t.insert(2, 2);
    /// t.insert(3, 3);
    /// assert_eq!(t.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Provides a forward iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use ABtree::AVL;
    ///
    /// let mut t: AVL<u32, u32> = AVL::new();
    ///
    /// t.insert(0, 0);
    /// t.insert(1, 1);
    /// t.insert(2, 2);
    ///
    /// let mut iter = t.iter();
    /// assert_eq!(iter.next(), Some((&0, &0)));
    /// assert_eq!(iter.next(), Some((&1, &1)));
    /// assert_eq!(iter.next_back(), Some((&2, &2)));
    /// ```
    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        let nodes = vec![self.root_node];
        let seen = HashSet::<NonNull<Node<K, V>>>::new();
        let nodes_back = vec![self.root_node];
        let seen_back = HashSet::<NonNull<Node<K, V>>>::new();
        Iter {
            next_nodes: nodes,
            seen: seen,
            next_back_nodes: nodes_back,
            seen_back: seen_back,
            _marker: PhantomData,
        }
    }

    /// Containment check
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::AVL;
    ///
    /// let mut t: AVL<u32, u32> = AVL::new();
    ///
    /// t.insert(0, 0);
    /// t.insert(1, 1);
    /// t.insert(2, 2);
    /// assert!(t.contains(&1));
    /// ```
    pub fn contains(&self, k: &K) -> bool {
        self.iter().any(|n| n.0.eq(k))
    }

    /// Removing key-value pair
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::AVL;
    ///
    /// let mut t: AVL<u32, u32> = AVL::new();
    ///
    /// t.insert(0, 0);
    /// t.insert(1, 1);
    /// t.insert(2, 2);
    /// assert_eq!(t.remove(&1), Some(1));
    /// assert_eq!(t.len(), 2);
    /// ```
    pub fn remove(&mut self, k: &K) -> Option<V> {
        let out = self._remove_node(k);
        Node::boxed_node(out).map(|n| n.value)
    }

    /// Peeking the root node
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::AVL;
    ///
    /// let mut t: AVL<u32, u32> = AVL::new();
    ///
    /// t.insert(0, 0);
    /// t.insert(1, 1);
    /// t.insert(2, 2);
    /// assert_eq!(t.peek_root(), Some((&1, &1)));
    /// ```
    pub fn peek_root<'a>(&'a self) -> Option<(&'a K, &'a V)> {
        self.root_node
            .as_ref()
            .map(|n| unsafe { (&(*n.as_ptr()).key, &(*n.as_ptr()).value) })
    }

    /// To check if shis tree is balanced
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::AVL;
    ///
    /// let mut t: AVL<u32, u32> = AVL::new();
    ///
    /// t.insert(0, 0);
    /// t.insert(1, 1);
    /// t.insert(2, 2);
    /// assert_eq!(t.is_balanced_tree(), true);
    /// ```
    pub fn is_balanced_tree(&self) -> bool {
        self._is_balanced_tree()
    }

    /// To check if shis tree is empty
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::AVL;
    ///
    /// let mut t: AVL<u32, u32> = AVL::new();
    ///
    /// t.insert(0, 0);
    /// t.insert(1, 1);
    /// t.insert(2, 2);
    /// assert_eq!(t.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Removes all elements from the AVL tree
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::AVL;
    ///
    /// let mut t: AVL<u32, u32> = AVL::new();
    ///
    /// t.insert(0, 0);
    /// t.insert(1, 1);
    /// t.insert(2, 2);
    /// t.clear();
    /// assert_eq!(t.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Get the value by key
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::AVL;
    ///
    /// let mut t: AVL<u32, u32> = AVL::new();
    ///
    /// t.insert(0, 0);
    /// t.insert(1, 1);
    /// t.insert(2, 2);
    /// assert_eq!(t.get(&1), Some(&1));
    /// ```
    pub fn get(&self, k: &K) -> Option<&V> {
        let mut outs: Vec<_> = self.iter().filter(|n| n.0.eq(k)).collect();
        if outs.len() == 0 {
            None
        } else {
            outs.pop().map(|o| o.1)
        }
    }
}
