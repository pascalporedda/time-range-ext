use std::ops::{Add, Sub};
use time::{OffsetDateTime, Time, Weekday};
use time::ext::NumericalDuration;
use crate::TimeRange;

#[derive(Debug, Clone)]
pub struct WorkingHours {
   pub start: Time,
   pub end: Time,
   pub active_days: Vec<Weekday>,

   pub lower_bound: Option<OffsetDateTime>,
   pub upper_bound: Option<OffsetDateTime>
}

impl WorkingHours {
    pub fn is_active(&self) -> bool {
        !self.active_days.is_empty()
    }

    pub fn active_during_ts(&self, time: OffsetDateTime) -> bool {
        self.active_days.contains(&time.weekday()) && self.start <= time.time() && time.time() <= self.end
    }

    pub fn active_during_day(&self, day: Weekday) -> bool {
        self.active_days.contains(&day)
    }

    pub fn working_time_in_range(&self, range: TimeRange) -> Vec<TimeRange> {
        if !self.is_active() {
            return vec![];
        }

        let range_start = range.start;
        let range_end = range.end;
        
        let mut working_times = vec![];
        let mut current = range_start;

        if let Some(previous_working_hours) = self.previous_working_hours(current) {
            if range_start <= previous_working_hours.end {
                working_times.push(previous_working_hours);
            }
        }

        while current < range_end {
            if !self.active_during_day(current.weekday()) {
                current = current.add(1.days());
                continue;
            }

            let next_shift = self.next_working_hours(current);

            if let Some(next_working_hours) = next_shift {
                if next_working_hours.start <= range_end && !working_times.contains(&next_working_hours) {
                    working_times.push(next_working_hours);
                }
            }

            current = current.add(1.days());
        }

        if let Some(first) = working_times.first_mut() {
            if first.start < range_start {
                first.start = range_start;
            }
        }

        if let Some(last) = working_times.last_mut() {
            if last.end > range_end {
                last.end = range_end;
            }
        }

        working_times
    }

    fn exceeds_bounds(&self, ts: OffsetDateTime) -> bool {
        self.lower_bound.map_or(false, |l| ts < l) || self.upper_bound.map_or(false, |u| ts > u)
    }

    fn previous_working_hours(&self, ts: OffsetDateTime) -> Option<TimeRange> {
        let mut current = ts;

        if !self.is_active() {
            return None;
        }

        if self.exceeds_bounds(current) {
            return None;
        }

        // handle case that the requested timestamp lies before start and end time, but would be
        // active on that day, and would therefore cause times that are invalid
        if self.start > ts.time() {
            current = current.sub(1.days());
        }

        while !self.active_during_day(current.weekday()) {
            current = current.sub(1.days());
        }

        // handle Case when we have a working time that has a start time of e.g., 23:30 and end time of 03:00
        if self.start > self.end {
            let start_date = current.replace_time(self.start);
            let mut end_date = current.replace_time(self.end);

            if end_date < start_date {
                end_date = end_date.add(1.days());
            }

            // always check if the replaced date of the working time is set,
            // and the end date would be greater than our current end_date
            if self.upper_bound.map_or(false, |r| end_date > r) {
                end_date = self.upper_bound.unwrap();
            }

            return Some(TimeRange {
                start: start_date,
                end: end_date,
            });
        }

        let mut end = current.replace_time(self.end);

        // the same check as above, but for the normal case where times are within the same day
        if self.upper_bound.map_or(false, |r| end > r) {
            end = self.upper_bound.unwrap();
        }

        let mut start = current.replace_time(self.start);

        if self.lower_bound.map_or(false, |l| start < l) {
            start = self.lower_bound.unwrap();
        }

        Some(TimeRange { start, end })
    }

    fn next_working_hours(&self, ts: OffsetDateTime) -> Option<TimeRange> {
        let mut current = ts;

        if !self.is_active() {
            return None;
        }

        if self.exceeds_bounds(current) {
            return None;
        }

        if current.time() > self.start && current.time() > self.end {
            current = current.add(1.days());
        }

        while !self.active_during_day(current.weekday()) {
            current = current.add(time::Duration::days(1));
        }

        if self.start > self.end {
            let start_date = current.replace_time(self.start);
            let mut end_date = current.replace_time(self.end).add(1.days());

            // always check if the replaced date of the times is set,
            // and the end date would be greater than our current end_date
            if self.upper_bound.map_or(false, |r| end_date > r) {
                end_date = self.upper_bound.unwrap();
            }

            if end_date < start_date {
                end_date = end_date.add(1.days());
            }

            return Some(TimeRange {
                start: start_date,
                end: end_date,
            });
        }

        let mut end = current.replace_time(self.end);
        if self.upper_bound.map_or(false, |r| end > r) {
            end = self.upper_bound.unwrap();
        }

        let mut start = current.replace_time(self.start);

        if self.lower_bound.map_or(false, |l| start < l) {
            start = self.lower_bound.unwrap();
        }

        if start > end {
            return None;
        }

        Some(TimeRange { start, end })
    }
}