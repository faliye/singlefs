#!/usr/bin/env python3
# admission: always 每一次调都判此刻的主工作区与这一次给的补丁目录，上一次的结论不替这一次作保
# run-condition: command git
"""按此刻的主工作区现算一份补丁目录：补丁撤回了一半、或经三方合并打进去、或主工作区上另有别人的改动，原补丁反着核不过时，用它交给 merge-and-verify.sh --skip-apply。

用法：
  patch-of-the-current-tree.py --out <新补丁目录> [--tree <树>] <补丁目录> [<补丁目录> …]
  patch-of-the-current-tree.py --selftest      # PATCH_OF_THE_CURRENT_TREE_BREAK=<项> 时必须判红（项名见文末）
  起法：task-server/task submit -- python3 scripts/gate/patch-of-the-current-tree.py --out <工作根下的新目录> <补丁目录> …

输入：一份或几份实现员补丁目录（格式由 .claude/agents/implementation-writer.md「产出」一节定），照打的次序给：
  crates.patch                 补丁点到的每份文件都要有对应的基线：baseline/<路径>，补丁新建的文件除外
  baseline/<相对仓根的路径>     补丁前原样
  mutations-append.tsv、mutations-replacements.tsv   变异行；mutations-delete.txt 不带（merge-and-verify.sh 不读它）
--tree 是此刻的树，缺省是这份脚本所在的仓根。

做什么：
  1. 补丁目录点到的每份文件，取最早点到它的那份补丁目录的基线（baseline/<路径>；那份补丁新建的文件没有基线，当作不存在），
     算「基线 → 此刻树里那一份」的 diff：在临时 git 仓里放基线、再放树里的现状，git diff --binary --full-index --no-renames，
     合成新 crates.patch（diff --git a/<路径> b/<路径>，路径不带目录前缀）。基线与树里一样的文件不进补丁；树里没有的文件写成删除。
     新补丁目录里的 baseline/ 放新补丁点到的那些文件取用的基线。
  2. 变异行：mutations-append.tsv 与 mutations-replacements.tsv 的每一行，整行（含制表符，逐字、不用正则）还在此刻树里 crates/mutations.tsv 的才留，
     几份补丁目录的行按出现次序合并、重复的留一行。一行不剩的文件不写。
  3. 在 --tree 上 git apply --check --reverse 新 crates.patch 核过才算成：核不过退 1、不留任何产物。
  4. 新补丁目录（--out）必须是不存在的、父目录在的；先在父目录里的临时目录里写全、核完再改名换上，失败时临时目录删净。
读基线、算 diff 都不碰 --tree：只读它里面的文件，在 --tree 上只跑 git apply --check。

新补丁的含义是「此刻树相对最早基线的全部改动」：主工作区上与补丁无关、但落在同一份文件里的别人的改动也在里面（它们本来就在此刻树里）；
补丁没点到的文件不在里面。拿它 --skip-apply 验，验的是此刻树上这些文件的现状。

退出码：0 成；1 新补丁在 --tree 上反着核不过（算完之后树又变了，或 diff 本身有误），没留产物；
  2 用法错、补丁目录读不了、补丁点到树外的路径、点到的已有文件在基线里没有、输出目录已在、临时目录建不成；
  3 补丁全撤回了：没有改动、没有变异行可留，没留产物；78 准入或运行条件不满足。

自证（--selftest）：在临时目录里造假仓（git init 的假树）与假补丁目录，十格，其中六格各配一条弄坏开关（文末）：
  补丁全打着、撤回一半、三方合并打进去、变异行被改过、输出目录已在、两份补丁点到同一份文件、基线缺失、算完之后树又被改、补丁全撤回、补丁点到树外。
  只起 git，不起 cargo；平常跑设了弄坏开关会退 2。

弄坏开关 PATCH_OF_THE_CURRENT_TREE_BREAK=<项>（只给 --selftest 证红用，打开后 --selftest 退 1；项名、弄坏哪一处、红在哪一格，「」里是失败行的开头）：
  PATCH_OF_THE_CURRENT_TREE_BREAK=keep-all-mutation-lines   变异行照抄，不筛此刻表里还在不在
      红在「✗ 自检：变异行被改过：只留此刻表里逐字还在的」
  PATCH_OF_THE_CURRENT_TREE_BREAK=skip-reverse-check   不在 --tree 上做 git apply --check --reverse
      红在「✗ 自检：算完补丁之后树又被改：反向核不过时退 1、不留产物」
  PATCH_OF_THE_CURRENT_TREE_BREAK=baseline-latest   几份补丁点到同一份文件时取最晚那份的基线
      红在「✗ 自检：两份补丁点到同一份文件：取最早那份的基线」
  PATCH_OF_THE_CURRENT_TREE_BREAK=out-overwritten   输出目录已在时先删掉它再写
      红在「✗ 自检：输出目录已在：拒、原目录一个字节不动」
  PATCH_OF_THE_CURRENT_TREE_BREAK=missing-baseline-as-new   已有文件在基线里没有时当作新建文件
      红在「✗ 自检：点到的已有文件在基线里没有：拒、不留产物」
  PATCH_OF_THE_CURRENT_TREE_BREAK=unsafe-paths-allowed   补丁点到绝对路径或带 .. 的路径时不拒
      红在「✗ 自检：补丁点到树外的路径：拒、树外一个字不写」
"""
import collections
import contextlib
import importlib.util
import io
import os
import shutil
import subprocess
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

