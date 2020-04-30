/*
    ABox axioms have the following format:
    "C[x]", "r[x, y]", "(some r C)[x]", etc.
    This makes parsing easy without the loss of readability
*/
use std::fmt;
use std::hash;
use std::clone::Clone;
use std::string;
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;

use concept::{Individual, Relation, Concept, AtomicConcept, ConceptType, parse_concept};


pub fn parse_abox(abox_str: &str) -> ABox {
    let abox_str = abox_str.trim();
    let mut abox = ABox::new();

    for line in abox_str.lines() {
        debug!("Parsing line: {}", line);
        if line.len() > 0 && !line.starts_with('#') {
            add_abox_axiom(&mut abox, &line);
        }
    }

    abox
}


pub fn add_abox_axiom(abox: &mut ABox, axiom_str: &str) {
    let axiom_str = axiom_str.trim();
    let start_idx = axiom_str.find("[").unwrap_or(0);
    let end_idx = axiom_str.find("]").unwrap_or(axiom_str.len());
    let arguments_str = &axiom_str[start_idx+1..end_idx].trim();
    debug!("arguments string: {}", arguments_str);
    let individuals = arguments_str
        .split(", ").map(|n| (Individual {name: n.to_string()}))
        .collect::<Vec<_>>();

    if arguments_str.contains(",") {
        // This is a relation axiom
        abox.axioms.insert(Box::new(RelationAxiom {
            relation: Relation { name: axiom_str[..start_idx].to_string() },
            lhs: individuals[0].clone(),
            rhs: individuals[1].clone()
        }));
    } else {
        // This is a concept axiom
        abox.axioms.insert(Box::new(ConceptAxiom {
            concept: parse_concept(&axiom_str[..start_idx]).convert_to_nnf(),
            individual: individuals[0].clone()
        }));
    }

    for x in individuals {
        abox.add_individual(x);
    }
}


#[derive(PartialEq)]
pub enum ABoxAxiomType { Concept, Relation }

#[derive(Debug, Clone)]
pub struct ABox {
    pub axioms: HashSet<Box<dyn ABoxAxiom>>,
    pub is_consistent: Option<bool>,
    pub is_complete: Option<bool>,
    pub individuals: HashSet<Individual>,
    pub pairwise_different_individuals: Vec<HashSet<Individual>>,
    pub replacements: HashMap<Individual, Individual>
}

impl ABox {
    pub fn new() -> ABox {
        ABox {
            axioms: HashSet::new(),
            is_consistent: None,
            is_complete: None,
            individuals: HashSet::new(),
            pairwise_different_individuals: vec![],
            replacements: HashMap::new()
        }
    }

    pub fn extract_model(&self) -> Model {
        let mut model = Model::new();

        model.individuals = self.individuals.clone().into_iter().collect::<Vec<Individual>>();
        model.relation_axioms = self.axioms.clone().into_iter()
            .filter(|a| a.axiom_type() == ABoxAxiomType::Relation)
            .map(|a| a.downcast_ref::<RelationAxiom>().unwrap().clone())
            .map(|a| a.clone())
            .collect::<Vec<RelationAxiom>>();
        model.concept_axioms = self.axioms.clone().into_iter()
            .filter(|a| a.axiom_type() == ABoxAxiomType::Concept)
            .map(|a| a.downcast_ref::<ConceptAxiom>().unwrap().clone())
            .filter(|a| a.concept.concept_type() == ConceptType::Atomic)
            .filter(|a| a.concept.downcast_ref::<AtomicConcept>().unwrap().name != "__TOP__")
            .map(|a| a.clone())
            .collect::<Vec<ConceptAxiom>>();
        model.replacements = self.replacements.clone();

        model
    }

    // pub fn create_new_individual(&mut self) -> Individual {
    //     let new_x = Individual {name: format!("x_#{}", abox.individuals.len())};

    //     self.individuals.insert(new_x.clone());
    //     self.add_top_axiom_for_individual(new_x);
    // }

    pub fn add_top_axiom_for_individual(&mut self, x: Individual) {
        self.axioms.insert(Box::new(ConceptAxiom {
            concept: Box::new(AtomicConcept {name: "__TOP__".to_string()}) as Box<dyn Concept>,
            individual: x
        }) as Box<dyn ABoxAxiom>);
    }

    pub fn add_individual(&mut self, x: Individual) {
        self.individuals.insert(x.clone());
        self.add_top_axiom_for_individual(x);
    }
}

impl fmt::Display for ABox {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let axioms = self.axioms.iter()
            .map(|a| a.to_string()).collect::<Vec<String>>().join("\n  - ");
        let individuals = self.individuals.iter()
            .map(|x| x.to_string()).collect::<Vec<String>>().join(", ");
        write!(fmt, "\nAxioms:\n  - {}\nIndividuals: {}", axioms, individuals)
    }
}


pub struct Model {
    individuals: Vec<Individual>,
    concept_axioms: Vec<ConceptAxiom>,
    relation_axioms: Vec<RelationAxiom>,
    replacements: HashMap<Individual, Individual>
}

impl Model {
    pub fn new() -> Model {
        Model {
            individuals: vec![],
            concept_axioms: vec![],
            relation_axioms: vec![],
            replacements: HashMap::new()
        }
    }
}

impl fmt::Display for Model {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let individuals = format!("Individuals: {}", self.individuals.iter()
            .map(|x| x.to_string()).collect::<Vec<String>>().join(", "));
        let concepts = format!("Concepts: {}", self.concept_axioms.iter()
            .map(|c| c.to_string()).collect::<Vec<String>>().join(", "));
        let relations = format!("Relations: {}", self.relation_axioms.iter()
            .map(|r| r.to_string()).collect::<Vec<String>>().join(", "));
        let replacements = format!("Replacements: {}", self.replacements.iter()
            .map(|(x, y)| format!("{} = {}", x.to_string(), y.to_string()))
            .collect::<Vec<String>>().join(", "));

        write!(fmt, "Model:\n - {}\n - {}\n - {}\n - {}\n", individuals, concepts, relations, replacements)
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
