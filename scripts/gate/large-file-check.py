#!/usr/bin/env python3
# admission: always 判的是此刻被判的树和还没推的提交，推与不推、压没压都在变，上一次的结论不替这一次作保
# run-condition: command git
"""被提交的东西里没有超过单文件上限的文件（门禁 code-tooling 的 committed-file-size 格的判法）。

上限 SINGLE_FILE_SIZE_LIMIT_BYTES 全仓只在这里定义：GitHub 的单文件推送硬上限 100 MiB，超过的推不上去，
事后只能重做提交。「超过」是严格大于：恰好等于上限的放行。

判两半：
  甲  被判的根里 git 跟踪的每个文件：暂存区里那一份 blob 的大小，与工作区里那个普通文件的大小，取大的那个
      （gate.sh --staged 的临时树里两者相同；不带 --staged 在工作区里跑时，改大了没暂存的文件也看得见）。
      子模块（gitlink）不判。
  乙  当前分支相对上游还没推上去的每笔提交里新增或改过的 blob（git rev-list --objects <上游>..HEAD）。
      拦两种甲看不见的：绕过门禁提交的（commit-tree 之类），和先提交了大文件又在后一笔提交里删掉的
      （HEAD 的树干净，推的时候那笔提交里的 blob 照样要上传）。红行带上最早带进这个 blob 的提交号。
      上游的认法：HEAD 在分支上就是这个分支的 @{upstream}；HEAD 是游离的（gate.sh --staged 的临时树）
      就取指着 HEAD 的本地分支各自的上游，每个上游各判一遍、结果并起来。
      没有上游（或还没有提交）时乙这一半本次未跑：打一行 `⊘`，设了 $GATE_NOT_RUN_FILE 就往里追加一行，
      甲照判；成功句里写明乙没跑，不报成乙判过。

用法：
    large-file-check.py [根] [--limit-bytes N] [--report-as <谁在报本次未跑>]
        根不给就判这份脚本所在的仓；根必须是 git 仓的顶层。
        --limit-bytes 只供样本把门限压小（门禁那一格只在样本根认 .large-file-limit，真仓与 --staged 不传），不改常量。
        --report-as 写进 $GATE_NOT_RUN_FILE 那一行的开头，缺省是 large-file-check.py。
    large-file-check.py <目录> --guard commit-index [--index-copy-directory <目录>] [--limit-bytes N]
    large-file-check.py <目录> --guard push [--remote <远端>] [--ref <对象名>]… [--known <对象名>]… [--judge-unknown] [--index-copy-directory <目录>] [--limit-bytes N]
        git 的 pre-commit、pre-merge-commit、prepare-commit-msg、pre-applypatch、pre-push 钩子（scripts/gate/large-file-git-hook.py）在提交、推送那一刻调：只判这一笔提交真要带进去的内容、这一次推送真要推出去的引用，
        标准输出只有一行 JSON（mode、limit、limit_text、checked、too_big（每项 path、size、where、commit、title）、unreadable、not_run、error；push 另有 remote、redo_base），退出码同下；
        用法错（退出码 2）的说明与出路写到标准错误、不打 JSON；不带 --guard 的用法错写到标准输出。目录是仓里任一目录，不必是顶层。两种模式：
          commit-index     这一笔提交要带进去的暂存区内容：暂存区相对 HEAD 新增或改过的 blob（还没有 HEAD 就是整个暂存区），where 是「暂存区」；
                           git commit -a、git commit <路径> 时 git 在调钩子之前已把要提交的内容放进钩子看到的索引（GIT_INDEX_FILE），所以只判索引就够；
          push             乙：--ref 给的每个对象名（钩子从标准输入读来的本地 sha）相对远端已有的提交还没有的提交里新增或改过的 blob
                           （git rev-list --objects <对象名…> --not <已有的>）。--ref 给的对象名标签剥到底之后是提交的照上面判；是树或 blob 的（指向 blob、树的轻量标签与带说明标签）
                           取它们能到的全部 blob，减去远端已有的（--known 与远端跟踪引用能到的全部对象），剩下超限的 where 写「引用指向的对象」、commit 为空。
                           已有的 = --known 给的对象名（钩子读到的远端引用当前值，本地有这个对象才用）加上
                           远端的跟踪引用（--remote 点名的是配置过的远端名取它；不给 --remote 取全部远端的；点名的是网址或路径写法、不是配置过的名字时一个都不取：
                           别的远端的跟踪引用说不出那个地址上有什么）；
                           两样都没有（远端没取回过、远端上也没有这些引用）时不知道远端已有什么：给了 --judge-unknown 就判这些对象能到的全部历史，不给就 not_run 写原因、退 0。
                           redo_base 是没推的提交接在已有的提交上的那一个（边界只有一个时给前 12 位，多个或没有是空串），重做提交时 git reset --soft 退到这里。
        暂存区的认法跟着环境变量 GIT_INDEX_FILE（相对路径在仓的顶层目录起算，与 git 自己一致）。判法只读：读的是索引文件的一份拷贝（git diff 顺手刷新并写回索引，
        GIT_OPTIONAL_LOCKS=0 也挡不住，写回落在拷贝上），用户的索引一个字节不写；拷贝放进 --index-copy-directory 给的目录（调用方建、调用方删，到点被杀也由它清），
        不给就自己在临时目录里建、收到 TERM / HUP / INT 与正常退出都删；git 的 diff.relative 配置不影响判法（起 git 一律带 -c diff.relative=false）。
        --break <项，逗号隔开> 只供 git 钩子的自证造红：real-index（直接读写用户的索引）、non-commit-skipped（--ref 里的树与 blob 不判）、
        already-not-subtracted（树与 blob 能到的 blob 不减远端已有的）；平常不给。
        上限与「超过」的判法同不带 --guard 时一份。这两种模式的样本与弄坏开关在 scripts/gate/large-file-git-hook.py 的 --selftest 里
        （真 git 仓里的 git commit、git push 各种写法），门禁 code-tooling 的 committed-file-size 格只判不带 --guard 的两半。

退出码：0 都不超（乙可以是本次未跑）；1 有超的（逐个列路径与大小，乙另列提交号）；2 用法错（含 --limit-bytes 的值不是 ASCII 十进制正整数）；
77 不带 --guard：根不是 git 仓的顶层，或既没有跟踪文件也没有待检查的未推送提交（--guard 下只有第一种：目录不在 git 仓里，空仓与没有提交的仓退 0）；
3 只在 --guard 下：git 没跑成或索引拷不了，大小判不了（JSON 的 error 写原因）；78 git 不在 PATH（开头的 preflight 按 run-condition: command git 拒，不打 JSON）。

弄坏开关（逗号隔开可给几项；这份脚本没有 --selftest，只在门禁样本根里认：门禁 code-tooling.sh 的 committed-file-size 格在样本根（当前目录有 .gate-cells 或 .gate-break-sample）里
把 CODE_TOOLING_BREAK=committed-size-<项> 折成这里的 <项>转来，平常跑设了退 2、点名只在自证里认。下面「点名的项」是 code-tooling.sh 的样本名）：
  LARGE_FILE_CHECK_BREAK=tracked-half-skipped  甲不判
      「committed-file-size-red」判绿
  LARGE_FILE_CHECK_BREAK=unpushed-half-skipped  乙不判
      「committed-file-size-unpushed-red」「committed-file-size-detached-red」判绿
  LARGE_FILE_CHECK_BREAK=limit-inclusive  恰好等于上限的也算超
      「committed-file-size-green」（里面放了一个恰好等于门限的文件）判红
  LARGE_FILE_CHECK_BREAK=no-upstream-as-pass  没有上游时乙当成判过、查了 0 笔
      「committed-file-size-no-upstream-green」里「乙（还没推的提交）本次未跑」那条 want 找不到
  LARGE_FILE_CHECK_BREAK=detached-unresolved  游离的 HEAD 一律当没有上游
      「committed-file-size-detached-red」判绿
"""
import contextlib
import json
import os
import re
import shutil
import signal
import stat
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

