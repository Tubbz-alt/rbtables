# rbtables

rbtables is a fast, lightweight, and extensible implementation of [rainbow tables](https://en.wikipedia.org/wiki/Rainbow_table) in Rust. 
It is intended as an API to support general use cases of rainbow tables. 
The user will need to supply hashing and reduction functions.

## Usage

Begin by implementing the `Hasher` trait containing the function `digest(&self, plaintext : &str) -> String`. 
This function accepts an arbitrary plaintext string and should produce a hexidecimal-encoded digest string.
For example, this example produces the hex encoding of a plaintext's MD5 hash:

```rust
use rbtables::prelude::Hasher;

struct MD5Hasher;
impl Hasher for MD5Hasher {
  fn digest(&self, plaintext : &str) -> String {
    format!("{:x}", md5::compute(plaintext.as_bytes()))
  }
}
```

Next, you will need to create a set of reduction function(s) by implementing the `Reducer` trait. 
You must implement the function `reduce(&self, hash : &str) -> String`, which accepts the output of your hasher and should produce another plaintext string.
A trivial example involves taking the first `n` characters from the hex encoding of the hash:

```rust
use rbtables::prelude::Reducer;

struct SubstringReducer {
  n: usize
}
impl Reducer for SubstringReducer {

  fn reduce(&self, hash : &str) -> String {
    String::from(&hash[..self.n])
  }

}
```

After that, you can build a rainbow table by supplying the hasher and a vector of reduction functions. 
The rainbow table will need to be supplied with seed values, which will determine the effectiveness of your table along with the reduction functions. 

See the crates.io documentation for additional information.
