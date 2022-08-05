use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The AST Root
struct Package<'a> {
    name: &'a [u8],
    syntax: &'a [u8],
    entries: Vec<Entry<'a>>,
}

impl<'a> Package<'a> {
    fn new(s: &'a TokenStream) -> Self {
        assert!(s.next_v() == b"syntax");
        s.next(); // '='
        s.next(); // '"'
        let syntax = s.next_v();
        s.next(); // '"'
        s.next(); // ';'
        assert!(s.next_v() == b"package");
        let name = s.next_v();
        s.next(); // ';'
        let mut entries = Vec::new();
        while let Some(token) = s.peek() {
            match (&token.0, token.1) {
                (TokenKind::Word, b"enum") => {
                    entries.push(Entry::Enum(Enum::new(s)));
                }
                (TokenKind::Word, b"message") => {
                    entries.push(Entry::Message(Message::new(s)));
                }
                _ => unreachable!(),
            };
        }
        Self {
            name,
            syntax,
            entries,
        }
    }
}

struct Message<'a> {
    name: &'a [u8],
    entries: Vec<Entry<'a>>,
}

impl<'a> Message<'a> {
    fn new(s: &'a TokenStream) -> Self {
        assert!(s.next_v() == b"message");
        let name = s.next_v();
        s.next(); // '{'
        let mut entries = Vec::new();
        while let Some(token) = s.peek() {
            match (&token.0, token.1) {
                (TokenKind::Symbol, b"}") => {
                    s.next();
                    if let Some((_, b";")) = s.peek() {
                        s.next();
                    }
                    break;
                }
                (TokenKind::Word, b"message") => {
                    entries.push(Entry::Message(Message::new(s)));
                }
                (TokenKind::Word, b"oneof") => {
                    entries.push(Entry::Oneof(Oneof::new(s)));
                }
                (TokenKind::Word, b"enum") => {
                    entries.push(Entry::Enum(Enum::new(s)));
                }
                _ => {
                    entries.push(Entry::MessageField(MessageField::new(s)));
                }
            };
        }
        Self { name, entries }
    }
}

enum Entry<'a> {
    Message(Message<'a>),
    MessageField(MessageField<'a>),
    Oneof(Oneof<'a>),
    Enum(Enum<'a>),
}

#[derive(Default)]
struct MessageField<'a> {
    name: &'a [u8],
    data_type: &'a [u8],
    tag: &'a [u8],
    optional: bool,
    repeated: bool,
}

impl<'a> MessageField<'a> {
    fn new(s: &'a TokenStream) -> Self {
        let mut ret = Self::default();
        while let Some(token) = s.peek() {
            match (&token.0, token.1) {
                (TokenKind::Word, b"optional") => {
                    s.next();
                    ret.optional = true;
                }
                (TokenKind::Word, b"repeated") => {
                    s.next();
                    ret.repeated = true;
                }
                (TokenKind::Word, _) if ret.data_type.is_empty() => {
                    ret.data_type = s.next_v();
                }
                (TokenKind::Word, _) if ret.name.is_empty() => {
                    ret.name = s.next_v();
                }
                (TokenKind::Symbol, b"=") => {
                    s.next();
                    ret.tag = s.next_v();
                    s.next(); // ';'
                    break;
                }
                _ => unreachable!(),
            };
        }
        ret
    }
}

struct Enum<'a> {
    name: &'a [u8],
    fields: Vec<EnumField<'a>>,
}

struct EnumField<'a> {
    name: &'a [u8],
    tag: &'a [u8],
}

impl<'a> Enum<'a> {
    fn new(s: &'a TokenStream) -> Self {
        assert!(s.next_v() == b"enum");
        let name = s.next_v();
        s.next(); // '{'
        let mut fields = Vec::new();
        while let Some(token) = s.peek() {
            match (&token.0, token.1) {
                (TokenKind::Symbol, b"}") => {
                    s.next();
                    if let Some((_, b";")) = s.peek() {
                        s.next();
                    }
                    break;
                }
                (TokenKind::Word, _) => {
                    let name = s.next_v();
                    s.next(); // '='
                    let tag = s.next_v();
                    s.next(); // ';'
                    fields.push(EnumField { name, tag });
                }
                _ => unreachable!(),
            };
        }
        Self { name, fields }
    }
}

struct Oneof<'a> {
    name: &'a [u8],
    fields: Vec<OneofField<'a>>,
}

struct OneofField<'a> {
    name: &'a [u8],
    data_type: &'a [u8],
    tag: &'a [u8],
}

