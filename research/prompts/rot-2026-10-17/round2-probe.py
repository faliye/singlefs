#!/usr/bin/env python3
"""第二轮知识腐烂探针：扫仓里的现状文档，把机械能判的疑似腐烂句列成候选表，给回扫员逐行判。

只读：用 git ls-files 与 git grep 看仓，不改仓内任何文件，不起编译，不跑门禁。

用法：
  python3 round2-probe.py <仓根> --out 候选.tsv [--only P1,P3]
  python3 round2-probe.py --selftest

扫的文档：.claude/kb/**/*.md（不含 decisions-history.md、experiments-history.md；"## 历史版本"之后的行不扫）、
crates/singlefs-checker/invariants/*.md、.claude/rules/*.md、.claude/agents/*.md、.claude/agent-common.md、
.claude/main-agent.md、.claude/skills/*/*.md、CLAUDE.md、README.md、README.zh.md、README.ja.md、scripts/**/README.md。
围栏代码块（```）里的行不扫。

探针（候选表第一列是探针号加四位编号：P10001、P30001……）：
  P1 反引号里的仓内路径，在工作区与 git 跟踪文件里都不存在。
  P3 反引号里的 Rust 标识符，在 crates/ scripts/ .claude/gate.d/ .claude/hooks/ .claude/scripts/ 里零命中。
  P4 引 checks-owed.md 已还清表里的 C 编号，同一句里却带「欠」「前置」之类开着的字样。
  P5 不变量状态列（或决策分项进度）与同句的「未实现」「已实现」说法相反。
  P7 已删的 agent 定义名、旧门禁阶段名、不在登记里的「X 格」。
  P9 带日期的现状句形态（只报形态，不判）。

候选表五列：组、旧事实、新事实、载体:行号、原文（截 500 字）。
弄坏开关：环境变量 ROUND2_PROBE_BREAK=<探针号>（如 P3）让那一条探针不报，--selftest 必须红。
"""
import argparse
import concurrent.futures
import difflib
import os
import re
import shutil
import subprocess
import sys
import tempfile

PROBES = ['P1', 'P3', 'P4', 'P5', 'P7', 'P9']
BREAK_VARIABLE = 'ROUND2_PROBE_BREAK'
HEADER = '组\t旧事实\t新事实\t载体\t原文\n'

PATH_PREFIXES = ('.claude/', 'crates/', 'scripts/', 'records/', 'research/', 'briefs/', 'litmus/')
PATH_EXTENSIONS = ('.md', '.rs', '.sh', '.py', '.tsv', '.toml', '.wgsl')
CODE_DIRECTORIES = ['crates', 'scripts', '.claude/gate.d', '.claude/hooks', '.claude/scripts']
SENTENCE_SPLIT = re.compile(r'[。；|]')
IDENTIFIER = re.compile(r'[A-Za-z_][A-Za-z0-9_]*')
BUILTIN_AGENT_TYPES = {'general-purpose', 'explore', 'plan', 'statusline-setup', 'claude-code-guide'}
STAGE_NAME = re.compile(r'(?<![A-Za-z0-9_./-])((?:checker-tier|layer0|doc|code|harness)(?:-[a-z0-9]+)+)(?![A-Za-z0-9_-])')
CELL_NAME = re.compile(r'(?<![A-Za-z0-9_-])([a-z][a-z0-9]*(?:-[a-z0-9]+)+)[`」]?\s*格(?![式局外子数言调力])')
AGENT_CONTEXTS = [
    re.compile(r'(?<![指分委调改重])派\s*[`「]([a-z][a-z0-9-]*)[`」]'),
    re.compile(r'[`「]([a-z][a-z0-9-]*)[`」]\s*交回'),
    re.compile(r'agent\s*[`「]([a-z][a-z0-9-]*)[`」]'),
]
OPEN_WORDS = ('欠', '前置', '等', '要先', '仍', '挡', '没做', '未做', '还没')
PAID_WORDS = ('已还清', '已还', '作废', '并入', '还清', '曾')
NOT_IMPLEMENTED_WORDS = ('未实现', '还没实现', '没实现', '待实现')
DATE = r'\d{4}-\d{2}-\d{2}'
DATED_STATE = [
    re.compile(r'现状[（(]' + DATE),
    re.compile(DATE + r'\s*现状[:：]'),
    re.compile(r'截至\s*' + DATE),
    re.compile(DATE + r'\s*起仍'),
]
INVARIANT_ID = re.compile(r'(?<![\w.-])(I-\d+\.\d+)(?!\d)')
DECISION_MENTION = re.compile(r'(?:D(\d+)(?:（[^）]*）)?\s*)?已定项\s*(\d+)')
CONNECTOR_ONLY = re.compile(r'^[\s、，,/和与及]*$')


