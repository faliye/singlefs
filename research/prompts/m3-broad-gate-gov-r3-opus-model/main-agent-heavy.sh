#!/usr/bin/env bash
# 要主 agent 跑（子 agent 起拷贝上的 admission.py / 驱动 --selftest 被重型闸按「提交时才跑的检查」拒）。在仓根：
#   SINGLEFS_HEAVY_TESTS=user-request nice -n 19 bash research/prompts/m3-broad-gate-gov-r3-opus-model/main-agent-heavy.sh <冻结副本树根> <草稿目录>
# 线程：单线程逐份跑；内存：每趟经 research/scripts/run-with-memory-cap.sh 1G（admission 自证起假 cargo，驱动自证约 100M）。
# 造拷贝：j1c-admission-mutants.py（admission.py 的 build_environment_lines 逐类写错 9 种）、j2d-driver-mutants.py（驱动升档证据分支删掉 6 种），各一份原样对照。
# 每行打「名字<TAB>admission 自证末行<TAB>驱动自证末行」（j2d 那几份只跑驱动自证）。
# 判读：原样两份是「admission 283 格里 1 格判错」与「驱动 74 格都对」；一份写错版两样都与原样同（admission 仍是 1 格、驱动仍是 74 格都对），
#   36 份门禁样本又不翻（outputs/j1c-sweep.txt、j2b-sweep.txt），就是谁都不盯它。
set -uo pipefail
frozen="$1"; work="$2"; here="$(cd "$(dirname "$0")" && pwd)"; cap="$frozen/research/scripts/run-with-memory-cap.sh"
base="$work/heavy-trees/base"; rm -rf "${work:?}/heavy-trees"; mkdir -p "$work/heavy-trees/j1c" "$work/heavy-trees/j2d"
rsync -a --exclude target --exclude .git "$frozen/" "$base/"
python3 "$here/j1c-admission-mutants.py" "$base" "$work/heavy-trees/j1c" > "$work/heavy-trees/j1c.list" || exit 1
python3 "$here/j2d-driver-mutants.py" "$base" "$work/heavy-trees/j2d" > "$work/heavy-trees/j2d.list" || exit 1
last_of() { grep -E "$2" "$1" | tail -n 1; }
for tree in "$base" $(cut -f1 "$work/heavy-trees/j1c.list" | sed "s|^|$work/heavy-trees/j1c/|"); do
  name="$(basename "$tree")"
  ( cd "$tree" && bash "$cap" 1G python3 research/scripts/admission.py --selftest ) > "$work/heavy-trees/$name.admission.log" 2>&1
  ( cd "$tree" && bash "$cap" 1G python3 research/scripts/broad-mutation.py --selftest ) > "$work/heavy-trees/$name.driver.log" 2>&1
  printf '%s\t%s\t%s\n' "$name" "$(last_of "$work/heavy-trees/$name.admission.log" '自证没过|自证通过|格都对|格判错')" "$(last_of "$work/heavy-trees/$name.driver.log" -- '--selftest：')"
done
for tree in $(cut -f1 "$work/heavy-trees/j2d.list" | sed "s|^|$work/heavy-trees/j2d/|"); do
  name="$(basename "$tree")"
  ( cd "$tree" && bash "$cap" 1G python3 research/scripts/broad-mutation.py --selftest ) > "$work/heavy-trees/$name.driver.log" 2>&1
  printf '%s\t—\t%s\n' "$name" "$(last_of "$work/heavy-trees/$name.driver.log" -- '--selftest：')"
done
