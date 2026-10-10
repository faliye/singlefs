#!/usr/bin/env bash
# 把原型装进冻结副本的一份拷贝：$1 = 草稿目录（装进 $1/tree），$2 = 模型目录，$3 = 改动全文。
# 冻结副本默认取 ~/.local/share/singlefs/work/m3-formal-code-b36-r1（B36_FROZEN 可换），装之前按它的清单核 sha256。
set -euo pipefail
W="$1"; MODEL="$2"; DIFF="$3"; FROZEN="${B36_FROZEN:-$HOME/.local/share/singlefs/work/m3-formal-code-b36-r1}"
rm -rf "$W/tree" "$W/wgsl-pre36-2"
rsync -a --exclude target --exclude .git "$FROZEN/tree/" "$W/tree/"
(cd "$W/tree" && sha256sum -c --quiet "$FROZEN/crates-sha256.txt")
# 36-1b 的 walk.rs 那几块（反打 = 第 35 批的认法 FL），36-2 的三份 wgsl（反打 = 36-2 之前的内核）
awk '/^diff --git a\/crates\/singlefs-checker\/src\/walk.rs/{on=1;print;next} /^diff --git /{on=0} /^````/{on=0} on' "$DIFF" > "$W/walk36-1b.patch"
awk '/^diff --git a\/crates\/singlefs-checker-tier\/src\/crash_judge_checker(_common|_scan)?\.wgsl/{on=1;print;next} /^diff --git /{on=0} /^````/{on=0} on' "$DIFF" > "$W/wgsl36-2.patch"
C="$W/tree/crates"
for n in probe pre36; do
  mkdir -p "$C/b36-checker-$n"
  cp -r "$C/singlefs-checker/src" "$C/b36-checker-$n/"
  cp "$MODEL/cargo/b36-checker-$n.toml" "$C/b36-checker-$n/Cargo.toml"
done
(cd "$C/b36-checker-pre36/src" && patch -s -R walk.rs < "$W/walk36-1b.patch")
python3 "$MODEL/tools/add_probes36.py" "$C/b36-checker-probe/src/walk.rs" new
python3 "$MODEL/tools/add_probes36.py" "$C/b36-checker-pre36/src/walk.rs" pre36
mkdir -p "$W/wgsl-pre36-2"
cp "$C"/singlefs-checker-tier/src/crash_judge_checker*.wgsl "$W/wgsl-pre36-2/"
(cd "$W/wgsl-pre36-2" && patch -s -R -p4 < "$W/wgsl36-2.patch")
for n in proto naga; do
  mkdir -p "$C/b36-opus-$n/src"
  cp "$MODEL/cargo/b36-opus-$n.toml" "$C/b36-opus-$n/Cargo.toml"
done
cp "$MODEL"/src/*.rs "$C/b36-opus-proto/src/"
cp "$MODEL"/naga/main.rs "$C/b36-opus-naga/src/main.rs"
touch "$C"/b36-*/src/*.rs
python3 - "$W/tree/Cargo.toml" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = '    "crates/singlefs-reference-filesystem",\n]'
assert s.count(old) == 1
new_members = ''.join(f'    "crates/{n}",\n' for n in ["b36-checker-probe", "b36-checker-pre36", "b36-opus-proto", "b36-opus-naga"])
open(p, 'w').write(s.replace(old, '    "crates/singlefs-reference-filesystem",\n' + new_members + ']'))
PY
