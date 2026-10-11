#!/usr/bin/env bash
# 攻方腿模型（m3-run-compare-split-r2）：编 closure-probe2（只用 std），在 build_trees.sh 建的几棵树上跑，输出落 <草稿目录>/outputs/。
# 用法：run_probe.sh <副本根> <草稿目录>（整条经 run-with-memory-cap.sh 起）
set -euo pipefail
copy_root="$1"; work="$2"; model="$(cd "$(dirname "$0")" && pwd)"
bash "$model/build_trees.sh" "$copy_root" "$work"
mkdir -p "$work/closure-probe2/src" "$work/outputs"
cp "$model/closure-probe2/Cargo.toml" "$work/closure-probe2/Cargo.toml"
sed "s|COPY_ROOT|$copy_root|" "$model/closure-probe2/src/main.rs.template" > "$work/closure-probe2/src/main.rs"
(cd "$work/closure-probe2" && CARGO_TARGET_DIR="$work/target-closure-probe2" cargo build --release --offline --quiet)
probe="$work/target-closure-probe2/release/closure-probe2"
t="$work/trees"
for tree in base k-gpu k-gpu-edit k-amp t-readfault move swap; do "$probe" table "$t/$tree" > "$work/outputs/table-$tree.tsv"; done
# ① 两条自证之二：读 *.wgsl / 自建管线的文件在不在「某一把」闭包里
for tree in base k-gpu k-amp; do echo "== tree=$tree"; "$probe" selfproof2 "$t/$tree"; done > "$work/outputs/selfproof2.out"
# ② k-gpu → k-gpu-edit（只改内核一个字）：整张表逐行比，各种结论各变了几行
{
  for pair in "base k-gpu" "k-gpu k-gpu-edit" "base t-readfault"; do
    set -- $pair
    join -t $'\t' -j1 <(awk -F'\t' 'NR>1{print $1"|"$2"\t"$3}' "$work/outputs/table-$1.tsv" | sort) \
                     <(awk -F'\t' 'NR>1{print $1"|"$2"\t"$3}' "$work/outputs/table-$2.tsv" | sort) |
      awk -F'\t' -v from="$1" -v to="$2" '{split($1,k,"|"); rows[k[1]]++; if ($2!=$3) {changed[k[1]]++; if (k[1]=="observer_code" || k[1]=="gpu_judge" || (k[1]=="cpu_judge" && changed[k[1]]<=3)) print "ROW_CHANGED from=" from " to=" to " kind=" k[1] " file=" k[2]}}
        END {for (kind in rows) printf "TABLE_DIFF from=%s to=%s kind=%s rows=%d changed=%d\n", from, to, kind, rows[kind], changed[kind]+0}' | sort
  done
} > "$work/outputs/table-diff.out"
# ③ 录入版本照「写各表的函数所在文件」取入口时的闭包：候选入口三份，在 base、k-gpu-edit、t-readfault 上各算一次
for tree in base k-gpu k-gpu-edit t-readfault; do
  for entry in crates/singlefs-checker-tier/src/crash_amplification.rs crates/singlefs-checker-tier/src/crash_injection.rs crates/singlefs-checker-tier/src/crash_given_states.rs; do
    echo "tree=$tree $("$probe" closure "$t/$tree" "$entry" | head -1)"
  done
done > "$work/outputs/record-entry-closures.out"
"$probe" closure "$t/base" crates/singlefs-checker-tier/src/crash_amplification.rs > "$work/outputs/closure-files-crash_amplification.out"
# ④ 簇 0 的验收「判法摘要的词法单元多重集不变」：CPU 判器与 GPU 判器两把闭包，base / move / swap 三棵树
for tree in base move swap; do
  for entry in crates/singlefs-checker-tier/src/crash.rs crates/singlefs-checker-tier/src/crash_judge_gpu.rs; do
    "$probe" tokens "$t/$tree" "$entry" > "$work/outputs/tokens-$tree-$(basename "$entry" .rs).out"
    echo "tree=$tree $(head -1 "$work/outputs/tokens-$tree-$(basename "$entry" .rs).out")"
  done
done > "$work/outputs/cluster0-acceptance.out"
for entry in crash crash_judge_gpu; do
  diff <(grep '^TOKEN_COUNT' "$work/outputs/tokens-base-$entry.out") <(grep '^TOKEN_COUNT' "$work/outputs/tokens-move-$entry.out") | sed "s/^/move-vs-base $entry: /" || true
done > "$work/outputs/cluster0-move-token-delta.out"
echo "PROBE_DONE"
