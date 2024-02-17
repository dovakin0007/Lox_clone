use crate::astprinter::AstPrinter;
use crate::token::{Token};

pub trait Visitor{
    type E;
    type S;
    fn visit_expression(&mut self, e:&Expr) -> String;
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary{
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        token: Token,
    },
    Unary {
        op: Token,
        expr: Box<Expr>,
    }
}

