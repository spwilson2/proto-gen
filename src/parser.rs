use std::{borrow::Cow, collections::HashMap, str::FromStr};

use crate::intern::{StringId, StringIntern};

type MessageId = String;

// This one will likely be in a separate file and pub.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Int32,
    Int64,
    Uint32,
    Uint64,
    String,
    Message(StringId),

    Undef,
}

impl Default for FieldType {
    fn default() -> Self {
        FieldType::Undef
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Package {}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Message {
    pub name: StringId,
    pub fields: Vec<Field>,
    pub messages: Vec<Message>,
}
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Field {
    pub name: StringId,
    pub idx: u32,
    pub ftype: FieldType,
    pub optional: bool,
}
#[derive(Debug, PartialEq, Default)]
pub struct Service {
    pub name: StringId,
    pub rpcs: Vec<Rpc>,
}
#[derive(Debug, PartialEq, Default)]
pub struct Rpc {
    pub name: StringId,
    pub arg_type: StringId,
    pub ret_type: StringId,
}

#[derive(Debug, PartialEq)]
pub enum TopLevelParse {
    SyntaxStatement, // Ignore for now...
    Package(Package),
    Service(Service),
    Message(Message),
}

pub struct Parser<I: Iterator<Item = char>> {
    // TODO: Rather than use individual copies of strings change to IDs and use this intern struct.
    intern: StringIntern,
    linenum: u32,
    colnum: u32,
    iterator: I,
    next_char: Option<char>,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Ident(String),
    Semicolon,
    BraceOpen,
    BraceClose,
    ParensOpen,
    ParensClose,
    Quote,
    Equals,
    Number(String),
    Comment(String),
    Whitespace,

    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    msg: String,
}

#[derive(Default, Debug)]
pub struct ParseTree {
    pub messages: Vec<Message>,
    pub services: Vec<Service>,
    intern: StringIntern,
    // TODO/Optiimization: Rather than duplicate, switch to a CoW topology of messages.
    message_cache: HashMap<StringId, Message>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg.as_str())
    }
}

struct StagingParseTree {
    str_cache: StringIntern,
}

impl ParseTree {
    pub fn get_str(&self, id: StringId) -> Cow<String> {
        // NOTE: Assumes that only one will ever be instantiated.
        self.intern.get_str(id).unwrap()
    }

    //pub fn get_message(&self, id: StringId) -> Option<Message> {
    //    // Need to support grabbing from nested contexts...
    //    self.message_cache.get(&id).and_then(|m| Some((*m).clone()))
    //}

    fn validate(&self) -> Result<(), ParseError> {
        // TODO: Validate RPCs use toplevel message types.
        // TODO: Validate Messages use Messages within scope.
        Ok(())
    }
}

const LINE_END: [char; 1] = ['\n'];

impl<I: Iterator<Item = char>> Parser<I> {
    pub fn unnext_char(&mut self, c: Option<char>) {
        assert_eq!(self.next_char, None);
        self.next_char = c;
    }
    pub fn next_char(&mut self) -> Option<char> {
        let c;
        if let Some(k) = self.next_char.take() {
            return Some(k);
        } else if let Some(k) = self.iterator.next() {
            c = k;
        } else {
            return None;
        }
        if LINE_END.contains(&c) {
            self.linenum += 1;
            self.colnum = 0;
        }
        self.colnum += 1;
        Some(c)
    }
    pub fn peek_char(&mut self) -> Option<char> {
        if let Some(c) = self.next_char {
            return Some(c);
        }
        self.next_char = self.iterator.next();
        self.next_char
    }
    pub fn try_next_char<F>(&mut self, cb: F) -> bool
    where
        F: FnOnce(char) -> bool,
    {
        if let Some(c) = self.peek_char() {
            let res = cb(c);
            if res {
                self.next_char();
            }
            return res;
        }
        false
    }

    pub fn new(i: I) -> Self {
        Self {
            intern: StringIntern::default(),
            linenum: 0,
            colnum: 0,
            iterator: i,
            next_char: None,
        }
    }

