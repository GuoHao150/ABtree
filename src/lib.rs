//! This crate named as ABtree but this not means it is a novel
//! data sturcture. It's just AVL tree and Btree
//! For the Btree module, what makes it different from that of BtreeMap in std
//! is this Btree can accept any number as the maximum number of inner node, as long
//! as the number grater or equal to 3

mod A;
mod B;

pub use A::AVL::AVL;
pub use B::Btree::BTree;

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use crate::A::AVL::AVL;
    use crate::B::Btree::BTree;
    #[test]
    fn avl_len() {
        let data = vec![
            (8, 8),
            (12, 12),
            (18, 18),
            (19, 19),
            (20, 20),
            (21, 21),
            (22, 22),
            (23, 23),
        ];
        let dl = data.len();
        let avl = AVL::from_iter(data);
        assert_eq!(avl.len(), dl);
    }

    #[test]
    fn btree_len() {
        let data = vec![
            (8, 8),
            (12, 12),
            (18, 18),
            (19, 19),
            (20, 20),
            (21, 21),
            (22, 22),
            (23, 23),
        ];
        let dl = data.len();
        let btr = BTree::from_iter(data);
        assert_eq!(btr.len(), dl);
        assert!(btr.contains(&8));
        assert!(!btr.contains(&80));
    }

    #[test]
    fn btree_set() {
        let data = vec![(8, 8), (12, 12)];
        let mut btr = BTree::from_iter(data);
        btr.set(8, 91);
        btr.set(10, 100);
        assert_eq!(btr.len(), 3);
        let v: Vec<_> = btr.into_iter().collect();
        assert_eq!(v, vec![(8, 91), (10, 100), (12, 12)]);

        let mut btr = BTree::new(3);
        for (k, v) in [(1, 1), (2, 2), (3, 3)] {
            btr.insert(k, v);
        }
        assert_eq!(btr.get(&2), Some(&2));
        assert_eq!(btr.len(), 3);
        btr.insert(2, 20);
        assert_eq!(btr.get(&2), Some(&20));
        assert_eq!(btr.len(), 3);
    }

    #[test]
    fn btree_rm() {
        let data = vec![
            (8, 8),
            (12, 12),
            (18, 18),
            (19, 19),
            (20, 20),
            (21, 21),
            (22, 22),
            (23, 23),
        ];
        let mut btr = BTree::new(5);
        for (k, v) in data {
            btr.insert(k, v);
        }
        assert_eq!(btr.remove(&19), Some(19));
        assert_eq!(btr.remove(&20), Some(20));
        assert_eq!(btr.remove(&30), None);

        let v: Vec<_> = btr.into_iter().collect();
        assert_eq!(
            v,
            vec![(8, 8), (12, 12), (18, 18), (21, 21), (22, 22), (23, 23)]
        );
    }

    #[test]
    fn btree_rm2() {
        let data = vec![
            (8, 8),
            (12, 12),
            (18, 18),
            (19, 19),
            (20, 20),
            (21, 21),
            (22, 22),
            (23, 23),
        ];
        let mut btr = BTree::new(3);
        for (k, v) in data {
            btr.insert(k, v);
        }
        assert_eq!(btr.remove(&19), Some(19));
        assert_eq!(btr.remove(&20), Some(20));
        assert_eq!(btr.remove(&30), None);

        let v: Vec<_> = btr.into_iter().collect();
        assert_eq!(
            v,
            vec![(8, 8), (12, 12), (18, 18), (21, 21), (22, 22), (23, 23)]
        );
    }

    #[test]
    fn btree_rm3() {
        let data = vec![
            (Box::new(8), 8),
            (Box::new(12), 12),
            (Box::new(18), 18),
            (Box::new(19), 19),
            (Box::new(20), 20),
            (Box::new(21), 21),
            (Box::new(22), 22),
            (Box::new(23), 23),
        ];
        let data2 = vec![
            (Box::new(8), 8),
            (Box::new(12), 12),
            (Box::new(18), 18),
            (Box::new(21), 21),
            (Box::new(22), 22),
            (Box::new(23), 23),
        ];

        let mut btr = BTree::new(3);
        for (k, v) in data {
            btr.insert(k, v);
        }
        assert_eq!(btr.remove(&Box::new(19)), Some(19));
        assert_eq!(btr.remove(&Box::new(20)), Some(20));
        assert_eq!(btr.remove(&Box::new(30)), None);

        let v: Vec<_> = btr.into_iter().collect();
        assert_eq!(v, data2);
    }

    #[test]
    fn btree_empty_iter() {
        let b: BTree<u32, u32> = BTree::new(3);
        assert_eq!(b.iter().next(), None);
        assert_eq!(b.iter().next_back(), None);
    }

    #[test]
    fn avl_empty_iter() {
        let a: AVL<i32, i32> = AVL::new();
        assert_eq!(a.iter().next(), None);
        assert_eq!(a.iter().next_back(), None);
    }

    #[test]
    fn btree_empty_into_iter() {
        let b: BTree<u32, u32> = BTree::new(3);
        let mut iter = b.into_iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn avl_empty_into_iter() {
        let a: AVL<i32, i32> = AVL::new();
        let mut iter = a.into_iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }
}
