pub mod position;
pub use position::*;

use std::fmt::Display;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{address::Address, user::UserId, Object};

pub struct OfficeId(Uuid);

impl Display for OfficeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ofc-{}", self.0)
    }
}

impl OfficeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

pub struct Office {
    id: OfficeId,
    created_at: DateTime<Utc>,
    // todo add validation maybe ?
    name: String,
    address: Address,
    owner: UserId,
    surface: Surface,
    available_positions: AvailablePositions,
    position_price_per_month: PositionPrice,
    // Using IDs here instead of a Box<Self> ensure better memory usage
    // since we don't have to use the heap
    parent_office: Option<OfficeId>,
}

impl Object for Office {
    fn get_id(&self) -> &Uuid {
        &self.id.0
    }

    fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

pub const AVAILABLE_POSITIONS_MIN: u16 = 40;
pub const AVAILABLE_POSITIONS_MAX: u16 = 180;

#[derive(Eq, PartialEq, Debug)]
pub struct AvailablePositions(u16);

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

        if max_positions_for_given_surface > available_positions {
            return Err(TooBigForGivenSurface {
                max_positions_for_given_surface,
            });
        }

        Ok(Self(available_positions))
    }
}

#[derive(Debug, thiserror::Error)]
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

#[derive(Eq, PartialEq, Debug)]
pub struct Surface(u16);

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
    positions: u8,
    per_square_meter: u8,
}
