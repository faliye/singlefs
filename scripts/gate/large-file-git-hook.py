#!/usr/bin/env python3
# admission: always 判的是 git 这一刻要提交的暂存区、要推出去的引用，上一次的结论不替这一次作保
# run-condition: command git python3
# gate-similar: large-file-check.py 判法与上限常量只在那一份（--guard commit-index、--guard push），这里是 git 钩子那一侧：读钩子的入参与标准输入、先调私有的提交窗口闸、起判法、写拒绝说明；门禁 code-tooling 的 committed-file-size 格判整个树，两处各管一个时点，并到一起不行
# gate-similar: git-hooks-install.py 它装钩子与核装没装上，这里是装好之后钩子自己跑的判法
"""git 的 pre-commit、pre-merge-commit、prepare-commit-msg、pre-applypatch、pre-push 钩子的判法：这一笔提交（含 git merge 生成的合并提交、cherry-pick、revert、rebase 重放出来的提交）
与 git am 打上的补丁真要进的内容、这一次推送真要推出去的每个引用里有没有超过单文件上限的文件。

scripts/gate/git-hooks/ 下五份钩子只 exec 这一份；钩子经 core.hooksPath（主工作树钩子目录的绝对路径）生效（装与核对：scripts/gate/git-hooks-install.py）。
上限与判「超过」的函数只在 scripts/gate/large-file-check.py 的 --guard，这里起它、读它的 JSON。

用法（git 起，不手敲）：
    large-file-git-hook.py pre-commit [<git 给的参数…>]
        先调私有的提交窗口闸（$(git rev-parse --git-common-dir)/hooks/pre-commit，在而且可执行才调；设了 core.hooksPath 之后 git 不再读 .git/hooks，
        这里接上），它退非 0 就照它的退出码拒，大文件不判；再判暂存区（large-file-check.py --guard commit-index）：git commit -a、git commit <路径>、
        --amend 时 git 在调钩子之前已把要提交的内容放进钩子看到的索引（GIT_INDEX_FILE），判这份索引就是判这一笔提交真要进的内容。
        判过且没超限的，把 <HEAD 的提交号>:<git ls-files -s -z 输出的 sha256>（索引的内容，不含时间戳）记进 <git 目录>/large-file-judged-index，同一笔提交接着起的 prepare-commit-msg 对得上就不再判。
    large-file-git-hook.py pre-merge-commit [<git 给的参数…>]
        git merge 生成合并提交时 git 起它，不起 pre-commit：与 pre-commit 同一套，先调私有的提交窗口闸（合并提交也是提交，窗口外不放行），再判暂存区。
        git 在调它之前已把合并结果放进索引（GIT_INDEX_FILE 指到它），判的是合并结果相对 HEAD（第一个父提交）新增或改过的 blob。
        拒了之后合并停在没提交的状态（MERGE_HEAD 还在）：改完用 git commit 收尾（起 pre-commit，再判一遍），或 git merge --abort。
        不生成合并提交的不起它：快进（带来的提交由 pre-push 判）、--squash 与 --no-commit（随后的 git commit 起 pre-commit）；冲突解完用 git commit 收尾的同样走 pre-commit。
    large-file-git-hook.py prepare-commit-msg <说明文件> [<来源> [<提交号>]]
        git 在写好默认说明之后、起编辑器之前起它，git commit --no-verify 与 git merge --no-verify 挡不住它；git commit、git merge 生成合并提交、cherry-pick、revert、
        rebase 逐笔重放与冲突后的 --continue 都起它。退非 0 时那一笔提交不生成。判的是同一份暂存区（与 pre-commit 同一个判法），
        不调私有的提交窗口闸（窗口闸管 git commit 本身）；pre-commit 或 pre-merge-commit 刚判过同一份暂存区（凭据对得上）的不重复判。
        来源是 merge 的按合并提交写说明；<git 目录>里有 CHERRY_PICK_HEAD、REVERT_HEAD、rebase-merge、rebase-apply、sequencer 的按重放写说明（出路写 --continue 与 --abort）。
    large-file-git-hook.py pre-applypatch
        git am 打完补丁、生成提交之前起它（git am --no-verify 挡得住它，只靠 pre-push 兜）；判的是 git am 放进索引的内容。退非 0 时那一笔补丁不提交，git am 停下。
    large-file-git-hook.py pre-push <远端名> <远端地址>      标准输入是 git 给的行 `<本地引用> <本地 sha> <远端引用> <远端 sha>`
        照这几行判：每个本地 sha 相对远端已有的提交（行里的远端 sha，加推送目标那个远端的跟踪引用）还没有的提交里新增或改过的 blob；删除引用的行（本地 sha 全是 0）不判；
        本地 sha 标签剥到底之后是树或 blob 的（指向 blob、树的标签）取它们能到的全部 blob，减去远端已有的；
        推送目标写成网址或路径（不是配置过的远端名）时，别的远端的跟踪引用不算它已有的，只认行里的远端 sha；
        远端没取回过、行里的远端 sha 也全是 0（新引用）时不知道远端已有什么，判这些 sha 能到的全部历史（large-file-check.py --guard push --judge-unknown）。
        不猜 refspec、push.default、remote.<名>.push：git 解出来的引用都在标准输入里。
    large-file-git-hook.py --selftest [<格名>…]
        真 git 仓里的 git commit、git merge、git am、cherry-pick、revert、rebase 与 git push 各种写法。每一格自己建临时目录与仓，格与格互不依赖，按每台默认重活线程数并行；
        不点名格就全跑，再把文件头每条弄坏开关逐个打开、各格并进同一个线程池核红在点名的格；点名了格的只跑那几格（单跑与整份跑判定相同，不跑开关轮）。
        设了 LARGE_FILE_GIT_HOOK_BREAK=<项> 时只跑这一条开关下的格（不跑开关轮），红的格逐条以「✗ 自证 <格名>：」起头。

没接的提交方式：commit-tree 加 update-ref 与 git stash 没有钩子可接，带来的超限文件由 pre-push 拦。

超限的拒绝（退出码 1，git 据此中止这一次提交或推送）：写原因与出路到标准错误：超限的文件逐个 `gzip -n -9 -k <文件>`、`git rm --cached <文件>`、`git add <文件>.gz`，
做完重新提交；压完仍超的写另一条路（分片入库，或不入库）；已经在没推的提交里的写整套重做命令（`git reset --soft <远端已有的最新提交>`、各文件重做、`git commit`、再推）；
引用直接指向的 blob 或树里的写不推这个引用、或压成 .gz 放进普通提交再打标签。
判不了的（判法起不来、超过 JUDGE_SECONDS 秒没判完、退出码不是 0 与 1、输出读不成 JSON、对象库里有读不到的对象、没取回过又没要求全判的远端）放行，
往标准错误写一行 `⊘` 说没判成与手动查的命令；不静默放过，也不因为判法自己出毛病挡住提交。判法放进它自己的会话起，到点先 TERM 再 KILL 整组，
索引拷贝放进本脚本建的目录、不论怎么结束都由本脚本删。

窗口外提交：私有的提交窗口闸认环境变量 SINGLEFS_COMMIT_OUTSIDE_WINDOW=1（确需在窗口外提交时显式设置）放行窗口，大文件照判；
绕开钩子的 git commit --no-verify、-n、`-c core.hooksPath=…` 由检出钩子 bash-command-detector.sh 的 ㉒ 拦。

退出码：0 放行；1 拒；2 用法错；78 准入没满足（preflight 拒，不判）；私有窗口闸退非 0 时照它的退出码。

自证用的环境变量只在自证标记在的时候认（SELFTEST_VARIABLE 点名一个目录，目录里有 SELFTEST_MARKER_NAME 那个文件；自证给它建的临时目录建好、删净，标记跟着在、跟着没），
祖先进程的命令行里带 --selftest 之类的字样不算标记：LARGE_FILE_GIT_HOOK_LIMIT_BYTES 把门限压小、LARGE_FILE_GIT_HOOK_SECONDS 把判法时限压短、
LARGE_FILE_GIT_HOOK_JUDGE_PATH 给的目录放在判法进程的 PATH 最前面（自证在那里放假 git；git 起钩子时先把自己的 exec-path 放到 PATH 最前，那里有一份真 git，
放在 git 命令自己 PATH 前面的假 git 到不了钩子起的判法）、LARGE_FILE_GIT_HOOK_BREAK 弄坏开关。

弄坏开关 LARGE_FILE_GIT_HOOK_BREAK=<项>（逗号隔开可给几项；只给 --selftest 证红用，平常设了退 2、点名只在自证标记在的时候认；
红的格逐条以「✗ 自证 <格名>：」起头；单跑一条开关：LARGE_FILE_GIT_HOOK_BREAK=<项> python3 scripts/gate/large-file-git-hook.py --selftest）：
  LARGE_FILE_GIT_HOOK_BREAK=pre-commit-judge-skipped  pre-commit 调完私有闸之后不判暂存区
      「✗ 自证 commit-staged-big-refused：」「✗ 自证 commit-judge-reads-index-copy：」「✗ 自证 commit-all-grown-refused：」「✗ 自证 commit-paths-grown-refused：」「✗ 自证 private-hook-then-judge：」「✗ 自证 merge-no-commit-then-commit-refused：」「✗ 自证 judge-timeout-leaves-nothing：」
  LARGE_FILE_GIT_HOOK_BREAK=pre-merge-commit-judge-skipped  pre-merge-commit 调完私有闸之后不判暂存区
      「✗ 自证 merge-big-refused：」
  LARGE_FILE_GIT_HOOK_BREAK=private-hook-skipped  不调私有的提交窗口闸（pre-commit 与 pre-merge-commit 都不调）
      「✗ 自证 private-hook-refusal-first：」「✗ 自证 private-hook-then-judge：」「✗ 自证 merge-private-hook-window：」
  LARGE_FILE_GIT_HOOK_BREAK=prepare-commit-msg-judge-skipped  prepare-commit-msg 不判暂存区
      「✗ 自证 commit-no-verify-refused-by-prepare-commit-msg：」「✗ 自证 stale-stamp-not-trusted：」「✗ 自证 merge-no-verify-refused-by-prepare-commit-msg：」「✗ 自证 cherry-pick-refused-by-prepare-commit-msg：」「✗ 自证 revert-restoring-big-refused：」「✗ 自证 rebase-conflict-then-continue-refused：」「✗ 自证 rebase-i-edit-then-continue-directly-refused：」「✗ 自证 real-prepare-commit-msg-shim-no-verify-refused：」
  LARGE_FILE_GIT_HOOK_BREAK=prepare-commit-msg-stamp-ignored  prepare-commit-msg 不看 pre-commit 留下的凭据、一律再判一遍
      「✗ 自证 commit-judged-once：」
  LARGE_FILE_GIT_HOOK_BREAK=stale-stamp-trusted  prepare-commit-msg 只要凭据文件在就当判过，不核 HEAD 与索引字节
      「✗ 自证 stale-stamp-not-trusted：」
  LARGE_FILE_GIT_HOOK_BREAK=pre-applypatch-judge-skipped  pre-applypatch 不判 git am 放进索引的内容
      「✗ 自证 am-big-refused：」「✗ 自证 real-pre-applypatch-shim-am-refused：」
  LARGE_FILE_GIT_HOOK_BREAK=push-reads-current-branch-only  pre-push 不看标准输入里的行，只判 HEAD
      「✗ 自证 push-clean-branch-passes：」「✗ 自证 push-other-branch-refused：」「✗ 自证 push-all-refused：」「✗ 自证 push-tags-refused：」「✗ 自证 push-new-name-refspec-refused：」「✗ 自证 push-wildcard-refspec-refused：」「✗ 自证 push-remote-already-has-big-passes：」「✗ 自证 push-to-path-remote-already-has-big-passes：」「✗ 自证 tag-to-blob-push-refused：」「✗ 自证 tag-to-tree-push-refused：」「✗ 自证 annotated-tag-to-blob-push-refused：」「✗ 自证 lightweight-tag-to-tree-push-refused：」
  LARGE_FILE_GIT_HOOK_BREAK=unknown-remote-not-judged  没取回过的远端当成判过（不带 --judge-unknown）
      「✗ 自证 push-never-fetched-refused：」「✗ 自证 push-to-url-refused：」「✗ 自证 push-to-path-another-remote-has-it-refused：」
  LARGE_FILE_GIT_HOOK_BREAK=remote-sha-ignored  不用行里的远端 sha 当「远端已有」
      「✗ 自证 push-remote-already-has-big-passes：」「✗ 自证 push-to-path-remote-already-has-big-passes：」
  LARGE_FILE_GIT_HOOK_BREAK=push-target-not-named  不把推送目标的名字交给判法（判法对照全部远端的跟踪引用，别的远端有的提交也算推送目标已有的）
      「✗ 自证 push-to-path-another-remote-has-it-refused：」
  LARGE_FILE_GIT_HOOK_BREAK=push-non-commit-objects-skipped  引用指向的树与 blob（标签剥到底之后不是提交）不判（--break non-commit-skipped）
      「✗ 自证 tag-to-blob-push-refused：」「✗ 自证 tag-to-tree-push-refused：」「✗ 自证 annotated-tag-to-blob-push-refused：」「✗ 自证 lightweight-tag-to-tree-push-refused：」
  LARGE_FILE_GIT_HOOK_BREAK=push-known-blobs-not-subtracted  树与 blob 能到的 blob 不减远端已有的（--break already-not-subtracted）
      「✗ 自证 tag-to-blob-remote-already-has-passes：」
  LARGE_FILE_GIT_HOOK_BREAK=index-copy-kept  判法到点被停之后不删索引拷贝所在的目录
      「✗ 自证 judge-timeout-leaves-nothing：」
  LARGE_FILE_GIT_HOOK_BREAK=judge-writes-user-index  判法直接读写用户的索引（--break real-index）
      「✗ 自证 commit-judge-reads-index-copy：」
  LARGE_FILE_GIT_HOOK_BREAK=hooks-path-written-relative  自证里装钩子的那一步把 core.hooksPath 写成相对路径（要判的几种仓位置里相对路径找不到钩子目录，钩子一个都不跑）
      「✗ 自证 old-commit-linked-worktree-commit-refused：」「✗ 自证 old-commit-linked-worktree-push-refused：」「✗ 自证 sparse-checkout-commit-refused：」「✗ 自证 work-tree-option-commit-refused：」「✗ 自证 git-dir-option-push-from-outside-refused：」「✗ 自证 git-dir-variable-push-from-subdirectory-refused：」
  LARGE_FILE_GIT_HOOK_BREAK=selftest-marker-granted-to-plain-run  自证里「平常跑」的那两格也带上自证标记（钩子认标记认得太宽）
      「✗ 自证 selftest-word-in-message-limit-ignored：」「✗ 自证 break-switch-without-marker-refused：」
"""
import contextlib
import hashlib
import json
import os
import re
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

