# Forest

Rust binary tree implementation. ~~Uses regular heap-allocated `Box` for now~~, Now using `NonNull`! In the future I will include an implementation with upfront allocation using `typed-arena`.

Functionality for now includes the most common methods you might need on a tree, `insert` `len` `fold` `into_iter` (owned iterator) `iter_mut` (mutable reference iterator) `contains` `is_empty`, and common iterator traits like `FromIterator` `IntoIterator` `Iterator` `Extend`.
