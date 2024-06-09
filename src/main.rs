use nbt_json::nbt_to_json;
use nbt_json::TagType;
use rust_i18n::i18n;
use rust_i18n::t;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process;

i18n!("locales", fallback = ["en_US", "ja_JP"]);

fn main() {
    if let Ok(lang) = env::var("LANG") {
        if let Some(lang) = lang.split(".").nth(0) {
            rust_i18n::set_locale(&lang);
        }
    }
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("{}", t!("error.no-filename"));
        process::exit(1);
    }
    if !Path::new(&args[1]).exists() {
        eprintln!("{}", t!("error.file-not-found", filename = args[1]));
        process::exit(1);
    }
    let mut json = HashMap::<String, TagType>::new();
    nbt_to_json(&args[1], &mut json);
    let json = serde_json::to_string(&json).unwrap();
    println!("{}", json);
}
