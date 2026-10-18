#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command git
"""只暂存这一轮的改动：几个会话共写同一批文件时，把工作区里「我的」那几块挑进暂存区，别人的留在工作区。

用法：
  stage-mine.py --match 正则 [--match 正则 …] [--foreign 正则 …] [--split 正则] [--by-row] [--dry-run] 文件…
  stage-mine.py --write-index 文件 内容来源文件 [--foreign 正则 …]
  stage-mine.py --selftest      # STAGE_MINE_BREAK=<项> 时必须判红（项名见文末）

怎么挑：拿暂存区里的版本（没暂存过就是 HEAD）当底，与工作区逐行比，得到一段段改动；
插入的整段再按「块」切开——默认 markdown 标题行、表格行、replay.sh 的登记行（`E<数字>|`）各起一块，其余行跟着前一块走；
一块里有任何一行命中 --match 就是我的，进暂存区；一行都不命中的留在工作区。
改了已有行的那种改动切不开：两个会话改了同一行时它会整段进来，所以任何进暂存区的块命中 --foreign 一律拒绝。
新文件不归它管：整份是这一轮的就直接 `git add`。

**挪了位置的行**：一行被挪到别处、内容又顺手改过时，逐行比对给出的是「这里删一行、那里插一行」两段，
插进来的那一段命中 --match，删掉的旧行却不命中——只挑前者，暂存区里就有新旧两份同一行。
所以删掉的行若是某个「我的」插入行的旧版（开头 12 个字相同、整行相似度 ≥ 0.6），跟着一起算我的，反过来也一样。
实测（2026-09-12）：experiments.md 里 E137 那一行在 HEAD 里排在 E138 后面，工作区里挪到 E138 前面并改了一个标签，
dry-run 报「留在工作区 1 块」，留下的正是那一行的旧版。

**--by-row（整表重排时按整行挑）**：另一个会话把一张表整体重排（行序全变、别的行改了标签）时，一次比对把整张表变成少数几个大块，
自己的一行与别人的十几行挤在同一块里，--match 与 --foreign 同时命中，按块只能拒绝。带 --by-row：这样的块若是「纯行式」就不拒绝，按整行挑——
纯行式的判据：块里（底里的行与工作区的行）每一行都是表格行（整行以 `|` 起头、以 `|` 收尾，缩进不计）或登记行（`E<数字>|` 起头），没有空行、没有正文行。
挑法：工作区里命中 --match 的新增行进暂存区，别的新增行和别人的删除行留在工作区，暂存区因此是「HEAD + 我的新增行」；
我的行插在它在工作区里的前一行（那一行在块里原样存在时）后面，前面没有可对的行就插在块首；
前一行在块里重复出现时按位置对应：它在工作区块里是第几次出现，就取底的块里同样第几次出现的那一处（`matching_index()`），不取第一处。
删除行里命中 --match 的、以及按上面「挪了位置的行」的判法是我的某个新增行旧版的（命中 --foreign 的除外），跟着一起删，不留新旧两份。
新增行里本身同时命中 --match 与 --foreign 的，仍然拒绝。不是纯行式的块照旧拒绝，拒绝句点名出路。没带 --by-row 时行为不变。

**--write-index（直接写索引前核 --foreign）**：手里已经有一份改好的内容，要直接写进索引（绕过按块挑）时，不自己 `git update-index`，走这一条：
内容与 HEAD 那一版比（文件不在 HEAD 里就当空文件），新增行里有命中 --foreign 的就拒绝、退 3、列出命中的行、一个字不写；没有才 `hash-object` + `update-index`。
文件相对仓根；内容来源文件按调用时的目录算。按块挑的写索引（`stage`）走同一个函数 `write_index`，不另抄；
它拿暂存区那一版当比对底（块里本来就在暂存区、别的会话暂存的行不算这一次新增的）。

为什么要有它：同一个文件里两个会话的块常常紧挨着插在同一处（两份变更史的最上面、索引表的末尾），
按行或按段手抄「HEAD + 我的」已经手写了四遍，每遍都要为那几个文件现写一段 python。

自证会红：`stage-mine.py --selftest` 造两份文件——一份两个会话各插一块、紧挨着，一份一行挪了位置又改了标签——
确认只暂存了我的、挪动的那一行没有留下旧版；再分别用 STAGE_MINE_NO_SPLIT=1、STAGE_MINE_NO_MOVE_PAIRING=1
关掉按块切与挪动配对，确认 selftest 判红。另造一张表（我的一行夹在别人五行中间、整表重排、别人的行改了标签）：
没带 --by-row 必须拒绝，带了必须只暂存我那一行、暂存区 = HEAD + 我那一行，块里夹正文行时带了也拒绝；
再造一份带别人新增行的内容喂给 --write-index，必须退 3、索引不动，只带我的新增行的必须写进去。
另造一个块（两张表各有一条分隔行 `|---|`，我的一行加在第二张表的分隔行后面），必须落在第二处。

弄坏开关 STAGE_MINE_BREAK=<项>（只给 --selftest 证红用，打开后 --selftest 退 1；项名、弄坏哪一处、红在哪个用例）：
  STAGE_MINE_BREAK=by-row-takes-block   --by-row 退回整块进：同时命中 --match 与 --foreign 的纯行式块整块进暂存区，别人的行跟着进来
      红在「整表重排按整行挑」（selftest: 整表重排应当只暂存我那一行……）
  STAGE_MINE_BREAK=write-index-unchecked   write_index() 写之前不核 --foreign
      红在「--write-index 带别人新增行」（selftest: --write-index 带别人新增行应当退 3……）
  STAGE_MINE_BREAK=first-occurrence-anchor   choose_rows() 找锚点退回取底的块里第一处命中（改前的写法）
      红在「块里有重复行」（selftest: 重复行：| MINE | 应当落在第二张表的 |---| 之后……）
STAGE_MINE_NO_SPLIT=1、STAGE_MINE_NO_MOVE_PAIRING=1 是更早的两个关闭开关，行为见上，selftest 写法不同（判红即退 0）。
"""
import difflib
import os
import re
import subprocess
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

