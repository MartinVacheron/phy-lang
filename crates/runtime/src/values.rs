use colored::*;
use ecow::EcoString;
use frontend::ast::stmt::{FnDeclStmt, Stmt, StructStmt};
use std::{
    cell::RefCell,
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    fmt::Display,
    rc::Rc,
};
use thiserror::Error;
use tools::results::{PhyReport, PhyResult};

use crate::{
    callable::{CallErr, Callable},
    environment::Env,
    interpreter::{InterpErr, Interpreter},
    native_functions::PhyNativeFn,
};

// -----------------
//  Error managment
// -----------------
#[derive(Debug, Error)]
pub enum RtValErr {
    // Negation
    #[error("can't negate a value that isn't either of type: int, real or bool")]
    UnNegatable,

    // Types operations
    #[error("operator '{0}' is not supported for operations on {1} type")]
    UnsupportedOpOnType(String, String),

    #[error("can't use this operator for operations on string and int types")]
    OpStrInt,

    #[error("operator '{0}' is not supported for string manipulation")]
    StringManip(String),

    // Instance
    #[error("structure dosen't have field '{0}'")]
    MissingFieldInStruct(EcoString),

    // Others
    #[error("can't use a null value in a binary operation")]
    OperationOnNull,

    #[error("operation not supported")]
    UnknownOperation,
}

impl PhyReport for RtValErr {
    fn get_err_msg(&self) -> String {
        format!("{}: {}", "Function error".red(), self)
    }
}

// ----------------
//  Runtime Values
// ----------------
#[derive(Debug, PartialEq, Clone)]
pub enum RtVal {
    IntVal(Rc<RefCell<Int>>),
    RealVal(Rc<RefCell<Real>>),
    StrVal(Rc<RefCell<Str>>),
    BoolVal(Rc<RefCell<Bool>>),
    FuncVal(Function),
    NativeFnVal(PhyNativeFn),
    StructVal(Rc<RefCell<Struct>>),
    InstanceVal(Rc<RefCell<Instance>>),
    Null,
}

trait Negate {
    fn negate(&mut self);
}

trait Operate<Rhs> {
    fn operate(&self, rhs: &Rhs, operator: &str) -> Result<RtVal, RtValErr>;
}

impl RtVal {
    pub fn new_null() -> Self {
        RtVal::Null
    }

    pub fn negate(&self) -> Result<(), RtValErr> {
        match &self {
            RtVal::IntVal(i) => i.borrow_mut().negate(),
            RtVal::RealVal(r) => r.borrow_mut().negate(),
            RtVal::BoolVal(b) => b.borrow_mut().negate(),
            _ => return Err(RtValErr::UnNegatable),
        }

        Ok(())
    }

    // TODO: Error handling for other operation
    pub fn operate(&self, rhs: &RtVal, operator: &str) -> Result<RtVal, RtValErr> {
        match (&self, &rhs) {
            (RtVal::IntVal(i1), RtVal::IntVal(i2)) => i1.borrow().operate(&*i2.borrow(), operator),
            (RtVal::RealVal(r1), RtVal::RealVal(r2)) => {
                r1.borrow().operate(&*r2.borrow(), operator)
            }
            (RtVal::IntVal(i1), RtVal::RealVal(r1)) => i1.borrow().operate(&*r1.borrow(), operator),
            (RtVal::RealVal(r1), RtVal::IntVal(i1)) => r1.borrow().operate(&*i1.borrow(), operator),
            (RtVal::StrVal(s1), RtVal::StrVal(s2)) => s1.borrow().operate(&*s2.borrow(), operator),
            (RtVal::StrVal(s1), RtVal::IntVal(i1)) => s1.borrow().operate(&*i1.borrow(), operator),
            (RtVal::IntVal(i1), RtVal::StrVal(s1)) => i1.borrow().operate(&*s1.borrow(), operator),
            (RtVal::BoolVal(b1), RtVal::BoolVal(b2)) => {
                b1.borrow().operate(&*b2.borrow(), operator)
            }
            (RtVal::Null, _) | (_, RtVal::Null) => Err(RtValErr::OperationOnNull),
            _ => Err(RtValErr::UnknownOperation),
        }
    }
}

// -------
//   Int
// -------
#[derive(Debug, PartialEq)]
pub struct Int {
    pub value: i64,
}

impl Negate for Int {
    fn negate(&mut self) {
        self.value *= -1;
    }
}

impl Operate<Int> for Int {
    fn operate(&self, rhs: &Int, operator: &str) -> Result<RtVal, RtValErr> {
        match operator {
            "+" => Ok((self.value + rhs.value).into()),
            "-" => Ok((self.value - rhs.value).into()),
            "*" => Ok((self.value * rhs.value).into()),
            "/" => Ok((self.value / rhs.value).into()),
            "%" => Ok((self.value % rhs.value).into()),
            "<" => Ok((self.value < rhs.value).into()),
            ">" => Ok((self.value > rhs.value).into()),
            "<=" => Ok((self.value <= rhs.value).into()),
            ">=" => Ok((self.value >= rhs.value).into()),
            "==" => Ok((self.value == rhs.value).into()),
            "!=" => Ok((self.value != rhs.value).into()),
            op => Err(RtValErr::UnsupportedOpOnType(op.to_string(), "int".into())),
        }
    }
}

