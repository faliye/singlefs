#!/usr/bin/env bash
# Q6：两轮判决、三份正文与调度记录里这一题引的计数，逐个现算。每行：编号<TAB>现算的东西<TAB>结果。
# 用法：q6_counts.sh <仓根> <第三轮冻结副本> <工作根> <草稿目录>
# 只读：冻结副本、集成副本产物、两轮辩方/攻方模型目录；写只落在 <草稿目录>。
set -uo pipefail
REPO="$1"; R3="$2"; WORKROOT="$3"; DRAFT="$4"
R1="${R1_FROZEN:-$WORKROOT/m3-defs-b9-r1/repo}"; R2="${R2_FROZEN:-$WORKROOT/m3-defs-b9-r2/repo}"
INTEGRATION="${INTEGRATION:-$WORKROOT/m3-tooling-b2-integration}"
R2OUT="${R2_MAIN_OUT:-$WORKROOT/m3-defs-b9-r2-main-out}"; R1OUT="${R1_MAIN_OUT:-$WORKROOT/m3-defs-b9-r1-main-out}"
PROMPTS="$REPO/research/prompts"; H="$REPO/scripts/agents/cite-check.py"
export PYTHONDONTWRITEBYTECODE=1
emit() { printf '%s\t%s\t%s\n' "$1" "$2" "$3"; }
count_not_ok() { grep -vc ': OK$' || true; }

# ── 快照与副本 ──
for r in 1 2 3; do
  frozen="$WORKROOT/m3-defs-b9-r$r/repo"
  emit "A-r$r-frozen" "在 r$r 冻结副本里 sha256sum -c 快照 frozen-copy-sha256.txt：行数 / OK / 非 OK" \
    "$(wc -l < "$PROMPTS/m3-defs-b9-r$r-snapshot/frozen-copy-sha256.txt") / $(cd "$frozen" && sha256sum -c "$PROMPTS/m3-defs-b9-r$r-snapshot/frozen-copy-sha256.txt" 2>&1 | grep -c ': OK$') / $(cd "$frozen" && sha256sum -c "$PROMPTS/m3-defs-b9-r$r-snapshot/frozen-copy-sha256.txt" 2>&1 | grep -vc ': OK$')"
  emit "A-r$r-kb" "在主工作区 sha256sum -c 快照 kb-sha256.txt：行数 / 非 OK（FAILED 的文件名）" \
    "$(wc -l < "$PROMPTS/m3-defs-b9-r$r-snapshot/kb-sha256.txt") / $(cd "$REPO" && sha256sum -c "$PROMPTS/m3-defs-b9-r$r-snapshot/kb-sha256.txt" 2>&1 | grep -c ': FAILED$') ($(cd "$REPO" && sha256sum -c "$PROMPTS/m3-defs-b9-r$r-snapshot/kb-sha256.txt" 2>&1 | grep ': FAILED$' | sed 's/: FAILED$//' | xargs -r -n1 basename | tr '\n' ' '))"
done
emit "A-r1-quote" "r1 两份腿报告里带 term-renames.md 或 171-三oracle合谋测试.md 加「:数字」或「@」的写法的行数" \
  "$(cat "$PROMPTS/m3-defs-b9-r1-sonnet-output.md" "$PROMPTS/m3-defs-b9-r1-opus-output.md" | grep -cE 'term-renames\.md(:[0-9]|@)|171-三oracle合谋测试\.md(:[0-9]|@)')"
emit "A-b6-snapshot" "上一题第三轮快照 m3-defs-b6-r3-snapshot 行数" "$(wc -l < "$PROMPTS/m3-defs-b6-r3-snapshot/frozen-copy-sha256.txt")"
emit "A-r2r3-paths" "r3 快照对 r2 快照：路径集合相同 / 哈希不同的行数" \
  "$(diff <(awk '{print $2}' "$PROMPTS/m3-defs-b9-r2-snapshot/frozen-copy-sha256.txt"|sort) <(awk '{print $2}' "$PROMPTS/m3-defs-b9-r3-snapshot/frozen-copy-sha256.txt"|sort) >/dev/null && echo 路径集合相同 || echo 路径集合不同) / $(diff "$PROMPTS/m3-defs-b9-r2-snapshot/frozen-copy-sha256.txt" "$PROMPTS/m3-defs-b9-r3-snapshot/frozen-copy-sha256.txt" | grep -c '^>')"

