extern crate regex;

use std::env;
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;
use std::collections::BTreeSet;

use regex::Regex;


fn read_ids<P>(filename: P) -> BTreeSet<String>
  where P: AsRef<Path>,
  {
    let mut set = BTreeSet::<String>::new();
    let file = File::open(filename).expect("Cannot find IDs file!");
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
      let l = line.unwrap();
      set.insert(l.to_string());
    }

   set 
  }


fn check_log(mut ids: BTreeSet<String>) -> Counts {
    // TODO: speed up by catching both URLs at once and use string comparison
    // like `url = re.capures[1]; id = re.captures[2]`; url == format!(hobbit_url, id)
    let re_hobbit = Regex::new("^.*requestUrl\":\"https://www..+/payment/([a-zA-Z0-9-]{36}).*$").unwrap();
    let re_columbus = Regex::new("^.*requestUrl\":\"https://www..+/order/profiles/([a-zA-Z0-9-]{36})/payments/new.*$").unwrap();

    let mut count = Counts {
      hobbit: 0,
      columbus: 0,
      hobbit_paid: 0,
      columbus_paid: 0,
      total_paid: ids.len(),
    };

    let stdin = io::stdin();
    let handle = stdin.lock();

    for some_line in handle.lines() {
      let line = some_line.unwrap();

      if !line.contains(":200,") { continue; }  // only status code 200 is interesting
      if !line.contains("/payment/") && !line.contains("/payments/new") { continue; }  // no one cares about assets

      if let Some(caps_hobbit) = re_hobbit.captures(&line) {
        count.hobbit += 1;
        if ids.remove(&caps_hobbit[1]) { count.hobbit_paid += 1; }

      } else if let Some(caps_columbus) = re_columbus.captures(&line) {
        count.columbus += 1;
        if ids.remove(&caps_columbus[1]) { count.columbus_paid += 1; }
      }
    }

    count
  }


struct Counts {
  hobbit: usize,
  columbus: usize,
  hobbit_paid: usize,
  columbus_paid: usize,
  total_paid: usize,
}

fn main() {
  let args: Vec<_> = env::args().collect();
  if args.len() < 2 {
    println!("Usage: {} file_with_paid_ids < log_file", args[0]);
    return
  }

  let ids_path = args[1].clone();
  let ids = read_ids(ids_path);
  let count = check_log(ids);

  /* TODO: conversion rates are wrong as long as not only unique visits are counted
  let hobbit_con_perc = (count.hobbit_paid as f64) * 100.0 / (count.hobbit as f64);
  let columbus_con_perc = (count.columbus_paid as f64) * 100.0 / (count.columbus as f64);
  */

  let hobbit_pay_perc = (count.hobbit_paid as f64) * 100.0 / (count.total_paid as f64);
  let columbus_pay_perc = (count.columbus_paid as f64) * 100.0 / (count.total_paid as f64);

  println!("hobbit:   {:3} calls lead to {:3} payments, that's {:.2}% of payments", count.hobbit, count.hobbit_paid, hobbit_pay_perc);
  println!("columbus: {:3} calls lead to {:3} payments, that's {:.2}% of payments", count.columbus, count.columbus_paid, columbus_pay_perc);
  println!("Total payments recorded: {}",  count.total_paid);
}
