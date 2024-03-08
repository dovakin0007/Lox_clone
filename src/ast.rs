use crate::token::{Token};


//Visitor is a way to Traverse the Syntax tree and store it in a complex tree structure
// there is much more robust way to write it but I went with something simple
pub trait Visitor{
    type E;
    type S;
    fn visit_expression(&mut self, e:&Expr) -> Self::E;
    fn visit_statement(&mut self, s: &Stmt) -> Self::S;
}


//Represents an expression which gets stored in AST
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {

    Assign {
        name: Token,
        value: Box<Expr>
    },
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
    },
    Variable{
        name: Token
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Expr),
    Print(Expr),
    VarDeclaration(Token, Option<Expr>)
}

