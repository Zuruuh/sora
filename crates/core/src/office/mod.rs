mod address;
mod contract;
mod position;
mod subdivision;
mod surface;

pub use address::*;
pub use contract::*;
pub use position::*;
pub use subdivision::*;
pub use surface::*;

use std::fmt::Display;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{user::UserId, Object};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug)]
pub struct Office {
    pub(crate) id: OfficeId,
    created_at: DateTime<Utc>,
    name: String,
    address: Address,
    owner: UserId,
    available_positions: AvailablePositions,
    position_price_per_month: PositionPrice,
}

impl Object for Office {
    fn get_uuid(&self) -> &Uuid {
        &self.id.0
    }

    fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

impl Office {
    pub fn get_id(&self) -> &OfficeId {
        &self.id
    }

    pub fn get_available_positions(&self) -> &AvailablePositions {
        &self.available_positions
    }

    pub fn get_position_price_per_month(&self) -> &PositionPrice {
        &self.position_price_per_month
    }

    pub(crate) fn new(
        name: String,
        address: Address,
        owner: UserId,
        available_positions: AvailablePositions,
        position_price_per_month: PositionPrice,
    ) -> Self {
        Self {
            id: OfficeId::new(),
            created_at: Utc::now(),
            name,
            address,
            owner,
            available_positions,
            position_price_per_month,
        }
    }

    /// `parts` Must be at least 2 elements long
    pub fn divide(
        &self,
        parts: Vec<(SubdividedAvailablePositions, PositionPrice)>,
    ) -> Result<Vec<OfficeSubdivision>, OfficeDivisionError> {
        use OfficeDivisionError::*;

        if parts.len() < 2 {
            return Err(MustBeDividedInAtLeastTwoParts);
        }

        let mut subdivisions = Vec::new();

        for (available_positions, position_price_per_month) in parts {
            subdivisions.push(OfficeSubdivision::new(
                self.name.clone(),
                self.address.clone(),
                self.owner,
                available_positions,
                position_price_per_month,
                self.id,
            ));
        }

        Ok(subdivisions)
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OfficeDivisionError {
    #[error("An office must be divided in at least two distinct parts")]
    MustBeDividedInAtLeastTwoParts,
}
