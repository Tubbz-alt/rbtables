
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

  fn find_plaintext(&'a self, hash : &[u8], table : RainbowTable) -> Option<&'a str> {
    let rf_len = self.reduction_functions.len();
    for rfn in (1..rf_len).rev() {

    }
    None
  }
}
