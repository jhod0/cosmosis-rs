# CosmoSIS for Rust
Rust bindings for the [CosmoSIS](https://bitbucket.org/joezuntz/cosmosis/wiki/Home) cosmological parameter estimation library.

## Getting Started

First off, install and build CosmoSIS. Then:

```bash
$ COSMOSIS=/path/to/cosmosis
$ export COSMOSIS_INC=$COSMOSIS/cosmosis/datablock
$ export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$COSMOSIS_INC
$ git clone https://github.com/jhod0/cosmosis-rs.git
$ cd cosmosis-rs
$ cargo test
```

This is a work in progress.
