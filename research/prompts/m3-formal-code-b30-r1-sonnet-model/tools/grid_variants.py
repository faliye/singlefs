#!/usr/bin/env python3
"""b30 正推腿 M1：在草稿副本里逐个施加变异行（data/mutants.tsv 的行序号，0 = 基线不施加），编、跑回退网格探针 b30_m1_rollback_grid，还原。
用法：REPO=… CARGO_TARGET_DIR=… grid_variants.py <tsv> <树根> <GRID> <序号|base>…
打 GRIDRESULT 行：序号、每格的结局（Completed 或 NewFinding 的位置）。"""
import os, re, subprocess, sys
sys.path.insert(0, os.path.dirname(__file__))
import mutants as m

def main():
    tsv, root, grid, *idxs = sys.argv[1:]
    repo = os.environ['REPO']
    rows = {r['i']: r for r in m.read(tsv)}
    for tag in idxs:
        r = None if tag == 'base' else rows[int(tag)]
        path = os.path.join(root, r['file']) if r else None
        src = open(path, encoding='utf-8').read() if r else None
        if r:
            assert src.count(r['old']) == 1
            open(path, 'w', encoding='utf-8').write(src.replace(r['old'], r['new']))
        try:
            env = dict(os.environ)
            env['GRID'] = grid
            env['RUN_WITH_MEMORY_CAP_WAIT_SECONDS'] = '150'
            env['RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON'] = 'b30 正推腿 M1 网格，上限取派发给的 8G'
            env['RUN_WITH_MEMORY_CAP_PARALLEL_REASON'] = '同会话别的子 agent 在跑，b30 云端正推腿'
            cmd = ['nice', '-n', '19', 'bash', f'{repo}/scripts/hosts/run-with-memory-cap.sh', '8G', 'bash', f'{repo}/scripts/hosts/capped.sh', '7',
                   'cargo', 'test', '--offline', '-p', 'singlefs-harness', '--test', 'b30_m1_rollback_grid', '--', '--nocapture']
            p = subprocess.run(cmd, cwd=root, env=env, capture_output=True, text=True, timeout=int(os.environ.get('GRID_TIMEOUT', '230')))
            out = p.stdout + p.stderr
            cells = re.findall(r'^CELL .*$', out, re.M)
            if not cells:
                print(f'GRIDRESULT\t{tag}\tNO_CELLS rc={p.returncode}\t' + out[-300:].replace('\n', ' '))
            for c in cells:
                c = re.sub(r'NewFinding \{ signature: ModelDisagreement \{ aspect: "([^"]*)".*position: (Operation\(\d+\)).*', r'NewFinding(\1 @\2)', c)
                print(f'GRIDRESULT\t{tag}\t{c[:200]}', flush=True)
        except subprocess.TimeoutExpired:
            print(f'GRIDRESULT\t{tag}\tTIMEOUT')
        finally:
            if r: open(path, 'w', encoding='utf-8').write(src)

main()
