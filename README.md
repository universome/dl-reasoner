### Tableau algorithm for ALCQ
This is a tableau-based reasoning algorithm for ALCQ description logic, implemented in rust.

For more information on ALCQ and the corresponding tableau algorithm refer to [An Overview of Tableau Algorithms for Description Logics](https://www.jstor.org/stable/20016336?seq=1#metadata_info_tab_contents).

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

### TODO
- tests
- print error messages instead of just panicking
- remove unnecessary heap allocations
- backtracking
