#!/usr/bin/env python3
# admission: always 每次都拿此刻的 git 对象库与快照目录现铺一棵树，上一次铺出的树不替这一次作保
# run-condition: command git
"""三方开工快照倒推成一棵树：判决里的 cite-check 拿它当 --root。

用法：
    snapshot-tree.py <轮名> --out <目录> [--repository <仓根>] [--list <清单文件名> …]
    snapshot-tree.py --selftest       # SNAPSHOT_TREE_BREAK=<项> 时必须判红
    snapshot-tree.py --help           # 打这一段用法（-h 同），退 0

读 <仓根>/research/prompts/<轮名>-snapshot/ 下的：
    清单：默认读快照目录里全部 *-sha256.txt（judged-sha256.txt、kb-sha256.txt、crates-sha256.txt……，按文件名排；sha256sum 的原样输出，每行「<sha256>  <路径>」；
          records/2026-10-23-subagent拆分提案.md 第一百二十二节 ①：只读 judged 那一份时 kb 清单不进树、也不报倒推不出），一份都没有算用法错；
          给了 --list 就只读点名的那几份（同一目录下的文件名，例 --list judged-sha256.txt --list kb-sha256.txt）。同一份文件在几份清单里 sha256 不一样算用法错。
    diff：judged-vs-head.diff（对 HEAD）或 judged-vs-index.diff（旧轮，对暂存区），两种都认；两份都在算用法错，一份都没有就每份都取此刻 HEAD。
铺法：只铺清单点名的文件，逐份：
    diff 里有它：基底取 diff 头 `index <基底>..<结果>` 记的基底 blob——全 0 是新文件、基底为空；此刻 HEAD 那一份正是这个 blob 的记「HEAD」，
        不是的（快照之后有提交改过它）照 blob 号在对象库里取，取不到就报「倒推不出」；改名的（rename from）基底按旧路径取。
        基底铺进一个单独的临时目录，用 git apply 打上 diff 里这一份的那一段，打不上报「diff 打不上」。
    diff 里没有它：取此刻 HEAD 那一份；HEAD 里没有，或 HEAD 那一份的 sha256 不是快照那一版（快照之后有提交改过、diff 里又没有它的基底），报「倒推不出」。
    倒推出的内容逐份算 sha256 与清单比，对不上的点名；对不上的与倒推不出的都不铺进树。
    搬迁换回（.claude/rules/path-moves.md 第 6 步：只差路径的不算改动）：快照之后有登记过的搬迁（<仓根>/.claude/gate.d/path-moves.tsv），搬迁那一趟把快照目录里的
        diff 与清单也换成了新路径，基底 blob 里却还是旧路径，上下文对不上、sha256 也对不上。照原样倒推不出或对不上时，再照登记反着换一遍
        （每一行新旧对调、次序倒排，换法就是 .claude/gate.d/lib-lag-baseline.py 的 rewrite_moved_paths，只按路径前缀的边界换）：
        diff 里有它的，这一份的那一段整段换回旧路径（逐行先去掉行首的 + - 空格记号再换，「-tools/…」才换得到）再取基底、再打；diff 里没有它的，此刻 HEAD 那一份换回旧路径再比；清单点名的路径在 diff 与 HEAD 里都找不到的，
        照登记往前、往后换一次路径再找。换回之后 sha256 对上的才铺进树（铺在清单点名的路径上），说明里写「照搬迁登记换回旧路径」与登记的行号；照原样已经对上的不换。
        登记表读不了、写坏了、lib-lag-baseline.py 载不进的，打一行提示、照原样倒推。
    两种最后的取法（fallback_content；照原样与照搬迁登记都倒推不出或对不上之后才试，先后次序如下）：
        diff 头记的结果 blob（index 行 .. 后面那个）还在对象库里、sha256 与快照相同的，取它（快照之后它被提交过、暂存过，或 diff 改坏了而结果还在）；
        此刻工作区那一份（仓根下清单点名的路径）sha256 与快照相同的，取它（快照那一刻暂存了、还没提交的新文件，对暂存区的 diff 里没有它）。
        这两种逐字节比 sha256，比不上的照原样报；取到的铺进树，说明里写取的是哪一种、照原样为什么不行；汇总行写「对上的里有 N 份取 diff 头记的结果 blob、N 份取工作区此刻那一份」。
--out 要么不在、要么是空目录，不许落在仓里（放派发提示给的草稿目录）。成功时另打一行判决里抄的 cite-check 命令。
退出码：0 点名的全部倒推出、sha256 全对上；1 有倒推不出或对不上的（对上的那几份照铺）；2 用法错（快照目录不在、清单读不了、两份 diff 都在、--out 不空或在仓里）。
和 scripts/devices/snapshot-check.sh 的分工：它只核此刻的工作区与 HEAD 还是不是快照那一版，不铺树；这一份按 diff 倒推出快照那一刻的内容。

弄坏开关 SNAPSHOT_TREE_BREAK 只在自证（--selftest）里认：平常跑设了它，不管值是什么，参数解析之前就退 2、点名它只在自证里认。几项可以逗号连写。
逐项写开关名、弄坏哪一处，下一行起写打开后自证判红的项（「」里是那条失败行里的原文）：
  SNAPSHOT_TREE_BREAK=skip-sha-check  lay_file() 不比 sha256，倒推出什么都当对上
      「✗ 自检：改坏的 diff：应当退 1」「✗ 自检：改坏的 diff：对不上的 kb/one.md 不该铺进树」「✗ 自检：diff 里没有、快照之后被提交改过：应当退 1」
      「✗ 自检：搬迁之后的旧轮：应当退 0」
  SNAPSHOT_TREE_BREAK=head-only  base_for() 基底一律取此刻 HEAD，不照 diff 头记的 blob 号取
      「✗ 自检：快照之后有提交：应当退 0」「✗ 自检：树里的 kb/one.md」
      「✗ 自检：改坏的 diff：应当退 1」「✗ 自检：基底 blob 取不到：应当退 1」「✗ 自检：对暂存区的旧轮：应当退 0」「✗ 自检：diff 里没有、快照之后被提交改过：应当退 1」
      「✗ 自检：搬迁之后的旧轮：应当退 0」「✗ 自检：搬迁之后的旧轮：树里的 tools/new-name/run.sh」（失败行里另带那一趟的输出：「✗ tools/new-name/run.sh  diff 打不上」「✗ kb/guide.md  diff 打不上」「✗ 快照 r5：清单点名 4 份」）
      「✗ kb/one.md  diff 打不上」「✗ 快照 r1：清单点名 7 份，倒推出 6 份」「✗ 快照 r2：清单点名 7 份，倒推出 6 份」「✗ 快照 r3：清单点名 7 份，倒推出 6 份」「✗ kb/staged.md  diff 打不上」「✗ 快照 r4：清单点名 1 份，倒推出 0 份」「✗ kb/later.md  倒推不出」「✗ 快照 r1：清单点名 7 份，倒推出 5 份」
  SNAPSHOT_TREE_BREAK=no-unquote  unquoted() 不解 git 的引号路径（\\ooo 八进制字节），只剥掉两边的引号；非 ASCII 文件名的那一份对不到 diff 里那一段
      「✗ 自检：快照之后有提交：应当退 0」
  SNAPSHOT_TREE_BREAK=index-diff-ignored  diff_path() 只认 judged-vs-head.diff，不认 judged-vs-index.diff
      「✗ 自检：对暂存区的旧轮：应当退 0」「✗ 自检：两份 diff 都在：应当退 2」
      「✗ kb/staged.md  倒推不出」「✗ 快照 r4：清单点名 1 份，倒推出 0 份」
  SNAPSHOT_TREE_BREAK=path-moves-ignored  build_tree() 不读搬迁登记表，diff、HEAD 与清单路径都不照登记换回旧路径（改前的样子）
      「✗ 自检：搬迁之后的旧轮：应当退 0」「✗ 自检：搬迁之后的旧轮：树里的 tools/new-name/run.sh」
      （失败行里另带那一趟的输出：「✗ tools/new-name/run.sh  diff 打不上」「✗ kb/guide.md  diff 打不上」「✗ kb/plain.md  倒推不出：diff 里没有它」
      「✗ 快照 r5：清单点名 4 份，倒推出 1 份」）
  SNAPSHOT_TREE_BREAK=diff-markers-kept  MovesBack.diff_text() 整段照换、不先去掉行首的 diff 记号（「-tools/…」的路径紧挨着 - 号，换不到）
      「✗ 自检：搬迁之后的旧轮：应当退 0」
  SNAPSHOT_TREE_BREAK=result-blob-ignored  fallback_content() 不取 diff 头记的结果 blob
      「✗ 自检：结果 blob 还在对象库里：应当退 0」（失败行里另带那一趟的输出：「✗ kb/result-blob.md  倒推不出」「✗ 快照 r6：清单点名 1 份」）
  SNAPSHOT_TREE_BREAK=worktree-ignored  fallback_content() 不取工作区此刻那一份
      「✗ 自检：工作区那一份对得上：应当退 1」（失败行里另带那一趟的输出：「✗ kb/worktree-only.md  倒推不出」「✗ kb/worktree-changed.md  倒推不出」「✗ 快照 r7：清单点名 2 份」）
  SNAPSHOT_TREE_BREAK=guard-off  refuse_break_outside_selftest() 不拒，平常跑设了开关照跑（关守卫项：只在它是唯一一项时认，与别的项同写照拒并点名它）
      「✗ 自检：平常跑设了弄坏开关：应当退 2 并点名 SNAPSHOT_TREE_BREAK 只在自证里认」
  SNAPSHOT_TREE_BREAK=guard-off-with-others  refuse_break_outside_selftest() 认同写在别的项旁边的 guard-off（改前的样子：「guard-off,skip-sha-check」平常跑照样执行）；
      只在自证进程里认（平常跑同写着它照拒），自证在进程里探同写那一格
      「✗ 自检：关守卫项与别的项同写：应当退 2 并点名 guard-off 只在它是唯一一项时认」
  SNAPSHOT_TREE_BREAK=judged-list-only  run() 不给 --list 时只读 judged-sha256.txt（改前的样子），快照目录里别的 *-sha256.txt 不进树也不报
      「✗ 自检：带两份清单的快照：应当退 0」「✗ 自检：快照目录里一份清单都没有：应当退 2」（失败行里另带那一趟的输出：「✗ 清单读不了」）
"""
import hashlib
import os
import re
import shutil
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402
BREAK_VARIABLE = "SNAPSHOT_TREE_BREAK"
BROKEN = set(item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item)

