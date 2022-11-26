mod parse;
mod rt;
mod dt;
mod ast;

use std::env::args;
use crate::dt::{ErrorKind, TResult, ValueObject, WrapValueObject};

fn doit(f: &str, target: &str) {
    let code = std::fs::read_to_string(f).expect(format!("open file({}) failed", f).as_str());

    let mut space = rt::RunSpace::default();
    for (k, v) in std::env::vars() {
        space.set(k.as_str(), WrapValueObject::from_box(Box::new(v)));
    }
    if let Ok(v) = std::env::current_dir() {
        if let Some(v) = v.to_str() {
            space.set("current_dir", WrapValueObject::from_box(Box::new(v.to_string())));
        }
    }
    if let Ok(v) = std::env::current_exe() {
        if let Some(v) = v.to_str() {
            space.set("current_exe", WrapValueObject::from_box(Box::new(v.to_string())));
        }
    }
    if let Err(e) = rt::exec_code(code.as_str(), &mut space) {
        let e = <ErrorKind as ValueObject>::to_str(&e);
        let e = e.unwrap();
        panic!("exec error: {}", e);
    }
    if let Err(e) = rt::exec_target(&mut space, target) {
        let e = <ErrorKind as ValueObject>::to_str(&e);
        let e = e.unwrap();
        panic!("build target {} error: {}", target, e);
    }
}

fn main() {
    let args = args().collect::<Vec<String>>();
    if args.len() != 2{
        panic!("args error")
    }
    let target = args.get(1).expect("args error");
    doit("./main.tentacle", target.as_str())
}