BREAK_VARIABLE = 'PATCH_OF_THE_CURRENT_TREE_BREAK'
BREAK_VARIABLES = (BREAK_VARIABLE,)
EXIT_REVERSE_CHECK_FAILED = 1
EXIT_USAGE = 2
EXIT_NOTHING_LEFT = 3
PATCH_FILE_NAME = 'crates.patch'
BASELINE_DIRECTORY_NAME = 'baseline'
MUTATION_APPEND_NAME = 'mutations-append.tsv'
MUTATION_REPLACEMENTS_NAME = 'mutations-replacements.tsv'
MUTATION_TABLE_PATH = 'crates/mutations.tsv'
WORK_DIRECTORY_PREFIX = '.patch-of-the-current-tree-'
HERE = os.path.dirname(os.path.realpath(__file__))
DEFAULT_TREE = os.path.normpath(os.path.join(HERE, '..', '..'))

Source = collections.namedtuple('Source', 'directory paths created')


class Refusal(Exception):
    """退 2 的拒绝：说明一句、出路一句。"""

    def __init__(self, problem, way_out, code=EXIT_USAGE):
        super().__init__(problem)
        self.problem = problem
        self.way_out = way_out
        self.code = code


def broken(item):
    return os.environ.get(BREAK_VARIABLE, '') == item


