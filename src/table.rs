use std::num::NonZeroU16;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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
