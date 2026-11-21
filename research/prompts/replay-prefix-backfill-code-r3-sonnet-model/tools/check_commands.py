import re, sys
root = sys.argv[1]
doc = open(root + '/crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md', encoding='utf-8').read()
block = doc.split('```sh', 1)[1].split('```', 1)[0]
# logical commands: a SINGLEFS_RUN_SCOPE line starts one; continuation lines end with backslash
lines = block.split('\n')
cmds, cur = [], None
for ln in lines:
    if ln.startswith('SINGLEFS_RUN_SCOPE='):
        if cur: cmds.append(cur)
        cur = ln
    elif cur is not None and ln.strip():
        cur += ' ' + ln.strip()
if cur: cmds.append(cur)
def test_fns(path):
    """return list of (name, line) of fn directly after #[test] (attribute stack allowed)"""
    rows = open(path, encoding='utf-8').read().split('\n')
    out = []
    pending = False
    for i, r in enumerate(rows, 1):
        s = r.strip()
        if s.startswith('#[test]'):
            pending = True
            continue
        if pending and s.startswith('#['):
            continue
        if pending:
            m = re.match(r'(?:pub\s+)?(?:async\s+)?fn\s+(\w+)', s)
            if m:
                out.append((m.group(1), i))
            pending = False
    return out
for c in cmds:
    scope = re.match(r"SINGLEFS_RUN_SCOPE='([^']*)'", c).group(1)
    rest = c.split("task-server/task submit -- ", 1)[1]
    if not rest.startswith('cargo test'):
        print('SKIP(not cargo test):', rest[:90]); continue
    package = re.search(r'-p (\S+)', rest).group(1)
    names = [t for t in rest.split(' -- ', 1)[1].replace('--exact', '').split() if t != '\\']
    declared = re.search(r'范围=(\d+) 个测试', scope)
    declared = int(declared.group(1)) if declared else None
    if ' --lib ' in rest:
        target = '--lib'
    else:
        target = re.search(r'--test (\S+)', rest).group(1)
    ok, bad = 0, []
    import collections
    dup = [n for n, k in collections.Counter(names).items() if k > 1]
    per_module = collections.Counter()
    for n in names:
        if target == '--lib':
            mod, _, fn = n.split('::')
            path = f'{root}/crates/{package}/src/{mod}.rs'
            fns = dict(test_fns(path))
            per_module[mod] += 1
        else:
            fn = n
            path = f'{root}/crates/{package}/tests/{target}.rs'
            fns = dict(test_fns(path))
        if fn in fns: ok += 1
        else: bad.append(n)
    print(f'package={package} target={target} names={len(names)} declared_scope={declared} found_as_test_fn={ok} missing={bad} duplicates={dup}')
    if target == '--lib':
        for mod, k in per_module.items():
            allfns = test_fns(f'{root}/crates/{package}/src/{mod}.rs')
            print(f'  module {mod}: listed={k} test_fns_in_file={len(allfns)} unlisted={[n for n,_ in allfns if f"{mod}::tests::{n}" not in names]}')
    else:
        allfns = test_fns(f'{root}/crates/{package}/tests/{target}.rs')
        print(f'  test_fns_in_file={len(allfns)} unlisted={[n for n,_ in allfns if n not in names]}')