    pub fn next_non_ws_token(&mut self) -> Option<Token> {
        loop {
            match self.next_token() {
                Some(Token::Whitespace) => continue,
                Some(Token::Comment(_)) => continue,
                other => return other,
            }
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        while let Some(c) = self.next_char() {
            // Ident.
            if c.is_ascii_alphabetic() || c == '_' {
                let mut s = String::new();
                s.push(c);
                while let Some(c) = self.next_char() {
                    if !(c == '_' || c.is_ascii_alphanumeric()) {
                        self.unnext_char(Some(c));
                        break;
                    }
                    s.push(c);
                }
                return Some(Token::Ident(s));
            }

            // Comment.
            // We may want to actually put this into a separate context so we
            // can bounce out and keep the whole comment.
            if c == '/' {
                let mut s = String::new();
                s.push(c);
                if let Some(c) = self.next_char() {
                    s.push(c);
                    if c == '/' {
                        // Singline comment
                        // Till newline
                        while let Some(c) = self.next_char() {
                            if LINE_END.contains(&c) {
                                break;
                            }
                            s.push(c);
                        }
                        return Some(Token::Comment(s));
                    } else if c == '*' {
                        // Multiline comment
                        while let Some(c) = self.next_char() {
                            s.push(c);
                            if c == '*' && self.peek_char() == Some('/') {
                                self.next_char();
                                s.push('/');
                                return Some(Token::Comment(s));
                            }
                        }
                    }
                    return Some(Token::Error(String::from("Expected /")));
                }
            }

            // Quote
            if c.is_numeric() {
                let mut s = String::new();
                s.push(c);
                while let Some(c) = self.next_char() {
                    if !c.is_numeric() {
                        self.unnext_char(Some(c));
                        break;
                    }
                    s.push(c);
                }
                return Some(Token::Number(s));
            }
            if c.is_whitespace() {
                while let Some(c) = self.next_char() {
                    if !c.is_whitespace() {
                        self.unnext_char(Some(c));
                        break;
                    }
                }
                return Some(Token::Whitespace);
            }
            return Some(match c {
                ';' => Token::Semicolon,
                '{' => Token::BraceOpen,
                '}' => Token::BraceClose,
                '(' => Token::ParensOpen,
                ')' => Token::ParensClose,
                '"' => Token::Quote,
                '=' => Token::Equals,
                _ => Token::Error(format!("Unexpected char")),
            });
        }
        None
    }

    pub fn parse(&mut self) -> Result<ParseTree, ParseError> {
        let mut tree = ParseTree::default();
        match self.next_parse() {
            Some(Ok(TopLevelParse::SyntaxStatement)) => (),
            Some(Err(e)) => return Err(e),
            _ => todo!(), // Error - Empty file or missing syntx statement before other defs.
        }
        // TODO: Package support: parse
        loop {
            match self.next_parse() {
                Some(Ok(item)) => match item {
                    TopLevelParse::Service(s) => tree.services.push(s),
                    TopLevelParse::Message(m) => tree.messages.push(m),
                    _ => todo!(), // Error
                },
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }
        // TODO/Optimization: Should really just move rather than clone.
        tree.intern = self.intern.clone();
        tree.validate()?;
        Ok(tree)
    }

    // TODO: Parser will need to do multiple passes:
    pub fn next_parse(&mut self) -> Option<Result<TopLevelParse, ParseError>> {
        while let Some(tok) = self.next_non_ws_token() {
            return match tok {
                Token::Ident(ident) => match ident.as_str() {
                    "syntax" => Some(self.parse_syntax()),
                    "service" => Some(self.parse_service()),
                    "message" => Some(
                        self.parse_message()
                            .and_then(|msg| (Ok(TopLevelParse::Message(msg)))),
                    ),
                    _ => panic!("Unexpected token"), // TODO handle unexpected token nicely
                },
                _ => panic!("Unexpected token"), // TODO handle unexpected token nicely
            };
        }
        None
    }

    pub fn parse_syntax(&mut self) -> Result<TopLevelParse, ParseError> {
        if self.next_non_ws_token() != Some(Token::Equals) {
            todo!() // Error
        }
        if self.next_non_ws_token() != Some(Token::Quote) {
            todo!() // Error
        }
        if self.next_token() != Some(Token::Ident(String::from("proto3"))) {
            todo!() // Error
        }
        if self.next_non_ws_token() != Some(Token::Quote) {
            todo!() // Error
        }
        if self.next_non_ws_token() != Some(Token::Semicolon) {
            todo!() // Error
        }
        return Ok(TopLevelParse::SyntaxStatement);
    }

    fn parse_service(&mut self) -> Result<TopLevelParse, ParseError> {
        let mut service = Service::default();
        match self.next_non_ws_token() {
            Some(Token::Ident(ident)) => service.name = self.intern.get_id(&ident),
            _ => todo!(), // Error
        }
        if self.next_non_ws_token() != Some(Token::BraceOpen) {
            todo!() // Error
        }
        loop {
            // Now parse rpcs or braceclose
            let tok = self.next_non_ws_token();
            match tok {
                Some(Token::Ident(maybe_rpc_ident)) => {
                    if maybe_rpc_ident.as_str() != "rpc" {
                        todo!() /*Error*/
                    }
                    service.rpcs.push(self.parse_rpc()?);
                }
                Some(Token::BraceClose) => break, // Done parsing
                _ => todo!(),                     // Error
            }
        }
        Ok(TopLevelParse::Service(service))
    }

    fn parse_rpc(&mut self) -> Result<Rpc, ParseError> {
        let mut rpc = Rpc::default();
        // Entered after RPC has been parsed
        match self.next_non_ws_token() {
            Some(Token::Ident(rpc_name)) => rpc.name = self.intern.get_id(&rpc_name),
            _ => todo!(), // Error
        }
        if self.next_non_ws_token() != Some(Token::ParensOpen) {
            todo!() // Error
        }
        match self.next_non_ws_token() {
            Some(Token::Ident(arg)) => rpc.arg_type = self.intern.get_id(&arg),
            _ => todo!(), // Error
        }
        if self.next_non_ws_token() != Some(Token::ParensClose) {
            todo!() // Error
        }
        match self.next_non_ws_token() {
            Some(Token::Ident(returns_kw)) => {
                if returns_kw.as_str() != "returns" {
                    todo!() // Error
                }
            }
            _ => todo!(), // Error
        }
        if self.next_non_ws_token() != Some(Token::ParensOpen) {
            todo!() // Error
        }
        match self.next_non_ws_token() {
            Some(Token::Ident(ret)) => rpc.ret_type = self.intern.get_id(&ret),
            _ => todo!(), // Error
        }
        if self.next_non_ws_token() != Some(Token::ParensClose) {
            todo!() // Error
        }
        if self.next_non_ws_token() != Some(Token::Semicolon) {
            todo!() // Error
        }
        Ok(rpc)
    }

    fn parse_message(&mut self) -> Result<Message, ParseError> {
        let mut message = Message::default();
        match self.next_non_ws_token() {
            Some(Token::Ident(ident)) => message.name = self.intern.get_id(&ident),
            _ => todo!(), // Error
        }
        if self.next_non_ws_token() != Some(Token::BraceOpen) {
            todo!() // Error
        }

        // TODO: Now parse fields, messages or braceclose
        loop {
            let tok = self.next_non_ws_token();
            match tok {
                Some(Token::BraceClose) => break,
                Some(Token::Ident(ident)) => {
                    match ident.as_str() {
                        "message" => message.messages.push(self.parse_message()?),
                        "optional" => {
                            if let Some(Token::Ident(ident)) = self.next_non_ws_token() {
                                let mut field = self.parse_field_of_type(ident)?;
                                field.optional = true;
                                message.fields.push(field);
                            } else {
                                todo!() // Error
                            }
                        }
                        _ => message.fields.push(self.parse_field_of_type(ident)?),
                    }
                }
                _ => todo!(), // Error
            }
        }
        Ok(message)
    }

    fn parse_field_of_type(&mut self, type_name: String) -> Result<Field, ParseError> {
        let mut field = Field::default();
        field.ftype = match type_name.as_str() {
            "int32" => FieldType::Int32,
            "int64" => FieldType::Int64,
            "uint32" => FieldType::Uint32,
            "uint64" => FieldType::Uint64,
            "string" => FieldType::String,
            ident => FieldType::Message(self.intern.get_id(&ident)), //Error
        };
        field.name = match self.next_non_ws_token() {
            Some(Token::Ident(fname)) => self.intern.get_id(&fname),
            _ => todo!(), // Error
        };
        if self.next_non_ws_token() != Some(Token::Equals) {
            todo!() // Error
        }
        field.idx = match self.next_non_ws_token() {
            Some(Token::Number(n)) => n.parse().unwrap(), // TODO: handle failure to convert to u32

            _ => todo!(), // Error
        };
        if self.next_non_ws_token() != Some(Token::Semicolon) {
            todo!() // Error
        }
        Ok(field)
    }
}

// Parser tests
#[test]
fn solo_syntax_test() {
    let ident = "syntax = \"proto3\";";
    let mut p = Parser::new(ident.chars());
    assert_eq!(p.next_parse(), Some(Ok(TopLevelParse::SyntaxStatement)));
    assert_eq!(p.next_parse(), None);
}

#[test]
fn solo_service_test() {
    let ident = "service hi {
        rpc do(something) returns (null);
    
    }";
    let mut p = Parser::new(ident.chars());
    assert_eq!(
        p.next_parse(),
        Some(Ok(TopLevelParse::Service(Service {
            name: p.intern.get_id("hi"),
            rpcs: vec![Rpc {
                name: p.intern.get_id("do"),
                arg_type: p.intern.get_id("something"),
                ret_type: p.intern.get_id("null"),
            }]
        })))
    );
    assert_eq!(p.next_parse(), None);
}

#[test]
fn solo_message_test() {
    let ident = "message HiReq {
        optional string msg = 1;
        message inner {
            int32 inner_field = 1;
        }
        inner idx = 2;
    }";
    let mut p = Parser::new(ident.chars());
    assert_eq!(
        p.next_parse(),
        Some(Ok(TopLevelParse::Message(Message {
            name: p.intern.get_id("HiReq"),
            fields: vec![
                Field {
                    name: p.intern.get_id("msg"),
                    idx: 1,
                    ftype: FieldType::String,
                    optional: true,
                },
                Field {
                    name: p.intern.get_id("idx"),
                    idx: 2,
                    ftype: FieldType::Message(p.intern.get_id("inner")),
                    optional: false,
                }
            ],
            messages: vec![Message {
                name: p.intern.get_id("inner"),
                fields: vec![Field {
                    name: p.intern.get_id("inner_field"),
                    idx: 1,
                    ftype: FieldType::Int32,
                    optional: false,
                },],
                messages: vec![],
            }],
        })))
    );
    assert_eq!(p.next_parse(), None);
}

