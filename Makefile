all: env start

env:
	cp .default.env .env

start:
	mkdir -p .container/data
	mkdir -p .container/cache
	docker compose -f .container/docker-compose.yml up -d

stop:
	docker compose -f .container/docker-compose.yml down

db-setup:
	@bash .container/scripts/db-setup.sh

db: db-setup

build:
	cargo build

deploy:
	cargo build -r

benchmark:
	@bash .container/scripts/benchmark.sh
