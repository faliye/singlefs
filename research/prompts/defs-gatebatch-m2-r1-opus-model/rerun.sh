#!/usr/bin/env bash
# defs-gatebatch-m2-r1 云端攻方（Opus）探针复跑：bash rerun.sh
# 只读被判的文件、拷进临时目录；不跑任何重型测试，不执行 54 号本身（拷进临时小仓、拿假 cargo 跑），喂闸只喂 JSON。
# 各探针退 1 = 有攻击格在今天的代码上不成立（打中）；退 2 = 对照格破了（装置本身不对）；退 0 = 全成立。
set -uo pipefail
export PYTHONDONTWRITEBYTECODE=1  # 不在模型目录里留 __pycache__
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf -- "$work"' EXIT
for probe in probe_j1_stage probe_j2_manifest probe_j3_environment probe_j4_hook; do
  python3 "$here/$probe.py" > "$work/$probe.log" 2>&1
  echo "$probe 今天的代码 exit=$?"
  grep '^SUMMARY' "$work/$probe.log"
done
# 改法 F1（54 号：登记表、54 号自己、admission.py 进范围一问；admission.py 进每条用例的指纹）与 F6（admission.py：登记的用例函数要标 #[ignore]），
# 套在临时拷贝上再跑 J1、J4（只在这几个探针上量过，被攻过零轮）
cp "$repo/.claude/gate.d/54-layer0-replay.sh" "$work/54-layer0-replay.sh"
cp "$repo/research/scripts/admission.py" "$work/admission.py"
if patch -s "$work/54-layer0-replay.sh" "$here/fix-f1-54-layer0-replay.patch" && patch -s "$work/admission.py" "$here/fix-f6-admission.patch"; then
  python3 "$here/probe_j1_stage.py" --stage "$work/54-layer0-replay.sh" --admission "$work/admission.py" > "$work/j1-fixed.log" 2>&1
  echo "probe_j1_stage 套 F1、F6 exit=$?"
  grep '^SUMMARY' "$work/j1-fixed.log"
  python3 "$here/probe_j4_hook.py" --admission "$work/admission.py" > "$work/j4-fixed.log" 2>&1
  echo "probe_j4_hook 套 F6 exit=$?"
  grep -E '^ATTACK.*I2 |^SUMMARY' "$work/j4-fixed.log" | cut -c1-120
else
  echo "改法补丁套不上（被判的文件变了）"
fi
