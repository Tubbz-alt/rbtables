
use std::collections::HashMap;

pub struct RainbowTable<'a> {
  chains: HashMap<&'a str, &'a str>,
  hashing_function: fn(&str) -> u8,
  reduction_functions: Vec<fn(u8) -> &'a str>
}

impl<'a> RainbowTable<'a> {

  fn new(hashing_function : fn(&str) -> u8, reduction_functions : Vec<fn(u8) -> &'a str>) -> RainbowTable<'a> {
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
      if !self.chains.contains_key(seed) {
        self.chains.insert(seed, seed);
        for reduction_function in &self.reduction_functions {
          let next_value = (reduction_function)((self.hashing_function)(self.chains.get(seed).unwrap()));
          self.chains.insert(seed, &next_value);
        }
      }
    }
    self
  }
}

fn find_plaintext(hash : u8, table : RainbowTable) -> Option<&str> {
  None
}
