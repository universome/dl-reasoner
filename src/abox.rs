/*
    ABox axioms have the following format:
    "C[x]", "r[x, y]", "(some r C)[x]", etc.
    This makes parsing easy without the loss of readability
*/
use std::fmt;
use std::hash;
use std::clone::Clone;
use std::string;
use std::collections::HashSet;

use concept::{Individual, Relation, Concept, parse_concept};

#[derive(PartialEq)]
pub enum ABoxAxiomType { Concept, Relation }

#[derive(Debug, Clone)]
pub struct ABox {
    pub axioms: HashSet<Box<dyn ABoxAxiom>>
    // pub axioms: Vec<Box<dyn ABoxAxiom>>
}

impl fmt::Display for ABox {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "ABox {}", self.axioms.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(" "))
    }
}

pub trait ABoxAxiom: fmt::Debug + fmt::Display + mopa::Any + ABoxAxiomClone {
    fn axiom_type(&self) -> ABoxAxiomType;
}
mopafy!(ABoxAxiom);

impl hash::Hash for dyn ABoxAxiom {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.to_string().hash(hasher);
    }
}

impl PartialEq for dyn ABoxAxiom {
    fn eq(&self, other: &dyn ABoxAxiom) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for dyn ABoxAxiom {}

impl Clone for Box<dyn ABoxAxiom> {
    fn clone(&self) -> Box<dyn ABoxAxiom> { self.clone_box() }
}

pub trait ABoxAxiomClone {
    fn clone_box(&self) -> Box<dyn ABoxAxiom>;
}

impl<T> ABoxAxiomClone for T where T: ABoxAxiom + Clone {
    fn clone_box(&self) -> Box<dyn ABoxAxiom> { Box::new(self.clone()) }
}

#[derive(Debug, Clone, Hash)]
pub struct ConceptAxiom {
    pub concept: Box<dyn Concept>,
    pub individual: Individual
}

impl fmt::Display for ConceptAxiom {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({})[{}]", self.concept, self.individual.name)
    }
}

impl ABoxAxiom for ConceptAxiom {
    fn axiom_type(&self) -> ABoxAxiomType { ABoxAxiomType::Concept }
}

#[derive(Debug, Clone, Hash)]
pub struct RelationAxiom {
    pub relation: Relation,
    pub lhs: Individual,
    pub rhs: Individual,
}

impl fmt::Display for RelationAxiom {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}({}, {})", self.relation.name, self.lhs.name, self.rhs.name)
    }
}

// impl string::ToString for RelationAxiom {
//     fn to_string(&self) -> String {
//         format!("{}({}, {})", self.relation.name, self.lhs.name, self.rhs.name)
//     }
// }

impl ABoxAxiom for RelationAxiom {
    fn axiom_type(&self) -> ABoxAxiomType { ABoxAxiomType::Relation }
}


pub fn parse_abox(abox_str: &str) -> ABox {
    let abox_str = abox_str.trim();
    let mut abox = ABox {axioms: HashSet::new()};

    for line in abox_str.lines() {
        println!("Parsing line: {}", line);
        abox.axioms.insert(parse_abox_axiom(&line));
    }

    abox
}


pub fn parse_abox_axiom(axiom_str: &str) -> Box<dyn ABoxAxiom> {
    let axiom_str = axiom_str.trim();
    let start_idx = axiom_str.find("[").unwrap_or(0);
    let end_idx = axiom_str.find("]").unwrap_or(axiom_str.len());
    let arguments_str = &axiom_str[start_idx+1..end_idx].trim();
    println!("arguments string: {}", arguments_str);
    let mut individuals = arguments_str
        .split(",").map(|n| (Individual {name: n.to_string()}))
        .collect::<Vec<_>>();

    if arguments_str.contains(",") {
        // This is a relation axiom
        Box::new(RelationAxiom {
            relation: Relation { name: axiom_str[..start_idx].to_string() },
            lhs: individuals.remove(0),
            rhs: individuals.remove(0),
        })
    } else {
        // This is a concept axiom
        Box::new(ConceptAxiom {
            concept: parse_concept(&axiom_str[..start_idx]).convert_to_nnf(),
            individual: individuals.remove(0)
        })
    }
}
