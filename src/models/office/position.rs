use super::Surface;

pub const POSITION_PRICE_MIN_IN_CENTS: u32 = 300;
pub const POSITION_PRICE_MAX_IN_CENTS: u32 = 800;

#[derive(Eq, PartialEq, Debug)]
pub struct PositionPrice {
    cents: u32,
}

impl PositionPrice {
    pub fn from_cents(cents: u32) -> Result<Self, PositionPriceError> {
        use PositionPriceError::*;

        if cents < POSITION_PRICE_MIN_IN_CENTS || cents > POSITION_PRICE_MAX_IN_CENTS {
            return Err(OutOfBounds);
        }

        Ok(Self { cents })
    }

    pub fn to_cents(&self) -> u32 {
        self.cents
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum PositionPriceError {
    #[error("Position price should be between {} and {}", (POSITION_PRICE_MIN_IN_CENTS as f32) / 100.0, (POSITION_PRICE_MAX_IN_CENTS as f32) / 100.0)]
    OutOfBounds,
}

#[cfg(test)]
mod price_tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(250)]
    #[case(850)]
    pub fn with_invalid_values(#[case] cents: u32) {
        let price = PositionPrice::from_cents(cents);

        assert_eq!(Err(PositionPriceError::OutOfBounds), price)
    }

    #[rstest]
    #[case(300)]
    #[case(800)]
    #[case(500)]
    pub fn with_valid_values(#[case] cents: u32) {
        let price = PositionPrice::from_cents(cents);

        assert!(price.is_ok());

        let price = price.unwrap();

        assert_eq!(cents, price.to_cents());
    }
}

pub const AVAILABLE_POSITIONS_MIN: u16 = 40;
pub const AVAILABLE_POSITIONS_MAX: u16 = 180;

#[derive(Eq, PartialEq, Debug)]
pub struct AvailablePositions(pub(self) u16);

impl AvailablePositions {
    pub fn new(
        available_positions: u16,
        surface: &Surface,
    ) -> Result<Self, AvailablePositionsError> {
        use AvailablePositionsError::*;

        if available_positions < AVAILABLE_POSITIONS_MIN
            || available_positions > AVAILABLE_POSITIONS_MAX
        {
            return Err(OutOfBounds);
        }

        let positions_constraints = surface.get_positions_constraints();

        let positions_batch_count = (surface.to_square_meters() as f32
            / positions_constraints.per_square_meter as f32)
            .floor() as u16;

        let max_positions_for_given_surface =
            positions_batch_count * positions_constraints.positions as u16;

        if available_positions > max_positions_for_given_surface {
            return Err(TooBigForGivenSurface {
                max_positions_for_given_surface,
            });
        }

        Ok(Self(available_positions))
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum AvailablePositionsError {
    #[error(
        "There should be at least {AVAILABLE_POSITIONS_MIN} and at most {AVAILABLE_POSITIONS_MAX} available positions"
    )]
    OutOfBounds,
    #[error(
        "With the given surface, the max positions count is {max_positions_for_given_surface}"
    )]
    TooBigForGivenSurface {
        max_positions_for_given_surface: u16,
    },
}

#[cfg(test)]
mod available_position_tests {
    use super::{AvailablePositionsError::*, *};
    use rstest::rstest;

    #[rstest]
    #[case(39)]
    #[case(181)]
    pub fn with_out_of_bounds_count(#[case] available_positions: u16) {
        let positions =
            AvailablePositions::new(available_positions, &Surface::from_square_meters(40));

        assert_eq!(Err(OutOfBounds), positions);
    }

    #[rstest]
    #[case(106, 150, 105)]
    #[case(55, 75, 50)]
    pub fn with_count_too_large_for_given_surface(
        #[case] available_positions: u16,
        #[case] surface: u16,
        #[case] maximum_positions: u16,
    ) {
        let positions =
            AvailablePositions::new(available_positions, &Surface::from_square_meters(surface));

        assert_eq!(
            Err(TooBigForGivenSurface {
                max_positions_for_given_surface: maximum_positions
            }),
            positions
        );
    }

    #[rstest]
    #[case(105, 150)]
    #[case(50, 75)]
    pub fn with_valid_positions(#[case] available_positions: u16, #[case] surface: u16) {
        let positions =
            AvailablePositions::new(available_positions, &Surface::from_square_meters(surface));

        assert_eq!(Ok(AvailablePositions(available_positions)), positions);
    }
}
