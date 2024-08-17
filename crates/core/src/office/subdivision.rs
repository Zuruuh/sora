use std::fmt::Display;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{user::UserId, Object};

use super::{
    Address, AvailablePositions, AvailablePositionsError, OfficeId, PositionPrice, Surface,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubdivisionId(pub(self) Uuid);

impl Display for SubdivisionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "spl-{}", self.0)
    }
}

impl SubdivisionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

#[derive(Debug)]
pub struct OfficeSubdivision {
    id: SubdivisionId,
    created_at: DateTime<Utc>,
    name: String,
    address: Address,
    owner: UserId,
    available_positions: SubdividedAvailablePositions,
    position_price_per_month: PositionPrice,
    parent_office: OfficeId,
}

impl OfficeSubdivision {
    pub fn new(
        name: String,
        address: Address,
        owner: UserId,
        available_positions: SubdividedAvailablePositions,
        position_price_per_month: PositionPrice,
        parent_office: OfficeId,
    ) -> Self {
        Self {
            id: SubdivisionId::new(),
            created_at: Utc::now(),
            name,
            address,
            owner,
            available_positions,
            position_price_per_month,
            parent_office,
        }
    }
}

impl Object for OfficeSubdivision {
    fn get_uuid(&self) -> &Uuid {
        &self.id.0
    }

    fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubdividedSurface(pub Surface);

impl SubdividedSurface {
    pub fn from_surface(
        surface: Surface,
        square_meters: u16,
    ) -> Result<Self, SubdividedSurfaceError> {
        use SubdividedSurfaceError::*;

        if square_meters > surface.to_square_meters() {
            return Err(CannotBeLargerThanParent);
        }

        Ok(Self(Surface::from_square_meters(square_meters)))
    }
}

#[derive(PartialEq, thiserror::Error, Debug)]
pub enum SubdividedSurfaceError {
    #[error("A subdivided surface cannot be larger than the initial one")]
    CannotBeLargerThanParent,
    // TODO: Add a minimum limit for surfaces (cannot have a surface of one square meter)
    // #[error("The surface left after the division will be of {left_surface}m², but it should be of at least {limit}m²")]
    // LeftSurfaceCannotBeSmallerThanMinimum {left_surface: u16},
}

#[cfg(test)]
mod subdivided_surface_test {
    use crate::office::{
        subdivision::{SubdividedSurface, SubdividedSurfaceError},
        Surface,
    };
    use rstest::rstest;

    #[rstest]
    #[case(32, 34)]
    pub fn with_value_larger_than_parent(#[case] parent_surface: u16, #[case] subsurface: u16) {
        let surface = Surface::from_square_meters(parent_surface);

        assert_eq!(
            Err(SubdividedSurfaceError::CannotBeLargerThanParent),
            SubdividedSurface::from_surface(surface, subsurface)
        );
    }

    #[rstest]
    #[case(32, 16)]
    pub fn with_valid_value(#[case] parent_surface: u16, #[case] subsurface: u16) {
        let surface = Surface::from_square_meters(parent_surface);

        assert_eq!(
            Ok(SubdividedSurface(Surface::from_square_meters(subsurface))),
            SubdividedSurface::from_surface(surface, subsurface),
        );
    }
}

#[derive(Debug)]
pub struct SubdividedAvailablePositions {
    pub(self) subdivided_surface: SubdividedSurface,
    pub(self) available_positions: u16,
}

impl SubdividedAvailablePositions {
    pub fn new(
        available_positions: u16,
        surface: SubdividedSurface,
        parent_available_positions: &AvailablePositions,
    ) -> Result<Self, SubdividedAvailablePositionsError> {
        use SubdividedAvailablePositionsError::*;

        if available_positions >= parent_available_positions.get_available_positions() {
            return Err(CannotBeLargerThanParent);
        }

        // Apply validation
        let _ = AvailablePositions::new(available_positions, surface.0.clone())
            .map_err(|err| AvailablePositionsBubbledError(err))?;

        Ok(Self {
            subdivided_surface: surface,
            available_positions,
        })
    }
}

#[derive(PartialEq, thiserror::Error, Debug)]
pub enum SubdividedAvailablePositionsError {
    #[error("A subdivided office cannot have more available positions than it's parent")]
    CannotBeLargerThanParent,
    #[error(transparent)]
    AvailablePositionsBubbledError(#[from] AvailablePositionsError),
}

#[cfg(test)]
mod subdivided_available_positions_test {
    use super::*;
    use crate::office::{AvailablePositions, Surface};
    use rstest::rstest;

    #[rstest]
    #[case(50, 45, SubdividedAvailablePositionsError::CannotBeLargerThanParent)]
    #[case(
        41,
        80,
        SubdividedAvailablePositionsError::AvailablePositionsBubbledError(AvailablePositionsError::TooBigForGivenSurface { max_positions_for_given_surface: 25 })
    )]
    #[case(
        12,
        85,
        SubdividedAvailablePositionsError::AvailablePositionsBubbledError(
            AvailablePositionsError::OutOfBounds
        )
    )]
    pub fn with_invalid_positions(
        #[case] available_positions: u16,
        #[case] parent_available_positions: u16,
        #[case] expected_error: SubdividedAvailablePositionsError,
    ) {
        let parent_surface = Surface::from_square_meters(120);
        let parent_positions =
            AvailablePositions::new(parent_available_positions, parent_surface.clone()).unwrap();

        let subdivided_available_positions = SubdividedAvailablePositions::new(
            available_positions,
            SubdividedSurface::from_surface(parent_surface, 40).unwrap(),
            &parent_positions,
        );

        assert!(subdivided_available_positions.is_err());

        let subdivided_available_positions = subdivided_available_positions.unwrap_err();

        assert_eq!(expected_error, subdivided_available_positions);
    }

    #[test]
    pub fn with_valid_position() {
        let parent_surface = Surface::from_square_meters(512);
        let parent_positions = AvailablePositions::new(90, parent_surface.clone()).unwrap();

        let subdivided_surface = SubdividedSurface::from_surface(parent_surface, 300).unwrap();
        let subdivided_available_positions =
            SubdividedAvailablePositions::new(45, subdivided_surface.clone(), &parent_positions);

        assert!(subdivided_available_positions.is_ok());

        let subdivided_available_positions = subdivided_available_positions.unwrap();
        assert_eq!(45, subdivided_available_positions.available_positions);
        assert_eq!(
            subdivided_surface,
            subdivided_available_positions.subdivided_surface
        );
    }
}
