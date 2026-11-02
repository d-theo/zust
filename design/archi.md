# Architecture zust

## Contrainte de base
Un binaire ne peut pas changer le dossier courant du shell parent.
=> le binaire `zust` calcule et affiche un chemin sur stdout, une fonction shell `z` fait le `cd`.

    z() { local d; d="$(zust query "$@")" && cd "$d"; }

## Intégration shell (script généré par `zust init bash|zsh`)
- hook sur changement de dossier (bash : PROMPT_COMMAND, zsh : chpwd) -> `zust add "$PWD"`
  (plutôt qu'un alias sur `cd` : couvre aussi pushd/popd, `cd -`, les autres outils)
- fonction `z` (Entrée) -> `cd` vers le meilleur choix
- complétion Tab sur `z` -> `zust tui <requête>` affiche les 10 meilleurs choix
  (TUI dessinée sur /dev/tty, chemin choisi renvoyé sur stdout)

## CLI
Sous-commandes internes : l'utilisateur ne les tape pas, le script shell les appelle.

| Ce que l'utilisateur tape          | Ce que le shell appelle  |
|------------------------------------|--------------------------|
| (une fois) `eval "$(zust init bash)"` | `zust init bash` -> affiche le script d'intégration |
| `cd xxx`                           | `zust add <dossier>` -> count += 1, last_access = now |
| `z proj` + Entrée                  | `zust query proj` -> affiche le meilleur chemin (code retour 1 si aucun) |
| `z proj` + Tab                     | `zust tui proj` -> mini-TUI (10 choix), affiche le chemin choisi |

Cas gérés directement par la fonction `z` : sans argument -> `cd ~`, `z -` -> `cd -`,
argument qui est un dossier existant -> `cd` direct (c'est ce qui se passe après un choix Tab).

## Modèle
struct Dir
    path: PathBuf
    count: f64         // nombre d'accès (incrémenté à chaque add, vieilli)
    last_access: u64   // timestamp unix (s)

impl Dir
    frecency(now: u64) -> f64
        count * poids(now - last_access)
        poids : < 1h -> 4 ; < 1 jour -> 2 ; < 1 semaine -> 0.5 ; sinon 0.25

## Stockage
struct DirStore
    file: PathBuf      // ~/.zust
    dirs: Vec<Dir>

impl DirStore
    load(file) -> Result<DirStore>      // fichier absent => store vide
    add(path, now) -> ()                // count += 1, last_access = now, ou nouvelle entrée
    remove(path) -> ()                  // dossier qui n'existe plus
    save() -> Result<()>                // écriture fichier temporaire + rename (atomique)

Format ligne : `path;count;last_access` ex. `/home/hane;12;1778353961`
Vieillissement : si somme des count > 10000, tous les count *= 0.9, suppression des count < 1.

## Sélection
struct DirSelector

impl DirSelector
    find_best(dirs: &[Dir], query: &[String], now: u64) -> Vec<&Dir>   // pur, sans accès disque
        1. filtre : chaque mot de la requête apparaît dans le chemin, dans l'ordre,
           insensible à la casse ; le dernier mot doit matcher le dernier segment
        2. tri par frecency décroissante
    main garde les n premiers qui existent encore (n = 1 pour query, 10 pour tui)
    et retire du store ceux qui n'existent plus

## Affichage
struct Display

impl Display
    select(paths: &[&Path]) -> Option<usize>   // TUI : flèches/Tab ou 1-9, 0 = 10e, + Entrée, Échap/q/Ctrl-C annule
                                               // un seul candidat => choisi directement, sans TUI
    print(path) -> ()                                // stdout, consommé par la fonction shell
