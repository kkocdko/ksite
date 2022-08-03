// https://developers.google.com/protocol-buffers/docs/proto
use std::iter::Peekable;

type TokenStream = Peekable<std::vec::IntoIter<(TokenKind, Vec<u8>)>>;

#[derive(Default, Debug)]
struct Package {
    name: Vec<u8>,
    syntax: Vec<u8>,
    messages: Vec<Message>,
}

impl Package {
    fn new(s: &mut TokenStream) -> Self {
        let mut package = Self::default();
        while let Some(token) = s.peek() {
            match (&token.0, &token.1[..]) {
                (TokenKind::Word, b"syntax") => {
                    s.next().unwrap(); // current token
                    s.next().unwrap(); // eq
                    s.next().unwrap(); // quote
                    package.syntax = s.next().unwrap().1;
                    // assert!(package.syntax == b"proto2"); // TODO
                    s.next().unwrap(); // quote
                    s.next().unwrap(); // semi
                }
                (TokenKind::Word, b"package") => {
                    s.next().unwrap(); // current token
                    package.name = s.next().unwrap().1;
                    s.next().unwrap(); // semi
                }
                (TokenKind::Word, b"message") => {
                    package.messages.push(Message::new(s));
                }
                _ => unreachable!(),
            };
        }
        package
    }
}

#[derive(Default, Debug)]
struct Message {
    name: Vec<u8>,
    fields: Vec<Field>,
}

impl Message {
    fn new(s: &mut TokenStream) -> Self {
        let mut message = Self::default();
        while let Some(token) = s.peek() {
            match (&token.0, &token.1[..]) {
                (TokenKind::Word, b"message") => {
                    s.next().unwrap(); // current token
                    message.name = s.next().unwrap().1; // current token
                    s.next().unwrap(); // '{'
                }
                (TokenKind::Symbol, b"}") => {
                    s.next().unwrap(); // current token
                    break;
                }
                _ => {
                    message.fields.push(Field::new(s));
                }
            };
        }
        message
    }
}

#[derive(Default, Debug)]
struct Field {
    name: Vec<u8>,
    data_type: Vec<u8>,
    tag: Vec<u8>,
    optional: bool,
    repeated: bool,
}

impl Field {
    fn new(s: &mut TokenStream) -> Self {
        let mut field = Self::default();
        while let Some(token) = s.peek() {
            match (&token.0, &token.1[..]) {
                (TokenKind::Word, b"optional") => {
                    s.next().unwrap(); // current token
                    field.optional = true;
                }
                (TokenKind::Word, b"repeated") => {
                    s.next().unwrap(); // current token
                    field.repeated = true;
                }
                (TokenKind::Word, _) if field.data_type.is_empty() => {
                    field.data_type = s.next().unwrap().1;
                    if !is_build_in_type(&field.data_type) && !field.repeated {
                        field.optional = true;
                    }
                }
                (TokenKind::Word, _) if field.name.is_empty() => {
                    field.name = s.next().unwrap().1;
                    if field.name.starts_with(b"opt") {
                        field.optional = true;
                    }
                }
                (TokenKind::Symbol, b"=") => {
                    s.next().unwrap(); // current token
                    field.tag = s.next().unwrap().1;
                    s.next().unwrap(); // semi
                    break;
                }
                _ => unreachable!(),
            };
        }
        field
    }
}

#[derive(PartialEq, Debug)]
enum TokenKind {
    Word,
    Symbol,
    Number,
    End,
}

fn next_token(s: &mut Peekable<std::vec::IntoIter<u8>>) -> (TokenKind, Vec<u8>) {
    fn is_symbol(v: u8) -> bool {
        match v {
            b'{' | b'}' | b'=' | b';' | b'"' => true,
            _ => false,
        }
    }
    let mut ret = Vec::new();
    let mut doing = false;
    let mut kind = TokenKind::End;
    while let Some(&v) = s.peek() {
        match doing {
            false if v.is_ascii_whitespace() => {
                s.next();
            }
            false => {
                kind = match v {
                    _ if is_symbol(v) => TokenKind::Symbol,
                    _ if v.is_ascii_digit() => TokenKind::Number,
                    _ => TokenKind::Word,
                };
                doing = true;
                ret.push(v);
                s.next();
            }
            true => match kind {
                TokenKind::Symbol => break,
                TokenKind::Number if v.is_ascii_digit() => {
                    ret.push(v);
                    s.next();
                }
                TokenKind::Word if !is_symbol(v) && !v.is_ascii_whitespace() => {
                    ret.push(v);
                    s.next();
                }
                _ => break,
            },
        }
    }
    (kind, ret)
}

