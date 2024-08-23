use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::{
    id::Identifier,
    model_id,
    office::{Office, OfficeId},
    user::UserId,
    Object,
};

model_id!(ContractId, "agr");

#[derive(Debug, derive_getters::Getters)]
pub struct Contract {
    id: ContractId,
    #[getter(skip)]
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

impl Contract {
    pub fn new(
        host: UserId,
        guest: UserId,
        office: OfficeId,
        rent: usize,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Self, ContractError> {
        use ContractError::*;
        let contract_length = end.signed_duration_since(start).num_days();

        if CONTRACT_DURATION_MINIMUM_DAYS as i64 > contract_length {
            return Err(TooShort {
                days: contract_length,
            });
        }

        Ok(Self {
            id: ContractId::new(),
            created_at: Utc::now(),
            host,
            guest,
            office,
            rent,
            start,
            end,
        })
    }

    pub fn for_office(
        office: &Office,
        guest: UserId,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Self, ContractError> {
        Self::new(
            *office.owner(),
            guest,
            *office.id(),
            office.available_positions() * office.position_price(),
            start,
            end,
        )
    }

    pub fn new_unchecked(
        id: ContractId,
        created_at: DateTime<Utc>,
        host: UserId,
        guest: UserId,
        office: OfficeId,
        rent: usize,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Self {
        Self {
            id,
            created_at,
            host,
            guest,
            office,
            rent,
            start,
            end,
        }
    }

    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}

pub const CONTRACT_DURATION_MINIMUM_DAYS: usize = 30 * 4;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ContractError {
    #[error("A contract must last at least {CONTRACT_DURATION_MINIMUM_DAYS} days, but tried to create one with {days} days")]
    TooShort { days: i64 },
}

#[cfg(test)]
mod duration_test {
    use chrono::Datelike;
    use rstest::rstest;

    use crate::office::RealOfficeId;

    use super::*;

    fn create_contract(start: NaiveDate, end: NaiveDate) -> Result<Contract, ContractError> {
        Contract::new(
            UserId::new(),
            UserId::new(),
            OfficeId::RealOffice(RealOfficeId::new()),
            30000,
            start,
            end,
        )
    }

    #[rstest]
    #[case((01, 01), (05, 01))]
    #[case((01, 01), (12, 31))]
    pub fn test_valid_duration(#[case] start: (u32, u32), #[case] end: (u32, u32)) {
        let contract = create_contract(
            NaiveDate::from_ymd_opt(2024, start.0, start.1).unwrap(),
            NaiveDate::from_ymd_opt(2024, end.0, end.1).unwrap(),
        );

        if let Ok(contract) = contract {
            assert_eq!(start.0, contract.start.month());
            assert_eq!(start.1 - 1, contract.start.day0());
            assert_eq!(end.0, contract.end.month());
            assert_eq!(end.1 - 1, contract.end.day0());
        } else {
            assert!(false);
        }
    }

    #[rstest]
    #[case((01, 01), (04, 29))]
    #[case((01, 01), (01, 01))]
    #[case((01, 01), (01, 02))]
    pub fn test_invalid_duration(#[case] start: (u32, u32), #[case] end: (u32, u32)) {
        let contract = create_contract(
            NaiveDate::from_ymd_opt(2024, start.0, start.1).unwrap(),
            NaiveDate::from_ymd_opt(2024, end.0, end.1).unwrap(),
        );

        if let Err(err) = contract {
            assert!(matches!(err, ContractError::TooShort { .. }));
        } else {
            assert!(false);
        }
    }
}
