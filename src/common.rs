use std::rc::Rc;

#[derive(Debug)]
pub struct Relation { pub name: String }

#[derive(Debug)]
pub struct Individual { pub name: String }

#[derive(Debug)]
pub enum Concept {
    Atomic(AtomicConcept),
    Not(NotConcept),
    Conjunction(ConjunctionConcept),
    Disjunction(DisjunctionConcept),
    Only(OnlyConcept),
    Some(SomeConcept)
}

#[derive(Debug)]
pub struct AtomicConcept {
    pub name: String
}

#[derive(Debug)]
pub struct NotConcept {
    pub concept: Rc<Concept>
}

#[derive(Debug)]
pub struct ConjunctionConcept {
    pub concepts: Rc<Vec<Concept>>
}

#[derive(Debug)]
pub struct DisjunctionConcept {
    pub concepts: Rc<Vec<Concept>>
}

#[derive(Debug)]
pub struct OnlyConcept {
    pub concept: Rc<Concept>,
    pub relation: Relation
}

#[derive(Debug)]
pub struct SomeConcept {
    pub concept: Rc<Concept>,
    pub relation: Relation
}



pub fn parse_concept(concept_str: &str) -> Concept {
    // Parses concept or panics if the string is not a correct concept
    // let mut words = concept_str.split(' ').collect();
    let concept_str = concept_str.trim();

    println!("Parsing concept: {}", concept_str);

    if &concept_str[..1] == "(" {
        // Our concept is wrapped up into brackets "(..)"
        parse_concept(&concept_str[1..(concept_str.len() - 1)])
    } else if concept_str.len() > 3 && &concept_str[..3] == "and" {
        // println!("It is and!");
        Concept::Conjunction(ConjunctionConcept { concepts: Rc::new(extract_concepts(&concept_str[3..])) })
    } else if concept_str.len() > 2 && &concept_str[..2] == "or" {
        // println!("It is or!");
        Concept::Disjunction(DisjunctionConcept { concepts: Rc::new(extract_concepts(&concept_str[2..])) })
    } else if concept_str.len() > 4 && &concept_str[..4] == "only" {
        // println!("It is only!");
        Concept::Only(OnlyConcept {
            relation: Relation {name: concept_str.chars().nth(5).unwrap().to_string()},
            concept: Rc::new(parse_concept(&concept_str[6..]))
        })
    } else if concept_str.len() > 4 && &concept_str[..4] == "some" {
        // println!("It is some!");
        Concept::Some(SomeConcept {
            relation: Relation {name: concept_str.chars().nth(5).unwrap().to_string()},
            concept: Rc::new(parse_concept(&concept_str[6..]))
        })
    } else if concept_str.len() > 3 && &concept_str[..3] == "not" {
        // println!("It is not!");
        Concept::Not(NotConcept { concept: Rc::new(parse_concept(&concept_str[2..])) })
    } else {
        // println!("It is an atomic concept!");
        // This is an Atomic Concept!
        Concept::Atomic(AtomicConcept { name: concept_str.to_string() })
    }
}


fn extract_concepts(concepts_str: &str) -> Vec<Concept> {
    // Takes a concepts string, seperated by whitespace and wrapped up in brackets,
    // parses them individually and returns a vector of concepts.
    let concepts_str = concepts_str.trim();
    println!("Extractinc concepts: {}", concepts_str);
    let mut concepts: Vec<Concept> = Vec::new();
    let mut curr_depth = 0;
    let mut curr_concept_start_idx = 0;
    let mut i = 0;

    while i < concepts_str.len() {
        if &concepts_str[i..i + 1] == "(" {
            curr_depth += 1; // Going a level deeper
        } else if &concepts_str[i..i + 1] == ")" {
            curr_depth -= 1; // Going a level out
        }

        if curr_depth == 0 {
            println!("Found concept: {}", &concepts_str[curr_concept_start_idx .. i + 1]);
            concepts.push(parse_concept(&concepts_str[curr_concept_start_idx .. i + 1]));
            curr_concept_start_idx = i + 1; // Next concept starts on the next character
            i += 1;
        }

        i += 1;
    }
    // for (i, c) in concepts_str.chars().enumerate() {
    //     if c == '(' {
    //         curr_depth += 1; // Going a level deeper
    //     } else if c == ')' {
    //         curr_depth -= 1; // Going a level out
    //     }

    //     if curr_depth == 0 {
    //         println!("Found concept: {}", &concepts_str[curr_concept_start_idx..i+1]);
    //         concepts.push(parse_concept(&concepts_str[curr_concept_start_idx..i+1]));
    //         curr_concept_start_idx = i; // Next concept starts on the next character
    //     }
    // }

    debug_assert!(concepts.len() > 0);

    concepts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_concepts() {
        assert_eq!(extract_concepts("C"), vec![AtomicConcept {name: "C"}]);
    }
}
