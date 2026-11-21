"""先验后换自检（入口 hook-swap.py 的 --selftest）的一块：selftest。main 的 --selftest 分支里才 import。
"""

import json
import os
import secrets
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time

import hook_swap_swap
from hook_swap_base import (
     REPOSITORY_ROOT, BREAK_VARIABLE, HOOKS_DIRECTORY, WORK_DIRECTORY_NAME, EXIT_BAD_INPUT, EXIT_CHANGED, EXIT_DRAFT_RED, EXIT_PENDING_OUTCOME,
     EXIT_ROLLED_BACK, EXIT_SWAPPED, EXIT_NEUTRAL_FEED_STALE, EXIT_PREVIOUS_SWAP_RUNNING, EXIT_SCRATCH_BUSY, OUTCOME_NAME, START_ROW_COLUMNS,
     BRACE_AND_PARAMETER_GLOB_FEEDS, BUILTIN_FEEDS, builtin_feeds_for, stale_feed_paths, BadInput, break_mode, read_text, sha256_of,
     embedded_python_segments, static_problems, feed_only, ENTRY_SCRIPT,
)
from hook_swap_callers import module_level_names, check_callers
from hook_swap_swap import (
     EXITED_PROCESS_STATES, outcome_path, write_outcome_atomically, STATE_LANDED, STATE_NOT_LANDED, read_outcome, seal_outcome, SCRATCH_LOCK_NAME,
     SCRATCH_LOCK_STALE_SECONDS, SCRATCH_LOCK_COLUMNS, REPOSITORY_LOCK_NAME, REMOVAL_SUFFIX, ScratchLock, scratch_lock_path, scratch_lock_content,
     scratch_busy_lines, read_scratch_record, link_scratch_record, release_scratch_lock, hold_scratch, acquire_record, repository_lock_path,
     hold_repository, swap,
)
from hook_swap_selftest_fixtures import (
     SELFTEST_GUARD, SELFTEST_SCOPE_GUARD, SELFTEST_LIBRARY, SELFTEST_LIBRARY_WITHOUT_V2, SELFTEST_FEEDS, SELFTEST_GOOD_EDIT,
     SELFTEST_OTHER_SESSION_LINE, SELFTEST_WAITING_GUARD_SECONDS, SELFTEST_WAITING_GUARD, SELFTEST_WAITING_FEEDS, SELFTEST_CLAIM_CHILD,
     SELFTEST_SCRIPT, SELFTEST_SCRIPT_SHELL, SELFTEST_SCRIPT_VERIFY, SELFTEST_PYTHON_VERSION_EDIT, SELFTEST_SPEC_DIRECTORY_FILES,
     SELFTEST_EXCLUDED_FILES, SELFTEST_CALLER_SAMPLES, SELFTEST_CALLERS_OF_REFUSED_V2, SELFTEST_OPAQUE_CALLER, SELFTEST_REGISTRY_CALLER,
     SELFTEST_PEER_LIBRARY, SELFTEST_REGISTRY_READ_CALLER, SELFTEST_FROM_IMPORT_CALLER, SELFTEST_REGISTRY_UNFOLDED_CALLER,
     SELFTEST_REGISTRY_GET_CALLER, SELFTEST_REGISTRY_ALIAS_CALLER, SELFTEST_REGISTRY_GET_UNFOLDED_CALLER, SELFTEST_SHADOW_LIBRARY,
     SELFTEST_SHADOW_DIRECTORY, SELFTEST_SAME_DIRECTORY_CALLER, SELFTEST_FAR_CALLER, SELFTEST_UNFOLDED_PATH_CALLER, SELFTEST_CONTINUED_HEREDOC,
     SELFTEST_CONTINUED_HEREDOC_BROKEN, SELFTEST_CONTINUED_HEREDOC_CALLER, SELFTEST_PATH_SETTER, SELFTEST_ARGUMENT_SETTER,
     SELFTEST_ENVIRONMENT_SETTER, SELFTEST_ATTRIBUTE_SIDE, SELFTEST_ATTRIBUTE_CALLER, selftest_setter_caller, SELFTEST_LOCAL_SETTER,
     selftest_tail_setter, selftest_function_setter, SELFTEST_UNRECOGNIZED_CALLERS, SELFTEST_MODULE_KEY_CALLER, SELFTEST_SHADOW_CALLERS,
)

