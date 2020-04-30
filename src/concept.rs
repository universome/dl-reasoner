use std::fmt;
use std::hash;
use std::clone::Clone;
use std::any::{Any, TypeId};
use std::marker::Sized;


fn extract_concepts(concepts_str: &str) -> Vec<Box<dyn Concept>> {
    // Takes a concepts string, seperated by whitespace and wrapped up in brackets,
    // parses them individually and returns a vector of concepts.
    let mut concepts_str = concepts_str.trim();
    debug!("Extracting concepts: {}", concepts_str);
    let mut concepts: Vec<Box<dyn Concept>> = Vec::new();
    let mut curr_depth = 0;
    let mut i = 0;

    while concepts_str.len() > 0 {
        if &concepts_str[i..(i+1)] == "(" {
            curr_depth += 1; // Going a level deeper
        } else if &concepts_str[i..(i+1)] == ")" {
            curr_depth -= 1; // Going a level out
        }

        if curr_depth == 0 {
            // Ok, we should extract something, but we have two alternatives:
            // - this is a compound concept of the form "(My .. Compound .. Concept)"
            // - this is an atomic concept of the form "MyConcept"
            let concept_str;

            if &concepts_str[..1] == "(" {
                assert!(&concepts_str[i..(i+ 1)] == ")", "Compound concept is in a bad format: {}", concepts_str);
                concept_str = &concepts_str[..(i+1)];
                debug!("Found a compound concept: {}", concept_str);
                concepts.push(parse_concept(concept_str));
            } else {
                let space_idx = concepts_str.chars().position(|c| c == ' ').unwrap_or(concepts_str.len());
                concept_str = &concepts_str[..space_idx];
                debug!("Found an atomic concept: {}", concept_str);
                concepts.push(parse_concept(concept_str));
            }

            i = 0;
            concepts_str = concepts_str[concept_str.len()..].trim();
        } else {
            i += 1;
        }
    }

    debug_assert!(concepts.len() > 0);

    concepts
}


