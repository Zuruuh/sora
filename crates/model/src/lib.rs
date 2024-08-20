use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod contract;
pub mod id;
pub mod office;
pub mod user;

pub trait Object {
    fn uuid(&self) -> &Uuid;

    fn get_string_id(&self) -> String {
        self.uuid().to_string()
    }

    fn created_at(&self) -> &DateTime<Utc>;
}
