use std::str::FromStr;

// This one will likely be in a separate file and pub.
#[derive(Debug, PartialEq)]
enum FieldType {
    Int32,
    Int64,
    Uint32,
    Uint64,
    String,
}

#[derive(Debug, PartialEq, Default)]
struct Package {}
#[derive(Debug, PartialEq, Default)]
struct Message {
    fields: Vec<Field>,
    messages: Vec<Message>,
}
#[derive(Debug, PartialEq)]
struct Field {
    ftype: FieldType,
}
#[derive(Debug, PartialEq, Default)]
struct Service {
    rpcs: Vec<Rpc>,
}
#[derive(Debug, PartialEq, Default)]
struct Rpc {
    name: String,
    arg_type: String,
    ret_type: String,
}

#[derive(Debug, PartialEq)]
enum TopLevelParse {
    SyntaxStatement, // Ignore for now...
    Package(Package),
    Service(Service),
    Message(Message),
}

struct Parser<I: Iterator<Item = char>> {
    message_nest: i32,
    quote_nest: i32,
    parens_nest: i32,
    linenum: u32,
    colnum: u32,
    // TODO Context?
    iterator: I,
    next_char: Option<char>,
}

#[derive(Debug, PartialEq)]
enum Token {
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

const LINE_END: [char; 1] = ['\n'];

impl<I: Iterator<Item = char>> Parser<I> {
    fn unnext_char(&mut self, c: Option<char>) {
        assert_eq!(self.next_char, None);
        self.next_char = c;
    }
    fn next_char(&mut self) -> Option<char> {
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
    fn peek_char(&mut self) -> Option<char> {
        if let Some(c) = self.next_char {
            return Some(c);
        }
        self.next_char = self.iterator.next();
        self.next_char
    }
    fn try_next_char<F>(&mut self, cb: F) -> bool
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
            message_nest: 0,
            quote_nest: 0,
            parens_nest: 0,
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

    // TODO: Parser will need to do multiple passes:
    pub fn next_parse(&mut self) -> Option<TopLevelParse> {
        while let Some(tok) = self.next_non_ws_token() {
            return match tok {
                Token::Ident(ident) => match ident.as_str() {
                    "syntax" => self.parse_syntax(),
                    "service" => self.parse_service(),
                    "message" => self.parse_message(),
                    _ => panic!("Unexpected token"), // TODO handle unexpected token nicely
                },
                _ => panic!("Unexpected token"), // TODO handle unexpected token nicely
            };
        }
        None
    }

    fn parse_syntax(&mut self) -> Option<TopLevelParse> {
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
        return Some(TopLevelParse::SyntaxStatement);
    }

    fn parse_service(&mut self) -> Option<TopLevelParse> {
        match self.next_non_ws_token() {
            Some(Token::Ident(ident)) => todo!(),
            _ => todo!(), // Error
        }
        if self.next_non_ws_token() != Some(Token::BraceOpen) {
            todo!() // Error
        }
        let mut service = Service::default();
        loop {
            // Now parse rpcs or braceclose
            let tok = self.next_non_ws_token();
            match tok {
                Some(Token::Ident(maybe_rpc_ident)) => {
                    if maybe_rpc_ident.as_str() != "rpc" {
                        todo!() /*Error*/
                    }
                    match self.parse_rpc() {
                        Some(rpc) => service.rpcs.push(rpc),
                        None => todo!(), // Error
                    }
                }
                Some(Token::BraceClose) => break, // Done parsing
                _ => todo!(),                     // Error
            }
        }
        Some(TopLevelParse::Service(service))
    }

    fn parse_rpc(&mut self) -> Option<Rpc> {
        let mut rpc = Rpc::default();
        // Entered after RPC has been parsed
        match self.next_non_ws_token() {
            Some(Token::Ident(rpc_name)) => rpc.name = rpc_name,
            _ => todo!(), // Error
        }
        if self.next_non_ws_token() != Some(Token::ParensOpen) {
            todo!() // Error
        }
        match self.next_non_ws_token() {
            Some(Token::Ident(arg)) => rpc.arg_type = arg,
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
            Some(Token::Ident(ret)) => rpc.ret_type = ret,
            _ => todo!(), // Error
        }
        if self.next_non_ws_token() != Some(Token::ParensClose) {
            todo!() // Error
        }
        if self.next_non_ws_token() != Some(Token::Semicolon) {
            todo!() // Error
        }
        Some(rpc)
    }

    fn parse_message(&mut self) -> Option<TopLevelParse> {
        match self.next_non_ws_token() {
            Some(Token::Ident(ident)) => todo!(),
            _ => todo!(), // Error
        }
        if self.next_non_ws_token() != Some(Token::BraceOpen) {
            todo!() // Error
        }

        let mut message = Message::default();
        // TODO: Now parse fields, messages or braceclose

        loop {
            let tok = self.next_non_ws_token();
            match tok {
                Some(Token::BraceClose) => break,
                Some(Token::Ident(ident)) => {
                    // TODO Support optional keyword
                    if ident.as_str() == "message" {
                        self.parse_message();
                    } else {
                        match self.parse_field_of_type(ident) {
                            Some(field) => message.fields.push(field),
                            _ => todo!(), // Fail, we saw text, so a field needed to come.
                        }
                    }
                }
                _ => todo!(), // Error
            }
        }
        Some(TopLevelParse::Message(message))
    }

    fn parse_field_of_type(&mut self, type_name: String) -> Option<Field> {
        let ftype = match type_name.as_str() {
            "int32" => FieldType::Int32,
            "int64" => FieldType::Int64,
            "uint32" => FieldType::Uint32,
            "uint32" => FieldType::Uint64,
            "string" => FieldType::String,
            _ => todo!(), //Error
        };
        let fname = match self.next_non_ws_token() {
            Some(Token::Ident(fname)) => fname,
            _ => todo!(), // Error
        };
        if self.next_non_ws_token() != Some(Token::Equals) {
            todo!() // Error
        }
        let number = match self.next_non_ws_token() {
            Some(Token::Number(n)) => n,
            _ => todo!(), // Error
        };
        if self.next_non_ws_token() != Some(Token::Semicolon) {
            todo!() // Error
        }
        Some(Field { ftype })
    }
}

// Parser tests
#[test]
fn solo_syntax_test() {
    let ident = "syntax = \"proto3\";";
    let mut p = Parser::new(ident.chars());
    assert_eq!(p.next_parse(), Some(TopLevelParse::SyntaxStatement));
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

fn main() {
    println!("Hello, world!");
}
