use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::num::NonZeroU16;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TableNumber(NonZeroU16);

impl TableNumber {
    pub const fn value(self) -> u16 {
        self.0.get()
    }

    pub(crate) fn within_configured_count(value: u16, table_count: u16) -> Option<Self> {
        if value > table_count {
            return None;
        }

        NonZeroU16::new(value).map(Self)
    }
}

impl TryFrom<i64> for TableNumber {
    type Error = TableNumberError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = u16::try_from(value).map_err(|_| TableNumberError)?;
        NonZeroU16::new(value).map(Self).ok_or(TableNumberError)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableNumberError;

impl Display for TableNumberError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("table number must be between 1 and 65535")
    }
}

impl Error for TableNumberError {}
