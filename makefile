up: clippy format build gen_rsa prepare_env
	docker compose up -d
	. ./scripts/sqlx_migrate.sh
	. ./scripts/setup_keycloak.sh

down:
	docker compose down

build:
	docker compose build

clippy:
	cargo clippy --fix --allow-dirty --allow-staged -- -D warnings

format:
	cargo +nightly fmt

infra:
	docker compose up dbpg redis -d

migrate:
	sqlx migrate run

redis:
	redis-cli

postgresql:
	psql --host=localhost --dbname=root --username=root --password

watch:
	cargo watch -w the_stack/src -x "run --bin the_stack"

console:
	tokio-console http://127.0.0.1:5555

tester:
	. ./.env
	cargo run --bin the_stack_tester

prepare_env:
	. ./scripts/prepare_env.sh

gen_rsa:
	. ./scripts/create_rsa_kp.sh

deps:
	cargo install sqlx-cli cargo-watch cargo-expand tokio-console
	sudo apt install -y postgresql-client redis-tools curl jq sed
	chmod -R +x ./scripts