impl Operate<Real> for Int {
    fn operate(&self, rhs: &Real, operator: &str) -> Result<RtVal, RtValErr> {
        match operator {
            "+" => Ok((self.value as f64 + rhs.value).into()),
            "-" => Ok((self.value as f64 - rhs.value).into()),
            "*" => Ok((self.value as f64 * rhs.value).into()),
            "/" => Ok((self.value as f64 / rhs.value).into()),
            "%" => Ok((self.value as f64 % rhs.value).into()),
            "<" => Ok(((self.value as f64) < rhs.value).into()),
            ">" => Ok((self.value as f64 > rhs.value).into()),
            "<=" => Ok((self.value as f64 <= rhs.value).into()),
            ">=" => Ok((self.value as f64 >= rhs.value).into()),
            "==" => Ok((self.value as f64 == rhs.value).into()),
            "!=" => Ok((self.value as f64 != rhs.value).into()),
            op => Err(RtValErr::UnsupportedOpOnType(op.to_string(), "int".into())),
        }
    }
}

impl Operate<Str> for Int {
    fn operate(&self, rhs: &Str, operator: &str) -> Result<RtVal, RtValErr> {
        match operator {
            "*" => Ok(rhs.value.repeat(self.value as usize).into()),
            _ => Err(RtValErr::OpStrInt),
        }
    }
}

// --------
//   Real
// --------
#[derive(Debug, PartialEq)]
pub struct Real {
    pub value: f64,
}

impl Negate for Real {
    fn negate(&mut self) {
        self.value *= -1.;
    }
}

impl Operate<Int> for Real {
    fn operate(&self, rhs: &Int, operator: &str) -> Result<RtVal, RtValErr> {
        match operator {
            "+" => Ok((self.value + rhs.value as f64).into()),
            "-" => Ok((self.value - rhs.value as f64).into()),
            "*" => Ok((self.value * rhs.value as f64).into()),
            "/" => Ok((self.value / rhs.value as f64).into()),
            "%" => Ok((self.value % rhs.value as f64).into()),
            "<" => Ok((self.value < rhs.value as f64).into()),
            ">" => Ok((self.value > rhs.value as f64).into()),
            "<=" => Ok((self.value <= rhs.value as f64).into()),
            ">=" => Ok((self.value >= rhs.value as f64).into()),
            "==" => Ok((self.value == rhs.value as f64).into()),
            "!=" => Ok((self.value != rhs.value as f64).into()),
            op => Err(RtValErr::UnsupportedOpOnType(op.to_string(), "real".into())),
        }
    }
}

impl Operate<Real> for Real {
    fn operate(&self, rhs: &Real, operator: &str) -> Result<RtVal, RtValErr> {
        match operator {
            "+" => Ok((self.value + rhs.value).into()),
            "-" => Ok((self.value - rhs.value).into()),
            "*" => Ok((self.value * rhs.value).into()),
            "/" => Ok((self.value / rhs.value).into()),
            "%" => Ok((self.value % rhs.value).into()),
            "<" => Ok((self.value < rhs.value).into()),
            ">" => Ok((self.value > rhs.value).into()),
            "<=" => Ok((self.value <= rhs.value).into()),
            ">=" => Ok((self.value >= rhs.value).into()),
            "==" => Ok((self.value == rhs.value).into()),
            "!=" => Ok((self.value != rhs.value).into()),
            op => Err(RtValErr::UnsupportedOpOnType(op.to_string(), "real".into())),
        }
    }
}

// ----------
//   String
// ----------
#[derive(Debug, PartialEq)]
pub struct Str {
    pub value: EcoString,
}

impl Operate<Str> for Str {
    fn operate(&self, rhs: &Str, operator: &str) -> Result<RtVal, RtValErr> {
        match operator {
            "+" => Ok(EcoString::from(format!("{}{}", self.value, rhs.value)).into()),
            "==" => Ok((self.value == rhs.value).into()),
            "!=" => Ok((self.value != rhs.value).into()),
            op => Err(RtValErr::StringManip(op.to_string())),
        }
    }
}

impl Operate<Int> for Str {
    fn operate(&self, rhs: &Int, operator: &str) -> Result<RtVal, RtValErr> {
        match operator {
            "*" => Ok(self.value.repeat(rhs.value as usize).into()),
            _ => Err(RtValErr::OpStrInt),
        }
    }
}

// --------
//   Bool
// --------
#[derive(Debug, PartialEq)]
pub struct Bool {
    pub value: bool,
}

impl Negate for Bool {
    fn negate(&mut self) {
        self.value = !self.value;
    }
}