SINGLE_FILE_SIZE_LIMIT_BYTES = 104857600  # 100 MiB，GitHub 的单文件推送硬上限；全仓只在这里定义
GITLINK_MODE = "160000"
MEBIBYTE = 1048576
DEFAULT_REPORT_AS = "large-file-check.py"
BREAKS = ("tracked-half-skipped", "unpushed-half-skipped", "limit-inclusive", "no-upstream-as-pass", "detached-unresolved")
GUARD_BREAKS = ("real-index", "non-commit-skipped", "already-not-subtracted")   # --guard 的 --break 项，只供 git 钩子的自证造红（文件头「--guard 的 --break」）
NON_COMMIT_WHERE = "引用指向的对象"
broken = set()


class GitFailure(Exception):
    pass


def git(root, *arguments, stdin_bytes=None, allow_failure=False):
    """在 root 里跑一条 git（路径不转义；diff.relative 不随用户的配置，输出的路径一律相对仓的顶层目录）；退出码非 0 时 allow_failure 给 None，否则抛 GitFailure。"""
    configuration = ["-c", "core.quotepath=false", "-c", "diff.relative=false"]
    completed = subprocess.run(["git", *configuration, "-C", root, *arguments], input=stdin_bytes,
                               capture_output=True)
    if completed.returncode != 0:
        if allow_failure:
            return None
        raise GitFailure(f"git {' '.join(arguments)} 退 {completed.returncode}：{completed.stderr.decode(errors='replace').strip()[:200]}")
    return completed.stdout