# ---------------------------------------------------------------- 仓的现状

def run_git(root, *arguments, check=True):
    result = subprocess.run(['git', '-C', root, *arguments], capture_output=True, text=True, errors='replace')
    if check and result.returncode not in (0, 1):
        raise RuntimeError(f'git {" ".join(arguments)} 退出 {result.returncode}：{result.stderr.strip()}')
    return result


def list_files(root, *extra):
    result = subprocess.run(['git', '-C', root, 'ls-files', '-z', *extra], capture_output=True)
    return [name for name in result.stdout.decode('utf-8', 'replace').split('\0') if name]


class Repository:
    def __init__(self, root):
        self.root = os.path.abspath(root)
        self.tracked = set(list_files(self.root))
        self.all_known = self.tracked | set(list_files(self.root, '--others', '--exclude-standard'))
        self.tracked_directories = set()
        self.by_basename = {}
        self.stems = set()
        for path in self.tracked:
            parts = path.split('/')
            for index in range(1, len(parts)):
                self.tracked_directories.add('/'.join(parts[:index]))
            self.by_basename.setdefault(parts[-1], []).append(path)
            for part in parts:
                self.stems.add(os.path.splitext(part)[0])
                self.stems.add(part)
        self.agent_names = self._agent_names()
        self.stage_names = self._stage_names()
        self.cell_names = self._cell_names()
        self.old_terms = self._old_terms()
        self.table_headers = set()
        self.invariant_status = {}
        self.decision_status = {}
        self.closed_debts = {}
        self.open_debts = set()
        self.closed_range = None

    def read_text(self, relative):
        try:
            with open(os.path.join(self.root, relative), encoding='utf-8', errors='replace') as handle:
                return handle.read()
        except OSError:
            return ''

    def exists(self, relative):
        relative = relative.rstrip('/')
        return (os.path.exists(os.path.join(self.root, relative)) or relative in self.tracked
                or relative in self.tracked_directories)

    def _agent_names(self):
        names = set()
        for path in self.all_known:
            match = re.fullmatch(r'\.claude/agents/([^/]+)\.md', path)
            if match and os.path.exists(os.path.join(self.root, path)):
                names.add(match.group(1))
        return names

    def _stage_names(self):
        names = set()
        for path in self.all_known:
            match = re.fullmatch(r'\.claude/gate\.d/([^/]+)\.sh', path)
            if match:
                names.add(match.group(1))
        return names

    def _cell_names(self):
        names = set()
        for path in sorted(self.all_known):
            if not (path.startswith(('.claude/gate.d/', 'scripts/gate/')) and path.endswith(('.sh', '.py'))):
                continue
            if '/fixtures/' in path:
                continue
            for line in self.read_text(path).split('\n'):
                match = re.match(r'\s*#\s*gate-cell:\s*(\S+)', line)
                if match:
                    names.add(match.group(1))
        return names

    def _old_terms(self):
        """term-renames.md 表里「旧」列：匹配列写「正则」的当正则，别的当整串。返回 [(整串或 None, 正则或 None)]。"""
        terms = []
        for line in self.read_text('.claude/kb/term-renames.md').split('\n'):
            if not line.startswith('|'):
                continue
            cells = [cell.strip() for cell in line.strip().strip('|').split('|')]
            old = cells[0].strip('`') if cells else ''
            if not old or old == '旧' or set(old) <= set('-: '):
                continue
            if len(cells) > 2 and cells[2] == '正则':
                try:
                    terms.append((None, re.compile(old)))
                except re.error:
                    terms.append((old, None))
            else:
                terms.append((old, None))
        return terms

    def mentions_old_term(self, name):
        for plain, pattern in self.old_terms:
            if plain is not None and plain in name:
                return True
            if pattern is not None and pattern.search(name):
                return True
        return False


# ---------------------------------------------------------------- 扫哪些文档

def is_scanned_document(path):
    if path.startswith('.claude/kb/') and path.endswith('.md'):
        return os.path.basename(path) not in ('decisions-history.md', 'experiments-history.md')
    patterns = [
        r'crates/singlefs-checker/invariants/[^/]+\.md',
        r'\.claude/rules/[^/]+\.md',
        r'\.claude/agents/[^/]+\.md',
        r'\.claude/agent-common\.md',
        r'\.claude/main-agent\.md',
        r'\.claude/skills/[^/]+/[^/]+\.md',
        r'CLAUDE\.md', r'README\.md', r'README\.zh\.md', r'README\.ja\.md',
        r'scripts/(?:.+/)?README\.md',
    ]
    return any(re.fullmatch(pattern, path) for pattern in patterns)