pub fn parse_concept(concept_str: &str) -> Box<dyn Concept> {
    // Parses concept or panics if the string is not a correct concept
    // let mut words = concept_str.split(' ').collect();
    let concept_str = concept_str.trim();

    debug!("Parsing concept: {}", concept_str);

    if &concept_str[..1] == "(" {
        // Our concept is wrapped up into brackets "(..)"
        // debug!("Parsing concept result: {}", parse_concept(&concept_str[1..(concept_str.len() - 1)]));
        parse_concept(&concept_str[1..(concept_str.len() - 1)])
    } else if concept_str.len() > 3 && &concept_str[..3] == "and" {
        // Concept has the format "and (A B C)"
        let subconcepts = extract_concepts(&concept_str[5..concept_str.len()-1]);
        assert!(subconcepts.len() >= 2, "Too few subconcepts inside a conjunction concept: {}", concept_str);
        // debug!("Parsing concept result: {}", Box::new(ConjunctionConcept { subconcepts: subconcepts.clone() }));
        Box::new(ConjunctionConcept { subconcepts: subconcepts })
    } else if concept_str.len() > 2 && &concept_str[..2] == "or" {
        // Concept has the format "or (A B C)"
        let subconcepts = extract_concepts(&concept_str[4..concept_str.len()-1]);
        assert!(subconcepts.len() >= 2, "Too few subconcepts inside a conjunction concept: {}", concept_str);
        // debug!("Parsing concept result: {}", Box::new(DisjunctionConcept { subconcepts: subconcepts.clone() }));
        Box::new(DisjunctionConcept { subconcepts: subconcepts })
    } else if concept_str.len() > 4 && &concept_str[..4] == "only" {
        let space_idx = concept_str[5..].chars().position(|c| c == ' ')
            .expect(&format!("Bad format for `only` concept: {}", concept_str));
        let relation_name = concept_str[5..5 + space_idx].to_string();

        Box::new(OnlyConcept {
            subconcept: parse_concept(&concept_str[(5 + &relation_name.len() + 1)..]),
            relation: Relation {name: relation_name},
        })
    } else if concept_str.len() > 4 && &concept_str[..4] == "some" {
        let space_idx = concept_str[5..].chars().position(|c| c == ' ')
            .expect(&format!("Bad format for `only` concept: {}", concept_str));
        let relation_name = concept_str[5..5 + space_idx].to_string();

        Box::new(SomeConcept {
            subconcept: parse_concept(&concept_str[(5 + &relation_name.len() + 1)..]),
            relation: Relation {name: relation_name},
        })
    } else if concept_str.len() > 3 && &concept_str[..3] == "not" {
        Box::new(NotConcept { subconcept: parse_concept(&concept_str[3..]) })
    } else if concept_str.len() > 2 && &concept_str[..2] == ">=" {
        let concept_str = concept_str[2..].trim(); // Has the from "2 r C" now

        // Extract amount
        let space_idx = concept_str.chars().position(|c| c == ' ')
            .expect(&format!("Bad format to extract amount from `at_least` concept: {}", concept_str));
        let amount = concept_str[..space_idx].parse::<usize>()
            .expect(&format!("Bad format of `amount` for `at_least` concept: {}", concept_str));

        // Extract relation name
        let concept_str = concept_str[space_idx..].trim(); // Has the form "relation Concept" now
        let space_idx = concept_str.chars().position(|c| c == ' ')
            .expect(&format!("Bad format to extract relation from `at_least` concept: {}", concept_str));
        let relation_name = concept_str[..space_idx].to_string();

        debug!("AtMost amount: {}", amount);
        debug!("AtMost relation_name: {}", relation_name);

        Box::new(AtLeastConcept {
            amount: amount,
            subconcept: parse_concept(&concept_str[(&relation_name.len() + 1)..]),
            relation: Relation {name: relation_name}
        })
    } else if concept_str.len() > 3 && &concept_str[..3] == "not" {
        // TODO: this is too similar to at-least concept parsing...
        let concept_str = concept_str[2..].trim(); // Has the from "2 r C" now

        // Extract amount
        let space_idx = concept_str.chars().position(|c| c == ' ')
            .expect(&format!("Bad format to extract amount from `at_most` concept: {}", concept_str));
        let amount = concept_str[..space_idx].parse::<usize>()
            .expect(&format!("Bad format of `amount` for `at_most` concept: {}", concept_str));

        // Extract relation name
        let concept_str = concept_str[space_idx..].trim(); // Has the form "myRelation MyConcept" now
        let space_idx = concept_str.chars().position(|c| c == ' ')
            .expect(&format!("Bad format to extract relation from `at_most` concept: {}", concept_str));
        let relation_name = concept_str[..space_idx].to_string();

        debug!("AtMost amount: {}", amount);
        debug!("AtMost relation_name: {}", relation_name);

        Box::new(AtMostConcept {
            amount: amount,
            subconcept: parse_concept(&concept_str[(&relation_name.len() + 1)..]),
            relation: Relation {name: relation_name}
        })
    } else {
        // This is an Atomic Concept!
        Box::new(AtomicConcept { name: concept_str.to_string() })
    }
}


#[derive(PartialEq)]
pub enum ConceptType {
    Atomic,
    Not,
    Conjunction,
    Disjunction,
    Only,
    Some,
    AtLeast,
    AtMost
}

pub trait Concept: fmt::Debug + fmt::Display + mopa::Any + ConceptClone {
    fn convert_to_nnf(&self) -> Box<dyn Concept>;
    fn concept_type(&self) -> ConceptType;

    fn negate(&self) -> Box<dyn Concept> {
        // Box::new(NotConcept{ subconcept: Box::new(self.clone()) })
        Box::new(NotConcept{ subconcept: Box::new(self).clone_box() })
    }

    fn replace_concept(&self, concept_old: Box<dyn Concept>, concept_new: Box<dyn Concept>) -> Box<dyn Concept>;
}

mopafy!(Concept);

impl hash::Hash for dyn Concept {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.to_string().hash(hasher);
    }
}

