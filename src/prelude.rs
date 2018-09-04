extern crate crossbeam;
extern crate num_cpus;
extern crate serde_json;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};

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

    pub fn reconstruct(&mut self, old_chains : &HashMap<String, String>) -> &mut RainbowTable<H, R> {
        for (key, value) in old_chains {
            self.chains.insert(key.to_owned(), value.to_owned());
        }
        self
    }

    pub fn add_seed<T: AsRef<str>>(&mut self, seed : T) -> &mut RainbowTable<H, R> {
        let wrapper = vec![seed];
        self.add_seeds(&wrapper)
    }

    pub fn add_seeds<T: AsRef<str>>(&mut self, seeds : &[T]) -> &mut RainbowTable<H, R> {
        for seed in seeds {
            let mut value = String::from(seed.as_ref());
            for reducer in &self.reducers {
                let hash = self.hasher.digest(&value);
                value = reducer.reduce(&hash);
            }
            self.chains.insert(value, String::from(seed.as_ref()));
        }
        self
    }

    fn _find_plaintext_single(&self, hash : &str) -> Option<String> {
        let rf_len = self.reducers.len();

        for idx in (0..rf_len).rev() {
            // Apply reduction functions successively starting at a given index 
            let mut reduced_plaintext = String::from("");
            let mut reduced_hash = String::from(hash);
            for rf_idx in idx..rf_len {
                reduced_plaintext = self.reducers[rf_idx].reduce(&reduced_hash[..]);
                reduced_hash = self.hasher.digest(&reduced_plaintext[..]);
            }

            // Chain contains our target plaintext if reduced plaintext is in the final column
            if self.chains.contains_key(&reduced_plaintext) {
                let mut target_plaintext = String::from(&self.chains.get(&reduced_plaintext).unwrap()[..]);
                let mut target_hash = self.hasher.digest(&target_plaintext[..]);

                // Recreate the chain until we reach the the plaintext that produced our original hash
                for rf_idx in 0..rf_len {
                    if target_hash == hash {
                        break;
                    }
                    target_plaintext = self.reducers[rf_idx].reduce(&target_hash[..]);
                    target_hash = self.hasher.digest(&target_plaintext[..]);
                }

                // We return the final plaintext
                if target_hash == hash {
                    return Some(target_plaintext);
                }
            }
        }

        None
    }

    fn _find_plaintext_multi(&self, hash : &str, num_threads : usize) -> Option<String>  {
        assert!(num_threads > 0);

        // Defer to single-threaded variant if thread count is set to 1
        if num_threads == 1 {
            return self._find_plaintext_single(hash);
        }

        crossbeam::scope(|scope| {
            let (tx, rx) = mpsc::channel();
            let rf_len : usize = self.reducers.len();
            let halt_counter = Arc::new(Mutex::new(false));

            for thread_idx in 0..num_threads {
                let tx = mpsc::Sender::clone(&tx);
                let halt_counter = halt_counter.clone();

                scope.spawn(move || {
                    // Set up some initial counters
                    let mut idx = thread_idx;
                    let mut sent = false; 

                    while idx < rf_len {
                        // Check if work has been halted because one thread was successful
                        {
                            let halt = halt_counter.lock().unwrap();
                            if *halt {
                                break;
                            }
                        }

                        // Apply reduction functions successively starting at a given index 
                        let mut reduced_plaintext = String::from("");
                        let mut reduced_hash = String::from(hash);
                        for rf_idx in idx..rf_len {
                            reduced_plaintext = self.reducers[rf_idx].reduce(&reduced_hash);
                            reduced_hash = self.hasher.digest(&reduced_plaintext);
                        }

                        // Chain contains our target plaintext if reduced plaintext is in the final column
                        if self.chains.contains_key(&reduced_plaintext) {
                            let mut target_plaintext = String::from(&self.chains.get(&reduced_plaintext).unwrap()[..]);
                            let mut target_hash = self.hasher.digest(&target_plaintext);

                            // Recreate the chain until we reach the the plaintext that produced our original hash
                            for rf_idx in 0..rf_len {
                                if target_hash == hash {
                                    break;
                                }
                                target_plaintext = self.reducers[rf_idx].reduce(&target_hash);
                                target_hash = self.hasher.digest(&target_plaintext);
                            }

                            // Post the final plaintext to the channel if found
                            if target_hash == hash {
                                tx.send(Some(target_plaintext)).unwrap();
                                sent = true;
                                break;
                            }
                        }

                        idx += num_threads;
                    }

                    // The thread was unsuccessful: post an empty work result
                    if !sent {
                        tx.send(None).unwrap();
                    }
                });
            }

            // This is a globally tracked result that will be returned on completion
            let mut result = None;

            // We will receive {num_threads} results from each thread
            for _ in 0..num_threads {
                match rx.recv() {
                    Ok(thread_result) => {
                        // One thread was successful
                        if thread_result.is_some() {
                            // Set global result 
                            result = thread_result; 

                            // Halt all other child threads 
                            let mut halt = halt_counter.lock().unwrap();
                            *halt = true;
                        }
                    },
                    _ => ()
                }
            }

            result 
        })
    }

    pub fn find_plaintext(&self, hash : &str) -> Option<String> {
        self._find_plaintext_multi(hash, num_cpus::get())
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(&self.chains).ok().unwrap()
    }
}
