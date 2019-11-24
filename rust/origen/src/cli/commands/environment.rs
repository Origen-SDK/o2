pub fn main(tname: Option<&str>) {
    if tname.is_none() {
    } else {
        let name = tname.unwrap();

        println!("Environment is: {}", name);
    }
}
