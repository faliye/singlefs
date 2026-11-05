#!/usr/bin/env python3
# admission: always 判的是此刻工作区、搬迁登记表与 git 历史的样子，上一次的结论不替这一次作保
# run-condition: command git python3
"""一条此刻不在的仓内路径，找它的关联路径（后继）并写出依据。

用法（路径都相对 --root，默认当前目录）：
  path-successor.py [--root 目录] [--overlap-cap N] <路径>…
  path-successor.py --selftest
每条路径打一行「<路径>\t<依据类别>\t<结论句>」；退出码 0 判完了（找不到也算判完），2 用法不对。
也可被 import：conclude(路径, 根) 交回 Conclusion，Conclusion.sentence() 是结论句；
门禁 doc-registries 的 governance-refs 格（.claude/gate.d/lib-governance-refs.py）照这样调，红行带上结论句。

按次序找，前一步找到就停：
  ① 登记：.claude/gate.d/path-moves.tsv 的行（读法是 .claude/gate.d/lib-lag-baseline.py 的 read_path_moves，写坏的行不认）
     按登记次序把旧前缀换成新前缀，换完此刻在的。
  ② 改名：删它的提交（git log --diff-filter=D -1 -- <路径>；按路径限定时改名出去的一侧也算删）里，
     不按路径限定再看一遍那次提交（git diff-tree -M），它被改名成了什么；改名之后又不在的顺着追，追到此刻在的为止。
     目录：那次提交里目录下改名出去的文件都换成同一个新前缀、新前缀此刻在，才算目录改了名。
  ③ 删了的（改名链追到头仍不在的，按链尾那一条判）：报删它的提交（短哈希与标题，标题常写去向）与前一版的取法 <提交>^:<路径>；
     再在那次提交里新加、改动、改名进来的文件里按内容重合排：旧版里去掉首尾空白之后长于 MINIMUM_LINE_CHARACTERS 字的行（去重）
     有多少在候选那一版里也有，比例不低于 OVERLAP_THRESHOLD、候选此刻在的列为后继并写比例；
     只比扩展名与被查的文件相同的，按与被查路径共有的前缀长短排、最多比 --overlap-cap 份（默认 DEFAULT_OVERLAP_CAP），上限与跳过的份数写进结论；
     另列此刻被跟踪、文件名（目录名）相同而且唯一的那一份。两样都没有的，类别是「删除无后继」。
     HEAD 里还在、只是工作区里删了的，前一版取法写 HEAD:<路径>，不比内容，只找同名。
  ④ 从没进过 git 的（git log --all 一条都没有）：
     先看上游副本：<根>/.claude/singlefs-ai-sop/<路径> 在磁盘上的（那棵树被 .gitignore 挡着，不在 git ls-files 里，只按磁盘判），
     类别是「上游副本」，后继交副本路径，结论句写明说上游的句子保留原写法、登进 .claude/gate.d/dead-path-kept.tsv，说本仓副本的改成副本路径；
     再找此刻被跟踪的文件（目录）里唯一以 /<路径> 结尾的那一份；不唯一或没有的，类别是「找不到」。
     只在别的分支进过 git（--all 有、HEAD 的历史里没有）的同样按 ④ 找，结论里注明。
  门禁样本副本：被问的路径本身不在样本目录下时，③ 的同名与 ④ 的后缀两样候选都排掉样本目录下的文件
     （样本目录按路径段认：任一段等于 fixtures）；唯一性按剩下的算，结论句写明排掉了几份。被问的路径本身在样本目录下的照旧不排。
依据类别：登记、改名、删除加内容重合、删除加同名、删除无后继、上游副本、后缀唯一、找不到；此刻在的是「在」。

弄坏开关 PATH_SUCCESSOR_BREAK=<项>（只给 --selftest 证红用；平常跑设了就拒，退 2）：
  PATH_SUCCESSOR_BREAK=moves-ignored  不读搬迁登记表（依据「登记」那一步不做）
      registered（「✗ 自检 registered：」）
  PATH_SUCCESSOR_BREAK=rename-chain-stops  改名只追一步，第一次改名的落点不在也当后继交出去（依据「改名」）
      renamed-twice（「✗ 自检 renamed-twice：」）
  PATH_SUCCESSOR_BREAK=overlap-ignored  删了的不比内容重合（依据「删除加内容重合」）
      deleted-with-overlap（「✗ 自检 deleted-with-overlap：」）
  PATH_SUCCESSOR_BREAK=same-name-ignored  删了的不找同名文件（依据「删除加同名」）
      deleted-with-same-name（「✗ 自检 deleted-with-same-name：」）
      same-name-beside-fixture-copy（「✗ 自检 same-name-beside-fixture-copy：」）
  PATH_SUCCESSOR_BREAK=previous-version-unreported  删了的结论句不写前一版的取法 <提交>^:<路径>（依据「删除无后继」）
      deleted-without-successor（「✗ 自检 deleted-without-successor：」）
  PATH_SUCCESSOR_BREAK=suffix-ignored  从没进过 git 的不按后缀找（依据「后缀唯一」）
      suffix-unique（「✗ 自检 suffix-unique：」）
      not-found（「✗ 自检 not-found：」：后缀命中几份那一句跟着没了）
      suffix-only-fixture-copy（「✗ 自检 suffix-only-fixture-copy：」：排掉几份样本副本那一句跟着没了）
      suffix-beside-fixture-copy（「✗ 自检 suffix-beside-fixture-copy：」）
  PATH_SUCCESSOR_BREAK=suffix-ambiguous-taken  后缀命中几份时取排在前面的那一份当后继，不要求唯一（依据「找不到」）
      not-found（「✗ 自检 not-found：」）
  PATH_SUCCESSOR_BREAK=fixtures-taken  同名与后缀两步不排门禁样本副本（依据「删除加同名」「后缀唯一」「找不到」）
      same-name-beside-fixture-copy（「✗ 自检 same-name-beside-fixture-copy：」）
      suffix-only-fixture-copy（「✗ 自检 suffix-only-fixture-copy：」）
      suffix-beside-fixture-copy（「✗ 自检 suffix-beside-fixture-copy：」）
  PATH_SUCCESSOR_BREAK=upstream-ignored  从没进过 git 的不看上游副本，落到后缀那一步（依据「上游副本」）
      upstream-copy（「✗ 自检 upstream-copy：」）
自证（--selftest）每一格在自己的临时 git 仓里造一种局面（registered、renamed-twice、deleted-with-overlap、deleted-with-same-name、
same-name-beside-fixture-copy、deleted-without-successor、upstream-copy、suffix-unique、suffix-only-fixture-copy、
suffix-beside-fixture-copy、not-found、present），判出的类别、后继与结论句要与答案相同；
不设开关时另把上面每一条开关各设上起一次自己的 --selftest，要退 1、红在点名的那一格。
"""
import os
import re
import shutil
import subprocess
import sys
import tempfile
import importlib.util
from typing import NamedTuple
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
LAG_BASELINE_LIBRARY = os.path.join(REPOSITORY, ".claude", "gate.d", "lib-lag-baseline.py")
BREAK_VARIABLE = "PATH_SUCCESSOR_BREAK"
BREAK_MODES = ("moves-ignored", "rename-chain-stops", "overlap-ignored", "same-name-ignored",
               "previous-version-unreported", "suffix-ignored", "suffix-ambiguous-taken", "fixtures-taken",
               "upstream-ignored")
