#!/usr/bin/env bash
# K1 用：在 <工作目录> 下建一棵 base（从 <仓根> 拷判法与 crates 要的几样），再用硬链接拷出四份 lib_function_units.py 各改一处的变体（v-E1…v-E4）
# 和一份在 crates/singlefs-core/src/checksum.rs 的 fn table_index 上加一行 #[inline(never)] 的源码改动树 t1。
# （E1…E5 的含义见 k1_edits.py 里的 EDITS；v-E5 是 E5。另有 v-cell、v-lvl 两份：自证那一格的两半各改一处，用来看 K4 里修补二批删掉的那一半有没有牙。）
# 用法：k1_trees.sh <仓根> <工作目录>
set -euo pipefail
repo="$1"; work="$2"; tools="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$work/base/.claude/gate.d"
cd "$repo"
for item in crates scripts Cargo.toml Cargo.lock .cargo litmus; do cp -a "$item" "$work/base/$item"; done
cp -a .claude/heavy-thread-default "$work/base/.claude/"
cp -a .claude/gate.d/stage-inputs.tsv "$work/base/.claude/gate.d/"
cp -a .claude/scripts "$work/base/.claude/scripts"
cp -a .claude/singlefs-ai-sop "$work/base/.claude/singlefs-ai-sop"
rm -rf "$work/base/.claude/scripts/__pycache__"
for edit in E1-first-line-after-attributes E2-qualified-name-separator E3-equivalent-rewrite E4-fingerprint-after-attributes E5-fingerprint-ignores-inline-attribute; do
  short="${edit%%-*}"
  cp -al "$work/base" "$work/v-$short"
  python3 -I "$tools/k1_edits.py" lfu "$edit" "$work/v-$short/scripts/gate/lib_function_units.py"
done
cp -al "$work/base" "$work/t1"
python3 -I "$tools/k1_edits.py" tree "$work/t1" crates/singlefs-core/src/checksum.rs table_index
cp -al "$work/base" "$work/v-cell"
python3 -I - "$work" <<'PY'
import os, sys
work = sys.argv[1]
def edit(path, old, new):
    text = open(path, encoding="utf-8").read()
    assert text.count(old) >= 1, (path, old[:40])
    temporary = path + ".partial"
    open(temporary, "w", encoding="utf-8").write(text.replace(old, new, 1))
    os.replace(temporary, path)
edit(work + "/v-cell/scripts/gate/lib_function_units.py",
     '        return snippet_of(self.source_rows[line - 1]) if 0 < line <= len(self.source_rows) else ""\n',
     '        return snippet_of(self.source_rows[line - 1]) if 1 <= line <= len(self.source_rows) else ""\n')
PY
cp -al "$work/base" "$work/v-lvl"
python3 -I - "$work" <<'PY'
import os, sys
work = sys.argv[1]
path = work + "/v-lvl/scripts/crash/lib_verdict_library.py"
old = '    return hashlib.sha256(text.encode("utf-8", "surrogateescape") if isinstance(text, str) else text).hexdigest()\n'
new = '    return hashlib.sha256(text.encode("utf-8", "replace") if isinstance(text, str) else text).hexdigest()\n'
text = open(path, encoding="utf-8").read()
assert text.count(old) >= 1
open(path + ".partial", "w", encoding="utf-8").write(text.replace(old, new, 1))
os.replace(path + ".partial", path)
PY
