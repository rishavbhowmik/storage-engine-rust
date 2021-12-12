pub struct Error {
    pub code: i32,
    pub message: String,
}
// add fmt::Debug trait to Error struct
use std::fmt;
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error {{ code: {}, message: {} }}",
            self.code, self.message
        )
    }
}