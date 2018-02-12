#![feature(box_syntax, box_patterns)]

extern crate rayon;

use std::iter::{Extend, FromIterator, IntoIterator};
use std::collections::VecDeque;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Tree<T> {
    root: Link<T>,
}

impl<T: Ord> BinaryTree<T> for Tree<T> {
    fn new(t: T) -> Self {
        Tree { root: Link::new(t) }
    }
    fn empty() -> Self {
        Tree { root: None }
    }
    fn insert(&mut self, t: T) {
        if self.root.is_none() {
            self.root = Link::new(t);
        } else {
            self.root.insert(t);
        }
    }
    fn contains(&self, t: &T) -> Option<&Link<T>> {
        self.root.contains(t)
    }
    fn len(&self) -> usize {
        self.root.len()
    }
    fn is_empty(&self) -> bool {
        self.root.is_empty()
    }
    fn fold<B, F>(&self, init: B, f: F) -> B
    where
        F: FnMut(B, &T) -> B,
    {
        self.root.fold(init, f)
    }
}

pub type Link<T> = Option<Box<Node<T>>>;

impl<T: Ord> PartialEq for Tree<T> {
    fn eq(&self, other: &Self) -> bool {
        match self.root {
            Some(box ref x) => match other.root {
                Some(box ref y) => *y == *x,
                None => false,
            },
            None => match other.root {
                Some(_) => false,
                None => true,
            },
        }
    }
}

pub trait BinaryTree<T: Ord> {
    fn empty() -> Self;
    fn new(t: T) -> Self;
    fn insert(&mut self, T);
    fn contains(&self, &T) -> Option<&Link<T>>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn fold<B, F>(&self, init: B, f: F) -> B
    where
        F: for<'a> FnMut(B, &'a T) -> B;
}

#[derive(Debug, PartialEq)]
pub struct Node<T> {
    pub l: Link<T>,
    pub r: Link<T>,
    pub data: T,
}

impl<T: Ord> Node<T> {
    fn new(data: T) -> Node<T> {
        Node {
            l: None,
            data,
            r: None,
        }
    }
}

impl<T> Tree<T> {
    fn iter<'a>(&'a self) -> TreeRefIter<'a, T> {
        let mut iter = TreeRefIter {
            unvisited: Vec::new(),
        };
        iter.push_left(&self.root);
        iter
    }
}

impl<T: Ord> BinaryTree<T> for Link<T> {
    fn empty() -> Self {
        None
    }

    fn new(t: T) -> Self {
        Some(box Node::new(t))
    }

    fn len(&self) -> usize {
        if let &Some(box Node { ref l, ref r, .. }) = self {
            1 + l.len() + r.len()
        } else {
            0
        }
        // without box syntax
        // if let &Some(ref node) = tree {
        //     let node = &*node;
        //     1 + node.l.len() + node.r.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn insert(&mut self, item: T) {
        if let &mut Some(ref mut node) = self {
            match item.cmp(&node.data) {
                Ordering::Less => if node.l == None {
                    node.l = <Link<T> as BinaryTree<T>>::new(item);
                } else {
                    node.l.insert(item);
                },
                Ordering::Greater => if node.r == None {
                    node.r = <Link<T> as BinaryTree<T>>::new(item);
                } else {
                    node.r.insert(item);
                },
                Ordering::Equal => {
                    return;
                }
            }
        }
    }

    fn contains(&self, item: &T) -> Option<&Link<T>> {
        if let &Some(box Node {
            ref l,
            ref r,
            ref data,
        }) = self
        {
            if item == data {
                Some(self)
            } else {
                let l_ = l.contains(item);
                let r_ = r.contains(item);
                match l_.is_some() {
                    true => l_,
                    false => match r_.is_some() {
                        true => r_,
                        false => None,
                    },
                }
            }
        } else {
            None
        }
    }

    fn fold<B, F>(&self, init: B, mut f: F) -> B
    where
        F: for<'a> FnMut(B, &'a T) -> B,
    {
        let mut acc = init;
        if let &Some(ref node) = self {
            let node = &*node;
            let mut stack = vec![node];
            while let Some(node) = stack.pop() {
                acc = f(acc, &node.data);
                if let Some(ref right) = node.r {
                    stack.push(right);
                }
                if let Some(ref left) = node.l {
                    stack.push(left);
                }
            }
            acc
        } else {
            acc
        }
    }
}
// Reference Iterator

pub struct TreeRefIter<'a, T: 'a> {
    unvisited: Vec<&'a Node<T>>,
}

impl<'a, T: 'a> TreeRefIter<'a, T> {
    fn push_left(&mut self, mut tree: &'a Link<T>) {
        while let Some(ref node) = *tree {
            self.unvisited.push(node);
            tree = &node.l;
        }
    }
}

impl<'a, T: 'a> IntoIterator for &'a Tree<T> {
    type Item = &'a T;
    type IntoIter = TreeRefIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for TreeRefIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        self.unvisited.pop().map(|node| {
            self.push_left(&node.r);
            return &node.data;
        })
    }
}

// Owned Iterator

pub struct TreeIter<T> {
    right: Vec<Link<T>>,
    cur: Option<T>,
}

