mod publication;
mod table_assignment;
mod value_types;

pub mod algorithms;

pub use crate::scheduling::TableNumber;
pub use publication::{MatchPublication, publish_scheduled_matches};
pub use table_assignment::{TableAssignmentEntrant, TableAssignmentError, assign_tables};
pub use value_types::EloRating;

#[cfg(test)]
mod table_assignment_tests;
