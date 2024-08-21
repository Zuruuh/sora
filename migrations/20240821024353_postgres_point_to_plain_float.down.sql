-- Add down migration script here
alter table offices add column coordinates point;
update offices set coordinates = point(longitude, latitude);
alter table offices drop column longitude, drop column latitude, alter column coordinates set not null;