impl PartialEq for dyn Concept {
    fn eq(&self, other: &dyn Concept) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for dyn Concept {}

pub trait ConceptClone {
    fn clone_box(&self) -> Box<dyn Concept>;
}

impl<T> ConceptClone for T where T: Concept + Clone {
    fn clone_box(&self) -> Box<dyn Concept> { Box::new(self.clone()) }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn Concept> {
    fn clone(&self) -> Box<dyn Concept> { self.clone_box() }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Relation { pub name: String }

impl fmt::Display for Relation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}(x, y)", self.name)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Individual { pub name: String }


impl Individual {
    pub fn is_younger(&self, other: &Individual) -> bool {
        // Checks if lhs is possible younger than other

        if !self.name.starts_with("_#") {
            return false; // This is an original (ancient) individual
        } else if other.name.starts_with("_#") {
            return true;
        } else {
            let self_num = self.name[3..].parse::<usize>().unwrap();
            let rhs_num = other.name[3..].parse::<usize>().unwrap();

            // greater number => self has appeared later
            self_num > rhs_num
        }
    }
}

impl fmt::Display for Individual {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

#[derive(Debug, Clone, Hash)]
pub struct AtomicConcept { name: String }

impl Concept for AtomicConcept {
    fn convert_to_nnf(&self) -> Box<dyn Concept> {
        Box::new(self.clone())
    }
    fn concept_type(&self) -> ConceptType { ConceptType::Atomic }

    fn replace_concept(&self, concept_old: Box<dyn Concept>, concept_new: Box<dyn Concept>) -> Box<dyn Concept> {
        match self.to_string() == concept_old.to_string() {
            true => concept_new,
            false => Box::new(self.clone()) as Box<dyn Concept>
        }
    }
}

impl fmt::Display for AtomicConcept {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.name)
    }
}

#[derive(Debug, Clone, Hash)]
pub struct NotConcept {
    pub subconcept: Box<dyn Concept>
}

impl Concept for NotConcept {
    fn convert_to_nnf(&self) -> Box<dyn Concept> {
        match self.subconcept.concept_type() {
            ConceptType::Atomic => {
                let subconcept = self.subconcept.downcast_ref::<AtomicConcept>().unwrap();
                Box::new(NotConcept { subconcept: Box::new(subconcept.clone()) })
            },
            ConceptType::Not => {
                let subconcept = self.subconcept.downcast_ref::<NotConcept>().unwrap();
                subconcept.subconcept.convert_to_nnf()
            },
            ConceptType::Conjunction => {
                // not and (A B C) => or ((not A) (not B) (not C))
                let subconcept = self.subconcept.downcast_ref::<ConjunctionConcept>().unwrap();
                // Box::new(AtomicConcept { name: "123".to_string() })
                Box::new(DisjunctionConcept {
                    subconcepts: subconcept.clone().subconcepts.iter()
                    .map(|c| { c.negate() })
                    .map(|c| {c.convert_to_nnf()})
                    .collect()
                })
            },
            ConceptType::Disjunction => {
                // not [or (A B C)] => and ((not A) (not B) (not C))
                let subconcept = self.subconcept.downcast_ref::<DisjunctionConcept>().unwrap();
                Box::new(ConjunctionConcept {
                    subconcepts: subconcept.clone().subconcepts.iter()
                    .map(|c| { Box::new(NotConcept{ subconcept: c.clone() }) })
                    .map(|c| {c.convert_to_nnf()})
                    .collect()
                })
            },
            ConceptType::Only => {
                // not [only A] => some [not A]
                let subconcept = self.subconcept.downcast_ref::<OnlyConcept>().unwrap();
                Box::new(SomeConcept {
                    relation: subconcept.relation.clone(),
                    subconcept: subconcept.subconcept.negate().convert_to_nnf()
                })
            },
            ConceptType::Some => {
                // not [some A] => only [not A]
                let subconcept = self.subconcept.downcast_ref::<SomeConcept>().unwrap();
                Box::new(OnlyConcept {
                    relation: subconcept.relation.clone(),
                    subconcept: subconcept.subconcept.negate().convert_to_nnf()
                })
            },
            ConceptType::AtLeast => {
                // not [some A] => only [not A]
                let subconcept = self.subconcept.downcast_ref::<AtLeastConcept>().unwrap();
                Box::new(AtMostConcept {
                    amount: subconcept.amount - 1,
                    relation: subconcept.relation.clone(),
                    subconcept: subconcept.subconcept.convert_to_nnf()
                })
            },
            ConceptType::AtMost => {
                // not [some A] => only [not A]
                let subconcept = self.subconcept.downcast_ref::<AtMostConcept>().unwrap();
                Box::new(AtLeastConcept {
                    amount: subconcept.amount + 1,
                    relation: subconcept.relation.clone(),
                    subconcept: subconcept.subconcept.convert_to_nnf()
                })
            }
        }
    }

