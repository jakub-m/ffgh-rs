use prost_build;
use std::io::Result;
fn main() -> Result<()> {
    let mut config = prost_build::Config::new();
    config.type_attribute(
        ".",
        //"\n",
        vec![
            "#[serde(deny_unknown_fields, default)]",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        ]
        .join("\n"),
    );
    config.compile_protos(&["src/config.proto"], &["src/"])?;
    Ok(())
}
