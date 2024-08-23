use chrono::NaiveDate;
use sora_model::contract::Contract;

#[derive(Debug, PartialEq, Eq)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DateRange {
    pub const fn new(start: NaiveDate, end: NaiveDate) -> Self {
        Self { start, end }
    }

    /// This representation will return true
    /// |=====self=======|
    ///      |======other======|
    pub fn overlap(&self, other: &Self) -> bool {
        self.start <= other.end && self.end >= other.start
    }

    /// This representation will return true
    ///       |====self===|
    /// |=======other=======|
    pub fn is_contained_in(&self, other: &Self) -> bool {
        other.start <= self.start && other.end >= self.end
    }
}

impl From<&Contract> for DateRange {
    fn from(value: &Contract) -> Self {
        Self::new(*value.start(), *value.end())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDate;
    use rstest::rstest;

    fn create_date_range(start: (i32, u32, u32), end: (i32, u32, u32)) -> DateRange {
        DateRange::new(
            NaiveDate::from_ymd_opt(start.0, start.1, start.2).unwrap(),
            NaiveDate::from_ymd_opt(end.0, end.1, end.2).unwrap(),
        )
    }

    #[rstest]
    #[case(
        (2024, 1, 1), (2024, 1, 10),
        (2024, 1, 11), (2024, 1, 20),
        true
    )]
    #[case(
        (2024, 1, 5), (2024, 1, 15),
        (2024, 1, 10), (2024, 1, 20),
        false
    )]
    #[case(
        (2024, 1, 5), (2024, 1, 15),
        (2024, 1, 5), (2024, 1, 15),
        false
    )]
    #[case(
        (2024, 1, 5), (2024, 1, 15),
        (2024, 1, 7), (2024, 1, 10),
        false
    )]
    #[case(
        (2024, 1, 1), (2024, 1, 10),
        (2024, 1, 10), (2024, 1, 20),
        false
    )]
    fn test_no_overlap(
        #[case] start1: (i32, u32, u32),
        #[case] end1: (i32, u32, u32),
        #[case] start2: (i32, u32, u32),
        #[case] end2: (i32, u32, u32),
        #[case] expected: bool,
    ) {
        let range1 = create_date_range(start1, end1);
        let range2 = create_date_range(start2, end2);

        assert_eq!(!range1.overlap(&range2), expected);
    }

    #[rstest]
    #[case(
        (2024, 1, 3), (2024, 1, 7),
        (2024, 1, 1), (2024, 1, 10),
        true
    )]
    #[case(
        (2024, 1, 1), (2024, 1, 10),
        (2024, 1, 3), (2024, 1, 7),
        false
    )]
    #[case(
        (2024, 1, 1), (2024, 1, 5),
        (2024, 1, 3), (2024, 1, 10),
        false
    )]
    #[case(
        (2024, 1, 3), (2024, 1, 10),
        (2024, 1, 1), (2024, 1, 5),
        false
    )]
    fn test_is_contained_in(
        #[case] start1: (i32, u32, u32),
        #[case] end1: (i32, u32, u32),
        #[case] start2: (i32, u32, u32),
        #[case] end2: (i32, u32, u32),
        #[case] expected: bool,
    ) {
        let range1 = create_date_range(start1, end1);
        let range2 = create_date_range(start2, end2);

        assert_eq!(range1.is_contained_in(&range2), expected);
    }
}

pub fn invert_ranges_in_boundary<'date_range, Iter>(
    ranges: Iter,
    start: NaiveDate,
    end: NaiveDate,
) -> Vec<DateRange>
where
    Iter: IntoIterator<Item = &'date_range DateRange>,
{
    let mut result = Vec::new();

    // Sort the ranges by their start date
    let mut ranges = ranges.into_iter().collect::<Vec<_>>();
    ranges.sort_by_key(|r| r.start);

    // Track the current start of the inverted range
    let mut current_start = start;

    for range in ranges {
        // If there's a gap between the current start and the next range's start, add it to the result
        if current_start < range.start {
            result.push(DateRange {
                start: current_start,
                end: range.start,
            });
        }
        // Move the current start to the end of the current range
        if current_start < range.end {
            current_start = range.end;
        }
    }

    // If there's a gap between the last range and the end date, add it
    if current_start < end {
        result.push(DateRange {
            start: current_start,
            end,
        });
    }

    result
}

#[cfg(test)]
mod invert_range_test {

    use super::*;
    use crate::range::DateRange;
    use rstest::rstest;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[rstest]
    #[case(
        "Single gap before and after a range",
        vec![DateRange::new(date(2023, 1, 10), date(2023, 1, 15))],
        date(2023, 1, 5),
        date(2023, 1, 20),
        vec![
            DateRange::new(date(2023, 1, 5), date(2023, 1, 10)),
            DateRange::new(date(2023, 1, 15), date(2023, 1, 20))
        ]
    )]
    #[case(
        "Multiple gaps between non-overlapping ranges",
        vec![
            DateRange::new(date(2023, 1, 10), date(2023, 1, 15)),
            DateRange::new(date(2023, 1, 20), date(2023, 1, 25))
        ],
        date(2023, 1, 5),
        date(2023, 1, 30),
        vec![
            DateRange::new(date(2023, 1, 5), date(2023, 1, 10)),
            DateRange::new(date(2023, 1, 15), date(2023, 1, 20)),
            DateRange::new(date(2023, 1, 25), date(2023, 1, 30))
        ]
    )]
    #[case(
        "No gaps when the range covers the entire period",
        vec![DateRange::new(date(2023, 1, 5), date(2023, 1, 30))],
        date(2023, 1, 5),
        date(2023, 1, 30),
        vec![]
    )]
    #[case(
        "Empty input results in one large range",
        vec![],
        date(2023, 1, 5),
        date(2023, 1, 30),
        vec![
            DateRange::new(date(2023, 1, 5), date(2023, 1, 30))
        ]
    )]
    #[case(
        "Multiple gaps with complex intervals",
        vec![
            DateRange::new(date(2023, 1, 10), date(2023, 1, 15)),
            DateRange::new(date(2023, 1, 18), date(2023, 1, 20))
        ],
        date(2023, 1, 5),
        date(2023, 1, 25),
        vec![
            DateRange::new(date(2023, 1, 5), date(2023, 1, 10)),
            DateRange::new(date(2023, 1, 15), date(2023, 1, 18)),
            DateRange::new(date(2023, 1, 20), date(2023, 1, 25))
        ]
    )]
    fn test_invert_ranges_in_boundary(
        #[case] scenario: &'static str,
        #[case] ranges: Vec<DateRange>,
        #[case] start: NaiveDate,
        #[case] end: NaiveDate,
        #[case] expected: Vec<DateRange>,
    ) {
        let result = invert_ranges_in_boundary(&ranges, start, end);
        assert_eq!(result, expected, "{scenario}");
    }
}
