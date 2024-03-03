use std::string::String;
use std::fmt::{Display, Formatter, Result as FmtResult};
use crate::ast::{Expr, Stmt, Visitor};
use crate::parser::Parser;
use crate::environment::Environment;
use crate::token::{Token, TokenType};

//represents an Interpreter struct

pub struct Interpreter {
    environment: Environment
}

//
impl Interpreter {
    //Does nothing for now?
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    // for now it returns Executed types
    pub fn interpret(&mut self, statement: Vec<Stmt>)  {
        for x in statement {
            self.visit_statement(&x).unwrap();
        }
    }

    fn stringify(&self, types: Types) -> String {
        match types {
            Types::Boolean(b) => b.to_string(),
            Types::Nil => "nil".to_string(),
            Types::Number(n) => n.to_string(),
            Types::ReturnString(s) => s,

        }
    }

}
impl Visitor for Interpreter {
    type E = Result<Types, String>;
    type S = Result<(), String>;
    fn visit_statement(&mut self, s: &Stmt) -> Self::S {
        match s {
            &Stmt::Expr(ref Expr) => {
                self.visit_expression(Expr).unwrap();
                Ok(())
            },
            Stmt::Print(Expr)=> {
                let e =self.visit_expression(Expr).unwrap();
                println!("{}", self.stringify(e));
                Ok(())
            },
            Stmt::VarDeclaration(Token,Expr) => {
                match Expr {
                    &Some(ref e) => {
                        let var_name = match Token.t_type.clone() {
                            TokenType::Identifier(x) => x,
                            _ => String::from(""),
                        };
                        let result =  self.visit_expression(&e).unwrap();
                        Ok(self.environment.define(var_name, Some(result)))

                    }
                    &None => {
                        let var_name = match Token.t_type.clone() {
                            TokenType::Identifier(x) => x,
                            _ => String::from("")
                        };
                        Ok(self.environment.define(var_name, None))

                    }
                }
            }
        }
    }


