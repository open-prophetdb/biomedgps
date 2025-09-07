---
layout: page
title: Getting Started
permalink: /getting-started/
---

# Getting Started with BioMedGPS

This guide will help you get up and running with BioMedGPS quickly. Whether you're a researcher looking to explore biomedical knowledge or a developer wanting to integrate with our platform, this page covers the essentials.

## Prerequisites

### For Users (Web Interface)
- A modern web browser (Chrome, Firefox, Safari, or Edge)
- Internet connection
- (Optional) Auth0 account for advanced features

### For Developers (Local Installation)
- **Backend Requirements:**
  - Rust (latest stable version)
  - PostgreSQL (12+ recommended)
  - Neo4j (4.0+ recommended)
  - Docker (optional, for easy database setup)

- **Frontend Requirements:**
  - Node.js (version 16.13.1)
  - Yarn package manager

## Quick Start (Web Interface)

If you just want to use BioMedGPS without installing anything locally:

1. **Visit the Platform**: Navigate to your BioMedGPS instance URL
2. **Create an Account**: Sign up using Auth0 authentication
3. **Explore the Dashboard**: Start with the main dashboard to get familiar with available features
4. **Try the Knowledge Graph**: Search for a disease or drug to see interactive visualizations

## Local Installation

### Step 1: Set up Databases

The easiest way is to use Docker:

```bash
# Clone the repository
git clone https://github.com/yjcyxky/biomedgps.git
cd biomedgps

# Start PostgreSQL and Neo4j with Docker
mkdir biomedgps && cd biomedgps
wget -O docker-compose.yml https://raw.githubusercontent.com/biomedgps/biomedgps/master/docker-compose.yml
docker-compose up -d
```

### Step 2: Install and Build Backend

```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the backend
make build-mac  # For macOS
# or
make build-linux  # For Linux
```

### Step 3: Initialize Database

```bash
# Set environment variables
export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps
export NEO4J_URL=neo4j://neo4j:password@localhost:7687

# Initialize the database schema
biomedgps-cli initdb

# Import sample data (optional)
biomedgps-cli importdb -f /data/entity.tsv -t entity -D
biomedgps-cli importdb -f /data/relation.tsv -t relation -D
```

### Step 4: Set up Frontend

```bash
cd studio

# Install dependencies
yarn install

# Configure Auth0 (replace with your credentials)
# Edit studio/src/app.tsx and update auth0_config

# Start development server
yarn start:local-dev
```

### Step 5: Start the Backend Server

```bash
# Start the server with API and UI
export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps
export NEO4J_URL=neo4j://neo4j:password@localhost:7687

biomedgps -H 0.0.0.0 -p 8888 --openapi --ui
```

## First Steps After Installation

### 1. Access the Platform
- Open your web browser
- Navigate to `http://localhost:8888` (or your configured URL)
- Log in with your Auth0 credentials

### 2. Explore the Dashboard
The dashboard provides an overview of:
- Knowledge graph statistics
- Available datasets
- Recent activity
- Quick access to main features

### 3. Try Key Features

#### Knowledge Graph Exploration
1. Go to **Predict & Explain Findings** → **Explain Your Results**
2. Search for a disease (e.g., "Alzheimer's disease")
3. Explore related genes, drugs, and pathways in the interactive graph

#### Drug Prediction
1. Navigate to **Predict & Explain Findings** → **Predict Drugs/Targets**
2. Enter a disease or gene of interest
3. Run predictions to discover potential therapeutic targets

#### Omics Data Analysis
1. Go to **Analyze Omics Data**
2. Upload your data or use sample datasets
3. Run statistical analysis workflows

## Configuration Options

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@host:port/dbname` |
| `NEO4J_URL` | Neo4j connection string | `neo4j://user:pass@host:port` |
| `AUTH0_CLIENT_ID` | Auth0 application client ID | `your-auth0-client-id` |
| `AUTH0_DOMAIN` | Auth0 domain | `your-domain.auth0.com` |

### Server Options

```bash
biomedgps --help
```

Common options:
- `-H, --host`: Set host address (127.0.0.1 or 0.0.0.0)
- `-p, --port`: Set port number (default: 3000)
- `--openapi`: Enable OpenAPI documentation
- `--ui`: Enable web UI
- `--debug`: Enable debug mode

## Troubleshooting

### Common Issues

**Database Connection Failed**
- Verify PostgreSQL/Neo4j are running
- Check connection strings in environment variables
- Ensure databases are initialized

**Frontend Build Errors**
- Verify Node.js version (must be 16.13.1)
- Clear yarn cache: `yarn cache clean`
- Delete node_modules and reinstall: `rm -rf node_modules && yarn install`

**Auth0 Configuration Issues**
- Check client ID and domain in configuration
- Verify callback URLs in Auth0 dashboard
- Ensure HTTPS is used in production

### Getting Help

- Check our [FAQ](faq.html) for common questions
- Visit [GitHub Issues](https://github.com/yjcyxky/biomedgps/issues) to report bugs
- Review the [User Guide](user-guide.html) for detailed feature documentation

## Next Steps

Now that you have BioMedGPS running:

1. **Read the [User Guide](user-guide.html)** - Learn about all features in detail
2. **Explore [Features](features.html)** - Understand what BioMedGPS can do
3. **Check the [API Reference](api-reference.html)** - For developers building integrations
4. **Join the community** - Contribute to the project on GitHub

Welcome to BioMedGPS! We're excited to see what discoveries you'll make with our platform.