DEFAULT_LIST = "judged-sha256.txt"   # 自证造快照时写的那一份；不给 --list 时读快照目录里全部 LIST_SUFFIX 结尾的（default_list_names）
LIST_SUFFIX = "-sha256.txt"
GUARD_OFF_ITEM = "guard-off"                          # 关守卫项：只在它是唯一一项时认（第一百二十二节 ②）
GUARD_OFF_WITH_OTHERS_ITEM = "guard-off-with-others"  # 它的弄坏开关：同写也认（改前的样子），只在自证进程里认
HEAD_DIFF = "judged-vs-head.diff"
INDEX_DIFF = "judged-vs-index.diff"
LIST_LINE = re.compile(r"^([0-9a-fA-F]{64}) [ *](.+)$")
INDEX_LINE = re.compile(r"^index ([0-9a-f]+)\.\.([0-9a-f]+)(?: (\d+))?$")
ESCAPES = {"n": b"\n", "t": b"\t", '"': b'"', "\\": b"\\", "a": b"\a", "b": b"\b", "f": b"\f", "r": b"\r", "v": b"\v"}
GIT_ENVIRONMENT_DROPPED = ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX", "GIT_COMMON_DIR", "GIT_OBJECT_DIRECTORY")
LAG_BASELINE_LIBRARY = os.path.join(REPOSITORY_ROOT, ".claude", "gate.d", "lib-lag-baseline.py")   # 搬迁登记的读法与换法只在这一份（path_moves、rewrite_moved_paths）


class MovesBack:
    """照搬迁登记反着换：登记表每一行新旧对调、次序倒排，换法交 lib-lag-baseline.py 的 rewrite_moved_paths（只按路径前缀的边界换）。"""

    def __init__(self, library, moves):
        self.library, self.moves = library, moves
        self.backwards = [library.PathMove(move.line, move.new, move.old, move.date, move.source) for move in reversed(moves)]

    def text(self, text):
        return self.library.rewrite_moved_paths(text, self.backwards)

    def diff_text(self, text):
        """diff 的一段逐行换：行首的 + - 空格是 diff 的记号，不算路径前面连着的字（rewrite_moved_paths 认前缀边界时「-scripts/…」不当路径开头），去掉记号再换。"""
        if "diff-markers-kept" in BROKEN:
            return self.text(text)
        return "".join(line[:1] + self.text(line[1:]) if line[:1] in ("+", "-", " ") else self.text(line) for line in text.splitlines(keepends=True))

    def data(self, data):
        return self.library.rewrite_moved_bytes(data, self.backwards)

    def lines_touching(self, text):
        """text 里（逐行去掉 diff 的 + - 空格记号）出现了新前缀的那几行登记的行号（说明里点名）。"""
        bodies = "\n".join(line[1:] if line[:1] in ("+", "-", " ") else line for line in text.splitlines())
        return [move.line for move in self.moves if self.library.moved_prefix_pattern(move.new).search(bodies)]

    def other_paths(self, path):
        """清单点名的路径照登记往前（换回旧前缀）、往后（换成新前缀）各换一次，去掉原样那一条。"""
        candidates = [self.text(path), self.library.rewrite_moved_paths(path, self.moves)]
        return [candidate for index, candidate in enumerate(candidates) if candidate != path and candidate not in candidates[:index]]


def moves_back_for(repository):
    """→ MovesBack 或 None：<repository>/.claude/gate.d/path-moves.tsv 登记的搬迁（读法照 lib-lag-baseline.py 的 path_moves）。
    没有登记、读不了、写坏了、那份库载不进的交 None（后两种打一行提示）；弄坏开关 path-moves-ignored 下交 None。"""
    if "path-moves-ignored" in BROKEN:
        return None
    try:
        import importlib.util
        specification = importlib.util.spec_from_file_location("lib_lag_baseline_for_snapshot_tree", LAG_BASELINE_LIBRARY)
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
        moves, problems = library.path_moves(cwd=repository)
    except Exception as error:   # 文件不在、语法错、接口对不上
        print(f"  ! 载不进 {LAG_BASELINE_LIBRARY}（{type(error).__name__}：{error}）：这一次不照搬迁登记换回旧路径，照原样倒推")
        return None
    if problems:
        print(f"  ! 搬迁登记表 {library.PATH_MOVES_TABLE} 写得不对（{'；'.join(problems)[:200]}）：这一次不照它换回旧路径，照原样倒推")
        return None
    return MovesBack(library, moves) if moves else None


class UsageError(Exception):
    pass


