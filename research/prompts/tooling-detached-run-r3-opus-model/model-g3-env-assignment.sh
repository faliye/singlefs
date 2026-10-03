#!/usr/bin/env bash
# 模型 G3-E：不经 systemd，把 detached-run.sh 里单元那个 bash 的代码（RUNNER 三段 + 收尾）原样抠出来在本进程里跑，
#   命令写成 detached-run.sh start … -- <命令> 交给单元的那几个词：开头是「带 = 而不是合法变量名」的词时，
#   单元里 exec env -- "$@" 把它当 名=值、照跑后面那条（两道钩子按 shell 的写法把它当命令词，看不到后面那条，见 probe-g3.txt E1–E5）。
# 用法：bash model-g3-env-assignment.sh <detached-run.sh 的路径>
set -u
script="$1"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr3-g3e.XXXXXX")"; trap 'rm -rf -- "$work"' EXIT
runner="$(awk '/^read -r -d .. DETACHED_RUN_RUNNER_(HEAD|EXIT|SIDE) <</{f=1;next} /^RUNNER$/{f=0} f' "$script")"$'\n''exit "$rc"'
for first in --force=1 1=1 a-b=1 a.b=1 FOO=1; do
  rec="$work/rec-${first%%=*}"; mkdir -p "$rec"; marker="$work/marker-${first%%=*}"
  bash -c "$runner" detached-run-runner "$rec" "$rec/log" "$work/side" "$first" touch "$marker" 2> "$work/err"
  echo "命令开头 $first：单元那个 bash 退 $?；登记 exit=$(cat "$rec/exit" 2>/dev/null)；后面那条（touch marker）跑没跑：$([[ -e "$marker" ]] && echo 跑了 || echo 没跑)；日志：$(head -c 120 "$rec/log" | tr '\n' ' ')"
done