def positive_integer(text):
    """ASCII 十进制写的正整数交 int；别的（空、带符号、str.isdigit() 认而 int() 不认的上标数字）交 None。"""
    return int(text) if re.fullmatch(r"[0-9]+", text) and int(text) > 0 else None


def exceeds(size, limit):
    return size >= limit if "limit-inclusive" in broken else size > limit


def readable(size):
    """字节数；够 1 MiB 的另括出 MiB 数（保留一位小数）。"""
    return f"{size} 字节（{size / MEBIBYTE:.1f} MiB）" if size >= MEBIBYTE else f"{size} 字节"


def limit_text(limit):
    """上限的写法：整 MiB 的括出 MiB 数，样本压小的门限只写字节数。"""
    return f"{limit} 字节（{limit // MEBIBYTE} MiB）" if limit % MEBIBYTE == 0 else f"{limit} 字节"


def object_sizes(root, names):
    """{对象名: (类型, 大小)}；读不到的对象交一个 ("missing", 0)。一次 cat-file --batch-check 全问完。"""
    names = sorted(set(names))
    if not names:
        return {}
    output = git(root, "cat-file", "--batch-check=%(objectname) %(objecttype) %(objectsize)",
                 stdin_bytes=("\n".join(names) + "\n").encode())
    found = {}
    for line in output.decode().splitlines():
        fields = line.split()
        if len(fields) == 3 and fields[1] != "missing":
            found[fields[0]] = (fields[1], int(fields[2]))
        elif fields:
            found[fields[0]] = ("missing", 0)
    return found


def tracked_entries(root):
    """索引里的跟踪条目 [(路径, blob 名)]，按路径去重（未合并的条目有几个阶段），不含子模块。"""
    output = git(root, "ls-files", "-s", "-z")
    entries = {}
    for record in output.split(b"\0"):
        if not record:
            continue
        meta, _, path = record.partition(b"\t")
        mode, name, _stage = meta.decode().split()
        if mode != GITLINK_MODE:
            entries.setdefault(os.fsdecode(path), name)
    return sorted(entries.items())


def judge_tracked(root, limit):
    """甲：返回 (查了几个文件, [(路径, 大小, 大的是哪一份)], [读不到的对象])。"""
    entries = tracked_entries(root)
    sizes = object_sizes(root, [name for _, name in entries])
    too_big, unreadable = [], []
    for path, name in entries:
        kind, blob_size = sizes.get(name, ("missing", 0))
        if kind == "missing":
            unreadable.append(path)
            continue
        size, where = blob_size, "暂存区"
        try:
            status = os.lstat(os.path.join(root, path))
            if stat.S_ISREG(status.st_mode) and status.st_size > size:
                size, where = status.st_size, "工作区"
        except OSError:
            pass
        if exceeds(size, limit):
            too_big.append((path, size, where))
    return len(entries), too_big, unreadable


def head_born(root):
    return git(root, "rev-parse", "--verify", "-q", "HEAD", allow_failure=True) is not None


def upstreams(root):
    """乙要对照的上游 [(名字, 提交号)] 与没有的原因。HEAD 在分支上取它的 @{upstream}；游离的取指着 HEAD 的本地分支各自的上游。"""
    if not head_born(root):
        return [], "仓里还没有任何提交"
    attached = git(root, "symbolic-ref", "-q", "HEAD", allow_failure=True)
    found = []
    if attached is not None:
        name = git(root, "rev-parse", "--abbrev-ref", "@{upstream}", allow_failure=True)
        target = git(root, "rev-parse", "--verify", "-q", "@{upstream}^{commit}", allow_failure=True)
        if name is not None and target is not None:
            found.append((name.decode().strip(), target.decode().strip()))
        if not found:
            return [], f"当前分支 {attached.decode().strip().removeprefix('refs/heads/')} 没有上游"
        return found, ""
    if "detached-unresolved" in broken:
        return [], "HEAD 是游离的"
    listing = git(root, "for-each-ref", "--points-at", "HEAD", "--format=%(refname)\t%(upstream)", "refs/heads")
    for line in listing.decode().splitlines():
        branch, _, upstream = line.partition("\t")
        if not upstream:
            continue
        target = git(root, "rev-parse", "--verify", "-q", f"{upstream}^{{commit}}", allow_failure=True)
        if target is not None:
            found.append((upstream.removeprefix("refs/remotes/").removeprefix("refs/heads/"), target.decode().strip()))
    if not found:
        return [], "HEAD 是游离的，指着它的本地分支都没有上游"
    return found, ""


