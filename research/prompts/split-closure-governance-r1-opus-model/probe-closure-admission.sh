#!/usr/bin/env bash
# 探针：门禁 code-tooling.sh 的单元闭包里有没有 admission.py 一族的单元、经哪条边进来。
#   判的是 UNKEYED_PROGRAM_WORDS 新登的 admission_run 那一行（scripts/gate/lib_function_units_registry.py）在效果上让不让 admission.py 出键：
#   按条复用的共用段只经 admission_run 起 admission.py library-files；那一支的 library_files_fetch、command_library_files 在闭包里，就是经别的边进来的。
# 用法：probe-closure-admission.sh <仓根> <输出目录>（仓根只读；闭包照仓根此刻的源码算，开头另核开工快照点名的 43 份还对不对得上）
# 输出：name=closure_snapshot_check ok=<对上的份数> failed=<对不上的份数>；name=closure_run exit=<码> units=<单元行数> refused=<拒算行数> via=<via 行数>；
#   name=closure_admission_family file=<文件> units=<单元行数>（一份一行）；name=closure_unit unit=<种类:限定名> present=<yes|no> via_rows=<经它进来的 via 行数>；
#   完整输出在 <输出目录>/code-tooling-closure.txt。
# 退出码：0 跑完（只打读数）；2 用法错；3 样本进程没收干净。
set -euo pipefail

if (($# != 2)); then
  echo "用法：probe-closure-admission.sh <仓根> <输出目录>" >&2
  exit 2
fi
repository="$1"
output="$2"
[[ -f "$repository/scripts/gate/lib_function_units.py" && -d "$output" ]] || { echo "  ✗ 仓根里没有 scripts/gate/lib_function_units.py，或输出目录不在" >&2; exit 2; }

snapshot_list="$repository/research/prompts/split-closure-governance-r1-snapshot/judged-sha256.txt"
if [[ -f "$snapshot_list" ]]; then
  (cd "$repository" && sha256sum -c "$snapshot_list") > "$output/snapshot-check.txt" 2>&1 || true   # 对不上的份数在下一行数出来，不当这一步失败
  echo "name=closure_snapshot_check ok=$(grep -c ': OK$' "$output/snapshot-check.txt") failed=$(grep -vc ': OK$' "$output/snapshot-check.txt")"
else
  echo "name=closure_snapshot_check ok=0 failed=missing-list"
fi

closure_file="$output/code-tooling-closure.txt"
cd "$repository"
setsid --wait python3 scripts/gate/lib_function_units.py closure . .claude/gate.d/code-tooling.sh --explain >"$closure_file" 2>"$output/code-tooling-closure.err" &
closure_pid=$!
if wait "$closure_pid"; then closure_exit=0; else closure_exit=$?; fi
if kill -0 -- "-$closure_pid" 2>/dev/null; then
  echo "  ✗ 闭包那一组跑完还有进程" >&2
  exit 3
fi
echo "name=closure_run exit=$closure_exit units=$(grep -vcE '^(via|refused|key|table-launch|entry)'$'\t' "$closure_file") refused=$(grep -c '^refused'$'\t' "$closure_file") via=$(grep -c '^via'$'\t' "$closure_file")"
for file in scripts/gate/admission.py scripts/gate/admission_base.py scripts/gate/admission_commands.py scripts/gate/admission_crash_case.py scripts/gate/admission_unit_key.py; do
  echo "name=closure_admission_family file=$file units=$(grep -vE '^(via|refused)'$'\t' "$closure_file" | grep -cF $'\t'"$file:")"
done
for unit in 'def	library_files_fetch	scripts/gate/admission_unit_key.py' 'def	command_library_files	scripts/gate/admission_unit_key.py' \
            'def	command_verdict_plan	scripts/gate/admission_commands.py' 'def	admission_run	.claude/gate.d/lib/code-tooling-shared-libraries.sh'; do
  kind="${unit%%	*}"; rest="${unit#*	}"; name="${rest%%	*}"; file="${rest#*	}"
  present=no
  if grep -qF "$kind"$'\t'"$name"$'\t'"$file:" "$closure_file"; then present=yes; fi
  echo "name=closure_unit unit=$kind:$name file=$file present=$present via_rows=$(grep -F 'via'$'\t'"$file:$name"$'\t' "$closure_file" | grep -c .)"
  grep -F 'via'$'\t'"$file:$name"$'\t' "$closure_file" | head -n 3 | sed 's/^/  via-sample /' || true
done
