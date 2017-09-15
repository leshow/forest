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


// #[derive(PartialOrd, PartialEq)]
// struct Tree<'a, T: 'a + Ord> {
//     l: Option<&'a mut Tree<'a, T>>,
//     r: Option<&'a mut Tree<'a, T>>,
//     data: T,
// }

// impl<'a, T: 'a + Ord> Tree<'a, T> {
//     fn new(t: T) -> Tree<'a, T> {
//         Tree {
//             l: None,
//             data: t,
//             r: None,
//         }
//     }
// }


// fn count<T: Ord>(tree: &Option<&mut Tree<T>>) -> i32 {
//     if let &Some(&mut Tree { ref l, ref r, .. }) = tree {
//         1 + count(l) + count(r)
//     } else {
//         0
//     }
// }
// fn insert<'a, T: 'a + Ord>(tree: &Option<&'a mut Tree<'a, T>>, item: T) {
//     if let &Some(&mut Tree { mut l, data, mut r }) = tree {
//         if item < data {
//             if l == None {
//                 let t_: Tree<'a, T> = Tree::new(item);
//                 l = Some(&mut t_);
//             } else {
//                 insert(&l, item);
//             }
//         } else if item > data {
//             if r == None {
//                 let t_: Tree<'a, T> = Tree::new(item);
//                 r = Some(&mut t_);
//             } else {
//                 insert(&r, item);
//             }
//         }
//     }
// }
// fn main() {}
