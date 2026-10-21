#!/usr/bin/env bash
# m4-trunc-r2 攻方腿模型的全部步骤：model.py 各模式逐臂并行跑，输出拼成 <输出目录> 下七份 .txt（按臂登记的次序，逐次相同）。
# 用法：bash run_all.sh <model.py> <输出目录>
#   并行件数取 SINGLEFS_THREAD_CAP（scripts/hosts/capped.sh 设的），没设取 4，最多 16；每件把退出码写进 parts/ 下自己的 .rc，
#   件数对不上或有非 0 的整次作废、退 1。
set -euo pipefail
model="${1:?缺 model.py 路径}"
out="${2:?缺输出目录}"
mkdir -p "$out/parts"
jobs="${SINGLEFS_THREAD_CAP:-4}"
if (( jobs > 16 )); then jobs=16; fi
mapfile -t arms < <(python3 -I "$model" list_arms)

piece() {   # piece <模式名> <臂> <model.py 的参数…>：一件，输出与退出码落进 parts/
  local name="$1" arm="$2"
  shift 2
  python3 -I "$model" "$@" > "$out/parts/$name-$arm.txt" 2> "$out/parts/$name-$arm.err"
}
export -f piece
export model out

rm -f "${out:?}"/parts/*.rc
{
  for arm in "${arms[@]}"; do
    printf '%s\n' "exhaustive $arm exhaustive 5 $arm" "after_prefix $arm after_prefix 3 $arm" \
      "checkpoint $arm checkpoint 4 $arm" "random $arm random 30000 12 20261017 $arm"
  done
  printf '%s\n' "fixed all fixed" "future all future 3000 8 7" "checkpoint_pairs all checkpoint_pairs 3000 8 11"
} | xargs -P "$jobs" -L 1 bash -c 'name="$0"; arm="$1"; if piece "$0" "$@"; then echo 0; else echo "$?"; fi > "$out/parts/$name-$arm.rc"'

expected=$(( ${#arms[@]} * 4 + 3 ))
count=$(find "$out/parts" -name '*.rc' | wc -l)
if [[ "$count" -ne "$expected" ]]; then
  echo "✗ 件数对不上：派了 $expected 件，收到 $count 个退出码，整次作废" >&2
  exit 1
fi
bad=$(grep -Lx 0 "$out"/parts/*.rc || true)
if [[ -n "$bad" ]]; then
  echo "✗ 有件退了非 0：$(echo "$bad" | tr '\n' ' ')" >&2
  exit 1
fi
for mode in exhaustive after_prefix checkpoint random; do
  for arm in "${arms[@]}"; do cat "$out/parts/$mode-$arm.txt"; done > "$out/$mode.txt"
done
for mode in fixed future checkpoint_pairs; do cp "$out/parts/$mode-all.txt" "$out/$mode.txt"; done
rm -rf "${out:?}/parts"
echo "✓ 跑完 $expected 件：$(ls "$out" | tr '\n' ' ')"