MINIMUM_LINE_CHARACTERS = 12        # 旧版里长于这么多字（去掉首尾空白之后）的行才拿来比，短行（括号、空注释）谁都重合
OVERLAP_THRESHOLD = 0.30            # 旧版的长行有这么大比例在候选里也有，才列为后继；量过：低于两成的是噪声（records/2026-10-30-路径关联与候选精度.md「现状」表）
DEFAULT_OVERLAP_CAP = 300           # 一次提交里最多比这么多份候选：归档提交一次删加几千份，全比太慢
RENAME_CHAIN_LIMIT = 64             # 改名链最多追这么多步，防环之外再兜一层
FIXTURE_SEGMENT = "fixtures"        # 路径里有一段等于它的，是门禁样本目录下的文件
UPSTREAM_COPY_DIRECTORY = ".claude/singlefs-ai-sop"   # 上游 singlefs-ai-sop 仓的本仓副本，被 .gitignore 挡着
KEPT_TABLE = ".claude/gate.d/dead-path-kept.tsv"

REGISTERED = "登记"
RENAMED = "改名"
DELETED_WITH_OVERLAP = "删除加内容重合"
DELETED_WITH_SAME_NAME = "删除加同名"
DELETED_WITHOUT_SUCCESSOR = "删除无后继"
UPSTREAM_COPY = "上游副本"
SUFFIX_UNIQUE = "后缀唯一"
NOT_FOUND = "找不到"
PRESENT = "在"


def broken(mode):
    return os.environ.get(BREAK_VARIABLE, "") == mode


class Removal(NamedTuple):
    """删掉（或改名出去）一条路径的提交：全哈希、短哈希、标题；HEAD 里还在、只是工作区删了的，full 是 None。"""
    full: object
    short: str
    subject: str

    def shown(self):
        return "工作区（HEAD 里还在，还没提交这次删除）" if self.full is None else f"{self.short}（{self.subject}）"


class Conclusion(NamedTuple):
    path: str
    kind: str
    successors: tuple          # ((后继路径, 一句依据), …)，按可信次序
    removal: object            # Removal 或 None
    previous_version: str      # 前一版的取法（<提交>^:<路径> 或 HEAD:<路径>），没有删除这一层的是空串
    notes: tuple               # 补充说明（改名链、比了几份、后缀命中几份……）

    def sentence(self):
        """结论句：有后继的「改成 <后继>（依据）」，删了没后继的「删于 <提交>（标题）；…<提交>^:<路径>」，找不到的「从没进过 git」。"""
        notes = "；".join(self.notes)
        tail = f"；{notes}" if notes else ""
        if self.kind == PRESENT:
            return "此刻在，不用找后继"
        if self.successors:
            choices = "，或 ".join(f"{path}（{self.kind}：{why}）" for path, why in self.successors)
            removed = ""
            if self.removal is not None:
                removed = f"；删于 {self.removal.shown()}" + (f"，前一版 `{self.previous_version}`" if self.previous_version else "")
            return f"改成 {choices}{removed}{tail}"
        if self.kind == DELETED_WITHOUT_SUCCESSOR:
            previous = "" if broken("previous-version-unreported") else f"，说那一次的事件句写成 `{self.previous_version}`"
            return f"删于 {self.removal.shown()}；现状句改写或删去{previous}{tail}"
        return f"从没进过 git{tail}" if not notes.startswith("只在别的分支") else f"{notes}"


