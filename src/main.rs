#![allow(unused)]

use std::env;
use std::fs;
// use std::core;

mod abox;
mod tbox;
mod common;
// mod nnf;

fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    println!("File {}", filename);
    // let string = "SUPERSTRING";
    // println!("SUPERSTRING? {}", string.chars().skip(60).collect::<String>());
    // println!("SUPERSTRING? {}", string.chars().take(60).collect::<String>());

    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");

    let a = abox::parse_abox(&contents);

    // println!("With text:\n{}", contents);
    println!("ABOX: {:#?}", a);
}
