#![feature(box_syntax, box_patterns)]

use std::iter::IntoIterator;
use std::cmp::Ordering;

// struct Node<T: Ord>(Option<Box<Tree<T>>>);
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
pub(crate) struct Node<T: Ord> {
    l: Tree<T>,
    r: Tree<T>,
    data: T,
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

// pub struct TreeIter<T: Ord> {
//     right: Vec<Tree<T>>,
//     cur: Option<T>,
// }

// impl<'a, T: Ord> IntoIterator for &'a Tree<T> {
//     type Item = T;
//     type IntoIter = TreeIter<T>;
//     fn into_iter(self) -> Self::IntoIter {
//         unimplemented!()
//     }
// }

// impl<'a, T: Ord> Iterator for &'a TreeIter<T> {
//     type Item = T;
//     fn next(&mut self) -> Option<T> {
//         unimplemented!()
//     }
// }

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
}
