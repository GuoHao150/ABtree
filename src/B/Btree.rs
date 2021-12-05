use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

///A b-tree with owned nodes
///and what makes it different from the BTreeMap in std
///is it choose arbitrary number as the maximum number of keys
///in each node, as long as the number grater or equal to 3
///Note if the number has been set to 3 it becomes a 2-3 tree
pub struct BTree<K: Ord, V> {
    root_node: OpNode<K, V>,
    len: usize,
    max_key_num: usize, // the maximun number of inner data
    min_key_num: usize, // the minimun number of inner data
    _marker: PhantomData<Box<Node<K, V>>>,
}

struct Node<K: Ord, V> {
    data: InnerData<K, V>,
    parent: OpNode<K, V>,
    children: Children<K, V>,
}

struct Data<K: Ord, V> {
    key: K,
    value: V,
}

type OpNode<K: Ord, V> = Option<NonNull<Node<K, V>>>;
type InnerData<K: Ord, V> = Option<NonNull<VecDeque<Data<K, V>>>>;
type Children<K: Ord, V> = Option<NonNull<VecDeque<OpNode<K, V>>>>;

impl<K: Ord, V> Node<K, V> {
    /// Get parent node
    #[inline]
    fn get_parent(node: OpNode<K, V>) -> OpNode<K, V> {
        node.as_ref().and_then(|n| unsafe { (*n.as_ptr()).parent })
    }

    /// Given a node return it's Children VecDeque
    #[inline]
    fn get_children(node: OpNode<K, V>) -> Children<K, V> {
        node.as_ref()
            .and_then(|n| unsafe { (*n.as_ptr()).children })
    }

    /// Given a node return one of it's child by index
    #[inline]
    fn get_child_by_index(node: OpNode<K, V>, idx: usize) -> OpNode<K, V> {
        let children_size = Node::get_children_size(node);
        if children_size == 0 {
            None
        } else {
            //assert!(idx < children_size, "Index out of range");
            Node::get_children(node).and_then(|c| unsafe {
                if idx < children_size {
                    (*c.as_ptr())[idx]
                } else {
                    None
                }
            })
        }
    }

    /// Given a node and one of it's child return the index of child
    #[inline]
    fn get_child_position(node: OpNode<K, V>, child: OpNode<K, V>) -> Option<usize> {
        let children = Node::get_children(node);
        children
            .as_ref()
            .and_then(|n| unsafe { (*n.as_ptr()).iter().position(|x| x.eq(&child)) })
    }

    /// Set parent node
    /// Note this will not set child node for input parent node
    #[inline]
    fn set_parent(child: OpNode<K, V>, parent: OpNode<K, V>) {
        child.as_ref().map(|c| unsafe {
            (*c.as_ptr()).parent = parent;
        });
    }

    /// Get the length of children for the given node
    #[inline]
    fn get_children_size(node: OpNode<K, V>) -> usize {
        match node {
            None => 0,
            Some(ref n) => unsafe {
                let children = (*n.as_ptr()).children;
                match children {
                    None => 0,
                    Some(ref c) => (*c.as_ptr()).len(),
                }
            },
        }
    }

    /// Get the inner data vec of a node
    #[inline]
    fn get_inner_data(node: OpNode<K, V>) -> InnerData<K, V> {
        node.as_ref().and_then(|n| unsafe { (*n.as_ptr()).data })
    }

    /// Get the inner node size, which is the length
    /// of VecDeque of inner data
    #[inline]
    fn get_data_size(node: OpNode<K, V>) -> usize {
        match node {
            None => 0,
            Some(ref n) => unsafe {
                let data = (*n.as_ptr()).data;
                match data {
                    None => 0,
                    Some(ref d) => (*d.as_ptr()).len(),
                }
            },
        }
    }

    /// Give a node and key reutrn the index of key's position
    /// in the inner data
    #[inline]
    fn get_key_index(node: OpNode<K, V>, k: &K) -> Option<usize> {
        let data = Node::get_inner_data(node);
        match data {
            None => None,
            Some(ref inner_d) => unsafe {
                let iter = (*inner_d.as_ptr()).iter().enumerate();
                let mut out: Option<usize> = None;
                for (idx, d) in iter {
                    if d.key.eq(k) {
                        out = Some(idx);
                    }
                }
                out
            },
        }
    }

    /// Push back a child node into node
    #[inline]
    fn push_back_child(node: OpNode<K, V>, child: OpNode<K, V>) {
        if child.is_some() {
            let children = Node::get_children(node);
            match children {
                None => unsafe {
                    let new_children = Box::new(VecDeque::from_iter([child]));
                    let new_children = NonNull::new(Box::into_raw(new_children));
                    node.as_ref().map(|n| {
                        (*n.as_ptr()).children = new_children;
                    });
                },
                Some(ref c) => unsafe { (*c.as_ptr()).push_back(child) },
            }
        }
    }

    /// Push front a child node into node
    #[inline]
    fn push_front_child(node: OpNode<K, V>, child: OpNode<K, V>) {
        if child.is_some() {
            let children = Node::get_children(node);
            match children {
                None => unsafe {
                    let new_children = Box::new(VecDeque::from_iter([child]));
                    let new_children = NonNull::new(Box::into_raw(new_children));
                    node.as_ref().map(|n| {
                        (*n.as_ptr()).children = new_children;
                    });
                },
                Some(ref c) => unsafe { (*c.as_ptr()).push_front(child) },
            }
        }
    }

