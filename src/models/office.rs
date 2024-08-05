use std::fmt::Display;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{address::Address, user::UserId};

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
    name: String,
    address: Address,
    owner: UserId,
    surface: Surface,
    available_positions: AvailablePositions,
    position_price_per_month: PositionPrice,
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

        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AvailablePositionsError {
    #[error(
        "There should be at least {AVAILABLE_POSITIONS_MIN} and at most {AVAILABLE_POSITIONS_MAX} available positions"
    )]
    OutOfBounds,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Surface(u16);

impl Surface {
    pub fn from_square_meters(square_meters: u16) -> Self {
        Self(square_meters)
    }
}

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

#[derive(thiserror::Error, Debug)]
pub enum PositionPriceError {
    #[error("Position price should be between {} and {}", (POSITION_PRICE_MIN_IN_CENTS as f32) / 100.0, (POSITION_PRICE_MAX_IN_CENTS as f32) / 100.0)]
    OutOfBounds,
}