def introducing_commits(root, commits, wanted):
    """{blob 名: (最早带进它的提交, 路径)}：commits 从旧到新，逐笔看它相对每个父提交新增或改过的条目。"""
    found = {}
    for commit in commits:
        output = git(root, "diff-tree", "-r", "-m", "--root", "--no-commit-id", "--no-renames", "-z", commit)
        parts = output.split(b"\0")
        index = 0
        while index + 1 < len(parts):
            if parts[index].startswith(b":"):
                new_name = parts[index][1:].split()[3].decode()
                if new_name in wanted and new_name not in found:
                    found[new_name] = (commit, os.fsdecode(parts[index + 1]))
                index += 2
            else:
                index += 1
        if len(found) == len(wanted):
            break
    return found


def judge_unpushed(root, limit, bases):
    """乙：返回 (查了几笔提交, 查了几个 blob, [(提交, 标题, 路径, 大小)], [读不到的对象])；每个上游各判一遍，结果并起来。"""
    return judge_revision_lists(root, limit, [[f"{base}..HEAD"] for _name, base in bases])


def judge_revision_lists(root, limit, revision_lists):
    """乙与 --guard push 共用：revision_lists 的每一组是一份 git rev-list 的参数（`<上游>..HEAD`，或 `<引用…> --not --remotes=<远端>`），每组各判一遍、结果并起来；
    返回值同 judge_unpushed。"""
    commits_seen, blobs_seen, too_big, unreadable = set(), set(), {}, set()
    for specs in revision_lists:
        commits = git(root, "rev-list", "--reverse", *specs).decode().split()
        commits_seen.update(commits)
        objects = git(root, "rev-list", "--objects", "--no-object-names", *specs).decode().splitlines()
        sizes = object_sizes(root, [line.split(" ", 1)[0] for line in objects])
        blobs = {name for name, (kind, _) in sizes.items() if kind == "blob"}
        unreadable.update(name for name, (kind, _) in sizes.items() if kind == "missing")
        blobs_seen.update(blobs)
        big = {name for name in blobs if exceeds(sizes[name][1], limit)}
        if big:
            introduced = introducing_commits(root, commits, big)
            for name in sorted(big):
                commit, path = introduced.get(name, ("", "（带进它的提交没找到，多半是合并提交里改过的）"))
                title = ""
                if commit:
                    title = git(root, "log", "-1", "--format=%s", commit).decode().strip()
                too_big[(commit, path, name)] = (commit, title, path, sizes[name][1])
    return len(commits_seen), len(blobs_seen), sorted(too_big.values(), key=lambda row: (row[0], row[2])), sorted(unreadable)


def report_not_run(report_as, reason):
    print(f"  ⊘ 乙（还没推的提交）本次未跑：{reason}；甲照判，乙这一半没有判过（设好上游再跑：git branch --set-upstream-to <远端>/<分支>）")
    side_channel = os.environ.get("GATE_NOT_RUN_FILE")
    if side_channel:
        with open(side_channel, "a", encoding="utf-8") as handle:
            handle.write(f"{report_as}：乙（还没推的提交）本次未跑，{reason}\n")