# ── 引文核对（cite-check）──
cc() { (cd "$REPO" && nice -n 19 python3 "$H" "$@" 2>&1 | tail -n 1 | sed 's/^ *✓ //'); }
R1S="$PROMPTS/m3-defs-b9-r1-sonnet-output.md"; R1O="$PROMPTS/m3-defs-b9-r1-opus-output.md"; R2S="$PROMPTS/m3-defs-b9-r2-sonnet-output.md"; R2O="$PROMPTS/m3-defs-b9-r2-opus-output.md"
emit "B-r1-cite-frozen-only" "cite-check r1 两份报告 --root <r1 冻结副本> --background r1 背景（判决 :13 写的命令）" "$(cc "$R1S" "$R1O" --root "$R1" --background "$PROMPTS/_m3-defs-b9-r1-background.md")"
emit "B-r1-cite-two-roots" "同上，--root 加上仓根" "$(cc "$R1S" "$R1O" --root "$R1" --root "$REPO" --background "$PROMPTS/_m3-defs-b9-r1-background.md")"
emit "B-r1-cite-sonnet" "cite-check 正推 r1 报告 --root <r1 冻结副本>" "$(cc "$R1S" --root "$R1")"
emit "B-r1-cite-sonnet-main" "cite-check 正推 r1 报告 --root 仓根（腿报告末行写 60/60 时的写法）：判红的处数（行号超出被引文件）" "$(cd "$REPO" && nice -n 19 python3 "$H" "$R1S" --root "$REPO" 2>&1 | grep -c '^  ✗ ')"
emit "B-r1-cite-opus" "cite-check 攻方 r1 报告 --root 仓根" "$(cc "$R1O" --root "$REPO")"
emit "B-r2-cite-both" "cite-check r2 两份报告 --root 仓根 --background r2 背景（判决 :13 写的命令）" "$(cc "$R2S" "$R2O" --root "$REPO" --background "$PROMPTS/_m3-defs-b9-r2-background.md")"
emit "B-r2-cite-sonnet" "cite-check 辩方 r2 报告 --root 仓根" "$(cc "$R2S" --root "$REPO")"
emit "B-r2-cite-opus" "cite-check 攻方 r2 报告 --root 仓根" "$(cc "$R2O" --root "$REPO")"

# ── 腿的报告和模型目录 ──
emit "B-report-sha" "四份腿报告 sha256 的头 8 位与末 4 位（r1 正推 / r1 攻 / r2 辩 / r2 攻）" \
  "$(for f in "$R1S" "$R1O" "$R2S" "$R2O"; do sha256sum "$f" | awk '{print substr($1,1,8)"…"substr($1,61,4)}'; done | tr '\n' ' ')"
emit "B-model-files" "模型目录文件数 / out 文件数（r1 正推 r1 攻 r2 辩 r2 攻）" \
  "$(for m in m3-defs-b9-r1-sonnet-model m3-defs-b9-r1-opus-model m3-defs-b9-r2-sonnet-model m3-defs-b9-r2-opus-model; do echo -n "$(find "$PROMPTS/$m" -type f | wc -l)/$(find "$PROMPTS/$m/out" -type f | wc -l) "; done)"
emit "B-r1-sonnet-rows" "r1 正推报告句表行数（Z1 Z3 Z4 Z6 Z7 / 合计）/ 判不一致的行" \
  "$(for z in Z1 Z3 Z4 Z6 Z7; do echo -n "$z=$(grep -cE "^\| $z-[0-9]+ \|" "$R1S") "; done)/ $(grep -cE '^\| Z[0-9]-[0-9]+ \|' "$R1S") / $(grep -E '^\| Z[0-9]-[0-9]+ \|' "$R1S" | awk -F'|' '$4 ~ /^ *不一致/ {print $2}' | tr -d ' ' | tr '\n' ' ')"
emit "B-r2-sonnet-fixed" "r2 辩方总表里判定以「修到」开头的行数" \
  "$(sed -n '/^## 总表/,/^## 第一轮判决的判重与判轻/p' "$R2S" | grep -cE '^\| .* \| 修到')"

