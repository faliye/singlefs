#!/usr/bin/env python3
"""no-task-numbers-governance-r2 攻方腿的探针：喂写闸、检出钩子 ⑰ 与几份写入工具（干跑），载入判据库判探针与现存真行，逐格打
「name=case label=<格> want=<pass|refuse> got=<pass|refuse> agree=<yes|no>」，末行「name=case_summary cases=N disagreements=M」。
want 是腿量到的结局（打中的格 want=pass 就是闸放过了该拒的写法）；disagreements=0 才算复跑与腿一致。
用法：cases.py --repo <仓根> --frozen <冻结副本> --model <模型目录> --work <新的草稿目录>
  写闸：CLAUDE_PROJECT_DIR 指到 <work>/fake-root（假仓，只放探针文件），钩子与判据库取 --repo 里的那一份。
  检出钩子：只判落在本仓里的写，探针目标用 --repo 下 research/prompts/no-task-numbers-governance-r2-opus-model/live-base/ 的文件（钩子只判、不执行命令）。
  写入工具：--repo 里的 insert-row.py、replace-batch.py 一律 --dry-run，目标同上；append-record.py 用 --root 指到假仓。
  判据库：从 --frozen 载入。
样本进程各自用新会话起，超时只对它那一组发 SIGKILL，结束后核那一组里没有留下进程。退出码：0 全一致；1 有不一致；2 用法错或前提不齐。"""
import argparse, importlib.util, json, os, re, signal, subprocess, sys

RESULTS = []
LIVE_BASE = 'research/prompts/no-task-numbers-governance-r2-opus-model/live-base'
NUMBER_PROBE = '那一路第 7 批交回'


def run(command, cwd, env=None, stdin=None, timeout=300):
    """起一个样本进程（新会话、自己一个进程组），返回 (退出码, 输出)。超时对那一组发 SIGKILL。"""
    process = subprocess.Popen(command, cwd=cwd, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                               text=True, start_new_session=True)
    try:
        output, _ = process.communicate(stdin, timeout=timeout)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        output, _ = process.communicate()
        return 124, (output or '') + '\n（超时，已停它那一组）'
    try:
        os.killpg(process.pid, 0)
        left = True
    except ProcessLookupError:
        left = False
    if left:
        os.killpg(process.pid, signal.SIGKILL)
        output += '\n（结束后那一组里还留着进程，已停）'
    return process.returncode, output


def record(label, want, got, detail=''):
    agree = 'yes' if want == got else 'no'
    RESULTS.append(agree)
    print(f'name=case label={label} want={want} got={got} agree={agree}' + (f' detail={detail}' if detail else ''))


def verdict_of(code):
    return 'pass' if code == 0 else 'refuse'


