use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "poly.pest"] // relative to src/
struct PolyParser;

fn handle_variable(variable_pair: pest::iterators::Pair<Rule>) {
    println!("Variable: {}", variable_pair.as_str());
}

fn handle_varpow(varpow_pair: pest::iterators::Pair<Rule>) {
    for pair in varpow_pair.into_inner() {
        match pair.as_rule() {
            Rule::variable => {
                handle_variable(pair);
            }
            Rule::exponent => {
                println!("Exponent: {}", pair.as_str());
            }
            _ => {
                println!("Unhandled varpow: {:?}", pair.as_rule());
            }
        }
    }
}

fn handle_factor(factor_pair: pest::iterators::Pair<Rule>) {
    for pair in factor_pair.into_inner() {
        match pair.as_rule() {
            Rule::varpow => {
                handle_varpow(pair);
            }
            Rule::parenth => {
                handle_parenth(pair);
            }
            _ => {
                println!("Unhandled factor: {:?}", pair.as_rule());
            }
        }
    }
}

fn handle_cfactor(cfactor_pair: pest::iterators::Pair<Rule>) {
    for pair in cfactor_pair.into_inner() {
        match pair.as_rule() {
            Rule::constant => {
                println!("C-factor constant: {}", pair.as_str());
            }
            Rule::factor => {
                handle_factor(pair);
            }
            _ => {
                println!("Unhandled C-factor: {:?}", pair.as_rule());
            }
        }
    }
}

fn handle_parenth(parenth_pair: pest::iterators::Pair<Rule>) {
    for pair in parenth_pair.into_inner() {
        match pair.as_rule() {
            Rule::expression => {
                println!("Parenthesis expression: {}", pair.as_str());
                handle_expression(pair);
                println!("End of parenthesis");
            }
            Rule::exponent => {
                println!("Exponent: {}", pair.as_str());
            }
            _ => {
                println!("Unhandled parenthesis: {:?}", pair.as_rule());
            }
        }
    }
}

fn handle_product(product_pair: pest::iterators::Pair<Rule>) {
    for pair in product_pair.into_inner() {
        match pair.as_rule() {
            Rule::cfactor => {
                handle_cfactor(pair);
            }
            Rule::factor => {
                handle_factor(pair);
            }   
            _ => {
                println!("Unhandled product: {:?}", pair.as_rule());
            }
        }
    }
}

fn handle_term(term_pair: pest::iterators::Pair<Rule>) {
    for pair in term_pair.into_inner() {
        match pair.as_rule() {
            Rule::cfactor => {
                handle_cfactor(pair);
            }
            Rule::product => {
                handle_product(pair);
            }
            Rule::constant => {
                println!("Constant: {}", pair.as_str());
            }
            _ => {
                println!("Unhandled term: {:?}", pair.as_rule());
            }
        }
    }
}

fn handle_expression(expression_pair: pest::iterators::Pair<Rule>) {
    for pair in expression_pair.into_inner() {
        match pair.as_rule() {
            Rule::term => {
                println!("Term: {}", pair.as_str());
                handle_term(pair);
            }
            _ => {
                println!("Unhandled expression: {:?}", pair.as_rule());
            }
        }
    }
}

fn poly2r1cs(polynomial: String) -> Result<(), String> {
    let input = "39x^2 + 5y + 2*x - 5 + (x^2 + 1)^6 + 4x*5y + xyz + 2xz + 3 + 2*y*z ((x + ((7x^5)^4(5x+t)^8*4x)) * (x - y))^4 + 8";

    println!("Parsing polynomial: {}", input);
    match PolyParser::parse(Rule::expression, input) {
        Ok(mut pairs) => {
            let expression = pairs.next().unwrap(); // top-level expression

            // Iterate over the inner parts of the expression (terms and operators)
            println!("Expression: {}", expression.as_str());
            handle_expression(expression);
            Ok(())
        }
        Err(e) => Err(format!("Error parsing polynomial: {}", e)),
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_poly2r1cs() {
        let polynomial = "3*x^2 + 2*x - 5".to_string();
        let result = poly2r1cs(polynomial);
        assert!(result.is_ok());
    }
}