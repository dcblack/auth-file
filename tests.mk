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
#   make tests-all AUTH=/path/to/auth

# NOTE: Individual test targets must be named test-<NAME>.
#       Targets beginning with tests- (plural) are for setup and teardown.
#       This naming convention allows automatic collection for the TEST_LIST.

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
Tests=printf "${BLU}${RULER}\nGeneral:${CYN} $1${OFF}\n"; printf "Running $@\n"               >>"${RESULTS}"
Passed=printf "${GRN}Test passed:${CYN} $@${OFF}\n"; printf "Passed $@\n"                 >>"${RESULTS}"
FailedExpectation=printf "${RED}Error:${OFF} $@\n" >&2; printf "Failed $@ - $1\n"              >>"${RESULTS}"
PassedExpectation=printf "${GRN}Success: expected $@ to fail - ${OFF} $1\n"; printf "Passed $@\n" >>"${RESULTS}"
Gold_test=$(if $(wildcard ${GOLD_DIR}/$1),cmp $1 $2,@printf "${YLW}Missing golden file: ${OFF}$1\n")

#.______________________________________________________________________________
#| * golden - create golden files [use only when you are certain]
golden:
	@$(call Info,Make $@)
	@$(call Prompt)
	mkdir -p ${GOLD_DIR}
	@$(call Prompt)
	"${AUTH}" --version > ${GOLD_DIR}/version.txt
	@$(call Prompt)
	PAGER=cat "${AUTH}" --help >"${GOLD_DIR}/help.txt"

#.______________________________________________________________________________
#| * tests-all - run all manual CLI tests
tests-all: tests-clear tests-setup ${TEST_LIST} tests-summary

#.______________________________________________________________________________
#| * tests-clear - remove manual test directories and artifacts
tests-clear:
	@$(call Tests,Remove database and all manual test files)
	@$(call Prompt)
	rm -fr "${TEST_DIR}" # remove database
	@$(call Prompt)
	rm -fr "${ART_DIR}" # remove manual artifacts
	@$(call Prompt)
	mkdir -p "${ART_DIR}" && date >"${RESULTS}"

#.______________________________________________________________________________
#| * tests-setup - build auth and create deterministic test files
tests-setup:
	@$(call Test,$@)
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
	@$(call Test,$@)
	@$(call Prompt)
	"${AUTH}" --version | tee ${ART_DIR}/version.txt
	$(call Gold_test,"version.txt","${ART_DIR}/version.txt")
	@$(call Passed)

#.______________________________________________________________________________
#| * test-help - test --help and -h
test-help:
	@$(call Test,$@)
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
	@$(call Test,$@)
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
	@$(call Test,$@)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --write "${TEST_DIR}/file3"
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --remove "${TEST_DIR}/file3"
	@$(call Prompt)
	if ${AUTH_ENV} "${AUTH}" --check "${TEST_DIR}/file3"; then \
	  $(call FailedExpectation,Expected removed file check to fail); \
	else \
	  $(call PassedExpectation,removed file no longer checks); \
	fi

#.______________________________________________________________________________
#| * test-missing - check unauthorized and nonexistent files
test-missing:
	@$(call Test,$@)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --remove "${TEST_DIR}/file4"
	if ${AUTH_ENV} "${AUTH}" --check "${TEST_DIR}/file4"; then \
	  $(call FailedExpectation,Expected unauthorized file check to fail); \
	else \
	  $(call PassedExpectation,unauthorized file rejected); \
	fi
	@$(call Prompt)
	if ${AUTH_ENV} "${AUTH}" --check "${TEST_DIR}/does-not-exist"; then \
	  $(call FailedExpectation,Expected missing file check to fail); \
	else \
	  $(call PassedExpectation,nonexistent file rejected); \
	fi


#.______________________________________________________________________________
#| * test-check-no-auth - --check must not request authorization or auth password
test-check-no-auth:
	@$(call Test,$@)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --write "${TEST_DIR}/file1"
	@$(call Prompt)
	env AUTH_OPTIONS="-d ${AUTH_DIR}" \
	    "${AUTH}" --request-password --cache-time=60 --check "${TEST_DIR}/file1"
	@$(call Passed)

#.______________________________________________________________________________
#| * test-cache - verify --cache-time=60 avoids repeated authorization prompts
test-cache:
	@$(call Test,$@)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --cache-time=60 --write "${TEST_DIR}/file4"
	@$(call Prompt)
	AUTH_OPTIONS="-d ${AUTH_DIR}" AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="${BAD_PASS}" \
	  "${AUTH}" --request-password --write "${TEST_DIR}/file5"
	@$(call Passed)

#.______________________________________________________________________________
#| * test-cache-reject - verify --cache-time rejects values above 120
test-cache-reject:
	@$(call Test,$@)
	@$(call Prompt)
	if ${AUTH_ENV} "${AUTH}" --request-password --cache-time=121 --write "${TEST_DIR}/file1"; then \
	  $(call FailedExpectation,Expected --cache-time=121 to fail); \
	else \
	  $(call PassedExpectation,--cache-time=121 rejected); \
	fi

#.______________________________________________________________________________
#| * test-request-password - force password route explicitly
test-request-password:
	@$(call Test,$@)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --write "${TEST_DIR}/file1"
	@$(call Passed)

