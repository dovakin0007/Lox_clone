use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::Types;

pub struct Environment{
    values: Rc<RefCell<HashMap<String, Option<Types>>>>
}

impl Environment{
    pub fn new() -> Self {
        Self{
            values: Rc::new(RefCell::new(HashMap::new()))
        }
    }

    pub fn define(&self, name: String, value: Option<Types>){
        self.values.borrow_mut().insert(name, value);
    }

    pub fn get(&mut self, name: String) -> Result<Option<Types>, String> {
        if self.values.borrow().contains_key(&name.clone()) == true{
            Ok(self.values.borrow_mut().get_mut(&name).cloned().unwrap())
        }else {
            Err(format!(" Undefined variable {name}"))
        }
    }
}