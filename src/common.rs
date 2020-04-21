pub struct Relation { name: String }
pub struct Individual { name: String }
pub enum Concept {
    AtomicConcept,
    ConjunctionConcept,
    DisjunctionConcept,
    OnlyConcept,
    SomeConcept
}

pub struct AtomicConcept { name: String }
pub struct ConjunctionConcept { concepts: Vec<Concept> }
pub struct DisjunctionConcept { concepts: Vec<Concept> }
pub struct OnlyConcept { concept: Concept, relation: Relation }
pub struct SomeConcept { concept: Concept, relation: Relation  }


fn parse_concept(concept_str: &str) -> Concept {
    // Parses concept or panics if the string is not a correct concept
    // let mut words = concept_str.split(' ').collect();
    let concept_str = concept_str.trim();

    if concept_str[0] == "(" {
        // Our concept is wrapped up into brackets "(..)"
        parse_concept(concept_str[1..(concept_str.len() - 1)])
    } else if concept_str[..3] == "and" {
        ConjunctionConcept { concepts: extract_concepts(concept_str[3..]) }
    } else if concept_str[..2] == "or" {
        DisjunctionConcept { concepts: extract_concepts(concept_str[2..]) }
    } else if concept_str[..4] == "only" {
        OnlyConcept { relation: concept_str[5], concept: parse_concept(concept_str[6..]) }
    } else if concept_str[..4] == "some" {
        SomeConcept { relation: concept_str[5], concept: parse_concept(concept_str[6..]) }
    } else if concept_str[..3] == "not" {
        NotConcept { concept: parse_concept(concept_str[2..]) }
    } else {
        // This is an Atomic Concept!
        AtomicConcept { name: String::from(concept_str) }
    }
}


fn extract_concepts(concepts_str: &str) -> Vec<Concept> {
    // Takes a concepts string, seperated by whitespace and wrapped up in brackets,
    // parses them individually and returns a vector of concepts.
    let concepts_str = concepts_str.trim();
    let mut concepts<Vec<Concept>>;
    let mut curr_depth = 0
    let mut curr_concept_start_idx = 0;

    for (i, c) in concepts_str.iter().enumerate() {
        if c == "(" {
            curr_depth += 1; // Going a level deeper
        } else if c == ")" {
            curr_depth -= 1; // Going a level out
        }

        if curr_depth == 0 {
            concepts.push(parse_concept(concept_str[curr_concept_start..i]))
            let mut curr_concept_start = i + 1; // Next concept starts on the next character
        }
    }

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
