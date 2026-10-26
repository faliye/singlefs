#!/usr/bin/env python3
"""闭包里每个 fn（按 MIR 调用方配到 unit-graph 的 fn 单元）按「直接要什么才进得了 Verus」分类。

用法：CORE_SRC=<core 的 src 目录> closure_classes.py <mir_callees.tsv> <vstd 源码目录> <闭包文件…>
类别（一个 fn 取最重的一类，按 D > C > B > A）：
  A 直接可验    只调 core 自己的函数、drop、? 与下标，和 vstd 有规格的 std 函数；
  B 只差格式    另外用了 format! / panic! / expect / Debug 这类机器（Verus 里 panic 与 expect 是「走不到」的义务，format! 的结果当任意串）；
  C 要改写      调了 vstd 没有规格的 std 函数、迭代器适配器（map / filter / chain / enumerate …）或闭包调用，要么改成循环，要么给 std 函数登记规格；
  D 信任边界    读写设备（BlockDevice / PoolReader 的方法）、校验和（crc32_castagnoli）或哈希：D13 已定项 14 的未解释函数，只能登记成可信声明。
按名字配，不按类型配；闭包里 fn 单元与 MIR 函数的配对只靠名字，重名的可能多配。是估数，不是 Verus 的判。
"""
import collections
import glob
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_src = open(os.path.join(HERE, 'std_coverage.py'), encoding='utf-8').read().split('\ndef run():')[0]
_ns = {'__file__': os.path.join(HERE, 'std_coverage.py')}
exec(compile(_src, 'std_coverage_head', 'exec'), _ns)
vstd_keys = _ns['vstd_keys']
INT_TYPES = _ns['INT_TYPES']

tsv, vroot, *closures = sys.argv[1:]
vkeys, vtrait_methods, _vtypes = vstd_keys(vroot)
defined = {d.strip() for d in open(tsv + '.defined', encoding='utf-8')}
source_types = set()
for f in glob.glob(os.path.join(os.environ['CORE_SRC'], '*.rs')):
    for m in re.finditer(r'^\s*(?:pub(?:\([^)]*\))? )?(?:struct|enum|trait|type|union) (\w+)', open(f, encoding='utf-8').read(), re.M):
        source_types.add(m.group(1))


def spec_exists(key):
    if key in vkeys or key.split('@')[0] in vtrait_methods:
        return True
    if key.startswith('num::'):
        return any(f"{t}::{key[5:]}" in vkeys for t in INT_TYPES)
    return False


# 闭包 fn 单元，按 (文件, 起行, 止行) 去重：{单元id: (名字, 文件, 起行, 止行, 所属 impl 的类型名或 None)}
units = {}
for c in closures:
    for line in open(c, encoding='utf-8'):
        p = line.rstrip('\n').split('\t')
        if p[0] != 'fn':
            continue
        file_path, span = p[2].rsplit(':', 1)
        start, end = (int(x) for x in span.split('-'))
        tail = p[1].split('singlefs_core::', 1)[-1] if p[1].startswith('singlefs_core::') else p[1]
        mt = re.search(r'<impl ([^>]*(?:<[^>]*>)?[^>]*)>::(\w+)$', tail)
        if mt:
            impl_type = re.sub(r'<.*', '', mt.group(1)).split(' for ')[-1].strip()
            name = mt.group(2)
        else:
            impl_type, name = None, tail.split('::')[-1]
        units[(file_path, start, end)] = (name, file_path, start, end, impl_type)

by_name = collections.defaultdict(list)
for uid, (name, file_path, start, end, impl_type) in units.items():
    by_name[name].append(uid)