    /// Pushing back an inner Data into a node
    #[inline]
    fn push_back_inner_data(node: OpNode<K, V>, inner_data: Option<Data<K, V>>) {
        if inner_data.is_some() {
            let i_data = inner_data.unwrap();
            node.as_ref().and_then(|n| unsafe {
                (*n.as_ptr()).data.as_ref().map(|v| {
                    (*v.as_ptr()).push_back(i_data);
                })
            });
        }
    }

    /// Pusing front an inner Data into a node
    #[inline]
    fn push_front_inner_data(node: OpNode<K, V>, inner_data: Option<Data<K, V>>) {
        if inner_data.is_some() {
            let i_data = inner_data.unwrap();
            node.as_ref().and_then(|n| unsafe {
                (*n.as_ptr()).data.as_ref().map(|v| {
                    (*v.as_ptr()).push_front(i_data);
                })
            });
        }
    }

    /// Given a node and key compare them and find a proper child
    /// If the key exists in the node it will return the node
    fn moving_target(mut cur_node: OpNode<K, V>, k: &K) -> OpNode<K, V> {
        'outer: loop {
            let inner_data = Node::get_inner_data(cur_node);
            let data_size = Node::get_data_size(cur_node);
            let children_size = Node::get_children_size(cur_node);
            if children_size == 0 {
                break cur_node;
            }
            match inner_data {
                None => {
                    break 'outer None;
                }
                Some(ref data) => unsafe {
                    let mut iter = (*data.as_ptr()).iter().enumerate();
                    'inner: while let Some((idx, x)) = iter.next() {
                        let ordering = x.key.cmp(k);
                        if idx == data_size - 1 {
                            match ordering {
                                Ordering::Equal => {
                                    break 'outer cur_node;
                                }
                                Ordering::Greater => {
                                    cur_node = Node::get_child_by_index(cur_node, idx);
                                    continue 'outer;
                                }
                                Ordering::Less => {
                                    cur_node = Node::get_child_by_index(cur_node, idx + 1);
                                    continue 'outer;
                                }
                            }
                        } else {
                            match ordering {
                                Ordering::Equal => {
                                    break 'outer cur_node;
                                }
                                Ordering::Greater => {
                                    cur_node = Node::get_child_by_index(cur_node, idx);
                                    continue 'outer;
                                }
                                Ordering::Less => {
                                    continue 'inner;
                                }
                            }
                        }
                    }
                },
            }
        }
    }

    /// Given a index remove a node's child
    #[inline]
    fn remove_child(node: OpNode<K, V>, idx: usize) -> OpNode<K, V> {
        let children = Node::get_children(node);
        let children_size = Node::get_children_size(node);
        assert!(children_size > idx, "Index {} out of boundary", idx);
        children
            .as_ref()
            .and_then(|c| unsafe { (*c.as_ptr()).remove(idx).unwrap_or(None) })
    }

    /// This method may not be useful
    #[inline]
    fn insert_child(node: OpNode<K, V>, idx: usize, new_c: OpNode<K, V>) {
        let children = Node::get_children(node);
        let children_size = Node::get_children_size(node);
        assert!(children_size >= idx, "Index {} out of boundary", idx);
        children
            .as_ref()
            .map(|c| unsafe { (*c.as_ptr()).insert(idx, new_c) });
    }

    /// Removing an inner data out of a node by the index
    #[inline]
    fn remove_data(node: OpNode<K, V>, idx: usize) -> Option<Data<K, V>> {
        let data = Node::get_inner_data(node);
        let data_size = Node::get_data_size(node);
        assert!(data_size > idx, "Index {} out of boundary", idx);
        data.as_ref()
            .and_then(|d| unsafe { (*d.as_ptr()).remove(idx) })
    }

    /// Insert a new Data into a node with a certain index
    #[inline]
    fn insert_data(node: OpNode<K, V>, idx: usize, d: Option<Data<K, V>>) {
        let data = Node::get_inner_data(node);
        let data_size = Node::get_data_size(node);
        assert!(data_size >= idx, "Index {} out of boundary", idx);
        if let Some(new_d) = d {
            data.as_ref()
                .map(|n| unsafe { (*n.as_ptr()).insert(idx, new_d) });
        }
    }

    /// The caller must guarantee the `new_d` is not None
    /// this method will put new Data in a sorted place
    /// And if the key exists it will update the value
    fn adding_data(node: OpNode<K, V>, new_d: Option<Data<K, V>>) -> OpNode<K, V> {
        let new_d = new_d.unwrap();
        let data = Node::get_inner_data(node);
        let data_size = Node::get_data_size(node);
        match data {
            None => {
                // create a new node with one key
                let d = Box::new(VecDeque::from_iter([new_d]));
                let n = Box::new(Node {
                    data: NonNull::new(Box::into_raw(d)),
                    parent: None,
                    children: None,
                });
                NonNull::new(Box::into_raw(n))
            }
            Some(ref inner_d) => unsafe {
                let mut iter = (*inner_d.as_ptr()).iter().enumerate();
                loop {
                    let next = iter.next();
                    if next.is_none() {
                        break node;
                    } else {
                        let (idx, x) = next.unwrap();
                        let ordering = x.key.cmp(&new_d.key);
                        match ordering {
                            Ordering::Equal => {
                                (*inner_d.as_ptr())[idx].value = new_d.value;
                                break node;
                            }
                            Ordering::Less => {
                                if idx < data_size - 1 {
                                    continue;
                                } else {
                                    Node::insert_data(node, data_size, Some(new_d));
                                    break node;
                                }
                            }
                            Ordering::Greater => {
                                Node::insert_data(node, idx, Some(new_d));
                                break node;
                            }
                        }
                    }
                }
            },
        }
    }

    /// Given an inner data which is a VecDeque and pop out
    /// the Data type from front
    #[inline]
    fn pop_front_inner_data(data: InnerData<K, V>) -> Option<Data<K, V>> {
        data.as_ref()
            .and_then(|d| unsafe { (*d.as_ptr()).pop_front() })
    }

    /// Given an inner data which is a VecDeque and pop out
    /// the Data type from back
    #[inline]
    fn pop_back_inner_data(data: InnerData<K, V>) -> Option<Data<K, V>> {
        data.as_ref()
            .and_then(|d| unsafe { (*d.as_ptr()).pop_back() })
    }

    /// Given an Children VecDeque pop out the child node from front
    #[inline]
    fn pop_front_child(children: Children<K, V>) -> OpNode<K, V> {
        children
            .as_ref()
            .and_then(|c| unsafe { (*c.as_ptr()).pop_front().unwrap_or(None) })
    }

    /// Given an Children VecDeque pop out the child node from back
    #[inline]
    fn pop_back_child(children: Children<K, V>) -> OpNode<K, V> {
        children
            .as_ref()
            .and_then(|c| unsafe { (*c.as_ptr()).pop_back().unwrap_or(None) })
    }

    /// Define m as the number of children size(can't be empty)
    /// the maximum key in a node is m - 1
    /// when the number of keys equals to the maximum
    /// split the node at index of [m / 2] - 1
    /// Note [] here is ceil operation
    fn split_node(node: OpNode<K, V>, split_idx: usize) -> OpNode<K, V> {
        let data = Node::get_inner_data(node);
        let data_size = Node::get_data_size(node);
        let children_size = Node::get_children_size(node);
        let children = Node::get_children(node);
        let mut l_n: usize = 0;
        let mut r_n = data_size - split_idx - 1;
        let mut left_node: OpNode<K, V> = None;
        let mut right_node: OpNode<K, V> = None;
        while l_n < split_idx {
            let left_data = Node::pop_front_inner_data(data);
            left_node = Node::adding_data(left_node, left_data);
            l_n += 1;
        }

        while r_n > 0 {
            let right_data = Node::pop_back_inner_data(data);
            right_node = Node::adding_data(right_node, right_data);
            r_n -= 1;
        }

        if children_size != 0 {
            // splitting children
            let mut l_n: usize = split_idx + 1;
            let mut r_n: usize = data_size - split_idx;
            while l_n > 0 {
                let child = Node::pop_front_child(children);
                Node::set_parent(child, left_node);
                Node::push_back_child(left_node, child);
                l_n -= 1;
            }

            while r_n > 0 {
                let child = Node::pop_back_child(children);
                Node::set_parent(child, right_node);
                Node::push_front_child(right_node, child);
                r_n -= 1;
            }
        }

        Node::set_parent(left_node, node);
        Node::set_parent(right_node, node);
        Node::push_back_child(node, left_node);
        Node::push_back_child(node, right_node);

        node
    }

    /// After splitting node call this method to merging two nodes
    /// from bottom to top
    /// Note the caller must guarantee lower_node is a child of upper_node
    /// And this is a one shot merging will not merging repeately
    /// so after merging the returned node could violate the B-tree rules
    fn merging_nodes(upper_node: OpNode<K, V>, lower_node: OpNode<K, V>) -> OpNode<K, V> {
        let lower_idx = Node::get_child_position(upper_node, lower_node).unwrap();
        let lower_data = Node::get_inner_data(lower_node);
        let lower_left = Node::get_child_by_index(lower_node, 0);
        let lower_right = Node::get_child_by_index(lower_node, 1);
        Node::remove_child(upper_node, lower_idx);
        Node::insert_data(
            upper_node,
            lower_idx,
            Node::pop_front_inner_data(lower_data),
        );
        Node::insert_child(upper_node, lower_idx, lower_right);
        Node::insert_child(upper_node, lower_idx, lower_left);
        Node::set_parent(lower_left, upper_node);
        Node::set_parent(lower_right, upper_node);
        upper_node
    }

    /// A node could be unbalanced after removing a key
    /// and this method will try to find one of it's sibling
    /// that has more keys
    fn get_rich_siblings(node: OpNode<K, V>, min_key_num: usize) -> OpNode<K, V> {
        let parent = Node::get_parent(node);
        let parent_children_size = Node::get_children_size(parent);
        let p_node_idx = Node::get_child_position(parent, node);
        match p_node_idx {
            None => None,
            Some(idx) => {
                if idx == 0 {
                    let sibling = Node::get_child_by_index(parent, 1);
                    if Node::get_data_size(sibling) > min_key_num {
                        sibling
                    } else {
                        None
                    }
                } else if idx == parent_children_size - 1 {
                    let sibling = Node::get_child_by_index(parent, parent_children_size - 2);
                    if Node::get_data_size(sibling) > min_key_num {
                        sibling
                    } else {
                        None
                    }
                } else {
                    let left_s = Node::get_child_by_index(parent, idx - 1);
                    let right_s = Node::get_child_by_index(parent, idx + 1);
                    if Node::get_data_size(left_s) > min_key_num {
                        left_s
                    } else if Node::get_data_size(right_s) > min_key_num {
                        right_s
                    } else {
                        None
                    }
                }
            }
        }
    }

    /// Find the minimum node for a given input node
    /// note this method returns a node not a Data
    fn get_minimum_node(mut cur_node: OpNode<K, V>) -> OpNode<K, V> {
        loop {
            match cur_node {
                None => {
                    break cur_node;
                }
                node @ Some(_) => {
                    let cur_children = Node::get_children(cur_node);
                    if cur_children.is_none() {
                        break cur_node;
                    }
                    let min_c = Node::get_child_by_index(node, 0);
                    if min_c.is_none() {
                        break cur_node;
                    } else {
                        cur_node = min_c;
                        continue;
                    }
                }
            }
        }
    }

    /// Find the maximum node for a given input node
    /// note this method returns a node not a Data
    fn get_maximum_node(mut cur_node: OpNode<K, V>) -> OpNode<K, V> {
        loop {
            match cur_node {
                None => {
                    break cur_node;
                }
                node @ Some(_) => {
                    let cur_children = Node::get_children(cur_node);
                    if cur_children.is_none() {
                        break cur_node;
                    }
                    let children_size = Node::get_children_size(node);
                    let max_c = Node::get_child_by_index(node, children_size - 1);
                    if max_c.is_none() {
                        break cur_node;
                    } else {
                        cur_node = max_c;
                        continue;
                    }
                }
            }
        }
    }

    /// Use this methods for boxed a node when pop out a empty-node
    /// this method exists because empty Vec still holds some memory
    /// so turn them into a Box to drop the node
    fn into_boxed(node: OpNode<K, V>) -> Option<Box<Node<K, V>>> {
        let data_size = Node::get_data_size(node);
        let children_size = Node::get_children_size(node);
        assert!(
            data_size + children_size == 0,
            "Droping node should be empty"
        );
        let data = node.and_then(|n| unsafe { (*n.as_ptr()).data.take() });
        let children = node.and_then(|n| unsafe { (*n.as_ptr()).children.take() });
        data.map(|d| unsafe {
            // dropping inner data
            let _data = Box::from_raw(d.as_ptr());
        });
        children.map(|c| unsafe {
            let _children = Box::from_raw(c.as_ptr());
        });
        node.map(|n| unsafe { Box::from_raw(n.as_ptr()) })
    }
}

