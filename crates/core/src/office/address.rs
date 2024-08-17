#[derive(Debug, Clone)]
pub struct Address {
    readable_name: String,
    coordinates: Coordinates,
}

impl Address {
    pub fn new(readable_name: String, coordinates: Coordinates) -> Self {
        Self {
            readable_name,
            coordinates,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
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
                Err(LongitudeOutOfBound(lon))
            }
            (_, lat) if lat < -LATITUDE_BOUNDARY || lat > LATITUDE_BOUNDARY => {
                Err(LatitudeOutOfBound(lat))
            }
            (longitude, latitude) => Ok(Self {
                longitude,
                latitude,
            }),
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CoordinatesError {
    #[error("Longitude must be between -{LONGITUDE_BOUNDARY} and {LONGITUDE_BOUNDARY}, got {0}")]
    LongitudeOutOfBound(f32),
    #[error("Latitude must be between -{LATITUDE_BOUNDARY} and {LATITUDE_BOUNDARY}, got {0}")]
    LatitudeOutOfBound(f32),
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

        assert_eq!(Err(LongitudeOutOfBound(longitude)), coordinates);
    }

    #[rstest]
    #[case(91.0)]
    #[case(91.0)]
    pub fn create_coordinates_with_invalid_latitude(#[case] latitude: f32) {
        let coordinates = Coordinates::new(45.0, latitude);

        assert_eq!(Err(LatitudeOutOfBound(latitude)), coordinates);
    }

    #[test]
    pub fn create_address() {
        let longitude = 45.158715;
        let latitude = -57.84721897;
        let coordinates = Coordinates::new(longitude, latitude);

        assert!(coordinates.is_ok());
        let coordinates = coordinates.unwrap();

        assert_eq!(coordinates.latitude, latitude);
        assert_eq!(coordinates.longitude, longitude);

        let address = Address::new("my_address".to_string(), coordinates.clone());
        assert_eq!("my_address".to_string(), address.readable_name);
        assert_eq!(coordinates, address.coordinates);
    }
}