class Document:
    def __init__(self, path, lines):
        self.path = path
        self.lines = lines  # [(行号, 文本)]，已去掉围栏代码块与历史版本节


def load_documents(repository):
    documents = []
    for path in sorted(repository.all_known):
        if not is_scanned_document(path):
            continue
        full = os.path.join(repository.root, path)
        if not os.path.isfile(full):
            continue
        in_fence = False
        kept = []
        for number, line in enumerate(repository.read_text(path).split('\n'), start=1):
            if path.startswith('.claude/kb/') and line.startswith('## 历史版本'):
                break
            if line.lstrip().startswith('```'):
                in_fence = not in_fence
                continue
            if in_fence:
                continue
            kept.append((number, line))
        documents.append(Document(path, kept))
    return documents


def gather_registry_columns(repository, documents):
    columns = set()
    for document in documents:
        for index, (_, line) in enumerate(document.lines[:-1]):
            if line.startswith('|') and re.match(r'^\|[\s:|-]+\|?\s*$', document.lines[index + 1][1]):
                for cell in line.strip().strip('|').split('|'):
                    columns.add(cell.strip().strip('`'))
    for path in repository.tracked:
        if path.endswith('.tsv') and '/fixtures/' not in path:
            for line in repository.read_text(path).split('\n'):
                if line and not line.startswith('#'):
                    columns.update(cell.strip() for cell in line.split('\t'))
                    break
    return columns


# ---------------------------------------------------------------- 事实：不变量、决策分项、欠账

def gather_invariant_status(repository):
    statuses = {}
    for path in sorted(repository.all_known):
        if not re.fullmatch(r'crates/singlefs-checker/invariants/0[^/]*\.md', path):
            continue
        status_column = None
        for number, line in enumerate(repository.read_text(path).split('\n'), start=1):
            if not line.startswith('|'):
                continue
            cells = [cell.strip() for cell in line.strip().strip('|').split('|')]
            if cells and cells[0] == 'ID':
                status_column = next((index for index, cell in enumerate(cells) if '状态' in cell), None)
                continue
            match = re.fullmatch(r'(I-\d+\.\d+)', cells[0]) if cells else None
            if match and status_column is not None and status_column < len(cells):
                cell = cells[status_column]
                kind = '已实现' if cell.startswith('已实现') else '未实现' if cell.startswith('未实现') else None
                if kind:
                    statuses[match.group(1)] = (kind, f'{path}:{number}')
    return statuses


def gather_decision_status(repository):
    statuses = {}
    for path in sorted(repository.all_known):
        if not re.fullmatch(r'\.claude/kb/decisions/[^/]+\.md', path):
            continue
        decision = None
        section = None
        for number, line in enumerate(repository.read_text(path).split('\n'), start=1):
            match = re.match(r'## D(\d+)\s', line)
            if match and decision is None:
                decision = match.group(1)
            if line.startswith('### '):
                section = line[4:].strip()
                continue
            if line.startswith('## 历史版本'):
                break
            if decision is None or section != '已定项':
                continue
            row = re.match(r'\|\s*(\d+)\s*\|', line)
            progress = re.search(r'\*\*进度：(已实现|未实现|无对象)。\*\*', line)
            if row and progress and progress.group(1) in ('已实现', '未实现'):
                statuses[(decision, row.group(1))] = (progress.group(1), f'{path}:{number}')
    return statuses


def gather_debts(repository):
    path = '.claude/kb/checks-owed.md'
    closed, opened = {}, set()
    start = end = None
    lines = repository.read_text(path).split('\n')
    for index, line in enumerate(lines):
        if start is None and line.strip() == '### 已还清':
            start = index
            continue
        if start is not None and end is None and re.match(r'#{1,3} ', line):
            end = index
    if start is not None and end is None:
        end = len(lines)
    for index, line in enumerate(lines):
        match = re.match(r'\|\s*(C\d+)\s*\|', line)
        if not match:
            continue
        if start is not None and start < index < end:
            closed.setdefault(match.group(1), index + 1)
        else:
            opened.add(match.group(1))
    for name in opened:
        closed.pop(name, None)
    return closed, opened, (start + 1, end) if start is not None else None


