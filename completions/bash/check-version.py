_check_version_py()
{
    local cur prev
    cur=${COMP_WORDS[COMP_CWORD]}
    prev=${COMP_WORDS[COMP_CWORD-1]}

    local -a options
    options=(
        -h --help
        --debug
        --use-dots
        -nm --no-message
        --show
        -s --silent
        -v --verbose
    )

    if [[ "$cur" == -* ]]; then
        COMPREPLY=( $(compgen -W "${options[*]}" -- "$cur") )
        return 0
    fi

    COMPREPLY=()
}

complete -F _check_version_py check-version.py