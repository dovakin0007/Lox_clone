use crate::ast::*;
use crate::token::{Token, TokenType};


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
impl Visitor for AstPrinter {
    type E = String;
    type S = String;

    fn visit_expression(&mut self, e: &Expr) -> String{
        match *e {
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
            } => format!("(Unary {:?} {:?})", op, self.visit_expression(expr))
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
