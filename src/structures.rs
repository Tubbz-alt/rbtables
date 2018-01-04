extern crate crossbeam;
extern crate num_cpus;
extern crate serde_json;

use std::collections::HashMap;
use std::sync::mpsc;

pub trait Hasher : Sync {

  fn digest(&self, plaintext : &str) -> String;

}

pub trait Reducer : Sync {

  fn reduce(&self, hash : &str) -> String;

}

pub struct RainbowTable<H, R> where H : Hasher, R : Reducer {
  chains: HashMap<String, String>,
  hasher: H,
  reducers: Vec<R>
}

impl<H, R> RainbowTable<H, R> where H : Hasher, R : Reducer {

  pub fn new(hasher: H, reducers : Vec<R>) -> RainbowTable<H, R> {
    RainbowTable {
      chains: HashMap::new(),
      hasher: hasher,
      reducers: reducers
    }
  }

  pub fn add_seed<T: AsRef<str>>(&mut self, seed : T) -> &mut RainbowTable<H, R> {
    let wrapper = vec![seed];
    self.add_seeds(&wrapper)
  }

  pub fn add_seeds<T: AsRef<str>>(&mut self, seeds : &[T]) -> &mut RainbowTable<H, R> {
    for seed in seeds {
      let mut next_value = String::from(seed.as_ref());
      for reducer in &self.reducers {
        let next_value_hash = self.hasher.digest(&next_value[..]);
        next_value = reducer.reduce(&next_value_hash[..]);
      }
      self.chains.insert(next_value, String::from(seed.as_ref()));
    }
    self
  }

  pub fn find_plaintext_single(&self, hash : &str) -> Option<String> {
    let rf_len = self.reducers.len();

    for current_playback in (0..rf_len).rev() {

      // Apply reduction functions successively starting at the index of current_playback
      // This will produce some plaintext that could be in the last row of our table
      let mut reduced_plaintext = String::from("");
      let mut reduced_hash = String::from(hash);
      for reduction_step in current_playback..rf_len {
        reduced_plaintext = self.reducers[reduction_step].reduce(&reduced_hash[..]);
        reduced_hash = self.hasher.digest(&reduced_plaintext[..]);
      }

      // If the reduced plaintext is in the final column of the table (is a key in our hashmap),
      // we now know that we have a chain that contains our target plaintext
      // because earlier we reconstructed a segment of the chain
      if self.chains.contains_key(&reduced_plaintext) {

        // Remember that target_hash will always be the hash of the target plaintext so it is
        // one step 'ahead' of the target_plaintext in the chain.
        let mut target_plaintext = String::from(&self.chains.get(&reduced_plaintext).unwrap()[..]);
        let mut target_hash = self.hasher.digest(&target_plaintext[..]);

        // Recreate the chain and apply each reduction function in order until we reach the
        // the plaintext that produced our original hash
        for reduction_step in 0..rf_len {
          if target_hash == hash {
            break;
          }
          target_plaintext = self.reducers[reduction_step].reduce(&target_hash[..]);
          target_hash = self.hasher.digest(&target_plaintext[..]);
        }

        // We return the final plaintext
        // Keep in mind that we can get false positives in the lookup if the rainbow table either ...
        //   1. has bad reductions functions
        //   2. is treated like a hash table, with multiple copies of the same reduction function
        if target_hash == hash {
          return Some(target_plaintext);
        }
      }
    }

    // No plaintext for the hash was found in the table
    None
  }

  pub fn find_plaintext_multi(&self, hash : &str, num_threads : usize) -> Option<String>  {
    assert!(num_threads > 1);

    let rf_len : usize = self.reducers.len();
    // The channels are used by the threads to send work results
    // If the thread sends an empty string as its result, then it has found nothing
    let (tx, rx) = mpsc::channel();

    // We use crossbeam rather than the native threading library
    // because crossbeam supports scoped threads that can use their parent's stack values
    crossbeam::scope(|scope| {

      for t in 0..num_threads {
        let job_queue : Vec<usize> = (0..rf_len).filter(|&j| j % num_threads == t).rev().collect();
        let tx = mpsc::Sender::clone(&tx);

        scope.spawn(move || {

          for current_playback in job_queue {
            let mut reduced_plaintext = String::from("");
            let mut reduced_hash = String::from(hash);
            for i in current_playback..rf_len {
              reduced_plaintext = self.reducers[i].reduce(&reduced_hash[..]);
              reduced_hash = self.hasher.digest(&reduced_plaintext[..]);
            }

            if self.chains.contains_key(&reduced_plaintext) {
              let mut target_plaintext = String::from(&self.chains.get(&reduced_plaintext).unwrap()[..]);
              let mut target_hash = self.hasher.digest(&target_plaintext[..]);

              for reduction_step in 0..rf_len {
                if target_hash == hash {
                  break;
                }
                target_plaintext = self.reducers[reduction_step].reduce(&target_hash[..]);
                target_hash = self.hasher.digest(&target_plaintext[..]);
              }

              // Found something, post it to the channel
              if target_hash == hash {
                tx.send(target_plaintext).unwrap();
              }
            }
          }

          // No results found, post empty result to signal that the thread has exited
          tx.send(String::from("")).unwrap();

        });
      }
    });

    for _ in 0..num_threads {
      let work_result = rx.recv().unwrap();
      // If we receive a non-empty work result, end child threads early by returning
      if work_result != "" {
        return Some(work_result);
      }
    }

    None
  }

  pub fn find_plaintext(&self, hash : &str) -> Option<String> {
    self.find_plaintext_multi(hash, num_cpus::get())
  }

  pub fn to_json_string(&self) -> String {
    serde_json::to_string(&self.chains).ok().unwrap()
  }
}