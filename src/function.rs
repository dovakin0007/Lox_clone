use std::fmt::{Debug, Display, Formatter};
use log::debug;
use crate::ast::Stmt;
use crate::environment::Environment;
use crate::error::Error;
use crate::interpreter::{Interpreter, Types};
use crate::token::Token;

pub trait Callable: Debug + Display {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: &Vec<Types>) -> Result<Types, Error>;
}

pub struct NativeFunction {
        pub arity:usize,
        pub body: Box<fn(&Vec<Types>) -> Types>
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native func>")
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native func>")
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, _: &mut Interpreter, args: &Vec<Types>) -> Result<Types, Error> {
        return Ok((self.body)(args));
    }

}

pub struct UserFunction {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
    pub closure: Environment,
    pub is_initializer: bool,
}

impl Debug for UserFunction{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "function {}", &self.name)
    }
}


impl Display for UserFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "function {}", &self.name)
    }
}

impl Callable for UserFunction{
    fn arity(&self) -> usize {
        return self.params.len();
    }

    fn call(&self, interpreter: &mut Interpreter, args: &Vec<Types>) -> Result<Types, Error> {
        let environement = Environment::from(self.closure.clone());

        for (param, arguments) in self.params.iter().zip(args.iter()){
            environement.define(param.lexeme.clone(), Option::from(arguments.clone()))
        }
       let is_error= interpreter.execute_block(&self.body, environement);

        match is_error {
            Err(Error::Return {
                value
                })=> {
                Ok(value)
            },
            Err(other) => Err(other),
            Ok(..) => {
                Ok(Types::Nil)
            }
        }

    }
}
