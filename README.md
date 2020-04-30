### Tableau algorithm for ALCQ
This is a tableau-based reasoning algorithm for ALCQ description logic, implemented in rust.
For information on ALCQ and the corresponding tableau algorithm refer to [An Overview of Tableau Algorithms for Description Logics](https://www.jstor.org/stable/20016336?seq=1#metadata_info_tab_contents).

### Features
- Parsing from files with a convenient input format
- Conversion to NNF
- (Quite) arbitrary concept/relation names
- Supported expansion rules:
    - "and"-rule expansion
    - "or"-rule expansion
    - "only"-rule expansion (i.e. for a universal quantifier)
    - "some"-rule expansion (i.e. for an existential quantifier)
    - "at-least"-rule expansion
    - "at-most"-rule expansion
    - choose-rule expansion
    - GCI expansion
- Interdependent definitions expansion
- Concept definitions and concept subsumptions in TBox
- Blocking with order (to prevent cycling blocking)

### Installation
To install the library, you should first [install rust and cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).
After that, just pull the project and build it with cargo (omit `--realease` flag if you want to build a debug version):
```
git clone https://github.com/universome/dl-reasoner
cd dl-reasoner
cargo build --release
```

### Usage
You should provide two files to the executable: a file with ABox axioms and a a file with TBox axioms.
Examples of ABox and TBox files are located in `data/` directory.
To run the executable, just type a command:
```
./path-to-dl-reasoner-executable path-to-abox-file.txt path-to-tbox-file.txt
```

### Input format
Note: you can always refer to examples for more details.

#### Concept format
ABox and TBox files share the same concept format.
Concepts have the following format:
- atomic concept: `ConceptName`
- conjunction: `and (ConceptA ConceptB ConceptC <...etc...>)`
- disjunction: `or (ConceptA ConceptB ConceptC <...etc...>)`
- negation: `not Concept`
- universal quantifier: `only relationName Concept`
- existential quantifier: `some relationName Concept`
- at-least concept: `>= 123 relationName Concept`
- at-most concept: `<= 123 relationName Concept`

You can aggregate nested concepts with the format above.
For example:
```
and (IsStudent (<= 2 isChildOf IsProfessor) IsHuman)
```
**Important Note**: if you use non-atomic concepts somewhere (like `(<= 2 isChildOf IsProfessor)` in the example above), you must wrap them into brackets!

#### ABox concept axiom format
Concept axioms are based on the concept format and have the format `MyConcept[x]`.
This implies that concept `MyConcept` is appled to individual `x`.
You can have a compound concept axiom, like `(>= 123 myRelation (and (A B C)))[x]`.

#### ABox relation axiom format
Relation axiom is the simplest one.
It has the format `relationName[x,y]` and means that we have a relation `relationName` between individuals `x` and `y`.

#### TBox definition format
Definition in a TBox has the format `ConceptName == SomeConceptDefinition`.

#### TBox inclusion format
Definition in a TBox has the format `SomeConceptA -> SomeConceptB`.

### TODO
- tests
- print error messages instead of just panicking
- remove unnecessary heap allocations
- backtracking
