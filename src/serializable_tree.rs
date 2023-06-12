use serde::Serialize;

use crate::{
    intern::StringIntern,
    parser::{FieldType, ParseTree},
};

#[derive(Debug)]
pub struct SerializeTree {
    pub messages: Vec<Message>,
    pub enums: Vec<Enum>,
    pub services: Vec<Service>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
    pub messages: Vec<Message>,
    pub enums: Vec<Enum>,
}
#[derive(Debug, PartialEq, Default, Clone, Serialize)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}
#[derive(Debug, PartialEq, Default, Clone, Serialize)]
pub struct EnumVariant {
    pub name: String,
    pub id: u32,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize)]
pub struct Field {
    pub name: String,
    pub idx: u32,
    pub ftype: String,
    pub optional: bool,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize)]
pub struct Service {
    pub name: String,
    pub rpcs: Vec<Rpc>,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize)]
pub struct Rpc {
    pub name: String,
    pub arg_type: String,
    pub ret_type: String,
}

fn field_type_to_rust_str(intern: &StringIntern, ft: &FieldType) -> String {
    match ft {
        FieldType::Int32 => "i32".into(),
        FieldType::Int64 => "i64".into(),
        FieldType::Uint32 => "u32".into(),
        FieldType::Uint64 => "u64".into(),
        FieldType::String => "String".into(),
        FieldType::Message(m) => intern.get_str(*m).unwrap().as_ref().clone(),
        FieldType::Enum(e) => intern.get_str(*e).unwrap().as_ref().clone(),
        FieldType::Undef => unimplemented!(),
    }
}

impl SerializeTree {
    fn rollup_enum(tree: &ParseTree, cur_enum: &crate::parser::Enum) -> Enum {
        let mut enum_ = Enum::default();
        enum_.name = tree.get_str(cur_enum.name).as_ref().clone();
        for var in cur_enum.variants.iter() {
            enum_.variants.push(EnumVariant {
                name: tree.get_str(var.name).as_ref().clone(),
                id: var.id,
            })
        }
        enum_
    }
    fn rollup_message(tree: &ParseTree, msg: &crate::parser::Message) -> Message {
        let mut fields = Vec::new();
        for field in msg.fields.iter() {
            fields.push(Field {
                name: tree.get_str(field.name).as_ref().clone(),
                idx: field.idx,
                ftype: field_type_to_rust_str(&tree.intern, &field.ftype),
                optional: field.optional,
            })
        }
        let mut messages = Vec::new();
        for message in msg.messages.iter() {
            messages.push(SerializeTree::rollup_message(tree, message));
        }

        let mut enums = Vec::new();
        for enum_ in msg.enums.iter() {
            enums.push(SerializeTree::rollup_enum(tree, enum_));
        }

        Message {
            name: tree.get_str(msg.name).as_ref().clone(),
            fields,
            messages,
            enums,
        }
    }
    pub fn from_parse_tree(tree: &ParseTree) -> Self {
        let mut messages = Vec::new();
        for msg in tree.messages.iter() {
            let mut fields = Vec::new();
            for field in msg.fields.iter() {
                fields.push(Field {
                    name: tree.get_str(field.name).as_ref().clone(),
                    idx: field.idx,
                    ftype: field_type_to_rust_str(&tree.intern, &field.ftype),
                    optional: field.optional,
                })
            }
            messages.push(Self::rollup_message(tree, msg));
        }
        let mut enums = Vec::new();
        for enum_ in tree.enums.iter() {
            enums.push(SerializeTree::rollup_enum(tree, enum_));
        }
        let mut services = Vec::new();
        for svc in tree.services.iter() {
            let mut service = Service::default();
            service.name = tree.get_str(svc.name).as_ref().clone();
            for rpc in svc.rpcs.iter() {
                service.rpcs.push(Rpc {
                    name: tree.get_str(rpc.name).as_ref().clone(),
                    arg_type: tree.get_str(rpc.arg_type).as_ref().clone(),
                    ret_type: tree.get_str(rpc.ret_type).as_ref().clone(),
                });
            }
            services.push(service);
        }
        Self {
            messages,
            services,
            enums,
        }
    }
}
