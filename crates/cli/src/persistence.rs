use sora_model::{contract::Contract, id::Identifier, office::Office, Object};
use sqlx::{postgres::PgQueryResult, PgPool, Postgres};

pub async fn persist_contract<'a, Executor>(
    contract: &Contract,
    office: &Office,
    pool: Executor,
) -> Result<PgQueryResult, sqlx::Error>
where
    Executor: sqlx::Executor<'a, Database = Postgres>,
{
    sqlx::query!(
        r#"
    insert into contracts (
        id,
        created_at,
        host_id,
        guest_id,
        office_id,
        rent,
        start,
        "end"
    ) values (
        $1::uuid,
        $2::timestamptz,
        $3::uuid,
        $4::uuid,
        $5::uuid,
        $6::integer,
        $7::date,
        $8::date
    ) on conflict do nothing"#,
        *contract.uuid(),
        *contract.created_at(),
        *contract.host().uuid(),
        *contract.guest().uuid(),
        *contract.office().uuid(),
        (office.available_positions() * office.position_price()) as i64,
        contract.start(),
        contract.end(),
    )
    .execute(pool)
    .await
}

pub async fn persist_office(office: &Office, pool: &PgPool) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
            insert into offices (
                id,
                created_at,
                name,
                address,
                longitude,
                latitude,
                owner_id,
                available_positions,
                surface,
                position_price,
                parent_office_id
            ) values (
                $1::uuid,
                $2::timestamptz,
                $3::varchar,
                $4::varchar,
                $5::float,
                $6::float,
                $7::uuid,
                $8::integer,
                $9::integer,
                $10::integer,
                $11::uuid
            );
        "#,
        office.uuid(),
        office.created_at(),
        office.name(),
        office.address(),
        *office.longitude() as f64,
        *office.latitude() as f64,
        office.owner().uuid(),
        *office.available_positions() as i32,
        *office.surface() as i32,
        *office.position_price() as i32,
        office.parent_office().map(|id| *id.uuid()),
    )
    .execute(pool)
    .await
}
