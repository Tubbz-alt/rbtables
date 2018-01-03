extern crate crossbeam;
extern crate num_cpus;

use std::collections::HashMap;
use std::sync::mpsc;

// The generic 'F' can represent either plaintext -> hash OR hash -> plaintext
// wherein any given hash is in string format.
pub struct RainbowTable {
  chains: HashMap<String, String>,
  hashing_function: fn(&str) -> String,
  reduction_functions: Vec<fn(&str) -> String>
}

impl RainbowTable {

  pub fn new(hashing_function : fn(&str) -> String, reduction_functions : Vec<fn(&str) -> String>) -> RainbowTable {
    RainbowTable {
      chains: HashMap::new(),
      hashing_function: hashing_function,
      reduction_functions: reduction_functions
    }
  }

  pub fn add_seed<T: AsRef<str>>(&mut self, seed : T) -> &mut RainbowTable {
    let wrapper = vec![seed];
    self.add_seeds(&wrapper)
  }

  pub fn add_seeds<T: AsRef<str>>(&mut self, seeds : &[T]) -> &mut RainbowTable {
    for seed in seeds {
      let mut next_value = String::from(seed.as_ref());
      for reduction_function in &self.reduction_functions {
        let next_value_hash = (self.hashing_function)(&next_value[..]);
        next_value = (reduction_function)(&next_value_hash[..]);
      }
      self.chains.insert(next_value, String::from(seed.as_ref()));
    }
    self
  }

  pub fn find_plaintext_single(&self, hash : &str) -> Option<String> {
    let rf_len = self.reduction_functions.len();

    for current_playback in (0..rf_len).rev() {

      // Apply reduction functions successively starting at the index of current_playback
      // This will produce some plaintext that could be in the last row of our table
      let mut reduced_plaintext = String::from("");
      let mut reduced_hash = String::from(hash);
      for reduction_step in current_playback..rf_len {
        reduced_plaintext = (self.reduction_functions[reduction_step])(&reduced_hash[..]);
        reduced_hash = (self.hashing_function)(&reduced_plaintext[..]);
      }

      // If the reduced plaintext is in the final column of the table (is a key in our hashmap),
      // we now know that we have a chain that contains our target plaintext
      // because earlier we reconstructed a segment of the chain
      if self.chains.contains_key(&reduced_plaintext) {

        // Remember that target_hash will always be the hash of the target plaintext so it is
        // one step 'ahead' of the target_plaintext in the chain.
        let mut target_plaintext = String::from(&self.chains.get(&reduced_plaintext).unwrap()[..]);
        let mut target_hash = (self.hashing_function)(&target_plaintext[..]);

        // Recreate the chain and apply each reduction function in order until we reach the
        // the plaintext that produced our original hash
        for reduction_step in 0..rf_len {
          if target_hash == hash {
            break;
          }
          target_plaintext = (self.reduction_functions[reduction_step])(&target_hash[..]);
          target_hash = (self.hashing_function)(&target_plaintext[..]);
        }

        // We return the final plaintext
        return Some(target_plaintext);
      }
    }

    // No plaintext for the hash was found in the table
    None
  }

  pub fn find_plaintext_multi(&self, hash : &str, num_threads : usize) -> Option<String>  {
    assert!(num_threads > 1);

    let rf_len : usize = self.reduction_functions.len();
    // The channels are used by the threads to send work results
    // If the thread sends an empty string as its result, then it has found nothing
    let (tx, rx) = mpsc::channel();

    for t in 0..num_threads {
      let job_queue : Vec<usize> = (0..rf_len).filter(|&j| j % num_threads == t).collect();

      let tx = mpsc::Sender::clone(&tx);

      // We use crossbeam rather than the native threading library
      // because crossbeam supports scoped threads that can use their parent's stack values
      crossbeam::scope(|scope| {
        scope.spawn(move || {
          for current_playback in job_queue {
            let mut reduced_plaintext = String::from("");
            let mut reduced_hash = String::from(hash);
            for i in current_playback..rf_len {
              reduced_plaintext = (self.reduction_functions[i])(&reduced_hash[..]);
              reduced_hash = (self.hashing_function)(&reduced_plaintext[..]);
            }

            if self.chains.contains_key(&reduced_plaintext) {
              let mut target_plaintext = String::from(&self.chains.get(&reduced_plaintext).unwrap()[..]);
              let mut target_hash = (self.hashing_function)(&target_plaintext[..]);

              for reduction_step in 0..rf_len {
                if target_hash == hash {
                  break;
                }
                target_plaintext = (self.reduction_functions[reduction_step])(&target_hash[..]);
                target_hash = (self.hashing_function)(&target_plaintext[..]);
              }

              // Found something, post it to the channel
              tx.send(target_plaintext).unwrap();
            }
          }

          // No results found, post empty result to signal that the thread has exited
          tx.send(String::from("")).unwrap();

        }).join();
      });
    }

    for _ in 0..num_threads {
      let work_result = rx.recv().unwrap();
      if work_result != "" {
        return Some(work_result);
      }
    }

    None
  }

  pub fn find_plaintext(&self, hash : &str) -> Option<String> {
    let cores = num_cpus::get();
    self.find_plaintext_multi(hash, cores - 1)
  }
}