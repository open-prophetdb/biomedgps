# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

BioMedGPS is a knowledge graph platform for drug repurposing and disease mechanism discovery. The codebase consists of:

- **Backend (Rust)**: REST API server using Poem framework with OpenAPI support
- **Frontend (React/TypeScript)**: Studio interface built with UmiJS, Ant Design, and visualization libraries
- **Database**: PostgreSQL for relational data, Neo4j for graph operations
- **Knowledge Graph**: Entity-relation model with embeddings for ML/AI features

### Key Components

- `src/`: Rust backend code
  - `api/`: REST API routes and authentication
  - `model/`: Database models (entities, relations, embeddings, etc.)
  - `algorithm/`: ML algorithms and graph neural networks
  - `query_builder/`: SQL and Cypher query construction
  - `schedule/`: Task management and workflow execution
- `studio/`: Frontend React application
- `migrations/`: Database schema migrations
- `examples/`: Sample data files
- `wasm/`: WebAssembly modules
- `cromwell-api-rs/`: Workflow management integration

## Development Commands

### Backend (Rust)
```bash
# Run tests with test database
make test

# Build for production (macOS)
make build-mac

# Build for production (Linux)
make build-linux

# Run development server
export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps
cargo run --bin biomedgps -- -H 0.0.0.0 -p 8888 --openapi --debug
```

### Frontend (React)
```bash
cd studio

# Install dependencies
yarn

# Local development
yarn start:local-dev

# Production build
yarn build:biomedgps-embed
```

### Database Setup
```bash
# Create test database with Docker
make test-db

# Initialize database schema
export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps
cargo run --bin biomedgps-cli -- initdb

# Import data
cargo run --bin biomedgps-cli -- importdb -f /data/entity.tsv -t entity -D
```

## Project Structure

### Data Flow
1. **Data Import**: CSV/TSV files → PostgreSQL/Neo4j via `biomedgps-cli`
2. **API Layer**: Rust backend serves REST endpoints with OpenAPI spec
3. **Frontend**: React app consumes API, provides graph visualization
4. **ML Pipeline**: Entity embeddings and graph neural networks for predictions

### Key Technologies
- **Backend**: Rust, Poem (web framework), SQLx (database), Neo4rs (graph DB)
- **Frontend**: React 18, UmiJS, Ant Design, Graphin (graph viz), Plotly.js
- **Databases**: PostgreSQL (with ML extensions), Neo4j
- **Auth**: Auth0 integration
- **Build**: Cargo (Rust), Yarn (Node.js), Make (orchestration)

### Environment Variables
Required for development:
- `DATABASE_URL`: PostgreSQL connection string
- `NEO4J_URL`: Neo4j connection string (optional for some features)

### Testing
- Backend tests: `cargo test` (requires test database)
- No specific frontend test commands configured
- Integration tests use Docker-based PostgreSQL instance

### Deployment
- Cross-compilation supported for Linux targets
- Docker images available via GitHub Container Registry
- Nginx configuration for production deployment
- Binary artifacts: `biomedgps` (server), `biomedgps-cli` (data management)