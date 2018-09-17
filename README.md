A Rust library to minimize memory allocations. Use `allocate()`
to get recyclable values of type `T`. When those recyclables
are dropped, they're returned to the recycler. The next time
`allocate()` is called, the value will be pulled from the
recycler instead being allocated from memory.