DEFAULT_SPLIT = r'^(#{1,6} |\||E[0-9]+\|)'
MOVE_HEAD_CHARACTERS = 12
MOVE_SIMILARITY = 0.6
ROW_PATTERN = re.compile(r'^(\s*\|.*\|\s*|E[0-9]+\|.*)$')
BROKEN = os.environ.get('STAGE_MINE_BREAK', '')


def git(args, cwd=None, stdin=None):
    return subprocess.run(['git', *args], cwd=cwd, input=stdin, capture_output=True, check=True).stdout


def split_into_blocks(lines, split_pattern):
    if os.environ.get('STAGE_MINE_NO_SPLIT') == '1':
        return [lines]
    blocks = []
    for line in lines:
        if not blocks or split_pattern.search(line):
            blocks.append([line])
        else:
            blocks[-1].append(line)
    return blocks


def slide_insertions(base_lines, work_lines, split_pattern):
    """把逐行比对切成（种类, 底里的行, 工作区的行）的段；插入段往下滑到以块首开头为止。

    difflib 对「插入的一段以空行开头、插入点后面也是空行」给哪种对齐都对，而它常把空行对到前面，
    插入段就以一个空行开头、自成一块、不命中 --match，被留在工作区，暂存区里的空行因此错位一行。
    滑动不改内容：插入段首行与后面相等段的首行相同时，把它换到相等段里去（git diff 的 slider 同理）。
    """
    segments = []
    matcher = difflib.SequenceMatcher(None, base_lines, work_lines, autojunk=False)
    for tag, base_start, base_end, work_start, work_end in matcher.get_opcodes():
        old_lines, new_lines = base_lines[base_start:base_end], work_lines[work_start:work_end]
        segments.append([('equal' if tag == 'equal' else 'insert' if tag == 'insert' else 'change'), old_lines, new_lines])
    for index, segment in enumerate(segments):
        if segment[0] != 'insert' or index + 1 >= len(segments) or segments[index + 1][0] != 'equal':
            continue
        following = segments[index + 1]
        inserted = segment[2]
        while inserted and following[1] and not split_pattern.search(inserted[0]) and following[1][0] == inserted[0]:
            moved = following[1][0]
            if index > 0 and segments[index - 1][0] == 'equal':
                segments[index - 1][1].append(moved)
                segments[index - 1][2].append(moved)
            else:
                segments.insert(index, ['equal', [moved], [moved]])
                index += 1
            inserted = inserted[1:] + [moved]
            following[1], following[2] = following[1][1:], following[2][1:]
        segment[2] = inserted
    return [tuple(segment) for segment in segments if segment[1] or segment[2]]


