use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

use crate::r1cs_system::R1CSSystem;
use crate::r1cs_constraint::{R1CSConstraint, Constant, ConstantMultiplication, Sum, Multiplication, Final};



#[derive(Parser)]
#[grammar = "poly.pest"] // relative to src/
struct PolyParser;


struct ParseTreeVisitor {
    visited_nodes: HashMap<(Rule, String), usize>, 
    r1cs_system: R1CSSystem,
}

impl ParseTreeVisitor {
    fn new() -> Self {
        ParseTreeVisitor {
            visited_nodes: HashMap::new(),
            r1cs_system: R1CSSystem::new(),
        }
    }

    fn cache_wrapper<F>(&mut self, f: F, pair: pest::iterators::Pair<Rule>) -> usize
    where F: for<'a> Fn(&'a mut Self, pest::iterators::Pair<Rule>) -> usize
    {
        println!("Visiting expression {} of type {:?}", pair.as_str(), pair.as_rule());
        let key = (pair.as_rule(), pair.as_str().to_string());
        if self.visited_nodes.contains_key(&key) {
            self.visited_nodes[&key]
        } else {
            let variable = f(self, pair);
            self.visited_nodes.insert(key, variable);
            variable
        }
    }

    fn visit_variable(&mut self, variable_pair: pest::iterators::Pair<Rule>) -> usize {
        println!("In variable");
        self.cache_wrapper(|s, input_pair| {
            s.r1cs_system.add_named_variable(input_pair.as_str().to_string())
        } , variable_pair)
    }


    fn visit_varpow(&mut self, varpow_pair: pest::iterators::Pair<Rule>) -> usize {
        println!("In varpow");
        self.cache_wrapper(|s, input_pair| {
            let mut exponent = 1;
            let mut variable_position = 0;
            for pair in input_pair.into_inner() {
                match pair.as_rule() {
                    Rule::variable => {
                        variable_position = s.visit_variable(pair);
                    }
                    Rule::exponent => {
                        exponent = pair.as_str().parse().expect("Not a number");
                    }
                    _ => {
                        println!("Unhandled varpow: {:?}", pair.as_rule());
                    }
                }
            }            
            println!("Back through varpow");
            s.create_exponentiation_constraints(exponent, variable_position)
        }, varpow_pair)
    }

    fn create_exponentiation_constraints(&mut self, mut exponent: i32, variable_position: usize) -> usize {
        if exponent == 1 {
            variable_position
        } else {
            let max_exponent_variable = self.r1cs_system.add_variable();
            let mut current_exponent_variable = max_exponent_variable;
            while exponent > 1 {
                if exponent % 2 == 0 {
                    exponent /= 2;
                    if exponent == 1 {
                        self.r1cs_system.add_constraint(R1CSConstraint::<Multiplication>::new(variable_position, variable_position, current_exponent_variable));
                    } else {
                        let new_exponent_variable = self.r1cs_system.add_variable();
                        self.r1cs_system.add_constraint(R1CSConstraint::<Multiplication>::new(new_exponent_variable, new_exponent_variable, current_exponent_variable));
                        current_exponent_variable = new_exponent_variable;
                    }
                } else {
                    let new_exponent_variable = self.r1cs_system.add_variable();
                    self.r1cs_system.add_constraint(R1CSConstraint::<Multiplication>::new(variable_position, new_exponent_variable, current_exponent_variable));
                    exponent -= 1;
                    current_exponent_variable = new_exponent_variable;
                }
            }
            println!("Back through exponentiation");
            max_exponent_variable
        }
    }
    
    fn visit_factor(&mut self, factor_pair: pest::iterators::Pair<Rule>) -> usize {
        println!("In factor");
        self.cache_wrapper(|s, input_pair| {
            let mut variable_position = 0;
            for pair in input_pair.into_inner() {
                match pair.as_rule() {
                    Rule::varpow => {
                        variable_position = s.visit_varpow(pair);
                    }
                    Rule::parenth => {
                        variable_position = s.visit_parenth(pair);
                    }
                    _ => {
                        println!("Unhandled factor: {:?}", pair.as_rule());
                    }
                }
            }
            println!("Back through factor");
            variable_position
        }, factor_pair)
    }

    fn visit_cfactor(&mut self, cfactor_pair: pest::iterators::Pair<Rule>) -> usize {
        println!("In cfactor");
        self.cache_wrapper(|s, input_pair| {
            let mut variable_position = 0;
            let mut variable_constant = 1;
            for pair in input_pair.into_inner() {
                match pair.as_rule() {
                    Rule::constant => {
                        variable_constant = pair.as_str().parse().expect("Not a number");
                    }
                    Rule::factor => {
                        variable_position = s.visit_factor(pair);
                    }
                    _ => {
                        println!("Unhandled C-factor: {:?}", pair.as_rule());
                    }
                }
            }
            println!("Back through cfactor");
            if variable_constant != 1 {
                let cfactor_variable = s.r1cs_system.add_variable();
                s.r1cs_system.add_constraint(R1CSConstraint::<ConstantMultiplication>::new(variable_constant, variable_position, cfactor_variable));
                cfactor_variable
            } else {
                variable_position
            }
        }, cfactor_pair)
    }

    fn visit_parenth(&mut self, parenth_pair: pest::iterators::Pair<Rule>) -> usize {
        println!("In parenth");
        self.cache_wrapper(|s, input_pair| {
            let mut exponent = 1;
            let mut variable_position = 0;
            for pair in input_pair.into_inner() {
                match pair.as_rule() {
                    Rule::expression => {
                        variable_position = s.visit_expression(pair);
                    }
                    Rule::exponent => {
                        exponent = pair.as_str().parse().expect("Not a number");
                    }
                    _ => {
                        println!("Unhandled parenthesis: {:?}", pair.as_rule());
                    }
                }
            }
            println!("Back through parenth");
            s.create_exponentiation_constraints(exponent, variable_position)
        }, parenth_pair)
    }

    fn visit_product(&mut self, product_pair: pest::iterators::Pair<Rule>) -> usize {
        println!("In product");
        self.cache_wrapper(|s, input_pair| {
            let mut variable_positions = Vec::new();
            for pair in input_pair.into_inner() {
                match pair.as_rule() {
                    Rule::cfactor => {
                        variable_positions.push(s.visit_cfactor(pair));
                    }
                    Rule::factor => {
                        variable_positions.push(s.visit_factor(pair));
                    }
                    _ => {
                        println!("Unhandled product: {:?}", pair.as_rule());
                    }
                }
            }
            println!("Back through product");
            if variable_positions.len() == 0 {
                0
            } else {
                let mut current_variable = variable_positions[0];
                println!("Current variable: {}", current_variable);
                for position in variable_positions.iter().skip(1) {
                    let product_variable = s.r1cs_system.add_variable();
                    s.r1cs_system.add_constraint(R1CSConstraint::<Multiplication>::new(current_variable, *position, product_variable));
                    current_variable = product_variable;
                }
                current_variable
            }
        }, product_pair)
    }

    fn visit_term(&mut self, term_pair: pest::iterators::Pair<Rule>) -> usize {
        println!("In term");
        self.cache_wrapper(|s, input_pair| {
            let mut variable_position = 0;
            for pair in input_pair.into_inner() {
                match pair.as_rule() {
                    Rule::cfactor => {
                        variable_position = s.visit_cfactor(pair);
                    }
                    Rule::product => {
                        variable_position = s.visit_product(pair);
                    }
                    Rule::constant => {
                        let constant = pair.as_str().parse().expect("Not a number");
                        variable_position = s.r1cs_system.add_variable();
                        s.r1cs_system.add_constraint(R1CSConstraint::<Constant>::new(constant, variable_position));
                    }
                    _ => {
                        println!("Unhandled term: {:?}", pair.as_rule());
                    }
                }
            }
            println!("Back through term");
            variable_position
        }, term_pair)
    }
    
    fn visit_add_or_sub_term(&mut self, add_or_sub_pair: pest::iterators::Pair<Rule>) -> usize {
        println!("In visit add or sub term");
        println!("Visiting expression {} of type {:?}", add_or_sub_pair.as_str(), add_or_sub_pair.as_rule());
        let mut variable_position = 0;
        for pair in add_or_sub_pair.into_inner() {
            match pair.as_rule() {
                Rule::term => {
                    variable_position = self.visit_term(pair);
                }
                _ => {
                    println!("Unhandled factor: {:?}", pair.as_rule());
                }
            }
        }
            println!("Back through add or sub");
        variable_position
    }
    
    fn visit_expression(&mut self, expression_pair: pest::iterators::Pair<Rule>) -> usize {
        println!("In expression");
        self.cache_wrapper(|s, input_pair| {
            let mut new_constraint = R1CSConstraint::<Sum>::new();
            let mut should_create_new_variable = false;
            let mut fallthrough_variable = 0;
            for pair in input_pair.into_inner() {
                match pair.as_rule() {
                    Rule::term => {
                        println!("We found a term");
                        fallthrough_variable = s.visit_term(pair);
                        new_constraint.add_to_sum(fallthrough_variable);
                        println!("We actually get here???");
                    }
                    Rule::add_term => {
                        new_constraint.add_to_sum(s.visit_add_or_sub_term(pair));
                        should_create_new_variable = true;
                    }
                    Rule::sub_term => {
                        new_constraint.subtract_from_sum(s.visit_add_or_sub_term(pair));
                        should_create_new_variable = true;
                    }
                    _ => {
                        println!("Unhandled expression: {:?}", pair.as_rule());
                    }
                }
            }
            println!("Back through expression");
            if should_create_new_variable {
                let expression_variable = s.r1cs_system.add_variable();
                new_constraint.set_right_hand_side(expression_variable);
                s.r1cs_system.add_constraint(new_constraint);
                expression_variable
            } else {
                fallthrough_variable
            }
        }, expression_pair)
    }  

    fn generate_r1cs(mut self, expression_pair: pest::iterators::Pair<Rule>, expected_result : i64) -> R1CSSystem {
        // Placeholder for generating the R1CS system from the visited nodes
        let variable_position = self.visit_expression(expression_pair);
        self.r1cs_system.add_constraint(R1CSConstraint::<Final>::new(expected_result, variable_position));
        self.r1cs_system
    }
}







