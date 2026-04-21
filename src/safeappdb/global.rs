

pub enum LogLevel {
    debug,
    info,
    warn,
    error,
}

pub enum Mode {
    debug,
    release,
}
pub struct Global {
    pub loglevel: LogLevel,
    pub log_file: String,
    pub state_file: String,
    pub config_file: String,
    pub config_file_gen: String,
    pub mode: Mode,
    pub endpoint: String,
}

