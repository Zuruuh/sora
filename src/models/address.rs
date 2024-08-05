pub struct Address {
    readable: String,
    coordinates: Coordinates,
}

impl Address {
    pub fn new(readable: String, coordinates: Coordinates) -> Self {
        Self {
            readable,
            coordinates,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Coordinates {
    longitude: f32,
    latitude: f32,
}

pub const LONGITUDE_BOUNDARY: f32 = 180.0;
pub const LATITUDE_BOUNDARY: f32 = 90.0;

impl Coordinates {
    pub fn new(longitude: f32, latitude: f32) -> Result<Self, CoordinatesError> {
        use CoordinatesError::*;

        match (longitude, latitude) {
            (lon, _) if lon < -LONGITUDE_BOUNDARY || lon > LONGITUDE_BOUNDARY => {
                Err(LongitudeOutOfBound)
            }
            (_, lat) if lat < -LATITUDE_BOUNDARY || lat > LATITUDE_BOUNDARY => {
                Err(LatitudeOutOfBound)
            }
            (longitude, latitude) => Ok(Self {
                longitude,
                latitude,
            }),
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum CoordinatesError {
    #[error("Longitude must be between -{LONGITUDE_BOUNDARY} and {LONGITUDE_BOUNDARY}")]
    LongitudeOutOfBound,
    #[error("Latitude must be between -{LATITUDE_BOUNDARY} and {LATITUDE_BOUNDARY}")]
    LatitudeOutOfBound,
}

#[cfg(test)]
mod test {
    use super::{CoordinatesError::*, *};
    use rstest::rstest;

    #[rstest]
    #[case(181.0)]
    #[case(-181.0)]
    pub fn create_coordinates_with_invalid_longitude(#[case] longitude: f32) {
        let coordinates = Coordinates::new(longitude, 45.0);

        assert_eq!(Err(LongitudeOutOfBound), coordinates);
    }

    #[rstest]
    #[case(91.0)]
    #[case(91.0)]
    pub fn create_coordinates_with_invalid_latitude(#[case] latitude: f32) {
        let coordinates = Coordinates::new(45.0, latitude);

        assert_eq!(Err(LatitudeOutOfBound), coordinates);
    }

    #[test]
    pub fn create_coordinates() {
        let longitude = 45.158715;
        let latitude = -57.84721897;
        let coordinates = Coordinates::new(longitude, latitude);

        assert!(coordinates.is_ok());
        let coordinates = coordinates.unwrap();

        assert_eq!(coordinates.latitude, latitude);
        assert_eq!(coordinates.longitude, longitude);
    }
}
