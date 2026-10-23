//! compiles extracted vanilla command parser ID into the command packet encoder

use std::{env, error::Error, fmt::Write as _, fs, path::PathBuf};

use heck::ToUpperCamelCase;
use serde::Deserialize;

#[derive(Deserialize)]
struct ArgumentType {
    id: i32,
    key: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = "build_assets/command_argument_types.json";
    println!("cargo:rerun-if-changed={path}");
    let arguments: Vec<ArgumentType> = serde_json::from_str(&fs::read_to_string(path)?)?;
    let mut source =
        String::from("impl ArgumentType { const fn discriminant(&self) -> i32 { match self {\n");
    for argument in arguments {
        let (_, name) = argument
            .key
            .split_once(':')
            .ok_or("Missing argument namespace")?;
        let variant = match name {
            "team_color" => "Color".to_owned(),
            "nbt_compound_tag" => "Nbt".to_owned(),
            name => name.to_upper_camel_case(),
        };
        let fields = match name {
            "float"
            | "double"
            | "integer"
            | "long"
            | "string"
            | "entity"
            | "score_holder"
            | "time"
            | "resource_or_tag"
            | "resource_or_tag_key"
            | "resource"
            | "resource_key"
            | "resource_selector" => " { .. }",
            _ => "",
        };
        writeln!(source, "Self::{variant}{fields} => {},", argument.id)?;
    }
    source.push_str("} } }\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR")?).join("command_argument_types.rs"),
        source,
    )?;
    Ok(())
}
