use concept::*;
use abox::*;
use tbox::*;


// pub struct Model {
//     individuals: Vec<Individual>,
//     relations: Vec<Relation>,
//     concepts: Vec<(AtomicConcept, Individual)>
// }


pub fn tableau_reasoning(abox: ABox, tbox: TBox) -> Option<ABox> {
    let mut aboxes = vec![abox];

    // TODO: check initial abox for consistency for O(n)
    // just to see that it does not contain immediate inconsistencies
    // Otherwise, we can feed abox via AND rule. Then we can skip the todo above.

    while aboxes.len() > 0 {
        debug!("Current number of aboxes: {}", aboxes.len());
        let abox = aboxes.pop().unwrap();
        let new_aboxes = perform_tableu_reasoning_step(&abox, &tbox);

        if new_aboxes.is_empty() {
            // Hooray! We have terminated! This means, that we are
            debug!("Found a model: {}", abox);
            return Some(abox);
        }

        let non_clashed_aboxes: Vec<ABox> = new_aboxes
            .into_iter()
            .filter(|a| a.is_consistent != Some(false))
            .collect();

        aboxes.extend(non_clashed_aboxes);
    }

    None
}

fn perform_tableu_reasoning_step(abox: &ABox, tbox: &TBox) -> Vec<ABox> {

    let new_abox =  apply_conjunction_rule(abox, tbox);
    if new_abox.is_some() { return vec![new_abox.unwrap()]; }

    let new_aboxes = apply_disjunction_rule(abox, tbox);
    if new_aboxes.len() > 0 { return new_aboxes; }

    let new_abox =  apply_only_rule(abox, tbox);
    if new_abox.is_some() { return vec![new_abox.unwrap()]; }

    let new_abox =  apply_some_rule(abox, tbox);
    if new_abox.is_some() { return vec![new_abox.unwrap()]; }

    vec![]
}


fn apply_conjunction_rule(abox: &ABox, tbox: &TBox) -> Option<ABox> {
    /// This function applies all conjunction axioms it finds at the current level
    /// (since they do not create any additional nodes, we can apply them all at once)
    let conjunction_axioms = extract_concept_axioms(abox, ConceptType::Conjunction);

    if conjunction_axioms.is_empty() {
        return None; // Cannot apply and-rule
    }

    let new_axioms = conjunction_axioms
        .iter()
        .map(|a| {
            let concept = a.concept.downcast_ref::<ConjunctionConcept>().unwrap();
            create_new_axioms(concept.subconcepts.clone(), a.individual.clone(), abox)
        })
        .find(|new_axioms| !new_axioms.is_empty());

    if new_axioms.is_none() {
        return None; // We have not found any expandable and-rule
    }

    let new_axioms = new_axioms.unwrap();
    let mut new_abox = abox.clone();
    let num_applicable_axioms = new_axioms
        .iter()
        .map(|a| a.downcast_ref::<ConceptAxiom>().unwrap())
        .map(|a| ConceptAxiom { concept: a.concept.negate(), individual: a.individual.clone() }) // Negating
        .map(|a| Box::new(a) as Box<dyn ABoxAxiom>)
        .filter(|a| !new_abox.axioms.contains(&*a))
        .count();

    if num_applicable_axioms != new_axioms.len() {
        // Conjunction rule was applied and we got an incosistent abox
        new_abox.is_consistent = Some(false);
    }

    new_abox.axioms.extend(new_axioms);

    Some(new_abox)
}


fn apply_disjunction_rule(abox: &ABox, tbox: &TBox) -> Vec<ABox> {
    /// This function expands a single disjunction rule among all the disjunction rules
    /// it finds at the current level. It expands the first one expandable.
    let disjunction_axioms = extract_concept_axioms(abox, ConceptType::Disjunction);

    if disjunction_axioms.is_empty() {
        return vec![]; // Cannot apply the rule
    }

    for axiom in disjunction_axioms {
        let concept = axiom.concept.downcast_ref::<DisjunctionConcept>().unwrap();
        let new_axioms = create_new_axioms(concept.subconcepts.clone(), axiom.individual.clone(), abox);

        if new_axioms.len() < concept.subconcepts.len() {
            // Some of the axioms are already in the abox which means
            // that we cannot expand with or-rule here.
            continue;
        }

        // Ok, good. We can now expand with the or-rule!
        return new_axioms
            .into_iter()
            .map(|a| create_new_abox_from_concept_axiom(a, abox))
            .collect::<Vec<ABox>>();
    }

    vec![]
}