def load_sibling(*relative_parts):
    """按路径载入同仓里的另一份脚本（和 lib_rebuilt_packages.py 载入 snapshot-tree.py 同一个办法）。"""
    path = os.path.join(HERE, *relative_parts)
    specification = importlib.util.spec_from_file_location('loaded_' + os.path.basename(path).replace('-', '_').replace('.py', ''), path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def git_environment():
    """去掉调用方带来的 GIT_ 变量（GIT_DIR 这类会让 git 去认别的仓），不读用户与系统的 git 配置。"""
    environment = {name: value for name, value in os.environ.items() if not name.startswith('GIT_')}
    environment['GIT_CONFIG_GLOBAL'] = os.devnull
    environment['GIT_CONFIG_NOSYSTEM'] = '1'
    return environment


def run_git(arguments, cwd, input_bytes=None):
    """在 cwd 里跑 git。GIT_CEILING_DIRECTORIES 设成 cwd 的父目录：git 只认 cwd 自己是不是仓根，不往上找别的仓
    （临时目录落在某个仓里时，git apply 会只认仓内子目录的路径）。"""
    environment = git_environment()
    environment['GIT_CEILING_DIRECTORIES'] = os.path.dirname(os.path.realpath(cwd))
    return subprocess.run(['git', '-c', 'core.autocrlf=false', '-c', 'core.fileMode=false', '-c', 'core.safecrlf=false',
                           '-c', 'core.quotePath=true', '--literal-pathspecs', *arguments],
                          cwd=cwd, input=input_bytes, capture_output=True, env=environment)


def run_git_checked(arguments, cwd, input_bytes=None):
    done = run_git(arguments, cwd, input_bytes)
    if done.returncode != 0:
        raise Refusal(f'git {" ".join(arguments[:2])} 在 {cwd} 里退 {done.returncode}：{done.stderr.decode("utf-8", "replace").strip()[:400]}',
                      '看 git 在不在 PATH 上、临时目录可不可写、有没有空间，再重跑同一条')
    return done.stdout


def read_text(path):
    with open(path, 'rb') as handle:
        return handle.read().decode('utf-8', 'surrogateescape')


def table_lines(text):
    """按 \\n 切行（不用 splitlines：它还按 \\x0c 这类字符切），不去 \\r，整行比对时逐字。"""
    lines = text.split('\n')
    if lines and lines[-1] == '':
        lines.pop()
    return lines


def created_paths_of(patch_path, text, scratch, unquote):
    """补丁新建（含重命名与拷贝的目标）的路径：git apply --summary 的 create mode 行与补丁头里的 rename to、copy to。"""
    created = set()
    summary = run_git_checked(['apply', '--summary', patch_path], scratch).decode('utf-8', 'surrogateescape')
    for line in summary.split('\n'):
        words = line.split(' ', 4)
        if len(words) == 5 and words[1:3] == ['create', 'mode']:
            created.add(unquote(words[4]))
    in_header = False
    for line in text.split('\n'):
        if line.startswith('diff --git '):
            in_header = True
        elif line.startswith('@@') or line.startswith('GIT binary patch'):
            in_header = False
        elif in_header and line.startswith(('rename to ', 'copy to ')):
            created.add(unquote(line.split(' to ', 1)[1]))
    return created


def read_sources(patch_directories, scratch):
    """→ [Source]：每个补丁目录点到的路径（按出现次序）与它新建的路径。补丁目录不在就拒；点到树外路径的拒。"""
    rebuilt = load_sibling('lib_rebuilt_packages.py')
    unquote = load_sibling('..', 'agents', 'snapshot-tree.py').unquoted
    sources = []
    for directory in patch_directories:
        if not os.path.isdir(directory):
            raise Refusal(f'补丁目录 {directory} 不在', '补丁目录写对再来')
        patch_path = os.path.join(directory, PATCH_FILE_NAME)
        paths, created = [], set()
        if os.path.isfile(patch_path):
            text = read_text(patch_path)
            paths = rebuilt.paths_in_patch(text)
            outside = [path for path in paths if os.path.isabs(path) or '..' in path.split('/')]
            if outside and not broken('unsafe-paths-allowed'):
                raise Refusal(f'补丁目录 {directory} 的 {PATCH_FILE_NAME} 点到树外的路径：{"、".join(outside[:5])}', '这样的补丁不收；让实现员重给一份只改仓里文件的补丁')
            created = created_paths_of(patch_path, text, scratch, unquote)
        sources.append(Source(directory, paths, created))
    return sources


def choose_baselines(sources):
    """→ {路径: 基线文件的路径，新建的文件是 None}：每份文件取最早点到它的那份补丁目录里的基线（弄坏开关 baseline-latest 取最晚的）。"""
    ordered = list(reversed(sources)) if broken('baseline-latest') else sources
    chosen, missing = {}, []
    for source in ordered:
        for path in source.paths:
            if path in chosen:
                continue
            baseline = os.path.join(source.directory, BASELINE_DIRECTORY_NAME, path)
            if os.path.isfile(baseline):
                chosen[path] = baseline
            elif path in source.created or broken('missing-baseline-as-new'):
                chosen[path] = None
            else:
                missing.append((source.directory, path))
    if missing:
        listed = '、'.join(f'{directory} 的 {BASELINE_DIRECTORY_NAME}/{path}' for directory, path in missing[:8])
        raise Refusal(f'补丁点到的已有文件在基线里没有，算不出基线：{listed}{" …" if len(missing) > 8 else ""}（共 {len(missing)} 份）',
                      '让实现员补上 baseline/（补丁前原样，副本里 git show :<路径> 取）再来；不要拿此刻树里的文件当基线')
    return chosen


def copy_into(source, root, path):
    destination = os.path.join(root, path)
    os.makedirs(os.path.dirname(destination), exist_ok=True)
    shutil.copyfile(source, destination)


def diff_from_baselines(tree, chosen, repo):
    """→ (crates.patch 的字节, 有改动的路径)：临时仓里放基线，再放树里的现状，git diff。"""
    run_git_checked(['init', '-q', repo], os.path.dirname(repo))
    with_baseline = [path for path, baseline in chosen.items() if baseline is not None]
    for path in with_baseline:
        copy_into(chosen[path], repo, path)
    if with_baseline:
        run_git_checked(['add', '--pathspec-from-file=-', '--pathspec-file-nul'], repo, b'\0'.join(path.encode('utf-8', 'surrogateescape') for path in with_baseline))
    present_new = []
    for path, baseline in chosen.items():
        current = os.path.join(tree, path)
        if os.path.lexists(current):
            copy_into(current, repo, path)
            if baseline is None:
                present_new.append(path)
        elif os.path.lexists(os.path.join(repo, path)):
            os.remove(os.path.join(repo, path))
    if present_new:
        run_git_checked(['add', '-N', '--pathspec-from-file=-', '--pathspec-file-nul'], repo, b'\0'.join(path.encode('utf-8', 'surrogateescape') for path in present_new))
    options = ['diff', '--no-color', '--no-ext-diff', '--no-textconv', '--no-renames', '--binary', '--full-index']
    patch = run_git_checked(options, repo)
    names = run_git_checked(options + ['--name-only', '-z'], repo).decode('utf-8', 'surrogateescape').split('\0')
    return patch, [name for name in names if name]


def surviving_rows(sources_directories, file_name, table):
    """→ (读到的行数, 留下的行)：整行逐字还在 table（此刻表的行集合）里的才留；重复的留第一次。"""
    kept, seen, read = [], set(), 0
    for directory in sources_directories:
        path = os.path.join(directory, file_name)
        if not os.path.isfile(path):
            continue
        for line in table_lines(read_text(path)):
            if not line:
                continue
            read += 1
            if (broken('keep-all-mutation-lines') or line in table) and line not in seen:
                seen.add(line)
                kept.append(line)
    return read, kept


def write_rows(path, rows):
    with open(path, 'wb') as handle:
        handle.write(('\n'.join(rows) + '\n').encode('utf-8', 'surrogateescape'))


def reverse_check(tree, patch_path):
    done = run_git(['apply', '--check', '--reverse', patch_path], tree)
    return done.returncode == 0, (done.stdout + done.stderr).decode('utf-8', 'replace').strip()


def make_patch(patch_directories, out, tree, after_diff=None):
    """→ 退出码。after_diff 只给自证：算完 diff 之后、核之前调一下（演「树又被改了」）。"""
    out = os.path.abspath(out)
    tree = os.path.abspath(tree)
    parent = os.path.dirname(out)
    if not os.path.isdir(parent):
        raise Refusal(f'输出目录 {out} 的父目录不在', '--out 写在已有的目录下（工作根下），目录本身不能已经在')
    if os.path.lexists(out):
        if not broken('out-overwritten'):
            raise Refusal(f'输出目录 {out} 已在，一个字节没动', '换一个不存在的目录名；要覆盖，自己确认里面没有要留的东西、删了它再来')
        shutil.rmtree(out)
    if not os.path.isdir(tree):
        raise Refusal(f'--tree {tree} 不是目录', '写对此刻的树')
    try:
        work = tempfile.mkdtemp(prefix=WORK_DIRECTORY_PREFIX, dir=parent)
    except OSError as error:
        raise Refusal(f'{parent} 下建不了临时目录：{error}', '看 --out 的父目录可不可写、有没有空间') from error
    try:
        return build_and_install(patch_directories, out, tree, work, after_diff)
    finally:
        shutil.rmtree(work, ignore_errors=True)
        if os.path.lexists(work):
            print(f'  ! 临时目录 {work} 没删净，请手工删掉')


def build_and_install(patch_directories, out, tree, work, after_diff):
    sources = read_sources(patch_directories, work)
    chosen = choose_baselines(sources)
    product = os.path.join(work, 'product')
    os.makedirs(product)
    patch_bytes, changed = diff_from_baselines(tree, chosen, os.path.join(work, 'repo'))
    rows = {}
    wanted = [(MUTATION_APPEND_NAME, '追加'), (MUTATION_REPLACEMENTS_NAME, '替换')]
    directories = [source.directory for source in sources]
    if any(os.path.isfile(os.path.join(directory, name)) for directory in directories for name, _ in wanted):
        table_path = os.path.join(tree, MUTATION_TABLE_PATH)
        if not os.path.isfile(table_path):
            raise Refusal(f'--tree {tree} 里没有 {MUTATION_TABLE_PATH}，筛不了变异行', '写对 --tree，或补丁目录里不带变异行')
        table = set(table_lines(read_text(table_path)))
        for name, _ in wanted:
            rows[name] = surviving_rows(directories, name, table)
    if patch_bytes:
        with open(os.path.join(product, PATCH_FILE_NAME), 'wb') as handle:
            handle.write(patch_bytes)
        for path in changed:
            if chosen.get(path) is not None:
                copy_into(chosen[path], os.path.join(product, BASELINE_DIRECTORY_NAME), path)
    for name, (_, kept) in rows.items():
        if kept:
            write_rows(os.path.join(product, name), kept)
    if after_diff is not None:
        after_diff()
    mutation_text = '；'.join(f'变异行{label} {len(rows[name][1])}/{rows[name][0]}' for name, label in wanted if name in rows)
    if not patch_bytes and not any(kept for _, kept in rows.values()):
        print(f'  - 补丁点到的 {len(chosen)} 份文件此刻都与基线相同，变异行也没有还在的：没有可验的改动，没留产物（{mutation_text or "没有变异行"}）')
        print('     → 怎么办：补丁已经整份撤回，不用验；要验别的补丁目录，换目录再来')
        return EXIT_NOTHING_LEFT
    if patch_bytes and not broken('skip-reverse-check'):
        passed, detail = reverse_check(tree, os.path.join(product, PATCH_FILE_NAME))
        if not passed:
            print(f'  ✗ 现算的 {PATCH_FILE_NAME} 在 {tree} 上 git apply --check --reverse 核不过，没留产物：{detail[:600]}')
            print('     → 怎么办：算完之后这些文件又被改了，或补丁点到的文件不是普通文件；确认主工作区不再动这些文件，再重跑同一条')
            return EXIT_REVERSE_CHECK_FAILED
    if os.path.lexists(out):
        raise Refusal(f'输出目录 {out} 在算的过程中被别人建了，没动它', '换一个目录名再来')
    os.rename(product, out)
    unchanged = len(chosen) - len(changed)
    print(f'  ✓ 现算了 {out}：{PATCH_FILE_NAME} 点到 {len(changed)} 份文件（给的 {len(directories)} 个补丁目录共点到 {len(chosen)} 份，{unchanged} 份此刻与基线相同、不在里面）；'
          f'{mutation_text or "没有变异行"}（留下的/读到的）；'
          f'{"已在 " + tree + " 上 git apply --check --reverse 核过" if patch_bytes and not broken("skip-reverse-check") else "没有 crates.patch，不核反向"}')
    return 0


def usage_error(message):
    print(f'  ✗ {message}')
    print('     → 怎么办：patch-of-the-current-tree.py --out <不存在的新补丁目录> [--tree <树>] <补丁目录> [<补丁目录> …]')
    return EXIT_USAGE


def run(arguments, after_diff=None):
    out, tree, directories, index = None, DEFAULT_TREE, [], 0
    while index < len(arguments):
        word = arguments[index]
        if word in ('--out', '--tree'):
            if index + 1 >= len(arguments):
                return usage_error(f'{word} 后面缺值')
            if word == '--out':
                out = arguments[index + 1]
            else:
                tree = arguments[index + 1]
            index += 2
        elif word.startswith('-'):
            return usage_error(f'认不出参数 {word}')
        else:
            directories.append(os.path.abspath(word))
            index += 1
    if out is None:
        return usage_error('没给 --out')
    if not directories:
        return usage_error('没给补丁目录')
    try:
        return make_patch(directories, out, tree, after_diff)
    except Refusal as refusal:
        print(f'  ✗ {refusal.problem}')
        print(f'     → 怎么办：{refusal.way_out}')
        return refusal.code
    except OSError as error:
        print(f'  ✗ 读写文件出错：{error}')
        print('     → 怎么办：看补丁目录、--tree 与 --out 的父目录在不在、可不可读写，再重跑同一条')
        return EXIT_USAGE


# ---- 自证 ----

ORIGINAL_LINES = [f'line {number}' for number in range(1, 11)]


def numbered(replacements=None):
    """十行的假源文件；replacements 是 {行号: 新内容}。"""
    lines = list(ORIGINAL_LINES)
    for number, text in (replacements or {}).items():
        lines[number - 1] = text
    return '\n'.join(lines) + '\n'


def mutation_row(name, path='crates/alpha/src/a.rs'):
    return f'{name}\t{path}\t1\t2\t-p alpha --lib\t{name}'


class World:
    """假仓：tree 是 git 仓，提交里是补丁前的原样；writer_patch 像实现员那样出一份补丁目录并提交，让下一份补丁以它为基线。"""

    def __init__(self, directory):
        self.directory = directory
        self.tree = os.path.join(directory, 'tree')
        os.makedirs(self.tree)
        self.git('init', '-q')

    def git(self, *arguments, input_bytes=None):
        return run_git_checked(list(arguments), self.tree, input_bytes)

    def write(self, path, text):
        full = os.path.join(self.tree, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, 'wb') as handle:
            handle.write(text.encode('utf-8', 'surrogateescape'))

    def read(self, path):
        full = os.path.join(self.tree, path)
        return read_text(full) if os.path.isfile(full) else None

    def commit(self):
        self.git('add', '-A')
        self.git('-c', 'user.name=selftest', '-c', 'user.email=selftest@example.invalid', '-c', 'commit.gpgsign=false', 'commit', '-q', '--allow-empty', '-m', 'state')
        return self.git('rev-parse', 'HEAD').decode('ascii').strip()

    def writer_patch(self, name, edits, append_rows=(), replacement_rows=()):
        """edits：{路径: 新内容，None 是删}。改工作区、出补丁目录（crates.patch 不含变异表、baseline/ 是提交里的原样）、变异表追加这些行，再提交。"""
        patch_directory = os.path.join(self.directory, name)
        os.makedirs(patch_directory)
        for path, text in edits.items():
            if text is None:
                os.remove(os.path.join(self.tree, path))
            else:
                self.write(path, text)
        self.git('add', '-A', '--', 'crates')
        diff_options = ['diff', '--cached', '--no-color', '--no-renames', '--binary']
        patch = self.git(*diff_options, '--', 'crates')
        names = [item for item in self.git(*diff_options, '--name-only', '-z', '--', 'crates').decode('utf-8').split('\0') if item]
        with open(os.path.join(patch_directory, PATCH_FILE_NAME), 'wb') as handle:
            handle.write(patch)
        for path in names:
            shown = run_git(['show', f'HEAD:{path}'], self.tree)
            if shown.returncode == 0:
                destination = os.path.join(patch_directory, BASELINE_DIRECTORY_NAME, path)
                os.makedirs(os.path.dirname(destination), exist_ok=True)
                with open(destination, 'wb') as handle:
                    handle.write(shown.stdout)
        for file_name, rows in ((MUTATION_APPEND_NAME, append_rows), (MUTATION_REPLACEMENTS_NAME, replacement_rows)):
            if rows:
                write_rows(os.path.join(patch_directory, file_name), list(rows))
        appended = self.read(MUTATION_TABLE_PATH) or ''
        self.write(MUTATION_TABLE_PATH, appended + ''.join(row + '\n' for row in list(append_rows) + list(replacement_rows)))
        self.commit()
        return patch_directory


def fresh_world(directory, files):
    world = World(directory)
    world.write(MUTATION_TABLE_PATH, mutation_row('existing') + '\n')
    for path, text in files.items():
        world.write(path, text)
    return world, world.commit()


def run_tool(arguments, after_diff=None):
    """进程内跑一遍 run()，→ (退出码, 标准输出与标准错误的文字)。"""
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer), contextlib.redirect_stderr(buffer):
        code = run(arguments, after_diff)
    return code, buffer.getvalue()


