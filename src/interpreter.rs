use std::string::String;
use std::fmt::{Display, Formatter, Result as FmtResult};
use crate::ast::{Expr, Visitor};
use crate::token::{Token, TokenType};

//represents an Interpreter struct
#[derive(Default)]
pub struct Interpreter {}

//
impl Interpreter {
    //Does nothing for now?
    pub fn new() -> Self {
        Default::default()
    }
    // for now it returns Executed types
    pub fn interpret(&mut self, expr: Expr) -> Types {
        self.visit_expression(&expr).unwrap_or_else(|msg| {
            eprintln!("{msg}");
            panic!()
        })
    }
}
impl Visitor for Interpreter {
    type E = Result<Types, String>;
    type S = Result<(), String>;


    // goes into every expression and does it recursively
    fn visit_expression(&mut self, e: &Expr) -> Self::E {
        match e {

            //executes for binary expression
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
            }
        }


        }
    }



// A simple type system to make it usable like JAVA Object in the Book
#[derive(Debug, PartialEq)]
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

#[test]
fn test_interpreter() {

    // Written by chat gpt
    let mut interpreter = Interpreter {};

    // Test addition of two numbers
    let expr1 = Expr::Binary {
        left: Box::new(Expr::Literal {
            token: Token {
                t_type: TokenType::Number(10.0),
                lexeme: String::new(),
                line: 0,
            },
        }),
        op: Token {
            t_type: TokenType::Plus,
            lexeme: String::new(),
            line: 0,
        },
        right: Box::new(Expr::Literal {
            token: Token {
                t_type: TokenType::Number(5.0),
                lexeme: String::new(),
                line: 0,
            },
        }),
    };
    let result1 = interpreter.interpret(expr1);
    println!("Result of expr1: {:?}", result1);
    assert_eq!(result1, Types::Number(15.0));

    // Test concatenation of two strings
    let expr2 = Expr::Binary {
        left: Box::new(Expr::Literal {
            token: Token {
                t_type: TokenType::String(String::from("Hello")),
                lexeme: String::new(),
                line: 0,
            },
        }),
        op: Token {
            t_type: TokenType::Plus,
            lexeme: String::new(),
            line: 0,
        },
        right: Box::new(Expr::Literal {
            token: Token {
                t_type: TokenType::String(String::from(" World")),
                lexeme: String::new(),
                line: 0,
            },
        }),
    };
    let result2 = interpreter.interpret(expr2);
    println!("Result of expr2: {:?}", result2);
    assert_eq!(
        result2,
        Types::ReturnString(String::from("Hello World"))
    );

    // Test unary negation
    let expr3 = Expr::Unary {
        op: Token {
            t_type: TokenType::Minus,
            lexeme: String::new(),
            line: 0,
        },
        expr: Box::new(Expr::Literal {
            token: Token {
                t_type: TokenType::Number(10.0),
                lexeme: String::new(),
                line: 0,
            },
        }),
    };
    let result3 = interpreter.interpret(expr3);
    println!("Result of expr3: {:?}", result3);
    assert_eq!(result3, Types::Number(-10.0));
}