def is_moved_version_of_any(line, candidates):
    """这一行是不是 candidates 里某一行挪了位置的版本（内容可能顺手改过）。"""
    stripped = line.strip()
    if len(stripped) < MOVE_HEAD_CHARACTERS:
        return False
    for candidate in candidates:
        other = candidate.strip()
        if len(other) < MOVE_HEAD_CHARACTERS or other[:MOVE_HEAD_CHARACTERS] != stripped[:MOVE_HEAD_CHARACTERS]:
            continue
        if difflib.SequenceMatcher(None, stripped, other, autojunk=False).ratio() >= MOVE_SIMILARITY:
            return True
    return False


def hits_any(line, patterns):
    return any(pattern.search(line) for pattern in patterns)


def matching_index(old_lines, new_lines, position):
    """工作区块里第 position 行，在底的块里对应的行号：这一行内容在工作区块里是第几次出现，就取底的块里同样第几次出现的那一处；
    底里出现次数不够就返回 None。块里有重复行（两张表各有一条分隔行）时，按位置对应才不会锚到第一处。"""
    line = new_lines[position]
    if BROKEN == 'first-occurrence-anchor':
        return old_lines.index(line) if line in old_lines else None
    occurrence = new_lines[:position + 1].count(line)
    seen = 0
    for index, old_line in enumerate(old_lines):
        if old_line == line:
            seen += 1
            if seen == occurrence:
                return index
    return None


def choose_rows(path, old_lines, new_lines, match_patterns, foreign_patterns):
    """纯行式的块按整行挑。返回（这一块进暂存区后的行, 进暂存区的新增行, 删掉的旧行, 留在工作区的新增行）；
    没有一行是我的就返回 None；拒绝时抛 SystemExit。"""
    for line in old_lines + new_lines:
        if not ROW_PATTERN.match(line.rstrip('\n')):
            print(f'  ✗ {path}：同时命中 --match 与 --foreign 的块不是纯行式（有一行不是表格行或登记行）：{line.rstrip()[:90]}')
            print('    → --by-row 只按整行挑纯表格行 / 登记行的块；换一个 --split 把这块切开，或者等对方先提交，或者把自己的行写成一份内容走 --write-index')
            raise SystemExit(4)
    mine_rows = [line for line in new_lines if hits_any(line, match_patterns)]
    for line in mine_rows:
        if hits_any(line, foreign_patterns):
            print(f'  ✗ {path}：我的新增行本身命中 --foreign：{line.rstrip()[:90]}')
            print('    → 这一行同时像我的也像别人的，--by-row 分不开：改 --match / --foreign 的正则，或者等对方先提交')
            raise SystemExit(4)
    dropped = set()
    for index, line in enumerate(old_lines):
        if hits_any(line, foreign_patterns):
            continue
        if hits_any(line, match_patterns):
            dropped.add(index)
        elif os.environ.get('STAGE_MINE_NO_MOVE_PAIRING') != '1' and is_moved_version_of_any(line, mine_rows):
            dropped.add(index)
    if not mine_rows and not dropped:
        return None
    insert_after = {}   # 底里的行号（-1 是块首）→ 接在它后面的我的行
    for position, line in enumerate(new_lines):
        if line not in mine_rows:
            continue
        anchor = -1
        for earlier in range(position - 1, -1, -1):
            if new_lines[earlier] in mine_rows:
                continue
            matched = matching_index(old_lines, new_lines, earlier)
            if matched is not None and matched not in dropped:
                anchor = matched
                break
        insert_after.setdefault(anchor, []).append(line)
    result = list(insert_after.get(-1, []))
    for index, line in enumerate(old_lines):
        if index not in dropped:
            result.append(line)
        result.extend(insert_after.get(index, []))
    result = [line if line.endswith('\n') or index == len(result) - 1 else line + '\n' for index, line in enumerate(result)]
    left_rows = [line for line in new_lines if line not in mine_rows]
    return result, mine_rows, [old_lines[index] for index in sorted(dropped)], left_rows