impl Operate<Bool> for Bool {
    fn operate(&self, rhs: &Bool, operator: &str) -> Result<RtVal, RtValErr> {
        match operator {
            "and" => Ok((self.value && rhs.value).into()),
            "or" => Ok((self.value || rhs.value).into()),
            "==" => Ok((self.value == rhs.value).into()),
            "!=" => Ok((self.value != rhs.value).into()),
            op => Err(RtValErr::UnsupportedOpOnType(op.to_string(), "bool".into())),
        }
    }
}

// ------------
//   Function
// ------------
#[derive(Debug, Clone)]
pub struct Function {
    pub name: EcoString,
    pub params: Rc<Vec<EcoString>>,
    pub body: Rc<Vec<Stmt>>,
    pub closure: Rc<RefCell<Env>>,
}

impl Function {
    pub fn new(stmt: &FnDeclStmt, closure: Rc<RefCell<Env>>) -> Self {
        Self {
            name: stmt.name.clone(),
            params: stmt.params.clone(),
            body: stmt.body.clone(),
            closure: closure.clone(),
        }
    }
}

impl RtVal {
    pub fn new_fn(stmt: &FnDeclStmt, closure: Rc<RefCell<Env>>) -> Self {
        RtVal::FuncVal(Function::new(stmt, closure))
    }

    pub fn new_struct(stmt: &StructStmt, methods: HashMap<EcoString, Function>) -> Self {
        RtVal::StructVal(Rc::new(RefCell::new(
            Struct {
                name: stmt.name.clone(),
                methods,
            }
        )))
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Callable for Function {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<RtVal>,
    ) -> Result<RtVal, PhyResult<CallErr>> {
        let mut new_env = Env::new(Some(self.closure.clone()));

        for (p, v) in self.params.iter().zip(args) {
            new_env
                .declare_var(p.clone(), v)
                .map_err(|_| PhyResult::new(CallErr::WrongFnParamDecl, None))?;
        }

        match interpreter.execute_block_stmt(&self.body, new_env) {
            Ok(_) => Ok(RtVal::new_null()),
            Err(e) => match e.err {
                InterpErr::Return(v) => Ok(v),
                _ => Err(PhyResult::new(
                    CallErr::FnExecution(e.err.to_string()),
                    None,
                )),
            },
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}


// -------------
//   Structure
// -------------
#[derive(Debug, PartialEq)]
pub struct Struct {
    pub name: EcoString,
    pub methods: HashMap<EcoString, Function>,
}

impl Callable for Rc<RefCell<Struct>> {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<RtVal>,
    ) -> Result<RtVal, PhyResult<CallErr>> {
        Ok(RtVal::InstanceVal(Rc::new(RefCell::new(Instance {
            strukt: self.clone(),
            fields: HashMap::new(),
        }))))
    }
}

impl Struct {
    pub fn find_method(&self, name: EcoString) -> Option<Function> {
        self.methods.get(&name).cloned()
    }
}

// ------------
//   Instance
// ------------
#[derive(Debug, PartialEq)]
pub struct Instance {
    pub strukt: Rc<RefCell<Struct>>,
    pub fields: HashMap<EcoString, RtVal>,
}

impl Instance {
    pub fn set(&mut self, name: EcoString, value: RtVal) -> Result<(), RtValErr> {
        if let Occupied(mut v) = self.fields.entry(name.clone()) {
            v.insert(value);
            Ok(())
        } else {
            Err(RtValErr::MissingFieldInStruct(name))
        }
    }
}

// --------
//   Into
// --------
impl From<i64> for RtVal {
    fn from(value: i64) -> Self {
        RtVal::IntVal(Rc::new(RefCell::new(Int { value })))
    }
}

impl From<f64> for RtVal {
    fn from(value: f64) -> Self {
        RtVal::RealVal(Rc::new(RefCell::new(Real { value })))
    }
}

impl From<EcoString> for RtVal {
    fn from(value: EcoString) -> Self {
        RtVal::StrVal(Rc::new(RefCell::new(Str {
            value: value.clone(),
        })))
    }
}

impl From<String> for RtVal {
    fn from(value: String) -> Self {
        RtVal::StrVal(Rc::new(RefCell::new(Str {
            value: value.into(),
        })))
    }
}

impl From<bool> for RtVal {
    fn from(value: bool) -> Self {
        RtVal::BoolVal(Rc::new(RefCell::new(Bool { value })))
    }
}

// -----------
//   Display
// -----------
impl Display for RtVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            RtVal::IntVal(i) => write!(f, "{}", i.borrow().value),
            RtVal::RealVal(r) => write!(f, "{}", r.borrow().value),
            RtVal::BoolVal(b) => write!(f, "{}", b.borrow().value),
            RtVal::StrVal(s) => write!(f, "\"{}\"", s.borrow().value),
            RtVal::FuncVal(func) => write!(f, "<fn {}>", func.name),
            RtVal::NativeFnVal(func) => write!(f, "{}", func),
            RtVal::StructVal(s) => write!(f, "<struct {}>", s.borrow().name),
            RtVal::InstanceVal(i) => write!(f, "<{} instance>", i.borrow().strukt.borrow().name),
            RtVal::Null => write!(f, "null"),
        }
    }
}