def judge(root, limit, report_as):
    top = git(root, "rev-parse", "--show-toplevel", allow_failure=True)
    if top is None or os.path.realpath(top.decode().strip()) != os.path.realpath(root):
        print(f"  ! 本次无对象可判：{root} 不是 git 仓的顶层，没有跟踪的文件可查")
        return 77
    problems = []
    tracked_count, tracked_big, tracked_unreadable = 0, [], []
    if "tracked-half-skipped" not in broken:
        tracked_count, tracked_big, tracked_unreadable = judge_tracked(root, limit)
    second_text, upstream_names = "", ""
    commit_count, blob_count, unpushed_big, unpushed_unreadable = 0, 0, [], []
    if "unpushed-half-skipped" in broken:
        second_text = "不判（弄坏开关 unpushed-half-skipped）"
    else:
        bases, why_not = upstreams(root)
        if bases:
            commit_count, blob_count, unpushed_big, unpushed_unreadable = judge_unpushed(root, limit, bases)
            upstream_names = "、".join(name for name, _ in bases)
            second_text = f"相对 {upstream_names} 查了还没推的 {commit_count} 笔提交里新增或改过的 {blob_count} 个 blob"
        elif "no-upstream-as-pass" in broken:
            second_text = "查了 0 笔提交（弄坏开关 no-upstream-as-pass：没有上游也当判过）"
        else:
            report_not_run(report_as, why_not)
            second_text = f"本次未跑（{why_not}）"
    if tracked_unreadable or unpushed_unreadable:
        for path in tracked_unreadable:
            print(f"  ✗ 甲：{path} 的 blob 在对象库里读不到，大小判不了")  # gate-lint:detail
        for name in unpushed_unreadable:
            print(f"  ✗ 乙：对象 {name} 在对象库里读不到，大小判不了")  # gate-lint:detail
        problems.append("对象库里有读不到的对象，这些没判")
    if tracked_big:
        print(f"  ✗ 甲：被提交的树里有 {len(tracked_big)} 个文件超过 {limit_text(limit)}，GitHub 的单文件推送上限，推不上去：")  # gate-lint:detail
        for path, size, where in sorted(tracked_big):
            print(f"     {path}  {readable(size)}（{where}）")  # gate-lint:detail
        problems.append("甲")
    if unpushed_big:
        print(f"  ✗ 乙：还没推的提交（相对 {upstream_names}）里新增或改过的 blob 有 {len(unpushed_big)} 个超过 {limit_text(limit)}，推的时候这几笔提交要整个上传：")  # gate-lint:detail
        for commit, title, path, size in unpushed_big:
            print(f"     提交 {commit[:10] or '（不明）'}（{title}）  {path}  {readable(size)}")  # gate-lint:detail
        problems.append("乙")
    if problems:
        print(f"  ✗ 提交里有超过单文件上限 {limit_text(limit)}的内容（{'、'.join(problems)}），逐个列在上面")
        print("     → 怎么办：对超限的文件 gzip -n -9 <文件> 压成 <文件>.gz 入库、删掉原文件；读它的地方改成认 .gz"
              "（例：scripts/experiment-apparatus/replay.sh 的 read_stored，.claude/gate.d/doc-experiments.sh 里用 gzip.open 读的那几处），"
              "引用照 .claude/rules/path-moves.md 全仓改名；乙列出的提交已经在没推的历史里，推之前重做那几笔提交，"
              "不让超限的 blob 留在任何一笔里（删掉它的后一笔提交不够，推的时候那个 blob 照样上传）；"
              "压完的 .gz 仍超上限的（随机数据、已压缩的格式压不小）不走 gzip：拆成每份不超过上限的分片入库（split -b 90M -d <文件> <文件>.part-，读它的地方按序拼回），"
              "或不入库（原文件放仓外、写进 .gitignore，在运行记录里登记路径、大小与 sha256）")
        return 1
    if tracked_count == 0 and commit_count == 0:
        print("  ! 本次无对象可判：仓里没有跟踪的文件，也没有待检查的未推送提交")
        return 77
    print(f"  ✓ 被提交的东西里没有超过 {limit_text(limit)}的文件：甲 查了 {tracked_count} 个跟踪的文件，乙 {second_text}")
    return 0


GUARD_MODES = ("commit-index", "push")
GUARD_USAGE = ("用法 large-file-check.py [根] [--limit-bytes <正整数>] [--report-as <名字>]；"
               "或 large-file-check.py <目录> --guard commit-index [--index-copy-directory <目录>] [--limit-bytes <正整数>]；"
               "或 large-file-check.py <目录> --guard push [--remote <远端>] [--ref <对象名>]… [--known <对象名>]… [--judge-unknown] [--index-copy-directory <目录>] [--limit-bytes <正整数>]")
GUARD_PUSH_OPTIONS = ("--remote", "--ref", "--known", "--judge-unknown")


def raw_entries(directory, *arguments):
    """git diff --raw 的记录 [(新模式, 新 blob 名, 状态字母, 相对仓顶层的路径)]；不认重命名（改名当删一个加一个）。
    用 git diff 而不用 diff-index：只碰了时间戳、内容没变的文件它不报成改过。"""
    output = git(directory, "diff", "-z", "--raw", "--no-renames", "--no-abbrev", "--no-ext-diff", *arguments)
    parts = output.split(b"\0")
    entries, index = [], 0
    while index + 1 < len(parts):
        if parts[index].startswith(b":"):
            _old_mode, new_mode, _old_name, new_name, status = parts[index][1:].decode().split()[:5]
            entries.append((new_mode, new_name, status[0], os.fsdecode(parts[index + 1])))
            index += 2
        else:
            index += 1
    return entries


def guard_result(mode, limit, checked, too_big, unreadable, not_run=""):
    return {"mode": mode, "limit": limit, "limit_text": limit_text(limit), "checked": checked,
            "too_big": too_big, "unreadable": unreadable, "not_run": not_run, "error": ""}