def choose(path, match_patterns, foreign_patterns, split_pattern, cwd, by_row=False):
    """返回（新的暂存内容, 进暂存区的块, 留在工作区的块）；每块是（行, 是不是按挪动配上的；按整行挑出的写 'row'）。拒绝时抛 SystemExit。"""
    try:
        base_text = git(['show', f':{path}'], cwd=cwd).decode('utf-8')
    except subprocess.CalledProcessError:
        print(f'  ✗ {path} 不在暂存区里（新文件）')
        print('    → 整份是这一轮的就直接 git add；不是这一轮的别碰')
        raise SystemExit(2)
    with open(os.path.join(cwd, path), encoding='utf-8') as handle:
        work_text = handle.read()
    if base_text == work_text:
        print(f'  ✗ {path}：工作区与暂存区相同，没有改动可挑')
        print('    → 这个文件这一轮没改（或已经整份暂存 / 提交过了），别把它列进来')
        raise SystemExit(3)
    base_lines = base_text.splitlines(keepends=True)
    work_lines = work_text.splitlines(keepends=True)
    is_mine = lambda lines: any(pattern.search(line) for line in lines for pattern in match_patterns)
    units = []   # 每段：[种类, 底里的行, 工作区的行, 是不是我的, 是不是按挪动配上的]
    for tag, old_lines, new_lines in slide_insertions(base_lines, work_lines, split_pattern):
        if tag == 'equal':
            units.append(['equal', old_lines, new_lines, False, False])
        elif tag == 'insert':
            for block in split_into_blocks(new_lines, split_pattern):
                units.append(['insert', [], block, is_mine(block), False])
        else:
            units.append(['change', old_lines, new_lines, is_mine(old_lines + new_lines), False])
    if os.environ.get('STAGE_MINE_NO_MOVE_PAIRING') != '1':
        my_new_lines = [line for unit in units if unit[3] for line in unit[2]]
        my_old_lines = [line for unit in units if unit[3] for line in unit[1]]
        for unit in units:
            if unit[0] == 'equal' or unit[3]:
                continue
            is_pure_deletion = bool(unit[1]) and not unit[2]
            is_pure_insertion = bool(unit[2]) and not unit[1]
            if (is_pure_deletion and any(is_moved_version_of_any(line, my_new_lines) for line in unit[1])) or \
               (is_pure_insertion and any(is_moved_version_of_any(line, my_old_lines) for line in unit[2])):
                unit[3], unit[4] = True, True
    result, staged, left = [], [], []
    for kind, old_lines, new_lines, mine, paired in units:
        if kind == 'equal':
            result.extend(old_lines)
            continue
        shown = new_lines or old_lines
        if mine and by_row and any(hits_any(line, foreign_patterns) for line in shown):
            if BROKEN == 'by-row-takes-block':
                result.extend(new_lines)
                staged.append((shown, 'row'))
                continue
            picked = choose_rows(path, old_lines, new_lines, match_patterns, foreign_patterns)
            if picked is None:
                result.extend(old_lines)
                left.append((shown, False))
                continue
            rows_result, mine_rows, dropped_lines, left_rows = picked
            result.extend(rows_result)
            for row in mine_rows or dropped_lines:
                staged.append(([row], 'row'))
            if left_rows:
                left.append((left_rows, False))
        elif mine:
            result.extend(new_lines)
            staged.append((shown, paired))
        else:
            result.extend(old_lines)
            left.append((shown, False))
    if not staged:
        print(f'  ✗ {path}：没有一块命中 --match')
        print('    → 查一下正则写对没有；这个文件里真没有这一轮的改动，就别把它列进来')
        raise SystemExit(3)
    for block, kind in staged:
        if kind == 'row':
            continue   # 按整行挑出的行已在 choose_rows 里核过 --foreign
        hits = [line for line in block for pattern in foreign_patterns if pattern.search(line)]
        if hits:
            print(f'  ✗ {path}：进暂存区的一块里有命中 --foreign 的行：{hits[0].rstrip()[:90]}')
            print('    → 两个会话的改动挤在同一块里：块里每一行都是表格行 / 登记行的，加 --by-row 按整行挑；'
                  '别的换一个 --split 把它们切开，或者等对方先提交')
            raise SystemExit(4)
    return ''.join(result), staged, left


