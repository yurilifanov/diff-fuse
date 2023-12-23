#!/usr/bin/env bash

base_dir=$(dirname $(realpath $0))

for path in $(find src -type f); do
    echo $path
    bash "$base_dir"/test_file.sh "$path" 10 5 1
done

