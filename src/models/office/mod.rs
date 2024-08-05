pub mod address;
pub mod position;
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
    surface: Surface,
    available_positions: AvailablePositions,
    position_price_per_month: PositionPrice,
    // Using IDs here instead of a Box<Self> ensure better memory usage
    // since we don't have to use the heap
    parent_office: Option<OfficeId>,
}

impl Office {
    pub fn new(
        name: String,
        address: Address,
        owner: UserId,
        surface: Surface,
        available_positions: AvailablePositions,
        position_price_per_month: PositionPrice,
        parent_office: Option<OfficeId>,
    ) -> Self {
        Self {
            id: OfficeId::new(),
            created_at: Utc::now(),
            name,
            address,
            owner,
            surface,
            available_positions,
            position_price_per_month,
            parent_office,
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