# ---------------------------------------------------------------- 探针

def clip(text):
    return text.replace('\t', ' ').strip()[:500]


def excerpt(line, focus):
    """原文列：行不超过 500 字就整行；超过就取命中处前后的一段（原样子串，不加省略号），不让命中处被截掉。"""
    line = line.replace('\t', ' ').strip()
    if len(line) <= 500:
        return line
    index = max(line.find(focus), 0)
    start = max(0, index - 200)
    return line[start:start + 500]


def probe_p1(repository, documents):
    rows = []
    basename_index = repository.by_basename
    for document in documents:
        for number, line in document.lines:
            for span in re.findall(r'`([^`\n]+)`', line):
                for token in span.split():
                    core = path_core(token)
                    if core is None:
                        continue
                    if path_exists(repository, core, document.path):
                        continue
                    same = basename_index.get(os.path.basename(core.rstrip('/')), [])
                    new_fact = '无' if not same else '同名文件在 ' + '、'.join(sorted(same)[:3])
                    rows.append((f'路径 {core} 在工作区与 git 都不存在', new_fact, f'{document.path}:{number}', excerpt(line, token)))
    return rows


def path_core(token):
    token = token.strip('"\'，,;；。()（）[]')
    if '=' in token:
        token = token.rsplit('=', 1)[1]
    if token.startswith(':!'):
        token = token[2:]
    if not token or token[0] in '-/~$' or token.startswith('../') or '://' in token:
        return None
    if re.search(r'[<>*…{}$?|]|\.\.\.', token):
        return None
    token = token.split('「')[0].split('#')[0].split('@')[0].split('::')[0]
    token = re.sub(r'^[A-Za-z0-9~^_.-]*:(?=[A-Za-z.])', '', token)
    token = token.lstrip(':')
    token = re.sub(r':-.*$', '', token)
    token = re.sub(r'\(.*$', '', token)
    token = re.sub(r'(:\d+(?:[-:,]\d+)*)+$', '', token).rstrip(':')
    if token.startswith('./'):
        token = token[2:]
    if not token or token[-1] in '-_':
        return None
    if token.startswith(PATH_PREFIXES):
        return token
    if '/' in token and token.endswith(PATH_EXTENSIONS):
        return token
    return None


def path_exists(repository, core, document_path):
    if repository.exists(core):
        return True
    if core.startswith(PATH_PREFIXES):
        return False
    relative = os.path.normpath(os.path.join(os.path.dirname(document_path), core))
    if repository.exists(relative):
        return True
    suffix = '/' + core
    return any(path.endswith(suffix) for path in repository.by_basename.get(os.path.basename(core), []))


def code_identifiers(span):
    span = span.strip()
    span = re.sub(r'\(.*\)$', '', span).rstrip('!')
    parts = span.split('::')
    if not all(IDENTIFIER.fullmatch(part) for part in parts):
        return []
    return [part for part in parts if qualifies_as_identifier(part)]


def qualifies_as_identifier(name):
    if len(name) < 8:
        return False
    if '_' in name:
        return True
    uppercase = sum(1 for character in name if character.isupper())
    return uppercase >= 2 and any(character.islower() for character in name)


def is_upper_snake(name):
    return '_' in name and name == name.upper()


def grep_hits(root, name, extra_pathspec=None):
    pathspec = extra_pathspec if extra_pathspec is not None else CODE_DIRECTORIES
    result = subprocess.run(['git', '-C', root, 'grep', '--untracked', '-F', '-q', '-e', name, '--', *pathspec],
                            capture_output=True)
    return result.returncode == 0


def probe_p3(repository, documents):
    occurrences = []
    for document in documents:
        for number, line in document.lines:
            seen = set()
            for span in re.findall(r'`([^`\n]+)`', line):
                for name in code_identifiers(span):
                    if name in seen:
                        continue
                    seen.add(name)
                    occurrences.append((name, document.path, number, line))
    columns = gather_registry_columns(repository, documents)
    names = sorted({name for name, *_ in occurrences})
    names = [name for name in names if name not in repository.cell_names and name not in columns]
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        found = dict(zip(names, pool.map(lambda name: grep_hits(repository.root, name), names)))
    missing = {name for name, hit in found.items() if not hit}
    other_places = {}
    non_markdown = [':!*.md', ':!records', ':!research', ':!briefs']
    for name in sorted(missing):
        result = subprocess.run(['git', '-C', repository.root, 'grep', '--untracked', '-l', '-F', '-e', name, '--', '.', *non_markdown],
                                capture_output=True, text=True, errors='replace')
        other_places[name] = [item for item in result.stdout.split('\n') if item][:2]
    rows = []
    for name, path, number, line in occurrences:
        if name not in missing:
            continue
        if is_upper_snake(name) and ('环境变量' in line or other_places[name]):
            continue
        new_fact = '无（非 md 文件里也没有）' if not other_places[name] else '别处命中：' + '、'.join(other_places[name])
        rows.append((f'代码名 {name} 在 {"、".join(CODE_DIRECTORIES)} 里零命中', new_fact, f'{path}:{number}', excerpt(line, name)))
    return rows


