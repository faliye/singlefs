#!/usr/bin/env bash
# K3 第四点：20-1 原补丁与合入后的 crash_injection.rs / wgsl / image.rs 比。
# 做法：把补丁打到 git HEAD 上（补丁的原底版），得到「HEAD + 20-1」；再与冻结副本里合入后的文件逐行 diff，
# 看被合入删掉的行里有没有 20-1 自己加的行、20-1 加的行在合入后是否都在、20-1 删的行有没有回来。
# 用法：bash patch_vs_merged.sh <补丁> <冻结副本根> <输出目录>
set -u
PATCH=$1; TREE=$2; OUT=$3; R=/home/fy5090/code/singlefs
rm -rf "$OUT"; mkdir -p "$OUT/pt"
FILES="crates/singlefs-checker-tier/src/crash_injection.rs crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl crates/singlefs-checker/src/image.rs"
for f in $FILES; do mkdir -p "$OUT/pt/$(dirname $f)"; git -C $R show HEAD:$f > "$OUT/pt/$f"; done
echo "补丁里 crash_injection.rs 的底版 blob：$(grep -m1 '^index' $PATCH)；HEAD 里那份的 blob：$(git hash-object $OUT/pt/crates/singlefs-checker-tier/src/crash_injection.rs)"
(cd "$OUT/pt" && patch -p1 < $PATCH | sed 's/^/  patch: /')
echo "打完之后 crash_injection.rs 的 blob：$(git hash-object $OUT/pt/crates/singlefs-checker-tier/src/crash_injection.rs)"
for f in $FILES; do
  diff "$OUT/pt/$f" "$TREE/$f" > "$OUT/d-$(basename $f).txt"
  echo "== $f：HEAD+20-1 对 合入后：被删(<) $(grep -c '^<' $OUT/d-$(basename $f).txt) 行，新增(>) $(grep -c '^>' $OUT/d-$(basename $f).txt) 行"
  grep '^<' "$OUT/d-$(basename $f).txt" | cut -c1-170 | sed 's/^/   被删 /'
done
python3 - "$PATCH" "$TREE" "$R" "$OUT" <<'PY'
import sys, subprocess, collections
patch, tree, repo, out = sys.argv[1:5]
files = ["crates/singlefs-checker-tier/src/crash_injection.rs", "crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl", "crates/singlefs-checker/src/image.rs"]
cur=None; add=collections.defaultdict(list); rem=collections.defaultdict(list)
for l in open(patch, encoding="utf-8").read().split("\n"):
    if l.startswith("diff --git"): cur=l.split(" b/")[-1]; continue
    if cur and l.startswith("+") and not l.startswith("+++"): add[cur].append(l[1:])
    if cur and l.startswith("-") and not l.startswith("---"): rem[cur].append(l[1:])
for f in files:
    merged = open(tree + "/" + f, encoding="utf-8").read().split("\n")
    head = subprocess.run(["git","-C",repo,"show","HEAD:"+f],capture_output=True,text=True).stdout.split("\n")
    cm, ch, ca, cr = collections.Counter(merged), collections.Counter(head), collections.Counter(add[f]), collections.Counter(rem[f])
    missing = [l[:80] for l, n in ca.items() if l.strip() and cm[l] < n]
    back = [l[:80] for l, n in cr.items() if l.strip() and ca[l] == 0 and ch[l] - n > 0 and cm[l] > ch[l] - n]
    removed_by_merge = [l[2:] for l in open(f"{out}/d-{f.split('/')[-1]}.txt", encoding="utf-8").read().split("\n") if l.startswith("< ")]
    own = [l[:80] for l in removed_by_merge if ca[l] > 0 and l.strip()]
    print(f"{f.split('/')[-1]}: 补丁加的不同非空行 {sum(1 for l in ca if l.strip())}；合入后少于补丁加的次数的行 {missing}；补丁删掉的行又回来 {back}；被合入删掉的行里 20-1 自己加的 {own}")
PY
