use std::fmt;
use std::hash;
use std::collections::HashSet;
use concept::{Concept, parse_concept};


pub fn parse_tbox(tbox_str: &str) -> TBox {
    debug!("Parsing TBox!");

    let tbox_str = tbox_str.trim();
    let mut tbox = TBox::new();

    for line in tbox_str.lines() {
        debug!("Parsing line: {}", line);

        if line.len() > 0 && !line.starts_with('#') {
            tbox.axioms.insert(Box::new(parse_tbox_axiom(line)));
        }
    }

    tbox
}


pub fn parse_tbox_axiom(tbox_line: &str) -> TBoxAxiom {
    let tbox_line = tbox_line.trim();
    let delimiter = if tbox_line.contains("==") { "==" } else { "->" };
    let axiom_type = if delimiter == "==" { TBoxAxiomType::Definition } else { TBoxAxiomType::Subsumption };
    let delimiter_idx = tbox_line.find(delimiter).unwrap();

    TBoxAxiom {
        axiom_type: axiom_type,
        lhs: parse_concept(&tbox_line[..delimiter_idx]).convert_to_nnf(),
        rhs: parse_concept(&tbox_line[delimiter_idx + 2..]).convert_to_nnf()
    }
}

#[derive(Debug, Clone)]
pub struct TBox {
    axioms: HashSet<Box<TBoxAxiom>>
}

impl TBox {
    pub fn new() -> TBox {
        TBox {axioms: HashSet::new()}
    }
}

impl fmt::Display for TBox {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "TBox:\n  - {}", self.axioms.iter().map(|a| a.to_string()).collect::<Vec<String>>().join("\n  - "))
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum TBoxAxiomType { Definition, Subsumption }

#[derive(Debug, Clone, Eq)]
pub struct TBoxAxiom {
    pub axiom_type: TBoxAxiomType,
    pub lhs: Box<dyn Concept>,
    pub rhs: Box<dyn Concept>,
}

impl fmt::Display for TBoxAxiom {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let delimiter = if self.axiom_type == TBoxAxiomType::Definition {"=="} else {"->"};
        write!(fmt, "{} {} {}", self.lhs.to_string(), delimiter, self.rhs.to_string())
    }
}

// We have to implement everyhing manually since
// damn rust cannor derive these traits without "move occurs" error
impl PartialEq for TBoxAxiom {
    fn eq(&self, other: &TBoxAxiom) -> bool {
        self.to_string() == other.to_string()
    }
}

impl hash::Hash for TBoxAxiom {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.to_string().hash(hasher);
    }
}