# ── r1 正推模型的 main 复跑日志（判决 :14 说「全 same」）──
if [[ -f "$R1OUT/rerun.log" ]]; then
  emit "B-r1-main-rerun-log" "主 agent 复跑日志 rerun.log：逐行逐行相同 / 不同的名字" \
    "$(grep -c '逐行相同$' "$R1OUT/rerun.log") / $(grep -c '不同或' "$R1OUT/rerun.log") ($(grep '不同或' "$R1OUT/rerun.log" | sed 's/ lines=.*//;s/name=//' | tr '\n' ' '))"
fi

# ── 样本档与复核汇总 ──
for r in 1 2 3; do
  emit "C-r$r-fixture-dirs" "r$r 冻结副本 fixtures 下样本目录数（code-tooling / doc-process-records / doc-text）" \
    "$(for st in code-tooling.sh doc-process-records.sh doc-text.sh; do echo -n "$(ls "$WORKROOT/m3-defs-b9-r$r/repo/.claude/gate.d/fixtures/$st" | wc -l) "; done)"
done
emit "C-r2-main-out-stage" "r2 主 agent 样本档日志末行的用例数（code-tooling / doc-process-records / doc-text）" \
  "$(for st in code-tooling.sh doc-process-records.sh doc-text.sh; do tail -n 1 "$R2OUT/log/s-stage-$st.log" | grep -oE '[0-9]+ 个用例' | tr '\n' ' '; done)"
if [[ -d "$R1OUT" ]]; then
  emit "C-r1-main-out-stage" "r1 主 agent 样本档日志末行的用例数（code-tooling / doc-process-records）" \
    "$(for f in 1-code-tooling 2-dpr; do tail -n 1 "$R1OUT/$f.log" | grep -oE '[0-9]+ 个用例' | tr '\n' ' '; done)"
fi
emit "C-verify-stage" "verify13 / verify14 的 stage-selftest 日志用例数（code-tooling doc-text doc-process-records）" \
  "$(for n in 13 14; do echo -n "v$n: "; for k in 011-stage-code-tooling.sh 012-stage-doc-text.sh 014-stage-doc-process-records.sh; do grep -oE '[0-9]+ 个用例' "$INTEGRATION/verify$n-out/$k.log" | tr '\n' ' '; done; done)"
emit "C-fixture-diff" "r1→r2 与 r2→r3 新加 / 改过的样本目录（三道样本档合计）" \
  "$(for pair in '1 2' '2 3'; do set -- $pair; a=0; m=0; for st in code-tooling.sh doc-process-records.sh doc-text.sh; do A="$WORKROOT/m3-defs-b9-r$1/repo/.claude/gate.d/fixtures/$st"; B="$WORKROOT/m3-defs-b9-r$2/repo/.claude/gate.d/fixtures/$st"; a=$((a+$(comm -13 <(ls "$A") <(ls "$B") | wc -l))); for d in $(comm -12 <(ls "$A") <(ls "$B")); do diff -rq "$A/$d" "$B/$d" >/dev/null 2>&1 || m=$((m+1)); done; done; echo -n "r$1→r$2 新加 $a 改过 $m; "; done)"
for n in 12 13 14; do
  s="$INTEGRATION/verify$n.summary"
  emit "D-verify$n" "verify$n.summary：编号行数 / .rc 文件数 / 含「错（」的行数 / 含「 对」的行 / 含「 记」的行 / 末行" \
    "$(grep -cE '^ *[0-9]+ ' "$s") / $(ls "$INTEGRATION/verify$n-out/"*.rc | wc -l) / $(grep -c '错（' "$s") / $(grep -cE ' 对$' "$s") / $(grep -cE ' 记$' "$s") / $(tail -n 1 "$s")"
  emit "D-verify$n-errors" "verify$n 判错的项（编号 名字）" "$(grep '错（' "$s" | awk '{print $1" "$2}' | tr '\n' ';')"
done
emit "D-verify12-dispatch" "verify12/13/14 派发闸自证「查了 N 种」" \
  "$(for n in 12 13 14; do grep -h '自检通过（查了' "$INTEGRATION/verify$n-out/"*self-runner-dispatch-guard*.log | grep -oE '查了 [0-9]+ 种' | tr '\n' ' '; done)"
