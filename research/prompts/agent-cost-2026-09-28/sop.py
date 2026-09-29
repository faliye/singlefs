"""上游 SOP（.claude/singlefs-ai-sop/）的用量与摩擦统计（2026-09-28 那份分析用）。在仓根跑：python3 research/prompts/agent-cost-2026-09-28/sop.py
读会话记录里谁读了哪份规则、真正调用了哪道 lint / 门禁脚本并判红多少，以及副本的体量与发版节奏。"""
import json, glob, os, re, datetime, collections, subprocess
ROOT = os.environ.get('AGENT_COST_ROOT', os.path.expanduser('~/.claude/projects/-home-fy5090-code-singlefs'))
SINCE = datetime.datetime.fromisoformat(os.environ.get('AGENT_COST_SINCE', '2026-09-20')).replace(tzinfo=datetime.timezone.utc)
U = '.claude/singlefs-ai-sop'
def recent(p): return datetime.datetime.fromtimestamp(os.path.getmtime(p), tz=datetime.timezone.utc) >= SINCE
UP = sorted(os.path.basename(p)[:-3] for p in glob.glob(f'{U}/rules/*.md')); LO = sorted(os.path.basename(p)[:-3] for p in glob.glob('.claude/rules/*.md'))
TOOLS = ['doc-lint.sh', 'gate.sh', 'gate-staged.sh', 'check.sh', 'gate-lint.sh', 'shell-lint.sh', 'rules-lint.sh', 'stage-selftest.sh', 'naming-lint.sh', 'preflight-lint.py', 'gate-overlap.py', 'hooks-registered.sh', 'show-me-test.sh', 'version-discipline.sh']
INV = re.compile(r'(?:^|[;&|(]\s*|\bbash\s+|\bpython3\s+|\bnice(?: -n \d+)?\s+bash\s+|SINGLEFS_HEAVY_TESTS=\S+\s+bash\s+)(?:\S*/)?(' + '|'.join(re.escape(t) for t in TOOLS) + r')\b')
sub_reads = collections.Counter(); sub_agents = collections.defaultdict(set); main_reads = collections.Counter(); prompt_refs = collections.Counter()
runs = collections.Counter(); reds = collections.Counter(); doc_kind = collections.Counter(); pre78 = 0; force = 0
def scan(path, who):
    global pre78, force
    id2tool = {}
    for line in open(path, errors='replace'):
        try: o = json.loads(line)
        except Exception: continue
        m = o.get('message', {}); c = m.get('content')
        if o.get('type') == 'assistant' and isinstance(c, list):
            for b in c:
                if b.get('type') != 'tool_use': continue
                s = json.dumps(b.get('input', {}), ensure_ascii=False)
                if b['name'] in ('Agent', 'Task'):
                    for r in UP + LO:
                        if r + '.md' in b['input'].get('prompt', ''): prompt_refs[r] += 1
                    continue
                if b['name'] in ('Read', 'Bash'):
                    for r in UP + LO:
                        if 'rules/' + r + '.md' in s:
                            if who == 'main': main_reads[r] += 1
                            else: sub_reads[r] += 1; sub_agents[r].add(path)
                if b['name'] == 'Bash':
                    cmd = b['input'].get('command', '')
                    if '--force' in cmd and ('gate' in cmd or 'preflight' in cmd or '.sh' in cmd): force += 1
                    if re.match(r'\s*(sed|grep|cat|awk|head|tail|wc|less|rg)\b', cmd): continue
                    mm = INV.search(cmd)
                    if mm: id2tool[b['id']] = mm.group(1); runs[(who, mm.group(1))] += 1
        elif o.get('type') == 'user' and isinstance(c, list):
            for b in c:
                if b.get('type') != 'tool_result': continue
                s = b.get('content'); s = s if isinstance(s, str) else json.dumps(s, ensure_ascii=False)
                if '退出码 78' in s or 'exit 78' in s or '准入不满足' in s: pre78 += 1
                t = id2tool.get(b.get('tool_use_id'))
                if not t: continue
                if re.search(r'^\s*✗|\n\s*✗|门禁未通过|判红', s): reds[(who, t)] += 1
                if t == 'doc-lint.sh':
                    for r in re.findall(r'✗[^\n]{0,80}', s):
                        doc_kind['编号或简称' if re.search(r'简称|编号|登记', r) else '历史节' if '历史' in r else '指代或自称' if re.search(r'指代|自称|上下文', r) else '文风' if re.search(r'翻译|古风', r) else '其他'] += 1
