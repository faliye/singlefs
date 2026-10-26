#!/usr/bin/env python3
"""run.py <案例模块名>... : 每个案例把命令喂两份钩子，结果写进 results/<模块>.jsonl，完整标准错误写进 logs/。
钩子是草稿目录里 repo/.claude/hooks/ 下的拷贝（与冻结副本逐字节相同）；环境里去掉 SINGLEFS_HEAVY_TESTS 与 CLAUDE_PROJECT_DIR；检出记录指向草稿目录。"""
import sys; sys.dont_write_bytecode = True
import json, os, subprocess, sys, importlib, concurrent.futures as cf
HERE = os.path.dirname(os.path.abspath(__file__))
D = os.environ['S2_WORK']   # 草稿目录：里面要有 build-skeleton.sh 搭好的 repo/ copy/ stagedrepo/ emptyrepo/ wr/
sys.path.insert(0, HERE)
CWD = {'draft': D, 'repo': D+'/repo', 'copy': D+'/copy', 'staged': D+'/stagedrepo', 'empty': D+'/emptyrepo'}
HOOKS = {'H': 'heavy-test-guard.sh', 'D': 'bash-command-detector.sh'}
def feed(case, which):
    ident = case['ident']; opt = case['opt']
    cwd = CWD.get(opt.get('cwd') or ('repo' if ident == 'main' else 'copy'), opt.get('cwd'))
    scratch = D + '/wr/scratch' if opt.get('wr') else D
    cmd = case['cmd'].replace('{D}', scratch)
    inp = {'tool_name': 'Bash', 'cwd': cwd, 'session_id': 's2-r2-test', 'tool_input': {'command': cmd}}
    if opt.get('bg'): inp['tool_input']['run_in_background'] = True
    if opt.get('timeout'): inp['tool_input']['timeout'] = opt['timeout']
    if ident != 'main':
        inp['agent_type'] = ident; inp['agent_id'] = 'agent-s2-r2-test'
    env = {k: v for k, v in os.environ.items() if k not in ('SINGLEFS_HEAVY_TESTS', 'CLAUDE_PROJECT_DIR')}
    env['AGENT_HOOK_DETECTIONS'] = D + '/detections.jsonl'
    if opt.get('wr'): env['SINGLEFS_WORK_ROOT'] = D + '/wr'
    r = subprocess.run(['bash', D+'/repo/.claude/hooks/'+HOOKS[which]], input=json.dumps(inp), text=True, capture_output=True, env=env, timeout=600)
    lines = [l for l in r.stderr.splitlines() if l.strip()]
    first = next((l for l in lines if l.lstrip().startswith('✗')), lines[0] if lines else '')
    os.makedirs(D+'/logs', exist_ok=True)
    open(f"{D}/logs/{case['id']}-{which}.txt", 'w').write(r.stderr)
    return r.returncode, first[:400]
def one(case):
    h = feed(case, 'H'); d = feed(case, 'D')
    return dict(id=case['id'], ident=case['ident'], cmd=case['cmd'], cwd=case['opt'].get('cwd'), bg=bool(case['opt'].get('bg')), exp_h=case['h'], exp_d=case['d'], rc_h=h[0], rc_d=d[0], err_h=h[1], err_d=d[1], cites=case['cites'], note=case['opt'].get('note',''), unspecified=bool(case['opt'].get('unspecified')))
if __name__ == '__main__':
    os.makedirs(D+'/results', exist_ok=True)
    mods = sorted(f[:-3] for f in os.listdir(HERE) if f.startswith('cases_') and f.endswith('.py')) if sys.argv[1:] == ['--all'] else sys.argv[1:]
    for m in mods:
        mod = importlib.import_module(m)
        for c in mod.CASES: c['id'] = m[len('cases_'):] + '.' + c['id']   # 编号加模块字母，免得不同模块的编号撞
        with cf.ThreadPoolExecutor(8) as ex:
            res = list(ex.map(one, mod.CASES))
        with open(f"{D}/results/{m}.jsonl.new", 'w') as f:
            for r in res: f.write(json.dumps(r, ensure_ascii=False)+'\n')
        os.replace(f"{D}/results/{m}.jsonl.new", f"{D}/results/{m}.jsonl")
        bad = [r for r in res if (r['rc_h'] != r['exp_h'] or r['rc_d'] != r['exp_d']) or r['unspecified']]
        print(m, 'cases', len(res), 'mismatch', len(bad))
        for r in bad:
            print(' ', r['id'], r['ident'], repr(r['cmd'][:90]), 'H', r['rc_h'], 'exp', r['exp_h'], '| D', r['rc_d'], 'exp', r['exp_d'], '|', (r['err_h'] if r['rc_h']!=r['exp_h'] else r['err_d'])[:160])
