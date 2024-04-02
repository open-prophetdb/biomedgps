import click
import os
import csv
import tempfile
import xml.etree.ElementTree as ET
import json

cli = click.Group()


@cli.command(help="Converts a DrugBank XML file to a JSON file.")
@click.option(
    "--input-file",
    "-i",
    required=True,
    type=click.Path(exists=True, file_okay=True, dir_okay=False),
    help="Path to the DrugBank XML file.",
)
@click.option(
    "--output-dir",
    "-o",
    required=True,
    type=click.Path(exists=True, file_okay=False, dir_okay=True),
    help="Path to the output directory for the JSON file.",
)
@click.option(
    "--format",
    "-f",
    type=click.Choice(["json", "linejson", "tsv"], case_sensitive=False),
    default="json",
    help="The format of the JSON file. Default is 'json'. The 'linejson' format writes each JSON object on a separate line.",
)
def tojson(input_file, output_dir, format):
    # Extracted from the DrugBank XML file
    def remove_namespace_and_hyphen(tag):
        """Remove the namespace from the tag name and replace hyphens with underscores."""
        # Remove namespace
        tag = tag.split("}")[-1]
        # Replace hyphens with underscores
        tag = tag.replace("-", "_")
        return tag

    def parse_element(elem):
        """Recursively parse an XML element into a dictionary or a list, ignoring attributes."""
        # Check for text content directly in this element
        if elem.text and elem.text.strip() and not elem.attrib:
            return elem.text.strip()
        else:
            result = {}

        # Add text content if present
        if elem.text and elem.text.strip() and elem.attrib:
            result["text"] = elem.text.strip()

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

    # For the JSON transformation
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
            return [
                set_default_if_empty(item, path[1:], default) for item in nested_dict
            ]

        if isinstance(nested_dict, dict):
            # Navigate through the nested dictionary along the path, except for the last key
            current_level = nested_dict

            for key in path[:-1]:
                if key in current_level:
                    if current_level[key] is None or current_level[key] == "":
                        return current_level
                    elif isinstance(current_level[key], list):
                        current_level[key] = [
                            set_default_if_empty(item, path[1:], default)
                            for item in current_level[key]
                        ]
                    elif isinstance(current_level[key], dict):
                        current_level[key] = set_default_if_empty(
                            current_level[key], path[1:], default
                        )

            final_key = path[-1]
            if final_key in current_level:
                if current_level[final_key] is not None:
                    if (
                        isinstance(current_level[final_key], str)
                        or isinstance(current_level[final_key], dict)
                    ) and isinstance(default, list):
                        current_level[final_key] = [current_level[final_key]]

                if current_level[final_key] is None or current_level[final_key] == "":
                    current_level[final_key] = default

            return current_level

    def transform_json(obj):
        """Transforms the JSON object to a consistent format. We want to ensure that all fields at the same level have the same type.

        Args:
            obj (dict): The JSON object to transform.

        Returns:
            object: The transformed JSON object.
        """
        if isinstance(obj, dict):
            new_obj = {}
            for key, value in obj.items():
                if (
                    isinstance(value, dict)
                    and len(value.keys()) == 1
                    and check_singular_plural(list(value.keys())[0], key)
                ):
                    subkey = list(value.keys())[0]
                    new_obj[key] = unify_to_array(transform_json(value[subkey]))
                else:
                    new_obj[key] = transform_json(value)

            return new_obj
        elif isinstance(obj, list):
            return [transform_json(item) for item in obj]

        return obj

    # Load the XML file
    print(f"Converting {input_file} to {format}...")
    tree = ET.parse(input_file)
    root = tree.getroot()

    # Extract the version and export date from the root element for the filename
    version = root.attrib["version"]
    exported_on = root.attrib["exported-on"]

    # Process all 'drug' elements
    print("Processing drug elements...")
    drugs_data = [
        parse_element(drug) for drug in root.findall("{http://www.drugbank.ca}drug")
    ]
    drugs_data = transform_json([drug for drug in drugs_data])

    def format_drugbank_id(drug):
        if "drugbank_id" in drug:
            drugbank_ids = drug["drugbank_id"]
            if isinstance(drugbank_ids, list):
                drug["drugbank_id"] = [
                    drug_id["text"] if isinstance(drug_id, dict) else drug_id
                    for drug_id in drugbank_ids
                ]
            elif isinstance(drugbank_ids, dict):
                drug["drugbank_id"] = drugbank_ids["text"]

        return drug

    # Fix drugbank_id, we don't want a mixed type, string or dict, for this field
    drugs_data = [format_drugbank_id(drug) for drug in drugs_data]

    def format_synonyms(drug):
        if "synonyms" in drug:
            synonyms = drug["synonyms"]
            if isinstance(synonyms, list):
                drug["synonyms"] = [
                    synonym["text"] if isinstance(synonym, dict) else synonym
                    for synonym in synonyms
                ]

        return drug

    # Fix synonyms, we don't want a complex type, dict, for this field. We want a list of strings.
    drugs_data = [format_synonyms(drug) for drug in drugs_data]

    # Fix drugbank_id in salts, we don't want a mixed type, string or dict, for this field
    def format_salts_drugbank_id(drug):
        if "salts" in drug:
            salts = drug["salts"]
            if isinstance(salts, list):
                for salt in salts:
                    if "drugbank_id" in salt:
                        drugbank_ids = salt["drugbank_id"]
                        if isinstance(drugbank_ids, dict):
                            salt["drugbank_id"] = drugbank_ids["text"]

        return drug

    drugs_data = [format_salts_drugbank_id(drug) for drug in drugs_data]

    # You can add more processing here to keep the type of the data consistent. These fields might have mixed types, e.g string and list.
    uncorrected_paths = [
        [["drugbank_id"], []],
        [["targets", "polypeptide"], []],
        [["pathways", "enzymes", "uniprot_id"], []],
        [["experimental_properties", "property"], []],
        [["snp_effects", "effect"], []],
        [["calculated_properties", "property"], []],
        [["snp_adverse_drug_reactions", "reaction"], []],
        [["classification", "alternative_parent"], []],
        [["classification", "substituent"], []],
        [["pdb_entries", "pdb_entry"], []],
        [["enzymes", "polypeptide"], []],
        [["carriers", "polypeptide"], []],
        [["transporters", "polypeptide"], []],
        [["products", "ndc_id"], ""],
        [["ahfs_codes"], []],
        [["pdb_entries"], []],
        [["snp_effects"], []],
        [["snp_adverse_drug_reactions"], []],
    ]

    for path, default in uncorrected_paths:
        drugs_data = [set_default_if_empty(drug, path, default) for drug in drugs_data]

    # We don't like the following fields, because they might cause issues when we import the data into a database.
    for drug in drugs_data:
        if "type" in drug:
            drug["compound_type"] = drug["type"]
            del drug["type"]

        if "state" in drug:
            drug["compound_state"] = drug["state"]
            del drug["state"]

        if "drugbank_id" in drug:
            drug["xrefs"] = drug["drugbank_id"]
            drug["drugbank_id"] = "DrugBank:" + drug["drugbank_id"][0] if drug["drugbank_id"][0].startswith("DB") else drug["drugbank_id"][0]

    # Prepare the output JSON file path using version and exported-on attributes
    json_file_path = os.path.join(output_dir, f"drugbank_{version}_{exported_on}.json")

    # Save the processed data to a JSON file
    print(f"Saving JSON file to {json_file_path}...")

    def format_value(value):
        if isinstance(value, list):
            formatted_elements = [format_value(item) for item in value]
            return "{" + ",".join(formatted_elements) + "}"
        elif isinstance(value, dict):
            return json.dumps(value, ensure_ascii=False)
        elif isinstance(value, str):
            escaped_value = value.replace("\\", "\\\\").replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t")
            return f'"{escaped_value}"'
        else:
            return str(value)


    def save_data_as_tsv(data, json_file_path):
        json_file_path = json_file_path.replace(".json", ".tsv")
        all_fields = [set(item.keys()) for item in data]
        common_fields = set.intersection(*all_fields)

        with open(json_file_path, "w", encoding="utf-8", newline="") as tsv_file:
            writer = csv.DictWriter(
                tsv_file,
                fieldnames=list(common_fields),
                delimiter="\t",
                extrasaction="ignore",
            )
            writer.writeheader()
            for item in data:
                row = {
                    field: format_value(item[field])
                    for field in common_fields
                    if field in item
                }
                writer.writerow(row)

    output_file = tempfile.NamedTemporaryFile(delete=True).name
    with open(output_file, "w", encoding="utf-8") as json_file:
        json.dump(drugs_data, json_file, ensure_ascii=False, indent=4)

    data = checktypes_wrapper(output_file, json_file_path)
    if format == "linejson":
        json_file_path = json_file_path.replace(".json", ".jsonl")
        with open(json_file_path, "w", encoding="utf-8") as json_file:
            for drug in data:
                json_file.write(json.dumps(drug, ensure_ascii=False) + "\n")
    elif format == "tsv":
        save_data_as_tsv(data, json_file_path)


