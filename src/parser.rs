
use std::fmt::{Debug, Display};
use std::ops::Deref;
use crate::token::{Token, TokenType};
use crate::ast::Expr;


// #[derive(Debug)]
// pub struct ParseError;
//
//
// impl Display for ParseError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Oh no, something bad went down")
//     }
// }
//
// impl Error for ParseError {
// }

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current:0
        }
    }


    pub fn parse(&mut self) -> Expr{
        self.expression()
    }
    fn expression(&mut self) -> Expr {
        return self.equality()

    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Equal | TokenType::BangEqual => self.advance(),
            _ => None
        }{
            let operator: Token = t.clone();
            let right = self.comparison();

            expr = Expr::Binary { left: Box::new(expr), op: operator.clone(), right: Box::new( right)};
        }
        return expr
    }

    fn comparison(&mut self) -> Expr{
        let mut expr = self.term();
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Greater |
            TokenType::GreaterEqual |
            TokenType::Less| TokenType::LessEqual => self.advance(),
            _ => None
        }{
            let operator:Token =t.clone();
            let  right = self.term();
            expr = Expr::Binary {left: Box::new(expr), op: operator.clone(), right: Box::new(right)}

        }
        return expr
    }

    fn term(&mut self)-> Expr{
        let mut expr = self.factor();
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Plus |
            TokenType::Minus => self.advance(),
            _ => None
        }{
            let operator: Token = t.clone();
            let right = self.factor();
            expr = Expr::Binary {left: Box::new(expr), op: operator.clone(), right: Box::new(right)}
        }
        return expr
    }

    fn factor(&mut self) -> Expr {
        let mut  expr = self.unary();
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type{
            TokenType::Star |
            TokenType::Slash => self.advance(),
            _ => None
        }{
            let operator = t.clone();
            let right = self.unary();
            expr = Expr::Binary {left:Box::new(expr), op: operator.clone(), right: Box::new(right) }
        }
        return expr
    }

    fn unary(&mut self) -> Expr{
        //dbg!(&self.peek().unwrap());
        if let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Bang |
            TokenType::Minus => self.advance(),
            _ => None
        }{
            let operator = t.clone();
            let right = self.unary();
            return  Expr::Unary {op: operator.clone(),  expr: Box::new(right) }
        }
        return self.primary();
    }

    fn primary(&mut self) -> Expr {
        self.advance().unwrap();
        //dbg!(&self.peek().unwrap());
        let previous_token = self.previous().unwrap();
        return match previous_token.t_type.clone() {
            TokenType::False => Expr::Literal { token: previous_token.clone() },
            TokenType::True => Expr::Literal { token: previous_token.clone() },
            TokenType::Nil => Expr::Literal { token: previous_token.clone() },
            TokenType::String(_) => Expr::Literal { token: previous_token.clone() },
            TokenType::Number(_) => Expr::Literal {token: previous_token.clone()},
           TokenType::LeftParen => {
                let expr = self.expression();
                self.consume(TokenType::RightParen, "Expect ')' after expression.").unwrap();
                Expr::Grouping {expr:Box::new(expr)}
            },
            t => panic!("Expects a expression at {} {:?}", previous_token.line.clone(), previous_token)
        }

    }


    fn synchronize(&mut self){
        self.advance();
        while(self.peek().unwrap().t_type != TokenType::EOF){
            if(self.previous().unwrap().t_type == TokenType::SemiColon){
            return;
            }

            match self.peek().unwrap().t_type {
                TokenType::Class|
                TokenType::Fun|
                TokenType::Var|
                TokenType::For|
                TokenType::If|
                TokenType::While|
                TokenType::Print|
                TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }
    fn peek(&self) -> Option<&Token>  {
        let mut index = self.current;

        if index >= self.tokens.len(){
            None
        }else{
            self.tokens.get(index)
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        let x = self.peek().unwrap();
        if x.t_type!=TokenType::EOF {
            self.current += 1
        }
        return self.tokens.get(self.current - 1)
    }

    fn consume(&mut self, token_type: TokenType, error_msg:&str) -> Result<Token, ()> {
        if (token_type == self.peek().unwrap().t_type){
            return Ok(self.advance().unwrap().clone());

        }else{
            crate::error(self.peek().unwrap().clone(), error_msg);
            Err(())
        }
    }

    fn previous(&mut self) -> Option<&Token>{
        return self.tokens.get(self.current - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        // Tokens representing a simple expression: 1 + 2
        let tokens = vec![
            Token {
                t_type: TokenType::Number(1.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Plus,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(2.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::EOF,
                lexeme: String::new(),
                line: 0,
            },
        ];

        let mut parser = Parser::new(tokens);
        let expr = parser.parse();

        // Verify that the parsed expression matches the expected structure
        assert_eq!(
            expr,
            Expr::Binary {
                left: Box::new(Expr::Literal {
                    token: Token {
                        t_type: TokenType::Number(1.0),
                        lexeme: String::new(),
                        line: 0,
                    }
                }),
                op: Token {
                    t_type: TokenType::Plus,
                    lexeme: String::new(),
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    token: Token {
                        t_type: TokenType::Number(2.0),
                        lexeme: String::new(),
                        line: 0,
                    }
                })
            }
        );
    }

    #[test]
    fn test_parse_with_parentheses() {
        // Tokens representing the expression: (1 + 2) * 3
        let tokens = vec![
            Token {
                t_type: TokenType::LeftParen,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(1.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Plus,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(2.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::RightParen,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Star,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(3.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::EOF,
                lexeme: String::new(),
                line: 0,
            },
        ];

        let mut parser = Parser::new(tokens);
        let expr = parser.parse();

        // Verify that the parsed expression matches the expected structure
        assert_eq!(
            expr,
            Expr::Binary {
                left: Box::new(Expr::Grouping {
                    expr: Box::new(Expr::Binary {
                        left: Box::new(Expr::Literal {
                            token: Token {
                                t_type: TokenType::Number(1.0),
                                lexeme: String::new(),
                                line: 0,
                            }
                        }),
                        op: Token {
                            t_type: TokenType::Plus,
                            lexeme: String::new(),
                            line: 0,
                        },
                        right: Box::new(Expr::Literal {
                            token: Token {
                                t_type: TokenType::Number(2.0),
                                lexeme: String::new(),
                                line: 0,
                            }
                        })
                    })
                }),
                op: Token {
                    t_type: TokenType::Star,
                    lexeme: String::new(),
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    token: Token {
                        t_type: TokenType::Number(3.0),
                        lexeme: String::new(),
                        line: 0,
                    }
                })
            }
        );
    }

}
