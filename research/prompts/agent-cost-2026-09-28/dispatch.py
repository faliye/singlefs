"""派发、闸、看门狗与工具用法的统计（2026-09-28 那份分析用）。只读 ~/.claude/projects/<项目>/ 下的会话记录。
用法：python3 dispatch.py            口径：SINCE 起改动过的主会话记录与它们 subagents/ 下的子 agent 记录。"""
import json, glob, os, re, sys, datetime, collections, bisect
ROOT = os.environ.get('AGENT_COST_ROOT', os.path.expanduser('~/.claude/projects/-home-fy5090-code-singlefs'))
SINCE = datetime.datetime.fromisoformat(os.environ.get('AGENT_COST_SINCE', '2026-09-20')).replace(tzinfo=datetime.timezone.utc)
def ts(o):
    try: return datetime.datetime.fromisoformat(o.get('timestamp', '').replace('Z', '+00:00'))
    except Exception: return None
def recent(path): return datetime.datetime.fromtimestamp(os.path.getmtime(path), tz=datetime.timezone.utc) >= SINCE
def records(path):
    for line in open(path, errors='replace'):
        try: yield json.loads(line), line
        except Exception: continue
def text_of(content): return content if isinstance(content, str) else (json.dumps(content, ensure_ascii=False) if content else '')
def folded(t): return t['input'] + t['cache_read'] * 0.1 + t['w5m'] * 1.25 + t['output'] * 5
def hook_name(s):
    m = re.search(r'hook error: \[bash [^\]]*?/([a-z\-]+\.sh)\]', s); return m.group(1) if m else '?'
PRIME_MARKS = re.compile("[\u2032\u2033\u2034\u02b9\u02ba]")   # 门禁 doc-text 的 prime-marks 格扫全仓，输出里不留角标字符
def norm(s): return PRIME_MARKS.sub("[角标]", re.sub(r"\d+", "N", re.sub(r"a[0-9a-f]{16}", "<id>", s)))
def sect(title): print(f'\n## {title}')

