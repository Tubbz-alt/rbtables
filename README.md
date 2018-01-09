# rbtables

__rbtables__ is a fast, parallel implementation of [rainbow tables](https://en.wikipedia.org/wiki/Rainbow_table) in Rust. It is intended as an API to support general use cases of rainbow tables. The user will need to supply hashing and reduction functions.

## Usage

Begin by implementing the `Hasher` trait which contains a single function `digest(&self, plaintext : &str) -> String`. The `Hasher` trait is a wrapper around any hashing function that allows data to be transferred between multiple uses of the `digest` function.

In `lib.rs`, you can see an example of this:

```rust
// Represents a hasher that performs the md5 digest
struct MD5Hasher;

impl MD5Hasher {

  fn new() -> MD5Hasher {
    MD5Hasher
  }

}

impl Hasher for MD5Hasher {

  fn digest(&self, plaintext : &str) -> String {
    let digest = md5::compute(plaintext.as_bytes());
    format!("{:x}", digest)
  }

}
```

Next, implement a set of reduction function(s). To do this, start by implementing the `Reducer` trait. You must implement a single function `reduce(&self, hash : &str) -> String`. The `Reducer` trait is a wrapper around any reduction function, similar to the `Hasher` trait.

In `lib.rs`, you can see an example of this:

```rust
// Represents a reducer that simply takes the first n characters of a hash to reduce it
struct SubstringReducer {
  n: usize
}

impl SubstringReducer {

  fn new(n : usize) -> SubstringReducer {
    SubstringReducer {
      n: n
    }
  }

}

impl Reducer for SubstringReducer {

  fn reduce(&self, hash : &str) -> String {
    String::from(&hash[..self.n])
  }

}
```

See the fn `build_sample_rainbow_table() -> RainbowTable<MD5Hasher, SubstringReducer>` in `lib.rs` for example usage of these traits.