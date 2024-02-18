
use crate::report;
use crate::token::TokenType;
use crate::token::Token;

//TODO ADD BETTER ERROR HANDLING
//Takes in bytes as input ands tokenizes it

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

    //return a vector of tokens
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut vec_tokens: Vec<Token> = vec![];

        while !self.at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(Some(token)) => vec_tokens.push(token),
                Ok(None) => {},
                Err(()) => {}
            };
        }
        vec_tokens.push(Token {
            t_type: TokenType::EOF,
            lexeme: String::new(),
            line: self.line as u32
        }
        );

        return vec_tokens;
    }

    //checks if the character matches if it does gives its own token Type else Throws and error
    fn scan_token(&mut self) -> Result<Option<Token>, ()> {
        let c = self.advance();
        match c {
            b'(' => Ok(Some(self.add_token(TokenType::LeftParen))),
            b')' => Ok(Some(self.add_token(TokenType::RightParen))),
            b'{' => Ok(Some(self.add_token(TokenType::LeftBrace))),
            b'}' => Ok(Some(self.add_token(TokenType::RightBrace))),
            b',' => Ok(Some(self.add_token(TokenType::Comma))),
            b'.' => Ok(Some(self.add_token(TokenType::Dot))),
            b'-' => Ok(Some(self.add_token(TokenType::Minus))),
            b'+' => Ok(Some(self.add_token(TokenType::Plus))),
            b';' => Ok(Some(self.add_token(TokenType::SemiColon))),
            b'*' => Ok(Some(self.add_token(TokenType::Star))),
            b':' => Ok(Some(self.add_token(TokenType::Colon))),
            b'!' => {
                Ok(if self.match_by(b'=') {
                    Some(self.add_token(TokenType::BangEqual))
                } else {
                    Some(self.add_token(TokenType::Bang))
                })
            },
            b'=' => {
                 Ok(if self.match_by(b'='){
                    Some(self.add_token(TokenType::EqualEqual))
                }else {
                     Some(self.add_token(TokenType::Equal))
                 })
            }
            b'<' => {
                Ok(if self.match_by(b'='){
                    Some(self.add_token(TokenType::LessEqual))
                }else {
                    Some(self.add_token(TokenType::Less))
                })
            }
            b'>' => {
                Ok(if self.match_by(b'='){
                    Some(self.add_token(TokenType::GreaterEqual))
                }else{
                   Some( self.add_token(TokenType::Greater))
                })
            }
            b'/' => {
                Ok(if self.match_by(b'/'){
                    while self.peek() != b'\n' && self.peek() != b'\0'{
                        self.advance();
                    }
                    None
                }else {
                    Some(self.add_token(TokenType::Slash))
                })
            }

            b' ' | b'\t' | b'\r' => {
                Ok(None)
            }

            b'\n' => {
                self.line += 1;
                Ok(None)
            }

            b'"' => {
                self.string()
            }

            c => {
                let c = if self.is_digit(c) {
                    self.number()
                }
                else if self.is_alpha(c){

                    let ident = self.identifier();
                    match ident.as_str() {
                        "and" => Ok(Some(self.add_token(TokenType::And))),
                        "break" => Ok(Some(self.add_token(TokenType::Break))),
                        "class" => Ok(Some(self.add_token(TokenType::Class))),
                        "else" => Ok(Some(self.add_token(TokenType::Else))),
                        "false" => Ok(Some(self.add_token(TokenType::False))),
                        "fun" => Ok(Some(self.add_token(TokenType::Fun))),
                        "for" => Ok(Some(self.add_token(TokenType::For))),
                        "if" => Ok(Some(self.add_token(TokenType::If))),
                        "nil" => Ok(Some(self.add_token(TokenType::Nil))),
                        "or" => Ok(Some(self.add_token(TokenType::Or))),
                        "print" => Ok(Some(self.add_token(TokenType::Print))),
                        "return" => Ok(Some(self.add_token(TokenType::Return))),
                        "super" => Ok(Some(self.add_token(TokenType::Super))),
                        "this" => Ok(Some(self.add_token(TokenType::This))),
                        "true" => Ok(Some(self.add_token(TokenType::True))),
                        "var" => Ok(Some(self.add_token(TokenType::Var))),
                        "while" => Ok(Some(self.add_token(TokenType::While))),
                         _ => Ok(Some(self.add_token(TokenType::Identifier(ident))))
                    }

                }else { let _ = report(self.line as u32, "at the", "Unexpected char");  Err(()) };
                return c;
            }
        }
    }
    //Creates an token
    fn add_token(&mut self, t_type: TokenType) -> Token {
        return Token {
            t_type,
            lexeme: self.sub_string(self.start, self.current).unwrap(),
            line: self.line as u32,
        }
    }

    // gets the current positions token value or char
    fn peek(&mut self) -> u8 {
        *self.source.get(self.current + 0).unwrap_or(&b'\0')
    }

    //returns the next position token value or char
    fn peek_next(&mut self) -> u8 {
        *self.source.get(self.current + 1).unwrap_or(&b'\0')
    }


    fn is_alpha(&self, c: u8) -> bool {
        return c.is_ascii_alphabetic() || c == b'_';
    }

    //checks whether its character or EOF
    fn match_by(&mut self, _: u8) -> bool {
        if self.at_end() == true {
            return false;
        }
        if self.peek() == b'\0' {
            return false;
        }
        self.current += 1;
        true
    }

    //moves one index returns previous index value
    fn advance(&mut self) -> u8 {
        let c = self.source[self.current];
        self.current += 1;
        return c
    }

    //checks whether the length or array is greater than index
    fn at_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    //returns a substring or returns an error
    fn sub_string(&self, start: usize, current: usize) -> Result<String, ()> {
        return String::from_utf8(self.source[start..current].to_vec()).map_err(|_| {
            report(self.line as u32, "at the ","Invalid Character");
        })
    }

    // returns whether its a string literal idk whats that called or throws an error
    fn string(&mut self) -> Result<Option<Token>, ()> {
        while self.peek() != b'"'  && !self.at_end(){
            if self.peek()==b'\n'{
                self.line += 1
            }
            self.advance();

        }
        if self.at_end(){
            report(self.line as u32, "at the", "Unterminated String");
            ()
        }
        self.advance();
        let string = self.sub_string(self.start+1, self.current-1)?;
        println!("{:?}", self.add_token(TokenType::String(string.clone())));
        Ok(Some(self.add_token(TokenType::String(string))))
    }
    //returns whether its a digit or not
    fn is_digit(&self, c:u8 ) -> bool {
        return c.is_ascii_digit()
    }
    //return an number literal
    fn number(&mut self) ->Result<Option<Token>, ()>{
        while self.peek().is_ascii_digit(){
            self.advance();
        }
        if self.peek() == b'.' && self.peek_next().is_ascii_digit(){
            self.advance();
            while self.peek().is_ascii_digit(){
                self.advance();
            }
        }

        let num: f64 = self.sub_string(self.start, self.current)?.parse::<f64>().expect("Weird, I'm super sure this ought to be a valid f64");

        Ok(Some(self.add_token(TokenType::Number(num))))
    }
    //return an identifier from reading a text file
    fn identifier(&mut self) -> String {
        while self.peek().is_ascii_alphanumeric()  {
            self.advance();
        }
        let ident: String = self.sub_string(self.start, self.current).expect("invalid character").parse::<String>().expect("Unable to parse the identifier or Invalid identifier");
        return ident;

    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::Scanner;
    use String;

    #[test]
    fn check_match_by(){

        let char_array = String::from("12.12 \n () hi \n while for");

        let binding = char_array.into_bytes();
        let mut sc = Scanner::new(&binding);
        let x = sc.scan_tokens();

        println!("{:?}", x);
        assert_eq!(true, true);
    }
}