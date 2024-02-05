
use crate::error;
use crate::token::TokenType;
use crate::token::Token;
use crate::token::TokenType::{DoubleSlash, Slash};


pub struct Scanner<'a>{
     source: &'a [u8],
    start: usize,
    current: usize,
    line: usize,
}

impl <'a> Scanner<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut vec_tokens: Vec<Token> = vec![];
        while (!self.at_end()) {
            self.start = self.current;
            vec_tokens.push(self.scan_token().unwrap());
        }
        vec_tokens.push((Token {
            t_type: TokenType::EOF,
            lexeme: String::new(),
            line: self.line as u32
        }
        ));

        return vec_tokens;
    }

    fn scan_token(&mut self) -> Result<Token, ()> {
        let c = self.advance();
        match c {
            b'(' => Ok((self.add_token(TokenType::LeftParen))),
            b')' => Ok(self.add_token(TokenType::RightParen)),
            b'{' => Ok(self.add_token(TokenType::LeftBrace)),
            b'}' => Ok(self.add_token(TokenType::RightBrace)),
            b',' => Ok(self.add_token(TokenType::Comma)),
            b'.' => Ok(self.add_token(TokenType::Dot)),
            b'-' => Ok(self.add_token(TokenType::Minus)),
            b'+' => Ok(self.add_token(TokenType::Plus)),
            b';' => Ok(self.add_token(TokenType::SemiColon)),
            b'*' => Ok(self.add_token(TokenType::Star)),
            b':' => Ok(self.add_token(TokenType::Colon)),
            b'!' => {
                Ok(if self.match_by(b'=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                })
            },
            b'=' => {
                 Ok(if self.match_by(b'='){
                    self.add_token(TokenType::EqualEqual)
                }else {
                     self.add_token(TokenType::Equal)
                 })
            }
            b'<' => {
                Ok(if self.match_by(b'='){
                    self.add_token(TokenType::LessEqual)
                }else {
                    self.add_token(TokenType::Less)
                })
            }
            b'>' => {
                Ok(if self.match_by(b'='){
                    self.add_token(TokenType::GreaterEqual)
                }else{
                    self.add_token(TokenType::Greater)
                })
            }
            b'/' => {
                Ok(if self.match_by(b'/'){
                    while self.peek() != b'\n' && self.peek() != b'\0'{
                        self.advance()
                    }
                    self.add_token(DoubleSlash)
                }else {
                    self.add_token(Slash)
                })
            }

            _ => {
                let _ = error(self.line as u32, "Unexpected char");

                Err(())
            }
        }
    }

    fn add_token(&mut self, t_type: TokenType) -> Token {
        return Token {
            t_type,
            lexeme: self.sub_string(self.start, self.current).unwrap(),
            line: self.line as u32,
        }
    }

    fn peek(&mut self) -> u8 {
        *self.source.get(self.current + 0).unwrap_or(&b'\0')
    }

    fn peek_next(&mut self) -> u8 {
        *self.source.get(self.current + 1).unwrap_or(&b'\0')
    }

    fn match_by(&mut self, c: u8) -> bool {
        if self.at_end() == true {
            return false;
        }
        if self.peek() == b'\0' {
            return false;
        }
        self.current += 1;
        true
    }

    fn advance(&mut self) -> u8 {
        let c = self.source[self.current];
        self.current += 1;
        return c
    }

    fn at_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    fn sub_string(&self, start: usize, current: usize) -> Result<String, ()> {
        return String::from_utf8(self.source[start..current].to_vec()).map_err(|err| {
            error(self.line as u32, "Invalid Character");
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::Scanner;
    use String;

    #[test]
    fn check_match_by(){

        let char_array = String::from("!=");

        let binding = char_array.into_bytes();
        let mut sc = Scanner::new(&binding);
        let x = sc.scan_tokens();

        println!("{:?}", x);
        assert_eq!(true, true);
    }
}