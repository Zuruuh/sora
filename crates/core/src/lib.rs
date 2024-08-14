use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod office;
pub mod user;

pub trait Object {
    fn get_id(&self) -> &Uuid;

    fn get_string_id(&self) -> String {
        self.get_id().to_string()
    }

    fn get_created_at(&self) -> &DateTime<Utc>;
}
