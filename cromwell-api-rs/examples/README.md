## How to test the workflow

1. Start the cromwell server

```bash
java -jar cromwell-82.jar server
```

2. Install the dependencies

```bash
conda activate biomedgps
conda install -c conda-forge xxx

# Example for installing the R package
R -e "devtools::install_git('file:///Users/jy006/Documents/Code/BioMiner/omix-insight-r')"
```

3. Run the workflow

```bash
cargo run --example hello_world
```

## How to deploy the workflow

1. Rsync the workflow to the server

```bash
rsync -avP ./cromwell-api-rs/examples/boxplot/ root@drugs.3steps.cn:/data/biomedgps/cromwell/workflows/boxplot-v0.1.0
```

2. Update the dependencies

```bash
# Login to the server

# Update the dependencies
/opt/miniconda3/envs/biomedgps/bin/R -e "devtools::install_git('file:///data/biomedgps/cromwell/workflows/boxplot-v0.1.0')"
```
