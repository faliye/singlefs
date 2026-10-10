#!/usr/bin/env bash
# 把原型装进一份冻结副本的拷贝（$1 = 草稿目录，里面要有 tree/ 与模型的 src/ cargo/ 与两个 python 脚本所在目录 $2）。
set -euo pipefail
W="$1"; MODEL="$2"; DIFF="$3"; FROZEN=/tmp/claude-1000/m3-formal-code-b35-r1/tree
rm -rf "$W/tree"
rsync -a --exclude target --exclude .git "$FROZEN/" "$W/tree/"
(cd "$W/tree" && sha256sum -c --quiet /tmp/claude-1000/m3-formal-code-b35-r1/crates-sha256.txt)
awk '/^diff --git a\/crates\/singlefs-checker\/src\/walk.rs/{on=1} /^diff --git a\/crates\/singlefs-harness\/tests\/checker_known_bad_images.rs/{on=0} on' "$DIFF" > "$W/walk35-1.patch"
C="$W/tree/crates"
for n in pre35 probe mutr mutn mutz; do
  mkdir -p "$C/b35-checker-$n"
  cp -r "$C/singlefs-checker/src" "$C/b35-checker-$n/"
  cp "$MODEL/cargo/b35-checker-$n.toml" "$C/b35-checker-$n/Cargo.toml"
done
(cd "$C/b35-checker-pre35/src" && patch -s -R walk.rs < "$W/walk35-1.patch")
python3 "$MODEL/tools/add_probes.py" "$C/b35-checker-pre35/src/walk.rs" pre35
python3 "$MODEL/tools/add_probes.py" "$C/b35-checker-probe/src/walk.rs" probe
for n in mutr mutn mutz; do python3 "$MODEL/tools/make_mutant.py" "$C/b35-checker-$n/src/walk.rs" "$n"; done
mkdir -p "$C/b35-opus-proto/src"
cp "$MODEL/cargo/b35-opus-proto.toml" "$C/b35-opus-proto/Cargo.toml"
cp "$MODEL"/src/*.rs "$C/b35-opus-proto/src/"
touch "$C"/b35-*/src/*.rs
python3 - "$W/tree/Cargo.toml" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = '    "crates/singlefs-reference-filesystem",\n]'
assert s.count(old) == 1
new_members = ''.join(f'    "crates/b35-checker-{n}",\n' for n in ["pre35", "probe", "mutr", "mutn", "mutz"]) + '    "crates/b35-opus-proto",\n'
open(p, 'w').write(s.replace(old, '    "crates/singlefs-reference-filesystem",\n' + new_members + ']'))
PY
