-- Add migration script here
create table if not exists coupon_set (
    "id" bigserial,

    "name" varchar,
    "created_at" timestamptz not null default now(),

    primary key("id")
);

create table if not exists coupon (
    "id" uuid default gen_random_uuid(), -- UUID v4
    "set_id" bigserial,

    "used" boolean default false,

    primary key("id"),
    constraint fk_coupon_set foreign key("set_id") references coupon_set("id")
);
