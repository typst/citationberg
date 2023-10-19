# Citationberg

[![Build status](https://github.com/typst/biblatex/workflows/Continuous%20integration/badge.svg)](https://github.com/typst/biblatex/actions)

<p align="center">
  <img alt="Dinkelberg meme: Dad from the TV show The Fairly Odd Parents exclaiming Citationberg" src="https://github.com/typst/citationberg/blob/main/assets/citationberg.png?raw=true" width="426">
</p>

A library for parsing CSL styles.

Citationberg deserializes CSL styles from XML into Rust structs. It supports
[CSL 1.0.2](https://docs.citationstyles.org/en/stable/specification.html).

This crate is not a CSL processor, so you are free to choose whatever data model
and data types you need for your bibliographic needs. If you need to render
citations, you can use [Hayagriva](https://github.com/typst/hayagriva) which
uses this crate under the hood.

Parse your style like this:

```rust
use citationberg::Style;
use std::fs;
let style =
    citationberg::Style::from_xml(
      &fs::read_to_string("tests/independent/ieee.csl").unwrap()
    ).unwrap();
    
if let Style::Independent(independent) = style {
    assert_eq!(independent.info.title.value, "IEEE");
    // Get started processing your style!
} else {
    panic!("IEEE is an independent style");
}
```

## Safety
This crate forbids unsafe code.

## License
This crate is dual-licensed under the MIT and Apache 2.0 licenses.
