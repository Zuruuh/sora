-- Add up migration script here

create table users (
    id uuid primary key unique not null,
    created_at timestamp not null default now(),
    first_name varchar not null,
    last_name varchar not null
);

create table offices (
    id uuid primary key unique not null,
    created_at timestamp not null default now(),
    name varchar not null,
    address varchar not null,
    coordinates point not null,
    owner_id uuid references users(id) not null,
    available_positions integer not null,
    surface integer not null,
    position_price integer not null
);

create table subdivided_offices (
    id uuid primary key unique not null,
    created_at timestamp not null default now(),
    name varchar not null,
    address varchar not null,
    coordinates point not null,
    owner_id uuid references users(id) not null,
    available_positions integer not null,
    surface integer not null,
    position_price integer not null,
    parent_office_id uuid references offices(id) not null
);

create table contracts (
    id uuid primary key unique not null,
    created_at timestamp not null default now(),
    host_id uuid references users(id) not null,
    guest_id uuid references users(id) not null,
    office_id uuid references offices(id) not null,
    rent integer not null,
    duration_start date not null,
    duration_end date not null
);
