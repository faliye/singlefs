"""写回的「依据」不许指到别的会话还没提交的改动：核依据文字里点名的路径（与行号，若写了）。

给 kb-spec-check.py（规格的「依据」列）与 stale-candidates.py --check-facts（事实表的「出处」列）import；没有自己的自证，
两个调用方各带一组样本（临时 git 仓里造），各自的自证走一遍红与绿。

为什么要有这一道：几个会话共写一个仓时，并行写回员会把别的会话还没提交的改动当依据写进 kb
（登记的流写成 35 条而 HEAD 里是 34 条，某条欠账写成「已做完」而依据只在别人暂存的改动里），提交一回退或那个会话改主意，kb 里的现状就成了没出处的断言。
这一批自己新写的判决与证据（还没提交的新文件）照例是写回的依据，不拦。

判法（只判路径，不判路径里的内容）：依据文字里每个从仓根起、带至少一层目录、末段带后缀的路径，
  ① 登记在 `.claude/batch-scope`（这一批的改动范围，一行一条路径，路径之后是理由；常设登记，门禁 doc-process-records 的 batch-scope 格核）：放行。
     没有调用方单次放行的口子；
  ② 已跟踪、工作区与暂存区都和 HEAD 相同：放行；
  ③ 已跟踪、工作区或暂存区与 HEAD 不同（含暂存的新文件、工作区删掉的文件）、又不在 ①：判红；
     例外：写了行号的，那几行在 HEAD 那一版里存在、且工作区与暂存区那一版的同一行与 HEAD 逐字相同，放行（别人改的是文件里别的地方）；
  ④ 未跟踪（HEAD 与暂存区都没有）、工作区里有的新文件：放行，不判红，调用方在输出里另列「依据是未跟踪的新文件 N 份：<名单>」；
  ⑤ HEAD、暂存区与工作区里都没有的：不判（早已归档的产物、写错的路径各有别的检查管；这里只拦「依据是别人没提交的改动」）。
不判的写法：带 < > * { } $ 的占位与通配、URL、绝对路径与 ~ 开头的路径、没有目录层的裸文件名（只写文件名的引用当已归档的东西，照 `.claude/agent-common.md`「规则怎么读」一节）。
仓根不是 git 仓、HEAD 不存在时没法核：依据里有路径就判红（不静默放行）。
未显式放行的已跟踪文件带 assume-unchanged 或 skip-worktree 时也判红：Git 可能跳过工作区差异，先核对并清除标记再查。

弄坏开关：这份库没有；两个调用方各有「不核依据」的开关（KB_SPEC_CHECK_BREAK=basis-unchecked、STALE_CANDIDATES_BREAK=provenance-unchecked）。
"""
import os
import re
import subprocess

BATCH_SCOPE = ".claude/batch-scope"
# 仓根下的首级目录沿用 ASCII 形态，后续目录和文件名允许中文。
# 首级不吞掉紧邻的中文叙述；文件名止于 ASCII 扩展名，保留「scripts/x.py中的实现」里的路径。
# 前面不许紧挨 ASCII 路径字符、冒号、<>*{}~$（URL、绝对路径、占位的一段）。
PATH_WITH_LINES = re.compile(
    r"(?<![A-Za-z0-9_.\-/:<>*{}~$])([A-Za-z0-9_.\-]+/(?:[\w.\-]+/)*[\w\-][\w.\-]*\.[A-Za-z][A-Za-z0-9]{0,7})(?![A-Za-z0-9_\-<>*{}$/]|\.[\w.\-])"
    r"(?::(\d+)(?:[-–](\d+))?)?")
SUFFIX = re.compile(r"\.[A-Za-z][A-Za-z0-9]{0,7}$")


def paths_cited(text):
    """依据文字里点名的路径：[(路径, 起行号或 None, 止行号或 None)]，同一条（路径、行号）只列一次，按出现次序。"""
    found, seen = [], set()
    for match in PATH_WITH_LINES.finditer(text or ""):
        path = match.group(1).rstrip(".")
        if not SUFFIX.search(path):
            continue
        start = int(match.group(2)) if match.group(2) else None
        end = int(match.group(3)) if match.group(3) else None
        key = (path, start, end)
        if key not in seen:
            seen.add(key)
            found.append(key)
    return found


