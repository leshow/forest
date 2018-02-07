# Forest

Rust binary tree implementation. Uses regular heap-allocated `Box` for now, in the future I will include an implementation using plain references in a `typed-arena`. 

Functionality for now includes the most common methods you might need on a tree, `insert` `len` `fold` `into_iter` (owned iterator) `iter_mut` (mutable reference iterator) `contains` `is_empty`, and common iterator traits like `FromIterator` `IntoIterator` `Iterator` `Extend`. 
