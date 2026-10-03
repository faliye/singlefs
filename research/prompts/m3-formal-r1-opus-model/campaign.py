#!/usr/bin/env python3
"""m3-formal-r1 攻方腿：有界穷举原型的变异战役（副本上跑，不碰仓）。

用法（整条经内存包装与线程上限跑，见 README.md）：
    python3 campaign.py <TREE> <TARGET> <OUT> <THREADS>
TREE   = 冻结副本的拷贝（rsync 出来的，里面 crates/singlefs-core/examples/ 已放好 formal_probe_replay_bounded.rs）
TARGET = 编译目录（放草稿目录，不进仓）
OUT    = 输出目录：campaign.log（原样输出）、results.tsv（每个变体 × 模式 × 判据的最小杀伤界）

变体 = 原实现 + crates/mutations.tsv 里落在 recovery.rs 第 2930–3230 行（replay_journal 与它的几个辅助函数）的入库变异 + 4 条自造变异。
每个变体：改 TREE 里的 recovery.rs、编原型、各模式从 n=1 扫到上沿；跑完还原并核 sha256。
"""
import hashlib, os, re, subprocess, sys, time

TREE, TARGET, OUT, THREADS = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
REC = TREE + '/crates/singlefs-core/src/recovery.rs'
BIN = TARGET + '/release/examples/formal_probe_replay_bounded'
# (模式, 原实现扫到的 n, 变体扫到的 n, 额外环境变量)
MODES = [
    ('base', 7, 6, {}),
    ('units', 5, 4, {}),
    ('units2', 4, 3, {}),
    ('stray', 6, 5, {}),
    ('wrap', 7, 6, {}),
    ('edge', 6, 5, {}),
    ('adv', 3, 2, {'PROBE_ADV_FOLLOW': 'monotone'}),
]
ORIG_ONLY = [
    ('residue', 4, {}),
    ('adv', 3, {}),  # 读法甲（D23 已定项 4 字面）
    ('mono', 5, {}),
    ('mono', 5, {'PROBE_MONO_RING_SLOTS': '6'}),
]
CRITERIA = 'RPMXF'
os.makedirs(OUT, exist_ok=True)
original = open(REC).read()
orig_sha = hashlib.sha256(original.encode()).hexdigest()
muts = []
for ln in open(TREE + '/crates/mutations.tsv'):
    if ln.startswith('#') or not ln.strip():
        continue
    f = ln.rstrip('\n').split('\t')
    if len(f) < 6 or not f[1].endswith('singlefs-core/src/recovery.rs'):
        continue
    o, r = f[2].replace('\\n', '\n'), f[3].replace('\\n', '\n')
    i = original.find(o)
    if i < 0 or original.count(o) != 1:
        continue
    line = original[:i].count('\n') + 1
    if 2930 <= line <= 3230:
        muts.append(('m%02d' % (len(muts) + 1), 'L%d %s' % (line, f[0]), o, r))
TAKE = 'for record in above.into_iter().take(in_flight_limit) {'
muts += [
    ('v1', '自造：一条都不施加（前缀取 0 条）', TAKE, 'for record in above.into_iter().take(0) {'),
    ('v2', '自造：不看在飞上限', TAKE, 'for record in above.into_iter().take(usize::MAX) {'),
    ('v3', '自造：少施加（前缀最多 2 条）', TAKE, 'for record in above.into_iter().take(in_flight_limit.min(2)) {'),
    ('v4', '自造：不验点名单元', '        let all_verified = !verify_named_units\n', '        let all_verified = true || !verify_named_units\n'),
]
env = dict(os.environ, CARGO_TARGET_DIR=TARGET)
log = open(OUT + '/campaign.log', 'w')
out = open(OUT + '/results.tsv', 'w')
out.write('id\tmode\tcriterion\tmin_n\tcount_at_min_n\tscanned_to_n\tname\n')
LINE = re.compile(r'mode=(\S+) n=(\d+) .*?states=(\d+) viol_R=(\d+) viol_P=(\d+)(?: viol_M=(\d+) viol_X=(\d+) viol_F=(\d+))?')
total_states = 0


def emit(text):
    log.write(text + '\n')
    log.flush()


def run(mode, nmin, nmax, extra):
    global total_states
    p = subprocess.run([BIN, mode, str(nmin), str(nmax), THREADS], capture_output=True, text=True, env=dict(os.environ, **extra))
    tag = mode + ''.join('+%s=%s' % kv for kv in sorted(extra.items()))
    first = {}
    for line in p.stdout.splitlines():
        emit(tag + '\t' + line)
        m = LINE.match(line)
        if m:
            total_states += int(m.group(3))
            counts = [g for g in m.groups()[3:] if g is not None]
            for k, v in zip(CRITERIA, counts):
                if int(v) > 0 and k not in first:
                    first[k] = (int(m.group(2)), int(v))
    if p.returncode != 0:
        emit('RUN_RC %d %s' % (p.returncode, p.stderr[-500:]))
    return tag, first


def build():
    return subprocess.run(['cargo', 'build', '--release', '-q', '-p', 'singlefs-core', '--example', 'formal_probe_replay_bounded'],
                          cwd=TREE, env=env, capture_output=True, text=True)


try:
    for mid, name, o, r in [('orig', '原实现', '', '')] + muts:
        started = time.time()
        src = original if mid == 'orig' else original.replace(o, r, 1)
        assert mid == 'orig' or src != original, mid
        open(REC, 'w').write(src)
        b = build()
        emit('=== %s %s build_rc=%d' % (mid, name, b.returncode))
        if b.returncode != 0:
            emit(b.stderr[-2000:])
            out.write('%s\t-\t-\tBUILD_FAIL\t-\t-\t%s\n' % (mid, name))
            continue
        killed = False
        plan = [(mode, (n_orig if mid == 'orig' else n_mut), extra) for mode, n_orig, n_mut, extra in MODES]
        if mid == 'orig':
            plan += ORIG_ONLY
        for mode, nmax, extra in plan:
            tag, first = run(mode, 1, nmax, extra)
            if mode == 'mono':
                continue
            for k in (CRITERIA if mode != 'adv' else 'RP'):
                n, count = first.get(k, ('-', '-'))
                out.write('%s\t%s\t%s\t%s\t%s\t%d\t%s\n' % (mid, tag, k, n, count, nmax, name))
            killed = killed or bool(first)
        if mid != 'orig' and not killed:
            # 到上沿一格都没红的变体：对抗域再扫 n=3
            tag, first = run('adv', 3, 3, {'PROBE_ADV_FOLLOW': 'monotone'})
            for k in 'RP':
                n, count = first.get(k, ('-', '-'))
                out.write('%s\t%s\t%s\t%s\t%s\t%d\t%s\n' % (mid, tag + '+n3', k, n, count, 3, name))
        out.flush()
        emit('--- %s secs=%.1f states_so_far=%d' % (mid, time.time() - started, total_states))
finally:
    open(REC, 'w').write(original)
    assert hashlib.sha256(open(REC).read().encode()).hexdigest() == orig_sha
    b = build()
    emit('restored %s rebuild_rc %d total_states %d variants %d' % (orig_sha, b.returncode, total_states, len(muts) + 1))