impl<'a> Oneof<'a> {
    fn new(s: &'a TokenStream) -> Self {
        assert!(s.next_v() == b"oneof");
        let name = s.next_v();
        s.next(); // '{'
        let mut fields = Vec::new();
        while let Some(token) = s.peek() {
            match (&token.0, token.1) {
                (TokenKind::Symbol, b"}") => {
                    s.next();
                    if let Some((_, b";")) = s.peek() {
                        s.next();
                    }
                    break;
                }
                _ => {
                    let data_type = s.next_v();
                    let name = s.next_v();
                    s.next(); // '='
                    let tag = s.next_v();
                    s.next(); // ';'
                    fields.push(OneofField {
                        name,
                        data_type,
                        tag,
                    });
                }
            };
        }
        Self { name, fields }
    }
}

enum TokenKind {
    Symbol,
    Number,
    Word,
    End,
}

type Token<'a> = (TokenKind, &'a [u8]);

struct TokenStream<'a> {
    tokens: Vec<Token<'a>>,
    idx: Cell<usize>,
}

impl<'a> TokenStream<'a> {
    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.idx.get())
    }

    fn next(&self) -> &Token<'a> {
        let ret = self.peek();
        self.idx.set(self.idx.get() + 1);
        ret.unwrap()
    }

    fn next_v(&self) -> &[u8] {
        self.next().1
    }

    fn new(mut s: &'a [u8]) -> Self {
        let mut tokens = Vec::new();
        loop {
            let token = next_token(&mut s);
            if let TokenKind::End = token.0 {
                break;
            }
            tokens.push(token);
        }
        Self {
            tokens,
            idx: Cell::new(0),
        }
    }
}

fn next_token<'a>(s: &mut &'a [u8]) -> Token<'a> {
    let mut kind = TokenKind::End;
    let mut begun = false;
    let mut range = (0, 0);
    let found = |from, c| from + s[from..].iter().position(|&v| v == c).unwrap();
    let is_symbol = |c| matches!(c, b'{' | b'}' | b'=' | b';' | b'"');
    while let Some(&v) = s.get(range.1) {
        match begun {
            false if v == b'/' => {
                // comments
                range.1 += 1;
                match s[range.1] {
                    b'/' => range.1 = found(range.1 + 1, b'\n'),
                    b'*' => loop {
                        range.1 = found(range.1 + 1, b'*');
                        if s[range.1 + 1] == b'/' {
                            range.1 += 2;
                            break;
                        }
                    },
                    _ => unreachable!(),
                }
            }
            false if v.is_ascii_whitespace() => {
                range.1 += 1;
            }
            false => {
                kind = match v {
                    _ if is_symbol(v) => TokenKind::Symbol,
                    _ if v.is_ascii_digit() => TokenKind::Number,
                    _ => TokenKind::Word,
                };
                range.0 = range.1;
                begun = true;
            }
            true => match kind {
                TokenKind::Symbol => {
                    range.1 += 1;
                    break;
                }
                TokenKind::Number if v.is_ascii_digit() => {
                    range.1 += 1;
                }
                TokenKind::Word if !is_symbol(v) && !v.is_ascii_whitespace() => {
                    range.1 += 1;
                }
                _ => break,
            },
        }
    }
    let ret = (kind, &s[range.0..range.1]);
    *s = &s[range.1..];
    ret
}

// performance hotpoint here
fn to_any_case(s: &[u8]) -> Vec<Vec<u8>> {
    let mut idx: usize = 0;
    let mut parts = Vec::new();
    while let Some(&f) = s.get(idx) {
        idx += 1;
        let mut part = vec![f];
        let (mut r_u, mut r_d) = (f.is_ascii_uppercase() as u8, f.is_ascii_uppercase() as u8);
        while let Some(&c) = s.get(idx) {
            let (c_u, c_d) = (c.is_ascii_uppercase() as u8, c.is_ascii_digit() as u8);
            match (r_u, r_d, c_u, c_d) {
                _ if c == b'_' => {
                    idx += 1;
                    break;
                }
                (_, _, 1, _) if idx + 1 < s.len() && s[idx + 1].is_ascii_lowercase() => {
                    break;
                }
                (1, _, 0, 0) | (1, _, 1, 0) | (0, 0, 0, 0) | (_, 0, 0, 1) | (_, 1, 0, _) => {
                    idx += 1;
                    part.push(c);
                }
                (0, _, 1, 0) => break,
                v => panic!("illegal state {:?}", v),
            }
            if c_d != 1 {
                r_u = c_u;
            }
            r_d = c_d;
        }
        parts.push(part);
    }
    parts
}

