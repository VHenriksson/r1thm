use std::marker::PhantomData;
use std::collections::HashMap;

// Todo: fix this docstring
/// MatrixRowEntry represents a single entry in a row of the matrix. Since R1CS systems generally
/// use sparse matrices, we represent each entry as a column index and its corresponding value.
/// Since only the relation between the rows in the three matrices is important, we do not need
/// to store information about the row itself.
pub struct R1CSConstraint {
    a: HashMap<usize, i64>,
    b: HashMap<usize, i64>,
    c: HashMap<usize, i64>,
}

pub struct R1CSSumConstraint {
    a: HashMap<usize, i64>,
    b: HashMap<usize, i64>,
    c: HashMap<usize, i64>,
}


impl R1CSConstraint {
    pub fn lhs_matches(&self, other_a: &HashMap<usize,i64>, other_b: &HashMap<usize,i64>) -> bool {
        self.a == *other_a && self.b == *other_b
    }

    pub fn rhs_matches(&self, other_c: &HashMap<usize,i64>) -> bool {
        self.c == *other_c
    }

    fn hashmap_str(x: &HashMap<usize,i64>) -> String {
        let mut values = vec![];
        for (position, value) in x.iter() {
            if *position == 0 {
                values.push(value.to_string());
            } else {
                if *value == 1 {
                    values.push(format!("a_{}", position));
                } else {
                    values.push(format!("{}a_{}", value, position));
                }
            }
        }
        values.join(" + ")
    }

    pub fn print(&self) {
        let a_string = Self::hashmap_str(&self.a);
        let b_string = Self::hashmap_str(&self.b);
        let c_string = Self::hashmap_str(&self.c);
        if a_string == "1" {
            println!("{} = {}", b_string, c_string);
        } else {
            println!("({})*({}) = {}", a_string, b_string, c_string)
        }
    }

    pub fn get_c_pairs (&self) -> Vec<(usize, i64)> {
        let mut pairs = vec![];
        for (position, value) in self.c.iter() {
            pairs.push((*position, *value));
        }
        pairs
    }

    pub fn new_multiplication_constraint(variable_a: usize, variable_b: usize, new_variable: usize) -> Self {
        R1CSConstraint {
            a: HashMap::from([(variable_a, 1)]),
            b: HashMap::from([(variable_b, 1)]),
            c: HashMap::from([(new_variable, 1)]),
        }
    }

    pub fn new_constant_multiplication_constraint(constant: i64, variable: usize, new_variable: usize) -> Self {
        R1CSConstraint {
            a: HashMap::from([(0,constant)]),
            b: HashMap::from([(variable, 1)]),
            c: HashMap::from([(new_variable, 1)]),
        }
    }

    pub fn new_final_constraint(expected_result : i64, variable_position : usize) -> Self {
        R1CSConstraint {
            a: HashMap::from([(0,1)]),
            b: HashMap::from([(variable_position, 1)]),
            c: HashMap::from([(0, expected_result)]),
        }
    }

    pub fn new_constant_constraint(constant: i64, new_variable: usize) -> Self {
        R1CSConstraint {
            a: HashMap::from([(0, 1)]),
            b: HashMap::from([(0, constant)]),
            c: HashMap::from([(new_variable, 1)]),
        }
    }
}








impl R1CSSumConstraint {
    pub fn new() -> Self {
        R1CSSumConstraint {
            a: HashMap::from([(0,1)]),
            b: HashMap::new(),
            c: HashMap::new(),
        }
    }
    pub fn add_to_sum(&mut self, position : usize) {
        self.b.insert(position, 1);
    }

    pub fn subtract_from_sum(&mut self, position : usize) {
        self.b.insert(position, -1);
    }
    
    pub fn set_right_hand_side(&mut self, position : usize) {
        self.c.insert(position, 1);
    }

    pub fn to_r1cs_constraint(self) -> R1CSConstraint {
        R1CSConstraint {
            a: self.a,
            b: self.b,
            c: self.c,
        }
    }
    
}