def numstat(patch_path, scratch):
    text = run_git_checked(['apply', '--numstat', patch_path], scratch).decode('utf-8')
    return sorted(line for line in text.split('\n') if line)


def replay(revision, patch_path, paths, scratch, world):
    """补丁前（revision 里）的文件套上补丁之后的内容：{路径: 内容或 None}。"""
    directory = tempfile.mkdtemp(prefix='replay-', dir=scratch)
    run_git_checked(['init', '-q', directory], scratch)
    for path in paths:
        shown = run_git(['show', f'{revision}:{path}'], world.tree)
        if shown.returncode == 0:
            full = os.path.join(directory, path)
            os.makedirs(os.path.dirname(full), exist_ok=True)
            with open(full, 'wb') as handle:
                handle.write(shown.stdout)
    run_git_checked(['apply', patch_path], directory)
    return {path: (read_text(os.path.join(directory, path)) if os.path.isfile(os.path.join(directory, path)) else None) for path in paths}


def leftovers(directory):
    return [name for name in os.listdir(directory) if name.startswith(WORK_DIRECTORY_PREFIX)]


BASIC_FILES = {'crates/alpha/src/a.rs': numbered(), 'crates/alpha/src/b.rs': numbered(), 'crates/alpha/src/d.rs': numbered()}


