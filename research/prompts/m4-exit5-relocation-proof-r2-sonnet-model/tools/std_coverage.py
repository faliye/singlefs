#!/usr/bin/env python3
"""core 调用的 std 函数，vstd 的规格罩到多少（按名字配的估数，不是 Verus 的判）。

用法：CORE_SRC=<core 的 src 目录> std_coverage.py <mir_callees.tsv> <vstd 源码目录> [闭包文件…]
闭包文件是 unit-graph closure 的输出；给了就只数闭包里的 fn（MIR 函数名按末段或 类型::方法 匹配到 fn 单元）。
匹配键：类型::方法，或 特质::方法@自身类型。vstd 一侧：assume_specification 的方括号里那一项（整数宏占位 $uN/$iN 展开成各整数类型），
外加 external_trait_specification / external_trait_extension 里的特质方法（按 特质::方法 配，不管自身类型）。
"""
import collections
import glob
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
INT_TYPES = ['u8', 'u16', 'u32', 'u64', 'u128', 'usize', 'i8', 'i16', 'i32', 'i64', 'i128', 'isize']

# mir_callees.py 的 main() 在 import 时会跑，所以只取它 main() 之前的函数定义
_head = open(os.path.join(HERE, 'mir_callees.py'), encoding='utf-8').read().split('\ndef main():')[0]
_ns = {}
exec(compile(_head, 'mir_callees_head', 'exec'), _ns)
normalize = _ns['normalize']


def vstd_keys(root):
    keys = set()
    trait_methods = set()
    types = set()
    names = ['slice.rs', 'array.rs', 'string.rs', 'vstd.rs', 'hash_map.rs', 'hash_set.rs']
    files = glob.glob(os.path.join(root, 'std_specs', '*.rs')) + [os.path.join(root, n) for n in names]
    for f in files:
        if not os.path.exists(f):
            continue
        text = open(f, encoding='utf-8').read()
        for m in re.finditer(r'assume_specification\s*(?:<[^\[]*?>)?\s*\[\s*(.+?)\s*\]\s*\(', text, re.S):
            raw = re.sub(r'\s+', ' ', m.group(1))
            k = normalize(raw)
            if '$uN' in k or '$iN' in k:
                for t in INT_TYPES:
                    keys.add(k.replace('$uN', t).replace('$iN', t))
            else:
                keys.add(k)
        for m in re.finditer(r'external_type_specification\]\s*(?:#\[[^\]]*\]\s*)*pub struct Ex\w+\s*(?:<[^>]*>)?\(\s*([\w:]+)', text, re.S):
            types.add(m.group(1).split('::')[-1])
        for m in re.finditer(r'pub trait Ex(\w+)[^{]*\{(.*?)\n\}', text, re.S):
            for fm in re.finditer(r'\n\s*(?:pub )?fn (\w+)', m.group(2)):
                trait_methods.add(f"{m.group(1)}::{fm.group(1)}")
    return keys, trait_methods, types


def closure_matcher(closure_files):
    units = set()
    for c in closure_files:
        for line in open(c, encoding='utf-8'):
            p = line.rstrip('\n').split('\t')
            if p[0] == 'fn':
                units.add(p[1])
    wanted = set()
    for u in units:
        tail = u.split('singlefs_core::', 1)[-1]
        mt = re.search(r'<impl ([^>]*(?:<[^>]*>)?[^>]*)>::(\w+)$', tail)
        if mt:
            ty = re.sub(r'<.*', '', mt.group(1)).split(' for ')[-1].strip()
            wanted.add(f"{ty}::{mt.group(2)}")
        else:
            wanted.add(tail.split('::')[-1])

    def in_closure(caller):
        base = caller.split('::{closure')[0]
        last2 = '::'.join(base.split('::')[-2:])
        last1 = base.split('::')[-1]
        return base in wanted or last2 in wanted or last1 in wanted
    return in_closure, len(units)


