    use chrono::NaiveDate;


#[derive(Debug)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DateRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        Self { start, end }
    }

    pub fn overlap(&self, other: &Self) -> bool {
        self.start <= other.end && self.end >= other.start
    }

    pub fn is_contained_in(&self, other: &Self) -> bool {
        other.start <= self.start && other.end >= self.end
    }
}

#[cfg(test)]
mod tests {
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