def cell_patch_applied_everywhere(directory):
    world, initial = fresh_world(directory, BASIC_FILES)
    rows = [mutation_row('added-row-alpha'), mutation_row('added-row-beta')]
    patch = world.writer_patch('patch', {'crates/alpha/src/a.rs': numbered({2: 'edited 2'}), 'crates/alpha/src/b.rs': numbered({9: 'edited 9'}),
                                         'crates/alpha/src/n.rs': 'new file\n', 'crates/alpha/src/d.rs': None}, append_rows=rows)
    out = os.path.join(directory, 'out')
    code, text = run_tool(['--out', out, '--tree', world.tree, patch])
    if code != 0:
        return f'应当退 0，实际退 {code}：{text.strip()[:300]}'
    paths = sorted(BASIC_FILES) + ['crates/alpha/src/n.rs']
    if numstat(os.path.join(out, PATCH_FILE_NAME), directory) != numstat(os.path.join(patch, PATCH_FILE_NAME), directory):
        return '现算的补丁与原补丁的改动行数对不上'
    if replay(initial, os.path.join(out, PATCH_FILE_NAME), paths, directory, world) != {path: world.read(path) for path in paths}:
        return '现算的补丁套在基线上，得不到此刻树里的文件'
    if not reverse_check(world.tree, os.path.join(out, PATCH_FILE_NAME))[0]:
        return '现算的补丁在此刻树上反着核不过'
    baselines = sorted(os.path.relpath(os.path.join(root, name), os.path.join(out, BASELINE_DIRECTORY_NAME)) for root, _, names in os.walk(os.path.join(out, BASELINE_DIRECTORY_NAME)) for name in names)
    if baselines != sorted(BASIC_FILES):
        return f'新补丁目录的 baseline/ 应当是原补丁点到的三份已有文件，实际 {baselines}'
    if read_text(os.path.join(out, MUTATION_APPEND_NAME)) != '\n'.join(rows) + '\n':
        return '变异行都还在，应当原样留下'
    return None


