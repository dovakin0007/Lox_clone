use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::error::Error;
use crate::error::Error::RunTime;
use crate::interpreter::Types;
use crate::token;
use crate::token::{Token, TokenType};


type Env = Rc<RefCell<HashMap<String, Option<Types>>>>;

#[derive(Clone, Debug)]
pub struct Environment{
    enclosing: Option<Box<Environment>>,
    values: Rc<RefCell<HashMap<String, Option<Types>>>>
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

    pub fn define(&self, name: String, value: Option<Types>){
        self.values.borrow_mut().insert(name, value);
    }

    pub fn get(&mut self, name: String) -> Result<Option<Types>, Error> {
        if self.values.borrow().contains_key(&name.clone()) == true{
            Ok(self.values.borrow_mut().get_mut(&name).cloned().unwrap())
        }else {

            if let Some(ref mut enclosing) = self.enclosing {
                enclosing.get(name.clone())
            }
            else {
                Err(Error::RunTime {
                    token: Token {
                        t_type: TokenType::Identifier(name.clone()),
                        lexeme: name,
                        line: 0,
                    },
                    message: "cannot get the value or hasn't been assigned yet".to_string(),
                })
            }

        }
    }

    pub fn assign(&mut self, name: &Token, value: &Types) -> Result<(), Error> {
        let ident_name = match name.clone().t_type {
            TokenType::Identifier(s) => Some(s),
            _ => None
        };
        if self.values.borrow().contains_key(&ident_name.clone().unwrap())  == true {
            self.values.borrow_mut().insert(ident_name.clone().unwrap(), Option::from(value.clone()));
            Ok(())
        }else {

            if let Some(ref mut enclosing)= self.enclosing{
                enclosing.assign(name, value)?
            }
            Err(Error::RunTime {
                token: name.clone(),
                message: "cannot get the value or hasn't been assigned yet".to_string(),
            })

        }
    }
}