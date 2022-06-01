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