def cell_half_withdrawn(directory):
    world, initial = fresh_world(directory, BASIC_FILES)
    patch = world.writer_patch('patch', {'crates/alpha/src/a.rs': numbered({2: 'edited 2'}), 'crates/alpha/src/b.rs': numbered({9: 'edited 9'})})
    world.write('crates/alpha/src/b.rs', numbered())
    if reverse_check(world.tree, os.path.join(patch, PATCH_FILE_NAME))[0]:
        return '夹具坏了：撤回一半之后原补丁在此刻树上不该反着核得过'
    out = os.path.join(directory, 'out')
    code, text = run_tool(['--out', out, '--tree', world.tree, patch])
    if code != 0:
        return f'应当退 0，实际退 {code}：{text.strip()[:300]}'
    touched = [line.split('\t')[2] for line in numstat(os.path.join(out, PATCH_FILE_NAME), directory)]
    if touched != ['crates/alpha/src/a.rs']:
        return f'现算的补丁应当只含还在的 a.rs，实际 {touched}'
    if os.path.exists(os.path.join(out, BASELINE_DIRECTORY_NAME, 'crates/alpha/src/b.rs')):
        return '撤回的 b.rs 不该在新补丁的 baseline/ 里'
    if not reverse_check(world.tree, os.path.join(out, PATCH_FILE_NAME))[0]:
        return '现算的补丁在此刻树上反着核不过'
    return None


def cell_merged_by_three_way(directory):
    world, initial = fresh_world(directory, {**BASIC_FILES, 'crates/alpha/src/c.rs': numbered()})
    patch = world.writer_patch('patch', {'crates/alpha/src/a.rs': numbered({5: 'edited 5'})})
    world.write('crates/alpha/src/a.rs', numbered({4: 'someone else 4', 5: 'edited 5'}))
    world.write('crates/alpha/src/c.rs', numbered({1: 'someone else 1'}))
    if reverse_check(world.tree, os.path.join(patch, PATCH_FILE_NAME))[0]:
        return '夹具坏了：别人的改动贴着补丁的行，原补丁在此刻树上不该反着核得过'
    out = os.path.join(directory, 'out')
    code, text = run_tool(['--out', out, '--tree', world.tree, patch])
    if code != 0:
        return f'应当退 0，实际退 {code}：{text.strip()[:300]}'
    touched = [line.split('\t')[2] for line in numstat(os.path.join(out, PATCH_FILE_NAME), directory)]
    if touched != ['crates/alpha/src/a.rs']:
        return f'现算的补丁应当只含补丁点到的 a.rs（不含没点到的 c.rs），实际 {touched}'
    if replay(initial, os.path.join(out, PATCH_FILE_NAME), ['crates/alpha/src/a.rs'], directory, world) != {'crates/alpha/src/a.rs': world.read('crates/alpha/src/a.rs')}:
        return '现算的补丁应当含别人的改动：套在基线上要得到此刻树里的 a.rs'
    if not reverse_check(world.tree, os.path.join(out, PATCH_FILE_NAME))[0]:
        return '现算的补丁在此刻树上反着核不过'
    return None


