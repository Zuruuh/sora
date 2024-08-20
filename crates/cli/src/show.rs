use std::{fmt::Display, str::FromStr};

use sqlx::PgPool;
use uuid::Uuid;

pub async fn show(pool: &PgPool, filter: Option<String>) -> color_eyre::Result<()> {
    let filter = filter
        .as_deref()
        .map(|str| str.trim().trim_matches(['\'', '"']))
        .map(|str| Filter::from_str(str).map_err(|_| FilterError::NoSuchFilter(str.to_string())))
        .unwrap_or(Ok(Filter::None))?;

    log::info!("Using filter {:?}", filter);

    match filter {
        Filter::Aggregate(aggregate) => {
            let condition = aggregate
                .condition()
                .map(|condition| format!("where {condition}"))
                .unwrap_or_default();
            let results = sqlx::query(&format!("select * from {aggregate} {condition}"))
                .fetch_all(pool)
                .await?;

            dbg!(results);
        }
        Filter::Id(aggregate, id) => {
            let result = sqlx::query(&format!("select * from {aggregate} where id = $1::uuid"))
                .bind(id)
                .fetch_one(pool)
                .await?;
            dbg!(result);
        }
        Filter::None => {
            let tables = [Aggregate::Users, Aggregate::Contracts, Aggregate::Offices];
            let queries = tables
                .iter()
                .map(|aggregate| format!("select * from {aggregate}"))
                .collect::<Vec<_>>();

            // necessary since the sql string query (`query` var) needs to stay in scope for
            // the future's lifetime
            let queries = queries
                .iter()
                .map(|query| sqlx::query(query).fetch_all(pool));

            let results = futures::future::join_all(queries)
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;

            for (i, table_results) in results.into_iter().enumerate() {
                log::info!("Displaying table data for {}", tables[i]);

                for result in table_results {
                    dbg!(result);
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Default)]
enum Filter {
    Aggregate(Aggregate),
    Id(Aggregate, Uuid),
    #[default]
    None,
}

#[derive(Debug, thiserror::Error)]
enum FilterError {
    #[error(r#"No filter matched for "{0}""#)]
    NoSuchFilter(String),
}

impl FromStr for Filter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match Aggregate::from_str(s) {
            Ok(aggregate) => Self::Aggregate(aggregate),
            Err(_) => {
                let prefix = s.chars().take(3).collect::<String>();
                let uuid = Uuid::from_str(s.chars().skip(4).collect::<String>().as_str())
                    .map_err(|_| ())?;

                let aggregate = Aggregate::from_str(&prefix)?;

                Self::Id(aggregate, uuid)
            }
        })
    }
}

#[derive(Debug)]
enum Aggregate {
    Contracts,
    Offices,
    OfficeSplit,
    Users,
}

impl Aggregate {
    pub fn condition(&self) -> Option<&'static str> {
        match self {
            Self::OfficeSplit => Some("parent_office_id is not null"),
            _ => None,
        }
    }
}

impl FromStr for Aggregate {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "usr" | "user" | "users" => Self::Users,
            "ofc" | "office" | "offices" => Self::Offices,
            "spl" => Self::OfficeSplit,
            "agr" | "contract" | "contracts" => Self::Contracts,
            _ => return Err(()),
        })
    }
}

impl Display for Aggregate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Aggregate::Contracts => "contracts",
                Aggregate::Offices | Aggregate::OfficeSplit => "offices",
                Aggregate::Users => "users",
            }
        )
    }
}