def refuse_break_outside_selftest(arguments, value=None):
    """弄坏开关只在自证里认：命令行不是单独一个 --selftest 而变量非空，交回退出码 2（参数解析之前判，不管值是什么）；否则交 None。
    关守卫项 guard-off 单独一项时不拒；与别的项同写照拒、点名它（弄坏开关 guard-off-with-others 在自证进程里设着时照认，改前的样子）。
    value 给了就判它、不读环境（自证在进程里探同写那一格用）。"""
    value = os.environ.get(BREAK_VARIABLE, "") if value is None else value
    if not value or arguments == ["--selftest"]:
        return None
    items = [item for item in value.split(",") if item]
    if items == [GUARD_OFF_ITEM]:
        return None
    if GUARD_OFF_ITEM in items:
        if GUARD_OFF_WITH_OTHERS_ITEM in BROKEN and sys.argv[1:] == ["--selftest"]:   # 弄坏开关：只在自证进程里认
            return None
        others = [item for item in items if item != GUARD_OFF_ITEM]
        print(f"  ✗ {BREAK_VARIABLE}={value}：关守卫项 {GUARD_OFF_ITEM} 只在它是唯一一项时认，与别的项（{'、'.join(others)}）同写照拒；这一次什么都没做")
        print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看某一项红在哪一格，跑 {BREAK_VARIABLE}=<项> python3 scripts/agents/snapshot-tree.py --selftest，一次只设那一项")
        return 2
    print(f"  ✗ {BREAK_VARIABLE}={value} 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次什么都没做")
    print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 {BREAK_VARIABLE}=<项> python3 scripts/agents/snapshot-tree.py --selftest")
    return 2


def unquoted(text):
    """git 的 C 风格引号路径（core.quotepath）还原成原来的路径；没加引号的原样返回。"""
    if len(text) < 2 or not (text.startswith('"') and text.endswith('"')):
        return text
    body = text[1:-1]
    if "no-unquote" in BROKEN:
        return body
    out = bytearray()
    index = 0
    while index < len(body):
        character = body[index]
        if character == "\\" and index + 1 < len(body):
            following = body[index + 1]
            if following in "01234567":
                out.append(int(body[index + 1:index + 4], 8))
                index += 4
                continue
            out += ESCAPES.get(following, following.encode("utf-8"))
            index += 2
            continue
        out += character.encode("utf-8")
        index += 1
    return out.decode("utf-8", "surrogateescape")


def without_prefix(path, prefix):
    return path[len(prefix):] if path is not None and path.startswith(prefix) else path


def header_paths(header):
    """`diff --git <a> <b>` 那一行里的两个路径（去掉 a/、b/）。"""
    if header.startswith('"'):
        end = 1
        while end < len(header) and not (header[end] == '"' and header[end - 1] != "\\"):
            end += 1
        first, rest = header[:end + 1], header[end + 2:]
        return without_prefix(unquoted(first), "a/"), without_prefix(unquoted(rest), "b/")
    if header.endswith('"'):
        start = header.rfind(' "')
        return without_prefix(header[:start], "a/"), without_prefix(unquoted(header[start + 1:]), "b/")
    for position in [match.start() for match in re.finditer(" b/", header)]:
        first, second = header[:position], header[position + 1:]
        if without_prefix(first, "a/") == without_prefix(second, "b/"):
            return without_prefix(first, "a/"), without_prefix(second, "b/")
    position = header.find(" b/")
    return without_prefix(header[:position], "a/"), without_prefix(header[position + 1:], "b/")


def diff_sections(text):
    """→ {新路径: 这一份的信息}；信息里有 old、new、old_blob、new_blob、mode、new_file、deleted、binary、text。"""
    pieces, current = [], None
    for line in text.splitlines(keepends=True):
        if line.startswith("diff --git "):
            current = [line]
            pieces.append(current)
        elif current is not None:
            current.append(line)
    sections = {}
    for piece in pieces:
        header_old, header_new = header_paths(piece[0][len("diff --git "):].rstrip("\n"))
        info = {"old": header_old, "new": header_new, "old_blob": None, "new_blob": None, "mode": None, "old_mode": None,
                "new_file": False, "deleted": False, "binary": False, "text": "".join(piece)}
        for line in piece[1:]:
            line = line.rstrip("\n")
            if line.startswith("@@"):
                break
            match = INDEX_LINE.match(line)
            if match:
                info["old_blob"], info["new_blob"] = match.group(1), match.group(2)
                info["mode"] = match.group(3) or info["mode"]
                info["old_mode"] = info["old_mode"] or match.group(3)
            elif line.startswith("new file mode "):
                info["new_file"], info["mode"] = True, line.split()[-1]
            elif line.startswith("deleted file mode "):
                info["deleted"] = True
            elif line.startswith("old mode "):
                info["old_mode"] = line.split()[-1]
            elif line.startswith("new mode "):
                info["mode"] = line.split()[-1]
            elif line.startswith(("rename from ", "copy from ")):
                info["old"] = unquoted(line.split(" ", 2)[2])
            elif line.startswith(("rename to ", "copy to ")):
                info["new"] = unquoted(line.split(" ", 2)[2])
            elif line.startswith("--- "):
                value = line[4:].rstrip("\t")
                if value != "/dev/null":
                    info["old"] = without_prefix(unquoted(value), "a/")
            elif line.startswith("+++ "):
                value = line[4:].rstrip("\t")
                if value != "/dev/null":
                    info["new"] = without_prefix(unquoted(value), "b/")
            elif line.startswith(("Binary files ", "GIT binary patch")):
                info["binary"] = True
        sections[info["new"]] = info
    return sections


def git(repository, *arguments):
    environment = {key: value for key, value in os.environ.items() if key not in GIT_ENVIRONMENT_DROPPED}
    return subprocess.run(["git", "-C", repository, *arguments], capture_output=True, env=environment)


def head_blob(repository, path):
    result = git(repository, "rev-parse", "--verify", "--quiet", f"HEAD:{path}")
    return result.stdout.decode().strip() if result.returncode == 0 else None


def restored_mode(repository, path, info=None, worktree=False):
    """diff 有模式就用它；否则从内容来源的 HEAD 树项或工作区取执行位。"""
    if info is not None and info["mode"]:
        return info["mode"]
    if worktree:
        return "100755" if os.stat(os.path.join(repository, path)).st_mode & 0o111 else "100644"
    for candidate in dict.fromkeys([path] + ([info["old"]] if info is not None else [])):
        result = git(repository, "ls-tree", "-z", "HEAD", "--", ":(literal)" + candidate)
        for entry in result.stdout.split(b"\0"):
            if b"\t" in entry:
                metadata, name = entry.split(b"\t", 1)
                if os.fsdecode(name) == candidate:
                    return metadata.split()[0].decode("ascii")
    return None


def checked_relative_path(path):
    """清单与 diff 的路径必须在树内；在建输出目录或临时补丁树之前检查。"""
    if os.path.isabs(path) or ".." in path.split("/") or os.path.normpath(path) in ("", "."):
        raise UsageError(f"路径必须是树内的相对文件路径（不含 ..）：{path}")
    return os.path.normpath(path)


def blob_by_abbreviation(repository, abbreviation):
    result = git(repository, "rev-parse", "--verify", "--quiet", f"{abbreviation}^{{blob}}")
    return result.stdout.decode().strip() if result.returncode == 0 else None


def blob_bytes(repository, blob):
    result = git(repository, "cat-file", "blob", blob)
    return result.stdout if result.returncode == 0 else None


def base_for(repository, info):
    """→ (基底字节或 None, 来源说明, 问题或 None)。"""
    if info["new_file"]:
        return None, "新文件（diff 头的基底全 0）", None
    old_path = info["old"]
    current = head_blob(repository, old_path)
    abbreviation = info["old_blob"]
    if "head-only" in BROKEN or not abbreviation:
        if current is None:
            return None, "", f"倒推不出：diff 头没有基底 blob 号，此刻 HEAD 里也没有 {old_path}"
        return blob_bytes(repository, current), "HEAD", None
    if set(abbreviation) == {"0"}:
        return None, "新文件（diff 头的基底全 0）", None
    if current is not None and current.startswith(abbreviation):
        return blob_bytes(repository, current), "HEAD", None
    full = blob_by_abbreviation(repository, abbreviation)
    content = blob_bytes(repository, full) if full else None
    if content is None:
        return None, "", f"倒推不出：diff 头记的基底 blob {abbreviation} 在对象库里取不到（此刻 HEAD 那一份也不是它）"
    return content, f"diff 头记的基底 blob {abbreviation}（此刻 HEAD 那一份不是它：快照之后有提交改过）", None


