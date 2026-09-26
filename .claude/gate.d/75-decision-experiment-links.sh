#!/usr/bin/env bash
# gate-stage: 决策与实验双向登记，实验或结论变了之后回看过决策
#
# 规则在 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。判据：
#   ① 实验页（待回填清单之外的）有「### 影响的决策」一节，表头 | 决策分项 | 关系 | 回看 |；
#      实验正文（历史版本与这一节之外）提到的每条决策（`D<号>（`）都要有一行。
#   ② 表里每一行：决策真实存在，写了分项的那条分项也存在；关系是 支撑 / 推翻 / 备料 / 不影响 之一；
#      回看格以日期开头，后跟「改了」或「不受影响：理由」。
#   ③ 双向：关系是支撑或推翻的那条分项，它的「**依据**」段要引回这个实验；反过来，
#      分项「**依据**」段引的每个实验，那个实验页里要有这一行、关系是支撑或推翻。待回填清单里的决策不判这一条。
#   ④ 时效（全量）：实验最近一次变动的日期——页内历史节带日期的 ### 标题、experiments-history.md 里标题
#      （标题没点名实验时看条目正文）点名它的条目——晚于表里某一行的回看日期，判红。
#   ⑤ 按这次改动（GATE_BASE / @{upstream} 的 merge-base / HEAD 起，工作区、暂存区、未跟踪文件都算）：
#      实验页正文（历史版本与影响的决策两节之外）改了、或 research/results/e<号>… 的产物变了，
#      这次改动要在那张表里改过至少一行；新加的三方判决 research/prompts/*-main-verification.md
#      要有「## 回看决策」一节（同样的表，或一行「不涉及决策：理由」）；这次改动里新写成「改了」的行，
#      它指的决策文件要在这次改动里。这次改动把实验的标题状态改成「已跑」（状态段恰好是这两个字）时，
#      决策正文（历史版本之前）引了它的每条决策（`E<号>（`，粗体、反引号、空格、半角括号的写法也认）都要回看：
#      这次改动在那份决策里新增或改写的行点到了它，或者表里那条决策的一行改过。搬了家的页按实验号找基准那一版的标题。
#   ⑥ 待回填清单 .claude/decision-links-pending：一行一个 E<号> 或 D<号>，# 后写理由；指到的要存在；
#      只减不增——比基准那一版多出来的行判红。
#   ⑦ 瘦身形态（待回填清单之外的决策）：首行 `## D<号> 简称 —— 状态` 后面不带括注；分项标题不带日期；
#      每个已定项有「**定案**：」「**射程**：」「**依据**：」「**欠**：」四块；索引表的定案格不带日期、不超过 100 字。
#   ⑧ 有实验才有决策（用户 2026-09-19）：已定项的依据段至少引一个实验，或写「无实验：理由」（理由至少八个字）；
#      成功行报出写「无实验」的已定项有几个。
#   ⑨ 实验必须对应决策（用户 2026-09-19）：待回填清单之外的实验页，表里至少一行关系是支撑 / 推翻 / 备料；
#      标题里写着「结论……作废」或「结论……退役」的实验不判（它们留着是为了记下那条路不通）。
#
# 为什么：2026-09-19 用户指出决策知识腐烂、分项膨胀、决策与实验关联不够，要求「做实验或者有结论后要回去看决策
# 是不是受到了影响」。当天现量：154 个实验页里 296 对「实验引了决策、决策没引回来」，决策里没有一处统一的依据栏；
# 一个实验重跑、结论变了之后，没有任何东西逼人回去看它撑着的那几条决策。
#
# 管不到的：回看写的理由对不对、关系判得对不对（支撑写成不影响也过得了③以外的判据），靠人与抽查；
# 实验正文只用编号不用「D<号>（」形态提到的决策，①看不见。
#
#   bash .claude/gate.d/75-decision-experiment-links.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
if [[ ! -d .claude/kb/experiments || ! -d .claude/kb/decisions ]]; then
  echo "  ✗ 找不到 .claude/kb/experiments 或 .claude/kb/decisions"
  echo "  → 怎么办：kb 挪了位置就同步改这个阶段里的路径。"
  exit 1
