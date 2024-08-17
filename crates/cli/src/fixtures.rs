use fake::{
    faker::{address::fr_fr::*, name::fr_fr::*},
    Fake,
};
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use sora_core::{
    office::{
        Address, AvailablePositions, Coordinates, Office, OfficeSubdivision, PositionPrice,
        SubdividedAvailablePositions, SubdividedSurface, Surface, LATITUDE_BOUNDARY,
        LONGITUDE_BOUNDARY, POSITION_PRICE_MAX_IN_CENTS, POSITION_PRICE_MIN_IN_CENTS,
    },
    user::{User, UserOfficeRelation},
};
use sqlx::PgPool;

const MINIMUM_USERS_TO_GENERATE: u16 = 2;
const MAXIMUM_USERS_TO_GENERATE: u16 = 6;

pub async fn create_fixtures(pool: &PgPool, rng: &mut ThreadRng) -> color_eyre::Result<()> {
    println!("Creating database fixtures");
    println!("First deleting existing data");
    let affected_rows =
        sqlx::query!("truncate table contracts, subdivided_offices, offices, users")
            .execute(pool)
            .await?
            .rows_affected();

    if affected_rows == 0 {
        println!("No existing data found!");
    } else {
        println!("Deleted {affected_rows} rows!");
    }

    let users_to_generate = rng.gen_range(MINIMUM_USERS_TO_GENERATE..=MAXIMUM_USERS_TO_GENERATE);

    let mut users = Vec::<User>::new();
    let mut offices = Vec::<Office>::new();
    let mut hosts_slots = (users_to_generate as f32 / 2.0).floor() as u8;

    println!("Creating {users_to_generate} users, containing {hosts_slots} hosts");

    for _ in 0..users_to_generate {
        let mut user = User::new(
            FirstName().fake_with_rng(rng),
            LastName().fake_with_rng(rng),
        );

        if hosts_slots > 0 {
            hosts_slots -= 1;

            let office = user.create_managed_office(
                SecondaryAddress().fake_with_rng(rng),
                Address::new(
                    format!(
                        "{} {}, {} {}",
                        BuildingNumber().fake_with_rng::<String, ThreadRng>(rng),
                        StreetName().fake_with_rng::<String, ThreadRng>(rng),
                        CityName().fake_with_rng::<String, ThreadRng>(rng),
                        ZipCode().fake_with_rng::<String, ThreadRng>(rng)
                    ),
                    Coordinates::new(
                        rng.gen_range(-LONGITUDE_BOUNDARY..LONGITUDE_BOUNDARY),
                        rng.gen_range(-LATITUDE_BOUNDARY..LATITUDE_BOUNDARY),
                    )?,
                ),
                AvailablePositions::new(
                    rng.gen_range(80..180),
                    Surface::from_square_meters(rng.gen_range(600..1000)),
                )?,
                PositionPrice::from_cents(
                    rng.gen_range(POSITION_PRICE_MIN_IN_CENTS..POSITION_PRICE_MAX_IN_CENTS),
                )?,
            );

            offices.push(office);
        }

        users.push(user);
    }

    let guests_count: usize = users
        .iter()
        .map(|user| user.has_managed_offices().then(|| 1).unwrap_or_default())
        .sum();

    let mut subdivided_offices = Vec::<OfficeSubdivision>::new();

    while guests_count > (offices.len() + subdivided_offices.len()) {
        println!(
            "Not enough offices for {guests_count} guests (only {} available), subdividing an office",
            offices.len() + subdivided_offices.len()
        );
        let office = match offices.pop() {
            None => unreachable!(),
            Some(office) => office,
        };

        let position_price = *office.get_position_price_per_month();
        let available_positions = *office.get_available_positions();

        let surface = *available_positions.get_surface();

        let subdivided_positions = (0..2)
            .map(|_| {
                SubdividedSurface::from_surface(
                    surface,
                    (surface.to_square_meters() as f32 / 2.0).floor() as u16,
                )
                .unwrap()
            })
            .map(|surface| {
                println!(
                    "{}, {:?}",
                    (available_positions.get_available_positions() as f32 / 2.0).floor() as u16,
                    &available_positions
                );
                SubdividedAvailablePositions::new(
                    (available_positions.get_available_positions() as f32 / 2.0).floor() as u16,
                    surface,
                    &available_positions,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        subdivided_offices.append(
            &mut office
                .divide(
                    subdivided_positions
                        .into_iter()
                        .map(|position| (position, position_price))
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
        );
    }

    dbg!(&subdivided_offices);

    let offices = offices
        .into_iter()
        .map(|office| OfficeOrSubdivision::Office(office))
        .chain(
            subdivided_offices
                .into_iter()
                .map(|subdivided_office| OfficeOrSubdivision::Subdivision(subdivided_office)),
        )
        .collect::<Vec<_>>();

    for user in users.iter_mut() {
        if !user.has_managed_offices() {
            user.use_office(*offices.choose(rng).unwrap().get_id());
        }
    }

    Ok(())
}

enum OfficeOrSubdivision {
    Office(Office),
    Subdivision(OfficeSubdivision),
}
