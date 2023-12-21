#!/usr/bin/env bash

UNDERLINE="==================================================================="
# TMP_DIR="$(mktemp -d -t tmp)"
TMP_DIR=./tmp
mkdir -p "$TMP_DIR"
echo "TMP_DIR=$TMP_DIR"

target="$1"
initial_commit="$2"
intermediate_commit="$3"
final_commit="$4"

function copy_state {
    git show "HEAD~$1":"$target" > "$TMP_DIR"/"$2"
}

function make_diff {
    local initial="HEAD~$1"
    local final="HEAD~$2"
    local name="$3"

    git diff --no-prefix --ignore-space-at-eol "$initial" "$final" "$target" | \
        sed -e "s/^diff --git [^[:space:]]*/Index:/" \
        -e "s/^index.*/$UNDERLINE/" \
        -e "s/^@@\(.*\)@@.*/@@\1@@/" \
        > "$TMP_DIR"/$name
}

copy_state $initial_commit initial.${target##*.}
copy_state $final_commit final.${target##*.}

make_diff $initial_commit $intermediate_commit left.diff
make_diff $intermediate_commit $final_commit right.diff
make_diff $initial_commit $final_commit expected.diff

cargo run -- "$TMP_DIR"/left.diff "$TMP_DIR"/right.diff > "$TMP_DIR"/actual.diff

patch "$TMP_DIR"/initial.${target##*.} \
    -i "$TMP_DIR"/actual.diff \
    -o "$TMP_DIR"/result.${target##*.}

diff "$TMP_DIR"/final.${target##*.} "$TMP_DIR"/result.${target##*.}

