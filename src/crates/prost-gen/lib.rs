use std::collections::HashMap;
use std::iter::Peekable;
use std::path::Path;

/// The AST Root
#[derive(Default)]
struct Package {
    name: Vec<u8>,
    syntax: Vec<u8>,
    entries: Vec<Entry>,
}

impl Package {
    fn new(s: &mut TokenStream) -> Self {
        let mut ret = Self::default();
        while let Some(token) = s.peek() {
            match (&token.0, &token.1[..]) {
                (TokenKind::Word, b"syntax") => {
                    s.next().unwrap(); // current token
                    s.next().unwrap(); // eq
                    s.next().unwrap(); // quote
                    ret.syntax = s.next().unwrap().1;
                    // https://developers.google.com/protocol-buffers/docs/proto
                    // https://developers.google.com/protocol-buffers/docs/proto3
                    // assert!(package.syntax == b"proto2"); // TODO
                    s.next().unwrap(); // quote
                    s.next().unwrap(); // semi
                }
                (TokenKind::Word, b"package") => {
                    s.next().unwrap(); // current token
                    ret.name = s.next().unwrap().1;
                    s.next().unwrap(); // semi
                }
                (TokenKind::Word, b"enum") => {
                    ret.entries.push(Entry::Enum(Enum::new(s)));
                }
                (TokenKind::Word, b"message") => {
                    ret.entries.push(Entry::Message(Message::new(s)));
                }
                _ => unreachable!(),
            };
        }
        ret
    }
}

#[derive(Default)]
struct Message {
    name: Vec<u8>,
    entries: Vec<Entry>,
}

impl Message {
    fn new(s: &mut TokenStream) -> Self {
        let mut ret = Self::default();
        assert!(s.next().unwrap().1 == b"message");
        ret.name = s.next().unwrap().1;
        s.next().unwrap(); // '{'
        while let Some(token) = s.peek() {
            match (&token.0, &token.1[..]) {
                (TokenKind::Symbol, b"}") => {
                    s.next().unwrap(); // current token
                    ignore_semi(s);
                    break;
                }
                (TokenKind::Word, b"message") => {
                    ret.entries.push(Entry::Message(Message::new(s)));
                }
                (TokenKind::Word, b"oneof") => {
                    ret.entries.push(Entry::Oneof(Oneof::new(s)));
                }
                (TokenKind::Word, b"enum") => {
                    ret.entries.push(Entry::Enum(Enum::new(s)));
                }
                _ => {
                    ret.entries.push(Entry::MessageField(MessageField::new(s)));
                }
            };
        }
        ret
    }
}

enum Entry {
    Message(Message),
    MessageField(MessageField),
    Oneof(Oneof),
    Enum(Enum),
}

#[derive(Default)]
struct MessageField {
    name: Vec<u8>,
    data_type: Vec<u8>,
    tag: Vec<u8>,
    optional: bool,
    repeated: bool,
}

