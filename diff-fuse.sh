#!/usr/bin/env bash
set -e

if [ ! -z "$(svn status)" ]; then
	echo "Directory tree not clean. Make sure 'svn status' output is empty"
	exit 1
fi

function get_prev_revision {
	echo -e "$(svn log -l 2 | grep '^r[0-9]* ' -o | tail -n 1 | tr -d 'r ')"
}

TMP_DIR="$(mktemp -d -t)"
revisions=$@

for rev in $revisions; do
	svn diff -c "$rev" >"$TMP_DIR"/"$rev".diff
done

svn update -r "$(echo $revisions | cut -d ' ' -f 1)"
svn update -r "$(get_prev_revision)"

for rev in $revisions; do
	svn patch "$TMP_DIR"/"$rev".diff
done

svn diff >./all.diff
echo "Output written to $(pwd)/all.diff"

rm -rf "$TMP_DIR"
svn update .

