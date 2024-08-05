pub const POSITION_PRICE_MIN_IN_CENTS: u32 = 300;
pub const POSITION_PRICE_MAX_IN_CENTS: u32 = 800;

#[derive(Eq, PartialEq, Debug)]
pub struct PositionPrice {
    cents: u32,
}

impl PositionPrice {
    pub fn from_cents(cents: u32) -> Result<Self, PositionPriceError> {
        use PositionPriceError::*;

        if cents < POSITION_PRICE_MIN_IN_CENTS || cents > POSITION_PRICE_MAX_IN_CENTS {
            return Err(OutOfBounds);
        }

        Ok(Self { cents })
    }

    pub fn to_cents(&self) -> u32 {
        self.cents
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum PositionPriceError {
    #[error("Position price should be between {} and {}", (POSITION_PRICE_MIN_IN_CENTS as f32) / 100.0, (POSITION_PRICE_MAX_IN_CENTS as f32) / 100.0)]
    OutOfBounds,
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(250)]
    #[case(850)]
    pub fn with_invalid_values(#[case] cents: u32) {
        let price = PositionPrice::from_cents(cents);

        assert_eq!(Err(PositionPriceError::OutOfBounds), price)
    }

    #[rstest]
    #[case(300)]
    #[case(800)]
    #[case(500)]
    pub fn with_valid_values(#[case] cents: u32) {
        let price = PositionPrice::from_cents(cents);

        assert!(price.is_ok());

        let price = price.unwrap();

        assert_eq!(cents, price.to_cents());
    }
}
