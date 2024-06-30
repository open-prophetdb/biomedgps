.PHONY: test dev-db test-db clean-test-db

test: clean-test-db test-db
	@printf "\nRunning unittest...\n"
	DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps cargo test

test-db: clean-test-db create-docker create-db

create-docker:
	@printf "\nLaunch postgres database...(default password: password)\n"
	# Make it compatible with mac and linux, the temp folder is different, so we need to mount both
	@docker run -v /tmp:/tmp -v /var/folders:/var/folders --name biomedgps -e POSTGRES_PASSWORD=password -e POSTGRES_USER=postgres -p 5432:5432 -d nordata/postgresml:v2.8.3-1c11927
	@sleep 3

create-db:
	@echo "Create database: test_biomedgps"
	@bash build/create-db.sh test_biomedgps 5432
	@echo "Migrate database: test_biomedgps"
	@export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && cargo run --bin biomedgps-cli -v initdb
	@cp -R ./examples/ /tmp/examples/
	@export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && cargo run --bin biomedgps-cli -v importdb -D -f /tmp/examples/entity.tsv -t entity
	@export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && cargo run --bin biomedgps-cli -v importdb -D -f /tmp/examples/relation.tsv -t relation
	@export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && cargo run --bin biomedgps-cli -v importdb -D -f /tmp/examples/entity_embedding.tsv -t entity_embedding
	@export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && cargo run --bin biomedgps-cli -v importdb -D -f /tmp/examples/knowledge_curation.tsv -t knowledge_curation

clean-test-db:
	@printf "Stop "
	@-docker stop biomedgps
	@printf "Clean "
	@-docker rm biomedgps

clean-studio:
	@printf "Clean studio...\n"
	@cd studio && rm -rf node_modules && rm -rf dist && yarn cache clean && cd ..

build-biomedgps-studio:
	@printf "Building studio based on openapi...\n"
	@mkdir -p assets
	@rm -rf frontend && cp -r studio frontend
	@cp studio/custom/logo/biomedgps.png frontend/public/assets/logo-white.png 
	@cp studio/custom/logo/biomedgps.png frontend/src/assets/logo-white.png
	@cp studio/custom/logo/biomedgps.png frontend/public/logo.png
	@cp studio/custom/route/rapex.ts frontend/config/routes.ts
	# @cd studio && yarn && yarn openapi || true
	@cd frontend && yarn
	@cd frontend && yarn build:biomedgps-embed && cd ..

build-rapex-studio:
	@printf "Building studio based on openapi...\n"
	@mkdir -p assets
	@rm -rf frontend && cp -r studio frontend && rm -rf frontend/node_modules
	@cp studio/custom/home/rapex.tsx frontend/src/pages/Home/index.tsx
	@cp studio/custom/logo/rapex.png frontend/public/assets/logo-white.png 
	@cp studio/custom/logo/rapex.png frontend/src/assets/logo-white.png
	@cp studio/custom/logo/rapex.png frontend/public/logo.png
	@cp studio/custom/route/rapex.ts frontend/config/routes.ts
	# @cd studio && yarn && yarn openapi || true
	@cd frontend && yarn
	@cd frontend && yarn build:rapex-embed && cd ..

build-biomedgps:
	@cargo build --release

build-biomedgps-linux:
	@cargo build --release --target=x86_64-unknown-linux-musl

build-mac: build-biomedgps-studio build-biomedgps
	@printf "\nDone!\n"

build-linux: build-biomedgps-studio build-biomedgps
	@printf "\nDone!\n"

build-linux-on-mac: build-biomedgps-studio build-biomedgps-linux
	@printf "\nDone!\n"

# You must run `make build-service` to build new api spec for studio when you change the api spec
build-service:
	@printf "Building service based on openapi...\n"
	@curl -H "Accept: application/json" http://localhost:3000/spec
	@cd studio && yarn && yarn openapi && cd ..

changelog:
	@printf "Generate changelog...\n"
	@python build/build_changelog.py --repo ../biominer-components --output-file ./studio/public/README/changelog.md --repo-name 'BioMedGPS UI'
	@python build/build_changelog.py --repo . --output-file ./studio/public/README/changelog.md --repo-name BioMedGPS

deploy: deploy-biomedgps

deploy-biomedgps: build-biomedgps-studio
	@docker run --rm -it -v "$(CURDIR)":/home/rust/src messense/rust-musl-cross:x86_64-musl cargo build --release
	@rsync -avP target/x86_64-unknown-linux-musl/release/biomedgps target/x86_64-unknown-linux-musl/release/biomedgps-cli root@drugs.3steps.cn:/data/biomedgps/bin
	@rsync -avP --delete assets/index.html root@drugs.3steps.cn:/var/www/html/biomedgps/index.html
	@rsync -avP --delete assets root@drugs.3steps.cn:/var/www/html/biomedgps/

deploy-rapex: build-rapex-studio
	@docker run --rm -it -v "$(CURDIR)":/home/rust/src messense/rust-musl-cross:x86_64-musl cargo build --release
	@rsync -avP target/x86_64-unknown-linux-musl/release/biomedgps target/x86_64-unknown-linux-musl/release/biomedgps-cli root@rapex.prophetdb.org:/data/rapex/bin
	@rsync -avP --delete assets/index.html root@rapex.prophetdb.org:/var/www/html/rapex/index.html
	@rsync -avP --delete assets root@rapex.prophetdb.org:/var/www/html/rapex/