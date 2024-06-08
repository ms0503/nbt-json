use nbt_json::nbt_to_json;
use nbt_json::TagType;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("must be specified filename");
        process::exit(1);
    }
    if !Path::new(&args[1]).exists() {
        eprintln!("specified file '{}' is not found", args[1]);
        process::exit(1);
    }
    let mut json = HashMap::<String, TagType>::new();
    nbt_to_json(&args[1], &mut json);
    let json = serde_json::to_string(&json).unwrap();
    println!("{}", json);
}
