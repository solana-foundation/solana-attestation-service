#[macro_export]
macro_rules! acc_info_as_str {
    ($info:expr) => {
        bs58::encode($info.key()).into_string().as_str()
    };
}

#[macro_export]
macro_rules! key_as_str {
    ($key:expr) => {
        bs58::encode($key).into_string().as_str()
    };
}