def probe_p4(repository, documents):
    closed, closed_range = repository.closed_debts, repository.closed_range
    rows = []
    for document in documents:
        for number, line in document.lines:
            if document.path == '.claude/kb/checks-owed.md' and closed_range and closed_range[0] <= number <= closed_range[1]:
                continue
            for sentence in SENTENCE_SPLIT.split(line):
                ids = sorted({'C' + value for value in re.findall(r'(?<![A-Za-z0-9])C(\d+)(?![A-Za-z0-9])', sentence)} & set(closed))
                if not ids:
                    continue
                bare = re.sub(r'(C\d+)（[^）]*）', r'\1', sentence)  # 编号后面括着的简称是名字，不是这一句的断言
                hit_words = [word for word in OPEN_WORDS if word in bare]
                if not hit_words or any(word in bare for word in PAID_WORDS):
                    continue
                old = '、'.join(ids) + f' 已在 checks-owed.md「已还清」表，这一句把它写成开着（命中字样：{"、".join(hit_words)}）'
                new = '；'.join(f'{name} 在已还清表第 {closed[name]} 行' for name in ids)
                rows.append((old, new, f'{document.path}:{number}', sentence.strip()))
                break
    return rows


def segments_after_mentions(sentence, mentions):
    """mentions: [(start, end, key)]。每个提及的片段：提及之后到下一个非连接词隔开的提及之前。"""
    segments = []
    for index, (start, end, key) in enumerate(mentions):
        stop = len(sentence)
        cursor_end = end
        for next_start, next_end, _ in mentions[index + 1:]:
            if CONNECTOR_ONLY.match(sentence[cursor_end:next_start]):
                cursor_end = next_end
                continue
            stop = next_start
            break
        segments.append((key, sentence[cursor_end:stop] if cursor_end <= stop else ''))
    return segments


def decision_mentions(sentence):
    mentions, last_decision = [], None
    for match in DECISION_MENTION.finditer(sentence):
        if match.group(1):
            last_decision = match.group(1)
        key = (last_decision, match.group(2)) if last_decision else None
        mentions.append((match.start(), match.end(), key))
    return mentions


def probe_p5(repository, documents):
    rows = []
    invariants = repository.invariant_status
    decisions = repository.decision_status
    for document in documents:
        for number, line in document.lines:
            for sentence in SENTENCE_SPLIT.split(line):
                found = []
                mentions = [(m.start(), m.end(), m.group(1)) for m in INVARIANT_ID.finditer(sentence)]
                for key, segment in segments_after_mentions(sentence, mentions):
                    found.append((key, segment, invariants.get(key), key))
                mentions = decision_mentions(sentence)
                for key, segment in segments_after_mentions(sentence, mentions):
                    label = f'D{key[0]} 已定项 {key[1]}' if key else None
                    found.append((key, segment, decisions.get(key) if key else None, label))
                for key, segment, status, label in found:
                    if status is None:
                        continue
                    kind, where = status
                    if kind == '已实现' and any(word in segment for word in NOT_IMPLEMENTED_WORDS):
                        rows.append((f'{label} 被写成未实现', f'状态/进度为已实现（{where}）', f'{document.path}:{number}', excerpt(sentence, label)))
                    elif kind == '未实现' and '已实现' in segment:
                        rows.append((f'{label} 被写成已实现', f'状态/进度为未实现（{where}）', f'{document.path}:{number}', excerpt(sentence, label)))
    return rows