def new_lines_against(base_text, content):
    """content 相对 base_text 新增（含改写后）的行。"""
    base_lines, content_lines = base_text.splitlines(keepends=True), content.splitlines(keepends=True)
    added = []
    for tag, _, _, work_start, work_end in difflib.SequenceMatcher(None, base_lines, content_lines, autojunk=False).get_opcodes():
        if tag in ('insert', 'replace'):
            added.extend(content_lines[work_start:work_end])
    return added


def write_index(path, content, cwd, foreign_patterns, base_ref):
    """往索引里写 path 的一份内容；写之前，与 base_ref（HEAD 或暂存区）那一版比，新增行里有命中 --foreign 的就拒绝（退 3）、一个字不写。
    base_ref 取 'HEAD' 或 'index'。--write-index 与按块挑的 stage() 都经这一处。"""
    if BROKEN != 'write-index-unchecked':
        try:
            base_text = git(['show', f'HEAD:{path}' if base_ref == 'HEAD' else f':{path}'], cwd=cwd).decode('utf-8')
        except subprocess.CalledProcessError:
            base_text = ''   # 文件不在 base_ref 里：整份都是新增
        hits = [line for line in new_lines_against(base_text, content) if hits_any(line, foreign_patterns)]
        if hits:
            shown_base = 'HEAD' if base_ref == 'HEAD' else '暂存区'
            print(f'  ✗ {path}：要写进索引的内容里，与{shown_base}那一版比，有 {len(hits)} 行新增的命中 --foreign，一个字没写：')
            for line in hits[:20]:
                print(f'      {line.rstrip()[:110]}')
            print('    → 这些是别人的行：从内容里去掉再写；确实是自己的，收窄 --foreign 的正则')
            return 3
    listing = git(['ls-files', '-s', '--', path], cwd=cwd).decode('utf-8').split()
    if listing:
        mode = listing[0]
    else:
        mode = '100755' if os.access(os.path.join(cwd, path), os.X_OK) else '100644'
    blob = git(['hash-object', '-w', '--stdin'], cwd=cwd, stdin=content.encode('utf-8')).decode('utf-8').strip()
    git(['update-index', '--add', '--cacheinfo', f'{mode},{blob},{path}'], cwd=cwd)
    return 0


def stage(path, content, cwd, foreign_patterns=()):
    return write_index(path, content, cwd, foreign_patterns, 'index')


def run_write_index(argv, cwd):
    foreign_patterns, positional = [], []
    arguments = iter(argv)
    for argument in arguments:
        if argument == '--foreign':
            foreign_patterns.append(re.compile(next(arguments)))
        else:
            positional.append(argument)
    if len(positional) != 2:
        print('  ✗ --write-index 要一个文件（相对仓根）和一个内容来源文件')
        print('    → stage-mine.py --write-index 文件 内容来源文件 --foreign "E138"')
        return 2
    path, source = positional
    with open(source, encoding='utf-8') as handle:
        content = handle.read()
    code = write_index(path, content, cwd, foreign_patterns, 'HEAD')
    if code == 0:
        print(f'✓ {path}：内容（{source}）写进了索引；' + (f'与 HEAD 比的新增行没有命中那 {len(foreign_patterns)} 条 --foreign 的' if foreign_patterns else '没给 --foreign，没核'))
    return code