def guard_staged(top, limit):
    """commit-index：暂存区相对 HEAD 新增或改过的 blob（还没有 HEAD 就是整个暂存区）。"""
    if head_born(top):
        entries = [(path, name) for new_mode, name, status, path in raw_entries(top, "--cached", "HEAD")
                   if status != "D" and new_mode != GITLINK_MODE]
    else:
        entries = tracked_entries(top)
    sizes = object_sizes(top, [name for _, name in entries])
    too_big, unreadable = [], []
    for path, name in entries:
        kind, size = sizes.get(name, ("missing", 0))
        if kind == "missing":
            unreadable.append(path)
        elif exceeds(size, limit):
            too_big.append({"path": path, "size": size, "where": "暂存区", "commit": "", "title": ""})
    return guard_result("commit-index", limit, len(entries), too_big, unreadable)


def configured_remote(top, name):
    return git(top, "config", "--get", f"remote.{name}.url", allow_failure=True) is not None


def push_remote(top, remote_given):
    """要对照的远端名；空串是没有可认的远端名：没给（对照全部远端的跟踪引用），或点名的是网址、路径写法、没配置过的名字（不对照任何远端的跟踪引用）。"""
    return remote_given if remote_given and configured_remote(top, remote_given) else ""


def classify_objects(top, names):
    """names 里本地读得到的对象，标签剥到底（<名>^{}）之后按类型分：([提交号…], [树或 blob 的对象名…])，各自去重保持次序；读不到的略过：git push 自己会报错、什么都不推。"""
    commits, others = [], []
    for name in names:
        peeled = git(top, "rev-parse", "--verify", "-q", f"{name}^{{}}", allow_failure=True)
        if not peeled:
            continue
        resolved = peeled.decode().strip()
        kind = git(top, "cat-file", "-t", resolved, allow_failure=True)
        kind = kind.decode().strip() if kind else ""
        if kind == "commit":
            commits.append(resolved)
        elif kind in ("tree", "blob"):
            others.append(resolved)
    return list(dict.fromkeys(commits)), list(dict.fromkeys(others))


def redo_base(top, specs):
    """没推的提交接在远端已有的提交上的那一个（rev-list --boundary 的边界提交只有一个时，取它的前 12 位）：重做提交时 git reset --soft 退到这里。多个或没有交空串。"""
    output = git(top, "rev-list", "--boundary", *specs, allow_failure=True)
    boundary = [line[1:] for line in (output or b"").decode().splitlines() if line.startswith("-")]
    return boundary[0][:12] if len(boundary) == 1 else ""


def judge_non_commit_objects(top, limit, objects, already):
    """引用直接指向的树与 blob（标签剥到底之后不是提交）能到的全部 blob，减去远端已有的（already 是远端已有的对象名与 rev-list 的参数，能到的对象都算已有）：
    交 ([too_big 项], [读不到的对象名])。rev-list 的 --not 对不挂在提交底下的树与 blob 不起作用，所以已有的对象单独列出来相减；只在有超限的 blob 时才列。"""
    names = {name: "" for name in objects}
    for line in git(top, "rev-list", "--objects", *objects).decode().splitlines():
        name, _, path = line.partition(" ")
        names.setdefault(name, path)
    sizes = object_sizes(top, list(names))
    unreadable = sorted(name for name, (kind, _) in sizes.items() if kind == "missing")
    big = {name for name, (kind, size) in sizes.items() if kind == "blob" and exceeds(size, limit)}
    if big and already and "already-not-subtracted" not in broken:
        big -= set(git(top, "rev-list", "--objects", "--no-object-names", *already).decode().split())
    too_big = [{"path": names[name] or f"（没有路径的 blob {name[:12]}）", "size": sizes[name][1], "where": NON_COMMIT_WHERE, "commit": "", "title": ""} for name in sorted(big)]
    return too_big, unreadable