def applied(info, base, work_parent):
    """基底铺进单独的临时目录，打上这一段 → (结果字节或 None, 问题或 None)。"""
    checked_relative_path(info["old"])
    checked_relative_path(info["new"])
    scratch = os.path.realpath(tempfile.mkdtemp(prefix=".snapshot-tree-apply-", dir=work_parent))
    try:
        tree = os.path.join(scratch, "tree")
        os.makedirs(tree)
        if base is not None:
            base_path = os.path.join(tree, info["old"])
            os.makedirs(os.path.dirname(base_path), exist_ok=True)
            with open(base_path, "wb") as handle:
                handle.write(base)
            if info["old_mode"] and info["old_mode"].endswith("755"):
                os.chmod(base_path, 0o755)   # diff 头记的是 100755 的，基底带上执行位，git apply 不再报类型不符
        patch = os.path.join(scratch, "section.diff")
        with open(patch, "w", encoding="utf-8", errors="surrogateescape") as handle:
            handle.write(info["text"])
        environment = {key: value for key, value in os.environ.items() if key not in GIT_ENVIRONMENT_DROPPED}
        environment["GIT_CEILING_DIRECTORIES"] = scratch
        result = subprocess.run(["git", "apply", "--whitespace=nowarn", patch], cwd=tree, capture_output=True, env=environment)
        if result.returncode != 0:
            reason = (result.stderr.decode("utf-8", "replace").strip().splitlines() or ["没有输出"])[0]
            return None, f"diff 打不上：git apply 退 {result.returncode}（{reason}）"
        result_path = os.path.join(tree, info["new"])
        if not os.path.isfile(result_path):
            return None, "diff 打完没有这份文件"
        with open(result_path, "rb") as handle:
            return handle.read(), None
    finally:
        shutil.rmtree(scratch, ignore_errors=True)


def blob_id(content):
    return hashlib.sha1(b"blob %d\0" % len(content) + content).hexdigest()


def section_content(repository, info, work_parent):
    """diff 里这一份倒推出的内容 → (内容或 None, 问题或 None, 基底来源, 来源类别)。"""
    if info["deleted"]:
        return None, "倒推不出：diff 里记的是删掉了这份，快照清单却点了它", "", None
    if info["binary"]:
        return None, "倒推不出：diff 里这一份是二进制、没带补丁数据", "", None
    base, source, problem = base_for(repository, info)
    if problem:
        return None, problem, "", None
    content, problem = applied(info, base, work_parent)
    if problem:
        return None, f"{problem}；基底取自{source}", source, None
    category = "new" if info["new_file"] or (info["old_blob"] and set(info["old_blob"]) == {"0"}) else ("head" if source == "HEAD" else "blob")
    return content, None, source, category


def digest_matches(content, digest):
    return content is not None and (hashlib.sha256(content).hexdigest() == digest.lower() or "skip-sha-check" in BROKEN)


def mismatch_explanation(info, content, digest, source):
    actual = hashlib.sha256(content).hexdigest()
    same_as_recorded = info["new_blob"] and blob_id(content).startswith(info["new_blob"])
    hint = ("倒推出的正是 diff 头记的结果 blob：diff 与清单不是同一刻取的" if same_as_recorded
            else f"倒推出的也不是 diff 头记的结果 blob {info['new_blob']}：diff 改坏了，或基底取错了")
    return f"sha256 对不上：倒推出 {actual[:12]}、快照 {digest[:12]}（{hint}；基底取自{source}）"


def moved_note(moves_back, text):
    lines = moves_back.lines_touching(text)
    return f"；照搬迁登记换回旧路径（.claude/gate.d/path-moves.tsv 第 {'、'.join(str(line) for line in lines)} 行，新前缀换回旧前缀）"


def fallback_content(repository, path, digest, info):
    """照原样与照搬迁登记都倒推不出或对不上之后的两种取法，先后试：diff 头记的结果 blob（index 行 .. 后面那个）还在对象库里、sha256 与快照相同的，取它；
    此刻工作区那一份（仓根下清单点名的路径，普通文件）sha256 与快照相同的，取它。→ (内容, 来源说明, 类别) 或 None。
    这里逐字节比 sha256，不看弄坏开关 skip-sha-check；弄坏开关 result-blob-ignored、worktree-ignored 各关掉一种。"""
    def exact(content):
        return content is not None and hashlib.sha256(content).hexdigest() == digest.lower()
    result = info.get("new_blob") if info is not None and not info["deleted"] else None
    if result and set(result) != {"0"} and "result-blob-ignored" not in BROKEN:
        full = blob_by_abbreviation(repository, result)
        content = blob_bytes(repository, full) if full else None
        if exact(content):
            return content, f"diff 头记的结果 blob {result}（照 diff 倒推不出或对不上；这个 blob 还在对象库里，sha256 与快照相同）", "result", restored_mode(repository, path, info)
    if "worktree-ignored" not in BROKEN:
        candidate = os.path.join(repository, path)
        if os.path.isfile(candidate) and not os.path.islink(candidate):
            with open(candidate, "rb") as handle:
                content = handle.read()
            if exact(content):
                return content, "工作区此刻那一份（照 diff 与 HEAD 都倒推不出或对不上；此刻工作区那一份 sha256 与快照相同）", "worktree", restored_mode(repository, path, info, worktree=True)
    return None


def failed_or_fallback(verdict, explanation, repository, path, digest, info, out):
    """lay_file 判下来倒推不出或对不上的：交 fallback_content 再试一次，取到了就铺进树、交 ok，说明里带上原来为什么不行；取不到照原样交回。"""
    recovered = fallback_content(repository, path, digest, info)
    if recovered is None:
        return verdict, explanation, None
    content, source, category, mode = recovered
    target = os.path.join(out, path)
    os.makedirs(os.path.dirname(target), exist_ok=True)
    with open(target, "wb") as handle:
        handle.write(content)
    if mode and mode.endswith("755"):
        os.chmod(target, 0o755)
    return "ok", f"来源：{source}  sha256 {digest[:12]}；照原样：{explanation}", category