impl<K: Ord, V> BTree<K, V> {
    /// adding key and value into tree
    fn _add(&mut self, k: K, v: V) {
        let mut cur_node = self.root_node;
        loop {
            if cur_node.is_none() {
                self.len += 1;
                self.root_node = Node::adding_data(cur_node, Some(Data { key: k, value: v }));
                break;
            }
            let children_size = Node::get_children_size(cur_node);
            if children_size != 0 {
                cur_node = Node::moving_target(cur_node, &k);
                continue;
            } else {
                self.len += 1;
                let added_node = Node::adding_data(cur_node, Some(Data { key: k, value: v }));
                self._up_merging(added_node);
                break;
            }
        }
    }

    /// Recursively merging cur_node and it's parent if necessary
    fn _up_merging(&mut self, mut cur_node: OpNode<K, V>) {
        loop {
            let data_size = Node::get_data_size(cur_node);
            let parent = Node::get_parent(cur_node);
            let parent_data_size = Node::get_data_size(parent);
            if data_size >= self.max_key_num {
                let splitted_node = Node::split_node(cur_node, self.min_key_num);
                if parent.is_none() {
                    self.root_node = splitted_node;
                    break;
                } else if parent_data_size + 1 < self.max_key_num {
                    Node::merging_nodes(parent, splitted_node);
                    break;
                } else if parent_data_size + 1 >= self.max_key_num {
                    cur_node = Node::merging_nodes(parent, splitted_node);
                    continue;
                }
            } else {
                break;
            }
        }
    }

