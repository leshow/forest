#![feature(box_syntax, box_patterns)]

use std::cmp::Ordering;

// struct Node<T: Ord>(Option<Box<Tree<T>>>);
type Tree<T> = Option<Box<Node<T>>>;

trait Link<T: Ord> {
    fn new(data: T) -> Tree<T>;
    fn insert(&mut self, item: T);
    fn len(&self) -> usize;
}

#[derive(PartialOrd, PartialEq, Debug)]
struct Node<T: Ord> {
    l: Tree<T>,
    r: Tree<T>,
    data: T,
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

impl<T: Ord> Link<T> for Tree<T> {
    fn new(data: T) -> Tree<T> {
        Some(box Node::new(data))
    }

    fn len(&self) -> usize {
        // without box syntax
        // if let &Some(ref node) = tree {
        //     let node = &*node;
        //     1 + count(&node.l) + count(&node.r)
        // } else {
        //     0
        // }
        if let &Some(box Node { ref l, ref r, .. }) = self {
            1 + l.len() + r.len()
        } else {
            0
        }
    }

    fn insert(&mut self, item: T) {
        if let &mut Some(ref mut node) = self {
            match item.cmp(&node.data) {
                Ordering::Less => if node.l == None {
                    node.l = <Tree<T> as Link<T>>::new(item);
                } else {
                    node.l.insert(item);
                },
                Ordering::Greater => if node.r == None {
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
}
fn fold<T: Ord, B, F>(tree: &Tree<T>, init: B, mut f: F) -> B
where
    F: for<'a> FnMut(B, &'a T) -> B,
{
    let mut acc = init;
    if let &Some(ref node) = tree {
        let node = &*node;
        let mut stack = vec![node];

        while let Some(ref node) = stack.pop() {
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
    println!("{:?}", fold(&tree, 0, |acc, &x| acc + x));
}
