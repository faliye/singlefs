#!/usr/bin/env python3
"""把 crates/mutations.tsv 里一条「E171R1 装置」变异施加在草稿副本的装置上、编 release 二进制、跑一遍装置的小规模模式（不是内联单测）、
与没变异的那份标准输出比。看的是：这条变异在装置自己的跑法里看不看得出（看不出的只靠 checker 档内联单测盯着，那是重型、这里不跑）。

用法：runtime_mutation.py <副本根> <变异表> <行名前缀> <基线标准输出文件> <装置参数…>
比较时去掉含 `ns=`、`nanoseconds`、`E7TIMING` 的行（计时）。输出一行 RESULT：exit 码与输出是否逐行相同。
每次起命令经 capped.sh 与 run-with-memory-cap.sh。变异在编完之后立即还原，二进制拷到草稿目录再跑。
"""
import os, shutil, subprocess, sys
root, table, prefix, baseline, *args = sys.argv[1:]
repo = os.environ.get('SINGLEFS_REPO', '/home/fy5090/code/singlefs')
scratch = os.path.dirname(os.path.abspath(root))
env = dict(os.environ)
env['RUN_WITH_MEMORY_CAP_PARALLEL_REASON'] = '派出去的正推腿与攻方腿各跑各的，同会话号'
env['RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON'] = '草稿副本里跑变异装置的小规模，没有峰值表记录'

def norm(text):
    return [l for l in text.splitlines() if 'ns=' not in l and 'nanoseconds' not in l and 'E7TIMING' not in l]

base = norm(open(baseline, encoding='utf-8').read())
for line in open(table, encoding='utf-8'):
    cols = line.rstrip('\n').split('\t')
    if len(cols) < 6 or not cols[0].startswith(prefix):
        continue
    name, rel, old, new = cols[0], cols[1], cols[2].replace('\\n', '\n'), cols[3].replace('\\n', '\n')
    path = os.path.join(root, rel)
    text = open(path, encoding='utf-8').read()
    if text.count(old) != 1:
        print(f'RESULT name={name[:40]} anchor_hits={text.count(old)} outcome=not_run')
        continue
    shutil.copy2(path, path + '.orig-for-mutation')
    try:
        with open(path, 'w', encoding='utf-8') as f:
            f.write(text.replace(old, new))
        b = subprocess.run(['nice', '-n', '19', 'bash', f'{repo}/research/scripts/capped.sh', '6', 'cargo', 'build', '--release', '--offline',
                            '-p', 'singlefs-checker-tier', '--bin', 'e171_three_oracle_collusion_after_the_oracle_fixes'],
                           cwd=root, env=env, capture_output=True, text=True, timeout=230)
    finally:
        shutil.move(path + '.orig-for-mutation', path)
    if b.returncode != 0:
        print(f'RESULT name={name[:40]} outcome=compile_error exit={b.returncode}')
        continue
    binary = os.path.join(scratch, 'mutated-bin-' + str(abs(hash(name)) % 100000))
    shutil.copy2(os.path.join(root, 'target/release/e171_three_oracle_collusion_after_the_oracle_fixes'), binary)
    outdir = os.path.join(scratch, 'out', 'mut-' + str(abs(hash(name)) % 100000))
    cmd = ['nice', '-n', '19', 'bash', f'{repo}/research/scripts/capped.sh', '6', 'bash', f'{repo}/research/scripts/run-with-memory-cap.sh', os.environ.get('MEMORY_CAP', '8G'), binary] + \
          [a.replace('@OUT@', outdir) for a in args]
    r = subprocess.run(cmd, cwd=scratch, env=env, capture_output=True, text=True, timeout=230)
    got = norm(r.stdout)
    same = got == base
    diff = [l for l in got if l not in base][:3]
    print(f'RESULT name={name[:60]} exit={r.returncode} stdout_same_as_unmutated={same} differing_lines={len(set(got) ^ set(base))}')
    for l in diff:
        print('  NEW_LINE', l[:400])
    os.remove(binary)
