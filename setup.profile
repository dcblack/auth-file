#!/usr/bin/env bash
#
(return 0 2>/dev/null) && SOURCED=1 || SOURCED=0
if [[ ${SOURCED} == 0 ]]; then
  echo "Don't run $0, source it" >&2
  exit 1
fi

#$Info: Sets up environment. $
export ACTION ENTRY_DIR GIT_WORK_DIR PROJECT_NAME PROJECT_DIR SETUP_PATH

# NOTES
# - PROJECT_NAME and SETUP_PATH are for debug purposes.
# - PROJECT_DIR is used to locate the top directory
function Realpath()
{
  if [[ $# == 0 ]]; then set - .; fi
  # shellcheck disable=SC2016
  local PERLSCRIPT='$p=abs_path(join(q( ),@ARGV));print $p if -e $p'
  /usr/bin/env perl '-MCwd(abs_path)' -le "${PERLSCRIPT}" "$*"
}

function Project_setup()
{
  export ACTION SETUP_PATH ENTRY_DIR
  ENTRY_DIR="$(pwd)"

  case "${ACTION}" in
    add|repeat)
      if git rev-parse --show-toplevel 1>/dev/null 2>&1; then
        # shellcheck disable=SC2034
        GIT_WORK_DIR="$(git rev-parse --show-toplevel)"
      else
        GIT_WORK_DIR=""
      fi
      PROJECT_DIR="$(dirname "${SETUP_PATH}")"
      PROJECT_NAME="$(basename "${PROJECT_DIR}")"
      export GIT_WORK_DIR PROJECT_NAME PROJECT_DIR ENTRY_DIR
      Prepend_path PATH "${PROJECT_DIR}/bin"
      Unique_path PATH
      if [[ -n "${GIT_WORK_DIR}" ]]; then
        header -Color -hbar=- "${GIT_WORK_DIR/*\/}"
      else
        header -Color -hbar=- "${ENTRY_DIR/*\/}"
      fi
      Report_info -ylw "${ENTRY_DIR}"
      echo "$1: ${PROJECT_NAME} environment set up"
      ;;
    rm|-rm|--rm)
      unset GIT_WORK_DIR
      PROJECT_NAME="$(basename "${PROJECT_DIR}")"
      Remove_path PATH "${PROJECT_DIR}/bin"
      echo "$1: ${PROJECT_NAME} environment removed"
      ;;
    *)
      ;;
  esac
}

# Works in ZSH and BASH
# shellcheck disable=SC2154
if [[ -n "${ZSH_VERSION}" ]]; then
  SETUP_PATH="$(Realpath "$0")"
else
  SETUP_PATH="$(Realpath "${BASH_SOURCE[0]}")"
fi
Project_setup "${SETUP_PATH}" "$@"

# Taf!
