extern crate clap; 

//use origen::test;
use clap::App; 

// This is the entry point for the Origen CLI tool
fn main() { 
    //test();
    App::new("origen")
       //.version("1.0")
       .about("Does great things!")
       .get_matches(); 
}
