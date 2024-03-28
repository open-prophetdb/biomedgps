## Prepare additional data for each entity and relation

### Compound

Get additional data for each compound from [DrugBank](https://www.drugbank.ca/).

```bash
python3 data/drugbank.py tojson --input xxx.xml --output data

python3 data/drugbank.py toparquet --input data/drugbank_5.1_2024-01-03.json --output data
```

### Gene