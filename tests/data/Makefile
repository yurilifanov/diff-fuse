UNDERLINE := ===================================================================

TARGETS := generate-svn/merge/success/1 generate-svn/merge/success/2

all: build svn

# For when "Error: failed to connect: dial tcp 127.0.0.1:XXXXX: ..."
podman-reset-vm:
	podman machine rm podman-machine-default
	podman machine init
	podman machine start

# For when "Error: statfs ... no such file or directory"
podman-restart:
	podman machine stop
	podman machine start

$(TARGETS): generate-%:
	podman run --rm -v ./$*:/src svn bash generate.sh
	rm -rf ./$*/.svn

.PHONY: svn
svn:
	podman run --rm -it svn bash

build:
	podman build . -t svn


generate-svn/merge/success/3: generate-%:
	mkdir -p "$*"
	git diff --no-prefix --ignore-space-at-eol HEAD~10 HEAD~5 ../../src/file_diff.rs | \
		sed -e "s/^diff --git [^[:space:]]*/Index:/" \
			-e "s/^index.*/$(UNDERLINE)/" \
			-e "s/^@@\(.*\)@@.*/@@\1@@/" \
			> "$*"/left.diff

	git diff --no-prefix --ignore-space-at-eol HEAD~5 HEAD ../../src/file_diff.rs | \
		sed -e "s/^diff --git [^[:space:]]*/Index:/" \
			-e "s/^index.*/$(UNDERLINE)/" \
			-e "s/^@@\(.*\)@@.*/@@\1@@/" \
			> "$*"/right.diff

	git diff --no-prefix --ignore-space-at-eol HEAD~10 HEAD ../../src/file_diff.rs | \
		sed -e "s/^diff --git [^[:space:]]*/Index:/" \
			-e "s/^index.*/$(UNDERLINE)/" \
			-e "s/^@@\(.*\)@@.*/@@\1@@/" \
			> "$*"/expected.diff
