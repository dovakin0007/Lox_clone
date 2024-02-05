pub mod token;
mod scanner;

use std::{env, process, fs};
use std::io::stdin;
use std::process::exit;
use proc_macro2;

// #[derive(Debug)]

static mut HAD_ERROR: bool = false;
fn main() {

    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    // awsedr

    if args.len() >4{
        println!("Usage: rlox [script]");

        process::exit(1)
    }else if args.len() ==3 {
        let path = args.get(2);
        println!("{:?}", path);
        match path {
            Some(x) => {
                run_file(x).unwrap_or(
                    process::exit(1)
                );
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
        let res = match buffer.trim_end() {
            "" => {
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
            let current_lines = token_string.split("\n").collect::<Vec<&str>>();
            for (index, token_string ) in current_lines.iter().enumerate() {
                println!("Line No: {}", index);
                run(token_string);
                unsafe {HAD_ERROR= false}
            }
            Ok(())
        }
        Err(e) => if e.kind() == std::io::ErrorKind::Interrupted{
            panic!("Unable to Open the file at given path: {:?}", e);
        }else{
            panic!(" {:?}", e);
        }
    }
}

pub fn run(token_stream :&str){
   let stream: proc_macro2::TokenStream = token_stream.parse().unwrap();
    let vec_token_stream = stream.into_iter().collect::<Vec<_>>();
    for t in vec_token_stream{
        println!("{:?}", t);
        unsafe {
            if HAD_ERROR {
                exit(65);
            }
        }

    }
}

pub fn error (line: u32, message: &str){
    report(line, "", message)
}

pub fn report(line: u32, where_line: &str, message: &str){
    eprintln!("[line {:?} ] Error {:?} :  {:?}", line, where_line, message);
    unsafe { HAD_ERROR = true; }
}