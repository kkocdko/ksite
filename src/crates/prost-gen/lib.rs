use std::cell::Cell;
// use std::collections::BTreeMap as HashMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The Root of AST.
struct Package<'a> {
    name: &'a [u8],
    syntax: &'a [u8],
    imports: Vec<&'a [u8]>,
    entries: Vec<Entry<'a>>,
}

impl<'a> Package<'a> {
    fn new(s: &'a TokenStream) -> Self {
        let mut ret = Self {
            name: b"",
            syntax: b"",
            imports: Vec::new(),
            entries: Vec::new(),
        };
        assert!(s.next().1 == b"syntax");
        s.next(); // '='
        s.next(); // '"'
        ret.syntax = s.next().1;
        s.next(); // '"'
        s.next(); // ';'
        assert!(s.next().1 == b"package");
        ret.name = s.next().1;
        s.next(); // ';'
        while let Some(token) = s.peek() {
            match (&token.0, token.1) {
                (TokenKind::Word, b"import") => {
                    s.next();
                    s.next(); // '"'
                    ret.imports.push(s.next().1);
                    s.next(); // '"'
                    s.next(); // ';'
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

enum Entry<'a> {
    Message(Message<'a>),
    MessageField(MessageField<'a>),
    Enum(Enum<'a>),
    Oneof(Oneof<'a>),
}

struct Message<'a> {
    name: &'a [u8],
    entries: Vec<Entry<'a>>,
}

impl<'a> Message<'a> {
    fn new(s: &'a TokenStream) -> Self {
        let mut ret = Self {
            name: b"",
            entries: Vec::new(),
        };
        assert!(s.next().1 == b"message");
        ret.name = s.next().1;
        s.next(); // '{'
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

struct MessageField<'a> {
    name: &'a [u8],
    data_type: &'a [u8],
    tag: &'a [u8],
    optional: bool,
    repeated: bool,
}

impl<'a> MessageField<'a> {
    fn new(s: &'a TokenStream) -> Self {
        let mut ret = Self {
            name: b"",
            data_type: b"",
            tag: b"",
            optional: false,
            repeated: false,
        };
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
                    ret.data_type = s.next().1;
                }
                (TokenKind::Word, _) if ret.name.is_empty() => {
                    ret.name = s.next().1;
                }
                (TokenKind::Symbol, b"=") => {
                    s.next();
                    ret.tag = s.next().1;
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
        let mut ret = Self {
            name: b"",
            fields: Vec::new(),
        };
        assert!(s.next().1 == b"enum");
        ret.name = s.next().1;
        s.next(); // '{'
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
                    let name = s.next().1;
                    s.next(); // '='
                    let tag = s.next().1;
                    s.next(); // ';'
                    ret.fields.push(EnumField { name, tag });
                }
                _ => unreachable!(),
            };
        }
        ret
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
        let mut ret = Self {
            name: b"",
            fields: Vec::new(),
        };
        assert!(s.next().1 == b"oneof");
        ret.name = s.next().1;
        s.next(); // '{'
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
                    let data_type = s.next().1;
                    let name = s.next().1;
                    s.next(); // '='
                    let tag = s.next().1;
                    s.next(); // ';'
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

enum TokenKind {
    Comment,
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
        let idx = self.idx.get();
        self.idx.set(idx + 1);
        self.tokens.get(idx).unwrap()
    }

    /// Create `TokenStream` from proto file data.
    fn new(mut s: &'a [u8]) -> Self {
        let mut ret = Self {
            tokens: Vec::new(),
            idx: Cell::new(0),
        };
        loop {
            match next_token(&mut s) {
                (TokenKind::End, _) => break,
                (TokenKind::Comment, _) => continue, // TODO: support comments
                token => ret.tokens.push(token),
            }
        }
        ret
    }
}

fn next_token<'a>(s: &mut &'a [u8]) -> Token<'a> {
    // https://github.com/rust-lang/rust/issues/94035
    // *s = s.trim_ascii_start();
    while matches!(s.first(), Some(c) if c.is_ascii_whitespace()) {
        *s = &s[1..];
    }
    let (kind, len) = match s.first() {
        Some(b'/') => {
            let mut iter = s.iter();
            let r = match s[1] {
                b'/' => iter.take_while(|&&c| c != b'\n').count(),
                b'*' => {
                    iter.next();
                    while *iter.next().unwrap() != b'/' {
                        iter.find(|&&c| c == b'*');
                    }
                    s.len() - iter.count()
                }
                _ => unreachable!(),
            };
            (TokenKind::Comment, r)
        }
        Some(b'{' | b'}' | b'=' | b';' | b'"') => (TokenKind::Symbol, 1),
        Some(c) if c.is_ascii_digit() => {
            let r = (s.iter()).take_while(|&&c| c.is_ascii_digit());
            (TokenKind::Number, r.count())
        }
        Some(_) => {
            let r = (s.iter()).take_while(|&&c| c.is_ascii_alphanumeric() || c == b'_');
            (TokenKind::Word, r.count())
        }
        None => return (TokenKind::End, b""),
    };
    assert!(len != 0);
    let body;
    (body, *s) = s.split_at(len);
    (kind, body)
}

/// Split an identifier into sections for case transforms later.
fn to_any_case(s: &[u8]) -> Vec<&[u8]> {
    // return vec![s];
    let mut parts = Vec::<&[u8]>::with_capacity(8);
    let mut range = (0, 0);
    let kindof = |c| match c {
        b'A'..=b'Z' => 1, //  1 = uppercase
        b'0'..=b'9' => 0, //  0 = digit
        _ => -1i8,        // -1 = lowercase or underline
    };
    'label: while let Some(&first) = s.get(range.1) {
        range.1 += 1;
        let mut kind_recent = kindof(first); // TODO: prefix with '_' ?
        while let Some(&current) = s.get(range.1) {
            let kind_current = kindof(current);
            match (kind_recent, kind_current) {
                _ if current == b'_' => {
                    parts.push(&s[range.0..range.1]);
                    range.1 += 1; // skip current char
                    range.0 = range.1;
                    continue 'label;
                }
                (-1, 1) => break,
                (1, 1) if matches!(s.get(range.1 + 1), Some(b'a'..=b'z')) => break,
                (_, 0 | -1) | (1 | 0, 1) => range.1 += 1,
                _ => unreachable!(),
                // v => panic!("illegal state {:?}", v),
            }
            if kind_current != 0 {
                kind_recent = kind_current;
            }
        }
        parts.push(&s[range.0..range.1]);
        range.0 = range.1;
    }
    parts
}

#[cfg(target_feature = "tests")]
fn test_to_any_case() {
    fn test_once(i: &[u8]) {
        // for part in to_any_case(i) {
        //     println!("part = {}", std::str::from_utf8(part).unwrap());
        // }

        use heck::{ToSnakeCase, ToUpperCamelCase};
        let s = std::str::from_utf8(i).unwrap();

        let mut o = Vec::new();
        push_big_camel(i, &mut o);
        let ans = String::from_utf8(o).unwrap();
        let expect = s.to_upper_camel_case();
        assert_eq!(expect, ans, "push_big_camel({s}) wrong");

        let mut o = Vec::new();
        push_snake(i, &mut o);
        let ans = String::from_utf8(o).unwrap();
        let expect = s.to_snake_case();
        assert_eq!(expect, ans, "push_snake({s}) wrong");
    }
    const CHARS: [u8; 4] = [b'A', b'0', b'a', b'_'];
    const LEN: usize = 9;
    fn recurse_test(i: usize, chars: &mut [u8; LEN]) {
        match i {
            1 | LEN if chars[i - 1] == b'_' => return, // prefix or suffix '_'
            2..=LEN if chars[i - 2..i] == *b"__" => return, // has "__"
            LEN => {
                test_once(chars);
                return;
            }
            _ => {}
        }
        for c in CHARS {
            chars[i] = c;
            recurse_test(i + 1, chars);
        }
    }
    recurse_test(0, &mut [0; LEN]);
}

#[rustfmt::skip]
fn is_rust_keyword(i: &[u8]) -> bool { matches!(i,
    // https://doc.rust-lang.org/std/index.html#keywords
    b"Self"|b"as"|b"async"|b"await"|b"break"|b"const"|b"continue"|b"crate"|b"dyn"|
    b"else"|b"enum"|b"extern"|b"false"|b"fn"|b"for"|b"if"|b"impl"|b"in"|b"let"|b"loop"|
    b"match"|b"mod"|b"move"|b"mut"|b"pub"|b"ref"|b"return"|b"self"|b"static"|b"struct"|
    b"super"|b"trait"|b"true"|b"type"|b"union"|b"unsafe"|b"use"|b"where"|b"while")
}

/// Push the identifier as big camel case.
fn push_big_camel(i: &[u8], o: &mut Vec<u8>) {
    let parts = to_any_case(i);
    let parts_len = parts.len();
    if parts_len == 1 && is_rust_keyword(parts[0]) {
        o.extend(b"r#");
    }
    for part in parts {
        o.push(part[0].to_ascii_uppercase());
        for c in &part[1..] {
            o.push(c.to_ascii_lowercase());
        }
    }
}

/// Push the identifier as snake case.
fn push_snake(i: &[u8], o: &mut Vec<u8>) {
    let parts = to_any_case(i);
    let parts_len = parts.len();
    if parts_len == 1 && is_rust_keyword(parts[0]) {
        o.extend(b"r#");
    }
    for part in parts {
        for c in part {
            o.push(c.to_ascii_lowercase());
        }
        o.push(b'_');
    }
    o.pop();
}

fn push_indent(n: i32, o: &mut Vec<u8>) {
    for _ in 0..n {
        o.extend(b"    ");
    }
}

/// Push the type name with mod path prefix.
fn push_type(depth_diff: i32, message: &Message, field: &MessageField, o: &mut Vec<u8>) {
    let mod_name = message.name;
    let data_type = field.data_type;
    // is imported from other pkg?
    if let Some(pos) = data_type.iter().position(|&c| c == b'.') {
        o.extend(b"super::"); // it's terrible! only supports 1 dot
        push_snake(&data_type[..pos], o);
        o.extend(b"::");
        push_big_camel(&data_type[pos + 1..], o);
        return;
    }
    match depth_diff {
        0 => {
            push_snake(mod_name, o);
            o.extend(b"::");
        }
        1 => {}
        2..=i32::MAX => {
            for _ in 0..(depth_diff - 1) {
                o.extend(b"super::");
            }
        }
        _ => unreachable!(),
    }
    push_big_camel(data_type, o);
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
        _ => b"custom",
    }
}

/// Translate AST to Rust source code.
fn translate(package: &Package) -> Vec<u8> {
    // https://developers.google.com/protocol-buffers/docs/proto
    // https://developers.google.com/protocol-buffers/docs/proto3

    type Context<'a> = HashMap<&'a [u8], (&'static [u8], i32)>; // <name, (type, depth)>
    let mut ctx = Context::new(); // names context
    let mut o = Vec::with_capacity(2048);

    fn handle_message(message: &Message, pbv: &[u8], ctx: &Context, depth: i32, o: &mut Vec<u8>) {
        let mut ctx = ctx.clone(); // sub context
        let mut has_nested = false;
        push_indent(depth, o);
        o.extend(b"#[derive(Clone, PartialEq, ::prost::Message)]\n");
        push_indent(depth, o);
        o.extend(b"pub struct ");
        push_big_camel(message.name, o);
        o.extend(b" {\n");
        for entry in &message.entries {
            match entry {
                Entry::Enum(inner) => {
                    has_nested = true;
                    ctx.insert(inner.name, (b"enum", depth));
                }
                Entry::Message(inner) => {
                    has_nested = true;
                    ctx.insert(inner.name, (b"message", depth));
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
                    let is_in_ctx = ctx.get(field.data_type).is_some();
                    let is_enum = is_in_ctx && ctx[field.data_type].0 == b"enum";
                    let is_optional = {
                        let is_prime = is_enum || rust_type != b"custom";
                        if field.optional && field.repeated {
                            assert!(matches!(field.data_type, b"string" | b"bytes") || !is_prime);
                        }
                        field.optional || (!is_prime && !field.repeated)
                    };

                    push_indent(depth + 1, o);
                    o.extend(b"#[prost(");
                    if is_enum {
                        o.extend(b"enumeration=\"");
                        push_type(depth - ctx[field.data_type].1, message, field, o);
                        o.extend(b"\", ");
                    } else if rust_type == b"custom" {
                        o.extend(b"message, ");
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
                    }
                    // # From proto doc
                    // The packed option can be enabled for repeated primitive fields to
                    // enable a more efficient representation on the wire. Rather than
                    // repeatedly writing the tag and type for each element, the entire array
                    // is encoded as a single length-delimited blob. In proto3, only explicit
                    // setting it to false will avoid using packed encoding.
                    if pbv == b"proto2"
                        && field.repeated
                        && rust_type != b"custom"
                        && field.data_type != b"bytes"
                        && field.data_type != b"string"
                    {
                        o.extend(b"packed=\"false\", ");
                    }
                    o.extend(b"tag=\"");
                    o.extend(field.tag);
                    o.extend(b"\", ");
                    if *o.last().unwrap() == b' ' {
                        o.pop();
                        o.pop();
                    }
                    o.extend(b")]\n");
                    push_indent(depth + 1, o);
                    o.extend(b"pub ");
                    push_snake(field.name, o);
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
                    } else if rust_type == b"custom" {
                        let depth_diff = match is_in_ctx {
                            true => depth - ctx[field.data_type].1,
                            false => 1, // is in included files
                        };
                        push_type(depth_diff, message, field, o);
                    } else {
                        o.extend(rust_type);
                    }
                    for _ in 0..field_depth {
                        o.extend(b">");
                    }
                    o.extend(b",\n");
                }
                Entry::Oneof(oneof) => {
                    push_indent(depth + 1, o);
                    o.extend(b"#[prost(oneof=\"");
                    push_snake(message.name, o);
                    o.extend(b"::");
                    push_big_camel(oneof.name, o);
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
                    push_indent(depth + 1, o);
                    o.extend(b"pub ");
                    push_snake(oneof.name, o);
                    o.extend(b": ::core::option::Option<");
                    push_snake(message.name, o);
                    o.extend(b"::");
                    push_big_camel(oneof.name, o);
                    o.extend(b">,\n");
                }
                Entry::Message(_) => {}
                Entry::Enum(_) => {}
            }
        }
        push_indent(depth, o);
        o.extend(b"}\n");
        if !has_nested {
            return;
        }
        push_indent(depth, o);
        o.extend(b"/// Nested message and enum types in `");
        o.extend(message.name);
        o.extend(b"`.\n");
        push_indent(depth, o);
        o.extend(b"pub mod ");
        push_snake(message.name, o);
        o.extend(b" {\n");
        for entry in &message.entries {
            match entry {
                Entry::Enum(inner) => {
                    handle_enum(inner, depth + 1, o);
                }
                Entry::Message(inner) => {
                    handle_message(inner, pbv, &ctx, depth + 1, o);
                }
                Entry::Oneof(oneof) => {
                    push_indent(depth + 1, o);
                    o.extend(b"#[derive(Clone, PartialEq, ::prost::Oneof)]\n");
                    push_indent(depth + 1, o);
                    o.extend(b"pub enum ");
                    push_big_camel(oneof.name, o);
                    o.extend(b" {\n");
                    for field in &oneof.fields {
                        push_indent(depth + 2, o);
                        o.extend(b"#[prost(");
                        if to_rust_type(field.data_type) == b"custom" {
                            o.extend(b"message");
                        } else {
                            o.extend(field.data_type);
                        }
                        o.extend(b", tag=\"");
                        o.extend(field.tag);
                        o.extend(b"\")]\n");
                        push_indent(depth + 2, o);
                        push_big_camel(field.name, o);
                        o.extend(b"(");
                        match to_rust_type(field.data_type) {
                            b"custom" => {
                                o.extend(b"super::");
                                push_big_camel(field.data_type, o);
                            }
                            v => o.extend(v),
                        }
                        o.extend(b"),\n");
                    }
                    push_indent(depth + 1, o);
                    o.extend(b"}\n");
                }
                Entry::MessageField(_) => {}
            }
        }
        push_indent(depth, o);
        o.extend(b"}\n");
    }

    fn handle_enum(enume: &Enum, mut depth: i32, o: &mut Vec<u8>) {
        push_indent(depth, o);
        o.extend(b"#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]\n");
        push_indent(depth, o);
        o.extend(b"#[repr(i32)]\n");
        push_indent(depth, o);
        o.extend(b"pub enum ");
        push_big_camel(enume.name, o);
        o.extend(b" {\n");
        depth += 1;
        for field in &enume.fields {
            push_indent(depth, o);
            push_big_camel(field.name, o);
            o.extend(b" = ");
            push_big_camel(field.tag, o);
            o.extend(b",\n");
        }
        depth -= 1;
        push_indent(depth, o);
        o.extend(b"}\n");
    }

    let depth = -1;
    for entry in &package.entries {
        match entry {
            Entry::Enum(inner) => {
                ctx.insert(inner.name, (b"enum", depth));
            }
            Entry::Message(_) => {}
            _ => unreachable!(),
        }
    }
    for entry in &package.entries {
        match entry {
            Entry::Enum(inner) => {
                handle_enum(inner, depth + 1, &mut o);
            }
            Entry::Message(inner) => {
                handle_message(inner, package.syntax, &ctx, depth + 1, &mut o);
            }
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
        let mut out = translate(&package);
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
    // return test_to_any_case();

    const IN_DIR: &str = "./ricq/ricq-core/src/pb";
    const OUT_DIR: &str = "./target/pb-out";
    let mut v = Vec::new();
    recurse_dir(&mut v, IN_DIR);
    fn recurse_dir(v: &mut Vec<PathBuf>, dir: impl AsRef<Path>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                recurse_dir(v, path);
            } else if matches!(path.extension(), Some(v) if v == "proto") {
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
