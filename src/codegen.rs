// TODO Take a parse tree and generate code.

use convert_case::{Case, Casing};
use std::io::{BufWriter, Write};

use crate::parser::{FieldType, ParseError, ParseTree, Parser};

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
        FieldType::Undef => unimplemented!(),
    }
}

fn fmt_struct(s: &str) -> String {
    s.to_case(Case::UpperCamel)
}

fn fmt_field(s: &str) -> String {
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

        // TODO: Based on name of the service? Generate RPC code.
        self.gen_service_handlers(writer)?;
        self.gen_service_callers(writer)?;
        Ok(())
    }

    fn gen_messages<W: Write>(
        &mut self,
        writer: &mut W,
        parse: &ParseTree,
    ) -> Result<(), std::io::Error> {
        // Create a struct:
        // - Title based on message name
        // - Populate fields and names
        for msg in parse.messages.iter() {
            writeln!(
                writer,
                "pub struct {} {{",
                fmt_struct(&parse.get_str(msg.name))
            )?;
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
            writeln!(writer, "}}")?
        }
        Ok(())
    }

    fn gen_service_callers<W: Write>(&mut self, writer: &mut W) -> Result<(), std::io::Error> {
        todo!();
        Ok(())
    }

    fn gen_service_handlers<W: Write>(&mut self, writer: &mut W) -> Result<(), std::io::Error> {
        todo!();
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
