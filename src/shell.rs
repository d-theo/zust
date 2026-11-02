pub fn init_script(shell: &str) -> Option<&'static str> {
    match shell {
        "bash" => Some(BASH),
        _ => None,
    }
}

const BASH: &str = r#"
__zust_hook() {
    if [[ "${__zust_oldpwd:-}" != "$PWD" ]]; then
        __zust_oldpwd="$PWD"
        command zust add -- "$PWD" 2>/dev/null
    fi
}

if [[ ";${PROMPT_COMMAND:-};" != *";__zust_hook;"* ]]; then
    PROMPT_COMMAND="__zust_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi

# remplace un éventuel alias z (z.sh) ; "function" évite l'expansion de l'alias
unalias z 2>/dev/null
function z {
    local last="${!#}"
    if [[ $# -eq 0 ]]; then
        builtin cd ~
    elif [[ $# -eq 1 && "$1" == "-" ]]; then
        builtin cd -
    elif [[ -d "$last" && ( $# -eq 1 || "$last" == /* ) ]]; then
        # dossier existant, ou chemin absolu inséré par Tab
        builtin cd -- "$last"
    else
        local dir
        dir="$(command zust query -- "$@")" && builtin cd -- "$dir"
    fi
}

__zust_complete() {
    local dir
    COMPREPLY=()
    dir="$(command zust tui -- "${COMP_WORDS[@]:1:COMP_CWORD}")" || return
    COMPREPLY=("$(printf '%q' "$dir")")
    # demande de statut au terminal : sa réponse \e[0n arrive comme une frappe,
    # liée à accept-line => la ligne `z <chemin>` est validée sans Entrée
    printf '\e[5n' > /dev/tty
}

complete -F __zust_complete z
if [[ $- == *i* ]]; then
    bind '"\e[0n": accept-line'
fi
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn bash_script_is_valid_syntax() {
        let status = Command::new("bash").args(["-n", "-c", BASH]).status().unwrap();
        assert!(status.success());
    }

    #[test]
    fn bash_script_replaces_existing_z_alias() {
        let script = format!("alias z='echo old'\n{BASH}\ntype -t z");
        let out = Command::new("bash").args(["--norc", "-O", "expand_aliases", "-c", &script]).output().unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "function");
    }

    #[test]
    fn unknown_shell_is_rejected() {
        assert!(init_script("fish").is_none());
    }
}