emit "D-verify13-doc-text" "verify13 cell-doc-text-four 红行：钟点行数 / 文件数 / 豁免表指不到条数" \
  "$(grep -oE '✗ [0-9]+ 行在描述里写了钟点、时区词或时间戳，在 [0-9]+ 份文件里' "$INTEGRATION"/verify13-out/*cell-doc-text-four.log | grep -oE '[0-9]+' | tr '\n' ' ') / $(grep -oE '豁免登记表里有 [0-9]+ 条指不到文件' "$INTEGRATION"/verify13-out/*cell-doc-text-four.log | grep -oE '[0-9]+')"
emit "D-verify14-fixture-claims" "verify14 cell-fixture-claims：缺红格的门禁数 / 文件名日期条数 / 样本正文日期处数" \
  "$(grep -oE '✗ [0-9]+ (道门禁有格缺红或缺绿样本|个文件名里的日期不可能是真的|处样本正文里的日期不可能是真的)' "$INTEGRATION"/verify14-out/*cell-fixture-claims.log | grep -oE '[0-9]+' | tr '\n' ' ')"
python3 - "$INTEGRATION" "$WORKROOT" "$DRAFT" "$REPO" <<'PY' | while IFS=$'\t' read -r a b c; do emit "$a" "$b" "$c"; done
import subprocess, sys, os, glob, re
integration, workroot, draft = sys.argv[1:4]
repo = os.path.join(integration, "repo")
names = subprocess.run(["git", "diff", "--name-only", "after-b10-fixup", "after-b11"], cwd=repo, capture_output=True, text=True).stdout.split()
samples = sorted({"/".join(n.split("/")[3:5]) for n in names if ".claude/gate.d/fixtures/" in n})
log = open(glob.glob(os.path.join(integration, "verify14-out", "*cell-fixture-claims.log"))[0], encoding="utf-8").read()
status = {}
for line in subprocess.run(["git", "diff", "--name-status", "after-b10-fixup", "after-b11", "--", ".claude/gate.d/fixtures"], cwd=repo, capture_output=True, text=True).stdout.splitlines():
    s, p = line.split("\t"); d = "/".join(p.split("/")[3:5]); status.setdefault(d, set()).add(s)
existing = [d for d in samples if subprocess.run(["git", "cat-file", "-e", "after-b10-fixup:.claude/gate.d/fixtures/" + d], cwd=repo, capture_output=True).returncode == 0]
print("D-b11-samples\t批 11 改动涉及的样本目录数；其中 after-b10-fixup 里没有的（新建）/ 已有的（改过）\t%d；新建 %d、已有 %d（%s）" % (len(samples), len(samples) - len(existing), len(existing), " ".join(existing)))
print("D-b11-samples-in-red\t这些样本目录在 verify14 cell-fixture-claims 日志里出现的次数（目录: 次数）\t%s" % "; ".join("%s: %d" % (d, log.count(d)) for d in samples))
snapshot = {l.split()[1] for l in open(os.path.join(sys.argv[4], "research/prompts/m3-defs-b9-r3-snapshot/frozen-copy-sha256.txt"), encoding="utf-8") if l.strip()}
non = [n for n in names if ".claude/gate.d/fixtures/" not in n]
print("D-b11-nonsample\tb11 非样本改动文件数 / 其中不在 r3 快照里的\t%d / %d" % (len(non), len([n for n in non if n not in snapshot])))
PY


# ── 行号类引用（r1 判决 :17、r2 判决 :22）与 r1 主 agent 门禁格日志 ──
emit "H-r1-lines" "r1 冻结副本：dispatch-prompt.py:43 含「书记员」/ :90-91 含 LONG_RUN_TYPES / runner-dispatch-guard.sh:39 与 doc-process-records.sh:103 含「不按 -r<数字> 形」" \
  "$(sed -n 43p "$R1/scripts/agents/dispatch-prompt.py" | grep -c 书记员) / $(sed -n 90,91p "$R1/scripts/agents/dispatch-prompt.py" | grep -c LONG_RUN_TYPES) / $(sed -n 39p "$R1/.claude/hooks/runner-dispatch-guard.sh" | grep -c '不按 -r<数字> 形') / $(sed -n 103p "$R1/.claude/gate.d/doc-process-records.sh" | grep -c '不按 -r<数字> 形')"
