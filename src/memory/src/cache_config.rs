#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheError {
    OutOfBounds,
    PolicyFailed,
    UnreachableState,
}

#[derive(Debug, Clone)]
pub enum CacheReturn {
    Hit(Vec<u8>),
    Miss,
}

impl From<CacheReturn> for Vec<u8> {
    fn from (ret: CacheReturn) -> Self {
        return match ret {
            CacheReturn::Hit(value) => value,
            CacheReturn::Miss => vec![]
        }
    }
}
