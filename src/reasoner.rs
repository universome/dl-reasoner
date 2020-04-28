use concept::*;
use abox::*;
use tbox::*;


pub struct Model {
    individuals: Vec<Individual>,
    relations: Vec<Relation>,
    concepts: Vec<(AtomicConcept, Individual)>
}


pub fn tableau_reasoning(abox: ABox, tbox: TBox) -> Option<Model> {
    // let mut aboxes = vec![abox];

    // while let Some(abox_to_explore) = aboxes.pop() {
    //     let new_aboxes = perform_tableu_reasoning_step(abox, tbox);
    //     aboxes.extend(new_aboxes);
    // }

    None
}

// fn perform_tableu_reasoning_step(abox: ABox, tbox: TBox) -> Option<ABox> {
//     if let Some(new_abox) = expand_with_and_rule(abox, tbox) {
//         Some(new_abox)
//     } else if Some(new_abox) = expand_with_or_rule(abox, tbox) {
//         Some(new_abox)
//     } else if Some(new_abox) = expand_with_only_rule(abox, tbox) {
//         Some(new_abox)
//     } else if Some(new_abox) = expand_with_some_rule(abox, tbox) {
//         Some(new_abox)
//     }

//     None
// }


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


fn apply_conjunction_rule(abox: ABox, tbox: TBox) -> Option<ABox> {
    /// This function applies all conjunction axioms it finds at the current level
    /// (since they do not create any additional nodes, we can apply them all at once)
    let conjunction_axioms: Vec<&ConceptAxiom> = abox.axioms
        .iter()
        .filter(|a| a.axiom_type() == ABoxAxiomType::Concept)
        .map(|a| a.downcast_ref::<ConceptAxiom>().unwrap())
        .filter(|a| a.concept.concept_type() == ConceptType::Conjunction)
        .collect::<Vec<&ConceptAxiom>>();

    if conjunction_axioms.is_empty() {
        return None; // Cannot apply and rule
    }

    let mut new_abox = abox.clone();
    let mut num_new_axioms = 0;

    for axiom in conjunction_axioms {
        let concept = axiom.concept.downcast_ref::<ConjunctionConcept>().unwrap();

        let new_axioms = concept.subconcepts.clone().into_iter()
            // Convert to an axiom
            .map(|sc| ConceptAxiom {concept: sc, individual: axiom.individual.clone() })
            .map(|a| Box::new(a) as Box<dyn ABoxAxiom>)
            // Remove axioms that we already have
            .filter(|a| !abox.axioms.contains(&*a))
            .collect::<Vec<Box<dyn ABoxAxiom>>>();

        let num_applicable_axioms = new_axioms
            // Remove axioms that we cannot apply
            .iter()
            .map(|a| a.downcast_ref::<ConceptAxiom>().unwrap())
            .map(|a| ConceptAxiom { concept: a.concept.negate(), individual: a.individual.clone() }) // Negating
            .map(|a| Box::new(a) as Box<dyn ABoxAxiom>)
            .filter(|a| !abox.axioms.contains(&*a))
            .count();

        if num_applicable_axioms == new_axioms.len() {
            num_new_axioms += num_applicable_axioms;
            new_abox.axioms.extend(new_axioms);
        } else {
            // Conjunction rule was applied and we got an incosistent abox
            new_abox.is_consistent = Some(false);
            return Some(new_abox);
        }
    }

    match num_new_axioms {
        0 => None, // Cannot expand anything with the and-rule
        _ => Some(new_abox) // We have successfully applied and-rule
    }
}
