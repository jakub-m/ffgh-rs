use prost_build;
use std::io::Result;
fn main() -> Result<()> {
    let mut config = prost_build::Config::new();
    config.type_attribute(
        ".",
        "#[derive(serde::Deserialize, serde::Serialize)]\n#[serde(deny_unknown_fields)]",
    );
    config.compile_protos(&["src/config.proto"], &["src/"])?;
    Ok(())
}
