#!/usr/bin/env python3
"""攻方腿探针：拿仓里现存的 .md、.tsv、.txt 真行逐份喂判据库 task-and-session-numbers 那一形态（all_hits，库自己的排除前缀与后缀判法照用），
数每个目录、每条判据行命中多少处、落在多少行，再按几类「多半不是号」的启发式各数一遍、各列至多 8 行样本。
启发式只是这份探针自己的分法（不是库的判法）：
  batch-size        批号那一条命中、前面紧挨 seq / 追加 / 覆写 / 攒 / 写入 / 流数 N（fsync 攒批的大小，是参数不是批号）
  chinese-ordinal   批号那一条命中的是「第<中文数字>批」（「第一批字节」这类泛指的次序，也有调度批号，要人看样本）
  task-count        任务号那一条命中、数字后面跟 个 / 条 / 份 / 项 / 次 / 件（任务的个数）
  bare-short-code   会话短号那一条命中、前后都不挨会话词（「(1a)」「（a6）」这类步号与标签，也可能是真短号，要人看样本）
  ab-without-colon  任务号那一条命中、同一行有「A/B 读数」却没有被放过（「A/B 读数任务 N」「**A/B 读数**：」「A/B读数：」）
用法：python3 scan_real_lines.py --library <lib-forbidden-notations.py> --repo <仓根> [--directory <相对仓根的目录> …]
退出码：0 扫完；2 用法错。"""
import argparse
import importlib.util
import os
import re
import sys

FORM = 'task-and-session-numbers'
DEFAULT_DIRECTORIES = ('.claude/kb', '.claude/rules', '.claude/agents', '.claude/skills', 'research/prompts', 'records')
NOT_READ = ('research/prompts/no-task-numbers-governance-r1-sonnet-output.md', 'research/prompts/no-task-numbers-governance-r1-sonnet-model/',
            'research/prompts/no-task-numbers-governance-r1-opus-output.md', 'research/prompts/no-task-numbers-governance-r1-opus-model/')
SUFFIXES = ('.md', '.tsv', '.txt')
SAMPLES_PER_CLASS = 8
BATCH_SIZE_BEFORE = re.compile(r'(?:seq|追加|覆写|攒|写入|流数\s*\d+)\s*$')
CHINESE_ORDINAL = re.compile(r'第\s*[一二三四五六七八九十百零两]+\s*批')
TASK_COUNT_AFTER = re.compile(r'\s*[个条份项次件]')
SESSION_CONTEXT = re.compile(r'(?:会话|给|问|告诉|通知|发给|交给|转给)\s*[「“"]?\s*$')


def load_library(path):
    spec = importlib.util.spec_from_file_location('lib_forbidden_notations', path)
    library = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(library)
    return library


def classes_of(line, column, name, matched):
    before, after = line[:column], line[column + len(matched):]
    found = []
    if name == '批号' and BATCH_SIZE_BEFORE.search(before):
        found.append('batch-size')
    if name == '批号' and CHINESE_ORDINAL.fullmatch(matched.strip()):
        found.append('chinese-ordinal')
    if name == '任务服务的任务号' and TASK_COUNT_AFTER.match(after):
        found.append('task-count')
    if name == '括号里的会话短号' and not SESSION_CONTEXT.search(before) and matched.lstrip()[:1] in '(（':
        found.append('bare-short-code')
    if name == '任务服务的任务号' and 'A/B' in line and '读数' in line:
        found.append('ab-without-colon')
    return found


def text_files(repo, directories):
    for directory in directories:
        for base, subdirectories, names in os.walk(os.path.join(repo, directory)):
            subdirectories.sort()
            for file_name in sorted(names):
                if not file_name.lower().endswith(SUFFIXES):
                    continue
                relative = os.path.relpath(os.path.join(base, file_name), repo)
                if relative.startswith(NOT_READ):
                    continue
                yield directory, relative


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--library', required=True)
    parser.add_argument('--repo', required=True)
    parser.add_argument('--directory', action='append')
    arguments = parser.parse_args()
    library = load_library(arguments.library)
    counts, line_counts, class_counts, samples, scanned = {}, {}, {}, {}, {}
    for directory, relative in text_files(arguments.repo, arguments.directory or DEFAULT_DIRECTORIES):
        scanned[directory] = scanned.get(directory, 0) + 1
        if library.is_excluded(FORM, relative):
            continue
        with open(os.path.join(arguments.repo, relative), encoding='utf-8', errors='replace') as handle:
            text = handle.read()
        lines = text.split('\n')
        hit_lines = set()
        for number, column, name, matched in library.all_hits(FORM, text, relative):
            counts[(directory, name)] = counts.get((directory, name), 0) + 1
            hit_lines.add((name, number))
            for kind in classes_of(lines[number - 1], column, name, matched):
                class_counts[(directory, kind)] = class_counts.get((directory, kind), 0) + 1
                bucket = samples.setdefault(kind, [])
                if len(bucket) < SAMPLES_PER_CLASS:
                    bucket.append(f'{relative}:{number} matched={matched!r} text={lines[number - 1][:160]!r}')
        for name, _ in hit_lines:
            line_counts[(directory, name)] = line_counts.get((directory, name), 0) + 1
    for directory, files in sorted(scanned.items()):
        print(f'name=scan_files directory={directory} files={files}')
    for (directory, name), hits in sorted(counts.items()):
        print(f'name=scan_count directory={directory} row={name} hits={hits} lines={line_counts[(directory, name)]}')
    for (directory, kind), hits in sorted(class_counts.items()):
        print(f'name=scan_class directory={directory} class={kind} hits={hits}')
    for kind, bucket in sorted(samples.items()):
        for sample in bucket:
            print(f'name=scan_sample class={kind} {sample}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
