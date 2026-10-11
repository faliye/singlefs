#!/usr/bin/env python3
"""对副本里的 walk.rs 逐条加变异、跑点名的 harness 测试、报哪些测试红；用法：mut.py <副本根> <目标目录> <变异名前缀>... ；
变异名前缀 own-never-open 是本腿自己的一条（把 LastPage 那一支的门改成 if false）。每条跑完把 walk.rs 还原。"""
import sys, subprocess, shutil, os
work, target = sys.argv[1], sys.argv[2]
orig_dir = work + '/orig'
repo = work + '/repo'
scripts = os.environ['SINGLEFS_REPO'] + '/scripts'
env = dict(os.environ, CARGO_TARGET_DIR=target)
rows = open(orig_dir + '/crates/mutations.tsv', encoding='utf-8').read().split('\n')
W = 'crates/singlefs-checker/src/walk.rs'
orig = open(orig_dir + '/' + W, encoding='utf-8').read()

def run(args, cap='4G'):
    cmd = ['nice', '-n', '19', 'bash', scripts + '/run-with-memory-cap.sh', cap, 'bash', scripts + '/capped.sh', '10', 'cargo', 'test', '--offline'] + args
    p = subprocess.run(cmd, cwd=repo, env=env, capture_output=True, text=True)
    out = p.stdout + p.stderr
    return p.returncode, [x for x in out.split('\n') if x.startswith('test ') and not x.startswith('test result') and ('FAILED' in x or ' ok' in x)]

for name in sys.argv[3:]:
    if name == 'own-never-open':
        a = "                ChainRecordView::LastPage => {\n                    if mount_root_instance_when_newest.is_some() {"
        b = "                ChainRecordView::LastPage => {\n                    if false {"
        args = ['-p', 'singlefs-harness', '--test', 'checker_known_bad_images', '--', 'the_newest_instance_table', 'an_abandoned_root_naming']
        label = name
    else:
        f = [l.split('\t') for l in rows if l.startswith(name)]
        f = [x for x in f if x[1].endswith('walk.rs') and 'singlefs-harness' in x[4]]
        assert len(f) == 1, (name, len(f))
        f = f[0]
        a, b = f[2].replace('\\n', '\n'), f[3].replace('\\n', '\n')
        args = f[4].split()
        label = f[0][:40] + '..' + f[0][-30:]
    assert orig.count(a) == 1
    open(repo + '/' + W, 'w', encoding='utf-8').write(orig.replace(a, b))
    rc, tests = run(args)
    shutil.copyfile(orig_dir + '/' + W, repo + '/' + W)
    reds = sorted(x for x in tests if 'FAILED' in x)
    print('MUTATION', label, '| rc', rc, '| 红的测试数', len(reds))
    for x in reds:
        print('RED', label, '::', x.replace(' ... FAILED', ''))
