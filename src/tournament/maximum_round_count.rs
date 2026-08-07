use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::num::NonZeroU16;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MaximumRoundCount(NonZeroU16);

impl MaximumRoundCount {
    pub const fn value(self) -> u16 {
        self.0.get()
    }
}

impl TryFrom<i64> for MaximumRoundCount {
    type Error = MaximumRoundCountError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value <= 0 {
            return Err(MaximumRoundCountError::NotPositive);
        }
        let value =
            u16::try_from(value).map_err(|_| MaximumRoundCountError::ExceedsStorageLimit)?;
        let value = NonZeroU16::new(value).ok_or(MaximumRoundCountError::NotPositive)?;
        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaximumRoundCountError {
    NotPositive,
    ExceedsStorageLimit,
}

impl Display for MaximumRoundCountError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPositive => {
                formatter.write_str("maximum round count must be greater than zero")
            }
            Self::ExceedsStorageLimit => {
                formatter.write_str("maximum round count exceeds the supported storage limit")
            }
        }
    }
}

impl Error for MaximumRoundCountError {}
