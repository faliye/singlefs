#!/usr/bin/env bash
# 登记成 light 的自证（与样本档全登记成轻的那个阶段目录）照登记的写法在仓拷贝上各跑一次，strace -f 记下每一个成功的 execve，
# 找起没起真的 cargo / rustc / rustdoc / qemu-system-* / herd7 / verus / z3（路径不在 /tmp 下的才算真的：自证造的替身都在 /tmp）。
# 用法：light-selftest-scan.sh <仓拷贝根> <输出目录> <并行件数> [<只跑的键…>]
# 每件：<输出目录>/<件号>.strace（execve 记录）、.out（自证的输出）、.rc（退出码）、.sec（挂钟秒）；末尾打一张表：件号、键、退出码、秒、真重活的 execve 数与头三条。
# 不跑：登记成 heavy / heavy-if-set 的；agent-watch.py（它起名字像 checker 档测试二进制的替身进程，会惊动主会话的看门狗）。
# 复跑挂钟：72 件自证加样本档一件，并行 6 件，本机 2026-10-01 量到的见报告。
set -uo pipefail
root="$1"; out="$2"; jobs="$3"; shift 3
only=("$@")
mkdir -p "$out"
heavy_re='/(cargo|rustc|rustdoc|cargo-verus|verus|rust-verify|z3|herd7|qemu-system-[a-z0-9_]+)"'
keys=(); commands=()
library_prefix=".claude/gate.d/"lib/
while IFS=$'\t' read -r key weight _reason; do
  [[ -z "$key" || "$key" == \#* ]] && continue
  [[ "$weight" == light ]] || continue
  [[ "$key" == research/scripts/agent-watch.py ]] && continue
  if ((${#only[@]})); then hit=0; for o in "${only[@]}"; do [[ "$o" == "$key" ]] && hit=1; done; ((hit)) || continue; fi
  # 门禁共用库直接执行就是跑自证（不带参数）；别的单独带 --selftest。前缀放变量里：写成 case 的字面模式时重型测试闸把那一行当成起共用库
  interpreter=bash; [[ "$key" == *.py ]] && interpreter=python3
  if [[ "${key#"$library_prefix"}" != "$key" ]]; then cmd=("$interpreter" "$root/$key"); else cmd=("$interpreter" "$root/$key" --selftest); fi
  keys+=("$key"); commands+=("$(printf '%q ' "${cmd[@]}")")
done < "$root/.claude/hooks/selftest-registry.tsv"
if ((${#only[@]} == 0)) || printf '%s\n' "${only[@]}" | grep -qx 'stage-samples'; then
  keys+=("stage-samples"); commands+=("$(printf '%q ' bash "$root/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" "$root/.claude/gate.d")")
fi
total=${#keys[@]}
echo "件数 $total，并行 $jobs"
for ((batch = 0; batch * jobs < total; batch++)); do
  rm -f "$out"/b"$batch"-*.rc
  dispatched=0
  for ((i = batch * jobs; i < total && i < (batch + 1) * jobs; i++)); do
    {
      started=$(date +%s)
      ( cd "$root" && eval "nice -n 19 strace -f -qq -e trace=execve -e signal=none -o $(printf '%q' "$out/$i.strace") ${commands[i]}" ) >"$out/$i.out" 2>&1
      rc=$?
      echo $(( $(date +%s) - started )) > "$out/$i.sec"
      echo "$rc" > "$out/b$batch-$i.rc"
    } &
    dispatched=$((dispatched + 1))
  done
  wait
  got=$(ls "$out"/b"$batch"-*.rc 2>/dev/null | wc -l)
  if [[ "$got" != "$dispatched" ]]; then echo "第 $batch 批派出 $dispatched 件、收回 $got 件：整批作废"; continue; fi
  for ((i = batch * jobs; i < total && i < (batch + 1) * jobs; i++)); do
    rc="$(cat "$out/b$batch-$i.rc")"
    real="$(grep -E "execve\\(\"[^\"]*$heavy_re" "$out/$i.strace" 2>/dev/null | grep -E '= 0$' | grep -vE 'execve\("/tmp/' || true)"
    count=$(grep -c . <<<"$real")
    [[ -z "$real" ]] && count=0
    printf '%s\t%s\trc=%s\tsec=%s\theavy_execve=%s\t%s\n' "$i" "${keys[i]}" "$rc" "$(cat "$out/$i.sec")" "$count" "$(head -3 <<<"$real" | sed -E 's/^[0-9]+ +//; s/, \[.{0,120}.*$//' | tr '\n' ' ')"
  done
done