fn apply_only_rule(abox: &ABox, tbox: &TBox) -> Option<ABox> {
    let only_axioms = extract_concept_axioms(abox, ConceptType::Only);

    if only_axioms.is_empty() {
        return None;
    }

    for axiom in only_axioms {
        let concept = axiom.concept.downcast_ref::<OnlyConcept>().unwrap();
        let other_individuals = extract_relation_rhs_individuals(&concept.relation, &axiom.individual, abox);
        let new_axiom = other_individuals
            .into_iter()
            .map(|y| Box::new(ConceptAxiom {concept: Box::new(concept.clone()), individual: y}) as Box::<dyn ABoxAxiom>)
            .find(|a| !abox.axioms.contains(a));

        if new_axiom.is_none() {
            continue;
        }

        return Some(create_new_abox_from_concept_axiom(new_axiom.unwrap(), abox))
    }

    None
}


fn apply_some_rule(abox: &ABox, tbox: &TBox) -> Option<ABox> {
    let some_axioms = extract_concept_axioms(abox, ConceptType::Some);

    if some_axioms.is_empty() {
        return None;
    }

    for axiom in some_axioms {
        let concept = axiom.concept.downcast_ref::<SomeConcept>().unwrap();
        let other_individuals = extract_relation_rhs_individuals(&concept.relation, &axiom.individual, abox);
        let other_individuals_concepts = other_individuals
            .into_iter()
            .map(|y| Box::new(ConceptAxiom {concept: Box::new(concept.clone()), individual: y}) as Box::<dyn ABoxAxiom>)
            .find(|a| abox.axioms.contains(a));

        if other_individuals_concepts.is_some() {
            continue;
        }

        let new_individual = Individual { name: format!("x_{}", abox.individuals.len()) };
        debug!("Creating new individual: {}", new_individual.name );

        let new_axiom = Box::new(ConceptAxiom {
            concept: Box::new(concept.clone()) as Box<dyn Concept>,
            individual: new_individual.clone()
        }) as Box<dyn ABoxAxiom>;

        let mut new_abox = create_new_abox_from_concept_axiom(new_axiom, abox);
        new_abox.individuals.insert(new_individual);

        return Some(new_abox);
    }

    None
}


fn extract_concept_axioms<'a>(abox: &'a ABox, concept_type: ConceptType) -> Vec<&'a ConceptAxiom> {
    abox.axioms
        .iter()
        .filter(|a| a.axiom_type() == ABoxAxiomType::Concept)
        .map(|a| a.downcast_ref::<ConceptAxiom>().unwrap())
        .filter(|a| a.concept.concept_type() == concept_type)
        .collect::<Vec<&ConceptAxiom>>()
}


fn create_new_axioms(concepts: Vec<Box<dyn Concept>>, individual: Individual, abox: &ABox) -> Vec<Box<dyn ABoxAxiom>> {
    concepts.into_iter()
        // Convert to an axiom
        .map(|sc| ConceptAxiom {concept: sc, individual: individual.clone() })
        .map(|a| Box::new(a) as Box<dyn ABoxAxiom>)
        // Remove axioms that we already have
        .filter(|a| !abox.axioms.contains(&*a))
        .collect()
}


fn create_new_abox_from_concept_axiom(axiom: Box<dyn ABoxAxiom>, abox: &ABox) -> ABox {
    debug_assert!(!abox.axioms.contains(&axiom));

    let mut new_abox = abox.clone();
    let concept_axiom = axiom.downcast_ref::<ConceptAxiom>().unwrap();
    let negated_axiom = Box::new(ConceptAxiom {
        concept: concept_axiom.concept.negate(),
        individual: concept_axiom.individual.clone()
    }) as Box<dyn ABoxAxiom>;

    new_abox.axioms.insert(axiom);

    if new_abox.axioms.contains(&negated_axiom) {
        new_abox.is_consistent = Some(false);
    }

    new_abox
}


fn extract_relation_rhs_individuals(relation: &Relation, individual: &Individual, abox: &ABox) -> Vec<Individual> {
    return abox.axioms
        .iter()
        .filter(|a| a.axiom_type() == ABoxAxiomType::Relation)
        .map(|a| a.downcast_ref::<RelationAxiom>().unwrap())
        .filter(|ra| &ra.relation == relation && &ra.lhs == individual)
        .map(|ra| ra.rhs.clone())
        .collect::<Vec<Individual>>()
}
