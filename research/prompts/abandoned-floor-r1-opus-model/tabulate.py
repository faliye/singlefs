#!/usr/bin/env python3
"""把 AF-CELL 行汇成表：每个实验、每个候选，抬 F 之后与各段后缀里出现过的违例集合与格数。"""
import re, sys, collections

cell_re = re.compile(r'AF-CELL cand=(\S+) abF=(\S+) exp=(\S+) (.*)$')
verdict_re = re.compile(r'=>(\[[^\]]*\])( NA\[[^\]]*\])?')

def parse(paths):
    for path in paths:
        for line in open(path, errors='replace'):
            m = cell_re.search(line)
            if m:
                yield m.group(1), m.group(2), m.group(3), m.group(4)

def fields(rest):
    out = {}
    for token in rest.split(' '):
        if '=' in token and not token.startswith('raise') and not token.startswith('post'):
            k, v = token.split('=', 1)
            if k not in out:
                out[k] = v
    return out

def main():
    mode = sys.argv[1]
    paths = sys.argv[2:]
    table = collections.defaultdict(collections.Counter)
    examples = {}
    for cand, abf, exp, rest in parse(paths):
        if exp.endswith('-setup'):
            continue
        f = fields(rest)
        raise_part = rest.split(' raise=', 1)[1].split(' post=', 1)[0] if ' raise=' in rest else ''
        post_part = rest.split(' post=[', 1)[1] if ' post=[' in rest else ''
        raise_ok = raise_part.startswith('ok')
        after = verdict_re.search(raise_part)
        after_set = after.group(1) + (after.group(2) or '') if after else '-'
        posts = collections.OrderedDict()
        for seg in post_part.rstrip(']').split(' | '):
            if ':' not in seg:
                continue
            name, body = seg.split(':', 1)
            sets = [a + (b or '') for a, b in verdict_re.findall(body)]
            posts[name] = '>'.join(sets) if sets else body[:60]
        before = f.get('before', '?')
        key = (exp, cand, abf)
        if mode == 'raise':
            status = 'ok' if raise_ok else ('refused' if raise_part else 'none')
            reds = set()
            if raise_ok and after_set != '[]':
                reds.add('after_raise:' + after_set)
            for name, sets in posts.items():
                for s in sets.split('>'):
                    if s and s != '[]' and not s.startswith('rollback') and not s.startswith('raise'):
                        reds.add(re.sub(r'\d+', 'N', name) + ':' + s)
            label = f"raise={status} before={before} " + (' ; '.join(sorted(reds)) if reds else 'all-green')
            table[key][label] += 1
            ex_key = (key, label)
            if ex_key not in examples:
                examples[ex_key] = ' '.join(f"{k}={v}" for k, v in f.items() if k in ('variant','remount','k','F','mode','hide_syspre','base','entry','landed_F','roots_at_F','all_abandoned','txg_before'))
    for key in sorted(table):
        print(f"== exp={key[0]} cand={key[1]} abF={key[2]}")
        for label, n in table[key].most_common():
            print(f"  {n:4d}  {label}   e.g. {examples[(key,label)]}")

main()