def probe_p7(repository, documents):
    rows = []
    good_agents = repository.agent_names | BUILTIN_AGENT_TYPES
    for document in documents:
        for number, line in document.lines:
            location = f'{document.path}:{number}'
            reported = set()
            for pattern in AGENT_CONTEXTS:
                for match in pattern.finditer(line):
                    name = match.group(1)
                    if name in good_agents or name in reported or repository.mentions_old_term(name):
                        continue
                    reported.add(name)
                    close = difflib.get_close_matches(name, sorted(repository.agent_names), n=2)
                    new_fact = '现有定义：' + '、'.join(sorted(repository.agent_names)[:12]) + (f'；近似：{"、".join(close)}' if close else '')
                    rows.append((f'agent 定义 {name} 在 .claude/agents/ 里不存在', new_fact, location, excerpt(line, name)))
            for match in STAGE_NAME.finditer(line):
                name = match.group(1)
                after = line[match.end():match.end() + 1]
                if after in ('/',) or re.match(r'\.[A-Za-z]', line[match.end():match.end() + 2]):
                    continue
                if (name in repository.stage_names or name in repository.stems or name in repository.cell_names
                        or repository.mentions_old_term(name) or name in reported):
                    continue
                reported.add(name)
                close = difflib.get_close_matches(name, sorted(repository.stage_names), n=2)
                new_fact = f'.claude/gate.d/ 现有阶段 {len(repository.stage_names)} 个' + (f'；近似：{"、".join(close)}' if close else '')
                rows.append((f'门禁阶段名 {name} 不在 .claude/gate.d/*.sh 里', new_fact, location, excerpt(line, name)))
            for match in CELL_NAME.finditer(line):
                name = match.group(1)
                if name in repository.cell_names or repository.mentions_old_term(name) or name in reported:
                    continue
                reported.add(name)
                close = difflib.get_close_matches(name, sorted(repository.cell_names), n=2)
                new_fact = f'门禁格登记现有 {len(repository.cell_names)} 格' + (f'；近似：{"、".join(close)}' if close else '')
                rows.append((f'门禁格名 {name} 不在 # gate-cell: 登记里', new_fact, location, excerpt(line, name)))
    return rows


def probe_p9(repository, documents):
    rows = []
    for document in documents:
        for number, line in document.lines:
            for pattern in DATED_STATE:
                found = pattern.search(line)
                if found:
                    rows.append(('带日期的现状句（形态命中，kb-discipline 第 8 条说这类是历史快照）', '只报形态，不判',
                                 f'{document.path}:{number}', excerpt(line, found.group(0))))
                    break
    return rows


PROBE_FUNCTIONS = {'P1': probe_p1, 'P3': probe_p3, 'P4': probe_p4, 'P5': probe_p5, 'P7': probe_p7, 'P9': probe_p9}


def collect(root, only=None):
    repository = Repository(root)
    documents = load_documents(repository)
    repository.table_headers = gather_registry_columns(repository, documents)
    repository.invariant_status = gather_invariant_status(repository)
    repository.decision_status = gather_decision_status(repository)
    repository.closed_debts, repository.open_debts, repository.closed_range = gather_debts(repository)
    broken = os.environ.get(BREAK_VARIABLE)
    candidates = []
    for probe in PROBES:
        if only and probe not in only:
            continue
        if broken == probe:
            continue
        found = PROBE_FUNCTIONS[probe](repository, documents)
        found.sort(key=lambda row: (row[2].rsplit(':', 1)[0], int(row[2].rsplit(':', 1)[1]), row[0]))
        for index, (old, new, location, line) in enumerate(found, start=1):
            candidates.append((f'{probe}{index:04d}', old, new, location, clip(line)))
    return candidates, len(documents)


def write_table(candidates, path):
    with open(path, 'w', encoding='utf-8') as handle:
        handle.write(HEADER)
        for group, old, new, location, line in candidates:
            handle.write(f'{group}\t{old.replace(chr(9), " ")}\t{new.replace(chr(9), " ")}\t{location}\t{line}\n')


def summary_line(candidates, scanned):
    counts = {probe: sum(1 for row in candidates if row[0].startswith(probe)) for probe in PROBES}
    parts = '、'.join(f'{probe} {counts[probe]}' for probe in PROBES)
    return f'✓ 候选 {len(candidates)} 行：{parts}；扫了 {scanned} 份文档'


# ---------------------------------------------------------------- 自证

