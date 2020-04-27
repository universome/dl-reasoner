use concept::*;
use abox::*;
use tbox::*;


pub struct Model {
    individuals: Vec<Individual>,
    relations: Vec<Relation>
}


pub fn tableau_reasoning(abox: ABox, tbox: TBox) -> Option<Model> {
    None
}

pub fn count_axiom_types(abox: &ABox, desired_concept_type: ConceptType) -> usize {
    abox.axioms.iter().filter(|&axiom| {
        match axiom.axiom_type() {
            ABoxAxiomType::Concept => {
                let concept_axiom = axiom.downcast_ref::<ConceptAxiom>().unwrap();

                concept_axiom.concept.concept_type() == desired_concept_type
            },
            _ => false
        }
    }).count()
}