pub fn poly2r1cs(polynomial: String, expected_result : i64) -> Result<R1CSSystem, String>  { 

    let visitor = ParseTreeVisitor::new();
    match PolyParser::parse(Rule::expression, polynomial.as_str()) {
        Ok(mut pairs) => {
            let expression = pairs.next().unwrap(); // top-level expression

            // Iterate over the inner parts of the expression (terms and operators)
            println!("Expression: {}", expression.as_str());
            Ok(visitor.generate_r1cs(expression, expected_result))
        }
        Err(e) => Err(format!("Error parsing polynomial: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn get_variable_positions(r1cs: &R1CSSystem, variables: Vec<String>) -> HashMap<String, usize> {
        let mut variable_positions = HashMap::new();
        assert_eq!(r1cs.input_size(), variables.len(), "Expected {} variables, found {}", variables.len(), r1cs.input_size());
        for variable in variables {
            let position = r1cs.get_variable(&variable);
            assert!(position.is_some(), "Variable {} not found", variable);
            let position = position.unwrap();
            assert_ne!(position, 0, "Variable {} has position 0", variable);
            for (other_variable, other_position) in variable_positions.iter() {
                assert_ne!(position, *other_position, "Variable {} has the same position as {}", variable, other_variable);
            }
            variable_positions.insert(variable, position);
        }
        variable_positions
    }

    fn handle_constraint(r1cs: &R1CSSystem, expected_a: HashMap<usize, i64>, expected_b: HashMap<usize, i64>) -> usize {
        let matching_constraint = r1cs.find_matching_constraint(&expected_a, &expected_b, None);
        assert!(matching_constraint.is_some(), "Looking for constraint on form ({}) * ({}). No matching constraint found", expected_a.iter().map(|(k, v)| if *k != 0 {format!("{}a_{}", v, k)} else {format!("{}", v)}).collect::<Vec<_>>().join(" + "), expected_b.iter().map(|(k, v)| format!("{}a_{}", v, k)).collect::<Vec<_>>().join(" + "));
        let matching_constraint = matching_constraint.unwrap();
        let c_pairs = matching_constraint.get_c_pairs();
        assert_eq!(c_pairs.len(), 1, "Expected exactly one entry in c");
        let (c_key, c_value) = c_pairs.iter().next().unwrap();
        assert_eq!(*c_value, 1, "Expected c value to be 1");
        *c_key
    }

    fn check_final_constraint(r1cs: &R1CSSystem, expected_final_variable: usize, expected_c: i64) {
        let expected_a = HashMap::from([(0, 1)]);
        let expected_b = HashMap::from([(expected_final_variable, 1)]);
        let expected_c = HashMap::from([(0, expected_c)]);
        let matching_constraint = r1cs.find_matching_constraint(&expected_a, &expected_b, Some(&expected_c));
        assert!(matching_constraint.is_some(), "No matching constraint found");
    }

    fn handle_add_constraint(r1cs: &R1CSSystem, variables_to_add_with_signs: Vec<(usize, i8)>) -> usize {
        let expected_a = HashMap::from([(0, 1)]);
        let mut expected_b = HashMap::new();
        for (variable, sign) in variables_to_add_with_signs {
            assert!(sign == 1 || sign == -1);
            expected_b.insert(variable, sign as i64);
        }
        handle_constraint(r1cs, expected_a, expected_b)
    }

    fn handle_const_constraint(r1cs: &R1CSSystem, constant: i64) -> usize {
        let expected_a = HashMap::from([(0, 1)]);
        let expected_b = HashMap::from([(0, constant)]);
        handle_constraint(r1cs, expected_a, expected_b)
    }

    fn handle_const_mult_constraint(r1cs: &R1CSSystem, const_mult: i64, variable: usize) -> usize {
        let expected_a = HashMap::from([(0, const_mult)]);
        let expected_b = HashMap::from([(variable, 1)]);
        handle_constraint(r1cs, expected_a, expected_b)
    }

    fn handle_mult_constraint(r1cs: &R1CSSystem, variable_a: usize, variable_b: usize) -> usize {
        let expected_a = HashMap::from([(variable_a, 1)]);
        let expected_b = HashMap::from([(variable_b, 1)]);
        handle_constraint(r1cs, expected_a, expected_b)
    }

    fn parse(polynomial: String, expected_result : i64) -> R1CSSystem {
        let result = poly2r1cs(polynomial, expected_result);
        assert!(result.is_ok(), "Error parsing polynomial: {:?}", result.err());
        result.unwrap() 
    }
    
    #[test]
    fn test_basic_sum() {
        let polynomial = "y + x + z".to_string();
        let expected_result = 31;
        let parsed_poly = parse(polynomial, expected_result);
        let pos = get_variable_positions(&parsed_poly, vec!["x".to_string(), "y".to_string(), "z".to_string()]);
        let final_variable = handle_add_constraint(&parsed_poly, vec![(pos["x"],1), (pos["y"],1), (pos["z"],1)]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_basic_sum_with_coefficients() {
        let polynomial = "67x + 7*y".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        let mut pos = get_variable_positions(&parsed_poly, vec!["x".to_string(), "y".to_string()]);
        pos.insert("67x".to_string(), handle_const_mult_constraint(&parsed_poly, 67, pos["x"]));
        pos.insert("7*y".to_string(), handle_const_mult_constraint(&parsed_poly, 7, pos["y"]));
        parsed_poly.print();
        let final_variable = handle_add_constraint(&parsed_poly, vec![(pos["67x"],1), (pos["7*y"],1)]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_basic_sum_with_constant() {
        let polynomial = "7 + 2x + 3y".to_string();
        let expected_result = 70;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let mut pos = get_variable_positions(&parsed_poly, vec!["x".to_string(), "y".to_string()]);
        pos.insert("7".to_string(), handle_const_constraint(&parsed_poly, 7));
        pos.insert("2x".to_string(), handle_const_mult_constraint(&parsed_poly, 2, pos["x"]));
        pos.insert("3y".to_string(), handle_const_mult_constraint(&parsed_poly, 3, pos["y"]));
        parsed_poly.print();
        let final_variable = handle_add_constraint(&parsed_poly, vec![(pos["7"],1), (pos["2x"],1), (pos["3y"],1)]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_basic_product() {
        let polynomial = "x*y".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let pos = get_variable_positions(&parsed_poly, vec!["x".to_string(), "y".to_string()]);
        let final_variable = handle_mult_constraint(&parsed_poly, pos["x"], pos["y"]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_recurring_product() {
        let polynomial = "x*y*z*u*v*w".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let mut pos = get_variable_positions(&parsed_poly, vec!["x".to_string(), "y".to_string(), "z".to_string(), "u".to_string(), "v".to_string(), "w".to_string()]);
        pos.insert("x*y".to_string(), handle_mult_constraint(&parsed_poly, pos["x"], pos["y"]));
        pos.insert("x*y*z".to_string(), handle_mult_constraint(&parsed_poly, pos["x*y"], pos["z"]));
        pos.insert("x*y*z*u".to_string(), handle_mult_constraint(&parsed_poly, pos["x*y*z"], pos["u"]));
        pos.insert("x*y*z*u*v".to_string(), handle_mult_constraint(&parsed_poly, pos["x*y*z*u"], pos["v"]));
        let final_variable = handle_mult_constraint(&parsed_poly, pos["x*y*z*u*v"], pos["w"]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_simple_exponentiation() {
        let polynomial = "x^2".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let pos = get_variable_positions(&parsed_poly, vec!["x".to_string()]);
        let final_variable = handle_mult_constraint(&parsed_poly, pos["x"], pos["x"]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }
    
    #[test]
    fn test_higher_exponentiation() {
        let polynomial = "x^8".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let pos = get_variable_positions(&parsed_poly, vec!["x".to_string()]);
        let mut final_variable = pos["x"];
        for _ in 0..3 {
            final_variable = handle_mult_constraint(&parsed_poly, final_variable, final_variable);
        }
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_odd_exponentiation() {
        let polynomial = "x^3".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let mut pos = get_variable_positions(&parsed_poly, vec!["x".to_string()]);
        pos.insert("x^2".to_string(), handle_mult_constraint(&parsed_poly, pos["x"], pos["x"]));
        let final_variable = handle_mult_constraint(&parsed_poly, pos["x"], pos["x^2"]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_complicated_exponentiation() {
        let polynomial = "x^13".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let mut pos = get_variable_positions(&parsed_poly, vec!["x".to_string()]);
        pos.insert("x^2".to_string(), handle_mult_constraint(&parsed_poly, pos["x"], pos["x"]));
        pos.insert("x^3".to_string(), handle_mult_constraint(&parsed_poly, pos["x"], pos["x^2"]));
        pos.insert("x^6".to_string(), handle_mult_constraint(&parsed_poly, pos["x^3"], pos["x^3"]));
        pos.insert("x^12".to_string(), handle_mult_constraint(&parsed_poly, pos["x^6"], pos["x^6"]));
        let final_variable = handle_mult_constraint(&parsed_poly, pos["x"], pos["x^12"]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_parenthesis() {
        let polynomial = "(x + y) + z".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let mut pos = get_variable_positions(&parsed_poly, vec!["x".to_string(), "y".to_string(), "z".to_string()]);
        pos.insert("(x + y)".to_string(), handle_add_constraint(&parsed_poly, vec![(pos["x"],1), (pos["y"], 1)]));
        let final_variable = handle_add_constraint(&parsed_poly, vec![(pos["(x + y)"],1), (pos["z"],1)]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_parenthesis_with_exponentiation() {
        let polynomial = "(x + y)^7".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let mut pos = get_variable_positions(&parsed_poly, vec!["x".to_string(), "y".to_string()]);
        pos.insert("(x + y)".to_string(), handle_add_constraint(&parsed_poly, vec![(pos["x"],1), (pos["y"],1)]));
        pos.insert("(x + y)^2".to_string(), handle_mult_constraint(&parsed_poly, pos["(x + y)"], pos["(x + y)"]));  
        pos.insert("(x + y)^3".to_string(), handle_mult_constraint(&parsed_poly, pos["(x + y)"], pos["(x + y)^2"]));
        pos.insert("(x + y)^6".to_string(), handle_mult_constraint(&parsed_poly, pos["(x + y)^3"], pos["(x + y)^3"]));
        let final_variable = handle_mult_constraint(&parsed_poly, pos["(x + y)"], pos["(x + y)^6"]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_handle_recurring_variable() {
        let polynomial = "x + x*x".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let mut pos = get_variable_positions(&parsed_poly, vec!["x".to_string()]);
        pos.insert("x*x".to_string(), handle_mult_constraint(&parsed_poly, pos["x"], pos["x"]));
        let final_variable = handle_add_constraint(&parsed_poly, vec![(pos["x"],1), (pos["x*x"],1)]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_repeated_exponentiation() {
        let polynomial = "(89(6x^3)^4)^2".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let mut pos = get_variable_positions(&parsed_poly, vec!["x".to_string()]);
        pos.insert("x^2".to_string(), handle_mult_constraint(&parsed_poly, pos["x"], pos["x"]));
        pos.insert("x^3".to_string(), handle_mult_constraint(&parsed_poly, pos["x"], pos["x^2"]));
        pos.insert("6x^3".to_string(), handle_const_mult_constraint(&parsed_poly, 6, pos["x^3"]));
        pos.insert("(6x^3)^2".to_string(), handle_mult_constraint(&parsed_poly, pos["6x^3"], pos["6x^3"]));
        pos.insert("(6x^3)^4".to_string(), handle_mult_constraint(&parsed_poly, pos["(6x^3)^2"], pos["(6x^3)^2"]));
        pos.insert("89(6x^3)^4".to_string(), handle_const_mult_constraint(&parsed_poly, 89, pos["(6x^3)^4"]));
        let final_variable = handle_mult_constraint(&parsed_poly, pos["89(6x^3)^4"], pos["89(6x^3)^4"]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_complicated_product () {
        let polynomial = "(7x^5)^4(5x+t)^8*4x".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let mut pos = get_variable_positions(&parsed_poly, vec!["x".to_string(), "t".to_string()]);
        pos.insert("x^2".to_string(), handle_mult_constraint(&parsed_poly, pos["x"], pos["x"]));
        pos.insert("x^4".to_string(), handle_mult_constraint(&parsed_poly, pos["x^2"], pos["x^2"]));
        pos.insert("x^5".to_string(), handle_mult_constraint(&parsed_poly, pos["x"], pos["x^4"]));
        pos.insert("7x^5".to_string(), handle_const_mult_constraint(&parsed_poly, 7, pos["x^5"]));
        pos.insert("(7x^5)^2".to_string(), handle_mult_constraint(&parsed_poly, pos["7x^5"], pos["7x^5"]));
        pos.insert("(7x^5)^4".to_string(), handle_mult_constraint(&parsed_poly, pos["(7x^5)^2"], pos["(7x^5)^2"]));
    }

    #[test]
    fn test_simple_difference () {
        let polynomial = "x - y".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let pos = get_variable_positions(&parsed_poly, vec!["x".to_string(), "y".to_string()]);
        let final_variable = handle_add_constraint(&parsed_poly, vec![(pos["x"],1), (pos["y"],-1)]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_complicated_sum_with_negatives () {
        let polynomial = "x - y - z + u - w + t + s - p".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let pos = get_variable_positions(&parsed_poly, vec!["x".to_string(), "y".to_string(), "z".to_string(), "u".to_string(), "w".to_string(), "t".to_string(), "s".to_string(), "p".to_string()]);
        let final_variable = handle_add_constraint(&parsed_poly, vec![(pos["x"],1), (pos["y"],-1), (pos["z"],-1), (pos["u"],1), (pos["w"],-1), (pos["t"],1), (pos["s"],1), (pos["p"],-1)]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_sum_starting_with_negatives () {
        let polynomial = "-x - y + z".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let pos = get_variable_positions(&parsed_poly, vec!["x".to_string(), "y".to_string(), "z".to_string()]);
        let final_variable = handle_add_constraint(&parsed_poly, vec![(pos["x"],-1), (pos["y"],-1), (pos["z"],1)]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_only_negative () {
        let polynomial = "-x".to_string();
        let expected_result = 10;
        let parsed_poly = parse(polynomial, expected_result);
        parsed_poly.print();
        let pos = get_variable_positions(&parsed_poly, vec!["x".to_string()]);
        let final_variable = handle_add_constraint(&parsed_poly, vec![(pos["x"],-1)]);
        check_final_constraint(&parsed_poly, final_variable, expected_result);
    }

    #[test]
    fn test_poly2r1cs() {
        let polynomial = "3*x^2 + 2*x - 5".to_string();
        let result = poly2r1cs(polynomial, 2);
        assert!(result.is_ok());
    }
}