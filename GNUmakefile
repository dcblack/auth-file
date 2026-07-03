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
#< There are two input variables that may be used to affect this:
#<
#< Name  | Description
#< ----  | -----------
#< TESTS | contains a list of makefiles containing test targets. Defaults to test.mk
#< VERS  | specifies a version and comments for use in tagging the git repository. This is required.
#< SEED  | controls randomization of test list
#<
#< Note - Individual test targets must be named test-NAME.
#<        Targets beginning with tests-NAME (plural) set up and summarize.
#<        This naming convention allows automatic collection for the TEST_LIST.

#| Make targets
#| ============
#|
#| | Target                   | Purpose
#| | ------                   | -------

# Dependencies
SHELL = $(firstword $(wildcard /bin/bash /usr/bin/bash))
GIT_EXE  = $(shell command -v git)
GREP_EXE  = $(firstword $(shell command -v ggrep) $(shell command -v grep))
DIFF_EXE  = $(firstword $(wildcard /usr/bin/diff /bin/diff))
TOP_MAKEFILE := $(realpath $(lastword $(MAKEFILE_LIST)))
TESTS = tests.mk # can be overridden
SEED  = 0
PHONIES := $(sort $(shell perl -lane 'print $$1 if m{^([a-zA-Z][-a-zA-Z0-9_]*):[^=]*$$};' ${TOP_MAKEFILE} ${TESTS}))

.PHONY: $(PHONIES)

.DEFAULT_GOAL := help

GIT_WORK_DIR   := $(shell ${GIT_EXE} rev-parse --show-toplevel)
DEV_TOOLS      := ${GIT_WORK_DIR}/dev-tools
ARTIFACTS      := ${GIT_WORK_DIR}/artifacts
ARCHIVES       := ${GIT_WORK_DIR}/ARCHIVE
ARCHIVE_NOW    := ${ARCHIVES}/auth-file-$(shell git describe --dirty --long).zip
SBOM_DIR       := ${GIT_WORK_DIR}/sbom
SBOM_FILE      := auth-file.dx
SBOM_FULLPATH  := ${SBOM_DIR}/${SBOM_FILE}.json
AUDIT_DIR      := ${ARTIFACTS}/audit
AUDIT_FULLPATH := ${AUDIT_DIR}/audit.txt
AUTH_DEV       := ${GIT_WORK_DIR}/target/debug/auth
AUTH_RELEASE   := ${GIT_WORK_DIR}/target/release/auth
TEST_LIST := $(shell ${DEV_TOOLS}/shuffle --save=${ARTIFACTS}/seed.txt --seed=${SEED} $(filter test-%,$(shell perl -lane 'print $$1 if m{^([a-zA-Z][-a-zA-Z0-9_]*):[^=]*$$};' ${TESTS})))
TOOLS_VERSIONS := tools-versions.txt
TOOLS_CURRENT  := ${ARTIFACTS}/${TOOLS_VERSIONS}
TOOLS_BLESSED  := ${GIT_WORK_DIR}/${TOOLS_VERSIONS}

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
  RED :=
  GRN :=
  YLW :=
  BLU :=
  MAG :=
  CYN :=
  OFF :=
