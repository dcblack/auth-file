#!/bin/bash

# Notes:
# - The targ-help and test-help process targets into markdown table rows
# - Everything should line up with the table beginning in column 1 of the output

set -u

THIS_MAKEFILE="$1"
TESTS="$2"
# TODO: Describe the following line. It's a bit convoluted. What is cd --?
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

if command -v ggrep >/dev/null 2>&1; then
  GREP_EXE="$(command -v ggrep)"
else
  GREP_EXE="$(command -v grep)"
fi

# Note that output is to be as follows:
# 1. Lines from THIS_MAKEFILE tagged with #<
# 2. Lines from TESTS tagged with #< if TESTS exists
# 3. Lines from THIS_MAKEFILE tagged with #|
# 4. Lines from TESTS tagged with #| if TESTS exists
# 5. Lines from TESTS tagged with the '$(call Test...)' if TESTS exists
#    ^ accomplished with the test-help.pl script
# 6. Lines from TESTS tagged with #> if TESTS exists
# 7. Lines from THIS_MAKEFILE tagged with #>

"${GREP_EXE}" '^#<' "${THIS_MAKEFILE}" | cut -c 3-
if [[ -r "${TESTS}" ]]; then
  "${GREP_EXE}" '^#<' "${TESTS}" | cut -c 3-
fi
"${SCRIPT_DIR}/targ-help.pl" "${THIS_MAKEFILE}"
if [[ -r "${TESTS}" ]]; then
  "${SCRIPT_DIR}/targ-help.pl" "${TESTS}"
  "${SCRIPT_DIR}/test-help.pl" "${TESTS}"
  "${GREP_EXE}" '^#>' "${TESTS}" | cut -c 3-
fi
"${GREP_EXE}" '^#>' "${THIS_MAKEFILE}" | cut -c 3-

# End of file