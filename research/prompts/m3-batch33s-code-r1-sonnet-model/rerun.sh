#!/usr/bin/env bash
# 用法：SINGLEFS_REPO=<冻结副本根，含 crates/> SINGLEFS_MAIN_REPO=<主仓根，取 research/scripts/> bash rerun.sh <空的输出目录>
# 复跑正推腿报告 0.1、0.2、0.7、0.8（不编译、不跑 cargo）：sha256 对 18 份、变异表静态检查、守卫 u32 仿真、变异表与合前对比。
# 报告 0.3–0.6、0.9 要编译（harness 基线、kernel-sizes 小程序、变异行逐条现跑），按 README.md 分条跑，不在这里。
# 输出写 <输出目录>/rerun.out，逐行是「标签 TAB 结果」；compare.sh 比两次输出。
set -euo pipefail
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO（冻结副本根）}"
: "${SINGLEFS_MAIN_REPO:?要设 SINGLEFS_MAIN_REPO（主仓根）}"
out_dir=${1:?要给一个输出目录}
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
[ -z "$(ls -A "$out_dir" 2>/dev/null)" ] || { echo "输出目录不空：$out_dir" >&2; exit 2; }
mkdir -p "$out_dir"
out=$out_dir/rerun.out
: > "$out"
# 0.1：18 份文件 sha256
cd "$SINGLEFS_REPO"
ok=$(sha256sum -c "$here/crates-sha256.txt" 2>&1 | grep -c ': OK$' || true)
printf '0.1 sha256 OK 份数\t%s\n' "$ok" >> "$out"
# 0.2：变异表静态检查（把冻结副本拷成临时副本再判，不动冻结副本）
work=$out_dir/work; mkdir -p "$work"
cp -a "$SINGLEFS_REPO"/crates "$SINGLEFS_REPO"/Cargo.toml "$SINGLEFS_REPO"/Cargo.lock "$SINGLEFS_REPO"/litmus "$work"/
mkdir -p "$work/.claude" "$work/research/scripts"
static=$(cd "$SINGLEFS_MAIN_REPO" && python3 research/scripts/crates-mutation-rows.py static-check "$work" 2>&1 | grep '^CHECKED' || true)
printf '0.2 static-check\t%s\n' "$static" >> "$out"
# 0.7：守卫仿真
printf '0.7 guard_model\t%s\n' "$(python3 -I "$here/guard_model.py" | tr '\n' ';')" >> "$out"
# 0.8：变异表与合前对比
zcat "$here/mutations-pre-s.tsv.gz" > "$work/mutations-pre-s.tsv"
python3 -I "$here/cmp_mut.py" "$work/mutations-pre-s.tsv" "$SINGLEFS_REPO/crates/mutations.tsv" > "$work/cmp.out"
printf '0.8 cmp_mut 头三行\t%s\n' "$(sed -n 1,3p "$work/cmp.out" | tr '\n' ';')" >> "$out"
printf '0.8 cmp_mut CARGO/TEST DIFF 行\t%s\n' "$(grep -c 'CARGO/TEST DIFF' "$work/cmp.out" || true)" >> "$out"
printf '0.8 cmp_mut 追加的 S 行（名字含「第33批」）\t%s\n' "$(grep -c '^3[0-9][0-9][0-9] 第33批' "$work/cmp.out" || true)" >> "$out"
rm -rf "$work"
cat "$out"