def candidate_units(header):
    """MIR 函数头 -> 闭包里可能是它的单元 id 列表（配不上交空）。"""
    base = header.split('::{closure')[0]
    m = re.match(r'(?:.*::)?<impl at (crates/singlefs-core/src/[\w./]+):(\d+):\d+: \d+:\d+>::(\w+)$', base)
    if m:
        file_path, impl_line, name = m.group(1), int(m.group(2)), m.group(3)
        found = [uid for uid in by_name.get(name, []) if uid[0] == file_path and uid[1] > impl_line]
        found.sort(key=lambda uid: uid[1])
        return found[:1]
    parts = base.split('::')
    name = parts[-1]
    if len(parts) >= 2 and parts[-2] and parts[-2][0].isupper():
        return [uid for uid in by_name.get(name, []) if units[uid][4] == parts[-2]]
    if len(parts) >= 2:
        stem = parts[-2]
        found = [uid for uid in by_name.get(name, []) if os.path.splitext(os.path.basename(uid[0]))[0] == stem]
        if found:
            return found
    return [uid for uid in by_name.get(name, []) if units[uid][4] is None]


flags_of = collections.defaultdict(set)
matched_headers = 0
header_candidates = {}
for caller, raw, key in (line.rstrip('\n').split('\t') for line in open(tsv, encoding='utf-8')):
    if caller not in header_candidates:
        header_candidates[caller] = candidate_units(caller)
    owners = header_candidates[caller]
    if not owners:
        continue
    flags = set()
    if key == 'drop':
        continue
    if key == 'fn-pointer-or-closure-call':
        flags.add('C')
    else:
        k_self = key.split('@')[-1] if '@' in key else None
        owner_type = key.split('::')[0] if '@' not in key else k_self
        if re.match(r'^(singlefs_format|singlefs_replay_prefix|libc)::', raw):
            pass
        elif re.search(r'crc32_castagnoli|Hash::hash|Hasher', raw):
            flags.add('D')
        elif key.startswith(('BlockDevice::', 'PoolReader::', 'File::', 'FileExt::')) or '@Device' in key or '@dyn' in key or '@Reader' in key:
            flags.add('D')
        elif key in defined or raw in defined or owner_type in source_types:
            pass
        elif re.match(r'^(Formatter|Arguments|Argument|rt)::', key) or key.startswith(('rt::', 'fmt::', 'panicking::')):
            flags.add('B')
        elif key.startswith(('Try::branch', 'FromResidual::from_residual', 'Index::index@', 'IndexMut::index_mut@')):
            pass
        else:
            if key.split('@')[0] in ('Result::expect', 'Option::expect', 'Option::unwrap', 'Result::unwrap'):
                flags.add('B')
            if not spec_exists(key):
                flags.add('C')
    for uid in owners:
        flags_of[uid] |= flags
for caller, owners in header_candidates.items():
    if owners:
        matched_headers += 1

counts = collections.Counter()
lines = collections.Counter()
unmatched_units = 0
mir_touched = set()
for caller, owners in header_candidates.items():
    mir_touched.update(owners)
for uid, (name, file_path, start, end, impl_type) in units.items():
    if uid not in mir_touched:
        unmatched_units += 1
    flags = flags_of.get(uid, set())
    worst = max(flags, key=lambda x: 'ABCD'.index(x)) if flags else 'A'
    counts[worst] += 1
    lines[worst] += end - start + 1

total = sum(counts.values())
print(f"闭包 fn 单元（去重后）{len(units)} 个；MIR 里没配上的单元 {unmatched_units} 个（归 A：空函数、trait 方法的默认体、或名字没配上）")
for k in 'ABCD':
    print(f"  类 {k}: fn 单元 {counts[k]}，行数 {lines[k]}")
print(f"  合计 fn 单元 {total}，行数 {sum(lines.values())}")
by_file = collections.defaultdict(lambda: collections.Counter())
for uid, (name, file_path, start, end, impl_type) in units.items():
    flags = flags_of.get(uid, set())
    worst = max(flags, key=lambda x: "ABCD".index(x)) if flags else "A"
    by_file[os.path.basename(file_path)][worst] += 1
    by_file[os.path.basename(file_path)]["行"] += end - start + 1
print("-- 按文件（fn 单元数 A/B/C/D 与行数，按 fn 数降序）")
for file_name, c in sorted(by_file.items(), key=lambda kv: -sum(kv[1][k] for k in "ABCD")):
    print(f"  {file_name:32s} A={c[chr(65)]:4d} B={c[chr(66)]:4d} C={c[chr(67)]:4d} D={c[chr(68)]:4d} 行={c[chr(34892)]}")
