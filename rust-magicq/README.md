# Rust MagicQ

`rust-magicq` is a Rust library for reading and writing MagicQ showfiles. [MagicQ](https://chamsyslighting.com/products/magicq) is a lighting control software used for programming and controlling lighting fixtures and effects in the entertainment industry.

## Usage

To use this library in your Rust project, simply add the following to your `Cargo.toml` file:

```toml
[dependencies]
rust-magicq = "0.1.0"
```

Showfiles can be read in using the `Showfile::from_str` method and `to_string()` can be called to write the contents back out:

```rust
use std::fs;
use std::path::Path;
use magicq::Showfile;

fn main() {
    // Read in a showfile
    let path = Path::new("show/empty.shw");
    let contents = fs::read_to_string(path).expect("Failed to read file");
    let showfile = Showfile::from_str(&contents).unwrap();

    // List the headers in the file
    for header in showfile.get_headers() {
        println!("{}", header);
    }

    // Print a list of CueStacks
    let cuestacks = showfile.get_sections().iter()
        .filter(|section| section.get_identifier() == &SectionIdentifier::CueStack)
        .map(|section| section[0][1].to_string());
    for cuestack in cuestacks {
        println!("{}", cuestack);
    }

    // Write back out the showfile
    let output = showfile.to_string();
    fs::write("show/myshow.shw", output).expect("Failed to write file");
}
```

For a full list of available methods, check out `src/showfile.rs`.

## Contributions

This library is still under heavy development so please ask before trying to make any large changes.
Smaller changes, bugfixes and docs/examples/tests are welcome though, open a pull request.

## License

This library is licensed under the [MIT License](LICENSE.txt). This means that you are free to use, share, and modify the library, as long as you give credit and include the license in any distributions or modifications.
