#!/usr/bin/env bash
# detached-run-script-signature: 假的，只记日志
d="$PWD"; while [ "$d" != / ] && [ ! -e "$d/.oracle-log" ]; do d="$(dirname "$d")"; done
printf 'detached-run %s\n' "$*" >> "$d/.oracle-log"