SELFTEST_FILES = {
    '.claude/agents/x.md': '# x\n',
    '.claude/gate.d/stage-one.sh': '#!/bin/bash\n# gate-cell: good-cell 一个登记过的格\n',
    'crates/demo/src/lib.rs': 'pub fn present_name_ok() {}\nconst PRESENT_CONST_OK: u8 = 1;\n',
    'scripts/real.sh': '#!/bin/bash\n',
    'scripts/code-real-file.sh': '#!/bin/bash\n',
    'scripts/tools/README.md': '# 工具\n\n现状（2026-10-01）：子目录的说明也在扫。\n',
    'crates/singlefs-checker/invariants/01-a.md': (
        '# 不变量\n\n| ID | 简称 | 不变量 | checker 状态 |\n|---|---|---|---|\n'
        '| I-1.1 | 甲 | 甲甲 | 已实现（x） |\n| I-1.2 | 乙 | 乙乙 | 未实现 |\n'),
    '.claude/kb/checks-owed.md': (
        '# 欠账\n\n| # | 简称 | 欠什么 |\n|---|---|---|\n| C1 | 开着的 | 欠 x |\n\n'
        '### 已还清\n\n| # | 简称 | 怎么还的 | 还清日期 |\n|---|---|---|---|\n'
        '| C2 | 还清了 | 欠 y 仍然在这一行里写着前置 | 2026-10-01 |\n'),
    '.claude/kb/decisions/01-x.md': (
        '## D1 简称 —— 已定\n\n### 已定项\n\n| # | 分项 | 定案 |\n|---|---|---|\n'
        '| 1 | **甲** | 内容 **状态：已定。** **进度：已实现。** |\n'
        '| 2 | **乙** | 内容 **状态：已定。** **进度：未实现。** |\n\n### 未定项\n\n无。\n'),
    '.claude/kb/term-renames.md': '# 对照\n\n| 旧 | 新 | 匹配 | 是什么 |\n|---|---|---|---|\n| layer0-old | new-name | 整串 | x |\n| `layer0-regex(?![A-Za-z0-9_-])` | new-regex | 正则 | y |\n',
    '.claude/kb/decisions-history.md': '`records/nope-history.md` 不在扫的范围里。\n',
    '.claude/kb/sub/real-sibling.md': '# 邻居\n',
    'records/r.md': '`.claude/nope-in-records.md` records 不扫。\n',
}

# (文本, 该行应报的探针集合)；围栏与历史版本里的行应一个都不报
SELFTEST_LINES = [
    ('# 测试文档', set()),
    ('见 `.claude/missing/file.md:12` 里', {'P1'}),
    ('见 `scripts/real.sh:3`、`crates/demo/src/*.rs`、`.claude/<轮>/x.md`、`sub/real-sibling.md`', set()),
    ('命令 `bash scripts/real.sh --x` 不报', set()),
    ('写法 `HEAD:scripts/real.sh`、`scripts/real.sh@小节`、`scripts/real.sh:3-5`、`.claude/kb/layout/05-` 不报', set()),
    ('版本前缀 `HEAD~1:.claude/missing/old.md` 照报', {'P1'}),
    ('相对命令目录的 `../mutation/nothing.sh` 与 pathspec `:!scripts/real.sh` 不报', set()),
    ('长行' + '填' * 700 + ' 远处 `.claude/far/missing.md` 要留在原文里', {'P1'}),
    ('出现 `missing_function_name` 一次', {'P3'}),
    ('出现 `present_name_ok`、`PRESENT_CONST_OK`、`short_x`、`good-cell`', set()),
    ('环境变量 `ENV_ONLY_VAR` 不报', set()),
    ('列名 `registry_column_x` 不报', set()),
    ('C2 仍然欠着', {'P4'}),
    ('C2 已还清，之前欠；C1 欠着；C2 是简称', set()),
    ('I-1.1 未实现', {'P5'}),
    ('I-1.2 已实现', {'P5'}),
    ('I-1.1 已实现，I-1.2 未实现；I-1.1 和 I-1.2 的编号', set()),
    ('D1 已定项 1 未实现', {'P5'}),
    ('D1 已定项 2 已实现', {'P5'}),
    ('D1 已定项 1 已实现，已定项 2 未实现', set()),
    ('派 `ghost-agent` 去做', {'P7'}),
    ('派 `x` 去做，`x` 交回，派 `general-purpose`', set()),
    ('指派 `not-an-agent` 与分派 `other-thing`', set()),
    ('C2（简称里有等号） 的简称不算；C2b 仍欠', set()),
    ('门禁 doc-gone 阶段', {'P7'}),
    ('门禁 stage-one、code-real-file、layer0-old、layer0-regex、doc-gone', {'P7'}),
    ('`ghost-cell` 格没登记', {'P7'}),
    ('`good-cell` 格与格式', set()),
    ('派 `ghost-agent` 读 `.claude/missing/other.md`', {'P1', 'P7'}),
    ('现状（2026-10-01）：旧', {'P9'}),
    ('2026-10-02 现状：旧', {'P9'}),
    ('截至 2026-10-03 旧', {'P9'}),
    ('2026-10-04 起仍旧', {'P9'}),
    ('2026-10-05 记录；现状很好', set()),
    ('```', set()),
    ('围栏里 `.claude/fence/missing.md` `fence_missing_name` 现状（2026-10-06）', set()),
    ('```', set()),
    ('## 历史版本', set()),
    ('历史里 `.claude/history/missing.md` 现状（2026-10-07）', set()),
]