    fn concept_type(&self) -> ConceptType { ConceptType::Not }

    fn negate(&self) -> Box<dyn Concept> {
        self.subconcept.clone() as Box<dyn Concept>
    }

    fn replace_concept(&self, concept_old: Box<dyn Concept>, concept_new: Box<dyn Concept>) -> Box<dyn Concept> {
        match self.to_string() == concept_old.to_string() {
            true => concept_new,
            false => Box::new(NotConcept {
                subconcept: self.subconcept.replace_concept(concept_old, concept_new)
            }) as Box<dyn Concept>
        }
    }
}

impl fmt::Display for NotConcept {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "not {}", self.subconcept)
    }
}

#[derive(Debug, Clone, Hash)]
pub struct ConjunctionConcept {
    pub subconcepts: Vec<Box<dyn Concept>>
}

impl Concept for ConjunctionConcept {
    fn convert_to_nnf(&self) -> Box<dyn Concept> {
        Box::new(ConjunctionConcept {
            subconcepts: self.subconcepts.iter().map(|c| { c.convert_to_nnf() }).collect()
        })
    }

    fn concept_type(&self) -> ConceptType { ConceptType::Conjunction }

    fn replace_concept(&self, concept_old: Box<dyn Concept>, concept_new: Box<dyn Concept>) -> Box<dyn Concept> {
        match &self.to_string() == &concept_old.to_string() {
            true => concept_new,
            false => Box::new(ConjunctionConcept {
                subconcepts: self.subconcepts.clone()
                    .iter()
                    .map(|c| c.replace_concept(concept_old.clone(), concept_new.clone()))
                    .collect::<Vec<Box<dyn Concept>>>()
            }) as Box<dyn Concept>
        }
    }
}

impl fmt::Display for ConjunctionConcept {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "and {}", self.subconcepts.iter()
            .map(|sc| format!("({})", sc.to_string())).collect::<Vec<String>>().join(" "))
    }
}

#[derive(Debug, Clone, Hash)]
pub struct DisjunctionConcept {
    pub subconcepts: Vec<Box<dyn Concept>>
}

impl fmt::Display for DisjunctionConcept {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "or {}", self.subconcepts.iter()
            .map(|sc| format!("({})", sc.to_string())).collect::<Vec<String>>().join(" "))
    }
}

impl Concept for DisjunctionConcept {
    fn convert_to_nnf(&self) -> Box<dyn Concept> {
        Box::new(DisjunctionConcept {
            subconcepts: self.subconcepts.iter().map(|c| { c.convert_to_nnf() }).collect()
        })
    }

    fn concept_type(&self) -> ConceptType { ConceptType::Disjunction }

