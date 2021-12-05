This crate named as ABtree but this not means it is a novel data sturcture. It’s just AVL tree and Btree.
For the Btree, what makes it different from that of BtreeMap in std is this Btree can accept any number as the maximum number of inner node, as long as the number grater or equal to 3

# AVL
```
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);

let mut iter = t.iter();
assert_eq!(iter.next(), Some((&0, &0)));
assert_eq!(iter.next(), Some((&1, &1)));
assert_eq!(iter.next_back(), Some((&2, &2)));
```

# Btree
```
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert_eq!(b.pop_min(), Some((1, 1)))
```