def run(argv, cwd):
    if argv[:1] == ['--write-index']:
        return run_write_index(argv[1:], cwd)
    match_patterns, foreign_patterns, paths = [], [], []
    split_pattern, dry_run, by_row = re.compile(DEFAULT_SPLIT), False, False
    arguments = iter(argv)
    for argument in arguments:
        if argument == '--match':
            match_patterns.append(re.compile(next(arguments)))
        elif argument == '--foreign':
            foreign_patterns.append(re.compile(next(arguments)))
        elif argument == '--split':
            split_pattern = re.compile(next(arguments))
        elif argument == '--dry-run':
            dry_run = True
        elif argument == '--by-row':
            by_row = True
        else:
            paths.append(argument)
    if not match_patterns or not paths:
        print('  ✗ 至少要一个 --match 和一个文件')
        print('    → stage-mine.py --match "E137|（其十）" --foreign "E138" 文件…')
        return 2
    total_staged = 0
    for path in paths:
        try:
            content, staged, left = choose(path, match_patterns, foreign_patterns, split_pattern, cwd, by_row)
        except SystemExit as refusal:
            return int(refusal.code)
        if not dry_run:
            code = stage(path, content, cwd, foreign_patterns)
            if code != 0:
                return code
        total_staged += len(staged)
        print(f'{path}：暂存 {len(staged)} 块，留在工作区 {len(left)} 块' + ('（--dry-run，暂存区没动）' if dry_run else ''))
        for block, paired in staged:
            print(f'  + {block[0].rstrip()[:100]}' + ('（按挪动配上的旧版，一起删）' if paired is True else '（--by-row 按整行挑出）' if paired == 'row' else ''))
        for block, _ in left:
            print(f'  · {block[0].rstrip()[:100]}')
    print(f'✓ 查了 {len(paths)} 个文件，暂存 {total_staged} 块')
    return 0