def guard_pushed(top, limit, options):
    """push：options["refs"] 里每个对象名相对远端已有的提交还没有的提交里新增或改过的 blob。已有的 = options["known"]（本地读得到的）加远端的跟踪引用
    （--not --remotes=<远端>；没点名远端的取全部远端的）；点名的是网址或路径写法（不是配置过的远端名）时不取任何跟踪引用，它们说的是别的远端有什么，不是这个地址上有什么；
    两样都没有（没取回过）时不知道远端已有什么：judge_unknown 给了就判这些对象能到的全部历史，没给不判、not_run 写原因。
    标签剥到底之后是树或 blob 的引用（classify_objects）不在提交的历史里：取它们能到的全部 blob，减去远端已有的对象能到的（judge_non_commit_objects）。"""
    remote = push_remote(top, options["remote"])
    address_only = bool(options["remote"]) and not remote
    tips, others = classify_objects(top, options["refs"])
    if "non-commit-skipped" in broken:
        others = []
    if not tips and not others:
        return {**guard_result("push", limit, 0, [], []), "remote": remote}
    already, known_others = classify_objects(top, options["known"])
    tracking = "" if address_only else git(top, "for-each-ref", "--count=1", "--format=%(refname)", f"refs/remotes/{remote}/" if remote else "refs/remotes/").strip()
    if tracking:
        already.append(f"--remotes={remote}" if remote else "--remotes")
    if not already and not known_others and not options["judge_unknown"]:
        unknown = f"本地没有任何关于 {options['remote']} 的记录" if address_only else f"本地没有{'远端 ' + remote + ' 的' if remote else ''}任何远端跟踪引用"
        return {**guard_result("push", limit, 0, [], [], not_run=f"{unknown}，不知道远端已有什么"), "remote": remote}
    commit_count, too_big, unreadable, specs = 0, [], [], []
    if tips:
        specs = [*tips, "--not", *already] if already else list(tips)
        commit_count, _blob_count, big, unreadable = judge_revision_lists(top, limit, [specs])
        too_big = [{"path": path, "size": size, "where": "没推的提交", "commit": commit, "title": title} for commit, title, path, size in big]
    redo = redo_base(top, specs) if too_big else ""
    if others:
        extra_big, extra_unreadable = judge_non_commit_objects(top, limit, others, [*already, *known_others])
        too_big += extra_big
        unreadable = sorted({*unreadable, *extra_unreadable})
    return {**guard_result("push", limit, commit_count, too_big, unreadable), "remote": remote, "redo_base": redo}


def effective_index_path(top):
    """这一次 git 读写的索引文件：GIT_INDEX_FILE 设了的取它（相对路径 git 在仓的顶层目录起算），没设的是仓的 git 目录（链接工作树各一份）里的 index。"""
    given = os.environ.get("GIT_INDEX_FILE")
    if given:
        return given if os.path.isabs(given) else os.path.join(top, given)
    return os.path.join(git(top, "rev-parse", "--absolute-git-dir").decode().strip(), "index")


@contextlib.contextmanager
def private_index(top, copy_directory=None):
    """判法读的是用户索引的一份拷贝：git diff（不带 --cached）会顺手刷新并写回索引，GIT_OPTIONAL_LOCKS=0 也挡不住；判法只读，一个字节都不写用户的索引。
    索引文件不存在的（新仓、私有索引还没建）拷贝也不存在，git 当空索引。拷贝放进 copy_directory（调用方建、调用方删；这里只删自己放的那一份），不给就自己建临时目录，
    正常退出与收到 TERM / HUP / INT（先转成 SystemExit，让这里的清理跑完）都删。弄坏开关 --break real-index 下直接用用户的索引。"""
    if "real-index" in broken:
        yield
        return
    source = effective_index_path(top)
    directory = copy_directory or tempfile.mkdtemp(prefix="large-file-check-index-")
    previous = os.environ.get("GIT_INDEX_FILE")

    def stop_cleanly(signal_number, _frame):
        raise SystemExit(128 + signal_number)

    handlers = {name: signal.signal(name, stop_cleanly) for name in (signal.SIGTERM, signal.SIGHUP, signal.SIGINT)}
    copy = os.path.join(directory, "index")
    try:
        if os.path.isfile(source):
            shutil.copyfile(source, copy)
        os.environ["GIT_INDEX_FILE"] = copy
        yield
    finally:
        if previous is None:
            os.environ.pop("GIT_INDEX_FILE", None)
        else:
            os.environ["GIT_INDEX_FILE"] = previous
        for name, handler in handlers.items():
            signal.signal(name, handler)
        if copy_directory:
            with contextlib.suppress(FileNotFoundError):
                os.remove(copy)
        else:
            shutil.rmtree(directory, ignore_errors=True)


def guard(directory, limit, mode, options):
    """--guard：交 (退出码, 要打成一行 JSON 的对象)；options 是 main_guard 解出来的几个键（push 的 remote、refs、known、judge_unknown，与 index_copy_directory）。"""
    top = git(directory, "rev-parse", "--show-toplevel", allow_failure=True)
    if top is None:
        return 77, guard_result(mode, limit, 0, [], [], not_run=f"{directory} 不在 git 仓里")
    top = os.path.realpath(top.decode().strip())
    try:
        with private_index(top, options["index_copy_directory"]):
            result = guard_staged(top, limit) if mode == "commit-index" else guard_pushed(top, limit, options)
    except (GitFailure, OSError) as error:
        return 3, {**guard_result(mode, limit, 0, [], []), "error": str(error)}
    return (1 if result["too_big"] else 0), result


