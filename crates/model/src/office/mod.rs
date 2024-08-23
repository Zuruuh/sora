use std::fmt::{Debug, Display};

use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::{validate_available_positions_for_surface, AvailablePositionsError};

use crate::{id::Identifier, model_id, user::UserId, Object};

mod validator;

model_id!(RealOfficeId, "ofc");
model_id!(OfficeSplitId, "spl");

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OfficeId {
    RealOffice(RealOfficeId),
    OfficeSplit(OfficeSplitId),
}

impl Display for OfficeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OfficeId::RealOffice(id) => write!(f, "{id}"),
            OfficeId::OfficeSplit(id) => write!(f, "{id}"),
        }
    }
}

impl Debug for OfficeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl OfficeId {
    pub fn uuid(&self) -> &Uuid {
        match self {
            OfficeId::RealOffice(id) => id.uuid(),
            OfficeId::OfficeSplit(id) => id.uuid(),
        }
    }
}

#[derive(Debug, derive_getters::Getters)]
pub struct Office {
    id: OfficeId,
    name: String,
    address: String,
    latitude: f32,
    longitude: f32,
    owner: UserId,
    #[getter(skip)]
    created_at: DateTime<Utc>,
    available_positions: usize,
    surface: usize,
    position_price: usize,
    parent_office: Option<RealOfficeId>,
}

impl Office {
    pub fn new_real(
        name: String,
        address: String,
        latitude: f32,
        longitude: f32,
        owner: UserId,
        available_positions: usize,
        surface: usize,
        position_price: usize,
    ) -> Result<Self, OfficeError> {
        use OfficeError::*;

        if !(-90.0..90.0).contains(&latitude) {
            return Err(LatitudeOutOfBounds(latitude));
        }

        if !(-180.0..180.0).contains(&longitude) {
            return Err(LongitudeOutOfBounds(longitude));
        }

        if let Some(error) = validate_available_positions_for_surface(available_positions, surface)
        {
            return Err(AvailablePositionsError(error));
        }

        if !(30000..80000).contains(&position_price) {
            return Err(PositionPriceOutOfBounds(position_price));
        }

        Ok(Self {
            id: OfficeId::RealOffice(RealOfficeId::new()),
            created_at: Utc::now(),
            name,
            address,
            latitude,
            longitude,
            owner,
            available_positions,
            surface,
            position_price,
            parent_office: None,
        })
    }

    pub fn new_unchecked(
        id: OfficeId,
        created_at: DateTime<Utc>,
        name: String,
        address: String,
        latitude: f32,
        longitude: f32,
        owner: UserId,
        available_positions: usize,
        surface: usize,
        position_price: usize,
        parent_office: Option<RealOfficeId>,
    ) -> Self {
        Self {
            id,
            created_at,
            name,
            address,
            latitude,
            longitude,
            owner,
            available_positions,
            surface,
            position_price,
            parent_office,
        }
    }

    /// The sum of `splits[*].available_positions` must be equal to `self.available_positions`
    /// Same for `self.surface`
    pub fn split(&self, splits: Vec<OfficeSplit>) -> Result<Vec<Self>, OfficeSplitError> {
        use OfficeSplitError::*;

        let parent_office_id = self
            .ensure_is_real_office()
            .map_err(|_| OfficeSubdivisionCannotBeSubdivided)?;

        let total_available_positions: usize =
            splits.iter().map(|split| split.available_positions).sum();
        let total_surface: usize = splits.iter().map(|split| split.surface).sum();

        if total_surface != self.surface {
            return Err(TotalSurfaceNotMatching {
                given: total_surface,
                expected: self.surface,
            });
        }

        if total_available_positions != self.available_positions {
            return Err(TotalAvailablePositionsNotMatching {
                given: total_available_positions,
                expected: self.available_positions,
            });
        }

        let mut offices = Vec::<Self>::new();

        for split in splits {
            offices.push(Self {
                id: OfficeId::OfficeSplit(OfficeSplitId::new()),
                created_at: Utc::now(),
                name: self.name.clone(),
                address: self.address.clone(),
                latitude: self.latitude,
                longitude: self.longitude,
                owner: self.owner,
                available_positions: split.available_positions,
                surface: split.surface,
                position_price: self.position_price,
                parent_office: Some(parent_office_id),
            });
        }

        Ok(offices)
    }

    fn ensure_is_real_office(&self) -> Result<RealOfficeId, ()> {
        match self.id {
            OfficeId::RealOffice(id) => Ok(id),
            OfficeId::OfficeSplit(_) => Err(()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OfficeSplitError {
    #[error("An office subdivision cannot be sub-divided again")]
    OfficeSubdivisionCannotBeSubdivided,
    #[error(
        "The sum of all given surfaces ({given}) does not match the expected value ({expected})"
    )]
    TotalSurfaceNotMatching { given: usize, expected: usize },
    #[error("The sum of all given available positions ({given}) does not match the expected value ({expected})")]
    TotalAvailablePositionsNotMatching { given: usize, expected: usize },
}

#[derive(Debug, thiserror::Error)]
pub enum OfficeError {
    #[error("Given latitude ({0}) is out of bounds.")]
    LatitudeOutOfBounds(f32),
    #[error("Given longitude ({0}) is out of bounds.")]
    LongitudeOutOfBounds(f32),
    #[error("Given available positions ({0}) is out of bounds.")]
    AvailablePositionsOutOfBounds(usize),
    #[error("Given price per position ({0}) is out of bounds.")]
    PositionPriceOutOfBounds(usize),
    #[error(transparent)]
    AvailablePositionsError(#[from] AvailablePositionsError),
}

#[derive(Debug)]
pub struct OfficeSplit {
    available_positions: usize,
    surface: usize,
}

impl OfficeSplit {
    pub fn new(
        available_positions: usize,
        surface: usize,
    ) -> Result<Self, AvailablePositionsError> {
        if let Some(error) = validate_available_positions_for_surface(available_positions, surface)
        {
            return Err(error);
        }

        Ok(Self {
            available_positions,
            surface,
        })
    }
}

impl Object for Office {
    fn uuid(&self) -> &uuid::Uuid {
        match &self.id {
            OfficeId::RealOffice(id) => &id.0,
            OfficeId::OfficeSplit(id) => &id.0,
        }
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(-193.61427, 63.272813)]
    #[case(193.61427, 63.272813)]
    #[case(123.61427, -93.272813)]
    #[case(123.61427, 93.272813)]
    fn test_invalid_coordinates(#[case] longitude: f32, #[case] latitude: f32) {
        let office = Office::new_real(
            "yo".to_string(),
            "10 my address".to_string(),
            latitude,
            longitude,
            UserId::new(),
            120,
            500,
            30000,
        );

        if let Err(err) = office {
            assert!(matches!(
                err,
                OfficeError::LatitudeOutOfBounds(_) | OfficeError::LongitudeOutOfBounds(_)
            ));
        } else {
            assert!(false);
        }
    }
}
