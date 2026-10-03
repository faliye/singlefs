#!/usr/bin/env bash
# 复现 V4（真单元，自证只起 sleep / true / bash）：对 v4_mutations.py 的每种错实现跑 --selftest（自证前缀换成 opusr2-<pid>-），
#   control 是只换前缀的原件。期望：control 与五种错实现都退 0、「自证 21 格都对」⇒ 这五种错实现自证放得过。
#   之后两段看 b、c 的后果（真单元）：b 起默认时限的单元读 TimeoutStopUSec；c 停一条收到 TERM 后 8 秒才退的命令，看 stop 几秒回来、报什么、后来登记里的 exit。
# 用法：bash repro-v4-selftest-wrong-impls.sh <仓根>（主 agent 带前缀跑，例如 SINGLEFS_HEAVY_TESTS=user-request）
set -u
repo="$1"; here="$(cd "$(dirname "$0")" && pwd)"; source "$here/repro-lib.sh"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-rv4.XXXXXX")"
for name in control a-linger-unreadable-silent b-default-stop-timeout-unset c-stop-nonblocking f-query-check-after-archive g-watch-skips-unknown; do
  python3 "$here/v4_mutations.py" "$repo/research/scripts/detached-run.sh" "$work/$name.sh" "$name" > /dev/null || { echo "$name 改不上"; continue; }
  script="$(opusr2_tree "$repo" "$work/$name" "$work/$name.sh")"
  bash "$script" --selftest > "$work/$name.out" 2>&1; rc=$?
  echo "$name：--selftest 退 $rc；$(grep -c '自证格不对' "$work/$name.out") 格不对；末行：$(tail -n 1 "$work/$name.out" | cut -c1-40)"
done
opusr2_residue
echo "== b 的后果：默认时限起一条 sleep 300"
script="$(opusr2_tree "$repo" "$work/b-effect" "$work/b-default-stop-timeout-unset.sh")"; label="opusr2-b-$$"; unit="singlefs-detached-$label.service"
DETACHED_RUN_STATE_DIRECTORY="$work/b-effect/state" bash "$script" start --label "$label" -- sleep 300 > /dev/null 2>&1
echo "单元 TimeoutStopUSec=$(systemctl --user show -p TimeoutStopUSec --value "$unit")；登记 stop-timeout=$(cat "$work/b-effect/state/$label/stop-timeout")；用户级默认 $(systemctl --user show -p DefaultTimeoutStopUSec --value)"
opusr2_stop_unit "$unit"
for name in control c-stop-nonblocking; do
  echo "== c 的后果（$name）：命令收到 TERM 后 8 秒才退 9"
  script="$(opusr2_tree "$repo" "$work/c-$name" "$work/$name.sh")"; label="opusr2-c-$$"; unit="singlefs-detached-$label.service"; st="$work/c-$name/state"
  DETACHED_RUN_STATE_DIRECTORY="$st" bash "$script" start --label "$label" -- bash -c 'trap "sleep 8; exit 9" TERM; sleep 300 & wait' > /dev/null 2>&1
  sleep 1; t0=$(date +%s)
  DETACHED_RUN_STATE_DIRECTORY="$st" bash "$script" stop "$label" > "$work/c-$name.stop" 2>&1; rc=$?
  echo "stop 退 $rc、用了 $(( $(date +%s) - t0 )) 秒：$(tail -n 1 "$work/c-$name.stop" | cut -c1-120)"
  sleep 10; echo "10 秒后登记 exit=$(cat "$st/$label/exit" 2>/dev/null || echo 没有)；stop-killed=$(cat "$st/$label/stop-killed" 2>/dev/null || echo 没有)"
  opusr2_stop_unit "$unit"
done
opusr2_residue
rm -rf -- "$work"
