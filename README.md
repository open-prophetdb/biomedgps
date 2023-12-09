<h2 align="center">BioMedGPS Platform</h2>
<p align="center">A knowledge graph system with graph neural network for drug discovery, disease mechanism and biomarker screening.</p>

<p align="center">
<img alt="GitHub Workflow Status" src="https://img.shields.io/github/workflow/status/yjcyxky/biomedgps/release?label=Build Status">
<img src="https://img.shields.io/github/license/yjcyxky/biomedgps.svg?label=License" alt="License"> 
<a href="https://github.com/yjcyxky/biomedgps/releases">
<img alt="Latest Release" src="https://img.shields.io/github/release/yjcyxky/biomedgps.svg?label=Latest%20Release"/>
</a>
</p>

<p align="center">NOTE: NOT READY FOR PRODUCTION YET.</p>

## Features

- [x] Knowledge graph studio for graph query, visualization and analysis.
- [x] Graph neural network for drug discovery, disease mechanism, biomarker screening and discovering response to toxicant exposure.
- [x] Support customized knowledge graph schema and data source.
- [x] Support customized graph neural network model.
- [x] Support customized omics datasets.
- [x] Integrated large language models (such as vicuna, rwkv, chatgpt etc. more details on [chat-publications](https://github.com/yjcyxky/chat-publications)) for answering questions.

## Ecosystem

| Name | Language | Description |
| :--- | :--- | :--- |
| [BioMedGPS Data](https://github.com/yjcyxky/biomedgps-data) | Python | For building the knowledge graph of BioMedGPS and training the graph neural network models. |
| [Chat Publications](https://github.com/yjcyxky/chat-publications) | Python | Ask questions and get answers from publications.|
| [BioMedical Knowledgebases](https://github.com/yjcyxky/biomedical-knowledgebases) | Markdown | A collection of biomedical knowledgebases, ontologies, datasets and publications etc. |
| [R Omics Utility](https://github.com/yjcyxky/r-omics-utils) | R | Utilities for omics data with R. It will be part of biomedgps system and provide visulization and analysis functions of multi-omics data. |

## Demo

### Ask questions with chatbot

![chatbot](https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/chatbot.png?raw=true)

### Find similar diseases with your queried disease

![disease](https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/disease-similarities.png?raw=true)

### Predict drugs and related genes for your queried disease

![disease](https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/drug-targets-genes.png?raw=true)

### Find potential paths between two nodes

![path](https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/path.png?raw=true)

## For Users

If you want to use the platform, please follow the instructions below. You can also join us to develop the platform, see [here](#join-us).

`Step 1`: You will need to install PostgreSQL and Neo4j. You can also use docker to launch the database, see [here](#install-docker-and-docker-compose). After that, you can run the following command to launch the database server.

```bash
mkdir biomedgps && cd biomedgps
wget -O docker-compose.yml https://raw.githubusercontent.com/biomedgps/biomedgps/master/docker-compose.yml

docker-compose up -d
```

`Step 2`: Download the latest release from [here](https://github.com/biomedgps/biomedgps/releases). You can also build the platform from source code, see [here](#for-developers).

`Step 3`: Init Database, Check and Import Data

```bash
# Init database
export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && biomedgps-cli initdb

# Check and import data, -t is the table name, -f is the data file path, -D is the delete flag
export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && biomedgps-cli importdb -f /data/entity.tsv -t entity -D

export NEO4J_URL=neo4j://neo4j:password@localhost:7687 && biomedgps-cli importgraph -f /data/relations.tsv -t relation -b 1000

export NEO4J_URL=neo4j://neo4j:password@localhost:7687 && biomedgps-cli importgraph -f /data/entities.tsv -t entity -b 1000
```

`Step 4`: Launch the platform, see more details on usage [here](#usage).

```bash
export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && export NEO4J_URL=neo4j://neo4j:password@localhost:7687/test_biomedgps && biomedgps -H
```

### Usage

Run the following command to see the usage.

```bash
$ biomedgps --help
biomedgps 0.1.0
Jingcheng Yang <yjcyxky@163.com>
BioMedGPS backend server

USAGE:
    biomedgps [FLAGS] [OPTIONS]

FLAGS:
    --debug          Activate debug mode short and long flags (--debug) will be deduced from the field's name
    -h, --help       Prints help information
    -o, --openapi    Activate openapi mode
    -u, --ui         Activate ui mode
    -V, --version    Prints version information

OPTIONS:
    -d, --database-url <database-url>    Database url, such as postgres://user:pass@host:port/dbname. You can also set
                                         it with env var: DATABASE_URL
    -H, --host <host>                    127.0.0.1 or 0.0.0.0 [default: 127.0.0.1]  [possible values: 127.0.0.1,
                                         0.0.0.0]
    -g, --neo4j-url <neo4j-url>          Graph Database url, such as neo4j://user:pass@host:port/dbname. You can also
                                         set it with env var: NEO4J_URL
    -p, --port <port>                    Which port [default: 3000]
```

### Example

If you want to launch the server with openapi and ui mode, run the following command.

```bash
export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && biomedgps -H 0.0.0.0 -p 8888 --openapi --ui
```

### For Linux with systemd

```bash
# You need to place the binary file in /opt/local/bin/biomedgps and all data files in /opt/local/data/biomedgps.
# Or you can change the path in the service file.
sudo cp build/biomedgps.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable biomedgps
sudo systemctl start biomedgps
```

### Install Docker and Docker Compose

```bash
# For MacOSx
brew install docker docker-compose

# For Linux [https://docs.docker.com/engine/install/debian/]
```

## For Developers

### Build docker image

```bash
git clone https://github.com/yjcyxky/biomedgps

cd biomedgps
bash build-docker.sh
```

### Requirements

If you want to build the platform from source code, please follow the instructions below. You can also join us to develop the platform, see [here](#join-us).

#### Server

- Rust
- Cargo
- PostgreSQL
- Neo4j

#### Frontend

- nodejs == 16.13.1
- yarn
- Reactjs
- Ant Design
- Plotly.js
- G6 - Graph Visualization Engine

### Installation (Development) for Server

#### 1. Install Rust and Cargo

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 2. Install PostgreSQL (use docker)

```bash
make test-db
```

#### 3. Run the server

```bash
export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && cargo run --bin biomedgps -- -H 0.0.0.0 -p 8888 --openapi --debug
```

### Installation (Development) for Frontend

#### 1. Install nodejs and yarn

```bash
curl -fsSL https://deb.nodesource.com/setup_16.x | sudo -E bash -
sudo apt-get install -y nodejs
npm install -g yarn

# Check the version. Make sure the nodejs version is 16.x.x. Other versions may not work.
node -v
```

#### 2. Install dependencies

```bash
cd studio
git submodule update --init --recursive

# Install the biominer-components package
cd ../../
git clone https://github.com/yjcyxky/biominer-components.git
cd biominer-components
## Install the dependencies
yarn
## Install the package locally
npm i -g yalc
yalc publish

cd ../biomedgps/studio
yalc add biominer-components
yarn install
```

#### 3. Run the frontend

```bash
yarn start:local-dev
```

### Build for Production

#### Compile the `biomedgps` and `biomedgps-cli` binaries

After installing the dependencies, run the following command to build the frontend and backend. The output will be in `target/release` folder. You will get a binary file named `biomedgps`.

```bash
# For MacOSx
make build-mac

# For Linux
make build-linux
```

#### Install docker and docker-compose

```bash
# For MacOSx
brew install docker docker-compose

# For Linux [https://docs.docker.com/engine/install/debian/]
```

#### Launch the database and server

1. Download docker directory
2. Run `docker-compose up -d` to launch the database

    ```bash
    cd docker && docker-compose up -d
    ```

3. Upload `biomedgps` and `biomedgps-cli` binaries to the server. `/opt/local/bin/biomedgps` and `/opt/local/bin/biomedgps-cli` are the default paths.

4. Run the following command to initialize the database

    ```bash
    export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && /opt/local/bin/biomedgps-cli initdb
    ```

5. Run the following command to upload the data

    Assume all data files are in `/data` folder. You need to ensure that the data directory is mounted to the docker container.

    ```bash
    # Upload entity data
    export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && /opt/local/bin/biomedgps-cli importdb -f /data/entity -t entity -D

    # Upload entity2d data
    export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && /opt/local/bin/biomedgps-cli importdb -f /data/entity2d.tsv -t entity2d -D

    # Upload relation data
    export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && /opt/local/bin/biomedgps-cli importdb -f /data/relation -t relation -D

    ...
    ```

6. Configure a nginx server to serve the frontend and backend

    - Download biomedgps_nginx.conf and modify it to fit your environment

    - Upload the modified biomedgps_nginx.conf to `/etc/nginx/config.d/biomedgps_nginx.conf`

    - Restart nginx

    ```bash
    systemctl restart nginx
    ```

## Join Us

## Contributing

Comming soon...

## License

Copyright © 2022 Jingcheng Yang

Distributed under the terms of the GNU Affero General Public License v3.0.