fi
base="HEAD"
in_git=0
changed=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1 && git rev-parse --verify -q HEAD >/dev/null 2>&1; then
  in_git=1
  # 改动范围与 56、68、69、97 号同一份取法：research/scripts/changed-paths.sh 的 gate 取法，未跟踪也算
  LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
  # shellcheck source=../../research/scripts/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  base="$(gate_diff_base gate)"
  changed="$(gate_changed_paths "$base" untracked)" || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一阶段什么都没比，不是通过。"
    exit 1
  }
fi
python3 - "$ROOT" "$base" "$in_git" <(printf '%s\n' "$changed") <<'PY'
import glob, os, re, subprocess, sys

root, base, in_git = sys.argv[1], sys.argv[2], sys.argv[3] == '1'
changed = {line for line in open(sys.argv[4], encoding='utf-8').read().split('\n') if line}
kb = os.path.join(root, '.claude', 'kb')
RELATIONS = ('支撑', '推翻', '备料', '不影响')
BASIS_RELATIONS = ('支撑', '推翻')
E_REF = re.compile(r'(?<![A-Za-z0-9])E(\d+)（')
D_REF = re.compile(r'(?<![A-Za-z0-9])D(\d+)（')
# 「决策正文引了它」按 doc-lint 认引用的写法认：粗体、反引号、编号与括号之间有空格、半角括号都算（第三轮三方 T1-b）
MENTION_REF = re.compile(r'(?<![A-Za-z0-9._-])[*`]*E(\d+)[*`]*\s*[（(]')
TABLE_HEAD = re.compile(r'^\|\s*决策分项\s*\|\s*关系\s*\|\s*回看\s*\|\s*$')
REVIEW = re.compile(r'^(20\d\d-\d\d-\d\d)\s*(改了|不受影响：\s*\S.{3,})')
DATE_HEAD = re.compile(r'^### (20\d\d-\d\d-\d\d)')


def read(path):
    return open(path, encoding='utf-8').read()


def history_start(lines):
    for index, line in enumerate(lines):
        if re.match(r'^## 历史版本\s*$', line):
            return index
    return len(lines)


def git(*args):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], cwd=root, capture_output=True, text=True)


def split_cells(line):
    return [cell.strip() for cell in re.sub(r'\\\|', '\x00', line.strip()).strip('|').split('|')]


def parse_decision_ref(cell):
    """「D28（挂载期承诺量） 已定项 2」→ (28, ('已定项', 2))；只写决策 → (28, None)；认不出 → None。"""
    head = re.match(r'^\**D(\d+)（', cell)
    if not head:
        return None
    item = re.search(r'(已定项|未定项)\s*(\d+)\s*\**$', cell)
    return int(head.group(1)), ((item.group(1), int(item.group(2))) if item else None)


def section_range(lines, heading_pattern, stop_level):
    """标题行号与它那一节的结束行号（下一个级别 ≤ stop_level 的标题之前），找不到返回 None。"""
    for index, line in enumerate(lines):
        if re.match(heading_pattern, line):
            for end in range(index + 1, len(lines)):
                level = re.match(r'^(#{1,6}) ', lines[end])
                if level and len(level.group(1)) <= stop_level:
                    return index, end
            return index, len(lines)
    return None


def table_rows(lines, start, end):
    """一节里那张影响的决策表的行：[(行号, 单元格)]；没有表头返回 None。"""
    rows, seen_head = [], False
    for index in range(start + 1, end):
        line = lines[index]
        if TABLE_HEAD.match(line.strip()):
            seen_head = True
            continue
        if seen_head and re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', line.strip()):
            continue
        if seen_head and line.strip().startswith('|'):
            rows.append((index, split_cells(line)))
        elif seen_head:
            break
    return rows if seen_head else None


problems = []   # (类别, 说明)


def bad(kind, text):
    problems.append((kind, text))