def lay_file(repository, path, digest, sections, out, work_parent, moves_back=None):
    """→ (结论 ok / unrecovered / mismatched, 这一行的说明, 来源类别)。照原样倒推不出或对不上、又有搬迁登记（moves_back）的，照登记换回旧路径再倒推一次。"""
    info = sections.get(path)
    if info is None and moves_back is not None:
        info = next((sections[other] for other in moves_back.other_paths(path) if other in sections), None)
    note = ""
    if info is not None:
        content, problem, source, category = section_content(repository, info, work_parent)
        if not digest_matches(content, digest) and moves_back is not None:
            text_back = moves_back.diff_text(info["text"])
            if text_back != info["text"]:
                info_back = next(iter(diff_sections(text_back).values()))
                content_back, problem_back, source_back, category_back = section_content(repository, info_back, work_parent)
                if digest_matches(content_back, digest):
                    note = moved_note(moves_back, info["text"])
                    info, content, problem, source, category = info_back, content_back, None, source_back, category_back
                else:
                    failed_back = problem_back or mismatch_explanation(info_back, content_back, digest, source_back)
                    note = f"；照搬迁登记换回旧路径之后也不行：{failed_back}"
        if problem:
            return failed_or_fallback("unrecovered", problem + note, repository, path, digest, info, out)
        if not digest_matches(content, digest):
            return failed_or_fallback("mismatched", mismatch_explanation(info, content, digest, source) + note, repository, path, digest, info, out)
        mode = restored_mode(repository, path, info)
    else:
        candidates = [path] + (moves_back.other_paths(path) if moves_back is not None else [])
        source_path, current = next(((candidate, blob) for candidate in candidates
                                     if (blob := head_blob(repository, candidate)) is not None), (path, None))
        if current is None:
            return failed_or_fallback("unrecovered", "倒推不出：diff 里没有它，此刻 HEAD 里也没有" + ("（照搬迁登记换过的路径也没有）" if len(candidates) > 1 else ""),
                                      repository, path, digest, None, out)
        content, source, category = blob_bytes(repository, current), "HEAD（diff 里没有它）", "head"
        mode = restored_mode(repository, source_path)
        if not digest_matches(content, digest) and moves_back is not None and digest_matches(moves_back.data(content), digest):
            note = moved_note(moves_back, content.decode("utf-8", "surrogateescape"))
            content = moves_back.data(content)
        if not digest_matches(content, digest):
            actual = hashlib.sha256(content).hexdigest()
            return failed_or_fallback("unrecovered", (f"倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 {actual[:12]} 不是快照的 {digest[:12]}"
                                                      "（快照之后有提交改过它，diff 里没有它的基底"
                                                      + ("；照搬迁登记换回旧路径也对不上" if moves_back is not None else "") + "）"), repository, path, digest, None, out)
    target = os.path.join(out, path)
    os.makedirs(os.path.dirname(target), exist_ok=True)
    with open(target, "wb") as handle:
        handle.write(content)
    if mode and mode.endswith("755"):
        os.chmod(target, 0o755)
    return "ok", f"来源：{source}  sha256 {digest[:12]}{note}", category


def read_list(path):
    entries = []
    try:
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            for line in handle:
                match = LIST_LINE.match(line.rstrip("\n"))
                if match:
                    entries.append((checked_relative_path(match.group(2)), match.group(1).lower()))
                elif line.strip() and not line.startswith("#"):
                    raise UsageError(f"清单 {path} 有无法识别的记录：{line.rstrip()!r}")
    except OSError as error:
        raise UsageError(f"清单读不了：{path}（{error.strerror}）")
    return entries


def diff_path(snapshot):
    head, index = os.path.join(snapshot, HEAD_DIFF), os.path.join(snapshot, INDEX_DIFF)
    if "index-diff-ignored" in BROKEN:
        return head if os.path.isfile(head) else None
    if os.path.isfile(head) and os.path.isfile(index):
        raise UsageError(f"快照目录里 {HEAD_DIFF} 与 {INDEX_DIFF} 都在，认不出这一轮的基底是 HEAD 还是暂存区")
    return head if os.path.isfile(head) else (index if os.path.isfile(index) else None)


def default_list_names(snapshot):
    """不给 --list 时读的清单：快照目录里全部 *-sha256.txt（按文件名排）；弄坏开关 judged-list-only 下只读 judged-sha256.txt（改前的样子）。一份都没有抛 UsageError。"""
    if "judged-list-only" in BROKEN:
        return [DEFAULT_LIST]
    names = sorted(name for name in os.listdir(snapshot) if name.endswith(LIST_SUFFIX) and os.path.isfile(os.path.join(snapshot, name)))
    if not names:
        raise UsageError(f"快照目录里没有 *{LIST_SUFFIX} 清单：{snapshot}")
    return names


def build_tree(round_name, out, repository, list_names):
    """→ 退出码；逐份与汇总打到 stdout。list_names 空的读快照目录里全部 *-sha256.txt（default_list_names）。"""
    snapshot = os.path.join(repository, "research", "prompts", f"{round_name}-snapshot")
    if not os.path.isdir(snapshot):
        raise UsageError(f"快照目录不在：{snapshot}")
    defaulted = not list_names
    if defaulted:
        list_names = default_list_names(snapshot)
    real_out, real_repository = os.path.realpath(out), os.path.realpath(repository)
    if real_out == real_repository or real_out.startswith(real_repository + os.sep):
        raise UsageError(f"--out 落在仓里：{out}")
    if os.path.exists(out) and (not os.path.isdir(out) or os.listdir(out)):
        raise UsageError(f"--out 已存在且不是空目录：{out}")
    entries, seen = [], {}
    for name in list_names:
        if "/" in name or name in ("", ".", ".."):
            raise UsageError(f"--list 只给快照目录里的文件名：{name}")
        for path, digest in read_list(os.path.join(snapshot, name)):
            if path in seen:
                if seen[path] != digest:
                    raise UsageError(f"{path} 在几份清单里的 sha256 不一样（{seen[path][:12]} 与 {digest[:12]}）")
                continue
            seen[path] = digest
            entries.append((path, digest))
    if not entries:
        raise UsageError(f"清单里一行 sha256 都没读到：{'、'.join(list_names)}")
    print(f"  - 读的清单 {len(list_names)} 份：{'、'.join(list_names)}" + ("（没给 --list，读快照目录里全部 *-sha256.txt）" if defaulted else ""))
    chosen = diff_path(snapshot)
    sections = {}
    if chosen:
        with open(chosen, encoding="utf-8", errors="surrogateescape") as handle:
            sections = diff_sections(handle.read())
        for info in sections.values():
            checked_relative_path(info["old"])
            checked_relative_path(info["new"])
    else:
        print(f"  - 快照目录里没有 {HEAD_DIFF} 与 {INDEX_DIFF}：每份都取此刻 HEAD")
    os.makedirs(out, exist_ok=True)
    work_parent = os.path.dirname(os.path.abspath(out))
    counts = {"ok": 0, "unrecovered": 0, "mismatched": 0}
    categories = {"head": 0, "blob": 0, "new": 0, "result": 0, "worktree": 0}
    moves_back, moved_back = moves_back_for(repository), 0
    for path, digest in entries:
        verdict, explanation, category = lay_file(repository, path, digest, sections, out, work_parent, moves_back)
        counts[verdict] += 1
        if verdict == "ok":
            categories[category] += 1
            moved_back += "照搬迁登记换回旧路径（" in explanation
            print(f"  ✓ {path}  {explanation}")
        else:
            print(f"  ✗ {path}  {explanation}")  # gate-lint:detail
    unlisted = sorted(set(sections) - set(seen))
    if unlisted:
        print(f"  - diff 里另有 {len(unlisted)} 份没被清单点名，没铺：{'、'.join(unlisted)}")
    produced = counts["ok"] + counts["mismatched"]
    diff_note = ((os.path.basename(chosen) if chosen else "没有 diff") + (f"；对上的里有 {moved_back} 份照搬迁登记换回旧路径" if moved_back else "")
                 + (f"；对上的里有 {categories['result']} 份取 diff 头记的结果 blob、{categories['worktree']} 份取工作区此刻那一份"
                    if categories["result"] or categories["worktree"] else ""))
    if counts["ok"] != len(entries):
        print(f"  ✗ 快照 {round_name}：清单点名 {len(entries)} 份，倒推出 {produced} 份、sha256 对上 {counts['ok']} 份；"
              f"倒推不出 {counts['unrecovered']} 份、对不上 {counts['mismatched']} 份（逐份列在上面；没对上的不铺进树；diff 用的是 {diff_note}）")  # gate-lint:summary
        print("  → 怎么办：倒推不出的照判决规则写明哪一份、为什么，cite-check 里它被引的那几处逐处人工核；对不上的先看 diff 与清单是不是同一刻取的、diff 有没有被改过")
        return 1
    print(f"  ✓ 快照 {round_name}：清单点名 {len(entries)} 份，倒推出 {produced} 份，sha256 对上 {counts['ok']} 份"
          f"（基底取自 HEAD {categories['head']} 份、diff 头记的 blob {categories['blob']} 份、新文件 {categories['new']} 份；diff 用的是 {diff_note}）；树在 {out}")
    print(f"  判决里的 cite-check 写成：python3 scripts/agents/cite-check.py <全部腿报告> --root {out} --background <背景材料>")
    return 0


