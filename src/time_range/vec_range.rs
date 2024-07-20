use time::OffsetDateTime;
use crate::time_range::{TimeRange};
use crate::time_range::time_range_ext::TimeRangeExt;

impl TimeRangeExt for Vec<TimeRange> {
    fn ends(&self) -> Vec<OffsetDateTime> {
        self.iter().map(|t| t.end).collect()
    }
    fn starts(&self) -> Vec<OffsetDateTime> {
        self.iter().map(|t| t.start).collect()
    }

    fn contains_ts(&self, ts: OffsetDateTime) -> bool {
        self.iter().any(|t| t.start <= ts && ts <= t.end)
    }

    fn range_within_ts(&self, ts: OffsetDateTime) -> Option<&TimeRange> {
        self.iter().find(|t| t.start <= ts && ts <= t.end)
    }

    fn overlaps(&self, other: &TimeRange) -> Option<&TimeRange> {
        self.iter().find(|r| other.overlaps(r))
    }

    fn get_overlapping_range(&self, tr: TimeRange) -> Option<TimeRange> {
        let range_start = self.range_within_ts(tr.start);
        let range_end = self.range_within_ts(tr.end);

        let within_range = self.overlaps(&tr);

        match (range_start, range_end, within_range) {
            (Some(start), Some(end), _) => {
                // To cover the last test case if there are gaps in between, but the end of the
                // passed tr just hits the start of a new time range in the vec
                if start.end != end.end {
                    Some(TimeRange {
                        start: tr.start,
                        end: start.end,
                    })
                } else {
                    Some(TimeRange {
                        start: tr.start,
                        end: tr.end,
                    })
                }
            }
            (Some(start), None, _) => Some(TimeRange {
                start: tr.start,
                end: start.end,
            }),
            (None, Some(end), _) => Some(TimeRange {
                start: end.start,
                end: tr.end,
            }),
            (None, None, Some(within)) => Some(*within),
            (None, None, None) => None,
        }
    }

    fn dedup_overlapping_ranges(mut self) -> Self {
        if self.is_empty() {
            return self;
        }

        self.sort_by(|a, b| a.start.cmp(&b.start));

        let mut result = Vec::new();

        let mut current_range = self[0];
        for range in self.into_iter().skip(1) {
            if current_range.overlaps(&range) {
                current_range = current_range.merge(&range);
            } else {
                result.push(current_range);
                current_range = range;
            }
        }
        result.push(current_range);

        result
    }

    fn times_between_contents(self, bounds: Option<TimeRange>) -> Self {
        let mut times_between = vec![];
        let mut windows = self.windows(2);

        if self.len() == 1 {
            if let Some(bounds) = bounds {
                if self[0].end < bounds.end {
                    times_between.push(TimeRange {
                        start: self[0].end,
                        end: bounds.end,
                    });
                }
            }

            return times_between;
        }

        while let Some(&[current, next]) = windows.next() {
            if current.end == next.start {
                continue;
            }

            let start = current.end;
            let end = next.start;

            times_between.push(TimeRange { start, end });
        }

        times_between
    }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;
    use crate::time_range::time_range_ext::TimeRangeExt;

    #[test]
    fn test_overlapping_range() {
        let ranges = vec![
            super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(10).unwrap(),
            },
            super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(15).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(20).unwrap(),
            },
            super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(20).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(30).unwrap(),
            },
        ];

        let tr = super::TimeRange {
            start: time::OffsetDateTime::from_unix_timestamp(5).unwrap(),
            end: time::OffsetDateTime::from_unix_timestamp(8).unwrap(),
        };

        let overlapping_range = ranges.get_overlapping_range(tr);

        assert_eq!(
            overlapping_range,
            Some(super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(5).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(8).unwrap(),
            })
        );

        let tr = super::TimeRange {
            start: time::OffsetDateTime::from_unix_timestamp(5).unwrap(),
            end: time::OffsetDateTime::from_unix_timestamp(15).unwrap(),
        };

        let overlapping_range = ranges.get_overlapping_range(tr);

        assert_eq!(
            overlapping_range,
            Some(super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(5).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(10).unwrap(),
            })
        );

        let ranges = vec![super::TimeRange {
            start: datetime!(2024-04-21 23:10:15.502 UTC),
            end: datetime!(2024-04-22 05:30:00.000 UTC),
        }];

        let tr = super::TimeRange {
            start: datetime!(2024-04-20 16:25:25.632 UTC),
            end: datetime!(2024-04-22 05:45:03.018 UTC),
        };

        assert_eq!(
            ranges.get_overlapping_range(tr),
            Some(super::TimeRange {
                start: datetime!(2024-04-21 23:10:15.502 UTC),
                end: datetime!(2024-04-22 05:30:00.000 UTC),
            })
        );
    }

    #[test]
    fn test_deduping() {
        let ranges = vec![
            super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(10).unwrap(),
            },
            super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(5).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(15).unwrap(),
            },
            super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(20).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(30).unwrap(),
            },
        ];

        let deduped_ranges = ranges.dedup_overlapping_ranges();

        assert_eq!(
            deduped_ranges,
            vec![
                super::TimeRange {
                    start: time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
                    end: time::OffsetDateTime::from_unix_timestamp(15).unwrap(),
                },
                super::TimeRange {
                    start: time::OffsetDateTime::from_unix_timestamp(20).unwrap(),
                    end: time::OffsetDateTime::from_unix_timestamp(30).unwrap(),
                },
            ]
        );

        let ranges = vec![
            super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(10).unwrap(),
            },
            super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(5).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(15).unwrap(),
            },
            super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(50).unwrap(),
            },
            super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(20).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(30).unwrap(),
            },
        ];

        let deduped_ranges = ranges.dedup_overlapping_ranges();

        assert_eq!(
            deduped_ranges,
            vec![super::TimeRange {
                start: time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
                end: time::OffsetDateTime::from_unix_timestamp(50).unwrap(),
            },]
        );
    }
}