fn make_legal_rust_ident(i: &mut Vec<u8>) {
    #[rustfmt::skip]
    #[inline]
    // https://doc.rust-lang.org/std/index.html#keywords
    fn is_rust_key_word(i: &[u8]) -> bool { matches!(i,
        b"Self"|b"as"|b"async"|b"await"|b"break"|b"const"|b"continue"|b"crate"|b"dyn"|
        b"else"|b"enum"|b"extern"|b"false"|b"fn"|b"for"|b"if"|b"impl"|b"in"|b"let"|b"loop"|
        b"match"|b"mod"|b"move"|b"mut"|b"pub"|b"ref"|b"return"|b"self"|b"static"|b"struct"|
        b"super"|b"trait"|b"true"|b"type"|b"union"|b"unsafe"|b"use"|b"where"|b"while")
    }
    if is_rust_key_word(i) {
        let mut p = b"r#".to_vec();
        p.append(i);
        *i = p;
    }
}

// TODO: cache using unsafe?
// static CASE_CACHE: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

fn to_big_camel(i: &[u8]) -> Vec<u8> {
    let mut o = Vec::new();
    for mut part in to_any_case(i) {
        part.make_ascii_lowercase();
        part[0].make_ascii_uppercase();
        o.append(&mut part);
    }
    make_legal_rust_ident(&mut o);
    o
}

fn to_snake(i: &[u8]) -> Vec<u8> {
    let mut o = Vec::new();
    for mut part in to_any_case(i) {
        part.make_ascii_lowercase();
        o.append(&mut part);
        o.push(b'_');
    }
    if o.ends_with(b"_") {
        o.pop();
    }
    make_legal_rust_ident(&mut o);
    o
}

#[cfg(target_feature = "tests")]
fn test_case() {
    fn test_once(i: &str) {
        // to_any_case(i.as_bytes())
        //     .into_iter()
        //     .map(|v| String::from_utf8(v).unwrap())
        //     .for_each(|v| {
        //         println!("{v}");
        //     });

        use heck::{ToSnakeCase, ToUpperCamelCase};

        let expect = i.to_upper_camel_case();
        let ans = String::from_utf8(to_big_camel(i.as_bytes())).unwrap();
        // dbg!(&ans);
        assert_eq!(expect, ans, "to_big_camel wrong");

        let expect = i.to_snake_case();
        let ans = String::from_utf8(to_snake(i.as_bytes())).unwrap();
        assert_eq!(expect, ans, "to_snake wrong");
    }
    test_once("ABC4Defg");
    test_once("abc4defg");
    test_once("ABC4DEFG");
    test_once("abC4dEfg");
    test_once("abC4d_efg");
    test_once("ab3efg");
    test_once("ab3Efg");
    test_once("abcDA3Eg");
    test_once("abcDA3EFg");
    test_once("abcDEFg");
    test_once("c2CReadReport");
}

fn to_rust_type(i: &[u8]) -> &'static [u8] {
    match i {
        b"bool" => b"bool",
        b"float" => b"f32",
        b"double" => b"f64",
        b"int32" | b"sint32" | b"sfixed32" => b"i32",
        b"int64" | b"sint64" | b"sfixed64" => b"i64",
        b"uint32" | b"fixed32" => b"u32",
        b"uint64" | b"fixed64" => b"u64",
        b"string" => b"::prost::alloc::string::String",
        b"bytes" => b"::prost::alloc::vec::Vec<u8>",
        _ => b"struct",
    }
}

