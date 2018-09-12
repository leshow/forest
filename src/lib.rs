#![feature(box_syntax, box_patterns, box_into_raw_non_null)]

extern crate rayon;

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::iter::{Extend, FromIterator, IntoIterator};
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Debug)]
pub struct Tree<T> {
    root: Option<Node<T>>,
    length: usize,
}

impl<T> Tree<T> {
    pub fn empty() -> Self {
        Tree {
            root: None,
            length: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl<T: Ord> Tree<T> {
    pub fn new(t: T) -> Self {
        Tree {
            root: Some(Node::new(t)),
            length: 1,
        }
    }
    pub fn insert(&mut self, t: T) {
        match self.root {
            None => {
                self.root = Some(Node::new(t));
            }
            Some(ref mut n) => {
                n.insert(t);
            }
        }
        self.length += 1;
    }
    pub fn contains(&self, t: &T) -> Option<&Node<T>> {
        match self.root {
            Some(ref n) => n.contains(t),
            None => None,
        }
    }
    pub fn fold<B, F>(&self, init: B, f: F) -> B
    where
        F: FnMut(B, &T) -> B,
    {
        match self.root {
            None => init,
            Some(ref n) => n.fold(init, f),
        }
    }
}

impl<T: PartialEq> PartialEq for Tree<T> {
    fn eq(&self, other: &Self) -> bool {
        match self.root {
            Some(ref x) => match other.root {
                Some(ref y) => y == x,
                None => false,
            },
            None => match other.root {
                Some(_) => false,
                None => true,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Node<T> {
    pub l: Link<T>,
    pub r: Link<T>,
    pub data: T,
    _marker: PhantomData<Box<Node<T>>>,
}

#[derive(Debug)]
pub struct Link<T> {
    ptr: Option<NonNull<Node<T>>>,
}

impl<T: PartialEq> PartialEq for Link<T> {
    fn eq(&self, other: &Self) -> bool {
        match self.ptr {
            Some(n) => match other.ptr {
                Some(o) => unsafe { *(n.as_ptr()) == *(o.as_ptr()) },
                None => false,
            },
            None => match other.ptr {
                Some(_) => false,
                None => true,
            },
        }
    }
}

impl<T> Link<T> {
    fn len(&self) -> usize {
        match self.ptr {
            Some(n) => unsafe { (&*n.as_ptr()).len() },
            None => 0,
        }
    }
    fn none() -> Self {
        Link { ptr: None }
    }
    fn is_none(&self) -> bool {
        match self.ptr {
            Some(_) => false,
            None => true,
        }
    }
}

// impl<T> AsMut<Option<Node<T>>> for Link<T> {
//     fn as_mut(&mut self) -> &mut Option<Node<T>> {
//         &mut self.ptr.map(|node| unsafe { *node.as_ptr() })
//     }
// }

// impl<T> AsRef<Option<Node<T>>> for Link<T> {
//     fn as_ref(&self) -> &Option<Node<T>> {
//         &self.ptr.map(|node| unsafe { *(node.as_ptr()) })
//     }
// }

impl<T: Ord> Link<T> {
    fn new(data: T) -> Self {
        Link {
            ptr: Some(Box::into_raw_non_null(Box::new(Node::new(data)))),
        }
    }
    fn contains(&self, item: &T) -> Option<&Node<T>> {
        match self.ptr {
            Some(ref o) => {
                let n = unsafe { &(*o.as_ptr()) };
                if n.data == *item {
                    return Some(&n);
                } else {
                    return n.contains(&item);
                }
            }
            None => None,
        }
    }
}

impl<T> Node<T> {
    fn len(&self) -> usize {
        1 + self.l.len() + self.r.len()
    }
}

impl<T: Ord> Node<T> {
    fn new(data: T) -> Self {
        Node {
            l: Link::none(),
            r: Link::none(),
            data,
            _marker: PhantomData,
        }
    }
    fn insert(&mut self, item: T) {
        match item.cmp(&self.data) {
            Ordering::Less => match self.l.ptr {
                None => self.l = Link::new(item),
                Some(node) => {
                    let node = unsafe { &mut (*node.as_ptr()) };
                    node.insert(item);
                }
            },
            Ordering::Greater => match self.r.ptr {
                None => self.r = Link::new(item),
                Some(node) => {
                    let node = unsafe { &mut (*node.as_ptr()) };
                    node.insert(item);
                }
            },
            Ordering::Equal => {
                return;
            }
        }
    }
    fn contains(&self, item: &T) -> Option<&Node<T>> {
        if self.data == *item {
            Some(&self)
        } else {
            let l_ = self.l.contains(item);
            let r_ = self.r.contains(item);
            match l_.is_some() {
                true => l_,
                false => match r_.is_some() {
                    true => r_,
                    false => None,
                },
            }
        }
    }

    fn fold<B, F>(&self, init: B, mut f: F) -> B
    where
        F: for<'a> FnMut(B, &'a T) -> B,
    {
        let mut acc = init;
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            acc = f(acc, &node.data);
            if let Some(right) = node.r.ptr {
                let r = unsafe { &(*right.as_ptr()) };
                stack.push(r);
            }
            if let Some(left) = node.l.ptr {
                let l = unsafe { &(*left.as_ptr()) };
                stack.push(l);
            }
        }
        acc
    }
}

impl<T> Tree<T> {
    fn iter<'a>(&'a self) -> TreeRefIter<'a, T> {
        let mut iter = TreeRefIter {
            unvisited: Vec::new(),
        };
        if let Some(ref root) = self.root {
            iter.push_left(&root.l);
        }
        iter
    }
}

// Reference Iterator

pub struct TreeRefIter<'a, T: 'a> {
    unvisited: Vec<&'a Node<T>>,
}

impl<'a, T: 'a> TreeRefIter<'a, T> {
    fn push_left(&mut self, mut tree: &'a Link<T>) {
        while let Some(node) = (*tree).ptr {
            let node = unsafe { &(*node.as_ptr()) };
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
        if let Some(root) = node.root {
            iter.add_left(root.l);
        }
        iter
    }

    fn add_left(&mut self, mut root: Link<T>) {
        while let Some(node) = root.ptr.take() {
            let Node { l, r, data, .. } = unsafe { *Box::from_raw(node.as_ptr()) };
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
        self.root.as_mut().map(|root| {
            deque.push_front(root.iter_mut());
        });
        IterMut(deque)
    }
}

impl<T> Node<T> {
    pub fn iter_mut(&mut self) -> NodeIterMut<T> {
        NodeIterMut {
            elem: Some(&mut self.data),
            left: self
                .l
                .ptr
                .as_mut()
                .map(|node| unsafe { &mut *node.as_ptr() }),
            right: self
                .r
                .ptr
                .as_mut()
                .map(|node| unsafe { &mut *node.as_ptr() }),
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
        for node in &tree {
            println!("{:?}", node);
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
