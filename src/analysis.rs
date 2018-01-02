
use std::collections::HashMap;
use std::sync::{Arc, RwLock, mpsc};
use std::thread;

pub struct RainbowTable {
  chains: HashMap<String, String>,
  hashing_function: fn(&str) -> &[u8],
  reduction_functions: Vec<fn(&[u8]) -> &str>
}

impl RainbowTable {

  fn new(hashing_function : fn(&str) -> &[u8], reduction_functions : Vec<fn(&[u8]) -> &str>) -> RainbowTable {
    RainbowTable {
      chains: HashMap::new(),
      hashing_function: hashing_function,
      reduction_functions: reduction_functions
    }
  }

  fn add_seed(&mut self, seed : &str) -> &mut RainbowTable {
    let wrapper = vec![seed];
    self.add_seeds(&wrapper)
  }

  fn add_seeds(&mut self, seeds : &[&str]) -> &mut RainbowTable {
    for seed in seeds {
      let mut next_value = seed as &str;
      for reduction_function in &self.reduction_functions {
        next_value = (reduction_function)((self.hashing_function)(next_value));
      }
      self.chains.insert(next_value.to_string(), seed.to_string());
    }
    self
  }

  fn find_plaintext(&self, hash : &[u8]) -> Option<&str> {
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

  fn find_plaintext_threaded(&self, hash : &[u8], num_threads : u8) -> Option<&str>  {
    assert!(num_threads > 1);

    let rfs = &self.reduction_functions;
    let hf = &self.hashing_function;

    let rf_len : usize = self.reduction_functions.len();
    let rf_playbacks : Vec<usize> = (0..rf_len).collect();
    let counter = Arc::new(RwLock::new(0));
    // let (tx, rx) = mpsc::channel();

    for _ in 0..num_threads {
      let counter = Arc::clone(&counter);
      let _ = thread::spawn(move || {
        let current_playback = counter.read().unwrap();
        while *current_playback < rf_len {
          /*
          let mut current_plaintext = "";
          let mut current_hash = hash;
          for j in *current_playback..rf_len {
            current_plaintext = (rfs[j])(current_hash);
            current_hash = (hf)(current_plaintext);
          }
          */

          let mut adjust_playback = counter.write().unwrap();
          *adjust_playback += 1;
        }
      }).join();
    }

    None
  }
}
