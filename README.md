This crate named as ABtree but this not means it is a novel data sturcture. It’s just AVL tree and Btree.
For the Btree, what makes it different from that of BtreeMap in std is this Btree can accept any number as the maximum number of inner node, as long as the number greater or equal to 3


# 1.AVL
### 1.1 create an empty AVL tree
```rust
use ABtree::AVL;

let t = AVL::<i32, i32>::new();
```

### 1.2 insert key-value pair
```rust
use ABtree::AVL;
let mut t = AVL::<i32, i32>::new();
t.insert(2, 3);
assert_eq!(t.len(), 1);
```

### 1.3 update value
If the key not exists it will add the key-value pair into the tree
```rust
use ABtree::AVL;
let mut t = AVL::<i32, i32>::new();
t.set(2, 2);
t.set(2, 31);
assert_eq!(t.get(&2), Some(&31));
```

### 1.4 get length
```rust
use ABtree::AVL;
let mut t = AVL::<i32, i32>::new();
t.insert(2, 2);
t.insert(3, 3);
assert_eq!(t.len(), 2);
```

### 1.5 make an iter for AVL
Note the next() and next_back() are two independent operations
which means a node can be traversed by both methods
```rust
use ABtree::AVL;

let t = AVL::<i32, i32>::new();
```


### 1.6 contains
```rust
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
assert!(t.contains(&1));
```

### 1.7 remove
```rust
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
assert_eq!(t.remove(&1), Some(1));
assert_eq!(t.len(), 2);
```

### 1.8 peeking the root node
```rust
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
assert_eq!(t.peek_root(), Some((&1, &1)));
```

### 1.9 is empty?
```rust
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
assert_eq!(t.is_empty(), false);
```

### 1.10 clearing the instance of AVL tree
```rust
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
t.clear();
assert_eq!(t.len(), 0);
```

### 1.11 get method
```rust
use ABtree::AVL;

let mut t: AVL<u32, u32> = AVL::new();

t.insert(0, 0);
t.insert(1, 1);
t.insert(2, 2);
assert_eq!(t.get(&1), Some(&1));
```

### 1.12 from_iter
```rust
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
```rust
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
```rust
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
b.insert(1, 1);
```

### 2.2 insert
```rust
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
b.insert(1, 1);
```

### 2.3 get
```rust
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert_eq!(b.get(&2), Some(&2));
```

### 2.4 set
```rust
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(3);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
b.set(2, 200);
```

### 2.5 contains
```rust
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert!(b.contains(&2));
```

### 2.6 remove
```rust
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert_eq!(b.remove(&2), Some(2));
```

### 2.7 iter
```rust
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert_eq!(b.remove(&2), Some(2));
```



### 2.8 get length
```rust
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert_eq!(b.len(), 3);
```

### 2.9 is empty?
```rust
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
assert!(!b.is_empty());
```

### 2.10 clear
```rust
use ABtree::BTree;
let mut b: BTree<i32, i32> = BTree::new(4);
let data = [(1, 1), (2, 2), (3, 3)];
for (k, v) in data {
    b.insert(k, v)
}
b.clear();
assert_eq!(b.len(), 0);
```

### 2.11 from_iter
If use from_iter() to create b-tree then the maximum number of a inner node size is 3
which makes it a 2-3 tree
```rust
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

### 2.12 into_iter
```rust
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
