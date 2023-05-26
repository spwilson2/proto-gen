use std::str::FromStr;

struct Package {}
struct Message {}
struct Field {}
struct Service {}
struct Rpc {}

enum ParseItem {}

struct Parser<I: Iterator<Item = char>> {
    message_nest: i32,
    quote_nest: i32,
    parens_nest: i32,
    comment_nest: i32,
    linenum: u32,
    colnum: u32,
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
    pub fn parse(i: I) -> Self {
        Self {
            message_nest: 0,
            quote_nest: 0,
            parens_nest: 0,
            comment_nest: 0,
            linenum: 0,
            colnum: 0,
            iterator: i,
            next_char: None,
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
                    if c == '/' {  // Singline comment
                        // Till newline
                        while let Some(c) = self.next_char() {
                            if LINE_END.contains(&c) {
                                break;
                            }
                            s.push(c);
                        }
                        return Some(Token::Comment(s));
                    } 
                    else if c == '*' {  // Multiline comment
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
}

#[test]
fn solo_ident_test() {
    let ident = "hello";
    let mut p = Parser::parse(ident.chars());
    assert_eq!(Some(Token::Ident(String::from(ident))), p.next_token());
    assert_eq!(None, p.next_token());
}
#[test]
fn single_tokens_test() {
    let chars = ";{}()\"=";
    let mut p = Parser::parse(chars.chars());
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
    let mut p = Parser::parse(number.chars());
    assert_eq!(Some(Token::Number(String::from(number))), p.next_token());
    assert_eq!(None, p.next_token());
}
#[test]
fn solo_comment_test() {
    let comment = "// hello comment";
    let mut p = Parser::parse(comment.chars());
    assert_eq!(Some(Token::Comment(String::from(comment))), p.next_token());
    assert_eq!(None, p.next_token());
}
#[test]
fn solo_multiline_comment_test() {
    let comment = "/* hello comment
    */";
    let mut p = Parser::parse(comment.chars());
    assert_eq!(Some(Token::Comment(String::from(comment))), p.next_token());
    assert_eq!(None, p.next_token());
}
#[test]
fn solo_whitespace_test() {
    let whitespace = "     
    
    ";
    let mut p = Parser::parse(whitespace.chars());
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
    let mut p = Parser::parse(src.chars());
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
    assert_eq!(Some(Token::Comment(String::from("// Some comment"))), p.next_token());
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
    assert_eq!(Some(Token::Ident(String::from("something"))), p.next_token());
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
