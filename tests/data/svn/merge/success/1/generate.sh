#!/usr/bin/env bash

base_dir="$(dirname $(realpath $0))"

cp "$base_dir"/initial.txt ./text.txt
svn add text.txt
svn commit -m "initial"

cp "$base_dir"/intermediate.txt ./text.txt
svn diff > left.diff
svn commit -m "intermediate"

cp "$base_dir"/final.txt ./text.txt
svn diff > right.diff
svn commit -m "final"

svn diff -r 2:HEAD > expected.diff
rm text.txt