def build_selftest_repository(directory):
    run_git(directory, 'init', '-q')
    document_lines = []
    for text, _ in SELFTEST_LINES:
        document_lines.append(text)
    files = dict(SELFTEST_FILES)
    files['.claude/kb/doc.md'] = '\n'.join(document_lines) + '\n'
    files['.claude/kb/other-table.md'] = '| registry_column_x | 说明 |\n|---|---|\n| a | b |\n'
    for relative, content in files.items():
        full = os.path.join(directory, relative)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, 'w', encoding='utf-8') as handle:
            handle.write(content)
    run_git(directory, 'add', '-A')


def selftest_expected():
    expected = []
    for index, (_, probes) in enumerate(SELFTEST_LINES, start=1):
        for probe in probes:
            expected.append((probe, f'.claude/kb/doc.md:{index}'))
    expected.append(('P9', 'scripts/tools/README.md:3'))
    return sorted(expected)


def index_of_long_line():
    return next(index for index, (text, _) in enumerate(SELFTEST_LINES, start=1) if text.startswith('长行'))


def selftest_core():
    directory = tempfile.mkdtemp(prefix='round2-probe-selftest-')
    try:
        build_selftest_repository(directory)
        candidates, scanned = collect(directory)
        actual = sorted((row[0][:2], row[3]) for row in candidates)
        expected = selftest_expected()
        wrong = False
        for probe in PROBES:
            wanted = [item for item in expected if item[0] == probe]
            got = [item for item in actual if item[0] == probe]
            if wanted != got:
                wrong = True
                print(f'  ✗ {probe} 报的与该报的不一致')
                print(f'     该报而没报：{[item for item in wanted if item not in got]}')
                print(f'     不该报而报了：{[item for item in got if item not in wanted]}')
        long_rows = [row for row in candidates if row[3] == f'.claude/kb/doc.md:{index_of_long_line()}']
        if len(long_rows) != 1 or '.claude/far/missing.md' not in long_rows[0][4] or len(long_rows[0][4]) > 500:
            print('  ✗ P1 长行的原文列没把命中处留在 500 字以内')
            wrong = True
        if wrong:
            return 1
        print(f'  ✓ {len(candidates)} 行候选恰是该报的，扫了 {scanned} 份文档')
        return 0
    finally:
        shutil.rmtree(directory, ignore_errors=True)


def selftest():
    if os.environ.get(BREAK_VARIABLE):
        print(f'  （{BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 已设：这一次自证应当红）')
    status = selftest_core()
    if status != 0 or os.environ.get(BREAK_VARIABLE):
        return status
    for probe in PROBES:
        environment = dict(os.environ, **{BREAK_VARIABLE: probe})
        result = subprocess.run([sys.executable, os.path.abspath(__file__), '--selftest-core'], env=environment,
                                capture_output=True, text=True)
        reported = sorted(set(re.findall(r'✗ (P\d) 报的', result.stdout)))
        if result.returncode == 0 or reported != [probe]:
            print(f'  ✗ 弄坏 {probe} 之后自证没有恰好在 {probe} 上红：退出 {result.returncode}，红在 {reported}')
            return 1
        print(f'  ✓ 弄坏 {probe}：自证红在 {probe}')
    print('✓ 自证绿：每个探针该报的恰好报出，六个弄坏开关各红在点名的探针')
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('root', nargs='?')
    parser.add_argument('--out')
    parser.add_argument('--only', help='只跑这几条探针，逗号分隔，如 P1,P3')
    parser.add_argument('--selftest', action='store_true')
    parser.add_argument('--selftest-core', action='store_true', help=argparse.SUPPRESS)
    arguments = parser.parse_args()
    if arguments.selftest_core:
        return selftest_core()
    if arguments.selftest:
        return selftest()
    if not arguments.root or not arguments.out:
        parser.error('要给仓根与 --out')
    only = set(arguments.only.split(',')) if arguments.only else None
    candidates, scanned = collect(arguments.root, only)
    write_table(candidates, arguments.out)
    print(summary_line(candidates, scanned))
    return 0


if __name__ == '__main__':
    sys.exit(main())
