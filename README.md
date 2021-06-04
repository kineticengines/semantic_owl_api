# SEMANTIC OWL API

A Rust implementation of [owlapi](https://github.com/owlcs/owlapi) which was originally written in Java.
However, Semantic Owl API is not a direct one-to-one implementation of owlapi.
While ideas are borrowed, the implementation is not. It also includes some features provided by
[Robot](https://github.com/ontodev/robot) which is also written in Java.

Why implement in Rust?

1 - A great number of semantic tools are implemented in Java hence limiting those of us who are
strongly inclined towards languages like Rust

2 - Need for a super performant tool which is not limited by the JVM memory constraints

## Base derivation

Semantic Owl API id derived from Owlapi version 5.1.17 onwards

## Packages

- Semantic owl api

- Semantic owl cli

## Development setup

### Nightly

- Install the latest nightly version of `rustup`
- In vscode, install [Matklad Rust extension](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer). If you have official
  [Rust extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust) installed, disable it
- Setup testing coverage by following [this guide](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/instrument-coverage.html)

### Stable

- Install the latest nightly version of `rustup`
- In vscode, install the [Official Rust extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust). If you have
  [Matklad Rust extension](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer) installed, disable it
- NOTE : `source-based-code-coverage` coverage is not available for stable at this moment hence you will have to relay on
  CI which runs using both nightly and stable build of `rustup`

## References

- https://github.com/benkay86/nom-tutorial/

- http://owlcs.github.io/owlapi/apidocs_5/overview-summary.html

- https://github.com/owlcs/owlapi/wiki/Documentation

- http://www.semantic-web-journal.net/sites/default/files/swj107_2.pdf

- https://www.w3.org/TR/2012/REC-owl2-syntax-20121211/

- https://www.w3.org/TR/2012/REC-owl2-primer-20121211/

- https://www.w3.org/TR/2012/REC-owl2-mapping-to-rdf-20121211/

- https://www.w3.org/TR/2014/REC-turtle-20140225/

- http://www.semantic-web-journal.net/system/files/swj1734.pdf
