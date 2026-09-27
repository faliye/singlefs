#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 实验产物判决行里的 false 字段有没有被实验页点名
#
# 判据：`research/scripts/replay.sh` 登记表里每一行登记的入库产物（路径在 research/results/ 下），
# 逐行找 `E7RESULT name=verdict ` 开头的行（现查过全仓产物：14 份产物里出现的 `name=verdict` 一律
# 是这个字面形式，没有 `name=verdict_candidate` / `_base` / `_unit` 这几种源码里存在、但从未落进已入库
# 产物的变体——这几种不在本阶段射程里，出现了本阶段也看不见，算已知空白）。
# 判决行里任何 `字段=false`，都要在它对应实验号的实验页里被点名，否则判红。
#
# 实验号 → 实验页：`.claude/kb/experiments/<去掉前导 E 的号>-*.md`（登记表第 1 列去掉前导 `E`，
# 与 kb-discipline.md「编号只能做索引」的登记位是同一份文件名约定）。
#
# 「点名」怎么认（现查过 kb/experiments/*.md 里已有的写法定的）：同一行里字段名**紧跟着**它的值 `false`
# （`字段=false`，字段名与等号、等号与 false 之间可以夹反引号与空白；同一行里别的字段的 false 不算），
# （kb 里一行常常是一整段散文，含标点分隔的若干分句，这里按物理行界定，不切分句），
# 且这一行落在下面两处之一：
#   ① 实验页「## 历史版本」下**最新那一条**历史条目（即 `## 历史版本` 之后第一个 `### ` 到下一个
#      `### `/`## ` 之间的文本）；
#   ② 正文（`## 历史版本` 之前的全部文本）。
# 只认字段名出现在正文的某处不够——正文可能是旧一轮留下的、没跟这一次的 false 一起刷新
# （已知会红的一格 E142 就是这样：正文「判决」一节原样抄着 `control_violations_ok=true` 的旧行，
# 字段名确实在正文里，但那是旧值的旧记录，不是这一次 false 的说明；只有字段名与 `false` 同一行
# 出现才算真的把这次的 false 讲清楚了）。
# 正文里那一行自己标着「第 N 次跑」、而标的次数里没有最新历史条目标题里的那一次，它是上一次跑留下的，不算点名。
# 实验页文件名补零（08-、09-）与不补零两种都认。登记的产物一份都不在 research/results/ 下、或一个 false 字段都没有，
# 本次无对象可判，退 77。
#
# 出路：在实验页那一次跑的历史条目里写明这个字段为什么是 false——是判据如实判出的结论，
# 还是装置或期望值的问题；后者照问题单重跑。
#
# 背景：`records/2026-09-16-subagent拆分提案.md` 第四十节第 33 行。
# 不开豁免：登记产物里的 false 一个都不豁免；今天有几个、点了名的几个，由成功行与汇总行现报，这里不写死。
#
# gate-similar: 40-results-cited.sh 判产物文件本身有没有被 experiments.md 点名，不看产物内部判决字段的值
# gate-similar: 88-quoted-result-lines.sh 判 kb 正文整行抄的 E7RESULT 行是不是逐字见于产物（反方向：产物是不是配得上 kb 已经抄的话），不看某个字段是不是 false
# gate-similar: 87-replay.sh 判产物今天复不复现得出来（逐字节比对），不读产物内容里的判决字段；三道都不判「产物里的 false 有没有被解释」，所以新开一道
#
# 弄坏开关：GATE_VERDICT_FALSE_SKIP_NAMED_CHECK=1 时跳过点名检查（把每个 false 字段都当成已点名），
# 只用来证明这道检查真的在起作用——不是提交时能用的旁路。
#
#   bash .claude/gate.d/84-verdict-false-named.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }

REPLAY=research/scripts/replay.sh
[[ -f "$REPLAY" ]] || { echo "  ! 找不到 $REPLAY，本阶段无对象可判"; exit 77; }

python3 - "$REPLAY" <<'PY'
import glob
import os
import re
import sys

replay_path = sys.argv[1]

ROW_RE = re.compile(r'^(E[0-9]+)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$')
rows = []
with open(replay_path, encoding='utf-8') as handle:
    for line in handle:
        line = line.rstrip('\n')
        m = ROW_RE.match(line)
        if m:
            rows.append((m.group(1), m.group(4)))

# 表格格式变了、一行都读不到，判红，不许当成「0 项，通过」——与门禁 87 号同一条判据
# （show-me-test.md「扫到 0 项也不是通过」）。
if not rows:
    print(f"  ✗ 从 {replay_path} 一行都没读到——登记表的格式变了？")
    print("     → 怎么办：本阶段靠 '编号|二进制|参数|产物|exact/timing' 这个形状取行（与门禁 87 号")
    print("               同一份判据），改表格式要同时改这两道。")
    sys.exit(1)

RESULTS_DIR = 'research/results'
EXP_DIR = '.claude/kb/experiments'
VERDICT_PREFIX = 'E7RESULT name=verdict '
skip_named_check = os.environ.get('GATE_VERDICT_FALSE_SKIP_NAMED_CHECK') == '1'

missing_files = []      # 登记了却在 research/results/ 下找不到文件（多半是已归档，见 87 号「产物已归档」那一档）
checked_files = 0
verdict_lines = 0
false_fields = []       # (exp, product, lineno, field)