for jl in glob.glob(f'{ROOT}/*.jsonl'):
    if recent(jl): scan(jl, 'main')
for jl in glob.glob(f'{ROOT}/*/subagents/agent-*.jsonl'):
    if recent(jl): scan(jl, 'sub')
print(f'# sop.py 口径：{SINCE.date()} 起的会话记录；副本 {U}，VERSION', open(f'{U}/VERSION').read().strip())
print('\n## 体量')
print('上游规则', len(UP), '篇', sum(os.path.getsize(f"{U}/rules/{r}.md") for r in UP), '字节；项目本地规则', len(LO), '篇', sum(os.path.getsize(f".claude/rules/{r}.md") for r in LO), '字节')
print('上游脚本', len(glob.glob(f'{U}/scripts/*.sh') + glob.glob(f'{U}/scripts/*.py')), '份；门禁格', sum(open(p, errors='replace').read().count('# gate-cell:') for p in glob.glob('.claude/gate.d/*.sh')), '个在', len(glob.glob('.claude/gate.d/*.sh')), '道里；样本文件', sum(len(f) for _, _, f in os.walk('.claude/gate.d/fixtures')), '份')
print('带准入声明的脚本', len([p for p in glob.glob('.claude/gate.d/*.sh') + glob.glob('.claude/hooks/*.sh') + glob.glob('.claude/scripts/*.sh') + glob.glob('research/scripts/*') if os.path.isfile(p) and 'admission:' in open(p, errors='replace').read()]), '；preflight-exclude 登记', sum(1 for l in open('.claude/preflight-exclude') if l.strip() and not l.startswith('#')) if os.path.exists('.claude/preflight-exclude') else 0)
log = subprocess.run(['git', '-C', '../singlefs-ai-sop-zh', 'log', f'--since={SINCE.date()}', '--oneline'], capture_output=True, text=True).stdout
print('上游仓', SINCE.date(), '起提交', len(log.splitlines()), '次；CHANGELOG 里提误报 / 误判 / 漏判的条目', sum(1 for l in open(f'{U}/CHANGELOG.md', errors='replace') if re.search(r'误报|误判|误红|漏判|漏报|假红', l)))
print('\n## 规则被谁读（U 上游、L 本地）')
print(f"{'rule':28}{'bytes':>7}{'sub reads':>10}{'sub agents':>11}{'main reads':>11}{'in prompts':>11}")
for r in UP: print(f"{'U ' + r:28}{os.path.getsize(f'{U}/rules/{r}.md'):>7}{sub_reads[r]:>10}{len(sub_agents[r]):>11}{main_reads[r]:>11}{prompt_refs[r]:>11}")
for r in LO: print(f"{'L ' + r:28}{os.path.getsize(f'.claude/rules/{r}.md'):>7}{sub_reads[r]:>10}{len(sub_agents[r]):>11}{main_reads[r]:>11}{prompt_refs[r]:>11}")
print('\n## lint 与门禁脚本的真正调用与判红（不算读源码）')
print(f"{'tool':22}{'main runs':>10}{'main ✗':>8}{'sub runs':>10}{'sub ✗':>7}")
for t in TOOLS: print(f"{t:22}{runs[('main', t)]:>10}{reds[('main', t)]:>8}{runs[('sub', t)]:>10}{reds[('sub', t)]:>7}")
print('doc-lint 的 ✗ 行按种类:', doc_kind.most_common())
print('准入不满足（退出码 78）出现', pre78, '次；带 --force 的调用', force, '次')