    // goes into every expression and does it recursively
    fn visit_expression(&mut self, e: &Expr) -> Self::E {
        match e {
            //executes for binary expression
            &Expr::Assign {
                ref name,
                ref value,
                ..
            } => {
                let new_value = self.visit_expression(value)?;

                self.environment.assign(&name, &new_value)?;
                return Ok(new_value)
            },

            &Expr::Binary {
                ref left,
                ref op,
                ref right,
                ..
            } => {
                let left_expr = self.visit_expression(left)?;
                let right_expr = self.visit_expression(right)?;

                return match (left_expr, op.clone(), right_expr) {
                    // For Strings
                    (Types::ReturnString(ls), t, Types::ReturnString(rs)) => match t.t_type.clone() {
                        TokenType::Plus => Ok(Types::ReturnString(String::from(format!("{}{}", ls, rs)))),
                        _ => Err(String::from(format!("cannot be appended to string {}", t.line.clone())))
                    },
                    //For number basic operation and comparison
                    (Types::Number(ln), t, Types::Number(rn)) => match t.t_type {
                        TokenType::Plus => Ok(Types::Number(ln + rn)),
                        TokenType::Minus => Ok(Types::Number(ln - rn)),
                        TokenType::Star => Ok(Types::Number(ln - rn)),
                        TokenType::Slash => if rn == 0.0 {
                            Err(String::from(format!("cannot be divided by zero at {}", t.line.clone())))
                        } else {
                            Ok(Types::Number(ln / rn))
                        },
                        TokenType::Greater => Ok(Types::Boolean(ln > rn)),
                        TokenType::GreaterEqual => Ok(Types::Boolean(ln >= rn)),
                        TokenType::Less => Ok(Types::Boolean(ln < rn)),
                        TokenType::LessEqual => Ok(Types::Boolean(ln <= rn)),
                        TokenType::EqualEqual => Ok(Types::Boolean(ln == rn)),
                        TokenType::BangEqual => Ok(Types::Boolean(ln != rn)),
                        _ => Err(String::from(format!("Invalid Expression type at line {}", t.line.clone())))
                    },
                    //For Type Nil
                    (Types::Nil, t, Types::Nil) => match t.t_type {
                        TokenType::Equal => Ok(Types::Boolean(true)),
                        TokenType::BangEqual => Ok(Types::Boolean(false)),
                        _ => Err(String::from(format!("cannot be divided by zero at {}", t.line.clone())))
                    }
                    //For Type boolean
                    (Types::Boolean(lb), t, Types::Boolean(rb)) => match t.t_type {
                        TokenType::Equal => Ok(Types::Boolean(lb == rb)),
                        TokenType::BangEqual => Ok(Types::Boolean(lb != rb)),
                        _ => Err(String::from(format!("Invalid Expression type at line {}", t.line.clone())))
                    },
                    _ => Err(String::from("Invalid ask Ivan for fix")),
                }
            },
            //For Grouping Expression
            &Expr::Grouping {
                ref expr
            } => self.visit_expression(expr),
            //For Literal returns a return the type
            &Expr::Literal {
                ref token
            } => match token.t_type.clone() {
                TokenType::Number(i) => Ok(Types::Number(i)),
                TokenType::True => Ok(Types::Boolean(true)),
                TokenType::False => Ok(Types::Boolean(false)),
                TokenType::Nil => Ok(Types::Nil),
                TokenType::String(s) => Ok(Types::ReturnString(s.clone())),
                _ => Err(String::from(format!("Invalid type at line {}", token.line.clone())))
            },

            // returns an unary expression
            &Expr::Unary {
                ref op,
                ref expr
            } => {
                let right = self.visit_expression(expr)?;
                match (right, op.t_type.clone()) {
                    //returns negative number
                    (Types::Number(n), TokenType::Minus) => Ok(Types::Number(-n)),
                    // returns type boolean if Boolean is false
                    (Types::Nil, TokenType::Bang) | (Types::Boolean(false), TokenType::Bang) => {
                        Ok(Types::Boolean(false))
                    },
                    // return false for any value idk whether it does for Zero lol
                    (_, TokenType::Bang) => Ok(Types::Boolean(false)),
                    _ => Err(String::from(format!("Invalid type at line {}", op.line.clone())))
                }
            },
            &Expr::Variable {
                ref name
            } => {
                let name = match name.t_type.clone() {
                    TokenType::Identifier(x) => x,
                    _ => String::from("")
                };
                match self.environment.get(name).unwrap() {
                    Some(v) => Ok(v),
                    None => Ok(Types::Nil)
                }
            }
        }
        }

    }



// A simple type system to make it usable like JAVA Object in the Book
#[derive(Debug, PartialEq, Clone)]
pub enum Types {
        Number(f64),
        ReturnString(String),
        Boolean(bool),
        Nil,
    }

