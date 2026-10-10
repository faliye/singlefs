import re,sys
# 26-2 #8 的钉值（crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs 的 assert_checker_counts），快档 46 个状态。
every, root, empty = 46, 9, 9
only_new_root = "I-1.10 I-3.1 I-3.9 I-3.11 I-5.2 I-5.4 I-9.1 I-9.2 I-9.4 I-9.6 I-9.7 I-9.10 I-9.12 I-9.13 I-9.15 MAPPING I-9.25 I-9.27 I-3.13 I-9.26 I-5.5 I-3.5".split()
never = "I-9.14 I-8.7 I-8.8 I-7.9 I-3.12 I-9.28".split()
exp = {}
for line in open(sys.argv[1]):
    m = re.match(r'QUICKINV (\S+)=(\d+)/(\d+)/(\d+)', line)
    if not m: continue
    inv, ev, vi, na = m.group(1), int(m.group(2)), int(m.group(3)), int(m.group(4))
    key = 'MAPPING' if inv == 'I-1.11' else inv
    if key in never: e = 0
    elif key in only_new_root: e = root
    elif key == 'I-3.10': e = None
    elif key in ('I-8.6','I-8.9'): e = every - empty
    elif key == 'I-8.10': e = every - empty - 3
    elif key == 'I-8.11': e = root + 3
    elif key == 'I-8.12': e = every - 21
    elif key == 'I-4.1': e = 6
    elif key == 'I-7.16': e = every - 1
    else: e = every
    ok = (e is None or ev == e) and vi == 0
    print(('ok ' if ok else 'MISMATCH ') + f'{inv} evaluated={ev} expected={e} violated={vi} na={na}')