sys.path.insert(0, os.path.join(REPOSITORY_ROOT, "scripts", "hosts"))
import heavy_threads  # noqa: E402

HOOK_SCRIPT = os.path.realpath(__file__)
CHECKER = os.path.join(SCRIPT_DIRECTORY, "large-file-check.py")
HOOKS_DIRECTORY = os.path.join(SCRIPT_DIRECTORY, "git-hooks")
JUDGE_SECONDS = 600           # 判法每一次起的时限；没取回过的远端判全部历史时最慢，到点放行并写一行 ⊘
STOP_GRACE_SECONDS = 3        # 到点先 TERM，过这么久还没走再 KILL 整组
LISTED = 5                    # 拒绝说明里最多逐个列几个文件
LIMIT_VARIABLE = "LARGE_FILE_GIT_HOOK_LIMIT_BYTES"
SECONDS_VARIABLE = "LARGE_FILE_GIT_HOOK_SECONDS"
JUDGE_PATH_VARIABLE = "LARGE_FILE_GIT_HOOK_JUDGE_PATH"
BREAK_VARIABLE = "LARGE_FILE_GIT_HOOK_BREAK"
SELFTEST_VARIABLE = "LARGE_FILE_GIT_HOOK_SELFTEST_DIRECTORY"
SELFTEST_MARKER_NAME = ".large-file-git-hook-selftest-marker"
STAMP_NAME = "large-file-judged-index"
REPLAY_STATE_NAMES = ("CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply", "sequencer")
BREAKS = ("pre-commit-judge-skipped", "pre-merge-commit-judge-skipped", "private-hook-skipped", "prepare-commit-msg-judge-skipped", "prepare-commit-msg-stamp-ignored",
          "stale-stamp-trusted", "pre-applypatch-judge-skipped", "push-reads-current-branch-only", "unknown-remote-not-judged", "remote-sha-ignored", "push-target-not-named",
          "push-non-commit-objects-skipped", "push-known-blobs-not-subtracted", "index-copy-kept", "judge-writes-user-index", "hooks-path-written-relative",
          "selftest-marker-granted-to-plain-run")
NO_GZIP_WAY = ("压完的 .gz 仍超过上限的（随机数据、已压缩的格式压不小）不再 gzip：拆成每份不超过上限的分片入库（用「split -b 90M -d <文件> <文件>.part-」，读它的地方按序拼回），"
               "或不入库（原文件放仓外、写进 .gitignore，在运行记录里登记路径、大小与 sha256）")
READ_STORED_EXAMPLE = "scripts/experiment-apparatus/replay.sh 的 read_stored"
KIND_NAMES = {"commit": "提交", "merge": "合并提交", "replay": "重放出来的提交", "am": "补丁提交（git am）", "push": "推送"}
WHAT_IS_JUDGED = {"commit": "这一笔提交要带进去的内容里", "merge": "这一笔合并提交要带进去的内容里", "replay": "这一笔重放出来的提交要带进去的内容里",
                  "am": "这一笔补丁提交（git am）要带进去的内容里", "push": "这一次推送要推上去的内容里"}
MERGE_FINISH = ("这一笔合并停在没提交的状态：超限的文件是解冲突时自己加进来的，做完这几步用 git commit 收尾（会再判一遍）；超限的文件在被合并那条分支自己的提交里，"
                "在合并里删掉没用（推的时候那些提交照样带着它）：先 git merge --abort，在那条分支上重做那几笔提交（git reset --soft <分叉点>、各文件重做、git commit）再合并")
REPLAY_FINISH = ("这一笔是 cherry-pick、revert 或 rebase 重放出来的：超限的文件是解冲突或手动加进来的，做完这几步用 git cherry-pick / revert / rebase --continue 接着做（会再判一遍）；"
                 "超限的文件本来就在被重放的那笔提交里，改索引没用：先 git cherry-pick / revert / rebase --abort，在原分支上重做那笔提交（git reset --soft <分叉点>、各文件重做、git commit）再重放")
AM_FINISH = ("git am 停在这一笔补丁上：超限的文件在补丁里，做完这几步用 git am --continue 接着做（会再判一遍），或 git am --abort 退出；"
             "要让补丁里不带超限的文件，在生成补丁的那一端重做那笔提交（超限的文件压成 .gz）再 git format-patch")
NON_COMMIT_WHERE = "引用指向的对象"
broken = set()


def inside_selftest():
    """自证标记在：SELFTEST_VARIABLE 点名的目录里有 SELFTEST_MARKER_NAME 那个文件（自证建临时目录时建、删临时目录时一起没）。祖先进程的命令行不算。"""
    directory = os.environ.get(SELFTEST_VARIABLE, "")
    return bool(directory) and os.path.isfile(os.path.join(directory, SELFTEST_MARKER_NAME))


def selftest_value(name):
    """自证用的环境变量：在自证标记在的时候才认，平常设了也不认。"""
    return os.environ.get(name, "") if inside_selftest() else ""


def warn(message):
    print(f"  ⊘ {message}", file=sys.stderr)


def flat(text, limit=200):
    """子进程的输出折成一行再引：换行留着会让引文里以 ✗ 起头的行变成自证的失败行。"""
    return " ".join(str(text).split())[:limit]


def git_text(*arguments):
    """在当前目录跑 git，交标准输出去掉两端空白；起不来或退非 0 交空串。"""
    try:
        completed = subprocess.run(["git", *arguments], capture_output=True, text=True)
    except OSError:
        return ""
    return completed.stdout.strip() if completed.returncode == 0 else ""


def stop_group(process):
    """只对自己用新会话起的这一个子进程的组发信号：先 TERM，过 STOP_GRACE_SECONDS 秒还没走再 KILL。"""
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, signal.SIGTERM)
    try:
        process.wait(timeout=STOP_GRACE_SECONDS)
    except subprocess.TimeoutExpired:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)


def valid_result(result):
    """判法 JSON 里钩子要读的几个键：too_big 的每一项有字符串的 path、where 与整数的 size，unreadable 是字符串的列表。"""
    items, unreadable = result.get("too_big", []), result.get("unreadable", [])
    if not isinstance(items, list) or not isinstance(unreadable, list) or not all(isinstance(name, str) for name in unreadable):
        return False
    return all(isinstance(item, dict) and isinstance(item.get("path"), str) and isinstance(item.get("where"), str)
               and isinstance(item.get("size"), int) and not isinstance(item.get("size"), bool)
               and isinstance(item.get("commit", ""), str) and isinstance(item.get("title", ""), str) for item in items)