// Tokenizer tests
#[test]
fn solo_ident_test() {
    let ident = "hello";
    let mut p = Parser::new(ident.chars());
    assert_eq!(Some(Token::Ident(String::from(ident))), p.next_token());
    assert_eq!(None, p.next_token());
}
#[test]
fn single_tokens_test() {
    let chars = ";{}()\"=";
    let mut p = Parser::new(chars.chars());
    assert_eq!(Some(Token::Semicolon), p.next_token());
    assert_eq!(Some(Token::BraceOpen), p.next_token());
    assert_eq!(Some(Token::BraceClose), p.next_token());
    assert_eq!(Some(Token::ParensOpen), p.next_token());
    assert_eq!(Some(Token::ParensClose), p.next_token());
    assert_eq!(Some(Token::Quote), p.next_token());
    assert_eq!(Some(Token::Equals), p.next_token());
    assert_eq!(None, p.next_token());
}
#[test]
fn solo_number_test() {
    let number = "12348";
    let mut p = Parser::new(number.chars());
    assert_eq!(Some(Token::Number(String::from(number))), p.next_token());
    assert_eq!(None, p.next_token());
}
#[test]
fn solo_comment_test() {
    let comment = "// hello comment";
    let mut p = Parser::new(comment.chars());
    assert_eq!(Some(Token::Comment(String::from(comment))), p.next_token());
    assert_eq!(None, p.next_token());
}
#[test]
fn solo_multiline_comment_test() {
    let comment = "/* hello comment
    */";
    let mut p = Parser::new(comment.chars());
    assert_eq!(Some(Token::Comment(String::from(comment))), p.next_token());
    assert_eq!(None, p.next_token());
}
#[test]
fn solo_whitespace_test() {
    let whitespace = "     
    
    ";
    let mut p = Parser::new(whitespace.chars());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(None, p.next_token());
}
#[test]
fn complex_test() {
    let src = "
syntax = \"proto3\";

// Some comment
service hi {
    rpc do(something) returns (null);

}";
    let mut p = Parser::new(src.chars());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(Some(Token::Ident(String::from("syntax"))), p.next_token());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(Some(Token::Equals), p.next_token());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(Some(Token::Quote), p.next_token());
    assert_eq!(Some(Token::Ident(String::from("proto3"))), p.next_token());
    assert_eq!(Some(Token::Quote), p.next_token());
    assert_eq!(Some(Token::Semicolon), p.next_token());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(
        Some(Token::Comment(String::from("// Some comment"))),
        p.next_token()
    );
    assert_eq!(Some(Token::Ident(String::from("service"))), p.next_token());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(Some(Token::Ident(String::from("hi"))), p.next_token());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(Some(Token::BraceOpen), p.next_token());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(Some(Token::Ident(String::from("rpc"))), p.next_token());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(Some(Token::Ident(String::from("do"))), p.next_token());
    assert_eq!(Some(Token::ParensOpen), p.next_token());
    assert_eq!(
        Some(Token::Ident(String::from("something"))),
        p.next_token()
    );
    assert_eq!(Some(Token::ParensClose), p.next_token());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(Some(Token::Ident(String::from("returns"))), p.next_token());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(Some(Token::ParensOpen), p.next_token());
    assert_eq!(Some(Token::Ident(String::from("null"))), p.next_token());
    assert_eq!(Some(Token::ParensClose), p.next_token());
    assert_eq!(Some(Token::Semicolon), p.next_token());
    assert_eq!(Some(Token::Whitespace), p.next_token());
    assert_eq!(Some(Token::BraceClose), p.next_token());
    assert_eq!(None, p.next_token());
}
