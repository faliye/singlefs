#!/usr/bin/env bash
# T4 攻方复现：一个「看起来对、其实错」的实现——watch 收到 HUP 时停掉还在跑的单元
# （HUP 正是「会话关闭」那一路），自证 11 格照样全绿，因为 Ⓔ 只发 TERM、不发 HUP / INT。
# 主 agent 带自己的前缀跑：bash repro-t4-watch-hup-gap.sh <detached-run.sh 的绝对路径>
set -uo pipefail
src="${1:?用法：repro-t4-watch-hup-gap.sh <detached-run.sh 的绝对路径>}"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr1-t4.XXXXXX")"
bad="$work/detached-run.sh"
mkdir -p "$work/.claude"
# 让 preflight 解析到真副本：把坏脚本放进真脚本旁边的同构目录
scripts_dir="$(cd "$(dirname "$src")" && pwd)"
bad="$scripts_dir/opusr1t4-detached-run.sh"
cp -- "$src" "$bad"
# 坏改法：watch_stopped 里，收到的信号是 HUP 时停掉还在跑的单元（独立于 watch-stops-units 开关）
python3 - "$bad" <<'PY'
import sys
p=sys.argv[1]; s=open(p,encoding="utf-8").read()
needle='  if detached_run_break_is watch-stops-units; then'
inject=('  if [[ "$1" == HUP ]]; then\n'
        '    for record in ${DETACHED_RUN_RUNNING_RECORDS[@]+"${DETACHED_RUN_RUNNING_RECORDS[@]}"}; do\n'
        '      systemctl --user stop "$(detached_run_field "$record" unit)" 2>/dev/null\n'
        '    done\n'
        '  fi\n')
assert needle in s, "anchor not found"
s=s.replace(needle, inject+needle, 1)
open(p,"w",encoding="utf-8").write(s)
print("injected HUP-stops-units into", p)
PY
echo "== 跑坏脚本的 --selftest（期望：仍退 0、11 格全绿 ⇒ 自证没有 HUP/INT 不停单元那一格，T4 打中）"
bash "$bad" --selftest; echo "   --selftest 退出码 $?"
rm -f -- "$bad"
rm -rf -- "$work"
