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
    // capture groups:
    //   1: path (for example "payment/8cff8e51-5dab-4c72-b534-25b73466d2e3")
    //   2: id if path is hobbit
    //   3: id if path is columbus
    let re = Regex::new("requestUrl\":\"https://www..+/(payment/([a-fA-F0-9-]{36})|order/profiles/([a-fA-F0-9-]{36})/payments/new)\"").unwrap();

    // regular expression to get the IP address
    let re_addr = Regex::new("remoteIp\":\"([0-9.]+)\"").unwrap();

    let mut count = Counts {
      hobbit: 0,
      columbus: 0,
      hobbit_paid: 0,
      columbus_paid: 0,
      total_paid: ids.len(),
    };

    // save ip:id combination to ensure unique counts
    let mut set = BTreeSet::<String>::new();

    let stdin = io::stdin();
    let handle = stdin.lock();

    for some_line in handle.lines() {
      let line = some_line.unwrap();

      if !line.contains(":200,") { continue; }  // only status code 200 is interesting
      if !line.contains("/payment/") && !line.contains("/payments/new") { continue; }  // no one cares about assets

      if let Some(captures) = re.captures(&line) {
        if let Some(ip_capture) = re_addr.captures(&line) {
          let mut ip = ip_capture[1].to_owned();

          // capture group 2 only exists for hobbit URLs
          if captures.get(2).is_some() {
            let id = &captures[2];
            ip.push_str(id);

            if set.insert(ip) {
              count.hobbit += 1;
              if ids.remove(id) { count.hobbit_paid += 1; }
            }

          // capture group 3 only exists for columbus URLs
          } else if captures.get(3).is_some() {
            let id = &captures[3];
            ip.push_str(id);

            if set.insert(ip) {
              count.columbus += 1;
              if ids.remove(id) { count.columbus_paid += 1; }
            }
          }
        }
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
  println!("{} of {} total payments", count.hobbit_paid + count.columbus_paid, count.total_paid);
}
