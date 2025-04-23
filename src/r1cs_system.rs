use std::collections::HashMap;
use crate::r1cs_constraint::{R1CSConstraint, ReadOnly};

pub struct R1CSSystem {
    variables: HashMap<String, usize>,
    constraints: Vec<R1CSConstraint<ReadOnly>>,
    next_index: usize,
}

impl R1CSSystem {
    pub fn new() -> Self {
        R1CSSystem {
            variables: HashMap::new(),
            constraints: Vec::new(),
            next_index: 1,
        }
    }

    pub fn size(&self) -> usize {
        self.constraints.len()
    }

    pub fn input_size(&self) -> usize {
        self.variables.len()
    }

    pub fn add_constraint<T>(&mut self, constraint: R1CSConstraint<T>)
    where R1CSConstraint<ReadOnly>: From<R1CSConstraint<T>> {
        self.constraints.push(R1CSConstraint::<ReadOnly>::from(constraint));
    }

    pub fn add_named_variable(&mut self, name: String) -> usize {
        let current_index = self.next_index;
        self.next_index += 1;
        self.variables.insert(name, current_index);
        current_index
    }

    pub fn add_variable(&mut self) -> usize {
        let current_index = self.next_index;
        self.next_index += 1;
        current_index
    }

    pub fn get_variable(&self, name: &str) -> Option<usize> {
        self.variables.get(name).copied()
    }
    
    pub fn print(&self) {
        println!("=== R1CS ===\n");
        for (variable_name, variable_position) in self.variables.iter() {
            println!("a_{} = {}", variable_position, variable_name);
        }
        for constraint in self.constraints.iter() {
            constraint.print();
        }
    }

    pub fn find_matching_constraint(&self, expected_a: &HashMap<usize, i64>, expected_b: &HashMap<usize, i64>, expected_c: Option<&HashMap<usize, i64>>,) -> Option<&R1CSConstraint<ReadOnly>> {
        self.constraints.iter().find(|constraint| {
            if expected_c.is_none() {
                constraint.lhs_matches(expected_a, expected_b)
            } else {
                constraint.lhs_matches(expected_a, expected_b) && constraint.rhs_matches(expected_c.unwrap())
            }
        })
    }

}
