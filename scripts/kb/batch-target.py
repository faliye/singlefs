#!/usr/bin/env python3
# admission: always 每一次调都是对此刻工作区和 HEAD 做一次提交对象，上一次的结论不替这一次作保
# run-condition: command git python3
"""把「这一批」的改动单独做成一个不进任何分支的提交对象，当 `stale-candidates.py --target` 用。

几个会话共写一个仓时，`stale-candidates.py --changes` 不给 `--target` 会把别的会话没提交的改动当成这一阶段的
（`.claude/batch-scope` 是几个会话共用的，分不出谁的改动）。这个脚本从 HEAD 起一份临时索引，只把调用方点名的内容写进去：
整份取工作区内容的路径，和按块挑的路径；别的路径的工作区改动一个字不进。

用法（路径都相对仓根）：
  batch-target.py [--path 路径]… [--deleted 路径]… [--patch 补丁目录]… [按块挑的组]… [--message 说明] [--scratch 目录]
  按块挑的组：--match 正则 [--match 正则…] [--foreign 正则…] [--split 正则] [--by-row] [--keep-foreign] 文件…
              （一组以文件收尾；文件后面再出现 --match / --foreign / --split / --by-row / --keep-foreign 就是下一组，正则与开关不带到下一组）
  batch-target.py --selftest

  --path 路径        整份取工作区内容（含没进 git 的新文件；路径是目录时取目录下全部）。路径不存在退 2
  --deleted 路径     这一批把它删了：HEAD 里有、工作区里没有；从提交对象里去掉
  --patch 补丁目录   这一批是一串补丁，已经打在工作区上（工作区里另有别的会话没提交的改动，补丁对 HEAD 打不上）：目录里的 *.patch 按文件名排序，给一份 .patch 文件也行；
                     可给几次，按打的次序给。做法：在临时索引里按相反的次序逐份 `git apply --cached -R` 倒掉补丁，得到「别人改过的那一版」（工作区去掉这一批），
                     再对补丁碰到的每个文件三方合成（`git merge-file`：当前 = HEAD 那一版，底 = 别人改过的那一版，另一方 = 工作区），
                     结果 = HEAD 加这一批，别人的改动不进。下面三样有一样就退 2、逐个列出、一个字不写：
                       · 倒不掉的块（补丁没完整打在工作区上，或工作区又被改过，git apply 报的每一行都列出）；
                       · 合成出冲突标记（这一批与别的会话改在同一处，或这一批改的是别人新建的文件、这一批删的文件别人改过）；
                       · 补丁删了的文件工作区里还在（结束提交里要删，工作区却还有）。
                     补丁碰到的路径不许再用 --path / --deleted / 按块挑点名
  --match / --foreign / --split / --by-row / --keep-foreign  含义与块的切法同 `scripts/gate/stage-mine.py`（import 它的 choose() / stage()，不另抄）：
                     拿 HEAD 那一版当底与工作区逐行比，一块里有一行命中 --match 才进，命中 --foreign 的块拒绝；
                     带 --keep-foreign 的组，命中 --foreign 的我的块不拒、照 HEAD 原样留在提交对象里（choose() 的 keep_foreign），我的块全命中 --foreign 的退 2；
                     不在 HEAD 里的文件，底按次序取：真索引里这一路径的那一版（第 0 档，别人或自己暂存的新文件）；没有就取搬迁登记表（.claude/gate.d/path-moves.tsv）
                     倒推出来的搬迁前旧路径在 HEAD 里的那一版；两样都没有（没进 git 的新文件）退 2，整份是这一批的就用 --path。底取自哪里写在标准错误。一块都没命中退 2
  --message 说明     提交对象的说明，默认 batch-target
  --scratch 目录     临时索引放哪（草稿目录）；默认用 TMPDIR。跑完临时索引连同目录一起删

怎么做的：`GIT_INDEX_FILE` 指临时文件，`git read-tree HEAD` 起底，`git add` / stage-mine 的写入都落在这份索引里，
`git write-tree` 加 `git commit-tree -p HEAD` 得到提交对象。真索引、任何 ref、工作区都不动；
对象库里多出几个没有引用的对象（blob、树、提交），git 的垃圾回收到期会清掉，用完不必删。
标准输出只打这个提交对象的哈希（40 位十六进制）；别的话都写标准错误。

退出码：0 做好了，哈希在标准输出；2 用法不对、点名的路径不存在、按块挑一块都没命中或被拒、--patch 做不出（倒不掉、冲突、删了的文件工作区里还在）、git 出错（标准输出为空）。

stdout 要接给 `--target`：
  target=$(python3 scripts/kb/batch-target.py --path records/某记录.md --match '其十|E137' --foreign 'E138' .claude/kb/experiments-history.md) \\
    && python3 scripts/kb/stale-candidates.py --changes --base 基准 --target "$target" --out 变更清单.md

弄坏开关 BATCH_TARGET_BREAK=<项>（只给 --selftest 证红用，打开后 --selftest 退 1；项名、弄坏哪一处、红在哪格）：
  BATCH_TARGET_BREAK=whole-worktree   写树之前把整个工作区都加进临时索引（别的路径的改动也进了提交对象）
      「✗ 自检：别的路径的工作区改动进了提交对象」
      连带红的格：「✗ 自检：真索引里别人暂存的东西进了提交对象」、「✗ 自检：只点名 mine.md，提交对象里却多了」、
      「✗ 自检：按块挑的文件里不是命中块的改动进了提交对象」、「✗ 自检：--keep-foreign 的组应当退 0」、
      「✗ 自检：只在暂存区的新文件按块挑，提交对象里该是暂存区那一版加命中的那一块」、「✗ 自检：搬迁后的路径按块挑，提交对象里该是旧路径在 HEAD 里的那一版加命中的那一块」、
      「✗ 自检：--patch 合成出的 app.txt 该是 HEAD 加这一批的两份补丁」、「✗ 自检：--patch 别人新建的文件进了提交对象，或补丁没碰的文件变了」
  BATCH_TARGET_BREAK=real-index   不设 GIT_INDEX_FILE，写进仓的真索引
      「✗ 自检：真索引变了」
      连带红的格：「✗ 自检：只在暂存区的新文件按块挑应当退 0」、「✗ 自检：不在 HEAD 里的文件按块挑做完，真索引变了」
  BATCH_TARGET_BREAK=match-whole-file   按块挑的文件整份取工作区内容，不按块
      「✗ 自检：按块挑的文件里不是命中块的改动进了提交对象」
      连带红的格：「✗ 自检：新文件按块挑应当退 2」、「✗ 自检：--keep-foreign 的组应当退 0」、
      「✗ 自检：只在暂存区的新文件按块挑，提交对象里该是暂存区那一版加命中的那一块」、「✗ 自检：搬迁后的路径按块挑，提交对象里该是旧路径在 HEAD 里的那一版加命中的那一块」
  BATCH_TARGET_BREAK=keep-foreign-dropped   按块挑的组不认 --keep-foreign（当没写，命中 --foreign 的块照旧整组拒）
      「✗ 自检：--keep-foreign 的组应当退 0」
  BATCH_TARGET_BREAK=head-only-base   按块挑只认 HEAD 里有的路径（不取真索引与搬迁前旧路径当底，改前的样子）
      「✗ 自检：只在暂存区的新文件按块挑应当退 0」、「✗ 自检：搬迁后的路径按块挑应当退 0」
  BATCH_TARGET_BREAK=merge-markers-accepted   --patch 合成出冲突标记也照收，不拒
      「✗ 自检：--patch 这一批与别人改在同一处应当退 2」
  BATCH_TARGET_BREAK=deleted-file-unchecked   --patch 不核「补丁删了的文件工作区里还在」
      「✗ 自检：--patch 补丁删了的文件工作区里还在应当退 2」
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

BREAK_MODES = ('whole-worktree', 'real-index', 'match-whole-file', 'keep-foreign-dropped', 'head-only-base', 'merge-markers-accepted',
               'deleted-file-unchecked')  # BATCH_TARGET_BREAK 认的取值，只给 --selftest 用
COMMIT_ENVIRONMENT = {'GIT_AUTHOR_NAME': 'batch-target', 'GIT_AUTHOR_EMAIL': 'batch-target@example.invalid',
                      'GIT_COMMITTER_NAME': 'batch-target', 'GIT_COMMITTER_EMAIL': 'batch-target@example.invalid'}
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
LAG_BASELINE_LIBRARY = os.path.join(REPOSITORY_ROOT, '.claude', 'gate.d', 'lib-lag-baseline.py')   # 搬迁登记表的读法只有这一份
MERGE_LABELS = ('HEAD', '别人改过的版本', '工作区')   # git merge-file 的三个标签，冲突标记里就是它们


class Refusal(Exception):
    """用法不对、路径不在、一块都没命中、git 出错：消息第一行说卡在哪，第二行写出路，之后每行是一处细节（--patch 的逐处问题）。"""


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


def git_bytes(root, arguments, environment=None, stdin=None):
    """run_git 的字节版（读 blob、写 blob 用）：返回（退出码, 标准输出字节, 标准错误文本）；不抛，由调用方判。"""
    completed = subprocess.run(['git', '--literal-pathspecs', '-C', root, *arguments], input=stdin, capture_output=True,
                               env=None if environment is None else {**os.environ, **environment})
    return completed.returncode, completed.stdout, completed.stderr.decode('utf-8', 'replace').strip()


def lag_baseline_library():
    """搬迁登记表的读法（.claude/gate.d/lib-lag-baseline.py）；库不在交 None。"""
    if not os.path.isfile(LAG_BASELINE_LIBRARY):
        return None
    specification = importlib.util.spec_from_file_location('lib_lag_baseline', LAG_BASELINE_LIBRARY)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def moved_from_paths(root, path):
    """搬迁登记表（被判的仓里那一份）倒推：这一路径搬来之前叫什么（最近一次搬迁之前的在前）；表里没有相关的行就是空表。"""
    library = lag_baseline_library()
    if library is None:
        return []
    moves, _problems = library.read_path_moves(root)
    return library.paths_before_moves(path, moves)[1:]


def missing_base_entry(root, head, path):
    """按块挑的文件不在 HEAD 里时的底：先取真索引里这一路径的第 0 档，再取搬迁前旧路径在 HEAD 里的那一版。
    返回（模式, 哈希, 底取自哪里的说明）；两样都没有交 None。"""
    for record in run_git(root, ['ls-files', '-s', '-z', '--', path]).split('\0'):
        meta, _tab, _name = record.partition('\t')
        fields = meta.split(' ')
        if len(fields) == 3 and fields[2] == '0':
            return fields[0], fields[1], '真索引'
    for old in moved_from_paths(root, path):
        for record in run_git(root, ['ls-tree', '-z', head, '--', old]).split('\0'):
            meta, _tab, _name = record.partition('\t')
            fields = meta.split(' ')
            if len(fields) == 3 and fields[1] == 'blob':
                return fields[0], fields[2], f'登记的搬迁前路径 {old} 在 HEAD 里'
    return None


def seed_missing_bases(root, head, files, environment):
    """按块挑的文件里 HEAD 没有的，把真索引或搬迁前旧路径那一版放进临时索引当底（choose() 在临时索引里取底）。
    返回每个放了底的文件的说明；两样都没有的不放（choose() 会退 2，说明在那里）。必须在把临时索引设进进程环境变量之前调，真索引才读得到。"""
    notes = []
    if os.environ.get('BATCH_TARGET_BREAK') == 'head-only-base':   # head-only-base 弄坏开关：退回只认 HEAD
        return notes
    for path in files:
        if run_git(root, ['ls-tree', '--name-only', head, '--', path]).strip():
            continue
        found = missing_base_entry(root, head, path)
        if found is None:
            continue
        mode, sha, source = found
        run_git(root, ['update-index', '-z', '--index-info'], environment, f'{mode} {sha}\t{path}\0')
        notes.append(f'{path} 不在 HEAD 里，按块挑以{source}那一版当底')
    return notes


def unquote_c(text):
    """git 把特殊字符的路径写成带双引号的 C 式转义（\\346\\226… 八进制字节、\\n、\\t、\\"、\\\\）：还原成路径。"""
    body, raw, index = text[1:-1], bytearray(), 0
    escapes = {'n': 10, 't': 9, '"': 34, '\\': 92, 'a': 7, 'b': 8, 'f': 12, 'r': 13, 'v': 11}
    while index < len(body):
        if body[index] != '\\':
            raw += body[index].encode('utf-8')
            index += 1
        elif body[index + 1] in '01234567':
            raw.append(int(body[index + 1:index + 4], 8))
            index += 4
        else:
            raw.append(escapes.get(body[index + 1], ord(body[index + 1])))
            index += 2
    return raw.decode('utf-8', 'replace')


