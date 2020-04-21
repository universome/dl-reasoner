/*
    ABox axioms have the following format:
    "C[x]", "r[x, y]", "(some r C)[x]", etc.
    This makes parsing easy without the loss of readability
*/
use std::string;
use common::{Individual, Relation, Concept, parse_concept};


pub enum ABoxAxiom { ConceptAxiom, RelationAxiom }

pub struct RelationAxiom {
    pub relation: Relation,
    pub lhs: Individual,
    pub rhs: Individual,
}

pub struct ConceptAxiom {
    pub concept: Concept,
    pub individual: Individual
}


pub fn parse_abox(abox_str: &str) -> Vec<ABoxAxiom> {
    let abox_str = abox_str.trim();
    let mut abox_axioms = Vec::new();

    for line in abox_str.lines() {
        println!("{}", line.unwrap());
        let line = line.unwrap();

        abox_axioms.push(parse_abox_axiom(line))
    }

    abox_axioms
}


pub fn parse_abox_axiom(axiom_str: &str) -> ABoxAxiom {
    let axiom_str = axiom_str.trim();
    let start_idx = axiom_str.find("[").unwrap_or(0);
    let end_idx = axiom_str.find("]").unwrap_or(axiom_str.len());
    let arguments_str = &axiom_str[start_idx..end_idx].trim();
    print!("arguments string: {}", arguments_str);
    let individuals = arguments_str.split(",").collect().map(|n| (Individual {name: String::from(n)}));

    if arguments_str.contains(",") {
        // This is a relation axiom
        let relation = Relation { name: String::from(axiom_str[..start_idx]) };

        RelationAxiom {
            relation: relation,
            lhs: individuals[0],
            rhs: individuals[1],
        }
    } else {
        // This is a concept axiom
        let concept = parse_concept(&axiom_str[..start_idx]);

        ConceptAxiom { concept: concept, individual: individuals[0]}
    }
}
