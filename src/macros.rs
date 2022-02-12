#[macro_export]
macro_rules! set_env {
    ($name:expr,$value:expr) => {
        std::env::set_var($name, $value);
    };
}

#[macro_export]
macro_rules! flush {
    () => {
        std::io::stdout().flush().unwrap();
        std::io::stderr().flush().unwrap();
    };
}
