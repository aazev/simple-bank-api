[![Build](https://github.com/aazev/simple-bank-api/actions/workflows/build.yml/badge.svg)](https://github.com/aazev/simple-bank-api/actions/workflows/build.yml)

# Simple Bank API

## Description

Simple bank api with encrypted account and transaction data. Please leave a star if you like the project. Feel free to review my code.

## How to run
Install rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Run tests:
```bash
cargo test
```

Copy the `.env.example` file to `.env` and set the environment variables.
```bash
cp .env.example .env
```

Startup the containers (you may skip if you have a local postgres instance):
```bash
make start
```

Run the server (use -r to run in release mode):
```bash
cargo run [-r]
```

## Endpoints

Please refer to the Swagger documentation at:

* `/docs` for swagger;
* `/redoc` for redoc;
* `/rapidoc` for rapidoc;

!!!!! Jobs and Scopes aren't fully implemented yet. !!!!!