def patch_paths(root, patch_file):
    """一份补丁碰到的路径与它删掉的路径：（全部路径[], 删掉的路径[]）。全部路径取 `git apply --numstat -z`（改名的两头都算），
    删掉的取 `git apply --summary` 的 delete mode 行。"""
    counted = subprocess.run(['git', '-C', root, 'apply', '--numstat', '-z', patch_file], capture_output=True)
    if counted.returncode != 0:
        raise Refusal(f'补丁 {patch_file} 读不了：{counted.stderr.decode("utf-8", "replace").strip()}\n确认它是 git diff 出的补丁、整份读得完')
    items = counted.stdout.decode('utf-8').split('\0')
    touched, index = [], 0
    while index < len(items):
        if not items[index]:
            index += 1
            continue
        fields = items[index].split('\t', 2)
        if len(fields) == 3 and fields[2] == '':   # 改名：后面跟着旧路径、新路径两条
            touched += items[index + 1:index + 3]
            index += 3
        else:
            touched.append(fields[2])
            index += 1
    summary = subprocess.run(['git', '-C', root, '-c', 'core.quotepath=false', 'apply', '--summary', patch_file], capture_output=True, text=True)
    deleted = []
    for line in summary.stdout.splitlines():
        found = re.match(r' delete mode [0-9]+ (.*)$', line)
        if found:
            deleted.append(unquote_c(found.group(1)) if found.group(1).startswith('"') else found.group(1))
    return touched, deleted


