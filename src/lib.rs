#![feature(box_syntax, box_patterns)]

extern crate rayon;

use std::iter::IntoIterator;
use std::cmp::Ordering;

#[derive(Debug, PartialOrd)]
pub struct Tree<T: Ord + Sync>(Option<Box<Node<T>>>);

impl<T: Ord + Sync> PartialEq for Tree<T> {
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

pub trait Link<T: Ord + Sync> {
    fn new(data: T) -> Tree<T>;
    fn insert(&mut self, T);
    fn contains(&self, &T) -> Option<&Tree<T>>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn par_len(&self) -> usize;
    fn fold<B, F>(&self, init: B, f: F) -> B
    where
        F: for<'a> FnMut(B, &'a T) -> B;
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Node<T: Ord + Sync> {
    pub l: Tree<T>,
    pub r: Tree<T>,
    pub data: T,
}

impl<T: Ord + Sync> Node<T> {
    fn new(data: T) -> Node<T> {
        Node {
            l: Tree(None),
            data,
            r: Tree(None),
        }
    }
}

impl<T: Ord + Sync> Link<T> for Tree<T> {
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

    fn par_len(&self) -> usize {
        if let Some(box Node { ref l, ref r, .. }) = self.0 {
            let (len_l, len_r) = rayon::join(|| l.len(), || r.len());
            1 + len_l + len_r
        } else {
            0
        }
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
                if l_.is_some() {
                    l_
                } else {
                    let r_ = r.contains(item);
                    if r_.is_some() {
                        r_
                    } else {
                        None
                    }
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

pub struct TreeIter<T: Ord + Sync> {
    right: Vec<Tree<T>>,
    cur: Option<T>,
}

impl<T: Ord + Sync> TreeIter<T> {
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

impl<T: Ord + Sync> IntoIterator for Tree<T> {
    type Item = T;
    type IntoIter = TreeIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        TreeIter::new(self)
    }
}

impl<T: Ord + Sync> Iterator for TreeIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let cur_node = self.cur.take();
        if let Some(t) = self.right.pop() {
            self.add_left(t)
        }
        cur_node
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

        println!("{:?}", tree);
        println!("{:?}", tree.len());
        println!("{:?}", tree.fold(0, |acc, &x| acc + x));
        let mut i = tree.into_iter();
        println!("{:?}", i.next());
        println!("{:?}", i.next());
        println!("{:?}", i.next());
    }
}