# ── 决策：分项与各分项的依据段 ──
decisions = {}
for path in sorted(glob.glob(os.path.join(kb, 'decisions', '*.md'))):
    lines = read(path).split('\n')
    body = lines[:history_start(lines)]
    head = next((re.match(r'^## D(\d+) ', line) for line in body if re.match(r'^## D(\d+) ', line)), None)
    if not head:
        continue
    number = int(head.group(1))
    items, basis, shapes, index_cells = set(), {}, [], []
    for index, line in enumerate(body):
        item = re.match(r'^(#{3,4}) (已定项|未定项) (\d+)', line)
        if not item:
            continue
        key = (item.group(2), int(item.group(3)))
        items.add(key)
        cited, in_basis, labels, basis_text = set(), False, set(), ''
        for inner in body[index + 1:]:
            level = re.match(r'^(#{1,6}) ', inner)
            if level and len(level.group(1)) <= len(item.group(1)):
                break
            label = re.match(r'^\*\*(定案|射程|依据|欠)\*\*：', inner)
            if label:
                labels.add(label.group(1))
            if inner.startswith('**依据**'):
                in_basis = True
            elif re.match(r'^\*\*[^*]{1,12}\*\*：', inner) or level:
                in_basis = False
            if in_basis:
                cited |= {int(e) for e in E_REF.findall(inner)}
                basis_text += inner + '\n'
        basis[key] = cited
        shapes.append((key, line, labels, cited, basis_text))
    for section in ('已定项', '未定项'):
        found = section_range(body, r'^### %s\s*$' % section, 3)
        if found:
            for line in body[found[0] + 1:found[1]]:
                if line.startswith('#### '):
                    break
                row = re.match(r'^\|\s*(\d+)\s*\|', line)
                if row:
                    items.add((section, int(row.group(1))))
                    index_cells.append((section, int(row.group(1)), split_cells(line)[-1]))
                # 没瘦身的决策有的用编号列表列分项（`9. **名字**…`），20 号数分项时也认这种写法
                listed = re.match(r'^(\d+)\.\s', line)
                if listed:
                    items.add((section, int(listed.group(1))))
    title = next(line for line in body if re.match(r'^## D(\d+) ', line))
    decisions[number] = {'path': os.path.relpath(path, root), 'items': items, 'basis': basis,
                         'shapes': shapes, 'index_cells': index_cells, 'title': title,
                         'mentions': {int(e) for e in MENTION_REF.findall('\n'.join(body))}}

# ── 待回填清单 ──
pending_path = os.path.join(root, '.claude', 'decision-links-pending')
pending_rel = '.claude/decision-links-pending'
pending = set()
pending_lines = read(pending_path).split('\n') if os.path.exists(pending_path) else []
for number, line in enumerate(pending_lines, 1):
    if not line.strip() or line.lstrip().startswith('#'):
        continue
    entry = re.match(r'^(E|D)(\d+)\s+#\s*(\S.{3,})$', line.strip())
    if not entry:
        bad('待回填', f'{pending_rel}:{number}：写成「E<号>  # 理由」或「D<号>  # 理由」，理由至少四个字——「{line.strip()[:30]}」')
        continue
    pending.add((entry.group(1), int(entry.group(2))))
if in_git and os.path.exists(pending_path):
    previous = git('show', f'{base}:{pending_rel}')
    if previous.returncode == 0:
        before = set()
        for line in previous.stdout.split('\n'):
            entry = re.match(r'^(E|D)(\d+)\s', line.strip())
            if entry:
                before.add((entry.group(1), int(entry.group(2))))
        for kind, number in sorted(pending - before):
            bad('待回填', f'{pending_rel} 新加了 {kind}{number}：清单只减不增，新写的实验页与决策当场登记，不进清单')