    /// The input node could be unbalanced which data size is less than
    /// self.min_key_num
    /// And removing a key could make some node unbalanced
    /// 这个方法中不会有借用前驱或者后继的情况，那是在remove的时候才有的
    fn _rebalancing(&mut self, mut cur_node: OpNode<K, V>) {
        loop {
            let parent = Node::get_parent(cur_node);
            let cur_children = Node::get_children(cur_node);
            let parent_data = Node::get_inner_data(parent);
            let cur_c_pos = Node::get_child_position(parent, cur_node);
            let data_size = Node::get_data_size(cur_node);
            if data_size == 0 && parent.is_none() {
                let first_child = Node::pop_front_child(cur_children);
                self.root_node = first_child;
                Node::set_parent(first_child, None);
                break;
            }
            // if cur_node is the only one node in the tree
            // or it is balanced then just returns
            if parent.is_none() || data_size >= self.min_key_num {
                break;
            }
            let rich_sibling = Node::get_rich_siblings(cur_node, self.min_key_num);
            if rich_sibling.is_none() {
                // pull a parent key down and merge it
                match cur_c_pos {
                    None => {
                        break;
                    }
                    Some(cur_c_idx) => {
                        if cur_c_idx == 0 {
                            // cur_node is the first child
                            // of it's parent
                            // （1）父亲node最左边的key弹出，push front到第二个child；
                            // （2）不平衡的cur node，将所有的key，pop back，再push front到sibling
                            // （3）父亲node弹出被清空的child连接
                            // （3）如果 cur_node 有子节点，将子节点push front到sibling
                            //Node::pop_front_child(parent_children);
                            let parent_out_data = Node::pop_front_inner_data(parent_data);
                            let next_sibling = Node::get_child_by_index(parent, 1);
                            Node::push_front_inner_data(next_sibling, parent_out_data);
                            let mut nd = Node::get_data_size(cur_node);
                            let cur_data = Node::get_inner_data(cur_node);
                            while nd > 0 {
                                let c_data = Node::pop_back_inner_data(cur_data);
                                Node::push_front_inner_data(next_sibling, c_data);
                                nd -= 1;
                            }
                            let mut nc = Node::get_children_size(cur_node);
                            let cur_children = Node::get_children(cur_node);
                            while nc > 0 {
                                let cur_c = Node::pop_back_child(cur_children);
                                Node::push_front_child(next_sibling, cur_c);
                                Node::set_parent(cur_c, next_sibling);
                                nc -= 1;
                            }
                            let empty_node = Node::remove_child(parent, cur_c_idx);
                            let _empty_node = Node::into_boxed(empty_node);
                            cur_node = parent;
                            continue;
                        } else {
                            let prev_sibling = Node::get_child_by_index(parent, cur_c_idx - 1);
                            let parent_out_data = Node::remove_data(parent, cur_c_idx - 1);
                            Node::push_back_inner_data(prev_sibling, parent_out_data);
                            let mut nd = Node::get_data_size(cur_node);
                            let cur_data = Node::get_inner_data(cur_node);
                            while nd > 0 {
                                let c_data = Node::pop_front_inner_data(cur_data);
                                Node::push_back_inner_data(prev_sibling, c_data);
                                nd -= 1;
                            }
                            let mut nc = Node::get_children_size(cur_node);
                            let cur_children = Node::get_children(cur_node);
                            while nc > 0 {
                                let cur_c = Node::pop_front_child(cur_children);
                                Node::push_back_child(prev_sibling, cur_c);
                                Node::set_parent(cur_c, prev_sibling);
                                nc -= 1;
                            }
                            let empty_node = Node::remove_child(parent, cur_c_idx);
                            let _empty_node = Node::into_boxed(empty_node);
                            cur_node = parent;
                            continue;
                        }
                    }
                }
            } else {
                // There are some rich siblings
                let sibling_pos = Node::get_child_position(parent, rich_sibling);
                match (cur_c_pos, sibling_pos) {
                    (Some(cur_idx), Some(sibling_idx)) => {
                        if cur_idx < sibling_idx {
                            let parent_out_data = Node::remove_data(parent, cur_idx);
                            Node::push_back_inner_data(cur_node, parent_out_data);
                            let sibling_out_data = Node::remove_data(rich_sibling, 0);
                            Node::insert_data(parent, cur_idx, sibling_out_data);
                            if Node::get_children_size(rich_sibling) != 0 {
                                let sibling_out_child = Node::remove_child(rich_sibling, 0);
                                Node::push_back_child(cur_node, sibling_out_child);
                                Node::set_parent(sibling_out_child, cur_node);
                            }
                            break;
                        } else {
                            let parent_out_data = Node::remove_data(parent, cur_idx - 1);
                            Node::push_front_inner_data(cur_node, parent_out_data);
                            let sibling_out_data = Node::remove_data(
                                rich_sibling,
                                Node::get_data_size(rich_sibling) - 1,
                            );
                            Node::insert_data(parent, cur_idx - 1, sibling_out_data);
                            if Node::get_children_size(rich_sibling) != 0 {
                                let sibling_out_child = Node::remove_child(
                                    rich_sibling,
                                    Node::get_data_size(rich_sibling) - 1,
                                );
                                Node::push_front_child(cur_node, sibling_out_child);
                                Node::set_parent(sibling_out_child, cur_node);
                            }
                            break;
                        }
                    }
                    _ => {
                        break;
                    }
                }
            }
        }
    }

