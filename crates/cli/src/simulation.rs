use std::{any::Any, collections::VecDeque, u32};

use chrono::{Days, Months, NaiveDate, Utc};
use sora_model::{
    contract::{self, Contract, ContractId, CONTRACT_DURATION_MINIMUM_DAYS},
    office::{Office, OfficeId, OfficeSplitId, RealOfficeId},
    user::{User, UserId},
};
use sqlx::PgPool;

use crate::range::DateRange;

pub async fn simulate(duration_in_months: usize, pool: &PgPool) -> color_eyre::Result<()> {
    let simulation = Simulation {
        start: Utc::now().date_naive(),
        end: Utc::now()
            .checked_add_months(Months::new(duration_in_months as u32))
            .unwrap()
            .date_naive(),
        target_days_in_office: 12 * 30, // ~1 year
    };

    let users = sqlx::query!("select * from users")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|user| {
            User::new_unchecked(
                UserId::from(user.id),
                user.created_at,
                user.first_name,
                user.last_name,
            )
        })
        .collect::<Vec<_>>();

    let offices = sqlx::query!("select * from offices where not exists(select sub.id from offices as sub where sub.parent_office_id = offices.id)")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|office| {
            // office is split if it has a parent
            let id = office
                .parent_office_id
                .map(|_| OfficeId::OfficeSplit(OfficeSplitId::from(office.id)))
                .unwrap_or_else(|| OfficeId::RealOffice(RealOfficeId::from(office.id)));

            Office::new_unchecked(
                id,
                office.created_at,
                office.name,
                office.address,
                office.latitude as f32,
                office.longitude as f32,
                UserId::from(office.owner_id),
                office.available_positions as usize,
                office.surface as usize,
                office.position_price as usize,
                office.parent_office_id.map(RealOfficeId::from),
            )
        })
        .collect::<Vec<_>>();

    let contracts = sqlx::query!(
        r#"
        select *,
        exists(
            select id from offices where id = contracts.office_id
        ) as is_split_office 
        from contracts
        where start >= $1::date AND "end" <= $2::date
        order by start asc"#,
        simulation.start,
        simulation.end
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|contract| {
        Contract::new_unchecked(
            ContractId::from(contract.id),
            contract.created_at,
            contract.host_id.into(),
            contract.guest_id.into(),
            contract
                .is_split_office
                .unwrap_or_default()
                .then(|| OfficeId::RealOffice(contract.office_id.into()))
                .unwrap_or_else(|| OfficeId::OfficeSplit(contract.office_id.into())),
            contract.rent as usize,
            contract.start,
            contract.end,
        )
    })
    .collect::<Vec<_>>();

    let contracts = simulation.simulate(
        &mut users.iter().collect(),
        offices.iter().collect(),
        contracts,
    )?;

    println!("Displaying user informations:");

    for user in users {
        let user_contracts = contracts
            .iter()
            .filter(|contract| contract.guest() == user.id())
            .collect::<Vec<_>>();

        let total_days_in_office = user_contracts
            .iter()
            .map(|contract| *contract.end() - *contract.start())
            .map(|time| time.num_days())
            .sum::<i64>();

        println!("Displaying informations for {}", user.id());

        println!(
            "> They will rent offices for {total_days_in_office} days with a total of {} contracts",
            user_contracts.len()
        );

        for contract in user_contracts {
            println!(
                "> Contract {} for office {} will span from {} to {}, for a total of {} days",
                contract.id(),
                contract.office(),
                contract.start(),
                contract.end(),
                (*contract.end() - *contract.start()).num_days()
            );
        }
    }

    println!("============================");
    println!("Displaying informations for offices:");

    for office in offices {
        let office_contracts = contracts
            .iter()
            .filter(|contract| contract.office() == office.id())
            .collect::<Vec<_>>();

        println!("Office {}:", office.id());
        for contract in office_contracts {
            println!(
                "> From {} to {} ({} days), office will be occupied by {}",
                contract.start(),
                contract.end(),
                (*contract.end() - *contract.start()).num_days(),
                contract.guest()
            );
        }
    }

    Ok(())
}

pub struct Simulation {
    start: NaiveDate,
    end: NaiveDate,
    target_days_in_office: usize,
}

