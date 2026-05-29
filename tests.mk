#!gmake -f
# -*- make -*- vim:syntax=make:sw=2:et:nospell

#< Simple commands to test
#< -----------------------
#<
#<   ```shell
#<   auth --help
#<   auth --version
#<   auth --write  [OPTIONS] FILENAME(S)
#<   auth --check  [OPTIONS] FILENAME(S)
#<   auth --remove [OPTIONS] FILENAME(S)
#<   auth --change-password [OPTIONS]
#<   auth --show-dir [OPTIONS]
#<   auth --stats [OPTIONS]
#<   ```

#<
#< Options to test
#< ---------------
#<
#< | Option               | Aspects to test
#< | ------               | -------
#< | --cache-time=SECONDS | 0,60,121
#< | --change-password    | authorized, too short, too long
#< | --check, -ck         | shortcut
#< | --color WHEN         | color, no-color
#< | --dir DIR, -d DIR    | shortcut
#< | --force, -f          | shortcut
#< | --help, -h           | shortcut
#< | --quiet, -q          | shortcut
#< | --remove, -rm        | authorized, shortcut
#< | --request-password   | bad, burner, reuse burner
#< | --default-root       | once only, conflicts with --root-dir
#< | --root-dir=PATH      | valid dir, not dir, bad path, duplicate rejection
#< | --show-dir           | authorized
#< | --silent, -s         | shortcut
#< | --stats              | authorized
#< | --verbose, -v        | shortcut
#< | --version            | correctness
#< | --write, -wr         | authorized, missing, duplicated

#<
#< Environment to test
#< -------------------
#<
#< | Variable             | Aspects to test
#< | ------               | -------
#< | AUTH_OPTIONS         | various, empty
#< | AUTH_TEST_FALLBACK_PASSWORD | test-only first-run fallback password
#< | AUTH_TEST_FALLBACK_PASSWORD_CONFIRM | confirmation for first-run fallback password
#< | AUTH_TEST_CURRENT_PASSWORD_OR_BURNER | later fallback/burner authorization
#< | NO_COLOR             | defined
#< | NOCOLOR              | defined
#< | PAGER                | less, more, cat, empty
#<

# These targets intentionally exercise the installed/current auth binary rather
# than Cargo integration tests. Override AUTH to test a different binary:
#
#   make test-all AUTH=/path/to/auth
#
AUTH       ?= ${GIT_WORK_DIR}/target/debug/auth
GOLD_DIR   = ${GIT_WORK_DIR}/golden
TEST_DIR   = /tmp/auth-file-manual-tests
AUTH_DIR   = ${TEST_DIR}/auth-test
ROOT_DIR   = ${TEST_DIR}/root
COPY_ROOT  = ${TEST_DIR}/root-copy
ART_DIR    = ${ARTIFACTS}/manual-tests
RESULTS    = ${ARTIFACTS}/test-results.txt
TEST_PASS  = Long-Test-Password-2026!
BAD_PASS   = Wrong-Test-Password-2026!
FILES      = file1 file2 file3 file4 file5

AUTH_ENV = AUTH_OPTIONS="-d ${AUTH_DIR}" \
           AUTH_TEST_FALLBACK_PASSWORD="${TEST_PASS}" \
           AUTH_TEST_FALLBACK_PASSWORD_CONFIRM="${TEST_PASS}" \
           AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="${TEST_PASS}"

ROOT_AUTH_ENV = AUTH_OPTIONS="-d ${AUTH_DIR} --root-dir=${ROOT_DIR}" \
                AUTH_TEST_FALLBACK_PASSWORD="${TEST_PASS}" \
                AUTH_TEST_FALLBACK_PASSWORD_CONFIRM="${TEST_PASS}" \
                AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="${TEST_PASS}"

Test=printf "${BLU}${RULER}\nTest:${CYN} $1${OFF}\n"; printf "Running $@\n"               >>"${RESULTS}"
Passed=printf "${GRN}Test passed:${CYN} $@${OFF}\n"; printf "Passed $@\n"                 >>"${RESULTS}"
ExpectFailed=printf "${RED}Error:${OFF} $@\n" >&2; printf "Passed $@ - $1\n"              >>"${RESULTS}"
ExpectPassed=printf "${GRN}Success: expected failure - ${OFF} $1\n"; printf "Failed $@\n" >>"${RESULTS}"
Gold_test=$(if $(wildcard ${GOLD_DIR}/$1),cmp $1 $2,@printf "${YLW}Missing golden file: ${OFF}$1\n")

#.______________________________________________________________________________
#| * golden - create golden files [use only when you are certain]
golden:
	@$(call Test,Create golden files)
	@$(call Prompt)
	mkdir -p ${GOLD_DIR}
	@$(call Prompt)
	"${AUTH}" --version > ${GOLD_DIR}/version.txt
	@$(call Prompt)
	PAGER=cat "${AUTH}" --help >"${GOLD_DIR}/help.txt"

