extern crate clap;

use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;
use std::fs::File;
use std::process::exit;
use std::collections::HashMap;

use clap::{Arg, App};

fn main() {

    let matches = App::new("cidr_to_asn")
        .version("0.1")
        .author("Jared M. Smith <jared@jaredsmith.io>")
        .arg(Arg::with_name("INPUT")
            .short("i")
            .long("input")
            .value_name("INPUT")
            .required(true)
            .help("The RIB file as input"))
        .arg(Arg::with_name("OUTPUT")
            .short("o")
            .long("output")
            .value_name("OUTPUT")
            .required(true)
            .help("The csv file as output"))
        .get_matches();

    let input_filename = matches.value_of("INPUT").unwrap();

    // println!("Reading in RIB data...");
    let f = match File::open(input_filename) {
        Ok(file) => file,
        Err(e) => {
            println!("{}", e);
            println!("Exiting...");
            exit(1);
        }
    };

    let mut mappings: HashMap<String, Vec<String>> = HashMap::new();

    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        // println!("{}", l);
        let entry: Vec<&str> = l.split('|').collect();

        let cidr = entry[5].to_string();
        let asn_nums_str = entry[6].to_string();

        let asn_nums: Vec<&str> = asn_nums_str.split(' ').collect();
        if asn_nums.is_empty() {
            continue;
        }

        let last_asn_num = asn_nums.last().unwrap();
        if mappings.contains_key(&cidr) {
            let mut existing_asn_nums = mappings.get_mut(&cidr).unwrap();
            existing_asn_nums.push(last_asn_num.to_string());
        } else {
            let mut new_asn_nums: Vec<String> = Vec::new();
            new_asn_nums.push(last_asn_num.to_string());
            mappings.insert(cidr, new_asn_nums);
        }
    }

    let output_filename = matches.value_of("OUTPUT").unwrap();

    let mut buffer = File::create(output_filename).unwrap();

    for (cidr, as_nums) in mappings {
        // println!("ASN's for {}", cidr);

        let mut asn_counts: HashMap<String, i32> = HashMap::new();
        for as_num in as_nums {
            if asn_counts.contains_key(&as_num) {
                let mut count = asn_counts.get_mut(&as_num).unwrap();
                *count += 1;
            } else {
                asn_counts.insert(as_num, 1);
            }
        }


        let mut most_common_as_num: String = String::new();
        let mut last_count: i32 = 0;

        for (as_num, count) in asn_counts {
            // println!("as_num {} occurs {} times", as_num, count);

            if count > last_count {
                most_common_as_num = as_num;
                last_count = count;
            }
        }

        write!(&mut buffer, "{},{}\n", cidr, most_common_as_num);

        // println!("Most Common AS Num for CIDR {}: {}",
        // cidr,
        // most_common_as_num);
        //
    }
}
