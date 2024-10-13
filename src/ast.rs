use crate::token::Token;

//Visitor is a way to Traverse the Syntax tree and store it in a complex tree structure
// there is much more robust way to write it but I went with something simple

pub trait Visitor {
    type E;
    type S;
    fn visit_expression(&mut self, e: &Expr) -> Self::E;
    fn visit_statement(&mut self, s: &Stmt) -> Self::S;
}

//Represents an expression which gets stored in AST
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        token: Token,
    },
    Logical {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Unary {
        op: Token,
        expr: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Class(Token, Vec<Stmt>),
    Expr(Expr),
    Function(Token, Vec<Token>, Vec<Stmt>),
    IfStmt(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Return(Token, Option<Expr>),
    VarDeclaration(Token, Option<Expr>),
    While(Expr, Box<Stmt>),
    Null,
}
