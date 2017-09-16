#![feature(box_syntax, box_patterns)]

#[derive(PartialOrd, PartialEq)]
struct Tree<T: Ord> {
    l: Option<Box<Tree<T>>>,
    r: Option<Box<Tree<T>>>,
    data: T,
}

impl<T: Ord> Tree<T> {
    fn new(t: T) -> Tree<T> {
        Tree {
            l: None,
            data: t,
            r: None,
        }
    }
}

fn count<T: Ord>(tree: &Option<Box<Tree<T>>>) -> i32 {
    // without box syntax
    // if let &Some(ref node) = tree {
    //     let node = &*node;
    //     1 + count(&node.l) + count(&node.r)
    // } else {
    //     0
    // }
    if let &Some(box Tree { ref l, ref r, .. }) = tree {
        1 + count(l) + count(r)
    } else {
        1
    }
}
fn insert<T: Ord>(tree: &mut Option<Box<Tree<T>>>, item: T) {
    if let &mut Some(ref mut node) = tree {
        if item < node.data {
            if node.l == None {
                let t_ = Tree::new(item);
                node.l = Some(box t_);
            } else {
                insert(&mut node.l, item);
            }
        } else if item > node.data {
            if node.r == None {
                let t_ = Tree::new(item);
                node.r = Some(box t_);
            } else {
                insert(&mut node.r, item);
            }
        }
    }
}

fn main() {}