fn to_big_camel(i: &[u8]) -> Vec<u8> {
    let mut o = Vec::<u8>::new();
    let mut next_is_big = true;
    for &c in i {
        if c.is_ascii_uppercase() {
            if let Some(v) = o.last_mut() {
                *v = v.to_ascii_lowercase();
            }
            o.push(c);
            next_is_big = false;
        } else if c == b'_' {
            next_is_big = true;
        } else if next_is_big {
            if let Some(v) = o.last_mut() {
                *v = v.to_ascii_lowercase();
            }
            o.push(c.to_ascii_uppercase());
            next_is_big = false;
        } else {
            o.push(c);
        }
    }
    o[0] = o[0].to_ascii_uppercase();
    o
}

fn to_snake(i: &[u8]) -> Vec<u8> {
    let mut o = Vec::new();
    for &c in i {
        if c.is_ascii_uppercase() {
            o.push(b'_');
            o.push(c.to_ascii_lowercase());
        } else {
            o.push(c);
        }
    }
    o[0] = o[0].to_ascii_lowercase();
    o
}

fn is_build_in_type(i: &Vec<u8>) -> bool {
    match &i[..] {
        b"double" => true,
        b"float" => true,
        b"int32" => true,
        b"int64" => true,
        b"uint32" => true,
        b"uint64" => true,
        b"sint32" => true,
        b"sint64" => true,
        b"fixed32" => true,
        b"fixed64" => true,
        b"sfixed32" => true,
        b"sfixed64" => true,
        b"bool" => true,
        b"string" => true,
        b"bytes" => true,
        _ => false,
    }
}

fn translate(package: Package) -> Vec<u8> {
    let mut out = Vec::<u8>::new();
    for message in package.messages {
        out.extend(b"#[derive(Clone, PartialEq, ::prost::Message)]\n");
        out.extend(b"pub struct ");
        out.extend(to_big_camel(&message.name));
        out.extend(b" {\n");
        for field in message.fields {
            // attr macro
            out.extend(b"    #[prost(");
            if is_build_in_type(&field.data_type) {
                if field.data_type == b"bytes" {
                    out.extend(b"bytes=\"vec\", ");
                } else {
                    out.extend(&field.data_type);
                    out.extend(b", ");
                }
            } else {
                out.extend(b"message, ");
            }
            if field.optional {
                out.extend(b"optional, ");
            }
            if field.repeated {
                out.extend(b"repeated, ");
                if is_build_in_type(&field.data_type) {
                    out.extend(b"packed=\"false\", ");
                }
            }
            out.extend(b"tag=\"");
            out.extend(field.tag);
            out.extend(b"\", ");
            if *out.last().unwrap() == b' ' {
                out.pop();
                out.pop();
            }
            out.extend(b")]\n");

            // value
            out.extend(b"    pub ");
            out.extend(to_snake(&field.name));
            out.extend(b": ");
            let mut depth = 0;
            if field.optional {
                out.extend(b"::core::option::Option<");
                depth += 1;
            }
            if field.repeated {
                out.extend(b"::prost::alloc::vec::Vec<");
                depth += 1;
            }
            match &field.data_type[..] {
                b"double" => out.extend(b"f64"),
                b"float" => out.extend(b"f32"),
                b"int32" => out.extend(b"i32"),
                b"int64" => out.extend(b"i64"),
                b"uint32" => out.extend(b"u32"),
                b"uint64" => out.extend(b"u64"),
                b"sint32" => out.extend(b"i32"),
                b"sint64" => out.extend(b"i64"),
                b"fixed32" => out.extend(b"u32"),
                b"fixed64" => out.extend(b"u64"),
                b"sfixed32" => todo!(),
                b"sfixed64" => todo!(),
                b"bool" => out.extend(b"bool"),
                b"string" => out.extend(b"::prost::alloc::string::String"),
                b"bytes" => out.extend(b"::prost::alloc::vec::Vec<u8>"),
                v => out.extend(to_big_camel(v)),
            }
            for _ in 0..depth {
                out.extend(b">");
            }
            out.extend(b",\n");
        }
        out.extend(b"}\n");
    }
    out
}

use std::collections::HashMap;
use std::path::Path;

pub fn compile_protos(
    protos: &[impl AsRef<Path>],
    _includes: &[impl AsRef<Path>],
) -> std::io::Result<()> {
    let mut packages = HashMap::<Vec<u8>, Package>::new();
    for path in protos {
        let mut src = std::fs::read(path)?.into_iter().peekable();
        let mut tokens = Vec::new();
        loop {
            let token = next_token(&mut src);
            if token.0 == TokenKind::End {
                break;
            }
            tokens.push(token);
        }
        let mut tokens: TokenStream = tokens.into_iter().peekable();
        let mut package = Package::new(&mut tokens);
        if let Some(existed) = packages.get_mut(&package.name) {
            existed.messages.append(&mut package.messages);
        } else {
            packages.insert(package.name.clone(), package);
        }
    }
    for (name, package) in packages {
        std::fs::write(
            format!(
                "{}/{}",
                std::env::var("OUT_DIR").unwrap(),
                String::from_utf8(name).unwrap()
            ),
            translate(package),
        )?;
    }
    Ok(())
}