# ── 瘦身形态与「有实验才有决策」（待回填清单之外的决策） ──
DATE = re.compile(r'20\d\d-\d\d-\d\d')
slim_items = no_experiment_items = 0
for decision, info in sorted(decisions.items()):
    if ('D', decision) in pending:
        continue
    where = info['path']
    # 半定 / 待定的决策，20 号要标题里写明未定几项（`—— 半定（一项未定）`），只许这一种括注
    if not re.match(r'^## D\d+ \S.*? —— (已定|半定|待定)(（[0-9一二两三四五六七八九十]+[项条]未定）)?\s*$', info['title']):
        bad('瘦身形态', f'{where}：首行写成「## D{decision} 简称 —— 状态」，状态后面除了半定 / 待定要写的「（N 项未定）」不带别的括注——「{info["title"][:40]}」')
    for (kind, item_number), heading, labels, cited, basis_text in info['shapes']:
        if DATE.search(heading):
            bad('瘦身形态', f'{where}：D{decision} {kind} {item_number} 的标题带日期，日期与来历进变更史')
        if kind != '已定项':
            continue
        slim_items += 1
        for label in ('定案', '射程', '依据', '欠'):
            if label not in labels:
                bad('瘦身形态', f'{where}：D{decision} 已定项 {item_number} 缺「**{label}**：」一块')
        if '依据' in labels and not cited:
            if re.search(r'无实验：\s*\S.{7,}', basis_text):
                no_experiment_items += 1
            else:
                bad('没有实验', f'{where}：D{decision} 已定项 {item_number} 的依据一个实验都没引——有实验才有决策；实在没有可量的，写「无实验：理由」')
    for kind, item_number, cell in info['index_cells']:
        # 「**状态：已定。**」是 20 号要的规范标记（检索端出来的是这一行），不算在一句话的字数里；
        # 31 号要写在未定项登记行里的两句规范标记（「改第一个事务的字节：…」与「动不动格式：…」）同理不算——
        # 31 号明写它要在登记行里（写在表下那一段里它定位不到），而它必须带日期，与这一条的「不带日期、
        # 不超过 100 字」直接相撞：2026-09-20 给 D2 未定项 21 与 D26 未定项 8 / 9 / 10 补上字节判决之后四行一起红。
        # 两句都剥：31 号是两把尺，一句量「这一版写不写出不同字节」、一句量「将来动不动格式」，
        # 缺哪一句它都判红，而两句都必须带日期 ⇒ 两句都不算进这一条的「不带日期、不超过 100 字」。
        without_marks = cell
        for mark in ('改第一个事务的字节', '动不动格式'):
            without_marks = re.sub(mark + r'：[^（(]*[（(][^）)]*[）)]', '', without_marks)
        plain = re.sub(r'\*\*|`', '', re.sub(r'\*\*状态：[^*]*\*\*', '', without_marks)).strip()
        if DATE.search(plain) or len(plain) > 100:
            bad('瘦身形态', f'{where}：D{decision} {kind}索引表第 {item_number} 行的定案格要一句话、不带日期、不超过 100 字（现在 {len(plain)} 字）')

# ── 实验最近一次变动的日期：页内历史节 + experiments-history.md ──
central_dates = {}
central_path = os.path.join(kb, 'experiments-history.md')
if os.path.exists(central_path):
    lines = read(central_path).split('\n')
    start = history_start(lines)
    current_date, heading_refs, body_refs = None, set(), set()

    def flush():
        refs = heading_refs or body_refs
        for experiment in refs:
            if current_date and current_date > central_dates.get(experiment, ''):
                central_dates[experiment] = current_date

    for line in lines[start:]:
        dated = DATE_HEAD.match(line)
        if dated or line.startswith('### ') or line.startswith('## '):
            flush()
            current_date = dated.group(1) if dated else None
            heading_refs = {int(e) for e in E_REF.findall(line)} if dated else set()
            body_refs = set()
            continue
        body_refs |= {int(e) for e in E_REF.findall(line)}
    flush()

