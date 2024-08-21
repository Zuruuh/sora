use chrono::{DateTime, Utc};

use crate::{id::Identifier, model_id, Object};

model_id!(UserId, "usr");

/// Making the assumption an user can be both a host and a guest at the same time
#[derive(Debug, derive_getters::Getters)]
pub struct User {
    id: UserId,
    #[getter(skip)]
    created_at: DateTime<Utc>,
    first_name: String,
    last_name: String,
}

impl Object for User {
    fn uuid(&self) -> &uuid::Uuid {
        &self.id.0
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

impl User {
    pub fn new(first_name: String, last_name: String) -> Self {
        Self {
            id: UserId::new(),
            created_at: Utc::now(),
            first_name,
            last_name,
        }
    }

    pub fn new_unchecked(
        id: UserId,
        created_at: DateTime<Utc>,
        first_name: String,
        last_name: String,
    ) -> Self {
        Self {
            id,
            created_at,
            first_name,
            last_name,
        }
    }
}
