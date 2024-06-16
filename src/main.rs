pub mod token;
mod scanner;
mod ast;
mod astprinter;
mod parser;
mod interpreter;
mod environment;
mod error;
mod function;

use std::{env, process, fs};
use std::io::stdin;
use std::process::exit;
use log::debug;
use crate::ast::{Expr, Stmt};
use crate::parser::Parser;
use crate::token::{Token, TokenType};

static mut HAD_ERROR: bool = false;
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() >4{
        println!("Usage: rlox [script]");

        process::exit(1)
    }else if args.len() ==3 {
        let path = args.get(2);
        println!("{:?}", path);
        match path {
            Some(x) => {
                run_file(x).unwrap_or_else(|_| exit(1));
            },
            None => {
                println!("Usage: rlox [script]");
                process::exit(1)
            }

        }
    }else if args.len()==1{
        run_prompt().expect("UNABLE TO READ LINE")
    }


}


pub fn run_prompt() -> Result<(), Box<dyn std::error::Error + 'static>>{
    let mut buffer = String::new();

    loop {
        print!("> ");
        stdin().read_line(&mut buffer)?;
        let res = match buffer.trim_end().as_bytes() {
            b"" => {
                exit(1);
            },
            line => line,

        };
        run(res);
        buffer.clear();
    }

}

pub fn run_file(my_str: &str) -> Result<(), Box<dyn std::error::Error + 'static>>{
    let bytes=fs::read_to_string(my_str);
    match bytes {
        Ok(token_string) => {
            let current_lines = token_string.into_bytes();
            // for (index, token_string ) in current_lines.iter().enumerate() {
            //     println!("Line No: {}", index);
            //     // run(token_string);
            //     println!("{:?}", current_lines);
            //     unsafe {HAD_ERROR= false}
            // }
            // println!("{:?}", current_lines);
            run(&current_lines);
            Ok(())
        }
        Err(e) => if e.kind() == std::io::ErrorKind::Interrupted{
            panic!("Unable to Open the file at given path: {:?}", e);
        }else{
            panic!(" {:?}", e);
        }
    }
}

pub fn run(token_stream : &[u8]){

    let mut scanner = scanner::Scanner::new(&*token_stream);
    let tokens= scanner.scan_tokens();

    let mut  parser:Parser= Parser::new(tokens);
    let expressions:Vec<Stmt> = parser.parse().unwrap();
    let mut interpreter = interpreter::Interpreter::new();
    interpreter.interpret(expressions);

}

pub fn error (token: Token, message: &str){
        if token.t_type == TokenType::EOF {
            report(token.line, "at end", message)
        }else{
            report(token.line, &*token.lexeme, message);
        }
}

pub fn report(line: u32, where_line: &str, message: &str){
    eprintln!("[line {:?} ] Error {:?} :  {:?}", line, where_line, message);
    unsafe { HAD_ERROR = true; }
}