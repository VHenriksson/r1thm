//! A R1CS is an equation on the form `A v ∘ B v = C v` where  `A`, `B`, and `C` are
//! matrices, `v` is an unknown vector and `∘` represents pointwise multiplication.
//! We can think of these as collections of equations on the form 
//! `Σ a_i u_i * Σ b_j u_j = Σ c_k u_k` where `u_i`, `u_j`, and `u_k` are variables.
//! 
//! In this implementation, the R1CS is represented as a collection of such equations.
//! 
//! Along with the equations, we also give a mapping from variable names to their indices.
//! This is not strictly necessary for a R1CS, since a variable can be identified by its
//! index, and thus need not be named. However, this mapping gives us the freedom to use
//! named variables and keeping track of their indices. A highly optimized implementation
//! of R1CS should probably not contain this feature.
//! 
//! At the moment, this module is not very useful, since there is no way to access the 
//! constraints after adding them to the system (except in the test functions). This should
//! obviously be changed when we start using the module somewhere.

use std::collections::HashMap;
use crate::r1cs_constraint::R1CSConstraint;

/// This struct represents a Rank-1 Constraint System (R1CS) constraint.
/// 
/// It has a list of constraints given as the type `R1CSConstraint`, and a mapping
/// from variable names to their indices.
pub struct R1CS {
    variables: HashMap<String, usize>,
    constraints: Vec<R1CSConstraint>,
    /// The next index to be used for a new variable.
    next_index: usize,
}

impl R1CS {
    /// Create a new, empty R1CS system.
    pub fn new() -> Self {
        R1CS {
            variables: HashMap::new(),
            constraints: Vec::new(),
            next_index: 1,
        }
    }

    /// Returns the size, i.e. the number of constraints in the system.
    pub fn size(&self) -> usize {
        self.constraints.len()
    }

    /// Returns the input size, i.e. the number of variables in the system.
    pub fn input_size(&self) -> usize {
        self.variables.len()
    }

    /// Adds a constraint to the system.
    pub fn add_constraint(&mut self, constraint: R1CSConstraint) {
        self.constraints.push(constraint);
    }

    /// Adds a new explicit variable to the system, i.e. a variable
    /// with a name.
    pub fn add_input_variable(&mut self, name: String) -> usize {
        let current_index = self.next_index;
        self.next_index += 1;
        self.variables.insert(name, current_index);
        current_index
    }

    /// Adds a new internal variable to the system, i.e. a variable
    /// that is not given as an input.
    pub fn add_variable(&mut self) -> usize {
        let current_index = self.next_index;
        self.next_index += 1;
        current_index
    }

    /// Get the index of a variable by its name.
    pub fn get_variable_index(&self, name: &str) -> Option<usize> {
        self.variables.get(name).copied()
    }

}


#[cfg(test)]
impl R1CS {
    
    /// Prints the R1CS system in a human-readable format.
    /// The system is printed as a list of the constraints.
    /// Used for testing and debugging purposes.
    pub fn print(&self) {
        println!("=== R1CS ===\n");
        for (variable_name, variable_position) in self.variables.iter() {
            println!("u_{} = {}", variable_position, variable_name);
        }
        for constraint in self.constraints.iter() {
            constraint.print();
        }
    }

    /// Given two hashmaps, `expected_a` and `expected_b`, and an optional hashmap `expected_c`,
    /// finds and returns a constraint in the system that matches the expected form.
    /// Used for testing and debugging purposes.
    pub fn find_matching_constraint(&self, expected_a: &HashMap<usize, i64>, expected_b: &HashMap<usize, i64>, expected_c: Option<&HashMap<usize, i64>>,) -> Option<&R1CSConstraint> {
        self.constraints.iter().find(|constraint| {
            if expected_c.is_none() {
                constraint.lhs_matches(expected_a, expected_b)
            } else {
                constraint.lhs_matches(expected_a, expected_b) && constraint.rhs_matches(expected_c.unwrap())
            }
        })
    }
}
