use chrono::{DateTime, Utc};
use std::{collections::BTreeMap, fmt::Display};
use uuid::Uuid;

use crate::office::{Address, AvailablePositions, Office, PositionPrice};

use super::{office::OfficeId, Object};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserId(Uuid);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "usr-{}", self.0)
    }
}

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

// Making the assumption an user can be both an host and a guest at the same time
#[derive(Debug)]
pub struct User {
    id: UserId,
    created_at: DateTime<Utc>,
    first_name: String,
    last_name: String,
    // TODO: delete
    offices: BTreeMap<OfficeId, UserOfficeRelation>,
}

// A single user cannot be both a guest and a host of the same office
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserOfficeRelation {
    Guest,
    Host,
}

impl Object for User {
    fn get_uuid(&self) -> &Uuid {
        &self.id.0
    }

    fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

impl User {
    pub fn get_id(&self) -> &UserId {
        &self.id
    }

    pub fn new(first_name: String, last_name: String) -> Self {
        Self {
            id: UserId::new(),
            created_at: Utc::now(),
            first_name,
            last_name,
            offices: BTreeMap::new(),
        }
    }

    pub fn create_managed_office(
        &mut self,
        name: String,
        address: Address,
        available_positions: AvailablePositions,
        position_price_per_month: PositionPrice,
    ) -> Office {
        let office = Office::new(
            name,
            address,
            self.id,
            available_positions,
            position_price_per_month,
        );

        self.offices.insert(office.id, UserOfficeRelation::Host);

        office
    }

    pub fn has_managed_offices(&self) -> bool {
        !self
            .offices
            .iter()
            .any(|(_, relation)| matches!(relation, UserOfficeRelation::Host))
    }

    pub fn use_office(&mut self, office: OfficeId) {
        self.offices.insert(office, UserOfficeRelation::Guest);
    }
}