def write_file(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w', encoding='utf-8') as handle:
        handle.write(text)


def load_library(frozen):
    spec = importlib.util.spec_from_file_location('lib_forbidden_notations', os.path.join(frozen, '.claude', 'gate.d', 'lib-forbidden-notations.py'))
    library = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(library)
    return library


# ── 写闸：Edit / MultiEdit / Write 喂进去，看退 0（放过）还是退 2（拒） ──
def write_guard_cases(repo, work):
    fake = os.path.join(work, 'fake-root')
    environment = dict(os.environ, CLAUDE_PROJECT_DIR=fake, AGENT_HOOK_DETECTIONS=os.path.join(work, 'write-guard-detections.jsonl'))
    hook = ['bash', os.path.join(repo, '.claude', 'hooks', 'write-guard.sh')]

    def feed(label, want, tool, target, tool_input, content=None):
        path = os.path.join(fake, target)
        if content is not None:
            write_file(path, content)
        payload = {'hook_event_name': 'PreToolUse', 'tool_name': tool, 'cwd': fake, 'tool_input': dict(tool_input, file_path=path)}
        code, output = run(hook, fake, environment, json.dumps(payload, ensure_ascii=False))
        if code not in (0, 2):
            record(label, want, f'error{code}', output.strip().splitlines()[-1][:120] if output.strip() else '')
            return
        record(label, want, verdict_of(code))

    fenced = '前文\n```\n' + NUMBER_PROBE + '\n```\n'
    feed('写闸:Edit 把批号挪出围栏（对照，应拒）', 'refuse', 'Edit', 'notes/fenced-a.md',
         {'old_string': '```\n' + NUMBER_PROBE + '\n```\n', 'new_string': NUMBER_PROBE + '\n'}, fenced)
    feed('写闸:Edit 删开头围栏行（新串不带号，批号落到围栏外）', 'pass', 'Edit', 'notes/fenced-b.md',
         {'old_string': '前文\n```\n', 'new_string': '前文\n'}, fenced)
    feed('写闸:Edit 在开头围栏前插一行围栏（后面那一行出了围栏）', 'pass', 'Edit', 'notes/fenced-c.md',
         {'old_string': '前文\n', 'new_string': '前文\n```\n'}, fenced)
    feed('写闸:Edit 把开头围栏的信息串改成带反引号（不再开围栏）', 'pass', 'Edit', 'notes/fenced-d.md',
         {'old_string': '前文\n```\n', 'new_string': '前文\n``` `\n'}, fenced)
    feed('写闸:MultiEdit 一次调用先写进围栏、再删开头围栏行', 'pass', 'MultiEdit', 'notes/multi.md',
         {'edits': [{'old_string': '前文\n', 'new_string': fenced},
                    {'old_string': '前文\n```\n', 'new_string': '前文\n'}]}, '前文\n')
    feed('写闸:Edit replace_all 一处在围栏里一处在外（对照，应拒）', 'refuse', 'Edit', 'notes/count-two.md',
         {'old_string': '占位甲', 'new_string': NUMBER_PROBE, 'replace_all': True}, '```\n占位甲\n```\n占位甲\n')
    verdict = 'research/prompts/demo-r2-main-verification.md'
    base = '## 回看决策\n'
    for label, want, line in (
            ('写闸:判决里 A/B 读数一行带别的任务号', 'pass', 'A/B 读数：任务 21036；顺带记下别的会话那条复跑是任务 21037、门禁是任务 21038'),
            ('写闸:判决里引用块里的 A/B 读数', 'pass', '> A/B 读数：任务 21036'),
            ('写闸:判决里 A/B 读数一行带结局行 task= 段（派发闸放过）', 'refuse', 'A/B 读数：任务 21036，结局 task=21036 type=succeeded exit=0'),
            ('写闸:判决里 A/B 读数一行带同字并列的号（派发闸放过）', 'refuse', 'A/B 读数：任务 21036（同 21037、21038 两臂）'),
            ('写闸:判决里 A/B 读数照骨架填（对照，应放）', 'pass', 'A/B 读数：任务 21036')):
        feed(label, want, 'Edit', verdict, {'old_string': base, 'new_string': base + line + '\n'}, base)
    feed('写闸:记录里 A/B 读数一行（对照，应拒）', 'refuse', 'Edit', 'records/probe.md',
         {'old_string': '前文\n', 'new_string': '前文\nA/B 读数：任务 21036\n'}, '前文\n')
    return fake, environment, hook


# ── 骨架：照 verdict-skeleton.py 打出的模板填，整份 Write 喂写闸 ──
def skeleton_cases(repo, work, fake, environment, hook):
    round_name = 'demo-r3'
    snapshot = os.path.join(fake, 'research', 'prompts', f'{round_name}-snapshot')
    os.makedirs(snapshot, exist_ok=True)
    write_file(os.path.join(snapshot, 'judged-sha256.txt'), '0' * 64 + '  .claude/gate.d/lib-forbidden-notations.py\n' + '1' * 64 + '  scripts/kb/insert-row.py\n')
    out = os.path.join(work, 'skeleton.md')
    code, output = run(['python3', os.path.join(repo, 'scripts', 'agents', 'verdict-skeleton.py'), round_name, '--repository', fake, '--out', out], work)
    if code != 0 or not os.path.isfile(out):
        record('骨架:打得出模板', 'pass', f'error{code}', output.strip().splitlines()[-1][:120] if output.strip() else '')
        return
    text = open(out, encoding='utf-8').read()
    filled = re.sub(r'^（待填：cite-check.*$', '核查：`task-server/task submit -- python3 scripts/agents/cite-check.py research/prompts/demo-r3-opus-output.md`，结局 type=succeeded exit=0。',
                    text, flags=re.M)
    filled = filled.replace('A/B 读数：任务 <号>', 'A/B 读数：任务 21036，有改法的那一支写进 0 次')
    for label, content in (('骨架:原样整份 Write', text), ('骨架:照待填提示填上读数与 A/B 读数之后整份 Write', filled)):
        path = os.path.join(fake, 'research', 'prompts', f'{round_name}-main-verification.md')
        if os.path.exists(path):
            os.remove(path)
        payload = {'hook_event_name': 'PreToolUse', 'tool_name': 'Write', 'cwd': fake, 'tool_input': {'file_path': path, 'content': content}}
        code, output = run(hook, fake, environment, json.dumps(payload, ensure_ascii=False))
        record(label, 'pass', verdict_of(code) if code in (0, 2) else f'error{code}', output.strip().replace('\n', ' ')[:160] if code else '')


# ── 检出钩子 ⑰：同一条命令里两次追加，后一次按起跑前的文件认围栏 ──
def detector_cases(repo, work):
    base = os.path.join(repo, LIVE_BASE)
    for name in ('open-fence.md', 'clean.md'):
        if not os.path.isfile(os.path.join(base, name)):
            record(f'检出钩子:探针目标 {name} 在仓里', 'pass', 'missing')
            return
    environment = dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, 'detector-detections.jsonl'))
    hook = ['bash', os.path.join(repo, '.claude', 'hooks', 'bash-command-detector.sh')]
    open_fence, clean = os.path.join(base, 'open-fence.md'), os.path.join(base, 'clean.md')
    for label, want, command in (
            ('检出钩子:往干净文件追加批号（对照，应拒）', 'refuse', f"printf '%s\\n' '{NUMBER_PROBE}' >> {clean}"),
            ('检出钩子:往开着围栏的文件追加批号（围栏里，应放）', 'pass', f"printf '%s\\n' '{NUMBER_PROBE}' >> {open_fence}"),
            ('检出钩子:同一条命令先追加收尾围栏、再追加批号（批号落到围栏外）', 'pass',
             f"printf '%s\\n' '```' >> {open_fence}; printf '%s\\n' '{NUMBER_PROBE}' >> {open_fence}")):
        payload = {'hook_event_name': 'PreToolUse', 'tool_name': 'Bash', 'cwd': repo, 'tool_input': {'command': command}}
        code, output = run(hook, repo, environment, json.dumps(payload, ensure_ascii=False))
        record(label, want, verdict_of(code) if code in (0, 2) else f'error{code}', '')


