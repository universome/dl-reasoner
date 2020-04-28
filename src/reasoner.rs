use concept::*;
use abox::*;
use tbox::*;


pub struct Model {
    individuals: Vec<Individual>,
    relations: Vec<Relation>,
    concepts: Vec<(AtomicConcept, Individual)>
}


pub fn tableau_reasoning(abox: ABox, tbox: TBox) -> Option<Model> {
    None
}


// pub fn count_axiom_types(abox: &ABox, desired_concept_type: ConceptType) -> usize {
//     abox.axioms.iter().filter(|&axiom| {
//         match axiom.axiom_type() {
//             ABoxAxiomType::Concept => {
//                 let concept_axiom = axiom.downcast_ref::<ConceptAxiom>().unwrap();

//                 concept_axiom.concept.concept_type() == desired_concept_type
//             },
//             _ => false
//         }
//     }).count()
// }

fn expand_with_and_rule(abox: ABox, tbox: TBox) -> Option<ABox> {
    // Ok, we have at least one conjunction
    let conjunction_axioms: Vec<&ConceptAxiom> = abox.axioms
        .iter()
        .filter(|a| a.axiom_type() == ABoxAxiomType::Concept)
        .map(|a| a.downcast_ref::<ConceptAxiom>().unwrap())
        .filter(|a| a.concept.concept_type() == ConceptType::Conjunction)
        .collect::<Vec<&ConceptAxiom>>();

    for axiom in conjunction_axioms {
        let concept = axiom.concept.downcast_ref::<ConjunctionConcept>().unwrap();
        let new_axioms = concept.subconcepts.clone().into_iter()
            .map(|sc| ConceptAxiom {concept: sc, individual: axiom.individual.clone() })
            .map(|a| Box::new(a) as Box<dyn ABoxAxiom>)
            .filter(|a| !abox.axioms.contains(&*a))
            .collect::<Vec<Box<dyn ABoxAxiom>>>();

        if !new_axioms.is_empty() {
            let mut new_abox = abox.clone();
            new_abox.axioms.extend(new_axioms);

            return Some(new_abox)
        }
    }

    None
}
