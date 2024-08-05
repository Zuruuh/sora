use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod address;
pub mod office;
pub mod user;

pub trait Object {
    fn get_id<'a>(&'a self) -> &'a Uuid;

    fn get_string_id(&self) -> String {
        self.get_id().to_string()
    }

    fn get_created_at<'a>(&'a self) -> &'a DateTime<Utc>;
}
