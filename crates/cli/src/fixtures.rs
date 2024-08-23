use crate::persistence::{persist_contract, persist_office};
use chrono::{Days, Utc};
use fake::{
    faker::{address::fr_fr::*, name::fr_fr::*},
    Fake,
};
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use sora_model::{
    contract::{Contract, CONTRACT_DURATION_MINIMUM_DAYS},
    id::Identifier,
    office::{Office, OfficeSplit},
    user::User,
    Object,
};
use sqlx::PgPool;

const MINIMUM_USERS_TO_GENERATE: usize = 2;
const MAXIMUM_USERS_TO_GENERATE: usize = 6;

const MINIMUM_OFFICES_TO_GENERATE: usize = 3;
const MAXIMUM_OFFICES_TO_GENERATE: usize = 5;

pub async fn create_fixtures(
    pool: &PgPool,
    rng: &mut ThreadRng,
    subdivide_offices: bool,
) -> color_eyre::Result<()> {
    log::info!("Creating database fixtures");
    log::info!("First deleting existing data");

    sqlx::query!("truncate table contracts, offices, users")
        .execute(pool)
        .await?;

    log::info!("All existing data deleted");

    let users_to_generate = rng.gen_range(MINIMUM_USERS_TO_GENERATE..=MAXIMUM_USERS_TO_GENERATE);
    let offices_to_generate = rng.gen_range(
        MINIMUM_OFFICES_TO_GENERATE
            ..=(MAXIMUM_OFFICES_TO_GENERATE - subdivide_offices.then_some(2).unwrap_or_default()),
    );

    let mut users = Vec::<User>::new();
    let mut offices = Vec::<Office>::new();

    for _ in 0..users_to_generate {
        let user = User::new(
            FirstName().fake_with_rng(rng),
            LastName().fake_with_rng(rng),
        );

        users.push(user);
    }

    for _ in 0..offices_to_generate {
        let office = Office::new_real(
            SecondaryAddress().fake_with_rng(rng),
            format!(
                "{} {}, {} {}",
                BuildingNumber().fake_with_rng::<String, ThreadRng>(rng),
                StreetName().fake_with_rng::<String, ThreadRng>(rng),
                CityName().fake_with_rng::<String, ThreadRng>(rng),
                ZipCode().fake_with_rng::<String, ThreadRng>(rng)
            ),
            rng.gen_range(-90.0..90.0),
            rng.gen_range(-180.0..180.0),
            *users.choose(rng).unwrap().id(),
            rng.gen_range(80..180),
            rng.gen_range(500..1000),
            rng.gen_range(30000..80000),
        )
        .unwrap();

        offices.push(office);
    }

    for user in users.iter() {
        sqlx::query!(
            r#"
            insert into users (
                id, created_at, first_name, last_name
            ) values (
                $1::uuid,
                $2::timestamptz,
                $3::varchar,
                $4::varchar
            );
        "#,
            user.id().uuid(),
            user.created_at(),
            user.first_name(),
            user.last_name(),
        )
        .execute(pool)
        .await?;
    }

    log::info!("Created {} users", users.len());

    for office in offices.iter() {
        persist_office(office, pool).await?;
    }

    log::info!("Created {} offices", offices.len());

    if subdivide_offices {
        let office_to_subdivide = offices.choose(rng).unwrap();
        let (available_positions_1, available_positions_2) = (
            (*office_to_subdivide.available_positions() as f32 / 2.0).floor() as usize,
            (*office_to_subdivide.available_positions() as f32 / 2.0).ceil() as usize,
        );

        let (surface_1, surface_2) = (
            (*office_to_subdivide.surface() as f32 / 2.0).floor() as usize,
            (*office_to_subdivide.surface() as f32 / 2.0).ceil() as usize,
        );

        let office_subdivisions = office_to_subdivide.split(vec![
            OfficeSplit::new(available_positions_1, surface_1)?,
            OfficeSplit::new(available_positions_2, surface_2)?,
        ])?;

        let office_subdivisions_count = office_subdivisions.len();
        for office_subdivision in office_subdivisions {
            persist_office(&office_subdivision, pool).await?;
        }

        log::info!("Created {office_subdivisions_count} office subdivisions");
    }

    log::info!("Creating a fake contract");

    let (guest, office) = {
        loop {
            let guest = users.choose(rng).unwrap();

            let office = offices
                .iter()
                .rev()
                .find(|office| office.owner() != guest.id());

            if let Some(office) = office {
                break (guest, office);
            }
        }
    };

    let start = rng
        .gen_range(CONTRACT_DURATION_MINIMUM_DAYS..(365 * 2 - CONTRACT_DURATION_MINIMUM_DAYS))
        as u64;
    let end = start + CONTRACT_DURATION_MINIMUM_DAYS as u64;

    let contract = Contract::new(
        *office.owner(),
        *guest.id(),
        *office.id(),
        office.position_price() * office.available_positions(),
        Utc::now()
            .checked_add_days(Days::new(start))
            .unwrap()
            .date_naive(),
        Utc::now()
            .checked_add_days(Days::new(end))
            .unwrap()
            .date_naive(),
    )?;

    persist_contract(&contract, &office, pool).await?;

    Ok(())
}