for exp, product in rows:
    path = os.path.join(RESULTS_DIR, product)
    if not os.path.isfile(path):
        missing_files.append((exp, product))
        continue
    checked_files += 1
    with open(path, encoding='utf-8', errors='replace') as handle:
        for lineno, raw in enumerate(handle, 1):
            stripped = raw.strip()
            if not stripped.startswith(VERDICT_PREFIX):
                continue
            verdict_lines += 1
            rest = stripped[len(VERDICT_PREFIX):]
            for token in rest.split():
                if '=' not in token:
                    continue
                field, value = token.split('=', 1)
                if value == 'false':
                    false_fields.append((exp, product, lineno, field))


def experiment_page(exp):
    number = int(exp[1:])
    # 实验页文件名有的补零（08-、09-），有的不补：两种都认
    matches = sorted(set(glob.glob(os.path.join(EXP_DIR, f'{number}-*.md'))
                         + glob.glob(os.path.join(EXP_DIR, f'{number:02d}-*.md'))))
    return matches[0] if matches else None


def split_page(text):
    marker = '\n## 历史版本'
    idx = text.find(marker)
    body = text if idx < 0 else text[:idx]
    if idx < 0:
        return body, ''
    tail = text[idx + len(marker):]
    head_match = re.search(r'^### .*$', tail, re.M)
    if not head_match:
        return body, '', ''
    start = head_match.end()
    next_match = re.search(r'^#{2,3} ', tail[start:], re.M)
    end = start + next_match.start() if next_match else len(tail)
    return body, tail[start:end], head_match.group(0)


RUN_TAG = re.compile(r'第([一二三四五六七八九十百零〇两0-9]+)次跑')


def is_named(field, body, latest, latest_heading):
    """返回 (点没点名, 正文里那一行若是上一次跑留下的就交出它)。字段名要紧跟着它自己的 =false，同一行别的字段的 false 不算。"""
    if skip_named_check:
        return True, None
    pattern = re.compile(r'(?<![A-Za-z0-9_])' + re.escape(field) + r'`?\s*[=:：]\s*`?false(?![A-Za-z0-9_])')
    if any(pattern.search(line) for line in latest.split('\n')):
        return True, None
    latest_tags = set(RUN_TAG.findall(latest_heading))
    stale = None
    for line in body.split('\n'):
        if not pattern.search(line):
            continue
        tags = set(RUN_TAG.findall(line))
        if tags and latest_tags and not (tags & latest_tags):
            stale = stale or line.strip()
            continue
        return True, None
    return False, stale


unnamed = []   # (exp, product, lineno, field, reason)
named_count = 0
page_cache = {}
for exp, product, lineno, field in false_fields:
    if exp not in page_cache:
        page_cache[exp] = experiment_page(exp)
    page = page_cache[exp]
    if page is None:
        unnamed.append((exp, product, lineno, field,
                         f'找不到实验页 {EXP_DIR}/{exp[1:]}-*.md'))
        continue
    with open(page, encoding='utf-8') as handle:
        text = handle.read()
    body, latest, latest_heading = split_page(text)
    named, stale = is_named(field, body, latest, latest_heading)
    if named:
        named_count += 1
    elif stale:
        unnamed.append((exp, product, lineno, field,
                         f'{page} 正文里点到它的那一行标的是别的某次跑（「{stale[:40]}」），'
                         f'最新历史条目是「{latest_heading.lstrip("# ").strip()[:30]}」：那是上一次跑留下的，不算点名这一次的 false'))
    else:
        unnamed.append((exp, product, lineno, field,
                         f'{page} 里既不在最新历史条目、也不在正文里被点名'))

print(f'  · 登记了 {len(rows)} 行产物；{checked_files} 份文件在 {RESULTS_DIR} 下找到、'
      f'{len(missing_files)} 份登记了却找不到（多半已归档）')
if missing_files:
    print(f'     没查到文件的登记行（{len(missing_files)} 个）：')
    for exp, product in missing_files:
        print(f'       {exp}  {RESULTS_DIR}/{product}')  # gate-lint:detail
print(f'  · 查到的文件里共 {verdict_lines} 行判决（name=verdict）；{len(false_fields)} 个字段是 false，'
      f'其中 {named_count} 个已点名、{len(unnamed)} 个没点名')

if unnamed:
    print('  ✗ 这些判决字段是 false，对应实验页没有点名：')
    for exp, product, lineno, field, reason in unnamed:
        print(f'     {exp}  {RESULTS_DIR}/{product}:{lineno}  {field}=false  ({reason})')   # gate-lint:detail
    print('     → 怎么办：在实验页那一次跑的历史条目里写明这个字段为什么是 false——是判据如实判出')
    print('               的结论，还是装置或期望值的问题；后者照问题单重跑，把结论一起写回历史条目。')
    sys.exit(1)

if checked_files == 0:
    print(f'  ⊘ 本次无对象可判：登记的 {len(rows)} 行产物一份都不在 {RESULTS_DIR} 下（上面逐个列了）')
    sys.exit(77)
if not false_fields:
    print(f'  ⊘ 本次无对象可判：查到的 {checked_files} 份产物里 {verdict_lines} 行判决，没有一个字段是 false')
    sys.exit(77)
print(f'  ✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 {named_count} 个）')
PY
