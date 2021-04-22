#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;


pub struct FibHeap<I> {
    min: Option<usize>,
    trees: Vec<TreeNode<I>>,
    n: usize,
}

impl<I> FibHeap<I>
where
    I: PartialEq + Eq + Clone,
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

    pub fn peek_min(&self) -> Option<&I> {
        self.min
            .map(|min_idx| &self.trees[min_idx])
            .map(|root| &root.item)
    }

    pub fn min_key(&self) -> Option<&u64> {
        self.min
            .map(|min_idx| &self.trees[min_idx])
            .map(|root| &root.key)
    }

    pub fn insert(&mut self, item: I, key: u64) {
        self.trees.push(TreeNode::new(item, key));
        self.n += 1;
        if let Some(&min_key) = self.min_key() {
            if key < min_key {
                self.min = Some(self.trees.len() - 1);
            }
        } else {
            debug_assert!(self.trees.len() == 1);
            self.min = Some(0)
        }
    }

    fn remove(&mut self, node: &TreeNode<I>) -> Option<I> {
        if let Some(pos) = self.trees.iter().position(|n| n == node) {
            if self.min == Some(self.trees.len() - 1) {
                self.min = Some(pos);
            }
            self.n -= 1;
            let tree_node = self.trees.swap_remove(pos);
            for c in tree_node.children {
                self.trees.push(c);
            }
            Some(tree_node.item)
        } else {
            None
        }
    }

    fn cut(&mut self, node: &TreeNode<I>) {
        for root in self.trees.iter_mut() {
            // TODO save pointers to roots in map
            if let Some(tree_node) = root.cut(node) {
                self.trees.push(tree_node);
                break;
            }
        }
    }

    pub fn decrease_key(&mut self, item: &I, new_key: u64) {
        for (i, root) in self.trees.iter_mut().enumerate() {
            // TODO save pointers to roots in map
            debug_assert!(!root.mark);
            match root.decrease_key(item, new_key) {
                DecreaseKeyResult::Unmarked(cutoff) => {
                    if new_key < *self.min_key().unwrap() {
                        // tree node corresponding to decreased key is first in cutoff
                        self.min = Some(self.trees.len());
                    }
                    for mut node in cutoff {
                        node.mark = false;
                        self.trees.push(node);
                    }
                    break;
                }
                DecreaseKeyResult::KeyDecreased => {
                    if new_key < *self.min_key().unwrap() {
                        self.min = Some(i);
                    }
                    break;
                }
                DecreaseKeyResult::Marked(_) => panic!("Should not happen!"),
                DecreaseKeyResult::NotFound => {}
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

            let max_rank = (4.0 * (self.n as f64).log2()).ceil() as usize;
            let mut roots: Vec<Option<TreeNode<I>>> = vec![None; max_rank + 1];

            for root in self.trees.drain(..) {
                let mut node = root;
                while roots.get(node.rank()).unwrap().is_some() {
                    let other = roots.remove(node.rank()).unwrap();
                    node = link(node, other);
                }
                let rank = node.rank();
                roots[rank] = Some(node);
            }

            *min_idx = 0;
            for root in roots {
                if let Some(root) = root {
                    let key = root.key;
                    self.trees.push(root);
                    if key < self.trees[*min_idx].key {
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

fn link<I>(mut first: TreeNode<I>, mut second: TreeNode<I>) -> TreeNode<I> {
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

enum DecreaseKeyResult<I> {
    NotFound,
    KeyDecreased,
    Marked(Vec<TreeNode<I>>),
    Unmarked(Vec<TreeNode<I>>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TreeNode<I> {
    children: Vec<TreeNode<I>>,
    key: u64,
    mark: bool,
    item: I,
}

impl<I> TreeNode<I>
where
    I: Eq + PartialEq,
{
    fn new(item: I, key: u64) -> Self {
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

    fn cut(&mut self, node: &TreeNode<I>) -> Option<TreeNode<I>> {
        for (i, c) in self.children.iter_mut().enumerate() {
            if c == node {
                return Some(self.children.swap_remove(i));
            } else {
                if let Some(res) = c.cut(node) {
                    return Some(res);
                }
            }
        }
        None
    }

    fn decrease_key(&mut self, item: &I, new_key: u64) -> DecreaseKeyResult<I> {
        if &self.item == item {
            if self.key > new_key {
                self.key = new_key;
                DecreaseKeyResult::KeyDecreased
            } else {
                DecreaseKeyResult::NotFound
            }
        } else {
            for (i, c) in self.children.iter_mut().enumerate() {
                match c.decrease_key(item, new_key) {
                    DecreaseKeyResult::KeyDecreased => {
                        if self.key > new_key {
                            let cutoff = vec![self.children.swap_remove(i)];
                            if self.mark {
                                return DecreaseKeyResult::Marked(cutoff);
                            } else {
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
                    DecreaseKeyResult::NotFound => {}
                }
            }

            DecreaseKeyResult::NotFound
        }
    }
}



#[cfg(test)]
mod test_heap {
    use super::*;


    #[test]
    fn test_heap_empty() {
        let mut heap: FibHeap<()> = FibHeap::new();

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


    #[quickcheck]
    fn insert_and_pop_all(mut input: Vec<u64>) -> bool {
        let mut heap = FibHeap::new();

        for item in input.iter() {
            heap.insert(*item, *item);
        }

        let mut sorted = Vec::<u64>::new();
        while let Some(item) = heap.pop_min() {
            sorted.push(item);
        }

        input.sort();

        input == sorted
    }
}