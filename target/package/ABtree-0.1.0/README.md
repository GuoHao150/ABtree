This crate named as ABtree but this not means it is a novel data sturcture. It’s just AVL tree and Btree.
For the Btree, what makes it different from that of BtreeMap in std is this Btree can accept any number as the maximum number of inner node, as long as the number grater or equal to 3

# Installation

```
[dependencies]
ABtree = "0.1.2"
```

# 1.AVL
### 1.1 create an empty AVL tree
```
use ABtree::AVL;

let t = AVL::<i32, i32>::new();
```

### 1.2 insert key-value pair
```
use ABtree::AVL;
let mut t = AVL::<i32, i32>::new();
t.insert(2, 3);
assert_eq!(t.len(), 1);
```

### 1.3 update value
If the key not exists it will add the key-value pair into the tree
```
use ABtree::AVL;
let mut t = AVL::<i32, i32>::new();
t.set(2, 2);
t.set(2, 31);
assert_eq!(t.get(&2), Some(&31));
```

### 1.4 get length
```
use ABtree::AVL;
let mut t = AVL::<i32, i32>::new();
t.insert(2, 2);
t.insert(3, 3);
assert_eq!(t.len(), 2);
```

### 1.5 make an iter for AVL
Note the next() and next_back() are two independent operations
which means a node can be traversed by both methods
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

### 1.6 contains
```
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
assert!(t.contains(&1));
```

### 1.7 remove
```
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
assert_eq!(t.remove(&1), Some(1));
assert_eq!(t.len(), 2);
```

### 1.8 peeking the root node
```
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
assert_eq!(t.peek_root(), Some((&1, &1)));
```

### 1.9 is empty?
```
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
assert_eq!(t.is_empty(), false);
```

### 1.10 clearing the instance of AVL tree
```
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
t.clear();
assert_eq!(t.len(), 0);
```

### 1.11 get method
```
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
assert_eq!(t.get(&1), Some(&1));
```

### 1.12 from_iter
```
use std::iter::FromIterator;
use ABtree::AVL;

let data = vec![
    (12, 1),
    (8, 1),
    (17, 1),
];
let a = AVL::from_iter(data);
```

### 1.13 into_iter
```
use std::iter::FromIterator;
use ABtree::AVL;

let data = vec![
    (12, 1),
    (8, 1),
    (17, 1),
];
let a = AVL::from_iter(data);
let iter = a.into_iter();
```

# 2.Btree
### 2.1 create an empty b-tree
choose any number as the maximum number for the inner node as long as this number greater or equal to 3
```
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
b.insert(1, 1);
```

### 2.2 insert
```
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
b.insert(1, 1);
```

### 2.3 get
```
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert_eq!(b.get(&2), Some(&2));
```

### 2.3 set
```
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(3);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
b.set(2, 200);
```

### 2.4 contains
```
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert!(b.contains(&2));
```

### 2.5 remove
```
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert_eq!(b.remove(&2), Some(2));
```

### 2.6 iter
```
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
let mut iter = b.iter();
assert_eq!(iter.next(), Some((&1, &1)));
assert_eq!(iter.next_back(), Some((&3, &3)));
```

### 2.7 get length
```
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert_eq!(b.len(), 3);
```

### 2.8 is empty?
```
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert!(!b.is_empty());
```

### 2.9
```
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
b.clear();
assert_eq!(b.len(), 0);
```

### 2.10 from_iter
If use from_iter() to create b-tree then the maximum number of a inner node size is 3
which makes it a 2-3 tree
```
use std::iter::FromIterator;
use ABtree::BTree;
let data1 = vec![
    (12, 1),
    (8, 1),
    (17, 1),
];
let b = BTree::from_iter(data1);
b.iter().for_each(|n| println!("{}", n.0));
```

### 2.11 into_iter
```
use std::iter::FromIterator;
use ABtree::BTree;
let data1 = vec![
    (12, 1),
    (8, 1),
    (17, 1),
];
let b = BTree::from_iter(data1);
b.into_iter().for_each(|n| println!("{}", n.0));
```