# ── 写入工具（干跑）：insert-row 插一行收尾围栏、replace-batch 只按头一处的前文判、两处依次施加 ──
def tool_cases(repo, work, fake):
    base = os.path.join(repo, LIVE_BASE)
    insert_row = ['python3', os.path.join(repo, 'scripts', 'kb', 'insert-row.py')]
    replace_batch = ['python3', os.path.join(repo, 'scripts', 'kb', 'replace-batch.py'), '--dry-run']
    fenced, count_two, clean = (os.path.join(base, name) for name in ('fenced-number.md', 'count-two.md', 'clean.md'))
    code, _ = run(insert_row + [fenced, '--after-literal', '前文', '--line', NUMBER_PROBE, '--dry-run'], repo)
    record('insert-row:围栏外插批号（对照，应拒）', 'refuse', verdict_of(code))
    code, _ = run(insert_row + [fenced, '--after-literal', '前文', '--line', '```', '--dry-run'], repo)
    record('insert-row:插一行围栏让下面的批号出围栏', 'pass', verdict_of(code))
    specs = {
        'replace-batch:count 2 一处在围栏里一处在外': ('pass', [{'file': count_two, 'old': '占位甲', 'new': NUMBER_PROBE, 'count': 2}]),
        'replace-batch:只换围栏外那一处（对照，应拒）': ('refuse', [{'file': count_two, 'old': '```\n占位甲\n```\n占位甲', 'new': '```\n占位甲\n```\n' + NUMBER_PROBE}]),
        'replace-batch:一份规格两处依次施加（先写进围栏、再删开头围栏行）': ('pass', [
            {'file': clean, 'old': '前文\n', 'new': '前文\n```\n' + NUMBER_PROBE + '\n```\n'},
            {'file': clean, 'old': '前文\n```\n', 'new': '前文\n'}]),
    }
    for index, (label, (want, spec)) in enumerate(specs.items()):
        spec_path = os.path.join(work, f'replace-batch-spec-{index}.json')
        write_file(spec_path, json.dumps(spec, ensure_ascii=False))
        code, output = run(replace_batch + [spec_path], repo)
        record(label, want, verdict_of(code))
    spec_path = os.path.join(work, 'replace-batch-task-number.json')
    write_file(spec_path, json.dumps([{'file': clean, 'old': '前文\n', 'new': '前文\n日志里任务 21036 那一行\n'}], ensure_ascii=False))
    code, output = run(replace_batch + [spec_path], repo)
    record('replace-batch:拒任务号时汇总行说成「写了会话号」', 'yes', 'yes' if ('新串写了任务号' in output and '新串写了会话号' in output) else 'no')
    ledger = ['python3', os.path.join(repo, 'scripts', 'kb', 'ledger-append-section.py'), '--title', '探针（2026-11-19）']
    body = os.path.join(work, 'ledger-body.txt')
    write_file(body, '那一路会话的改动还没交回。\n')
    code, _ = run(ledger + ['--body-file', body, '--judgement', '别的会话第 105 批', '--dry-run'], repo)
    record('ledger-append-section:索引行判定格里的批号（记录里的表格行）', 'pass', verdict_of(code))


