extern crate crossbeam;
extern crate num_cpus;

use std::collections::HashMap;
use std::sync::{Arc, RwLock, mpsc};

// The generic 'F' can represent either plaintext -> hash OR hash -> plaintext
// wherein any given hash is in string format.
pub struct RainbowTable<F> where F : Fn(&str) -> &str + Sync {
  chains: HashMap<String, String>,
  hashing_function: F,
  reduction_functions: Vec<F>
}

impl<F> RainbowTable<F> where F : Fn(&str) -> &str + Sync {

  fn new(hashing_function : F, reduction_functions : Vec<F>) -> RainbowTable<F> {
    RainbowTable {
      chains: HashMap::new(),
      hashing_function: hashing_function,
      reduction_functions: reduction_functions
    }
  }

  fn add_seed<T: AsRef<str>>(&mut self, seed : T) -> &mut RainbowTable<F> {
    let wrapper = vec![seed];
    self.add_seeds(&wrapper)
  }

  fn add_seeds<T: AsRef<str>>(&mut self, seeds : &[T]) -> &mut RainbowTable<F> {
    for seed in seeds {
      let mut next_value = seed.as_ref();
      for reduction_function in &self.reduction_functions {
        let next_value_hash = (self.hashing_function)(next_value);
        next_value = (reduction_function)(next_value_hash);
      }
      self.chains.insert(next_value.to_string(), seed.as_ref().to_string());
    }
    self
  }

  fn find_plaintext_single(&self, hash : &str) -> Option<&str> {
    let rf_len = self.reduction_functions.len();

    for current_playback in (0..rf_len).rev() {

      // Apply reduction functions successively starting at the index of current_playback
      // This will produce some plaintext that could be in the last row of our table
      let mut reduced_plaintext = "";
      let mut reduced_hash = hash;
      for reduction_step in current_playback..rf_len {
        reduced_plaintext = (self.reduction_functions[reduction_step])(reduced_hash);
        reduced_hash = (self.hashing_function)(reduced_plaintext);
      }

      // If the reduced plaintext is in the final column of the table (is a key in our hashmap),
      // we now know that we have a chain that contains our target plaintext
      // because earlier we reconstructed a segment of the chain
      if self.chains.contains_key(reduced_plaintext) {

        // Remember that target_hash will always be the hash of the target plaintext so it is
        // one step 'ahead' of the target_plaintext in the chain.
        let mut target_plaintext = &self.chains.get(reduced_plaintext).unwrap()[..];
        let mut target_hash = (self.hashing_function)(target_plaintext);

        // Recreate the chain and apply each reduction function in order until we reach the
        // the plaintext that produced our original hash
        for reduction_step in 0..rf_len {
          if target_hash == hash {
            break;
          }
          target_plaintext = (self.reduction_functions[reduction_step])(target_hash);
          target_hash = (self.hashing_function)(target_plaintext);
        }

        // We return the final plaintext
        return Some(target_plaintext);
      }
    }

    // No plaintext for the hash was found in the table
    None
  }

  fn find_plaintext_multi(&self, hash : &str, num_threads : usize) -> Option<&str>  {
    assert!(num_threads > 1);

    let rf_len : usize = self.reduction_functions.len();
    // The counter determines at which reduction function the thread will be starting from
    let counter = Arc::new(RwLock::new(0));
    // The channels are used by the threads to send work results
    // If the thread sends an empty string as its result, then it has found nothing
    let (tx, rx) = mpsc::channel();

    for _ in 0..num_threads {
      let tx = mpsc::Sender::clone(&tx);
      let counter = Arc::clone(&counter);

      // We use crossbeam rather than the native threading library
      // because crossbeam supports scoped threads that can use their parent's stack values
      crossbeam::scope(|scope| {
        scope.spawn(move || {
          let counter_reader = counter.read().unwrap();
          let counter_value = *counter_reader;
          let current_playback = rf_len - counter_value - 1;

          // The inner loop is modified from the fn find_plaintext_simple
          // except that the results are posted to the channel rather than immediately returned
          while counter_value < rf_len {
            let mut reduced_plaintext = "";
            let mut reduced_hash = hash;
            for i in current_playback..rf_len {
              reduced_plaintext = (self.reduction_functions[i])(reduced_hash);
              reduced_hash = (self.hashing_function)(reduced_plaintext);
            }

            if self.chains.contains_key(reduced_plaintext) {
              let mut target_plaintext = &self.chains.get(reduced_plaintext).unwrap()[..];
              let mut target_hash = (self.hashing_function)(target_plaintext);

              for reduction_step in 0..rf_len {
                if target_hash == hash {
                  break;
                }
                target_plaintext = (self.reduction_functions[reduction_step])(target_hash);
                target_hash = (self.hashing_function)(target_plaintext);
              }

              // Found something, post it to the channel
              tx.send(target_plaintext).unwrap();
            }

            let mut adjust_playback = counter.write().unwrap();
            *adjust_playback += 1;
          }

          // No results found, post empty result to signal that the thread has exited
          tx.send("").unwrap();

        }).join();
      });
    }

    for _ in 0..num_threads {
      let work_result = rx.recv().unwrap();
      if work_result != "" {

        // Try to end threads gracefully by preventing them from processing any future chains
        let mut end_threads = counter.write().unwrap();
        *end_threads = rf_len;

        return Some(work_result);
      }
    }

    None
  }

  fn find_plaintext(&self, hash : &str) -> Option<&str> {
    let cores = num_cpus::get();
    self.find_plaintext_multi(hash, cores - 1)
  }
}

#[cfg(test)]
mod tests {

    fn md5_hashing_function(plaintext : &str) -> &str {

    }

    fn simple_reduction_function(hash : &str) -> &str {

    }

    fn simple_reduction_function2(hash : &str) -> &str{

    }

    #[test]
    fn build_rainbow_table() {

    }

    #[test]
    fn execute_rainbow_table_single() {

    }

    #[test]
    fn execute_rainbow_table_multi() {

    }

    #[test]
    fn execute_rainbow_table_system() {

    }

}