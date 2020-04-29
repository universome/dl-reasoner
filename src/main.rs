#![allow(unused)]
use std::env;
use std::fs;

#[macro_use] extern crate mopa;
#[macro_use] extern crate log;
extern crate fern;
extern crate chrono;

mod abox;
mod tbox;
mod concept;
mod reasoner;

fn main() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log"))
        .apply();

    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");

    let abox = abox::parse_abox(&contents);
    debug!("Intiial ABox: {}", abox);

    match reasoner::tableau_reasoning(abox, tbox::TBox::new()) {
        None => info!("No model was found."),
        Some(a) => {
            info!("Found a model!");
            info!("{}", abox::remove_non_atomic_concepts(&a));
        }
    }
}