def read_patch_plan(root, patch_arguments):
    """--patch 点名的目录（或 .patch 文件）→ [(补丁文件, 碰到的路径, 删掉的路径)]：按给的次序，同一目录里按文件名排序。"""
    plan = []
    for argument in patch_arguments:
        full = os.path.abspath(argument)
        if os.path.isdir(full):
            files = [os.path.join(full, name) for name in sorted(os.listdir(full)) if name.endswith('.patch')]
        elif os.path.isfile(full):
            files = [full]
        else:
            raise Refusal(f'--patch 点名的 {argument} 不存在\n给补丁目录（里面放 *.patch 文件），或一份 .patch 文件')
        if not files:
            raise Refusal(f'补丁目录 {argument} 里没有 *.patch 文件\n补丁文件要以 .patch 结尾')
        for patch_file in files:
            plan.append((patch_file, *patch_paths(root, patch_file)))
    return plan


def patch_label(patch_file):
    return os.path.join(os.path.basename(os.path.dirname(patch_file)), os.path.basename(patch_file))


def synthesize_patches(root, head, plan, scratch_directory):
    """按补丁算「HEAD 加这一批」：返回（{路径: (模式, 哈希) 或 None（去掉）}, 问题表）。问题表非空时条目不能用。
    1 补丁删了的文件工作区里还在 → 问题；2 临时索引放工作区那一版（删了的放没有），按相反次序逐份 git apply --cached -R 倒掉补丁，得到别人改过的那一版，倒不掉 → 问题；
    3 每个文件三方合成（当前 HEAD、底 别人改过的那一版、另一方 工作区），有冲突 → 问题。"""
    problems, entries = [], {}
    touched, deleted_after = [], set()
    for _patch_file, paths, deleted in plan:
        touched += [path for path in paths if path not in touched]
        deleted_after = (deleted_after | set(deleted)) - (set(paths) - set(deleted))   # 之后又被别的补丁碰到的，不算删了
    zero = '0' * len(head)
    for path in sorted(deleted_after):
        # deleted-file-unchecked 弄坏开关：不核补丁删了的文件工作区里还在
        if os.path.lexists(os.path.join(root, path)) and os.environ.get('BATCH_TARGET_BREAK') != 'deleted-file-unchecked':
            problems.append(f'{path}：补丁删了它，结束提交里要删，工作区里却还在（补丁没打全，或打完又被写回来了）')
    environment = {'GIT_INDEX_FILE': os.path.join(scratch_directory, 'patch-index')}
    run_git(root, ['read-tree', head], environment)
    setup = []
    for path in touched:
        full = os.path.join(root, path)
        if path in deleted_after or not os.path.isfile(full):
            setup.append(f'0 {zero}\t{path}\0')
        else:
            sha = run_git(root, ['hash-object', '-w', '--', path]).strip()
            setup.append(f'{"100755" if os.access(full, os.X_OK) else "100644"} {sha}\t{path}\0')
    if setup:
        run_git(root, ['update-index', '-z', '--index-info'], environment, ''.join(setup))
    for patch_file, _paths, _deleted in reversed(plan):
        code, _out, error = git_bytes(root, ['apply', '--cached', '-R', '--whitespace=nowarn', patch_file], environment)
        if code != 0:
            problems += [f'{patch_label(patch_file)} 倒不掉（补丁没完整打在工作区上，或打完工作区又被改过）：{line.strip()}' for line in error.splitlines() if line.strip()]
            return entries, problems
    merge_directory = os.path.join(scratch_directory, 'merge')
    os.makedirs(merge_directory)
    for number, path in enumerate(touched):
        full = os.path.join(root, path)
        work = open(full, 'rb').read() if os.path.isfile(full) else None
        code, head_blob, _error = git_bytes(root, ['cat-file', 'blob', f'{head}:{path}'])
        head_blob = head_blob if code == 0 else None
        base_blob, mode = None, "100755" if os.access(full, os.X_OK) else "100644"
        for record in run_git(root, ['ls-files', '-s', '-z', '--', path], environment).split('\0'):
            meta, _tab, _name = record.partition('\t')
            fields = meta.split(' ')
            if len(fields) == 3 and fields[2] == '0':
                mode = fields[0]
                base_blob = git_bytes(root, ['cat-file', 'blob', fields[1]])[1]
        if work is None:   # 这一批把它删了（改名的旧名也在这里）
            if head_blob is None:
                continue
            if head_blob == base_blob:
                entries[path] = None
            else:
                problems.append(f'{path}：这一批删了它，可是 HEAD 那一版与别人改过的那一版不同（别的会话改过这个文件），做不出只含这一批的结果')
            continue
        if base_blob == work:   # 补丁倒掉之后与工作区一样：这个文件上这一批没有净改动
            continue
        if base_blob is None:   # 这一批新建的
            if head_blob is not None and head_blob != work:
                problems.append(f'{path}：这一批新建了它，可是 HEAD 里已经有一份不同的')
                continue
            result = work
        elif head_blob is None:
            problems.append(f'{path}：HEAD 里没有它（别的会话新建的文件），这一批又改了它，做不出只含这一批的版本')
            continue
        elif b'\0' in head_blob + base_blob + work:
            problems.append(f'{path}：是二进制文件，三方合成不了')
            continue
        else:
            sides = []
            for name, data in (('current', head_blob), ('base', base_blob), ('other', work)):
                side = os.path.join(merge_directory, f'{number}-{name}')
                with open(side, 'wb') as handle:
                    handle.write(data)
                sides.append(side)
            merged = subprocess.run(['git', 'merge-file', '-p', '-L', MERGE_LABELS[0], '-L', MERGE_LABELS[1], '-L', MERGE_LABELS[2], *sides],
                                    cwd=root, capture_output=True)
            marked = re.search(rb'^<<<<<<< ' + re.escape(MERGE_LABELS[0].encode('utf-8')) + rb'$', merged.stdout, re.M) is not None
            if merged.returncode < 0 or merged.returncode > 127:
                problems.append(f'{path}：git merge-file 出错：{merged.stderr.decode("utf-8", "replace").strip()}')
                continue
            # merge-markers-accepted 弄坏开关：有冲突标记也照收
            if (merged.returncode > 0 or marked) and os.environ.get('BATCH_TARGET_BREAK') != 'merge-markers-accepted':
                problems.append(f'{path}：这一批与别人的改动落在同一处，三方合成出 {max(merged.returncode, 1)} 处冲突')
                continue
            result = merged.stdout
        sha = git_bytes(root, ['hash-object', '-w', '--stdin'], stdin=result)[1].decode('ascii').strip()
        entries[path] = (mode, sha)
    return entries, problems


