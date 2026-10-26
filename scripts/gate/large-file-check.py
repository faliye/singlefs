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

退出码：0 都不超（乙可以是本次未跑）；1 有超的（逐个列路径与大小，乙另列提交号）；2 用法错；77 根不是 git 仓的顶层、或既没有跟踪文件也没有待检查的未推送提交。

弄坏开关 LARGE_FILE_CHECK_BREAK=<项>（逗号隔开可给几项；门禁那一格把 CODE_TOOLING_BREAK=committed-size-<项> 折成这里的 <项>）：
  tracked-half-skipped    甲不判：committed-file-size-red 判绿
  unpushed-half-skipped   乙不判：committed-file-size-unpushed-red、committed-file-size-detached-red 判绿
  limit-inclusive         恰好等于上限的也算超：committed-file-size-green（里面放了一个恰好等于门限的文件）判红
  no-upstream-as-pass     没有上游时乙当成判过、查了 0 笔：committed-file-size-no-upstream-green 里「乙（还没推的提交）本次未跑」那条 want 找不到
  detached-unresolved     游离的 HEAD 一律当没有上游：committed-file-size-detached-red 判绿
"""
import os
import stat
import subprocess
import sys

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
broken = set()


class GitFailure(Exception):
    pass


def git(root, *arguments, stdin_bytes=None, allow_failure=False):
    """在 root 里跑一条 git（路径不转义）；退出码非 0 时 allow_failure 给 None，否则抛 GitFailure。"""
    completed = subprocess.run(["git", "-c", "core.quotepath=false", "-C", root, *arguments], input=stdin_bytes,
                               capture_output=True)
    if completed.returncode != 0:
        if allow_failure:
            return None
        raise GitFailure(f"git {' '.join(arguments)} 退 {completed.returncode}：{completed.stderr.decode(errors='replace').strip()[:200]}")
    return completed.stdout


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
    commits_seen, blobs_seen, too_big, unreadable = set(), set(), {}, set()
    for _name, base in bases:
        commits = git(root, "rev-list", "--reverse", f"{base}..HEAD").decode().split()
        commits_seen.update(commits)
        objects = git(root, "rev-list", "--objects", "--no-object-names", f"{base}..HEAD").decode().splitlines()
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
              "不让超限的 blob 留在任何一笔里（删掉它的后一笔提交不够，推的时候那个 blob 照样上传）")
        return 1
    if tracked_count == 0 and commit_count == 0:
        print("  ! 本次无对象可判：仓里没有跟踪的文件，也没有待检查的未推送提交")
        return 77
    print(f"  ✓ 被提交的东西里没有超过 {limit_text(limit)}的文件：甲 查了 {tracked_count} 个跟踪的文件，乙 {second_text}")
    return 0


def main(arguments):
    limit, report_as, root = SINGLE_FILE_SIZE_LIMIT_BYTES, DEFAULT_REPORT_AS, None
    rest = list(arguments)
    while rest:
        argument = rest.pop(0)
        if argument == "--limit-bytes" and rest and rest[0].isdigit() and int(rest[0]) > 0:
            limit = int(rest.pop(0))
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


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
