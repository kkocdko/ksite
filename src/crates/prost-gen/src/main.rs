use std::path::Path;

fn recursion(v: &mut Vec<String>, dir: impl AsRef<Path>) -> std::io::Result<()> {
    let rd = std::fs::read_dir(dir)?;
    for x in rd {
        let de = x?;
        let path = de.path();
        if path.is_dir() {
            recursion(v, path)?;
        } else {
            let path = path.into_os_string().into_string().expect("path error");
            if path.ends_with(".proto") {
                v.push(path);
            }
        }
    }
    Ok(())
}

fn main() {
    let mut v = Vec::<String>::new();
    recursion(&mut v, "sample/pb").unwrap();

    std::env::set_var("OUT_DIR", "target/out");
    // prost_build::compile_protos(&v, &["sample/pb"]).unwrap();
    prost_gen::compile_protos(&v, &["sample/pb"]).unwrap();
    // let src = include_bytes!("goal/a.proto");
    // let out = translate(src.to_vec());
    // std::fs::write("src/goal/a.rs.out", out).unwrap();
    // prost_build::compile_protos(&vec!["src/goal/a.proto"], &["src/goal"]).unwrap();
    // println!("{:#?}", package);
    // loop {
    //     let (kind, body) = next_token(&mut src);
    //     print!("{0:>10}  ", format!("{:?}", kind));
    //     println!("{}", String::from_utf8(body).unwrap());
    //     if kind == TokenKind::End {
    //         break;
    //     }
    // }
}
