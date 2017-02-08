#[macro_use]
extern crate clap;
extern crate regex;

use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;
use std::collections::{BTreeSet, BTreeMap};
use std::default::Default;

use clap::App;
use regex::Regex;


fn read_ids<P>(filename: P) -> BTreeMap<String, String>
where P: AsRef<Path>,
{
  let mut set = BTreeMap::<String, String>::new();
  let file = File::open(filename).expect("Cannot find IDs file!");
  let reader = io::BufReader::new(file);

  for line in reader.lines() {
    let l = line.unwrap();
    set.insert(l.to_string(), "".to_string());
  }

 set 
}

fn read_csv<P>(filename: P) -> BTreeMap<String, String>
where P: AsRef<Path>,
{
  let mut map = BTreeMap::<String, String>::new();
  let file = File::open(filename).expect("Cannot find CSV!");
  let reader = io::BufReader::new(file);

  for line in reader.lines() {
    let l = line.unwrap();
    let parts: Vec<&str> = l.split(",").collect();

    if parts.len() < 3 {
      println!("WARNING! Ignoring invalid CSV line: {:?}", l);
      continue;
    }

    map.insert(parts[2].to_string(), parts[1].to_string());
  }

  map
}


fn check_log(mut ids: BTreeMap<String, String>) -> Counts {
    // capture groups:
    //   1: path (for example "payment/8cff8e51-5dab-4c72-b534-25b73466d2e3")
    //   2: id if path is hobbit
    //   3: id if path is columbus
    let re = Regex::new("requestUrl\":\"https://www..+/(payment/([a-fA-F0-9-]{36})|order/profiles/([a-fA-F0-9-]{36})/payments/new)\"").unwrap();

    // regular expression to get the IP address
    let re_addr = Regex::new("remoteIp\":\"([0-9.]+)\"").unwrap();

    // yay to default values! https://doc.rust-lang.org/stable/std/default/trait.Default.html
    let mut count = Counts { total_paid: ids.len(), ..Default::default() };

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
              if let Some(value) = ids.remove(id) {
                count.hobbit_paid += 1;
                match value.as_ref() {
                  "paypal" => count.hobbit_paypal += 1,
                  "paypal_vault" => count.hobbit_paypal_vault += 1,
                  "credit_card" => count.hobbit_credit_card += 1,
                  "sofort" => count.hobbit_sofort += 1,
                  _ => println!("Whoops? Hobbit?!"),
                }
              }
            }

          // capture group 3 only exists for columbus URLs
          } else if captures.get(3).is_some() {
            let id = &captures[3];
            ip.push_str(id);

            if set.insert(ip) {
              count.columbus += 1;
              if let Some(value) = ids.remove(id) {
                count.columbus_paid += 1;
                match value.as_ref() {
                  "paypal" => count.columbus_paypal += 1,
                  "paypal_vault" => count.columbus_paypal_vault += 1,
                  "credit_card" => count.columbus_credit_card += 1,
                  "sofort" => count.columbus_sofort += 1,
                  _ => println!("Whoops? Columbus?!"),
                }
              }
            }
          }
        }
      }
    }

    count
}

fn print_stats_for_humans(count: Counts) {
  let hobbit_con_perc = (count.hobbit_paid as f64) * 100.0 / (count.hobbit as f64);
  let columbus_con_perc = (count.columbus_paid as f64) * 100.0 / (count.columbus as f64);

  let hobbit_pay_perc = (count.hobbit_paid as f64) * 100.0 / ((count.hobbit_paid + count.columbus_paid) as f64);
  let columbus_pay_perc = (count.columbus_paid as f64) * 100.0 / ((count.hobbit_paid + count.columbus_paid) as f64);

  println!("hobbit:   {:3} calls lead to {:3} payments, that's {:.2}% of payments and a conversion rate of {:.2}%", count.hobbit, count.hobbit_paid, hobbit_pay_perc, hobbit_con_perc);
  println!("columbus: {:3} calls lead to {:3} payments, that's {:.2}% of payments and a conversion rate of {:.2}%", count.columbus, count.columbus_paid, columbus_pay_perc, columbus_con_perc);
  println!("{} of {} total payments", count.hobbit_paid + count.columbus_paid, count.total_paid);
  println!("Hobbit   pp / cc / sofort: {:3} / {:3} / {:3}", count.hobbit_paypal + count.hobbit_paypal_vault, count.hobbit_credit_card, count.hobbit_sofort);
  println!("Columbus pp / cc / sofort: {:3} / {:3} / {:3}", count.columbus_paypal + count.columbus_paypal_vault, count.columbus_credit_card, count.columbus_sofort);
}


// print statistics in CSV format visits, payments, pp, cc, sofort for hobbit and columbus in one line
fn print_stats(count: Counts) {
  println!("{},{},{},{},{},{},{},{},{},{}",
    count.hobbit, count.hobbit_paid, count.hobbit_paypal + count.hobbit_paypal_vault, count.hobbit_credit_card, count.hobbit_sofort,
    count.columbus, count.columbus_paid, count.columbus_paypal + count.columbus_paypal_vault, count.columbus_credit_card, count.columbus_sofort
  );
}

#[derive(Default)]
struct Counts {
  hobbit: usize,
  columbus: usize,
  hobbit_paid: usize,
  columbus_paid: usize,
  total_paid: usize,
  hobbit_credit_card: usize,
  hobbit_paypal: usize,
  hobbit_paypal_vault: usize,
  hobbit_sofort: usize,
  columbus_credit_card: usize,
  columbus_paypal: usize,
  columbus_paypal_vault: usize,
  columbus_sofort: usize,
}

fn main() {
  let args = App::new(crate_name!())
      .version(crate_version!())
      .author(crate_authors!("\n"))
      .about(crate_description!())
      .args_from_usage("-h, --human-readable 'Print statistics in a human readable format'
                        <ids_path> 'Path of CSV or plain text file with IDs to check for'")
      .get_matches();

  let ids_path = args.value_of("ids_path").unwrap();

  let ids = match ids_path.ends_with(".csv") {
    true => read_csv(ids_path),
    false => read_ids(ids_path),
  };
  let count = check_log(ids);

  if args.is_present("human-readable") {
    print_stats_for_humans(count);
  } else {
    print_stats(count);
  }
}
