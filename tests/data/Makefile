all: build svn

TARGETS := generate-svn/merge/success/1

$(TARGETS): generate-%:
	podman run --rm -v ./$*:/src svn bash generate.sh
	rm -rf ./$*/.svn

.PHONY: svn
svn:
	podman run --rm -it svn bash

build:
	podman build . -t svn
