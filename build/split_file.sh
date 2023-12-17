#!/bin/bash

# File to split
input=$1
num=$2

if [ -z "$input" ]; then
    echo "Usage: $0 <file>"
    exit 1
fin

if [ -z "$num" ]; then
    num=1000000
fi

# Header extraction
header=$(head -n 1 "$input")

# Split file, excluding the header
tail -n +2 "$input" | split -l $num - splitted_file_

# Add header to each part
for file in splitted_file_*; do
    (echo "$header"; cat "$file") > tmp && mv tmp "$file".tsv
done
