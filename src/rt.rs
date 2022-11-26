use std::collections::HashMap;
use crate::{dt, ast, parse};
use crate::dt::{TResult, ErrorKind, WrapValueObject, TList, TNone, TTargetObject};

pub type LocalCommandFunctionType = fn(Vec<String>) -> TResult<WrapValueObject>;

#[derive(Debug, Clone)]
pub struct RunSpace {
    local_commands: HashMap<String, LocalCommandFunctionType>,
    vars: HashMap<String, WrapValueObject>,
}

impl RunSpace {
    pub fn set(&mut self, key: &str, value: WrapValueObject) {
        self.vars.insert(key.to_string(), value.clone());
    }
    pub fn get(&self, key: &str) -> Option<WrapValueObject> {
        self.vars.get(key).map(|v| { v.clone() })
    }
    pub fn add_local_command(&mut self, name: &str, f: LocalCommandFunctionType) {
        self.local_commands.insert(name.to_string(), f);
    }
}

impl Default for RunSpace {
    fn default() -> Self {
        let mut space = Self {
            local_commands: HashMap::new(),
            vars: HashMap::default(),
        };
        space.add_local_command(
            "message",
            |args| {
                let v = args.join(" ");
                println!("{}", v);
                Ok(TNone::a_none())
            },
        );
        space
    }
}

pub fn exec_ast(ast: &ast::Node, space: &mut RunSpace) -> TResult<WrapValueObject> {
    match ast {
        ast::Node::Name(k) => {
            space.get(k).map_or_else(
                || {
                    let meg = format!("key {} not found", k);
                    Err(ErrorKind::make_run_err(meg.as_str()))
                },
                |v| {
                    Ok(v)
                },
            )
        }
        ast::Node::Value(v) => {
            match v {
                ast::ValueData::Int(i) => { Ok(WrapValueObject::from_box(Box::from(*i))) }
                ast::ValueData::Float(i) => { Ok(WrapValueObject::from_box(Box::from(*i))) }
                ast::ValueData::String(i) => { Ok(WrapValueObject::from_box(Box::new(i.clone()))) }
            }
        }
        ast::Node::Expr(v) => {
            let (left_node, op, right_node) = v.as_ref();
            let left_value = exec_ast(left_node, space)?;
            let right_value = exec_ast(right_node, space)?;

            match op {
                ast::OperatorData::Eq => { left_value.t_eq(right_value) }
                ast::OperatorData::NotEq => { left_value.t_not_eq(right_value) }
                ast::OperatorData::Add => { left_value.unwrap().add(right_value.unwrap()) }
                ast::OperatorData::Sub => { left_value.unwrap().sub(right_value.unwrap()) }
                ast::OperatorData::Mul => { left_value.unwrap().mul(right_value.unwrap()) }
                ast::OperatorData::Div => { left_value.unwrap().div(right_value.unwrap()) }
            }
        }
        ast::Node::SetAttr { name, value } => {
            let value = exec_ast(value, space)?;
            space.set(name, value.clone());
            Ok(TNone::a_none())
        }
        ast::Node::Command { command, args } => {
            let mut args_str = Vec::new();
            for i in args {
                let v = exec_ast(i, space)?;
                let v = v.unwrap().to_str()?;
                args_str.push(v);
            }
            if let Some(f) = space.local_commands.get(command) {
                return f(args_str);
            }
            let p = std::process::Command::new(command).args(args_str).output();
            match p {
                Ok(v) => {
                    let v = v.status.code().map_or(
                        TNone::a_none(),
                        |v| { WrapValueObject::from_box(Box::from(v as i64)) },
                    );
                    Ok(v)
                }
                Err(e) => {
                    Err(ErrorKind::CommandError(command.to_string(), e.to_string()))
                }
            }
        }
        ast::Node::Target { name, require, body } => {
            let v = TTargetObject {
                name: name.clone(),
                require: require.clone(),
                body: body.clone(),
            };
            space.set(name, WrapValueObject::from_box(Box::new(v)));
            Ok(TNone::a_none())
        }
        ast::Node::If { if_node, elif_nodes, else_node } => {
            let (check_exp, body) = if_node.as_ref();
            let v = exec_ast(check_exp, space)?;
            if exec_ast(check_exp, space)?.to_bool()? {
                for i in body {
                    exec_ast(i, space)?;
                }
                return Ok(TNone::a_none());
            }
            for (check_exp, body) in elif_nodes {
                if exec_ast(check_exp, space)?.to_bool()? {
                    for i in body {
                        exec_ast(i, space)?;
                    }
                    return Ok(TNone::a_none());
                }
            }
            if let Some(body) = else_node {
                for i in body {
                    exec_ast(i, space)?;
                }
                return Ok(TNone::a_none());
            }
            return Ok(TNone::a_none());
        }
        ast::Node::Module { body } => {
            for i in body {
                exec_ast(i, space)?;
            }
            Ok(TNone::a_none())
        }
    }
}
//
// pub fn exec_code(input: &str, space: Option<&mut RunSpace>) -> TResult<WrapValueObject> {
//     let ast = parse_code(input)?;
//     match space {
//         Some(v) => {
//             exec_ast(&ast, v)
//         }
//         None => {
//             exec_ast(&ast, &mut RunSpace::default())
//         }
//     }
// }
//
// pub fn eval_code(input: &str, space: Option<&mut RunSpace>) -> TResult<WrapValueObject> {
//     let ast = parse_expr(input)?;
//     match space {
//         Some(v) => {
//             exec_ast(&ast, v)
//         }
//         None => {
//             exec_ast(&ast, &mut RunSpace::default())
//         }
//     }
// }


