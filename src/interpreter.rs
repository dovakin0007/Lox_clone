use crate::ast::{Expr, Stmt, Visitor};
use crate::environment::Environment;
use crate::error::Error::InvalidStmt;
use crate::error::{error, Error};
use crate::function;
use crate::function::{Callable, NativeFunction, UserFunction};
use crate::token::{Token, TokenType};
use std::cmp::PartialEq;
use std::env::args;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::rc::Rc;
use std::string::String;
use std::time::{SystemTime, UNIX_EPOCH};
use log::debug;
use crate::ast::Stmt::Function;

//represents an Interpreter struct

pub struct Interpreter {
    pub globals: Environment,
    environment: Environment,
}

macro_rules! istruthy {
    ($i:expr) => {{
        match $i {
            Types::Nil | Types::Boolean(false) => false,
            _ => true,
        }
    }};
}

impl Interpreter {
    //Does nothing for now?
    pub fn new() -> Self {
        let globals = Environment::new();

        let clock: Types = Types::Callable(Rc::new(Box::new(NativeFunction {
            arity: 0,
            body: Box::new(|args: &Vec<Types>| {
                Types::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Could not retrieve time.")
                        .as_millis() as f64,
                )
            }),
        })));

        let asset_eql = Types::Callable(Rc::new(Box::new(NativeFunction {
            arity: 2,
            body: Box::new(|args: &Vec<Types>| {
                Types::Boolean(args[0].clone().to_string() == args[1].clone().to_string())
            }),
        })));
        globals.define("assert".to_string(), Some(asset_eql));
        globals.define("clock".to_string(), Some(clock));
        Self {
            globals: globals.clone(),
            environment: globals.clone(),
        }
    }

    // for now it returns Executed types
    pub fn interpret(&mut self, statement: Vec<Stmt>) -> Result<(), Error> {
        for x in statement {
            self.visit_statement(&x)?;
        }
        Ok(())
    }

    fn stringify(&self, types: Types) -> String {
        match types {
            Types::Boolean(b) => b.to_string(),
            Types::Nil => "nil".to_string(),
            Types::Number(n) => n.to_string(),
            Types::ReturnString(s) => s,
            Types::Callable(f) => f.to_string(),
        }
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, env: Environment) -> Result<(), Error> {
        let previous = self.environment.clone();
        let steps = || -> Result<(), Error> {
            self.environment = env;

            for statement in statements {
                self.visit_statement(&statement)?
            }
            Ok(())
        };
        let result = steps();
        self.environment = previous;
        result
    }
}
impl Visitor for Interpreter {
    type E = Result<Types, Error>;
    type S = Result<(), Error>;
    fn visit_statement(&mut self, s: &Stmt) -> Self::S {
        match s {
            &Stmt::Block(ref stmts) => {
                self.execute_block(stmts, Environment::from(self.environment.clone()))?;
                Ok(())
            }
            &Stmt::Expr(ref Expr) => {
                self.visit_expression(Expr)?;
                Ok(())
            }

            &Stmt::Function(ref name_token, ref parameters, ref body) => {
                let user_function = Types::Callable(Rc::new(Box::new(UserFunction{
                    name: name_token.clone(),
                    params: parameters.clone(),
                    body: body.clone(),
                    closure: self.environment.clone(),
                    is_initializer: false
                })));

                self.environment.define(name_token.lexeme.clone(), Some(user_function));

                Ok(())
            }
            &Stmt::Return(ref Token, ref expr) => {

                let return_value = expr.clone().map(|v| self.visit_expression(&v))
                    .unwrap_or(Ok(Types::Nil))?;

                Err(Error::Return {value: return_value})
            }
            &Stmt::IfStmt(ref Expr, ref then, ref else_option) => {
                if istruthy!(&self.visit_expression(Expr)?) {
                    self.visit_statement(then)?;
                } else {
                    if let Some(ref else_stmt) = else_option {
                        self.visit_statement(else_stmt)?;
                    }
                }
                Ok(())
            }

            Stmt::Print(Expr) => {
                let e = self.visit_expression(Expr)?;
                println!("{}", self.stringify(e));
                Ok(())
            }

            Stmt::VarDeclaration(Token, Expr) => match Expr {
                &Some(ref e) => {
                    let var_name = match Token.t_type.clone() {
                        TokenType::Identifier(x) => x,
                        _ => String::from(""),
                    };
                    let result = self.visit_expression(&e)?;
                    Ok(self.environment.define(var_name, Some(result)))
                }
                &None => {
                    let var_name = match Token.t_type.clone() {
                        TokenType::Identifier(x) => x,
                        _ => String::from(""),
                    };
                    Ok(self.environment.define(var_name, None))
                }
            },
            Stmt::While(ref Expr, ref Stmt) => {
                while istruthy!(self.visit_expression(Expr)?) {
                    self.visit_statement(Stmt)?
                }
                Ok(())
            }
            _ => Err(InvalidStmt),
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
                self.environment.assign(&name, new_value.clone())?;
                return Ok(new_value);
            }
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
                    (Types::ReturnString(ls), t, Types::ReturnString(rs)) => match t.t_type.clone()
                    {
                        TokenType::Plus => {
                            Ok(Types::ReturnString(String::from(format!("{}{}", ls, rs))))
                        }
                        _ => Err(Error::RunTime {
                            token: t.clone(),
                            message: "Operands must be two numbers or two strings".to_string(),
                        }),
                    },
                    //For number basic operation and comparison
                    (Types::Number(ln), t, Types::Number(rn)) => match t.t_type {
                        TokenType::Plus => Ok(Types::Number(ln + rn)),
                        TokenType::Minus => Ok(Types::Number(ln - rn)),
                        TokenType::Star => Ok(Types::Number(ln - rn)),
                        TokenType::Slash => {
                            if rn == 0.0 {
                                Err(Error::RunTime {
                                    token: t.clone(),
                                    message: "Operands must be two numbers or two strings"
                                        .to_string(),
                                })
                            } else {
                                Ok(Types::Number(ln / rn))
                            }
                        }
                        TokenType::Greater => Ok(Types::Boolean(ln > rn)),
                        TokenType::GreaterEqual => Ok(Types::Boolean(ln >= rn)),
                        TokenType::Less => Ok(Types::Boolean(ln < rn)),
                        TokenType::LessEqual => Ok(Types::Boolean(ln <= rn)),
                        TokenType::EqualEqual => Ok(Types::Boolean(ln == rn)),
                        TokenType::BangEqual => Ok(Types::Boolean(ln != rn)),
                        _ => Err(Error::RunTime {
                            token: t.clone(),
                            message: "Operands must be two numbers to compare".to_string(),
                        }),
                    },
                    //For Type Nil
                    (Types::Nil, t, Types::Nil) => match t.t_type {
                        TokenType::Equal => Ok(Types::Boolean(true)),
                        TokenType::BangEqual => Ok(Types::Boolean(false)),
                        _ => Err(Error::RunTime {
                            token: t.clone(),
                            message: "Operands must be nil ".to_string(),
                        }),
                    },
                    //For Type boolean
                    (Types::Boolean(lb), t, Types::Boolean(rb)) => match t.t_type {
                        TokenType::Equal => Ok(Types::Boolean(lb == rb)),
                        TokenType::BangEqual => Ok(Types::Boolean(lb != rb)),
                        _ => Err(Error::RunTime {
                            token: t.clone(),
                            message: "Operands must be boolean".to_string(),
                        }),
                    },
                    _ => Err(Error::RunTime {
                        token: op.clone(),
                        message: "Idk what to print".to_string(),
                    }),
                };
            }
            //For Grouping Expression
            &Expr::Grouping { ref expr } => self.visit_expression(expr),
            //For Literal returns a return the type
            &Expr::Literal { ref token } => match token.t_type.clone() {
                TokenType::Number(i) => Ok(Types::Number(i)),
                TokenType::True => Ok(Types::Boolean(true)),
                TokenType::False => Ok(Types::Boolean(false)),
                TokenType::Nil => Ok(Types::Nil),
                TokenType::String(s) => Ok(Types::ReturnString(s.clone())),
                _ => Err(Error::RunTime {
                    token: token.clone(),
                    message: "That's not a literal".to_string(),
                }),
            },

            &Expr::Logical {
                ref left,
                ref op,
                ref right,
            } => {
                let left_result = self.visit_expression(left)?;
                if op.t_type == TokenType::Or {
                    if istruthy!(left_result) {
                        return Ok(left_result);
                    }
                } else {
                    if !(istruthy!(left_result)) {
                        return Ok(left_result);
                    }
                }
                self.visit_expression(right)
            }

            // returns an unary expression
            &Expr::Unary { ref op, ref expr } => {
                let right = self.visit_expression(expr)?;
                match (right, op.t_type.clone()) {
                    //returns negative number
                    (Types::Number(n), TokenType::Minus) => Ok(Types::Number(-n)),
                    // returns type boolean if Boolean is false
                    (Types::Nil, TokenType::Bang) | (Types::Boolean(false), TokenType::Bang) => {
                        Ok(Types::Boolean(false))
                    }
                    // return false for any value idk whether it does for Zero lol
                    (_, TokenType::Bang) => Ok(Types::Boolean(false)),
                    _ => Err(Error::RunTime {
                        token: op.clone(),
                        message: "Invalid unary Expression".to_string(),
                    }),
                }
            }
            &Expr::Call {
                ref callee,
                ref paren,
                ref arguments,
                ..
            } => {
                let callee_value = self.visit_expression(callee)?;
                let arguments: Result<Vec<Types>, Error> = arguments
                    .into_iter()
                    .map(|e| self.visit_expression(e))
                    .collect();
                let args = arguments?;

                if let Types::Callable(function) = callee_value {
                    let args_len = args.len();
                    if args_len != function.arity() {
                        Err(Error::RunTime {
                            token: paren.clone(),
                            message: format!(
                                "Expected {} arguments but got {}.",
                                function.arity(),
                                args_len
                            ),
                        })
                    } else {
                        function.call(self, &args)
                    }
                } else {
                    Err(Error::RunTime {
                        token: paren.clone(),
                        message: "Can only call functions and classes.".to_string(),
                    })
                }
            }
            &Expr::Variable { ref name, .. } => {
                let name = name.lexeme.clone();
                match self.environment.get(name)? {
                    Some(v) => Ok(v),
                    None => Ok(Types::Nil),
                }
            }
        }
    }
}

