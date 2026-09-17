#!/usr/bin/env bash
# 模型：在 singlefs 仓的副本（rsync，不带 .git 与 target）里走三段序列，不碰真仓。
#   ① kb-scribe 第 3 步：一条已定分项翻回未定（D22 已定项 7，副本里只动索引表那一行）→ relabel-item.py 改写 → 它名下的 22 号、不归它的 33 号各判什么；
#      E142 留存产物里印着同一个标签的那一行、源码里对应的字符串字面量改没改。
#   ② kb-scribe 第 1 步：replace-batch.py 的规格里有一条 .claude/kb/ 之外的文件（CLAUDE.md）→ 写不写。
#   ③ experiment-runner 第 3 步：照定义字面在仓根跑 `bash research/scripts/mutate.sh …`（参数照 kb 实验页的写法）→ 报什么。
#   SINGLEFS_ROOT=/home/fy5090/code/singlefs WORK_PARENT=/tmp/claude-1000/agent-defs-r1-opus bash copy-repo-sequences.sh
# 耗时约一分钟；拷两份 79 MB 副本进 WORK_PARENT 下的临时目录，跑完删掉。输出 name=… 行，末行 name=done。
set -uo pipefail
ROOT="${SINGLEFS_ROOT:-/home/fy5090/code/singlefs}"
PARENT="${WORK_PARENT:-/tmp/claude-1000/agent-defs-r1-opus}"
WORK="$(mktemp -d "$PARENT/copy-sequences.XXXXXX")"
trap 'rm -rf "${WORK:?}"' EXIT
rsync -a --exclude target --exclude .git "$ROOT/" "$WORK/repo/" || { echo "name=error step=rsync"; exit 2; }
cd "$WORK/repo" || exit 2
DECISION=".claude/kb/decisions/22-单元原子性怎么合成.md"

for stage in 22-item-ref-status.sh 33-mutation-tables.sh 87-replay.sh; do
  owners="$(awk -F'\t' -v stage="$stage" '$1 == stage { print $2 }' .claude/gate.d/stage-owners.tsv)"
  echo "name=owner stage=$stage owners=$owners"
done

# ---- ① relabel ----
bash .claude/gate.d/22-item-ref-status.sh >/dev/null 2>&1; echo "name=before stage=22 rc=$?"
bash .claude/gate.d/33-mutation-tables.sh >/dev/null 2>&1; echo "name=before stage=33 rc=$?"
python3 - "$DECISION" <<'PY' || { echo "name=error step=flip"; exit 2; }
import pathlib, sys
path = pathlib.Path(sys.argv[1]); lines = path.read_text(encoding="utf-8").split("\n")
settled = [i for i, l in enumerate(lines) if l.startswith("| 7 |")]
assert len(settled) == 1, settled
lines.pop(settled[0])
open_rows = [i for i, l in enumerate(lines) if l.startswith("| 6 | **zoned")]
assert len(open_rows) == 1, open_rows
lines.insert(open_rows[0] + 1, "| 7 | **根记录的字段表（模型：翻回未定）** | **未定**：模型里人为翻回 |")
path.write_text("\n".join(lines), encoding="utf-8")
PY
python3 - "$WORK/repo" <<'PY'
import importlib.util, io, contextlib, sys
root = sys.argv[1]
spec = importlib.util.spec_from_file_location("relabel", root + "/research/scripts/relabel-item.py")
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
buffer = io.StringIO()
with contextlib.redirect_stdout(buffer):
    code, review = module.relabel(root, "D22", 7, False)
lines = buffer.getvalue().strip().split("\n")
rs_files = [l for l in lines if l.strip().startswith("research/") and ".rs：" in l]
print(f"name=relabel rc={code} research_rs_files_rewritten={len(rs_files)} summary={lines[-1].strip()}")
PY
bash .claude/gate.d/22-item-ref-status.sh >/dev/null 2>&1; echo "name=after stage=22 rc=$?"
out33="$(bash .claude/gate.d/33-mutation-tables.sh 2>&1)"; rc33=$?
echo "name=after stage=33 rc=$rc33 detail=$(grep -m1 'e100_superblock_slot' <<<"$out33" | sed 's/^ *//')"
product="$(awk -F'|' '$1 == "E142" { print $4 }' research/scripts/replay.sh)"
echo "name=e142_product file=$product lines_with_old_label=$(grep -c 'D22已定项7的表与字节表七' "research/results/$product")"
echo "name=e142_source old_literal=$(grep -c '"G4", "根记录字段序：D22已定项7' research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs) new_literal=$(grep -c '"G4", "根记录字段序：D22未定项7' research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs)"

# ---- ② replace-batch 写 kb 之外的文件 ----
old_line="$(grep -m1 '^当前里程碑' CLAUDE.md)"
python3 - "$old_line" > "$WORK/spec.json" <<'PY'
import json, sys
print(json.dumps([{"file": "CLAUDE.md", "old": sys.argv[1], "new": sys.argv[1] + "（模型：书记员写的）"}], ensure_ascii=False))
PY
python3 research/scripts/replace-batch.py "$WORK/spec.json" >/dev/null 2>&1; echo "name=replace_batch target=CLAUDE.md rc=$? written=$(grep -c '（模型：书记员写的）' CLAUDE.md)"

# ---- ③ mutate.sh 在仓根跑 ----
mkdir -p "$WORK/tmp"
mutate_err="$(TMPDIR="$WORK/tmp" MUTATE_TARGET_DIR="$WORK/mutate-target" timeout 300 bash research/scripts/mutate.sh e67-device-subset research/e7-index-bench/src/bin/e67_device_subset.rs research/mutations/e67_device_subset.tsv 2>&1 >/dev/null)"; mutate_rc=$?
echo "name=mutate_from_repo_root rc=$mutate_rc stderr=$mutate_err"
cargo_err="$(CARGO_TARGET_DIR="$WORK/mutate-target" timeout 120 cargo test --release --bin e67-device-subset 2>&1 | head -1)"
echo "name=cargo_from_repo_root first_line=$cargo_err"
# ---- ④ 49 号 --write 碰别的会话写到一半的变更史条目 ----
python3 - <<'PY'
import pathlib
path = pathlib.Path(".claude/kb/decisions-history/2026-09.md")
text = path.read_text(encoding="utf-8")
anchor = "## 历史版本\n\n"
assert text.count(anchor) == 1
foreign = "### 2026-09-17（模型）：别的会话刚写下的条目，快查两行还没补\n\n- **改前**：甲。\n- **改后**：乙。\n- **依据**：丙。\n\n"
path.write_text(text.replace(anchor, anchor + foreign), encoding="utf-8")
PY
bash .claude/gate.d/49-history-brief.sh --write >/dev/null 2>&1; echo "name=history_write rc=$? placeholders_in_foreign_entry=$(grep -A5 "别的会话刚写下的条目" .claude/kb/decisions-history/2026-09.md | grep -c "（待补）")"
echo "name=done"
