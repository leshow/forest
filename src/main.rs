#![feature(box_syntax, box_patterns)]

use std::iter::IntoIterator;
use std::cmp::Ordering;

#[derive(Debug, PartialOrd)]
pub struct Tree<T: Ord>(Option<Box<Node<T>>>);

impl<T: Ord> PartialEq for Tree<T> {
    fn eq(&self, other: &Self) -> bool {
        match self.0 {
            Some(box ref x) => match other.0 {
                Some(box ref y) => y == x,
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
    fn insert(&mut self, item: T);
    fn len(&self) -> usize;
    fn fold<B, F>(&self, init: B, f: F) -> B
    where
        F: for<'a> FnMut(B, &'a T) -> B;
}

#[derive(Debug, PartialEq, PartialOrd)]
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
        // without box syntax
        // if let &Some(ref node) = tree {
        //     let node = &*node;
        //     1 + count(&node.l) + count(&node.r)
        // } else {
        //     0
        // }
        if let &Some(box Node { ref l, ref r, .. }) = &self.0 {
            1 + l.len() + r.len()
        } else {
            0
        }
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
    fn fold<B, F>(&self, init: B, mut f: F) -> B
    where
        F: for<'a> FnMut(B, &'a T) -> B,
    {
        let mut acc = init;
        if let &Some(ref node) = &self.0 {
            let node = &*node;
            let mut stack = vec![node];
            while let Some(ref node) = stack.pop() {
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
        //     self.right.push(r);
        //     self.cur = Some(data);
        //     root = l;
        // }
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
        match self.right.pop() {
            Some(t) => self.add_left(t),
            _ => {}
        }
        return cur_node;
    }
}

fn main() {
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

    println!("{:?}", tree);
    println!("{:?}", tree.len());
    println!("{:?}", tree.fold(0, |acc, &x| acc + x));
    let mut i = tree.into_iter();
    println!("{:?}", i.next());
    println!("{:?}", i.next());
    println!("{:?}", i.next());
}
