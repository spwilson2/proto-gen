// TODO Take a parse tree and generate code.

use convert_case::{Case, Casing};
use std::io::{BufWriter, Write};

use crate::parser::{FieldType, Message, ParseError, ParseTree, Parser, Service};

//trait Codegen {}
struct RustCodeGen {}

const fn field_type_to_rust_str(ft: &FieldType) -> &'static str {
    match ft {
        FieldType::Int32 => "i32",
        FieldType::Int64 => "i64",
        FieldType::Uint32 => "u32",
        FieldType::Uint64 => "u64",
        FieldType::String => "&[u8]",
        FieldType::Message(_) => unimplemented!(),
        FieldType::Enum(_) => unimplemented!(),
        FieldType::Undef => unimplemented!(),
    }
}

fn fmt_struct(s: &str) -> String {
    s.to_case(Case::UpperCamel)
}

fn fmt_field(s: &str) -> String {
    s.to_case(Case::Snake)
}
fn fmt_func(s: &str) -> String {
    s.to_case(Case::Snake)
}

// TODO: Impleent
//impl Codegen for RustCodeGen {}
impl RustCodeGen {
    pub fn gen<W: Write>(
        &mut self,
        writer: &mut W,
        parse: &ParseTree,
    ) -> Result<(), std::io::Error> {
        // For each message, create a struct.
        self.gen_messages(writer, parse)?;

        for service in parse.services.iter() {
            // TODO: Based on name of the service? Generate RPC code.
            //self.gen_service_handlers(writer)?;
            self.gen_service_callers(writer, service, parse)?;
        }
        Ok(())
    }

    fn gen_messages<W: Write>(
        &mut self,
        writer: &mut W,
        parse: &ParseTree,
    ) -> Result<(), std::io::Error> {
        for msg in parse.messages.iter() {
            self.gen_message(writer, parse, msg)?;
        }
        Ok(())
    }

    fn gen_message<W: Write>(
        &mut self,
        writer: &mut W,
        parse: &ParseTree,
        msg: &Message,
    ) -> Result<(), std::io::Error> {
        // Create a struct:
        // - Title based on message name
        // - Populate fields and names
        writeln!(
            writer,
            "pub struct {} {{",
            fmt_struct(&parse.get_str(msg.name))
        )?;
        // TODO Create structs for nested structs as well.
        // TODO  Probably should also derive stuff..
        for field in msg.fields.iter() {
            let name = fmt_field(&parse.get_str(field.name));
            let type_ = match &field.ftype {
                FieldType::Message(msg) => {
                    let m = *msg;
                    fmt_struct(&parse.get_str(m))
                }
                f => String::from(field_type_to_rust_str(&f)),
            };
            writeln!(writer, "pub {}: {},", name, type_)?;
        }
        writeln!(writer, "}}")?;
        for msg in msg.messages.iter() {
            self.gen_message(writer, parse, msg)?;
        }
        Ok(())
    }

    fn gen_service_callers<W: Write>(
        &mut self,
        writer: &mut W,
        service: &Service,
        parse: &ParseTree,
    ) -> Result<(), std::io::Error> {
        // TODO: Write code for the backend handling as well.
        for rpc in service.rpcs.iter() {
            writeln!(
                writer,
                "pub fn {}(arg: {}) -> {} {{ todo!() }}",
                fmt_func(&parse.get_str(rpc.name)),
                fmt_struct(&parse.get_str(rpc.arg_type)),
                fmt_struct(&parse.get_str(rpc.ret_type))
            )?;
        }
        Ok(())
    }

    fn gen_service_handlers<W: Write>(
        &mut self,
        writer: &mut W,
        service: &Service,
        parse: &ParseTree,
    ) -> Result<(), std::io::Error> {
        // TODO: Write code for the backend handling as well.
        for rpc in service.rpcs.iter() {
            writeln!(
                writer,
                "pub fn {}(arg: {}) -> {} {{ todo!() }}",
                fmt_func(&parse.get_str(rpc.name)),
                fmt_struct(&parse.get_str(rpc.arg_type)),
                fmt_struct(&parse.get_str(rpc.ret_type))
            )?;
        }
        Ok(())
    }
}

#[test]
fn test_struct_write() -> Result<(), ParseError> {
    let text = "
    syntax = \"proto3\";
    message TestMessage {
        int32 page_number = 1;
    }
    ";
    let mut p = Parser::new(text.chars());
    let tree = p.parse()?;
    let mut gen = RustCodeGen {};
    let mut w = BufWriter::new(vec![]);
    gen.gen(&mut w, &tree);

    assert_eq!(
        String::from_utf8_lossy(w.buffer()),
        "pub struct TestMessage {
pub page_number: i32,
}
"
    );
    Ok(())
}

#[test]
fn test_rpc_write_caller() -> Result<(), ParseError> {
    let text = "
    syntax = \"proto3\";
    message TestMessage {
    }
    service Backend {
        rpc doer(TestMessage) returns (TestMessage);
    }
    ";
    let mut p = Parser::new(text.chars());
    let tree = p.parse()?;
    let mut gen = RustCodeGen {};
    let mut w = BufWriter::new(vec![]);
    gen.gen(&mut w, &tree);

    assert_eq!(
        String::from_utf8_lossy(w.buffer()),
        "pub struct TestMessage {
}
pub fn doer(arg: TestMessage) -> TestMessage { todo!() }
"
    );

    Ok(())
}

#[test]
fn test_rpc_write_handler() -> Result<(), ParseError> {
    let text = "
    syntax = \"proto3\";
    message TestMessage {
    }
    service Backend {
        rpc doer(TestMessage) returns (TestMessage);
    }
    ";
    let mut p = Parser::new(text.chars());
    let tree = p.parse()?;
    let mut gen = RustCodeGen {};
    let mut w = BufWriter::new(vec![]);
    gen.gen(&mut w, &tree);

    assert_eq!(
        String::from_utf8_lossy(w.buffer()),
        "pub struct TestMessage {
}
pub fn doer(arg: TestMessage) -> TestMessage { todo!() }
"
    );

    Ok(())
}