def run(arguments):
    if arguments and arguments[0] in ("--help", "-h"):
        print(__doc__)
        return 0
    round_name, out, repository, list_names = None, None, REPOSITORY_ROOT, []
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument in ("--out", "--repository", "--list"):
            if index + 1 >= len(arguments):
                print(f"✗ {argument} 缺值\n→ 怎么办：snapshot-tree.py <轮名> --out <目录> [--repository <仓根>] [--list <清单文件名>]")
                return 2
            value = arguments[index + 1]
            if argument == "--out":
                out = os.path.abspath(value)
            elif argument == "--repository":
                repository = os.path.abspath(value)
            else:
                list_names.append(value)
            index += 2
            continue
        if argument.startswith("-") or round_name is not None:
            print(f"✗ 不认的参数：{argument}\n→ 怎么办：只认一个轮名与 --out、--repository、--list（用法跑 snapshot-tree.py --help）")
            return 2
        round_name = argument
        index += 1
    if round_name is None or out is None:
        print("✗ 没给轮名或 --out\n→ 怎么办：snapshot-tree.py <轮名> --out <草稿目录下一个不存在的目录>")
        return 2
    try:
        return build_tree(round_name, out, repository, list_names)
    except UsageError as error:
        print(f"✗ {error}\n→ 怎么办：快照在 research/prompts/<轮名>-snapshot/；--out 给草稿目录下一个不存在的目录；--list 只给那个目录里的清单文件名")
        return 2


