pub mod info;
pub mod admin;
pub mod fun;
pub mod level_cookies;

#[allow(dead_code)]
pub fn stringify_error<X: std::fmt::Debug>(error: X) -> String{
    return format!("An Error occured: {:?}", error)
}
