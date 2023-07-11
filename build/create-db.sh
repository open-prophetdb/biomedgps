#!/bin/bash
# WARNING
# The create-db.sh file is used for local postgres database.
# This file is listed in .gitignore and will be excluded from version control by Git.
set -e # exit immediately if a command exits with a non-zero status.

export SCRIPT_DIR=$(dirname "$0")
export PGPASSWORD=password

if [ -z "$1" ]; then
    database="test_biomedgps"
else
    database="$1"
fi

POSTGRES="psql -h localhost -p $2 --username postgres"

# create database for superset
$POSTGRES <<EOSQL
CREATE DATABASE $database OWNER postgres;
EOSQL

export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps
