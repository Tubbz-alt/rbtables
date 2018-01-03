pub mod structures;

#[cfg(test)]
mod tests {

  extern crate md5;
  extern crate easybench;
  extern crate num_cpus;

  use structures::RainbowTable;

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
    for _ in 0..99 {
      rfs.push(simple_reduction_function2);
    }

    let seeds = vec!["test", "monster", "test2", "amazing"];

    let mut table : RainbowTable = RainbowTable::new(md5_hashing_function, rfs);
    table.add_seeds(&seeds);
    for i in 0..100 {
      let seed = format!("{}", i);
      table.add_seed(seed);
    }

    table
  }

  #[test]
  fn execute_rainbow_table_single() {
    let table = build_rainbow_table();
    assert_eq!(Some(String::from("monster")), table.find_plaintext_single("8bf4e6addd72a9c4c4714708d2941528"));
    assert_eq!(Some(String::from("8bf4e")), table.find_plaintext_single("18b11cf86b4a3fd75e3fd9ac3485bdb6"));
    println!("CORES=1 {}", easybench::bench(|| table.find_plaintext_single("8bf4e6addd72a9c4c4714708d2941528") ));
  }

  #[test]
  fn execute_rainbow_table_multi() {
    let table = build_rainbow_table();
    assert_eq!(Some(String::from("monster")), table.find_plaintext_multi("8bf4e6addd72a9c4c4714708d2941528", 2));
    assert_eq!(Some(String::from("8bf4e")), table.find_plaintext_multi("18b11cf86b4a3fd75e3fd9ac3485bdb6", 2));
    println!("CORES=2 {}", easybench::bench(|| table.find_plaintext_multi("8bf4e6addd72a9c4c4714708d2941528", 2) ));
  }

  #[test]
  fn execute_rainbow_table_system() {
    let table = build_rainbow_table();
    assert_eq!(Some(String::from("monster")), table.find_plaintext("8bf4e6addd72a9c4c4714708d2941528"));
    assert_eq!(Some(String::from("8bf4e")), table.find_plaintext("18b11cf86b4a3fd75e3fd9ac3485bdb6"));
    println!("CORES={} {}", num_cpus::get() - 1, easybench::bench(|| table.find_plaintext("8bf4e6addd72a9c4c4714708d2941528") ));
  }
}