def cell_mutation_rows_edited(directory):
    world, _ = fresh_world(directory, BASIC_FILES)
    appended = [mutation_row('kept-row-alpha'), mutation_row('edited-row'), mutation_row('kept-row-beta')]
    replaced = [mutation_row('kept-replacement'), mutation_row('edited-replacement')]
    patch = world.writer_patch('patch', {'crates/alpha/src/a.rs': numbered({2: 'edited 2'})}, append_rows=appended, replacement_rows=replaced)
    table = world.read(MUTATION_TABLE_PATH)
    world.write(MUTATION_TABLE_PATH, table.replace(appended[1] + '\n', appended[1] + ' changed\n').replace(replaced[1] + '\n', replaced[1].replace('\t2\t', '\t3\t') + '\n'))
    out = os.path.join(directory, 'out')
    code, text = run_tool(['--out', out, '--tree', world.tree, patch])
    if code != 0:
        return f'应当退 0，实际退 {code}：{text.strip()[:300]}'
    got_append = read_text(os.path.join(out, MUTATION_APPEND_NAME))
    got_replacements = read_text(os.path.join(out, MUTATION_REPLACEMENTS_NAME))
    if got_append != appended[0] + '\n' + appended[2] + '\n' or got_replacements != replaced[0] + '\n':
        return f'只该留逐字还在表里的行：追加应当是 kept-row-alpha、kept-row-beta，替换应当是 kept-replacement，实际追加 {got_append!r}、替换 {got_replacements!r}'
    return None


def cell_output_directory_exists(directory):
    world, _ = fresh_world(directory, BASIC_FILES)
    patch = world.writer_patch('patch', {'crates/alpha/src/a.rs': numbered({2: 'edited 2'})})
    out = os.path.join(directory, 'out')
    os.makedirs(out)
    with open(os.path.join(out, 'marker'), 'w', encoding='utf-8') as handle:
        handle.write('keep\n')
    code, text = run_tool(['--out', out, '--tree', world.tree, patch])
    if code != 2 or '已在' not in text:
        return f'应当退 2、说输出目录已在，实际退 {code}：{text.strip()[:300]}'
    if not os.path.isfile(os.path.join(out, 'marker')) or read_text(os.path.join(out, 'marker')) != 'keep\n' or os.listdir(out) != ['marker']:
        return '原目录里的东西被动过'
    if leftovers(directory):
        return f'留下了临时目录 {leftovers(directory)}'
    return None


def cell_two_patches_share_a_file(directory):
    world, initial = fresh_world(directory, BASIC_FILES)
    earlier_patch = world.writer_patch('earlier-patch', {'crates/alpha/src/a.rs': numbered({1: 'edit near the top'})})
    later_patch = world.writer_patch('later-patch', {'crates/alpha/src/a.rs': numbered({1: 'edit near the top', 8: 'edit near the bottom'})})
    out = os.path.join(directory, 'out')
    code, text = run_tool(['--out', out, '--tree', world.tree, earlier_patch, later_patch])
    if code != 0:
        return f'应当退 0，实际退 {code}：{text.strip()[:300]}'
    got = replay(initial, os.path.join(out, PATCH_FILE_NAME), ['crates/alpha/src/a.rs'], directory, world)
    if got != {'crates/alpha/src/a.rs': world.read('crates/alpha/src/a.rs')}:
        return '现算的补丁套在最早的基线上应当得到此刻树里的 a.rs（两份补丁的改动都在）'
    return None


def cell_baseline_missing(directory):
    world, _ = fresh_world(directory, BASIC_FILES)
    patch = world.writer_patch('patch', {'crates/alpha/src/a.rs': numbered({2: 'edited 2'})})
    shutil.rmtree(os.path.join(patch, BASELINE_DIRECTORY_NAME))
    out = os.path.join(directory, 'out')
    code, text = run_tool(['--out', out, '--tree', world.tree, patch])
    if code != 2 or 'crates/alpha/src/a.rs' not in text or BASELINE_DIRECTORY_NAME not in text:
        return f'应当退 2、点名 baseline 里缺的 a.rs，实际退 {code}：{text.strip()[:300]}'
    if os.path.lexists(out) or leftovers(directory):
        return '不该留产物或临时目录'
    return None


