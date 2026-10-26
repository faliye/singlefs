#!/usr/bin/env bash
# X3「1579 → 1232」「792 → 436」的另一条路数：不动调用方（不换 HostLookup 的 Popen），而是把被调方 host-resources.py 换成只有一行 `pass` 的文件，
# 闭包照原样连到它、只剩它自己的一个模块体；数出来的单元若等于切边的数加 1，说明少掉的恰好是只经它可达的那几个（别的路到不了它，也不经它到别处）。
# 用法：x3-empty-callee.sh <原树> <放变体的目录> <输出目录>
set -u
base="$1" variants="$2" out="$3"
tree="$variants/hr-empty"; rm -rf -- "$tree"; cp -al "$base" "$tree"
rm -f "$tree/scripts/hosts/host-resources.py"; printf 'pass\n' > "$tree/scripts/hosts/host-resources.py"
mkdir -p "$out"
for entry in scripts/gate/verus-proof-check.py::check_proof scripts/mutation/broad-mutation.py::Runner.escalate_step; do
  label="$(printf '%s' "$entry" | tr '/:.' '___')"
  (cd "$tree" && nice -n 19 python3 -B scripts/gate/lib_function_units.py closure . "$entry") > "$out/empty-$label.txt" 2>&1; rc=$?
  echo "name=empty_callee entry=$entry rc=$rc units=$(awk -F'\t' '$1!="refused" && $1!="key" && NF>=4' "$out/empty-$label.txt" | wc -l) host_resources_units=$(grep -cP '\tscripts/hosts/host-resources\.py:' "$out/empty-$label.txt") refusals=$(awk -F'\t' '$1=="refused"{print $2}' "$out/empty-$label.txt" | tr '\n' ',')"
done
