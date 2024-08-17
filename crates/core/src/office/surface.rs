#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Surface(pub(self) u16);

impl Surface {
    pub fn from_square_meters(square_meters: u16) -> Self {
        Self(square_meters)
    }

    pub fn to_square_meters(&self) -> u16 {
        self.0
    }

    pub fn get_positions_constraints(&self) -> PositionsConstraints {
        PositionsConstraints {
            positions: 5,
            per_square_meter: if self.0 > 60 { 7 } else { 8 },
        }
    }
}

#[derive(Debug)]
pub struct PositionsConstraints {
    pub positions: u8,
    pub per_square_meter: u16,
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(65, 7)]
    #[case(122, 7)]
    #[case(55, 8)]
    pub fn test_constraints(#[case] square_meters: u16, #[case] expected_square_meters: u16) {
        let surface = Surface::from_square_meters(square_meters);

        assert_eq!(
            expected_square_meters,
            surface.get_positions_constraints().per_square_meter
        );
    }
}
