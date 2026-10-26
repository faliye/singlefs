#!/usr/bin/env bash
# detached-run-script-signature: 假的，只记日志
dir="$PWD"
while [ "$dir" != / ] && [ ! -e "$dir/.oracle-log" ]; do dir="$(dirname "$dir")"; done
printf 'detached-run %s\n' "$*" >> "$dir/.oracle-log"
