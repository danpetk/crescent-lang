use std::process;

fn main() {
    let _filename =  std::env::args().nth(1).unwrap_or_else(||{
        eprintln!("Invalid Arguments!");
        eprintln!("Expected Usage: crescent-lang {{filename}}");
        process::exit(1)        
    });
}
