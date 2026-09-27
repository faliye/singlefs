#!/usr/bin/env bash
# admission: always 改的是 ask-user-claim-guard.sh 今天那一份的副本，随时可能改
# run-condition: command python3
# defs-m2-closeout-r2 云端攻方 F13：改法 G1（只在我的模型上量过、被攻过零轮）——FILE_AND_LINE 第一支「/ 之后」只认两种：
# 全 ASCII 的文件名（HEAD 那一版原样），或以登记过的扩展名收尾、名字里可带汉字的文件名；不带扩展名的「/汉字：数」不再算文件:行号。
# 拷一份今天的 hook 到临时目录、只换这一处，跑副本自己的 --selftest，再逐条喂 cases-f13.json 里的句子。仓里的 hook 不动。
# 用法（仓根下）：bash research/prompts/defs-m2-closeout-r2-opus-model/f13-fix-g1.sh
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
work="$(mktemp -d "${TMPDIR:-/tmp}/f13-fix-g1.XXXXXX")"
python3 - "$root/.claude/hooks/ask-user-claim-guard.sh" "$work/ask-user-claim-guard.sh" <<'PY'
import sys
source, target = sys.argv[1], sys.argv[2]
text = open(source, encoding="utf-8").read()
old = '    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{FILE_NAME_CHARACTER}+"\n'
new = '    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/(?:{ASCII_PATH_CHARACTER}+|{FILE_NAME_CHARACTER}*\\.(?:{EXTENSION_ALTERNATION}))"\n'
if text.count(old) != 1:
    sys.exit(f"原文命中 {text.count(old)} 次，不是 1 次：今天的 hook 已经不是被判的那一版")
open(target, "x", encoding="utf-8").write(text.replace(old, new))
PY
bash "$work/ask-user-claim-guard.sh" --selftest > "$work/selftest.out" 2>&1
selftest_status=$?
echo "copy selftest exit=$selftest_status: $(grep -o '自检通过（查了 [0-9]* 种）\|✗.*' "$work/selftest.out" | head -3 | tr '\n' ' ')"
python3 - "$work/ask-user-claim-guard.sh" "$here/cases-f13.json" <<'PY'
import json, subprocess, sys
hook, cases_path = sys.argv[1], sys.argv[2]
for case in json.load(open(cases_path, encoding="utf-8")):
    if case["hook"] != "ask":
        continue
    payload = {"tool_name": "AskUserQuestion", "tool_input": {"questions": [{"question": case["command"], "header": "p", "options": [{"label": "a", "description": "a"}]}]}}
    result = subprocess.run(["bash", hook], input=json.dumps(payload), capture_output=True, text=True)
    print(case["id"].replace("-now", "-fixG1") + f"\texit={result.returncode}")
PY
rm -rf -- "${work:?}"
exit "$selftest_status"
