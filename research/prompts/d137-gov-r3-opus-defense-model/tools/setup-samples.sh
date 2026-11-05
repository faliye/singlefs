#!/usr/bin/env bash
# 建喂钩子用的样本目录：setup-samples.sh <样本目录>（不在或为空）。只写样本目录。
set -euo pipefail
s="${1:?样本目录}"
mkdir -p "$s/fx" "$s/sub"
printf 'echo hi\n' > "$s/fx/light.sh"
printf 'echo hi\n' > "$s/s.sh"
printf 'cargo test --all\n' > "$s/heavy.sh"
printf 'a\nb\n' > "$s/f.txt"
printf '{ print $1 }\n' > "$s/p.awk"
printf 'print(1)\n' > "$s/t.py"
printf 'cargo test --all\n' > "$s/e-heavy.sh"
echo "name=samples dir=$s"