impl MessageField {
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
                }
                (TokenKind::Word, _) if field.name.is_empty() => {
                    field.name = s.next().unwrap().1;
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

#[derive(Default)]
struct Enum {
    name: Vec<u8>,
    fields: Vec<EnumField>,
}

struct EnumField {
    name: Vec<u8>,
    tag: Vec<u8>,
}

impl Enum {
    fn new(s: &mut TokenStream) -> Self {
        let mut ret = Self::default();
        assert!(s.next().unwrap().1 == b"enum");
        ret.name = s.next().unwrap().1;
        s.next().unwrap(); // '{'
        while let Some(token) = s.peek() {
            match (&token.0, &token.1[..]) {
                (TokenKind::Symbol, b"}") => {
                    s.next().unwrap();
                    ignore_semi(s);
                    break;
                }
                (TokenKind::Word, _) => {
                    let name = s.next().unwrap().1;
                    s.next().unwrap(); // '='
                    let tag = s.next().unwrap().1;
                    s.next().unwrap(); // ';'
                    ret.fields.push(EnumField { name, tag });
                }
                _ => unreachable!(),
            };
        }
        ret
    }
}

#[derive(Default)]
struct Oneof {
    name: Vec<u8>,
    fields: Vec<OneofField>,
}

struct OneofField {
    name: Vec<u8>,
    data_type: Vec<u8>,
    tag: Vec<u8>,
}

impl Oneof {
    fn new(s: &mut TokenStream) -> Self {
        let mut ret = Self::default();
        assert!(s.next().unwrap().1 == b"oneof");
        ret.name = s.next().unwrap().1;
        s.next().unwrap(); // '{'
        while let Some(token) = s.peek() {
            match (&token.0, &token.1[..]) {
                (TokenKind::Symbol, b"}") => {
                    s.next().unwrap();
                    ignore_semi(s);
                    break;
                }
                _ => {
                    let data_type = s.next().unwrap().1;
                    let name = s.next().unwrap().1;
                    s.next().unwrap(); // '='
                    let tag = s.next().unwrap().1;
                    s.next().unwrap(); // ';'
                    ret.fields.push(OneofField {
                        name,
                        data_type,
                        tag,
                    });
                }
            };
        }
        ret
    }
}

fn ignore_semi(s: &mut TokenStream) {
    if let Some((TokenKind::Symbol, b)) = s.peek() {
        if b == b";" {
            s.next().unwrap();
        }
    }
}

enum TokenKind {
    Word,
    Symbol,
    Number,
    End,
}

type TokenStream = Peekable<std::vec::IntoIter<(TokenKind, Vec<u8>)>>;

fn next_token(s: &mut Peekable<std::vec::IntoIter<u8>>) -> (TokenKind, Vec<u8>) {
    fn is_symbol(v: u8) -> bool {
        matches!(v, b'{' | b'}' | b'=' | b';' | b'"')
    }
    let mut ret = Vec::new();
    let mut doing = false;
    let mut kind = TokenKind::End;
    while let Some(&v) = s.peek() {
        match doing {
            false if v == b'/' => {
                // comments
                assert!(s.next().unwrap() == b'/');
                match s.next().unwrap() {
                    b'/' => {
                        for v in s.by_ref() {
                            if v == b'\n' {
                                break;
                            }
                        }
                    }
                    b'*' => {
                        while let Some(v) = s.next() {
                            if v == b'*' {
                                assert!(s.next().unwrap() == b'/');
                                break;
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
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
    use heck::ToUpperCamelCase;
    let mut o: Vec<u8> = std::str::from_utf8(i).unwrap().to_upper_camel_case().into();
    if is_rust_key_word(&o) {
        let mut p = b"r#".to_vec();
        p.append(&mut o);
        p
    } else {
        o
    }
}

fn to_snake(i: &[u8]) -> Vec<u8> {
    use heck::ToSnakeCase;
    let o: Vec<u8> = std::str::from_utf8(i).unwrap().to_snake_case().into();
    if is_rust_key_word(&o) {
        let mut p = b"r#".to_vec();
        p.extend(o);
        p
    } else {
        o
    }
}

fn to_rust_type(i: &[u8]) -> &'static [u8] {
    match i {
        b"double" => b"f64",
        b"float" => b"f32",
        b"int32" => b"i32",
        b"int64" => b"i64",
        b"uint32" => b"u32",
        b"uint64" => b"u64",
        b"sint32" => b"i32",
        b"sint64" => b"i64",
        b"fixed32" => b"u32",
        b"fixed64" => b"u64",
        b"sfixed32" => todo!(),
        b"sfixed64" => todo!(),
        b"bool" => b"bool",
        b"string" => b"::prost::alloc::string::String",
        b"bytes" => b"::prost::alloc::vec::Vec<u8>",
        _ => b"struct",
    }
}

#[rustfmt::skip]
fn is_rust_key_word(i: &[u8]) -> bool {
    1 == match i {
        // https://doc.rust-lang.org/std/index.html#keywords
        b"SelfTy"=>1,b"as"=>1,b"async"=>1,b"await"=>1,b"break"=>1,b"const"=>1,
        b"continue"=>1,b"crate"=>1,b"dyn"=>1,b"else"=>1,b"enum"=>1,b"extern"=>1,
        b"false"=>1,b"fn"=>1,b"for"=>1,b"if"=>1,b"impl"=>1,b"in"=>1,b"let"=>1,
        b"loop"=>1,b"match"=>1,b"mod"=>1,b"move"=>1,b"mut"=>1,b"pub"=>1,b"ref"=>1,
        b"return"=>1,b"self"=>1,b"static"=>1,b"struct"=>1,b"super"=>1,b"trait"=>1,
        b"true"=>1,b"type"=>1,b"union"=>1,b"unsafe"=>1,b"use"=>1,b"where"=>1,b"while"=>1,
        _ => 0,
    }
}

fn indent(n: i32, o: &mut Vec<u8>) {
    for _ in 0..n {
        o.extend(b"    ");
    }
}

fn translate(package: Package) -> Vec<u8> {
    // names context
    let mut cx = HashMap::<Vec<u8>, &'static [u8]>::new();
    let mut o = Vec::<u8>::new();
    fn handle_message(
        depth: i32,
        pbv: i32,
        message: Message,
        o: &mut Vec<u8>,
        cx: &HashMap<Vec<u8>, &'static [u8]>,
    ) {
        let mut cx = cx.clone(); // sub context
        let mut nested = false;
        indent(depth, o);
        o.extend(b"#[derive(Clone, PartialEq, ::prost::Message)]\n");
        indent(depth, o);
        o.extend(b"pub struct ");
        o.extend(to_big_camel(&message.name));
        o.extend(b" {\n");
        for msg_entry in &message.entries {
            if let Entry::Message(i_message) = msg_entry {
                nested = true;
                cx.insert(i_message.name.clone(), b"message");
            }
        }
        for msg_entry in &message.entries {
            match msg_entry {
                Entry::MessageField(field) => {
                    // # From proto doc
                    // For string, bytes, and message fields, optional is compatible with
                    // repeated. Given serialized data of a repeated field as input, clients that
                    // expect this field to be optional will take the last input value if it's a
                    // primitive type field or merge all input elements if it's a message type
                    // field. Note that this is not generally safe for numeric types, including
                    // bools and enums. Repeated fields of numeric types can be serialized in the
                    // packed format, which will not be parsed correctly when an optional field
                    // is expected.
                    let rust_type = to_rust_type(&field.data_type);
                    let _in_ctx = cx.get(&field.data_type);
                    let is_in_ctx = rust_type == b"struct" && _in_ctx.is_some();
                    let is_enum = _in_ctx.map(|v| v == b"enum") == Some(true);
                    let is_optional = {
                        if field.optional && field.repeated {
                            assert!(
                                field.data_type == b"string"
                                    || field.data_type == b"bytes"
                                    || (rust_type == b"struct" && !is_enum)
                            );
                        }
                        field.optional || (rust_type == b"struct" && !field.repeated && !is_enum)
                    };

                    // attr macros
                    indent(depth + 1, o);
                    o.extend(b"#[prost(");
                    if rust_type == b"struct" {
                        if cx.get(&field.data_type).map(|v| v == b"enum") == Some(true) {
                            o.extend(b"enumeration=\"");
                            o.extend(to_big_camel(&field.data_type));
                            o.extend(b"\", ");
                        } else {
                            o.extend(b"message, ");
                        }
                    } else if field.data_type == b"bytes" {
                        o.extend(b"bytes=\"vec\", ");
                    } else {
                        o.extend(&field.data_type);
                        o.extend(b", ");
                    }
                    if is_optional {
                        o.extend(b"optional, ");
                    }
                    if field.repeated {
                        o.extend(b"repeated, ");
                        // # From proto doc
                        // The packed option can be enabled for repeated primitive fields to
                        // enable a more efficient representation on the wire. Rather than
                        // repeatedly writing the tag and type for each element, the entire array
                        // is encoded as a single length-delimited blob. In proto3, only explicit
                        // setting it to false will avoid using packed encoding.
                        if pbv == 2 // proto2
                            && rust_type != b"struct"
                            && field.data_type != b"bytes"
                            && field.data_type != b"string"
                        {
                            o.extend(b"packed=\"false\", ");
                        }
                    }
                    o.extend(b"tag=\"");
                    o.extend(&field.tag);
                    o.extend(b"\", ");
                    if *o.last().unwrap() == b' ' {
                        o.pop();
                        o.pop();
                    }
                    o.extend(b")]\n");

                    // value
                    indent(depth + 1, o);
                    o.extend(b"pub ");
                    o.extend(to_snake(&field.name));
                    o.extend(b": ");
                    let mut depth = 0;
                    // don't use `field.optional`, that only rely on whether `optional` keyword
                    // appeared in source file and AST. see `is_optional`'s define above.
                    if is_optional {
                        o.extend(b"::core::option::Option<");
                        depth += 1;
                    }
                    if field.repeated {
                        o.extend(b"::prost::alloc::vec::Vec<");
                        depth += 1;
                    }
                    if is_enum {
                        o.extend(b"i32");
                    } else if is_in_ctx {
                        o.extend(to_snake(&message.name));
                        o.extend(b"::");
                        o.extend(to_big_camel(&field.data_type));
                    } else if rust_type == b"struct" {
                        o.extend(to_big_camel(&field.data_type))
                    } else {
                        o.extend(rust_type);
                    }
                    for _ in 0..depth {
                        o.extend(b">");
                    }
                    o.extend(b",\n");
                }
                Entry::Oneof(oneof) => {
                    nested = true;

                    // attr macros
                    indent(depth + 1, o);
                    o.extend(b"#[prost(oneof=\"");
                    o.extend(to_snake(&message.name));
                    o.extend(b"::");
                    o.extend(to_big_camel(&oneof.name));
                    o.extend(b"\", tags=\"");
                    for field in &oneof.fields {
                        o.extend(&field.tag);
                        o.extend(b", ");
                    }
                    if *o.last().unwrap() == b' ' {
                        o.pop();
                        o.pop();
                    }
                    o.extend(b"\")]\n");

                    // value
                    indent(depth + 1, o);
                    o.extend(b"pub ");
                    o.extend(to_snake(&oneof.name));
                    o.extend(b": ::core::option::Option<");
                    o.extend(to_snake(&message.name));
                    o.extend(b"::");
                    o.extend(to_big_camel(&oneof.name));
                    o.extend(b">,\n");
                }
                Entry::Message(_) => {}
                _ => unreachable!(),
            }
        }
        indent(depth, o);
        o.extend(b"}\n");
        if nested {
            indent(depth, o);
            o.extend(b"/// Nested message and enum types in `");
            o.extend(&message.name);
            o.extend(b"`.\n");
            indent(depth, o);
            o.extend(b"pub mod ");
            o.extend(to_snake(&message.name));
            o.extend(b" {\n");
            for msg_entry in message.entries {
                match msg_entry {
                    Entry::Message(message) => {
                        handle_message(depth + 1, pbv, message, o, &cx);
                    }
                    Entry::Oneof(oneof) => {
                        indent(depth + 1, o);
                        o.extend(b"#[derive(Clone, PartialEq, ::prost::Oneof)]\n");
                        indent(depth + 1, o);
                        o.extend(b"pub enum ");
                        o.extend(to_big_camel(&oneof.name));
                        o.extend(b" {\n");
                        for field in oneof.fields {
                            // attr macros
                            indent(depth + 2, o);
                            o.extend(b"#[prost(");
                            if to_rust_type(&field.data_type) == b"struct" {
                                o.extend(b"message");
                            } else {
                                o.extend(&field.data_type);
                            }
                            o.extend(b", tag=\"");
                            o.extend(field.tag);
                            o.extend(b"\")]\n");

                            // value
                            indent(depth + 2, o);
                            o.extend(to_big_camel(&field.name));
                            o.extend(b"(");
                            match to_rust_type(&field.data_type) {
                                b"struct" => {
                                    o.extend(b"super::");
                                    o.extend(to_big_camel(&field.data_type));
                                }
                                v => o.extend(v),
                            }
                            o.extend(b"),\n");
                        }
                        indent(depth + 1, o);
                        o.extend(b"}\n");
                    }
                    _ => {}
                }
            }
            indent(depth, o);
            o.extend(b"}\n");
        }
    }
    for pkg_entry in &package.entries {
        match pkg_entry {
            Entry::Enum(enume) => {
                cx.insert(enume.name.clone(), b"enum");
            }
            Entry::Message(_) => {}
            _ => unreachable!(),
        }
    }
    for pkg_entry in package.entries {
        match pkg_entry {
            Entry::Message(message) => {
                let pbv = match &package.syntax[..] {
                    b"proto2" => 2,
                    b"proto3" => 3,
                    _ => panic!("syntax not supported"),
                };
                handle_message(0, pbv, message, &mut o, &cx);
            }
            Entry::Enum(enume) => {
                o.extend(b"#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]\n");
                o.extend(b"#[repr(i32)]\n");
                o.extend(b"pub enum ");
                o.extend(to_big_camel(&enume.name));
                o.extend(b" {\n");
                for field in enume.fields {
                    indent(1, &mut o);
                    o.extend(to_big_camel(&field.name));
                    o.extend(b" = ");
                    o.extend(to_big_camel(&field.tag));
                    o.extend(b",\n");
                }
                o.extend(b"}\n");
            }
            _ => unreachable!(),
        }
    }
    o
}

fn read_to_token_stream(path: impl AsRef<Path>) -> TokenStream {
    let mut src = std::fs::read(path).unwrap().into_iter().peekable();
    let mut tokens = Vec::new();
    loop {
        let token = next_token(&mut src);
        if let TokenKind::End = token.0 {
            break;
        }
        tokens.push(token);
    }
    tokens.into_iter().peekable()
}

pub fn compile_protos(
    protos: &[impl AsRef<Path>],
    _includes: &[impl AsRef<Path>],
) -> std::io::Result<()> {
    let mut outs = HashMap::<Vec<u8>, Vec<u8>>::new();
    // let begin_instant = std::time::Instant::now();
    for path in protos {
        // dbg!(path.as_ref());
        let mut tokens = read_to_token_stream(path);
        let package = Package::new(&mut tokens);
        let name = package.name.clone();
        let mut out = translate(package);
        if let Some(existed) = outs.get_mut(&name) {
            existed.append(&mut out);
        } else {
            outs.insert(name, out);
        }
    }
    // println!("{}", begin_instant.elapsed().as_millis());
    for (name, out) in outs {
        std::fs::write(
            format!(
                "{}/{}.rs",
                std::env::var("OUT_DIR").unwrap(),
                String::from_utf8(name).unwrap()
            ),
            out,
        )?;
    }
    Ok(())
}

fn recurse_dir(v: &mut Vec<String>, dir: impl AsRef<Path>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            recurse_dir(v, path);
        } else if path.extension().map(|v| v == "proto").unwrap() {
            v.push(path.to_str().unwrap().into());
        }
    }
}

pub fn main() {
    const IN_DIR: &str = "target/1_in";
    const OUT_DIR: &str = "target/2_out";
    let mut v = Vec::new();
    recurse_dir(&mut v, IN_DIR);

    std::fs::remove_dir_all(OUT_DIR).ok();
    std::fs::create_dir_all(OUT_DIR).unwrap();
    std::env::set_var("OUT_DIR", OUT_DIR);

    // prost_build_offical::compile_protos(&v, &[IN_DIR]).unwrap();
    compile_protos(&v, &[IN_DIR]).unwrap();
}
