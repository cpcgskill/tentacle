use crate::dt::{ValueObject};

#[derive(Debug, PartialEq, Clone)]
pub enum OperatorData {
    Eq,
    NotEq,
    Add,
    Sub,
    Mul,
    Div,
}

impl OperatorData {
    // 获得运算符优先级
    pub fn get_priority(&self) -> isize {
        match self {
            Self::Eq => -1,
            Self::NotEq => -1,
            Self::Add => 0,
            Self::Sub => 0,
            Self::Mul => 1,
            Self::Div => 1,
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum ValueData {
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Name(String),
    Value(ValueData),
    List(Vec<Node>),
    Expr(Box<(Node, OperatorData, Node)>),
    SetAttr {
        name: String,
        value: Box<Node>,
    },
    Command {
        command: String,
        args: Vec<Node>,
    },
    Target {
        name: String,
        require: Vec<String>,
        body: Vec<Node>,
    },
    If {
        if_node: Box<(Node, Vec<Node>)>,
        elif_nodes: Vec<(Node, Vec<Node>)>,
        else_node: Option<Vec<Node>>,
    },
    For {
        item_var_name: String,
        source_exp: Box<Node>,
        body: Vec<Node>,
    },
    Module {
        body: Vec<Node>,
    },
}

impl<'a> Node {
    pub fn create_expr(left_node: Node, op: OperatorData, right_node: Node) -> Node {
        Node::Expr(Box::from((left_node, op, right_node)))
    }
}

impl<'a> ValueObject for Node {}