def selftest():
    import contextlib
    import io
    work = tempfile.mkdtemp(prefix="snapshot-tree-selftest-")
    failures, checked = [], 0
    try:
        repository = os.path.join(work, "repository")
        os.makedirs(repository)

        def git_here(*arguments):
            subprocess.run(["git", "-C", repository, "-c", "user.email=t@t", "-c", "user.name=t", "-c", "commit.gpgsign=false",
                            "-c", "core.quotepath=true", *arguments], check=True, capture_output=True)

        def write(path, text, executable=False):
            full = os.path.join(repository, path)
            os.makedirs(os.path.dirname(full), exist_ok=True)
            with open(full, "w", encoding="utf-8") as handle:
                handle.write(text)
            if executable:
                os.chmod(full, 0o755)

        def snapshot(round_name, paths, diff_arguments, diff_name):
            directory = os.path.join(repository, "research", "prompts", f"{round_name}-snapshot")
            os.makedirs(directory)
            with open(os.path.join(directory, DEFAULT_LIST), "w", encoding="utf-8") as handle:
                for path in paths:
                    with open(os.path.join(repository, path), "rb") as source:
                        handle.write(f"{hashlib.sha256(source.read()).hexdigest()}  {path}\n")
            result = subprocess.run(["git", "-C", repository, "-c", "core.quotepath=true", "diff", "--no-color", "--no-ext-diff", "-M",
                                     *diff_arguments, "--", *paths, *[p for p in renamed_from if p not in paths]],
                                    check=True, capture_output=True)
            with open(os.path.join(directory, diff_name), "wb") as handle:
                handle.write(result.stdout)
            return directory

        def case(label, arguments, expected_code, must_contain=(), must_not_contain=()):
            nonlocal checked
            checked += 1
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                code = run(arguments)
            output = buffer.getvalue()
            missing = [text for text in must_contain if text not in output]
            present = [text for text in must_not_contain if text in output]
            if code != expected_code or missing or present:
                failures.append(f"{label}：应当退 {expected_code}" + "".join(f" 并点名 {text}" for text in must_contain)
                                + f"，实际退 {code}；缺 {missing}，多了 {present}\n{output}")
            return output

        git_here("init", "-q")
        renamed_from = ["kb/old-name.md"]
        write("kb/one.md", "a1\na2\na3\n")
        write("records/记录.md", "r1\nr2\n")
        write("tools/run.sh", "echo a\n", executable=True)
        write("kb/same.md", "s\n")
        write("kb/later.md", "l1\n")
        write("kb/staged.md", "x1\n")
        write("kb/old-name.md", "o1\no2\no3\no4\n")
        git_here("add", "-A")
        git_here("commit", "-qm", "base")
        # 开工快照那一刻：别的会话没提交的改动、一份新文件、一次改名带改动
        write("kb/one.md", "a1\nA2\na3\n")
        write("records/记录.md", "r1\nr2\nr3\n")
        write("tools/run.sh", "echo b\n", executable=True)
        write("kb/new.md", "n\n")
        git_here("add", "kb/new.md")
        git_here("mv", "kb/old-name.md", "kb/new-name.md")
        write("kb/new-name.md", "o1\nO2\no3\no4\n")
        listed = ["kb/one.md", "records/记录.md", "tools/run.sh", "kb/new.md", "kb/same.md", "kb/later.md", "kb/new-name.md"]
        head_snapshot = snapshot("r1", listed, ["HEAD"], HEAD_DIFF)
        # 对暂存区的旧轮：暂存区是 x2，工作区是 x3
        write("kb/staged.md", "x2\n")
        git_here("add", "kb/staged.md")
        write("kb/staged.md", "x3\n")
        renamed_from = []
        snapshot("r4", ["kb/staged.md"], [], INDEX_DIFF)
        # 快照之后：提交把 one.md 再改一次、新文件换了内容、staged.md 改成 x4，记录.md 留着不提交
        write("kb/one.md", "a1\nA2\na3\nmore\n")
        write("kb/new.md", "n2\n")
        write("kb/staged.md", "x4\n")
        git_here("add", "kb/one.md", "kb/new.md", "kb/staged.md", "kb/new-name.md", "tools/run.sh")
        git_here("commit", "-qm", "after")

        tree = os.path.join(work, "tree-after")
        case("快照之后有提交", ["r1", "--out", tree, "--repository", repository], 0,
             ("倒推出 7 份，sha256 对上 7 份", "kb/one.md  来源：diff 头记的基底 blob", "kb/new.md  来源：新文件",
              "kb/same.md  来源：HEAD（diff 里没有它）", "records/记录.md  来源：HEAD", "kb/new-name.md  来源：diff 头记的基底 blob"))
        with open(os.path.join(head_snapshot, DEFAULT_LIST), encoding="utf-8") as handle:
            for line in handle:
                digest, path = line.rstrip("\n").split("  ", 1)
                laid = os.path.join(tree, path)
                checked += 1
                if not os.path.isfile(laid) or hashlib.sha256(open(laid, "rb").read()).hexdigest() != digest:
                    failures.append(f"树里的 {path}：应当在且 sha256 与快照清单相同")
        if [name for name in os.listdir(work) if name.startswith(".snapshot-tree-apply-")]:
            failures.append("打 diff 的临时目录：应当用完就删")

        broken = os.path.join(repository, "research", "prompts", "r2-snapshot")
        shutil.copytree(head_snapshot, broken)
        with open(os.path.join(broken, HEAD_DIFF), encoding="utf-8") as handle:
            text = handle.read()
        with open(os.path.join(broken, HEAD_DIFF), "w", encoding="utf-8") as handle:
            handle.write(text.replace("+A2\n", "+A9\n", 1))
        case("改坏的 diff", ["r2", "--out", os.path.join(work, "tree-broken"), "--repository", repository], 1,
             ("kb/one.md  sha256 对不上", "对不上 1 份"))
        checked += 1
        if os.path.exists(os.path.join(work, "tree-broken", "kb", "one.md")):
            failures.append("改坏的 diff：对不上的 kb/one.md 不该铺进树")

        missing_blob = os.path.join(repository, "research", "prompts", "r3-snapshot")
        shutil.copytree(head_snapshot, missing_blob)
        with open(os.path.join(missing_blob, HEAD_DIFF), encoding="utf-8") as handle:
            text = handle.read()
        one_index = re.search(r"diff --git a/kb/one.md b/kb/one.md\nindex ([0-9a-f]+)\.\.", text).group(1)
        with open(os.path.join(missing_blob, HEAD_DIFF), "w", encoding="utf-8") as handle:
            handle.write(text.replace(f"index {one_index}..", "index 0123456789abcdef..", 1))
        case("基底 blob 取不到", ["r3", "--out", os.path.join(work, "tree-missing"), "--repository", repository], 1,
             ("kb/one.md  倒推不出：diff 头记的基底 blob 0123456789abcdef 在对象库里取不到", "倒推不出 1 份"))

        case("对暂存区的旧轮", ["r4", "--out", os.path.join(work, "tree-index"), "--repository", repository], 0,
             ("kb/staged.md  来源：diff 头记的基底 blob", "diff 用的是 judged-vs-index.diff"))

        # 两种最后的取法（fallback_content）：r6 的 diff 记的基底取不到、结果 blob 在对象库里（git hash-object -w 写进去，工作区里没有这份）；
        # r7 的基底与结果 blob 都取不到，一份工作区里 sha256 对得上（取它），一份工作区里改过（照报倒推不出）。开关 result-blob-ignored、worktree-ignored 下各红一格
        def hand_snapshot(round_name, rows, diff_text):
            directory = os.path.join(repository, "research", "prompts", f"{round_name}-snapshot")
            os.makedirs(directory)
            with open(os.path.join(directory, DEFAULT_LIST), "w", encoding="utf-8") as handle:
                for listed_path, listed_text in rows:
                    handle.write(f"{hashlib.sha256(listed_text.encode()).hexdigest()}  {listed_path}\n")
            with open(os.path.join(directory, HEAD_DIFF), "w", encoding="utf-8") as handle:
                handle.write(diff_text)

        def hand_section(section_path, result_blob, old_line, new_line):
            return (f"diff --git a/{section_path} b/{section_path}\nindex 0123456789ab..{result_blob} 100644\n--- a/{section_path}\n+++ b/{section_path}\n"
                    f"@@ -1 +1 @@\n-{old_line}\n+{new_line}\n")
        stored = subprocess.run(["git", "-C", repository, "hash-object", "-w", "--stdin"], input=b"b2\n", check=True, capture_output=True).stdout.decode().strip()
        hand_snapshot("r6", [("kb/result-blob.md", "b2\n")], hand_section("kb/result-blob.md", stored[:12], "b1", "b2"))
        case("结果 blob 还在对象库里", ["r6", "--out", os.path.join(work, "tree-result-blob"), "--repository", repository], 0,
             (f"kb/result-blob.md  来源：diff 头记的结果 blob {stored[:12]}", "照原样：倒推不出", "对上的里有 1 份取 diff 头记的结果 blob、0 份取工作区此刻那一份"))
        write("kb/worktree-only.md", "w2\n")
        write("kb/worktree-changed.md", "c3\n")
        hand_snapshot("r7", [("kb/worktree-only.md", "w2\n"), ("kb/worktree-changed.md", "c2\n")],
                      hand_section("kb/worktree-only.md", "fedcba987654", "w1", "w2") + hand_section("kb/worktree-changed.md", "fedcba987655", "c1", "c2"))
        case("工作区那一份对得上", ["r7", "--out", os.path.join(work, "tree-worktree"), "--repository", repository], 1,
             ("kb/worktree-only.md  来源：工作区此刻那一份", "✗ kb/worktree-changed.md  倒推不出", "对上的里有 0 份取 diff 头记的结果 blob、1 份取工作区此刻那一份"))
        checked += 1
        if os.path.exists(os.path.join(work, "tree-worktree", "kb", "worktree-changed.md")):
            failures.append("工作区那一份对得上：工作区里改过的 kb/worktree-changed.md 不该铺进树")

        write("kb/later.md", "l2\n")
        git_here("add", "kb/later.md")
        git_here("commit", "-qm", "later")
        case("diff 里没有、快照之后被提交改过", ["r1", "--out", os.path.join(work, "tree-later"), "--repository", repository], 1,
             ("kb/later.md  倒推不出：diff 里没有它", "倒推不出 1 份"))

        # 快照之后有一次登记过的搬迁（tools/old-name → tools/new-name）：搬迁那一趟把快照目录里的清单与 diff 也换成了新路径，基底 blob 里还是旧路径
        # （.claude/rules/path-moves.md 第 6 步：只差路径的不算改动；开关 path-moves-ignored 下这一格红）
        moved = os.path.join(work, "repository-moved")
        os.makedirs(moved)

        def git_moved(*arguments):
            subprocess.run(["git", "-C", moved, "-c", "user.email=t@t", "-c", "user.name=t", "-c", "commit.gpgsign=false", *arguments],
                           check=True, capture_output=True)

        def write_moved(path, text):
            full = os.path.join(moved, path)
            os.makedirs(os.path.dirname(full), exist_ok=True)
            with open(full, "w", encoding="utf-8") as handle:
                handle.write(text)

        git_moved("init", "-q")
        write_moved("tools/old-name/run.sh", "echo tools/old-name/run.sh\n")
        write_moved("kb/guide.md", "用 tools/old-name/run.sh 跑\n第二行\n第三行\n")
        write_moved("kb/plain.md", "见 tools/old-name/run.sh\n")
        write_moved("kb/remove.md", "tools/old-name/run.sh 写在第一列\n保留\n")
        git_moved("add", "-A")
        git_moved("commit", "-qm", "base")
        write_moved("tools/old-name/run.sh", "echo tools/old-name/run.sh 改过\n")
        write_moved("kb/guide.md", "用 tools/old-name/run.sh 跑\n第二行改过\n第三行\n")
        write_moved("kb/remove.md", "保留\n")   # 删掉的那一行在 diff 里是「-tools/…」：记号紧挨着路径
        moved_listed = ["tools/old-name/run.sh", "kb/guide.md", "kb/plain.md", "kb/remove.md"]
        moved_snapshot = os.path.join(moved, "research", "prompts", "r5-snapshot")
        os.makedirs(moved_snapshot)
        with open(os.path.join(moved_snapshot, DEFAULT_LIST), "w", encoding="utf-8") as handle:
            for path in moved_listed:
                with open(os.path.join(moved, path), "rb") as source:
                    handle.write(f"{hashlib.sha256(source.read()).hexdigest()}  {path}\n")
        moved_diff = subprocess.run(["git", "-C", moved, "diff", "--no-color", "--no-ext-diff", "HEAD", "--", *moved_listed], check=True, capture_output=True).stdout
        with open(os.path.join(moved_snapshot, HEAD_DIFF), "wb") as handle:
            handle.write(moved_diff)
        # 搬迁那一趟：git mv、全仓（快照目录里的清单与 diff 也算）把旧路径换成新路径、登记、提交
        git_moved("mv", "tools/old-name", "tools/new-name")
        for path in ("tools/new-name/run.sh", "kb/guide.md", "kb/plain.md", "kb/remove.md",
                     f"research/prompts/r5-snapshot/{DEFAULT_LIST}", f"research/prompts/r5-snapshot/{HEAD_DIFF}"):
            with open(os.path.join(moved, path), encoding="utf-8") as handle:
                rewritten = handle.read().replace("tools/old-name", "tools/new-name")
            write_moved(path, rewritten)
        write_moved(".claude/gate.d/path-moves.tsv", "# 搬迁登记表\ntools/old-name\ttools/new-name\t2026-10-23\trecords/2026-10-23-搬迁.md\n")
        write_moved("records/2026-10-23-搬迁.md", "# 搬迁\n")
        git_moved("add", "-A")
        git_moved("commit", "-qm", "move")
        case("搬迁之后的旧轮", ["r5", "--out", os.path.join(work, "tree-moved"), "--repository", moved], 0,
             ("倒推出 4 份，sha256 对上 4 份", "对上的里有 4 份照搬迁登记换回旧路径",
              "tools/new-name/run.sh  来源：diff 头记的基底 blob", "kb/guide.md  来源：diff 头记的基底 blob", "kb/plain.md  来源：HEAD（diff 里没有它）",
              "kb/remove.md  来源：diff 头记的基底 blob", "照搬迁登记换回旧路径（.claude/gate.d/path-moves.tsv 第 2 行"))
        checked += 1
        laid_moved = os.path.join(work, "tree-moved", "tools", "new-name", "run.sh")
        if not os.path.isfile(laid_moved) or open(laid_moved, encoding="utf-8").read() != "echo tools/old-name/run.sh 改过\n":
            failures.append("搬迁之后的旧轮：树里的 tools/new-name/run.sh 应当是快照那一刻的内容（里面还是旧路径）")

        # 快照目录里两份清单（judged-sha256.txt 与 kb-sha256.txt）、没有 diff：不给 --list 两份都读、都倒推（开关 judged-list-only 下只读 judged 那一份，这一格红）
        two_lists = os.path.join(repository, "research", "prompts", "r8-snapshot")
        os.makedirs(two_lists)
        for list_name, listed_path in ((DEFAULT_LIST, "kb/same.md"), ("kb-sha256.txt", "tools/run.sh")):
            with open(os.path.join(repository, listed_path), "rb") as source, open(os.path.join(two_lists, list_name), "w", encoding="utf-8") as handle:
                handle.write(f"{hashlib.sha256(source.read()).hexdigest()}  {listed_path}\n")
        case("带两份清单的快照", ["r8", "--out", os.path.join(work, "tree-two-lists"), "--repository", repository], 0,
             ("清单点名 2 份，倒推出 2 份，sha256 对上 2 份", "kb/same.md  来源：HEAD", "tools/run.sh  来源：HEAD", "读的清单 2 份：judged-sha256.txt、kb-sha256.txt"))
        os.makedirs(os.path.join(repository, "research", "prompts", "r9-snapshot"))
        with open(os.path.join(repository, "research", "prompts", "r9-snapshot", "notes.md"), "w", encoding="utf-8") as handle:
            handle.write("没有清单\n")
        os.makedirs(os.path.join(work, "not-empty"))
        with open(os.path.join(work, "not-empty", "x"), "w"):
            pass
        shutil.copy(os.path.join(head_snapshot, HEAD_DIFF), os.path.join(repository, "research", "prompts", "r4-snapshot", HEAD_DIFF))
        usage_cases = [
            ("快照目录不在", ["nothere", "--out", os.path.join(work, "tree-x"), "--repository", repository], 2, ("快照目录不在",)),
            ("--out 不空", ["r1", "--out", os.path.join(work, "not-empty"), "--repository", repository], 2, ("不是空目录",)),
            ("--out 在仓里", ["r1", "--out", os.path.join(repository, "tree"), "--repository", repository], 2, ("落在仓里",)),
            ("两份 diff 都在", ["r4", "--out", os.path.join(work, "tree-y"), "--repository", repository], 2, ("都在",)),
            ("清单读不了", ["r1", "--out", os.path.join(work, "tree-z"), "--repository", repository, "--list", "nothere.txt"], 2, ("清单读不了",)),
            ("快照目录里一份清单都没有", ["r9", "--out", os.path.join(work, "tree-w"), "--repository", repository], 2, ("没有 *-sha256.txt 清单",)),
            ("没给 --out", ["r1", "--repository", repository], 2, ("没给轮名或 --out",)),
            ("--help", ["--help"], 0, ("用法",)),
        ]
        for label, arguments, code, texts in usage_cases:
            case(label, arguments, code, texts)
        # 平常跑设了弄坏开关：子进程里带上变量、不带 --selftest，要退 2 并点名变量只在自证里认（探针选项谁都不认得，没有这道拒的只会退用法错、不点名变量）
        # 关守卫项 guard-off 只在单独一项时认：开关 guard-off 设着时探针只写它一项（照认，这一格红）；同写的那一格另起一次
        checked += 1
        probe_value = GUARD_OFF_ITEM if GUARD_OFF_ITEM in BROKEN else "skip-sha-check"
        probe = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), "r1", "--out", os.path.join(work, "tree-probe"),
                                "--repository", repository, "--break-guard-probe-unknown-option"],
                               env=dict(os.environ, **{BREAK_VARIABLE: probe_value}), stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)
        named = any(BREAK_VARIABLE in row and "自证" in row for row in (probe.stdout + probe.stderr).split("\n"))
        if probe.returncode != 2 or not named:
            failures.append(f"平常跑设了弄坏开关：应当退 2 并点名 {BREAK_VARIABLE} 只在自证里认，实际退 {probe.returncode}，点名 {named}")
        # 关守卫项与别的项同写：平常跑（子进程）退 2 并点名它；进程里同一判法再探一次（开关 guard-off-with-others 只在自证进程里认，设着时这一格红）
        checked += 2
        combined_value = f"{GUARD_OFF_ITEM},skip-sha-check"
        combined = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), "r1", "--out", os.path.join(work, "tree-combined"),
                                   "--repository", repository, "--break-guard-probe-unknown-option"],
                                  env=dict(os.environ, **{BREAK_VARIABLE: combined_value}), stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)
        if combined.returncode != 2 or "唯一一项" not in combined.stdout + combined.stderr:
            failures.append(f"关守卫项与别的项同写、平常跑：应当退 2 并点名 {GUARD_OFF_ITEM} 只在它是唯一一项时认，实际退 {combined.returncode}")
        combined_buffer = io.StringIO()
        with contextlib.redirect_stdout(combined_buffer):
            combined_code = refuse_break_outside_selftest(["r1", "--out", os.path.join(work, "tree-combined")], value=combined_value)
        if combined_code != 2 or "唯一一项" not in combined_buffer.getvalue():
            failures.append(f"关守卫项与别的项同写：应当退 2 并点名 {GUARD_OFF_ITEM} 只在它是唯一一项时认，实际 {combined_code}")
        if failures:
            for failure in failures:
                print(f"  ✗ 自检：{failure}")  # gate-lint:detail
            print("  → 看 base_for() / applied() / lay_file() / unquoted() / diff_path() / moves_back_for()；"
                  "SNAPSHOT_TREE_BREAK（skip-sha-check、head-only、no-unquote、index-diff-ignored、path-moves-ignored、diff-markers-kept、result-blob-ignored、worktree-ignored）设着的话这里本来就该红")
            return 1
        print(f"  ✓ snapshot-tree 自检通过（查了 {checked} 项）：快照之后有提交的按 diff 头记的基底 blob 倒推、新文件、改名、非 ASCII 引号路径、"
              f"diff 里没有的取 HEAD，铺出的树逐份 sha256 与清单相同；改坏的 diff 判对不上、基底 blob 取不到与 diff 里没有又被提交改过的判倒推不出，都退 1、不铺进树；"
              f"对暂存区的旧轮照认；照原样倒推不出或对不上的，diff 头记的结果 blob 在对象库里、或工作区此刻那一份 sha256 与快照相同的取它，工作区里改过的照报倒推不出；快照之后有登记过的搬迁的，diff、HEAD 与清单路径照 path-moves.tsv 反着换回旧路径再倒推，对上的铺在清单点名的路径上；用法错 {len(usage_cases) - 1} 种退 2，--help 退 0；不给 --list 时快照目录里两份清单都读、都倒推；"
              f"平常跑设了弄坏开关退 2 并点名只在自证里认，关守卫项与别的项同写也退 2 并点名它")
        return 0
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    refused_exit = refuse_break_outside_selftest(sys.argv[1:])
    if refused_exit is not None:
        sys.exit(refused_exit)
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(run(sys.argv[1:]))
