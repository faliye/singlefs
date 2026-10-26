#!/usr/bin/env python3
# admission: always 每一次调都是对此刻工作区和 HEAD 做一次提交对象，上一次的结论不替这一次作保
# run-condition: command git python3
"""把「这一批」的改动单独做成一个不进任何分支的提交对象，当 `stale-candidates.py --target` 用。

几个会话共写一个仓时，`stale-candidates.py --changes` 不给 `--target` 会把别的会话没提交的改动当成这一阶段的
（`.claude/batch-scope` 是几个会话共用的，分不出谁的改动）。这个脚本从 HEAD 起一份临时索引，只把调用方点名的内容写进去：
整份取工作区内容的路径，和按块挑的路径；别的路径的工作区改动一个字不进。

用法（路径都相对仓根）：
  batch-target.py [--path 路径]… [--deleted 路径]… [按块挑的组]… [--message 说明] [--scratch 目录]
  按块挑的组：--match 正则 [--match 正则…] [--foreign 正则…] [--split 正则] [--by-row] [--keep-foreign] 文件…
              （一组以文件收尾；文件后面再出现 --match / --foreign / --split / --by-row / --keep-foreign 就是下一组，正则与开关不带到下一组）
  batch-target.py --selftest

  --path 路径        整份取工作区内容（含没进 git 的新文件；路径是目录时取目录下全部）。路径不存在退 2
  --deleted 路径     这一批把它删了：HEAD 里有、工作区里没有；从提交对象里去掉
  --match / --foreign / --split / --by-row / --keep-foreign  含义与块的切法同 `scripts/gate/stage-mine.py`（import 它的 choose() / stage()，不另抄）：
                     拿 HEAD 那一版当底与工作区逐行比，一块里有一行命中 --match 才进，命中 --foreign 的块拒绝；
                     带 --keep-foreign 的组，命中 --foreign 的我的块不拒、照 HEAD 原样留在提交对象里（choose() 的 keep_foreign），我的块全命中 --foreign 的退 2；
                     新文件不归按块挑管（整份是这一批的就用 --path）。一块都没命中退 2
  --message 说明     提交对象的说明，默认 batch-target
  --scratch 目录     临时索引放哪（草稿目录）；默认用 TMPDIR。跑完临时索引连同目录一起删

怎么做的：`GIT_INDEX_FILE` 指临时文件，`git read-tree HEAD` 起底，`git add` / stage-mine 的写入都落在这份索引里，
`git write-tree` 加 `git commit-tree -p HEAD` 得到提交对象。真索引、任何 ref、工作区都不动；
对象库里多出几个没有引用的对象（blob、树、提交），git 的垃圾回收到期会清掉，用完不必删。
标准输出只打这个提交对象的哈希（40 位十六进制）；别的话都写标准错误。

退出码：0 做好了，哈希在标准输出；2 用法不对、点名的路径不存在、按块挑一块都没命中或被拒、git 出错（标准输出为空）。

stdout 要接给 `--target`：
  target=$(python3 scripts/kb/batch-target.py --path records/某记录.md --match '其十|E137' --foreign 'E138' .claude/kb/experiments-history.md) \\
    && python3 scripts/kb/stale-candidates.py --changes --base 基准 --target "$target" --out 变更清单.md

弄坏开关 BATCH_TARGET_BREAK=<项>（只给 --selftest 证红用，打开后 --selftest 退 1；项名、弄坏哪一处、红在哪格）：
  BATCH_TARGET_BREAK=whole-worktree   写树之前把整个工作区都加进临时索引（别的路径的改动也进了提交对象）
      「✗ 自检：别的路径的工作区改动进了提交对象」
      连带红的格：「✗ 自检：真索引里别人暂存的东西进了提交对象」、「✗ 自检：只点名 mine.md，提交对象里却多了」、
      「✗ 自检：按块挑的文件里不是命中块的改动进了提交对象」、「✗ 自检：--keep-foreign 的组应当退 0」
  BATCH_TARGET_BREAK=real-index   不设 GIT_INDEX_FILE，写进仓的真索引
      「✗ 自检：真索引变了」
  BATCH_TARGET_BREAK=match-whole-file   按块挑的文件整份取工作区内容，不按块
      「✗ 自检：按块挑的文件里不是命中块的改动进了提交对象」
      连带红的格：「✗ 自检：新文件按块挑应当退 2」、「✗ 自检：--keep-foreign 的组应当退 0」
  BATCH_TARGET_BREAK=keep-foreign-dropped   按块挑的组不认 --keep-foreign（当没写，命中 --foreign 的块照旧整组拒）
      「✗ 自检：--keep-foreign 的组应当退 0」
"""
import importlib.util
import os
import re
import shutil
import subprocess
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

