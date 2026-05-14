#!make -f
# -*- make -*- vim:syntax=make:sw=2:et:nospell
#|------------------------------------------------------------------------------
#|
#| # Description
#|
#| Conveniences to run some simple stuff.
#|
#| # Targets
#|

# Dependencies
SHELL := bash
GIT_EXE  = $(shell command -v git)
GREP_EXE = $(firstword $(shell command -v ggrep) $(shell command -v grep))
THIS_MAKEFILE := $(realpath $(lastword $(MAKEFILE_LIST)))
PHONIES := $(shell perl -lane 'print $$1 if m{^([a-zA-Z][-a-zA-Z0-9_]*):[^=]*$$};' ${THIS_MAKEFILE})

.PHONY: $(PHONIES)

.DEFAULT_GOAL := help

GIT_WORK_PATH := $(shell ${GIT_EXE} rev-parse --show-toplevel)
SBOM_PATH := ${GIT_WORK_PATH}/sbom/sbom.json

#.______________________________________________________________________________
#| * help - display documentation
help: # default target
	@if command -v glow 1>/dev/null; then\
	   ${GREP_EXE} '^#|' ${THIS_MAKEFILE} | cut -c 3- | glow -p;\
	 else\
	   ${GREP_EXE} '^#|' ${THIS_MAKEFILE} | cut -c 3-;\
	 fi

#.______________________________________________________________________________
#| * unpack - extract version from archives
unpack:
ifeq ("${VERS}","")
	@echo ""$(error Must specify VERS=)
endif
	check-version ${VERS}
	unpack "$(firstword ${VERS})"

#.______________________________________________________________________________
#| * upload - test, commit, tag and push
upload:
ifeq ("${VERS}","")
	@echo ""$(error Must specify VERS=)
endif
	check-version ${VERS}
	cargo fmt --all
	cargo check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all-targets --all-features
	check-version ${VERS}
ifeq ($(words ${VERS}),1)
	git commit -a
	git tag -a -s "v${VERS}"
else
	git commit -a -m "${VERS}"
	git tag -a -s -m "${VERS}" "v$(firstword ${VERS})"
endif
	git push

#.______________________________________________________________________________
#| * sbom - create a software bill of materials
sbom:
	cargo cyclonedx --format json > ${SBOM_PATH}

#|
#|------------------------------------------------------------------------------
# The end
