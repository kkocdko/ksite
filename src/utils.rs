use anyhow::Result;
pub trait OptionResult<T> {
    fn e(self) -> Result<T>;
}
impl<T> OptionResult<T> for Option<T> {
    fn e(self) -> Result<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(anyhow::anyhow!("Option is None")),
        }
    }
}

#[macro_export]
macro_rules! care {
    ($result:expr) => {{
        let result = $result;
        if let Err(e) = &result {
            eprintln!("[cared error] {}:{} {:?}", file!(), line!(), e);
        }
        result
    }};
    ($result:expr, $arg:tt) => {{
        match care!($result) {
            Ok(v) => v,
            _ => $arg,
        }
    }};
}
