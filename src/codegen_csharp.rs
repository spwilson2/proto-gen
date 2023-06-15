use convert_case::{Case, Casing};
use std::{
    collections::HashMap,
    io::{BufWriter, Write},
};
use tera::{to_value, Context, Tera, Value};

use crate::{
    parser::{FieldType, Message, ParseError, ParseTree, Parser, Service},
    serializable_tree::{self, SerializeTree},
};

fn fmt_struct(s: &str) -> String {
    s.to_case(Case::UpperCamel)
}

fn fmt_field(s: &str) -> String {
    s.to_case(Case::Snake)
}
fn fmt_func(s: &str) -> String {
    s.to_case(Case::Snake)
}

fn tera_func(args: &HashMap<String, Value>) -> tera::Result<tera::Value> {
    Ok(to_value(args.get(&String::from("name")).unwrap())?)
}

fn render_msg<W: Write>(
    tera: &mut Tera,
    mut ctx: Context,
    writer: &mut W,
    message: &serializable_tree::Message,
) {
    // Recurse
    for msg in message.messages.iter() {
        render_msg(tera, ctx.clone(), writer, msg);
    }
    for enum_ in message.enums.iter() {
        render_enum(tera, ctx.clone(), writer, enum_);
    }
    ctx.insert("message", message);
    tera.render_to("gen-message.cs.tera", &ctx, writer).unwrap()
}
fn render_enum<W: Write>(
    tera: &mut Tera,
    mut ctx: Context,
    writer: &mut W,
    enum_: &serializable_tree::Enum,
) {
    ctx.insert("enum", enum_);
    tera.render_to("gen-enum.cs.tera", &ctx, writer).unwrap()
}
fn render_service<W: Write>(
    tera: &mut Tera,
    mut ctx: Context,
    writer: &mut W,
    service: &serializable_tree::Service,
) {
    ctx.insert("service", service);
    tera.render_to("gen-service.cs.tera", &ctx, writer).unwrap()
}

pub struct CsharpCodeGen;
impl CsharpCodeGen {
    pub fn gen<W: Write>(
        writer: &mut W,
        parse: &ParseTree,
        serial: &SerializeTree,
    ) -> Result<(), std::io::Error> {
        // Use globbing
        let mut tera = match tera::Tera::new("templates/csharp/**") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.register_function("fmt_struct", tera_func); // TODO:
        tera.register_function("fmt_var", tera_func); // TODO:
        tera.register_function("fmt_type", tera_func); // TODO:
        let ctx = tera::Context::new();
        tera.render_to("gen-builtin.cs", &ctx, &mut *writer)
            .unwrap();
        for msg in serial.messages.iter() {
            // Render all messages recursively (tera doesn't support)
            render_msg(&mut tera, ctx.clone(), writer, msg);
        }
        for enum_ in serial.enums.iter() {
            render_enum(&mut tera, ctx.clone(), writer, enum_);
        }
        for service in serial.services.iter() {
            render_service(&mut tera, ctx.clone(), writer, service);
        }
        Ok(())
    }
}

#[test]
fn test_tera() -> Result<(), ParseError> {
    let text = "
    syntax = \"proto3\";
    message Input {
        KeyCode key_code = 0;
    }
    enum KeyCode {
        Spacebar = 0;
        Enter = 1;
    }
    message Empty {}
    service Backend {
        rpc InputEvent(Input) returns (Empty);
    }
    ";
    let mut p = Parser::new(text.chars());
    let tree = p.parse()?;
    let mut w = BufWriter::new(vec![]);
    let serial = SerializeTree::from_parse_tree(&tree);
    CsharpCodeGen::gen(&mut w, &tree, &serial);

    println!("{}", String::from_utf8_lossy(w.buffer()));
    //    assert_eq!(
    //        String::from_utf8_lossy(w.buffer()),
    //        "pub struct TestMessage {
    //}
    //pub fn doer(arg: TestMessage) -> TestMessage { todo!() }
    //"
    // );

    Ok(())
}