RECORDS_PROBES = (   # (探针, 腿量到的结局)；append-record.py --dry-run 追加到只有一行「前文」的假记录
    ('- 依赖表追加第 7 批（照现存记录行的写法）', 'pass'), ('- 那一路会话写入第 7 批的改动', 'pass'), ('- 别的会话覆写批 105 的补丁', 'pass'),
    ('- 这一批追加批注后第 7 批交回', 'pass'), ('- 它的批 105 处理的是写闸', 'pass'), ('- 它的批 105 行为不对', 'pass'),
    ('- 它的批 105 项目里改了三份文件', 'pass'),
    ('- 日志 `~/.local/state/singlefs/task-server/logs/task-b0f6b921-21036.attempt-1.log` 第 19–20 行', 'pass'),
    ('- 编译目录 `builds/task-b0f6b921-21036-1` 是服务留的', 'pass'),
    ('- 结局 type=succeeded exit=0 log=/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-21036.attempt-1.log', 'pass'),
    ('- 长跑拆分：项 x 用时 9000 秒，任务 21036；顺带记下别的会话的任务 21037、21038 也慢', 'pass'),
    ('| 别的会话的第 105 批还没交回', 'pass'), ('- 任务 21036 条件不满足就重交', 'pass'), ('- TASK 21036 退 0', 'pass'),
    ('- 它的批 105 改了三份文件', 'refuse'), ('- 日志里任务 21036 那一行', 'refuse'), ('- **长跑拆分**：项 x，任务 21036', 'refuse'),
    ('- 它的第 105 批写入了三份文件', 'refuse'),
)
VERDICT_PROBES = (   # (探针, 判据库在判决里的结局, 派发闸收不收)
    ('A/B 读数：任务 21036；顺带记下别的会话那条复跑是任务 21037、门禁是任务 21038', 'pass', True),
    ('> A/B 读数：任务 21036', 'pass', True),
    ('A/B 读数：任务 21036，结局 task=21036 type=succeeded exit=0', 'refuse', True),
    ('A/B 读数：任务 21036（同 21037、21038 两臂）', 'refuse', True),
    ('A/B 读数：任务 21036', 'pass', True), ('1. A/B 读数：任务 21036', 'refuse', False), ('A/B 读数：TASK 21036', 'pass', True),
    ('A/B 读数：任务 21036 与任务 21037', 'pass', True), ('A/B 读数：见日志 task-b0f6b921-21036.attempt-1.log', 'pass', False),
    ('A/B 读数：任务 21036，结局 `TASK_SERVER_RESULT task=21036 type=succeeded exit=0`', 'refuse', True),
    ('A/B 读数：任务 21036；task-server/task log 21036 末行', 'refuse', True),
)


def dispatch_patterns(frozen):
    """派发闸认零轮读数那一行的三条从它的源码原样抽出来执行（抽法同 scripts/agents/verdict-skeleton.py 的 dispatch_guard_reading_patterns）。"""
    guard = open(os.path.join(frozen, '.claude', 'hooks', 'runner-dispatch-guard.sh'), encoding='utf-8').read()
    namespace = {'re': re}
    for name in ('LINE_START', 'ZERO_ROUND_READING_LINE', 'ZERO_ROUND_TASK_NUMBER'):
        exec(re.search(rf'^{name} = .*$', guard, re.M).group(0), namespace)
    return namespace['ZERO_ROUND_READING_LINE'], namespace['ZERO_ROUND_TASK_NUMBER']