emit "H-r2-registry" "r2 冻结副本：叫停登记里 m3-batch63-code-r1 在第几行 / 出处写第几行 / 记录第 1594、1595 行各含轮名" \
  "$(grep -n 'm3-batch63-code-r1' "$R2/research/prompts/abandoned-rounds.tsv" | cut -d: -f1) / $(grep 'm3-batch63-code-r1' "$R2/research/prompts/abandoned-rounds.tsv" | grep -oE '第 [0-9]+ 行' | head -1) / $(sed -n 1594p "$R2/records/2026-10-01-删除与并发事务.md" | grep -c m3-batch63-code-r1) $(sed -n 1595p "$R2/records/2026-10-01-删除与并发事务.md" | grep -c m3-batch63-code-r1)"
if [[ -d "$R1OUT/attack" ]]; then
  A="$R1OUT/attack"
  emit "H-r1-attack-logs" "r1 主 agent 攻方门禁格日志：abandoned-after b9 红且点名「命中 2 次」/ r3 与对照绿 / htd 插一处 改一行 绿 / htd-shell 点名处数与 var_from_registry.sh 出现次数 / exempt 有表 红 没表 绿" \
    "$(grep -c '命中 2 次' "$A/1-abandoned-b9.log") / $(grep -c '^  ✓ doc-process-records.sh' "$A/1-abandoned-r3.log") $(grep -c '^  ✓ doc-process-records.sh' "$A/1-control-G.log") / $(grep -c '^  ✓ code-tooling.sh' "$A/2-htd-insert.log") $(grep -c '^  ✓ code-tooling.sh' "$A/2-htd-edit.log") / $(grep -oE '[0-9]+ 处自己定' "$A/2-htd-shell.log" | head -1), $(grep -c var_from_registry "$A/2-htd-shell.log") / $(grep -c '^  ✗ code-tooling.sh' "$A/4-exempt-base.log") $(grep -c '^  ✓ code-tooling.sh' "$A/4-exempt-nobase.log")"
fi

# ── 补丁规模 ──
for b in b9 b10 b11; do
  d="$WORKROOT/m3-tooling-$b/patch.diff"
  emit "E-patch-$b" "$b patch.diff：文件数 / 加 / 删 / sha256 头 8 位 / report.md sha 头 8 末 4" \
    "$(grep -c '^diff --git' "$d") / $(grep -E '^\+' "$d" | grep -vc '^+++') / $(grep -E '^-' "$d" | grep -vc '^---') / $(sha256sum "$d" | cut -c1-8) / $(sha256sum "$WORKROOT/m3-tooling-$b/report.md" | awk '{print substr($1,1,8)"…"substr($1,61,4)}')"
done
emit "E-checklist" "三份清单：抄行数 / 不抄行数 / kb 清单标题数（r1 r2 r3）" \
  "$(for r in 1 2 3; do f="$PROMPTS/_m3-defs-b9-r$r-checklist.md"; echo -n "$(grep -cE '\| 抄 \|' "$f")/$(grep -cE '\| 不抄' "$f")/$(grep -c '^### 小节清单' "$f") "; done)"

# ── r2 主 agent 门禁格日志里的点名数 ──
L="$R2OUT/log"
emit "F-htd-named" "r2 主 agent 日志：htd-s3 r2 / htd-s3 r1 / s3-shell（处，文件数）/ htd-shell（处，var_from_registry 出现次数）" \
  "$(for f in a-htd-s3-r2 a-htd-s3-r1; do grep -oE '[0-9]+ 处自己定' "$L/$f.log" | head -1; done | tr '\n' ' ') / $(grep -oE '[0-9]+ 处自己定' "$L/s-s3-shell.log" | head -1), $(sed -n '3,40p' "$L/s-s3-shell.log" | grep -oE 'scripts/experiments/[a-z0-9_]+\.sh' | sort -u | wc -l) 份, 点名行 $(grep -cE '^     scripts/[^ ]+:[0-9]+  ' "$L/s-s3-shell.log") / $(grep -oE '[0-9]+ 处自己定' "$L/o-htd-shell.log" | head -1), $(grep -c var_from_registry "$L/o-htd-shell.log")"
