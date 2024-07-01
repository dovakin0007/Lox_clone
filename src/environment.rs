use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::E;
use std::rc::Rc;
use std::string;
use crate::error::Error;
use crate::error::Error::RunTime;
use crate::interpreter::Types;
use crate::token::{Token, TokenType};



#[derive(Clone, Debug)]
pub struct Environment{
    pub enclosing: Option<Box<Environment>>,
    values: Rc<RefCell<HashMap<std::string::String, Types>>>
}

impl Environment{
    pub fn new() -> Self {
        Self{
            enclosing: None,
            values: Rc::new(RefCell::new(HashMap::new()))
        }
    }

    pub fn from(enclosing: Environment) -> Self {
        Self {
            enclosing: Some(Box::new(enclosing)),
            values: Rc::new(RefCell::new(HashMap::new()))
        }
    }

    pub fn define(&self, name: std::string::String, value: Option<Types>){
        self.values.borrow_mut().insert(name, value.unwrap());
    }

    fn ancestor(&self, distance: &usize) -> Box<Environment> {
        let parent = self.enclosing.clone().expect(&format!("No enclosing environment at {}", 1));
        let mut env = Box::clone(&parent);

        for i in 1..*distance {
            let parent = self.enclosing.clone().expect(&format!("No enclosing environment at {}", i));
            env = Box::clone(&parent);
        }
        env
    }

    pub fn get_at(&self, distance: &usize, name: &Token) -> Result<Types, Error> {
        let key = &*name.lexeme;
        let key_str = string::String::from(key);
        if distance > &0 {
        Ok(self.ancestor(distance).values.borrow().get(&key_str).expect(&format!("Undefined variable '{}'", key)).clone())
        }else{
         Ok(self.values.borrow().get(&key_str).expect(&format!("Undefined variable '{}'", key)).clone())
        }
    }

    pub fn get(&mut self, name: &Token) -> Result<Types, Error> {
        let name_lexeme = name.lexeme.clone();
        if self.values.borrow().contains_key(&name_lexeme) == true{
            // println!("{:?}",self.values.clone().borrow().get(&name).cloned().unwrap());
            Ok(self.values.borrow_mut().get(&name_lexeme).unwrap().clone())
        }else {
            if let Some(ref mut enclosing) = self.enclosing {
               enclosing.get(name)
            }
            else {
                Err(RunTime {
                    token: Token {
                        t_type: TokenType::Identifier(name_lexeme.clone()),
                        lexeme: name_lexeme,
                        line: 0,
                    },
                    message: "cannot get the value or hasn't been assigned yet".to_string(),
                })
            }

        }
    }

    pub fn assign_at(&mut self, name: &Token, value: &Types, distance: &usize) -> Result <(), Error> {
        let key = &*name.lexeme;
        let key_str = string::String::from(key);
        if distance > &0{
            self.ancestor(distance).values.borrow_mut().insert(key_str, value.clone());
        }else {
            self.values.borrow_mut().insert(key_str, value.clone());
        }
        Ok(())
    }

    pub fn assign(&mut self, name: &Token, value: &Types) -> Result<(), Error> {
        let ident_name = name.lexeme.clone();
        if self.values.borrow().contains_key(&ident_name)  == true {
            self.values.borrow_mut().insert(ident_name.clone(), value.clone());
            Ok(())
        }else {
            if let Some(ref mut enclosing)= self.enclosing{
                enclosing.assign(name, value)
            }else {
                Err(RunTime {
                    token: name.clone(),
                    message: "cannot get the value or hasn't been assigned yet".to_string(),
                })
            }
        }
    }
}