else
  RED := [1;91m
  GRN := [1;92m
  YLW := [1;93m
  BLU := [1;94m
  MAG := [1;95m
  CYN := [1;96m
  OFF := [0m
endif
RULER   := ------------------------------------------------------------
Prompt   = printf "${CYN}%% ${OFF}"
Info     = printf "${BLU}${RULER}\nInfo:${CYN} $1${OFF}\n";
Error    = printf "${RED}Error:${OFF} $1\n"
Warning  = printf "${YLW}Warning:${OFF} $1\n"
Finished = printf "${CYN}Finished:${OFF} $1\n"
define Vars
  $(foreach var,$(strip $1),printf "${BLU}${var}${OFF} = $(value ${var})\n";)
endef

#.______________________________________________________________________________
#| * help - display documentation
#      Lines beginning with #<, #|, or #> are used to extract documentation.
#      #< represents the beginning of documentation
#      #| represents target documentation
#      #> represents the end of documentation
#      A special call to the Test macro also goes to targets if in the TESTS makefile if it exists.
help: # default target
	@python3 ${DEV_TOOLS}/make-help.py ${TOP_MAKEFILE} ${TESTS}

#.______________________________________________________________________________
#| * vars - display make vars of interest
VARS :=$(sort \
  ARCHIVE_NOW \
  ARCHIVES \
  ARTIFACTS \
  AUTH_DEV \
  AUTH_ENV \
  AUTH_RELEASE \
  DEV_TOOLS \
  GIT_WORK_DIR \
  PHONIES \
  RESULTS \
  ROOT_AUTH_ENV \
  TEST_LIST \
  TESTS \
  TOOLS_CURRENT \
  TOOLS_BLESSED \
  TOP_MAKEFILE \
  VERS \
)

vars:
	@$(call Info,Internal make variables)
	@$(call Vars,${VARS})

#.______________________________________________________________________________
#| * tools-current - dump current tools into artifacts
tools-current:
	@$(call Info,Pulling tool versions)
	@date -u +"Updated: %A %Y-%m-%d %H:%M GMT" >"${TOOLS_CURRENT}"
	@rustup --version 2>&1 | grep ^rustup >>"${TOOLS_CURRENT}"
	@rustc  --version 2>&1 | grep ^rustc  >>"${TOOLS_CURRENT}"
	@cargo  --version 2>&1 | grep ^cargo  >>"${TOOLS_CURRENT}"
#.______________________________________________________________________________
#| * tools-check - compare the current tools versions against blessed
tools-check: tools-current
	@$(call Info,Comparing current against blessed tool versions)
	@if [[ -r "${TOOLS_BLESSED}" ]]; then \
          ${DIFF_EXE} --ignore-matching-lines='^(Updated|Blessed).*' "${TOOLS_BLESSED}" "${TOOLS_CURRENT}" \
          && printf "[1;92mTools are blessed\n[0m" \
          && perl -pe 's/^/| /' "${TOOLS_BLESSED}"; \
        else \
          printf "[1;93mWarning:[93m Not yet tools-blessed[0m\n"; \
	  perl -pe 's/^/| /' "${TOOLS_CURRENT}" ;\
	fi
#.______________________________________________________________________________
#| * tools-blessed - check the tools versions

tools-blessed: tools-current
	@mv "${TOOLS_CURRENT}" "${TOOLS_BLESSED}"
	@perl -pi -e 's/Updated/Blessed/' "${TOOLS_BLESSED}"
	@$(call Info,Current tools are now blessed -- Remember to commit.)

#.______________________________________________________________________________
#| * version - check the version
version:
	python3 "${DEV_TOOLS}/check-version.py" --show

#.______________________________________________________________________________
#| * archive - create an archive to share
archive:
	${GIT_EXE} archive --format=zip HEAD > ${ARCHIVE_NOW}
	@$(call Info,Created ${ARCHIVE_NOW})

#.______________________________________________________________________________
#| * status displays git status
status:
	@git status  -s -uno
	@git describe --long --dirty

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
	@echo "Testing complete" >> "${ARTIFACTS}/verified.txt";

verified:
	@date -u +"Verified: %A %Y-%m-%d %H:%M" > "${ARTIFACTS}/verified.txt"; \
	uname -a >> "${ARTIFACTS}/verified.txt"; \
	python3 ${DEV_TOOLS}/check-version.py --show >> "${ARTIFACTS}/verified.txt"

#.______________________________________________________________________________
#| * syntax - basic checks
syntax: tools-check fmt check clippy
	@echo "Syntax completed" >> "${ARTIFACTS}/verified.txt";

#.______________________________________________________________________________
#| * verify - run all tests
verify: syntax test
	@echo "Verification complete" >> "${ARTIFACTS}/verified.txt";
	@$(call Finished,Verification complete)

#.______________________________________________________________________________
#| * upload VERS='#.#.# Note' - commit to GitHub (aka push)
upload: verify
	python3 ${DEV_TOOLS}/check-version.py ${VERS}
	set -- ${VERS}; \
        if [[ $$# == 1 ]]; then \
          ${GIT_EXE} commit -a; \
          ${GIT_EXE} tag -a -s "v$$1"; \
        else \
          ${GIT_EXE} commit -a -m "$$1"; \
          ${GIT_EXE} tag -a -s -m "$$1" "v$(firstword $$1)"; \
        fi
	${GIT_EXE} push

push: upload

#.______________________________________________________________________________
#| * dev - compile a development version
${AUTH_DEV}: dev
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