# ---------------------------------------------------------------- git 与工作区

def git(root, *arguments, data=None):
    """在 root 里起 git：(退出码, 标准输出字节)。不在 git 仓里、git 出错都只是退出码非 0，调用方当「没有这一层」。"""
    result = subprocess.run(["git", "-c", "core.quotepath=false", "--literal-pathspecs", *arguments],
                            cwd=root, input=data, capture_output=True)
    return result.returncode, result.stdout


def is_repository(root):
    code, output = git(root, "rev-parse", "--is-inside-work-tree")
    return code == 0 and output.strip() == b"true"


TRACKED_CACHE = {}


def tracked_files(root):
    """此刻被跟踪的文件（相对 root）；同一个进程里按根缓存（governance-refs 一趟要问几十条）。"""
    key = os.path.realpath(root)
    if key not in TRACKED_CACHE:
        code, output = git(root, "ls-files", "-z")
        TRACKED_CACHE[key] = sorted(item.decode("utf-8", "replace") for item in output.split(b"\0") if item) if code == 0 else []
    return TRACKED_CACHE[key]


def tracked_directories(root):
    directories = set()
    for path in tracked_files(root):
        parts = path.split("/")[:-1]
        for end in range(1, len(parts) + 1):
            directories.add("/".join(parts[:end]))
    return sorted(directories)


def exists(root, path):
    return os.path.exists(os.path.join(root, path))


def is_directory_path(root, path, removal=None):
    """被查的路径是不是目录：写法带结尾的 /，或删之前那一版是一棵树。"""
    if path.endswith("/"):
        return True
    if removal is not None and removal.full is not None:
        code, output = git(root, "cat-file", "-t", f"{removal.full}^:{path}")
        return code == 0 and output.strip() == b"tree"
    return False


def last_removal(root, path):
    """HEAD 的历史里最近一次删掉（或改名出去）path 的提交；没有交 None。"""
    code, output = git(root, "log", "--diff-filter=D", "--format=%H%x09%h%x09%s", "-1", "--", path)
    if code != 0 or not output.strip():
        return None
    full, short, subject = output.decode("utf-8", "replace").rstrip("\n").split("\t", 2)
    return Removal(full, short, subject)


def ever_in_git(root, path, scope):
    """git log <scope> -1 -- path 有没有输出（scope 是 --all 或 HEAD）。"""
    code, output = git(root, "log", scope, "-1", "--format=%h", "--", path)
    return code == 0 and bool(output.strip())


CHANGES_CACHE = {}


def commit_changes(root, full):
    """一次提交相对它第一个父提交的改动：[(状态字母, 旧路径, 新路径)]，带改名检测。
    同一个进程里按（根, 提交）缓存：大提交的改名检测要一秒多，改名链与内容重合两步各要一次。"""
    key = (os.path.realpath(root), full)
    if key not in CHANGES_CACHE:
        CHANGES_CACHE[key] = parse_changes(root, full)
    return CHANGES_CACHE[key]


def parse_changes(root, full):
    code, output = git(root, "diff-tree", "-r", "-M", "--name-status", "-z", f"{full}^", full)
    if code != 0:
        return []
    fields = [item.decode("utf-8", "replace") for item in output.split(b"\0")]
    changes, index = [], 0
    while index < len(fields) and fields[index]:
        status = fields[index][:1]
        if status in "RC":
            changes.append((status, fields[index + 1], fields[index + 2]))
            index += 3
        else:
            changes.append((status, fields[index + 1], fields[index + 1]))
            index += 2
    return changes


def rename_target(root, removal, path, directory):
    """那次提交里 path 改名成了什么；目录要下面改名出去的文件都换成同一个新前缀、新前缀此刻在。没改名交 None。"""
    renames = [(old, new) for status, old, new in commit_changes(root, removal.full) if status == "R"]
    bare = path.rstrip("/")
    if not directory:
        for old, new in renames:
            if old == bare:
                return new
        return None
    prefixes = set()
    for old, new in renames:
        if old.startswith(bare + "/"):
            rest = old[len(bare):]
            if not new.endswith(rest):
                return None
            prefixes.add(new[:-len(rest)])
    if len(prefixes) == 1:
        return prefixes.pop() + ("/" if path.endswith("/") else "")
    return None


