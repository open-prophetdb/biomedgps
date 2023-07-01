<h2 align="center">BioMedGPS</h2>
<p align="center">A knowledge graph system with graph neural network for drug discovery, disease mechanism and biomarker screening.</p>

<p align="center">
<img alt="GitHub Workflow Status" src="https://img.shields.io/github/workflow/status/yjcyxky/biomedgps/release?label=Build Status">
<img src="https://img.shields.io/github/license/yjcyxky/biomedgps.svg?label=License" alt="License"> 
<a href="https://github.com/yjcyxky/biomedgps/releases">
<img alt="Latest Release" src="https://img.shields.io/github/release/yjcyxky/biomedgps.svg?label=Latest%20Release"/>
</a>
</p>

<p align="center">NOTE: NOT READY FOR PRODUCTION YET.</p>

## Requirements

### Server
- Rust
- Cargo
- PostgreSQL
- Neo4j

### Frontend

- nodejs == 16.13.1
- yarn

## Installation (Development) for Server

### 1. Install Rust and Cargo

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install PostgreSQL (use docker)

```bash
make test-db
```

### 3. Run the server

```bash
export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && cargo run -- -H 0.0.0.0 -p 8888 -D data --openapi --debug
```

## Installation (Development) for Frontend

### 1. Install nodejs and yarn

```bash
curl -fsSL https://deb.nodesource.com/setup_16.x | sudo -E bash -
sudo apt-get install -y nodejs
npm install -g yarn
```

### 2. Install dependencies

```bash
cd studio
yarn install
```

### 3. Run the frontend

```bash
yarn start:local-dev
```

## Build for Production

### Compile the `biomedgps` and `biomedgps-cli` binaries

After installing the dependencies, run the following command to build the frontend and backend. The output will be in `target/release` folder. You will get a binary file named `biomedgps`.

```bash
# For MacOSx
make build-mac

# For Linux
make build-linux
```

### Install docker and docker-compose

```bash
# For MacOSx
brew install docker docker-compose

# For Linux [https://docs.docker.com/engine/install/debian/]
```

### Launch the database and server

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

    Assume all data files are in `/data` folder.

    ```bash
    # Upload dataset data
    export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && /opt/local/bin/biomedgps-cli import -f /data/dataset.tsv -t dataset -D

    # Upload count data
    export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && /opt/local/bin/biomedgps-cli import -f /data/rnmp_count_output/ -t count -D

    # Upload background_frequency data
    export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && /opt/local/bin/biomedgps-cli import -f /data/background_frequency.tsv -t background_frequency -D
    ```

6. Upload all bedgraph files to `/data/bedgraph` folder

7. Upload all bigwig files to `/data/bigwig` folder

8. Upload all genome files to `/data/genome` folder

9. Configure a nginx server to serve the frontend and backend

    - Download biomedgps_nginx.conf and modify it to fit your environment

    - Upload the modified biomedgps_nginx.conf to `/etc/nginx/config.d/biomedgps_nginx.conf`

    - Restart nginx

    ```bash
    systemctl restart nginx
    ```

## Usage

Run the following command to see the usage.

```bash
$ biomedgps --help
biomedgps 0.1.0
Jingcheng Yang <yjcyxky@163.com>
rNMP Database

USAGE:
    biomedgps [FLAGS] [OPTIONS] --data-dir <data-dir>

FLAGS:
        --debug      Activate debug mode short and long flags (--debug) will be deduced from the field's name
    -h, --help       Prints help information
    -o, --openapi    Activate openapi mode
    -u, --ui         Activate ui mode
    -V, --version    Prints version information

OPTIONS:
    -D, --data-dir <data-dir>            Data directory. It is expected to contain bedgraph files and reference genomes
    -d, --database-url <database-url>    Database url, such as postgres:://user:pass@host:port/dbname. You can also set
                                         it with env var: DATABASE_URL
    -H, --host <host>                    127.0.0.1 or 0.0.0.0 [default: 127.0.0.1]  [possible values: 127.0.0.1,
                                         0.0.0.0]
    -p, --port <port>                    Which port [default: 3000]
```

## Example

If you want to launch the server with openapi and ui mode, run the following command. The data directory must contain bedgraph, bigwig and genome subdirectories. The bedgraph files must be named as `*.bedgraph` and the genome files must be named as `*.fa.gz`, `*.fa.gz.fai`, `*.fa.gz.gzi`, `*.gff3.gz`, `*.gff3.gz.tbi`, `*.gff3.gz.ix`, `*.gff3.gz.ixx` and `*.gff3.gz_meta.json`.

```bash
export DATABASE_URL=postgres://postgres:password@localhost:5432/test_biomedgps && biomedgps -H 0.0.0.0 -p 8888 -D data --openapi --ui
```

## For Linux with systemd

```bash
# You need to place the binary file in /opt/local/bin/biomedgps and all data files in /opt/local/data/biomedgps.
# Or you can change the path in the service file.
sudo cp build/biomedgps.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable biomedgps
sudo systemctl start biomedgps
```

## Issues
1. Species names are not consistent in the `biomedgps_background_frequency`, `biomedgps_dataset` and `biomedgps_count` tables.

## Contributing
Comming soon...

## License
Copyright © 2022 Jingcheng Yang

Distributed under the terms of the GNU Affero General Public License v3.0.