def library_cases(repo, frozen, work, fake, library):
    form = 'task-and-session-numbers'
    reading_line, task_number = dispatch_patterns(frozen)
    for index, (probe, want, dispatch_want) in enumerate(VERDICT_PROBES, 1):
        got = 'refuse' if library.all_hits(form, probe, 'research/prompts/demo-r2-main-verification.md') else 'pass'
        record(f'判据库:判决里的 A/B 探针 {index}', want, got)
        accepts = any(task_number.search(m.group('value')) for m in reading_line.finditer(probe))
        record(f'派发闸:同一行 A/B 探针 {index}（判据库 {want}）', 'accept' if dispatch_want else 'reject', 'accept' if accepts else 'reject')
    write_file(os.path.join(fake, '.claude', 'gate.d', 'lib-forbidden-notations.py'),
               open(os.path.join(frozen, '.claude', 'gate.d', 'lib-forbidden-notations.py'), encoding='utf-8').read())
    record_path = os.path.join(fake, 'records', 'append-probe.md')
    write_file(record_path, '前文\n')
    probes = os.path.join(work, 'records-probes.txt')
    write_file(probes, '\n'.join(probe for probe, _ in RECORDS_PROBES) + '\n')
    code, output = run(['python3', os.path.join(repo, 'scripts', 'kb', 'append-record.py'), record_path, probes, '--dry-run', '--root', fake], repo)
    if code not in (0, 7):
        record('append-record:在假仓上跑得起来', 'pass', f'error{code}', output.strip().replace('\n', ' ')[:160])
        return
    refused = {int(number) for number in re.findall(r'第 (\d+) 行', output)}
    for index, (probe, want) in enumerate(RECORDS_PROBES, 1):
        record(f'append-record:记录探针 {index}', want, 'refuse' if index + 1 in refused else 'pass', probe[:40])
    kb_path = '.claude/kb/experiments/182-被抛弃根读不出的三条岔路.md'
    lines = open(os.path.join(repo, kb_path), encoding='utf-8').read().split('\n')
    real = next((line for line in lines if '同 196642 / 393384' in line), None)
    if real is None:
        record('判据库:kb 现存真行「同 196642 / 393384」在', 'pass', 'missing')
        return
    record('判据库:kb 现存真行（实验数据「同 196642 / 393384」）被当任务号', 'refuse', 'refuse' if library.all_hits(form, real, kb_path) else 'pass')
    quoted = real.replace('同 196642 / 393384', '`同 196642 / 393384`')
    record('判据库:同一行照出路放进反引号', 'refuse', 'refuse' if library.all_hits(form, quoted, kb_path) else 'pass')


def flipped_hits(library, before, after, relative):
    """我提的改法（只在我的模型上量过、被攻过零轮）：写之前、写之后整份各判一遍，写之后多出来的违例行（按行文计数）就拒。"""
    form = 'task-and-session-numbers'
    counted = {}
    for number, _, _, _ in library.all_hits(form, before, relative):
        line = before.split('\n')[number - 1]
        counted[line] = counted.get(line, 0) + 1
    added = []
    for number, _, _, matched in library.all_hits(form, after, relative):
        line = after.split('\n')[number - 1]
        if counted.get(line, 0) > 0:
            counted[line] -= 1
        else:
            added.append(matched)
    return added


def fix_model_cases(library):
    fenced = '前文\n```\n' + NUMBER_PROBE + '\n```\n'
    relative = 'research/prompts/demo-r2-model/notes.md'
    for label, want, before, after in (
            ('改法模型:删开头围栏行', 'refuse', fenced, '前文\n' + NUMBER_PROBE + '\n```\n'),
            ('改法模型:插一行围栏', 'refuse', fenced, '前文\n```\n```\n' + NUMBER_PROBE + '\n```\n'),
            ('改法模型:count 2 一处在外', 'refuse', '```\n占位甲\n```\n占位甲\n', '```\n' + NUMBER_PROBE + '\n```\n' + NUMBER_PROBE + '\n'),
            ('改法模型:批号写进围栏（应放）', 'pass', '前文\n', fenced),
            ('改法模型:文件里已有旧违例、这一次只改别处（应放）', 'pass', '它的批 105 改了三份\n前文\n', '它的批 105 改了三份\n前文改了\n')):
        record(label, want, 'refuse' if flipped_hits(library, before, after, relative) else 'pass')


