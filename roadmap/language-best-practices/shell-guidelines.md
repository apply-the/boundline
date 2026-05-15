# Shell Guidelines

## Principi

Shell è adatto a glue code, automazione semplice e orchestrazione. Non è adatto a business logic complessa. Se lo script cresce troppo, passare a un linguaggio più strutturato.

## Impostazioni iniziali

Per Bash, usare impostazioni conservative.

```bash
#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
```

Attenzione: `set -e` ha corner case. Non sostituisce gestione errori consapevole.

## Quote sempre

Quasi tutte le variabili devono essere quotate.

### Da evitare

```bash
rm -rf $TARGET_DIR
```

### Preferibile

```bash
rm -rf -- "$TARGET_DIR"
```

## Validare input

```bash
if [[ $# -ne 1 ]]; then
  echo "usage: $0 <target-dir>" >&2
  exit 2
fi

target_dir=$1
```

## Funzioni piccole

```bash
log_info() {
  printf '[INFO] %s\n' "$*" >&2
}
```

Usare `local` dentro funzioni Bash.

```bash
create_archive() {
  local source_dir=$1
  local output_file=$2

  tar -czf "$output_file" -C "$source_dir" .
}
```

## Cleanup con trap

Usare `trap` per cleanup di file temporanei.

```bash
tmp_dir=$(mktemp -d)
cleanup() {
  rm -rf -- "$tmp_dir"
}
trap cleanup EXIT
```

Questo è il meccanismo pratico più vicino a cleanup deterministico in shell.

## File temporanei

Usare `mktemp`. Non costruire path temporanei prevedibili.

### Da evitare

```bash
tmp_file="/tmp/my-script-output"
```

### Preferibile

```bash
tmp_file=$(mktemp)
```

## Comandi esterni

Controllare errori e output.

```bash
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "not inside a git repository" >&2
  exit 1
fi
```

## Parsing

Evitare parsing fragile con `cut`, `awk` e regex quando esiste output machine-readable.

Preferire:

- `jq` per JSON
- `yq` per YAML se disponibile
- formati delimitati chiari
- opzioni `--porcelain` o `--json` dei tool

## Sicurezza

Regole:

- non usare `eval` salvo casi eccezionali
- non concatenare comandi con input utente
- usare array Bash per argomenti dinamici
- usare `--` prima di path o valori utente quando supportato

```bash
args=(--message "$message" --author "$author")
git commit "${args[@]}"
```

## Portabilità

Decidere se lo script è POSIX sh o Bash. Non mischiare.

Per Bash:

```bash
#!/usr/bin/env bash
```

Per POSIX:

```sh
#!/bin/sh
```

Se usi array, `[[ ]]`, process substitution o `local`, non è POSIX sh.

## Cose da evitare

- variabili non quotate
- `eval`
- parsing di `ls`
- path temporanei prevedibili
- script lunghi con business logic complessa
- ignorare exit code
- assumere GNU tools se deve girare su macOS/BSD
- segreti stampati nei log
