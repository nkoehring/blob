blob
====

**raw log data scanner**

Look through all the logfiles and count occurences of specific strings (or regular expression matches)

Currently very specific to [our](https://fromatob.github.io/) needs. But I plan to make the expressions configurable.



Usage
-----

```
cat /all/your/log/files/*.json | blob [FLAGS] <ids_path>

FLAGS:
      --help              Prints help information
  -h, --human-readable    Print statistics in a human readable format
  -V, --version           Prints version information
ARGS:
  <ids_path>    Path of CSV or plain text file with IDs to check for

```

Output defaults to CSV. Fields are in order:

 * (hobbit) visits
 * (hobbit) payments
 * (hobbit) payments with paypal
 * (hobbit) payments with credit card
 * (hobbit) payments with sofort
 * (columbus) visits
 * (columbus) payments
 * (columbus) payments with paypal
 * (columbus) payments with credit card
 * (columbus) payments with sofort

Build
-----

```sh
# Make sure, Rust is installed. I only tested with nightly.
cargo build --release
```
