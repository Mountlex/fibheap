#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub struct FibHeap<I, K> {
    min: Option<usize>,
    trees: Vec<TreeNode<I, K>>,
    n: usize,
}

impl<I, K> FibHeap<I, K>
where
    I: PartialEq + Eq + Clone,
    K: Ord + Clone,
{
    pub fn new() -> Self {
        FibHeap {
            min: None,
            trees: vec![],
            n: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn empty(&self) -> bool {
        self.n == 0
    }

    pub fn peek_min(&self) -> Option<&I> {
        self.min
            .map(|min_idx| &self.trees[min_idx])
            .map(|root| &root.item)
    }

    pub fn min_key(&self) -> Option<&K> {
        self.min
            .map(|min_idx| &self.trees[min_idx])
            .map(|root| &root.key)
    }

    pub fn insert(&mut self, item: I, key: K) {
        let key_ref = &key;
        self.trees.push(TreeNode::new(item, key.clone()));
        self.n += 1;
        if let Some(min_key) = self.min_key() {
            if key_ref < min_key {
                self.min = Some(self.trees.len() - 1);
            }
        } else {
            debug_assert!(self.trees.len() == 1);
            self.min = Some(0)
        }
    }

    pub fn decrease_key(&mut self, item: &I, new_key: K) {
        if !self.empty() {
            let current_min_key_copy = self.min_key().unwrap().clone();
            for (i, root) in self.trees.iter_mut().enumerate() {
                debug_assert!(!root.mark);
                match root.decrease_key(item, &new_key) {
                    DecreaseKeyResult::Unmarked(mut cutoff) => {
                        root.mark = false; // Unmark root if marked                  
                        if new_key < current_min_key_copy {
                            // tree node corresponding to decreased key is first in cutoff
                            self.min = Some(self.trees.len());
                        }
                        if let Some(decreased_child) = cutoff.first_mut() {
                            decreased_child.key = new_key;
                        }
                        for mut node in cutoff {
                            node.mark = false;
                            self.trees.push(node);
                        }
                        break;
                    }
                    DecreaseKeyResult::ItemFound => {
                        root.key = new_key;
                        if root.key < current_min_key_copy {
                            self.min = Some(i);
                        }
                        break;
                    }
                    DecreaseKeyResult::Marked(_) => {},
                    DecreaseKeyResult::NotFound => {}
                    DecreaseKeyResult::NoDecrease => {
                        break;
                    }
                }
            }
    }
    }

    pub fn pop_min(&mut self) -> Option<I> {
        let mut removed_item: Option<I> = None;
        if let Some(min_idx) = &mut self.min {
            self.n -= 1;
            let tree_node = self.trees.swap_remove(*min_idx);
            removed_item = Some(tree_node.item);
            for c in tree_node.children {
                self.trees.push(c);
            }

            let max_rank = (2.0 * (self.n as f64).log(1.6)).ceil() as usize;
            let mut roots: Vec<Option<TreeNode<I, K>>> = vec![None; max_rank + 1];

            for root in self.trees.drain(..) {
                let mut node = root;

                while roots.get(node.rank()).unwrap().is_some() {
                    let other = std::mem::replace(&mut roots[node.rank()], None).unwrap();
                    node = link(node, other);
                }
                let rank = node.rank();
                roots[rank] = Some(node);
            }

            *min_idx = 0;
            for root in roots {
                if let Some(mut root) = root {
                    root.mark = false;
                    self.trees.push(root);
                    if self.trees.last().unwrap().key < self.trees[*min_idx].key {
                        *min_idx = self.trees.len() - 1;
                    }
                }
            }
        }
        if self.n == 0 {
            self.min = None;
        }

        removed_item
    }
}

fn link<I,K>(mut first: TreeNode<I,K>, mut second: TreeNode<I,K>) -> TreeNode<I,K> where K: Ord {
    if first.key < second.key {
        second.mark = false;
        first.children.push(second);
        first
    } else {
        first.mark = false;
        second.children.push(first);
        second
    }
}

enum DecreaseKeyResult<I, K> {
    NotFound,
    NoDecrease,
    ItemFound,
    Marked(Vec<TreeNode<I, K>>),
    Unmarked(Vec<TreeNode<I, K>>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TreeNode<I, K> {
    children: Vec<TreeNode<I, K>>,
    key: K,
    mark: bool,
    item: I,
}

impl<I, K> TreeNode<I, K>
where
    I: Eq + PartialEq,
    K: Ord + Clone
{
    fn new(item: I, key: K) -> Self {
        TreeNode {
            item,
            key,
            mark: false,
            children: vec![],
        }
    }

    fn rank(&self) -> usize {
        self.children.len()
    }

    fn decrease_key(&mut self, item: &I, new_key: &K) -> DecreaseKeyResult<I, K> {
        if &self.item == item {
            if &self.key > new_key {
                DecreaseKeyResult::ItemFound
            } else {
                DecreaseKeyResult::NoDecrease
            }
        } else {
            for (i, c) in self.children.iter_mut().enumerate() {
                match c.decrease_key(item, new_key) {
                    DecreaseKeyResult::ItemFound => {
                        if &self.key > new_key {
                            let cut_child = self.children.swap_remove(i);
                            let cutoff = vec![cut_child];
                            if self.mark {
                                return DecreaseKeyResult::Marked(cutoff);
                            } else {
                                self.mark = true;
                                return DecreaseKeyResult::Unmarked(cutoff);
                            }
                        } else {
                            return DecreaseKeyResult::Unmarked(vec![]);
                        }
                    }
                    DecreaseKeyResult::Marked(mut cutoff) => {
                        cutoff.push(self.children.swap_remove(i));
                        if self.mark {
                            return DecreaseKeyResult::Marked(cutoff);
                        } else {
                            self.mark = true;
                            return DecreaseKeyResult::Unmarked(cutoff);
                        }
                    }
                    DecreaseKeyResult::Unmarked(cutoff) => {
                        return DecreaseKeyResult::Unmarked(cutoff)
                    }
                    DecreaseKeyResult::NoDecrease => {
                        return DecreaseKeyResult::NoDecrease;
                    }
                    DecreaseKeyResult::NotFound => {}
                }
            }

            DecreaseKeyResult::NotFound
        }
    }
}

#[cfg(test)]
mod test_heap {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_heap_empty() {
        let mut heap: FibHeap<(),()> = FibHeap::new();

        assert_eq!(heap.len(), 0);
        assert_eq!(heap.peek_min(), None);
        assert_eq!(heap.min_key(), None);
        assert_eq!(heap.pop_min(), None);
    }

    #[test]
    fn test_heap_insert_construction() {
        let mut heap = FibHeap::new();

        heap.insert(1, 1);
        heap.insert(2, 1);
        heap.insert(3, 2);

        assert_eq!(heap.len(), 3);

        assert_eq!(heap.peek_min(), Some(&1));
        assert_eq!(heap.min_key(), Some(&1));
    }

    #[test]
    fn test_heap_pop_min_single() {
        let mut heap = FibHeap::new();

        heap.insert(1, 1);
        heap.insert(2, 2);
        heap.insert(3, 3);
        heap.insert(4, 4);
        heap.insert(5, 5);
        heap.insert(6, 6);

        assert_eq!(heap.pop_min(), Some(1));
        assert_eq!(heap.peek_min(), Some(&2));
        assert_eq!(heap.len(), 5);
    }

    #[test]
    fn test_heap_pop_and_insert() {
        let mut heap = FibHeap::new();

        heap.insert(1, 1);
        heap.insert(2, 2);
        heap.insert(3, 3);

        assert_eq!(heap.pop_min(), Some(1));
        assert_eq!(heap.pop_min(), Some(2));

        heap.insert(2, 2);
        heap.insert(5, 5);
        heap.insert(6, 6);

        assert_eq!(heap.pop_min(), Some(2));
        assert_eq!(heap.pop_min(), Some(3));

        heap.insert(1, 1);
        heap.insert(2, 2);

        assert_eq!(heap.pop_min(), Some(1));
        assert_eq!(heap.pop_min(), Some(2));

        heap.insert(7, 7);
        heap.insert(8, 8);

        assert_eq!(heap.pop_min(), Some(5));
        assert_eq!(heap.pop_min(), Some(6));
        assert_eq!(heap.pop_min(), Some(7));
        assert_eq!(heap.pop_min(), Some(8));

        assert_eq!(heap.len(), 0);
        assert_eq!(heap.peek_min(), None);
        assert_eq!(heap.min_key(), None);
        assert_eq!(heap.pop_min(), None);
    }

    #[test]
    fn test_heap_decrease_key() {
        let mut heap = FibHeap::new();

        heap.insert(3, 3);
        heap.insert(4, 4);

        heap.decrease_key(&3, 2);
        assert_eq!(heap.peek_min(), Some(&3));
        assert_eq!(heap.min_key(), Some(&2));

        heap.decrease_key(&4, 1);
        assert_eq!(heap.peek_min(), Some(&4));
        assert_eq!(heap.min_key(), Some(&1));
    }

    #[test]
    fn test_heap_decrease_key_not_exists() {
        let mut heap = FibHeap::new();

        heap.insert(2, 2);
        heap.insert(3, 3);
        heap.insert(4, 4);

        heap.decrease_key(&5, 1);
        heap.decrease_key(&6, 1);

        assert_eq!(heap.pop_min(), Some(2));
        assert_eq!(heap.pop_min(), Some(3));
        assert_eq!(heap.pop_min(), Some(4));
        assert_eq!(heap.pop_min(), None);
    }

    #[test]
    fn test_heap_decrease_and_pop() {
        let mut heap = FibHeap::new();

        heap.insert(1, 1);
        heap.insert(2, 2);
        heap.insert(3, 3);
        heap.insert(4, 4);
        heap.insert(5, 5);
        heap.insert(6, 6);

        assert_eq!(heap.pop_min(), Some(1));
        assert_eq!(heap.pop_min(), Some(2));

        heap.decrease_key(&6, 1);
        heap.insert(2, 2);

        assert_eq!(heap.peek_min(), Some(&6));
        assert_eq!(heap.min_key(), Some(&1));

        assert_eq!(heap.pop_min(), Some(6));
        assert_eq!(heap.pop_min(), Some(2));

        heap.decrease_key(&3, 3);
        heap.decrease_key(&4, 2);
        heap.decrease_key(&5, 1);

        assert_eq!(heap.pop_min(), Some(5));
        assert_eq!(heap.pop_min(), Some(4));
        assert_eq!(heap.pop_min(), Some(3));
        assert_eq!(heap.pop_min(), None);
        assert_eq!(heap.len(), 0);
    }

    #[test]
    fn test_heap_equal_keys() {
        let mut heap = FibHeap::new();

        heap.insert(1, 1);
        heap.insert(2, 1);

        assert_eq!(heap.pop_min(), Some(1));
        assert_eq!(heap.pop_min(), Some(2));
        assert_eq!(heap.pop_min(), None);
    }


    #[quickcheck]
    fn insert_and_pop_all(input: HashSet<u64>) -> bool {
        let mut heap = FibHeap::new();

        for item in input.iter() {
            heap.insert(*item, *item);
        }

        let mut sorted = Vec::<u64>::new();
        while let Some(item) = heap.pop_min() {
            sorted.push(item);
        }

        sorted.as_slice().windows(2).all(|w| w[0] <= w[1])
    }

    #[quickcheck]
    fn insert_and_decrease_and_pop(input: HashSet<u64>) -> bool {
        let input_vec = input.into_iter().collect::<Vec<u64>>();
        let mut heap = FibHeap::new();

        for item in input_vec.iter() {
            heap.insert(*item, u64::MAX);
        }

        for item in input_vec.iter() {
            heap.decrease_key(item, *item);
        }

        let mut sorted = Vec::<u64>::new();
        while let Some(item) = heap.pop_min() {
            sorted.push(item);
        }

        sorted.as_slice().windows(2).all(|w| w[0] <= w[1])
    }
}