    // pop out the maximum Data out of the tree
    fn _pop_max_data(&mut self) -> Option<Data<K, V>> {
        let cur_node = self.root_node;
        if cur_node.is_none() {
            None
        } else {
            if self.len == 1 {
                self.len = 0;
                self.root_node = None;
                let max_node = Node::get_maximum_node(cur_node);
                let max_data = Node::get_inner_data(max_node);
                Node::pop_back_inner_data(max_data)
            } else {
                let max_node = Node::get_maximum_node(cur_node);
                let max_data = Node::get_inner_data(max_node);
                let out = Node::pop_back_inner_data(max_data);
                self.len -= 1;
                // After poping the min_node could be unbalanced
                self._rebalancing(max_node);
                out
            }
        }
    }

    // pop out the minimum Data out of the tree
    fn _pop_min_data(&mut self) -> Option<Data<K, V>> {
        let cur_node = self.root_node;
        if cur_node.is_none() {
            None
        } else {
            let min_node = Node::get_minimum_node(cur_node);
            let min_data = Node::get_inner_data(min_node);
            if self.len == 1 {
                self.len = 0;
                self.root_node = None;
                Node::pop_front_inner_data(min_data)
            } else {
                let out = Node::pop_front_inner_data(min_data);
                self.len -= 1;
                // After poping the min_node could be unbalanced
                self._rebalancing(min_node);
                out
            }
        }
    }

    /// Give a ref of key return value
    #[inline]
    fn _get(&self, k: &K) -> Option<&V> {
        let node = Node::moving_target(self.root_node, k);
        let inner_data = Node::get_inner_data(node);
        match inner_data {
            None => None,
            Some(ref data) => unsafe {
                while let Some(d) = (*data.as_ptr()).iter().next() {
                    if d.key.eq(k) {
                        return Some(&d.value);
                    }
                }
                None
            },
        }
    }