/// Translate AST to Rust source code.
fn translate(package: Package) -> Vec<u8> {
    // https://developers.google.com/protocol-buffers/docs/proto
    // https://developers.google.com/protocol-buffers/docs/proto3

    type Context<'a> = HashMap<&'a [u8], (&'static [u8], i32)>; // <name, (type, depth)>
    let mut ctx = Context::new(); // names context
    let mut o = Vec::<u8>::new();

    fn indent(n: i32, o: &mut Vec<u8>) {
        for _ in 0..n {
            o.extend(b"    ");
        }
    }

    fn mod_path(sub: i32, cur_mod: &[u8], o: &mut Vec<u8>) {
        match sub {
            0 => {
                o.append(&mut to_snake(cur_mod));
                o.extend(b"::");
            }
            1 => {}
            2..=i32::MAX => {
                for _ in 0..(sub - 1) {
                    o.extend(b"super::");
                }
            }
            _ => unreachable!(),
        }
    }

    fn handle_message(message: &Message, pbv: &[u8], ctx: &Context, depth: i32, o: &mut Vec<u8>) {
        let mut cx = ctx.clone(); // sub context
        let mut has_nested = false;
        indent(depth, o);
        o.extend(b"#[derive(Clone, PartialEq, ::prost::Message)]\n");
        indent(depth, o);
        o.extend(b"pub struct ");
        o.append(&mut to_big_camel(message.name));
        o.extend(b" {\n");
        for entry in &message.entries {
            match entry {
                Entry::Enum(Enum { name, .. }) => {
                    has_nested = true;
                    cx.insert(name, (b"enum", depth));
                }
                Entry::Message(Message { name, .. }) => {
                    has_nested = true;
                    cx.insert(name, (b"message", depth));
                }
                Entry::Oneof(_) => {
                    has_nested = true;
                    // oneof is anonymous
                }
                Entry::MessageField(_) => {}
            }
        }
        for entry in &message.entries {
            match entry {
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
                    let rust_type = to_rust_type(field.data_type);
                    let _in_ctx = cx.get(field.data_type);
                    let is_in_ctx = rust_type == b"struct" && _in_ctx.is_some();
                    let is_enum = _in_ctx.map(|v| v.0 == b"enum") == Some(true);
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
                        if cx.get(field.data_type).map(|v| v.0 == b"enum") == Some(true) {
                            o.extend(b"enumeration=\"");
                            mod_path(depth - cx[field.data_type].1, message.name, o);
                            o.append(&mut to_big_camel(field.data_type));
                            o.extend(b"\", ");
                        } else {
                            o.extend(b"message, ");
                        }
                    } else if field.data_type == b"bytes" {
                        o.extend(b"bytes=\"vec\", ");
                    } else {
                        o.extend(field.data_type);
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
                        if pbv == b"proto2"
                            && rust_type != b"struct"
                            && field.data_type != b"bytes"
                            && field.data_type != b"string"
                        {
                            o.extend(b"packed=\"false\", ");
                        }
                    }
                    o.extend(b"tag=\"");
                    o.extend(field.tag);
                    o.extend(b"\", ");
                    if *o.last().unwrap() == b' ' {
                        o.pop();
                        o.pop();
                    }
                    o.extend(b")]\n");

                    // value
                    indent(depth + 1, o);
                    o.extend(b"pub ");
                    o.append(&mut to_snake(field.name));
                    o.extend(b": ");
                    let mut field_depth = 0;
                    // don't use `field.optional`, that only rely on whether `optional` keyword
                    // appeared in source file and AST. see `is_optional`'s define above.
                    if is_optional {
                        o.extend(b"::core::option::Option<");
                        field_depth += 1;
                    }
                    if field.repeated {
                        o.extend(b"::prost::alloc::vec::Vec<");
                        field_depth += 1;
                    }
                    if is_enum {
                        o.extend(b"i32");
                    } else if is_in_ctx {
                        mod_path(depth - cx[field.data_type].1, message.name, o);
                        o.append(&mut to_big_camel(field.data_type));
                    } else if rust_type == b"struct" {
                        o.append(&mut to_big_camel(field.data_type))
                    } else {
                        o.extend(rust_type);
                    }
                    for _ in 0..field_depth {
                        o.extend(b">");
                    }
                    o.extend(b",\n");
                }
                Entry::Oneof(oneof) => {
                    // attr macros
                    indent(depth + 1, o);
                    o.extend(b"#[prost(oneof=\"");
                    o.append(&mut to_snake(message.name));
                    o.extend(b"::");
                    o.append(&mut to_big_camel(oneof.name));
                    o.extend(b"\", tags=\"");
                    for field in &oneof.fields {
                        o.extend(field.tag);
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
                    o.append(&mut to_snake(oneof.name));
                    o.extend(b": ::core::option::Option<");
                    o.append(&mut to_snake(message.name));
                    o.extend(b"::");
                    o.append(&mut to_big_camel(oneof.name));
                    o.extend(b">,\n");
                }
                Entry::Message(_) => {}
                Entry::Enum(_) => {}
            }
        }
        indent(depth, o);
        o.extend(b"}\n");
        if has_nested {
            indent(depth, o);
            o.extend(b"/// Nested message and enum types in `");
            o.extend(message.name);
            o.extend(b"`.\n");
            indent(depth, o);
            o.extend(b"pub mod ");
            o.append(&mut to_snake(message.name));
            o.extend(b" {\n");
            for entry in &message.entries {
                match entry {
                    Entry::Enum(v) => handle_enum(v, depth + 1, o),
                    Entry::Message(v) => handle_message(v, pbv, &cx, depth + 1, o),
                    Entry::Oneof(oneof) => {
                        indent(depth + 1, o);
                        o.extend(b"#[derive(Clone, PartialEq, ::prost::Oneof)]\n");
                        indent(depth + 1, o);
                        o.extend(b"pub enum ");
                        o.append(&mut to_big_camel(oneof.name));
                        o.extend(b" {\n");
                        for field in &oneof.fields {
                            // attr macros
                            indent(depth + 2, o);
                            o.extend(b"#[prost(");
                            if to_rust_type(field.data_type) == b"struct" {
                                o.extend(b"message");
                            } else {
                                o.extend(field.data_type);
                            }
                            o.extend(b", tag=\"");
                            o.extend(field.tag);
                            o.extend(b"\")]\n");

                            // value
                            indent(depth + 2, o);
                            o.append(&mut to_big_camel(field.name));
                            o.extend(b"(");
                            match to_rust_type(field.data_type) {
                                b"struct" => {
                                    o.extend(b"super::");
                                    o.append(&mut to_big_camel(field.data_type));
                                }
                                v => o.extend(v),
                            }
                            o.extend(b"),\n");
                        }
                        indent(depth + 1, o);
                        o.extend(b"}\n");
                    }
                    Entry::MessageField(_) => {}
                }
            }
            indent(depth, o);
            o.extend(b"}\n");
        }
    }

    fn handle_enum(enume: &Enum, mut depth: i32, o: &mut Vec<u8>) {
        indent(depth, o);
        o.extend(b"#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]\n");
        indent(depth, o);
        o.extend(b"#[repr(i32)]\n");
        indent(depth, o);
        o.extend(b"pub enum ");
        o.append(&mut to_big_camel(enume.name));
        o.extend(b" {\n");
        depth += 1;
        for field in &enume.fields {
            indent(depth, o);
            o.append(&mut to_big_camel(field.name));
            o.extend(b" = ");
            o.append(&mut to_big_camel(field.tag));
            o.extend(b",\n");
        }
        depth -= 1;
        indent(depth, o);
        o.extend(b"}\n");
    }

    let depth = -1;
    for entry in &package.entries {
        match entry {
            Entry::Enum(v) => (ctx.insert(v.name, (b"enum", depth)), ()).1, // aha
            Entry::Message(_) => {}
            _ => unreachable!(),
        }
    }
    for entry in &package.entries {
        match entry {
            Entry::Enum(v) => handle_enum(v, depth + 1, &mut o),
            Entry::Message(v) => handle_message(v, package.syntax, &ctx, depth + 1, &mut o),
            _ => unreachable!(),
        }
    }
    o
}

