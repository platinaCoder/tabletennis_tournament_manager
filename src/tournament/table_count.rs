use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::num::NonZeroU16;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TableCount(NonZeroU16);

impl TableCount {
    pub const fn value(self) -> u16 {
        self.0.get()
    }
}

impl TryFrom<i64> for TableCount {
    type Error = TableCountError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value <= 0 {
            return Err(TableCountError::NotPositive);
        }

        let value = u16::try_from(value).map_err(|_| TableCountError::ExceedsStorageLimit)?;
        let value = NonZeroU16::new(value).ok_or(TableCountError::NotPositive)?;
        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableCountError {
    NotPositive,
    ExceedsStorageLimit,
}

impl Display for TableCountError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPositive => formatter.write_str("table count must be greater than zero"),
            Self::ExceedsStorageLimit => {
                formatter.write_str("table count exceeds the supported storage limit")
            }
        }
    }
}

impl Error for TableCountError {}