def run():
    args = sys.argv[1:]
    tsv, vroot, closures = args[0], args[1], args[2:]
    vkeys, vtrait_methods, vtypes = vstd_keys(vroot)
    defined = [d.strip() for d in open(tsv + '.defined', encoding='utf-8')]
    own_names = set()
    for d in defined:
        own_names.add(d)
        if '::' in d and '<impl at' not in d:
            own_names.add('::'.join(d.split('::')[-2:]))
    source_types = set()
    srcdir = os.environ.get('CORE_SRC')
    if srcdir:
        for f in glob.glob(os.path.join(srcdir, '*.rs')):
            text = open(f, encoding='utf-8').read()
            for m in re.finditer(r'^\s*(?:pub(?:\([^)]*\))? )?(?:struct|enum|trait|type|union) (\w+)', text, re.M):
                source_types.add(m.group(1))
    rows = [line.rstrip('\n').split('\t') for line in open(tsv, encoding='utf-8')]
    def spec_exists(key):
        if key in vkeys or key.split('@')[0] in vtrait_methods:
            return True
        if key.startswith('num::'):
            return any(f"{t}::{key[5:]}" in vkeys for t in INT_TYPES)
        return False

    in_closure = None
    nunits = None
    if closures:
        in_closure, nunits = closure_matcher(closures)
    cls = collections.Counter()
    std_calls = collections.Counter()
    std_callers = collections.defaultdict(set)
    uncovered = collections.Counter()
    callers = set()
    for caller, raw, key in rows:
        if in_closure is not None and not in_closure(caller):
            continue
        callers.add(caller)
        if key == 'drop':
            cls['drop 胶水'] += 1
            continue
        if key == 'fn-pointer-or-closure-call':
            cls['函数指针或闭包调用'] += 1
            continue
        if re.match(r'^(singlefs_format|singlefs_replay_prefix|libc)::', raw):
            cls['别的 crate（format / replay-prefix / libc）'] += 1
            continue
        k_self = key.split('@')[-1] if '@' in key else None
        owner = key.split('::')[0] if '@' not in key else k_self
        if key in own_names or raw in own_names or re.sub(r'::<.*', '', raw) in own_names or owner in source_types:
            cls['core 自己的类型与函数（含 derive 出来的特质方法）'] += 1
            continue
        if re.match(r'^(Formatter|Arguments|Argument|rt)::', key) or key.startswith(('rt::', 'fmt::', 'panicking::')):
            cls['格式化与 panic 机器（format! / panic! / Debug）'] += 1
            continue
        if key.startswith(('Try::branch', 'FromResidual::from_residual', 'Index::index@', 'IndexMut::index_mut@')):
            cls['Verus 原生支持（? 与下标；探针 q01 q02 r05 r17 过）'] += 1
            continue
        cls['std / core / alloc'] += 1
        std_calls[key] += 1
        std_callers[key].add(caller)
        if not spec_exists(key):
            uncovered[key] += 1
    print(f"范围：{'闭包（%d 个 fn 单元）' % nunits if closures else '整个 core'}；MIR 调用方函数 {len(callers)}；vstd 规格键 {len(vkeys)}，特质方法 {len(vtrait_methods)}，external_type_specification 类型 {len(vtypes)}")
    for k, v in cls.most_common():
        print(f"  类别 {k}: {v} 处调用")
    keys = list(std_calls)
    covered = [k for k in keys if spec_exists(k)]
    print(f"std 相关的不同调用键 {len(keys)}；按名字配上 vstd 规格 {len(covered)}；没配上 {len(keys) - len(covered)}")
    total = sum(std_calls.values())
    covered_calls = sum(v for k, v in std_calls.items() if spec_exists(k))
    print(f"调用点 {total}；落在配上规格的键上 {covered_calls}；落在没配上的键上 {total - covered_calls}")
    print('-- 没配上规格的键，按调用点数排（前 70；调用点数 / 涉及的调用方函数数）')
    for k, v in uncovered.most_common(70):
        print(f"  {v:5d} / {len(std_callers[k]):4d}  {k}")


run()
