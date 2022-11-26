use std::cell::{Cell};
use nom::{
    IResult,
    branch::{alt},
    character::complete::{digit1, char, space0},
    bytes::complete::{tag, take_while1},
    combinator::{not, fail},
    multi::{many0, many0_count},
    sequence::{pair, delimited, separated_pair, terminated},
};
use nom::combinator::opt;

use crate::dt::{TResult, ErrorKind};
use crate::ast::{Node, ValueData, OperatorData};


#[derive(Debug, PartialEq, Clone)]
pub struct Parser {
    pub(crate) this_line: Cell<usize>,
    pub(crate) indentation: Cell<usize>,
}


impl<'a> Parser {
    pub fn next_to(&self, s: usize) {
        self.this_line.set(self.this_line.get() + s);
    }
    pub fn next_line(&self) {
        self.next_to(1);
    }
    pub fn this_line(&self) -> usize {
        self.this_line.get()
    }
    pub fn set_indentation(&self, s: usize) {
        self.indentation.set(s);
    }
    pub fn get_indentation(&self) -> usize {
        self.indentation.get()
    }


    pub fn parse_alphanumeric_underscore(input: &'a str) -> IResult<&'a str, &'a str> {
        take_while1(|c: char| {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => true,
                _ => false,
            }
        })(input)
    }
    pub fn parse_name(input: &'a str) -> IResult<&'a str, &'a str> {
        let (input, _) = tag("$")(input)?;
        let (input, value) = Parser::parse_alphanumeric_underscore(input)?;
        Ok((input, value))
    }

    pub fn parse_name_node(input: &'a str) -> IResult<&'a str, Node> {
        let (input, value) = Parser::parse_name(input)?;
        Ok((input, Node::Name(value.to_string())))
    }

    pub fn parse_operator_data(input: &str) -> IResult<&str, OperatorData> {
        let (input, value) = alt((
            tag("=="),
            tag("!="),
            tag("+"),
            tag("-"),
            tag("*"),
            tag("/"),
        ))(input)?;
        let value = match value {
            "==" => OperatorData::Eq,
            "!=" => OperatorData::NotEq,
            "+" => OperatorData::Add,
            "-" => OperatorData::Sub,
            "*" => OperatorData::Mul,
            "/" => OperatorData::Div,
            _ => { return fail::<_, OperatorData, _>(input); }
        };
        Ok((input, value))
    }


    pub fn parse_value(input: &'a str) -> IResult<&str, Node> {
        let (input, value) = alt((
            |input: &'a str| {
                let mut symbol: f64 = 1.0;
                let (input, test_symbol) = opt(tag("+"))(input)?;
                if let Some(_) = test_symbol {
                    symbol = 1.0;
                }
                let (input, test_symbol) = opt(tag("-"))(input)?;
                if let Some(_) = test_symbol {
                    symbol = -1.0;
                }

                let (input, left_value) = digit1(input)?;
                let (input, _) = char('.')(input)?;
                let (input, right_value) = digit1(input)?;
                let value = format!("{}.{}", left_value, right_value);
                Ok((input, ValueData::Float(value.parse::<f64>().unwrap() * symbol)))
            },
            |input: &'a str| {
                let mut symbol: i64 = 1;
                let (input, test_symbol) = opt(tag("+"))(input)?;
                if let Some(_) = test_symbol {
                    symbol = 1;
                }
                let (input, test_symbol) = opt(tag("-"))(input)?;
                if let Some(_) = test_symbol {
                    symbol = -1;
                }

                let (input, value) = digit1(input)?;
                Ok((input, ValueData::Int(value.parse::<i64>().unwrap() * symbol)))
            },
            |input: &'a str| {
                let is_escape = Cell::new(false);
                let (input, _) = tag("\"")(input)?;
                let (input, value) = take_while1(|c: char| {
                    if is_escape.get() {
                        is_escape.set(false);
                        return true;
                    }
                    if c == '"' {
                        false
                    } else {
                        if c == '\\' {
                            is_escape.set(true);
                        }
                        true
                    }
                })(input)?;
                let (input, _) = tag("\"")(input)?;
                Ok((input, ValueData::String(value.to_string())))
            },
        ))(input)?;
        Ok((input, Node::Value(value)))
    }

    pub fn parse_a_have_value_node(input: &'a str) -> IResult<&'a str, Node> {
        let (input, value) = delimited(
            space0,
            alt((
                |input: &'a str| { Parser::parse_value(input) },
                |input: &'a str| { Parser::parse_name_node(input) },
                |input: &'a str| {
                    delimited(
                        char('('),
                        |input: &'a str| { Parser::parse_expr(input) },
                        char(')'),
                    )(input)
                },
            )),
            space0,
        )(input)?;
        Ok((input, value))
    }

    pub fn parse_expr(input: &'a str) -> IResult<&'a str, Node> {
        // println!("this is {:?}", input);
        let (input, mut left_node) = Parser::parse_a_have_value_node(input)?;
        // println!("left_node is {:?}", left_node);


        let (input, mut right_vec) = many0(pair(
            |input: &'a str| { Parser::parse_operator_data(input) },
            |input: &'a str| { Parser::parse_a_have_value_node(input) },
        ))(input)?;

        // println!("right_vec is {:?}", right_vec);

        while right_vec.len() > 0 {
            let (op, mut right_node) = right_vec.remove(0);
            // println!("(op, right_node) is {:?} {:?}", op, right_node);
            if right_vec.len() > 0 {
                let (next_op, next_right_node) = right_vec.remove(0);
                if next_op.get_priority() > op.get_priority() {
                    right_node = Node::create_expr(right_node, next_op, next_right_node);
                    right_vec.insert(0, (op, right_node));
                    continue;
                } else {
                    right_vec.insert(0, (next_op, next_right_node));
                }
            }
            left_node = Node::create_expr(left_node, op.clone(), right_node.clone())
        }

        Ok((input, left_node))
    }
    pub fn parse_command(input: &'a str) -> IResult<&'a str, Node> {
        let (input, command) = Parser::parse_alphanumeric_underscore(input)?;
        let (input, args) = many0(delimited(
            space0,
            alt((
                |input: &'a str| { Parser::parse_a_have_value_node(input) },
                |input: &'a str| {
                    let (input, value) = take_while1(|c: char| {
                        match c {
                            '\t' | ' ' | '\r' | '\n' => false,
                            _ => true,
                        }
                    })(input)?;
                    Ok((input, Node::Value(ValueData::String(value.to_string()))))
                },
            )),
            space0,
        ))(input)?;
        Ok((input, Node::Command { command: command.to_string(), args }))
    }
    pub fn parse_set_attr(input: &'a str) -> IResult<&'a str, Node> {
        let (input, node) = separated_pair(
            |input: &'a str| { Parser::parse_name(input) },
            delimited(space0, char('='), space0),
            alt((
                |input: &'a str| { Parser::parse_expr(input) },
                |input: &'a str| { Parser::parse_command(input) },
            )),
        )(input)?;
        Ok((input, Node::SetAttr { name: node.0.to_string(), value: Box::from(node.1) }))
    }
    pub fn parse_crlf_or_ending(ctx: &Parser, input: &'a str) -> IResult<&'a str, ()> {
        let (input, _) = space0(input)?;
        if input.len() > 0 {
            let (input, _) = alt((tag("\n"), tag("\r\n")))(input)?;
            ctx.next_line();
            Ok((input, ()))
        } else {
            Ok((input, ()))
        }
    }

    pub fn parse_blank_line(ctx: &Parser, input: &'a str) -> IResult<&'a str, ()> {
        // println!("input {:#?}", input);
        // let (input, _) = space0(input)?;
        // let (input, _) = Parser::parse_crlf_or_ending(ctx, input)?;
        let (input, _) = pair(
            many0(alt((tag(" "), tag("\t")))),
            alt((tag("\n"), tag("\r\n"))),
        )(input)?;
        // println!("output {:#?}", input);
        // ctx.next_line();
        Ok((input, ()))
    }


    pub fn parse_target_block(ctx: &Parser, input: &'a str) -> IResult<&'a str, Node> {
        let (input, _) = delimited(space0, tag("target"), space0)(input)?;
        let (input, name) = Parser::parse_name(input)?;
        let (input, _) = delimited(space0, tag(":"), space0)(input)?;
        let (input, require_nodes) = many0(
            terminated(
                |input: &'a str| {
                    let (input, value) = Parser::parse_name(input)?;
                    Ok((input, value.to_string()))
                },
                space0,
            )
        )(input)?;
        let (input, _) = Parser::parse_crlf_or_ending(ctx, input)?;

        let (input, body) = Parser::parse_block(ctx, input, ctx.get_indentation() + 1)?;
        Ok((input, Node::Target {
            name: name.to_string(),
            require: require_nodes,
            body,
        }))
    }

    pub fn parse_if_block(ctx: &Parser, input: &'a str) -> IResult<&'a str, Node> {
        let now_indentation = ctx.get_indentation();
        // println!("parse_if_block if {:?}", input);
        // if
        let (input, _) = delimited(space0, tag("if"), space0)(input)?;
        let (input, if_check_exp) = Parser::parse_expr(input)?;
        let (input, _) = delimited(space0, tag(":"), space0)(input)?;
        let (input, _) = Parser::parse_crlf_or_ending(ctx, input)?;

        let (input, if_node_body) = Parser::parse_block(ctx, input, ctx.get_indentation() + 1)?;
        ctx.set_indentation(now_indentation);

        // println!("parse_if_block elif {:?}", input);
        // elif
        let (input, elif_nodes) = many0(|input: &'a str| {
            // 略过空行
            let (input, _) = many0(|input: &'a str| { Parser::parse_blank_line(ctx, input) })(input)?;


            let (input, _) = delimited(space0, tag("elif"), space0)(input)?;
            let (input, check_exp) = Parser::parse_expr(input)?;
            let (input, _) = delimited(space0, tag(":"), space0)(input)?;
            let (input, body) = Parser::parse_block(ctx, input, ctx.get_indentation() + 1)?;
            ctx.set_indentation(now_indentation);
            Ok((input, (check_exp, body)))
        })(input)?;

        // println!("parse_if_block else {:?}", input);
        // else
        let (input, else_node) = opt(|input: &'a str| {
            // 略过空行
            let (input, _) = many0(|input: &'a str| { Parser::parse_blank_line(ctx, input) })(input)?;
            // println!("parse_if_block else parse_blank_line end {:?}", input);


            let (input, _) = delimited(space0, tag("else"), space0)(input)?;
            let (input, _) = delimited(space0, tag(":"), space0)(input)?;

            // println!("parse_if_block else keyword end {:?}", input);

            let (input, body) = Parser::parse_block(ctx, input, ctx.get_indentation() + 1)?;
            ctx.set_indentation(now_indentation);

            // println!("parse_if_block else parse_block end {:?}", input);

            Ok((input, body))
        })(input)?;

        // println!("parse_if_block end {:?}", input);
        Ok((input, Node::If {
            if_node: Box::new((if_check_exp, if_node_body)),
            elif_nodes,
            else_node,
        }))
    }

    pub fn parse_module(ctx: &Parser, input: &'a str) -> IResult<&'a str, Node> {
        let (input, value) = Parser::parse_block(ctx, input, 0)?;
        // 清空一下剩余的空字符串避免后续检测错误
        let (input, _) = alt((tag("\n"), space0))(input)?;
        // println!("parse input {:?} len = {:?}", input, input.len());
        Ok((input, Node::Module { body: value }))
    }

    pub fn parse_item(ctx: &Parser, input: &'a str) -> IResult<&'a str, Node> {
        // 略过空行
        let (input, _) = many0(|input: &'a str| { Parser::parse_blank_line(ctx, input) })(input)?;

        // 检查缩进
        let (input, indentation) = many0_count(alt((
            tag("\t"),
            tag("    "),
        )))(input)?;
        let (input, _) = not(tag(" "))(input)?;
        if indentation != ctx.get_indentation() {
            return fail::<_, Node, _>(input);
        }

        // 主解析器
        let (input, node) = alt((
            |input: &'a str| {
                let (input, node) = Parser::parse_set_attr(input)?;
                let (input, _) = Parser::parse_crlf_or_ending(ctx, input)?;
                Ok((input, node))
            },
            |input: &'a str| { Parser::parse_target_block(ctx, input) },
            |input: &'a str| { Parser::parse_if_block(ctx, input) },
            |input: &'a str| {
                let (input, node) = Parser::parse_expr(input)?;
                let (input, _) = Parser::parse_crlf_or_ending(ctx, input)?;
                Ok((input, node))
            },
            |input: &'a str| {
                let (input, node) = Parser::parse_command(input)?;
                let (input, _) = Parser::parse_crlf_or_ending(ctx, input)?;
                Ok((input, node))
            },
        ))(input)?;

        // 再次略过空行
        let (input, _) = many0(|input: &'a str| { Parser::parse_blank_line(ctx, input) })(input)?;

        Ok((input, node))
    }

    pub fn parse_block(ctx: &Parser, input: &'a str, need_indentation: usize) -> IResult<&'a str, Vec<Node>> {
        many0(
            alt((
                |input: &'a str| {
                    ctx.set_indentation(need_indentation);
                    // println!("parse_block iter item(indentation={}) start {:?}", ctx.get_indentation(), input);
                    let (input, node) = Parser::parse_item(ctx, input)?;
                    // println!("parse_block iter item(indentation={}) end {:?}", ctx.get_indentation(), input);
                    Ok((input, node))
                },
            ))
        )(input)
    }
}

impl Default for Parser {
    fn default() -> Self {
        Parser { this_line: Cell::new(1), indentation: Cell::new(0) }
    }
}

pub fn parse_code(input: &str) -> TResult<Node> {
    // let line_count = input.find('\n').unwrap_or(0)
    let ctx = Parser::default();
    match Parser::parse_module(&ctx, input) {
        Ok((output, node)) => {
            if output.len() > 0 {
                Err(ErrorKind::Syntax {
                    error_line_number: ctx.this_line(),
                })
            } else {
                Ok(node)
            }
        }
        Err(_) => {
            Err(ErrorKind::Syntax {
                error_line_number: ctx.this_line(),
            })
        }
    }
}

pub fn parse_expr(input: &str) -> TResult<Node> {
    match Parser::parse_expr(input) {
        Ok((output, node)) => {
            if output.len() > (output.find(' ').unwrap_or(0) + output.find('\t').unwrap_or(0)) {
                Err(ErrorKind::Syntax { error_line_number: 1 })
            } else {
                Ok(node)
            }
        }
        Err(_) => {
            Err(ErrorKind::Syntax { error_line_number: 1 })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse::*;

    #[test]
    fn test_parse_name_node() {
        assert_eq!(Parser::parse_name_node("$11T"), Ok(("", Node::Name("11T".to_string()))));
    }

    #[test]
    fn test_value_parse() {
        assert_eq!(Parser::parse_value("11"), Ok(("", Node::Value(ValueData::Int(11)))));
        assert_eq!(Parser::parse_value("11.5"), Ok(("", Node::Value(ValueData::Float(11.5)))));
        assert_eq!(Parser::parse_value(r###""11.5""###), Ok(("", Node::Value(ValueData::String("11.5".to_string())))));
    }

    #[test]
    fn test_expr_parse() {
        assert_eq!(
            Parser::parse_expr("18 + 6 * 8.5 * (9+1)"),
            Ok((
                "",
                Node::create_expr(
                    Node::Value(ValueData::Int(18)),
                    OperatorData::Add,
                    Node::create_expr(
                        Node::create_expr(
                            Node::Value(ValueData::Int(6)),
                            OperatorData::Mul,
                            Node::Value(ValueData::Float(8.5)),
                        ),
                        OperatorData::Mul,
                        Node::create_expr(
                            Node::Value(ValueData::Int(9)),
                            OperatorData::Add,
                            Node::Value(ValueData::Int(1)),
                        ),
                    ),
                ),
            ))
        );
        assert_eq!(Parser::parse_expr("$test+6"), Ok(("",
                                                      Node::create_expr(
                                                          Node::Name("test".to_string()),
                                                          OperatorData::Add,
                                                          Node::Value(ValueData::Int(6)),
                                                      )
        )));
        let code = r#""aa" + "bb" + 15 + " " + 10.5"#;
        println!("parse {:#?}", code);
        let v = Parser::parse_expr(
            code,
        );
        println!("IResult {:#?}", v);
    }

    #[test]
    fn test_parse_command() {
        let v = Parser::parse_command(
            r#"message test_command2 target_index ("aa" + "bb" + 15 + " " + 10.5)"#,
        );
        println!("{:#?}", v);
    }

    #[test]
    fn test_parse_set_attr() {
        assert_eq!(Parser::parse_set_attr("$t = 15\n"), Ok((
            "\n",
            Node::SetAttr {
                name: "t".to_string(),
                value: Box::new(Node::Value(ValueData::Int(15))),
            },
        )));
    }

    #[test]
    fn test_parse_line() {
        let ctx = Parser::default();
        assert_eq!(Parser::parse_item(&ctx, "$t = 15\n$t = 15"), Ok((
            "$t = 15",
            Node::SetAttr {
                name: "t".to_string(),
                value: Box::new(Node::Value(ValueData::Int(15))),
            },
        )));
    }

    #[test]
    fn print_build() {
        let code = r###"
$target_index = 15
message test_command1
message test_command2 target_index ("aa" + "bb" + 15 + " " + 10.5)
message test_command3 target_index ($target_index+1)
"###;
        println!("{:#?}", parse_code(code));
        let code = r###"
$test_value = message test_command
"###;
        println!("{:#?}", parse_code(code));
        let code = r###"
"这是一个字符串"
"###;
        println!("{:#?}", parse_code(code));
        let code = r###"
if $test_value:
    $test_value = 0
elif $test_value:
    $test_value = 0
else:
    $test_value = 2
if $test_value:
    $test_value = 0

"###;
        println!("{:#?}", parse_code(code));
        let code = r###"
target $main:
    message target main is $main"###;
        println!("{:#?}", parse_code(code));
        let code = r###"
target $build: $clean
    message test_command target_index ($target_index+1)
target $main:
    message target main is $main"###;
        println!("{:#?}", parse_code(code));
    }
}