def blobs(root, specifications):
    """一次 git cat-file --batch 取几份 <提交>:<路径> 的内容：{规格: 字节}；取不到的不在字典里。"""
    if not specifications:
        return {}
    process = subprocess.run(["git", "cat-file", "--batch"], cwd=root, capture_output=True,
                             input="".join(f"{specification}\n" for specification in specifications).encode("utf-8"))
    contents, output, position = {}, process.stdout, 0
    for specification in specifications:
        end = output.find(b"\n", position)
        if end < 0:
            break
        header = output[position:end].split()
        position = end + 1
        if len(header) == 3 and header[1] == b"blob":
            size = int(header[2])
            contents[specification] = output[position:position + size]
            position += size + 1
    return contents


def long_lines(data):
    return {line.strip() for line in data.decode("utf-8", "replace").split("\n") if len(line.strip()) > MINIMUM_LINE_CHARACTERS}


def common_prefix_length(left, right):
    length = 0
    for left_character, right_character in zip(left, right):
        if left_character != right_character:
            break
        length += 1
    return length


# ---------------------------------------------------------------- 四步

LAG_BASELINE_MODULE = []


def lag_baseline_library():
    """搬迁登记表的读法（.claude/gate.d/lib-lag-baseline.py），同一个进程里只装一次。"""
    if not LAG_BASELINE_MODULE:
        specification = importlib.util.spec_from_file_location("lib_lag_baseline", LAG_BASELINE_LIBRARY)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        LAG_BASELINE_MODULE.append(module)
    return LAG_BASELINE_MODULE[0]

def registered_successor(root, path):
    """① 搬迁登记表按登记次序换前缀，换完此刻在的：(新路径, 依据) 或 None。"""
    if broken("moves-ignored"):
        return None
    lag_baseline = lag_baseline_library()
    moves, _problems = lag_baseline.read_path_moves(root)
    current = path.rstrip("/")
    for move in moves:
        moved = lag_baseline.moved_path(current, move)
        if moved != current:
            current = moved
            if exists(root, current):
                return current + ("/" if path.endswith("/") else ""), f"{lag_baseline.PATH_MOVES_TABLE} 第 {move.line} 行 {move.old} → {move.new}"
    return None


def overlap_successors(root, removal, path, cap):
    """③ 那次提交里新加、改动、改名进来的同扩展名文件按内容重合排：([(路径, 依据)], [说明])。"""
    old = blobs(root, [f"{removal.full}^:{path}"]).get(f"{removal.full}^:{path}")
    if old is None:
        return [], []
    old_lines = long_lines(old)
    if not old_lines:
        return [], [f"旧版里没有长于 {MINIMUM_LINE_CHARACTERS} 字的行，没比内容重合"]
    extension = os.path.splitext(path)[1]
    candidates = sorted({new for status, _old, new in commit_changes(root, removal.full)
                         if status in "AMRCT" and new != path and os.path.splitext(new)[1] == extension},
                        key=lambda candidate: (-common_prefix_length(candidate, path), candidate))
    compared, skipped = candidates[:cap], len(candidates) - min(cap, len(candidates))
    contents = blobs(root, [f"{removal.full}:{candidate}" for candidate in compared])
    scored, gone = [], 0
    for candidate in compared:
        data = contents.get(f"{removal.full}:{candidate}")
        if data is None:
            continue
        shared = len(old_lines & long_lines(data))
        ratio = shared / len(old_lines)
        if ratio < OVERLAP_THRESHOLD:
            continue
        if not exists(root, candidate):
            gone += 1
            continue
        scored.append((ratio, shared, candidate))
    scored.sort(key=lambda item: (-item[0], item[2]))
    successors = [(candidate, f"旧版 {len(old_lines)} 行长行里 {shared} 行在它那一版里，重合 {round(ratio * 100)}%")
                  for ratio, shared, candidate in scored]
    notes = [f"在删它的那次提交里比了 {len(compared)} 份扩展名相同的文件（上限 {cap}），跳过 {skipped} 份"] if candidates else []
    if gone:
        notes.append(f"另有 {gone} 份重合够而此刻不在")
    return successors, notes


def in_fixtures(path):
    """路径有一段等于 fixtures：门禁样本目录下的文件（目录）。"""
    return FIXTURE_SEGMENT in path.rstrip("/").split("/")


def without_fixture_copies(path, matches):
    """被问的路径不在样本目录下时，候选里排掉样本目录下的：(剩下的, 排掉几份)。在样本目录下的、或开关 fixtures-taken 下照旧不排。"""
    if in_fixtures(path) or broken("fixtures-taken"):
        return matches, 0
    kept = [candidate for candidate in matches if not in_fixtures(candidate)]
    return kept, len(matches) - len(kept)


def fixture_note(step, excluded):
    """排掉样本副本的那一句；一份没排的交 None。"""
    if not excluded:
        return None
    return f"{step}排掉了 {excluded} 份门禁样本副本（路径里有 {FIXTURE_SEGMENT} 这一段，被问的路径不在样本目录下）"