def main_sessions():
    disp = collections.Counter(); prompt_chars = 0; nprompt = 0
    rej = collections.Counter(); rej_reason = collections.defaultdict(collections.Counter)
    desc_repeat = collections.Counter(); sendmsg = collections.Counter(); taskstop = collections.Counter()
    notif = collections.Counter(); tooluse = collections.Counter(); heads = collections.Counter()
    post_alarm = collections.Counter(); alarms = collections.Counter(); detections = collections.Counter()
    launched = 0; batch_sizes = []; serial_after_hb = 0
    for jl in sorted(glob.glob(f'{ROOT}/*.jsonl')):
        if not recent(jl): continue
        sid = os.path.basename(jl)[:8]; id2cmd = {}; pending_alarm = False; disp_times = []; hb_times = []; pending_disp = {}
        for o, line in records(jl):
            typ = o.get('type'); m = o.get('message', {}); c = m.get('content'); t = ts(o)
            if typ == 'assistant' and isinstance(c, list):
                tools = [b for b in c if b.get('type') == 'tool_use']
                if pending_alarm and tools:
                    kinds = set()
                    for b in tools:
                        if b['name'] == 'Bash':
                            cmd = b['input'].get('command', '')
                            kinds.add('watch' if ('watch.sh' in cmd or 'agent-watch' in cmd) else ('read-output' if ('.output' in cmd or 'tail' in cmd) else 'bash-other'))
                        else: kinds.add(b['name'])
                    post_alarm['+'.join(sorted(kinds))] += 1; pending_alarm = False
                for b in tools:
                    tooluse[b['name']] += 1
                    if b['name'] == 'Bash':
                        cmd = b['input'].get('command', ''); id2cmd[b['id']] = cmd
                        if 'watch.sh' in cmd or 'agent-watch.py' in cmd: heads['watch.sh/agent-watch.py'] += 1; continue
                        toks = cmd.strip().split('\n')[0].split(); h = toks[0] if toks else ''
                        if h in ('cd', 'TZ=Asia/Tokyo', 'nice', 'bash', 'python3', 'timeout') and len(toks) > 1: h = h + ' ' + toks[1].split('/')[-1][:25]
                        heads[h[:34]] += 1
                    elif b['name'] in ('Agent', 'Task'):
                        disp[b['input'].get('subagent_type', '?')] += 1; prompt_chars += len(b['input'].get('prompt', '')); nprompt += 1
                        desc_repeat[(sid, b['input'].get('description', '')[:40])] += 1; pending_disp[b['id']] = t
                    elif b['name'] == 'SendMessage':
                        to = str(b['input'].get('to', '')); msg = b['input'].get('message', '')
                        if re.fullmatch(r'a[0-9a-f]{16}', to): sendmsg['例行询问' if ('在做什么' in msg or '还差几步' in msg or 'progress.md' in msg) else '纠正/追加'] += 1
                        else: sendmsg['给别的会话'] += 1
                    elif b['name'] == 'TaskStop':
                        tid = str(b['input'].get('task_id') or b['input'].get('taskId') or ''); taskstop['agent' if re.fullmatch(r'a[0-9a-f]{16}', tid) else 'bash/other'] += 1
            elif typ == 'user':
                txt = text_of(c)
                if isinstance(c, list):
                    for b in c:
                        if b.get('type') != 'tool_result': continue
                        s = text_of(b.get('content'))
                        if b.get('tool_use_id') in pending_disp:
                            t0 = pending_disp.pop(b['tool_use_id']); r = o.get('toolUseResult')
                            if isinstance(r, dict) and r.get('agentId'): launched += 1; disp_times.append(t0)
                        if 'hook error' in s and ('✗' in s):
                            h = hook_name(s); rej[h] += 1
                            r = re.search(r'✗ ?(.{0,46})', s)
                            if r: rej_reason[h][norm(r.group(1))[:44]] += 1
                        if 'agent-watch' in s and '⚠️' in s:
                            for a in re.findall(r'⚠️ ?([^：:（(\\"]{2,24})', s): alarms[norm(a).strip()[:22]] += 1
                        for d in re.finditer(r'hook 检出（[^）]*）：([^：]{2,28})：([^\\|]{3,40})', s): detections[d.group(2)[:30]] += 1
                if '<task-notification>' in txt:
                    st = re.search(r'<status>(\w+)</status>', txt); st = st.group(1) if st else '?'
                    ec = re.search(r'exit code (\d+)', txt); ec = ec.group(1) if ec else '-'
                    tu = re.search(r'<tool-use-id>([^<]+)</tool-use-id>', txt); cmd = id2cmd.get(tu.group(1), '') if tu else ''
                    tid = re.search(r'<task-id>([^<]+)</task-id>', txt); tid = tid.group(1) if tid else ''
                    if re.fullmatch(r'a[0-9a-f]{16}', tid): k = 'agent'; hb_times.append(t)
                    elif 'watch.sh' in cmd or 'agent-watch.py' in cmd: k = 'watchdog'; pending_alarm = (ec == '3')
                    elif re.search(r'cargo|prove-red|mutate\.sh|replay\.sh|run-with-memory-cap|capped\.sh|gate\.sh|gate-staged|layer0', cmd): k = 'heavy/cargo'
                    else: k = 'bash-other'
                    notif[(k, st, ec if k == 'watchdog' else '')] += 1
        disp_times.sort(); cur = 1
        for a, b in zip(disp_times, disp_times[1:]):
            if (b - a).total_seconds() <= 90: cur += 1
            else: batch_sizes.append(cur); cur = 1
        if disp_times: batch_sizes.append(cur)
        hb_times = sorted(x for x in hb_times if x)
        for t0 in disp_times:
            j = bisect.bisect_right(hb_times, t0)
            if j > 0 and (t0 - hb_times[j - 1]).total_seconds() <= 600: serial_after_hb += 1
    sect('派发：按类型（Agent 工具调用次数，含被闸拒的）')
    print('合计', sum(disp.values()), '；真正起来的', launched, '；平均提示长度（字符）', prompt_chars // max(1, nprompt))
    for k, v in disp.most_common(): print(f'  {k:28}{v}')
    sect('主 agent 侧的钩子拒绝（按钩子、按原因）')
    for h, v in rej.most_common():
        print(f'  {h}: {v}')
        for k2, v2 in rej_reason[h].most_common(6): print(f'      {v2:4d} {k2}')
    rep = [(k, v) for k, v in desc_repeat.items() if v >= 3]
    sect('同一描述在同一会话里派第三次以上')
    print('描述数', len(rep), '；覆盖派发', sum(v for _, v in rep), '；占派发', f'{sum(v for _, v in rep) / max(1, sum(disp.values())):.0%}')
    for k, v in sorted(rep, key=lambda kv: -kv[1])[:8]: print(f'  {v:3d} {k}')
    sect('派发批与串行')
    print('批（相邻派发间隔 90 秒内）中位', sorted(batch_sizes)[len(batch_sizes) // 2] if batch_sizes else 0, '；只派 1 个的批占', f'{sum(1 for b in batch_sizes if b == 1) / max(1, len(batch_sizes)):.0%}', '；最大一批', max(batch_sizes or [0]))
    print('交回通知之后 10 分钟内的派发', serial_after_hb, f'（{serial_after_hb / max(1, launched):.0%}）')
    sect('主 agent 发的消息与 TaskStop'); print(dict(sendmsg)); print('TaskStop', dict(taskstop))
    sect('后台任务通知（类别、状态、看门狗退出码）')
    for k, v in sorted(notif.items(), key=lambda kv: -kv[1])[:16]: print(f'  {v:5d} {k}')
    sect('看门狗告警之后主 agent 的第一批工具调用'); print(sum(post_alarm.values()), dict(post_alarm.most_common(6)))
    sect('主 agent 读到的看门狗告警名（含重复读）')
    for k, v in alarms.most_common(14): print(f'  {v:5d} {k}')
    sect('看门狗转报的 hook 检出，按内容')
    for k, v in detections.most_common(10): print(f'  {v:5d} {k}')
    sect('主 agent 的工具调用按名字'); 
    for k, v in tooluse.most_common(12): print(f'  {k:20}{v}')
    sect('主 agent 的 Bash 命令开头'); 
    for k, v in heads.most_common(24): print(f'  {v:6d} {k}')

def subagents():
    by_tool = collections.defaultdict(collections.Counter); res_chars = collections.defaultdict(collections.Counter)
    rewrites = collections.Counter(); rewrite_tokens = collections.Counter(); hook_err = collections.Counter(); hook_reason = collections.defaultdict(collections.Counter)
    kinds = collections.defaultdict(collections.Counter); cls = collections.defaultdict(collections.Counter)
    med_stats = collections.defaultdict(list)
    gov_kw = re.compile(r'\.claude/agents|\.claude/hooks|agent-common|main-agent|门禁|gate\.d|钩子|hook|定义|规则|rules/|watch|看门狗|research/scripts|治理|严查|governance|同步记录|knowledge-sync|回扫')
    fs_kw = re.compile(r'crates/|不变量|invariants|checker|崩溃|层 0|layer0|实验|E1\d\d|决策|D\d\d?（|kb/decisions|里程碑')
    for meta in glob.glob(f'{ROOT}/*/subagents/agent-*.meta.json'):
        jl = meta.replace('.meta.json', '.jsonl')
        if not os.path.exists(jl) or not recent(jl): continue
        atype = json.load(open(meta)).get('agentType', '?'); id2name = {}; prompt = None; seen = {}; order = []; c = collections.Counter()
        for o, line in records(jl):
            typ = o.get('type'); m = o.get('message', {}); content = m.get('content')
            if prompt is None and typ == 'user': prompt = text_of(content)
            if typ == 'assistant':
                u = m.get('usage'); mid = m.get('id') or o.get('uuid')
                if u and mid not in seen: order.append(mid)
                if u: seen[mid] = u
                for b in content if isinstance(content, list) else []:
                    if isinstance(b, dict) and b.get('type') == 'tool_use':
                        id2name[b['id']] = b['name']; by_tool[atype][b['name']] += 1
                        if b['name'] == 'Bash':
                            cmd = b['input'].get('command', ''); c['bash'] += 1
                            if re.search(r'cargo (test|run|build|clippy|fmt|check)|prove-red|capped\.sh|run-with-memory-cap', cmd): c['cargo'] += 1
                            if re.search(r'\bsed -n\b|\bgrep -n\b', cmd): c['slice_read'] += 1
                        elif b['name'] == 'Read': c['read'] += 1
                        elif b['name'] == 'Edit': c['edit'] += 1
            elif typ == 'user' and isinstance(content, list):
                for b in content:
                    if b.get('type') != 'tool_result': continue
                    s = text_of(b.get('content')); res_chars[atype][id2name.get(b.get('tool_use_id'), '?')] += len(s)
                    if 'hook error' in s:
                        h = hook_name(s); kind = 'reject' if '✗' in s else 'other'; hook_err[(h, kind)] += 1
                        r = re.search(r'✗ ?(.{0,46})', s)
                        if r: hook_reason[h][norm(r.group(1))[:44]] += 1
        calls = [seen[k] for k in order]
        def ctx(u): return u.get('cache_read_input_tokens', 0) + u.get('cache_creation_input_tokens', 0) + u.get('input_tokens', 0)
        for u in calls[1:]:
            cc = u.get('cache_creation_input_tokens', 0)
            if cc > 50000 and cc > 0.5 * ctx(u): rewrites[atype] += 1; rewrite_tokens[atype] += cc
        t = collections.Counter()
        for u in calls: t['input'] += u.get('input_tokens', 0); t['cache_read'] += u.get('cache_read_input_tokens', 0); t['w5m'] += u.get('cache_creation_input_tokens', 0); t['output'] += u.get('output_tokens', 0)
        for k in ('bash', 'cargo', 'slice_read', 'read', 'edit'): med_stats[(atype, k)].append(c[k])
        p = (prompt or '')[:4000]
        if atype == 'implementation-writer':
            k = 'writer:合入后验证' if ('合入后验证' in p or '打全部补丁' in p) else 'writer:交补丁' if '交补丁' in p else 'writer:只修锚点' if ('只修' in p[:800] and 'mutations.tsv' in p[:1200]) else 'writer:接调查/分诊' if ('调查员' in p or '分诊报告' in p or 'investigator' in p or 'mutation-triage' in p) else 'writer:其他'
        else: k = atype
        kinds[k]['n'] += 1; kinds[k]['calls'] += len(calls); kinds[k]['folded'] += folded(t)
        g = len(gov_kw.findall(p)); f = len(fs_kw.findall(p))
        cl = 'governance' if (atype == 'tooling-writer' or (g > f and g >= 3)) else ('filesystem' if f > 0 else 'unclear')
        cls[cl]['n'] += 1; cls[cl]['calls'] += len(calls); cls[cl]['folded'] += folded(t)
    sect('子 agent 的工具调用（按类型，前六种工具）')
    for a, cnt in sorted(by_tool.items(), key=lambda kv: -sum(kv[1].values()))[:10]: print(f'  {a:24} 合计 {sum(cnt.values()):6d} ', ', '.join(f'{k}={v}' for k, v in cnt.most_common(6)))
    sect('子 agent 每个的中位数：Bash、编译测试、切片读、Read、Edit')
    for a in sorted({a for a, _ in med_stats}, key=lambda a: -len(med_stats[(a, 'bash')])):
        row = []
        for k in ('bash', 'cargo', 'slice_read', 'read', 'edit'):
            l = sorted(med_stats[(a, k)]); row.append(l[len(l) // 2] if l else 0)
        print(f'  {a:24} n={len(med_stats[(a, "bash")]):4d} ' + ' '.join(f'{k}={v}' for k, v in zip(('bash', 'cargo', 'slice', 'read', 'edit'), row)))
    sect('工具结果字符数（按类型，百万字符）')
    for a, cnt in sorted(res_chars.items(), key=lambda kv: -sum(kv[1].values()))[:8]: print(f'  {a:24} 合计 {sum(cnt.values()) / 1e6:7.1f}M ', ', '.join(f'{k}={v / 1e6:.1f}M' for k, v in cnt.most_common(4)))
    sect('5 分钟缓存过期后的整份重写（第二次调用起，缓存写超过 5 万且超过上下文一半）')
    for a, v in rewrites.most_common(): print(f'  {a:24} {v:5d} 次  {rewrite_tokens[a] / 1e6:7.1f}M token')
    print('  合计', sum(rewrites.values()), '次', f'{sum(rewrite_tokens.values()) / 1e6:.1f}M token')
    sect('子 agent 侧的钩子错误（按钩子、拒绝或其他）')
    for k, v in hook_err.most_common(): print(f'  {v:5d} {k}')
    for h in ('handback-guard.sh', 'bash-command-detector.sh', 'heavy-test-guard.sh', 'write-guard.sh'):
        print(f'  == {h}')
        for k, v in hook_reason[h].most_common(5): print(f'      {v:4d} {k}')
    sect('实现员按活的种类，与写回两跳')
    for k, cnt in sorted(kinds.items(), key=lambda kv: -kv[1]['folded']):
        if k.startswith('writer') or k in ('kb-scribe', 'kb-spec-drafter', 'investigator', 'mutation-triage'):
            print(f"  {k:22} n={cnt['n']:4d} calls={cnt['calls']:6d} folded={cnt['folded'] / 1e6:7.1f}M avg={cnt['folded'] / cnt['n'] / 1e6:5.1f}M")
    sect('治理类与文件系统类（按派发提示关键词粗判）')
    tot = sum(v['folded'] for v in cls.values())
    for k, cnt in sorted(cls.items(), key=lambda kv: -kv[1]['folded']): print(f"  {k:12} agents={cnt['n']:4d} calls={cnt['calls']:6d} folded={cnt['folded'] / 1e6:7.1f}M ({cnt['folded'] / tot:.0%})")

if __name__ == '__main__':
    print(f'# dispatch.py 口径：{ROOT} 下 {SINCE.date()} 起改动过的会话记录')
    main_sessions(); subagents()
