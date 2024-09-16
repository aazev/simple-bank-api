set -o errexit -o pipefail -o noclobber -o nounset

is_specific_version_installed() {
    local REQUIRED_PKG=$1
    local TEST_STRING=$2

    command -v $REQUIRED_PKG >/dev/null 2>&1 || {
        echo >&2 "nvm is required, but it's not installed.  Aborting."
        exit 1
    }

    if [ "$($REQUIRED_PKG --version | awk "/$TEST_STRING/ {print }" | wc -l)" -ge 1 ]; then
        return 0
    else
        return 1
    fi
}

is_command_available() {
    local REQUIRED_PKG=$1

    if command -v $REQUIRED_PKG >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

die() {
    printf '\e[31m%s\e[0m' "Error: $1"
    exit 1
}

# run_on_db $1=docker_id $2=dbname $3=sql
run_on_db() {
    docker exec "$1" psql -h localhost -U api_agent_user -d "$2" -c "$3"
}

full_line() {
    printf %"$(tput cols)"s | tr " " "$1"
}

header() {
    if  [ -f "$LOG_PATH" ]; then
        rm $LOG_PATH
    fi

    full_line "="
    echo "${1^^}"
    full_line "="

    echo "===================================================" >$LOG_PATH
    echo "${1^^}" >>$LOG_PATH
    echo "===================================================" >>$LOG_PATH
}

step() {
    echo "- $1"

    echo "" >>$LOG_PATH
    echo "---------------------------------------------------" >>$LOG_PATH
    echo "# $1" >>$LOG_PATH
    echo "---------------------------------------------------" >>$LOG_PATH
}

get_database_docker_id() {
    docker ps -qf "name=simple_bank_api-db"
}

start_docker_container() {
    docker start $1 || true
    sleep 5
}

# $1=docker_id $2=dbname
drop_db_connections() {
    local sql="SELECT pg_terminate_backend(pg_stat_activity.pid) FROM pg_stat_activity WHERE pg_stat_activity.datname = '$2' AND pid <> pg_backend_pid();"
    run_on_db "$1" "$2" "$sql"
}

mkdir -p .container/scripts/logs 2>&1 || true
