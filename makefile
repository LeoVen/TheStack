up: clippy format build
	docker compose up -d
	sleep 5
	sqlx migrate run
	docker compose logs -f

build:
	docker compose build

clippy:
	cargo clippy --fix --allow-dirty --allow-staged -- -D warnings

format:
	cargo +nightly fmt

down:
	docker compose down

infra:
	docker compose up dbpg redis -d

migrate:
	sqlx migrate run

redis:
	redis-cli

postgres:
	psql --host=localhost --dbname=default --username=root --password

deps:
	cargo install sqlx-cli
	sudo apt install -y postgresql-client redis-tools
