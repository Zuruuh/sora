use std::ops::Deref;

use chrono::{Days, Months, NaiveDate, Utc};
use sora_model::{
    contract::{Contract, ContractError, ContractId, CONTRACT_DURATION_MINIMUM_DAYS},
    office::{Office, OfficeId, OfficeSplitId, RealOfficeId},
    user::{User, UserId},
};
use sqlx::PgPool;

use crate::range::{invert_ranges_in_boundary, DateRange};

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

    let contracts =
        simulation.simulate(users.iter().collect(), offices.iter().collect(), contracts)?;

    println!("Simulation done, printing best solution found:");
    println!("Displaying informations for offices");
    println!("");

    for office in offices {
        let office_contracts = contracts
            .iter()
            .filter(|contract| contract.office() == office.id())
            .collect::<Vec<_>>();

        println!("Office {}:", office.id());
        for contract in office_contracts {
            println!(
                "> From {} to {} ({} days), office will be occupied by {} with contract {}",
                contract.start(),
                contract.end(),
                (*contract.end() - *contract.start()).num_days(),
                contract.guest(),
                contract.id()
            );
        }
        println!("");
    }

    println!("============================");
    println!("Displaying user informations:");
    println!("");

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

            println!(
                "> The rent for said office is of {}€/month, user will be paying a total of {}€ for the contract duration", 
                *contract.rent() as f32 / 100.0,
                (*contract.rent() as f32 * ((*contract.end() - *contract.start()).num_weeks() as f32 / 7.0)) / 100.0
            );
        }

        println!("");
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
        users: Vec<&User>,
        offices: Vec<&Office>,
        contracts: Vec<Contract>,
    ) -> Result<Vec<Contract>, ContractError> {
        let mut contracts = contracts;

        'users: for user in users {
            let user_unavailable_ranges = contracts
                .iter()
                .filter(|contract| contract.guest() == user.id())
                .map(DateRange::from)
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
                'user_availability: for user_availability in user_ranges_to_fill.iter() {
                    let contracts_for_office = contracts
                        .iter()
                        .filter(|contract| contract.office() == office_candidate.id())
                        .collect::<Vec<_>>();

                    let office_unavailabilities = contracts_for_office
                        .iter()
                        .map(Deref::deref)
                        .map(DateRange::from)
                        .collect::<Vec<_>>();

                    let office_availabilities = invert_ranges_in_boundary(
                        office_unavailabilities.iter(),
                        self.start,
                        self.end,
                    );

                    // The 3 vars above should be recalculated every time a new contract is added
                    // but for simplicity we'll just recalculate them on each loop

                    for office_availability in office_availabilities.iter() {
                        let overlap_start = user_availability.start.max(office_availability.start);
                        let overlap_end = user_availability.end.min(office_availability.end);
                        // dbg!(&overlap_start, overlap_end);

                        if overlap_start < overlap_end {
                            let contract_end = overlap_start
                                .checked_add_days(Days::new(user_missing_office_days as u64))
                                .unwrap()
                                .min(overlap_end);

                            log::info!(
                                "> Trying to lock office {} from {} to {} for user {}",
                                office_candidate.id(),
                                overlap_start,
                                contract_end,
                                user.id()
                            );

                            let contract = match Contract::for_office(
                                office_candidate,
                                *user.id(),
                                overlap_start,
                                contract_end,
                            ) {
                                Ok(contract) => contract,
                                Err(err) => {
                                    log::error!("Tried to create a contract but failed ({err}). Skipping to next scenario");

                                    continue;
                                }
                            };

                            let days_locked = (contract_end - overlap_start).num_days() as usize;
                            let user_missing_office_days =
                                user_missing_office_days.saturating_sub(days_locked);

                            contracts.push(contract);

                            if user_missing_office_days == 0 {
                                log::info!(
                                    "User {} has locked all necessary days, switching to next user",
                                    user.id()
                                );

                                continue 'users;
                            }

                            continue 'user_availability;
                        }
                    }

                    if user_missing_office_days > 0 {
                        log::info!(
                            "User {} still needs to lock {} more days",
                            user.id(),
                            user_missing_office_days
                        );
                    }
                }
            }
        }

        Ok(contracts)
    }
}
