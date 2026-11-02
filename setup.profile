#!/usr/bin/env bash

(return 0 2>/dev/null) && SOURCED=1 || SOURCED=0
if [[ ${SOURCED} == 0 ]]; then
  echo "Don't run $0, source it" >&2
  exit 1
fi

#$Info: Sets up environment. $
export ENTRY_DIR GIT_WORK_DIR PROJECT_NAME PROJECT_DIR SETUP_PATH PATH

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

function Header()
{
  if builtin command -v header >/dev/null 2>&1; then
    header "$@"
  else
    while [[ "$1" =~ ^- ]]; do shift; done
    local text
    text="$(tr '[:lower:]' '[:upper:]' <<<"$*")" #>>>
    printf "[1;96m%s[0m\n" "${text}"
  fi
}

function Project_setup()
{
  # @brief does the real work of setup
  export ACTION ENTRY_DIR SETUP_PATH
  if [[ -n "$2" ]]; then ACTION="$2"; fi
  ENTRY_DIR="$(pwd)"

  # Color support
  #
  # shellcheck disable=SC2034
  local RED="[91m" BLU="[92m" YLW="[93m" GRN="[94m" MAG="[95m" CYN="[96m" WHT="[97m" BLD="[m" OFF="[0m" 

  case "${ACTION}" in
    add|repeat)

      if git rev-parse --show-toplevel 1>/dev/null 2>&1; then
        # shellcheck disable=SC2034
        GIT_WORK_DIR="$(git rev-parse --show-toplevel)"
      else
        GIT_WORK_DIR=""
      fi
      PROJECT_DIR="$(dirname "${SETUP_PATH}")"
      if command -v find 1>/dev/null 2>&1; then
        PROJECT_BIN="$(find "${PROJECT_DIR}" -maxdepth 2 -type "d" -name "bin")"
      else
        PROJECT_BIN="$(Realpath bin)"
      fi
      PROJECT_NAME="$(basename "${PROJECT_DIR}")"
      export ENTRY_DIR GIT_WORK_DIR PROJECT_DIR PROJECT_NAME
      Prepend_path PATH "${PROJECT_BIN}"
      Unique_path PATH
      if [[ -n "${GIT_WORK_DIR}" ]]; then
        Header -uc -Color -hbar=- "${GIT_WORK_DIR/*\/}"
      else
        Header -uc -Color -hbar=- "${ENTRY_DIR/*\/}"
      fi
      Report_info -ylw "${ENTRY_DIR}"
      echo "${BLD}${CYN}PROJECT_BIN=${OFF}'${YLW}${PROJECT_BIN}${OFF}'"
      echo "${BLD}${WHT}$1:${OFF} ${PROJECT_NAME} environment set up"
      ;;
    rm|-rm|--rm)
      unset GIT_WORK_DIR
      PROJECT_NAME="$(basename "${PROJECT_DIR}")"
      Remove_path PATH "${PROJECT_BIN}"
      echo "${BLD}${CYN}$1: ${PROJECT_NAME} environment removed${OFF}"
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
