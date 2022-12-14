use std::any::{Any, TypeId};
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Display, Formatter};
use std::path::Iter;
use std::sync::{Arc, Mutex, MutexGuard};
use crate::ast;

pub trait ValueObject: Any {
    fn tid(&self) -> TypeId {
        self.type_id()
    }
    // fn downcast_ref<T>(&self) -> Option<&T> {
    //     let any_self = self as &dyn Any;
    //     any_self.downcast_ref::<T>()
    // }
    fn add(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> { Err(ErrorKind::FunctionNotImplemented) }
    fn sub(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> { Err(ErrorKind::FunctionNotImplemented) }
    fn mul(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> { Err(ErrorKind::FunctionNotImplemented) }
    fn div(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> { Err(ErrorKind::FunctionNotImplemented) }

    fn eq(&self, right: &Box<dyn ValueObject>) -> TResult<bool> { Err(ErrorKind::FunctionNotImplemented) }
    fn not_eq(&self, right: &Box<dyn ValueObject>) -> TResult<bool> { Ok(!(self.eq(right)?)) }

    fn to_str(&self) -> TResult<String> {
        // format!("{}", self.type_id())
        // let p = unsafe { self as *const i32 as usize };
        Ok(format!("object <{:p}>", self))
    }
    fn to_repr_str(&self) -> TResult<String> {
        let v = self.to_str()?;
        Ok(format!(r#""{}""#, v))
    }
    fn to_bool(&self) -> TResult<bool> { Ok(true) }

    fn get_iter(&self) -> TResult<WrapValueObject> { Err(ErrorKind::FunctionNotImplemented) }
    fn iter_next(&mut self) -> TResult<Option<WrapValueObject>> { Err(ErrorKind::FunctionNotImplemented) }
}

pub fn downcast_ref<T: 'static>(v: &Box<dyn ValueObject>) -> Option<&T> {
    if TypeId::of::<T>() == v.tid() {
        Some(unsafe { &*(v as *const dyn Any as *const Box<T>) })
    } else {
        None
    }
}

impl Display for dyn ValueObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.to_str() {
            Ok(obj) => {
                f.write_str(format!("{}", obj).as_str())?;
                Ok(())
            }
            Err(_) => {
                Err(std::fmt::Error)
            }
        }
    }
}

impl Debug for dyn ValueObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.to_str() {
            Ok(obj) => {
                f.write_str(format!("{:?}", obj).as_str())?;
                Ok(())
            }
            Err(_) => {
                Err(std::fmt::Error)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct WrapValueObject {
    obj: Arc<RefCell<Box<dyn ValueObject>>>,
}

impl WrapValueObject {
    pub fn from_box(obj: Box<dyn ValueObject>) -> Self {
        WrapValueObject {
            obj: Arc::from(RefCell::from(obj)),
        }
    }
    pub fn unwrap(&self) -> Ref<'_, Box<dyn ValueObject>> {
        RefCell::borrow(self.obj.borrow())
    }
    pub fn unwrap_mut(&self) -> RefMut<'_, Box<dyn ValueObject>> {
        RefCell::borrow_mut(self.obj.borrow())
    }
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        let v = &*self.unwrap();
        if TypeId::of::<T>() == v.tid() {
            Some(unsafe { &*(v as *const dyn Any as *const Box<T>) })
        } else {
            None
        }
    }
    pub fn t_add(&self, right: Self) -> TResult<WrapValueObject> {
        // let v = right.unwrap();
        // v.add()
        self.unwrap().add(&*right.unwrap())
    }
    pub fn t_sub(&self, right: Self) -> TResult<WrapValueObject> {
        self.unwrap().sub(&*right.unwrap())
    }
    pub fn t_mul(&self, right: Self) -> TResult<WrapValueObject> {
        self.unwrap().mul(&*right.unwrap())
    }
    pub fn t_div(&self, right: Self) -> TResult<WrapValueObject> {
        self.unwrap().div(&*right.unwrap())
    }

    pub fn t_eq(&self, right: Self) -> TResult<WrapValueObject> {
        let v = self.unwrap().eq(&*right.unwrap())?;
        Ok(WrapValueObject::from_box(Box::new(v)))
    }
    pub fn t_not_eq(&self, right: Self) -> TResult<WrapValueObject> {
        let v = self.unwrap().eq(&*right.unwrap())?;
        Ok(WrapValueObject::from_box(Box::new(v)))
    }
    pub fn t_get_iter(&self) -> TResult<WrapValueObject> { self.unwrap().get_iter() }
    pub fn t_iter_next(&mut self) -> TResult<Option<WrapValueObject>> { self.unwrap_mut().iter_next() }


    pub fn to_str(&self) -> TResult<String> { self.unwrap().to_str() }
    pub fn to_repr_str(&self) -> TResult<String> { self.unwrap().to_repr_str() }
    pub fn to_bool(&self) -> TResult<bool> { self.unwrap().to_bool() }
}

pub type TResult<T> = Result<T, ErrorKind>;

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorKind {
    Syntax {
        error_line_number: usize,
    },
    FunctionNotImplemented,
    RuntimeError(String),
    CommandError(String, String),
    NameError(String),
    TypeError,
}

impl ErrorKind {
    pub fn make_run_err(message: &str) -> Self {
        Self::RuntimeError(message.to_string())
    }
}

impl ValueObject for ErrorKind {
    fn to_str(&self) -> TResult<String> {
        let v = match self {
            ErrorKind::Syntax { error_line_number } => {
                format!("SyntaxError: line {}", error_line_number)
            }
            ErrorKind::FunctionNotImplemented => {
                format!("FunctionNotImplementedError")
            }
            ErrorKind::RuntimeError(v) => {
                format!("RuntimeError: {}", v)
            }
            ErrorKind::CommandError(command, message) => {
                format!("CommandError({}): {}", command, message)
            }
            ErrorKind::NameError(name) => {
                format!("NameError: name '{}' is not defined", name)
            }
            ErrorKind::TypeError => {
                format!("TypeError")
            }
        };
        Ok(v)
    }
}


impl ValueObject for i64 {
    fn add(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> {
        if let Some(right) = downcast_ref::<i64>(right) {
            let v = (*self) + (*right);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else if let Some(right) = downcast_ref::<f64>(right) {
            let v = ((*self) as f64) + (*right);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else {
            Err(ErrorKind::FunctionNotImplemented)
        }
    }
    fn sub(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> {
        if let Some(right) = downcast_ref::<i64>(right) {
            let v = (*self) - (*right);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else if let Some(right) = downcast_ref::<f64>(right) {
            let v = ((*self) as f64) - (*right);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else {
            Err(ErrorKind::FunctionNotImplemented)
        }
    }
    fn mul(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> {
        if let Some(right) = downcast_ref::<i64>(right) {
            let v = (*self) * (*right);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else if let Some(right) = downcast_ref::<f64>(right) {
            let v = ((*self) as f64) * (*right);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else {
            Err(ErrorKind::FunctionNotImplemented)
        }
    }
    fn div(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> {
        if let Some(right) = downcast_ref::<i64>(right) {
            let v = ((*self) as f64) / ((*right) as f64);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else if let Some(right) = downcast_ref::<f64>(right) {
            let v = ((*self) as f64) / (*right);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else {
            Err(ErrorKind::FunctionNotImplemented)
        }
    }
    fn eq(&self, right: &Box<dyn ValueObject>) -> TResult<bool> {
        if let Some(right) = downcast_ref::<i64>(right) {
            Ok((*self) == (*right))
        } else if let Some(right) = downcast_ref::<f64>(right) {
            Ok(((*self) as f64) == (*right))
        } else {
            Ok(false)
        }
    }
    fn to_str(&self) -> TResult<String> {
        Ok(format!("{}", self))
    }
    fn to_repr_str(&self) -> TResult<String> {
        self.to_str()
    }
    fn to_bool(&self) -> TResult<bool> {
        Ok((*self) != (0 as i64))
    }
}

impl ValueObject for f64 {
    fn add(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> {
        if let Some(right) = downcast_ref::<i64>(right) {
            let v = (*self) + ((*right) as f64);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else if let Some(right) = downcast_ref::<f64>(right) {
            let v = ((*self) as f64) + ((*right) as f64);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else {
            Err(ErrorKind::FunctionNotImplemented)
        }
    }
    fn sub(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> {
        if let Some(right) = downcast_ref::<i64>(right) {
            let v = (*self) - ((*right) as f64);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else if let Some(right) = downcast_ref::<f64>(right) {
            let v = ((*self) as f64) - ((*right) as f64);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else {
            Err(ErrorKind::FunctionNotImplemented)
        }
    }
    fn mul(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> {
        if let Some(right) = downcast_ref::<i64>(right) {
            let v = (*self) * ((*right) as f64);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else if let Some(right) = downcast_ref::<f64>(right) {
            let v = ((*self) as f64) * ((*right) as f64);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else {
            Err(ErrorKind::FunctionNotImplemented)
        }
    }
    fn div(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> {
        if let Some(right) = downcast_ref::<i64>(right) {
            let v = ((*self) as f64) / ((*right) as f64);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else if let Some(right) = downcast_ref::<f64>(right) {
            let v = ((*self) as f64) / ((*right) as f64);
            Ok(WrapValueObject::from_box(Box::new(v)))
        } else {
            Err(ErrorKind::FunctionNotImplemented)
        }
    }
    fn eq(&self, right: &Box<dyn ValueObject>) -> TResult<bool> {
        if let Some(right) = downcast_ref::<i64>(right) {
            Ok((*self) == ((*right) as f64))
        } else if let Some(right) = downcast_ref::<f64>(right) {
            Ok((*self) == (*right))
        } else {
            Ok(false)
        }
    }
    fn to_str(&self) -> TResult<String> {
        Ok(format!("{}", self))
    }
    fn to_repr_str(&self) -> TResult<String> {
        self.to_str()
    }
    fn to_bool(&self) -> TResult<bool> {
        Ok((*self) != (0 as f64))
    }
}

impl ValueObject for String {
    fn add(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> {
        if let Some(right) = downcast_ref::<String>(right) {
            let mut v = String::new();
            v.push_str(self.as_str());
            v.push_str(right.as_str());
            return Ok(WrapValueObject::from_box(Box::new(v)));
        }
        if let Some(right) = downcast_ref::<i64>(right) {
            let mut v = String::new();
            v.push_str(self.as_str());
            v.push_str(right.to_string().as_str());
            return Ok(WrapValueObject::from_box(Box::new(v)));
        }
        if let Some(right) = downcast_ref::<f64>(right) {
            let mut v = String::new();
            v.push_str(self.as_str());
            v.push_str(right.to_string().as_str());
            return Ok(WrapValueObject::from_box(Box::new(v)));
        }
        return Err(ErrorKind::FunctionNotImplemented);
    }
    fn eq(&self, right: &Box<dyn ValueObject>) -> TResult<bool> {
        if let Some(right) = downcast_ref::<Self>(right) {
            Ok((*self) == (*right))
        } else {
            Ok(false)
        }
    }
    fn to_str(&self) -> TResult<String> {
        Ok(self.clone())
    }
}

impl ValueObject for bool {
    fn to_str(&self) -> TResult<String> {
        Ok(format!("{}", self))
    }
    fn to_repr_str(&self) -> TResult<String> {
        self.to_str()
    }
    fn to_bool(&self) -> TResult<bool> {
        Ok(*self)
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct TNone;

impl ValueObject for TNone {
    fn eq(&self, right: &Box<dyn ValueObject>) -> TResult<bool> { Ok(self.tid() == right.tid()) }
    fn to_str(&self) -> TResult<String> {
        Ok(format!("None"))
    }
    fn to_repr_str(&self) -> TResult<String> {
        self.to_str()
    }
    fn to_bool(&self) -> TResult<bool> { Ok(false) }
}

impl TNone {
    pub fn a_none() -> WrapValueObject {
        WrapValueObject::from_box(Box::from(TNone::default()))
    }
}

// pub struct IterObject{}

pub type TList = Vec<WrapValueObject>;

#[derive(Debug, Clone)]
pub struct TListIter {
    list: TList,
    this: usize,
}

impl ValueObject for TList {
    fn add(&self, right: &Box<dyn ValueObject>) -> TResult<WrapValueObject> {
        if let Some(right) = downcast_ref::<Self>(right) {
            let mut v = self.clone();
            for i in right {
                v.push(i.clone())
            }
            return Ok(WrapValueObject::from_box(Box::new(v)));
        }
        return Err(ErrorKind::FunctionNotImplemented);
    }
    fn to_str(&self) -> TResult<String> {
        let mut v = String::new();
        v.push('[');
        let mut self_iter = self.iter();
        if let Some(i) = self_iter.next() {
            v.push_str(i.to_repr_str()?.as_str());
            for i in self_iter {
                v.push_str(", ");
                v.push_str(i.to_repr_str()?.as_str());
            }
        }
        v.push(']');
        Ok(v)
    }
    fn to_repr_str(&self) -> TResult<String> {
        self.to_str()
    }
    fn get_iter(&self) -> TResult<WrapValueObject> {
        Ok(WrapValueObject::from_box(Box::new(TListIter {
            list: self.clone(),
            this: 0,
        })))
    }
}

impl ValueObject for TListIter {
    fn get_iter(&self) -> TResult<WrapValueObject> { Ok(WrapValueObject::from_box(Box::new(self.clone()))) }
    fn iter_next(&mut self) -> TResult<Option<WrapValueObject>> {
        let v = self.list.get(self.this);
        match v {
            Some(v) => {
                self.this = self.this + 1;
                Ok(Some(v.clone()))
            }
            None => {
                Ok(None)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct TTargetObject {
    pub(crate) name: String,
    pub(crate) require: Vec<String>,
    pub(crate) body: Vec<ast::Node>,
}

impl ValueObject for TTargetObject {
    fn to_str(&self) -> TResult<String> {
        let mut require_str = "".to_string();
        let mut require_iter = self.require.iter();
        if let Some(i) = require_iter.next() {
            require_str.push_str(r#"""#);
            require_str.push_str(i.as_str());
            require_str.push_str(r#"""#);
            for i in require_iter {
                require_str.push_str(r#", ""#);
                require_str.push_str(i.as_str());
                require_str.push_str(r#"""#);
            }
        }
        Ok(format!(r#"TargetObject("{}", body_size={}, require=[{}])"#, self.name, self.body.len(), require_str))
    }
    fn to_repr_str(&self) -> TResult<String> {
        self.to_str()
    }
}

