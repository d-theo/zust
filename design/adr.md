01 database light ligne à ligne stockée dans la home du user .zust => path;count;last_access
    format exemple /home/hane;12;1778353961 (score calculé à chaque z, non stocké)
02 intégration shell via `eval "$(zust init bash)"` :
    - hook PROMPT_COMMAND (plutôt qu'un alias sur cd) -> `zust add "$PWD"` à chaque changement de dossier
      couvre cd, pushd/popd, cd -, etc.
    - `z` est une fonction shell : le binaire renvoie un chemin, le shell fait le cd
      (un processus ne peut pas changer le dossier courant de son shell parent)
03 Tab sur `z` ouvre une mini-TUI (10 meilleurs choix, validation directe) dessinée sur /dev/tty ; le choix remplace la saisie
