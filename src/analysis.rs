
pub struct RainbowTable<'a> {
  chain_heads: Vec<&'a str>,
  chain_tails: Vec<&'a str>,
  hashing_function: fn(&str) -> &'a [u8],
  reduction_functions: Vec<fn(&[u8]) -> &'a str>
}

impl<'a> RainbowTable<'a> {

  fn new(hashing_function : fn(&str) -> &'a [u8], reduction_functions : Vec<fn(&[u8]) -> &'a str>) -> RainbowTable<'a> {
    RainbowTable {
      chain_heads: Vec::new(),
      chain_tails: Vec::new(),
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
      if !self.chain_heads.contains(seed) {
        self.chain_heads.push(seed);
        self.chain_tails.push(seed);
        let n = self.chain_tails.len() - 1;
        for reduction_function in &self.reduction_functions {
          let next_value = (reduction_function)((self.hashing_function)(self.chain_tails[n]));
          self.chain_tails[n] = next_value;
        }
      }
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
      if self.chain_tails.contains(&current_plaintext) {
        let chain_index = self.chain_tails.iter().position(|&t| t == current_plaintext).unwrap();
        let mut chain_plaintext = self.chain_heads[chain_index];
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
