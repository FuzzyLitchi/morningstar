# Hello!

This is a "simple" Rust implementation of DES. I made this to have a hackable version of DES I could cryptanalyze.

# Running

run `cargo test` to run test and `cargo run --release` to run the cryptanalysis (in src/bin/jupiter.rs). It's important to use `--release` as otherwise it will be build a debug build and be very slow.

xoxo

# Potential improvments:

- `perf` seems to show that permute is a significant portion of the runtime, so maybe we should speed it up.
- https://stackoverflow.com/questions/43575633/fast-bit-permutation
- In general, the bit operation LUTs in this are design for 1-indexed left-to-right binary representations. Mordern software works with 0-indexed right-to-left. This makes everything awkward. It would be nice to transform the LUTs before compilation instead of transforming the indexes at runtime.
- make asserts debug_asserts.
- opt-level=3


## Notes (perosnal!!)
Current best runtime 3.885 seconds
remove asserts (and losing some of the type guarentees) makes it run at 2.765 seconds. about 1.1 seconds less. 28% decrease. Small potatoes

Made expansion fast. now it's 1.846 sec with no other optimization. :sunglasses:
