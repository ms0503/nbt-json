use clap::ArgAction;
use clap::Parser;
use nbt_json::json;
use nbt_json::nbt;
use nbt_json::nbt::FileType;
use rust_i18n::i18n;
use rust_i18n::t;
use serde_json::error::Category;
use std::collections::HashMap;
use std::convert::AsRef;
use std::env;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process;
use std::string::ToString;
use std::sync::LazyLock;

i18n!("locales", fallback = ["en_US", "ja_JP"]);

static HELP_TEMPLATE: LazyLock<String> = LazyLock::new(|| {
    color_print::cformat!(
        "\
{}

<strong><u>{}:</u></strong> {}

<strong><u>{}:</u></strong>
{}

<strong><u>{}:</u></strong>
{}
",
        "{about}",
        t!("help.usage"),
        "{usage}",
        t!("help.arguments"),
        "{positionals}",
        t!("help.options"),
        "{options}"
    )
});

static HELP_ABOUT: LazyLock<String> = LazyLock::new(|| t!("help.about").to_string());
static HELP_FILENAME: LazyLock<String> = LazyLock::new(|| t!("help.filename").to_string());
static HELP_FILETYPE: LazyLock<String> = LazyLock::new(|| t!("help.filetype").to_string());
static HELP_FORCE: LazyLock<String> = LazyLock::new(|| t!("help.force").to_string());
static HELP_HELP: LazyLock<String> = LazyLock::new(|| t!("help.help").to_string());
static HELP_OUT: LazyLock<String> = LazyLock::new(|| t!("help.out").to_string());
static HELP_VERSION: LazyLock<String> = LazyLock::new(|| t!("help.version").to_string());

fn main() {
    if let Ok(lang) = env::var("LANG") {
        if let Some(lang) = lang.split(".").next() {
            rust_i18n::set_locale(lang);
        }
    }
    let cli = Cli::parse();
    let path = Path::new(&cli.filename);
    let out_path = if let Some(ref out) = cli.out {
        if out == "-" {
            None
        } else {
            Some(Path::new(out))
        }
    } else {
        None
    };
    if !path.exists() {
        eprintln!("{}", t!("error.file-not-found", filename = cli.filename));
        process::exit(1);
    }
    if let Some(out_path) = out_path {
        if out_path.exists() {
            if cli.force {
                eprintln!(
                    "{}",
                    t!(
                        "warn.out-file-already-exist",
                        filename = cli.out.clone().unwrap()
                    )
                );
            } else {
                eprintln!(
                    "{}",
                    t!("error.out-file-already-exist", filename = cli.out.unwrap())
                );
                process::exit(1);
            }
        }
    }
    match path.extension() {
        Some(ext) if ext == "dat" => {
            let nbt = fs::read(path);
            if let Err(err) = nbt {
                eprintln!("{}", t!("error.read-failed", reason = err.kind()));
                process::exit(1);
            }
            let nbt = nbt.unwrap();
            let mut json = HashMap::new();
            nbt::to_json(nbt, &mut json);
            let json = serde_json::to_string(&json).unwrap();
            output(cli.out, path, json.into_bytes());
        }
        Some(ext) if ext == "json" => {
            let json = fs::read(path);
            if let Err(err) = json {
                eprintln!("{}", t!("error.read-failed", reason = err.kind()));
                process::exit(1);
            }
            let json = json.unwrap();
            let json = serde_json::from_slice(&json);
            if let Err(err) = json {
                let cat = err.classify();
                if cat == Category::Io {
                    eprintln!(
                        "{}",
                        t!("error.invalid-json", reason = err.io_error_kind().unwrap())
                    );
                } else {
                    eprintln!(
                        "{}",
                        t!("error.invalid-json", reason = format!("{:?}", cat))
                    );
                }
                process::exit(1);
            }
            let json = json.unwrap();
            let mut buf = HashMap::new();
            if let Err(err) = nbt::resolve_tag_types(&mut buf, &json) {
                eprintln!("{}", t!("error.tag-resolving-failed", reason = err));
                process::exit(1);
            }
            let mut nbt = Vec::new();
            json::to_nbt(buf, &mut nbt, cli.filetype);
            output(cli.out, path, nbt);
        }
        _ => {}
    }
}

fn output(out: Option<String>, path: &Path, data: Vec<u8>) {
    match out {
        Some(name) if &*name == "-" => {
            if let Err(err) = io::stdout().write(&data) {
                eprintln!("{}", t!("error.output-failed", reason = err.kind()));
                process::exit(1);
            }
        }
        Some(name) => {
            if let Err(err) = fs::write(name, data) {
                eprintln!("{}", t!("error.output-failed", reason = err.kind()));
                process::exit(1);
            }
        }
        None => {
            let name = path.with_extension("json");
            if let Err(err) = fs::write(name, data) {
                eprintln!("{}", t!("error.output-failed", reason = err.kind()));
                process::exit(1);
            }
        }
    }
}

#[derive(Debug, Parser)]
#[command(about = HELP_ABOUT.as_str(), author, disable_help_flag = true, disable_version_flag = true, help_template = HELP_TEMPLATE.as_str(), long_about = None, version
)]
struct Cli {
    #[arg(help = HELP_FILENAME.as_str())]
    filename: String,
    #[arg(default_value = "raw", help = HELP_FILETYPE.as_str(), long, short = 't')]
    filetype: FileType,
    #[arg(help = HELP_FORCE.as_str(), long, short)]
    force: bool,
    #[arg(action = ArgAction::Help, help = HELP_HELP.as_str(), long, short)]
    help: Option<bool>,
    #[arg(help = HELP_OUT.as_str(), long, short)]
    out: Option<String>,
    #[arg(action = ArgAction::Version, help = HELP_VERSION.as_str(), long, short = 'V')]
    version: Option<bool>,
}
