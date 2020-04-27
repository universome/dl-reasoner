#![allow(unused)]

use std::env;
use std::fs;

#[macro_use]
extern crate mopa;

mod abox;
mod tbox;
mod concept;
mod reasoner;

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
    println!("{:#?}", a);
}
