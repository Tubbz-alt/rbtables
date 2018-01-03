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

  fn new(hashing_function : fn(&str) -> String, reduction_functions : Vec<fn(&str) -> String>) -> RainbowTable {
    RainbowTable {
      chains: HashMap::new(),
      hashing_function: hashing_function,
      reduction_functions: reduction_functions
    }
  }

  fn add_seed<T: AsRef<str>>(&mut self, seed : T) -> &mut RainbowTable {
    let wrapper = vec![seed];
    self.add_seeds(&wrapper)
  }

  fn add_seeds<T: AsRef<str>>(&mut self, seeds : &[T]) -> &mut RainbowTable {
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

  fn find_plaintext_single(&self, hash : &str) -> Option<String> {
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

  fn find_plaintext_multi(&self, hash : &str, num_threads : usize) -> Option<String>  {
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

  fn find_plaintext(&self, hash : &str) -> Option<String> {
    let cores = num_cpus::get();
    self.find_plaintext_multi(hash, cores - 1)
  }
}

#[cfg(test)]
mod tests {

    extern crate md5;

    use analysis::RainbowTable;

    fn md5_hashing_function(plaintext : &str) -> String {
      let digest = md5::compute(plaintext.as_bytes());
      format!("{:x}", digest)
    }

    fn simple_reduction_function(hash : &str) -> String {
      String::from(&hash[..5])
    }

    fn simple_reduction_function2(hash : &str) -> String {
      String::from(&hash[..6])
    }

    fn build_rainbow_table() -> RainbowTable {
      let mut rfs : Vec<fn(&str) -> String> = Vec::new();
      rfs.push(simple_reduction_function);
      for _ in 0..100 {
        rfs.push(simple_reduction_function2);
      }

      let seeds = vec!["test", "monster", "test2", "amazing"];

      let mut table : RainbowTable = RainbowTable::new(md5_hashing_function, rfs);
      table.add_seeds(&seeds);
      table
    }

    #[test]
    fn execute_rainbow_table_single() {
      let table = build_rainbow_table();
      assert_eq!(Some(String::from("monster")), table.find_plaintext_single("8bf4e6addd72a9c4c4714708d2941528"));
      assert_eq!(Some(String::from("8bf4e")), table.find_plaintext_single("18b11cf86b4a3fd75e3fd9ac3485bdb6"));
    }

    #[test]
    fn execute_rainbow_table_multi() {
      let table = build_rainbow_table();
      assert_eq!(Some(String::from("monster")), table.find_plaintext_multi("8bf4e6addd72a9c4c4714708d2941528", 2));
      assert_eq!(Some(String::from("8bf4e")), table.find_plaintext_multi("18b11cf86b4a3fd75e3fd9ac3485bdb6", 2));
    }

    #[test]
    fn execute_rainbow_table_system() {
      let table = build_rainbow_table();
      assert_eq!(Some(String::from("monster")), table.find_plaintext("8bf4e6addd72a9c4c4714708d2941528"));
      assert_eq!(Some(String::from("8bf4e")), table.find_plaintext("18b11cf86b4a3fd75e3fd9ac3485bdb6"));
    }

}