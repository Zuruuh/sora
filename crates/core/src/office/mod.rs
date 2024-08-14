pub mod address;
pub mod position;
pub mod subdivision;
pub mod surface;

pub use address::*;
pub use position::*;
pub use surface::*;

use std::fmt::Display;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{user::UserId, Object};

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
    available_positions: AvailablePositions,
    position_price_per_month: PositionPrice,
}

impl Office {
    pub fn new(
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
}

impl Object for Office {
    fn get_id(&self) -> &Uuid {
        &self.id.0
    }

    fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}
