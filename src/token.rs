use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hasher, Hash};

    #[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-Char tokens
    LeftParen, RightParen, LeftBrace,
    RightBrace, Comma,Colon,
    Dot, Minus, Plus, SemiColon, Slash, Star,

    //One or Two Tokens
    Bang, BangEqual,Equal,
    EqualEqual, GreaterEqual, Greater, LessEqual, Less,

    //Literals
    Identifier(String), String(String), Number(f64),

    //Keywords
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print,Return, Super, This, True, Var, While, Break,
    //Comment

    DoubleSlash,
    //End Of File
    EOF,

}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self{
            TokenType::LeftParen => f.write_str("("),
            TokenType::RightParen => f.write_str(")"),
            TokenType::LeftBrace => f.write_str("{"),
            TokenType::RightBrace => f.write_str("}"),
            TokenType::Colon => f.write_str(":"),
            TokenType::Comma => f.write_str(","),
            TokenType::Dot => f.write_str("."),
            TokenType::Minus => f.write_str("-"),
            TokenType::Plus => f.write_str("+"),
            TokenType::SemiColon => f.write_str(";"),
            TokenType::Slash => f.write_str("/"),
            TokenType::Star => f.write_str("*"),
            TokenType::Bang => f.write_str("!"),
            TokenType::BangEqual => f.write_str("!="),
            TokenType::Equal => f.write_str("="),
            TokenType::EqualEqual => f.write_str("=="),
            TokenType::Greater => f.write_str(">"),
            TokenType::GreaterEqual => f.write_str(">="),
            TokenType::Less => f.write_str("<"),
            TokenType::LessEqual => f.write_str("<="),
            TokenType::Identifier(s) => f.write_str(s),
            TokenType::String(s) => s.fmt(f),
            TokenType::Number(n) => n.fmt(f),
            TokenType::And => f.write_str("and"),
            TokenType::Break => f.write_str("break"),
            TokenType::Class => f.write_str("class"),
            TokenType::Else => f.write_str("else"),
            TokenType::False => f.write_str("false"),
            TokenType::Fun => f.write_str("fun"),
            TokenType::For => f.write_str("for"),
            TokenType::If => f.write_str("if"),
            TokenType::Nil => f.write_str("nil"),
            TokenType::Or => f.write_str("or"),
            TokenType::Print => f.write_str("print"),
            TokenType::Return => f.write_str("return"),
            TokenType::Super => f.write_str("super"),
            TokenType::This => f.write_str("this"),
            TokenType::True => f.write_str("true"),
            TokenType::Var => f.write_str("var"),
            TokenType::While => f.write_str("while"),
            TokenType::DoubleSlash=> f.write_str(""),
            TokenType::EOF => f.write_str("\\d"),
        }
    }
}

//Token struct to present a token
//lexeme might be used for identifier I guess
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub t_type: TokenType,
    pub lexeme: String,
    pub line: u32,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.t_type.fmt(f)
    }
}

impl  Hash for Token {
    fn hash<H: Hasher>(&self, state:&mut H){
        self.lexeme.hash(state);
        self.line.hash(state);
    }
}

impl Eq for Token{}

