pub(super) fn validate_available_positions_for_surface(
    available_positions: usize,
    surface: usize,
) -> Option<AvailablePositionsError> {
    use AvailablePositionsError::*;

    if !(40..180).contains(&available_positions) {
        return Some(AvailablePositionsOutOfBounds(available_positions));
    }

    let positions_constraints = position_count_constraint_for_surface(surface);

    let positions_batch_count =
        (surface as f32 / positions_constraints.per_square_meter as f32).floor() as usize;

    let max_positions_for_given_surface = positions_batch_count * positions_constraints.positions;

    if available_positions > max_positions_for_given_surface {
        return Some(TooMuchAvailablePositionsForGivenSurface {
            available_positions,
            max_positions_for_given_surface,
        });
    }

    None
}

#[cfg(test)]
mod available_positions_for_surface_test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(39, 40)]
    #[case(181, 40)]
    pub fn with_out_of_bounds_count(#[case] available_positions: usize, #[case] surface: usize) {
        let error = validate_available_positions_for_surface(available_positions, surface);

        assert_eq!(
            Some(AvailablePositionsError::AvailablePositionsOutOfBounds(
                available_positions
            )),
            error
        );
    }

    #[rstest]
    #[case(106, 150, 105)]
    #[case(55, 75, 50)]
    pub fn with_count_too_large_for_given_surface(
        #[case] available_positions: usize,
        #[case] surface: usize,
        #[case] maximum_positions: usize,
    ) {
        let error = validate_available_positions_for_surface(available_positions, surface);

        assert_eq!(
            Some(
                AvailablePositionsError::TooMuchAvailablePositionsForGivenSurface {
                    available_positions,
                    max_positions_for_given_surface: maximum_positions,
                }
            ),
            error
        );
    }

    #[rstest]
    #[case(105, 150)]
    #[case(50, 75)]
    pub fn with_valid_positions(#[case] available_positions: usize, #[case] surface: usize) {
        let error = validate_available_positions_for_surface(available_positions, surface);

        assert!(error.is_none());
    }
}

#[derive(Debug)]
pub struct PositionsConstraints {
    positions: usize,
    per_square_meter: usize,
}

pub fn position_count_constraint_for_surface(surface: usize) -> PositionsConstraints {
    PositionsConstraints {
        positions: 5,
        per_square_meter: if surface > 60 { 7 } else { 8 },
    }
}

#[cfg(test)]
mod position_count_constraint_test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(65, 7)]
    #[case(122, 7)]
    #[case(55, 8)]
    fn test_surface_position_constraints(
        #[case] surface: usize,
        #[case] expected_count_per_square_meters: usize,
    ) {
        assert_eq!(
            expected_count_per_square_meters,
            position_count_constraint_for_surface(surface).per_square_meter
        );
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum AvailablePositionsError {
    #[error("Given available positions ({0}) is out of bounds.")]
    AvailablePositionsOutOfBounds(usize),
    #[error("Given available positions {available_positions} is greater than the maximum value computed for given surface ({max_positions_for_given_surface}).")]
    TooMuchAvailablePositionsForGivenSurface {
        available_positions: usize,
        max_positions_for_given_surface: usize,
    },
}
