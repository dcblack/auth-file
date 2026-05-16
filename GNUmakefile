#!make -f
# -*- make -*- vim:syntax=make:sw=2:et:nospell
# Note: See help target to understand special comment style
#<------------------------------------------------------------------------------
#<
#< Description
#< ===========
#<
#< Conveniences to run some simple stuff.
#<
#| Make targets
#| ============
#|
#| | Target                   | Purpose
#| | ------                   | -------

# Dependencies
SHELL = /bin/bash
GIT_EXE  = $(shell command -v git)
THIS_MAKEFILE := $(realpath $(lastword $(MAKEFILE_LIST)))
PHONIES := $(shell perl -lane 'print $$1 if m{^([a-zA-Z][-a-zA-Z0-9_]*):[^=]*$$};' ${THIS_MAKEFILE})

.PHONY: $(PHONIES)

.DEFAULT_GOAL := help

GIT_WORK_PATH := $(shell ${GIT_EXE} rev-parse --show-toplevel)
SBOM_PATH := ${GIT_WORK_PATH}/sbom/sbom.json

#.______________________________________________________________________________
#| * help - display documentation
#      Lines beginning with #<, #|, or #> are used to extract documentation.
#      #< represents the beginning of documentation
#      #| represents target documentation
#      #> represents the end of documentation
#      A special call to the Test macro also goes to targets if in the TESTS makefile if it exists.
TESTS = tests.mk # can be overridden
help: # default target
	@bin/make-help.bash ${THIS_MAKEFILE} ${TESTS}

#.______________________________________________________________________________
#| * version - check the version
version:
	check-version --show

#.______________________________________________________________________________
#| * unpack - extract version from archives
unpack:
	if [[ -n "${VERS}" ]]; then \
          check-version "${VERS}"; \
          unpack "$(check-version ${VERS})"; \
        else \
          unpack "$(check-version)" \
        fi

#.______________________________________________________________________________
#| * validate - run stringent checks (Clippy) and all tests
validate:
	cargo fmt --all
	cargo check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all-targets --all-features
	date > validated.txt
	uname -a >> validated.txt

#.______________________________________________________________________________
#| * upload - validate
upload: validate
	check-version ${VERS}
	set -- ${VERS}; \
    if [[ $$# == 1 ]]; then \
      git commit -a; \
      git tag -a -s "v${VERS}"; \
    else \
      git commit -a -m "${VERS}"; \
      git tag -a -s -m "${VERS}" "v$(firstword ${VERS})"; \
    fi
	git push

#.______________________________________________________________________________
#| * sbom - create a software bill of materials
sbom:
	cargo cyclonedx --format json > ${SBOM_PATH}

ifneq ("$(wildcard ${TESTS})","")
  include ${TESTS}
endif

#>
#>------------------------------------------------------------------------------
# The end
