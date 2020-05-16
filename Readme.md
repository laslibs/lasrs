![Test](https://github.com/laslibs/lasrs/workflows/Test/badge.svg?branch=master)

## Lasrs

A Rust library for parsing geophysical well log (.las) files

Supports only version 2.0 of [LAS Specification](https://www.cwls.org/wp-content/uploads/2017/02/Las2_Update_Feb2017.pdf). For more information about this format, see the [Canadian Well Logging Society](http://www.cwls.org).

- Usage
  And this to your cargo.toml

  ```toml
  [dependencies]
  regex = "1"

  ```

  and this to your crate root (if you're using Rust 2015):

  ```rust
  extern crate lasrs;
  ```

  A example of reading version of well log headers.

  ```rust
  use lasrs::Las;

  fn main() {
      let las = Las::new("./sample/example.las");

      assert_eq!(
          vec!["DEPT", "DT", "RHOB", "NPHI", "SFLU", "SFLA", "ILM", "ILD"],
          las.headers()
      );

      let las = Las::new("./sample/A10.las");

      assert_eq!(
          vec!["DEPT", "Perm", "Gamma", "Porosity", "Fluvialfacies", "NetGross"],
          las.headers()
      );
  }
  ```

- Documentation

  [Module documentation with examples](https://doc.rs/lasrs)

- Test

  - Clone this repo and run:

  ```sh
  cargo test
  ```