pub fn exec_code(input: &str, space: &mut RunSpace) -> TResult<WrapValueObject> {
    let ast = parse::parse_code(input)?;
    exec_ast(&ast, space)
}

pub fn eval_code(input: &str, space: &mut RunSpace) -> TResult<WrapValueObject> {
    let ast = parse::parse_expr(input)?;
    exec_ast(&ast, space)
}

pub fn exec_target(space: &mut RunSpace, target: &str) -> TResult<WrapValueObject> {
    let v = space.get(target);
    match v {
        Some(v) => {
            let v = dt::downcast_ref::<dt::TTargetObject>(v.unwrap());
            match v {
                Some(v) => {
                    let v = v.clone();
                    for i in v.require {
                        exec_target(space, i.as_str())?;
                    }
                    for i in v.body {
                        exec_ast(&i, space)?;
                    }
                    Ok(dt::TNone::a_none())
                }
                None => Err(ErrorKind::TypeError)
            }
        }
        None => Err(ErrorKind::NameError(target.to_string()))
    }
}

#[cfg(test)]
mod test {
    use crate::rt::{eval_code, exec_code, exec_target, RunSpace};

    #[test]
    fn test_eval_code() {
        let code = r###"("test expr " + "这是个用来测试的字符串" + "-" + 15 + "-" + 10.5)"###;
        let mut space = RunSpace::default();
        println!("{:?}", eval_code(code, &mut space));
    }

    #[test]
    fn test_exec() {
        let code = r###"
$target_index = 15
message test_command1
message test_command2 target_index ("test expr" + " 测试整数格式化 " + -15 + " 测试浮点数格式化 " + 10.5)
message test_command3 target_index ($target_index+-1)
target $clean:
    message target clean is $clean
target $build: $clean
    "用于测试的target"
    message target build is $build
message test_print_target $build

$select = 0
if $select == 0:
    message test eq and not eq "$select is" 0
elif $select == 1:
    message test eq and not eq "$select is" 1
elif $select == 2:
    message test eq and not eq "$select is" 2
else:
    message test eq and not eq "$select is not (0, 1, 2)"

$select = "a"
if $select == "a":
    message test eq and not eq "$select is" a
elif $select == "b":
    message test eq and not eq "$select is" b
elif $select == "c":
    message test eq and not eq "$select is" c
else:
    message test eq and not eq "$select is not (a, b, c)"
"###;
        let mut space = RunSpace::default();
        println!("# test exec_code: ");
        println!("{:?}", exec_code(code, &mut space));
        println!("# test exec_target: ");
        println!("{:?}", exec_target(&mut space, "build"));
    }
}