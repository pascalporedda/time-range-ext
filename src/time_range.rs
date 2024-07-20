use time::OffsetDateTime;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod vec_range;
pub mod time_range_ext;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimeRange {
    #[cfg_attr(feature = "serde", serde(with = "time::serde::rfc3339"))]
    pub start: OffsetDateTime,
    #[cfg_attr(feature = "serde", serde(with = "time::serde::rfc3339"))]
    pub end: OffsetDateTime,
}

impl TimeRange {
    pub fn duration(&self) -> time::Duration {
        self.end - self.start
    }

    pub fn from(start: OffsetDateTime, end: OffsetDateTime) -> Self {
        TimeRange { start, end }
    }

    pub fn overlaps(&self, other: &TimeRange) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    pub fn merge(&self, other: &TimeRange) -> TimeRange {
        TimeRange {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}