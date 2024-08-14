use std::fmt::Display;

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::{user::UserId, Object};

use super::{OfficeId, SubdivisionId};

pub struct ContractId(Uuid);

impl Display for ContractId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "agr-{}", self.0)
    }
}

impl ContractId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

pub const CONTRACT_DURATION_MINIMUM_DAYS: i64 = 30 * 4;

#[derive(Debug, PartialEq)]
pub struct ContractDuration {
    start: NaiveDate,
    end: NaiveDate,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ContractDurationError {
    #[error("A contract must last at least {CONTRACT_DURATION_MINIMUM_DAYS} days")]
    TooShort,
}

impl ContractDuration {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Result<Self, ContractDurationError> {
        use ContractDurationError::*;
        let days = end.signed_duration_since(start).num_days();

        if CONTRACT_DURATION_MINIMUM_DAYS > days {
            return Err(TooShort);
        }

        Ok(Self { end, start })
    }
}

#[cfg(test)]
mod duration_test {
    use chrono::Datelike;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case((01, 01), (05, 01))]
    #[case((01, 01), (12, 31))]
    pub fn test_valid_duration(#[case] start: (u32, u32), #[case] end: (u32, u32)) {
        let duration = ContractDuration::new(
            NaiveDate::from_ymd_opt(2024, start.0, start.1).unwrap(),
            NaiveDate::from_ymd_opt(2024, end.0, end.1).unwrap(),
        );

        assert!(duration.is_ok());
        let duration = duration.unwrap();

        assert_eq!(start.0, duration.start.month());
        assert_eq!(start.1 - 1, duration.start.day0());
        assert_eq!(end.0, duration.end.month());
        assert_eq!(end.1 - 1, duration.end.day0());
    }

    #[rstest]
    #[case((01, 01), (04, 29))]
    #[case((01, 01), (01, 01))]
    #[case((01, 01), (01, 02))]
    pub fn test_invalid_duration(#[case] start: (u32, u32), #[case] end: (u32, u32)) {
        let duration = ContractDuration::new(
            NaiveDate::from_ymd_opt(2024, start.0, start.1).unwrap(),
            NaiveDate::from_ymd_opt(2024, end.0, end.1).unwrap(),
        );

        assert_eq!(Err(ContractDurationError::TooShort), duration);
    }
}

pub struct Contract {
    id: ContractId,
    created_at: DateTime<Utc>,
    host: UserId,
    guest: UserId,
    office: ContractOffice,
    rent: u32,
    duration: ContractDuration,
}

pub enum ContractOffice {
    Office(OfficeId),
    Subdivision(SubdivisionId),
}

impl Object for Contract {
    fn get_id(&self) -> &Uuid {
        &self.id.0
    }

    fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

impl Contract {
    pub fn new(
        host: UserId,
        guest: UserId,
        office: ContractOffice,
        rent: u32,
        duration: ContractDuration,
    ) -> Self {
        Self {
            id: ContractId::new(),
            created_at: Utc::now(),
            host,
            guest,
            office,
            rent,
            duration,
        }
    }
}