@cli.command(help="Check the types of the data in a JSON file.")
@click.option(
    "--input-file",
    "-i",
    required=True,
    type=click.Path(exists=True, file_okay=True, dir_okay=False),
    help="Path to the JSON file.",
)
@click.option(
    "--output-file",
    "-o",
    required=False,
    type=click.Path(exists=False, file_okay=True, dir_okay=False),
    help="Path to the output file for the type check results.",
)
def checktypes(input_file, output_file=None):
    checktypes_wrapper(input_file, output_file)

def checktypes_wrapper(input_file, output_file=None):
    import json
    from collections import defaultdict

    def fix_none(data, path, default=""):
        """Fix None values in a nested data structure based on a given path.

        Args:
            data: The nested data structure to fix.
            path: A list of keys representing the path to the target value. such as ["snp_effects", "effect[]", "rs_id"]
            default: The default value to set if the target is None. Default is an empty string.

        Returns:
            The nested data structure with the None values fixed.
        """
        if not path:
            return

        current_path = path[0]
        next_path = path[1:]

        if isinstance(data, list):
            # If the current path is a list, recursively fix None values for each item in the list
            for item in data:
                fix_none(item, path, default)
        elif isinstance(data, dict):
            if "[]" in current_path:
                # Deal with the list, remove '[]', and recursively process the list elements
                key = current_path.replace("[]", "")
                if key in data and isinstance(data[key], list):
                    for item in data[key]:
                        fix_none(item, next_path, default)
            else:
                # Deal with the dict, recursively process the dict elements
                if current_path in data:
                    if next_path:
                        fix_none(data[current_path], next_path, default)
                    else:
                        # If we reach the end of the path, set the default value if the current value is None
                        if data[current_path] is None:
                            data[current_path] = default

    def collect_types(obj, path="", type_map=defaultdict(set)):
        """Collects the types of the data in a nested structure while traversing it and stores the types in a dictionary.

        Args:
            obj (dict): The nested data structure to traverse.
            path (str): The current path in the nested data structure.
            type_map (dict): The dictionary to store the types of the data.

        Returns:
            dict: The dictionary containing the types of the data.
        """
        if isinstance(obj, dict):
            # Record the dictionary type directly and recursively process each key-value pair
            type_map[path].add("Dict")
            for k, v in obj.items():
                new_path = f"{path} > {k}" if path else k
                collect_types(v, new_path, type_map)
        elif isinstance(obj, list):
            # Record the list type as Array and recursively process each item
            type_map[path].add("Array")
            for item in obj:
                new_path = f"{path}[]" if path else "[]"
                collect_types(item, new_path, type_map)
        elif obj is None:
            # Record the None type
            type_map[path].add("NoneType")
        else:
            # Record the type of the object
            type_map[path].add(type(obj).__name__)

    def find_inconsistencies(type_map):
        """Prints the paths with inconsistent data types found in the type map.

        Args:
            type_map (dict): The dictionary containing the types of the data.
        """
        print("\n\nInconsistent data types found:")
        for path, types in type_map.items():
            if len(types) > 1:
                if "NoneType" in types and len(types) == 2:
                    continue

                print(f"Path: {path}, ValueTypes: {', '.join(types)}")

    def find_consistent_paths(type_map):
        """Prints the paths with consistent data types found in the type map.

        Args:
            type_map (dict): The dictionary containing the types of the data.
        """
        print("\nConsistent data types found:")
        for path, types in type_map.items():
            if len(types) == 1:
                formatted_path = path.lstrip(">")
                print(f"Path: {formatted_path}, ValueType: {', '.join(types)}")
            if len(types) == 2 and "NoneType" in types:
                formatted_path = path.lstrip(">")
                print(f"Path: {formatted_path}, ValueType: {', '.join(types)}")

    # Read the JSON file
    with open(input_file, "r") as f:
        data = json.load(f)

        # Ensure the data is a list
        if not isinstance(data, list):
            raise ValueError("JSON data is not a list.")

        # Collect the types of the data
        type_map = defaultdict(set)
        for item in data:
            collect_types(item, type_map=type_map)

        find_consistent_paths(type_map)
        find_inconsistencies(type_map)

    if output_file:
        for path, types in type_map.items():
            paths = path.split(" > ")
            types = sorted(list(types))
            if types == ["NoneType", "str"]:
                fix_none(data, paths, default="")
            elif types == ["Array", "NoneType"]:
                fix_none(data, paths, default=[])
            elif types == ["str"]:
                fix_none(data, paths, default="")

        with open(output_file, "w") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)

        return data


cli.add_command(tojson)
cli.add_command(checktypes)

if __name__ == "__main__":
    cli()
