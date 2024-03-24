CREATE TABLE spell (
  id bigserial primary key,
  name VARCHAR,
  damage INT not null,
  created_at TIMESTAMPTZ NOT NULL default now(),
  updated_at TIMESTAMPTZ NOT NULL default now()
);

INSERT INTO spell (id, name, damage) VALUES
(1, 'Fireball', 30),
(2, 'Ice Shard', 25),
(3, 'Thunderbolt', 35),
(4, 'Earthquake', 40),
(5, 'Healing Light', -25),
(6, 'Arcane Missile', 20),
(7, 'Poison Cloud', 15),
(8, 'Chain Lightning', 45);
