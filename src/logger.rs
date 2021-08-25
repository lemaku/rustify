extern crate dotenv;

/// Basic Logger. Behaves just like println! macro, but writes log to
/// file if environment variable LOG_TO_FILE is set to true and
/// LOG_FILE_PATH contains a path to an existing file.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        dotenv::dotenv().ok();
        let log_message = format!("{} > [rustify]: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),format!($($arg)*));
        let log_to_file: bool = std::env::var("LOG_TO_FILE").unwrap_or("false".to_string()) == "true";
        if log_to_file {
            let log_file = std::env::var("LOG_FILE_PATH").expect("Please set LOG_FILE_PATH in your .env file");
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(log_file)
                .unwrap();
            file.write_all(log_message.to_string().as_bytes()).unwrap();
        }
        print!("{}", log_message);
    })
}
