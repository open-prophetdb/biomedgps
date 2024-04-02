## Requirements

- Python 3.11+
- json2parquet

```bash
# macOSx
brew install domoritz/homebrew-tap/json2parquet

# Linux
cargo install json2parquet
```

- Required Python packages: `pip install click duckdb`

## Prepare additional data for each entity and relation

### Compound

Get additional data for each compound from [DrugBank](https://www.drugbank.ca/). You might need to request access to the DrugBank data. If you have access, download the DrugBank XML file and save it to the `data` directory. We assume the file is named `drugbank_5.1_2024-01-03.xml`.

```bash
python3 data/drugbank.py tojson --input-file data/drugbank/drugbank_5.1_2024-01-03.xml --output-dir data/drugbank

python3 data/drugbank.py tojson --input-file data/drugbank/drugbank_5.1_2024-01-03.xml --output-dir data/drugbank --format linejson
json2parquet data/drugbank/drugbank_5.1_2024-01-03.jsonl data/drugbank/drugbank_5.1_2024-01-03.parquet
```

### Gene