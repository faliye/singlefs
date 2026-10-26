#!/usr/bin/env python3
"""从 `rustc --emit=mir` 的输出里抽出每个 MIR 函数调用了谁。

用法：mir_callees.py <.mir 文件> <输出 tsv>
输出每行：调用方 MIR 函数名 <tab> 被调用者原文 <tab> 归一化后的键（类型::方法 或 特质::方法@自身类型）
被调用者原文是 MIR 打印的路径；归一化只去掉泛型实参，不做类型解析（所以按名字配，不按类型配）。
"""
import re
import sys


def strip_generics(text):
    """去掉所有 <...>（认 '->' 不当收尾），保留 '<impl ...>' 与 '<X as T>' 由调用方先处理。"""
    out = []
    depth = 0
    i = 0
    while i < len(text):
        c = text[i]
        if c == '<':
            depth += 1
        elif c == '>' and not (i > 0 and text[i - 1] == '-'):
            depth = max(0, depth - 1)
        elif depth == 0:
            out.append(c)
        i += 1
    return ''.join(out)


def split_callee(line):
    """从一行 MIR 语句里切出被调用者表达式；不是调用返回 None。"""
    if '-> [return:' not in line and '-> unwind' not in line:
        return None
    eq = line.find(' = ')
    start = eq + 3 if eq >= 0 else 0
    s = line[start:].strip()
    depth_angle = 0
    i = 0
    while i < len(s):
        c = s[i]
        if c == '<':
            depth_angle += 1
        elif c == '>' and not (i > 0 and s[i - 1] == '-'):
            depth_angle = max(0, depth_angle - 1)
        elif c == '(' and depth_angle == 0:
            return s[:i]
        elif c == '{' and depth_angle == 0 and s.startswith('{'):
            return None
        i += 1
    return None


def normalize(callee):
    c = callee.strip()
    if c.startswith(('move ', 'copy ')) or re.match(r'^_\d+$', c):
        return 'fn-pointer-or-closure-call'
    if c.startswith('const '):
        c = c[6:]
    if c.startswith('<'):
        # <SELF as TRAIT>::rest
        depth = 0
        end = None
        for i, ch in enumerate(c):
            if ch == '<':
                depth += 1
            elif ch == '>' and not (i > 0 and c[i - 1] == '-'):
                depth -= 1
                if depth == 0:
                    end = i
                    break
        inner = c[1:end]
        rest = c[end + 1:]
        m = re.search(r'\bas\b', inner)
        # 自身类型 = 第一个顶层 ' as ' 之前
        d = 0
        split_at = None
        for i, ch in enumerate(inner):
            if ch == '<':
                d += 1
            elif ch == '>' and not (i > 0 and inner[i - 1] == '-'):
                d -= 1
            elif d == 0 and inner[i:i + 4] == ' as ':
                split_at = i
                break
        if split_at is None:
            self_ty, trait = inner, ''
        else:
            self_ty, trait = inner[:split_at], inner[split_at + 4:]
        self_ty = strip_generics(self_ty).strip()
        self_last = re.split(r'::', self_ty)[-1] if '::' in self_ty else self_ty
        trait_last = strip_generics(trait).split('::')[-1].strip()
        pieces = [x for x in strip_generics(rest).split('::') if x]
        method = pieces[-1].strip() if pieces else ''
        if self_ty.startswith('[') or self_ty.startswith('&['):
            self_last = 'slice'
        return f"{trait_last}::{method}@{self_last}" if trait_last else f"{self_last}::{method}"
    # 路径：先把 <impl ...> 变成 impl
    c2 = re.sub(r'<impl [^>]*(?:\[[^\]]*\])?[^>]*>', '<impl>', c)
    parts = strip_generics(c2.replace('<impl>', 'IMPL')).split('::')
    parts = [p for p in parts if p]
    if 'IMPL' in parts:
        k = parts.index('IMPL')
        owner = parts[k - 1] if k > 0 else 'IMPL'
        return f"{owner}::{parts[-1]}"
    if len(parts) >= 2:
        return f"{parts[-2]}::{parts[-1]}"
    return parts[-1] if parts else c


def main():
    src, out = sys.argv[1], sys.argv[2]
    current = None
    rows = []
    defined = []
    with open(src, encoding='utf-8') as f:
        for raw in f:
            if raw.startswith('fn '):
                m = re.match(r'fn (.+?)\(_?\d*:?', raw)
                name = raw[3:].split('(', 1)[0] if m is None else m.group(1)
                # 带 <impl at ...> 的名字里有括号，整段取到 '(_1' 之前
                m2 = re.match(r'fn (.*?)\((?:_\d+:|\)|\s)', raw)
                current = m2.group(1) if m2 else name
                defined.append(current)
                continue
            if current is None:
                continue
            callee = split_callee(raw)
            if callee:
                rows.append((current, callee.strip(), normalize(callee)))
    with open(out, 'w', encoding='utf-8') as f:
        for r in rows:
            f.write('\t'.join(r) + '\n')
    with open(out + '.defined', 'w', encoding='utf-8') as f:
        for d in defined:
            f.write(d + '\n')
    print(f"mir fn={len(defined)} calls={len(rows)}")


main()