def run_judge(directory, mode, extra_arguments):
    """起 large-file-check.py --guard <mode>：交 (JSON 对象, "") 或 (None, 判不了的原因)。退 0、1 才读 JSON。"""
    override = selftest_value(SECONDS_VARIABLE)
    seconds = int(override) if re.fullmatch(r"[0-9]+", override) and int(override) > 0 else JUDGE_SECONDS
    index_directory = tempfile.mkdtemp(prefix="large-file-git-hook-index-")
    command = [sys.executable, "-B", CHECKER, directory, "--guard", mode, "--index-copy-directory", index_directory, *extra_arguments]
    limit = selftest_value(LIMIT_VARIABLE)
    if re.fullmatch(r"[0-9]+", limit) and int(limit) > 0:
        command += ["--limit-bytes", limit]
    for switch, guard_break in (("judge-writes-user-index", "real-index"), ("push-non-commit-objects-skipped", "non-commit-skipped"),
                                ("push-known-blobs-not-subtracted", "already-not-subtracted")):
        if switch in broken:
            command += ["--break", guard_break]
    environment = {name: value for name, value in os.environ.items() if name not in ("LARGE_FILE_CHECK_BREAK", BREAK_VARIABLE)}
    path_first = selftest_value(JUDGE_PATH_VARIABLE)
    if path_first:
        environment["PATH"] = path_first + os.pathsep + environment.get("PATH", "")
    try:
        with subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, stdin=subprocess.DEVNULL, env=environment, text=True,
                              start_new_session=True) as process:
            try:
                stdout, stderr = process.communicate(timeout=seconds)
            except subprocess.TimeoutExpired:
                stop_group(process)
                return None, f"超过 {seconds} 秒没判完"
    except OSError as error:
        return None, f"起不来（{error}）"
    finally:
        if "index-copy-kept" not in broken:
            shutil.rmtree(index_directory, ignore_errors=True)
    lines = stdout.strip().splitlines()
    try:
        result = json.loads(lines[-1])
    except (ValueError, IndexError):
        return None, f"退出码 {process.returncode}，输出读不成 JSON（{flat(stderr or stdout, 160)}）"
    if not isinstance(result, dict):
        return None, f"退出码 {process.returncode}，输出的最后一行不是 JSON 对象"
    if process.returncode not in (0, 1):
        return None, f"退出码 {process.returncode}（{result.get('error') or result.get('not_run') or '没写原因'}）"
    if not valid_result(result):
        return None, f"退出码 {process.returncode}，输出缺字段或类型不对"
    return result, ""


def size_messages(kind, result):
    """拒绝说明的两段（原因，出路）：列超限的文件（最多 LISTED 个），出路逐个文件写命令；已经在没推的提交里的写整套重做命令，引用直接指向的对象写不推这个引用。"""
    items = result["too_big"]
    listed = "；".join(f"{item['path']}（{item['size']} 字节，{item['where']}" + (f"，提交 {item['commit'][:10]}「{item.get('title', '')}」" if item.get("commit") else "") + "）"
                      for item in items[:LISTED])
    more = f"；另有 {len(items) - LISTED} 个没列" if len(items) > LISTED else ""
    reason = f"  ✗ {WHAT_IS_JUDGED[kind]}有 {len(items)} 个文件超过单文件上限 {result.get('limit_text', '')}：{listed}{more}：GitHub 的单文件推送上限，超过的推不上去，事后只能重做提交"
    ways, pushed = [], False
    for item in items[:LISTED]:
        quoted, compressed = shlex.quote(item["path"]), shlex.quote(item["path"] + ".gz")
        if item["where"] == "没推的提交":
            pushed = True
            ways.append(f"{item['path']}：`git rm --cached --ignore-unmatch -- {quoted}`、`gzip -n -9 -k {quoted}`、`git add {compressed}`")
            continue
        if item["where"] == NON_COMMIT_WHERE:
            ways.append(f"{item['path']}：这是引用直接指向的对象（指向 blob 或树的标签、引用），不在任何提交里，没有 git rm --cached 可做：不推这个引用"
                        "（本地的标签 git tag -d <标签名>，别的引用 git update-ref -d <引用>）；文件要入库就先 gzip -n -9 -k 压成 .gz、放进普通提交里再推")
            continue
        steps = [f"`gzip -n -9 -k {quoted}`", f"`git rm --cached {quoted}`", f"`git add {compressed}`"]
        ways.append(f"{item['path']}：{'、'.join(steps)}")
    finish = {"commit": "做完这几步重新 git commit", "merge": MERGE_FINISH, "replay": REPLAY_FINISH, "am": AM_FINISH,
              "push": "做完按下面重做没推的提交再推" if pushed else "做完再推"}[kind]
    way = ("     → 怎么办：超限的文件逐个：" + "；".join(ways) + "（路径相对仓的顶层目录；原文件留在工作区的加进 .gitignore 或删掉）；"
           f"读它的地方改成认 .gz（例：{READ_STORED_EXAMPLE}），引用照 .claude/rules/path-moves.md 全仓改名；{NO_GZIP_WAY}；" + finish)
    if pushed:
        base = result.get("redo_base") or "<远端已有的最新提交>"
        way += (f"\n     → 超限的 blob 已经在没推的提交里，要重做那几笔提交（删掉它的后一笔提交不够，推的时候那个 blob 照样上传），逐条单独起、别用 && 串成一条："
                f"①`git reset --soft {base}`（没推的提交合回暂存区、工作区不动；推的不是当前分支的，先 git switch 到那个分支）；"
                "②上面每个超限文件各做一遍（后一笔提交已经删掉原文件的，只做 git rm --cached --ignore-unmatch 那一条）；"
                "③`git commit -m '<说明>'`；④再推")
    return reason, way


def conclude(kind, result, why, manual_command):
    """判法交回的结论 → 退出码：超限拒（1）；判不了的放行并写一行 ⊘；没判到的（not_run）、读不到的对象同样放行并写一行 ⊘。"""
    if result is None:
        warn(f"没判成这一次{KIND_NAMES[kind]}要带的内容里有没有超过单文件上限的文件（{why}），照常放行；手动查：{manual_command}")
        return 0
    if result.get("not_run"):
        warn(f"没判这一次{KIND_NAMES[kind]}（{result['not_run']}），照常放行；手动查：{manual_command}")
    if result.get("unreadable"):
        warn(f"对象库里有 {len(result['unreadable'])} 个对象读不到：{'、'.join(result['unreadable'][:3])}，这几个没判")
    if result["too_big"]:
        reason, way = size_messages(kind, result)
        print(reason, file=sys.stderr)
        print(way, file=sys.stderr)
        return 1
    return 0


def private_hook_chain(arguments):
    """私有的提交窗口闸：$(git rev-parse --git-common-dir)/hooks/pre-commit 在而且可执行才调；是这份钩子自己（core.hooksPath 指到 .git/hooks）不调。交它的退出码，没调交 0。"""
    if "private-hook-skipped" in broken:
        return 0
    completed = subprocess.run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"], capture_output=True, text=True)
    if completed.returncode != 0:
        return 0
    private = os.path.join(completed.stdout.strip(), "hooks", "pre-commit")
    own = os.path.join(HOOKS_DIRECTORY, "pre-commit")
    if not (os.path.isfile(private) and os.access(private, os.X_OK)) or os.path.realpath(private) == os.path.realpath(own):
        return 0
    return subprocess.run([private, *arguments]).returncode


def judged_stamp():
    """判过的暂存区的凭据：交 (凭据文件的路径, <HEAD 的提交号>:<git ls-files -s -z 输出的 sha256>)；git 目录或索引读不到交 None。
    取 ls-files 的输出而不取索引文件的字节：两次钩子之间 git 只刷新文件的时间戳、重写索引，字节会变而内容（模式、blob、阶段、路径）不变。索引取 GIT_INDEX_FILE 指的那份，没设取这个仓的 index。"""
    git_directory = git_text("rev-parse", "--absolute-git-dir")
    if not git_directory:
        return None
    try:
        listing = subprocess.run(["git", "ls-files", "-s", "-z"], capture_output=True)
    except OSError:
        return None
    if listing.returncode != 0:
        return None
    return os.path.join(git_directory, STAMP_NAME), f"{git_text('rev-parse', '--verify', '-q', 'HEAD')}:{hashlib.sha256(listing.stdout).hexdigest()}"


def record_judged():
    stamp = judged_stamp()
    if stamp is not None:
        with contextlib.suppress(OSError), open(stamp[0], "w", encoding="utf-8") as handle:
            handle.write(stamp[1])


def judged_already():
    """pre-commit 或 pre-merge-commit 刚判过同一份暂存区（同一个 HEAD、同样的索引内容）：交 True 并删掉凭据；对不上、没有凭据交 False。"""
    stamp = judged_stamp()
    if stamp is None:
        return False
    try:
        with open(stamp[0], encoding="utf-8") as handle:
            recorded = handle.read()
    except OSError:
        return False
    if recorded != stamp[1] and "stale-stamp-trusted" not in broken:
        return False
    with contextlib.suppress(FileNotFoundError):
        os.remove(stamp[0])
    return True


def replay_in_progress():
    git_directory = git_text("rev-parse", "--absolute-git-dir")
    return bool(git_directory) and any(os.path.exists(os.path.join(git_directory, name)) for name in REPLAY_STATE_NAMES)


def judge_staged(kind):
    """判 git 放进钩子看到的索引里的内容：交 (退出码, 判成了而且没超限)。"""
    result, why = run_judge(os.getcwd(), "commit-index", [])
    code = conclude(kind, result, why, f"python3 {os.path.relpath(CHECKER, REPOSITORY_ROOT)} <仓里的目录> --guard commit-index")
    return code, result is not None and not result.get("not_run") and not result["too_big"]


def judge_index(kind, arguments, skip_switch):
    """pre-commit 与 pre-merge-commit 共用：先调私有的提交窗口闸，再判暂存区、判过没超限的记凭据；skip_switch 是这一种钩子对应的弄坏开关。"""
    code = private_hook_chain(arguments)
    if code != 0 or skip_switch in broken:
        return code
    code, judged = judge_staged(kind)
    if judged:
        record_judged()
    return code


def pre_commit(arguments):
    return judge_index("commit", arguments, "pre-commit-judge-skipped")


def pre_merge_commit(arguments):
    return judge_index("merge", arguments, "pre-merge-commit-judge-skipped")


def prepare_commit_msg(arguments):
    """参数是 <说明文件> [<来源> [<提交号>]]：来源是 merge 的按合并提交写说明，在 cherry-pick、revert、rebase 里的按重放写；pre-commit 刚判过同一份暂存区的不再判。"""
    if "prepare-commit-msg-judge-skipped" in broken:
        return 0
    if "prepare-commit-msg-stamp-ignored" not in broken and judged_already():
        return 0
    source = arguments[1] if len(arguments) > 1 else ""
    return judge_staged("merge" if source == "merge" else "replay" if replay_in_progress() else "commit")[0]


def pre_applypatch(arguments):
    if "pre-applypatch-judge-skipped" in broken:
        return 0
    return judge_staged("am")[0]


def is_deletion(sha):
    return set(sha) <= {"0"}


def push_lines(stream):
    """标准输入里 git 给的行 `<本地引用> <本地 sha> <远端引用> <远端 sha>`；格式不对的行略过。"""
    rows = []
    for line in stream:
        fields = line.split()
        if len(fields) == 4:
            rows.append(fields)
    return rows


