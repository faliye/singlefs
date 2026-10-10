#!/usr/bin/env python3
"""对一份树副本（就地改、跑完还原）里的变异表行逐条施加、跑 harness 档测试、还原；一次只起一条 cargo。结果一行一个文件，断了续跑。

  mutrun.py --tree <树根> --target <编译目录> --tools <放 research/scripts 的仓根> --table <tsv> --rows 2883,2930,...
            --outdir <结果目录> [--deadline <epoch 秒，0 = 不限>] [--threads 10]
每行一个结果文件 <outdir>/<表文件名>-<行号>.txt，内容一行：
  ROW<TAB>表文件名<TAB>行号<TAB>名字前 40 字<TAB>结局<TAB>点名的测试红没红<TAB>红了几个
结局：RED（至少一个测试红）、GREEN（全绿＝没抓到）、INVALID（编不过）、WRAPPER（包装的结局 250–254）、ANCHOR<n>（锚点命中 n 次，没施加）、SKIP（要 GPU 的特性，不跑）
基线（还原后的原树上同一条命令全绿）每条不同的命令记一个 <outdir>/BASE-<序号>.txt。过了 --deadline 在开下一行之前退出 75（再跑同一条命令续）。
"""
import argparse, hashlib, os, re, subprocess, sys, time

ap = argparse.ArgumentParser()
ap.add_argument('--tree', required=True)
ap.add_argument('--target', required=True)
ap.add_argument('--tools', required=True)
ap.add_argument('--table', required=True)
ap.add_argument('--rows', required=True)
ap.add_argument('--outdir', required=True)
ap.add_argument('--deadline', type=float, default=0)
ap.add_argument('--threads', default='10')
args = ap.parse_args()
os.makedirs(args.outdir, exist_ok=True)

env = dict(os.environ)
env['CARGO_TARGET_DIR'] = args.target
env['RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON'] = '上限按派发提示给的编译 6G、跑 4G，高于峰值表里同键实测的 2 倍也照给'
env.setdefault('RUN_WITH_MEMORY_CAP_WAIT_SECONDS', '1500')


def cargo(cap, cargo_args):
    env['RUN_WITH_MEMORY_CAP_KEY'] = 'm3b28-sonnet-' + ('compile' if '--no-run' in cargo_args else 'run')
    cmd = ['nice', '-n', '19', 'bash', os.path.join(args.tools, 'research/scripts/capped.sh'), args.threads,
           'bash', os.path.join(args.tools, 'research/scripts/run-with-memory-cap.sh'), cap,
           'cargo', 'test', '--offline', '--manifest-path', os.path.join(args.tree, 'Cargo.toml')] + cargo_args
    p = subprocess.run(cmd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, cwd=args.tools)
    if 250 <= p.returncode <= 254:
        print('包装退 %d：%s' % (p.returncode, p.stdout[-600:]), file=sys.stderr, flush=True)
    return p.returncode, p.stdout


def compile_only(cargs):
    return cargo('6G', (cargs[:cargs.index('--')] if '--' in cargs else cargs) + ['--no-run'])


def over():
    return args.deadline and time.time() > args.deadline


def emit(name, line):
    with open(os.path.join(args.outdir, name), 'x' if not os.path.exists(os.path.join(args.outdir, name)) else 'w', encoding='utf-8') as f:
        f.write(line + '\n')
    print(line, flush=True)


lines = open(args.table, encoding='utf-8').read().split('\n')
tablename = os.path.basename(args.table)
for row in [int(x) for x in args.rows.split(',') if x]:
    resultfile = '%s-%d.txt' % (tablename, row)
    if os.path.exists(os.path.join(args.outdir, resultfile)):
        continue
    if over():
        sys.exit(75)
    c = lines[row - 1].split('\t')
    name, f, anchor, repl, cmd, expect = c[0], c[1], c[2].replace('\\n', '\n'), c[3].replace('\\n', '\n'), c[4], c[5]
    short = name[:40]
    if '--features' in cmd:
        emit(resultfile, 'ROW\t%s\t%d\t%s\tSKIP\t-\t-' % (tablename, row, short))
        continue
    path = os.path.join(args.tree, f)
    pristine = open(path, encoding='utf-8').read()
    hits = pristine.count(anchor)
    if hits != 1:
        emit(resultfile, 'ROW\t%s\t%d\t%s\tANCHOR%d\t-\t-' % (tablename, row, short, hits))
        continue
    cargs = cmd.split()
    basefile = 'BASE-%s.txt' % hashlib.sha1(cmd.encode()).hexdigest()[:8]
    if not os.path.exists(os.path.join(args.outdir, basefile)):
        compile_only(cargs)
        rc, out = cargo('4G', cargs)
        emit(basefile, 'BASE\t%s\t%s\t%s' % (cmd[-70:], 'ok' if rc == 0 else 'RED rc=%d' % rc, ' '.join(re.findall(r'test result: \w+\. \d+ passed; \d+ failed', out)[:1])))
    try:
        open(path, 'w', encoding='utf-8').write(pristine.replace(anchor, repl, 1))
        rc, out = compile_only(cargs)
        if rc != 0:
            res = 'WRAPPER' if 250 <= rc <= 254 else 'INVALID'
            emit(resultfile, 'ROW\t%s\t%d\t%s\t%s\t-\t-' % (tablename, row, short, res))
            continue
        rc, out = cargo('4G', cargs)
        failed = re.findall(r'^test (\S+) \.\.\. FAILED$', out, re.M)
        if 250 <= rc <= 254:
            res = 'WRAPPER'
        else:
            res = 'RED' if failed else ('GREEN' if rc == 0 else 'RC%d' % rc)
        named = 'named_red' if expect in failed else 'named_not_red'
        emit(resultfile, 'ROW\t%s\t%d\t%s\t%s\t%s\t%d' % (tablename, row, short, res, named, len(failed)))
    finally:
        open(path, 'w', encoding='utf-8').write(pristine)
