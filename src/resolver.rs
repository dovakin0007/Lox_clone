use std::collections::HashMap;
use crate::ast::{Expr, Stmt, Visitor};
use crate::error::{parse_error, Error};
use crate::interpreter::Interpreter;
use crate::report;
use crate::token::Token;

#[derive(Debug)]
pub enum FunctionType {
     None, 
    Function,
}

#[derive(Debug)]
pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}



impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None
        }
    }

     fn resolve_stmt(&mut self, statement: &Stmt) {
        let _ = self.visit_statement(statement);
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        let _ = self.visit_expression(expr);
    }


    pub fn resolve_stmts(&mut self, stmts: &Vec<Stmt>) {
        for statement in stmts {
            self.resolve_stmt(statement);
        }
    }

    fn begin_scope(&mut self){
        self.scopes.push(HashMap::new())
    }
    fn end_scope(&mut self){
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token){

        match self.scopes.last_mut() {
            Some (ref mut scope) => {
                if scope.contains_key(&name.lexeme) {
                    parse_error(name,  "Already a variable with this name in this scope.")
                }
                scope.insert(name.lexeme.clone(), false);
            }
            None => ()
        }
    }

    fn define(&mut self, name: &Token) {
        match self.scopes.last_mut() {
            Some(ref mut scope) => {
                scope.insert(name.lexeme.clone(), true);

            }
            None => ()
        }
    }

    fn resolve_local(&mut self, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme){
                self.interpreter.resolve(name, i);
            }
        }
    }

    fn resolve_function(&mut self, params: &Vec<Token>, body:&Vec<Stmt>, function_tpe: FunctionType) {
        self.current_function = function_tpe;
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
    
        self.resolve_stmts(body);
        self.end_scope();
    }
}

impl<'a> Visitor for  Resolver<'_> {
    type E = Result<(), Error>;
    type S = Result<(), Error>;
    fn visit_expression(&mut self, e: &Expr) -> Self::E {
        match e {

            &Expr::Assign {ref name, ref value} => {
                self.resolve_expr(value);
                self.resolve_local(name);
                Ok(())
            }
            &Expr::Binary {ref left, ref op, ref right}=>{
                self.resolve_expr(left);
                self.resolve_expr(right);
                Ok(())
            }
            &Expr::Call {ref callee, ref paren, ref arguments}=>{
                self.resolve_expr(callee);
                for argument in arguments {
                    self.resolve_expr(argument);
                }
                Ok(())
            }
            &Expr::Grouping {
                ref expr
            }=> {
                self.resolve_expr(expr);
                Ok(())
            }
            &Expr::Literal {ref token} => {
                Ok(())
            }
            &Expr::Logical {ref left, ref op, ref right}=>{
                self.resolve_expr(left);
                self.resolve_expr(right);
                Ok(())
            }
            &Expr::Unary {ref op, ref expr} => {
                self.resolve_expr(expr);
                Ok(())
            }
            &Expr::Variable{
                ref name
            }=> {
                if let Some(scopes) = self.scopes.last_mut() {
                    if let Some(flag) = scopes.get(&name.lexeme){
                        if flag==&false {
                            report(name.clone().line, &format!(" at '{}'", name.lexeme), "Can't read local variable in its own initializer.")
                        }
                    }
                };
                self.resolve_local(name);
                Ok(())
            }
        }
    }

    fn visit_statement(&mut self, s: &Stmt) -> Self::S {
        match s {
            &Stmt::Block(ref stmt) => {
                self.begin_scope();
                
                self.resolve_stmts(stmt);
                self.end_scope();
                Ok(())
            }
            &Stmt::Expr(ref expr) => {
                self.resolve_expr(expr);
                Ok(())
            }

            &Stmt::Function(ref name, ref params, ref body) => {
                self.declare(name);
                self.define(name);
                self.resolve_function(params, body, FunctionType::Function);
                Ok(())
            }
            &Stmt::IfStmt(ref condition, ref then, ref else_stmt)=> {
                self.resolve_expr(condition);
                self.resolve_stmt(then);
                if let Some(else_stmt) = else_stmt {
                    self.resolve_stmt(else_stmt);
                }
                Ok(())
            }
            &Stmt::Print(ref expr)=> {
                self.resolve_expr(expr);
                Ok(())
            }
            &Stmt::Return(ref keyword,ref value)=> {
                if let FunctionType::None = &self.current_function {
                    parse_error(keyword,"Cannot return from top-level code.")
                }
                if let Some(keyword_value) = value{
                    self.resolve_expr(keyword_value);
                };
                Ok(())
            }
            &Stmt::VarDeclaration(ref name, ref initializer) => {
                self.declare(name);
                if let Some(init) = initializer {
                    self.resolve_expr(init)
                }
                Ok(self.define(name))

            }
            &Stmt::While(ref conditon, ref block) => {
                self.resolve_expr(conditon);
                self.resolve_stmt(block);
                Ok(())
            } &Stmt::Null=> {
                Ok(())
            }
        }
    }
}
