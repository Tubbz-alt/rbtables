
use std::collections::HashMap;

pub struct RainbowTable<'a> {
  chains: HashMap<String, String>,
  hashing_function: fn(&str) -> &'a [u8],
  reduction_functions: Vec<fn(&[u8]) -> &'a str>
}

impl<'a> RainbowTable<'a> {

  fn new(hashing_function : fn(&str) -> &'a [u8], reduction_functions : Vec<fn(&[u8]) -> &'a str>) -> RainbowTable<'a> {
    RainbowTable {
      chains: HashMap::new(),
      hashing_function: hashing_function,
      reduction_functions: reduction_functions
    }
  }

  fn add_seed(&'a mut self, seed : &'a str) -> &'a mut RainbowTable {
    let wrapper = vec![seed];
    self.add_seeds(&wrapper)
  }

  fn add_seeds(&'a mut self, seeds : &[&'a str]) -> &'a mut RainbowTable {
    for seed in seeds {
      let mut next_value = seed as &str;
      for reduction_function in &self.reduction_functions {
        next_value = (reduction_function)((self.hashing_function)(next_value));
      }
      self.chains.insert(next_value.to_string(), seed.to_string());
    }
    self
  }

  fn find_plaintext(&'a self, hash : &[u8]) -> Option<&'a str> {
    let rf_len = self.reduction_functions.len();
    let mut current_plaintext = "";
    let mut current_hash = hash;
    for i in (0..rf_len).rev() {
      for j in i..rf_len {
        current_plaintext = (self.reduction_functions[j])(current_hash);
        current_hash = (self.hashing_function)(current_plaintext);
      }
      if self.chains.contains_key(current_plaintext) {
        let mut chain_plaintext = &self.chains.get(current_plaintext).unwrap()[..];
        let mut chain_hash = (self.hashing_function)(chain_plaintext);
        let mut n = 0;
        while chain_hash != hash {
          chain_plaintext = (self.reduction_functions[n])(chain_hash);
          chain_hash = (self.hashing_function)(chain_plaintext);
          n += 1;
        }
        return Some(chain_plaintext);
      }
      current_hash = hash;
    }
    None
  }
}
