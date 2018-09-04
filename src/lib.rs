pub mod structures;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests {
    extern crate md5;

    use structures::RainbowTable;
    use structures::Hasher;
    use structures::Reducer;

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
        // Test that the rainbow table can find hashes that appear randomly in the table
        
    }

    #[test]
    fn test_rainbow_table_find_none() {
        // Test that the table will return None when presented with a hash not in the table 
        assert_eq!(None, TABLE.find_plaintext("8a1fb399dcf220db935995abce6a1532"));
    }
}