def pre_push(arguments, stream):
    remote = arguments[0] if arguments else ""
    rows = push_lines(stream)
    manual = f"python3 {os.path.relpath(CHECKER, REPOSITORY_ROOT)} <仓里的目录> --guard push --remote <远端> --ref <本地 sha>"
    if "push-reads-current-branch-only" in broken:
        extra = ["--ref", "HEAD"] + (["--remote", remote] if remote else []) + ([] if "unknown-remote-not-judged" in broken else ["--judge-unknown"])
        if rows and all(is_deletion(row[1]) for row in rows):
            return 0
    else:
        local = list(dict.fromkeys(row[1] for row in rows if not is_deletion(row[1])))
        if not local:
            return 0
        known = [] if "remote-sha-ignored" in broken else list(dict.fromkeys(row[3] for row in rows if not is_deletion(row[3])))
        extra = ["--remote", remote] if remote and "push-target-not-named" not in broken else []
        for sha in local:
            extra += ["--ref", sha]
        for sha in known:
            extra += ["--known", sha]
        if "unknown-remote-not-judged" not in broken:
            extra.append("--judge-unknown")
    result, why = run_judge(os.getcwd(), "push", extra)
    return conclude("push", result, why, manual)


def settle_break_switches(arguments):
    """弄坏开关只在自证里认：BREAK_VARIABLE 设了、而命令行不是 --selftest、自证标记也不在，点名它、说它只在自证里认，交回退出码 2；否则交 None。"""
    value = os.environ.get(BREAK_VARIABLE)
    if not value or arguments[:1] == ["--selftest"] or inside_selftest():
        return None
    print(f"  ✗ {BREAK_VARIABLE} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 python3 scripts/gate/large-file-git-hook.py --selftest（末尾逐个打开）", file=sys.stderr)
    return 2


# ── 自证 ──
SELFTEST_LIMIT = 1000   # 门限压到 1000 字节：样本里的「大文件」是 2000 字节
PRE_COMMIT_ONLY = ("pre-commit",)
PRE_MERGE_COMMIT_ONLY = ("pre-merge-commit",)
PREPARE_ONLY = ("prepare-commit-msg",)
APPLYPATCH_ONLY = ("pre-applypatch",)
PRE_PUSH_ONLY = ("pre-push",)
COMMIT_AND_PREPARE = ("pre-commit", "prepare-commit-msg")
MERGE_AND_PREPARE = ("pre-merge-commit", "prepare-commit-msg")
PRIVATE_WINDOW_GATE = '#!/usr/bin/env bash\necho "PRIVATE-WINDOW-GATE" >&2\n[[ "${SINGLEFS_COMMIT_OUTSIDE_WINDOW:-}" == 1 ]] && exit 0\nexit 1\n'
CELLS = {}


