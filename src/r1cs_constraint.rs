//! This module defines the R1CSConstraint struct and its associated methods.
//! It provides a way to represent and manipulate constraints in a Rank-1 Constraint System (R1CS).
//! 
//! A single R1CS constraint is an expression of the form `Σ a_i u_i * Σ b_j u_j = Σ c_k u_k` where
//! the `u_i` are some formal variables, the `a_i`, `b_i` and `c_i` denotes some numbers, and 
//! `Σ` denotes summation over the indices `i`, `j`, and `k`.
//! For technical purposes, we let `u_0` be the constant `1`.
//! For example, `(4u_1 + 6u_5) * (9 + 3u_1 + 9u_10 + 7u_32) = 8u_1 + u_2` would be a valid R1CS constraint.
//! 
//! Note that a R1CS constraint can be represented as three sparse vectors `a`, `b`, and `c` corrsponding to the
//! numbers `a_i`, `b_i`, and `c_i` respectively.
//! We do not consider the variables explicitly, but consider them to be identified by a unique index, i.e. 
//! the variables `u_1`, `u_2`, `u_3`, etc. are identified by `1`, `2`, `3`, etc.
//! Since the constraints are assumed to be sparse, we represent each sum as a hashmap, where the key represents the
//! variable (by the associated index) and the value represents the coefficient of that variable.
//! 
//! At the moment, this module is not very useful, since there is no way to access the 
//! innards of the constraint after creating it (except in the test functions). This should
//! obviously be changed when we start using the module somewhere.


use std::collections::HashMap;

/// This struct represents a Rank-1 Constraint System (R1CS) constraint.
/// 
/// For a constraint on the form `Σ a_i u_i * Σ b_i u_i = Σ c_i u_i`
/// the value of `a[i]` is the coefficient of `u_i` in the first sum,
/// `b[i]` is the coefficient of `u_i` in the second sum, and 
/// `c[i]` is the coefficient of `u_i` in the third sum. 
pub struct R1CSConstraint {
    a: HashMap<usize, i64>,
    b: HashMap<usize, i64>,
    c: HashMap<usize, i64>,
}


/// This struct represents a Rank-1 Constraint System (R1CS) constraint.
/// 
/// This struct is used to build special types of R1CS constraints, more specifically
/// constraints which forms a simple sum of other variables.
/// 
/// For a constraint on the form `Σ a_i u_i * Σ b_i u_i = Σ c_i u_i`
/// the value of `a[i]` is the coefficient of `u_i` in the first sum,
/// `b[i]` is the coefficient of `u_i` in the second sum, and 
/// `c[i]` is the coefficient of `u_i` in the third sum. 
pub struct R1CSSumConstraint {
    a: HashMap<usize, i64>,
    b: HashMap<usize, i64>,
    c: HashMap<usize, i64>,
}


impl R1CSConstraint {
    /// Creates a new R1CS constraint representing multiplication of two variables.
    /// More precicely, we use this if we have variables `u_i`, `u_j`, and `u_k`, and want to 
    /// represent the constraint `u_i * u_j = u_k`, 
    pub fn new_multiplication_constraint(i: usize, j: usize, k: usize) -> Self {
        R1CSConstraint {
            a: HashMap::from([(i, 1)]),
            b: HashMap::from([(j, 1)]),
            c: HashMap::from([(k, 1)]),
        }
    }

    /// Creates a new R1CS constraint representing multiplication of a variable and a constant.
    /// More precicely, we use this if we have variables `u_j`, and `u_k`, the constant `s` and want to
    /// represent the constraint `s * u_j = u_k`.
    pub fn new_constant_multiplication_constraint(s: i64, j: usize, k: usize) -> Self {
        R1CSConstraint {
            a: HashMap::from([(0,s)]),
            b: HashMap::from([(j, 1)]),
            c: HashMap::from([(k, 1)]),
        }
    }

    /// Creates a new R1CS constraint representing the "result" of the R1CS system. I.e.
    /// the constraint that `u_j = s` for some constant `s`.
    pub fn new_final_constraint(s : i64, j : usize) -> Self {
        R1CSConstraint {
            a: HashMap::from([(0,1)]),
            b: HashMap::from([(j, 1)]),
            c: HashMap::from([(0, s)]),
        }
    }

