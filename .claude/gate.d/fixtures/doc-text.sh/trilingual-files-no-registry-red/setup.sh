#!/usr/bin/env bash
# 没有登记表 .claude/trilingual-documents，只有三份 README：四格的对象都从登记表取，登记表不在就判红。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put README.en.md '# Sample'
put README.md '# 样本'
put README.ja.md '# サンプル'