// A simple type system to make it usable like JAVA Object in the Book
#[derive(Debug, Clone)]
pub enum Types {
    Number(f64),
    ReturnString(String),
    Boolean(bool),
    Nil,
    Callable(Rc<Box<dyn Callable>>),
}

// implements Display Trait to print
impl Display for Types {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &Types::Boolean(b) => write!(f, "{}", b),
            &Types::Nil => write!(f, "nil"),
            &Types::Number(n) => write!(f, "{}", n),
            &Types::ReturnString(ref s) => write!(f, "\"{}\"", s.to_string()),
            &Types::Callable(ref call) => write!(f, "{}", call),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn test_assignment() {
//         let mut interpreter = Interpreter::new();
//
//         // Simulate a variable declaration: var x = 10;
//         let stmt1 = Stmt::VarDeclaration(
//             Token {
//                 t_type: TokenType::Identifier(String::from("x")),
//                 lexeme: String::from("x"),
//                 line: 0,
//             },
//             Some(Expr::Literal {
//                 token: Token {
//                     t_type: TokenType::Number(10.0),
//                     lexeme: String::from("10"),
//                     line: 0,
//                 },
//             }),
//         );
//
//         // Execute the variable declaration
//         interpreter.visit_statement(&stmt1).unwrap();
//
//         // Simulate an assignment: x = 20;
//         let stmt2 = Stmt::Expr(Expr::Assign {
//             name: Token {
//                 t_type: TokenType::Identifier(String::from("x")),
//                 lexeme: String::from("x"),
//                 line: 0,
//             },
//             value: Box::new(Expr::Literal {
//                 token: Token {
//                     t_type: TokenType::Number(20.0),
//                     lexeme: String::from("20"),
//                     line: 0,
//                 },
//             }),
//         });
//
//         // Execute the assignment
//         interpreter.visit_statement(&stmt2).unwrap();
//
//         // Verify that the value of x has been updated to 20
//         let value_of_x = interpreter.environment.get(String::from("x")).unwrap().unwrap();
//         assert_eq!(value_of_x, Types::Number(20.0));
//     }
//
//     #[test]
//     fn test_assignment_usage_in_different_statement() {
//         let mut interpreter = Interpreter::new();
//
//         // Simulate a variable declaration: var x = 5;
//         let stmt1 = Stmt::VarDeclaration(
//             Token {
//                 t_type: TokenType::Identifier(String::from("x")),
//                 lexeme: String::from("x"),
//                 line: 0,
//             },
//             Some(Expr::Literal {
//                 token: Token {
//                     t_type: TokenType::Number(5.0),
//                     lexeme: String::from("5"),
//                     line: 0,
//                 },
//             }),
//         );
//
//         // Execute the variable declaration
//         interpreter.visit_statement(&stmt1).unwrap();
//
//         // Simulate a variable declaration using the value of x: var y = x + x;
//         let stmt2 = Stmt::VarDeclaration(
//             Token {
//                 t_type: TokenType::Identifier(String::from("y")),
//                 lexeme: String::from("y"),
//                 line: 0,
//             },
//             Some(Expr::Binary {
//                 left: Box::new(Expr::Variable {
//                     name: Token {
//                         t_type: TokenType::Identifier(String::from("x")),
//                         lexeme: String::from("x"),
//                         line: 0,
//                     },
//                 }),
//                 op: Token {
//                     t_type: TokenType::Plus,
//                     lexeme: String::from("+"),
//                     line: 0,
//                 },
//                 right: Box::new(Expr::Variable {
//                     name: Token {
//                         t_type: TokenType::Identifier(String::from("x")),
//                         lexeme: String::from("x"),
//                         line: 0,
//                     },
//                 }),
//             }),
//         );
//
//         // Execute the variable declaration using the value of x
//         interpreter.visit_statement(&stmt2).unwrap();
//
//         // Verify that the value of y is equal to 10 (x + x)
//         let value_of_y = interpreter.environment.get(String::from("y")).unwrap().unwrap();
//         assert_eq!(value_of_y, Types::Number(10.0));
//     }
//
//     #[test]
//     fn test_block_statements() {
//         // Define tokens representing a block of code: { var x = 5; var y = x + 10; }
//         let tokens = vec![
//             Token {
//                 t_type: TokenType::LeftBrace,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Var,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Identifier(String::from("x")),
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Equal,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Number(5.0),
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::SemiColon,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Var,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Identifier(String::from("y")),
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Equal,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Identifier(String::from("x")),
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Plus,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Number(10.0),
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::SemiColon,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::RightBrace,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::EOF,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//         ];
//
//         // Create the parser and parse the tokens into statements
//         let mut parser = Parser::new(tokens);
//         let statements = parser.parse();
//
//         // Create the interpreter and interpret the statements
//         let mut interpreter = Interpreter::new();
//         let _ = interpreter.interpret(statements.unwrap().clone());
//
//         // Verify the values of x and y after executing the block
//         let value_of_x = interpreter.environment.get(String::from("x")).unwrap().unwrap();
//         let value_of_y = interpreter.environment.get(String::from("y")).unwrap().unwrap();
//
//         assert_eq!(value_of_x, Types::Number(5.0));
//         assert_eq!(value_of_y, Types::Number(15.0)); // y = x + 10
//     }
//     #[test]
//     fn test_if_statement() {
//         // Define tokens representing the if statement: if (x == 5) { print "true"; }
//         let tokens = vec![
//             Token {
//                 t_type: TokenType::If,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::LeftParen,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Identifier(String::from("x")),
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::EqualEqual,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Number(5.0),
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::RightParen,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::LeftBrace,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::Print,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::String(String::from("true")),
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::SemiColon,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::RightBrace,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//             Token {
//                 t_type: TokenType::EOF,
//                 lexeme: String::new(),
//                 line: 0,
//             },
//         ];
//
//         // Create the parser and parse the tokens into statements
//         let mut parser = Parser::new(tokens);
//         let statements = parser.parse().unwrap();
//
//         // Create the interpreter and interpret the statements
//         let mut interpreter = Interpreter::new();
//         interpreter.interpret(statements);
//
//         // As the print statement inside the if block should execute, it should print "true".
//         // You may need to redirect stdout to capture the printed output for testing.
//         // Here, we are assuming it's printed directly to stdout for simplicity.
//     }
//
//
// }
