use semver::{Version, VersionReq};

fn main() {
    let ver = VersionReq::parse("1.20.x").unwrap();

    let v1 = Version::parse("1.21.1").unwrap();
    let v2 = Version::parse("1.21.2").unwrap();
    println!("{:#?}", ver);
    println!("{:#?}", v1);
    println!("{:#?}", v1 < v2);
    let v3 = v2.clone();
    let mut vs = vec![v2, v1];
    vs.sort();
    println!("{:#?}", vs);

    println!("{:#?}", ver.matches(&v3));
}
