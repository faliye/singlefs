#!/usr/bin/env bash
# 模型 M7（gate-shrink-r1 云端攻方）：B1、B2、B3、B8 各造一个违规输入，看「今天判它的那一格」与「方案落地后被指望接手的那一道」各判什么。
# 在草稿目录里拷一份仓（不带 target、.git、research/results、research/prompts），git init 一个空仓，四样违规一起放进去：
#   V1（B1）crates/singlefs-core/src/instance_table.rs 末尾加一个用 rows.retain( 的函数（C333 那一笔的前置进来了）
#        → 43 号 --check row27-preconditions；另 grep 全部门禁脚本谁读这个文件
#   V2（B2）.claude/settings.json 里 write-guard 的 matcher 从 Write|Edit 收成 Write；heavy-test-guard 从 Bash 那一条挪到 matcher Edit；
#        session-start 的 matcher 从 startup|resume|compact 收成 startup
#        → 共享 hooks-registered.sh 与 47 号 --check agent-write-scope
#   V3（B3）checks-owed.md 已还清那张表末尾再登记一次 C333，编号后面是全角空格
#        → 43 号 --check table-shape 与共享 doc-lint.sh（看输出里有没有 C333）
#   V8（B8）.claude/rules/verification.md 加一行引 E996（臆造的实验）；.claude/kb/pitfalls.md 历史版本之前加一行，E996 与 D89 各引两次
#        → 10 号 --check experiment-refs、decision-refs 与共享 doc-lint.sh、number-name-sync.sh（看输出里有没有 E996 / D89）
# 编号取 E996、D89：本仓 .claude、CLAUDE.md、records、research/scripts 里一处都没有（门禁样本里有 E999，会替 doc-lint 的 H 凑够三次）。
# 日志里可能有 NEL 这类字节，grep 一律带 -a 当文本读。
# 用法：bash m7_copy_injections.sh <仓根> <草稿目录>
set -uo pipefail
repository_root="$(cd "${1:?仓根}" && pwd)"
scratch="${2:?草稿目录}"
copy="$scratch/m7-copy"
rm -rf "${copy:?}"
rsync -a --exclude target --exclude .git --exclude research/results --exclude research/prompts "$repository_root/" "$copy/"
git -C "$copy" init -q
cd "$copy" || exit 2
before_row27=0; env -u GATE_NOT_RUN_FILE bash .claude/gate.d/43-checks-owed-and-closeout.sh . --check row27-preconditions > "$scratch/m7-v1-before.log" 2>&1 || before_row27=$?
printf '\n// 样本：行回收的前置进来了\nfn reclaim_rows_sample(rows: &mut Vec<u64>) { rows.retain(|row| *row != 0); }\n' >> crates/singlefs-core/src/instance_table.rs
python3 - <<'PY'
import json
path = ".claude/settings.json"; settings = json.load(open(path, encoding="utf-8"))
pre = settings["hooks"]["PreToolUse"]; moved = []
for entry in pre:
    if entry.get("matcher") == "Write|Edit":
        entry["matcher"] = "Write"
    if entry.get("matcher") == "Bash":
        moved = [hook for hook in entry["hooks"] if "heavy-test-guard.sh" in hook["command"]]
        entry["hooks"] = [hook for hook in entry["hooks"] if "heavy-test-guard.sh" not in hook["command"]]
pre.append({"matcher": "Edit", "hooks": moved})
for entry in settings["hooks"]["SessionStart"]:
    entry["matcher"] = "startup"
json.dump(settings, open(path, "w", encoding="utf-8"), ensure_ascii=False, indent=2)
owed = ".claude/kb/checks-owed.md"; lines = open(owed, encoding="utf-8").read().split("\n")
paid_start = max(index for index, line in enumerate(lines) if line.startswith("<!-- doc-lint:registry"))
history = next(index for index, line in enumerate(lines) if line.startswith("## 历史版本"))
last_row = max(index for index in range(paid_start, history) if lines[index].startswith("| C"))
lines.insert(last_row + 1, "| C333　| 删行那次发布被重放 | 样本：同一编号在已还清表里再登记一次，编号后面是全角空格 | 2026-09-28 |")
open(owed, "w", encoding="utf-8").write("\n".join(lines))
rule = ".claude/rules/verification.md"
open(rule, "a", encoding="utf-8").write("\n- 样本：E996（臆造的实验） 的产物当依据。\n")
kb = ".claude/kb/pitfalls.md"; text = open(kb, encoding="utf-8").read(); cut = text.index("\n## 历史版本")
open(kb, "w", encoding="utf-8").write(text[:cut] + "\n\n- 样本：E996（臆造的实验） 与 D89（臆造的决策） 各引两次：E996（臆造的实验）、D89（臆造的决策）。\n" + text[cut:])
PY
run() { local name="$1"; shift; local code=0; env -u GATE_NOT_RUN_FILE "$@" > "$scratch/m7-$name.log" 2>&1 || code=$?; echo "$name rc=$code"; }
echo "v1-before rc=$before_row27"
run v1-row27 bash .claude/gate.d/43-checks-owed-and-closeout.sh . --check row27-preconditions
echo "v1 其他读 instance_table.rs 的门禁脚本：$(grep -rlF 'instance_table.rs' "$repository_root"/.claude/gate.d/*.sh "$repository_root"/.claude/gate.d/*.py | xargs -n1 basename | tr '\n' ' ')"
run v2-hooks-registered bash .claude/singlefs-ai-sop/scripts/hooks-registered.sh .
run v2-agent-write-scope bash .claude/gate.d/47-code-tooling-selftests-and-registries.sh . --check agent-write-scope
run v3-table-shape bash .claude/gate.d/43-checks-owed-and-closeout.sh . --check table-shape
run v8-experiment-refs bash .claude/gate.d/10-references-and-invariants.sh . --check experiment-refs
run v8-decision-refs bash .claude/gate.d/10-references-and-invariants.sh . --check decision-refs
run v8-number-name-sync bash .claude/singlefs-ai-sop/scripts/number-name-sync.sh .
run v3-v8-doc-lint bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .
echo "doc-lint 输出里点名 C333 的行（F 报「多处登记位」，G 报「裸引用」）："; grep -a 'C333' "$scratch/m7-v3-v8-doc-lint.log" | cut -c1-120
echo "doc-lint 输出里 E996 或 D89 出现 $(grep -acE 'E996|D89' "$scratch/m7-v3-v8-doc-lint.log") 行"
echo "number-name-sync 输出里 E996 或 D89 出现 $(grep -acE 'E996|D89' "$scratch/m7-v8-number-name-sync.log") 行"
echo "hooks-registered 末行：$(tail -1 "$scratch/m7-v2-hooks-registered.log")"
echo "agent-write-scope 的 ✗ 行："; grep '✗' "$scratch/m7-v2-agent-write-scope.log"
echo "table-shape 的 ✗ 行："; grep -A1 '✗ 欠账表' "$scratch/m7-v3-table-shape.log"
cd "$scratch" && rm -rf "${copy:?}"