#.______________________________________________________________________________
#| * test-all - run all manual CLI tests
test-all: test-clear test-setup test-version test-help test-write-check test-remove \
          test-missing test-cache test-cache-reject test-request-password \
          test-bad-password test-show-dir test-stats test-root-dir test-root-directives test-color \
          test-auth-options test-summary

#.______________________________________________________________________________
#| * test-clear - remove manual test directories and artifacts
test-clear:
	@$(call Test,Remove database and all manual test files)
	@$(call Prompt)
	rm -fr "${TEST_DIR}" # remove database
	@$(call Prompt)
	rm -fr "${ART_DIR}" # remove manual artifacts
	@$(call Prompt)
	mkdir -p "${ART_DIR}" && date >"${RESULTS}"

#.______________________________________________________________________________
#| * test-setup - build auth and create deterministic test files
test-setup:
	@$(call Test,Set up)
	@$(call Prompt)
	cargo build
	@$(call Prompt)
	mkdir -p "${AUTH_DIR}" "${ROOT_DIR}" "${COPY_ROOT}"
	@$(call Prompt)
	for f in ${FILES}; do \
	  printf "manual test content for $$f\n" >"${TEST_DIR}/$$f"; \
	  printf "rooted manual test content for $$f\n" >"${ROOT_DIR}/rel-$$f"; \
	done
	@$(call Prompt)
	cp -R "${ROOT_DIR}/." "${COPY_ROOT}/"

#.______________________________________________________________________________
#| * test-version - test --version
test-version:
	@$(call Test,Version)
	@$(call Prompt)
	"${AUTH}" --version | tee ${ART_DIR}/version.txt
	$(call Gold_test,"version.txt","${ART_DIR}/version.txt")
	@$(call Passed)

#.______________________________________________________________________________
#| * test-help - test --help and -h
test-help:
	@$(call Test,Help)
	@$(call Prompt)
	PAGER=cat "${AUTH}" --help >"${ART_DIR}/help-long.txt"
	@$(call Prompt)
	PAGER=cat "${AUTH}" -h >"${ART_DIR}/help-short.txt"
	@$(call Prompt)
	cmp "${ART_DIR}/help-long.txt" "${ART_DIR}/help-short.txt"
	$(call Gold_test,"help.txt","${ART_DIR}/help-long.txt")
	@$(call Passed)

#.______________________________________________________________________________
#| * test-write-check - write and check several files
test-write-check:
	@$(call Test,Write and check several files)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --write "${TEST_DIR}/file1" "${TEST_DIR}/file2"
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --check "${TEST_DIR}/file1" "${TEST_DIR}/file2"
	@$(call Prompt)
	cd "${TEST_DIR}" && ${AUTH_ENV} "${AUTH}" -ck file1 file2
	@$(call Passed)

#.______________________________________________________________________________
#| * test-remove - remove one authorization and confirm it fails check
test-remove:
	@$(call Test,Remove one authorized file)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --write "${TEST_DIR}/file3"
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --remove "${TEST_DIR}/file3"
	@$(call Prompt)
	if ${AUTH_ENV} "${AUTH}" --check "${TEST_DIR}/file3"; then \
	  $(call ExpectPassed,Expected removed file check to fail); \
	else \
	  $(call ExpectFailed,removed file no longer checks); \
	fi

#.______________________________________________________________________________
#| * test-missing - check unauthorized and nonexistent files
test-missing:
	@$(call Test,Unauthorized and nonexistent files)
	@$(call Prompt)
	if ${AUTH_ENV} "${AUTH}" --check "${TEST_DIR}/file4"; then \
	  $(call ExpectPassed,Expected unauthorized file check to fail); \
	else \
	  $(call ExpectFailed,unauthorized file rejected); \
	fi
	@$(call Prompt)
	if ${AUTH_ENV} "${AUTH}" --check "${TEST_DIR}/does-not-exist"; then \
	  $(call ExpectPassed,Expected missing file check to fail); \
	else \
	  $(call ExpectFailed,nonexistent file rejected); \
	fi

#.______________________________________________________________________________
#| * test-cache - verify --cache-time=60 avoids repeated authorization prompts
test-cache:
	@$(call Test,Authorization cache)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --cache-time=60 --write "${TEST_DIR}/file4"
	@$(call Prompt)
	AUTH_OPTIONS="-d ${AUTH_DIR}" AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="${BAD_PASS}" \
	  "${AUTH}" --request-password --cache-time=60 --write "${TEST_DIR}/file5"
	@$(call Passed)

#.______________________________________________________________________________
#| * test-cache-reject - verify --cache-time rejects values above 120
test-cache-reject:
	@$(call Test,Reject --cache-time=121)
	@$(call Prompt)
	if ${AUTH_ENV} "${AUTH}" --request-password --cache-time=121 --write "${TEST_DIR}/file1"; then \
	  $(call ExpectPassed,Expected --cache-time=121 to fail); \
	else \
	  $(call ExpectFailed,--cache-time=121 rejected); \
	fi