def batch_scope_paths(root):
    """.claude/batch-scope 里登记的路径（每个非注释行的第一段）；文件不在就是空集。"""
    scope = set()
    try:
        with open(os.path.join(root, BATCH_SCOPE), encoding="utf-8", errors="replace") as handle:
            for line in handle:
                line = line.strip()
                if line and not line.startswith("#"):
                    scope.add(line.split()[0])
    except OSError:
        pass
    return scope


def _split_lines(data):
    """按换行切成行（不带换行符）；空内容是零行。"""
    return data.split(b"\n")[:-1] + ([data.rsplit(b"\n", 1)[-1]] if data and not data.endswith(b"\n") else [])


class CommittedState:
    """仓里一条路径此刻的状态：HEAD 那一版、暂存区那一版、工作区那一版、有没有跟踪、和 HEAD 比有没有不同；按路径缓存。"""

    def __init__(self, root):
        self.root = root
        self.cache = {}
        probe = subprocess.run(["git", "-C", root, "rev-parse", "--verify", "-q", "HEAD^{commit}"], capture_output=True, text=True)
        self.usable = probe.returncode == 0

    def _git(self, *arguments):
        return subprocess.run(["git", "--literal-pathspecs", "-C", self.root, "-c", "core.quotepath=false", *arguments], capture_output=True,
                              env={**os.environ, "GIT_OPTIONAL_LOCKS": "0"})

    def head_lines(self, path):
        """HEAD 里这个文件的各行；HEAD 里没有它就是 None。"""
        key = ("head", path)
        if key not in self.cache:
            shown = self._git("show", f"HEAD:{path}")
            if shown.returncode != 0:
                listed = self._git("ls-tree", "-z", "HEAD", "--", path)
                if listed.returncode != 0 or listed.stdout:
                    raise RuntimeError(f'{path} 的 HEAD 内容读取失败，无法确认依据状态')
            self.cache[key] = None if shown.returncode != 0 else _split_lines(shown.stdout)
        return self.cache[key]

    def line_count(self, path):
        """HEAD 里这个文件的行数；HEAD 里没有它就是 None。"""
        lines = self.head_lines(path)
        return None if lines is None else len(lines)

    def index_lines(self, path):
        """暂存区里这个文件的各行；暂存区里没有它就是 None。"""
        key = ("index", path)
        if key not in self.cache:
            shown = self._git("show", f":{path}")
            if shown.returncode != 0 and self.tracked(path):
                raise RuntimeError(f'{path} 的暂存区内容读取失败，无法确认依据状态')
            self.cache[key] = None if shown.returncode != 0 else _split_lines(shown.stdout)
        return self.cache[key]

    def worktree_lines(self, path):
        """工作区里这个文件的各行；不是普通文件就是 None。"""
        full = os.path.join(self.root, path)
        if not os.path.isfile(full):
            return None
        with open(full, "rb") as handle:
            return _split_lines(handle.read())

    def tracked(self, path):
        """暂存区里有这个路径（已跟踪）。"""
        key = ("tracked", path)
        if key not in self.cache:
            listed = self._git("ls-files", "-z", "--", path)
            if listed.returncode != 0:
                raise RuntimeError(f'{path} 的 git ls-files 失败，无法确认依据状态')
            self.cache[key] = bool(listed.stdout)
        return self.cache[key]

    def differs(self, path):
        """工作区或暂存区的这个路径与 HEAD 不同：(工作区不同, 暂存区不同)。"""
        key = ("differs", path)
        if key not in self.cache:
            worktree = self._git("diff", "--quiet", "--no-ext-diff", "HEAD", "--", path)
            staged = self._git("diff", "--quiet", "--no-ext-diff", "--cached", "--", path)
            for result in (worktree, staged):
                if result.returncode not in (0, 1):
                    raise RuntimeError(f'{path} 的 git diff 失败（退出码 {result.returncode}），无法确认依据状态')
            flags = self._git("ls-files", "-v", "-z", "--", path)
            if flags.returncode != 0:
                raise RuntimeError(f'{path} 的 git ls-files 失败，无法确认依据状态')
            if any(entry[:1].islower() or entry[:1] == b'S' for entry in flags.stdout.split(b'\0') if entry):
                raise RuntimeError(f'{path} 设置了 assume-unchanged 或 skip-worktree，git diff 可能跳过工作区改动；'
                                   '先核对文件并清除相应标记，再检查依据状态')
            self.cache[key] = (worktree.returncode == 1, staged.returncode == 1)
        return self.cache[key]


