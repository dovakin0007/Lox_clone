use crate::ast::Expr::Literal;
use crate::ast::Stmt::IfStmt;
use crate::ast::{Expr, Stmt};
use crate::error::{parse_error, Error};
use crate::interpreter::Types;
use crate::token::TokenType::{False, LeftParen, Return, RightParen, SemiColon, True};
use crate::token::{Token, TokenType};
use log::debug;
use std::io::ErrorKind::Other;

//TODO match macro

macro_rules! matches {
    ($sel: ident, $( $x:expr),*) => {
        {
            if $( $sel.check($x) )||* {
                $sel.advance();
                true
            } else {
                false
            }
        }
    };
}
//Parser takes an input of tokens
#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    //returns an expression tree from the Vector
    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements: Vec<Stmt> = Vec::new();
        while self.peek().unwrap().t_type != TokenType::EOF {
            statements.push(self.declaration()?)
        }
        return Ok(statements);
    }

    fn declaration(&mut self) -> Result<Stmt, Error> {
        let statement = match self.peek().unwrap().t_type {
            TokenType::Var => self.var_declaration(),
            TokenType::Class => self.class_declaration(),
            TokenType::Fun => self.function_declaration("function"),
            _ => self.statement(),
        };
        match statement {
            Err(Error::Parse) => {
                self.synchronize();
                Ok(Stmt::Null)
            }
            other => other,
        }
    }

    fn function_declaration(&mut self, kind: &str) -> Result<Stmt, Error> {
        let name = self.consume_identifier(format!("Expect {} name.", kind).as_str())?;
        self.consume(
            TokenType::LeftParen,
            format!("Expect '(' after {} name.", kind).as_str(),
        )?;
        let mut params: Vec<Token> = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    let val = self.peek().unwrap();
                    self.error(val, "Cannot have more than 255 parameters.");
                }
                let current_token_type = self.peek().unwrap().clone().t_type;

                params.push(self.consume(current_token_type, "Expect parameter name.")?);
                if !matches!(self, TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(
            TokenType::LeftBrace,
            format!("Expect '{{' before {} body.", kind).as_str(),
        )?;
        let body = self.block_statement()?;
        Ok(Stmt::Function(name, params, body))
    }

    fn class_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.consume_identifier("Expect class name.")?;

        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;
        let mut methods: Vec<Stmt> = Vec::new();
        while !self.check(TokenType::RightBrace) {
            methods.push(self.function_declaration("method")?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Stmt::Class(name, methods))
    }

    fn var_declaration(&mut self) -> Result<Stmt, Error> {
        let token = self.consume_identifier("Expect variable name")?.clone();
        let initializer = if self.peek().unwrap().t_type.clone() == TokenType::Equal {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };

        let _ = self
            .consume(
                self.peek().unwrap().t_type.clone(),
                "Expected `;` after variable declaration",
            )
            .unwrap();
        Ok(Stmt::VarDeclaration(token, initializer))
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        return match self.peek().unwrap().t_type {
            TokenType::For => {
                self.advance();
                self.for_statement()
            }
            TokenType::If => {
                self.advance();
                self.if_statement()
            }

            TokenType::Print => {
                self.advance();
                self.print_statement()
            }
            TokenType::Return => {
                self.advance();
                self.return_statement()
            }
            TokenType::While => {
                self.advance();
                self.while_statement()
            }

            TokenType::LeftBrace => {
                self.advance();
                Ok(Stmt::Block(self.block_statement()?))
            }
            _ => self.expression_statement(),
        };
    }

    fn for_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(LeftParen, "Expect '(' after 'for'.")?;

        let initializer = match self.peek().unwrap().t_type {
            TokenType::Var => Some(self.var_declaration()?),
            TokenType::SemiColon => None,
            _ => Some(self.expression_statement()?),
        };
        let condition = if !self.check(TokenType::SemiColon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::SemiColon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            let inc_stmt = Stmt::Expr(inc);
            body = Stmt::Block(vec![body, inc_stmt])
        };

        body = Stmt::While(
            condition.unwrap_or(Literal {
                token: Token {
                    t_type: True,
                    lexeme: "".to_string(),
                    line: self.peek().unwrap().line.clone(),
                },
            }),
            Box::new(body),
        );

        if let Some(init_stmt) = initializer {
            body = Stmt::Block(vec![init_stmt, body])
        }
        Ok(body)
    }
    fn if_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(LeftParen, "Expect '(' after 'if'.")?;
        let expr = self.expression()?;
        self.consume(RightParen, "Expect ')' after if condition.")?;
        let then_stmt = self.statement()?;
        match self.advance().unwrap().t_type.clone() {
            TokenType::Else => {
                let else_branch = self.statement()?;
                return Ok(IfStmt(
                    expr,
                    Box::new(then_stmt),
                    Some(Box::new(else_branch)),
                ));
            }
            _ => return Ok(IfStmt(expr, Box::new(then_stmt), None)),
        };
    }

    fn while_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(LeftParen, "Expect '(' after 'while'.")?;
        let expr = self.expression()?;
        self.consume(RightParen, "Expect ')' after 'while'.")?;

        let body = self.statement()?;
        Ok(Stmt::While(expr, Box::new(body)))
    }

    fn return_statement(&mut self) -> Result<Stmt, Error> {
        let keyword = self.previous().unwrap().clone();
        let value = if !self.check(SemiColon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(SemiColon, "Expect ';' after return value.")?;
        Ok(Stmt::Return(keyword.clone(), value))
    }

    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Expr(expr))
    }

    fn block_statement(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements: Vec<Stmt> = Vec::new();
        while match self.peek().unwrap().t_type.clone() {
            TokenType::RightBrace | TokenType::EOF => false,
            _ => true,
        } {
            statements.push(self.declaration()?)
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")
            .unwrap();
        return Ok(statements);
    }
    //using recursive decent parsing method
    fn expression(&mut self) -> Result<Expr, Error> {
        return Ok(self.assignment()?);
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.or()?;

        if let Some(_) = match self.peek().unwrap().t_type {
            TokenType::Equal => self.advance(),
            _ => None,
        } {
            let value = self.assignment()?;
            match expr {
                Expr::Variable { name, .. } => {
                    return Ok(Expr::Assign {
                        name: name,
                        value: Box::new(value),
                    })
                }
                _ => panic!("unable to assign "),
            }
        }
        return Ok(expr);
    }
    fn or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.and()?;

        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Or => self.advance(),
            _ => None,
        } {
            let operator = t.clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right.clone()),
            };
        }
        return Ok(expr);
    }

    fn and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;
        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::And => self.advance(),
            _ => None,
        } {
            let op = t.clone();
            let right = self.equality()?;

            expr = Expr::Logical {
                left: Box::new(expr),
                op: op.clone(),
                right: Box::new(right.clone()),
            };
        }
        return Ok(expr);
    }

    //checks whether the expression is an equality expression returns an expression
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::EqualEqual | TokenType::BangEqual => self.advance(),
            _ => None,
        } {
            let operator: Token = t.clone();
            let right = self.comparison()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator.clone(),
                right: Box::new(right),
            };
        }
        return Ok(expr);
    }

    //checks whether the given expression is comparison operation returns an expression
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => self.advance(),
            _ => None,
        } {
            let operator: Token = t.clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator.clone(),
                right: Box::new(right),
            }
        }
        return Ok(expr);
    }

    //checks whether the expression is add or sub and returns an expression
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Plus | TokenType::Minus => self.advance(),
            _ => None,
        } {
            let operator: Token = t.clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator.clone(),
                right: Box::new(right),
            }
        }
        return Ok(expr);
    }

    //checks whether the expression is mul or div and returns an expression
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;
        //dbg!(&self.peek().unwrap());
        while let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Star | TokenType::Slash => self.advance(),
            _ => None,
        } {
            let operator = t.clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator.clone(),
                right: Box::new(right),
            }
        }
        return Ok(expr);
    }
    // returns an unary expression
    fn unary(&mut self) -> Result<Expr, Error> {
        //dbg!(&self.peek().unwrap());
        if let Some(t) = match self.peek().unwrap().t_type {
            TokenType::Bang | TokenType::Minus => self.advance(),
            _ => None,
        } {
            let operator = t.clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op: operator.clone(),
                expr: Box::new(right),
            });
        }
        return Ok(self.callee()?);
    }
    // returns a literal such as String, bool, Number and also grouping expression

    fn callee(&mut self) -> Result<Expr, Error> {
        let mut expr = self.primary()?;
        loop {
            if self.peek().unwrap().t_type.clone() == TokenType::LeftParen {
                self.advance();
                expr = self.finish_call(expr)?
            } else {
                break;
            }
        }
        return Ok(expr);
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Error> {
        let mut args: Vec<Expr> = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                if args.len() >= 255 {
                    self.error(self.peek().unwrap(), "Cannot have more than 255 arguments.");
                }
                args.push(self.expression()?);
                if !matches!(self, TokenType::Comma) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments: args,
        })
    }
    fn primary(&mut self) -> Result<Expr, Error> {
        // dbg!(&self.peek().unwrap());
        let previous_token = self.peek().unwrap().clone();

        let expr = match previous_token.t_type {
            TokenType::False => Expr::Literal {
                token: previous_token.clone(),
            },
            TokenType::True => Expr::Literal {
                token: previous_token.clone(),
            },
            TokenType::Nil => Expr::Literal {
                token: previous_token.clone(),
            },
            TokenType::String(_) => Expr::Literal {
                token: previous_token.clone(),
            },
            TokenType::Number(_) => Expr::Literal {
                token: previous_token.clone(),
            },
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.")
                    .unwrap();
                Expr::Grouping {
                    expr: Box::new(expr),
                }
            }
            TokenType::Identifier(_) => Expr::Variable {
                name: previous_token.clone(),
            },

            _ => return Err(self.error(self.peek().unwrap(), "Expect expression.")),
        };
        self.advance();
        return Ok(expr);
    }

    //TODO
    fn error(&self, token: &Token, message: &str) -> Error {
        parse_error(token, message);
        Error::Parse
    }

    fn synchronize(&mut self) {
        self.advance();
        while self.peek().unwrap().t_type != TokenType::EOF {
            if self.previous().unwrap().t_type == TokenType::SemiColon {
                return;
            }

            match self.peek().unwrap().t_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => self.advance(),
            };
        }
    }

    //return the current value or current token based on the index
    fn peek(&self) -> Option<&Token> {
        let index = self.current;
        if index >= self.tokens.len() {
            None
        } else {
            self.tokens.get(index)
        }
    }

    //advance the current to next and returns previous index value in the vector
    fn advance(&mut self) -> Option<&Token> {
        let x = self.peek().unwrap();
        if x.t_type != TokenType::EOF {
            self.current += 1
        }
        return self.tokens.get(self.current - 1);
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        if self.peek().unwrap().t_type == TokenType::EOF {
            return false;
        }
        token_type == self.peek().unwrap().t_type
    }

    //checks if right parenthesis exists or throws an error
    fn consume(&mut self, token_type: TokenType, error_msg: &str) -> Result<Token, Error> {
        if token_type == self.peek().unwrap().t_type {
            return Ok(self.advance().unwrap().clone());
        } else {
            Err(self.error(&self.peek().unwrap(), error_msg))
        }
    }

    fn consume_identifier(&mut self, error_msg: &str) -> Result<Token, Error> {
        self.advance().unwrap();
        let token_type = self.peek().unwrap().t_type.clone();
        match token_type {
            TokenType::Identifier(_) => self.consume(token_type, error_msg),
            _ => Err(self.error(&self.peek().unwrap(), error_msg)),
        }
    }
    //gets the previous element in the vector
    fn previous(&mut self) -> Option<&Token> {
        return self.tokens.get(self.current - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;

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
    fn test_if_statement() {
        // Tokens representing the if statement: if (x == 5) { print "true"; }
        let tokens = vec![
            Token {
                t_type: TokenType::If,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::LeftParen,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Identifier(String::from("x")),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::EqualEqual,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Number(5.0),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::RightParen,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::LeftBrace,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::Print,
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::String(String::from("true")),
                lexeme: String::new(),
                line: 0,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: String::new(),
                line: 0,
            },
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
        let statements = parser.parse();

        //println!("{:?}", &statements.unwrap());
        // Validate the output
        assert_eq!(
            statements.unwrap(),
            vec![Stmt::IfStmt(
                Expr::Binary {
                    left: Box::new(Expr::Variable {
                        name: Token {
                            t_type: TokenType::Identifier(String::from("x")),
                            lexeme: String::new(),
                            line: 0,
                        }
                    }),
                    op: Token {
                        t_type: TokenType::EqualEqual,
                        lexeme: String::new(),
                        line: 0,
                    },
                    right: Box::new(Expr::Literal {
                        token: Token {
                            t_type: TokenType::Number(5.0),
                            lexeme: String::new(),
                            line: 0,
                        }
                    })
                },
                Box::new(Stmt::Block(vec![Stmt::Print(Expr::Literal {
                    token: Token {
                        t_type: TokenType::String(String::from("true")),
                        lexeme: String::new(),
                        line: 0,
                    }
                })])),
                None,
            )]
        );
    }

    #[test]
    fn test_function_call() {
        // Define tokens representing a function call: myFunction(arg1, arg2);
        let tokens = vec![
            Token {
                t_type: TokenType::Identifier("myFunction".to_string()),
                lexeme: "myFunction".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::LeftParen,
                lexeme: "(".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Identifier("arg1".to_string()),
                lexeme: "arg1".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Comma,
                lexeme: ",".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Identifier("arg2".to_string()),
                lexeme: "arg2".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::RightParen,
                lexeme: ")".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: ";".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::EOF,
                lexeme: "".to_string(),
                line: 1,
            },
        ];

        // Create the parser and parse the tokens into an expression
        let mut parser = Parser::new(tokens);
        let expr = parser.expression().unwrap();

        // Verify that the expression is a function call with the correct callee and arguments
        match expr {
            Expr::Call {
                callee, arguments, ..
            } => {
                // Verify the callee
                match *callee {
                    Expr::Variable { name } => {
                        assert_eq!(name.lexeme, "myFunction");
                    }
                    _ => panic!("Unexpected callee type"),
                }

                // Verify the arguments
                assert_eq!(arguments.len(), 2);
                match &arguments[0] {
                    Expr::Variable { name } => {
                        assert_eq!(name.lexeme, "arg1");
                    }
                    _ => panic!("Unexpected argument type"),
                }
                match &arguments[1] {
                    Expr::Variable { name } => {
                        assert_eq!(name.lexeme, "arg2");
                    }
                    _ => panic!("Unexpected argument type"),
                }
            }
            _ => panic!("Unexpected expression type"),
        }
    }

    #[test]
    fn test_function_call_with_expression() {
        // Define tokens representing a function call with an expression as an argument: myFunction(2 * x, y + 3);
        let tokens = vec![
            Token {
                t_type: TokenType::Identifier("myFunction".to_string()),
                lexeme: "myFunction".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::LeftParen,
                lexeme: "(".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Number(2.0),
                lexeme: "2".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Star,
                lexeme: "*".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Identifier("x".to_string()),
                lexeme: "x".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Comma,
                lexeme: ",".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Identifier("y".to_string()),
                lexeme: "y".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Plus,
                lexeme: "+".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Number(3.0),
                lexeme: "3".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::RightParen,
                lexeme: ")".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::SemiColon,
                lexeme: ";".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::EOF,
                lexeme: "".to_string(),
                line: 1,
            },
        ];

        // Create the parser and parse the tokens into an expression
        let mut parser = Parser::new(tokens);
        let expr = parser.expression().unwrap();

        // Verify that the expression is a function call with the correct callee and arguments
        match expr {
            Expr::Call {
                callee, arguments, ..
            } => {
                // Verify the callee
                match *callee {
                    Expr::Variable { name } => {
                        assert_eq!(name.lexeme, "myFunction");
                    }
                    _ => panic!("Unexpected callee type"),
                }

                // Verify the arguments
                assert_eq!(arguments.len(), 2);
                match &arguments[0] {
                    Expr::Binary { left, right, op } => {
                        match *left.clone() {
                            Expr::Literal { token } => {
                                assert_eq!(token.t_type, TokenType::Number(2.0));
                            }
                            _ => panic!("Unexpected argument type"),
                        }
                        match *right.clone() {
                            Expr::Variable { name } => {
                                assert_eq!(name.lexeme, "x");
                            }
                            _ => panic!("Unexpected argument type"),
                        }
                        match op.t_type {
                            TokenType::Star => {}
                            _ => panic!("Unexpected operator"),
                        }
                    }
                    _ => panic!("Unexpected argument type"),
                }
                match &arguments[1] {
                    Expr::Binary { left, right, op } => {
                        match *left.clone() {
                            Expr::Variable { name } => {
                                assert_eq!(name.lexeme, "y");
                            }
                            _ => panic!("Unexpected argument type"),
                        }
                        match *right.clone() {
                            Expr::Literal { token } => {
                                assert_eq!(token.t_type, TokenType::Number(3.0));
                            }
                            _ => panic!("Unexpected argument type"),
                        }
                        match op.t_type {
                            TokenType::Plus => {}
                            _ => panic!("Unexpected operator"),
                        }
                    }
                    _ => panic!("Unexpected argument type"),
                }
            }
            _ => panic!("Unexpected expression type"),
        }
    }

    #[test]
    fn test_function_declaration() {
        // Define tokens representing a function declaration: fun myFunction(param1, param2) { /* function body */ }
        let tokens = vec![
            Token {
                t_type: TokenType::Fun,
                lexeme: "fun".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Identifier("myFunction".to_string()),
                lexeme: "myFunction".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::LeftParen,
                lexeme: "(".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Identifier("param1".to_string()),
                lexeme: "param1".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Comma,
                lexeme: ",".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::Identifier("param2".to_string()),
                lexeme: "param2".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::RightParen,
                lexeme: ")".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::LeftBrace,
                lexeme: "{".to_string(),
                line: 1,
            },
            // Omitted tokens representing the function body
            Token {
                t_type: TokenType::RightBrace,
                lexeme: "}".to_string(),
                line: 1,
            },
            Token {
                t_type: TokenType::EOF,
                lexeme: "".to_string(),
                line: 1,
            },
        ];

        // Create the parser and parse the tokens into a statement
        let mut parser = Parser::new(tokens);
        let statement = parser.function_declaration("function").unwrap();
        dbg!(statement.clone());

        // Verify the statement is a function declaration with the correct name, parameters, and body
        match statement {
            Stmt::Function(name, params, body) => {
                assert_eq!(name.lexeme, "myFunction");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].lexeme, "param1");
                assert_eq!(params[1].lexeme, "param2");

                // Omitted: Verify the body
            }
            _ => panic!("Unexpected statement type"),
        }
    }
}
