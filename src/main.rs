#![allow(unused)]
use std::env;
use std::fs;
use std::time::{Duration, Instant};

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
            // out.finish(format_args!(
            //     "{}[{}][{}] {}",
            //     chrono::Local::now().format("[%H:%M:%S]"),
            //     record.target(),
            //     record.level(),
            //     message
            // ))
            out.finish(format_args!(
                "[{}] {}",
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        // .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log"))
        .apply();

    let start = Instant::now();
    run_reasoner();
    let duration = start.elapsed();
    info!("Running time: {:?}", duration);
}

fn run_reasoner() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    match command.as_str() {
        "check-consistency" => {
            let abox_filename = &args[2];
            let tbox_filename = &args[3];

            let abox_file_contents = fs::read_to_string(abox_filename).unwrap();
            let tbox_file_contents = fs::read_to_string(tbox_filename).unwrap();

            let mut abox = abox::parse_abox(&abox_file_contents);
            debug!("Initial abox: {}", abox);

            let mut tbox = tbox::parse_tbox(&tbox_file_contents);
            debug!("Initial tbox: {}", tbox);

            tbox.expand_all_definitions();
            tbox.apply_definitions_to_abox(&mut abox);
            tbox.apply_definitions_to_inclusions();
            let super_gci = tbox.aggregate_inclusions().map(|c| Box::new(c) as Box<dyn concept::Concept>);

            debug!("Abox after definitions applied: {}", abox);

            match reasoner::tableau_reasoning(abox, super_gci) {
                None => info!("No model was found."),
                Some(a) => {
                    info!("Found a model!");
                    info!("{}", a.extract_model());
                }
            }
        },
        "check-subsumption" => {

            // Initialzing TBox
            let tbox_filename = &args[2];
            let tbox_file_contents = fs::read_to_string(tbox_filename).unwrap();
            let mut tbox = tbox::parse_tbox(&tbox_file_contents);
            debug!("Initial tbox: {}", tbox);

            tbox.expand_all_definitions();
            tbox.apply_definitions_to_inclusions();
            let super_gci = tbox.aggregate_inclusions();
            assert!(super_gci.is_some(), "Error: you have not provided a subsumption to check!");

            // Initialzing ABox
            let mut abox = abox::ABox::new();
            let x = concept::Individual {name: "a".to_string()};
            let super_concept = Box::new(super_gci.clone().unwrap()) as Box<dyn concept::Concept>;
            let super_concept = super_concept.negate().convert_to_nnf();
            abox.add_individual(x.clone());
            abox.axioms.insert(Box::new(abox::ConceptAxiom {
                concept: super_concept.clone(),
                individual: x,
            }) as Box<dyn abox::ABoxAxiom>);

            match reasoner::tableau_reasoning(abox, Some(super_concept)) {
                None => info!("Subsimption is valid."),
                Some(a) => info!("Subsimption is not valid.")
            }
        },
        _ => panic!("Error: unknown command: {}", command)
    }
}