    fn replace_concept(&self, concept_old: Box<dyn Concept>, concept_new: Box<dyn Concept>) -> Box<dyn Concept> {
        match &self.to_string() == &concept_old.to_string() {
            true => concept_new,
            false => Box::new(DisjunctionConcept {
                subconcepts: self.subconcepts.clone()
                    .iter()
                    .map(|c| c.replace_concept(concept_old.clone(), concept_new.clone()))
                    .collect::<Vec<Box<dyn Concept>>>()
            }) as Box<dyn Concept>
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct OnlyConcept {
    pub subconcept: Box<dyn Concept>,
    pub relation: Relation
}

impl Concept for OnlyConcept {
    fn convert_to_nnf(&self) -> Box<dyn Concept> {
        Box::new(OnlyConcept {
            relation: self.relation.clone(),
            subconcept: self.subconcept.convert_to_nnf()
        })
    }

    fn concept_type(&self) -> ConceptType { ConceptType::Only }

    fn replace_concept(&self, concept_old: Box<dyn Concept>, concept_new: Box<dyn Concept>) -> Box<dyn Concept> {
        match self.to_string() == concept_old.to_string() {
            true => concept_new,
            false => Box::new(OnlyConcept {
                relation: self.relation.clone(),
                subconcept: self.subconcept.replace_concept(concept_old, concept_new)
            }) as Box<dyn Concept>
        }
    }
}

impl fmt::Display for OnlyConcept {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "only {} ({})", self.relation.name, self.subconcept)
    }
}

#[derive(Debug, Clone, Hash)]
pub struct SomeConcept {
    pub subconcept: Box<dyn Concept>,
    pub relation: Relation
}

impl Concept for SomeConcept {
    fn convert_to_nnf(&self) -> Box<dyn Concept> {
        Box::new(SomeConcept {
            relation: self.relation.clone(),
            subconcept: self.subconcept.convert_to_nnf()
        })
    }

    fn concept_type(&self) -> ConceptType { ConceptType::Some }

    fn replace_concept(&self, concept_old: Box<dyn Concept>, concept_new: Box<dyn Concept>) -> Box<dyn Concept> {
        match self.to_string() == concept_old.to_string() {
            true => concept_new,
            false => Box::new(SomeConcept {
                relation: self.relation.clone(),
                subconcept: self.subconcept.replace_concept(concept_old, concept_new)
            }) as Box<dyn Concept>
        }
    }
}

impl fmt::Display for SomeConcept {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "some {} ({})", self.relation.name, self.subconcept)
    }
}

#[derive(Debug, Clone, Hash)]
pub struct AtLeastConcept {
    pub amount: usize,
    pub relation: Relation,
    pub subconcept: Box<dyn Concept>
}

impl Concept for AtLeastConcept {
    fn convert_to_nnf(&self) -> Box<dyn Concept> {
        Box::new(AtLeastConcept {
            amount: self.amount,
            relation: self.relation.clone(),
            subconcept: self.subconcept.convert_to_nnf()
        })
    }

    fn concept_type(&self) -> ConceptType { ConceptType::AtLeast }

    fn replace_concept(&self, concept_old: Box<dyn Concept>, concept_new: Box<dyn Concept>) -> Box<dyn Concept> {
        match self.to_string() == concept_old.to_string() {
            true => concept_new,
            false => Box::new(AtLeastConcept {
                amount: self.amount,
                relation: self.relation.clone(),
                subconcept: self.subconcept.replace_concept(concept_old, concept_new)
            }) as Box<dyn Concept>
        }
    }
}

impl fmt::Display for AtLeastConcept {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, ">= {} {} ({})", self.amount, self.relation.name, self.subconcept)
    }
}

#[derive(Debug, Clone, Hash)]
pub struct AtMostConcept {
    pub amount: usize,
    pub relation: Relation,
    pub subconcept: Box<dyn Concept>
}

impl Concept for AtMostConcept {
    fn convert_to_nnf(&self) -> Box<dyn Concept> {
        Box::new(AtMostConcept {
            amount: self.amount,
            relation: self.relation.clone(),
            subconcept: self.subconcept.convert_to_nnf()
        })
    }

    fn concept_type(&self) -> ConceptType { ConceptType::AtMost }

    fn replace_concept(&self, concept_old: Box<dyn Concept>, concept_new: Box<dyn Concept>) -> Box<dyn Concept> {
        match self.to_string() == concept_old.to_string() {
            true => concept_new,
            false => Box::new(AtMostConcept {
                amount: self.amount,
                relation: self.relation.clone(),
                subconcept: self.subconcept.replace_concept(concept_old, concept_new)
            }) as Box<dyn Concept>
        }
    }
}

impl fmt::Display for AtMostConcept {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "<= {} {} ({})", self.amount, self.relation.name, self.subconcept)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_concepts() {
        assert_eq!(extract_concepts("C"), vec![AtomicConcept {name: "C"}]);
    }
}