    /// removing by key
    fn _remove(&mut self, k: &K) -> Option<V> {
        let node = Node::moving_target(self.root_node, k);
        let parent = Node::get_parent(node);
        let target_idx = Node::get_key_index(node, k);
        match target_idx {
            None => None,
            Some(idx) => {
                if self.len == 1 {
                    self.len = 0;
                    self.root_node = None;
                    Node::get_inner_data(node)
                        .and_then(|d| Node::pop_front_inner_data(Some(d)))
                        .map(|o| o.value)
                } else {
                    self.len -= 1;
                    let left_child = Node::get_child_by_index(node, idx);
                    let right_child = Node::get_child_by_index(node, idx + 1);
                    let no_children = left_child == None;

                    if !no_children {
                        let left_max = Node::get_maximum_node(left_child);
                        let right_min = Node::get_minimum_node(right_child);
                        let left_max_is_rich = Node::get_data_size(left_max) > self.min_key_num;
                        let right_min_is_rich = Node::get_data_size(right_min) > self.min_key_num;
                        let removed_out = Node::remove_data(node, idx);
                        if left_max_is_rich {
                            let replace_data =
                                Node::remove_data(left_max, Node::get_data_size(left_max) - 1);
                            Node::insert_data(node, idx, replace_data);
                            removed_out.map(|n| n.value)
                        } else if right_min_is_rich {
                            let replace_data = Node::remove_data(right_min, 0);
                            Node::insert_data(node, idx, replace_data);
                            removed_out.map(|n| n.value)
                        } else {
                            let replace_data =
                                Node::remove_data(left_max, Node::get_data_size(left_max) - 1);
                            Node::insert_data(node, idx, replace_data);
                            self._rebalancing(left_max);
                            removed_out.map(|n| n.value)
                        }
                    } else {
                        let removed_out = Node::remove_data(node, idx);
                        if parent.is_some() {
                            if Node::get_data_size(node) < self.min_key_num {
                                self._rebalancing(node);
                                removed_out.map(|d| d.value)
                            } else {
                                removed_out.map(|d| d.value)
                            }
                        } else {
                            removed_out.map(|d| d.value)
                        }
                    }
                }
            }
        }
    }
}

pub struct IntoIter<K: Ord, V>(BTree<K, V>);

impl<K: Ord, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        self.0._pop_min_data().map(|d| (d.key, d.value))
    }
}

impl<K: Ord, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0._pop_max_data().map(|d| (d.key, d.value))
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

impl<K: Ord, V> Drop for BTree<K, V> {
    fn drop(&mut self) {
        struct DropGuard<'a, K: Ord, V>(&'a mut BTree<K, V>);

        impl<'a, K: Ord, V> Drop for DropGuard<'a, K, V> {
            fn drop(&mut self) {
                while self.0._pop_min_data().is_some() {}
            }
        }

        while let Some(d) = self._pop_min_data() {
            let guard = DropGuard(self);
            drop(d);
            mem::forget(guard);
        }
    }
}

struct NextNodes<K: Ord, V> {
    node: OpNode<K, V>,
    index: Cell<usize>,
}

pub struct Iter<'a, K: Ord, V> {
    next_nodes: Vec<NextNodes<K, V>>,
    seen: HashSet<OpNode<K, V>>,
    next_back_nodes: Vec<NextNodes<K, V>>,
    seen_back: HashSet<OpNode<K, V>>,
    _marker: PhantomData<&'a Node<K, V>>,
}

impl<'a, K: Ord, V> Iter<'a, K, V> {
    fn next_ascending(&mut self) -> Option<(&'a K, &'a V)> {
        loop {
            let head_node = self.next_nodes.pop();
            let cur_node = if head_node.is_some() {
                let h = head_node.unwrap();
                if self.seen.contains(&h.node) {
                    None
                } else {
                    Some(h)
                }
            } else {
                None
            };

            match cur_node {
                None => {
                    if self.next_nodes.len() == 0 {
                        break None;
                    } else {
                        continue;
                    }
                }
                Some(node_wrapper) => unsafe {
                    let cur_idx = node_wrapper.index.get();
                    let node = node_wrapper.node;
                    let data = Node::get_inner_data(node);
                    let is_the_last_data = Node::get_data_size(node) - 1 == cur_idx;
                    let left = Node::get_child_by_index(node, cur_idx);
                    let right = Node::get_child_by_index(node, cur_idx + 1);
                    let left_child = if self.seen.contains(&left) || left.is_none() {
                        None
                    } else {
                        left
                    };
                    let right_child = if self.seen.contains(&right) || right.is_none() {
                        None
                    } else {
                        right
                    };

                    if left_child.is_none() && right_child.is_none() && !is_the_last_data {
                        node_wrapper.index.set(cur_idx + 1);
                        self.next_nodes.push(node_wrapper);
                        break data
                            .as_ref()
                            .map(|d| &(*d.as_ptr())[cur_idx])
                            .map(|d| (&d.key, &d.value));
                    } else if left_child.is_none() && right_child.is_none() && is_the_last_data {
                        self.seen.insert(node);
                        break data
                            .as_ref()
                            .map(|d| &(*d.as_ptr())[cur_idx])
                            .map(|d| (&d.key, &d.value));
                    } else if left_child.is_some() && right_child.is_none() && !is_the_last_data {
                        self.next_nodes.push(node_wrapper);
                        self.next_nodes.push(NextNodes {
                            node: left_child,
                            index: Cell::new(0),
                        });
                        continue;
                    } else if left_child.is_some() && right_child.is_none() && is_the_last_data {
                        self.next_nodes.push(node_wrapper);
                        self.next_nodes.push(NextNodes {
                            node: left_child,
                            index: Cell::new(0),
                        });
                        continue;
                    } else if left_child.is_some() && right.is_some() && !is_the_last_data {
                        self.next_nodes.push(node_wrapper);
                        self.next_nodes.push(NextNodes {
                            node: left_child,
                            index: Cell::new(0),
                        });
                        continue;
                    } else if left_child.is_some() && right_child.is_some() && is_the_last_data {
                        self.next_nodes.push(node_wrapper);
                        self.next_nodes.push(NextNodes {
                            node: left_child,
                            index: Cell::new(0),
                        });
                        continue;
                    } else if left_child.is_none() && right_child.is_some() && !is_the_last_data {
                        let out = data
                            .as_ref()
                            .map(|d| &(*d.as_ptr())[cur_idx])
                            .map(|d| (&d.key, &d.value));
                        node_wrapper.index.set(cur_idx + 1);
                        self.next_nodes.push(node_wrapper);
                        break out;
                    } else {
                        // left_child.is_none && right_child.is_some() && is_the_last_data
                        self.seen.insert(node);
                        let out = data
                            .as_ref()
                            .map(|d| &(*d.as_ptr())[cur_idx])
                            .map(|d| (&d.key, &d.value));
                        node_wrapper.index.set(cur_idx);
                        self.next_nodes.push(NextNodes {
                            node: right_child,
                            index: Cell::new(0),
                        });
                        break out;
                    }
                },
            }
        }
    }