def main_guard(arguments):
    """--guard 的参数解析与输出：交退出码，标准输出只打一行 JSON；参数不对交 2（说明走标准错误，不污染那一行）。"""
    limit, directory, mode = SINGLE_FILE_SIZE_LIMIT_BYTES, None, None
    options = {"remote": "", "refs": [], "known": [], "judge_unknown": False, "index_copy_directory": None}
    rest = list(arguments)
    while rest:
        argument = rest.pop(0)
        if argument == "--limit-bytes" and rest and positive_integer(rest[0]):
            limit = positive_integer(rest.pop(0))
        elif argument == "--guard" and rest and rest[0] in GUARD_MODES:
            mode = rest.pop(0)
        elif argument == "--remote" and rest:
            options["remote"] = rest.pop(0)
        elif argument == "--ref" and rest:
            options["refs"].append(rest.pop(0))
        elif argument == "--known" and rest:
            options["known"].append(rest.pop(0))
        elif argument == "--judge-unknown":
            options["judge_unknown"] = True
        elif argument == "--index-copy-directory" and rest:
            options["index_copy_directory"] = rest.pop(0)
        elif argument == "--break" and rest:
            broken.update(item for item in rest.pop(0).split(",") if item in GUARD_BREAKS)
        elif not argument.startswith("-") and directory is None:
            directory = argument
        else:
            print(f"  ✗ 认不得的参数：{argument}", file=sys.stderr)
            print(f"     → 怎么办：{GUARD_USAGE}", file=sys.stderr)
            return 2
    given_for_push = bool(options["remote"] or options["refs"] or options["known"] or options["judge_unknown"])
    copy_directory = options["index_copy_directory"]
    if directory is None or not os.path.isdir(directory) or (mode != "push" and given_for_push) or (copy_directory is not None and not os.path.isdir(copy_directory)):
        print(f"  ✗ --guard 要给仓里的一个目录；{'、'.join(GUARD_PUSH_OPTIONS)} 只在 push 下认；--index-copy-directory 要给已有的目录", file=sys.stderr)
        print(f"     → 怎么办：{GUARD_USAGE}", file=sys.stderr)
        return 2
    broken.update(item for item in os.environ.get("LARGE_FILE_CHECK_BREAK", "").split(",") if item in BREAKS)
    code, result = guard(os.path.realpath(directory), limit, mode, options)
    print(json.dumps(result))   # 默认转义非 ASCII：路径里有不是 UTF-8 的字节也打得出来
    return code


def main(arguments):
    if "--guard" in arguments:
        return main_guard(arguments)
    limit, report_as, root = SINGLE_FILE_SIZE_LIMIT_BYTES, DEFAULT_REPORT_AS, None
    rest = list(arguments)
    while rest:
        argument = rest.pop(0)
        if argument == "--limit-bytes" and rest and positive_integer(rest[0]):
            limit = positive_integer(rest.pop(0))
        elif argument == "--report-as" and rest and rest[0].strip():
            report_as = rest.pop(0)
        elif not argument.startswith("-") and root is None:
            root = argument
        else:
            print(f"  ✗ 认不得的参数：{argument}")
            print("     → 怎么办：用法 large-file-check.py [根] [--limit-bytes <正整数>] [--report-as <名字>]")
            return 2
    root = os.path.realpath(root) if root else REPOSITORY_ROOT
    if not os.path.isdir(root):
        print(f"  ✗ 根不在：{root}")
        print("     → 怎么办：给仓的顶层目录，或不给参数判这份脚本所在的仓")
        return 2
    broken.update(item for item in os.environ.get("LARGE_FILE_CHECK_BREAK", "").split(",") if item in BREAKS)
    try:
        return judge(root, limit, report_as)
    except GitFailure as error:
        print(f"  ✗ git 没跑成，大小判不了：{error}")
        print("     → 怎么办：按 git 的报错修好仓库状态（索引没坏、对象库读得到）再跑；判不了的不能当成没有超限的文件")
        return 1


BREAK_VARIABLES = ("LARGE_FILE_CHECK_BREAK",)


def settle_break_switches():
    """弄坏开关只在门禁样本根里认（checks-owed.md C622），参数解析之前先过这一道：BREAK_VARIABLES 里有设了的，而当前目录不是样本根
    （没有 .gate-cells 与 .gate-break-sample，或设了 GATE_STAGED_FROM），点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    sample_root = (os.path.exists(".gate-cells") or os.path.exists(".gate-break-sample")) and not os.environ.get("GATE_STAGED_FROM")
    if not setting or sample_root:
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（门禁样本根里由 code-tooling.sh 转来）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 CODE_TOOLING_BREAK=committed-size-<项> 喂 code-tooling.sh 的样本（文件头）", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches()
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