SCAN_DIRECTORIES = ('records', '.claude/kb', 'research/prompts', '.claude/rules', '.claude/agents', '.claude/skills')
LOG_PATH = re.compile(r'task-[0-9a-f]{8}-\d{4,}')
SAME_JOINED = re.compile(r'(?<![相不共如协合])同\s*\d{4,}\s*(?:/|与|和)\s*\d{4,}')


def scan_real_lines(repo, library):
    """现存真行：日志、编译目录、现场目录名里带任务号而整行放过的；按同一分句攒批放宽放过的批号（不在表格行里）；「同 N / M」被当任务号的非任务行。
    每份文件整份交库判（围栏照认），命中的行号与候选行对出放过的。打「name=scan_class class=<类> dir=<顶层目录> lines=<行数>」与逐行「name=scan_line」。"""
    form = 'task-and-session-numbers'
    batch_pattern = next(pattern for key, _, pattern, _ in library.FORMS_BY_KEY[form]['patterns'] if key == 'batch-numbers')
    counts = {}
    for directory in SCAN_DIRECTORIES:
        for base, _, files in os.walk(os.path.join(repo, directory)):
            for name in sorted(files):
                if not name.lower().endswith(('.md', '.tsv', '.txt')):
                    continue
                relative = os.path.relpath(os.path.join(base, name), repo)
                if library.is_excluded(form, relative) or relative.startswith(('research/prompts/no-task-numbers-governance-r2-opus', 'research/prompts/no-task-numbers-governance-r2-sonnet')):
                    continue
                try:
                    text = open(os.path.join(repo, relative), encoding='utf-8').read()
                except (UnicodeDecodeError, OSError):
                    continue
                lines = text.split('\n')
                hit_lines = {number for number, _, _, _ in library.all_hits(form, text, relative)}
                fenced = library.fenced_line_flags(lines)
                top = relative.split('/')[0] if not relative.startswith('.claude/') else '/'.join(relative.split('/')[:2])
                for number, line in enumerate(lines, 1):
                    classes = []
                    if LOG_PATH.search(line) and number not in hit_lines:
                        classes.append('log-path-task-number-passed')
                    if number not in hit_lines and not fenced[number - 1] and not line.lstrip().startswith('|'):
                        for match in batch_pattern.finditer(line):
                            if library.inside_code_span(line, match.start()):
                                continue
                            before = line[:match.start()]
                            clause = before[max(before.rfind(mark) for mark in library.CLAUSE_BREAKS) + 1:]
                            if library.BATCH_SIZE_WORDS.search(clause) and relative.startswith('records/'):
                                classes.append('clause-batch-size-exempt-in-records')
                                break
                    if SAME_JOINED.search(line) and number in hit_lines and not relative.startswith('records/') \
                            and not re.search(r'任务|[Tt]ask', line):
                        classes.append('same-joined-refused-outside-records')
                    for cls in classes:
                        counts[(cls, top)] = counts.get((cls, top), 0) + 1
                        print(f'name=scan_line class={cls} at={relative}:{number} text={line.strip()[:120]}')
    for (cls, top), number in sorted(counts.items()):
        print(f'name=scan_class class={cls} dir={top} lines={number}')
    print(f'name=scan_total classes={len({cls for cls, _ in counts})} lines={sum(counts.values())}')


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    for option in ('--repo', '--frozen', '--model', '--work'):
        parser.add_argument(option, required=True)
    arguments = parser.parse_args()
    repo, frozen, work = (os.path.realpath(path) for path in (arguments.repo, arguments.frozen, arguments.work))
    if not os.path.isfile(os.path.join(frozen, '.claude', 'gate.d', 'lib-forbidden-notations.py')):
        print(f'  ✗ 冻结副本里没有判据库：{frozen}', file=sys.stderr)
        return 2
    os.makedirs(work, exist_ok=True)
    library = load_library(frozen)
    fake, environment, hook = write_guard_cases(repo, work)
    skeleton_cases(repo, work, fake, environment, hook)
    detector_cases(repo, work)
    tool_cases(repo, work, fake)
    library_cases(repo, frozen, work, fake, library)
    fix_model_cases(library)
    disagreements = RESULTS.count('no')
    print(f'name=case_summary cases={len(RESULTS)} disagreements={disagreements}')
    scan_real_lines(repo, library)
    return 0 if disagreements == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
