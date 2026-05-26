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

GIT_WORK_DIR   := $(shell ${GIT_EXE} rev-parse --show-toplevel)
DEV_TOOLS      := ${GIT_WORK_DIR}/dev-tools
ARTIFACTS      := ${GIT_WORK_DIR}/artifacts
SBOM_DIR       := ${ARTIFACTS}/sbom
SBOM_FILE      := auth-file.dx
SBOM_FULLPATH  := ${SBOM_DIR}/${SBOM_FILE}.json
AUDIT_DIR      := ${ARTIFACTS}/audit
AUDIT_FULLPATH := ${AUDIT_DIR}/audit.txt
AUTH_DEBUG     := ${GIT_WORK_DIR}/target/debug/auth
AUTH_RELEASE   := ${GIT_WORK_DIR}/target/release/auth

ifdef VERS
  ifeq ($(words ${VERS}),0)
    override undefine VERS
  else
    ifeq ($(words ${VERS}),1)
    override VERS:=$(firstword ${VERS})
    else
    override VERS:=$(firstword ${VERS}) '$(wordlist 2,$(words ${VERS}),${TEXT})'
    endif
  endif
endif

# Special macros to add some color
ifdef NOCOLOR
  Warning=echo "$1"
  Finished=echo "$1"
else
  Warning=printf "[1;93mWarning:[0m $1\n"
  Finished=printf "[1;96mFinished:[0m $1\n"
endif

#.______________________________________________________________________________
#| * help - display documentation
#      Lines beginning with #<, #|, or #> are used to extract documentation.
#      #< represents the beginning of documentation
#      #| represents target documentation
#      #> represents the end of documentation
#      A special call to the Test macro also goes to targets if in the TESTS makefile if it exists.
TESTS = tests.mk # can be overridden
help: # default target
	@python3 ${DEV_TOOLS}/make-help.py ${THIS_MAKEFILE} ${TESTS}

#.______________________________________________________________________________
#| * version - check the version
version:
	python3 ${DEV_TOOLS}/check-version.py --show

#.______________________________________________________________________________
#| * unpack VERS=#.#.# - extract version from archives
unpack:
	if [[ -n '${VERS}' ]]; then \
	          python3 ${DEV_TOOLS}/check-version.py ${VERS}; \
	          python3 ${DEV_TOOLS}/unpack.py "$(python3 ${DEV_TOOLS}/check-version.py ${VERS})"; \
	        else \
	          python3 ${DEV_TOOLS}/unpack.py "$(python3 ${DEV_TOOLS}/check-version.py)" \
	        fi

#.______________________________________________________________________________
#| * fmt - cargo format 
fmt:
	cargo fmt --all
	@$(call Finished,Formatted)

#.______________________________________________________________________________
#| * check - basic syntax and rust compiler requirements
check:
	cargo check
	@$(call Finished,Check passed)

#.______________________________________________________________________________
#| * clippy - deep static analysis
clippy:
	cargo clippy --all-targets --all-features -- -D warnings
	@$(call Finished,Clippy passed)

#.______________________________________________________________________________
#| * test - basic cargo tests
test:
	cargo test --all-targets --all-features
	@$(call Finished,Test complete)

#.______________________________________________________________________________
#| * verify - run all tests
verify: fmt check clippy test
	@echo "Verification complete" > "${ARTIFACTS}/verified.txt"; \
	date                     >> "${ARTIFACTS}/verified.txt"; \
	uname -a                 >> "${ARTIFACTS}/verified.txt"; \
	python3 ${DEV_TOOLS}/check-version.py --show >> "${ARTIFACTS}/verified.txt"
	@$(call Finished,Verification complete)

#.______________________________________________________________________________
#| * upload VERS='#.#.# Reason' - commit to GitHub (aka push)
upload: verify
	python3 ${DEV_TOOLS}/check-version.py ${VERS}
	set -- ${VERS}; \
        if [[ $$# == 1 ]]; then \
          git commit -a; \
          git tag -a -s "v$$1"; \
        else \
          git commit -a -m "$$1"; \
          git tag -a -s -m "$$1" "v$(firstword $$1)"; \
        fi
	git push

push: upload

#.______________________________________________________________________________
#| * dev - compile a development version
${AUTH_DEBUG}: dev
dev:
	cargo build --all-targets --all-features

#.______________________________________________________________________________
#| * release - compile a release version
${AUTH_RELEASE}: release
release:
	cargo build --release --all-targets --all-features

#.______________________________________________________________________________
#| * ci - run continuous integration tests
ci:
	@echo "TODO: Not yet implemented"
#.______________________________________________________________________________
#| * audit - run a security audit
audit:
	mkdir -p '${AUDIT_DIR}'
	cargo audit 2>&1 | tee ${AUDIT_FULLPATH}
	@$(call Finished,Created ${AUDIT_FULLPATH})

#.______________________________________________________________________________
#| * sbom - create a software bill of materials
sbom:
	@$(call Warning,TODO: sbom is not yet complete)
	cargo cyclonedx --verbose \
                        --target all \
                        --all \
                        --format json \
                        --override-filename ${SBOM_FILE}
	mkdir -p '${SBOM_DIR}'
	mv '${SBOM_FILE}.json' '${SBOM_DIR}'/
	@$(call Finished,Created ${SBOM_FULLPATH})

#.______________________________________________________________________________
#| * install - copy release to ${HOME}/bin
install: ${AUTH_RELEASE}
	mkdir -p "${HOME}/bin"
	rsync -av "${AUTH_RELEASE}" "${HOME}/bin/"
	@$(call Installed,auth is now installed as ${AUTH_RELEASE})

ifneq ("$(wildcard ${TESTS})","")
  include ${TESTS}
endif

#>
#>------------------------------------------------------------------------------
# The end
