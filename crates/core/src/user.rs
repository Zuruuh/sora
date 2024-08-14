use chrono::{DateTime, Utc};
use std::fmt::Display;
use uuid::Uuid;

use super::{office::OfficeId, Object};

pub struct UserId(Uuid);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "usr-{}", self.0)
    }
}

impl UserId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

// Making the assumption an user can be both an host and a guest at the same time
pub struct User {
    id: UserId,
    created_at: DateTime<Utc>,
    first_name: String,
    last_name: String,
    managed_offices: Vec<OfficeId>,
    used_offices: Vec<OfficeId>,
}

impl Object for User {
    fn get_id(&self) -> &Uuid {
        &self.id.0
    }

    fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

impl User {
    pub fn new(
        first_name: String,
        last_name: String,
        managed_offices: Vec<OfficeId>,
        used_offices: Vec<OfficeId>,
    ) -> Self {
        Self {
            id: UserId::new(),
            created_at: Utc::now(),
            first_name,
            last_name,
            managed_offices,
            used_offices,
        }
    }
}
