use thiserror::Error;

pub type Time = i64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeInterval {
    start: Time,
    end: Time,
}

impl TimeInterval {
    pub fn new(start: Time, end: Time) -> Result<Self, TimeIntervalError> {
        if start > end {
            Err(TimeIntervalError)
        } else {
            Ok(Self { start, end })
        }
    }
}

impl PartialEq<(Time, Time)> for TimeInterval {
    fn eq(&self, &(start, end): &(Time, Time)) -> bool {
        self.start == start && self.end == end
    }
}

impl TryFrom<(Time, Time)> for TimeInterval {
    type Error = TimeIntervalError;

    fn try_from((start, end): (Time, Time)) -> Result<Self, Self::Error> {
        Self::new(start, end)
    }
}

#[derive(Debug, Error)]
#[error("A valid time interval must have a start time less than or equal tothe end time")]
pub struct TimeIntervalError;

#[derive(Clone, Debug)]
pub struct TimeIntervals {
    // When we construct a TimeIntervals structure, we sort the intervals by their start time and
    // keep them in a Vec as contiguously allocated memory to optimize memory access and lower
    // initialization time (fewer allocations and pointer indirection). We're optimizing for the
    // search case and do not expect the set of intervals to be expanded post-construction. If we
    // needed to support the abilility for high-rates of mutation such as adding new individual
    // intervals to the set, we could consider a more complex data structure like a binary tree or a
    // B+-tree with a high order to optimize the cost of mutating intervals in our index while
    // maintaining performant logarithmic search capability.
    intervals: Vec<TimeInterval>,
}

impl TimeIntervals {
    pub fn new(mut intervals: Vec<TimeInterval>) -> Self {
        // Because TimeInterval is correct by construction and cannot be constructed with an invalid
        // start & end time via its public API, we do not have to do any additional validation on
        // the list of intervals before sorting them and constructing ourself.

        // Sort the intervals by their start time which sets us up for the ability to do a binary
        // search by start time in the Vec.
        intervals.sort_by_key(|interval| interval.start);

        // One thing we can do to optimize the search space is to remove any intervals that overlap
        // or are perfectly adjacent to each other (I'm only considering adjacency because we're
        // dealing with low resolution i64 timestamps here). This will reduce the search space of
        // our binary search. Also, our requirements are only to know if a target time is within our
        // overall interval space, we do not need to know about any particular interval as there is
        // no associated data of interest.
        //
        // So to do this, we iterate through the sorted intervals and combine any interval that
        // overlaps or is adjacent with the previous interval. Prime case for a reduction!
        intervals = intervals.into_iter().fold(Vec::new(), |mut intervals, interval| {
            // If the last interval in our Vec overlaps or is adjacent with the current interval, we merge them.
            if let Some(last) = intervals.last_mut() {
                if last.end >= interval.start - 1 {
                    last.end = last.end.max(interval.end);
                } else {
                    intervals.push(interval);
                }
            } else {
                intervals.push(interval);
            }

            intervals
        });

        Self { intervals }
    }

    pub fn contains_time(&self, time: Time) -> bool {
        // If there are no intervals, don't bother doing the binary search. This guarantees that the
        // returned index from partition_point is within the bounds of the Vec.
        if self.intervals.is_empty() {
            return false;
        }

        // https://doc.rust-lang.org/std/primitive.slice.html#method.partition_point partition_point
        // does a binary search in the Vec. We're looking for the last (right-most in the Vec)
        // interval that has a start time that is less than or equal to our target time. The
        // returned index is the point just after that right-most interval, or the length of the Vec
        // if there is no point. Therefore we need to take a look at the interval just before that
        // index and see if our target time is less than or equal to the end time of that interval.
        //
        // Of course I could have implemented the binary search myself. But why would I want to
        // waste time doing that? This is a standard algorithm and I trust Rust's stdlib.
        let index = self.intervals.partition_point(|interval| interval.start <= time);
        index > 0 && self.intervals[index - 1].end >= time
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }
}

// These are just some of the convenient ways to construct a TimeIntervals structure.
impl From<Vec<TimeInterval>> for TimeIntervals {
    fn from(intervals: Vec<TimeInterval>) -> Self {
        Self::new(intervals)
    }
}

impl FromIterator<TimeInterval> for TimeIntervals {
    fn from_iter<T: IntoIterator<Item = TimeInterval>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

// This is a convenient way to attempt to construct a TimeIntervals structure from a slice of time
// tuples.
impl TryFrom<&[(Time, Time)]> for TimeIntervals {
    type Error = TimeIntervalError;

