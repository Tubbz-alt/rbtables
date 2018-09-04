#[macro_use]
extern crate lazy_static;

// Silences warnings about unused macros
lazy_static!{}

pub mod prelude;

#[cfg(test)]
mod tests {
    extern crate md5;
    extern crate rand;

    use prelude::{Hasher, Reducer, RainbowTable};
    use self::rand::{thread_rng, Rng};

    // This hasher performs an MD5 digest
    struct MD5Hasher;
    impl Hasher for MD5Hasher {
        fn digest(&self, plaintext : &str) -> String {
            format!("{:x}", md5::compute(plaintext.as_bytes()))
        }
    }

    // This reducer simply takes the first n characters of a hash 
    struct SubstringReducer {
        n: usize
    }
    impl Reducer for SubstringReducer {
        fn reduce(&self, hash : &str) -> String {
            String::from(&hash[..self.n])
        }
    }

    lazy_static! {
        static ref TABLE : RainbowTable<MD5Hasher, SubstringReducer> = {
            // Build a list of substring reducers, each of which will be applied in succession 
            let mut rfs : Vec<SubstringReducer> = Vec::new();
            for i in 6..27 {
                rfs.push(SubstringReducer { n : i });
            }

            // Create the table using an MD5 hasher and the list of substring reducers
            let mut table : RainbowTable<MD5Hasher, SubstringReducer> = RainbowTable::new(MD5Hasher, rfs);

            // Add numbers in [0, 999] as test seeds 
            for i in 0..1000 {
                table.add_seed(format!("{}", i));
            }

            table
        };
    }

    #[test]
    fn test_rainbow_table_find_seeds() {
        // Test that the rainbow table can find the hashes of the seed values
        for i in 0..1000 {
            let seed = format!("{}", i);
            let seed_hash = format!("{:x}", md5::compute(seed.as_bytes()));
            assert_eq!(Some(seed), TABLE.find_plaintext(&seed_hash));
        }
    }

    #[test]
    fn test_rainbow_table_find_random() {
        // Build out a list of reducers used in the actual table
        let mut rfs : Vec<SubstringReducer> = Vec::new();
        let hf = MD5Hasher;
        for i in 6..27 {
            rfs.push(SubstringReducer { n : i });
        }

        // Test that the rainbow table can find hashes that appear randomly in the table
        let mut rng = thread_rng();
        for _ in 0..1000 {
            let mut value = format!("{}", rng.gen_range(0, 1000));
            let reductions = rng.gen_range(1, rfs.len());
            for i in 0..reductions {
                value = rfs[i].reduce(&hf.digest(&value[..])[..]); 
            }
            let mut hash = hf.digest(&value[..]);
            assert_eq!(Some(value), TABLE.find_plaintext(&hash));
        }
    }

    #[test]
    fn test_rainbow_table_find_none() {
        // Test that the table will return None when presented with a hash not in the table 
        assert_eq!(None, TABLE.find_plaintext("8a1fb399dcf220db935995abce6a1532"));
    }
}

