use core::fmt::Error;

pub fn config_loader() -> Result<String, Error> {
    let f = std::fs::File::open("config.yaml").unwrap();
    let d: String = serde_yaml::from_reader(f).unwrap();
    println!("Read YAML string: {}", d);
    Ok(d)
}
