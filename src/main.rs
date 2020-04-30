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

    let abox_filename = &args[1];
    let tbox_filename = &args[2];

    let abox_file_contents = fs::read_to_string(abox_filename).unwrap();
    let tbox_file_contents = fs::read_to_string(tbox_filename).unwrap();

    let mut abox = abox::parse_abox(&abox_file_contents);
    debug!("Initial abox: {}", abox);

    let mut tbox = tbox::parse_tbox(&tbox_file_contents);
    debug!("Initial tbox: {}", tbox);

    tbox.expand_all_definitions();
    tbox.apply_definitions_to_abox(&mut abox);
    tbox.apply_definitions_to_inclusions();
    let super_gci = tbox.aggregate_inclusions();

    debug!("Abox after definitions applied: {}", abox);

    match reasoner::tableau_reasoning(abox, super_gci) {
        None => info!("No model was found."),
        Some(a) => {
            info!("Found a model!");
            info!("{}", abox::remove_non_primitive_concepts(&a));
        }
    }
}
