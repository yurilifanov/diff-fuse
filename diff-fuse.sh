#!/usr/bin/env bash

# TODO: check that state is clean
    TMP_DIR="$(mktemp -d -t)"
revisions=$@

for rev in "${revisions[@]}"; do
    svn diff -c "$rev" > "$TMP_DIR"/"$rev".diff
done

svn update "$revisions[0]"
# TODO: go back to previous revision

for rev in "${revisions[@]}"; do
    svn patch "$TMP_DIR"/"$rev".diff
done

svn diff > ./all.diff
echo "Output written to $(pwd)/all.diff"

