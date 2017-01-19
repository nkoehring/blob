blob
====

Look through all the logfiles and count occurences of specific strings (or regular expression matches)

Currently very specific to [our](https://fromatob.github.io/) needs. But I plan to make the expressions configurable.

Usage
-----

```sh
cat /path/to/logs/*.json | ./blob /path/to/ids
```


Build
-----

```sh
# Make sure, Rust is installed. I only tested with nightly.
cargo build --release
```