def selftest():
    with tempfile.TemporaryDirectory() as repo:
        git(['init', '-q'], cwd=repo)
        git(['config', 'user.email', 'selftest@example.invalid'], cwd=repo)
        git(['config', 'user.name', 'selftest'], cwd=repo)
        history_base = '# 变更史\n\n## 历史版本\n\n### 2026-09-11（其八）：旧的\n\n- 旧\n\n| 编号 | 结论 |\n|---|---|\n| E136 | 旧 |\n\nE136|e136||x.out|exact\n'
        index_base = '| 编号 | 结论 |\n|---|---|\n| E136（样本） | 旧的一行 |\n| E138（别人的） | 已经提交的一行 |\n| E137（样本丁） | 挂在未定项 6 下面的一行 |\n'
        for name, text in (('history.md', history_base), ('index.md', index_base)):
            with open(os.path.join(repo, name), 'w', encoding='utf-8') as handle:
                handle.write(text)
        git(['add', 'history.md', 'index.md'], cwd=repo)
        git(['commit', '-q', '-m', 'base'], cwd=repo)
        mine_block = '### 2026-09-11（其十）：我的\n\n- 我的\n\n'
        their_block = '### 2026-09-11（其九）：别人的\n\n- 别人的\n\n'
        history_work = history_base.replace('### 2026-09-11（其八）', mine_block + their_block + '### 2026-09-11（其八）', 1)
        history_work = history_work.replace('| E136 | 旧 |\n', '| E136 | 旧 |\n| E137 | 我的 |\n| E138 | 别人的 |\n', 1)
        history_work = history_work.replace('E136|e136||x.out|exact\n', 'E136|e136||x.out|exact\nE137|e137||y.out|exact\nE138|e138||z.out|exact\n', 1)
        index_work = '| 编号 | 结论 |\n|---|---|\n| E136（样本） | 旧的一行 |\n| E137（样本丁） | 挂在已定项 6 下面的一行 |\n| E138（别人的） | 已经提交的一行 |\n'
        for name, text in (('history.md', history_work), ('index.md', index_work)):
            with open(os.path.join(repo, name), 'w', encoding='utf-8') as handle:
                handle.write(text)
        history_expected = history_base.replace('### 2026-09-11（其八）', mine_block + '### 2026-09-11（其八）', 1)
        history_expected = history_expected.replace('| E136 | 旧 |\n', '| E136 | 旧 |\n| E137 | 我的 |\n', 1)
        history_expected = history_expected.replace('E136|e136||x.out|exact\n', 'E136|e136||x.out|exact\nE137|e137||y.out|exact\n', 1)
        history_code = run(['--match', '其十|E137', '--foreign', '其九|E138', 'history.md'], repo)
        index_code = run(['--match', '已定项 6', 'index.md'], repo)
        history_ok = history_code == 0 and git(['show', ':history.md'], cwd=repo).decode('utf-8') == history_expected
        index_ok = index_code == 0 and git(['show', ':index.md'], cwd=repo).decode('utf-8') == index_work
        forced_off = [flag for flag in ('STAGE_MINE_NO_SPLIT', 'STAGE_MINE_NO_MOVE_PAIRING') if os.environ.get(flag) == '1']
        if forced_off:
            if history_ok and index_ok:
                print(f'selftest: 关掉 {"、".join(forced_off)} 之后仍然只暂存了我的 —— 检查坏了'); return 1
            print(f'selftest: 关掉 {"、".join(forced_off)} 确认判红（别人的块被卷进来、或挪动那一行留下了旧版）'); return 0
        if not history_ok:
            print('selftest: 紧挨着的两块没切开，暂存区不等于「HEAD + 我的」—— 正是本脚本要防的形态'); return 1
        if not index_ok:
            print('selftest: 挪了位置的那一行留下了旧版，暂存区里有两份 —— 正是本脚本要防的形态'); return 1
        if run(['--match', '不存在的标记', 'history.md'], repo) != 3:
            print('selftest: --match 一处都没命中时没有拒绝'); return 1
        problem = selftest_by_row() or selftest_write_index() or selftest_duplicate_anchor()
        if problem:
            print(f'selftest: {problem}'); return 1
        print('selftest: 通过（紧挨着的两块、两行表格、两行登记各切开，只暂存了我的；挪了位置的那一行只留新版；没命中时拒绝；'
              '整表重排时 --by-row 只暂存我那一行、没带 --by-row 或块里夹正文行时拒绝；块里有重复行时我的行按位置对应锚到第二处；--write-index 带别人新增行时退 3、不带时写进索引）')
        return 0


def selftest_repo(directory, files):
    git(['init', '-q'], cwd=directory)
    git(['config', 'user.email', 'selftest@example.invalid'], cwd=directory)
    git(['config', 'user.name', 'selftest'], cwd=directory)
    for name, text in files.items():
        with open(os.path.join(directory, name), 'w', encoding='utf-8') as handle:
            handle.write(text)
    git(['add', *files], cwd=directory)
    git(['commit', '-q', '-m', 'base'], cwd=directory)