/// Compile `.proto` files into Rust files during a Cargo build.
pub fn compile_protos(
    protos: &[impl AsRef<Path>],
    _includes: &[impl AsRef<Path>],
) -> std::io::Result<()> {
    let mut outs = HashMap::<Vec<u8>, Vec<u8>>::new();
    // let begin_instant = std::time::Instant::now();
    for path in protos {
        // dbg!(path.as_ref());
        let src = std::fs::read(path)?;
        let token_stream = TokenStream::new(&src);
        let package = Package::new(&token_stream);
        let name = package.name.to_vec();
        let mut out = translate(package);
        if let Some(existed) = outs.get_mut(&name) {
            existed.append(&mut out);
        } else {
            outs.insert(name, out);
        }
    }
    // println!("{} ms", begin_instant.elapsed().as_micros() as f64 / 1000.0);
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

/// # DON'T USE THIS IN LIB TARGET!
pub fn main() {
    // return test_case();

    const IN_DIR: &str = "target/1_in";
    const OUT_DIR: &str = "target/2_out";
    let mut v = Vec::new();
    recurse_dir(&mut v, IN_DIR);
    fn recurse_dir(v: &mut Vec<PathBuf>, dir: impl AsRef<Path>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                recurse_dir(v, path);
            } else if path.extension().map(|v| v == "proto").unwrap() {
                v.push(path);
            }
        }
    }

    std::fs::remove_dir_all(OUT_DIR).ok();
    std::fs::create_dir_all(OUT_DIR).unwrap();
    std::env::set_var("OUT_DIR", OUT_DIR);

    // prost_build_offical::compile_protos(&v, &[IN_DIR]).unwrap();
    compile_protos(&v, &[IN_DIR]).unwrap();
}
