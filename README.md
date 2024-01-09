# hrm-optimizer
 an optimizer for the assembly language in the game "Human Resource Machine"

### Features:
 - Dead code elimination
 - Redundant instruction trimming
 - Jump statement simplification

### TO DO:
 - Extend `.hrm` files to include memory layout info (maybe include level number?)
 - Dataflow analysis passes
   - Basic block simplification (re-parse into an AST?)
   - Live variable analysis (how to deal with pointers?)
   - Kildall's method (limit fix point iteration number?)
 - Note all optimizations in made in [the solutions repo](https://github.com/atesgoral/hrm-solutions)
 - Make and formalize a memory model for "indirect access" (pointers)
   - "accessing a tile directly and indirectly in the program is UB"?
   - "all static mem at end, growable at beggining, overwriting is UB"?
 - Formalize optimizer in [Coq](https://coq.inria.fr/) or [Lean](https://github.com/leanprover/lean4)
