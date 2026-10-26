#!/usr/bin/env python3
"""第三轮辩方腿 Z4：判器闭包键「去掉 Verus 的规格与证明块」的原型（文本级，只覆盖下面列的几种写法，不是门禁实现）。
对源文件里 verus! { … } 里的内容：
  整条丢：spec fn / proof fn / axiom fn（含 open、closed、pub 等前缀）
  fn 头：丢 requires / ensures / decreases / recommends / opens_invariants / no_unwind 子句（到函数体的 { 之前）；-> (名: 类型) 改成 -> 类型
  fn 体：丢 proof { … }；丢 assert(…) [by (…)? { … }]、assume(…)、admit()、reveal(…)、reveal_with_fuel(…) 语句；丢 let ghost / let tracked 语句；
         循环头里的 invariant / invariant_except_break / ensures / decreases 子句
其余原样。用法：strip_verus.py <输入文件> > <输出文件>
"""
import re, sys

TOKEN = re.compile(r'//[^\n]*|/\*.*?\*/|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\'|\'[A-Za-z_][A-Za-z_0-9]*|[A-Za-z_][A-Za-z_0-9]*!?|[0-9][0-9A-Za-z_.]*|->|=>|::|\.\.=|\.\.|==|!=|<=|>=|&&|\|\||\+=|-=|\*=|/=|%=|<<=|>>=|&=|\|=|\^=|<<|>>|[^\sA-Za-z_0-9]', re.S)
OPEN = {'(': ')', '[': ']', '{': '}'}
CLOSE = {')', ']', '}'}
CLAUSES = {'requires', 'ensures', 'decreases', 'recommends', 'opens_invariants', 'no_unwind', 'invariant', 'invariant_except_break', 'default_ensures'}
GHOST_CALLS = {'assume', 'admit', 'reveal', 'reveal_with_fuel', 'reveal_hide', 'reveal_hide_local', 'assert'}

def tokens(text):
    return [t for t in TOKEN.findall(text) if not t.startswith('//') and not t.startswith('/*')]

def group_end(toks, i):
    """toks[i] 是开括号，返回配对闭括号的下标。"""
    depth = 0
    for j in range(i, len(toks)):
        if toks[j] in OPEN: depth += 1
        elif toks[j] in CLOSE:
            depth -= 1
            if depth == 0: return j
    raise ValueError('括号不配平')

def strip_body_tokens(toks):
    out = []; i = 0
    while i < len(toks):
        t = toks[i]
        if t == 'proof' and i + 1 < len(toks) and toks[i + 1] == '{':
            i = group_end(toks, i + 1) + 1
            continue
        if t in GHOST_CALLS and i + 1 < len(toks) and toks[i + 1] == '(' and (i == 0 or toks[i - 1] not in ('.', '::')):
            j = group_end(toks, i + 1) + 1
            if t == 'assert' and j < len(toks) and toks[j] == 'by':
                j += 1
                if j < len(toks) and toks[j] == '(':
                    j = group_end(toks, j) + 1
                if j < len(toks) and toks[j] == '{':
                    j = group_end(toks, j) + 1
            if j < len(toks) and toks[j] == ';':
                j += 1
            i = j
            continue
        if t == 'let' and i + 1 < len(toks) and toks[i + 1] in ('ghost', 'tracked'):
            j = i
            depth = 0
            while j < len(toks):
                if toks[j] in OPEN: depth += 1
                elif toks[j] in CLOSE: depth -= 1
                elif toks[j] == ';' and depth == 0: break
                j += 1
            i = j + 1
            continue
        if t in CLAUSES and out and out[-1] != '.':
            # 循环头 / fn 头里的子句：到函数体或循环体的 { 之前
            depth = 0; j = i
            while j < len(toks):
                if toks[j] in ('(', '['): depth += 1
                elif toks[j] in (')', ']'): depth -= 1
                elif toks[j] == '{' and depth == 0: break
                j += 1
            i = j
            continue
        if t in OPEN and t == '{':
            j = group_end(toks, i)
            inner = strip_body_tokens(toks[i + 1:j])
            out.extend(['{'] + inner + ['}'])
            i = j + 1
            continue
        out.append(t); i += 1
    return out

def strip_item(toks):
    """一个 verus! 里的条目（含属性）。返回丢完的 token 表。"""
    head_kinds = []
    k = 0
    # 找 fn 关键字前的限定词
    while k < len(toks) and toks[k] != 'fn' and toks[k] not in ('{', ';') and toks[k] not in ('struct', 'enum', 'impl', 'trait', 'const', 'mod', 'use', 'type', 'static'):
        head_kinds.append(toks[k]); k += 1
    if k < len(toks) and toks[k] == 'fn':
        if 'spec' in head_kinds or 'proof' in head_kinds or 'axiom' in head_kinds:
            return []
        out = strip_body_tokens(toks)
        # -> (名: 类型) 改成 -> 类型
        res = []; i = 0
        while i < len(out):
            if out[i] == '->' and i + 1 < len(out) and out[i + 1] == '(' :
                j = group_end(out, i + 1)
                inner = out[i + 2:j]
                if len(inner) >= 2 and inner[1] == ':':
                    res.extend(['->'] + inner[2:]); i = j + 1; continue
            res.append(out[i]); i += 1
        return res
    if k < len(toks) and toks[k] in ('impl', 'trait', 'mod'):
        # 容器：递归处理花括号里的条目
        b = toks.index('{') if '{' in toks else None
        if b is None: return toks
        e = group_end(toks, b)
        return toks[:b] + ['{'] + strip_items(toks[b + 1:e]) + ['}'] + toks[e + 1:]
    return toks

def split_items(toks):
    items = []; i = 0
    while i < len(toks):
        start = i
        depth = 0
        while i < len(toks):
            t = toks[i]
            if t in OPEN and t != '{': depth += 1
            elif t in (')', ']'): depth -= 1
            elif t == '{' and depth == 0:
                i = group_end(toks, i) + 1
                # struct/enum/impl 等以 } 结尾；fn 体也是
                break
            elif t == ';' and depth == 0:
                i += 1; break
            i += 1
        items.append(toks[start:i])
    return items

def strip_items(toks):
    out = []
    for item in split_items(toks):
        out.extend(strip_item(item))
    return out

def strip_source(text):
    """只改 verus! { … } 里面，块外的文本原样不动。"""
    out = []; pos = 0
    for m in re.finditer(r'verus!\s*\{', text):
        if m.start() < pos: continue
        # 用词法器从 { 起配括号，拿到块的字符区间
        depth = 0; end = None; inner_start = m.end()
        for tm in TOKEN.finditer(text, m.end() - 1):
            tk = tm.group(0)
            if tk in ('{',): depth += 1
            elif tk == '}':
                depth -= 1
                if depth == 0: end = tm.start(); break
        if end is None: raise ValueError('verus! 块没配平')
        inner = text[inner_start:end]
        stripped = '\n'.join(strip_items(tokens(inner))) + '\n'
        out.append(text[pos:inner_start]); out.append('\n' + stripped); pos = end
    out.append(text[pos:])
    return ''.join(out)

if __name__ == '__main__':
    sys.stdout.write(strip_source(open(sys.argv[1], encoding='utf-8').read()))
