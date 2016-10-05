//  cidr_to_asn
//  usage: cidr_to_asn --help
//
//  Versio: 0.1.0
//  Author: Jared M. Smith <jared@jaredsmith.io>
//
//


extern crate clap;

use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;
use std::fs::File;
use std::process::exit;
use std::collections::HashMap;

use clap::{Arg, App};

fn main() {
    // The main function of the program

    // Parse CLI arguments
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

    // Open RIB file for reading
    let f = match File::open(input_filename) {
        Ok(file) => file,
        Err(e) => {
            println!("{}", e);
            println!("Exiting...");
            exit(1);
        }
    };

    // Define the hashmap for mapping CIDR's to AS #'s
    let mut mappings: HashMap<String, Vec<String>> = HashMap::new();
    let file = BufReader::new(&f);

    for line in file.lines() {

        // Break up each line
        let l = line.unwrap();
        let entry: Vec<&str> = l.split('|').collect();

        // Extract the CIDR and the AS level path
        let cidr = entry[5].to_string();
        let asn_nums_str = entry[6].to_string();

        // Break up the AS level path
        let asn_nums: Vec<&str> = asn_nums_str.split(' ').collect();
        if asn_nums.is_empty() {
            continue;
        }

        // Get the last AS number in the path
        let last_asn_num = asn_nums.last().unwrap();

        // Build the mappings from CIDR's to the last AS number in each path
        if mappings.contains_key(&cidr) {
            let mut existing_asn_nums = mappings.get_mut(&cidr).unwrap();
            existing_asn_nums.push(last_asn_num.to_string());
        } else {
            let mut new_asn_nums: Vec<String> = Vec::new();
            new_asn_nums.push(last_asn_num.to_string());
            mappings.insert(cidr, new_asn_nums);
        }
    }

    // Open a file for writing the final results
    let output_filename = matches.value_of("OUTPUT").unwrap();
    let mut buffer = File::create(output_filename).unwrap();

    // Go through each mapping, find the most common AS number among the different possibilities,
    // and build another mapping to be written to a file
    // We have to do this because AS's could lie about their routes, so we choose the most common
    // AS number and assume that is the true route
    for (cidr, as_nums) in mappings {

        // Build a mapping of AS nums to the number of times they show up
        // In Python this would be super easy, but I don't know a better way to do this in Rust yet
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

        // Get the most common AS
        for (as_num, count) in asn_counts {

            if count > last_count {
                most_common_as_num = as_num;
                last_count = count;
            }
        }

        // Write the results to a file in CSV format
        write!(&mut buffer, "{},{}\n", cidr, most_common_as_num);
    }
}
