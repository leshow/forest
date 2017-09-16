// extern crate typed_arena;

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
