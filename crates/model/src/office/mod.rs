use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::{id::Identifier, model_id, user::UserId, Object};

model_id!(RealOfficeId, "ofc");
model_id!(OfficeSplitId, "spl");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfficeId {
    RealOfficeId(RealOfficeId),
    OfficeSplit(OfficeSplitId),
}

impl Display for OfficeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OfficeId::RealOfficeId(id) => id.fmt(f),
            OfficeId::OfficeSplit(id) => id.fmt(f),
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

        if latitude < -90.0 || latitude > 90.0 {
            return Err(LatitudeOutOfBounds(latitude));
        }

        if longitude < -180.0 || longitude > 180.0 {
            return Err(LongitudeOutOfBounds(longitude));
        }

        if let Some(error) = validate_available_positions_with_surface(available_positions, surface)
        {
            return Err(AvailablePositionsError(error));
        }

        if position_price < 30000 || position_price > 80000 {
            return Err(PositionPriceOutOfBounds(position_price));
        }

        Ok(Self {
            id: OfficeId::RealOfficeId(RealOfficeId::new()),
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

    /// The sum of `splits[*].available_positions` must be equal to `self.available_positions`
    /// Same for `self.surface`
    pub fn split(&self, splits: Vec<OfficeSplit>) -> Result<Vec<Self>, OfficeSplitError> {
        use OfficeSplitError::*;

        let parent_office_id = self
            .is_real_office()
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

    fn is_real_office(&self) -> Result<RealOfficeId, ()> {
        match self.id {
            OfficeId::RealOfficeId(id) => Ok(id),
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

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(65, 7)]
    #[case(122, 7)]
    #[case(55, 8)]
    fn test_surface_position_constraints(
        #[case] surface: usize,
        #[case] expected_count_per_square_meters: usize,
    ) {
        assert_eq!(
            expected_count_per_square_meters,
            position_count_constraint_for_surface(surface).per_square_meter
        );
    }
}

#[derive(Debug)]
pub struct PositionsConstraints {
    positions: usize,
    per_square_meter: usize,
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
        Ok(Self {
            available_positions,
            surface,
        })
    }
}

fn validate_available_positions_with_surface(
    available_positions: usize,
    surface: usize,
) -> Option<AvailablePositionsError> {
    use AvailablePositionsError::*;

    if available_positions < 40 || available_positions > 180 {
        return Some(AvailablePositionsOutOfBounds(available_positions));
    }

    let positions_constraints = position_count_constraint_for_surface(surface);

    let positions_batch_count =
        (surface as f32 / positions_constraints.per_square_meter as f32).floor() as usize;

    let max_positions_for_given_surface = positions_batch_count * positions_constraints.positions;

    if available_positions > max_positions_for_given_surface {
        return Some(TooMuchAvailablePositionsForGivenSurface {
            available_positions,
            max_positions_for_given_surface,
        });
    }

    None
}

fn position_count_constraint_for_surface(surface: usize) -> PositionsConstraints {
    PositionsConstraints {
        positions: 5,
        per_square_meter: if surface > 60 { 7 } else { 8 },
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AvailablePositionsError {
    #[error("Given available positions ({0}) is out of bounds.")]
    AvailablePositionsOutOfBounds(usize),
    #[error("Given available positions {available_positions} is greater than the maximum value computed for given surface ({max_positions_for_given_surface}).")]
    TooMuchAvailablePositionsForGivenSurface {
        available_positions: usize,
        max_positions_for_given_surface: usize,
    },
}

impl Object for Office {
    fn uuid(&self) -> &uuid::Uuid {
        match &self.id {
            OfficeId::RealOfficeId(id) => &id.0,
            OfficeId::OfficeSplit(id) => &id.0,
        }
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}
