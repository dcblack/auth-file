#!gmake -f

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
#< | --root-dir=PATH      | valid dir, not dir, bad path
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
#< | NO_COLOR             | defined
#< | NOCOLOR              | defined
#< | PAGER                | glow, less, empty
#<

TEST_DIR := /tmp/auth
AUTH_DIR := ${TEST_DIR}/dbdir
ROOT_DIR := ${TEST_DIR}/root
FILES    := file1 file2 file3 file4 file5

RULER := ------------------------------------------------------------
Prompt=printf "[1;92m%% [0m"
Test=printf "[1;94m${RULER}\nTest:[2;96m $1[0m\n"

#.______________________________________________________________________________
#| * test-all - run all the tests
test-all: test-clear test-setup test1 test2 test3

test-setup:
	@$(call Test,Set up)
	@$(call Prompt)
	mkdir -p "${ROOT_DIR}"
	@$(call Prompt)
	for f in ${FILES}; do\
	  rand >${TEST_DIR}/$$f;\
	  rand >${ROOT_DIR}/rel-$$f;\
	done
	@$(call Prompt)
	cd ${TEST_DIR}; echo "hello" >file0; auth --write file0

test-clear:
	@$(call Test,Remove database and all files)
	@$(call Prompt)
	rm -fr "${ROOT_DIR}"
test1:
	@$(call Test,Version)
	@$(call Prompt)
	auth --version

test2:
	@$(call Test,Help)
	@$(call Prompt)
	auth --help

test3:
	@$(call Test,Write and check several files)
	@$(call Prompt)
	auth --write ${TEST_DIR}/file1
	@$(call Prompt)
	cd ${TEST_DIR} && auth --write file2
	# Make sure location doesn't matter
	@$(call Prompt)
	auth --check ${TEST_DIR}/file2
	@$(call Prompt)
	cd ${TEST_DIR} && auth --check file1

test4:
	@$(call Attempt to reauthorize file1)
	cd ${TEST_DIR} && auth --check file1
	@$(call Prompt)
	auth --write ${TEST_DIR}/file1

#TAF!
