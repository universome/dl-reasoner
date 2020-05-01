# Tableau algorithm for ALCQ
This is a tableau-based reasoning algorithm for ALCQ description logic, implemented in rust.
For information on ALCQ and the corresponding tableau algorithm refer to [An Overview of Tableau Algorithms for Description Logics](https://www.jstor.org/stable/20016336?seq=1#metadata_info_tab_contents).

# Features
- Checking for subsumption and consistency
- Supported expansion rules:
    - "and"-rule expansion
    - "or"-rule expansion
    - "only"-rule expansion (i.e. for a universal quantifier)
    - "some"-rule expansion (i.e. for an existential quantifier)
    - "at-least"-rule expansion
    - "at-most"-rule expansion
    - "choose"-rule expansion
    - GCI expansion
- Parsing from files with a convenient input format
- Conversion to NNF
- (Quite) arbitrary concept/relation names
- Interdependent definitions expansion
- Concept definitions and concept subsumptions in TBox
- Blocking with caring about the order (to prevent cycling blocking)
- It feels fast (but I have not tested it on large datasets)

# Installation
To install the library, you should first [install rust and cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).
After that, just pull the project and build it with cargo (omit `--realease` flag if you want to build a debug version, but note that it goes without any optimizations):
```
git clone https://github.com/universome/dl-reasoner
cd dl-reasoner
cargo build --release
```

# Usage
Note: refer to [examples section](#examples) for more information.
For simplicity, below we assume that `dl-reasoner` executable is located in `./target/release/dl-reasoner` (since it is located there by default after running the installation).

#### Checking consistency
To check for consistency you should run `check-consistency` subcommand and provide two arguments: a path to a file with ABox axioms and a path to a file with TBox axioms.
```
./target/release/dl-reasoner check-consistency path-to-abox.txt path-to-tbox.txt
```

#### Checking subsumption
To check if a subsumption is valid, you should put the subsumption you want to check into your `path-to-tbox.txt` file and run `check-subsumption` subcommand:
```
./target/release/dl-reasoner check-subsumption path-to-tbox.txt
```

# Input format
Note: refer to [examples](#examples) for more details.

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
Notes:
- if you use non-atomic concepts somewhere (like `(<= 2 isChildOf IsProfessor)` in the example above), you must wrap them into brackets!
- Top concept has name `__TOP__`

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

# Examples
## Checking consistency
### Example 1
Imagine, that we have the following ABox:
```
ABox = {hasChild(joe,ann), hasChild(joe,eva), hasChild(joe,mary), ParentWithMax2Children(joe)}
```
with respect to TBox:
```
TBox = {ParentWithMax2Children ≡≤ 2HasChild.⊤}
```

We want to check our ABox for consistency and find a corresponding model (if such exists).
For this, we convert ABox in a way that is described in [input format section](#input-format):
```
hasChild[joe, ann]
hasChild[joe, eva]
hasChild[joe, mary]
ParentWithMax2Children[joe]
```
And we write TBox as:
```
ParentWithMax2Children == (<= 2 hasChild __TOP__)
```
After that we run the reasoner with `check-consistency` command:
```
./target/release/dl-reasoner check-consistency examples/find-model-1/abox.txt examples/find-model-1/tbox.txt
```

This finds a model for us:
```
[INFO] Expanding TBox definitions...
[INFO] Applying expanded TBox definitions to an ABox...
[INFO] Applying expanded TBox definitions to GCIs...
[INFO] Aggregating GCIs into a single one...
[INFO] Found a model!
[INFO] Model:
 - Individuals: mary, joe, ann
 - Concepts:
 - Relations: hasChild(joe, ann), hasChild(joe, mary)
 - Replacements: eva = mary
[INFO] Running time: 1.412092ms
```
As one can see, we did replacement "eva -> mary" which made our ABox consistent.
Corresponding `abox.txt` and `tbox.txt` are located in `examples/find-model-1` directory.

### Example 2
Imagine now, that we are given the following ABox and TBox:
```
ABox = {r(a,b), r(b,d), r(d,c), r(a,c), r(c,d), A(d)}
TBox = {}
```
And we want to check if individual `a` an instance of the concept:
```
∃r.((A ⊓ ∃r.A) ⊔ (¬A ⊓ ∃r.∃r.¬A))
```
For this, we extend our ABox with the concept axiom:
```
ABox = ABox ∪ {(∃r.((A ⊓ ∃r.A) ⊔ (¬A ⊓ ∃r.∃r.¬A)))(a)}
```
and check it for consistency.
Corresponding `abox.txt` and `tbox.txt` are provided in `examples/find-model-2` directory.
Running `check-consistency` command
```
./target/release/dl-reasoner check-consistency examples/find-model-1/abox.txt examples/find-model-1/tbox.txt
```
gives output:
```
[INFO] Expanding TBox definitions...
[INFO] Applying expanded TBox definitions to an ABox...
[INFO] Applying expanded TBox definitions to GCIs...
[INFO] Aggregating GCIs into a single one...
[INFO] No model was found.
[INFO] Running time: 5.080364ms
```
Which means that `a` is not an instance of that concept.

## Checking subsumption
In the examples below we will assume that our "real" TBox is empty.
If you want ot use a non-empty one, then you should add the relevant definitions and negated GCIs into the same tbox.txt file with your subsumption (sorry for that).

### Example 1
Imagine, that we want to check consistency of the following subsumption with respect to an empty TBox:
```
∀r.∀s.A ⊓ ∃r.∀s.B ⊓ ∀r.∃s.C ⊑ ∃r.∃s.(A ⊓ B ⊓ C)
```
Again, we write `tbox.txt` the following way (as described in [input format section](#input-format)):
```
(and ((only r (only s A)) (some r (only s B)) (only r (some s C)))) -> some r (some s (and (A B C)))
```
and then we run the reasoner with `check-subsumption` command:
```
./target/release/dl-reasoner check-subsumption examples/subsumption-1/tbox.txt
```
Our reasoner has successfully checked the subsumption:
```
[INFO] Expanding TBox definitions...
[INFO] Applying expanded TBox definitions to GCIs...
[INFO] Aggregating GCIs into a single one...
[INFO] Subsumption is valid.
[INFO] Running time: 2.881441ms
```

### Example 2
We want to check the following subsumption:
```
∀r.∀s.A ⊓ (∃r.∀s.¬A ⊔ ∀r.∃s.B) ⊑ ∀r.∃s.(A ⊓ B) ⊔ ∃r.∀s.¬B
```
Adding missing brackets and translating it to the format that is suitable for parsing, we get the following `tbox.txt`:
```
and ((only r (only s A)) (or ((some r (only s not A)) (only r (some s B))))) -> or ((only r (some s (and (A B)))) (some r (only s (not B))))
```
Now, we can run the reasoning:
```
./target/release/dl-reasoner check-subsumption examples/subsumption-2/tbox.txt
```

In this example, the provided subsumption is also valid and we get the output:
```
[INFO] Expanding TBox definitions...
[INFO] Applying expanded TBox definitions to GCIs...
[INFO] Aggregating GCIs into a single one...
[INFO] Subsumption is valid.
[INFO] Running time: 7.958508ms
```

As one can noted, all these examples ran in under 10ms which I suppose is quite fast.

### TODO
- tests
- print error messages instead of just panicking
- remove unnecessary heap allocations
- backtracking