def selftest():
    failures, checked = [], 0
    work = tempfile.mkdtemp(prefix="hook-swap-selftest-")

    def fresh_repository(name):
        root = os.path.join(work, name, "repo")
        os.makedirs(os.path.join(root, HOOKS_DIRECTORY))
        os.makedirs(os.path.join(root, "research"))
        os.makedirs(os.path.join(root, "scripts", "sub"))
        for relative, content in (("research/notes.txt", "记录\n"), ("scripts/tool.py", SELFTEST_SCRIPT), ("scripts/sub/tool.sh", SELFTEST_SCRIPT_SHELL)):
            with open(os.path.join(root, relative), "w", encoding="utf-8") as handle:
                handle.write(content)
        for file_name, text, file_mode in (("fake-guard.sh", SELFTEST_GUARD, 0o755), ("fake_lib.py", SELFTEST_LIBRARY, 0o644)):
            path = os.path.join(root, HOOKS_DIRECTORY, file_name)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text)
            os.chmod(path, file_mode)
        return root

    def hooks_file(root, file_name):
        return os.path.join(root, HOOKS_DIRECTORY, file_name)

    def run(label, spec, want_code, want_files, want_text=None, after_draft=None, after_swap=None, want_new_inode=None, verify=None, hook=".claude/hooks/fake-guard.sh"):
        nonlocal checked
        checked += 1
        root = fresh_repository(f"case-{checked}")
        scratch = os.path.join(work, f"case-{checked}", "scratch")
        inode_before = os.stat(hooks_file(root, want_new_inode)).st_ino if want_new_inode else None
        try:
            code, lines = swap(spec, hook, scratch, root, feeds=SELFTEST_FEEDS, verify=verify,
                               after_draft=(lambda: after_draft(root)) if after_draft else None,
                               after_swap=(lambda: after_swap(root)) if after_swap else None)
        except BadInput as error:
            code, lines = EXIT_BAD_INPUT, str(error).splitlines()
        problems = []
        if code != want_code:
            problems.append(f"退出码 {code}（应当 {want_code}）")
        for file_name, want in want_files.items():
            got = read_text(hooks_file(root, file_name))
            if got != want:
                problems.append(f"{file_name} 的内容不对（sha256 {sha256_of(got)[:12]}，应当 {sha256_of(want)[:12]}）")
        if want_text is not None and not any(want_text in line for line in lines):
            problems.append(f"输出里没有「{want_text}」")
        if code not in (EXIT_SWAPPED, EXIT_BAD_INPUT) and not any(line.lstrip().startswith("→") for line in lines):
            problems.append("拒绝没带「→」出路")
        leftovers = [name for name in os.listdir(os.path.join(root, HOOKS_DIRECTORY)) if name.startswith(".")]
        if leftovers:
            problems.append(f"钩子目录里留下临时文件 {leftovers}")
        if os.path.exists(os.path.join(scratch, WORK_DIRECTORY_NAME, "tree")):
            problems.append("草稿树没删")
        if want_new_inode and os.stat(hooks_file(root, want_new_inode)).st_ino == inode_before:
            problems.append(f"{want_new_inode} 换上之后 inode 没变：在原 inode 上写的")
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")

    good_library = SELFTEST_LIBRARY.replace(SELFTEST_GOOD_EDIT["old"], SELFTEST_GOOD_EDIT["new"], 1)
    untouched = {"fake-guard.sh": SELFTEST_GUARD, "fake_lib.py": SELFTEST_LIBRARY}
    # 过：草稿与换上之后都验过，定点换上（新 inode）
    run("过：草稿里验过、换上、换上之后再验过", [SELFTEST_GOOD_EDIT], EXIT_SWAPPED, {"fake-guard.sh": SELFTEST_GUARD, "fake_lib.py": good_library},
        want_text="✓ 换上了 1 份", want_new_inode="fake_lib.py")
    # 草稿红不换（开关 swap-on-draft-red 下这三格红）
    run("草稿红不换：钩子 bash 这一层的语法错（尾部多一行 if true; then）",
        [{"file": ".claude/hooks/fake-guard.sh", "old": "\nPY\n", "new": "\nPY\nif true; then\n"}], EXIT_DRAFT_RED, untouched, want_text="bash -n 不过")
    run("草稿红不换：钩子内嵌 python 编译不过，静态检查点名那一段（开关 skip-embedded-python 下红）",
        [{"file": ".claude/hooks/fake-guard.sh", "old": "sys.exit(0)\n", "new": "sys.exit(0\n"}], EXIT_DRAFT_RED, untouched,
        want_text="内嵌 python 第 1 段（第 4 行起）编译不过")
    run("草稿红不换：判法改错，该拒的放行了", [{"file": ".claude/hooks/fake_lib.py", "old": 'MARKER = "HEAVY"', "new": 'MARKER = "HEAVYX"'}],
        EXIT_DRAFT_RED, untouched, want_text="喂「重的照拒」：退 0（应当 2）")

    # 换上之后红回滚：验完草稿之后别的会话删了被调的函数；换上之后它又在同一份钩子里加了一行，回滚只改回这一次的那一处，那一行留着
    # （开关 skip-post-swap-check、rollback-whole-file 下红）
    def other_session_drops_v2(root):
        with open(hooks_file(root, "fake_lib.py"), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_LIBRARY_WITHOUT_V2)

    def other_session_appends(root):
        with open(hooks_file(root, "fake-guard.sh"), "a", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE)
    run("换上之后红回滚：反向定点替换只改回这一次的那一处，别的会话加的一行留着",
        [{"file": ".claude/hooks/fake-guard.sh", "old": "lib.refused(command)", "new": "lib.refused_v2(command)"}], EXIT_ROLLED_BACK,
        {"fake-guard.sh": SELFTEST_GUARD + SELFTEST_OTHER_SESSION_LINE, "fake_lib.py": SELFTEST_LIBRARY_WITHOUT_V2},
        want_text="已按反向定点替换回滚 1 份", after_draft=other_session_drops_v2, after_swap=other_session_appends)

    # 验的时候主工作区那份被改了：不换（开关 skip-base-check 下红）
    def other_session_edits_library(root):
        with open(hooks_file(root, "fake_lib.py"), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE + SELFTEST_LIBRARY)
    run("验的时候主工作区那份被改了：一个字节不写", [SELFTEST_GOOD_EDIT], EXIT_CHANGED,
        {"fake-guard.sh": SELFTEST_GUARD, "fake_lib.py": SELFTEST_OTHER_SESSION_LINE + SELFTEST_LIBRARY}, after_draft=other_session_edits_library)
    # 规格不收的两种
    run("规格里的文件不在 .claude/hooks/ 这一层：不收", [{"file": "research", "old": "a", "new": "b"}], EXIT_BAD_INPUT, untouched)
    run("新串在改后的文件里不止一处（回滚找不回原处）：不收", [{"file": ".claude/hooks/fake_lib.py", "old": '"HEAVY"', "new": "command"}],
        EXIT_BAD_INPUT, untouched)
    # 内嵌 python 的切段：注释行里的不算、<<- 去掉行首制表符、没收尾的报出来
    checked += 1
    segments = embedded_python_segments("# python3 - <<'X'\npython3 - <<-'EOF'\nprint(1)\n\tEOF\npython3 /dev/fd/3 3<<PY\nx = (\n")
    if [(start, closed) for start, _, closed in segments] != [(2, True), (5, False)]:
        failures.append(f"内嵌 python 切段不对：{[(start, closed) for start, _, closed in segments]}（应当 [(2, True), (5, False)]）")
    # 内置喂法里的脚本路径都在：绿是现在的 BUILTIN_FEEDS 一条都不旧；红是搬家之前的旧路径要被点名（开关 stale-builtin-feed 往真表里混一条旧的）
    checked += 1
    stale_sample = [{"label": "旧路径", "command": "bash scripts/mutate.s? crates/mutations.tsv"}, {"label": "带通配的新路径", "command": "bash scripts/mutation/mutate.s? x"}]
    if [label for label, _ in stale_feed_paths(stale_sample, REPOSITORY_ROOT)] != ["旧路径"]:
        failures.append("旧路径的喂法没被点名，或带通配的新路径被误点")
    live_feeds = [feed for feeds in BUILTIN_FEEDS.values() for feed in feeds]
    if break_mode() == "stale-builtin-feed":
        live_feeds = live_feeds + stale_sample[:1]
    for label, word in stale_feed_paths(live_feeds, REPOSITORY_ROOT):
        failures.append(f"内置喂法「{label}」里的 {word} 在 {REPOSITORY_ROOT} 下对不上：脚本搬过家，这条喂法钉不住任何东西")
    # 内置喂法带花括号与参数通配的六条（C644）：钩子自己的自证表里有，换上的钩子在这几条上退回放行时，内置喂法要能先验出来
    # （开关 brace-feeds-dropped 下这一格与下面「花括号写法上放行的钩子，先验停下」红）
    checked += 1
    guard_relative = ".claude/hooks/project-code-guard.sh"
    present = builtin_feeds_for(guard_relative, break_mode()) or []
    missing = [feed["label"] for feed in BRACE_AND_PARAMETER_GLOB_FEEDS if feed not in present]
    exits = sorted((feed["exit"] for feed in BRACE_AND_PARAMETER_GLOB_FEEDS))
    if missing or len(BRACE_AND_PARAMETER_GLOB_FEEDS) != 6 or exits != [0, 0, 2, 2, 2, 2]:
        failures.append(f"内置喂法带花括号与参数通配的六条：缺 {missing}，共 {len(BRACE_AND_PARAMETER_GLOB_FEEDS)} 条，退出码 {exits}（应当 4 条退 2、2 条退 0）")
    # 照内置喂法的表答的假钩子：全对的先验过、换上；只在第一条花括号写法（mutate.{sh,x}）上退 0 的，先验停下并点名那一条
    guard_table = {}
    for feed in BUILTIN_FEEDS[guard_relative]:
        guard_table[feed["command"]] = feed["exit"]
    brace_feed = BRACE_AND_PARAMETER_GLOB_FEEDS[0]
    brace_hook_text = ("#!/usr/bin/env bash\nset -uo pipefail\n# 版本 1\npython3 /dev/fd/3 3<<'PY'\nimport json, os, sys\n"
                       f"table = json.loads({json.dumps(json.dumps(guard_table))})\nforgiven = json.loads({json.dumps(json.dumps([brace_feed['command']]))})\n"
                       "command = json.load(sys.stdin).get('tool_input', {}).get('command', '').replace(os.environ['CLAUDE_PROJECT_DIR'], '{root}')\n"
                       "code = 0 if command in forgiven else table.get(command, 0)\n"
                       "if code:\n    print('✗ 被拒：' + command, file=sys.stderr)\n"
                       "    print('     → 怎么办：自证用的假钩子，只按内置喂法的表答，没有别的出路', file=sys.stderr)\nsys.exit(code)\nPY\n")
    correct_hook_text = brace_hook_text.replace("forgiven = json.loads(" + json.dumps(json.dumps([brace_feed["command"]])) + ")", "forgiven = []")
    for label, text, want_code, want_text in (
            ("内置喂法对全对的钩子：先验过、换上", correct_hook_text, EXIT_SWAPPED, "✓ 换上了 1 份"),
            ("花括号写法上放行的钩子，先验停下（内置喂法点名 mutate.{sh,x} 那一条）", brace_hook_text, EXIT_DRAFT_RED, f"喂「{brace_feed['label']}」：退 0（应当 2）")):
        checked += 1
        root = os.path.join(work, f"guard-case-{checked}", "repo")
        os.makedirs(os.path.join(root, HOOKS_DIRECTORY))
        guard_path = os.path.join(root, guard_relative)
        with open(guard_path, "w", encoding="utf-8") as handle:
            handle.write(text)
        try:
            code, lines = swap([{"file": guard_relative, "old": "# 版本 1\n", "new": "# 版本 2\n"}], guard_relative, os.path.join(work, f"guard-case-{checked}", "scratch"), root)
        except BadInput as error:
            code, lines = EXIT_BAD_INPUT, str(error).splitlines()
        problems = []
        if code != want_code:
            problems.append(f"退出码 {code}（应当 {want_code}）")
        if not any(want_text in line for line in lines):
            problems.append(f"输出里没有「{want_text}」")
        wrong_lines = [line for line in lines if line.lstrip().startswith("- 喂「")]
        if want_code == EXIT_DRAFT_RED and len(wrong_lines) != 1:
            problems.append(f"点名的不对的喂法应当恰好一条，实际 {len(wrong_lines)} 条")
        swapped_text = read_text(guard_path)
        if want_code == EXIT_DRAFT_RED and swapped_text != text:
            problems.append("先验红了钩子还是被改了")
        if want_code == EXIT_SWAPPED and swapped_text != text.replace("# 版本 1\n", "# 版本 2\n"):
            problems.append("全对的钩子没换成新版")
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")

    # 标了 neutral 的喂法先喂换上之前那一版（草稿树里、没套规格）：对不上的是喂法失效，退 8、点名那一条、不套规格、一个字节不写（开关 neutral-base-skipped 下前两格红）；
    # 样本是按落点判的假钩子（SELFTEST_SCOPE_GUARD）：.md 的目标落在草稿树里（草稿树在工作根下）的同一条喂法，喂主工作区的那一版退 2、喂草稿树里的那一版退 0，
    # 写闸这类按落点分仓内仓外、工作根内外的钩子上标了 neutral 的越界拒喂法就是这个样子；.txt 的目标两处都退 0，期望是 0 的喂法一律写它：
    # 喂法的目标跟着树走（{root} 在草稿里验时是草稿树、换上之后验与回滚之后核时是主工作区），期望要在这三处都成立的才能写进会换上的规格
    scope_hook_relative = ".claude/hooks/fake-scope-guard.sh"
    scope_version_edit = {"file": scope_hook_relative, "old": "# 版本 1\n", "new": "# 版本 2\n"}
    scope_refuse_everything_edit = {"file": scope_hook_relative, "old": 'if landing.endswith(".md") and not inside_work_root:', "new": "if True:"}
    scope_stale_label = "写回员写钩子目录越界拒"
    scope_allowed_label = "写范围内的文件放行"

    def scope_feed(label, want_exit, neutral):
        suffix = "md" if want_exit == 2 else "txt"
        feed_input = {"tool_name": "Write", "agent_type": "kb-scribe", "tool_input": {"file_path": f"{{root}}/.claude/hooks/hook-swap-feed-scope.{suffix}", "content": "说明\n"}}
        feed = {"label": label, "exit": want_exit, "input": feed_input}
        if neutral:
            feed["neutral"] = True
        return feed

    def scope_case(label, feeds, want_code, want_text, named=(), dry_run=False, want_swapped=False, want_refused_text=None, edit=scope_version_edit):
        nonlocal checked
        checked += 1
        name = f"scope-case-{checked}"
        root = fresh_repository(name)
        scratch = os.path.join(work, name, "scratch")
        guard_text = SELFTEST_SCOPE_GUARD.replace("__WORK_ROOT__", json.dumps(os.path.realpath(scratch)))
        guard_path = hooks_file(root, "fake-scope-guard.sh")
        with open(guard_path, "w", encoding="utf-8") as handle:
            handle.write(guard_text)
        os.chmod(guard_path, 0o755)
        try:
            code, lines = swap([edit], scope_hook_relative, scratch, root, feeds=feeds, dry_run=dry_run)
        except BadInput as error:
            code, lines = EXIT_BAD_INPUT, str(error).splitlines()
        problems = []
        if code != want_code:
            problems.append(f"退出码 {code}（应当 {want_code}）")
        if not any(want_text in line for line in lines):
            problems.append(f"输出里没有「{want_text}」")
        named_lines = [line for line in lines if line.lstrip().startswith("- 喂「")]
        if len(named_lines) != len(named) or not all(any(f"喂「{each}」" in line for line in named_lines) for each in named):
            problems.append(f"点名的喂法应当恰好是 {list(named)}，实际 {named_lines}")
        if want_refused_text is not None and not any(want_refused_text in line for line in named_lines):
            problems.append(f"点名的那一行里没有「{want_refused_text}」")
        if code != EXIT_SWAPPED and not any(line.lstrip().startswith("→") for line in lines):
            problems.append("拒绝没带「→」出路")
        wanted = guard_text.replace("# 版本 1\n", "# 版本 2\n") if want_swapped else guard_text
        if read_text(guard_path) != wanted:
            problems.append("钩子的内容不对：" + ("没换成新版" if want_swapped else "一个字节都不该写"))
        if not want_swapped and os.path.exists(outcome_path(root, scratch)):
            problems.append("没换上却写了结局文件")
        if os.path.exists(os.path.join(scratch, WORK_DIRECTORY_NAME, "tree")):
            problems.append("草稿树没删")
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")
        return root

    stale_feeds = [scope_feed(scope_allowed_label, 0, True), scope_feed(scope_stale_label, 2, True), scope_feed("新判法钉的一条", 2, False)]
    stale_root = scope_case("喂法失效：标了 neutral 的喂法对换上之前那一版在草稿树里就不成立，退 8、点名那一条、不套规格、一个字节不写", stale_feeds, EXIT_NEUTRAL_FEED_STALE,
                            "是喂法失效，不是规格的毛病", named=[scope_stale_label], want_refused_text="退 0，应当 2")
    # 样本里的那条喂法喂主工作区的那一版是对的（退 2）：失效只在草稿树里
    checked += 1
    code, lines = feed_only(scope_hook_relative, stale_root, feeds=[scope_feed(scope_stale_label, 2, True)])
    if code != EXIT_SWAPPED:
        failures.append(f"喂法失效的样本：同一条喂法喂主工作区的那一版应当对（退 {EXIT_SWAPPED}），实际 {code}；输出 {lines[-3:]}")
    scope_case("喂法失效：--dry-run 同样退 8、点名那一条", stale_feeds, EXIT_NEUTRAL_FEED_STALE, "是喂法失效，不是规格的毛病", named=[scope_stale_label], dry_run=True)
    # neutral 的喂法对旧版成立、新版不对：照旧报草稿里验不过，退 1、点名那一条（新版把工作根里的目标也拒了）
    scope_case("neutral 的喂法对旧版成立、新版不对：照旧报草稿里验不过", [scope_feed(scope_allowed_label, 0, True)], EXIT_DRAFT_RED,
               f"喂「{scope_allowed_label}」：退 2（应当 0）", named=[scope_allowed_label], edit=scope_refuse_everything_edit)
    # 没有 neutral 喂法的规格不喂旧版：非 neutral 的喂法在旧版上过不过都不管，只喂套了规格的那一版（旧版上就不成立的那一条照旧报草稿里验不过，不报喂法失效）
    scope_case("没有 neutral 喂法的规格照旧：不喂旧版，喂套了规格的那一版、换上", [scope_feed(scope_allowed_label, 0, False)], EXIT_SWAPPED, "✓ 换上了 1 份", want_swapped=True)
    scope_case("没有 neutral 喂法的规格照旧：旧版上就不成立的喂法照旧报草稿里验不过", [scope_feed(scope_stale_label, 2, False)], EXIT_DRAFT_RED,
               f"喂「{scope_stale_label}」：退 0（应当 2）", named=[scope_stale_label])
    scope_case("neutral 的喂法对旧版成立、新版也成立：照旧换上", [scope_feed(scope_allowed_label, 0, True)], EXIT_SWAPPED, "✓ 换上了 1 份", want_swapped=True)

    # 脚本规格：文件在脚本规格目录下，不喂钩子，换上之后跑 --verify 的命令
    def run_scripts(label, spec, want_code, want_files, want_text=None, verify=None, hook=None, feeds=None, dry_run=False, after_draft=None, after_swap=None,
                    want_new_inode=None):
        nonlocal checked
        checked += 1
        root = fresh_repository(f"scripts-case-{checked}")
        scratch = os.path.join(work, f"scripts-case-{checked}", "scratch")
        inode_before = os.stat(os.path.join(root, want_new_inode)).st_ino if want_new_inode else None
        try:
            code, lines = swap(spec, hook, scratch, root, feeds=feeds, dry_run=dry_run, verify=verify,
                               after_draft=(lambda: after_draft(root)) if after_draft else None,
                               after_swap=(lambda: after_swap(root)) if after_swap else None)
        except BadInput as error:
            code, lines = EXIT_BAD_INPUT, str(error).splitlines()
        problems = []
        if code != want_code:
            problems.append(f"退出码 {code}（应当 {want_code}）")
        for relative, want in want_files.items():
            got = read_text(os.path.join(root, relative))
            if got != want:
                problems.append(f"{relative} 的内容不对（sha256 {sha256_of(got)[:12]}，应当 {sha256_of(want)[:12]}）")
        if want_text is not None and not any(want_text in line for line in lines):
            problems.append(f"输出里没有「{want_text}」")
        if code != EXIT_SWAPPED and not any(line.lstrip().startswith("→") for line in lines):
            problems.append("拒绝没带「→」出路")
        for directory in ("scripts", os.path.join("scripts", "sub")):
            leftovers = [name for name in os.listdir(os.path.join(root, directory)) if name.startswith(".")]
            if leftovers:
                problems.append(f"{directory} 里留下临时文件 {leftovers}")
        if os.path.exists(os.path.join(scratch, WORK_DIRECTORY_NAME, "tree")):
            problems.append("草稿树没删")
        if want_new_inode and os.stat(os.path.join(root, want_new_inode)).st_ino == inode_before:
            problems.append(f"{want_new_inode} 换上之后 inode 没变：在原 inode 上写的")
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")

    script_untouched = {"scripts/tool.py": SELFTEST_SCRIPT, "scripts/sub/tool.sh": SELFTEST_SCRIPT_SHELL}
    script_version_edit = {"file": "scripts/tool.py", "old": "VERSION = 1\n", "new": "VERSION = 2\n"}
    script_version_two = SELFTEST_SCRIPT.replace("VERSION = 1\n", "VERSION = 2\n", 1)
    run_scripts("脚本规格：过：草稿里验过、换上、验证命令退 0（新 inode）", [script_version_edit], EXIT_SWAPPED,
                {**script_untouched, "scripts/tool.py": script_version_two}, want_text="验证命令退 0", verify=SELFTEST_SCRIPT_VERIFY, want_new_inode="scripts/tool.py")
    run_scripts("脚本规格：过：深一层的 .sh、不给 --verify 只做语法检查", [{"file": "scripts/sub/tool.sh", "old": "echo tool\n", "new": "echo tool2\n"}], EXIT_SWAPPED,
                {**script_untouched, "scripts/sub/tool.sh": "#!/usr/bin/env bash\necho tool2\n"}, want_text="✓ 换上了 1 份")
    run_scripts("脚本规格：--dry-run 停在草稿，一个字节不写", [script_version_edit], EXIT_SWAPPED, script_untouched, want_text="--dry-run：停在草稿", dry_run=True)
    # 草稿红不换（开关 swap-on-draft-red 下这两格红）
    run_scripts("脚本规格：草稿红不换：.py 编译不过", [{"file": "scripts/tool.py", "old": "return 0\n", "new": "return (0\n"}], EXIT_DRAFT_RED, script_untouched,
                want_text="编译不过", verify=SELFTEST_SCRIPT_VERIFY)
    run_scripts("脚本规格：草稿红不换：深一层的 .sh 的 bash -n 不过", [{"file": "scripts/sub/tool.sh", "old": "echo tool\n", "new": "echo tool\nif true; then\n"}],
                EXIT_DRAFT_RED, script_untouched, want_text="bash -n 不过")

    # 验证红回滚：语法对、验证命令退 1；换上之后别的会话又在同一份脚本里加了一行，回滚只改回这一次的那一处，那一行留着
    # （开关 script-verify-skipped、skip-post-swap-check、rollback-whole-file 下红）
    def other_session_appends_script(root):
        with open(os.path.join(root, "scripts", "tool.py"), "a", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE)
    run_scripts("脚本规格：验证红回滚：验证命令退 1、反向定点替换只改回这一次的那一处，别的会话加的一行留着",
                [{"file": "scripts/tool.py", "old": "    return 0\n", "new": "    return 1\n"}], EXIT_ROLLED_BACK,
                {**script_untouched, "scripts/tool.py": SELFTEST_SCRIPT + SELFTEST_OTHER_SESSION_LINE},
                want_text="验证命令退 1", verify=SELFTEST_SCRIPT_VERIFY, after_swap=other_session_appends_script)

    # 验的时候主工作区那份被改了：不换（开关 skip-base-check 下红）
    def other_session_edits_script(root):
        with open(os.path.join(root, "scripts", "tool.py"), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE + SELFTEST_SCRIPT)
    run_scripts("脚本规格：验的时候主工作区那份被改了：一个字节不写", [script_version_edit], EXIT_CHANGED,
                {**script_untouched, "scripts/tool.py": SELFTEST_OTHER_SESSION_LINE + SELFTEST_SCRIPT}, after_draft=other_session_edits_script,
                verify=SELFTEST_SCRIPT_VERIFY)
    # 规格不收的几种（开关 script-paths-unrestricted 下第一种红）
    run_scripts("脚本规格：脚本规格目录之外的文件不收", [{"file": "research/notes.txt", "old": "记录", "new": "笔记"}], EXIT_BAD_INPUT, script_untouched)
    run_scripts("脚本规格：钩子文件与脚本混在一份规格里：不收", [{"file": ".claude/hooks/fake_lib.py", "old": 'MARKER = "HEAVY"', "new": 'MARKER = "HEAVYX"'}, script_version_edit],
                EXIT_BAD_INPUT, script_untouched, hook=".claude/hooks/fake-guard.sh", feeds=SELFTEST_FEEDS)
    run_scripts("脚本规格：带 --hook：不收（脚本没有钩子要喂）", [script_version_edit], EXIT_BAD_INPUT, script_untouched, hook=".claude/hooks/fake-guard.sh")
    run_scripts("脚本规格：带 --feeds：不收", [script_version_edit], EXIT_BAD_INPUT, script_untouched, feeds=SELFTEST_FEEDS)
    run("钩子规格：带 --verify：不收", [SELFTEST_GOOD_EDIT], EXIT_BAD_INPUT, untouched, verify="true")
    run("钩子规格：不带 --hook：不收", [SELFTEST_GOOD_EDIT], EXIT_BAD_INPUT, untouched, hook=None)

    # 脚本规格目录：scripts/ 之外的五处每处一格收得进、换上（新 inode、执行位照留）、回读对；五处连同 scripts/ 的一份写进同一份规格一起换上；
    # 不收的子目录每处一格拒、一个字节不写（开关 script-gate-fixtures-accepted、script-server-target-accepted、script-server-src-accepted 下各自点名的格红）
    directory_samples = {**{relative: entry[:4] for relative, entry in SELFTEST_SPEC_DIRECTORY_FILES.items()},
                         **{relative: entry[:4] for relative, entry in SELFTEST_EXCLUDED_FILES.items()},
                         "scripts/tool.py": (SELFTEST_SCRIPT, *SELFTEST_PYTHON_VERSION_EDIT, 0o644)}

    def directories_repository(name):
        root = fresh_repository(name)
        for relative, (content, _, _, file_mode) in directory_samples.items():
            path = os.path.join(root, relative)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(content)
            os.chmod(path, file_mode)
        return root

    def directory_case(label, relatives, want_code, want_text):
        nonlocal checked
        checked += 1
        name = f"directory-case-{checked}"
        root = directories_repository(name)
        inodes = {relative: os.stat(os.path.join(root, relative)).st_ino for relative in relatives}
        spec = [{"file": relative, "old": directory_samples[relative][1], "new": directory_samples[relative][2]} for relative in relatives]
        try:
            code, lines = swap(spec, None, os.path.join(work, name, "scratch"), root)
        except BadInput as error:
            code, lines = EXIT_BAD_INPUT, str(error).splitlines()
        problems = [] if code == want_code else [f"退出码 {code}（应当 {want_code}）"]
        if not any(want_text in line for line in lines):
            problems.append(f"输出里没有「{want_text}」")
        for relative in relatives:
            path = os.path.join(root, relative)
            content, old, new, file_mode = directory_samples[relative]
            want = content.replace(old, new, 1) if want_code == EXIT_SWAPPED else content
            if read_text(path) != want:
                problems.append(f"{relative} 回读的内容不对（sha256 {sha256_of(read_text(path))[:12]}，应当 {sha256_of(want)[:12]}）")
            if want_code == EXIT_SWAPPED and os.stat(path).st_ino == inodes[relative]:
                problems.append(f"{relative} 换上之后 inode 没变：在原 inode 上写的")
            if os.stat(path).st_mode & 0o777 != file_mode:
                problems.append(f"{relative} 的权限成了 {os.stat(path).st_mode & 0o777:o}（应当 {file_mode:o}）")
            leftovers = [entry for entry in os.listdir(os.path.dirname(path)) if entry.startswith(".")]
            if leftovers:
                problems.append(f"{os.path.dirname(relative)} 里留下临时文件 {leftovers}")
        if os.path.exists(os.path.join(work, name, "scratch", WORK_DIRECTORY_NAME, "tree")):
            problems.append("草稿树没删")
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")

    for relative in SELFTEST_SPEC_DIRECTORY_FILES:
        directory_case(f"脚本规格：{os.path.dirname(relative)}/ 下的文件收得进、换上、回读对", [relative], EXIT_SWAPPED, "✓ 换上了 1 份")
    directory_case("脚本规格：六处脚本规格目录的文件写进同一份规格，一起换上、回读对", ["scripts/tool.py", *SELFTEST_SPEC_DIRECTORY_FILES], EXIT_SWAPPED,
                   f"✓ 换上了 {1 + len(SELFTEST_SPEC_DIRECTORY_FILES)} 份")
    for relative, entry in SELFTEST_EXCLUDED_FILES.items():
        directory_case(entry[4], [relative], EXIT_BAD_INPUT, "脚本规格不收这一处")

    # 调用方引到的名字：换上的库去掉一个名字，各种载入写法的调用方（spec 加函数返回、heredoc 里 import as、self 属性、形参、列表按下标取、global 元组、
    # 转手交给另一份库里的函数）都要被点名；库里名字齐的照换，认不出去向的写法列成本次未判；只核不换用同一套判法
    def callers_repository(name, opaque=False, registry=False, only=None, extra=None):
        root = fresh_repository(name)
        samples = dict(SELFTEST_CALLER_SAMPLES if only is None else only)
        if opaque:
            samples["fake_opaque.py"] = SELFTEST_OPAQUE_CALLER
        if registry:
            samples["fake_registry.py"] = SELFTEST_REGISTRY_CALLER
            samples["fake_peer.py"] = SELFTEST_PEER_LIBRARY
        for file_name, text in samples.items():
            with open(hooks_file(root, file_name), "w", encoding="utf-8") as handle:
                handle.write(text)
        for relative, text in (extra or {}).items():    # 钩子目录之外的文件（相对仓根）
            os.makedirs(os.path.dirname(os.path.join(root, relative)), exist_ok=True)
            with open(os.path.join(root, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
        return root

    def swap_callers(name, root, spec, feeds):
        try:
            return swap(spec, ".claude/hooks/fake-guard.sh", os.path.join(work, name, "scratch"), root, feeds=feeds)
        except BadInput as error:
            return EXIT_BAD_INPUT, str(error).splitlines()

    def callers_case(label, spec, feeds, want_code, want_library, want_lines=(), opaque=False, registry=False, only=None, extra=None, unwanted_lines=()):
        nonlocal checked
        checked += 1
        name = f"callers-case-{checked}"
        root = callers_repository(name, opaque, registry, only, extra)
        code, lines = swap_callers(name, root, spec, feeds)
        problems = [] if code == want_code else [f"退出码 {code}（应当 {want_code}）"]
        if read_text(hooks_file(root, "fake_lib.py")) != want_library:
            problems.append("fake_lib.py 的内容不对")
        problems += [f"输出里没有「{want}」" for want in want_lines if not any(want in line for line in lines)]
        problems += [f"输出里不该有「{unwanted}」" for unwanted in unwanted_lines if any(unwanted in line for line in lines)]
        if code != EXIT_SWAPPED and not any(line.lstrip().startswith("→") for line in lines):
            problems.append("拒绝没带「→」出路")
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")
        return lines

    removed_name_spec = [{"file": ".claude/hooks/fake_lib.py", "old": "def refused_v2(command):", "new": "def refused_v3(command):"}]
    removed_label = "引到的名字：库里去掉一个名字，各种载入写法的调用方都被点名，草稿验不过、一个字节不写"
    removed_lines = callers_case(removed_label, removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED, SELFTEST_LIBRARY,
                                 want_lines=[f"{name}:" for name in SELFTEST_CALLERS_OF_REFUSED_V2])
    named_lines = [line for line in removed_lines if "引了 fake_lib.refused_v2" in line]
    if len(named_lines) != len(SELFTEST_CALLERS_OF_REFUSED_V2) or any("fake-guard.sh" in line or "fake_outer.py" in line for line in named_lines):
        failures.append(f"{removed_label}：点名的应当恰好是引了 refused_v2 的那 {len(SELFTEST_CALLERS_OF_REFUSED_V2)} 份，实际 {len(named_lines)} 行 {named_lines}")
    callers_case("引到的名字：名字都在的换上，汇总里写缺 0 处", [SELFTEST_GOOD_EDIT], SELFTEST_FEEDS, EXIT_SWAPPED, good_library, want_lines=["缺 0 处"])
    callers_case("引到的名字：载入的库交给认不出的调用，列成本次未判、仍照常换上", [SELFTEST_GOOD_EDIT], SELFTEST_FEEDS, EXIT_SWAPPED, good_library,
                 want_lines=["本次未判（引到的名字）：.claude/hooks/fake_opaque.py", "交给 register"], opaque=True)
    # sys.modules 按库名取而本文件没存过、只用 from 库 import 名字引库的调用方：缺的名字点名，键折不出的列成本次未判
    # （开关 registry-read-unseen 下 sys.modules 那两格红，from-import-only-skipped 下 from 库 import 那一格红）
    registry_read_only = {"fake_registry_read.py": SELFTEST_REGISTRY_READ_CALLER}
    from_import_only = {"fake_from_import.py": SELFTEST_FROM_IMPORT_CALLER}
    callers_case("引到的名字：sys.modules 按库名取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写", removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED,
                 SELFTEST_LIBRARY, want_lines=["fake_registry_read.py:6 引了 fake_lib.refused_v2"], only=registry_read_only)
    callers_case("引到的名字：sys.modules 按折不出的键取而本文件没存过的，列成本次未判、仍照常换上", [SELFTEST_GOOD_EDIT], SELFTEST_FEEDS, EXIT_SWAPPED, good_library,
                 want_lines=["本次未判（引到的名字）：.claude/hooks/fake_registry_unfolded.py", "按折不出的键取"], only={"fake_registry_unfolded.py": SELFTEST_REGISTRY_UNFOLDED_CALLER})
    callers_case("引到的名字：只用 from 库 import 名字引库的调用方，缺的名字点名，草稿验不过、一个字节不写", removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED,
                 SELFTEST_LIBRARY, want_lines=["fake_from_import.py:1 引了 fake_lib.refused_v2"], only=from_import_only)
    callers_case("引到的名字：from 库 import 名字与 sys.modules 按库名取的调用方，名字都在的照换，汇总里写缺 0 处", [SELFTEST_GOOD_EDIT], SELFTEST_FEEDS, EXIT_SWAPPED,
                 good_library, want_lines=["缺 0 处"], only={**registry_read_only, **from_import_only})
    # sys.modules 的另两种取法：.get(键, 缺省) 与 from sys import modules 之后的 modules[键]，照下标那一形认，键折不出的列成本次未判
    # （开关 registry-get-ignored 下 .get 那两格红，registry-alias-ignored 下别名那一格红，registry-read-unseen 下三格都红）
    callers_case("引到的名字：sys.modules.get(键, 缺省) 取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写", removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED,
                 SELFTEST_LIBRARY, want_lines=["fake_registry_get.py:6 引了 fake_lib.refused_v2"], only={"fake_registry_get.py": SELFTEST_REGISTRY_GET_CALLER})
    callers_case("引到的名字：from sys import modules 之后 modules[键] 取而本文件没存过的，库里缺的名字点名，草稿验不过、一个字节不写", removed_name_spec, SELFTEST_FEEDS[:2],
                 EXIT_DRAFT_RED, SELFTEST_LIBRARY, want_lines=["fake_registry_alias.py:6 引了 fake_lib.refused_v2"], only={"fake_registry_alias.py": SELFTEST_REGISTRY_ALIAS_CALLER})
    callers_case("引到的名字：sys.modules.get(折不出的键) 取而本文件没存过的，列成本次未判、仍照常换上", [SELFTEST_GOOD_EDIT], SELFTEST_FEEDS, EXIT_SWAPPED, good_library,
                 want_lines=["本次未判（引到的名字）：.claude/hooks/fake_registry_get_unfolded.py", "按折不出的键取"],
                 only={"fake_registry_get_unfolded.py": SELFTEST_REGISTRY_GET_UNFOLDED_CALLER})
    # 同名的库：库按调用方运行时真解到的那一份核。scripts/shadow_library/fake_lib.py 是同名的另一份；自己目录里有同名的、往 sys.path 最前放它所在的目录、按路径载入它的调用方
    # 解到它，不核；往 sys.path 最前放 .claude/hooks/、按路径载入 .claude/hooks/fake_lib.py 的解到要核的那一份，缺的名字点名；认不出解到哪一份的列成本次未判，仓里只有一份的照核
    # （开关 library-by-stem-only 下只按文件名认：同目录那一格、点名恰好两份那一格、认不出那一格红）
    removed_library = SELFTEST_LIBRARY.replace(removed_name_spec[0]["old"], removed_name_spec[0]["new"], 1)
    shadow_library_file = {f"{SELFTEST_SHADOW_DIRECTORY}/fake_lib.py": SELFTEST_SHADOW_LIBRARY}
    callers_case("引到的名字：自己目录里有同名的库的调用方，解到那一份、不核，汇总里写缺 0 处、照常换上", removed_name_spec, SELFTEST_FEEDS[:2], EXIT_SWAPPED, removed_library,
                 want_lines=["缺 0 处"], only={},
                 extra={**shadow_library_file, f"{SELFTEST_SHADOW_DIRECTORY}/same_directory_caller.py": SELFTEST_SAME_DIRECTORY_CALLER})
    shadow_label = "引到的名字：同名的库不止一份时按调用方运行时解到的那一份核，只点名解到要核的那一份的调用方，草稿验不过、一个字节不写"
    shadow_wanted = sorted(relative for relative, (_, in_hooks) in SELFTEST_SHADOW_CALLERS.items() if in_hooks)
    shadow_lines = callers_case(shadow_label, removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED, SELFTEST_LIBRARY, want_lines=[f"{relative}:" for relative in shadow_wanted], only={},
                                extra={**shadow_library_file, **{relative: text for relative, (text, _) in SELFTEST_SHADOW_CALLERS.items()}})
    shadow_named = [line for line in shadow_lines if "引了 fake_lib.refused_v2" in line]
    if len(shadow_named) != len(shadow_wanted) or any(not any(f"{relative}:" in line for line in shadow_named) for relative in shadow_wanted):
        failures.append(f"{shadow_label}：点名的应当恰好是解到 .claude/hooks/fake_lib.py 的那 {len(shadow_wanted)} 份 {shadow_wanted}，实际 {shadow_named}")
    callers_case("引到的名字：仓里只有一份的库，离它再远的调用方也照核，缺的名字点名，草稿验不过、一个字节不写", removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED,
                 SELFTEST_LIBRARY, want_lines=["scripts/far_from_hooks/far_caller.py:5 引了 fake_lib.refused_v2"], only={},
                 extra={"scripts/far_from_hooks/far_caller.py": SELFTEST_FAR_CALLER})
    callers_case("引到的名字：同名的库不止一份而认不出解到哪一份的调用方，列成本次未判、仍照常换上", removed_name_spec, SELFTEST_FEEDS[:2], EXIT_SWAPPED, removed_library,
                 want_lines=["本次未判（引到的名字）：scripts/unfolded_path/unfolded_path_caller.py", "本次未判（引到的名字）：scripts/nowhere/nowhere_caller.py", "运行时解到哪一份认不出"], only={},
                 extra={**shadow_library_file, "scripts/unfolded_path/unfolded_path_caller.py": SELFTEST_UNFOLDED_PATH_CALLER, "scripts/nowhere/nowhere_caller.py": SELFTEST_FAR_CALLER})
    # 调用方 import 的别的模块替它往 sys.path 放目录（imported_reach）与折路径时认的两种写法（import 进来的模块的常量、环境变量顶替的缺省）：
    # 都要解到被换的那一份、缺的名字点名；放的目录折不出的列成本次未判，点名是哪份模块
    # （开关 imported-sys-path-ignored 下前两格与第三格红，environment-default-unfolded 下环境变量那一格红，imported-constants-unfolded 下常量那一格红；
    # library-by-stem-only 下折不出那一格红）
    via_import_label = "引到的名字：调用方 import 的模块往 sys.path 最前放了被换的库所在的目录，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写"
    callers_case(via_import_label, removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED, SELFTEST_LIBRARY,
                 want_lines=["scripts/via_import/via_import_caller.py:6 引了 fake_lib.refused_v2"], only={},
                 extra={"scripts/via_import/path_setter.py": SELFTEST_PATH_SETTER, "scripts/via_import/via_import_caller.py": selftest_setter_caller("path_setter"),
                        "scripts/via_import/fake_lib.py": SELFTEST_SHADOW_LIBRARY})
    callers_case("引到的名字：调用方 import 的模块往 sys.path 放的目录折不出，列成本次未判、仍照常换上", removed_name_spec, SELFTEST_FEEDS[:2], EXIT_SWAPPED, removed_library,
                 want_lines=["本次未判（引到的名字）：scripts/via_unfolded/via_unfolded_caller.py",
                             "它 import 的 scripts/via_unfolded/argument_setter.py 往 sys.path 放的目录有折不出的"], only={},
                 extra={**shadow_library_file, "scripts/via_unfolded/argument_setter.py": SELFTEST_ARGUMENT_SETTER,
                        "scripts/via_unfolded/via_unfolded_caller.py": selftest_setter_caller("argument_setter")})
    callers_case("引到的名字：调用方 import 的模块放的目录取自 os.environ.get(…) or 缺省，按缺省折，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写",
                 removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED, SELFTEST_LIBRARY,
                 want_lines=["scripts/via_environment/via_environment_caller.py:6 引了 fake_lib.refused_v2"], only={},
                 extra={**shadow_library_file, "scripts/via_environment/environment_setter.py": SELFTEST_ENVIRONMENT_SETTER,
                        "scripts/via_environment/via_environment_caller.py": selftest_setter_caller("environment_setter")})
    callers_case("引到的名字：调用方放的目录取自 import 进来的模块的常量，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写",
                 removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED, SELFTEST_LIBRARY,
                 want_lines=["scripts/via_attribute/via_attribute_caller.py:10 引了 fake_lib.refused_v2"], only={},
                 extra={**shadow_library_file, "scripts/via_attribute/attribute_side.py": SELFTEST_ATTRIBUTE_SIDE,
                        "scripts/via_attribute/via_attribute_caller.py": SELFTEST_ATTRIBUTE_CALLER})
    # 往 sys.path 放目录的写法在函数里、目录半折（几种可能、只知道末尾几段）：名字在函数里赋的照函数里那一处折；前半截折不出而末尾几段写死的、取自函数几处返回的，
    # 只在同名库所在的目录落在其中一种里时才认不出，一份都不落在里面的不挡同目录里的同名库
    # （开关 function-local-names-ignored 下这五格都红；several-choices-unfolded 下「取自函数的几处返回」两格红；known-tail-unfolded 下「末尾几段」两格与「几处返回」两格红；
    # vague-directories-ignored 下两格「列成本次未判」红；imported-sys-path-ignored 下「函数里往 sys.path 放目录」一格与两格「列成本次未判」红；
    # library-by-stem-only 下四格「照常换上」红）
    local_label = "引到的名字：调用方 import 的模块在函数里往 sys.path 放目录，名字只在那个函数里赋值，别处另有同名的，照函数里的赋值折，解到被换的那一份，缺的名字点名，草稿验不过、一个字节不写"
    tail_clear_label = "引到的名字：调用方 import 的模块放的目录前半截折不出、末尾几段与同名库所在的目录对不上，不挡同目录里的同名库，解到那一份、不核，照常换上"
    tail_match_label = "引到的名字：调用方 import 的模块放的目录前半截折不出、末尾几段正是被换的库所在的目录，列成本次未判、仍照常换上"
    choices_clear_label = "引到的名字：调用方 import 的模块放的目录取自函数的几处返回，都不是同名库所在的目录，不挡同目录里的同名库，解到那一份、不核，照常换上"
    choices_match_label = "引到的名字：调用方 import 的模块放的目录取自函数的几处返回，其中一种是被换的库所在的目录，列成本次未判、仍照常换上"
    callers_case(local_label, removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED, SELFTEST_LIBRARY,
                 want_lines=["scripts/via_local/via_local_caller.py:6 引了 fake_lib.refused_v2"], only={},
                 extra={"scripts/via_local/local_setter.py": SELFTEST_LOCAL_SETTER, "scripts/via_local/via_local_caller.py": selftest_setter_caller("local_setter"),
                        "scripts/via_local/fake_lib.py": SELFTEST_SHADOW_LIBRARY})
    for half_label, half_directory, half_setter_name, half_setter_text, half_reason in (
            (tail_clear_label, "via_tail", "tail_setter", selftest_tail_setter("scripts/frozen_side"), None),
            (tail_match_label, "via_tail_match", "tail_setter", selftest_tail_setter(".claude/hooks"), "（…/.claude/hooks）"),
            (choices_clear_label, "via_function", "function_setter", selftest_function_setter("scripts/exact_side"), None),
            (choices_match_label, "via_function_match", "function_setter", selftest_function_setter(".claude/hooks"), "（.claude/hooks、…/scripts/shared_side）")):
        half_caller = f"scripts/{half_directory}/{half_directory}_caller.py"
        half_unjudged = f"本次未判（引到的名字）：{half_caller}"
        callers_case(half_label, removed_name_spec, SELFTEST_FEEDS[:2], EXIT_SWAPPED, removed_library,
                     want_lines=["缺 0 处"] if half_reason is None else [half_unjudged, half_reason], unwanted_lines=[half_unjudged] if half_reason is None else [], only={},
                     extra={f"scripts/{half_directory}/{half_setter_name}.py": half_setter_text, half_caller: selftest_setter_caller(half_setter_name),
                            f"scripts/{half_directory}/fake_lib.py": SELFTEST_SHADOW_LIBRARY})
    # 文本里点名了库而载入与绑定都没认出的写法：逐处列成本次未判（开关 unrecognized-loads-unlisted 下红）
    callers_case("引到的名字：文本里点名了库而载入与绑定都没认出的写法（不带 as 的带点 import、SourceFileLoader、runpy.run_path），列成本次未判、仍照常换上",
                 [SELFTEST_GOOD_EDIT], SELFTEST_FEEDS, EXIT_SWAPPED, good_library,
                 want_lines=["本次未判（引到的名字）：scripts/unrecognized/dotted_import_caller.py:1 import scripts.shadow_library.fake_lib 不带 as",
                             "本次未判（引到的名字）：scripts/unrecognized/source_file_loader_caller.py:8 SourceFileLoader(…) 载入 fake_lib",
                             "本次未判（引到的名字）：scripts/unrecognized/run_path_caller.py:8 run_path(…) 载入 fake_lib"], only={},
                 extra={**shadow_library_file, **SELFTEST_UNRECOGNIZED_CALLERS})
    # X.__module__ 当 sys.modules 的键，X 从被换的库 import 来：取出的算那份库（开关 module-attribute-key-unfolded、registry-read-unseen 下红）
    callers_case("引到的名字：sys.modules[X.__module__]，X 从被换的库 import 来，取出的算那份库，缺的名字点名，草稿验不过、一个字节不写",
                 removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED, SELFTEST_LIBRARY,
                 want_lines=["fake_module_key.py:8 引了 fake_lib.refused_v2"], only={"fake_module_key.py": SELFTEST_MODULE_KEY_CALLER})
    # 起头那一行以反斜杠收尾的 heredoc：切段、静态检查、调用方引到的名字三处都从续行结束的下一行读正文（开关 heredoc-continuation-ignored 下这三格红）
    checked += 1
    continued_segments = embedded_python_segments(SELFTEST_CONTINUED_HEREDOC, break_mode())
    if continued_segments != [(3, "import sys\nprint(sys.argv)\n", True)]:
        failures.append(f"内嵌 python 切段：起头那一行以反斜杠收尾，正文从续行结束的下一行起：切出 {continued_segments}（应当 [(3, 'import sys\\nprint(sys.argv)\\n', True)]）")
    checked += 1
    continued_directory = os.path.join(work, "continued-heredoc")
    os.makedirs(continued_directory)
    continued_checked = {}
    for continued_name, continued_text in (("good", SELFTEST_CONTINUED_HEREDOC), ("broken", SELFTEST_CONTINUED_HEREDOC_BROKEN)):
        continued_path = os.path.join(continued_directory, f"{continued_name}.sh")
        with open(continued_path, "w", encoding="utf-8") as handle:
            handle.write(continued_text)
        continued_checked[continued_name] = static_problems(continued_path, f"{continued_name}.sh", break_mode())
    if continued_checked["good"] != ([], 1) or len(continued_checked["broken"][0]) != 1 or "编译不过" not in continued_checked["broken"][0][0]:
        failures.append(f"内嵌 python 静态检查：起头那一行以反斜杠收尾的 heredoc，正文编译过、正文里的语法错点名：正文没问题的交 {continued_checked['good']}（应当 ([], 1)），"
                        f"括号没收的交 {continued_checked['broken']}（应当恰好一条「编译不过」）")
    callers_case("引到的名字：起头那一行以反斜杠收尾的 heredoc 里的调用方，缺的名字点名，草稿验不过、一个字节不写", removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED,
                 SELFTEST_LIBRARY, want_lines=["fake_continued.sh:8 引了 fake_lib.refused_v2"], only={"fake_continued.sh": SELFTEST_CONTINUED_HEREDOC_CALLER})
    # 按键存进 sys.modules 再取出（SELFTEST_REGISTRY_CALLER）：别的库的键取出的名字不算到被换的库头上，被换的库自己的键取出的照核，键折不出的列成本次未判
    # （开关 subscript-key-ignored 下这三格都红：取出的都当先存进去的 fake_lib，client_only、peer_only 被点名成 fake_lib 缺的名字）
    def registry_misattributed(lines):
        return [line for line in lines if "引了 fake_lib.client_only" in line or "引了 fake_lib.peer_only" in line]

    registry_label = "引到的名字：sys.modules 按键存取，另一份库的键取出的名字不算到换上的库头上，照换、键折不出的列成本次未判"
    registry_lines = callers_case(registry_label, [SELFTEST_GOOD_EDIT], SELFTEST_FEEDS, EXIT_SWAPPED, good_library,
                                  want_lines=["缺 0 处", "本次未判（引到的名字）：.claude/hooks/fake_registry.py", "按折不出的键取"], registry=True)
    if registry_misattributed(registry_lines):
        failures.append(f"{registry_label}：别的库的名字被算到 fake_lib 头上：{registry_misattributed(registry_lines)}")
    registry_red_label = "引到的名字：sys.modules 按键存取，换上的库自己的键取出的名字真缺了仍被点名，别的库的名字不点名"
    registry_red_lines = callers_case(registry_red_label, removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED, SELFTEST_LIBRARY,
                                      want_lines=["fake_registry.py:"], registry=True)
    registry_named = [line for line in registry_red_lines if "fake_registry.py:" in line and "引了 fake_lib.refused_v2" in line]
    if len(registry_named) != 1 or registry_misattributed(registry_red_lines):
        failures.append(f"{registry_red_label}：fake_registry.py 引 fake_lib.refused_v2 的应当恰好点名 1 处（库自己的键取出的那一处），实际 {registry_named}；"
                        f"别的库的名字被点名 {registry_misattributed(registry_red_lines)}")
    # 另一份库也在这一次要核的库里：它的键取出的名字照它核（fake_peer 去掉 peer_only，点名 fake_peer.peer_only，不点名 fake_lib）
    checked += 1
    peer_label = "引到的名字：sys.modules 按键存取，另一份库也在要核的库里时，它的键取出的名字照它核、缺了点名它"
    root = callers_repository(f"callers-case-{checked}", registry=True)
    with open(hooks_file(root, "fake_peer.py"), "w", encoding="utf-8") as handle:
        handle.write(SELFTEST_PEER_LIBRARY.replace("def peer_only(", "def peer_other("))
    code, lines = check_callers(root, [".claude/hooks/fake_lib.py", ".claude/hooks/fake_peer.py"], os.environ.get(BREAK_VARIABLE, ""))
    named = [line for line in lines if line.startswith("  ✗ ")]
    if code != EXIT_DRAFT_RED or len(named) != 1 or "引了 fake_peer.peer_only" not in named[0] or "fake_registry.py:" not in named[0]:
        failures.append(f"{peer_label}：退出码 {code}（应当 {EXIT_DRAFT_RED}），点名的应当恰好是 fake_registry.py 引 fake_peer.peer_only 那一行，实际 {named}")
    # 只核不换：缺名字退 1 并逐处点名，名字齐退 0，不是 .py 的库路径拒（退 2）；一个字节不写
    checked += 1
    root = callers_repository(f"callers-case-{checked}")
    with open(hooks_file(root, "fake_lib.py"), "w", encoding="utf-8") as handle:
        handle.write(SELFTEST_LIBRARY.replace("def refused_v2(", "def refused_v3("))
    code, lines = check_callers(root, [])
    problems = [] if code == EXIT_DRAFT_RED else [f"缺名字时退 {code}（应当 {EXIT_DRAFT_RED}）"]
    if len([line for line in lines if line.startswith("  ✗ ") and "引了 fake_lib.refused_v2" in line]) != len(SELFTEST_CALLERS_OF_REFUSED_V2):
        problems.append(f"逐处点名的应当 {len(SELFTEST_CALLERS_OF_REFUSED_V2)} 行")
    with open(hooks_file(root, "fake_lib.py"), "w", encoding="utf-8") as handle:
        handle.write(SELFTEST_LIBRARY)
    code, lines = check_callers(root, [".claude/hooks/fake_lib.py"])
    if code != EXIT_SWAPPED or not any("缺 0 处" in line for line in lines):
        problems.append(f"名字齐时退 {code}（应当 {EXIT_SWAPPED}）或汇总里没有「缺 0 处」")
    try:
        check_callers(root, ["research/notes.txt"])
        problems.append("库路径不是 .py 的没被拒")
    except BadInput:
        pass
    if problems:
        failures.append(f"只核不换（--check-callers）：{'；'.join(problems)}")
    # 库里模块层的名字怎么认：import、class、def、全局赋值（元组、星号）、for / try / except 里的、函数里 global 的；动态定义的写法要报
    checked += 1
    defined, dynamic = module_level_names("import os as operating\nfrom a import b\n\n\nclass C:\n    pass\n\n\ndef f():\n    global late\n    late = 1\n\n\n"
                                          "for loop_variable in []:\n    pass\ntry:\n    import tomllib\nexcept ImportError as error:\n    tomllib = None\nx, (y, *z) = 1, (2, 3)\n")
    wanted = {"operating", "b", "C", "f", "late", "loop_variable", "tomllib", "error", "x", "y", "z"}
    if not wanted <= defined or "os" in defined or dynamic:
        failures.append(f"库里模块层的名字认错：缺 {sorted(wanted - defined)}，多了 {sorted(defined & {'os'})}，动态写法 {dynamic}")
    defined, dynamic = module_level_names("from a import *\n\n\ndef __getattr__(name):\n    return 1\n\n\nglobals()['v'] = 1\n")
    if len(dynamic) != 3:
        failures.append(f"库里动态定义名字的写法没认全：{dynamic}（应当 3 种：星号导入、__getattr__、globals()）")
    # 结局文件（文件头「结局文件」）：换上之前写开始，全换完、回读、再验过才写完成；被杀在两份之间，下一趟拒、点名反向规格；按内容判，手工退回之后不再拦
    # （开关 outcome-start-omitted 下「被杀在两份之间，只有开始」「被杀在两份之间之后，下一趟拒」两格红；outcome-complete-first 下「换完的一趟依次写」「被杀在两份之间，只有开始」
    # 「被杀在两份之间之后，下一趟拒」「全换上了还没写完成就被杀」四格红；pending-not-checked 下「被杀在两份之间之后，下一趟拒」「全换上了还没写完成就被杀」两格红；
    # outcome-reader-trusts-file 下「被杀在两份之间，只有开始」「完成之后文件又被改」「被杀之后手工退回到换前」三格红）
    class KilledForSelftest(BaseException):
        """模拟被内存上限直接杀掉：不走 except Exception，也不走回滚。"""

    outcome_spec = [{"file": ".claude/hooks/fake-guard.sh", "old": "lib.refused(command)", "new": "lib.refused_v2(command)"}, SELFTEST_GOOD_EDIT]
    outcome_hook = ".claude/hooks/fake-guard.sh"
    second_spec = [{"file": ".claude/hooks/fake_lib.py", "old": "# 新判法\n", "new": "# 新判法二\n"}]
    reader_mode = os.environ.get(BREAK_VARIABLE, "")   # 读结局文件的函数直接调，弄坏开关要自己带进去（swap() 里的 break_mode() 读的是同一个环境变量）

    def outcome_repository():
        nonlocal checked
        checked += 1
        root = fresh_repository(f"outcome-{checked}")
        subprocess.run(["git", "init", "-q", root], check=True, capture_output=True)   # 结局文件放在 git-dir 里
        return root, os.path.join(work, f"outcome-{checked}", "scratch")

    def swap_for_outcome(root, scratch, spec, **hooks):
        try:
            return swap(spec, outcome_hook, scratch, root, feeds=SELFTEST_FEEDS, **hooks)
        except BadInput as error:
            return EXIT_BAD_INPUT, str(error).splitlines()

    def kill_after_first_file(relative):
        raise KilledForSelftest()

    def kill_after_all_files():
        raise KilledForSelftest()

    def killed_swap(root, scratch, **hooks):
        try:
            swap_for_outcome(root, scratch, outcome_spec, **hooks)
        except KilledForSelftest:
            return
        failures.append("结局文件：模拟被杀的那一趟没有被杀（自证的钩子没生效）")

    # 模拟被杀的那一趟是自证进程自己起的，开始行记的进程此刻还在；要它「不在」，把开始行的进程号改成 pid_max（进程号都小于它，这个号上不会有进程）
    pid_never_used = int(read_text("/proc/sys/kernel/pid_max").strip())

    def rewrite_swap_process(path, columns):
        """改结局文件开始行里的几段（段名取 START_ROW_COLUMNS）；没有结局文件的不动（开关 outcome-start-omitted 下被杀在第一份之后就没有）。"""
        if not os.path.exists(path):
            return
        rewritten = []
        for line in read_text(path).splitlines():
            row = line.split("\t")
            if row[0] == "开始":
                for name, value in columns.items():
                    row[START_ROW_COLUMNS.index(name)] = str(value)
            rewritten.append("\t".join(row) + "\n")
        write_outcome_atomically(path, "".join(rewritten))

    label = "结局文件：换完的一趟依次写开始、文件、回读、完成，--read-outcome 退 0"
    root, scratch = outcome_repository()
    code, lines = swap_for_outcome(root, scratch, outcome_spec)
    path = outcome_path(root, scratch)
    kinds = [row.split("\t")[0] for row in read_text(path).splitlines() if row and not row.startswith("#")] if os.path.exists(path) else []
    read_code, _ = read_outcome(path, root, reader_mode) if os.path.exists(path) else (None, None)
    problems = []
    if code != EXIT_SWAPPED:
        problems.append(f"换上退 {code}（应当 {EXIT_SWAPPED}）：{lines[-3:]}")
    if kinds != ["开始", "文件", "文件", "回读", "回读", "完成"]:
        problems.append(f"结局文件的行种类 {kinds}（应当 开始、文件、文件、回读、回读、完成）")
    if not path.startswith(os.path.join(root, ".git") + os.sep):
        problems.append(f"结局文件 {path} 不在仓根的 git-dir 下")
    if read_code != 0:
        problems.append(f"--read-outcome 退 {read_code}（应当 0）")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 完成之后文件又被改：--read-outcome 退 4 并点名
    label = "结局文件：完成之后文件又被改，--read-outcome 退 4 并点名"
    if os.path.exists(path):
        with open(hooks_file(root, "fake-guard.sh"), "a", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE)
        read_code, read_lines = read_outcome(path, root, reader_mode)
        if read_code != 4 or not any("fake-guard.sh" in line for line in read_lines):
            failures.append(f"{label}：--read-outcome 退 {read_code}（应当 4），输出 {read_lines[:3]}")
    else:
        failures.append(f"{label}：上一格没留下结局文件")
    # --dry-run 不写结局文件
    label = "结局文件：--dry-run 不写结局文件"
    root, scratch = outcome_repository()
    code, lines = swap(outcome_spec, outcome_hook, scratch, root, feeds=SELFTEST_FEEDS, dry_run=True)
    if code != EXIT_SWAPPED or os.path.exists(outcome_path(root, scratch)):
        failures.append(f"{label}：退 {code}，结局文件{'在' if os.path.exists(outcome_path(root, scratch)) else '不在'}（应当 {EXIT_SWAPPED}、不在）")
    # 没有结局文件
    label = "结局文件：没有结局文件，--read-outcome 退 2"
    read_code, _ = read_outcome(outcome_path(root, scratch), root, reader_mode)
    if read_code != 2:
        failures.append(f"{label}：退 {read_code}（应当 2）")
    # 被杀在两份之间：只有开始与文件行
    label = "结局文件：被杀在两份之间，只有开始与文件行，--read-outcome 退 3、点名已换上与没换上"
    root, scratch = outcome_repository()
    killed_swap(root, scratch, after_file_swapped=kill_after_first_file)
    path = outcome_path(root, scratch)
    rewrite_swap_process(path, {"进程号": pid_never_used})   # 那一趟被杀了：开始行记的进程不在
    read_code, read_lines = read_outcome(path, root, reader_mode) if os.path.exists(path) else (None, ["（没有结局文件）"])
    problems = []
    if read_code != 3:
        problems.append(f"--read-outcome 退 {read_code}（应当 3）")
    if not any(f"{STATE_LANDED}：.claude/hooks/fake-guard.sh" in line for line in read_lines) or not any(f"{STATE_NOT_LANDED}：.claude/hooks/fake_lib.py" in line for line in read_lines):
        problems.append(f"没有逐份点名「已换上 fake-guard.sh」「没换上 fake_lib.py」：{read_lines[:4]}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 下一趟拒
    label = "结局文件：被杀在两份之间之后，下一趟拒（退 7），点名反向规格，一个字节不写"
    library_before = read_text(hooks_file(root, "fake_lib.py"))
    guard_before = read_text(hooks_file(root, "fake-guard.sh"))
    code, lines = swap_for_outcome(root, scratch, [SELFTEST_GOOD_EDIT])
    problems = []
    if code != EXIT_PENDING_OUTCOME:
        problems.append(f"下一趟退 {code}（应当 {EXIT_PENDING_OUTCOME}）")
    if not any("reverse-" in line and ".json" in line for line in lines):
        problems.append(f"拒绝说明里没有点名反向规格 reverse-*.json：{lines[:3]}")
    if read_text(hooks_file(root, "fake_lib.py")) != library_before or read_text(hooks_file(root, "fake-guard.sh")) != guard_before:
        problems.append("拒绝了还是写了字节")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 手工退回到换前之后，按内容判不再拦：自己造仓、自己被杀一趟，不借上一格留下的状态（上一格的下一趟在开关 pending-not-checked、outcome-complete-first 下照换、会把库改掉）
    label = "结局文件：被杀之后手工退回到换前，下一趟照换"
    root, scratch = outcome_repository()
    killed_swap(root, scratch, after_file_swapped=kill_after_first_file)
    rewrite_swap_process(outcome_path(root, scratch), {"进程号": pid_never_used})   # 那一趟被杀了：开始行记的进程不在
    with open(hooks_file(root, "fake-guard.sh"), "w", encoding="utf-8") as handle:
        handle.write(SELFTEST_GUARD)
    code, lines = swap_for_outcome(root, scratch, [SELFTEST_GOOD_EDIT])
    if code != EXIT_SWAPPED:
        failures.append(f"{label}：退 {code}（应当 {EXIT_SWAPPED}）：{lines[-3:]}")
    # 全换上了、还没写完成就被杀：下一趟拒，--seal-outcome 补写之后照换
    label = "结局文件：全换上了还没写完成就被杀，下一趟拒（退 7）、--seal-outcome 补写完成之后照换"
    root, scratch = outcome_repository()
    killed_swap(root, scratch, after_swap=kill_after_all_files)
    path = outcome_path(root, scratch)
    rewrite_swap_process(path, {"进程号": pid_never_used})   # 那一趟被杀了：开始行记的进程不在
    problems = []
    read_code, _ = read_outcome(path, root, reader_mode) if os.path.exists(path) else (None, None)
    if read_code != 3:
        problems.append(f"--read-outcome 退 {read_code}（应当 3：没写完成）")
    code, lines = swap_for_outcome(root, scratch, second_spec)
    if code != EXIT_PENDING_OUTCOME:
        problems.append(f"下一趟退 {code}（应当 {EXIT_PENDING_OUTCOME}）")
    seal_code, seal_lines = seal_outcome(path, root, reader_mode) if os.path.exists(path) else (None, None)
    if seal_code != EXIT_SWAPPED:
        problems.append(f"--seal-outcome 退 {seal_code}（应当 0）：{seal_lines}")
    read_code, _ = read_outcome(path, root, reader_mode) if os.path.exists(path) else (None, None)
    if read_code != 0:
        problems.append(f"补写之后 --read-outcome 退 {read_code}（应当 0）")
    code, lines = swap_for_outcome(root, scratch, second_spec)
    if code != EXIT_SWAPPED:
        problems.append(f"补写完成之后下一趟退 {code}（应当 {EXIT_SWAPPED}）：{lines[-3:]}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")

    # 上一趟没写完成时看起它的进程：每格各起一趟全换上了、还没写完成就被杀的换上（开始行记的是自证进程自己，此刻还在），按格改开始行，再起下一趟
    # （开关 running-swap-not-checked 下「进程还在」一格红；swap-process-pid-only 下「进程号被别的进程占了」「在另一台上起的」两格红；
    # pending-not-checked、outcome-complete-first 下四格都红）
    def previous_swap_case(case_label, columns, want_code, wanted, unwanted=()):
        case_root, case_scratch = outcome_repository()
        killed_swap(case_root, case_scratch, after_swap=kill_after_all_files)
        rewrite_swap_process(outcome_path(case_root, case_scratch), columns)
        before = {name: read_text(hooks_file(case_root, name)) for name in ("fake-guard.sh", "fake_lib.py")}
        case_code, case_lines = swap_for_outcome(case_root, case_scratch, second_spec)
        case_problems = [] if case_code == want_code else [f"下一趟退 {case_code}（应当 {want_code}）"]
        case_problems += [f"输出里没有「{fragment}」" for fragment in wanted if not any(fragment in line for line in case_lines)]
        case_problems += [f"输出里不该有「{fragment}」" for fragment in unwanted if any(fragment in line for line in case_lines)]
        if any(read_text(hooks_file(case_root, name)) != text for name, text in before.items()):
            case_problems.append("拒绝了还是写了字节")
        if case_problems:
            failures.append(f"{case_label}：{'；'.join(case_problems)}；输出头几行 {case_lines[:2]}")
    gone_head = "上一趟换上没有写完成（进程 {} 起的，被杀或回滚不成）"
    previous_swap_case("结局文件：上一趟的进程还在，下一趟拒（退 9），只叫等它写完成，不给退回与补写的出路，一个字节不写", {}, EXIT_PREVIOUS_SWAP_RUNNING,
                       (f"上一趟换上还没完（进程 {os.getpid()}，起了 ", "等它写完成再来"), unwanted=("reverse-", "--seal-outcome", "二选一"))
    previous_swap_case("结局文件：上一趟的进程不在了，下一趟拒（退 7），说明同被杀的那一趟", {"进程号": pid_never_used}, EXIT_PENDING_OUTCOME,
                       (gone_head.format(pid_never_used), "reverse-", "二选一"))
    previous_swap_case("结局文件：上一趟的进程号被别的进程占了（启动刻对不上），下一趟拒（退 7）", {"进程启动刻": "0"}, EXIT_PENDING_OUTCOME,
                       (gone_head.format(os.getpid()), "reverse-", "二选一"))
    previous_swap_case("结局文件：上一趟在另一台上起的，下一趟拒（退 7），先叫看那一趟还在不在跑", {"机器": "0" * 16}, EXIT_PENDING_OUTCOME,
                       ("不是在这一台上起的，这一台看不到它还在不在", "先看起它的那一趟还在不在跑", "reverse-", "二选一"))
    # 换上之后验不过、已回滚：写退回
    label = "结局文件：换上之后红回滚，写退回，--read-outcome 退 1"

    def other_session_drops_v2_for_outcome(root_now):
        with open(hooks_file(root_now, "fake_lib.py"), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_LIBRARY_WITHOUT_V2)
    root, scratch = outcome_repository()
    code, lines = swap_for_outcome(root, scratch, [{"file": ".claude/hooks/fake-guard.sh", "old": "lib.refused(command)", "new": "lib.refused_v2(command)"}],
                                   after_draft=lambda: other_session_drops_v2_for_outcome(root))
    path = outcome_path(root, scratch)
    read_code, _ = read_outcome(path, root, reader_mode) if os.path.exists(path) else (None, None)
    if code != EXIT_ROLLED_BACK or read_code != 1:
        failures.append(f"{label}：换上退 {code}（应当 {EXIT_ROLLED_BACK}），--read-outcome 退 {read_code}（应当 1）")
    # 草稿目录的占用（文件头「草稿目录的占用」）：同一个 --scratch 上并发的几趟，后一趟退 10、不建树、不写文件，前一趟照常；不同的 --scratch 互不挡；持有者没了的旧记录不挡；
    # 并发的两趟是真起的两个 hook-swap.py 进程（--dry-run），前一趟的假钩子停在被喂到的那一步；每个开关红在哪几格见文件头弄坏开关一节
    lock_mode = os.environ.get(BREAK_VARIABLE, "")
    lock_wait_seconds = SELFTEST_WAITING_GUARD_SECONDS + 10

    def lock_sample(name, waiting):
        """一份并发样本：仓、规格文件、喂法文件（路径）。waiting 的钩子被喂到时写就绪文件、等放行文件才退；不 waiting 的是平常的假钩子。"""
        root = fresh_repository(name)
        directory = os.path.join(work, name)
        sample = {"root": root, "spec": os.path.join(directory, "spec.json"), "feeds": os.path.join(directory, "feeds.json"),
                  "ready": os.path.join(directory, "ready"), "release": os.path.join(directory, "release"), "hook": ".claude/hooks/fake-guard.sh"}
        spec, feeds = [SELFTEST_GOOD_EDIT], SELFTEST_FEEDS
        if waiting:
            sample["hook"] = ".claude/hooks/fake-wait-guard.sh"
            guard_path = os.path.join(root, sample["hook"])
            with open(guard_path, "w", encoding="utf-8") as handle:
                handle.write(SELFTEST_WAITING_GUARD.replace("__READY__", sample["ready"]).replace("__RELEASE__", sample["release"]))
            os.chmod(guard_path, 0o755)
            spec, feeds = [{"file": sample["hook"], "old": "# 版本 1\n", "new": "# 版本 2\n"}], SELFTEST_WAITING_FEEDS
        for path, data in ((sample["spec"], spec), (sample["feeds"], feeds)):
            with open(path, "w", encoding="utf-8") as handle:
                json.dump(data, handle, ensure_ascii=False)
        return sample

    def start_run(sample, scratch):
        """真起一个 hook-swap.py --dry-run（自己的会话与进程组，输出合并成一份管道）。"""
        command = [sys.executable, ENTRY_SCRIPT, sample["spec"], "--hook", sample["hook"], "--scratch", scratch, "--root", sample["root"],
                   "--feeds", sample["feeds"], "--dry-run"]
        return subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)

    def live_group_members(group):
        """进程组 group 里此刻还在的进程号（/proc/<进程号>/stat 右括号之后第 3 项是进程组号；僵尸不算）。"""
        members = []
        for entry in os.listdir("/proc"):
            if not entry.isdigit():
                continue
            try:
                with open(f"/proc/{entry}/stat", encoding="utf-8", errors="replace") as handle:
                    fields = handle.read().rsplit(")", 1)[1].split()
            except (OSError, IndexError):
                continue
            if len(fields) > 2 and fields[2] == str(group) and fields[0] not in EXITED_PROCESS_STATES:
                members.append(int(entry))
        return members

    def finish_run(child, timeout_seconds):
        """等 child 退出 → (退出码, 合并输出, 组里留下的进程号)。超时、或退出后组里还有进程，只对 child 自己那一组发 KILL。"""
        note = ""
        try:
            output, _ = child.communicate(timeout=timeout_seconds)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            output, _ = child.communicate()
            note = f"\n（{timeout_seconds} 秒没退出，对它的进程组发了 KILL）"
        left = live_group_members(child.pid)
        if left:
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        return child.returncode, output + note, left

    def start_holder(sample, scratch):
        """起一趟占着 scratch 的 hook-swap.py，等它的假钩子被喂到（就绪文件出现）才交回 → (child, None)；等不到交 (None, 原因)，那一趟已收拾掉。"""
        child = start_run(sample, scratch)
        deadline = time.monotonic() + lock_wait_seconds
        while not os.path.exists(sample["ready"]):
            if child.poll() is not None or time.monotonic() > deadline:
                code, output, _ = finish_run(child, 5)
                return None, f"前一趟没走到被喂钩子那一步：退 {code}，输出末几行 {output.splitlines()[-4:]}"
            time.sleep(0.02)
        return child, None

    def release_holder(sample, child):
        with open(sample["release"], "w", encoding="utf-8"):
            pass
        return finish_run(child, lock_wait_seconds)

    def plant_record(scratch, overrides, age_seconds=0, raw=None, suffix=""):
        """在占用记录的位置放一份别人写的记录：字段取这一趟的、再用 overrides 改几段（段名取 SCRATCH_LOCK_COLUMNS），raw 给了就原样写；age_seconds 把修改时间拨回去；
        suffix 给了（REMOVAL_SUFFIX）放在记录路径加这个后缀的位置（别的趟摘旧记录时放的那份身份） → (路径, 内容)。"""
        path = scratch_lock_path(scratch, lock_mode) + suffix
        os.makedirs(os.path.dirname(path), exist_ok=True)
        row = dict(zip(SCRATCH_LOCK_COLUMNS, scratch_lock_content().rstrip("\n").split("\t")))
        row.update({column: str(value) for column, value in overrides.items()})
        text = raw if raw is not None else "\t".join(row[column] for column in SCRATCH_LOCK_COLUMNS) + "\n"
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        if age_seconds:
            old = time.time() - age_seconds
            os.utime(path, (old, old))
        return path, text

    def plain_swap(name, scratch, spec=None, dry_run=True, **hooks):
        """进程内跑一趟平常的 swap（假钩子）→ (退出码, 输出行)；BadInput 折成 (2, 行)，别的异常不接。"""
        root = fresh_repository(name)
        try:
            return swap(spec or [SELFTEST_GOOD_EDIT], ".claude/hooks/fake-guard.sh", scratch, root, feeds=SELFTEST_FEEDS, dry_run=dry_run, **hooks)
        except BadInput as error:
            return EXIT_BAD_INPUT, str(error).splitlines()

    def scratch_extras(scratch):
        """草稿目录的 hook-swap/ 里除占用记录之外还有什么（没建树、没写文件就是空的）。"""
        directory = os.path.join(scratch, WORK_DIRECTORY_NAME)
        return [name for name in sorted(os.listdir(directory)) if name != SCRATCH_LOCK_NAME] if os.path.isdir(directory) else []

    def record_leftovers(scratch):
        """占用记录所在目录里以 swap.lock 起头的东西（记录本身、摘走的旧记录、临时文件）。"""
        directory = os.path.dirname(scratch_lock_path(scratch, lock_mode))
        return [name for name in sorted(os.listdir(directory)) if name.startswith(SCRATCH_LOCK_NAME)] if os.path.isdir(directory) else []

    # 同一个 --scratch 上两趟并发
    checked += 1
    label = "草稿目录的占用：同一个 --scratch 上两趟并发，后一趟退 10、没有回溯、没碰前一趟的草稿树，前一趟照常"
    shared_scratch = os.path.join(work, "lock-shared", "scratch")
    first_sample, second_sample = lock_sample("lock-shared-first", True), lock_sample("lock-shared-second", False)
    problems = []
    first, why = start_holder(first_sample, shared_scratch)
    if first is None:
        problems.append(why)
    else:
        tree = os.path.join(shared_scratch, WORK_DIRECTORY_NAME, "tree")
        tree_before = os.path.isdir(tree)
        second_code, second_output, second_left = finish_run(start_run(second_sample, shared_scratch), lock_wait_seconds)
        tree_after = os.path.isdir(tree)
        first_code, first_output, first_left = release_holder(first_sample, first)
        if second_code != EXIT_SCRATCH_BUSY:
            problems.append(f"后一趟退 {second_code}（应当 {EXIT_SCRATCH_BUSY}）")
        if "Traceback" in second_output:
            problems.append("后一趟的输出里有 Traceback")
        problems += [f"后一趟的输出里没有「{want}」" for want in (shared_scratch, f"进程 {first.pid}", "等那一趟结束再交", "换一个 --scratch") if want not in second_output]
        if not (tree_before and tree_after):
            problems.append(f"前一趟的草稿树被动了（后一趟起之前{'在' if tree_before else '不在'}，之后{'在' if tree_after else '不在'}）")
        if first_code != EXIT_SWAPPED or "Traceback" in first_output or "✓ 草稿里验过" not in first_output:
            problems.append(f"前一趟没有照常跑完：退 {first_code}，输出末几行 {first_output.splitlines()[-4:]}")
        if second_left or first_left:
            problems.append(f"两趟的进程组里留下了进程 {second_left + first_left}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 不同的 --scratch 上两趟并发
    checked += 1
    label = "草稿目录的占用：不同的 --scratch 上两趟并发，都照常"
    first_sample, second_sample = lock_sample("lock-different-first", True), lock_sample("lock-different-second", False)
    problems = []
    first, why = start_holder(first_sample, os.path.join(work, "lock-different", "scratch-first"))
    if first is None:
        problems.append(why)
    else:
        second_code, second_output, second_left = finish_run(start_run(second_sample, os.path.join(work, "lock-different", "scratch-second")), lock_wait_seconds)
        first_code, first_output, first_left = release_holder(first_sample, first)
        if second_code != EXIT_SWAPPED or "✓ 草稿里验过" not in second_output:
            problems.append(f"后一趟没有照常跑完：退 {second_code}（应当 {EXIT_SWAPPED}），输出末几行 {second_output.splitlines()[-4:]}")
        if first_code != EXIT_SWAPPED or "Traceback" in first_output:
            problems.append(f"前一趟没有照常跑完：退 {first_code}，输出末几行 {first_output.splitlines()[-4:]}")
        if second_left or first_left:
            problems.append(f"两趟的进程组里留下了进程 {second_left + first_left}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 持有者被 KILL（来不及删记录）之后，后来的一趟不被挡
    checked += 1
    label = "草稿目录的占用：持有者被杀（留下记录）之后，后来的一趟不被挡"
    killed_scratch = os.path.join(work, "lock-killed", "scratch")
    first_sample, second_sample = lock_sample("lock-killed-first", True), lock_sample("lock-killed-second", False)
    problems = []
    first, why = start_holder(first_sample, killed_scratch)
    if first is None:
        problems.append(why)
    else:
        os.killpg(first.pid, signal.SIGKILL)
        _, _, first_left = finish_run(first, 10)
        if not os.path.exists(scratch_lock_path(killed_scratch, lock_mode)):
            problems.append("被 KILL 的持有者没有留下占用记录（样本没造成「持有者已死而记录留着」）")
        second_code, second_output, second_left = finish_run(start_run(second_sample, killed_scratch), lock_wait_seconds)
        if second_code != EXIT_SWAPPED or "✓ 草稿里验过" not in second_output:
            problems.append(f"后来的一趟被挡或没跑完：退 {second_code}（应当 {EXIT_SWAPPED}），输出末几行 {second_output.splitlines()[-4:]}")
        if second_left or first_left:
            problems.append(f"进程组里留下了进程 {second_left + first_left}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 记录里的进程号还有进程在，可启动刻对不上（进程号被别的进程占了）：不挡
    checked += 1
    label = "草稿目录的占用：记录里的进程号被别的进程占了（启动刻对不上），不挡后来的一趟"
    reused_scratch = os.path.join(work, "lock-reused", "scratch")
    plant_record(reused_scratch, {"进程启动刻": "0"})
    code, lines = plain_swap("lock-reused", reused_scratch)
    if code != EXIT_SWAPPED or not any("✓ 草稿里验过" in line for line in lines):
        failures.append(f"{label}：退 {code}（应当 {EXIT_SWAPPED}），输出末几行 {lines[-3:]}")
    # 退出时放锁：正常、规格不对（BadInput）、抛异常、Ctrl-C 各一趟，记录与临时文件都不留
    checked += 1
    label = "草稿目录的占用：退出时放锁（正常、规格不对、抛异常、Ctrl-C 各一趟），记录与临时文件都不留"
    problems = []

    def raise_runtime_error():
        raise RuntimeError("自证里造的异常")

    def raise_keyboard_interrupt():
        raise KeyboardInterrupt()
    for index, (what, spec, dry_run, hooks) in enumerate((
            ("正常 --dry-run", None, True, {}),
            ("规格不对（旧串对不上）", [{"file": ".claude/hooks/fake_lib.py", "old": "文件里没有的旧串", "new": "x"}], True, {}),
            ("抛异常", None, False, {"after_draft": raise_runtime_error}),
            ("Ctrl-C", None, False, {"after_draft": raise_keyboard_interrupt}))):
        release_scratch = os.path.join(work, f"lock-release-{index}", "scratch")
        try:
            plain_swap(f"lock-release-{index}", release_scratch, spec=spec, dry_run=dry_run, **hooks)
        except (RuntimeError, KeyboardInterrupt):
            pass
        left = record_leftovers(release_scratch)
        if left:
            problems.append(f"{what}之后留下 {left}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 被 TERM 杀掉时先放锁
    checked += 1
    label = "草稿目录的占用：被 TERM 杀掉时先放锁，退出状态仍是被 TERM 杀的"
    term_scratch = os.path.join(work, "lock-term", "scratch")
    term_sample = lock_sample("lock-term-holder", True)
    problems = []
    first, why = start_holder(term_sample, term_scratch)
    if first is None:
        problems.append(why)
    else:
        os.killpg(first.pid, signal.SIGTERM)
        term_code, term_output, term_left = finish_run(first, lock_wait_seconds)
        if term_code != -signal.SIGTERM:
            problems.append(f"退出状态 {term_code}（应当 {-signal.SIGTERM}，被 TERM 杀的）")
        if record_leftovers(term_scratch):
            problems.append(f"被 TERM 杀掉之后占用记录还在：{record_leftovers(term_scratch)}")
        if term_left:
            problems.append(f"进程组里留下了进程 {term_left}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 这一台起的、进程还在的记录挡住后来的一趟：说明写进程号与主机名，不动那份记录、不建树
    checked += 1
    label = "草稿目录的占用：这一台起的、进程还在的记录挡住后来的一趟，说明写进程号与主机名，不动那份记录、不建树"
    live_scratch = os.path.join(work, "lock-live", "scratch")
    live_path, live_text = plant_record(live_scratch, {})
    code, lines = plain_swap("lock-live", live_scratch)
    problems = [] if code == EXIT_SCRATCH_BUSY else [f"退 {code}（应当 {EXIT_SCRATCH_BUSY}）"]
    problems += [f"输出里没有「{want}」" for want in (f"进程 {os.getpid()}", socket.gethostname(), "这一台", "等那一趟结束再交", "换一个 --scratch")
                 if not any(want in line for line in lines)]
    if not any(line.lstrip().startswith("→") for line in lines):
        problems.append("拒绝没带「→」出路")
    if not os.path.exists(live_path) or read_text(live_path) != live_text:
        problems.append("被挡的一趟动了那份记录")
    if scratch_extras(live_scratch):
        problems.append(f"被挡的一趟在草稿目录里留下了 {scratch_extras(live_scratch)}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 另一台起的、刚刷新过（或离过期差一分钟、读不全）的记录挡住后来的一趟，记录原样留着
    checked += 1
    label = "草稿目录的占用：另一台起的、刚刷新过的记录（含读不全的）挡住后来的一趟"
    problems = []
    for index, (what, overrides, age, raw, wanted) in enumerate((
            ("另一台起的、刚刷新", {"机器": "f" * 16}, 0, None, "另一台"),
            ("另一台起的、离过期差一分钟", {"机器": "f" * 16}, SCRATCH_LOCK_STALE_SECONDS - 60, None, "另一台"),
            ("读不全的、刚刷新", {}, 0, "乱码\n", "读不全"))):
        elsewhere_scratch = os.path.join(work, f"lock-elsewhere-{index}", "scratch")
        elsewhere_path, elsewhere_text = plant_record(elsewhere_scratch, overrides, age, raw)
        code, lines = plain_swap(f"lock-elsewhere-{index}", elsewhere_scratch)
        if code != EXIT_SCRATCH_BUSY or not any(wanted in line for line in lines):
            problems.append(f"{what}：退 {code}（应当 {EXIT_SCRATCH_BUSY}），输出里{'有' if any(wanted in line for line in lines) else '没有'}「{wanted}」")
        if not os.path.exists(elsewhere_path) or read_text(elsewhere_path) != elsewhere_text:
            problems.append(f"{what}：被挡的一趟动了那份记录")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 另一台起的、过了期还没刷新的记录（含读不全的）不挡
    checked += 1
    label = "草稿目录的占用：另一台起的、过了期还没刷新的记录（含读不全的）不挡后来的一趟"
    problems = []
    for index, (what, overrides, raw) in enumerate((("另一台起的、过了期", {"机器": "f" * 16}, None), ("读不全的、过了期", {}, "乱码\n"))):
        stale_scratch = os.path.join(work, f"lock-stale-{index}", "scratch")
        plant_record(stale_scratch, overrides, SCRATCH_LOCK_STALE_SECONDS + 60, raw)
        code, lines = plain_swap(f"lock-stale-{index}", stale_scratch)
        if code != EXIT_SWAPPED or not any("✓ 草稿里验过" in line for line in lines):
            problems.append(f"{what}：退 {code}（应当 {EXIT_SWAPPED}），输出末几行 {lines[-3:]}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 持有期间每隔心跳间隔刷新记录的修改时间（把修改时间拨回一小时前，等它被刷新）
    checked += 1
    label = "草稿目录的占用：持有期间每隔心跳间隔刷新记录的修改时间"
    beat_scratch = os.path.join(work, "lock-heartbeat", "scratch")
    held, busy = hold_scratch(beat_scratch, lock_mode, heartbeat_seconds=0.05)
    problems = []
    if held is None:
        problems.append(f"占不到：{busy}")
    else:
        try:
            if not os.path.exists(held.path):
                problems.append("没有放占用记录")
            else:
                long_ago = time.time() - 3600
                os.utime(held.path, (long_ago, long_ago))
                deadline = time.monotonic() + 3
                while time.monotonic() < deadline and time.time() - os.stat(held.path).st_mtime > 60:
                    time.sleep(0.02)
                if time.time() - os.stat(held.path).st_mtime > 60:
                    problems.append("持有期间记录的修改时间没有被刷新")
        finally:
            release_scratch_lock(held)
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 几趟同时放记录：只有一趟放成，放成的内容写全，临时文件都收拾掉
    checked += 1
    label = "草稿目录的占用：几趟同时放记录只有一趟放成，放成的内容写全"
    race_path = os.path.join(work, "lock-race", SCRATCH_LOCK_NAME)
    os.makedirs(os.path.dirname(race_path))
    contenders = 8
    contents = [scratch_lock_content() for _ in range(contenders)]
    results = [None] * contenders
    start_together = threading.Barrier(contenders)

    def contend(index):
        start_together.wait()
        results[index] = link_scratch_record(race_path, contents[index], lock_mode)
    racers = [threading.Thread(target=contend, args=(index,)) for index in range(contenders)]
    for racer in racers:
        racer.start()
    for racer in racers:
        racer.join()
    winners = [index for index, won in enumerate(results) if won]
    problems = [] if len(winners) == 1 else [f"放成的有 {len(winners)} 趟（应当 1）"]
    if len(winners) == 1 and read_text(race_path) != contents[winners[0]]:
        problems.append("留下的记录不是放成那一趟写的内容")
    stray = [name for name in os.listdir(os.path.dirname(race_path)) if name != SCRATCH_LOCK_NAME]
    if stray:
        problems.append(f"临时文件没收拾：{stray}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 放不了记录（草稿目录里的 hook-swap 是个文件）：退 2、给出路，不抛回溯
    checked += 1
    label = "草稿目录的占用：放不了记录时退 2 并给出路，不抛回溯"
    blocked_scratch = os.path.join(work, "lock-unwritable", "scratch")
    os.makedirs(blocked_scratch)
    with open(os.path.join(blocked_scratch, WORK_DIRECTORY_NAME), "w", encoding="utf-8"):
        pass
    try:
        code, lines = plain_swap("lock-unwritable", blocked_scratch)
    except Exception as error:  # noqa: BLE001 自证要把任何没接住的异常折成红格
        code, lines = None, [f"抛了 {type(error).__name__}：{error}"]
    problems = [] if code == EXIT_BAD_INPUT else [f"退 {code}（应当 {EXIT_BAD_INPUT}）"]
    if not any("放不了草稿目录的占用记录" in line for line in lines) or not any(line.lstrip().startswith("→") for line in lines):
        problems.append(f"拒绝说明里没有「放不了草稿目录的占用记录」或没带「→」出路：{lines[:2]}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 三趟同时摘同一份旧记录：remover 判过旧记录不挡、动手摘之前 A 摘掉它放成自己的；remover 摘的当中 C 来放。只许一趟占着，占着那一趟的记录还在盘上
    # （开关 stale-record-renamed-away 下 A 的记录被挪开、C 放成、放回去撞上 C 的、A 的记录被删，A 与 C 同时占着）
    checked += 1
    label = "草稿目录的占用：三趟同时摘同一份旧记录，只有一趟占着，占着那一趟的记录还在盘上，没留下临时文件"
    three_scratch = os.path.join(work, "lock-three", "scratch")
    three_path, _ = plant_record(three_scratch, {"进程号": pid_never_used}, 100)   # 写它的那一趟已经不在
    holders = {}

    def holder_takes(name):
        holders[name], _ = hold_scratch(three_scratch, lock_mode, heartbeat_seconds=3600)
    remover = ScratchLock(three_path, scratch_lock_content(), lock_mode)
    remover_busy = acquire_record(remover, SCRATCH_LOCK_COLUMNS, 3600, after_judged=lambda: holder_takes("A"), after_taken=lambda: holder_takes("C"))
    holders["remover"] = remover if remover_busy is None else None
    holding = sorted(name for name, held in holders.items() if held is not None)
    problems = [] if len(holding) == 1 else [f"同时占着的有 {holding or '没有'}（应当只有一趟）"]
    if len(holding) == 1 and (not os.path.exists(three_path) or read_scratch_record(three_path) != holders[holding[0]].content):
        problems.append(f"占着的 {holding[0]} 的记录不在盘上（被别的趟删了）")
    stray = [name for name in os.listdir(os.path.dirname(three_path)) if name.startswith(SCRATCH_LOCK_NAME) and name != SCRATCH_LOCK_NAME]
    if stray:
        problems.append(f"留下了 {stray}")
    for held in holders.values():
        if held is not None:
            release_scratch_lock(held)
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # link 在服务端做成而回复丢了：报已有、读回来是自己写的记录，当放成；报已有而读回来是别人的记录，不算放成（开关 link-reply-lost-refused 下前一半红，scratch-record-overwrites 下后一半红）
    checked += 1
    label = "草稿目录的占用：link 在服务端做成而回复丢了（报已有、读回来是自己写的记录），当放成；报已有而读回来是别人的记录，不算放成"
    reply_path = os.path.join(work, "lock-reply-lost", SCRATCH_LOCK_NAME)
    foreign_path = os.path.join(work, "lock-reply-foreign", SCRATCH_LOCK_NAME)
    os.makedirs(os.path.dirname(reply_path))
    os.makedirs(os.path.dirname(foreign_path))
    mine, foreign = scratch_lock_content(), scratch_lock_content()

    def link_then_report_exists(source, target):
        os.link(source, target)
        raise FileExistsError(17, "File exists (自证：回复丢了，重发时报已有)", target)
    problems = []
    if link_scratch_record(reply_path, mine, lock_mode, link=link_then_report_exists) is not True:
        problems.append("自己的记录已在盘上、link 报已有，没当放成")
    if read_text(reply_path) != mine:
        problems.append("盘上的记录不是自己写的内容")
    with open(foreign_path, "w", encoding="utf-8") as handle:
        handle.write(foreign)
    if link_scratch_record(foreign_path, scratch_lock_content(), lock_mode) is not False or read_text(foreign_path) != foreign:
        problems.append("别人的记录在，被当成放成了或被动了")
    stray = [name for directory in (reply_path, foreign_path) for name in os.listdir(os.path.dirname(directory)) if name != SCRATCH_LOCK_NAME]
    if stray:
        problems.append(f"临时文件没收拾：{stray}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 摘旧记录的占用（.taking，文件头「草稿目录的占用」）：摘的那一趟被 KILL、被 TERM 来不及放、建 .taking 的 link 回复丢了，留下的 .taking 都不再挡后来的一趟；
    # 摘的那一趟还占着的才挡，并在拒绝说明里点名它。摘的那一趟是真起的子进程（SELFTEST_CLAIM_CHILD），停在 .taking 已放成之后再发信号
    # （开关 taking-claim-unjudged 下 KILL 一格与过了期那一格红，taking-claim-kept-on-term 下 TERM 一格红，taking-reply-lost-refused 下回复丢了一格红，taking-holder-unnamed 下挡住的一格红）
    claim_child = os.path.join(work, "claim-child.py")
    with open(claim_child, "w", encoding="utf-8") as handle:
        handle.write(SELFTEST_CLAIM_CHILD)

    def stop_claiming_child(record, kind, signal_number, held_scratch=""):
        """起一个子进程（自己的会话）摘 record 上已经不在的那一趟留下的旧记录，等它停在 .taking 已放成之后，对它那一组发 signal_number →
        (到没到摘的当中, 发信号时 .taking 在不在, 退出码, 合并输出, 组里留下的进程号)。held_scratch 给了，子进程摘之前先占住这个草稿目录。"""
        marker = os.path.join(work, f"claiming-{secrets.token_hex(4)}")
        command = [sys.executable, claim_child, ENTRY_SCRIPT, record, kind, marker, lock_mode] + ([held_scratch] if held_scratch else [])
        child = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
        deadline = time.monotonic() + lock_wait_seconds
        while not os.path.exists(marker) and child.poll() is None and time.monotonic() < deadline:
            time.sleep(0.02)
        reached = os.path.exists(marker)
        claim_seen = os.path.exists(record + REMOVAL_SUFFIX)
        if child.poll() is None:
            os.killpg(child.pid, signal_number)
        code, output, left = finish_run(child, lock_wait_seconds)
        return reached, claim_seen, code, output, left

    def claiming_problems(what, result, wanted_code):
        """stop_claiming_child 的结果有没有造成样本（走到摘的当中、.taking 在、按 wanted_code 退出、组里不留进程） → 问题列表。"""
        reached, claim_seen, code, output, left = result
        if not reached:
            return [f"{what}：子进程没走到摘旧记录的当中：退 {code}，输出末几行 {output.splitlines()[-3:]}"]
        problems = []
        if not claim_seen:
            problems.append(f"{what}：摘的当中没有 {REMOVAL_SUFFIX}（样本没造成「摘的那一趟停在 {REMOVAL_SUFFIX} 放成之后」）")
        if code != wanted_code:
            problems.append(f"{what}：子进程的退出状态 {code}（应当 {wanted_code}）")
        if left:
            problems.append(f"{what}：进程组里留下了进程 {left}")
        return problems

    def plant_dead_record(path, repository):
        """path 上放一份写它的那一趟已经不在的记录（repository 为真的是按仓的记录，末尾多一段草稿目录）。"""
        row = scratch_lock_content("dead-holder-scratch" if repository else None).rstrip("\n").split("\t")
        row[SCRATCH_LOCK_COLUMNS.index("进程号")] = str(pid_never_used)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("\t".join(row) + "\n")
    # 摘旧记录的那一趟被 KILL（来不及放 .taking）：按仓的真换、--seal-outcome、草稿目录三处的占用，后来的一趟都占得到，.taking 当场摘掉
    checked += 1
    label = "草稿目录的占用：摘旧记录的那一趟被 KILL（留下 .taking）之后，按仓的真换、--seal-outcome 与草稿目录的占用都不被挡，.taking 当场摘掉"
    root, scratch = outcome_repository()
    outcome_file = outcome_path(root, scratch)
    problems = []
    for what, record, kind, take in (
            ("按仓的真换", repository_lock_path(outcome_file, scratch, lock_mode), "repository", lambda: hold_repository(outcome_file, scratch, lock_mode)),
            ("--seal-outcome", repository_lock_path(outcome_file, None, lock_mode), "repository", lambda: hold_repository(outcome_file, None, lock_mode)),
            ("草稿目录", scratch_lock_path(scratch, lock_mode), "scratch", lambda: hold_scratch(scratch, lock_mode))):
        plant_dead_record(record, kind == "repository")
        killed = stop_claiming_child(record, kind, signal.SIGKILL)
        problems += claiming_problems(what, killed, -signal.SIGKILL)
        held, busy = take()
        if held is None:
            problems.append(f"{what}被留下的 {REMOVAL_SUFFIX} 挡住：{busy[:1]}")
        else:
            release_scratch_lock(held)
        if os.path.exists(record + REMOVAL_SUFFIX):
            problems.append(f"{what}占到之后 {REMOVAL_SUFFIX} 还在盘上")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 摘旧记录的当中被 TERM 杀掉（停任务发的就是它）：摘的那一趟的处理放掉 .taking，没占着草稿目录与已占着草稿目录的各一趟，退出状态仍是被 TERM 杀的
    checked += 1
    label = "草稿目录的占用：摘旧记录的当中被 TERM 杀掉（没占着草稿目录与已占着各一趟），放掉 .taking 与草稿目录的记录，退出状态仍是被 TERM 杀的"
    problems = []
    for index, (what, kind, holds_scratch) in enumerate((("没占着草稿目录的一趟", "scratch", False), ("已占着草稿目录的一趟", "repository", True))):
        term_base = os.path.join(work, f"lock-taking-term-{index}")
        held_scratch = os.path.join(term_base, "held-scratch")
        record = os.path.join(term_base, "git-dir", REPOSITORY_LOCK_NAME) if kind == "repository" else scratch_lock_path(os.path.join(term_base, "scratch"), lock_mode)
        plant_dead_record(record, kind == "repository")
        terminated = stop_claiming_child(record, kind, signal.SIGTERM, held_scratch if holds_scratch else "")
        problems += claiming_problems(what, terminated, -signal.SIGTERM)
        if os.path.exists(record + REMOVAL_SUFFIX):
            problems.append(f"{what}被 TERM 杀掉之后 {REMOVAL_SUFFIX} 还在盘上")
        if holds_scratch and os.path.exists(scratch_lock_path(held_scratch, lock_mode)):
            problems.append(f"{what}被 TERM 杀掉之后草稿目录的占用记录还在盘上")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 建 .taking 的 link 在服务端做成而回复丢了：只发生在摘旧记录那一次 link 上，报已有、读回来是自己写的，当放成，不是别的趟正在摘
    checked += 1
    label = "草稿目录的占用：摘旧记录那一次建 .taking 的 link 在服务端做成而回复丢了（报已有、读回来是自己写的），当放成，摘掉旧记录，不留 .taking"
    lost_scratch = os.path.join(work, "lock-taking-lost", "scratch")
    lost_path, _ = plant_record(lost_scratch, {"进程号": pid_never_used}, 100)
    lost_replies = []
    real_link = os.link

    def link_reports_exists_for_taking(source, target):
        real_link(source, target)
        if str(target).endswith(REMOVAL_SUFFIX):
            lost_replies.append(target)
            raise FileExistsError(17, "File exists (自证：回复丢了，重发时报已有)", target)
    lost_lock = ScratchLock(lost_path, scratch_lock_content(), lock_mode)
    os.link = link_reports_exists_for_taking
    try:
        lost_busy = acquire_record(lost_lock, SCRATCH_LOCK_COLUMNS, 3600)
    finally:
        os.link = real_link
    problems = [] if lost_replies else [f"样本没造成「link 在服务端做成而回复丢了」（建 {REMOVAL_SUFFIX} 的 link 没被调到）"]
    if lost_busy is not None:
        problems.append(f"当成了别的趟正在摘：{lost_busy}")
    else:
        if read_scratch_record(lost_path) != lost_lock.content:
            problems.append("放成之后盘上的记录不是自己写的")
        release_scratch_lock(lost_lock)
    if os.path.exists(lost_path + REMOVAL_SUFFIX):
        problems.append(f"留下了 {REMOVAL_SUFFIX}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # .taking 在、写它的那一趟还占着（这一台起的进程还在、另一台起的刚放下）：挡住后来的一趟，拒绝说明点名摘的那一趟与 .taking，记录与 .taking 都不动；
    # 另一台起的、过了期还没动的 .taking 不挡，摘掉旧记录照常占到
    checked += 1
    label = "草稿目录的占用：.taking 在而摘的那一趟还占着，挡住后来的一趟并被点名；另一台起的、过了期的不挡"
    problems = []
    for index, (what, overrides, age, wanted_words) in enumerate((
            ("这一台起的、进程还在", {}, 0, (f"进程 {os.getpid()}", "这一台")),
            ("另一台起的、刚放下", {"机器": "f" * 16}, 0, ("另一台",)),
            ("另一台起的、过了期", {"机器": "f" * 16}, SCRATCH_LOCK_STALE_SECONDS + 60, None))):
        taking_scratch = os.path.join(work, f"lock-taking-{index}", "scratch")
        record_path, record_text = plant_record(taking_scratch, {"进程号": pid_never_used}, 100)   # 记录是已经不在的那一趟留下的
        taking_path, taking_text = plant_record(taking_scratch, overrides, age, suffix=REMOVAL_SUFFIX)
        code, lines = plain_swap(f"lock-taking-{index}", taking_scratch)
        if wanted_words is None:
            if code != EXIT_SWAPPED or not any("✓ 草稿里验过" in line for line in lines):
                problems.append(f"{what}：退 {code}（应当 {EXIT_SWAPPED}），输出末几行 {lines[-3:]}")
            if os.path.exists(taking_path):
                problems.append(f"{what}：{REMOVAL_SUFFIX} 没摘掉")
            continue
        if code != EXIT_SCRATCH_BUSY:
            problems.append(f"{what}：退 {code}（应当 {EXIT_SCRATCH_BUSY}）")
        problems += [f"{what}：输出里没有「{want}」" for want in wanted_words + (REMOVAL_SUFFIX,) if not any(want in line for line in lines)]
        if not any(line.lstrip().startswith("→") for line in lines):
            problems.append(f"{what}：拒绝没带「→」出路")
        if not os.path.exists(record_path) or read_text(record_path) != record_text or not os.path.exists(taking_path) or read_text(taking_path) != taking_text:
            problems.append(f"{what}：被挡的一趟动了记录或 {REMOVAL_SUFFIX}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 两趟同时摘同一份已死的 .taking（记录与 .taking 都是已经不在的那一趟留下的）：两趟都判过 .taking 不再挡、都要动手删它。两条线程照固定次序交错：
    # 包住 os.unlink 与 os.link 的闸门，先删已死 .taking 的一趟等另一趟也走到删的那一步，另一趟等它放上自己的 .taking 才删；删记录同样一前一后
    # （先删的放上自己的记录之后，另一趟才删）。只许一趟放得成摘取记录：另一趟被挡、走不到删的那一步，闸门等它结束就放行。草稿目录的记录与按仓的记录各一格。
    # 闸门另记下删 .taking 的每一下删的是谁放的：只许删已死的那一份与自己放的。这一条单独核，不靠「占到的只有一趟」：两趟都删得下手时，后删的一趟删掉先删那一趟刚放上的 .taking，
    # 先删的那一趟由删记录前后的核对（removal_claim_lost）认出被接手、不占，占到的照样只有一趟
    # （开关 taking-reap-unclaimed 下两格都红：后删的一趟删掉了先删那一趟放上的 .taking；taking-claim-unjudged 下两格红、dead-holder-blocks 下草稿目录那一格红：都被挡、没有一趟占到；
    # stale-record-renamed-away 下草稿目录那一格红：不放 .taking，没有一趟走到删已死的 .taking）
    reap_roles = ("first", "second")
    real_unlink = os.unlink

    def double_reap_problems(record, repository, take):
        """record 上放已死的记录与 .taking（repository 为真的记录末尾多一段草稿目录），两条线程各起一趟 take() → 占到的 ScratchLock 或 None，
        照上面的次序交错 → 问题列表（含删掉另一趟放上的 .taking 的每一下）；占到的几趟收尾时放掉。"""
        taking = record + REMOVAL_SUFFIX
        plant_dead_record(record, repository)
        plant_dead_record(taking, False)
        dead_taking = read_scratch_record(taking)
        reached = {(role, step): threading.Event() for role in reap_roles for step in ("taking", "record", "linked-taking", "linked-record", "done")}
        gated_steps = {taking: "taking", record: "record"}
        placed_by = {}   # 放上 .taking 的内容 → 放的那一趟
        dead_removers, foreign_removals = [], []   # 删掉已死 .taking 的几趟；(删的那一趟, 放的那一趟)：删掉的是另一趟放上的 .taking

        def note_taking_removal(role):
            """role 这一趟正要删 .taking：删的是已死的那一份记进 dead_removers，不是 role 自己放的记进 foreign_removals。"""
            try:
                current = read_scratch_record(taking)
            except FileNotFoundError:
                return
            if current == dead_taking:
                dead_removers.append(role)
            elif placed_by.get(current) != role:
                foreign_removals.append((role, placed_by.get(current, "不知哪一趟")))

        def wait_other(own, step):
            """own 这一趟停在闸门上，等另一趟走到 step 或已经结束（最多 lock_wait_seconds 秒）。"""
            other = reap_roles[1 - reap_roles.index(own)]
            deadline = time.monotonic() + lock_wait_seconds
            while not (reached[(other, step)].is_set() or reached[(other, "done")].is_set()) and time.monotonic() < deadline:
                time.sleep(0.01)

        def gated_unlink(path, *arguments, **keywords):
            role, step = threading.current_thread().name, gated_steps.get(os.fspath(path))
            if role in reap_roles and step is not None and not reached[(role, step)].is_set():
                reached[(role, step)].set()
                wait_other(role, step if role == reap_roles[0] else f"linked-{step}")
            if role in reap_roles and step == "taking":
                note_taking_removal(role)
            return real_unlink(path, *arguments, **keywords)

        def gated_link(source, target, *arguments, **keywords):
            role, step = threading.current_thread().name, gated_steps.get(os.fspath(target))
            placing = read_scratch_record(source) if role in reap_roles and step == "taking" else None
            result = real_link(source, target, *arguments, **keywords)
            if placing is not None:
                placed_by[placing] = role
            if role in reap_roles and step is not None:
                reached[(role, f"linked-{step}")].set()
            return result
        held, errors = {}, {}

        def trip(role):
            try:
                held[role] = take()
            except Exception as error:  # noqa: BLE001 自证要把任何没接住的异常折成红格
                errors[role] = f"{type(error).__name__}：{error}"
            finally:
                reached[(role, "done")].set()
        threads = [threading.Thread(target=trip, args=(role,), name=role) for role in reap_roles]
        os.unlink, os.link = gated_unlink, gated_link
        try:
            for thread in threads:
                thread.start()
            for thread in threads:
                thread.join(lock_wait_seconds * 4)
        finally:
            os.unlink, os.link = real_unlink, real_link
        problems = [f"{role} 抛了 {why}" for role, why in errors.items()]
        problems += [f"{thread.name} 没跑完" for thread in threads if thread.is_alive()]
        problems += [f"{remover} 删掉了 {owner} 放上的 {REMOVAL_SUFFIX}（只许删已死的那一份与自己放的）" for remover, owner in foreign_removals]
        if len(dead_removers) > 1:
            problems.append(f"删已死的 {REMOVAL_SUFFIX} 的有 {dead_removers}（应当只有一趟）")
        holding = [role for role in reap_roles if held.get(role) is not None]
        if len(holding) != 1:
            problems.append(f"占到的有 {holding or '没有'}（应当只有一趟）")
        else:
            if not reached[(holding[0], "taking")].is_set():
                problems.append(f"占到的 {holding[0]} 没走到删已死的 {REMOVAL_SUFFIX} 那一步（样本没造成「两趟都判过 {REMOVAL_SUFFIX} 不再挡」）")
            if not os.path.exists(record) or read_scratch_record(record) != held[holding[0]].content:
                problems.append(f"占到的 {holding[0]} 的记录不在盘上")
        stray = sorted(name for name in os.listdir(os.path.dirname(record)) if name.startswith(os.path.basename(record)) and name != os.path.basename(record))
        if stray:
            problems.append(f"留下了 {stray}")
        for lock in held.values():
            if lock is not None:
                release_scratch_lock(lock)
        return problems

    def scratch_trip(record):
        """一趟占草稿目录的记录 record（照 hold_scratch 的放法，不经草稿目录算路径）→ 占到的 ScratchLock 或 None。"""
        lock = ScratchLock(record, scratch_lock_content(), lock_mode)
        return lock if acquire_record(lock, SCRATCH_LOCK_COLUMNS, 3600) is None else None
    checked += 1
    label = "草稿目录的占用：两趟同时摘同一份已死的 .taking（草稿目录的记录），只有一趟删得了 .taking、占到，它的记录在盘上，不留 .taking 与摘取记录"
    reap_record = os.path.join(work, "lock-taking-reap", SCRATCH_LOCK_NAME)
    problems = double_reap_problems(reap_record, False, lambda: scratch_trip(reap_record))
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    checked += 1
    label = "按仓的占用：两趟同时摘同一份已死的 .taking（按仓的记录），只有一趟删得了 .taking、占到，它的记录在盘上，不留 .taking 与摘取记录"
    reap_outcome = os.path.join(work, "lock-taking-reap-repository", "git-dir", OUTCOME_NAME)
    reap_scratch = os.path.join(work, "lock-taking-reap-repository", "scratch")
    problems = double_reap_problems(repository_lock_path(reap_outcome, reap_scratch, lock_mode), True,
                                    lambda: hold_repository(reap_outcome, reap_scratch, lock_mode, heartbeat_seconds=3600)[0])
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 摘的那一趟放着 .taking 的期间刷新它的修改时间：摘的那一趟（线程，.taking 上的身份写成另一台）停在 .taking 放成之后，把 .taking 的修改时间拨回过期之前、等心跳刷新它；
    # 这时来的一趟被挡住、.taking 原样，摘的那一趟放行之后照常占到（开关 taking-no-heartbeat 下红：没刷新，来的一趟当它过期、摘掉 .taking 占了；
    # elsewhere-holder-ignored 下红：来的一趟不看刷新、另一台的一律当放了；dead-holder-blocks、stale-record-renamed-away 下红：摘的那一趟走不到放着 .taking 的当中）
    elsewhere_thread = "remover-elsewhere"
    real_scratch_lock_content = scratch_lock_content
    elsewhere_machine = "f" * 16

    # 占用记录的代码读的是 hook_swap_swap 模块里的 scratch_lock_content（acquire_record 在那里按全局名取），换身份的替身要换在那个模块上；
    # 换在本模块的全局名上，被测代码照读真函数，摘的那一趟放的 .taking 写的还是这一台的身份。
    def use_scratch_lock_content(replacement):
        hook_swap_swap.scratch_lock_content = replacement

    def taking_machine(path):
        """path 上那份身份的机器一段；读不到交 None。"""
        try:
            return read_scratch_record(path).split("\t")[SCRATCH_LOCK_COLUMNS.index("机器")]
        except (OSError, IndexError):
            return None

    def content_elsewhere_for_remover(directory=None):
        """线程 elsewhere_thread 里写的身份把机器一段换成另一台的，别的线程照常（摘的那一趟放的 .taking 看着是另一台起的）。"""
        text = real_scratch_lock_content(directory)
        if threading.current_thread().name != elsewhere_thread:
            return text
        fields = text.split("\t")
        fields[SCRATCH_LOCK_COLUMNS.index("机器")] = elsewhere_machine
        return "\t".join(fields)

    def unrefreshed_seconds(path):
        """path 此刻多少秒没刷新；不在交 0。"""
        try:
            return time.time() - os.stat(path).st_mtime
        except FileNotFoundError:
            return 0
    checked += 1
    label = "草稿目录的占用：摘的那一趟放着 .taking 期间刷新它的修改时间，另一台上还活着的摘的那一趟放了超过过期秒数也照挡后来的一趟"
    beat_record = os.path.join(work, "lock-taking-heartbeat", SCRATCH_LOCK_NAME)
    beat_taking = beat_record + REMOVAL_SUFFIX
    plant_dead_record(beat_record, False)
    paused, resume = threading.Event(), threading.Event()
    beat_remover = ScratchLock(beat_record, scratch_lock_content(), lock_mode)
    beat_result = {}

    def remover_pauses():
        paused.set()
        resume.wait(lock_wait_seconds)

    def beat_remover_runs():
        try:
            beat_result["busy"] = acquire_record(beat_remover, SCRATCH_LOCK_COLUMNS, 0.05, after_taken=remover_pauses)
        finally:
            paused.set()
    problems = []
    use_scratch_lock_content(content_elsewhere_for_remover)
    try:
        remover_thread = threading.Thread(target=beat_remover_runs, name=elsewhere_thread)
        remover_thread.start()
        paused.wait(lock_wait_seconds)
        taking_text = read_scratch_record(beat_taking) if os.path.exists(beat_taking) else None
        if taking_text is None:
            problems.append(f"摘的那一趟没走到放着 {REMOVAL_SUFFIX} 的当中（样本没造成）")
        elif taking_machine(beat_taking) != elsewhere_machine:
            problems.append(f"摘的那一趟放的 {REMOVAL_SUFFIX} 身份不是另一台起的（样本没造成：替身没换到占用记录读的那个模块上）")
        else:
            long_ago = time.time() - SCRATCH_LOCK_STALE_SECONDS - 60
            os.utime(beat_taking, (long_ago, long_ago))
            deadline = time.monotonic() + 3
            while time.monotonic() < deadline and unrefreshed_seconds(beat_taking) > SCRATCH_LOCK_STALE_SECONDS:
                time.sleep(0.02)
            if unrefreshed_seconds(beat_taking) > SCRATCH_LOCK_STALE_SECONDS:
                problems.append(f"摘的那一趟放着 {REMOVAL_SUFFIX} 期间没刷新它的修改时间")
            comer = ScratchLock(beat_record, scratch_lock_content(), lock_mode)
            if acquire_record(comer, SCRATCH_LOCK_COLUMNS, 3600) is None:
                problems.append(f"来的一趟没被挡住（把还活着的摘的那一趟当成过期，摘掉了它的 {REMOVAL_SUFFIX}）")
                release_scratch_lock(comer)
            if not os.path.exists(beat_taking) or read_scratch_record(beat_taking) != taking_text:
                problems.append(f"来的一趟动了摘的那一趟的 {REMOVAL_SUFFIX}")
        resume.set()
        remover_thread.join(lock_wait_seconds)
    finally:
        use_scratch_lock_content(real_scratch_lock_content)
    if beat_result.get("busy", "没跑完") is not None:
        problems.append(f"摘的那一趟放行之后没占到：{beat_result.get('busy', '没跑完')}")
    else:
        release_scratch_lock(beat_remover)
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 摘的那一趟停住被接手：摘的那一趟（线程，.taking 上的身份写成另一台，心跳间隔一小时：整个进程停住、心跳也停）核过记录、正要删的时候停在包住 os.unlink 的闸门上，
    # .taking 的修改时间拨回过期之前；这时来的一趟当它死了，摘掉它的 .taking、接手摘下去、占到。摘的那一趟放行之后删掉的是接手那一趟的记录，
    # 删之后核出 .taking 已不是自己的，不占、拒绝说明写被接手（开关 taken-over-unchecked 下红：两趟都占到；elsewhere-holder-never-expires、taking-claim-unjudged 下红：
    # 来的一趟被挡、摘的那一趟照占；dead-holder-blocks、stale-record-renamed-away 下红：摘的那一趟走不到删记录之前）
    checked += 1
    label = "草稿目录的占用：摘的那一趟停住、.taking 被当成已死接手之后，它恢复了不占、拒绝说明写被接手，只有接手的一趟占到"
    over_record = os.path.join(work, "lock-taking-taken-over", SCRATCH_LOCK_NAME)
    over_taking = over_record + REMOVAL_SUFFIX
    plant_dead_record(over_record, False)
    at_record_unlink, stopped_or_done, comer_done = threading.Event(), threading.Event(), threading.Event()

    def unlink_stops_remover(path, *arguments, **keywords):
        if threading.current_thread().name == elsewhere_thread and os.fspath(path) == over_record and not at_record_unlink.is_set():
            at_record_unlink.set()
            stopped_or_done.set()
            comer_done.wait(lock_wait_seconds)
        return real_unlink(path, *arguments, **keywords)
    over_remover = ScratchLock(over_record, scratch_lock_content(), lock_mode)
    over_result = {}

    def over_remover_runs():
        try:
            over_result["busy"] = acquire_record(over_remover, SCRATCH_LOCK_COLUMNS, 3600)
        finally:
            stopped_or_done.set()
    problems = []
    comer, comer_busy = None, "没起"
    os.unlink = unlink_stops_remover
    use_scratch_lock_content(content_elsewhere_for_remover)
    try:
        remover_thread = threading.Thread(target=over_remover_runs, name=elsewhere_thread)
        remover_thread.start()
        stopped_or_done.wait(lock_wait_seconds)
        if not at_record_unlink.is_set() or not os.path.exists(over_taking):
            problems.append("摘的那一趟没停在核过记录、删之前（样本没造成）")
        elif taking_machine(over_taking) != elsewhere_machine:
            problems.append(f"摘的那一趟放的 {REMOVAL_SUFFIX} 身份不是另一台起的（样本没造成：替身没换到占用记录读的那个模块上）")
        else:
            long_ago = time.time() - SCRATCH_LOCK_STALE_SECONDS - 60
            os.utime(over_taking, (long_ago, long_ago))
            comer = ScratchLock(over_record, scratch_lock_content(), lock_mode)
            comer_busy = acquire_record(comer, SCRATCH_LOCK_COLUMNS, 3600)
        comer_done.set()
        remover_thread.join(lock_wait_seconds)
    finally:
        os.unlink = real_unlink
        use_scratch_lock_content(real_scratch_lock_content)
    over_busy = over_result.get("busy", "没跑完")
    if comer_busy is not None:
        problems.append(f"接手的一趟没占到：{comer_busy}")
    if over_busy is None:
        problems.append("摘的那一趟恢复之后照占（没认出被接手）")
    elif not isinstance(over_busy, tuple) or not any("接手" in line for line in scratch_busy_lines(os.path.dirname(over_record), over_record, *over_busy)):
        problems.append(f"摘的那一趟不占，可拒绝说明里没写被接手：{over_busy}")
    stray = sorted(name for name in os.listdir(os.path.dirname(over_record)) if name.startswith(SCRATCH_LOCK_NAME) and name != SCRATCH_LOCK_NAME)
    if stray:
        problems.append(f"留下了 {stray}")
    for lock, busy in ((comer, comer_busy), (over_remover, over_busy)):
        if lock is not None and busy is None:
            release_scratch_lock(lock)
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 按仓的占用：真换在草稿验完之前，另一个草稿目录的真换进来，在读结局文件之前退 10、点名前一趟的草稿目录、什么都不写；另一个草稿目录的 --dry-run 照常；前一趟照常换完
    # （开关 repository-lock-keyed-by-scratch 下后一趟的真换也换上、前一趟读到基线被改而退 3）
    checked += 1
    label = "按仓的占用：两个草稿目录的真换，后一趟在读结局文件之前退 10、点名前一趟的草稿目录、没写任何东西，前一趟照常换完；另一个草稿目录的 --dry-run 照常"
    root, scratch = outcome_repository()
    other_scratch, dry_scratch = os.path.join(os.path.dirname(scratch), "scratch-other"), os.path.join(os.path.dirname(scratch), "scratch-dry")
    inner = {}

    def other_scratch_swaps():
        inner["real"] = swap_for_outcome(root, other_scratch, [SELFTEST_GOOD_EDIT])
        inner["dry"] = swap_for_outcome(root, dry_scratch, [SELFTEST_GOOD_EDIT], dry_run=True)
    code, lines = swap_for_outcome(root, scratch, outcome_spec, after_draft=other_scratch_swaps)
    real_code, real_lines = inner.get("real", (None, []))
    problems = []
    if real_code != EXIT_SCRATCH_BUSY:
        problems.append(f"后一趟的真换退 {real_code}（应当 {EXIT_SCRATCH_BUSY}）：{real_lines[-2:]}")
    if not any(f"用的草稿目录 {scratch}" in line and "这个仓的换上" in line for line in real_lines) or not any(line.lstrip().startswith("→") for line in real_lines):
        problems.append(f"后一趟的拒绝说明没点名前一趟的草稿目录 {scratch} 或没带「→」出路：{real_lines[:2]}")
    other_work = os.path.join(other_scratch, WORK_DIRECTORY_NAME)
    if os.path.isdir(other_work) and os.listdir(other_work):
        problems.append(f"后一趟在它的草稿目录里留下了 {os.listdir(other_work)}")
    if inner.get("dry", (None, []))[0] != EXIT_SWAPPED:
        problems.append(f"另一个草稿目录的 --dry-run 退 {inner.get('dry', (None, []))[0]}（应当 {EXIT_SWAPPED}）")
    if code != EXIT_SWAPPED:
        problems.append(f"前一趟退 {code}（应当 {EXIT_SWAPPED}）：{lines[-2:]}")
    kinds = [row.split("\t")[0] for row in read_text(outcome_path(root, scratch)).splitlines() if row and not row.startswith("#")] if os.path.exists(outcome_path(root, scratch)) else []
    if kinds.count("开始") != 1:
        problems.append(f"结局文件里的开始行有 {kinds.count('开始')} 条（应当 1 条，后一趟没写）")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 补写完成要占按仓的占用：一趟真换正在换上（占着记录），--seal-outcome 占不到，退 10、不补写，那一趟照常写完成
    # （开关 seal-outcome-lock-skipped 下它改退 9：没占记录、只剩看开始行那一趟还在；repository-lock-keyed-by-scratch 下同）
    checked += 1
    label = "补写完成：另一趟真换正在换上时，--seal-outcome 占不到按仓的占用，退 10、不补写"
    root, scratch = outcome_repository()
    sealed = {}

    def seal_while_swapping():
        sealed["code"], sealed["lines"] = seal_outcome(outcome_path(root, scratch), root, reader_mode)
    code, lines = swap_for_outcome(root, scratch, outcome_spec, after_swap=seal_while_swapping)
    problems = [] if sealed.get("code") == EXIT_SCRATCH_BUSY else [f"--seal-outcome 退 {sealed.get('code')}（应当 {EXIT_SCRATCH_BUSY}）：{sealed.get('lines', [])[:1]}"]
    if not any(line.lstrip().startswith("→") for line in sealed.get("lines", [])):
        problems.append("拒绝没带「→」出路")
    if code != EXIT_SWAPPED:
        problems.append(f"换上的那一趟退 {code}（应当 {EXIT_SWAPPED}）：{lines[-2:]}")
    if "补记" in read_text(outcome_path(root, scratch)):
        problems.append("被拒的 --seal-outcome 还是写了完成行")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 开始行记的那一趟还在、可没占着记录（占记录的那一趟没了、开始行的进程是自证自己）：--seal-outcome 退 9、不补写（开关 seal-outcome-running-unchecked、outcome-complete-first 下红）
    checked += 1
    label = "补写完成：开始行那一趟还在（没占着记录），--seal-outcome 退 9、不补写"
    root, scratch = outcome_repository()
    killed_swap(root, scratch, after_swap=kill_after_all_files)
    path = outcome_path(root, scratch)
    before_text = read_text(path) if os.path.exists(path) else None
    seal_code, seal_lines = seal_outcome(path, root, reader_mode) if os.path.exists(path) else (None, ["（没有结局文件）"])
    problems = [] if seal_code == EXIT_PREVIOUS_SWAP_RUNNING else [f"--seal-outcome 退 {seal_code}（应当 {EXIT_PREVIOUS_SWAP_RUNNING}）：{seal_lines[:1]}"]
    if not any(f"进程 {os.getpid()}" in line for line in seal_lines):
        problems.append(f"拒绝说明没点名开始行记的进程 {os.getpid()}")
    if before_text is not None and read_text(path) != before_text:
        problems.append("被拒的 --seal-outcome 还是改了结局文件")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 喂法目标：file_path 解开符号链接之后要落在草稿树里（开关 feed-target-unchecked 下前两格红；第三格写在实拷目录下，开关下照绿）
    def target_feed(feed_label, file_path):
        return {"label": feed_label, "exit": 0, "input": {"tool_name": "Write", "tool_input": {"file_path": file_path, "content": "记录\n"}}}
    checked += 1
    label = "喂法目标落在符号链接上：草稿红、点名那一条与出路、不喂"
    link_root = fresh_repository("target-link")
    link_scratch = os.path.join(work, "target-link", "scratch")
    link_feeds = [target_feed("写仓根下的链接目标", "{root}/research/notes.txt"), target_feed("相对路径落在链接上", "scripts/tool.py"),
                  target_feed("写实拷目录下的目标", "{root}/.claude/hooks/feed-target.md")]
    code, lines = swap([SELFTEST_GOOD_EDIT], ".claude/hooks/fake-guard.sh", link_scratch, link_root, feeds=link_feeds, dry_run=True)
    feed_logs = os.path.join(link_scratch, WORK_DIRECTORY_NAME)
    log_text = "".join(read_text(os.path.join(feed_logs, name)) for name in sorted(os.listdir(feed_logs)) if name.startswith("feeds-"))
    problems = []
    if code != EXIT_DRAFT_RED:
        problems.append(f"退出码 {code}（应当 {EXIT_DRAFT_RED}）")
    for named in ("写仓根下的链接目标", "相对路径落在链接上"):
        if not any(f"喂「{named}」" in line and "出路：" in line for line in lines):
            problems.append(f"没点名「{named}」与出路")
        if f"\t{named}\t退 " in log_text:
            problems.append(f"「{named}」落在树外还是喂了")
    if any("喂「写实拷目录下的目标」" in line for line in lines) or "\t写实拷目录下的目标\t退 0" not in log_text:
        problems.append("写在实拷目录下的目标被点名或没喂")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")
    checked += 1
    label = "喂法目标写在实拷的 .claude/hooks/ 下：照喂、换上"
    inside_root = fresh_repository("target-inside")
    code, lines = swap([SELFTEST_GOOD_EDIT], ".claude/hooks/fake-guard.sh", os.path.join(work, "target-inside", "scratch"), inside_root,
                       feeds=[target_feed("绝对路径写实拷目录", "{root}/.claude/hooks/feed-target.md"),
                              target_feed("相对路径写实拷目录", ".claude/hooks/feed-target.md"), *SELFTEST_FEEDS])
    if code != EXIT_SWAPPED:
        failures.append(f"{label}：退出码 {code}（应当 {EXIT_SWAPPED}）；输出末几行 {lines[-4:]}")
    checked += 1
    label = "只喂不换：喂法目标落在仓根之外：点名那一条与出路"
    only_root = fresh_repository("target-feed-only")
    code, lines = feed_only(".claude/hooks/fake-guard.sh", only_root,
                            feeds=[target_feed("写仓根之外的目标", os.path.join(work, "target-feed-only", "outside.md")),
                                   target_feed("写仓根里的目标", "{root}/research/notes.txt")])
    problems = []
    if code != EXIT_DRAFT_RED:
        problems.append(f"退出码 {code}（应当 {EXIT_DRAFT_RED}）")
    if not any("✗" in line and "喂「写仓根之外的目标」" in line and "出路：" in line for line in lines):
        problems.append("没点名「写仓根之外的目标」与出路")
    if not any("✓" in line and "写仓根里的目标" in line for line in lines):
        problems.append("仓根里的目标没喂过")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")
    shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ hook-swap 自检：{failure}")  # gate-lint:detail
    if failures:
        print(f"  → 看 swap() / check_tree() / rollback() / embedded_python_segments() 的判法；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ hook-swap 自检通过（查了 {checked} 种）：草稿与换上之后都验过的换上新 inode；钩子 bash 层语法错、内嵌 python 编译不过、判法改错的草稿一个字节不写；"
          "换上之后验不过的按反向定点替换回滚、别的会话这段时间里加的一行留着；验的时候主工作区被改了不换；"
          "不在 .claude/hooks/ 也不在脚本规格目录下的文件、回滚找不回原处的规格不收；内嵌 python 切段认 <<- 与没收尾的；内置喂法里的脚本路径都在仓里；内置喂法带花括号与参数通配的六条，"
          "在花括号写法上放行的钩子先验停下并点名那一条；脚本规格（脚本规格目录下任意层）：草稿语法检查、--dry-run 停在草稿、换上之后验证命令退 0 才算验过，"
          "验证命令退 1 按反向定点替换回滚（别的会话加的一行留着）、验的时候被改不换、脚本规格目录之外的文件与钩子脚本混写的规格不收、"
          ".claude/gate.d/、.claude/scripts/、task-server/、prune-server/、gpu-verification-server/ 下的文件各自收得进、换上（新 inode、权限照留）、回读对，"
          "六处的文件写进同一份规格一起换上，.claude/gate.d/fixtures/ 与两个服务的 target/、src/ 下的文件不收、一个字节不写、"
          "脚本规格带 --hook 或 --feeds、钩子规格带 --verify 或不带 --hook 都不收；换上的 .py 库去掉一个名字，七种载入写法的调用方（spec 加函数返回、heredoc 里 import as、self 属性、形参、"
          "列表按下标取、global 元组、转手交给另一份库的函数）都被点名、草稿验不过，名字齐的照换，认不出去向的写法列成本次未判；只核不换 --check-callers 缺名字退 1、名字齐退 0；"
          "调用方 import 的别的模块往 sys.path 放的目录（含取自模块常量、环境变量没设时的缺省的）照真解到的那一份核、折不出的列成本次未判，"
          "函数里赋的名字照函数里那一处折，前半截折不出而末尾几段写死的目录、取自函数几处返回的目录只在同名库所在的目录落在其中一种里时才列成本次未判，"
          "文本里点名了库而载入与绑定都没认出的写法列成本次未判，sys.modules[X.__module__] 的 X 从被换的库 import 来的算那份库，"
          "起头那一行以反斜杠收尾的 heredoc 从续行结束的下一行读正文（切段、静态检查、调用方引到的名字三处）；"
          "sys.modules 按键存取的，换上的库自己的键取出的名字照核、缺了点名，另一份库的键取出的名字不算到换上的库头上（那份库也在要核的库里时照它核），键折不出的列成本次未判；"
          "本文件没存过的 sys.modules 按库名取、只用 from 库 import 名字引库的调用方，缺的名字同样点名，按折不出的键取的列成本次未判；"
          "喂法的 file_path 解开符号链接之后落在草稿树外的，草稿红、点名那一条与出路、不喂（只喂不换同样点名），写在实拷的 .claude/hooks/ 下的照喂、换上；"
          "标了 neutral 的喂法对换上之前那一版在草稿树里就不成立的，退 8、点名那一条、不套规格、一个字节不写（--dry-run 同样），对旧版成立而新版不对的照旧报草稿里验不过，"
          "没有 neutral 喂法的规格不喂旧版；"
          "结局文件：换完的一趟依次写开始、文件、回读、完成，--dry-run 不写，被杀在两份之间只有开始与文件行、下一趟拒（退 7）并点名反向规格、手工退回到换前之后不再拦，"
          "全换上了没写完成的要 --seal-outcome 补写，换上之后红回滚写退回，完成之后文件又被改 --read-outcome 退 4；"
          "没写完成而起它的进程还在这一台上（机器、开机、启动刻都对得上）的下一趟退 9、只叫等它，进程不在、进程号被别的进程占了的退 7 照被杀的那一趟办，"
          "在另一台起的退 7、先叫看那一趟还在不在跑；"
          "草稿目录的占用：同一个 --scratch 上两趟并发（真起两个进程），后一趟退 10、没有回溯、没碰前一趟的草稿树，前一趟照常，不同的 --scratch 都照常；"
          "持有者被杀、进程号被别的进程占了的旧记录不挡；另一台起的记录看修改时间（刚刷新过的挡、过了期的不挡，读不全的同样）；"
          "退出（正常、规格不对、抛异常、Ctrl-C）与被 TERM 杀掉时放锁；持有期间刷新记录；几趟同时放记录只有一趟放成；放不了记录退 2 给出路；"
          "三趟同时摘同一份旧记录只有一趟占着、它的记录还在盘上；link 回复丢了而读回来是自己的记录当放成、是别人的不算；"
          "摘旧记录的占用 .taking：摘的那一趟被 KILL（按仓的真换、--seal-outcome、草稿目录三处）、被 TERM（占着与没占着草稿目录各一趟）、建 .taking 的 link 回复丢了，留下的都不再挡后来的一趟，"
          "摘的那一趟还占着的挡住并在拒绝说明里点名它，另一台起的过了期的不挡；两趟同时摘同一份已死的 .taking（草稿目录的与按仓的记录各一格）只有一趟删得了、占到，没有一趟删掉另一趟放上的 .taking，不留摘取记录；"
          "摘的那一趟放着 .taking 期间刷新它，另一台上还活着的照挡后来的一趟；摘的那一趟停住被接手之后恢复了不占、拒绝说明写被接手；"
          "按仓的占用：两个草稿目录的真换后一趟在读结局文件之前退 10 并点名前一趟的草稿目录、什么都不写，另一个草稿目录的 --dry-run 照常；"
          "--seal-outcome 占同一份（真换正在换上时退 10）、开始行那一趟还在时退 9，都不补写")
    return 0
