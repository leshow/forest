#![feature(box_syntax, box_patterns)]

extern crate rayon;

use std::iter::{FromIterator, IntoIterator};
use std::collections::VecDeque;
use std::convert::AsMut;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Tree<T: Ord>(Option<Box<Node<T>>>);

impl<T: Ord> AsMut<Option<Box<Node<T>>>> for Tree<T> {
    fn as_mut(&mut self) -> &mut Option<Box<Node<T>>> {
        &mut self.0
    }
}

impl<T: Ord> PartialEq for Tree<T> {
    fn eq(&self, other: &Self) -> bool {
        match self.0 {
            Some(box ref x) => match other.0 {
                Some(box ref y) => *y == *x,
                None => false,
            },
            None => match other.0 {
                Some(_) => false,
                None => true,
            },
        }
    }
}

pub trait Link<T: Ord> {
    fn new(data: T) -> Tree<T>;
    fn insert(&mut self, T);
    fn contains(&self, &T) -> Option<&Tree<T>>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn fold<B, F>(&self, init: B, f: F) -> B
    where
        F: for<'a> FnMut(B, &'a T) -> B;
}

#[derive(Debug, PartialEq)]
pub struct Node<T: Ord> {
    pub l: Tree<T>,
    pub r: Tree<T>,
    pub data: T,
}

impl<T: Ord> Node<T> {
    fn new(data: T) -> Node<T> {
        Node {
            l: Tree(None),
            data,
            r: Tree(None),
        }
    }
}

impl<T: Ord> Link<T> for Tree<T> {
    fn new(data: T) -> Tree<T> {
        Tree(Some(box Node::new(data)))
    }

    fn len(&self) -> usize {
        if let Some(box Node { ref l, ref r, .. }) = self.0 {
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
        if let Some(ref mut node) = self.0 {
            match item.cmp(&node.data) {
                Ordering::Less => if node.l.0 == None {
                    node.l = <Tree<T> as Link<T>>::new(item);
                } else {
                    node.l.insert(item);
                },
                Ordering::Greater => if node.r.0 == None {
                    node.r = <Tree<T> as Link<T>>::new(item);
                } else {
                    node.r.insert(item);
                },
                Ordering::Equal => {
                    return;
                }
            }
        }
    }

    fn contains(&self, item: &T) -> Option<&Tree<T>> {
        if let Some(box Node {
            ref l,
            ref r,
            ref data,
        }) = self.0
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
        if let Some(ref node) = self.0 {
            let node = &*node;
            let mut stack = vec![node];
            while let Some(node) = stack.pop() {
                acc = f(acc, &node.data);
                if let Some(ref right) = node.r.0 {
                    stack.push(right);
                }
                if let Some(ref left) = node.l.0 {
                    stack.push(left);
                }
            }
            acc
        } else {
            acc
        }
    }
}

pub struct TreeIter<T: Ord> {
    right: Vec<Tree<T>>,
    cur: Option<T>,
}

impl<T: Ord> TreeIter<T> {
    pub fn new(node: Tree<T>) -> TreeIter<T> {
        let mut iter = TreeIter {
            right: vec![],
            cur: None,
        };
        iter.add_left(node);
        iter
    }

    fn add_left(&mut self, mut root: Tree<T>) {
        // https://github.com/rust-lang/rust/issues/19828
        // while let Some(box Node { l, r, data }) = root.0.take() {
        while let Some(box node) = root.0 {
            let Node { l, r, data } = node;
            self.right.push(r);
            self.cur = Some(data);
            root = l;
        }
    }
}

impl<T: Ord> IntoIterator for Tree<T> {
    type Item = T;
    type IntoIter = TreeIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        TreeIter::new(self)
    }
}

impl<T: Ord> Iterator for TreeIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let cur_node = self.cur.take();
        if let Some(t) = self.right.pop() {
            self.add_left(t)
        }
        cur_node
    }
}

pub struct NodeIterMut<'a, T: 'a + Ord> {
    elem: Option<&'a mut T>,
    left: Option<&'a mut Node<T>>,
    right: Option<&'a mut Node<T>>,
}

pub enum State<'a, T: 'a + Ord> {
    Elem(&'a mut T),
    Node(&'a mut Node<T>),
}

pub struct IterMut<'a, T: 'a + Ord>(VecDeque<NodeIterMut<'a, T>>);

impl<T: Ord> Tree<T> {
    pub fn iter_mut(&mut self) -> IterMut<T> {
        let mut deque = VecDeque::new();
        self.as_mut().map(|root| deque.push_front(root.iter_mut()));
        IterMut(deque)
    }
}

impl<T: Ord> Node<T> {
    pub fn iter_mut(&mut self) -> NodeIterMut<T> {
        NodeIterMut {
            elem: Some(&mut self.data),
            left: self.l.as_mut().map(|node| &mut **node),
            right: self.r.as_mut().map(|node| &mut **node),
        }
    }
}

impl<'a, T: Ord> Iterator for NodeIterMut<'a, T> {
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

impl<'a, T: Ord> DoubleEndedIterator for NodeIterMut<'a, T> {
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

impl<'a, T: Ord> Iterator for IterMut<'a, T> {
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

impl<'a, T: Ord> DoubleEndedIterator for IterMut<'a, T> {
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut tree = Tree::new(1);
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
        println!("{:?}", tree.fold(0, |acc, &x| acc + x));
        let mut i = tree.into_iter();
        println!("{:?}", i.next());
        println!("{:?}", i.next());
        println!("{:?}", i.next());
    }
}
