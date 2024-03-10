use std::fmt;
use std::fmt::{Debug, Formatter, write};
use crate::token::{Token, TokenType};

pub fn error(line: u32, message:&str)  {
    report_err(line, "", message)
}

pub fn report_err(line: u32, where_: &str, message: &str) {
    eprintln!("[line {}] Error{}: {}", line, where_, message);
}

pub fn parse_error(token: &Token, message:&str) {
    if token.t_type == TokenType::EOF {
        report_err(token.line, "at end", message);
    }else {
        report_err(token.line, &format!(" at '{}'", token.lexeme), message);
    }
}

#[derive(Debug)]
pub enum Error {
    Parse,
    RunTime{
        token: Token,
        message: String,
    },
    InvalidStmt
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
     match self {
         Error::Parse => write!(f, "ParseError"),
         Error::RunTime {message, ..} => write!(f, "RuntimeError {}", message),
         Error::InvalidStmt => write!(f,"invalid Statement or Null")
     }
    }
}


impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Lox Error"
    }
}