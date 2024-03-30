-- Add migration script here
do
$proc$
declare
    c bigint;
begin
    select count(*) from coupon_set into c;
    with
        set_series as (select * from generate_series(c+1, c+25) as n),
        sets as (insert into coupon_set ("name") select 'Coupon Set ' || ss.n from set_series ss returning id),
        coupon_series as (select * from generate_series(1, 1000))
    insert into coupon ("set_id") select cs.id from sets as cs left join coupon_series on 1=1;
end
$proc$;
