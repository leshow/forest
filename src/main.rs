#![feature(box_syntax, box_patterns)]

use std::cmp::Ordering;

type MyTree<T> = Option<Box<Tree<T>>>;

#[derive(PartialOrd, PartialEq, Debug)]
struct Tree<T: Ord> {
    l: MyTree<T>,
    r: MyTree<T>,
    data: T,
}

impl<T: Ord> Tree<T> {
    fn new_unboxed(data: T) -> Tree<T> {
        Tree {
            l: None,
            data,
            r: None,
        }
    }
    fn new(data: T) -> MyTree<T> {
        Some(box Tree {
            l: None,
            data,
            r: None,
        })
    }
}

fn len<T: Ord>(tree: &MyTree<T>) -> i32 {
    // without box syntax
    // if let &Some(ref node) = tree {
    //     let node = &*node;
    //     1 + count(&node.l) + count(&node.r)
    // } else {
    //     0
    // }
    if let &Some(box Tree { ref l, ref r, .. }) = tree {
        1 + len(l) + len(r)
    } else {
        0
    }
}

fn insert<T: Ord>(tree: &mut MyTree<T>, item: T) {
    if let &mut Some(ref mut node) = tree {
        match item.cmp(&node.data) {
            Ordering::Less => if node.l == None {
                node.l = Tree::new(item);
            } else {
                insert(&mut node.l, item);
            },
            Ordering::Greater => if node.r == None {
                node.r = Tree::new(item);
            } else {
                insert(&mut node.r, item);
            },
            Ordering::Equal => {
                return;
            }
        }
    }
}

fn fold<T: Ord, B, F>(tree: &MyTree<T>, init: B, mut f: F) -> B
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
    insert(&mut tree, 2);
    insert(&mut tree, 3);
    insert(&mut tree, 4);
    insert(&mut tree, 5);
    insert(&mut tree, 6);
    insert(&mut tree, 7);
    insert(&mut tree, 8);
    insert(&mut tree, -18);
    insert(&mut tree, -10);
    insert(&mut tree, -1);
    insert(&mut tree, -2);

    println!("{:?}", tree);
    println!("{:?}", len(&tree));
    println!("{:?}", fold(&tree, 0, |acc, &x| acc + x));
}
