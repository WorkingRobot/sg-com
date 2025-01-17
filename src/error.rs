use crate::bindings::SG_Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Error(SG_Error);

impl Error {
    pub fn code(&self) -> SG_Error {
        self.0
    }

    pub fn is_ok(&self) -> bool {
        self.0 == SG_Error::SG_ERROR_OK
    }
}

impl SG_Error {
    pub fn into_result(self) -> Result<()> {
        if self == SG_Error::SG_ERROR_OK {
            Ok(())
        } else {
            Err(Error(self))
        }
    }
}

impl From<SG_Error> for Error {
    fn from(error_code: SG_Error) -> Self {
        Self(error_code)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error code: {:?}", self.0)
    }
}

impl std::error::Error for Error {}