BREAK_MODES = ('whole-worktree', 'real-index', 'match-whole-file', 'keep-foreign-dropped')  # BATCH_TARGET_BREAK 认的取值，只给 --selftest 用
COMMIT_ENVIRONMENT = {'GIT_AUTHOR_NAME': 'batch-target', 'GIT_AUTHOR_EMAIL': 'batch-target@example.invalid',
                      'GIT_COMMITTER_NAME': 'batch-target', 'GIT_COMMITTER_EMAIL': 'batch-target@example.invalid'}


class Refusal(Exception):
    """用法不对、路径不在、一块都没命中、git 出错：消息第一行说卡在哪，第二行写出路。"""


def load_stage_mine():
    """按块的切法只有 scripts/gate/stage-mine.py 一份：import 它，不抄。"""
    path = os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', 'gate', 'stage-mine.py')
    spec = importlib.util.spec_from_file_location('stage_mine', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run_git(root, arguments, environment=None, stdin=None):
    completed = subprocess.run(['git', '--literal-pathspecs', '-C', root, *arguments], input=stdin, capture_output=True, text=True,
                               env=None if environment is None else {**os.environ, **environment})
    if completed.returncode != 0:
        raise Refusal(f'git {" ".join(arguments)} 失败：{completed.stderr.strip() or completed.stdout.strip()}\n'
                      '确认在 git 仓里、HEAD 存在、路径是相对仓根的；再跑一遍')
    return completed.stdout


def parse_arguments(argv):
    """返回（整份取的路径, 删除的路径, 按块挑的组, 说明, 草稿目录）。每组 = {match, foreign, split, by_row, files}。"""
    paths, deleted, groups, message, scratch = [], [], [], 'batch-target', None
    group = None
    tokens = iter(argv)

    def value(flag):
        try:
            return next(tokens)
        except StopIteration:
            raise Refusal(f'{flag} 后面缺值\n照文件头的用法补上')

    for token in tokens:
        if token == '--path':
            paths.append(value(token))
        elif token == '--deleted':
            deleted.append(value(token))
        elif token == '--message':
            message = value(token)
        elif token == '--scratch':
            scratch = value(token)
        elif token in ('--match', '--foreign', '--split', '--by-row', '--keep-foreign'):
            if group is None or group['files']:
                group = {'match': [], 'foreign': [], 'split': None, 'by_row': False, 'keep_foreign': False, 'files': []}
                groups.append(group)
            if token == '--match':
                group['match'].append(value(token))
            elif token == '--foreign':
                group['foreign'].append(value(token))
            elif token == '--split':
                group['split'] = value(token)
            elif token == '--keep-foreign':
                # keep-foreign-dropped 弄坏开关：不认这个开关
                group['keep_foreign'] = os.environ.get('BATCH_TARGET_BREAK') != 'keep-foreign-dropped'
            else:
                group['by_row'] = True
        elif token.startswith('--'):
            raise Refusal(f'认不得的参数 {token}\n用法在文件头')
        else:
            if group is None:
                raise Refusal(f'文件 {token} 前面没有 --match\n整份取工作区内容用 --path {token}；按块挑先写 --match 正则')
            group['files'].append(token)
    for group in groups:
        if not group['files']:
            raise Refusal('有一组 --match 后面没跟文件\n把要按块挑的文件写在它后面')
        if not group['match']:
            raise Refusal(f'文件 {group["files"][0]} 这一组没有 --match\n按块挑至少一个 --match 正则；整份取用 --path')
    if not paths and not deleted and not groups:
        raise Refusal('没有点名任何路径\n--path 整份取、--match 正则 文件 按块挑、--deleted 去掉被删的')
    return paths, deleted, groups, message, scratch


def clean_relative(root, path):
    """相对仓根的规范路径；越出仓根的拒。"""
    normal = os.path.normpath(path)
    if os.path.isabs(normal) or normal == '..' or normal.startswith('..' + os.sep) or normal == '.':
        raise Refusal(f'路径 {path} 不是仓内的相对路径\n路径写成相对仓根的，例：.claude/kb/experiments-history.md')
    return normal


def build(root, argv):
    paths, deleted, groups, message, scratch = parse_arguments(argv)
    paths = [clean_relative(root, p) for p in paths]
    deleted = [clean_relative(root, p) for p in deleted]
    picked_files = [clean_relative(root, f) for group in groups for f in group['files']]
    for name in set(paths) & set(picked_files) | (set(paths) & set(deleted)) | (set(picked_files) & set(deleted)):
        raise Refusal(f'{name} 同时用了两种取法\n一个路径只能整份取、按块挑、去掉三选一')
    for path in paths + picked_files:
        if not os.path.lexists(os.path.join(root, path)):
            raise Refusal(f'点名的路径不存在：{path}\n路径写错了就改；这一批把它删了就用 --deleted {path}')
    head = run_git(root, ['rev-parse', '--verify', 'HEAD^{commit}']).strip()
    for path in deleted:
        if os.path.lexists(os.path.join(root, path)):
            raise Refusal(f'--deleted {path}：工作区里还有它\n没删就别写 --deleted；整份取用 --path')
        if not run_git(root, ['ls-tree', '--name-only', head, '--', path]).strip():
            raise Refusal(f'--deleted {path}：HEAD 里没有它\n路径写错了就改；HEAD 里本来就没有的不用写')
    scratch_directory = tempfile.mkdtemp(prefix='batch-target-', dir=scratch)
    try:
        index_file = os.path.join(scratch_directory, 'index')
        # real-index 弄坏开关：不设 GIT_INDEX_FILE，写进真索引
        environment = {} if os.environ.get('BATCH_TARGET_BREAK') == 'real-index' else {'GIT_INDEX_FILE': index_file}
        run_git(root, ['read-tree', head], environment)
        for path in paths:
            run_git(root, ['add', '-A', '--', path], environment)
        for path in deleted:
            run_git(root, ['rm', '-q', '--cached', '-r', '--', path], environment)
        if groups:
            stage_mine = load_stage_mine()
            saved = {name: os.environ.get(name) for name in environment}
            os.environ.update(environment)   # stage-mine 的 git() 读继承来的环境变量，临时索引经它落到 choose() 与 stage()
            try:
                for group in groups:
                    split = re.compile(group['split'] or stage_mine.DEFAULT_SPLIT)
                    match = [re.compile(p) for p in group['match']]
                    foreign = [re.compile(p) for p in group['foreign']]
                    for path in (clean_relative(root, f) for f in group['files']):
                        if os.environ.get('BATCH_TARGET_BREAK') == 'match-whole-file':
                            run_git(root, ['add', '-A', '--', path], environment)
                            continue
                        try:
                            with open(os.devnull, 'w') as silent:
                                saved_out, sys.stdout = sys.stdout, silent
                                try:
                                    content, _staged, _left = stage_mine.choose(path, match, foreign, split, root, group['by_row'], group['keep_foreign'])
                                finally:
                                    sys.stdout = saved_out
                        except SystemExit as refusal:
                            reasons = {2: '它不在 HEAD 里（新文件）：整份是这一批的就用 --path',
                                       3: '一块都没命中 --match、工作区与 HEAD 相同，或带 --keep-foreign 时命中的块全都命中 --foreign：查正则、查这个文件这一批改没改',
                                       4: '进来的块里有命中 --foreign 的行（两个会话的改动挤在同一块）：加 --by-row、--keep-foreign（那一块照 HEAD 原样留下）、换 --split 或等对方先提交'}
                            raise Refusal(f'{path} 按块挑没做成（stage-mine 退 {refusal.code}）\n{reasons.get(refusal.code, "照 scripts/gate/stage-mine.py 文件头")}')
                        with open(os.devnull, 'w') as silent:
                            saved_out, sys.stdout = sys.stdout, silent
                            try:
                                code = stage_mine.stage(path, content, root, foreign)
                            finally:
                                sys.stdout = saved_out
                        if code != 0:
                            raise Refusal(f'{path} 要写进临时索引的内容里有命中 --foreign 的新增行\n收窄 --foreign，或等对方先提交')
            finally:
                for name, old in saved.items():
                    if old is None:
                        os.environ.pop(name, None)
                    else:
                        os.environ[name] = old
        if os.environ.get('BATCH_TARGET_BREAK') == 'whole-worktree':
            run_git(root, ['add', '-A', '--', '.'], environment)
        tree = run_git(root, ['write-tree'], environment).strip()
        commit = run_git(root, ['commit-tree', tree, '-p', head, '-m', message], {**environment, **COMMIT_ENVIRONMENT}).strip()
        return commit
    finally:
        shutil.rmtree(scratch_directory, ignore_errors=True)


# ---------------------------------------------------------------- 自证

def selftest():
    broken = os.environ.get('BATCH_TARGET_BREAK', '')
    if broken and broken not in BREAK_MODES:
        print(f'✗ 自检：BATCH_TARGET_BREAK={broken} 认不得')
        print(f'  → 取值只有 {" ".join(BREAK_MODES)}（文件头「弄坏开关」一段）；平常跑 --selftest 不设它')
        return 2
    problems = scenario(broken)
    if broken:
        # 设了开关：自证必须红
        if not problems:
            print(f'✗ 自检：弄坏开关 {broken} 打开了，自证仍然全绿 —— 自证看不出这一处')
            print('  → 看 scenario() 里对应的那一格')
            return 1
        for problem in problems:
            print('✗ 自检：' + problem)
        print(f'  → 弄坏开关 {broken} 是故意的；不设 BATCH_TARGET_BREAK 再跑 --selftest')
        return 1
    caught = []
    for problem in problems:
        print('✗ 自检：' + problem)
    if problems:
        print('  → 改 batch-target.py 对应的分支，再跑 --selftest')
        return 1
    for mode in BREAK_MODES:
        if not scenario(mode):
            print(f'✗ 自检：弄坏开关 {mode}：故意弄坏一处，自证没红')
            print('  → 看 scenario() 里对应的那一格')
            return 1
        caught.append(mode)
    print(f'✓ 自检通过：别的路径的工作区改动不进提交对象、点名的整份取、按块挑只进命中块、--keep-foreign 的组把挤着别人改动的块照 HEAD 留下（没带它拒）、被删的去掉、点名的路径不存在与一块都没命中都退 2 且标准输出为空、'
          f'真索引与 ref 没变；弄坏开关 {len(caught)} 个各开一次、都被抓（{" ".join(caught)}）')
    return 0


def scenario(mode):
    """在临时 git 仓里按文件头的用法跑一遍，返回不合预期的句子（空表示全对）。mode 非空时给子进程设 BATCH_TARGET_BREAK。"""
    problems = []
    with tempfile.TemporaryDirectory() as repo:
        def git(*arguments, check=True):
            return subprocess.run(['git', '-C', repo, *arguments], capture_output=True, text=True, check=check).stdout

        def write(name, text):
            full = os.path.join(repo, name)
            os.makedirs(os.path.dirname(full), exist_ok=True)
            with open(full, 'w', encoding='utf-8') as handle:
                handle.write(text)

        def invoke(*arguments):
            environment = {name: value for name, value in os.environ.items() if name != 'BATCH_TARGET_BREAK'}
            if mode:
                environment['BATCH_TARGET_BREAK'] = mode
            return subprocess.run([sys.executable, os.path.realpath(__file__), *arguments], cwd=repo, capture_output=True, text=True, env=environment)

        def committed(commit, name):
            shown = subprocess.run(['git', '-C', repo, 'show', f'{commit}:{name}'], capture_output=True, text=True)
            return shown.stdout if shown.returncode == 0 else None

        git('init', '-q')
        git('config', 'user.email', 'selftest@example.invalid')
        git('config', 'user.name', 'selftest')
        history_base = '# 变更史\n\n### 2026-09-11（其八）：旧的\n\n- 旧\n'
        shared_base = '# 记录\n\n## 一节\n\n旧名 出现在这一行\n\n## 二节\n\n别的\n\n## 末节\n\n末尾\n'
        shared_expected = shared_base.replace('## 末节\n', '## 新名 三节\n\n新名 的正文\n\n## 末节\n', 1)
        files = {'other.md': '别的会话的文件\n', 'mine.md': '这一批的文件\n', 'staged.md': '别人暂存的文件\n', 'removed.md': '这一批删掉的\n',
                 'history.md': history_base, 'shared.md': shared_base}
        for name, text in files.items():
            write(name, text)
        git('add', '-A')
        git('commit', '-q', '-m', 'base')
        head = git('rev-parse', 'HEAD').strip()
        write('other.md', '别的会话的文件\n别的会话改的一行\n')
        write('mine.md', '这一批的文件\n这一批改的一行\n')
        write('new-mine.md', '这一批新写的文件，还没进 git\n')
        write('new-other.md', '别的会话新写的文件，也没进 git\n')
        os.remove(os.path.join(repo, 'removed.md'))
        mine_block = '### 2026-09-11（其十）：我的\n\n- 我的一行\n\n'
        their_block = '### 2026-09-11（其九）：别人的\n\n- 别人的一行\n\n'
        write('history.md', history_base.replace('### 2026-09-11（其八）', mine_block + their_block + '### 2026-09-11（其八）', 1))
        # 同一行挤着我的改名与别人的改动（「系统配置」），另一块「新名 三节」整块是我的
        write('shared.md', shared_expected.replace('旧名 出现在这一行\n', '新名 出现在这一行，系统配置\n', 1))
        write('staged.md', '别人暂存的文件\n别人暂存的一行\n')
        git('add', 'staged.md')    # 真索引里放一样别人暂存的东西：提交对象里不许有它，做完它还得原样在
        index_path = os.path.join(repo, '.git', 'index')

        def snapshot():
            with open(index_path, 'rb') as handle:
                index_bytes = handle.read()
            return (index_bytes, git('for-each-ref'), git('rev-parse', 'HEAD'), git('stash', 'list'))

        before = snapshot()
        result = invoke('--path', 'mine.md', '--path', 'new-mine.md', '--deleted', 'removed.md',
                        '--match', '其十', '--foreign', '其九', 'history.md')
        after = snapshot()
        commit = result.stdout.strip()
        if result.returncode != 0 or not re.fullmatch(r'[0-9a-f]{40}', commit) or result.stdout != commit + '\n':
            problems.append(f'点名的路径与按块挑的组应当退 0、标准输出只有一个 40 位哈希：退 {result.returncode}，stdout={result.stdout!r}，stderr={result.stderr[-300:]!r}')
            return problems
        if git('cat-file', '-t', commit).strip() != 'commit' or git('rev-parse', commit + '^').strip() != head:
            problems.append('做出来的不是以 HEAD 为父的提交对象')
        if committed(commit, 'mine.md') != '这一批的文件\n这一批改的一行\n' or committed(commit, 'new-mine.md') != '这一批新写的文件，还没进 git\n':
            problems.append('点名的路径（含没进 git 的新文件）没有整份取工作区内容')
        if committed(commit, 'other.md') != files['other.md'] or committed(commit, 'new-other.md') is not None:
            problems.append('别的路径的工作区改动进了提交对象')
        if committed(commit, 'staged.md') != files['staged.md']:
            problems.append('真索引里别人暂存的东西进了提交对象')
        if committed(commit, 'removed.md') is not None:
            problems.append('--deleted 点名的路径还留在提交对象里')
        if committed(commit, 'history.md') != history_base.replace('### 2026-09-11（其八）', mine_block + '### 2026-09-11（其八）', 1):
            problems.append('按块挑的文件里不是命中块的改动进了提交对象（要的是 HEAD + 命中的那一块）')
        if before[0] != after[0]:
            problems.append('真索引变了（临时索引应当只在草稿里）')
        if before[1:] != after[1:]:
            problems.append('ref 或 HEAD 变了（提交对象不许进任何分支）')
        branches = git('branch', '--contains', commit).strip() if git('cat-file', '-t', commit).strip() == 'commit' else ''
        if branches:
            problems.append(f'提交对象被分支包含了：{branches}')
        for name, text in (('other.md', '别的会话的文件\n别的会话改的一行\n'), ('history.md', None)):
            if text is not None:
                with open(os.path.join(repo, name), encoding='utf-8') as handle:
                    if handle.read() != text:
                        problems.append(f'工作区的 {name} 被改动了')
        # 只用 --path：别的路径的改动同样不进
        only_path = invoke('--path', 'mine.md').stdout.strip()
        if re.fullmatch(r'[0-9a-f]{40}', only_path):
            if git('diff', '--name-only', head, only_path).split() != ['mine.md']:
                problems.append(f'只点名 mine.md，提交对象里却多了：{git("diff", "--name-only", head, only_path).split()}')
        else:
            problems.append('只点名一个路径应当退 0 并打哈希')
        # --keep-foreign：同一行挤着别人改动的那一块照 HEAD 原样留在提交对象里，我的另一块照进；同一组没带它整组拒
        kept = invoke('--match', '新名', '--foreign', '系统配置', '--keep-foreign', 'shared.md')
        kept_commit = kept.stdout.strip()
        if kept.returncode != 0 or not re.fullmatch(r'[0-9a-f]{40}', kept_commit) or committed(kept_commit, 'shared.md') != shared_expected:
            problems.append(f'--keep-foreign 的组应当退 0、提交对象里 shared.md = HEAD + 「新名 三节」那一块（同一行挤着别人改动的那一块照 HEAD 原样）：'
                            f'退 {kept.returncode}，stdout={kept.stdout!r}，stderr={kept.stderr[-300:]!r}')
        without_keep = invoke('--match', '新名', '--foreign', '系统配置', 'shared.md')
        if mode != 'match-whole-file' and (without_keep.returncode != 2 or without_keep.stdout != ''):
            problems.append(f'同一行挤着别人改动、没带 --keep-foreign 的组应当退 2、标准输出为空：退 {without_keep.returncode}，stdout={without_keep.stdout!r}')
        # 红的输入：路径不存在、一块都没命中、新文件按块挑、同一路径两种取法
        for what, arguments in (('点名的路径不存在', ['--path', 'no-such-file.md']),
                                ('--match 一块都没命中', ['--match', '不存在的标记', 'history.md']),
                                ('新文件按块挑', ['--match', '新写', 'new-mine.md']),
                                ('--deleted 点名的路径工作区里还有', ['--deleted', 'mine.md']),
                                ('同一路径两种取法', ['--path', 'history.md', '--match', '其十', 'history.md']),
                                ('没点名任何路径', [])):
            refused = invoke(*arguments)
            if mode == 'match-whole-file' and what == '--match 一块都没命中':
                continue   # 这个开关把按块挑换成整份取，一块都没命中的拒绝本来就不在它的路上
            if refused.returncode != 2 or refused.stdout != '' or '\n' not in refused.stderr.strip():
                problems.append(f'{what}应当退 2、标准输出为空、标准错误写出路：退 {refused.returncode}，stdout={refused.stdout!r}，stderr={refused.stderr[-200:]!r}')
    return problems


def main():
    if sys.argv[1:2] == ['--selftest']:
        return selftest()
    try:
        root = run_git(os.getcwd(), ['rev-parse', '--show-toplevel']).strip()
        commit = build(root, sys.argv[1:])
    except Refusal as refusal:
        lines = str(refusal).split('\n')
        print(f'  ✗ {lines[0]}', file=sys.stderr)
        print(f'     → 怎么办：{lines[1] if len(lines) > 1 else "照文件头的用法"}', file=sys.stderr)
        return 2
    print(commit)
    return 0


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main())