def same_name_successor(root, path, directory):
    """③ 此刻被跟踪、文件名（目录名）相同而且唯一的那一份（排掉样本副本之后）：((路径, 依据) 或 None, 排掉几份)。"""
    if broken("same-name-ignored"):
        return None, 0
    name = os.path.basename(path.rstrip("/"))
    pool = tracked_directories(root) if directory else tracked_files(root)
    matches, excluded = without_fixture_copies(path, [candidate for candidate in pool if os.path.basename(candidate) == name])
    if len(matches) == 1:
        return (matches[0] + ("/" if directory else ""), f"此刻被跟踪的{'目录' if directory else '文件'}里名叫 {name} 的只有这一份"), excluded
    return None, excluded


def upstream_copy_successor(root, path):
    """④ 上游副本：<根>/.claude/singlefs-ai-sop/<路径> 在磁盘上的（不进 git，只按磁盘判）：((副本路径, 依据), [说明]) 或 (None, [])。"""
    if broken("upstream-ignored"):
        return None, []
    bare = path.rstrip("/")
    copy = f"{UPSTREAM_COPY_DIRECTORY}/{bare}"
    if not exists(root, copy):
        return None, []
    copy += "/" if path.endswith("/") else ""
    why = f"这是上游 singlefs-ai-sop 仓里的路径，本仓副本在 {copy}，不进 git"
    note = f"说上游的句子保留原写法、登进 {KEPT_TABLE}（理由写「相对仓外副本的路径」）；说本仓副本的改成副本路径"
    return (copy, why), [note]


def suffix_successor(root, path):
    """④ 此刻被跟踪的文件（目录）里唯一以 /<路径> 结尾的那一份（排掉样本副本之后）：((路径, 依据) 或 None, 剩下几份, 排掉几份)。"""
    if broken("suffix-ignored"):
        return None, 0, 0
    bare = path.rstrip("/")
    pool = tracked_directories(root) if path.endswith("/") else tracked_files(root)
    matches, excluded = without_fixture_copies(path, [candidate for candidate in pool if candidate.endswith("/" + bare)])
    if len(matches) == 1 or (matches and broken("suffix-ambiguous-taken")):
        return (matches[0] + ("/" if path.endswith("/") else ""), f"此刻被跟踪的里只有这一份以 /{bare} 结尾"), len(matches), excluded
    return None, len(matches), excluded


def conclude(path, root=".", overlap_cap=DEFAULT_OVERLAP_CAP):
    """一条路径的结论（Conclusion）。路径相对 root，可带结尾的 / 表示目录。"""
    if exists(root, path):
        return Conclusion(path, PRESENT, (), None, "", ())
    registered = registered_successor(root, path)
    if registered:
        return Conclusion(path, REGISTERED, (registered,), None, "", ())
    notes = [] if is_repository(root) else ["被判的根不是 git 仓，历史与被跟踪的文件都取不到"]
    # ② 改名链
    current, chain, visited = path, [], {path.rstrip("/")}
    removal = last_removal(root, current)
    while removal is not None and len(chain) < RENAME_CHAIN_LIMIT:
        target = rename_target(root, removal, current, is_directory_path(root, current, removal))
        if target is None:
            break
        chain.append(f"{removal.short} {current} → {target}")
        if broken("rename-chain-stops") or exists(root, target):
            return Conclusion(path, RENAMED, ((target, "改名链 " + "；".join(chain)),), None, "", ())
        if target.rstrip("/") in visited:
            notes.append("改名链绕回了走过的路径，停在这里")
            break
        visited.add(target.rstrip("/"))
        current, removal = target, last_removal(root, target)
    if chain:
        notes.insert(0, "改名链 " + "；".join(chain) + f"，链尾 {current} 此刻也不在")
    # ③ 删了的（链尾那一条）
    if removal is None and ever_in_git(root, current, "HEAD"):
        code, _output = git(root, "cat-file", "-e", f"HEAD:{current.rstrip('/')}")
        if code == 0:
            removal = Removal(None, "HEAD", "")
    if removal is not None:
        directory = is_directory_path(root, current, removal)
        previous = f"{removal.short}^:{current.rstrip('/')}" if removal.full is not None else f"HEAD:{current.rstrip('/')}"
        successors = []
        if removal.full is not None and not directory and not broken("overlap-ignored"):
            overlapping, overlap_notes = overlap_successors(root, removal, current.rstrip("/"), overlap_cap)
            successors.extend(overlapping)
            notes.extend(overlap_notes)
        kind = DELETED_WITH_OVERLAP if successors else None
        same_name, excluded = same_name_successor(root, current, directory)
        notes.extend(filter(None, [fixture_note("找同名时", excluded)]))
        if same_name and same_name[0] not in {candidate for candidate, _why in successors}:
            successors.append(same_name)
            kind = kind or DELETED_WITH_SAME_NAME
        return Conclusion(path, kind or DELETED_WITHOUT_SUCCESSOR, tuple(successors), removal, previous, tuple(notes))
    # ④ 从没进过 git（或只在别的分支）
    if ever_in_git(root, path, "--all"):
        notes.append("只在别的分支进过 git（git log --all 找得到，HEAD 的历史里没有）")
    upstream, upstream_notes = upstream_copy_successor(root, path)
    if upstream:
        return Conclusion(path, UPSTREAM_COPY, (upstream,), None, "", tuple(notes + upstream_notes))
    found, matched, excluded = suffix_successor(root, path)
    notes.extend(filter(None, [fixture_note("按后缀找时", excluded)]))
    if found:
        return Conclusion(path, SUFFIX_UNIQUE, (found,), None, "", tuple(notes))
    if matched > 1:
        notes.append(f"以 /{path.rstrip('/')} 结尾的被跟踪的有 {matched} 份，不唯一")
    return Conclusion(path, NOT_FOUND, (), None, "", tuple(notes))


