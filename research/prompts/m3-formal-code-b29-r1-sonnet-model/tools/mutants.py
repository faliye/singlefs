#!/usr/bin/env python3
"""b29 正推腿：从改动全文里抽变异行，在草稿副本里逐条施加、编、跑、还原（只跑 harness 档的行）。

用法：
  mutants.py extract <diff.md> <out.tsv>        抽出 mutations-append / replacements 各块里的行（块名、序号、六段）
  mutants.py anchors <tsv> <树根>               逐行数原文在文件里命中几次（恰 1 才算锚点好）
  mutants.py list <tsv> [--harness]
  mutants.py run <tsv> <树根> <序号> [<序号>…]  施加→cargo test→还原；打 RESULT 行
环境：CARGO_TARGET_DIR、REPO（主仓根，找 research/scripts）必须设。
"""
import os, re, subprocess, sys, time

def unesc(s):
    return s.replace('\\n', '\n')

def extract(diff, out):
    lines = open(diff, encoding='utf-8').read().split('\n')
    block = None; section = None; rows = []
    for ln in lines:
        m = re.match(r'^## (.*)$', ln)
        if m: section = m.group(1)
        m = re.match(r'^### (mutations-append\.tsv|mutations-replacements\.tsv|mutations-delete\.txt)', ln)
        if m: block = m.group(1); continue
        if ln.startswith('````text'): inside = True; continue
        if ln.startswith('````') and block is not None: block = None if ln == '````' else block; continue
        if block in ('mutations-append.tsv', 'mutations-replacements.tsv') and ln.count('\t') >= 5 and not ln.startswith('#'):
            rows.append((section, block, ln))
    with open(out, 'w', encoding='utf-8') as f:
        for i, (sec, blk, ln) in enumerate(rows):
            f.write(f'{i}\t{sec}\t{blk}\t{ln}\n')
    print('rows', len(rows))

def read(tsv):
    rows = []
    for ln in open(tsv, encoding='utf-8').read().split('\n'):
        if not ln: continue
        p = ln.split('\t')
        rows.append(dict(i=int(p[0]), sec=p[1], blk=p[2], name=p[3], file=p[4], old=unesc(p[5]), new=unesc(p[6]), cmd=p[7], test=p[8] if len(p) > 8 else ''))
    return rows

def anchors(tsv, root):
    bad = 0
    for r in read(tsv):
        path = os.path.join(root, r['file'])
        n = open(path, encoding='utf-8').read().count(r['old']) if os.path.exists(path) else -1
        kind = 'replacement' if r['blk'].endswith('replacements.tsv') else 'append'
        # 替换行的原文是「改坏之前的源码」不是；替换行第三段是原文行名……（本表六段都是 名/文件/原文/替换文/cmd/test）
        print(f"ANCHOR\t{r['i']}\t{r['sec']}\t{kind}\t{n}\t{r['name'][:70]}")
        if n != 1: bad += 1
    print('NOT_ONE', bad)

def run(tsv, root, idxs):
    repo = os.environ['REPO']
    rows = {r['i']: r for r in read(tsv)}
    for i in idxs:
        r = rows[i]
        path = os.path.join(root, r['file'])
        src = open(path, encoding='utf-8').read()
        if src.count(r['old']) != 1:
            print(f"RESULT\t{i}\tANCHOR_{src.count(r['old'])}\t{r['name']}"); continue
        open(path, 'w', encoding='utf-8').write(src.replace(r['old'], r['new']))
        t0 = time.time()
        try:
            args = r['cmd'].split(' ')
            # cmd 形如 "-p pkg --test x -- filter"；用例名过滤放在 -- 之后
            base = ['nice', '-n', '19', 'bash', f'{repo}/research/scripts/run-with-memory-cap.sh', '8G', 'bash', f'{repo}/research/scripts/capped.sh', '10', 'cargo', 'test', '--offline'] + args
            env = dict(os.environ)
            env['RUN_WITH_MEMORY_CAP_WAIT_SECONDS'] = '150'
            env['RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON'] = '变异跑道，上限取派发给的 8G'
            env['RUN_WITH_MEMORY_CAP_PARALLEL_REASON'] = '派出去的几个 agent 各跑各的，这条是正推腿'
            p = subprocess.run(base, cwd=root, env=env, capture_output=True, text=True, timeout=int(os.environ.get('MUT_TIMEOUT', '215')))
            out = p.stdout + p.stderr
            failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
            passed = re.findall(r'^test (\S+) \.\.\. ok', out, re.M)
            if re.search(r'^error(\[E\d+\])?:', out, re.M) and not failed and not passed:
                res = 'INVALID_COMPILE'
            elif failed:
                res = 'CAUGHT failed=' + ','.join(f.split('::')[-1][:60] for f in failed)
            elif passed:
                res = f'SURVIVED passed={len(passed)}'
            else:
                res = f'NO_TEST_OUTPUT rc={p.returncode}'
            open(f"{os.environ.get('MUTLOG','/tmp')}/mut-{i}.log", 'w').write(out)
        except subprocess.TimeoutExpired:
            res = 'TIMEOUT'
        finally:
            open(path, 'w', encoding='utf-8').write(src)
        line = f"RESULT\t{i}\t{res}\t{int(time.time()-t0)}s\t{r['sec']}\t{r['name'][:80]}"
        print(line, flush=True)
        open(f"{os.environ.get('MUTLOG','/tmp')}/results.tsv", 'a').write(line + '\n')

if __name__ == '__main__':
    c = sys.argv[1]
    if c == 'extract': extract(sys.argv[2], sys.argv[3])
    elif c == 'anchors': anchors(sys.argv[2], sys.argv[3])
    elif c == 'list':
        for r in read(sys.argv[2]):
            if '--harness' in sys.argv and 'singlefs-checker-tier' in r['cmd']: continue
            print(r['i'], r['sec'], r['blk'][:12], r['name'][:60], '|', r['cmd'][:90])
    elif c == 'run': run(sys.argv[2], sys.argv[3], [int(x) for x in sys.argv[4:]])