def cell_tree_changes_after_the_diff(directory):
    world, _ = fresh_world(directory, BASIC_FILES)
    patch = world.writer_patch('patch', {'crates/alpha/src/a.rs': numbered({2: 'edited 2'})})
    out = os.path.join(directory, 'out')
    code, text = run_tool(['--out', out, '--tree', world.tree, patch], after_diff=lambda: world.write('crates/alpha/src/a.rs', numbered({2: 'edited 2', 3: 'changed meanwhile'})))
    if code != EXIT_REVERSE_CHECK_FAILED or 'git apply --check --reverse' not in text:
        return f'应当退 1、说反向核不过，实际退 {code}：{text.strip()[:300]}'
    if os.path.lexists(out) or leftovers(directory):
        return '不该留产物或临时目录'
    return None


def cell_everything_withdrawn(directory):
    world, _ = fresh_world(directory, BASIC_FILES)
    patch = world.writer_patch('patch', {'crates/alpha/src/a.rs': numbered({2: 'edited 2'})})
    world.write('crates/alpha/src/a.rs', numbered())
    out = os.path.join(directory, 'out')
    code, text = run_tool(['--out', out, '--tree', world.tree, patch])
    if code != EXIT_NOTHING_LEFT:
        return f'应当退 3，实际退 {code}：{text.strip()[:300]}'
    if os.path.lexists(out) or leftovers(directory):
        return '不该留产物或临时目录'
    return None


def cell_path_outside_the_tree(directory):
    world, _ = fresh_world(directory, BASIC_FILES)
    patch = os.path.join(directory, 'patch')
    os.makedirs(patch)
    with open(os.path.join(patch, PATCH_FILE_NAME), 'w', encoding='utf-8') as handle:
        handle.write('diff --git a/../escape.txt b/../escape.txt\nnew file mode 100644\n--- /dev/null\n+++ b/../escape.txt\n@@ -0,0 +1 @@\n+x\n')
    with open(os.path.join(directory, 'escape.txt'), 'w', encoding='utf-8') as handle:
        handle.write('x\n')
    out = os.path.join(directory, 'out')
    code, text = run_tool(['--out', out, '--tree', world.tree, patch])
    if code != 2 or '树外' not in text:
        return f'应当退 2、点名树外的路径，实际退 {code}：{text.strip()[:300]}'
    if os.path.lexists(out) or leftovers(directory) or read_text(os.path.join(directory, 'escape.txt')) != 'x\n':
        return '不该留产物、临时目录，也不该动树外的文件'
    return None


CELLS = (
    ('补丁全打着：现算的补丁与原补丁是同一份改动、反向核过', cell_patch_applied_everywhere),
    ('撤回一半：现算的补丁只含还在的那一半、反向核过', cell_half_withdrawn),
    ('三方合并打进去、主工作区另有别人的改动：现算的补丁含那些改动、反向核过', cell_merged_by_three_way),
    ('变异行被改过：只留此刻表里逐字还在的', cell_mutation_rows_edited),
    ('输出目录已在：拒、原目录一个字节不动', cell_output_directory_exists),
    ('两份补丁点到同一份文件：取最早那份的基线', cell_two_patches_share_a_file),
    ('点到的已有文件在基线里没有：拒、不留产物', cell_baseline_missing),
    ('算完补丁之后树又被改：反向核不过时退 1、不留产物', cell_tree_changes_after_the_diff),
    ('补丁全撤回：退 3、不留产物', cell_everything_withdrawn),
    ('补丁点到树外的路径：拒、树外一个字不写', cell_path_outside_the_tree),
)


def selftest():
    failures = 0
    scratch = tempfile.mkdtemp(prefix='patch-of-the-current-tree-selftest-')
    try:
        for name, cell in CELLS:
            directory = tempfile.mkdtemp(prefix='cell-', dir=scratch)
            try:
                problem = cell(directory)
            except Exception as error:  # noqa: BLE001 自证里任何异常都是这一格红
                problem = f'抛了异常 {type(error).__name__}：{error}'
            if problem is not None:
                failures += 1
                print(f'  ✗ 自检：{name}：{problem}')  # gate-lint:detail
        setting = os.environ.get(BREAK_VARIABLE, '')
        if failures:
            print(f'  共 {len(CELLS)} 格，红 {failures} 格{"（弄坏开关 " + BREAK_VARIABLE + "=" + setting + " 设着，红是它弄出来的）" if setting else ""}')
            print('     → 怎么办：看上面每一行「✗ 自检：<格名>：<问题>」，按那一格改 patch-of-the-current-tree.py，再跑一遍 --selftest；设着弄坏开关的红是预期')
            return 1
        print(f'  ✓ patch-of-the-current-tree.py 自证 {len(CELLS)} 格全绿（假仓与假补丁目录，只起了 git）')
        return 0
    finally:
        shutil.rmtree(scratch, ignore_errors=True)


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f'/proc/{pid}/cmdline', 'rb') as handle:
                if b'--selftest' in handle.read().split(b'\0'):
                    return True
            with open(f'/proc/{pid}/stat', 'rb') as handle:
                pid = int(handle.read().rsplit(b')', 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or arguments[:1] == ['--selftest'] or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/gate/patch-of-the-current-tree.py --selftest", file=sys.stderr)
    return 2


def main():
    if sys.argv[1:2] == ['--selftest']:
        sys.exit(selftest())
    sys.exit(run(sys.argv[1:]))


if __name__ == '__main__':
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    main()