def parse_arguments(argv):
    """返回（整份取的路径, 删除的路径, 按块挑的组, 说明, 草稿目录, 补丁目录）。每组 = {match, foreign, split, by_row, files}。"""
    paths, deleted, groups, message, scratch, patches = [], [], [], 'batch-target', None, []
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
        elif token == '--patch':
            patches.append(value(token))
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
    if not paths and not deleted and not groups and not patches:
        raise Refusal('没有点名任何路径\n--path 整份取、--match 正则 文件 按块挑、--deleted 去掉被删的、--patch 补丁目录')
    return paths, deleted, groups, message, scratch, patches


def clean_relative(root, path):
    """相对仓根的规范路径；越出仓根的拒。"""
    normal = os.path.normpath(path)
    if os.path.isabs(normal) or normal == '..' or normal.startswith('..' + os.sep) or normal == '.':
        raise Refusal(f'路径 {path} 不是仓内的相对路径\n路径写成相对仓根的，例：.claude/kb/experiments-history.md')
    return normal


def build(root, argv):
    paths, deleted, groups, message, scratch, patches = parse_arguments(argv)
    paths = [clean_relative(root, p) for p in paths]
    deleted = [clean_relative(root, p) for p in deleted]
    picked_files = [clean_relative(root, f) for group in groups for f in group['files']]
    plan = read_patch_plan(root, patches)
    patch_touched = []
    for _patch_file, touched, _deleted in plan:
        patch_touched += [clean_relative(root, path) for path in touched if clean_relative(root, path) not in patch_touched]
    selections = paths + deleted + picked_files + patch_touched
    for index, name in enumerate(selections):
        for other in selections[:index]:
            if name == other or name.startswith(other + '/') or other.startswith(name + '/'):
                raise Refusal(f'{name} 与 {other} 重复或互相包含，后一次会覆盖前一次选择\n'
                              '一个路径只选择一次；整份取目录时不要再按块挑它底下的文件')
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
        if plan:
            entries, problems = synthesize_patches(root, head, plan, scratch_directory)
            if problems:
                raise Refusal(f'--patch 做不出「按补丁算的这一批」：{len(problems)} 处不对（逐处列在下面）\n'
                              '没倒掉的补丁先打全再重做；冲突是这一批与别的会话改在同一处，等对方先提交，或这几个文件改用 --match 按块挑；补丁删了的文件核对工作区里为什么还在\n'
                              + '\n'.join(problems))
            zero = '0' * len(head)
            lines = [f'{entry[0]} {entry[1]}\t{path}\0' if entry else f'0 {zero}\t{path}\0' for path, entry in entries.items()]
            if lines:
                run_git(root, ['update-index', '-z', '--index-info'], environment, ''.join(lines))
        if groups:
            for note in seed_missing_bases(root, head, picked_files, environment):
                print(f'  ℹ {note}', file=sys.stderr)
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
                            reasons = {2: '它不在 HEAD 里，真索引里没有，搬迁登记表倒推的旧路径在 HEAD 里也没有（新文件）：整份是这一批的就用 --path',
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
    print(f'✓ 自检通过：别的路径的工作区改动不进提交对象、点名的整份取、按块挑只进命中块、--keep-foreign 的组把挤着别人改动的块照 HEAD 留下（没带它拒）、被删的去掉、不在 HEAD 里的文件按块挑以真索引或搬迁前旧路径那一版当底、点名的路径不存在与一块都没命中都退 2 且标准输出为空、'
          f'--patch 把叠改同一文件的两份补丁外加别人的改动合成成 HEAD 加这一批（冲突、倒不掉、删了的文件工作区里还在都退 2 并逐个列出）、'
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
                 'history.md': history_base, 'shared.md': shared_base, 'olddir/moved.md': history_base}
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
        # 只在暂存区的新文件（HEAD 里没有，真索引里有）：暂存的是 history_base，工作区里在最上面又加了我的一块和别人的一块
        both_blocks = history_base.replace('### 2026-09-11（其八）', mine_block + their_block + '### 2026-09-11（其八）', 1)
        mine_only = history_base.replace('### 2026-09-11（其八）', mine_block + '### 2026-09-11（其八）', 1)
        write('staged-new.md', history_base)
        git('add', 'staged-new.md')
        write('staged-new.md', both_blocks)
        # 搬迁后的路径：olddir 整个搬成 newdir（登记在搬迁登记表里），工作区里只有新路径，里面同样加了两块；旧路径在这一批里要用 --deleted 去掉
        os.remove(os.path.join(repo, 'olddir', 'moved.md'))
        write('newdir/moved.md', both_blocks)
        write('.claude/gate.d/path-moves.tsv', 'olddir\tnewdir\t2026-10-23\trecords/2026-10-23-moves.md\n')
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
        # 不在 HEAD 里的文件按块挑：底取真索引里那一版（只在暂存区的新文件），或搬迁登记表倒推的旧路径在 HEAD 里的那一版；真索引照旧不变
        staged_new = invoke('--match', '其十', '--foreign', '其九', 'staged-new.md')
        staged_new_commit = staged_new.stdout.strip()
        if staged_new.returncode != 0 or not re.fullmatch(r'[0-9a-f]{40}', staged_new_commit):
            problems.append(f'只在暂存区的新文件按块挑应当退 0、标准输出只有一个 40 位哈希：退 {staged_new.returncode}，stdout={staged_new.stdout!r}，stderr={staged_new.stderr[-300:]!r}')
        elif committed(staged_new_commit, 'staged-new.md') != mine_only:
            problems.append('只在暂存区的新文件按块挑，提交对象里该是暂存区那一版加命中的那一块（别人的那一块不进）')
        moved = invoke('--deleted', 'olddir/moved.md', '--match', '其十', '--foreign', '其九', 'newdir/moved.md')
        moved_commit = moved.stdout.strip()
        if moved.returncode != 0 or not re.fullmatch(r'[0-9a-f]{40}', moved_commit):
            problems.append(f'搬迁后的路径按块挑应当退 0、标准输出只有一个 40 位哈希：退 {moved.returncode}，stdout={moved.stdout!r}，stderr={moved.stderr[-300:]!r}')
        elif committed(moved_commit, 'newdir/moved.md') != mine_only or committed(moved_commit, 'olddir/moved.md') is not None:
            problems.append('搬迁后的路径按块挑，提交对象里该是旧路径在 HEAD 里的那一版加命中的那一块、旧路径用 --deleted 去掉')
        if snapshot()[0] != before[0]:
            problems.append('不在 HEAD 里的文件按块挑做完，真索引变了（底只许放进临时索引）')
        problems += patch_scenario(mode)
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


def patch_scenario(mode):
    """--patch 的红绿样本：临时仓里造「别的会话没提交的改动 + 一串已经打在工作区上的补丁」，返回不合预期的句子（空表示全对）。
    mode 非空时给子进程设 BATCH_TARGET_BREAK。样本：叠改同一文件的两份补丁外加别人改的同一文件，合成 = HEAD 加这一批；
    补丁改在别人改过的那一行上（冲突）、补丁删了的文件工作区里还在、补丁没打在工作区上，各自判退 2 并逐个列出。"""
    problems = []
    with tempfile.TemporaryDirectory() as parent:
        main_repo, author_repo = os.path.join(parent, 'main'), os.path.join(parent, 'author')

        def git_in(repo, *arguments):
            return subprocess.run(['git', '-C', repo, '-c', 'user.email=selftest@example.invalid', '-c', 'user.name=selftest', *arguments],
                                  capture_output=True, text=True, check=True).stdout

        def write_in(repo, name, text):
            with open(os.path.join(repo, name), 'w', encoding='utf-8') as handle:
                handle.write(text)

        def read_in(repo, name):
            with open(os.path.join(repo, name), encoding='utf-8') as handle:
                return handle.read()

        def author_patch(name):
            """作者仓里相对上一次提交的全部改动存成补丁目录 <name>/crates.patch，再提交（下一份补丁接着它做）。"""
            directory = os.path.join(parent, name)
            os.makedirs(directory)
            git_in(author_repo, 'add', '-A')
            write_in(directory, 'crates.patch', git_in(author_repo, 'diff', '--cached'))
            git_in(author_repo, 'commit', '-q', '-m', name)
            return directory

        def invoke(*arguments):
            environment = {name: value for name, value in os.environ.items() if name != 'BATCH_TARGET_BREAK'}
            if mode:
                environment['BATCH_TARGET_BREAK'] = mode
            return subprocess.run([sys.executable, os.path.realpath(__file__), *arguments], cwd=main_repo, capture_output=True, text=True, env=environment)

        def committed(commit, name):
            shown = subprocess.run(['git', '-C', main_repo, 'show', f'{commit}:{name}'], capture_output=True, text=True)
            return shown.stdout if shown.returncode == 0 else None

        os.makedirs(main_repo)
        git_in(main_repo, 'init', '-q')
        base_text = ''.join(f'L{number}\n' for number in range(1, 10))
        others_text = base_text.replace('L7\n', 'L7-others\n')
        for name, text in (('app.txt', base_text), ('gone.txt', '待删\n'), ('keep.txt', '留着\n')):
            write_in(main_repo, name, text)
        git_in(main_repo, 'add', '-A')
        git_in(main_repo, 'commit', '-q', '-m', 'base')
        subprocess.run(['git', 'clone', '-q', main_repo, author_repo], check=True, capture_output=True)
        write_in(author_repo, 'app.txt', others_text)       # 别的会话没提交的改动：改了第七行、新建一个文件
        write_in(author_repo, 'others.txt', '别人新建的\n')
        git_in(author_repo, 'add', '-A')
        git_in(author_repo, 'commit', '-q', '-m', 'others')
        write_in(author_repo, 'app.txt', others_text.replace('L5\n', 'L5-batch\n'))   # 这一批的第一份补丁：改第五行（上下文里带着别人改的第七行）、新建一个文件
        write_in(author_repo, 'fresh.txt', '这一批新建的\n')
        first_patch = author_patch('line-five-edit')
        write_in(author_repo, 'app.txt', others_text.replace('L5\n', 'L5-final\n').replace('L3\n', 'L3\nL3-added\n'))   # 第二份补丁：同一行再改一次、再插一行、删一个文件
        os.remove(os.path.join(author_repo, 'gone.txt'))
        second_patch = author_patch('insert-and-delete')
        write_in(author_repo, 'app.txt', others_text.replace('L5\n', 'L5-final\n').replace('L3\n', 'L3\nL3-added\n').replace('L7-others\n', 'L7-batch\n'))
        conflicting_patch = author_patch('line-seven-edit')    # 改在别人改过的那一行上
        os.remove(os.path.join(author_repo, 'keep.txt'))
        keep_deleting_patch = author_patch('delete-keep')
        write_in(author_repo, 'app.txt', others_text.replace('L5\n', 'L5-final\n').replace('L3\n', 'L3\nL3-added\n').replace('L7-others\n', 'L7-batch\n').replace('L2\n', 'L2-edit\n'))
        unapplied_patch = author_patch('line-two-edit')
        # 主工作区：HEAD + 别人的改动 + 这一批的前两份补丁
        write_in(main_repo, 'app.txt', others_text)
        write_in(main_repo, 'others.txt', '别人新建的\n')
        git_in(main_repo, 'apply', os.path.join(first_patch, 'crates.patch'))
        git_in(main_repo, 'apply', os.path.join(second_patch, 'crates.patch'))
        expected_app = base_text.replace('L3\n', 'L3\nL3-added\n').replace('L5\n', 'L5-final\n')
        synthesized = invoke('--patch', first_patch, '--patch', second_patch)
        commit = synthesized.stdout.strip()
        if synthesized.returncode != 0 or not re.fullmatch(r'[0-9a-f]{40}', commit) or synthesized.stdout != commit + '\n':
            problems.append(f'--patch 两份补丁叠改同一文件、外加别人改的同一文件，应当退 0、标准输出只有一个 40 位哈希：退 {synthesized.returncode}，'
                            f'stdout={synthesized.stdout!r}，stderr={synthesized.stderr[-400:]!r}')
        else:
            if committed(commit, 'app.txt') != expected_app:
                problems.append(f'--patch 合成出的 app.txt 该是 HEAD 加这一批的两份补丁（别人改的第七行不进）：{committed(commit, "app.txt")!r}')
            if committed(commit, 'fresh.txt') != '这一批新建的\n':
                problems.append('--patch 补丁新建的文件没进提交对象')
            if committed(commit, 'gone.txt') is not None:
                problems.append('--patch 补丁删了的文件还留在提交对象里')
            if committed(commit, 'others.txt') is not None or committed(commit, 'keep.txt') != '留着\n':
                problems.append('--patch 别人新建的文件进了提交对象，或补丁没碰的文件变了')
            if read_in(main_repo, 'app.txt') != others_text.replace('L5\n', 'L5-final\n').replace('L3\n', 'L3\nL3-added\n'):
                problems.append('--patch 做完工作区里的 app.txt 变了（只许读工作区）')
        git_in(main_repo, 'apply', os.path.join(conflicting_patch, 'crates.patch'))
        conflicting = invoke('--patch', conflicting_patch)
        if conflicting.returncode != 2 or conflicting.stdout != '' or 'app.txt' not in conflicting.stderr or '冲突' not in conflicting.stderr:
            problems.append(f'--patch 这一批与别人改在同一处应当退 2、标准输出为空、标准错误点名 app.txt 的冲突：退 {conflicting.returncode}，'
                            f'stdout={conflicting.stdout!r}，stderr={conflicting.stderr[-300:]!r}')
        kept = invoke('--patch', keep_deleting_patch)
        if kept.returncode != 2 or kept.stdout != '' or 'keep.txt' not in kept.stderr or '工作区里却还在' not in kept.stderr:
            problems.append(f'--patch 补丁删了的文件工作区里还在应当退 2、标准输出为空、标准错误点名 keep.txt：退 {kept.returncode}，stdout={kept.stdout!r}，stderr={kept.stderr[-300:]!r}')
        unapplied = invoke('--patch', unapplied_patch)
        if unapplied.returncode != 2 or unapplied.stdout != '' or 'app.txt' not in unapplied.stderr or '倒不掉' not in unapplied.stderr:
            problems.append(f'--patch 补丁没打在工作区上应当退 2、标准输出为空、标准错误点名倒不掉的 app.txt：退 {unapplied.returncode}，stdout={unapplied.stdout!r}，stderr={unapplied.stderr[-300:]!r}')
        overlapped = invoke('--patch', first_patch, '--path', 'app.txt')
        if overlapped.returncode != 2 or overlapped.stdout != '':
            problems.append(f'--patch 碰到的路径又用 --path 点名应当退 2、标准输出为空：退 {overlapped.returncode}，stdout={overlapped.stdout!r}')
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
        for detail in lines[2:]:
            print(f'       · {detail}', file=sys.stderr)
        return 2
    print(commit)
    return 0


BREAK_VARIABLES = ("BATCH_TARGET_BREAK",)


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or arguments[:1] == ["--selftest"] or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/kb/batch-target.py --selftest", file=sys.stderr)
    return 2


if __name__ == '__main__':
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main())