#.______________________________________________________________________________
#| * test-request-password - force password route explicitly
test-request-password:
	@$(call Test,Request password route)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --write "${TEST_DIR}/file1"
	@$(call Passed)

#.______________________________________________________________________________
#| * test-bad-password - wrong password should fail when no cache is present
test-bad-password:
	@$(call Test,Bad auth password)
	@$(call Prompt)
	if env AUTH_OPTIONS="-d ${AUTH_DIR}" \
	    AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="${BAD_PASS}" \
	    "${AUTH}" --request-password --cache-time=0 --write "${TEST_DIR}/file1"; then \
	  $(call ExpectFailed,Expected bad password to fail); \
	else \
	  $(call ExpectPassed, bad auth password rejected); \
	fi

#.______________________________________________________________________________
#| * test-show-dir - authorized --show-dir
test-show-dir:
	@$(call Test,Show directory)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --show-dir >"${ART_DIR}/show-dir.txt"
	@$(call Prompt)
	cat "${ART_DIR}/show-dir.txt"
	@$(call Passed)

#.______________________________________________________________________________
#| * test-stats - authorized --stats
test-stats:
	@$(call Test,Stats)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --stats >"${ART_DIR}/stats.txt"
	@$(call Prompt)
	cat "${ART_DIR}/stats.txt"
	@$(call Passed)

#.______________________________________________________________________________
#| * test-root-dir - root-relative identity works across copied roots
test-root-dir:
	@$(call Test,Root-relative portable identity)
	@$(call Prompt)
	${ROOT_AUTH_ENV} "${AUTH}" --request-password --write "${ROOT_DIR}/rel-file1"
	@$(call Prompt)
	AUTH_OPTIONS="-d ${AUTH_DIR} --root-dir=${COPY_ROOT}" \
	AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="${TEST_PASS}" \
	  "${AUTH}" --request-password --check "${COPY_ROOT}/rel-file1"
	@$(call Passed)

#.______________________________________________________________________________
#| * test-color - color modes and NO_COLOR/NOCOLOR
test-color:
	@$(call Test,Color options)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --color always --check "${TEST_DIR}/file1" 2>"${ART_DIR}/color-always.err" || true
	@$(call Prompt)
	NO_COLOR=1 ${AUTH_ENV} "${AUTH}" --color auto --check "${TEST_DIR}/file1" 2>"${ART_DIR}/color-nocolor.err" || true
	@$(call Prompt)
	NOCOLOR=1 ${AUTH_ENV} "${AUTH}" --color auto --check "${TEST_DIR}/file1" 2>"${ART_DIR}/color-nocolor-legacy.err" || true
	@$(call Passed)

#.______________________________________________________________________________
#| * test-auth-options - AUTH_OPTIONS supplies directory and root options
test-auth-options:
	@$(call Test,AUTH_OPTIONS)
	@$(call Prompt)
	AUTH_OPTIONS="-d ${AUTH_DIR} --root-dir=${ROOT_DIR}" \
	AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="${TEST_PASS}" \
	  "${AUTH}" --request-password --check "${ROOT_DIR}/rel-file1"
	@$(call Passed)

#.______________________________________________________________________________
#| * test-summary - summarize manual test artifacts
test-summary:
	@$(call Test,Manual test artifacts)
	@$(call Prompt)
	find "${ART_DIR}" -maxdepth 1 -type f -print | sort
	@printf "${BLU}${RULER}\nTest${CYN} summary${OFF}\n"; \
         printf "${RED}%d failures${OFF}\n"  $$(grep -c '^Failed'  "${RESULTS}"); \
         printf "${GRN}%d passed${OFF}\n"    $$(grep -c '^Passed'  "${RESULTS}"); \
         printf "${CYN}%d tests ran${OFF}\n" $$(grep -c '^Running' "${RESULTS}");

#.______________________________________________________________________________
#| * test-root-directives - root directive hardening smoke tests
test-root-directives:
	@$(call Test,Root directive hardening)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --default-root --check "${TEST_DIR}/file1" || true
	@$(call Prompt)
	if ${AUTH_ENV} "${AUTH}" --default-root --root-dir=${ROOT_DIR} --check "${TEST_DIR}/file1"; then \
	  $(call ExpectPassed,Expected duplicate root directives to fail); \
	else \
	  $(call ExpectFailed,duplicate root directives rejected); \
	fi
	@$(call Prompt)
	if AUTH_OPTIONS="-d ${AUTH_DIR} --default-root" "${AUTH}" --root-dir=${ROOT_DIR} --check "${TEST_DIR}/file1"; then \
	  $(call ExpectPassed,Expected AUTH_OPTIONS plus CLI root directive to fail); \
	else \
	  $(call ExpectFailed,AUTH_OPTIONS plus CLI root directive rejected); \
	fi

# This line remains to indicate the last line of this file
