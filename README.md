# IPLD `[no-std]` Bundle

This crate bundles the following crates:
- https://github.com/n0-computer/iroh-car
- https://github.com/multiformats/rust-cid
- https://github.com/multiformats/rust-multibase
- https://github.com/multiformats/rust-multihash
- https://github.com/ipld/serde_ipld_dagcbor
- https://github.com/ipld/rust-ipld-core

Into a `[no_std]` crate that can be compiled and executed in WASM environments. The individual crates listed above are notoriously problematic to get working together in `[no_std]`.

You shouldn't default to using this crate. Consider using it only if you've exhausted your other options and you're getting compilation errors related to multiple definitions of panic handlers or other signs of `std` leakage into your code.