# ---------------------------------------------------------------- 命令行

def main(arguments):
    root, cap, paths = ".", DEFAULT_OVERLAP_CAP, []
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument in ("--root", "--overlap-cap"):
            if index + 1 >= len(arguments):
                print(f"  ✗ {argument} 后面缺值", file=sys.stderr)
                print("     → 怎么办：写成 --root <目录> 或 --overlap-cap <正整数>（用法在文件头）", file=sys.stderr)
                return 2
            value = arguments[index + 1]
            if argument == "--root":
                root = value
            elif value.isdigit() and int(value) > 0:
                cap = int(value)
            else:
                print(f"  ✗ --overlap-cap 要正整数，给的是 {value}", file=sys.stderr)
                print("     → 怎么办：写一个正整数，或不写取默认的 " + str(DEFAULT_OVERLAP_CAP), file=sys.stderr)
                return 2
            index += 2
            continue
        if argument.startswith("-"):
            print(f"  ✗ 认不出的选项 {argument}", file=sys.stderr)
            print("     → 怎么办：只认 --root、--overlap-cap、--selftest 与路径（文件头「用法」）", file=sys.stderr)
            return 2
        paths.append(argument)
        index += 1
    if not paths:
        print("  ✗ 一条路径都没给", file=sys.stderr)
        print("     → 怎么办：path-successor.py [--root 目录] <路径>…（路径相对 --root）", file=sys.stderr)
        return 2
    if not os.path.isdir(root):
        print(f"  ✗ --root 给的 {root} 不是目录", file=sys.stderr)
        print("     → 怎么办：给仓根（或要查的那棵树的根）", file=sys.stderr)
        return 2
    for path in paths:
        conclusion = conclude(path, root, cap)
        print(f"{path}\t{conclusion.kind}\t{conclusion.sentence()}")
    return 0


# ---------------------------------------------------------------- 自证

SELFTEST_ENVIRONMENT = {"GIT_AUTHOR_NAME": "selftest", "GIT_AUTHOR_EMAIL": "selftest@example.invalid",
                        "GIT_COMMITTER_NAME": "selftest", "GIT_COMMITTER_EMAIL": "selftest@example.invalid",
                        "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_SYSTEM": os.devnull}


