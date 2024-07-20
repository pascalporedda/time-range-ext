use time::OffsetDateTime;
use crate::time_range::TimeRange;

pub trait TimeRangeExt {
    fn ends(&self) -> Vec<OffsetDateTime>;
    fn starts(&self) -> Vec<OffsetDateTime>;

    fn contains_ts(&self, start: OffsetDateTime) -> bool;

    fn range_within_ts(&self, ts: OffsetDateTime) -> Option<&TimeRange>;
    fn overlaps(&self, other: &TimeRange) -> Option<&TimeRange>;

    fn get_overlapping_range(&self, ts: TimeRange) -> Option<TimeRange>;

    fn dedup_overlapping_ranges(self) -> Self;

    fn times_between_contents(self, bounds: Option<TimeRange>) -> Self;
}
