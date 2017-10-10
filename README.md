# Forest (Forust)

This was done as an exercise in using Box and more 'advanced' Rust features. I plan to make it library quality, but for now it contains some things that aren't idiomatic Rust, for instance the length function is recursive.

I also plan to include a fast binary tree version that uses references instead of boxing and a typed arena for allocation. But for now, there is a basic working binary tree implementation with `IntoIterator` and `Iterator` implementations.