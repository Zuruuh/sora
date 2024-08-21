-- Add up migration script here
alter table offices add column longitude float, add column latitude float;

update offices set longitude = coordinates[0], latitude = coordinates[1];

alter table offices
    alter column longitude set not null,
    alter column latitude set not null,
    drop column coordinates;