def _lines_unchanged(committed, path, start, end):
    """写了行号的引用：那几行在 HEAD 里存在，且工作区与暂存区里与 HEAD 不同的那一版，同一行与 HEAD 逐字相同。
    与 HEAD 不同的那一版缺了（工作区删掉、暂存区删掉，读出来是 None）或行数不够，就拒（交 False），不跳过；与 HEAD 相同的那一版不用比。"""
    head = committed.head_lines(path)
    last = end if end is not None else start
    first = start
    if first < 1 or last < first or last > len(head):
        return False
    wanted = head[first - 1:last]
    worktree_differs, staged_differs = committed.differs(path)
    for differs, lines in ((worktree_differs, committed.worktree_lines(path)), (staged_differs, committed.index_lines(path))):
        if not differs:
            continue
        if lines is None or len(lines) < last or lines[first - 1:last] != wanted:
            return False
    return True


def basis_check(root, text, committed=None):
    """依据文字 text 里的路径：(问题说明表, 未跟踪的新文件表)。问题表空表示过；committed 可传 CommittedState 复用缓存。"""
    cited = paths_cited(text)
    if not cited:
        return [], []
    committed = committed or CommittedState(root)
    if not committed.usable:
        return [f"{root} 不是 git 仓或没有提交，核不了依据里的路径 {'、'.join(path for path, _, _ in cited[:5])} 在不在 HEAD 里"], []
    scope = batch_scope_paths(root)
    problems, untracked = [], []
    for path, start, end in cited:
        if path in scope:
            continue
        shown = f"{path}:{start}{'-' + str(end) if end is not None else ''}" if start is not None else path
        try:
            if committed.head_lines(path) is None:
                if committed.tracked(path):
                    problems.append(f"{shown} 是暂存区里的新文件（HEAD 里没有，别的会话暂存的改动；也没登记在 {BATCH_SCOPE}）")
                elif committed.worktree_lines(path) is not None and path not in untracked:
                    untracked.append(path)
                continue
            worktree_differs, staged_differs = committed.differs(path)
            if not (worktree_differs or staged_differs):
                continue
            where = "工作区与暂存区" if worktree_differs and staged_differs else ("工作区" if worktree_differs else "暂存区")
            if start is None:
                problems.append(f"{shown} 已跟踪，{where}那一版与 HEAD 不同（别的会话还没提交的改动，路径没登记在 {BATCH_SCOPE}）")
            elif not _lines_unchanged(committed, path, start, end):
                problems.append(f"{shown} 引的那几行在 {where}那一版里与 HEAD 不同、或 HEAD 那一版没有（HEAD 那一版有 {committed.line_count(path)} 行，"
                                f"路径没登记在 {BATCH_SCOPE}）")
        except (OSError, RuntimeError) as error:
            problems.append(f'{shown}：{error}')
    return problems, untracked


def uncommitted_problems(root, text, committed=None):
    """basis_check 的问题表（旧入口，不要未跟踪名单的调用方用）。"""
    return basis_check(root, text, committed)[0]


def untracked_note(untracked):
    """未跟踪的新文件名单写成输出里另列的一行；名单空就是 None。"""
    if not untracked:
        return None
    return f"依据是未跟踪的新文件 {len(untracked)} 份：{'、'.join(untracked)}"


ADVICE = ("引 HEAD 里已有的版本（`git show HEAD:<路径>` 读到的），或这一批自己新写、还没跟踪的文件；"
          "这份改动属于这一批的，把路径登记进 .claude/batch-scope（触发文件才行）；"
          "别的会话还没提交的改动不当依据，等它提交再写")
