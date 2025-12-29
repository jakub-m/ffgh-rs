use ffgh::config::Config;

fn main() {
    let yaml = r#"
        queries: []
        display_order: []
        attribution_order: []
        annotations:
        - foo
        - bar
    "#;

    let c: Config = serde_yaml::from_str(yaml).unwrap();
    println!("{c:?}");
}
