#!/usr/bin/env bash

export LOG_PATH=.container/scripts/logs/db-setup.log

source .container/scripts/functions.sh

header "Local Database - Setup"

reset_database() {
    pgid=$(get_database_docker_id)
    start_docker_container "$pgid"
    drop_db_connections "$pgid" simple_bank_api || true

    run_on_db "$pgid" postgres "DROP DATABASE IF EXISTS simple_bank_api;" >>$LOG_PATH 2>&1 || true
    run_on_db "$pgid" postgres "CREATE DATABASE simple_bank_api;" >>$LOG_PATH 2>&1 || true
    run_on_db "$pgid" postgres "GRANT ALL PRIVILEGES ON database simple_bank_api to api_agent_user;" >>$LOG_PATH 2>&1
}

if ! is_command_available "docker"; then
    die 'Docker must be installed'
fi

if ! is_command_available "docker compose"; then
    die 'Docker Compose must be installed'
fi

step 'Cleaning previous logs' true

mkdir -p .container/data

step 'Starting the Local Environment'
make start >>$LOG_PATH 2>&1

step 'Reseting the Database'
reset_database >>$LOG_PATH 2>&1

step 'Your database is set and ready to receive data!'
