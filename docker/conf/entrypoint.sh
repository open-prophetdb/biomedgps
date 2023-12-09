#!/bin/bash
#
# Start local dev
#
echo "Starting PostgresML"
service postgresql start

if [ -z ${POSTGRES_USER} ]; then
    echo "You must set POSTGRES_USER variable"
    exit 1
fi

if [ -z ${POSTGRES_PASSWORD} ]; then
    echo "You must set POSTGRES_PASSWORD variable"
    exit 1
fi

if [ -z ${POSTGRES_DB} ]; then
    echo "You must set POSTGRES_DB variable"
    exit 1
fi

# Setup users
useradd postgresml -m 2> /dev/null 1>&2
sudo -u postgresml touch /home/postgresml/.psql_history 2> /dev/null 1>&2
sudo -u postgres createuser root --superuser --login 2> /dev/null 1>&2
sudo -u postgres psql -c "CREATE ROLE postgresml PASSWORD '${POSTGRES_PASSWORD}' SUPERUSER LOGIN" 2> /dev/null 1>&2
sudo -u postgres psql -c "CREATE ROLE ${POSTGRES_USER} PASSWORD '${POSTGRES_PASSWORD}' SUPERUSER LOGIN" 2> /dev/null 1>&2
sudo -u postgres createdb postgresml --owner postgresml 2> /dev/null 1>&2
sudo -u postgres createdb ${POSTGRES_DB} --owner ${POSTGRES_USER} 2> /dev/null 1>&2
sudo -u postgres psql -c 'ALTER ROLE postgresml SET search_path TO public,pgml' 2> /dev/null 1>&2

echo "Starting dashboard"
PGPASSWORD=${POSTGRES_PASSWORD} psql -c 'CREATE EXTENSION IF NOT EXISTS pgml' \
        -d postgresml \
        -U postgresml \
        -h 127.0.0.1 \
        -p 5432 2> /dev/null 1>&2

bash /app/dashboard.sh &

tail -f /dev/null