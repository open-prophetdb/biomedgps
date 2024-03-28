import click
import os
import xml.etree.ElementTree as ET
import json
from collections import defaultdict

cli = click.Group()

def remove_namespace_and_hyphen(tag):
    """Remove the namespace from the tag name and replace hyphens with underscores."""
    # Remove namespace
    tag = tag.split('}')[-1]
    # Replace hyphens with underscores
    tag = tag.replace('-', '_')
    return tag

def parse_element(elem):
    """Recursively parse an XML element into a dictionary or a list, ignoring attributes."""
    # Check for text content directly in this element
    if elem.text and elem.text.strip():
        return elem.text.strip()
    else:
        result = {}

    for name, value in elem.attrib.items():
        result[remove_namespace_and_hyphen(name)] = value

    # Group children by tag to decide on list or single item
    children_by_tag = {}
    for child in elem:
        key = remove_namespace_and_hyphen(child.tag)
        children_by_tag.setdefault(key, []).append(parse_element(child))

    # Add children to result, converting to lists or dicts as appropriate
    for key, children in children_by_tag.items():
        if len(children) > 1:
            result[key] = children  # Set as list if multiple children with same tag
        else:
            result[key] = children[0]  # Single item, not a list

    return result or None

def unify_to_array(value):
    if value is None:
        return []
    elif isinstance(value, list):
        return value
    else:
        return [value]
    
def check_singular_plural(singular, plural):
    if singular == "category" and plural == "categories":
        return True
    return singular + "s" == plural

def set_default_if_empty(nested_dict, path, default):
    """
    Sets a default value in a nested dictionary based on a given path if the final key is not set or empty.
    
    :param nested_dict: The nested dictionary to modify.
    :param path: A list of keys representing the path to the target value.
    :param default: The default value to set if the target is not set or empty.
    """
    if isinstance(nested_dict, list):
        return [set_default_if_empty(item, path[1:], default) for item in nested_dict]
    
    if isinstance(nested_dict, dict):
        # Navigate through the nested dictionary along the path, except for the last key
        current_level = nested_dict

        for key in path[:-1]:
            if key in current_level:
                if current_level[key] is None or current_level[key] == "":
                    return current_level
                elif isinstance(current_level[key], list):
                    current_level[key] = [set_default_if_empty(item, path[1:], default) for item in current_level[key]]
                elif isinstance(current_level[key], dict):
                    current_level[key] = set_default_if_empty(current_level[key], path[1:], default)

        final_key = path[-1]
        if final_key in current_level and current_level[final_key] is not None:
            if (isinstance(current_level[final_key], str) or 
                isinstance(current_level[final_key], dict)) and isinstance(default, list):
                current_level[final_key] = [current_level[final_key]]

        return current_level


def transform_json(obj):
    if isinstance(obj, dict):
        new_obj = {}
        for key, value in obj.items():
            if isinstance(value, dict) and len(value.keys()) == 1 and check_singular_plural(list(value.keys())[0], key):
                subkey = list(value.keys())[0]
                new_obj[key] = unify_to_array(
                    transform_json(value[subkey])
                )
            else:
                new_obj[key] = transform_json(value)

        return new_obj
    elif isinstance(obj, list):
        return [transform_json(item) for item in obj]

    return obj

def check_types(data, parent_key='', path_types=defaultdict(list)):
    if isinstance(data, dict):
        for key, value in data.items():
            if isinstance(value, (dict, list)):
                check_types(value, f"{parent_key}.{key}" if parent_key else key, path_types)
            else:
                path_types[f"{parent_key}.{key}" if parent_key else key].append(type(value).__name__)
    elif isinstance(data, list):
        for i, item in enumerate(data):
            check_types(item, f"{parent_key}[{i}]" if parent_key else str(i), path_types)
    else:
        path_types[parent_key].append(type(data).__name__)

    return path_types

def find_inconsistencies(path_types):
    inconsistencies = {}
    for path, types in path_types.items():
        if len(set(types)) > 1:  # More than one unique type for the path
            inconsistencies[path] = set(types)
    return inconsistencies

@cli.command(help="Converts a DrugBank XML file to a JSON file.")
@click.option('--input', '-i', required=True, type=click.Path(exists=True, file_okay=True, dir_okay=False), help="Path to the DrugBank XML file.")
@click.option('--output', '-o', required=True, type=click.Path(exists=True, file_okay=False, dir_okay=True), help="Path to the output directory for the JSON file.")
def tojson(input, output):
    # Load the XML file
    print(f'Converting {input} to JSON...')
    tree = ET.parse(input)
    root = tree.getroot()

    # Extract the version and export date from the root element for the filename
    version = root.attrib['version']
    exported_on = root.attrib['exported-on']

    # Process all 'drug' elements
    print('Processing drug elements...')
    drugs_data = [parse_element(drug) for drug in root.findall('{http://www.drugbank.ca}drug')]
    drugs_data = transform_json([drug for drug in drugs_data])

    # You can add more processing here to keep the type of the data consistent
    drugs_data = [set_default_if_empty(drug, ['drugbank_id'], []) for drug in drugs_data]
    drugs_data = [set_default_if_empty(drug, ['targets', 'polypeptide'], []) for drug in drugs_data]
    drugs_data = [set_default_if_empty(drug, ['pathways', 'enzymes', 'uniprot_id'], []) for drug in drugs_data]

    # Prepare the output JSON file path using version and exported-on attributes
    json_file_path = f'{output}/drugbank_{version}_{exported_on}.json'

    # Save the processed data to a JSON file
    print(f'Saving JSON file to {json_file_path}...')
    with open(json_file_path, 'w', encoding='utf-8') as json_file:
        json.dump(drugs_data, json_file, ensure_ascii=False, indent=4)


@click.command(help="Converts a DrugBank json file to a parquet file.")
@click.option('--input', '-i', required=True, type=click.Path(exists=True, file_okay=True, dir_okay=False), help="Path to the DrugBank JSON file.")
@click.option('--output', '-o', required=True, type=click.Path(exists=True, file_okay=False, dir_okay=True), help="Path to the output directory for the Parquet file.")
def toparquet(input, output):
    import pandas as pd
    import pyarrow as pa
    import pyarrow.parquet as pq

    # Load the JSON file
    print(f'Converting {input} to Parquet...')
    with open(input, 'r', encoding='utf-8') as json_file:
        drugs_data = json.load(json_file)

    # Convert the JSON data to a DataFrame
    df = pd.json_normalize(drugs_data)

    # 新增类型检查代码
    path_types = check_types(drugs_data)
    inconsistencies = find_inconsistencies(path_types)
    if inconsistencies:
        print("Found inconsistencies in the data types after json_normalize:")
        for path, types in inconsistencies.items():
            print(f"Path: {path}, Types: {types}")
    else:
        print("No inconsistencies found in the data types after json_normalize.")

    # Prepare the output Parquet file path
    parquet_file_path = f'{output}/{os.path.splitext(os.path.basename(input))[0]}.parquet'

    # Save the DataFrame to a Parquet file
    print(f'Saving Parquet file to {parquet_file_path}...')
    table = pa.Table.from_pandas(df)
    pq.write_table(table, parquet_file_path)

cli.add_command(tojson)
cli.add_command(toparquet)

if __name__ == '__main__':
    cli()