    /// Creates a new R1CS constraint representing a constant. I.e. if `s` is a constant,
    /// and `u_k` is a variable, we want to represent the constraint `s = u_k`.
    pub fn new_constant_constraint(s: i64, k: usize) -> Self {
        R1CSConstraint {
            a: HashMap::from([(0, 1)]),
            b: HashMap::from([(0, s)]),
            c: HashMap::from([(k, 1)]),
        }
    }
}


impl R1CSSumConstraint {

    /// Creates a new R1CSSumConstraint representing a sum of variables.
    /// At the beginning, this is the empty constraint `1 * 0 = 0`.
    pub fn new() -> Self {
        R1CSSumConstraint {
            a: HashMap::from([(0,1)]),
            b: HashMap::new(),
            c: HashMap::new(),
        }
    }


    /// Adds a variable to the left hand side of the constraint.
    /// More precicely, if we already represent the constraint `Σ u_j = rhs`
    /// for some sum and some right hand side `rhs`, then after this call we have `Σ u_j + u_i = rhs`.
    /// 
    /// **Note**: This forces us to create intermediate constraints, which may not be the
    /// actual constraints we want to represent. 
    pub fn add_to_sum(&mut self, i : usize) {
        self.b.insert(i, 1);
    }

    /// Adds a variable to the left hand side of the constraint.
    /// More precicely, if we already represent the constraint `Σ u_j = rhs`
    /// for some sum and some right hand side `rhs`, then after this call we have `Σ u_j - u_i = rhs`.
    /// 
    /// **Note**: This forces us to create intermediate constraints, which may not be the
    /// actual constraints we want to represent. 
    pub fn subtract_from_sum(&mut self, position : usize) {
        self.b.insert(position, -1);
    }
    
    /// Sets the right hand side of the constraint.
    /// More precicely, if we already represent the constraint `Σ u_j = 0`,
    /// then after this call we have `Σ u_j = s`.
    /// 
    /// **Note**: This forces us to create intermediate constraints, which may not be the
    /// actual constraints we want to represent. 
    pub fn set_right_hand_side(&mut self, s : usize) {
        self.c.insert(s, 1);
    }

    /// Transforms an object of type `R1CSSumConstraint` to an object of type `R1CSConstraint`.
    pub fn to_r1cs_constraint(self) -> R1CSConstraint {
        R1CSConstraint {
            a: self.a,
            b: self.b,
            c: self.c,
        }
    }
    
}

#[cfg(test)]
impl R1CSConstraint {
    /// Checks if a constraint has the expected form of its `a` and `b` vectors. This method is used
    /// only for testing and debugging purposes.
    pub fn lhs_matches(&self, other_a: &HashMap<usize,i64>, other_b: &HashMap<usize,i64>) -> bool {
        self.a == *other_a && self.b == *other_b
    }

    /// Checks if a constraint has the expected form of its `c` vector. This method is used
    /// only for testing and debugging purposes.
    pub fn rhs_matches(&self, other_c: &HashMap<usize,i64>) -> bool {
        self.c == *other_c
    }

    /// Returns the `c` vector of the constraint as a vector of pairs.
    /// Used for testing and debugging purposes.
    pub fn get_c_pairs (&self) -> Vec<(usize, i64)> {
        let mut pairs = vec![];
        for (position, value) in self.c.iter() {
            pairs.push((*position, *value));
        }
        pairs
    }


    /// Presents the hashmap as a string of the form `Σ a_i u_i`
    /// Used in debugging.
    fn hashmap_str(x: &HashMap<usize,i64>) -> String {
        let mut values = vec![];
        for (position, value) in x.iter() {
            if *position == 0 {
                values.push(value.to_string());
            } else {
                if *value == 1 {
                    values.push(format!("u_{}", position));
                } else {
                    values.push(format!("{}u_{}", value, position));
                }
            }
        }
        values.join(" + ")
    }

    /// Prints the constraint on the form `Σ a_i u_i * Σ b_i u_i = Σ c_i u_i`
    /// Used in debugging.
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




}