#.______________________________________________________________________________
#| * test-bad-password - wrong password should fail when no cache is present
test-bad-password:
	@$(call Test,$@)
	@$(call Prompt)
	rm -fr "${TEST_DIR}/bad-password"
	@$(call Prompt)
	mkdir -p "${TEST_DIR}/bad-password/auth-test"
	@$(call Prompt)
	printf "bad password test\n" >"${TEST_DIR}/bad-password/file1"
	@$(call Prompt)
	env AUTH_OPTIONS="-d ${TEST_DIR}/bad-password/auth-test" \
	    AUTH_TEST_FALLBACK_PASSWORD="${TEST_PASS}" \
	    AUTH_TEST_FALLBACK_PASSWORD_CONFIRM="${TEST_PASS}" \
	    AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="${TEST_PASS}" \
	    "${AUTH}" --request-password --cache-time=0 --write "${TEST_DIR}/bad-password/file1"
	@$(call Prompt)
	if env AUTH_OPTIONS="-d ${TEST_DIR}/bad-password/auth-test" \
	    AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="${BAD_PASS}" \
	    "${AUTH}" --request-password --cache-time=0 --write "${TEST_DIR}/bad-password/file1"; then \
	  $(call FailedExpectation,Bad auth password returned OK!); \
	  exit 1; \
	else \
	  $(call PassedExpectation,Bad auth password rejected); \
	fi

#.______________________________________________________________________________
#| * test-show-dir - authorized --show-dir
test-show-dir:
	@$(call Test,$@)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --show-dir >"${ART_DIR}/show-dir.txt"
	@$(call Prompt)
	cat "${ART_DIR}/show-dir.txt"
	@$(call Passed)

#.______________________________________________________________________________
#| * test-stats - authorized --stats
test-stats:
	@$(call Test,$@)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --stats >"${ART_DIR}/stats.txt"
	@$(call Prompt)
	cat "${ART_DIR}/stats.txt"
	@$(call Passed)

#.______________________________________________________________________________
#| * test-root-dir - root-relative identity works across copied roots
test-root-dir:
	@$(call Test,$@)
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
	@$(call Test,$@)
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
	@$(call Test,$@)
	@$(call Prompt)
	rm -fr "${TEST_DIR}/auth-options"
	@$(call Prompt)
	mkdir -p "${TEST_DIR}/auth-options/auth-test" "${TEST_DIR}/auth-options/root"
	@$(call Prompt)
	printf "auth options rooted content\n" >"${TEST_DIR}/auth-options/root/rel-file1"
	@$(call Prompt)
	env AUTH_OPTIONS="-d ${TEST_DIR}/auth-options/auth-test --root-dir=${TEST_DIR}/auth-options/root" \
	    AUTH_TEST_FALLBACK_PASSWORD="${TEST_PASS}" \
	    AUTH_TEST_FALLBACK_PASSWORD_CONFIRM="${TEST_PASS}" \
	    AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="${TEST_PASS}" \
	    "${AUTH}" --request-password --write "${TEST_DIR}/auth-options/root/rel-file1"
	@$(call Prompt)
	env AUTH_OPTIONS="-d ${TEST_DIR}/auth-options/auth-test --root-dir=${TEST_DIR}/auth-options/root" \
	    "${AUTH}" --check "${TEST_DIR}/auth-options/root/rel-file1"
	@$(call Passed)


#.______________________________________________________________________________
#| * test-setup-profile-safety - changed setup.profile blocks source-style workflow
test-setup-profile-safety:
	@$(call Test,Changed setup.profile is rejected)
	@$(call Prompt)
	printf "export AUTH_PROFILE_OK=1\n" >"${TEST_DIR}/setup.profile"
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --request-password --write "${TEST_DIR}/setup.profile"
	@$(call Prompt)
	printf "export AUTH_PROFILE_OK=0\n" >"${TEST_DIR}/setup.profile"
	@$(call Prompt)
	if ${AUTH_ENV} "${AUTH}" --check "${TEST_DIR}/setup.profile"; then \
	  $(call FailedExpectation,Expected modified setup.profile to fail); \
	else \
	  $(call PassedExpectation,modified setup.profile rejected); \
	fi

#.______________________________________________________________________________
#| * test-root-directives - root directive hardening smoke tests
test-root-directives:
	@$(call Test,$@)
	@$(call Prompt)
	${AUTH_ENV} "${AUTH}" --default-root --check "${TEST_DIR}/file1" || true
	@$(call Prompt)
	if ${AUTH_ENV} "${AUTH}" --default-root --root-dir=${ROOT_DIR} --check "${TEST_DIR}/file1"; then \
	  $(call FailedExpectation,Expected duplicate root directives to fail); \
	else \
	  $(call PassedExpectation,duplicate root directives rejected); \
	fi
	@$(call Prompt)
	if AUTH_OPTIONS="-d ${AUTH_DIR} --default-root" "${AUTH}" --root-dir=${ROOT_DIR} --check "${TEST_DIR}/file1"; then \
	  $(call FailedExpectation,Expected AUTH_OPTIONS plus CLI root directive to fail); \
	else \
	  $(call PassedExpectation,AUTH_OPTIONS plus CLI root directive rejected); \
	fi

#.______________________________________________________________________________
#| * tests-summary - summarize manual test artifacts
tests-summary:
	@$(call Tests,Manual test artifacts)
	@$(call Prompt)
	find "${ART_DIR}" -maxdepth 1 -type f -print | sort
	@printf "${BLU}${RULER}\nTest${CYN} summary${OFF}\n"; \
         printf "${RED}%d failures${OFF}\n"  $$(${GREP_EXE} -c '^Failed'  "${RESULTS}"); \
         printf "${GRN}%d passed${OFF}\n"    $$(${GREP_EXE} -c '^Passed'  "${RESULTS}"); \
         printf "${CYN}%d tests ran${OFF}\n" $$(${GREP_EXE} -c '^Running' "${RESULTS}");
	${GREP_EXE} ^Failed "${RESULTS}"

# This line remains to indicate the last line of this file
