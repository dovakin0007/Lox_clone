use std::io::ErrorKind::Other;
use crate::token::{Token, TokenType};
use crate::ast::{Expr, Stmt};
use crate::error::{parse_error, Error};


//Parser takes an input of tokens
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

    //returns an expression tree from the Vector
    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error>{
        let mut statements: Vec<Stmt> = Vec::new();
        while(self.peek().unwrap().t_type != TokenType::EOF){
            statements.push(self.declaration()?)
        }
        return Ok(statements)
    }


    fn declaration(&mut self) -> Result<Stmt, Error> {
        let statement = match self.peek().unwrap().t_type {
            TokenType::Var => self.var_declaration(),
            _ => self.statement()

        };
        match statement {
            Err(Error::Parse) => {
                self.synchronize();
                Ok(Stmt::Null)
            }
            other => other
        }
    }

    fn var_declaration (&mut self) -> Result<Stmt, Error> {
        let token = self.consume_identifier("Expect variable name").unwrap().clone();
        let initializer = if self.peek().unwrap().t_type.clone() == TokenType::Equal{
            self.advance();
            Some(self.expression()?)
        }else { None };

        let _ = self.consume(self.peek().unwrap().t_type.clone(), "Expected `;` after variable declaration").unwrap();
        Ok(Stmt::VarDeclaration(token, initializer))
    }



    fn statement(&mut self) -> Result<Stmt, Error> {
        return match self.peek().unwrap().t_type {
            TokenType::Print => {
                self.advance();
                self.print_statement()
            },
            TokenType::LeftBrace => {
                self.advance();
                Ok(Stmt::Block(self.block_statement()?))
            },
            _=> self.expression_statement()

        }
    }

    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let expr =self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error>{
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Expr(expr))
    }

    fn block_statement(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements: Vec<Stmt> = Vec::new();
        while match self.peek().unwrap().t_type.clone() {
            TokenType::RightBrace | TokenType::EOF=> false,
            _ => true,
        }{
            statements.push(self.declaration()?)
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.").unwrap();
        return  Ok(statements)
    }
    //using recursive decent parsing method
    fn expression(&mut self) -> Result<Expr, Error> {
        return Ok(self.assignment()?);
    }

    fn assignment(&mut self) -> Result<Expr, Error> {

       let mut expr = self.equality()?;

        if let Some(_) = match self.peek().unwrap().t_type {
            TokenType::Equal => self.advance(),
            _ => None,
        }{
            let value = self.assignment()?;
            match expr {
                Expr::Variable { name, ..} => {
                    return Ok(Expr::Assign {
                        name: name,
                        value: Box::new(value)
                    })

                }
                _ => panic!("unable to assign ")
            }

        }
        return Ok(expr)
    }

    //checks whether the expression is an equality expression returns an expression
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::EqualEqual | TokenType::BangEqual => self.advance(),
            _ => None
        }{
            let operator: Token = t.clone();
            let right = self.comparison()?;

            expr = Expr::Binary { left: Box::new(expr), op: operator.clone(), right: Box::new( right)};
        }
        return Ok(expr)
    }

    //checks whether the given expression is comparison operation returns an expression
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Greater |
            TokenType::GreaterEqual |
            TokenType::Less| TokenType::LessEqual => self.advance(),
            _ => None
        }{
            let operator:Token =t.clone();
            let  right = self.term()?;
            expr = Expr::Binary {left: Box::new(expr), op: operator.clone(), right: Box::new(right)}

        }
        return Ok(expr)
    }

    //checks whether the expression is add or sub and returns an expression
    fn term(&mut self)-> Result<Expr, Error> {
        let mut expr = self.factor()?;
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Plus |
            TokenType::Minus => self.advance(),
            _ => None
        }{
            let operator: Token = t.clone();
            let right = self.factor()?;
            expr = Expr::Binary {left: Box::new(expr), op: operator.clone(), right: Box::new(right)}
        }
        return Ok(expr)
    }

    //checks whether the expression is mul or div and returns an expression
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut  expr = self.unary()?;
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type{
            TokenType::Star |
            TokenType::Slash => self.advance(),
            _ => None
        }{
            let operator = t.clone();
            let right = self.unary()?;
            expr = Expr::Binary {left:Box::new(expr), op: operator.clone(), right: Box::new(right) }
        }
        return Ok(expr)
    }
    // returns an unary expression
    fn unary(&mut self) -> Result<Expr, Error>{
        //dbg!(&self.peek().unwrap());
        if let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Bang |
            TokenType::Minus => self.advance(),
            _ => None
        }{
            let operator = t.clone();
            let right = self.unary()?;
            return Ok(Expr::Unary { op: operator.clone(), expr: Box::new(right) })
        }
        return Ok(self.primary()?);
    }
    // returns a literal such as String, bool, Number and also grouping expression
    fn primary(&mut self) -> Result<Expr, Error> {
        // dbg!(&self.peek().unwrap());
        let previous_token = self.peek().unwrap().clone();

        let expr =  match previous_token.t_type {
            TokenType::False => Expr::Literal { token: previous_token.clone() },
            TokenType::True => Expr::Literal { token: previous_token.clone() },
            TokenType::Nil => Expr::Literal { token: previous_token.clone() },
            TokenType::String(_) => Expr::Literal { token: previous_token.clone() },
            TokenType::Number(_) => Expr::Literal {token: previous_token.clone()},
           TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.").unwrap();
                Expr::Grouping {expr:Box::new(expr)}
            },
            TokenType::Identifier(_) => {

                Expr::Variable {
                name: previous_token.clone()
                }
            },


            _ => return Err(self.error(self.peek().unwrap(), "Expect expression.")),
        };
        self.advance();
        return Ok(expr)
    }

   //TODO
   fn error(&self, token:&Token, message: &str) -> Error{
       parse_error(token, message);
       Error::Parse
   }

   fn synchronize(&mut self){
        self.advance();
        while self.peek().unwrap().t_type != TokenType::EOF{
            if self.previous().unwrap().t_type == TokenType::SemiColon{
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
                _ => self.advance()
            };
        }
    }

    //return the current value or current token based on the index
    fn peek(&self) -> Option<&Token>  {
        let index = self.current;
        if index >= self.tokens.len(){
            None
        }else{
            self.tokens.get(index)
        }
    }

    //advance the current to next and returns previous index value in the vector
    fn advance(&mut self) -> Option<&Token> {
        let x = self.peek().unwrap();
        if x.t_type!=TokenType::EOF {
            self.current += 1
        }
        return self.tokens.get(self.current - 1)
    }

    //checks if right parenthesis exists or throws an error
    fn consume(&mut self, token_type: TokenType, error_msg:&str) -> Result<Token, Error> {
        if token_type == self.peek().unwrap().t_type{
            return Ok(self.advance().unwrap().clone());

        }else{

            Err(self.error(&self.peek().unwrap(), error_msg))
        }
    }

    fn consume_identifier(&mut self, error_msg: &str) -> Result<Token, Error> {
        self.advance().unwrap();
        let token_type = self.peek().unwrap().t_type.clone();
        match token_type {
            TokenType::Identifier(_) => self.consume(token_type, error_msg),
            _ => Err(self.error(&self.peek().unwrap(), error_msg))
        }

    }
    //gets the previous element in the vector
    fn previous(&mut self) -> Option<&Token>{
        return self.tokens.get(self.current - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create tokens for testi

    #[test]
    fn test_variable_declaration() {
        // Define tokens representing the variable declaration: var x = 5;
        let tokens = vec![
            Token {
                t_type: TokenType::Var,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Identifier(String::from("x")),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Equal,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(5.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::EOF,
                lexeme: String::new(),
                line: 0,
            },
        ];

        // Create the parser and parse the tokens into statements
        let mut parser = Parser::new(tokens);
        let statements = parser.parse();

        // Validate the output
        assert_eq!(
            statements.unwrap(),
            vec![Stmt::VarDeclaration(
                Token {
                    t_type: TokenType::Identifier(String::from("x")),
                    lexeme: String::new(),
                    line: 0,
                },
                Some(Expr::Literal {
                    token: Token {
                        t_type: TokenType::Number(5.0),
                        lexeme: String::new(),
                        line: 0,
                    },
                })
            )]
        );
    }

    #[test]
    fn test_assignment() {
        // Tokens representing the assignment: x = 5;
        let tokens = vec![
            Token {
                t_type: TokenType::Identifier(String::from("x")),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Equal,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(5.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::EOF,
                lexeme: String::new(),
                line: 0,
            },
        ];

        // Create the parser and parse the tokens into statements
        let mut parser = Parser::new(tokens);
        let statements = parser.parse();

        // Validate the output
        assert_eq!(
            statements.unwrap(),
            vec![Stmt::Expr(Expr::Assign {
                name: Token {
                    t_type: TokenType::Identifier(String::from("x")),
                    lexeme: String::new(),
                    line: 0,
                },
                value: Box::new(Expr::Literal {
                    token: Token {
                        t_type: TokenType::Number(5.0),
                        lexeme: String::new(),
                        line: 0,
                    },
                }),
            })]
        );
    }

    #[test]
    fn test_block_statement_with_variables() {
        // Define tokens representing the block statement: var x = 5; { x=10; var y = 10; print x; }
        let tokens = vec![
            // Variable declaration: var x = 5;
            Token {
                t_type: TokenType::Var,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Identifier(String::from("x")),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Equal,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(5.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: String::new(),
                line: 0,
            },
            // Block start
            Token {
                t_type: TokenType::LeftBrace,
                lexeme: String::new(),
                line: 0,
            },
            // Assignment: x=10;
            Token {
                t_type: TokenType::Identifier(String::from("x")),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Equal,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(10.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: String::new(),
                line: 0,
            },
            // Variable declaration: var y = 10;
            Token {
                t_type: TokenType::Var,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Identifier(String::from("y")),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Equal,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(10.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: String::new(),
                line: 0,
            },
            // Print statement: print x;
            Token {
                t_type: TokenType::Print,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Identifier(String::from("x")),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: String::new(),
                line: 0,
            },
            // Block end
            Token {
                t_type: TokenType::RightBrace,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::EOF,
                lexeme: String::new(),
                line: 0,
            },
        ];

        // Create the parser and parse the tokens into statements
        let mut parser = Parser::new(tokens);
        let statements = parser.parse().unwrap();

        // Validate the output
        dbg!(statements.clone());
        assert_eq!(
            statements,
            vec![
                Stmt::VarDeclaration(
                    Token {
                        t_type: TokenType::Identifier(String::from("x")),
                        lexeme: String::new(),
                        line: 0,
                    },
                    Some(Expr::Literal {
                        token: Token {
                            t_type: TokenType::Number(5.0),
                            lexeme: String::new(),
                            line: 0,
                        },
                    })
                ),
                Stmt::Block(vec![
                    Stmt::Expr(Expr::Assign {
                        name: Token {
                            t_type: TokenType::Identifier(String::from("x")),
                            lexeme: String::new(),
                            line: 0,
                        },
                        value: Box::new(Expr::Literal {
                            token: Token {
                                t_type: TokenType::Number(10.0),
                                lexeme: String::new(),
                                line: 0,
                            },
                        }),
                    }),
                    Stmt::VarDeclaration(
                        Token {
                            t_type: TokenType::Identifier(String::from("y")),
                            lexeme: String::new(),
                            line: 0,
                        },
                        Some(Expr::Literal {
                            token: Token {
                                t_type: TokenType::Number(10.0),
                                lexeme: String::new(),
                                line: 0,
                            },
                        })
                    ),
                    Stmt::Print(Expr::Variable {
                        name: Token {
                            t_type: TokenType::Identifier(String::from("x")),
                            lexeme: String::new(),
                            line: 0,
                        },
                    }),
                ]),
            ]
        );
    }
}