class Sandbox:
    """一格自证的临时目录：仓、裸远端、钩子目录都建在里面；git 一律读不到用户与系统的配置；switch 是这一格设的弄坏开关（逗号隔开）。
    environment 是带自证标记与自证门限的环境（钩子认），plain_environment 是不带的（要证「平常跑」的几格用）。"""

    def __init__(self, directory, switch=""):
        self.directory = directory
        self.broken = {item for item in switch.split(",") if item}
        self.temporary = os.path.join(directory, "tmp")
        os.makedirs(self.temporary)
        with open(os.path.join(directory, SELFTEST_MARKER_NAME), "w", encoding="utf-8") as handle:
            handle.write("自证的标记：这个目录在，钩子才认自证门限与弄坏开关\n")
        dropped = ("LARGE_FILE_CHECK_BREAK", BREAK_VARIABLE, SELFTEST_VARIABLE, LIMIT_VARIABLE, SECONDS_VARIABLE, JUDGE_PATH_VARIABLE)
        self.plain_environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_") and name not in dropped}
        self.plain_environment.update(GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1", GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@t", GIT_COMMITTER_NAME="t",
                                      GIT_COMMITTER_EMAIL="t@t", GIT_TERMINAL_PROMPT="0", GIT_EDITOR="true", GIT_CEILING_DIRECTORIES=directory, TMPDIR=self.temporary)
        self.environment = dict(self.plain_environment, **{LIMIT_VARIABLE: str(SELFTEST_LIMIT), SELFTEST_VARIABLE: directory})
        if switch:
            self.environment[BREAK_VARIABLE] = switch
        if "selftest-marker-granted-to-plain-run" in self.broken:
            self.plain_environment[SELFTEST_VARIABLE] = directory

    def git(self, repository, *arguments, check=True, environment=None, input_text=None):
        completed = subprocess.run(["git", "-C", repository, *arguments], capture_output=True, text=True, input=input_text, env=dict(self.environment, **(environment or {})))
        if check and completed.returncode != 0:
            raise RuntimeError(f"装置里的 git {' '.join(arguments)} 退 {completed.returncode}：{flat(completed.stderr)}")
        return completed

    def plain_git(self, repository, *arguments, environment=None):
        """不带自证标记与门限的环境里起 git（要证「平常跑」的格用）。"""
        return subprocess.run(["git", "-C", repository, *arguments], capture_output=True, text=True, env=dict(self.plain_environment, **(environment or {})))

    def write(self, repository, name, size):
        path = os.path.join(repository, name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb") as handle:
            handle.write(b"x" * size)

    def stage(self, repository, name, size):
        self.write(repository, name, size)
        self.git(repository, "add", name)

    def hooks_directory(self, names):
        """names 是钩子名的元组：建一个只装这几个钩子的目录，每个钩子都 exec 这份脚本、第一个参数是钩子名；不给就是进仓的 scripts/gate/git-hooks/（真钩子文件）。"""
        if names is None:
            return HOOKS_DIRECTORY
        directory = os.path.join(self.directory, "hooks-" + "-".join(names))
        os.makedirs(directory, exist_ok=True)
        for name in names:
            path = os.path.join(directory, name)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(f'#!/usr/bin/env bash\nexec python3 -B {shlex.quote(HOOK_SCRIPT)} {name} "$@"\n')
            os.chmod(path, 0o755)
        return directory

    def repository(self, name, hooks=None, tracked=("base.txt",), location_independent=False):
        """一个小仓：分支 main、core.hooksPath 写成钩子目录的绝对路径（hooks 的写法见 hooks_directory）、基准提交只有小文件（不经钩子提交）。
        location_independent 的仓用在「装钩子的人该写绝对路径」那几格：弄坏开关 hooks-path-written-relative 下写成相对路径（找不到钩子目录，钩子一个都不跑）。"""
        path = os.path.join(self.directory, name)
        subprocess.run(["git", "init", "-q", path], check=True, env=self.environment, capture_output=True)
        self.git(path, "symbolic-ref", "HEAD", "refs/heads/main")
        hooks_path = os.path.join("scripts", "gate", "git-hooks") if location_independent and "hooks-path-written-relative" in self.broken else self.hooks_directory(hooks)
        self.git(path, "config", "core.hooksPath", hooks_path)
        for tracked_name in tracked:
            self.write(path, tracked_name, 10)
        self.git(path, "add", *tracked)
        self.commit_without_hooks(path, "base")
        return path

    def commit_without_hooks(self, repository, message):
        self.git(repository, "-c", "core.hooksPath=/dev/null", "commit", "-q", "-m", message)

    def remote(self, name):
        path = os.path.join(self.directory, name + ".git")
        subprocess.run(["git", "init", "-q", "--bare", path], check=True, env=self.environment, capture_output=True)
        return path

    def with_remote(self, repository, name):
        """裸远端 origin，main 已推上去（推的是小提交，不经钩子），本地有跟踪引用。"""
        remote = self.remote(name)
        self.git(repository, "remote", "add", "origin", remote)
        self.git(repository, "-c", "core.hooksPath=/dev/null", "push", "-q", "-u", "origin", "main")
        return remote

    def commit_big(self, repository, name="big.bin", message="big"):
        self.stage(repository, name, 2000)
        self.commit_without_hooks(repository, message)

    def head(self, repository):
        return self.git(repository, "rev-parse", "HEAD").stdout.strip()

    def remote_ref(self, remote, ref):
        return self.git(remote, "rev-parse", "--verify", "-q", ref, check=False).stdout.strip()

    def remote_head(self, remote, branch="main"):
        return self.remote_ref(remote, f"refs/heads/{branch}")

    def private_gate(self, repository):
        """私有的提交窗口闸：.git/hooks/pre-commit，打一行 PRIVATE-WINDOW-GATE，设了 SINGLEFS_COMMIT_OUTSIDE_WINDOW=1 才放行。"""
        path = os.path.join(repository, ".git", "hooks", "pre-commit")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(PRIVATE_WINDOW_GATE)
        os.chmod(path, 0o755)

    def diff_tracer(self):
        """判法 PATH 前面的一份 git：起 diff 的那一次把它看到的 GIT_INDEX_FILE 记进日志，再交给真 git。交 (放假 git 的目录, 日志路径)。"""
        directory, log = os.path.join(self.directory, "trace-bin"), os.path.join(self.directory, "trace-index.log")
        os.makedirs(directory)
        with open(os.path.join(directory, "git"), "w", encoding="utf-8") as handle:
            handle.write(f'#!/bin/sh\ncase " $* " in *" diff "*) echo "$GIT_INDEX_FILE" >> {shlex.quote(log)} ;; esac\nexec {shutil.which("git")} "$@"\n')
        os.chmod(os.path.join(directory, "git"), 0o755)
        return directory, log

    def traced_indexes(self, log):
        if not os.path.exists(log):
            return []
        with open(log, encoding="utf-8") as handle:
            return handle.read().split()

    def landed_in_history(self, repository, name):
        """name 这个文件出现在任何引用与 HEAD 能到的提交的改动里。"""
        return name in self.git(repository, "log", "--all", "--name-only", "--format=").stdout.split()

    def diverged(self, name, hooks, feature_file, size):
        """feature 上有一笔带 feature_file 的提交（不经钩子），main 上又有一笔小提交：合并不是快进。"""
        repository = self.repository(name, hooks=hooks)
        self.git(repository, "checkout", "-q", "-b", "feature")
        self.stage(repository, feature_file, size)
        self.commit_without_hooks(repository, "feature work")
        self.git(repository, "checkout", "-q", "main")
        self.stage(repository, "main-only.txt", 10)
        self.commit_without_hooks(repository, "main work")
        return repository


def selftest_cell(function):
    """把函数登记成一格自证：格名是函数名里的下划线换成连字符；函数收一个 Sandbox，交 (对不对, 不对时的说明)。"""
    cell_name = function.__name__.replace("_", "-")

    CELLS[cell_name] = function

    return function


def failure(done):
    return f"退 {done.returncode}：{flat(done.stderr, 300)}"


def refused_with(completed, *needles):
    return completed.returncode != 0 and all(needle in completed.stderr for needle in needles)


# 提交（只装 pre-commit）
@selftest_cell
def commit_small_passes(box):
    repository = box.repository("repo", hooks=PRE_COMMIT_ONLY)
    box.stage(repository, "small.txt", 10)
    done = box.git(repository, "commit", "-q", "-m", "small", check=False)
    return done.returncode == 0, f"只暂存了小文件的 git commit 应当通过，{failure(done)}"


@selftest_cell
def commit_staged_big_refused(box):
    repository = box.repository("repo", hooks=PRE_COMMIT_ONLY)
    box.stage(repository, "big.bin", 2000)
    before = box.head(repository)
    done = box.git(repository, "commit", "-q", "-m", "x", check=False)
    return (refused_with(done, "单文件上限", "gzip -n -9 -k big.bin", "git rm --cached big.bin", "git add big.bin.gz") and box.head(repository) == before,
            f"暂存了超门限文件的 git commit 应当被拒、出路逐个文件写、提交没进；{failure(done)}")


@selftest_cell
def commit_judge_reads_index_copy(box):
    repository = box.repository("repo", hooks=PRE_COMMIT_ONLY)
    box.stage(repository, "big.bin", 2000)
    tracer, log = box.diff_tracer()
    box.git(repository, "commit", "-q", "-m", "x", check=False, environment={JUDGE_PATH_VARIABLE: tracer})
    traced = box.traced_indexes(log)
    return (bool(traced) and all("large-file-git-hook-index-" in path for path in traced),
            f"判法起的 git diff 应当读索引拷贝（路径带 large-file-git-hook-index-），不读用户的索引；看到的 GIT_INDEX_FILE：{traced}")


@selftest_cell
def commit_unstaged_growth_passes(box):
    repository = box.repository("repo", hooks=PRE_COMMIT_ONLY, tracked=("base.txt", "grown.txt"))
    box.write(repository, "grown.txt", 3000)
    box.stage(repository, "new.txt", 10)
    done = box.git(repository, "commit", "-q", "-m", "x", check=False)
    return done.returncode == 0, f"工作区改大了而没暂存的文件不进这一笔，应当通过，{failure(done)}"


@selftest_cell
def commit_all_grown_refused(box):
    repository = box.repository("repo", hooks=PRE_COMMIT_ONLY, tracked=("base.txt", "grown.txt"))
    box.write(repository, "grown.txt", 3000)
    done = box.git(repository, "commit", "-q", "-a", "-m", "x", check=False)
    return refused_with(done, "grown.txt", "单文件上限"), f"git commit -a 把改大了的跟踪文件带进提交，应当被拒，{failure(done)}"


@selftest_cell
def commit_paths_grown_refused(box):
    repository = box.repository("repo", hooks=PRE_COMMIT_ONLY, tracked=("base.txt", "grown.txt"))
    box.write(repository, "grown.txt", 3000)
    done = box.git(repository, "commit", "-q", "-m", "x", "grown.txt", check=False)
    return refused_with(done, "grown.txt", "单文件上限"), f"git commit <路径> 点名改大了的文件，应当被拒，{failure(done)}"


# 私有的提交窗口闸
@selftest_cell
def private_hook_refusal_first(box):
    repository = box.repository("repo", hooks=PRE_COMMIT_ONLY)
    box.private_gate(repository)
    box.stage(repository, "small.txt", 10)
    done = box.git(repository, "commit", "-q", "-m", "x", check=False)
    return refused_with(done, "PRIVATE-WINDOW-GATE") and "单文件上限" not in done.stderr, f"私有的窗口闸退 1 时应当先拒、不再判大文件；{failure(done)}"


@selftest_cell
def private_hook_then_judge(box):
    repository = box.repository("repo", hooks=PRE_COMMIT_ONLY)
    box.private_gate(repository)
    box.stage(repository, "big.bin", 2000)
    done = box.git(repository, "commit", "-q", "-m", "x", check=False, environment={"SINGLEFS_COMMIT_OUTSIDE_WINDOW": "1"})
    return refused_with(done, "PRIVATE-WINDOW-GATE", "单文件上限"), f"设了放行窗口的环境变量之后私有闸放行、大文件照判，应当被拒；{failure(done)}"


# 合并提交：git merge 生成合并提交时起的是 pre-merge-commit、不是 pre-commit
@selftest_cell
def merge_small_passes(box):
    repository = box.diverged("repo", PRE_MERGE_COMMIT_ONLY, "feature.txt", 10)
    done = box.git(repository, "merge", "-q", "--no-edit", "-m", "merge", "feature", check=False)
    parents = box.git(repository, "rev-list", "--parents", "-n", "1", "HEAD").stdout.split()
    return (done.returncode == 0 and len(parents) == 3,
            f"被合并的分支只带小文件，合并提交应当通过、生成两个父提交的提交；退 {done.returncode}，HEAD 的父提交 {len(parents) - 1} 个：{flat(done.stderr)}")


@selftest_cell
def merge_big_refused(box):
    repository = box.diverged("repo", PRE_MERGE_COMMIT_ONLY, "big.bin", 2000)
    before = box.head(repository)
    done = box.git(repository, "merge", "-q", "--no-edit", "-m", "merge", "feature", check=False)
    return (refused_with(done, "合并提交", "big.bin", "单文件上限", "git merge --abort") and box.head(repository) == before,
            f"被合并的分支带着超门限文件，合并提交应当被拒、合并提交没进、出路写 git merge --abort；{failure(done)}")


@selftest_cell
def merge_no_commit_then_commit_refused(box):
    repository = box.diverged("repo", PRE_COMMIT_ONLY, "big.bin", 2000)
    box.git(repository, "merge", "-q", "--no-commit", "--no-ff", "feature")
    done = box.git(repository, "commit", "-q", "-m", "merge", check=False)
    return refused_with(done, "big.bin", "单文件上限"), f"git merge --no-commit 之后用 git commit 收尾走 pre-commit，带着超门限文件应当被拒；{failure(done)}"


# 一份状态上先拒再放：拒的那次之后合并停在没提交的状态，要 --abort 才能再合并一次，两步是同一个场景
@selftest_cell
def merge_private_hook_window(box):
    repository = box.diverged("repo", PRE_MERGE_COMMIT_ONLY, "feature.txt", 10)
    box.private_gate(repository)
    before = box.head(repository)
    done = box.git(repository, "merge", "-q", "--no-edit", "-m", "merge", "feature", check=False)
    refused = refused_with(done, "PRIVATE-WINDOW-GATE") and box.head(repository) == before
    box.git(repository, "merge", "--abort", check=False)
    done_outside = box.git(repository, "merge", "-q", "--no-edit", "-m", "merge", "feature", check=False, environment={"SINGLEFS_COMMIT_OUTSIDE_WINDOW": "1"})
    return (refused and done_outside.returncode == 0,
            f"合并提交也是提交：私有的窗口闸退 1 时应当拒、设了放行窗口的环境变量时放行；拒的那次 {'对' if refused else '不对：' + failure(done)}，放行的那次退 {done_outside.returncode}：{flat(done_outside.stderr)}")


# 推送（只装 pre-push）
def unpushed_big_repository(box):
    """main 上有一笔加 big.bin 的提交与后一笔删掉它的提交（都没推）；side 分支停在远端已有的 main 上。"""
    repository = box.repository("repo", hooks=PRE_PUSH_ONLY)
    remote = box.with_remote(repository, "repo")
    box.commit_big(repository, message="add big")
    box.git(repository, "rm", "-q", "big.bin")
    box.commit_without_hooks(repository, "drop big")
    box.git(repository, "branch", "side", "refs/remotes/origin/main")
    return repository, remote


@selftest_cell
def push_unpushed_big_refused(box):
    repository, remote = unpushed_big_repository(box)
    before = box.remote_head(remote)
    done = box.git(repository, "push", "-q", "origin", "main", check=False)
    return (refused_with(done, "没推的提交", "git reset --soft", "git commit -m") and box.remote_head(remote) == before,
            f"没推的提交里有超门限的 blob（后一笔又删了），git push 应当被拒、给整套重做命令；{failure(done)}")


@selftest_cell
def push_clean_branch_passes(box):
    repository, _remote = unpushed_big_repository(box)
    done = box.git(repository, "push", "-q", "origin", "side:refs/heads/side", check=False)
    return done.returncode == 0, f"当前分支欠着超门限的 blob，推干净的分支应当通过，{failure(done)}"


@selftest_cell
def push_delete_passes(box):
    repository = box.repository("repo", hooks=PRE_PUSH_ONLY)
    box.with_remote(repository, "repo")
    box.git(repository, "-c", "core.hooksPath=/dev/null", "push", "-q", "origin", "main:refs/heads/stale")   # 先放一个远端分支，下面只删它
    box.git(repository, "update-ref", "-d", "refs/remotes/origin/stale", check=False)
    done = box.git(repository, "push", "-q", "origin", ":stale", check=False)
    return done.returncode == 0, f"只删远端分支的 git push 不带对象，应当通过，{failure(done)}"


def branches_repository(box):
    """main 上没欠；feature 比基准多一笔加 big.bin 的提交，标签 v-big 指着它；side 是干净的分支。"""
    repository = box.repository("repo", hooks=PRE_PUSH_ONLY)
    remote = box.with_remote(repository, "repo")
    box.git(repository, "branch", "side")
    box.git(repository, "checkout", "-q", "-b", "feature")
    box.commit_big(repository, message="add big")
    box.git(repository, "tag", "v-big")
    box.git(repository, "checkout", "-q", "main")
    return repository, remote


@selftest_cell
def push_other_branch_refused(box):
    repository, remote = branches_repository(box)
    done = box.git(repository, "push", "-q", "origin", "feature", check=False)
    return refused_with(done, "big.bin") and not box.remote_head(remote, "feature"), f"当前分支是干净的 main，推 feature 应当按 feature 判、被拒，{failure(done)}"


@selftest_cell
def push_all_refused(box):
    repository, _remote = branches_repository(box)
    done = box.git(repository, "push", "-q", "--all", "origin", check=False)
    return refused_with(done, "big.bin"), f"git push --all 应当被拒，{failure(done)}"


@selftest_cell
def push_tags_refused(box):
    repository, _remote = branches_repository(box)
    done = box.git(repository, "push", "-q", "--tags", "origin", check=False)
    return refused_with(done, "big.bin"), f"git push --tags 推的标签指着带超门限 blob 的提交，应当被拒，{failure(done)}"


@selftest_cell
def push_new_name_refspec_refused(box):
    repository, remote = branches_repository(box)
    done = box.git(repository, "push", "-q", "origin", "feature:refs/heads/new-name", check=False)
    return refused_with(done, "big.bin") and not box.remote_head(remote, "new-name"), f"<源>:<新名字> 的 refspec 应当照标准输入里的行判、被拒，{failure(done)}"


@selftest_cell
def push_wildcard_refspec_refused(box):
    repository, _remote = branches_repository(box)
    done = box.git(repository, "push", "-q", "origin", "refs/heads/*:refs/heads/*", check=False)
    return refused_with(done, "big.bin"), f"通配的 refspec 应当被拒，{failure(done)}"


@selftest_cell
def push_small_branch_passes(box):
    repository, _remote = branches_repository(box)
    done = box.git(repository, "push", "-q", "origin", "side", check=False)
    return done.returncode == 0, f"干净的分支应当通过，{failure(done)}"


# 远端没取回过、推向网址或路径、远端已有超门限的 blob
@selftest_cell
def push_never_fetched_refused(box):
    repository = box.repository("repo", hooks=PRE_PUSH_ONLY)
    box.commit_big(repository, message="add big")
    fresh = box.remote("fresh-remote")
    box.git(repository, "remote", "add", "fresh", fresh)
    done = box.git(repository, "push", "-q", "fresh", "main", check=False)
    return refused_with(done, "big.bin") and not box.remote_head(fresh), f"没取回过的远端、新引用，应当判全部历史、被拒，{failure(done)}"


@selftest_cell
def push_to_url_refused(box):
    repository = box.repository("repo", hooks=PRE_PUSH_ONLY)
    box.commit_big(repository, message="add big")
    path_remote = box.remote("path-remote")
    done = box.git(repository, "push", "-q", path_remote, "main", check=False)
    return refused_with(done, "big.bin") and not box.remote_head(path_remote), f"推向路径写法的远端（不是配置过的名字），应当被拒，{failure(done)}"


@selftest_cell
def push_remote_already_has_big_passes(box):
    repository = box.repository("repo", hooks=PRE_PUSH_ONLY)
    box.with_remote(repository, "repo")
    box.commit_big(repository, message="add big")
    box.git(repository, "-c", "core.hooksPath=/dev/null", "push", "-q", "origin", "main")   # 远端已经有这笔带超门限 blob 的提交
    box.git(repository, "update-ref", "-d", "refs/remotes/origin/main")                      # 本地的跟踪引用没了：只剩推送那一刻行里的远端 sha 说明远端已有什么
    box.stage(repository, "small.txt", 10)
    box.commit_without_hooks(repository, "small")
    done = box.git(repository, "push", "-q", "origin", "main", check=False)
    return done.returncode == 0, f"远端已有的提交里的超门限 blob 不该再拦，应当通过，{failure(done)}"


# 推向路径写法的远端：别的远端（origin）的跟踪引用说的是 origin 有什么，不是这个路径上有什么；只认标准输入行里的远端 sha
@selftest_cell
def push_to_path_another_remote_has_it_refused(box):
    repository = box.repository("repo", hooks=PRE_PUSH_ONLY)
    box.with_remote(repository, "repo")
    box.commit_big(repository, message="add big")
    box.git(repository, "-c", "core.hooksPath=/dev/null", "push", "-q", "origin", "main")   # origin 与本地的 origin/main 都已有这笔带超门限 blob 的提交
    backup = box.remote("backup")
    done = box.git(repository, "push", "-q", backup, "main", check=False)
    return refused_with(done, "big.bin") and not box.remote_head(backup), f"别的远端已有这笔提交、推送目标（写成路径）没有，应当照全部历史判、被拒；{failure(done)}"


@selftest_cell
def push_to_path_remote_already_has_big_passes(box):
    repository = box.repository("repo", hooks=PRE_PUSH_ONLY)
    path_remote = box.remote("path-remote")
    box.commit_big(repository, message="add big")
    box.git(repository, "-c", "core.hooksPath=/dev/null", "push", "-q", path_remote, "main")   # 目标仓已有这笔带超门限 blob 的提交，本地没有它的跟踪引用
    box.stage(repository, "small.txt", 10)
    box.commit_without_hooks(repository, "small")
    done = box.git(repository, "push", "-q", path_remote, "main", check=False)
    return (done.returncode == 0 and box.remote_head(path_remote) == box.head(repository),
            f"推送目标（写成路径）已有的提交里的超门限 blob 不该再拦，行里的远端 sha 说它已有；应当通过；{failure(done)}")


# 判法到点被停：索引拷贝与子进程都不留
@selftest_cell
def judge_timeout_leaves_nothing(box):
    repository = box.repository("repo", hooks=PRE_COMMIT_ONLY)
    box.stage(repository, "small.txt", 10)
    slow = os.path.join(box.directory, "slow-bin")
    os.makedirs(slow)
    with open(os.path.join(slow, "git"), "w", encoding="utf-8") as handle:
        handle.write(f'#!/bin/sh\ncase " $* " in *" diff "*) sleep 30 ;; esac\nexec {shutil.which("git")} "$@"\n')
    os.chmod(os.path.join(slow, "git"), 0o755)
    started = time.monotonic()
    done = box.git(repository, "commit", "-q", "-m", "x", check=False, environment={JUDGE_PATH_VARIABLE: slow, SECONDS_VARIABLE: "2"})
    elapsed = time.monotonic() - started
    leftovers = [entry for entry in os.listdir(box.temporary) if entry.startswith("large-file-git-hook-index-")]
    survivors = processes_mentioning(f"{repository} --guard")
    return (done.returncode == 0 and "⊘" in done.stderr and not leftovers and not survivors and elapsed < 25,
            f"判法超时：放行并写 ⊘、不留索引拷贝与进程、不等到 sleep 结束；退 {done.returncode}，用了 {elapsed:.1f} 秒，留下 {leftovers}、进程 {survivors}，stderr：{flat(done.stderr)}")


# prepare-commit-msg：--no-verify 挡不住它；cherry-pick、revert、rebase 的逐笔重放与 --continue 起它
@selftest_cell
def commit_no_verify_refused_by_prepare_commit_msg(box):
    repository = box.repository("repo", hooks=COMMIT_AND_PREPARE)
    box.stage(repository, "big.bin", 2000)
    before = box.head(repository)
    done = box.git(repository, "commit", "-q", "--no-verify", "-m", "x", check=False)
    return refused_with(done, "单文件上限", "big.bin") and box.head(repository) == before, f"git commit --no-verify 不起 pre-commit，prepare-commit-msg 应当判、拒、提交没进；{failure(done)}"


@selftest_cell
def commit_judged_once(box):
    repository = box.repository("repo", hooks=COMMIT_AND_PREPARE)
    box.stage(repository, "small.txt", 10)
    tracer, log = box.diff_tracer()
    before = box.head(repository)
    done = box.git(repository, "commit", "-q", "-m", "x", check=False, environment={JUDGE_PATH_VARIABLE: tracer})
    traced = box.traced_indexes(log)
    return (done.returncode == 0 and box.head(repository) != before and len(traced) == 1,
            f"pre-commit 判过同一份暂存区，prepare-commit-msg 对得上凭据就不再判：判法起 git diff 应当恰好 1 次、提交进；退 {done.returncode}，起了 {len(traced)} 次：{flat(done.stderr)}")


@selftest_cell
def stale_stamp_not_trusted(box):
    repository = box.repository("repo", hooks=PREPARE_ONLY)
    box.stage(repository, "small.txt", 10)
    judged = subprocess.run([sys.executable, "-B", HOOK_SCRIPT, "pre-commit"], cwd=repository, capture_output=True, text=True, env=box.environment)   # 留下小暂存区的凭据
    box.stage(repository, "big.bin", 2000)
    before = box.head(repository)
    done = box.git(repository, "commit", "-q", "--no-verify", "-m", "x", check=False)
    return (judged.returncode == 0 and refused_with(done, "单文件上限", "big.bin") and box.head(repository) == before,
            f"凭据记的是暂存区变大之前的字节，对不上，prepare-commit-msg 应当照判、拒；先判的那次退 {judged.returncode}：{flat(judged.stderr)}；{failure(done)}")


@selftest_cell
def merge_no_verify_refused_by_prepare_commit_msg(box):
    repository = box.diverged("repo", MERGE_AND_PREPARE, "big.bin", 2000)
    before = box.head(repository)
    done = box.git(repository, "merge", "-q", "--no-verify", "--no-edit", "-m", "merge", "feature", check=False)
    return (refused_with(done, "合并提交", "big.bin", "单文件上限") and box.head(repository) == before,
            f"git merge --no-verify 不起 pre-merge-commit，prepare-commit-msg（来源 merge）应当判、拒、合并提交没进；{failure(done)}")


@selftest_cell
def cherry_pick_refused_by_prepare_commit_msg(box):
    repository = box.repository("repo", hooks=PREPARE_ONLY)
    box.git(repository, "checkout", "-q", "-b", "feature")
    box.commit_big(repository, message="feature big")
    box.git(repository, "checkout", "-q", "main")
    before = box.head(repository)
    done = box.git(repository, "cherry-pick", "feature", check=False)
    return refused_with(done, "单文件上限", "big.bin") and box.head(repository) == before, f"cherry-pick 重放带超门限文件的提交，prepare-commit-msg 应当判、拒、提交没进；{failure(done)}"


@selftest_cell
def revert_restoring_big_refused(box):
    repository = box.repository("repo", hooks=PREPARE_ONLY)
    box.commit_big(repository, message="add big")
    box.git(repository, "rm", "-q", "big.bin")
    box.commit_without_hooks(repository, "drop big")
    before = box.head(repository)
    done = box.git(repository, "revert", "--no-edit", "HEAD", check=False)
    return refused_with(done, "单文件上限", "big.bin") and box.head(repository) == before, f"revert 把删掉的超门限文件加回来，prepare-commit-msg 应当判、拒、提交没进；{failure(done)}"


@selftest_cell
def rebase_conflict_then_continue_refused(box):
    repository = box.repository("repo", hooks=PREPARE_ONLY, tracked=("base.txt", "shared.txt"))
    box.git(repository, "checkout", "-q", "-b", "feature")
    box.write(repository, "shared.txt", 11)
    box.git(repository, "add", "shared.txt")
    box.commit_without_hooks(repository, "feature edit")
    box.git(repository, "checkout", "-q", "main")
    box.write(repository, "shared.txt", 12)
    box.git(repository, "add", "shared.txt")
    box.commit_without_hooks(repository, "main edit")
    box.git(repository, "checkout", "-q", "feature")
    started = box.git(repository, "rebase", "main", check=False)   # 两边改同一行：冲突，停下
    box.write(repository, "shared.txt", 13)                         # 解冲突，顺手把超门限的文件一起暂存
    box.git(repository, "add", "shared.txt")
    box.stage(repository, "big.bin", 2000)
    done = box.git(repository, "rebase", "--continue", check=False)
    return (started.returncode != 0 and refused_with(done, "单文件上限", "big.bin") and not box.landed_in_history(repository, "big.bin"),
            f"rebase 冲突解完带着超门限文件 --continue，prepare-commit-msg 应当判、拒、文件没进提交；冲突那一步退 {started.returncode}，--continue {failure(done)}")


@selftest_cell
def rebase_i_edit_then_continue_directly_refused(box):
    repository = box.repository("repo", hooks=PREPARE_ONLY)
    box.git(repository, "checkout", "-q", "-b", "feature")
    for name in ("early.txt", "late.txt"):
        box.stage(repository, name, 10)
        box.commit_without_hooks(repository, name)
    editor = os.path.join(box.directory, "sequence-editor.sh")
    with open(editor, "w", encoding="utf-8") as handle:
        handle.write("#!/bin/sh\nsed -i '1s/^pick/edit/' \"$1\"\n")
    os.chmod(editor, 0o755)
    started = box.git(repository, "rebase", "-i", "main", check=False, environment={"GIT_SEQUENCE_EDITOR": editor})   # 在第一笔提交上停下（edit）
    box.stage(repository, "big.bin", 2000)
    done = box.git(repository, "rebase", "--continue", check=False)
    return (refused_with(done, "单文件上限", "big.bin") and not box.landed_in_history(repository, "big.bin"),
            f"rebase -i 停在 edit 上、暂存了超门限的文件就直接 --continue，prepare-commit-msg 应当判、拒、文件没进提交；停下那一步退 {started.returncode}：{flat(started.stderr)}；--continue {failure(done)}")


# pre-applypatch：git am 打完补丁、生成提交之前起它
def patch_of_feature_commit(box, repository, size):
    """feature 分支上加一个 size 字节文件的提交（不经钩子）做成补丁，回到 main；交补丁文件的路径。"""
    box.git(repository, "checkout", "-q", "-b", "feature")
    box.stage(repository, "patched.bin", size)
    box.commit_without_hooks(repository, "add patched")
    patch_directory = os.path.join(box.directory, "patches")
    box.git(repository, "format-patch", "-q", "-1", "-o", patch_directory)
    box.git(repository, "checkout", "-q", "main")
    return os.path.join(patch_directory, sorted(os.listdir(patch_directory))[0])


@selftest_cell
def am_big_refused(box):
    repository = box.repository("repo", hooks=APPLYPATCH_ONLY)
    patch = patch_of_feature_commit(box, repository, 2000)
    before = box.head(repository)
    done = box.git(repository, "am", "-q", patch, check=False)
    return refused_with(done, "单文件上限", "patched.bin") and box.head(repository) == before, f"git am 打带超门限文件的补丁，pre-applypatch 应当判、拒、提交没进；{failure(done)}"


@selftest_cell
def am_small_passes(box):
    repository = box.repository("repo", hooks=APPLYPATCH_ONLY)
    patch = patch_of_feature_commit(box, repository, 10)
    before = box.head(repository)
    done = box.git(repository, "am", "-q", patch, check=False)
    return done.returncode == 0 and box.head(repository) != before, f"git am 打只带小文件的补丁应当通过、提交进；{failure(done)}"


# 进仓的真钩子文件（scripts/gate/git-hooks/ 里的五份）：prepare-commit-msg、pre-applypatch 那两份只在这里被真起过
@selftest_cell
def real_prepare_commit_msg_shim_no_verify_refused(box):
    repository = box.repository("repo", hooks=None)
    box.stage(repository, "big.bin", 2000)
    before = box.head(repository)
    done = box.git(repository, "commit", "-q", "--no-verify", "-m", "x", check=False)
    return refused_with(done, "单文件上限", "big.bin") and box.head(repository) == before, f"进仓的 prepare-commit-msg 在 --no-verify 下应当判、拒、提交没进；{failure(done)}"


@selftest_cell
def real_pre_applypatch_shim_am_refused(box):
    repository = box.repository("repo", hooks=None)
    patch = patch_of_feature_commit(box, repository, 2000)
    before = box.head(repository)
    done = box.git(repository, "am", "-q", patch, check=False)
    return refused_with(done, "单文件上限", "patched.bin") and box.head(repository) == before, f"进仓的 pre-applypatch 在 git am 下应当判、拒、提交没进；{failure(done)}"


# 仓位置：core.hooksPath 是绝对路径，钩子在这些相对路径找不到钩子目录的位置里照跑
@selftest_cell
def old_commit_linked_worktree_commit_refused(box):
    main = box.repository("main", hooks=None, location_independent=True)
    box.stage(main, "newer.txt", 10)
    box.commit_without_hooks(main, "newer")
    linked = os.path.join(box.directory, "linked")
    box.git(main, "worktree", "add", "-q", "--detach", linked, "HEAD~1")   # 检出的是更老的提交
    box.stage(linked, "big.bin", 2000)
    before = box.head(linked)
    done = box.git(linked, "commit", "-q", "-m", "x", check=False)
    return refused_with(done, "单文件上限", "big.bin") and box.head(linked) == before, f"链接工作树检出老提交，提交带超门限文件应当被拒；{failure(done)}"


@selftest_cell
def old_commit_linked_worktree_push_refused(box):
    main = box.repository("main", hooks=None, location_independent=True)
    remote = box.with_remote(main, "main")
    box.stage(main, "newer.txt", 10)
    box.commit_without_hooks(main, "newer")
    linked = os.path.join(box.directory, "linked")
    box.git(main, "worktree", "add", "-q", "--detach", linked, "HEAD~1")
    box.commit_big(linked, message="add big")
    done = box.git(linked, "push", "-q", "origin", "HEAD:refs/heads/old-line", check=False)
    return refused_with(done, "单文件上限", "big.bin") and not box.remote_head(remote, "old-line"), f"链接工作树检出老提交，推带超门限文件的提交应当被拒；{failure(done)}"


@selftest_cell
def sparse_checkout_commit_refused(box):
    repository = box.repository("repo", hooks=None, location_independent=True, tracked=("base.txt", "other.txt"))
    box.git(repository, "sparse-checkout", "set", "--no-cone", "/base.txt", "/big.bin")
    box.stage(repository, "big.bin", 2000)
    before = box.head(repository)
    done = box.git(repository, "commit", "-q", "-m", "x", check=False)
    return refused_with(done, "单文件上限", "big.bin") and box.head(repository) == before, f"稀疏检出的树里提交带超门限文件应当被拒；{failure(done)}"


@selftest_cell
def work_tree_option_commit_refused(box):
    repository = box.repository("repo", hooks=None, location_independent=True)
    box.stage(repository, "big.bin", 2000)
    empty = os.path.join(box.directory, "empty-directory")
    os.makedirs(empty)
    before = box.head(repository)
    done = box.git(repository, f"--work-tree={empty}", "commit", "-q", "-m", "x", check=False)
    return refused_with(done, "单文件上限", "big.bin") and box.head(repository) == before, f"git --work-tree=<空目录> commit 带超门限文件应当被拒；{failure(done)}"


@selftest_cell
def git_dir_option_push_from_outside_refused(box):
    repository = box.repository("repo", hooks=None, location_independent=True)
    remote = box.with_remote(repository, "repo")
    box.commit_big(repository, message="add big")
    outside = os.path.join(box.directory, "outside")
    os.makedirs(outside)
    done = box.git(outside, f"--git-dir={os.path.join(repository, '.git')}", "push", "-q", "origin", "main", check=False)
    return refused_with(done, "单文件上限", "big.bin") and box.remote_head(remote) != box.head(repository), f"git --git-dir=<仓> 在仓外推带超门限文件的提交应当被拒；{failure(done)}"


@selftest_cell
def git_dir_variable_push_from_subdirectory_refused(box):
    repository = box.repository("repo", hooks=None, location_independent=True)
    remote = box.with_remote(repository, "repo")
    box.commit_big(repository, message="add big")
    subdirectory = os.path.join(repository, "sub")
    os.makedirs(subdirectory)
    done = box.git(subdirectory, "push", "-q", "origin", "main", check=False, environment={"GIT_DIR": os.path.join(repository, ".git")})
    return refused_with(done, "单文件上限", "big.bin") and box.remote_head(remote) != box.head(repository), f"GIT_DIR 指到仓、在子目录里推带超门限文件的提交应当被拒；{failure(done)}"


# 推指向 blob、树的引用：对象不在任何提交的历史里，取它们能到的全部 blob、减去远端已有的
def blob_repository(box, size=2000):
    """远端取回过；big.bin 写成 blob（hash-object -w）但没有任何提交带它。交 (仓, 裸远端, blob 的对象名)。"""
    repository = box.repository("repo", hooks=PRE_PUSH_ONLY)
    remote = box.with_remote(repository, "repo")
    box.write(repository, "big.bin", size)
    return repository, remote, box.git(repository, "hash-object", "-w", "big.bin").stdout.strip()


def tree_of(box, repository, blob):
    return box.git(repository, "mktree", input_text=f"100644 blob {blob}\tbig.bin\n").stdout.strip()


@selftest_cell
def tag_to_blob_push_refused(box):
    repository, remote, blob = blob_repository(box)
    box.git(repository, "tag", "big-blob", blob)
    done = box.git(repository, "push", "-q", "origin", "refs/tags/big-blob", check=False)
    return refused_with(done, "单文件上限", "引用指向的对象") and not box.remote_ref(remote, "refs/tags/big-blob"), f"轻量标签指着超门限的 blob，推标签应当被拒；{failure(done)}"


@selftest_cell
def tag_to_tree_push_refused(box):
    repository, remote, blob = blob_repository(box)
    box.git(repository, "tag", "-a", "-m", "t", "big-tree", tree_of(box, repository, blob))
    done = box.git(repository, "push", "-q", "origin", "refs/tags/big-tree", check=False)
    return refused_with(done, "单文件上限", "big.bin") and not box.remote_ref(remote, "refs/tags/big-tree"), f"带说明的标签指着带超门限 blob 的树，推标签应当被拒；{failure(done)}"


@selftest_cell
def annotated_tag_to_blob_push_refused(box):
    repository, remote, blob = blob_repository(box)
    box.git(repository, "tag", "-a", "-m", "t", "big-blob", blob)
    done = box.git(repository, "push", "-q", "origin", "refs/tags/big-blob", check=False)
    return refused_with(done, "单文件上限", "引用指向的对象") and not box.remote_ref(remote, "refs/tags/big-blob"), f"带说明的标签指着超门限的 blob，推标签应当被拒；{failure(done)}"


@selftest_cell
def lightweight_tag_to_tree_push_refused(box):
    repository, remote, blob = blob_repository(box)
    box.git(repository, "tag", "big-tree", tree_of(box, repository, blob))
    done = box.git(repository, "push", "-q", "origin", "refs/tags/big-tree", check=False)
    return refused_with(done, "单文件上限", "big.bin") and not box.remote_ref(remote, "refs/tags/big-tree"), f"轻量标签指着带超门限 blob 的树，推标签应当被拒；{failure(done)}"


@selftest_cell
def tag_to_small_blob_push_passes(box):
    repository, remote, blob = blob_repository(box, size=10)
    box.git(repository, "tag", "small-blob", blob)
    done = box.git(repository, "push", "-q", "origin", "refs/tags/small-blob", check=False)
    return done.returncode == 0 and bool(box.remote_ref(remote, "refs/tags/small-blob")), f"标签指着小 blob，推标签应当通过；{failure(done)}"


@selftest_cell
def tag_to_blob_remote_already_has_passes(box):
    repository = box.repository("repo", hooks=PRE_PUSH_ONLY)
    remote = box.with_remote(repository, "repo")
    box.commit_big(repository, message="add big")
    box.git(repository, "-c", "core.hooksPath=/dev/null", "push", "-q", "origin", "main")   # 远端已经有这个超门限的 blob
    box.git(repository, "tag", "big-blob", box.git(repository, "hash-object", "big.bin").stdout.strip())
    done = box.git(repository, "push", "-q", "origin", "refs/tags/big-blob", check=False)
    return done.returncode == 0 and bool(box.remote_ref(remote, "refs/tags/big-blob")), f"标签指着远端已有的超门限 blob，不该再拦，应当通过；{failure(done)}"


# 自证标记：祖先进程命令行里的 --selftest 字样不算
@selftest_cell
def selftest_word_in_message_limit_ignored(box):
    repository = box.repository("repo", hooks=PRE_COMMIT_ONLY)
    box.stage(repository, "big.bin", 2000)   # 比自证门限大、比真门限小
    before = box.head(repository)
    done = box.plain_git(repository, "commit", "-q", "-m", "--selftest", environment={LIMIT_VARIABLE: str(SELFTEST_LIMIT)})
    return done.returncode == 0 and box.head(repository) != before, f"没有自证标记时钩子不认自证门限（git commit -m --selftest 的说明不是标记），2000 字节的文件应当通过；{failure(done)}"


@selftest_cell
def break_switch_without_marker_refused(box):
    repository = box.repository("repo", hooks=PRE_COMMIT_ONLY)
    box.stage(repository, "big.bin", 2000)
    before = box.head(repository)
    done = box.plain_git(repository, "commit", "-q", "-m", "--selftest", environment={BREAK_VARIABLE: "pre-commit-judge-skipped"})
    return refused_with(done, BREAK_VARIABLE, "弄坏开关") and box.head(repository) == before, f"没有自证标记时设弄坏开关应当退 2、提交没进（git commit -m --selftest 的说明不是标记）；{failure(done)}"


def processes_mentioning(marker):
    """/proc 里命令行带 marker 的进程号（只读）。"""
    found = []
    for entry in os.listdir("/proc"):
        if not entry.isdigit() or int(entry) == os.getpid():
            continue
        try:
            with open(f"/proc/{entry}/cmdline", "rb") as handle:
                text = handle.read().replace(b"\0", b" ").decode(errors="replace")
        except OSError:
            continue
        if marker in text:
            found.append(int(entry))
    return found


EXPECTED_RED = {
    "pre-commit-judge-skipped": {"commit-staged-big-refused", "commit-judge-reads-index-copy", "commit-all-grown-refused", "commit-paths-grown-refused", "private-hook-then-judge",
                                 "merge-no-commit-then-commit-refused", "judge-timeout-leaves-nothing"},
    "pre-merge-commit-judge-skipped": {"merge-big-refused"},
    "private-hook-skipped": {"private-hook-refusal-first", "private-hook-then-judge", "merge-private-hook-window"},
    "prepare-commit-msg-judge-skipped": {"commit-no-verify-refused-by-prepare-commit-msg", "stale-stamp-not-trusted", "merge-no-verify-refused-by-prepare-commit-msg",
                                         "cherry-pick-refused-by-prepare-commit-msg", "revert-restoring-big-refused", "rebase-conflict-then-continue-refused",
                                         "rebase-i-edit-then-continue-directly-refused", "real-prepare-commit-msg-shim-no-verify-refused"},
    "prepare-commit-msg-stamp-ignored": {"commit-judged-once"},
    "stale-stamp-trusted": {"stale-stamp-not-trusted"},
    "pre-applypatch-judge-skipped": {"am-big-refused", "real-pre-applypatch-shim-am-refused"},
    "push-reads-current-branch-only": {"push-clean-branch-passes", "push-other-branch-refused", "push-all-refused", "push-tags-refused", "push-new-name-refspec-refused",
                                       "push-wildcard-refspec-refused", "push-remote-already-has-big-passes", "push-to-path-remote-already-has-big-passes", "tag-to-blob-push-refused",
                                       "tag-to-tree-push-refused", "annotated-tag-to-blob-push-refused", "lightweight-tag-to-tree-push-refused"},
    "unknown-remote-not-judged": {"push-never-fetched-refused", "push-to-url-refused", "push-to-path-another-remote-has-it-refused"},
    "remote-sha-ignored": {"push-remote-already-has-big-passes", "push-to-path-remote-already-has-big-passes"},
    "push-target-not-named": {"push-to-path-another-remote-has-it-refused"},
    "push-non-commit-objects-skipped": {"tag-to-blob-push-refused", "tag-to-tree-push-refused", "annotated-tag-to-blob-push-refused", "lightweight-tag-to-tree-push-refused"},
    "push-known-blobs-not-subtracted": {"tag-to-blob-remote-already-has-passes"},
    "index-copy-kept": {"judge-timeout-leaves-nothing"},
    "judge-writes-user-index": {"commit-judge-reads-index-copy"},
    "hooks-path-written-relative": {"old-commit-linked-worktree-commit-refused", "old-commit-linked-worktree-push-refused", "sparse-checkout-commit-refused", "work-tree-option-commit-refused",
                                    "git-dir-option-push-from-outside-refused", "git-dir-variable-push-from-subdirectory-refused"},
    "selftest-marker-granted-to-plain-run": {"selftest-word-in-message-limit-ignored", "break-switch-without-marker-refused"},
}


def run_cell(task):
    """一格自证：自己建临时目录与仓、自己删；装置出错（git 起不来、文件写不进）也是这一格红，不让整份自证崩。"""
    name, switch = task
    with tempfile.TemporaryDirectory(prefix="large-file-git-hook-selftest-") as directory:
        try:
            ok, message = CELLS[name](Sandbox(directory, switch))
        except (RuntimeError, OSError, IndexError, ValueError, subprocess.SubprocessError) as error:
            ok, message = False, f"装置出错（{type(error).__name__}）：{flat(error)}"
    return name, ok, message


def run_cells(tasks, jobs):
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        return list(pool.map(run_cell, tasks))


def switch_wheel(jobs):
    """逐个打开文件头登记的弄坏开关，各自要红在点名的格（含点名的，且只红点名的）；开关与格拼成一个任务表、并进同一个线程池。"""
    tasks = [(name, switch) for switch in EXPECTED_RED for name in CELLS]
    red_of = {switch: set() for switch in EXPECTED_RED}
    for (name, switch), (_name, ok, _message) in zip(tasks, run_cells(tasks, jobs)):
        if not ok:
            red_of[switch].add(name)
    failures = 0
    for switch, cells in EXPECTED_RED.items():
        if red_of[switch] == cells:
            print(f"  · 开关轮：{switch} 红在「{'、'.join(sorted(cells))}」，与点名的相同")
        else:
            failures += 1
            print(f"  ✗ 开关轮：{switch} 打开后红在「{'、'.join(sorted(red_of[switch])) or '（没有）'}」，点名的是「{'、'.join(sorted(cells))}」")  # gate-lint:detail
    if failures:
        print(f"  ✗ 开关轮 {len(EXPECTED_RED)} 个开关里 {failures} 个没红在点名的格")
        print("     → 怎么办：改那一处判法，或照实改文件头的开关表")
        return 1
    print(f"  ✓ 开关轮：{len(EXPECTED_RED)} 个开关逐个打开（{len(tasks)} 格次，并行 {jobs} 路），各红在点名的格")
    return 0


def run_selftest(names):
    unknown = [name for name in names if name not in CELLS]
    if unknown:
        print(f"  ✗ 没有这几格自证：{'、'.join(unknown)}", file=sys.stderr)
        print(f"     → 怎么办：点名的格名从下面选：{'、'.join(CELLS)}", file=sys.stderr)
        return 2
    try:
        jobs = heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 取不了每台默认重活线程数：{error}")
        print(f"     → 怎么办：{error.howto}")
        return 2
    switch = ",".join(item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item in BREAKS)
    results = run_cells([(name, switch) for name in names or CELLS], jobs)
    for name, passed, message in results:
        if not passed:
            print(f"  ✗ 自证 {name}：{message}")  # gate-lint:detail
    failed = [name for name, passed, _ in results if not passed]
    if failed:
        print(f"  ✗ --selftest：{len(results)} 格里 {len(failed)} 格不对")
        print("     → 怎么办：照上面点名的格查钩子的判法；弄坏开关打开时红在点名的格是对的")
        return 1
    print(f"  ✓ --selftest：{len(results)} 格都对（并行 {jobs} 路；提交：暂存区、-a、点名路径、判法读的是索引拷贝；私有窗口闸先后；"
          "合并提交：通过、带超限文件被拒、--no-commit 之后收尾被拒、私有窗口闸；prepare-commit-msg：--no-verify、合并、cherry-pick、revert、rebase 冲突与 edit 之后 --continue、凭据对得上不重判；"
          "pre-applypatch：git am；仓位置：老提交的链接工作树、稀疏检出、--work-tree、--git-dir、GIT_DIR；"
          "推送：没推的提交、别的分支、--all、--tags、新名字与通配的 refspec、没取回过的远端、路径写法的远端（别的远端已有的不算它已有、它自己已有的放行）、远端已有的、只删分支、"
          "指向 blob 与树的标签；判法超时不留东西；自证标记：说明里的 --selftest 字样不是标记）；"
          "拒绝说明的出路命令喂检出钩子 ㉒：本次未跑（那一道只拦绕开钩子的写法，出路命令里没有）")
    return 0 if names or switch else switch_wheel(jobs)


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return run_selftest(arguments[1:])
    broken.update(item for item in (os.environ.get(BREAK_VARIABLE, "") if inside_selftest() else "").split(",") if item in BREAKS)
    if arguments[:1] == ["pre-commit"]:
        return pre_commit(arguments[1:])
    if arguments[:1] == ["pre-merge-commit"]:
        return pre_merge_commit(arguments[1:])
    if arguments[:1] == ["prepare-commit-msg"]:
        return prepare_commit_msg(arguments[1:])
    if arguments[:1] == ["pre-applypatch"]:
        return pre_applypatch(arguments[1:])
    if arguments[:1] == ["pre-push"]:
        return pre_push(arguments[1:], sys.stdin)
    print(f"  ✗ 认不得的参数：{' '.join(arguments)}", file=sys.stderr)
    print("     → 怎么办：这份脚本由 git 的钩子（scripts/gate/git-hooks/ 里的 pre-commit、pre-merge-commit、prepare-commit-msg、pre-applypatch、pre-push）起；自证 python3 scripts/gate/large-file-git-hook.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