# ── 实验页 ──
experiments, rows_total = {}, 0
for path in sorted(glob.glob(os.path.join(kb, 'experiments', '*.md'))):
    rel = os.path.relpath(path, root)
    lines = read(path).split('\n')
    head_line = next((line for line in lines if re.match(r'^## E(\d+) ', line)), None)
    if not head_line:
        continue
    number = int(re.match(r'^## E(\d+) ', head_line).group(1))
    # 只认「结论……作废 / 退役」：标题里别处出现这两个词（「上界 A 作废」「容器退役记录」）的实验结论照样成立，照判（第三轮三方 T1-f）
    voided = re.search(r'结论[^，。：]{0,3}(作废|退役)', head_line) is not None
    hist = history_start(lines)
    section = section_range(lines[:hist], r'^### 影响的决策\s*$', 3)
    rows = table_rows(lines, section[0], section[1]) if section else None
    outside = [line for index, line in enumerate(lines[:hist]) if not (section and section[0] <= index < section[1])]
    mentioned = {int(d) for d in D_REF.findall('\n'.join(outside))}
    local_dates = [m.group(1) for m in (DATE_HEAD.match(line) for line in lines[hist:]) if m]
    latest = max(local_dates + ([central_dates[number]] if number in central_dates else []), default='')
    is_pending = ('E', number) in pending
    experiments[number] = {'rel': rel, 'lines': lines, 'hist': hist, 'section': section, 'rows': rows or []}
    if rows is None and not is_pending:
        bad('没有表', f'{rel}：E{number} 没有「### 影响的决策」表（表头 | 决策分项 | 关系 | 回看 |）')
        continue
    covered = set()
    for index, cells in rows or []:
        rows_total += 1
        where = f'{rel}:{index + 1}'
        if len(cells) != 3:
            bad('行形状', f'{where}：影响的决策表一行要三格，这一行 {len(cells)} 格')
            continue
        ref = parse_decision_ref(cells[0])
        if not ref:
            bad('行形状', f'{where}：第一格要写「D<号>（简称）」或「D<号>（简称） 已定项 k」——「{cells[0][:30]}」')
            continue
        decision, item = ref
        covered.add(decision)
        if decision not in decisions:
            bad('指不到', f'{where}：D{decision} 在 decisions/ 下没有正文')
            continue
        if item and item not in decisions[decision]['items']:
            bad('指不到', f'{where}：D{decision} 没有{item[0]} {item[1]}')
        relation = cells[1].strip('* ')
        if relation not in RELATIONS:
            bad('行形状', f'{where}：关系写「{relation[:12]}」，只认 支撑 / 推翻 / 备料 / 不影响')
            continue
        if relation in BASIS_RELATIONS and item is None and decisions[decision]['items']:
            bad('行形状', f'{where}：关系是{relation}就要写到分项（D{decision} 有分项）')
        review = REVIEW.match(cells[2].strip('* '))
        if not review:
            bad('行形状', f'{where}：回看格写「YYYY-MM-DD 改了」或「YYYY-MM-DD 不受影响：理由」——「{cells[2][:30]}」')
            continue
        if latest and review.group(1) < latest:
            bad('回看过期', f'{where}：E{number} 最近一次变动在 {latest}，对 D{decision} 的回看停在 {review.group(1)}')
        if relation in BASIS_RELATIONS and item and ('D', decision) not in pending \
                and number not in decisions[decision]['basis'].get(item, set()):
            bad('不对称', f'{where}：E{number} 说它{relation} D{decision} {item[0]} {item[1]}，而那条分项的「**依据**」段没引 E{number}')
    if not is_pending and not voided and rows is not None \
            and not any(len(cells) == 3 and cells[1].strip('* ') in ('支撑', '推翻', '备料') for _, cells in rows):
        bad('没对应决策', f'{rel}：E{number} 的影响的决策表里没有一行是支撑 / 推翻 / 备料——实验必须对应决策（结论作废或退役的实验在标题里写明，就不判这一条）')
    if not is_pending:
        for decision in sorted(mentioned - covered):
            bad('没登记', f'{rel}：E{number} 正文提到 D{decision}，影响的决策表里没有它那一行')

# ── 决策这一侧：依据引的实验，实验页里要有支撑或推翻那一行 ──
for decision, info in sorted(decisions.items()):
    if ('D', decision) in pending:
        continue
    for item, cited in sorted(info['basis'].items()):
        for experiment in sorted(cited):
            page = experiments.get(experiment)
            if not page:
                bad('指不到', f"{info['path']}：D{decision} {item[0]} {item[1]} 的依据引了 E{experiment}，kb 里没有这个实验页")
                continue
            ok = any(parse_decision_ref(cells[0]) == (decision, item) and cells[1].strip('* ') in BASIS_RELATIONS
                     for _, cells in page['rows'] if len(cells) == 3)
            if not ok:
                bad('不对称', f"{info['path']}：D{decision} {item[0]} {item[1]} 的依据引了 E{experiment}，{page['rel']} 的影响的决策表里没有「支撑 / 推翻」这一行")
