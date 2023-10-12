use std::path::Path;

fn main() -> Result<(), anyhow::Error> {
    let p1 = Path::new("aa/b/c/d.tar.gz");

    println!("{:?}", p1.file_stem());

    Ok(())
}
