use chrono::{DateTime, NaiveDate, Utc};

use crate::{model_id, office::OfficeId, user::UserId, Object};

model_id!(ContractId, "agr");

#[derive(Debug)]
pub struct Contract {
    id: ContractId,
    created_at: DateTime<Utc>,
    host: UserId,
    guest: UserId,
    office: OfficeId,
    rent: usize,
    start: NaiveDate,
    end: NaiveDate,
}

impl Object for Contract {
    fn uuid(&self) -> &uuid::Uuid {
        &self.id.0
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}
