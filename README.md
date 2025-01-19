nzb-rs
========

![Tests](https://img.shields.io/github/actions/workflow/status/Ravencentric/nzb-rs/tests.yml?label=tests&link=https%3A%2F%2Fgithub.com%2FRavencentric%2Fnzb-rs%2Factions%2Fworkflows%2Ftests.yml)
![Latest Version](https://img.shields.io/crates/v/nzb-rs?link=https%3A%2F%2Fcrates.io%2Fcrates%2Fnzb-rs)
[![Documentation](https://docs.rs/nzb-rs/badge.svg)](https://docs.rs/nzb-rs)
![License](https://img.shields.io/crates/l/nzb-rs)

`nzb-rs` is a [spec](https://sabnzbd.org/wiki/extra/nzb-spec) compliant parser for [NZB](https://en.wikipedia.org/wiki/NZB) files.

This library is a partial port of the Python [`nzb`](https://pypi.org/project/nzb/) library.  
While the parser API is similar to the original, it is not identical. Additionally, the meta editor API has not been implemented.

## Installation

To install `nzb-rs`, add the following to your `Cargo.toml`:

```toml
[dependencies]
nzb-rs = "0.1.1"
```

Optional features:

- `serde`: Enables serialization and deserialization via [serde](https://crates.io/crates/serde).

## Example

```rust
use nzb_rs::{InvalidNZBError, NZB};

fn main() -> Result<(), InvalidNZBError> {
    let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE nzb PUBLIC "-//newzBin//DTD NZB 1.1//EN" "http://www.newzbin.com/DTD/nzb/nzb-1.1.dtd">
        <nzb
            xmlns="http://www.newzbin.com/DTD/2003/nzb">
            <file poster="John &lt;nzb@nowhere.example&gt;" date="1706440708" subject="[1/1] - &quot;Big Buck Bunny - S01E01.mkv&quot; yEnc (1/2) 1478616">
                <groups>
                    <group>alt.binaries.boneless</group>
                </groups>
                <segments>
                    <segment bytes="739067" number="1">9cacde4c986547369becbf97003fb2c5-9483514693959@example</segment>
                    <segment bytes="739549" number="2">70a3a038ce324e618e2751e063d6a036-7285710986748@example</segment>
    
                </segments>
            </file>
        </nzb>
        "#;
    let nzb = NZB::parse(xml)?;
    println!("{:#?}", nzb);
    assert_eq!(nzb.file().name(), Some("Big Buck Bunny - S01E01.mkv"));
    Ok(())
}

```

## Safety

- This library must not panic. Any panic should be considered a bug and reported.
- This library uses [`roxmltree`](https://crates.io/crates/roxmltree) for parsing the NZB. `roxmltree` is written entirely in safe Rust, so by Rust's guarantees the worst that a malicious NZB can do is to cause a panic.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](https://github.com/Ravencentric/nzb-rs/blob/main/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](https://github.com/Ravencentric/nzb-rs/blob/main/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
