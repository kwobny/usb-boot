_usbbootmgr_completion() {
    local file_complete_command=(compgen -A file -- )

    # Format: <short|long> <none|required> COMPLETE_COMMAND
    #         OPTION_TYPE  ARGUMENT_TYPE
    local -A options_info=(
        [-c]="short required file_complete_command"
    )
    # Each value is:
    # NEW_OPTIONS POSITIONAL_ARGS
    # POSITIONAL_ARGS = array of COMPLETE_COMMAND
    local -A subcommands=(
        [help]="help_options help_args"
        change-kernel upgrade-kernel
    )
    local -A help_options=()
    local help_args=()

    local all_possible=( "${!options_info[@]}" "${!subcommands[@]}" )

    # last_word_type can be one of the following:
    # option_expecting_argument
    # clean_state
    local last_word_type=clean_state
    local last_option=

    local i
    for i in (( i=1; i <= $COMP_CWORD; i++ )); do
        local word="${COMP_WORDS[$i]}"
        if [[ $i -eq $COMP_CWORD ]]; then
            local is_current_word=true
        else
            local is_current_word=false
        fi

        case "$last_word_type" in
            'option_expecting_argument')
                if [[ $is_current_word == true ]]; then
                    local info=( ${options_info["$last_option"]} )
                    local -n complete_command="${info[2]}"
                    COMPREPLY=( $("${complete_command[@]}" "$word") )
                    return
                else
                    last_word_type=clean_state
                    continue
                fi
                ;;
            'clean_state')
                if [[ $is_current_word == true ]]; then
                    COMPREPLY=( $(
                        compgen -W "${all_possible[*]}" -- "$word"
                    ) )
                    return
                else
                    local option
                    for option in "${!options_info[@]}"; do
                        if [[ "$option" == "$word" ]]; then
                            local info=( ${options_info["$option"]} )
                            local arg_type="${info[1]}"
                            case "$arg_type" in
                                'none')
                                    last_word_type=clean_state
                                    ;;
                                'required')
                                    last_word_type=option_expecting_argument
                                    ;;
                            esac
                            continue 2
                        fi
                    done

                    local subcommand
                    for subcommand in "${!subcommands[@]}"; do
                        if [[ "$subcommand" == "$word" ]]; then
                            local info=( ${subcommands[subcommand]} )
                            local -n new_options="${info[0]}"
                            options_info=( "${new_options[@]}" )
                            continue 2
                        fi
                    done
                    continue
                fi
                ;;
        esac
    done

    COMPREPLY=( $(compgen -W "${combined_possibilities[*]}" -- "$current_word") )
}

complete -F _usbbootmgr_completion usbbootmgr
