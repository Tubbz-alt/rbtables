pub mod structures;

#[cfg(test)]
mod tests {

  extern crate md5;
  extern crate num_cpus;

  use structures::RainbowTable;
  use structures::Hasher;
  use structures::Reducer;

  // Represents a hasher that performs the md5 digest
  struct Md5Hasher;

  impl Md5Hasher {

    fn new() -> Md5Hasher {
      Md5Hasher
    }

  }

  impl Hasher for Md5Hasher {

    fn digest(&self, plaintext : &str) -> String {
      let digest = md5::compute(plaintext.as_bytes());
      format!("{:x}", digest)
    }

  }

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

  fn build_sample_rainbow_table() -> RainbowTable<Md5Hasher, SubstringReducer> {
    let mut rfs : Vec<SubstringReducer> = Vec::new();
    for i in 6..27 {
      rfs.push(SubstringReducer::new(i));
    }

    let mut table : RainbowTable<Md5Hasher, SubstringReducer> = RainbowTable::new(Md5Hasher::new(), rfs);
    table.add_seed("monster");
    for i in 0..999 {
      let seed = format!("{}", i);
      table.add_seed(seed);
    }

    table
  }

  #[test]
  fn execute_rainbow_table_system() {
    let table = build_sample_rainbow_table();
    assert_eq!(Some(String::from("monster")), table.find_plaintext("8bf4e6addd72a9c4c4714708d2941528"));
    assert_eq!(Some(String::from("8bf4e6")), table.find_plaintext("8a1fb399dcf220db935995abce6a1532"));
  }
}

