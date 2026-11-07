#!/usr/bin/env python3
"""T4 的文本模型：第 88 批产品 GPU 包七份文件的补丁块，按词法单元对齐删去行与加进行，
列出每一处被换掉的标识符（旧名 -> 新名）与对不上的残余；再在冻结副本里数旧名还剩几处、新名改前是否已有。
只读文件，不编、不跑项目代码。
用法：t4_token_alignment.py <改动全文 md> <冻结副本根>
"""
import difflib
import re
import sys
from collections import defaultdict
from pathlib import Path

TOKEN = re.compile(
    r'r#*"(?:.|\n)*?"#*|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\'|[A-Za-z_][A-Za-z0-9_]*|\d[\d_]*(?:\.\d+)?[a-z0-9]*|::|->|=>|<<|>>|\S'
)


def strip_comment(line):
    """去掉 // 起的注释（含 /// 文档注释）；串里的 // 不算。"""
    out, in_string, escaped = [], False, False
    for position, character in enumerate(line):
        if in_string:
            out.append(character)
            if escaped:
                escaped = False
            elif character == '\\':
                escaped = True
            elif character == '"':
                in_string = False
            continue
        if character == '"':
            in_string = True
            out.append(character)
            continue
        if line.startswith('//', position):
            break
        out.append(character)
    return ''.join(out)


def tokens(lines):
    joined = '\n'.join(strip_comment(line) for line in lines)
    return TOKEN.findall(joined)


def batch_eighty_eight_sections(markdown):
    text = Path(markdown).read_text(encoding='utf-8')
    start = text.index('## 第 88 批')
    sections = {}
    current = None
    for line in text[start:].splitlines():
        heading = re.match(r'^### (crates/\S+)$', line)
        if heading:
            current = heading.group(1)
            sections[current] = {'removed': [], 'added': []}
            continue
        if current is None or line.startswith('---') or line.startswith('+++'):
            continue
        if line.startswith('-'):
            sections[current]['removed'].append(line[1:])
        elif line.startswith('+'):
            sections[current]['added'].append(line[1:])
    return sections


def identifier(token):
    return re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', token) is not None


def main():
    markdown, frozen = sys.argv[1], Path(sys.argv[2])
    for path, section in batch_eighty_eight_sections(markdown).items():
        old, new = tokens(section['removed']), tokens(section['added'])
        renames = defaultdict(set)
        residue = []
        matcher = difflib.SequenceMatcher(a=old, b=new, autojunk=False)
        for kind, old_start, old_end, new_start, new_end in matcher.get_opcodes():
            if kind == 'equal':
                continue
            old_part, new_part = old[old_start:old_end], new[new_start:new_end]
            if (kind == 'replace' and len(old_part) == len(new_part)
                    and all(identifier(a) and identifier(b) for a, b in zip(old_part, new_part))):
                for a, b in zip(old_part, new_part):
                    renames[a].add(b)
            else:
                residue.append((kind, ' '.join(old_part), ' '.join(new_part)))
        source = (frozen / path).read_text(encoding='utf-8')
        source_tokens = tokens(source.splitlines())
        print(f'== {path} removed_tokens={len(old)} added_tokens={len(new)}')
        for name in sorted(renames):
            targets = sorted(renames[name])
            left = source_tokens.count(name)
            print(f'rename {name} -> {",".join(targets)} one_to_one={len(targets) == 1} '
                  f'old_name_left_in_frozen_file={left}')
        for kind, old_text, new_text in residue:
            print(f'residue {kind} old=[{old_text[:300]}] new=[{new_text[:300]}]')
    removed_copy_against_the_kept_copy(markdown, frozen)
    return 0


def removed_copy_against_the_kept_copy(markdown, frozen):
    """region_record_kernels.rs 删掉的那份 low_and_high_words_of_the_slot 与 live_slot_kernels.rs 留下的那份，去注释后逐个词法单元比
    （留下的那份去掉 pub(crate) 再比）。"""
    diff = Path(markdown).read_text(encoding='utf-8').splitlines()
    section = diff.index('### crates/singlefs-background-gpu/src/region_record_kernels.rs')
    start = diff.index('-fn low_and_high_words_of_the_slot(slot: SlotNumber) -> [u32; 2] {', section)
    removed = []
    for line in diff[start:]:
        if not line.startswith('-'):
            break
        removed.append(line[1:])
    removed_tokens = tokens(removed)
    source = (frozen / 'crates/singlefs-background-gpu/src/live_slot_kernels.rs').read_text(encoding='utf-8').splitlines()
    begin = next(i for i, line in enumerate(source) if 'fn low_and_high_words_of_the_slot(' in line)
    end = next(i for i in range(begin, len(source)) if source[i] == '}')
    kept_tokens = tokens(source[begin:end + 1])
    if kept_tokens[:4] == ['pub', '(', 'crate', ')']:
        kept_tokens = kept_tokens[4:]
    print(f'== low_and_high_words_of_the_slot removed_copy_diff_lines={start + 1}-{start + len(removed)} '
          f'kept_copy_live_slot_kernels_lines={begin + 1}-{end + 1} identical_after_dropping_pub_crate={removed_tokens == kept_tokens}')


if __name__ == '__main__':
    sys.exit(main())
