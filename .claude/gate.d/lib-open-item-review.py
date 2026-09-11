#!/usr/bin/env python3
"""60 号阶段的复核判据：一条未定项对它点名的决策 Dn，最近一次「复核」是什么时候。

复核 = 条目块历史里**最近一次让 Dn 的点名次数变多**的那次提交（条目诞生那次也算）；
工作区里条目块比 HEAD 多点了 Dn 一次，算刚复核过。改别的地方不算。

为什么不是「条目块最后一次改动」：那样任何一次改动都能把红消掉。实测（2026-09-11）：
D19（块指针的结构与宽度预算） 未定项 6 那一格因 D16 的状态变动被标红，同日往那一格补了一段性能数（与 D16 无关），
红就没了——D16 那两句还成不成立是事后手核的，门禁没逼。

条目块：从分项首行起，到下一条分项（编号列表项或「| k |」表格行）或下一个标题之前。
表格行一行就是一条分项，不再像改前那样一直延到下一个标题、把后面几行分项都算进来。

用法：lib-open-item-review.py <文件> <工作区里条目首行的行号> <Dn:状态变动时间>…
打印没复核过的那几个 Dn，一行一个。
"""
import re
import subprocess
import sys

ITEM_BOUNDARY = re.compile(r'^\s*\d+\.\s|^### |^## |^\|\s*\d+\s*\|')


def block_end(lines, start_index):
    for index in range(start_index + 1, len(lines)):
        if ITEM_BOUNDARY.match(lines[index]):
            return index - 1
    return min(start_index + 20, len(lines) - 1)


def mention_count(decision, text):
    return len(re.findall(r'(?<![A-Za-z0-9])' + decision + r'(?!\d)', text))


def head_start_index(head_lines, work_first_line):
    for index, line in enumerate(head_lines):
        if line == work_first_line:
            return index
    # 首行在工作区里改过：按同一个分项号在 HEAD 的「### 未定项」那一节里找
    prefix = re.match(r'^(\|\s*\d+\s*\||\s*\d+\.\s)', work_first_line)
    if not prefix:
        return None
    inside = False
    for index, line in enumerate(head_lines):
        if re.match(r'^### 未定项\s*$', line):
            inside = True
            continue
        if re.match(r'^#{2,4} ', line):
            inside = False
        if inside and line.startswith(prefix.group(1)):
            return index
    return None


def history_of_block(path, first_line_number, last_line_number):
    """条目块的历史（HEAD 里第 first..last 行，从 1 数）：[(提交时间, 改前, 改后)]，新的在前。"""
    log = subprocess.run(['git', 'log', '-L', f'{first_line_number},{last_line_number}:{path}', '--format=%x00%ct'],
                         capture_output=True, text=True).stdout
    history = []
    for chunk in log.split('\x00')[1:]:
        lines = chunk.split('\n')
        try:
            committed = int(lines[0].strip())
        except ValueError:
            continue
        before, after, inside_hunk = [], [], False
        for line in lines[1:]:
            if line.startswith('@@'):
                inside_hunk = True
                continue
            if not inside_hunk:
                continue
            if line.startswith('-'):
                before.append(line[1:])
            elif line.startswith('+'):
                after.append(line[1:])
            elif line.startswith(' '):
                before.append(line[1:])
                after.append(line[1:])
        history.append((committed, '\n'.join(before), '\n'.join(after)))
    return history


def main():
    path, work_line_number = sys.argv[1], int(sys.argv[2])
    dependencies = [(entry.split(':')[0], int(entry.split(':')[1])) for entry in sys.argv[3:]]
    head = subprocess.run(['git', 'show', f'HEAD:{path}'], capture_output=True, text=True)
    if head.returncode != 0:
        return   # 文件还没进过 HEAD：比任何决策都新，无从陈旧
    head_lines = head.stdout.split('\n')
    with open(path, encoding='utf-8') as handle:
        work_lines = handle.read().split('\n')
    work_start = work_line_number - 1
    head_start = head_start_index(head_lines, work_lines[work_start])
    if head_start is None:
        return   # HEAD 里没有这条分项 = 新加的，无从陈旧
    head_end = block_end(head_lines, head_start)
    work_end = block_end(work_lines, work_start)
    head_block = '\n'.join(head_lines[head_start:head_end + 1])
    work_block = '\n'.join(work_lines[work_start:work_end + 1])
    history = history_of_block(path, head_start + 1, head_end + 1)
    if not history:
        return   # 取不到历史：判不了，不报（与改前同口径）
    for decision, changed_at in dependencies:
        if mention_count(decision, work_block) > mention_count(decision, head_block):
            continue   # 工作区里刚补了一句点名它的复核
        reviewed_at = next((committed for committed, before, after in history
                            if mention_count(decision, after) > mention_count(decision, before)), 0)
        if changed_at > reviewed_at:
            print(decision)


if __name__ == '__main__':
    main()
