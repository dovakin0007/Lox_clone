use std::fmt::format;
use crate::ast::*;
use crate::token::{Token, TokenType};
// use crate::token::TokenType::String;


//Ast Printer is used to print out the AST tree structure
#[derive(Debug, PartialEq)]
pub struct AstPrinter{
    str : String
}

impl AstPrinter {
    fn new() -> Self{
        Self{
            str: String::new(),
        }
    }
}

//Implements Visitor Trait
impl Visitor for AstPrinter {
    type E = String;
    type S = String;

    // Traverses tree recursively and represent an expression
    fn visit_expression(&mut self, e: &Expr) -> Self::E{
        match *e {
            Expr::Assign {
                ref name,
                ref value
            } => {
                format!("Assignment {:?} {:?}", name, self.visit_expression(value))
            },
            Expr::Binary {
                ref left,
                ref op,
                ref right,
                ..
            } => format!("(Binary {:?} {:?} {:?})", op, self.visit_expression(left), self.visit_expression(right)),
            Expr::Grouping {
                ref expr,
                ..
            } => format!("Group {:?}", self.visit_expression(expr)),
            Expr::Literal {
                ref token,
                ..
            } =>format!("Literal {:?}", token),
            Expr::Unary {
                ref op,
                ref expr,
                ..
            } => format!("(Unary {:?} {:?})", op, self.visit_expression(expr)),
            Expr::Variable {
                ref name,
                ..
            } => format!("(Variable {:?})", name)
        }
    }

    fn visit_statement(&mut self, s: &Stmt) -> Self::S {
        match *s {
            Stmt::Block(ref statements) => format!("(Block Statement {:?})", statements.iter().map(|x| self.visit_statement(s)).collect::<String>()),
            Stmt::Expr(ref expr) => format!("(Expression Statement {})", self.visit_expression(expr)),
            Stmt::Print(ref expr) => format!("Print Statement {}", self.visit_expression(expr).as_str()),
            Stmt::VarDeclaration(ref token, ref expr_opt) => format!("Variable Declaration {:?} {:?}", token,
                                                                     match expr_opt {
                &Some(ref expr) => self.visit_expression(expr),
                &None => "nil".to_string(),
            }
            )
        }
    }

}

#[cfg(test)]
#[test]
fn test_ast_printer() {
    let expr = Expr::Binary {
        left: Box::new(Expr::Literal {
            token: Token {
                t_type: TokenType::Number(1.0),
                lexeme: "1".to_string(),
                line: 1,
            },
        }),
        op: Token {
            t_type: TokenType::Plus,
            lexeme: "+".to_string(),
            line: 1,
        },
        right: Box::new(Expr::Literal {
            token: Token {
                t_type: TokenType::Number(2.0),
                lexeme: "2".to_string(),
                line: 1,
            },
        }),
    };

    let mut printer = AstPrinter::new();
     println!("{:?}", printer.visit_expression(&expr));
    println!("{}", printer.str);

    assert_eq!(true, true);
}
