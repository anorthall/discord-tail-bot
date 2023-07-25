use secrecy::Secret;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub discord_token: Secret<String>,
    pub log_file: String,
    pub channel_id: u64,
    pub pattern: String,
    pub wait_time: u64,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}
