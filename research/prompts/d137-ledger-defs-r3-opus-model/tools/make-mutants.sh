#!/usr/bin/env bash
# 造 compile-then-swap.py 的两个变异体（d137-ledger-defs-r3 攻方腿）：都只动 build() 里的一行，命令行解析与 swap() 不动。
#   release-drops-features  带 --release 时不把特性交给 cargo（「if features and mode != "features-dropped":」后面加「and not release」）
#   release-ignored         不把 --release 交给 cargo（「profile = ["--release"] if release else []」换成「profile = []」）
# 每个变异体放进 <输出目录>/<变异名>/scripts/hosts/，同目录结构拷上它 import 的两份库（scripts/kb/lib_atomic_replace.py、.claude/scripts/project_preflight.py）。
# 用法：bash make-mutants.sh <冻结副本或仓根> <输出目录：不在或是空目录>
# 退出码：0 造完；2 用法错、输出目录非空，或变异点在源文件里命中不是恰好 1 处。只拷文件、只起 bash 自己，不起本仓的代码。
set -euo pipefail
repository="${1:?要给冻结副本或仓根}"
out="${2:?要给输出目录}"
if [[ -e "$out" && -n "$(ls -A "$out")" ]]; then
  echo "  ✗ 输出目录 $out 已存在且非空：给一个新的" >&2
  exit 2
fi
source_script="$repository/scripts/hosts/compile-then-swap.py"
build_mutant() {   # build_mutant <变异名> <原行> <换成的行>
  local name="$1" anchor="$2" replacement="$3" hits line
  hits="$(grep -cxF -- "$anchor" "$source_script" || true)"
  if [[ "$hits" != 1 ]]; then
    echo "  ✗ 变异点「$anchor」在 $source_script 里命中 $hits 处（应当 1 处）" >&2
    exit 2
  fi
  mkdir -p "$out/$name/scripts/hosts" "$out/$name/scripts/kb" "$out/$name/.claude/scripts"
  cp "$repository/scripts/kb/lib_atomic_replace.py" "$out/$name/scripts/kb/"
  cp "$repository/.claude/scripts/project_preflight.py" "$out/$name/.claude/scripts/"
  while IFS= read -r line; do
    if [[ "$line" == "$anchor" ]]; then
      printf '%s\n' "$replacement"
    else
      printf '%s\n' "$line"
    fi
  done < "$source_script" > "$out/$name/scripts/hosts/compile-then-swap.py"
  echo "name=mutant_built mutant=$name source_sha256=$(sha256sum "$source_script" | cut -c1-8) changed_lines=$(diff "$source_script" "$out/$name/scripts/hosts/compile-then-swap.py" | grep -c '^[<>]' || true)"
}
build_mutant release-drops-features '    if features and mode != "features-dropped":' '    if features and mode != "features-dropped" and not release:'
build_mutant release-ignored '    profile = ["--release"] if release else []' '    profile = []'
