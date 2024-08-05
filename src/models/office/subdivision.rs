use std::fmt::Display;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{user::UserId, Object};

use super::{
    Address, AvailablePositions, AvailablePositionsError, OfficeId, PositionPrice, Surface,
};

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
    fn get_id(&self) -> &Uuid {
        &self.id.0
    }

    fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

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

pub struct SubdividedAvailablePositions {
    pub(self) subdivided_surface: SubdividedSurface,
    pub(self) available_positions: u16,
}

impl SubdividedAvailablePositions {
    pub fn new(
        available_positions: u16,
        surface: SubdividedSurface,
        parent_available_positions: &mut AvailablePositions,
    ) -> Result<Self, SubdividedAvailablePositionsError> {
        use SubdividedAvailablePositionsError::*;

        if available_positions > parent_available_positions.get_available_positions() {
            return Err(CannotBeLargerThanParent);
        }

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
