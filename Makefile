.PHONY: test dev-db test-db clean-test-db

test: clean-test-db test-db
	@printf "\nRunning unittest...\n"
	DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps cargo test

test-db: clean-test-db
	@printf "\nLaunch postgres database...(default password: password)\n"
	@docker run -v /Users/codespace/Downloads:/Users/codespace/Downloads --name biomedgps -e POSTGRES_PASSWORD=password -e POSTGRES_USER=postgres -p 5432:5432 -d postgres:10.0
	@sleep 3
	@echo "Create database: test_biomedgps"
	@bash build/create-db.sh test_biomedgps 5432

clean-test-db:
	@printf "Stop "
	@-docker stop biomedgps
	@printf "Clean "
	@-docker rm biomedgps

build-studio:
	@cd studio && yarn && yarn openapi && yarn build:embed && cd ..

build-biomedgps:
	@cargo build --release

build-biomedgps-linux:
	@cargo build --release --target=x86_64-unknown-linux-musl

build-mac: build-studio build-biomedgps
	@printf "\nBuilding...\n"

build-linux: build-studio build-biomedgps-linux
	@printf "\nBuilding...\n"

# You must run `make build-service` to build new api spec for studio when you change the api spec
build-service:
	@printf "Building service based on openapi...\n"
	@curl -H "Accept: application/json" http://localhost:3000/spec
	@cd studio && yarn && yarn openapi && cd ..