emit "F-htd-insert-edit" "r2 主 agent 日志：htd-insert / htd-edit 绿行里「扫了 N 份」" \
  "$(for f in o-htd-insert o-htd-edit; do grep -oE '扫了 [0-9]+ 份' "$L/$f.log" | head -1; done | tr '\n' ' ')"
emit "F-abandoned" "r2 主 agent 日志：abandoned 各树绿(0)/红(1)（misread r2 r1、after、s5 四棵 r2、四棵 r1）" \
  "$(for f in a-misread-r2 a-misread-r1 o-abandoned-after s-s5-append-note-r2 s-s5-reworded-r2 s-s5-insert-then-reword-r2 s-s5-insert-then-append-r2 s-s5-append-note-r1 s-s5-reworded-r1 s-s5-insert-then-reword-r1 s-s5-insert-then-append-r1; do if grep -q '^  ✓ doc-process-records.sh' "$L/$f.log"; then echo -n "0 "; else echo -n "1 "; fi; done)"
emit "F-misread-hint" "r2 主 agent 日志 a-misread-r2：「照行号提示取那一行照收」所在行的出处行号" "$(grep -oE '第 [0-9]+ 行：' "$L/a-misread-r2.log" | head -1)"
emit "F-doc-text-leak" "r2 主 agent 三份 doc-text 日志里编的私有值整串/半截的命中份数" \
  "$(grep -lE 'sample-second-host-77|/srv/sample-private/second-checkout|host-77|-checkout[^〕]|demo-peer-host-42|demo-private' "$L"/s-s6-*.log "$L"/o-z5.log 2>/dev/null | wc -l)"
emit "F-r1-z5-lines" "r1 主 agent z5 日志里含编的私有值（8 字窗口）的行数" \
  "$(python3 -I - "$PROMPTS/m3-defs-b9-r1-opus-model/feeds/z5" "$R1OUT/attack/3-z5.log" <<'PY'
import re, sys
base, log = sys.argv[1:3]
vals = set()
for fn in ("multi-host.env", ".private-values-hostname"):
    for l in open(base + "/" + fn, encoding="utf-8"):
        l = l.strip()
        if not l or l.startswith("#"): continue
        v = l.split("=", 1)[1] if "=" in l else l
        for w in re.split(r"\s+", v.strip().strip("\"'")):
            if len(w) >= 8: vals.add(w)
n = 0
for l in open(log, encoding="utf-8"):
    if any(w[a:a+8] in l for w in vals for a in range(0, len(w) - 7)): n += 1
print(n)
PY
)"
emit "F-s2-rows" "辩方 s2-exit-matrix.txt 形态行数（b9 列与 b10 列相同的行数）" \
  "$(tail -n +2 "$PROMPTS/m3-defs-b9-r2-sonnet-model/out/s2-exit-matrix.txt" | awk -F'\t' 'NF>=3 && $1 !~ /^##/ && $1 !~ /^→/' | wc -l) ($(tail -n +2 "$PROMPTS/m3-defs-b9-r2-sonnet-model/out/s2-exit-matrix.txt" | awk -F'\t' 'NF>=3 && $2==$3' | wc -l))"
emit "F-b11-htd" "verify14 b11 门禁日志：htd-s3 / s3-shell / htd-shell 的「N 处」；s5 四棵（append-note reworded insert-then-reword insert-then-append）的退出码" \
  "$(for k in 234-b11-htd-s3 240-b11-s3-shell 242-b11-htd-shell; do grep -oE '[0-9]+ 处自己定' "$INTEGRATION/verify14-out/$k.log" | head -1; done | tr '\n' ' ') / $(for k in 236 237 238 239; do awk '{print $1}' "$INTEGRATION/verify14-out/$k.rc"; done | tr '\n' ' ')"

# ── 语料数 ──
emit "G-htd-trees" "heavy-thread-defaults 在 b6-r1 / b6-r2 / b6-r3 / b9-r1 / b9-r2 / b9-r3 冻结副本上的 扫N份/可疑M处/表K行/自读变量V个" \
  "$(for d in m3-defs-b6-r1 m3-defs-b6-r2 m3-defs-b6-r3 m3-defs-b9-r1 m3-defs-b9-r2 m3-defs-b9-r3; do F="$WORKROOT/$d/repo"; python3 -B "$PROMPTS/m3-defs-b9-r1-opus-model/tools/z2_htd_cell.py" "$F" "$F" 2>&1 | grep -oE '扫了 [0-9]+ 份|可疑写法 [0-9]+ 处|表 [0-9]+ 行|读的线程变量 [0-9]+ 个' | tr '\n' ' '; echo -n '| '; done)"