// implements Display Trait to print
impl Display for Types {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &Types::Boolean(b) => write!(f, "{}", b),
            &Types::Nil => write!(f, "nil"),
            &Types::Number(n) => write!(f, "{}", n),
            &Types::ReturnString(ref s) => write!(f, "\"{}\"", s.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter() {
        // Define tokens representing the expression: print 5 + 3;
        let tokens = vec![
            Token {
                t_type: TokenType::Print,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(5.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Plus,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(3.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::EOF,
                lexeme: String::new(),
                line: 0,
            },
        ];

        // Create the parser and parse the tokens into statements
        let mut parser = Parser::new(tokens);
        let statements = parser.parse();

        // Create the interpreter and interpret the statements
        let mut interpreter = Interpreter::new();
        interpreter.interpret(statements.clone());

        // Print the executed statements for debugging
        for stmt in statements {
            println!("Executed statement: {:?}", stmt);
        }
    }

    #[test]
    fn test_var_declaration() {
        // Define tokens representing variable declarations: var x; and var y = 10 + 5;
        let tokens = vec![
            // var x;
            Token {
                t_type: TokenType::Var,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Identifier(String::from("x")),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: String::new(),
                line: 0,
            },
            // var y = 10 + 5;
            Token {
                t_type: TokenType::Var,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Identifier(String::from("y")),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Equal,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(10.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Plus,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(5.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::EOF,
                lexeme: String::new(),
                line: 0,
            },
        ];

        // Create the parser and parse the tokens into statements
        let mut parser = Parser::new(tokens);
        let statements = parser.parse();

        // Create the interpreter and interpret the statements
        let mut interpreter = Interpreter::new();
        interpreter.interpret(statements.clone());

        // Print the executed statements for debugging
        for stmt in statements {
            println!("Executed statement: {:?}", stmt);
        }
    }

    #[test]
    fn test_assignment() {
        let mut interpreter = Interpreter::new();

        // Simulate a variable declaration: var x = 10;
        let stmt1 = Stmt::VarDeclaration(
            Token {
                t_type: TokenType::Identifier(String::from("x")),
                lexeme: String::from("x"),
                line: 0,
            },
            Some(Expr::Literal {
                token: Token {
                    t_type: TokenType::Number(10.0),
                    lexeme: String::from("10"),
                    line: 0,
                },
            }),
        );

        // Execute the variable declaration
        interpreter.visit_statement(&stmt1).unwrap();

        // Simulate an assignment: x = 20;
        let stmt2 = Stmt::Expr(Expr::Assign {
            name: Token {
                t_type: TokenType::Identifier(String::from("x")),
                lexeme: String::from("x"),
                line: 0,
            },
            value: Box::new(Expr::Literal {
                token: Token {
                    t_type: TokenType::Number(20.0),
                    lexeme: String::from("20"),
                    line: 0,
                },
            }),
        });

        // Execute the assignment
        interpreter.visit_statement(&stmt2).unwrap();

        // Verify that the value of x has been updated to 20
        let value_of_x = interpreter.environment.get(String::from("x")).unwrap().unwrap();
        assert_eq!(value_of_x, Types::Number(20.0));
    }

    #[test]
    fn test_assignment_usage_in_different_statement() {
        let mut interpreter = Interpreter::new();

        // Simulate a variable declaration: var x = 5;
        let stmt1 = Stmt::VarDeclaration(
            Token {
                t_type: TokenType::Identifier(String::from("x")),
                lexeme: String::from("x"),
                line: 0,
            },
            Some(Expr::Literal {
                token: Token {
                    t_type: TokenType::Number(5.0),
                    lexeme: String::from("5"),
                    line: 0,
                },
            }),
        );

        // Execute the variable declaration
        interpreter.visit_statement(&stmt1).unwrap();

        // Simulate a variable declaration using the value of x: var y = x + x;
        let stmt2 = Stmt::VarDeclaration(
            Token {
                t_type: TokenType::Identifier(String::from("y")),
                lexeme: String::from("y"),
                line: 0,
            },
            Some(Expr::Binary {
                left: Box::new(Expr::Variable {
                    name: Token {
                        t_type: TokenType::Identifier(String::from("x")),
                        lexeme: String::from("x"),
                        line: 0,
                    },
                }),
                op: Token {
                    t_type: TokenType::Plus,
                    lexeme: String::from("+"),
                    line: 0,
                },
                right: Box::new(Expr::Variable {
                    name: Token {
                        t_type: TokenType::Identifier(String::from("x")),
                        lexeme: String::from("x"),
                        line: 0,
                    },
                }),
            }),
        );

        // Execute the variable declaration using the value of x
        interpreter.visit_statement(&stmt2).unwrap();

        // Verify that the value of y is equal to 10 (x + x)
        let value_of_y = interpreter.environment.get(String::from("y")).unwrap().unwrap();
        assert_eq!(value_of_y, Types::Number(10.0));
    }


}
