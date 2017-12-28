
pub struct Chain<'a> {
  start: &'a str,
  end: &'a str
}

pub struct Table<'a> {
  chains: Vec<Chain<'a>>,
  hashing_function: fn(&str) -> u8,
  reduction_functions: Vec<fn(u8) -> &'a str>
}

impl<'a> Table<'a> {

  fn new(hashing_function : fn(str) -> u8, reduction_functions : Vec<fn(u8) -> str>) -> Table<'a> {
    Table {
      chains: Vec::new(),
      hashing_function: hashing_function,
      reduction_functions: reduction_functions
    }
  }

  fn add_seed(&mut self, seed : &str) -> &'a mut Table {
    let chain_end = "";
    for reduction_function in self.reduction_functions {
      chain_end = (reduction_function)((self.hashing_function)(seed))
    }
    let chain = Chain { start: seed, end: chain_end };
    self.chains.push(chain);
    self
  }

  fn add_seeds(&mut self, seeds : &[String]) -> &'a mut Table {
    for seed in seeds {
      self.add_seed(seed);
    }
    self
  }
}
