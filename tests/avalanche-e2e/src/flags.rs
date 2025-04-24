/// Defines flag options.
#[derive(Debug)]
pub struct Options {
    pub log_level: String,
    pub spec_path: String,
    pub skip_prompt: bool,
}

pub fn command() -> Command {
    Command::new("flags")
        .about("Test flags and configuration")
        .arg(
            Arg::new("SKIP_PROMPT")
                .long("skip-prompt")
                .help("Skips prompt mode")
                .required(false)
                .num_args(0),
        )
        .arg(
            Arg::new("LOG_LEVEL")
                .long("log-level")
                .help("Sets the log level for test output")
                .required(false)
                .num_args(1)
                .default_value("info"),
        )
}
