# r1thm

R1thm is a Rust library for R1CS (Rank-1 Constraint Systems). The purpose is for the library author to learn about R1CS, and the library is not intended for production use.

The main functionality of R1thm is the function `poly2r1cs`, which takes a polynomial and converts it into a rank-1 constraint system. The polynomial is parsed using a pest parser, and the visitor pattern is used to traverse the parse tree and generate the R1CS constraints.