class SelftestRepository:
    """临时目录里的一个小 git 仓：write 写文件、commit 提交、move 改名、remove 删；退出时整个目录删掉。"""

    def __enter__(self):
        self.root = tempfile.mkdtemp(prefix="path-successor-selftest.")
        self.run("init", "-q", "-b", "master")
        return self

    def __exit__(self, *_exception):
        shutil.rmtree(self.root, ignore_errors=True)
        TRACKED_CACHE.pop(os.path.realpath(self.root), None)

    def run(self, *arguments):
        environment = dict(os.environ)
        environment.update(SELFTEST_ENVIRONMENT)
        subprocess.run(["git", *arguments], cwd=self.root, env=environment, check=True, capture_output=True)

    def write(self, path, text):
        full = os.path.join(self.root, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(text)

    def commit(self, subject):
        self.run("add", "-A")
        self.run("commit", "-q", "-m", subject)
        TRACKED_CACHE.pop(os.path.realpath(self.root), None)

    def move(self, old, new):
        os.makedirs(os.path.dirname(os.path.join(self.root, new)), exist_ok=True)
        self.run("mv", old, new)

    def remove(self, path):
        self.run("rm", "-q", path)


def numbered_lines(label, count):
    return "".join(f"{label} 这一行写的是第 {index} 条足够长的内容，拿来比重合\n" for index in range(count))


def expect(conclusion, kind, successors, fragments=()):
    """判出的类别、后继路径与结论句里该有的片段：不对的交一句，对了交 None。"""
    got = [candidate for candidate, _why in conclusion.successors]
    if conclusion.kind != kind or got != list(successors):
        return f"类别 {conclusion.kind}、后继 {got}，答案是 {kind}、{list(successors)}（结论句：{conclusion.sentence()}）"
    missing = [fragment for fragment in fragments if fragment not in conclusion.sentence()]
    if missing:
        return f"结论句里没有 {missing}（结论句：{conclusion.sentence()}）"
    return None


def case_registered():
    with SelftestRepository() as repository:
        repository.write("new/place/item.txt", "搬过来的文件\n")
        repository.write(".claude/gate.d/path-moves.tsv", "old/place\tnew/place\t2026-10-30\trecords/2026-10-30-样本.md\n")
        repository.commit("基准")
        return expect(conclude("old/place/item.txt", repository.root), REGISTERED, ["new/place/item.txt"], ("path-moves.tsv 第 1 行",))


def case_renamed_twice():
    with SelftestRepository() as repository:
        repository.write("docs/first/a.md", numbered_lines("改名", 5))
        repository.commit("加 a")
        repository.move("docs/first/a.md", "docs/second/a.md")
        repository.commit("挪到 second")
        repository.move("docs/second/a.md", "docs/third/b.md")
        repository.commit("改名成 b")
        return expect(conclude("docs/first/a.md", repository.root), RENAMED, ["docs/third/b.md"], ("docs/second/a.md → docs/third/b.md",))


def case_deleted_with_overlap():
    with SelftestRepository() as repository:
        old = numbered_lines("旧布局", 10)
        repository.write("src/legacy_layout.rs", old)
        repository.write("src/low.rs", numbered_lines("无关", 4))
        repository.write("zz/far.rs", numbered_lines("远处", 4))
        repository.commit("基准")
        repository.remove("src/legacy_layout.rs")
        kept = "\n".join(old.split("\n")[:8]) + "\n"
        repository.write("src/system_configuration.rs", kept + numbered_lines("系统配置新写", 30))
        repository.write("src/low.rs", numbered_lines("无关", 4) + old.split("\n")[0] + "\n")
        repository.write("src/legacy_layout.txt", old + numbered_lines("扩展名不同", 30))
        repository.write("zz/far.rs", numbered_lines("远处改过", 4))
        repository.commit("拆 legacy_layout：系统配置搬到 system_configuration.rs")
        return expect(conclude("src/legacy_layout.rs", repository.root, overlap_cap=2), DELETED_WITH_OVERLAP, ["src/system_configuration.rs"],
                      ("重合 80%", "上限 2", "跳过 1 份", "^:src/legacy_layout.rs", "拆 legacy_layout"))


def case_deleted_with_same_name():
    with SelftestRepository() as repository:
        repository.write("scripts/kb/fixtures/facts.tsv", numbered_lines("样本副本", 3))
        repository.commit("基准")
        repository.remove("scripts/kb/fixtures/facts.tsv")
        repository.write("scripts/kb/facts.tsv", numbered_lines("正本另写", 3))
        repository.commit("删样本副本")
        return expect(conclude("scripts/kb/fixtures/facts.tsv", repository.root), DELETED_WITH_SAME_NAME, ["scripts/kb/facts.tsv"],
                      ("名叫 facts.tsv 的只有这一份",))


def case_same_name_beside_fixture_copy():
    with SelftestRepository() as repository:
        repository.write("notes/facts.tsv", numbered_lines("要删的事实表", 3))
        repository.write(".claude/gate.d/fixtures/sample-green/notes/facts.tsv", numbered_lines("门禁样本副本", 3))
        repository.write("data/facts.tsv", numbered_lines("真文件", 3))
        repository.commit("基准")
        repository.remove("notes/facts.tsv")
        repository.commit("删 notes/facts.tsv")
        return expect(conclude("notes/facts.tsv", repository.root), DELETED_WITH_SAME_NAME, ["data/facts.tsv"],
                      ("名叫 facts.tsv 的只有这一份", "找同名时排掉了 1 份门禁样本副本"))


def case_deleted_without_successor():
    with SelftestRepository() as repository:
        repository.write("notes/gone.md", numbered_lines("要删的", 3))
        repository.write("notes/kept.txt", "留着\n")
        repository.commit("基准")
        repository.remove("notes/gone.md")
        repository.commit("删 gone：不再需要")
        conclusion = conclude("notes/gone.md", repository.root)
        short = conclusion.removal.short if conclusion.removal else "?"
        return expect(conclusion, DELETED_WITHOUT_SUCCESSOR, [], (f"删于 {short}（删 gone：不再需要）", f"`{short}^:notes/gone.md`"))


def case_suffix_unique():
    with SelftestRepository() as repository:
        repository.write(".claude/kb/tooling.md", "工具页\n")
        repository.commit("基准")
        return expect(conclude("kb/tooling.md", repository.root), SUFFIX_UNIQUE, [".claude/kb/tooling.md"])


def case_upstream_copy():
    with SelftestRepository() as repository:
        repository.write(".gitignore", f"/{UPSTREAM_COPY_DIRECTORY}/\n")
        repository.write(".claude/scripts/x.sh", "本仓包装\n")
        repository.write(f"{UPSTREAM_COPY_DIRECTORY}/scripts/x.sh", "上游正本\n")
        repository.commit("基准")
        return expect(conclude("scripts/x.sh", repository.root), UPSTREAM_COPY, [f"{UPSTREAM_COPY_DIRECTORY}/scripts/x.sh"],
                      ("上游 singlefs-ai-sop 仓里的路径", f"本仓副本在 {UPSTREAM_COPY_DIRECTORY}/scripts/x.sh，不进 git", KEPT_TABLE, "相对仓外副本的路径", "说本仓副本的改成副本路径"))


def case_suffix_only_fixture_copy():
    with SelftestRepository() as repository:
        repository.write(".claude/gate.d/fixtures/doc-experiments.sh/translation-green/.claude/kb/decisions/01-样例.ja.md", "样本\n")
        repository.commit("基准")
        return expect(conclude(".claude/kb/decisions/01-样例.ja.md", repository.root), NOT_FOUND, [],
                      ("从没进过 git", "按后缀找时排掉了 1 份门禁样本副本"))


def case_suffix_beside_fixture_copy():
    with SelftestRepository() as repository:
        repository.write(".claude/gate.d/fixtures/sample-green/.claude/kb/tooling.md", "样本\n")
        repository.write(".claude/kb/tooling.md", "工具页\n")
        repository.commit("基准")
        return expect(conclude("kb/tooling.md", repository.root), SUFFIX_UNIQUE, [".claude/kb/tooling.md"],
                      ("按后缀找时排掉了 1 份门禁样本副本",))


def case_not_found():
    with SelftestRepository() as repository:
        repository.write("left/ghost/none.md", "左\n")
        repository.write("right/ghost/none.md", "右\n")
        repository.commit("基准")
        return expect(conclude("ghost/none.md", repository.root), NOT_FOUND, [], ("从没进过 git", "有 2 份，不唯一"))


def case_present():
    with SelftestRepository() as repository:
        repository.write("here/now.md", "在\n")
        repository.commit("基准")
        return expect(conclude("here/now.md", repository.root), PRESENT, [])


SELFTEST_CASES = (("registered", case_registered), ("renamed-twice", case_renamed_twice),
                  ("deleted-with-overlap", case_deleted_with_overlap), ("deleted-with-same-name", case_deleted_with_same_name),
                  ("same-name-beside-fixture-copy", case_same_name_beside_fixture_copy),
                  ("deleted-without-successor", case_deleted_without_successor), ("upstream-copy", case_upstream_copy),
                  ("suffix-unique", case_suffix_unique), ("suffix-only-fixture-copy", case_suffix_only_fixture_copy),
                  ("suffix-beside-fixture-copy", case_suffix_beside_fixture_copy),
                  ("not-found", case_not_found), ("present", case_present))
BREAK_NAMED_CASE = {"moves-ignored": "registered", "rename-chain-stops": "renamed-twice", "overlap-ignored": "deleted-with-overlap",
                    "same-name-ignored": "deleted-with-same-name", "previous-version-unreported": "deleted-without-successor",
                    "suffix-ignored": "suffix-unique", "suffix-ambiguous-taken": "not-found",
                    "fixtures-taken": "suffix-beside-fixture-copy", "upstream-ignored": "upstream-copy"}


def run_cases():
    """逐格跑：[(格名, 问题句或 None)]。每格自己的临时仓，互不依赖。"""
    results = []
    for name, case in SELFTEST_CASES:
        try:
            results.append((name, case()))
        except (subprocess.CalledProcessError, OSError) as error:
            results.append((name, f"造局面时出错：{error}"))
    return results


def selftest():
    mode = os.environ.get(BREAK_VARIABLE, "")
    if mode and mode not in BREAK_MODES:
        print(f"✗ 自检：{BREAK_VARIABLE}={mode} 认不得")
        print(f"  → 取值只有 {' '.join(BREAK_MODES)}（文件头「弄坏开关」一段）；平常跑 --selftest 不设它")
        return 2
    results = run_cases()
    failed = [(name, problem) for name, problem in results if problem]
    for name, problem in failed:
        print(f"✗ 自检 {name}：{problem}")  # gate-lint:detail
    if mode:
        if not failed:
            print(f"✗ 自检：弄坏开关 {mode} 打开了，自证仍然全绿 —— 自证看不出这一处")
            print(f"  → 看 {BREAK_NAMED_CASE[mode]} 那一格")
        else:
            print(f"  → 弄坏开关 {mode} 是故意的；不设 {BREAK_VARIABLE} 再跑 --selftest")
        return 1
    if failed:
        print("  → 改 path-successor.py 对应的那一步，再跑 --selftest")
        return 1
    # 每条开关单跑一次约一两秒，逐条串着跑；并行要另取线程数，省下的几秒不值
    caught = []
    for switch in BREAK_MODES:
        environment = dict(os.environ)
        environment[BREAK_VARIABLE] = switch
        run = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest"], env=environment, capture_output=True, text=True)
        named = f"✗ 自检 {BREAK_NAMED_CASE[switch]}："
        if run.returncode != 1 or named not in run.stdout:
            print(f"✗ 自检：弄坏开关 {switch} 打开之后没红在 {BREAK_NAMED_CASE[switch]} 那一格（退 {run.returncode}）")
            print("  → 看那一格与开关读的那一处；输出原文：" + run.stdout.strip().replace("\n", " / ")[:400])
            return 1
        caught.append(switch)
    print(f"✓ 自检通过：{len(results)} 格（{'、'.join(name for name, _problem in results)}）都判对；"
          f"弄坏开关 {len(caught)} 条各开一次、都红在点名的那一格（{' '.join(caught)}）")
    return 0


BREAK_VARIABLES = (BREAK_VARIABLE,)


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
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道：命令行以 --selftest 起头、或祖先里有自证的放行；
    都不是而设了开关的，点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or arguments[:1] == ["--selftest"] or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/kb/path-successor.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    sys.exit(main(sys.argv[1:]))