impl Simulation {
    fn simulate(
        &self,
        users: &mut VecDeque<&User>,
        offices: Vec<&Office>,
        contracts: Vec<Contract>,
    ) -> Result<Vec<Contract>, SimulationError> {
        let user = match users.front() {
            Some(user) => user,
            None => return Ok(contracts),
        };

        let user_unavailable_ranges = contracts
            .iter()
            .filter(|contract| contract.guest() == user.id())
            .map(|contract| DateRange::new(*contract.start(), *contract.end()))
            .collect::<Vec<_>>();

        let user_ranges_to_fill =
            invert_ranges_in_boundary(user_unavailable_ranges.iter(), self.start, self.end);

        let office_candidates = offices
            .iter()
            .filter(|office| office.owner() != user.id())
            .collect::<Vec<_>>();

        let user_total_office_days = user_unavailable_ranges
            .iter()
            .map(|range| (range.end - range.start).num_days())
            .sum::<i64>() as usize;

        let user_missing_office_days = (self.target_days_in_office - user_total_office_days)
            .max(CONTRACT_DURATION_MINIMUM_DAYS);

        log::info!("User {} has {user_total_office_days}/{} days of locked office, which means we still need to lock at least {user_missing_office_days} days!", user.id(), self.target_days_in_office);

        for office_candidate in office_candidates {
            let contracts_for_office = contracts
                .iter()
                .filter(|contract| contract.office() == office_candidate.id())
                .collect::<Vec<_>>();

            let office_unavailabilities = contracts_for_office
                .iter()
                .map(|contract| DateRange::new(*contract.start(), *contract.end()))
                .collect::<Vec<_>>();

            let office_availabilities =
                invert_ranges_in_boundary(office_unavailabilities.iter(), self.start, self.end);

            for user_availability in user_ranges_to_fill.iter() {
                // If user availability is completely contained in this office unavailability
                // go to next user availability
                if office_unavailabilities.iter().any(|office_unavailability| {
                    user_availability.is_contained_in(office_unavailability)
                }) {
                    continue;
                }

                let end = user_availability
                    .start
                    .checked_add_days(Days::new(user_missing_office_days as u64))
                    .unwrap()
                    .min(user_availability.end);

                log::info!(
                    "> Trying to lock office {} from {} to {} for user {}",
                    office_candidate.id(),
                    user_availability.start,
                    end,
                    user.id()
                );

                let contract = Contract::for_office(
                    office_candidate,
                    *user.id(),
                    user_availability.start,
                    end,
                )
                .unwrap();

                let missing_days =
                    user_missing_office_days as i64 - (end - user_availability.start).num_days();

                if missing_days <= 0 {
                    log::info!(
                        "User {} has locked all necessary days, switching to next user",
                        user.id()
                    );
                    users.pop_front();
                }

                let mut contracts = contracts;
                contracts.push(contract);

                return self.simulate(users, offices, contracts);
            }

            // for office_unavailable_range in office_unavailable_ranges {
            //     if let Some(range) = user_ranges_to_fill.iter().find(|user_unavailability| {
            //         user_unavailability.overlap(&office_unavailable_range)
            //     }) {
            //         dbg!(range);
            //     }
            // }
        }

        todo!()
    }
}

fn invert_ranges_in_boundary<'date_range, Iter>(
    ranges: Iter,
    start: NaiveDate,
    end: NaiveDate,
) -> VecDeque<DateRange>
where
    Iter: Clone + IntoIterator<Item = &'date_range DateRange>,
{
    let should_map_window = ranges.clone().into_iter().count() > 1;
    let mut ranges: VecDeque<_> = should_map_window
        .then(|| {
            ranges
                .clone()
                .into_iter()
                .map_windows(|&[first, second]| DateRange::new(second.start, first.end))
                .collect()
        })
        .unwrap_or_else(|| {
            ranges
                .into_iter()
                .flat_map(|range| {
                    [
                        DateRange::new(start, range.start),
                        DateRange::new(range.end, end),
                    ]
                })
                .collect()
        });

    if ranges.is_empty() {
        ranges.push_back(DateRange::new(start, end));

        return ranges;
    }

    if let Some(first) = ranges.front() {
        if (first.start - start).num_days() > CONTRACT_DURATION_MINIMUM_DAYS as i64 {
            ranges.push_front(DateRange::new(start, first.start));
        }
    }

    if let Some(last) = ranges.back() {
        if (end - last.end).num_days() > CONTRACT_DURATION_MINIMUM_DAYS as i64 {
            ranges.push_back(DateRange::new(last.end, end));
        }
    }

    ranges
}

#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
    // #[error("Could not find an office for user {user}")]
    // NoOfficeAvailableForUser { user: UserId },
}
