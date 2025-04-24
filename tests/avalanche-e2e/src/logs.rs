pub fn init(log_level: &str) {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, log_level);
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
}