for r in 1 3; do
  F="$WORKROOT/m3-defs-b9-r$r/repo"
  emit "G-r$r-anchors" "r$r 冻结副本排除表每行锚串命中被判行的条数分布（命中 N 条的行数）" \
    "$(HTD_ANCHOR_DUMP=1 python3 -B "$PROMPTS/m3-defs-b9-r1-opus-model/tools/z2_htd_cell.py" "$F" "$F" 2>&1 | awk -F'\t' '/^ANCHOR/{split($5,a," "); c[a[2]]++; t++} END{printf "共 %d 行：", t; for(k=1;k<=9;k++) if(c[k]) printf "%d条×%d行 ", k, c[k]}')"
done
emit "G-round-names" "r1 / r3 冻结副本：目录三种文件名认出的轮名数 / 并上登记表第一列 / 以 -r<数字> 结尾 / 含 a-z0-9- 以外字符" \
  "$(for r in 1 3; do python3 -I - "$WORKROOT/m3-defs-b9-r$r/repo" <<'PY'
import sys, importlib.util, re, os
F = sys.argv[1]
spec = importlib.util.spec_from_file_location("lib", F + "/.claude/hooks/lib_abandoned_rounds.py"); m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
rows, _ = m.read_rows(F)
d = set()
for e in os.listdir(F + "/research/prompts"):
    for form in m.ROUND_MATERIAL_FORMS:
        f = form.match(e)
        if f: d.add(f.group(1)); break
u = m.recognised_round_names(F, rows)
print("r%s: %d / %d / %d / %d; " % (F.split("-r")[-1].split("/")[0], len(d), len(u), sum(1 for n in u if re.search(r"-r[0-9]+$", n)), sum(1 for n in u if re.search(r"[^a-z0-9-]", n))), end="")
PY
done)"
emit "G-labels-195" "上一题辩方 out/fm-*.txt 行数" "$(cat "$PROMPTS"/m3-defs-b6-r2-sonnet-model/out/fm-*.txt | grep -c .)"
emit "G-gate-stages" "r2 / r3 冻结副本 .claude/gate.d/*.sh 个数 / 调打码库（shown_path 等）的个数" \
  "$(for r in 2 3; do d="$WORKROOT/m3-defs-b9-r$r/repo/.claude/gate.d"; echo -n "$(ls "$d"/*.sh | wc -l) / $(grep -l 'shown_path\|PRIVATE_VALUES_LIBRARY\|load_private_targets' "$d"/*.sh | wc -l) ; "; done)"
emit "G-real-records" "r2 冻结副本 records/*.md：份数 / 恰有一个 <名>-r<数字> 轮名且带 6 字以上「」的行数 / 份数 / (文件,轮名) 两行以上的对数" \
  "$(cd "$R2" && python3 -I - <<'PY'
import re, glob, collections
name = re.compile(r'(?<![A-Za-z0-9_\-])([a-z0-9][a-z0-9-]*-r[0-9]+)(?![A-Za-z0-9_\-])'); quote = re.compile(r'「([^」]*)」')
lines = 0; files = set(); per = collections.Counter()
for f in sorted(glob.glob('records/*.md')):
    for l in open(f, encoding='utf-8'):
        ns = set(name.findall(l))
        if len(ns) != 1: continue
        if not any(len(re.sub(r'\s', '', q)) >= 6 for q in quote.findall(l)): continue
        lines += 1; files.add(f); per[(f, next(iter(ns)))] += 1
print(len(glob.glob('records/*.md')), "/", lines, "/", len(files), "/", sum(1 for v in per.values() if v >= 2))
PY
)"
emit "G-prompts-now" "工作根下 */prompt.md 现有份数 / 其中含围栏行（行首至多 3 格的三个反引号或三个波浪线）的份数" \
  "$(ls "$WORKROOT"/*/prompt.md | wc -l) / $(grep -lE '^ {0,3}(```|~~~)' "$WORKROOT"/*/prompt.md 2>/dev/null | wc -l)"