    fn next_descending(&mut self) -> Option<(&'a K, &'a V)> {
        loop {
            let last_node = self.next_back_nodes.pop();
            let cur_node = if last_node.is_some() {
                let l = last_node.unwrap();
                if self.seen_back.contains(&l.node) {
                    None
                } else {
                    Some(l)
                }
            } else {
                None
            };

            match cur_node {
                None => {
                    if self.next_back_nodes.len() == 0 {
                        break None;
                    } else {
                        continue;
                    }
                }
                Some(node_wrapper) => unsafe {
                    let cur_idx = node_wrapper.index.get();
                    let node = node_wrapper.node;
                    let data = Node::get_inner_data(node);
                    let is_the_first_data = cur_idx == 0;
                    let left = Node::get_child_by_index(node, cur_idx);
                    let right = Node::get_child_by_index(node, cur_idx + 1);
                    let left_child = if self.seen_back.contains(&left) || left.is_none() {
                        None
                    } else {
                        left
                    };
                    let right_child = if self.seen_back.contains(&right) || right.is_none() {
                        None
                    } else {
                        right
                    };

                    if left_child.is_none() && right_child.is_none() && !is_the_first_data {
                        node_wrapper.index.set(cur_idx - 1);
                        self.next_back_nodes.push(node_wrapper);
                        break data
                            .as_ref()
                            .map(|d| &(*d.as_ptr())[cur_idx])
                            .map(|d| (&d.key, &d.value));
                    } else if left_child.is_none() && right_child.is_none() && is_the_first_data {
                        self.seen_back.insert(node);
                        break data
                            .as_ref()
                            .map(|d| &(*d.as_ptr())[cur_idx])
                            .map(|d| (&d.key, &d.value));
                    } else if left_child.is_some() && right_child.is_none() && !is_the_first_data {
                        let out = data
                            .as_ref()
                            .map(|d| &(*d.as_ptr())[cur_idx])
                            .map(|d| (&d.key, &d.value));
                        node_wrapper.index.set(cur_idx - 1);
                        self.next_back_nodes.push(node_wrapper);
                        break out;
                    } else if left_child.is_some() && right_child.is_none() && is_the_first_data {
                        self.seen_back.insert(node);
                        let out = data
                            .as_ref()
                            .map(|d| &(*d.as_ptr())[cur_idx])
                            .map(|d| (&d.key, &d.value));
                        node_wrapper.index.set(cur_idx);
                        self.next_back_nodes.push(NextNodes {
                            node: left_child,
                            index: Cell::new(Node::get_data_size(left_child) - 1),
                        });
                        break out;
                    } else if left_child.is_some() && right_child.is_some() && !is_the_first_data {
                        self.next_back_nodes.push(node_wrapper);
                        self.next_back_nodes.push(NextNodes {
                            node: right_child,
                            index: Cell::new(Node::get_data_size(right_child) - 1),
                        });
                        continue;
                    } else if left_child.is_some() && right_child.is_some() && is_the_first_data {
                        self.next_back_nodes.push(node_wrapper);
                        self.next_back_nodes.push(NextNodes {
                            node: right_child,
                            index: Cell::new(Node::get_data_size(right_child) - 1),
                        });
                        continue;
                    } else if left_child.is_none() && right_child.is_some() && !is_the_first_data {
                        self.next_back_nodes.push(node_wrapper);
                        self.next_back_nodes.push(NextNodes {
                            node: right_child,
                            index: Cell::new(Node::get_data_size(right_child) - 1),
                        });
                        continue;
                    } else {
                        // left_child.is_none && right_child.is_some() && is_first_data
                        self.next_back_nodes.push(node_wrapper);
                        self.next_back_nodes.push(NextNodes {
                            node: right_child,
                            index: Cell::new(Node::get_data_size(right_child) - 1),
                        });
                        continue;
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
    }
}

impl<'a, K: Ord, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next_descending()
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for BTree<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let inputs: Vec<_> = iter.into_iter().collect();
        if inputs.is_empty() {
            return BTree::<K, V>::new(5);
        }
        let mut out = BTree::<K, V>::new(5);
        for (k, v) in inputs {
            out.insert(k, v);
        }
        out
    }
}

impl<K: Ord, V> IntoIterator for BTree<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<K: Ord + Copy, V: Copy> Clone for BTree<K, V> {
    fn clone(&self) -> Self {
        let mut out = BTree::<K, V>::new(self.max_key_num);
        for (k, v) in self.iter() {
            out.insert(*k, *v)
        }
        out
    }
}

unsafe impl<K: Ord + Send, V: Send> Send for BTree<K, V> {}

unsafe impl<K: Ord + Sync, V: Sync> Sync for BTree<K, V> {}

unsafe impl<K: Ord + Send, V: Send> Send for Iter<'_, K, V> {}

unsafe impl<K: Ord + Sync, V: Sync> Sync for Iter<'_, K, V> {}

impl<K: Ord, V> BTree<K, V> {
    /// Create a B-tree with some order.
    /// and the order is maximum number of keys that
    /// a node in the B-tree can hold
    /// and the minimum number of order is 3 which will make this a 2-3 tree
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let b: BTree<i32, i32> = BTree::new(4);
    /// ```
    pub fn new(order: usize) -> Self {
        assert!(order >= 3, "Degree should be greater or equal to 3");
        let max_c = order + 1;
        let min = max_c as f64 / 2.0_f64;
        BTree {
            root_node: None,
            len: 0,
            max_key_num: order,
            min_key_num: min.ceil() as usize - 1,
            _marker: PhantomData,
        }
    }

    /// Adding a pair of key and value into the tree
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let mut b: BTree<i32, i32> = BTree::new(4);
    /// b.insert(1, 1);
    /// ```   
    pub fn insert(&mut self, k: K, v: V) {
        self._add(k, v)
    }

    /// Poping out the minimum key-value pair in the tree
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let mut b: BTree<i32, i32> = BTree::new(4);
    /// let data = [(1, 1), (2, 2), (3, 3)];
    /// for (k, v) in data {
    ///     b.insert(k, v)
    /// }
    /// assert_eq!(b.pop_min(), Some((1, 1)))
    /// ```   
    pub fn pop_min(&mut self) -> Option<(K, V)> {
        self._pop_min_data().map(|n| (n.key, n.value))
    }

    /// Poping out the maximum key-value pair in the tree
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let mut b: BTree<i32, i32> = BTree::new(4);
    /// let data = [(1, 1), (2, 2), (3, 3)];
    /// for (k, v) in data {
    ///     b.insert(k, v)
    /// }
    /// assert_eq!(b.pop_max(), Some((3, 3)));
    /// ```   
    pub fn pop_max(&mut self) -> Option<(K, V)> {
        self._pop_max_data().map(|n| (n.key, n.value))
    }

    /// Give a reference of key try to return
    /// the reference    
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let mut b: BTree<i32, i32> = BTree::new(4);
    /// let data = [(1, 1), (2, 2), (3, 3)];
    /// for (k, v) in data {
    ///     b.insert(k, v)
    /// }
    /// assert_eq!(b.get(&2), Some(&2));
    /// ```   
    pub fn get(&self, k: &K) -> Option<&V> {
        let mut outs: Vec<_> = self.iter().filter(|n| n.0.eq(k)).collect();
        if outs.len() == 0 {
            None
        } else {
            outs.pop().map(|o| o.1)
        }
    }

    /// Updating the key with a new value
    /// and if the key is not exists it will
    /// adding the key-value pair into the tree
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let mut b: BTree<i32, i32> = BTree::new(3);
    /// let data = [(1, 1), (2, 2), (3, 3)];
    /// for (k, v) in data {
    ///     b.insert(k, v)
    /// }
    /// //b.set(2, 200);
    /// ```   
    pub fn set(&mut self, k: K, v: V) {
        self.insert(k, v)
    }

    /// Check if Btree contains some key
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let mut b: BTree<i32, i32> = BTree::new(4);
    /// let data = [(1, 1), (2, 2), (3, 3)];
    /// for (k, v) in data {
    ///     b.insert(k, v)
    /// }
    /// assert!(b.contains(&2));
    /// ```   
    pub fn contains(&self, k: &K) -> bool {
        self.iter().any(|n| n.0.eq(k))
    }

    /// Removing by key
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let mut b: BTree<i32, i32> = BTree::new(4);
    /// let data = [(1, 1), (2, 2), (3, 3)];
    /// for (k, v) in data {
    ///     b.insert(k, v)
    /// }
    /// assert_eq!(b.remove(&2), Some(2));
    /// ```   
    pub fn remove(&mut self, k: &K) -> Option<V> {
        self._remove(k)
    }

    /// Making an iter of Btree
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let mut b: BTree<i32, i32> = BTree::new(4);
    /// let data = [(1, 1), (2, 2), (3, 3)];
    /// for (k, v) in data {
    ///     b.insert(k, v)
    /// }
    /// let mut iter = b.iter();
    /// assert_eq!(iter.next(), Some((&1, &1)));
    /// assert_eq!(iter.next_back(), Some((&3, &3)));
    /// ```      
    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        let seen = HashSet::new();
        let seen_back = HashSet::new();
        Iter {
            next_nodes: vec![NextNodes {
                node: self.root_node,
                index: Cell::new(0),
            }],
            seen: seen,
            next_back_nodes: vec![NextNodes {
                node: self.root_node,
                index: Cell::new(Node::get_data_size(self.root_node) - 1),
            }],
            seen_back: seen_back,
            _marker: PhantomData,
        }
    }

    /// Get the length
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let mut b: BTree<i32, i32> = BTree::new(4);
    /// let data = [(1, 1), (2, 2), (3, 3)];
    /// for (k, v) in data {
    ///     b.insert(k, v)
    /// }
    /// assert_eq!(b.len(), 3);
    /// ```      
    pub fn len(&self) -> usize {
        self.len
    }

    /// To tell if this tree is empty
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let mut b: BTree<i32, i32> = BTree::new(4);
    /// let data = [(1, 1), (2, 2), (3, 3)];
    /// for (k, v) in data {
    ///     b.insert(k, v)
    /// }
    /// assert!(!b.is_empty());
    /// ```      
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clearing the tree
    ///
    /// # Example
    ///
    /// ```
    /// use ABtree::BTree;
    /// let mut b: BTree<i32, i32> = BTree::new(4);
    /// let data = [(1, 1), (2, 2), (3, 3)];
    /// for (k, v) in data {
    ///     b.insert(k, v)
    /// }
    /// b.clear();
    /// assert_eq!(b.len(), 0);
    /// ```      
    pub fn clear(&mut self) {
        *self = Self::new(self.max_key_num);
    }
}