for kind, number in sorted(pending):
    exists = number in (experiments if kind == 'E' else decisions)
    if not exists:
        bad('待回填', f'{pending_rel} 里的 {kind}{number} 在 kb 里没有正文')

# ── 按这次改动 ──
triggered = 0
became_ran_checked = 0


def added_lines(rel):
    """这次改动里 rel 新加的行：[(新文件里的行号, 内容)]；未跟踪的文件整份都算。"""
    if git('ls-files', '--error-unmatch', rel).returncode != 0:
        return [(index, line) for index, line in enumerate(read(os.path.join(root, rel)).split('\n'))]
    diff = git('diff', '--no-renames', '--no-color', '--no-ext-diff', '-U0', base, '--', rel).stdout
    result, new_line, in_header = [], 0, True
    for line in diff.split('\n'):
        hunk = re.match(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@', line)
        if hunk:
            in_header = False
            new_line = int(hunk.group(1)) - 1
            count = int(hunk.group(2)) if hunk.group(2) is not None else 1
            if count == 0:
                result.append((new_line, None))   # 纯删除：记它在新文件里的位置
            continue
        # 「+++ 」只在第一个 @@ 之前是文件头：正文里以「++ 」开头的一行新增在 diff 里也长成「+++ 」
        if line.startswith('+') and not in_header:
            result.append((new_line, line[1:]))
            new_line += 1
    return result


def ran_status(title_line):
    """标题的状态段（「——」之后、括注之前）恰好是「已跑」：部分已跑、已跑但作废都不算。"""
    status = re.match(r'^## E\d+ .*?——\s*(.*)$', title_line or '')
    return bool(status) and re.sub(r'（.*$', '', status.group(1)).strip() == '已跑'


def base_title(rel, number=None):
    """基准那一版的标题行；基准里没有这一页返回 None。"""
    shown = git('show', f'{base}:{rel}')
    # 基准里这个路径没有页（这一批搬了家）：按实验号在基准那棵树里找它原来那页的标题，搬家不算「改成已跑」
    if shown.returncode != 0 and number is not None:
        found = git('grep', '-h', '-m1', '-E', f'^## E{number} ', base, '--', '.claude/kb/experiments/')
        hit = next((l[l.find('## E'):] for l in found.stdout.split('\n') if '## E' in l), None)
        return hit if found.returncode == 0 else None
    if shown.returncode != 0:
        return None
    return next((line for line in shown.stdout.split('\n') if re.match(r'^## E\d+ ', line)), None)


def check_changed_rows(rel, row_lines):
    for text in row_lines:
        cells = split_cells(text)
        if len(cells) != 3:
            continue
        ref = parse_decision_ref(cells[0])
        review = REVIEW.match(cells[2].strip('* '))
        if ref and review and review.group(2) == '改了' and ref[0] in decisions \
                and decisions[ref[0]]['path'] not in changed:
            bad('改了没改', f"{rel}：这次写了「改了」D{ref[0]}，而 {decisions[ref[0]]['path']} 不在这次改动里")


if in_git:
    result_hits = {}
    for rel in changed:
        match = re.match(r'^research/results/e(\d+)[^0-9]', rel)
        if match:
            result_hits.setdefault(int(match.group(1)), rel)
    for number, page in sorted(experiments.items()):
        rel = page['rel']
        body_changed = False
        added = added_lines(rel) if rel in changed and os.path.exists(os.path.join(root, rel)) else []
        section = page['section']
        for line_number, _ in added:
            inside_table = section and section[0] <= line_number < section[1]
            if line_number < page['hist'] and not inside_table:
                body_changed = True
        table_touched = [text for line_number, text in added
                         if text is not None and section and section[0] <= line_number < section[1]
                         and text.strip().startswith('|') and not TABLE_HEAD.match(text.strip())
                         and not re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', text.strip())]
        check_changed_rows(rel, table_touched)
        reasons = []
        if body_changed:
            reasons.append('正文改了')
        if number in result_hits:
            reasons.append(f'产物 {result_hits[number]} 变了')
        if reasons:
            triggered += 1
            if not table_touched:
                bad('没回看', f"{rel}：这次改动里 E{number} {'、'.join(reasons)}，影响的决策表一行都没回看")
        # ⑤ 的第二半：这一批把实验改成已跑，正文里引了它的每条决策都要回看——决策文件在这一批里，
        # 或者影响表里那条决策的一行在这一批里改过。只看「表里任一行被改过」放不过「决策在欠账块里等它、表里却没它」那一种。
        title_line = next((line for line in page['lines'] if re.match(r'^## E\d+ ', line)), None)
        if body_changed and ran_status(title_line) and not ran_status(base_title(rel, number)):
            reviewed = {ref[0] for ref in (parse_decision_ref(split_cells(text)[0]) for text in table_touched) if ref}
            for decision, info in sorted(decisions.items()):
                if number not in info['mentions'] or ('D', decision) in pending:
                    continue
                became_ran_checked += 1
                # 决策文件在这一批里还不够：要这一批在那份决策里新增或改写的行点到了这个实验号（改一处别的不算回看）
                touched_mention = info['path'] in changed and any(
                    text and re.search(r'(?<![A-Za-z0-9])E%d(?![0-9])' % number, text) for _, text in added_lines(info['path']))
                if not touched_mention and decision not in reviewed:
                    bad('没回看引用它的决策', f"{rel}：这次改动把 E{number} 改成已跑，{info['path']} 的正文引了它，"
                        f"而这次改动在那份决策里没有一行点到它、影响的决策表里 D{decision} 那一行也没回看")
    for rel in sorted(changed):
        if not re.match(r'^research/prompts/[^/]+-main-verification\.md$', rel) or not os.path.exists(os.path.join(root, rel)):
            continue
        if git('cat-file', '-e', f'{base}:{rel}').returncode == 0:
            continue
        triggered += 1
        lines = read(os.path.join(root, rel)).split('\n')
        section = section_range(lines, r'^## 回看决策\s*$', 2)
        if not section:
            bad('没回看', f'{rel}：新写的三方判决没有「## 回看决策」一节')
            continue
        rows = table_rows(lines, section[0], section[1])
        waived = any(re.match(r'^不涉及决策：\s*\S.{3,}', line.strip()) for line in lines[section[0] + 1:section[1]])
        if not rows and not waived:
            bad('没回看', f'{rel}：「## 回看决策」里既没有表行，也没有一行「不涉及决策：理由」')
            continue
        for index, cells in rows or []:
            ref = parse_decision_ref(cells[0]) if len(cells) == 3 else None
            if not ref or not REVIEW.match(cells[2].strip('* ')):
                bad('行形状', f'{rel}:{index + 1}：回看决策表一行写成 | D<号>（简称） [已定项 k] | 关系 | YYYY-MM-DD 改了 / 不受影响：理由 |')
        check_changed_rows(rel, [lines[index] for index, _ in rows or []])

pending_experiments = sum(1 for kind, _ in pending if kind == 'E')
pending_decisions = sum(1 for kind, _ in pending if kind == 'D')
summary = (f'实验页 {len(experiments)} 个（待回填 {pending_experiments}）、决策 {len(decisions)} 条（待回填 {pending_decisions}）、'
           f'已瘦身决策的已定项 {slim_items} 个（其中写「无实验」的 {no_experiment_items} 个）、'
           f'表行 {rows_total} 行、这次改动触发回看 {triggered} 处、改成已跑而引了它的决策 {became_ran_checked} 条')
if problems:
    kinds = {}
    for kind, _ in problems:
        kinds[kind] = kinds.get(kind, 0) + 1
    print(f"  ✗ 决策与实验的登记有 {len(problems)} 处问题（{'、'.join(f'{k} {v}' for k, v in kinds.items())}）；{summary}：")  # gate-lint:summary
    for kind, text in problems:
        print(f'      [{kind}] {text}')  # gate-lint:detail
    print('  → 怎么办：格式与判据见 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。')
    print('            实验出了新结论、换了产物，就逐行回看它影响的决策，回看格写当天日期与「改了」或「不受影响：理由」；')
    print('            支撑 / 推翻的那条分项在「**依据**」段引回这个实验；待回填清单只能删行，新实验页当场写全这张表。')
    sys.exit(1)
print(f'  ✓ 决策与实验双向登记对得上、回看不过期（{summary}）')
PY
