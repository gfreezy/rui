#[test]
fn test_it() {
    let my_struct = MyStruct {
        name: "MixIn Works".to_string(),
    };
    assert_eq!(my_struct.name(), "MixIn Works");
}

fn main() {
    println!("Hello, world!");
}