impl<T> TreeIter<T> {
    fn new(node: Tree<T>) -> TreeIter<T> {
        let mut iter = TreeIter {
            right: vec![],
            cur: None,
        };
        iter.add_left(node.root);
        iter
    }

    fn add_left(&mut self, mut root: Link<T>) {
        // https://github.com/rust-lang/rust/issues/19828
        // while let Some(box Node { l, r, data }) = root.take() {
        while let Some(box node) = root.take() {
            let Node { l, r, data } = node;
            self.right.push(r);
            self.cur = Some(data);
            root = l;
        }
    }
}

impl<T> IntoIterator for Tree<T> {
    type Item = T;
    type IntoIter = TreeIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        TreeIter::new(self)
    }
}

impl<T> Iterator for TreeIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let cur_node = self.cur.take();
        if let Some(t) = self.right.pop() {
            self.add_left(t)
        }
        cur_node
    }
}

// Mutable Iterator

pub struct NodeIterMut<'a, T: 'a> {
    elem: Option<&'a mut T>,
    left: Option<&'a mut Node<T>>,
    right: Option<&'a mut Node<T>>,
}

pub enum State<'a, T: 'a> {
    Elem(&'a mut T),
    Node(&'a mut Node<T>),
}

pub struct IterMut<'a, T: 'a>(VecDeque<NodeIterMut<'a, T>>);

impl<T> Tree<T> {
    pub fn iter_mut(&mut self) -> IterMut<T> {
        let mut deque = VecDeque::new();
        self.root
            .as_mut()
            .map(|root| deque.push_front(root.iter_mut()));
        IterMut(deque)
    }
}

impl<T> Node<T> {
    pub fn iter_mut(&mut self) -> NodeIterMut<T> {
        NodeIterMut {
            elem: Some(&mut self.data),
            left: self.l.as_mut().map(|node| &mut **node),
            right: self.r.as_mut().map(|node| &mut **node),
        }
    }
}

impl<'a, T> Iterator for NodeIterMut<'a, T> {
    type Item = State<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.left.take() {
            Some(node) => Some(State::Node(node)),
            None => match self.elem.take() {
                Some(elem) => Some(State::Elem(elem)),
                None => match self.right.take() {
                    Some(node) => Some(State::Node(node)),
                    None => None,
                },
            },
        }
    }
}

impl<'a, T> DoubleEndedIterator for NodeIterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.right.take() {
            Some(node) => Some(State::Node(node)),
            None => match self.elem.take() {
                Some(elem) => Some(State::Elem(elem)),
                None => match self.left.take() {
                    Some(node) => Some(State::Node(node)),
                    None => None,
                },
            },
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.front_mut().and_then(|node_it| node_it.next()) {
                Some(State::Elem(elem)) => return Some(elem),
                Some(State::Node(node)) => self.0.push_front(node.iter_mut()),
                None => if let None = self.0.pop_front() {
                    return None;
                },
            }
        }
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.back_mut().and_then(|node_it| node_it.next_back()) {
                Some(State::Elem(elem)) => return Some(elem),
                Some(State::Node(node)) => self.0.push_back(node.iter_mut()),
                None => if let None = self.0.pop_back() {
                    return None;
                },
            }
        }
    }
}

impl<T: Ord> Extend<T> for Tree<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl<T: Ord> FromIterator<T> for Tree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = Tree::empty();
        tree.extend(iter);
        tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len_and_fold() {
        let mut tree = Tree::empty();
        tree.insert(1);
        tree.insert(2);
        tree.insert(3);
        tree.insert(4);
        tree.insert(5);
        tree.insert(6);
        tree.insert(7);
        tree.insert(8);
        tree.insert(-18);
        tree.insert(-10);
        tree.insert(-1);
        tree.insert(-2);

        assert_eq!(12, tree.len());
        assert_eq!(5, tree.fold(0, |acc, &x| acc + x));
    }

    #[test]
    fn test_plain_iter() {
        let mut tree = Tree::empty();
        tree.insert(1);
        tree.insert(2);
        tree.insert(3);
        tree.insert(4);
        tree.insert(5);
        tree.insert(6);
        tree.insert(7);
        tree.insert(8);

        tree.into_iter().zip(1..9).for_each(|(node, i)| {
            assert_eq!(node, i);
        });
    }

    #[test]
    fn test_extend() {
        let mut tree = Tree::empty();
        tree.extend(0..10);
        tree.into_iter().zip(0..10).for_each(|(node, i)| {
            assert_eq!(node, i);
        });
    }

    #[test]
    fn test_from_iter() {
        let tree = Tree::from_iter(vec![0, 1, 2, 3, 4, 5]);

        assert_eq!(tree.len(), 6);
    }

    #[test]
    fn test_iter_mut() {
        let mut tree = Tree::from_iter(vec![0, 1, 2, 3, 4, 5]);

        for node in tree.iter_mut() {
            *node += 1;
        }
        assert_eq!(tree.len(), 6);
        assert_eq!(1 + 2 + 3 + 4 + 5 + 6, tree.into_iter().sum())
    }
    #[test]
    fn test_ref_mut() {
        let mut tree = Tree::from_iter(vec![0, 1, 2, 3, 4, 5]);

        for node in &tree {
            if *node < 0 {
                panic!();
            } else if *node > 5 {
                panic!();
            }
        }
        assert_eq!(tree.len(), 6);
    }
}
