use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;

use concept::*;
use abox::*;
use tbox::*;


pub fn tableau_reasoning(abox: ABox, super_gci: Option<ConjunctionConcept>) -> Option<ABox> {
    debug!("\n\n<======== Starting tableau algorithm ========>\n");
    let mut aboxes = vec![abox];

    // TODO: check initial abox for consistency for O(n)
    // just to see that it does not contain immediate inconsistencies
    // Otherwise, we can feed abox via AND rule. Then we can skip the todo above.

    while aboxes.len() > 0 {
        debug!("Current number of aboxes: {}", aboxes.len());
        let abox = aboxes.pop().unwrap();
        let new_aboxes = perform_tableu_reasoning_step(&abox, &super_gci);

        if new_aboxes.is_empty() {
            // Hooray! We have terminated! This means, that we have reached a consistent leave
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

fn perform_tableu_reasoning_step(abox: &ABox, super_gci: &Option<ConjunctionConcept>) -> Vec<ABox> {
    let new_abox =  apply_conjunction_rule(abox);
    if new_abox.is_some() { return vec![new_abox.unwrap()]; }

    let new_aboxes = apply_disjunction_rule(abox);
    if new_aboxes.len() > 0 { return new_aboxes; }

    let new_abox =  apply_only_rule(abox);
    if new_abox.is_some() { return vec![new_abox.unwrap()]; }

    let new_abox =  apply_some_rule(abox);
    if new_abox.is_some() { return vec![new_abox.unwrap()]; }

    vec![]
}


fn apply_conjunction_rule(abox: &ABox) -> Option<ABox> {
    /// This function applies all conjunction axioms it finds at the current level
    /// (since they do not create any additional nodes, we can apply them all at once)
    let conjunction_axioms = extract_concept_axioms(abox, ConceptType::Conjunction);

    if conjunction_axioms.is_empty() {
        debug!("Tried to expand AND rule, but there are no relevant axioms in the abox: {}", abox);
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
        debug!("Tried to expand AND rule, but the expansion is already in ABox.");
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
        debug!("Obtained an incosistency while expanding AND rule.");
    }

    new_abox.axioms.extend(new_axioms);

    debug!("Successfully expanded AND rule.");
    Some(new_abox)
}


fn apply_disjunction_rule(abox: &ABox) -> Vec<ABox> {
    /// This function expands a single disjunction rule among all the disjunction rules
    /// it finds at the current level. It expands the first one expandable.
    let disjunction_axioms = extract_concept_axioms(abox, ConceptType::Disjunction);

    if disjunction_axioms.is_empty() {
        debug!("Tried to expand OR rule, but there are no relevant axioms.");
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
        debug!("Successfully expanded OR rule.");
        return new_axioms
            .into_iter()
            .map(|a| create_new_abox_from_concept_axiom(a, abox))
            .collect::<Vec<ABox>>();
    }

    debug!("All OR axioms are non-expandable.");

    vec![]
}


fn apply_only_rule(abox: &ABox) -> Option<ABox> {
    let only_axioms = extract_concept_axioms(abox, ConceptType::Only);

    if only_axioms.is_empty() {
        debug!("Tried to expand ONLY rule, but there are no relevant axioms.");
        return None;
    }

    for axiom in only_axioms {
        let concept = axiom.concept.downcast_ref::<OnlyConcept>().unwrap();
        let other_individuals = extract_rhs_for_relation(&concept.relation, &axiom.individual, abox);
        let new_axiom = other_individuals
            .into_iter()
            .map(|y| Box::new(ConceptAxiom {
                concept: concept.subconcept.clone() as Box<dyn Concept>,
                individual: y
            }) as Box::<dyn ABoxAxiom>)
            .find(|a| !abox.axioms.contains(a));

        if new_axiom.is_none() {
            continue;
        }

        debug!("Successfully expanded ONLY rule: {} => {}", axiom, new_axiom.clone().unwrap());
        return Some(create_new_abox_from_concept_axiom(new_axiom.unwrap(), abox))
    }

    debug!("All ONLY axioms are non-expandable.");

    None
}


fn apply_some_rule(abox: &ABox) -> Option<ABox> {
    let some_axioms = extract_concept_axioms(abox, ConceptType::Some);

    if some_axioms.is_empty() {
        debug!("Tried to expand SOME rule, but there are no relevant axioms.");
        return None;
    }

    for axiom in some_axioms {
        if check_if_blocked(abox, &axiom.individual) {
            debug!("Tried to expand {}, but it is blocked.", axiom);
            return None;
        }

        let concept = axiom.concept.downcast_ref::<SomeConcept>().unwrap();
        let rhs_individuals = extract_rhs_for_relation(&concept.relation, &axiom.individual, abox);
        debug!("Found rhs individuals: {}", rhs_individuals.iter()
            .map(|x| x.name.to_string()).collect::<Vec<String>>().join(" "));
        let rhs_concept_axiom = rhs_individuals
            .into_iter()
            .map(|y| Box::new(ConceptAxiom {
                concept: concept.subconcept.clone() as Box<dyn Concept>,
                individual: y
            }) as Box::<dyn ABoxAxiom>)
            .find(|a| abox.axioms.contains(a));

        if rhs_concept_axiom.is_some() {
            continue;
        }

        let new_individual = Individual { name: format!("x_#{}", abox.individuals.len()) };
        debug!("Creating new individual: {}", new_individual.name);

        let new_axiom = Box::new(ConceptAxiom {
            concept: concept.subconcept.clone() as Box<dyn Concept>,
            individual: new_individual.clone()
        }) as Box<dyn ABoxAxiom>;

        let mut new_abox = create_new_abox_from_concept_axiom(new_axiom, abox);
        new_abox.individuals.insert(new_individual.clone());
        new_abox.axioms.insert(Box::new(RelationAxiom {
            lhs: axiom.individual.clone(),
            rhs: new_individual.clone(),
            relation: concept.relation.clone()
        }) as Box<dyn ABoxAxiom>);

        debug!("Successfully expanded SOME rule: {}", axiom);
        return Some(new_abox);
    }

    debug!("All SOME axioms are non-expandable.");

    None
}


fn apply_at_least_rule(abox: &ABox) -> Option<ABox> {
    let at_least_axioms = extract_concept_axioms(abox, ConceptType::AtLeast);

    if at_least_axioms.is_empty() {
        debug!("Tried to expand AtLeast rule, but there are no relevant axioms.");
        return None;
    }

    let expandable_axiom = at_least_axioms
        .iter()
        .find(|a| {
            let concept = a.concept.downcast_ref::<AtLeastConcept>().unwrap();

            if check_if_blocked(abox, &a.individual) {
                debug!("Tried to expand {}, but it is blocked.", a);
                return false;
            }

            let possible_rhs: HashSet<Individual> = HashSet::from_iter(
                extract_rhs_for_relation(&concept.relation, &a.individual, abox).iter().cloned());

            // Searching for a set of pairwise different individuals that would satisfy the constraints
            abox.pairwise_different_individuals.iter().find(|&diff_individuals| {
                if diff_individuals.len() < concept.amount {
                    return false;
                }

                diff_individuals.iter().all(|rhs| {
                    let contains_relation = possible_rhs.contains(rhs);
                    let contains_concept = abox.axioms.contains(&(Box::new(ConceptAxiom {
                        individual: a.individual.clone(),
                        concept: concept.subconcept.clone()
                    }) as Box::<dyn ABoxAxiom>));

                    contains_relation && contains_concept
                })
            }).is_none() // I.e. "there are no such c_1, ..., c_n, that ..."
        });

    if expandable_axiom.is_none() {
        debug!("Tried to expand AtLeast rule, but possible expansions are already in ABox.");
        return None;
    }

    let mut new_abox = abox.clone();
    let axiom = expandable_axiom.unwrap();
    let concept = axiom.concept.downcast_ref::<AtLeastConcept>().unwrap();
    let mut new_individuals = HashSet::new();

    for _ in 0..concept.amount {
        let new_individual = Individual { name: format!("x_#{}", new_abox.individuals.len()) };
        debug!("Creating new individual: {}", new_individual.name);

        // Adding the concept
        new_abox.axioms.insert(Box::new(ConceptAxiom {
            concept: concept.subconcept.clone() as Box<dyn Concept>,
            individual: new_individual.clone()
        }) as Box<dyn ABoxAxiom>);

        // Adding the relation
        new_abox.axioms.insert(Box::new(RelationAxiom {
            relation: concept.relation.clone(),
            lhs: axiom.individual.clone(),
            rhs: new_individual.clone()
        }) as Box<dyn ABoxAxiom>);

        new_individuals.insert(new_individual.clone());
        new_abox.individuals.insert(new_individual.clone());
    }

    new_abox.pairwise_different_individuals.push(new_individuals);

    if !is_at_least_concept_valid(&new_abox, &axiom.individual, &concept) {
        new_abox.is_consistent = Some(false);
    }

    Some(new_abox)
}


fn apply_at_most_rule(abox: &ABox) -> Vec<ABox> {
    let at_most_axioms = extract_concept_axioms(abox, ConceptType::AtLeast);

    if at_most_axioms.is_empty() {
        debug!("Tried to expand AtMost rule, but there are no relevant axioms.");
        return vec![];
    }

    for axiom in at_most_axioms {
        let concept = axiom.concept.downcast_ref::<AtMostConcept>().unwrap();
        let others = extract_rhs_for_relation(&concept.relation, &axiom.individual, abox);
        let others_with_concept = filter_by_concept(others, &concept.subconcept, abox);

        if others_with_concept.len() < concept.amount + 1 {
            continue;
        }

        let mut replacements = HashMap::new();

        for y in others_with_concept.clone() {
            // Finding those individuals that do not have y ≠ z relation
            // Since such relations are stored in abox.pairwise_different_individuals field
            // We should iterate over them all to check if there both y and z are in the same set
            // If there is not such a set, then we can use y to replace z
            // and symmetrically replace z instead of y
            // (but this latter case will be caught in an outer loop iteration for z)
            let can_be_equal_to_y = others_with_concept.clone().into_iter().filter(|z| {
                abox.pairwise_different_individuals
                    .iter()
                    .find(|xs| {xs.contains(&y) && xs.contains(&z)})
                    .is_none()
            }).collect::<Vec<Individual>>();

            if !can_be_equal_to_y.is_empty() {
                replacements.insert(y, can_be_equal_to_y);
            }
        }

        if replacements.len() < concept.amount + 1 {
            continue;
        }

        debug!("We have found an AtMost axiom, which can be expanded: {}", axiom);

        // Now, we should keep only n+1 replacements (knowing, that inequality relation is symmetric)
        let mut variables_to_keep: HashSet<Individual> = HashSet::from_iter(replacements
            .keys().map(|k| k.clone()).collect::<Vec<Individual>>()[..concept.amount+1].iter().cloned());

        for x_old in replacements.clone().keys() {
            if variables_to_keep.contains(x_old) {
                replacements.insert(x_old.clone(), replacements[x_old].clone()
                    .into_iter().filter(|x| variables_to_keep.contains(&x)).collect());
            } else {
                replacements.remove(x_old);
            }
        }

        let mut new_aboxes = vec![];

        for (x_old, xs_new) in replacements {
            for x_new in xs_new {
                new_aboxes.push(replace_individual_in_abox(abox, x_old.clone(), x_new));
            }
        }

        debug_assert!(new_aboxes.len() > 0);

        return new_aboxes;
    }

    debug!("Tried to expand AtMost rule, but all possible expansions are already in ABox.");
    vec![]
}


fn apply_choose_rule(abox: &ABox) -> Vec<ABox> {
    let at_most_axioms = extract_concept_axioms(abox, ConceptType::AtLeast);

    if at_most_axioms.is_empty() {
        debug!("Tried to apply choose rule, but there are no relevant axioms.");
        return vec![];
    }

    for axiom in at_most_axioms {
        let concept = axiom.concept.downcast_ref::<AtMostConcept>().unwrap();
        let others = extract_rhs_for_relation(&concept.relation, &axiom.individual, abox);

        for y in others {
            let y_concept = Box::new(ConceptAxiom {
                individual: y.clone(),
                concept: concept.subconcept.clone()
            }) as Box<dyn ABoxAxiom>;

            let y_not_concept = Box::new(ConceptAxiom {
                individual: y.clone(),
                concept: concept.subconcept.negate().convert_to_nnf()
            }) as Box<dyn ABoxAxiom>;

            if !abox.axioms.contains(&y_concept) && !abox.axioms.contains(&y_not_concept) {
                let mut new_abox_y = abox.clone();
                let mut new_abox_y_not = abox.clone();

                new_abox_y.axioms.insert(y_concept);
                new_abox_y_not.axioms.insert(y_not_concept);

                debug!("Successfully appled choose-rule for axiom {} and individual {}", axiom, y);
                return vec![new_abox_y, new_abox_y_not];
            }
        }
    }

    vec![]
}


fn extract_concept_axioms<'a>(abox: &'a ABox, concept_type: ConceptType) -> Vec<&'a ConceptAxiom> {
    abox.axioms
        .iter()
        .filter(|a| a.axiom_type() == ABoxAxiomType::Concept)
        .map(|a| a.downcast_ref::<ConceptAxiom>().unwrap())
        .filter(|a| a.concept.concept_type() == concept_type)
        .collect::<Vec<&ConceptAxiom>>()
}


fn create_new_axioms(concepts: Vec<Box<dyn Concept>>,
                     individual: Individual, abox: &ABox) -> Vec<Box<dyn ABoxAxiom>> {
    concepts
        .into_iter()
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


fn check_if_blocked(abox: &ABox, y: &Individual) -> bool {
    /// Checks if the individual x is blocked by some other individual
    abox.individuals
        .iter()
        .filter(|x| x != &y)
        .find(|x| is_blocking(abox, x, y))
        .is_none()
}


fn is_blocking(abox: &ABox, lhs: &Individual, rhs: &Individual) -> bool {
    // Checks if the individual rhs is blocked by an individual lhs
    // If lhs is younger, then it cannot block rhs
    !lhs.is_younger(rhs) && abox.axioms
        .iter()
        .map(|a| a.downcast_ref::<ConceptAxiom>().unwrap())
        .filter(|ca| &ca.individual == rhs)
        .map(|ca| Box::new(ConceptAxiom {
            concept: ca.concept.clone(),
            individual: lhs.clone()
        }) as Box<dyn ABoxAxiom>)
        .all(|ca| abox.axioms.contains(&ca))
}


fn extract_rhs_for_relation(relation: &Relation, individual: &Individual, abox: &ABox) -> Vec<Individual> {
    return abox.axioms
        .iter()
        .filter(|a| a.axiom_type() == ABoxAxiomType::Relation)
        .map(|a| a.downcast_ref::<RelationAxiom>().unwrap())
        .filter(|ra| &ra.relation == relation && &ra.lhs == individual)
        .map(|ra| ra.rhs.clone())
        .collect::<Vec<Individual>>()
}


fn is_at_least_concept_valid(abox: &ABox, individual: &Individual, concept: &AtLeastConcept) -> bool {
    // at_least concept is valid if there is no at_most concept with the smaller amount
    (1..concept.amount).find(|&n| {
        abox.axioms.contains(&(Box::new(ConceptAxiom {
            concept: Box::new(AtMostConcept {
                subconcept: concept.subconcept.clone(),
                relation: concept.relation.clone(),
                amount: n,
            }) as Box<dyn Concept>,
            individual: individual.clone()
        }) as Box<dyn ABoxAxiom>))
    }).is_none()
}


fn filter_by_concept(individuals: Vec<Individual>,
                     concept: &Box<dyn Concept>, abox: &ABox) -> Vec<Individual> {
    individuals
        .into_iter()
        .filter(|x| {
            abox.axioms.contains(&(Box::new(ConceptAxiom {
                individual: x.clone(),
                concept: concept.clone()
            }) as Box<dyn ABoxAxiom>))
        })
        .collect()
}


fn replace_individual_in_abox(abox: &ABox, x_old: Individual, x_new: Individual) -> ABox {
    debug_assert!(abox.individuals.contains(&x_old));
    debug_assert!(abox.individuals.contains(&x_new));

    let mut new_abox = abox.clone();

    new_abox.axioms = HashSet::from_iter(new_abox.axioms.into_iter().map(|a| {
        match a.axiom_type() {
            ABoxAxiomType::Concept => {
                let concept_axiom = a.downcast_ref::<ConceptAxiom>().unwrap();

                if concept_axiom.individual != x_old {
                    a
                } else {
                    Box::new(ConceptAxiom {
                        concept: concept_axiom.concept.clone(),
                        individual: x_new.clone()
                    })
                }
            },
            ABoxAxiomType::Relation => {
                let relation_axiom = a.downcast_ref::<RelationAxiom>().unwrap();

                if relation_axiom.lhs != x_old && relation_axiom.rhs != x_old {
                    a
                } else if relation_axiom.lhs == x_old {
                    Box::new(RelationAxiom {
                        lhs: x_new.clone(),
                        rhs: relation_axiom.rhs.clone(),
                        relation: relation_axiom.relation.clone()
                    })
                } else if relation_axiom.rhs == x_old {
                    Box::new(RelationAxiom {
                        lhs: relation_axiom.lhs.clone(),
                        rhs: x_new.clone(),
                        relation: relation_axiom.relation.clone()
                    })
                } else {
                    unreachable!();
                }
            }
        }
    }));

    for pairwise_diffs in &mut new_abox.pairwise_different_individuals {
        if !pairwise_diffs.contains(&x_old) {
            continue;
        }

        pairwise_diffs.remove(&x_old);

        if pairwise_diffs.contains(&x_new) {
            new_abox.is_consistent = Some(false);
        } else {
            pairwise_diffs.insert(x_new.clone());
        }
    }

    new_abox
}
