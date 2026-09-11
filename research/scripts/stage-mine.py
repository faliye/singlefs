#!/usr/bin/env python3
"""只暂存这一轮的改动：几个会话共写同一批文件时，把工作区里「我的」那几块挑进暂存区，别人的留在工作区。

用法：
  stage-mine.py --match 正则 [--match 正则 …] [--foreign 正则 …] [--split 正则] [--dry-run] 文件…
  stage-mine.py --selftest

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

为什么要有它：同一个文件里两个会话的块常常紧挨着插在同一处（两份变更史的最上面、索引表的末尾），
按行或按段手抄「HEAD + 我的」已经手写了四遍，每遍都要为那几个文件现写一段 python。

自证会红：`stage-mine.py --selftest` 造两份文件——一份两个会话各插一块、紧挨着，一份一行挪了位置又改了标签——
确认只暂存了我的、挪动的那一行没有留下旧版；再分别用 STAGE_MINE_NO_SPLIT=1、STAGE_MINE_NO_MOVE_PAIRING=1
关掉按块切与挪动配对，确认 selftest 判红。
"""
import difflib
import os
import re
import subprocess
import sys
import tempfile

DEFAULT_SPLIT = r'^(#{1,6} |\||E[0-9]+\|)'
MOVE_HEAD_CHARACTERS = 12
MOVE_SIMILARITY = 0.6


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


def choose(path, match_patterns, foreign_patterns, split_pattern, cwd):
    """返回（新的暂存内容, 进暂存区的块, 留在工作区的块）；每块是（行, 是不是按挪动配上的）。拒绝时抛 SystemExit。"""
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
        if mine:
            result.extend(new_lines)
            staged.append((shown, paired))
        else:
            result.extend(old_lines)
            left.append((shown, False))
    if not staged:
        print(f'  ✗ {path}：没有一块命中 --match')
        print('    → 查一下正则写对没有；这个文件里真没有这一轮的改动，就别把它列进来')
        raise SystemExit(3)
    for block, _ in staged:
        hits = [line for line in block for pattern in foreign_patterns if pattern.search(line)]
        if hits:
            print(f'  ✗ {path}：进暂存区的一块里有命中 --foreign 的行：{hits[0].rstrip()[:90]}')
            print('    → 两个会话的改动挤在同一块里：换一个 --split 把它们切开，或者等对方先提交')
            raise SystemExit(4)
    return ''.join(result), staged, left


def stage(path, content, cwd):
    listing = git(['ls-files', '-s', '--', path], cwd=cwd).decode('utf-8').split()
    mode = listing[0]
    blob = git(['hash-object', '-w', '--stdin'], cwd=cwd, stdin=content.encode('utf-8')).decode('utf-8').strip()
    git(['update-index', '--cacheinfo', f'{mode},{blob},{path}'], cwd=cwd)


def run(argv, cwd):
    match_patterns, foreign_patterns, paths = [], [], []
    split_pattern, dry_run = re.compile(DEFAULT_SPLIT), False
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
        else:
            paths.append(argument)
    if not match_patterns or not paths:
        print('  ✗ 至少要一个 --match 和一个文件')
        print('    → stage-mine.py --match "E137|（其十）" --foreign "E138" 文件…')
        return 2
    total_staged = 0
    for path in paths:
        try:
            content, staged, left = choose(path, match_patterns, foreign_patterns, split_pattern, cwd)
        except SystemExit as refusal:
            return int(refusal.code)
        if not dry_run:
            stage(path, content, cwd)
        total_staged += len(staged)
        print(f'{path}：暂存 {len(staged)} 块，留在工作区 {len(left)} 块' + ('（--dry-run，暂存区没动）' if dry_run else ''))
        for block, paired in staged:
            print(f'  + {block[0].rstrip()[:100]}' + ('（按挪动配上的旧版，一起删）' if paired else ''))
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
        print('selftest: 通过（紧挨着的两块、两行表格、两行登记各切开，只暂存了我的；挪了位置的那一行只留新版；没命中时拒绝）')
        return 0


def main():
    if sys.argv[1:2] == ['--selftest']:
        sys.exit(selftest())
    top = git(['rev-parse', '--show-toplevel']).decode('utf-8').strip()
    sys.exit(run(sys.argv[1:], top))


if __name__ == '__main__':
    main()