def selftest_by_row():
    """整表重排：我的一行夹在别人五行中间、行序全变、别人的行改了标签。返回问题描述，没问题返回 None。"""
    table_base = '# 表\n\n| 编号 | 结论 |\n|---|---|\n| E131 | 甲 |\n| E132 | 乙 |\n| E133 | 丙 |\n| E134 | 丁 |\n| E135 | 戊 |\n'
    table_work = ('# 表\n\n| 编号 | 结论 |\n|---|---|\n| E135 | 戊（改了标签） |\n| E134 | 丁（改了标签） |\n| E133 | 丙 |\n'
                  '| E137 | 我的一行 |\n| E132 | 乙（改了标签） |\n| E131 | 甲（改了标签） |\n')
    table_expected = table_base.replace('| E133 | 丙 |\n', '| E133 | 丙 |\n| E137 | 我的一行 |\n', 1)
    note_base = '| E131 | 甲 |\n说明一行\n| E132 | 乙 |\n'
    note_work = '| E132 | 乙（改了标签） |\n说明改过\n| E137 | 我的一行 |\n| E131 | 甲（改了标签） |\n'
    arguments = ['--match', 'E137', '--foreign', 'E13[1-5]']
    with tempfile.TemporaryDirectory() as repo:
        selftest_repo(repo, {'table.md': table_base, 'note.md': note_base})
        for name, text in (('table.md', table_work), ('note.md', note_work)):
            with open(os.path.join(repo, name), 'w', encoding='utf-8') as handle:
                handle.write(text)
        if run(arguments + ['table.md'], repo) != 4 or git(['show', ':table.md'], cwd=repo).decode('utf-8') != table_base:
            return '整表重排没带 --by-row 应当拒绝（退 4）、暂存区不动'
        code = run(arguments + ['--by-row', 'table.md'], repo)
        staged_text = git(['show', ':table.md'], cwd=repo).decode('utf-8')
        if code != 0 or staged_text != table_expected:
            return '整表重排应当只暂存我那一行、暂存区 = HEAD + 我那一行（--by-row 按整行挑，别人的行和删除行留在工作区）'
        with open(os.path.join(repo, 'table.md'), encoding='utf-8') as handle:
            if handle.read() != table_work:
                return '整表重排：工作区被改动了'
        if run(arguments + ['--by-row', 'note.md'], repo) != 4 or git(['show', ':note.md'], cwd=repo).decode('utf-8') != note_base:
            return '整表重排块里夹正文行，带 --by-row 也应当拒绝（退 4）、暂存区不动'
    return None


def selftest_duplicate_anchor():
    """块里有重复行（两张表各有一条分隔行）：我的一行加在第二张表的分隔行后面，必须插在那一处，不能锚到第一张表的。"""
    base_lines = ['| a |\n', '|---|\n', '| x |\n', '| b |\n', '|---|\n', '| y |\n']
    work_lines = base_lines[:5] + ['| MINE |\n'] + base_lines[5:]
    picked = choose_rows('sample.md', base_lines, work_lines, [re.compile('MINE')], [])
    if picked is None:
        return '重复行：我的新增行 | MINE | 没有被挑出来'
    if picked[0] != work_lines:
        return ('重复行：| MINE | 应当落在第二张表的 |---| 之后、| y | 之前（暂存内容 = 工作区这一块），实际 '
                + ''.join(picked[0]).replace(chr(10), ' ⏎ '))
    return None


def selftest_write_index():
    table_base = '| 编号 | 结论 |\n|---|---|\n| E131 | 甲 |\n| E132 | 乙 |\n'
    mine_only = table_base + '| E137 | 我的一行 |\n'
    with_foreign = mine_only + '| E138 | 别人新增的一行 |\n'
    with tempfile.TemporaryDirectory() as repo:
        selftest_repo(repo, {'table.md': table_base})
        with open(os.path.join(repo, 'source.txt'), 'w', encoding='utf-8') as handle:
            handle.write(with_foreign)
        arguments = ['--write-index', 'table.md', os.path.join(repo, 'source.txt'), '--foreign', 'E131|E138']
        if run(arguments, repo) != 3 or git(['show', ':table.md'], cwd=repo).decode('utf-8') != table_base:
            return '--write-index 带别人新增行应当退 3、索引一个字不动'
        with open(os.path.join(repo, 'source.txt'), 'w', encoding='utf-8') as handle:
            handle.write(mine_only)
        if run(arguments, repo) != 0 or git(['show', ':table.md'], cwd=repo).decode('utf-8') != mine_only:
            return '--write-index 只带我的新增行（HEAD 里原有的行命中 --foreign 不算新增）应当写进索引'
    return None


def main():
    if sys.argv[1:2] == ['--selftest']:
        sys.exit(selftest())
    top = git(['rev-parse', '--show-toplevel']).decode('utf-8').strip()
    sys.exit(run(sys.argv[1:], top))


if __name__ == '__main__':
    preflight(__file__)
    main()
