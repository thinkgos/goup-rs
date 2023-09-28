use which::which;

fn main() {
    let result = which("go").unwrap();
    println!("Go Bin: {:?}", result);
}
