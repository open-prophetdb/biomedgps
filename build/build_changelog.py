import subprocess
import re
import click
from collections import defaultdict


@click.command("Build Changelog")
@click.option("--repo", help="The repository to build the changelog for", required=True)
@click.option("--repo-name", help="The name of the repository to use in the changelog", required=True)
@click.option("--output-file", help="The file to write the changelog to", required=True)
def build(repo, output_file, repo_name):
    changelog = defaultdict(set)
    current_date = ""

    print(f"Building changelog for {repo_name}")
    output = subprocess.check_output(['git', 'log', '--pretty=format:%ai %d %s', '--date=short', '--reverse'], cwd=repo)
    log = output.decode('utf-8')

    for line in log.split('\n'):
        date_match = re.search(r'^(\d{4}-\d{2}-\d{2})', line)
        if date_match:
            current_date = date_match.group(1)

        log_message = re.sub(r'\s*\([^)]*\)\s*', ' ', line[11:]).strip()
        log_message = re.sub(r"\d{2}:\d{2}:\d{2}\s+-\d{4}\s+", "", log_message)

        log_message = re.sub(r"Upgrade biominer-components to v?\d+\.\d+\.\d+", "Upgrade the BioMedGPS UI", log_message)

        if log_message:
            changelog[current_date].add(log_message)

    try:
        with open(output_file, "r") as file:
            existing_content = file.read()
    except FileNotFoundError:
        existing_content = ""

    new_log_content = f"## {repo_name}\n\n"
    for date, messages in sorted(changelog.items(), reverse=True):
        new_log_content += f"- {date}\n"
        for message in messages:
            new_log_content += f"  - {message}\n"
        new_log_content += "\n"

    try:
        with open(output_file, "r+") as file:
            existing_content = file.read()

            pattern = re.compile(rf"## {repo_name}\n\n(.*?)(\n## |\Z)", re.DOTALL)
            match = pattern.search(existing_content)
            if match:
                updated_content = pattern.sub(new_log_content + r"\2", existing_content, count=1)
            else:
                updated_content = new_log_content + existing_content

            file.seek(0)
            file.write(updated_content)
            file.truncate()
    except FileNotFoundError:
        with open(output_file, "w") as file:
            file.write(new_log_content)

    print(updated_content)


if __name__ == "__main__":
    build()