    fn try_from(intervals: &[(Time, Time)]) -> Result<Self, Self::Error> {
        intervals
            .iter()
            .map(|&interval| TimeInterval::try_from(interval))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_interval() {
        // start == end
        assert!(TimeInterval::new(1, 1).is_ok());

        // start < end
        assert!(TimeInterval::new(1, 2).is_ok());

        // start > end
        assert!(TimeInterval::new(2, 1).is_err());
    }

    #[test]
    fn already_sorted() -> Result<(), TimeIntervalError> {
        const INTERVALS: &[(Time, Time)] = &[(1, 2), (3, 4), (5, 6)];
        let time_intervals = TimeIntervals::try_from(INTERVALS)?;

        // Intervals perfectly contain 1-6, but nothing outside of that range.
        assert!(!time_intervals.contains_time(0));
        assert!(time_intervals.contains_time(1));
        assert!(time_intervals.contains_time(2));
        assert!(time_intervals.contains_time(3));
        assert!(time_intervals.contains_time(4));
        assert!(time_intervals.contains_time(5));
        assert!(time_intervals.contains_time(6));
        assert!(!time_intervals.contains_time(7));

        // We also make sure that the only interval we have is [1, 6] to verify we optimized the
        // search space.
        assert_eq!(time_intervals.intervals, [(1, 6)]);

        Ok(())
    }

    #[test]
    fn out_of_order() -> Result<(), TimeIntervalError> {
        const INTERVALS: &[(Time, Time)] = &[(3, 4), (1, 2), (5, 6)];
        let time_intervals = TimeIntervals::try_from(INTERVALS)?;

        // Intervals perfectly contain 1-6, but nothing outside of that range.
        assert!(!time_intervals.contains_time(0));
        assert!(time_intervals.contains_time(1));
        assert!(time_intervals.contains_time(2));
        assert!(time_intervals.contains_time(3));
        assert!(time_intervals.contains_time(4));
        assert!(time_intervals.contains_time(5));
        assert!(time_intervals.contains_time(6));
        assert!(!time_intervals.contains_time(7));

        // We also make sure that the only interval we have is [1, 6] to verify we optimized the
        // search space.
        assert_eq!(time_intervals.intervals, [(1, 6)]);

        Ok(())
    }

    #[test]
    fn gaps() -> Result<(), TimeIntervalError> {
        const INTERVALS: &[(Time, Time)] = &[(5, 10), (100, 200), (50, 60)];
        let time_intervals = TimeIntervals::try_from(INTERVALS)?;

        assert!(!time_intervals.contains_time(4));
        assert!(time_intervals.contains_time(5));
        assert!(time_intervals.contains_time(8));
        assert!(time_intervals.contains_time(10));
        assert!(!time_intervals.contains_time(11));

        assert!(!time_intervals.contains_time(30));

        assert!(!time_intervals.contains_time(49));
        assert!(time_intervals.contains_time(50));
        assert!(time_intervals.contains_time(55));
        assert!(time_intervals.contains_time(60));
        assert!(!time_intervals.contains_time(61));

        assert!(!time_intervals.contains_time(75));

        assert!(!time_intervals.contains_time(99));
        assert!(time_intervals.contains_time(100));
        assert!(time_intervals.contains_time(150));
        assert!(time_intervals.contains_time(200));
        assert!(!time_intervals.contains_time(201));

        // We also verify that there were no possible optimizations of the search space.
        assert_eq!(time_intervals.intervals, [(5, 10), (50, 60), (100, 200)]);

        Ok(())
    }

    #[test]
    fn overlapping() -> Result<(), TimeIntervalError> {
        const INTERVALS: &[(Time, Time)] = &[(50, 100), (75, 150), (90, 500), (30, 75)];
        let time_intervals = TimeIntervals::try_from(INTERVALS)?;

        // The valid range is anything between [30, 500].
        assert!(!time_intervals.contains_time(29));
        assert!(time_intervals.contains_time(30));
        assert!(time_intervals.contains_time(80));
        assert!(time_intervals.contains_time(100));
        assert!(time_intervals.contains_time(155));
        assert!(time_intervals.contains_time(400));
        assert!(time_intervals.contains_time(500));
        assert!(!time_intervals.contains_time(501));

        // We also make sure that the only interval we have is [30, 500] to verify we optimized the
        // search space.
        assert_eq!(time_intervals.intervals, [(30, 500)]);

        Ok(())
    }

    #[test]
    fn sparse_overlapping() -> Result<(), TimeIntervalError> {
        const INTERVALS: &[(Time, Time)] = &[(50, 100), (75, 150), (200, 500), (300, 700)];
        let time_intervals = TimeIntervals::try_from(INTERVALS)?;

        // The valid range is [50, 150] and [200, 700].
        assert!(!time_intervals.contains_time(49));
        assert!(time_intervals.contains_time(50));
        assert!(time_intervals.contains_time(100));
        assert!(time_intervals.contains_time(150));
        assert!(!time_intervals.contains_time(151));
        assert!(!time_intervals.contains_time(199));
        assert!(time_intervals.contains_time(200));
        assert!(time_intervals.contains_time(400));
        assert!(time_intervals.contains_time(700));
        assert!(!time_intervals.contains_time(701));

        // We also make sure that the only intervals we have are [50, 150] and [200, 700] to verify
        // we optimized the search space.
        assert_eq!(time_intervals.intervals, [(50, 150), (200, 700)]);

        Ok(())
    }

    #[test]
    fn overshadowed() -> Result<(), TimeIntervalError> {
        const INTERVALS: &[(Time, Time)] = &[(20, 40), (30, 50), (60, 90), (1, 100)];
        let time_intervals = TimeIntervals::try_from(INTERVALS)?;

        // The valid range is [1, 100] because the last interval overshadows the previous ones.
        assert!(!time_intervals.contains_time(0));
        assert!(time_intervals.contains_time(1));
        assert!(time_intervals.contains_time(55));
        assert!(time_intervals.contains_time(100));
        assert!(!time_intervals.contains_time(101));

        // We also make sure that the only interval we have is [1, 100] to verify we optimized the
        // search space.
        assert_eq!(time_intervals.intervals, [(1, 100)]);

        Ok(())
    }
}
