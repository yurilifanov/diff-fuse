all: build svn

TARGETS := generate-svn/merge/success/1 generate-svn/merge/success/2

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
