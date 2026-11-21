# 写闸 write-guard.sh 的 python 部分之一：自证 selftest、弄坏开关只在自证里认的拒绝与读 hook 输入的入口流程。
# 入口 write-guard.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 WRITE_GUARD_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from __main__ import SELFTEST_MARK_CONTENT, SELFTEST_MARK_VARIABLE, WRITE_GUARD_PART_NAMES
from write_guard_decisions import (
    DEFAULT_DETECTIONS, HOOK_SWAP_SCRIPT, LEDGER_REGISTRY, MAKE_SWAP_SPEC_SCRIPT, NOTATIONS_LIBRARIES, NOTATIONS_LIBRARY_LOADS,
    SELFTEST_REGISTRY_TABLE, STAGE_SELFTEST_SCRIPT, SWAP_WITH_VERIFY_REMEDY, TEMPORARY_SERVICE_TABLE, TEMPORARY_SERVICE_WRAPPER, UNJUDGED, decide,
    decide_ledger_append, decide_script_swap_only, decide_shell_syntax, load_notations_library, notations_library_path, note_unjudged,
    record_detection, selftest_base_directory, unparsable_input_entry, work_roots, work_roots_error,
)
import write_guard_decisions

def selftest(hook_dir):
    os.environ[SELFTEST_MARK_VARIABLE] = SELFTEST_MARK_CONTENT   # 自证起的钩子子进程（环境原样继承）靠它认：弄坏开关与关闭开关只在自证里认
    if work_roots is None:
        print(f"  ✗ 自检读不到工作根的定义 {os.path.join(hook_dir, 'lib_work_root.py')}：{work_roots_error!r}")
        print("    → 怎么办：恢复那份库（工作根与会话临时目录根只在那里定义），SINGLEFS_WORK_ROOT_BREAK 只写它认的值，再跑 --selftest")
        return 1
    # 工作根下的样本路径不看弄坏开关（configured_work_root）；会话临时目录根照库里的写法
    new_root, session_temporary_root = work_roots.configured_work_root(), work_roots.claude_session_temporary_root()
    base = selftest_base_directory(os.path.join(hook_dir, "agent-write-scope.tsv"))
    if base is None:
        print("  ✗ 自检找不到一处能放现场的临时目录：TMPDIR、/tmp、/var/tmp、/dev/shm 要么落进写范围表的仓外模式、要么在 git 仓里、要么写不进去")
        print("    → 怎么办：给一个不在 git 仓里、也不在 agent-write-scope.tsv 任何一条绝对路径模式底下的可写目录当 TMPDIR 再跑")
        return 1
    work = tempfile.mkdtemp(dir=base)
    outside_directory = work + "-outside"   # 符号链接那几格仓外的一头：work 旁边、同样不在 git 仓与写范围的绝对模式里
    cases = []
    library = notations_library_path(hook_dir)
    try:
        subprocess.run(["git", "init", "-q", work], check=True)
        subprocess.run(["git", "-C", work, "config", "user.email", "t@t"], check=True)
        subprocess.run(["git", "-C", work, "config", "user.name", "t"], check=True)
        os.makedirs(os.path.join(work, ".claude", "agents"))
        os.makedirs(os.path.join(work, ".claude", "kb"))
        for name in ("kb-scribe", "implementation-writer", "experiment-runner", "unscoped-writer"):
            open(os.path.join(work, ".claude", "agents", f"{name}.md"), "w").write("---\n")
        open(os.path.join(work, "tracked.md"), "w").write("a\n")
        subprocess.run(["git", "-C", work, "add", "tracked.md"], check=True)
        subprocess.run(["git", "-C", work, "commit", "-qm", "t"], check=True)
        open(os.path.join(work, "untracked.md"), "w").write("b\n")
        open(os.path.join(work, ".claude", "kb", "untracked-kb.md"), "w").write("c\n")
        outside = tempfile.NamedTemporaryFile(delete=False, dir=base)
        outside.close()
        table = os.path.join(work, "scope.tsv")
        shutil.copyfile(os.path.join(hook_dir, "agent-write-scope.tsv"), table)
        if os.environ.get("WRITE_GUARD_BREAK") == "scope-new-rows-dropped":   # 弄坏开关：表副本退回没有这两行的样子
            with open(table, encoding="utf-8") as table_source:
                kept_rows = [row for row in table_source if not row.startswith(("kb-scribe\tcrates/singlefs-checker/invariants/", "experiment-runner\t.claude/gate.d/replay-comparison-units.tsv\t"))]
            # 跑前登记那一行退回只认原登记的写法（放宽到带轮号之前的样子）
            kept_rows = [row.replace("experiment-runner\tresearch/prompts/e*-preregistration*.md\t", "experiment-runner\tresearch/prompts/e*-preregistration.md\t", 1)
                         for row in kept_rows]
            with open(table, "w", encoding="utf-8") as table_target:
                table_target.writelines(kept_rows)
        if os.environ.get("WRITE_GUARD_BREAK") == "scope-runner-language-rows-dropped":   # 弄坏开关：表副本去掉执行员的英语、日语实验索引与历史条目四行
            with open(table, encoding="utf-8") as table_source:
                kept_rows = [row for row in table_source if not row.startswith(tuple(f"experiment-runner\t.claude/kb/{name}\t" for name in (
                    "experiments.en.md", "experiments.ja.md", "experiments-history.en.md", "experiments-history.ja.md")))]
            with open(table, "w", encoding="utf-8") as table_target:
                table_target.writelines(kept_rows)
        def case(label, tool_name, agent, path, want):
            hook_input = {"tool_name": tool_name, "tool_input": {"file_path": path}}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library)[0])
        def finding_case(label, tool_name, agent, path, want):
            hook_input = {"tool_name": tool_name, "tool_input": {"file_path": path}}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library)[2])
        os.makedirs(os.path.join(work, "scripts"))
        open(os.path.join(work, "scripts", "y.sh"), "w").write("#!/usr/bin/env bash\necho ok\n")
        # 落点：符号链接（弄坏开关 WRITE_GUARD_BREAK=landing-lexical-only 下解不开，「链接指到范围外」那几格红）；仓外的那一头放在 work 旁边的 outside_directory
        os.makedirs(outside_directory)
        os.makedirs(os.path.join(work, "crates", "singlefs-core"))
        os.makedirs(os.path.join(work, "crates", "singlefs-verification", "src", "bin"))
        open(os.path.join(work, "crates", "singlefs-verification", "src", "bin", "e161_x.rs"), "w").write("fn main() {}\n")
        os.symlink(outside_directory, os.path.join(work, "crates", "escape"))                              # 范围内的链接指到仓外
        os.symlink(os.path.join(outside_directory, "not-yet"), os.path.join(work, "crates", "dangling"))   # 范围内的悬空链接指到仓外
        os.symlink(os.path.join(work, ".claude", "kb"), os.path.join(work, "crates", "kb-link"))           # 范围内的链接指到仓里范围外
        os.symlink(os.path.join(work, "crates", "singlefs-core"), os.path.join(work, "crates", "inner"))   # 范围内的链接指到范围内
        os.symlink(os.path.join(work, "crates"), os.path.join(outside_directory, "back"))                  # 仓外的链接指回范围内
        os.symlink(os.path.join(work, "untracked.md"), os.path.join(outside_directory, "untracked-link"))   # 仓外的链接指到仓里未跟踪的文件
        os.symlink(os.path.join(work, "crates", "singlefs-verification", "src", "bin", "e161_x.rs"), os.path.join(outside_directory, "e161-link.rs"))
        def content_case(label, tool_name, path, tool_input, want):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=path)}
            return (label, want, decide(hook_input, work, table, library)[0])
        def prime_case(label, tool_name, agent, tool_input, want, library_path=library):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=f"{work}/crates/a.rs")}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library_path)[0])
        cases += [
            # 六、语法：钩子、门禁、项目脚本、研究脚本目录下的 .sh / .py
            content_case("语法:hooks 下写进语法错误的 .sh", "Write", f"{work}/.claude/hooks/x.sh", {"content": "if [ ; then\n  echo\n"}, 2),
            content_case("语法:hooks 下语法对的 .sh", "Write", f"{work}/.claude/hooks/x.sh", {"content": "#!/usr/bin/env bash\necho ok\n"}, 0),
            content_case("语法:Edit 把研究脚本改出语法错误", "Edit", f"{work}/scripts/y.sh", {"old_string": "echo ok", "new_string": "if [ ; then"}, 2),
            content_case("语法:Edit 研究脚本合法改动", "Edit", f"{work}/scripts/y.sh", {"old_string": "echo ok", "new_string": "echo fine"}, 0),
            content_case("语法:门禁目录下语法错误的 .py", "Write", f"{work}/.claude/gate.d/z.py", {"content": "def (:\n"}, 2),
            content_case("语法:门禁目录下语法对的 .py", "Write", f"{work}/.claude/gate.d/z.py", {"content": "print(1)\n"}, 0),
            content_case("语法:别处的 .sh 不判", "Write", f"{work}/crates/run.sh", {"content": "if [ ; then\n"}, 0),
            # 一、整份覆盖未跟踪文件
            case("覆盖:未跟踪的已有文件用 Write", "Write", None, f"{work}/untracked.md", 2),
            case("覆盖:未跟踪的已有文件用 Edit", "Edit", None, f"{work}/untracked.md", 0),
            case("覆盖:已跟踪文件", "Write", None, f"{work}/tracked.md", 0),
            case("覆盖:不存在的文件", "Write", None, f"{work}/missing.md", 0),
            case("覆盖:仓外文件", "Write", None, outside.name, 0),
            # 二、写范围
            case("范围:主 agent", "Edit", None, f"{work}/crates/a.rs", 0),
            # 没有项目定义的 agent 只许写工作根（弄坏开关 builtin-agents-unscoped 下「拒」的三格红）
            case("范围:内置 agent 写仓里拒", "Edit", "general-purpose", f"{work}/crates/a.rs", 2),
            case("范围:内置 agent 写仓外别处拒", "Write", "general-purpose", outside.name, 2),
            case("范围:定义已删的旧实例写仓里拒", "Write", "gate-triage", f"{work}/records/x.md", 2),
            case("范围:内置 agent 写工作根放行", "Write", "general-purpose", f"{new_root}/builtin-x/result.md", 0),
            case("范围:kb-scribe 范围内", "Edit", "kb-scribe", f"{work}/.claude/kb/decisions/01-x.md", 0),
            case("范围:kb-scribe 越界 crates", "Edit", "kb-scribe", f"{work}/crates/a.rs", 2),
            case("范围:kb-scribe 越界 records", "Edit", "kb-scribe", f"{work}/records/x.md", 2),
            case("范围:kb-scribe 写不变量清单的 .md", "Edit", "kb-scribe", f"{work}/crates/singlefs-checker/invariants/cp-01.md", 0),
            case("范围:kb-scribe 不变量清单目录里的 .rs 越界", "Edit", "kb-scribe", f"{work}/crates/singlefs-checker/invariants/x.rs", 2),
            case("范围:kb-scribe 写不变量清单子目录里的 .md", "Edit", "kb-scribe", f"{work}/crates/singlefs-checker/invariants/sub/x.md", 0),
            case("范围:kb-scribe 不变量清单子目录里的 .rs 越界", "Edit", "kb-scribe", f"{work}/crates/singlefs-checker/invariants/sub/x.rs", 2),
            case("范围:kb-scribe checker 源码越界", "Edit", "kb-scribe", f"{work}/crates/singlefs-checker/src/lib.rs", 2),
            case("范围:implementation-writer 范围内", "Edit", "implementation-writer", f"{work}/crates/singlefs-core/src/a.rs", 0),
            case("范围:.. 绕路", "Edit", "implementation-writer", f"{work}/crates/../.claude/kb/x.md", 2),
            case("范围:implementation-writer 越界 research", "Edit", "implementation-writer", f"{work}/research/x.md", 2),
            case("范围:工作根下的报告目录", "Write", "implementation-writer", f"{new_root}/agents/implementation-writer/result.md", 0),
            case("范围:会话临时目录根下写拒（scratchpad 之外）", "Write", "implementation-writer", f"{session_temporary_root}/agents/implementation-writer/result.md", 2),
            case("范围:会话临时目录根下的 scratchpad 里写拒", "Write", "implementation-writer",
                 f"{session_temporary_root}/-home-selftest-project/selftest-session/scratchpad/result.md", 2),
            case("范围:工作根同名前缀的兄弟目录", "Write", "implementation-writer", f"{new_root}-other/result.md", 2),
            case("范围:仓外别处", "Edit", "implementation-writer", "/etc/passwd", 2),
            case("范围:范围内的链接指到范围外，经它写拒", "Edit", "implementation-writer", f"{work}/crates/escape/hosts", 2),
            case("范围:范围内的链接指到范围外，经它写不存在的子目录拒", "Write", "implementation-writer", f"{work}/crates/escape/no-such/x.rs", 2),
            case("范围:范围内的悬空链接指到范围外拒", "Write", "implementation-writer", f"{work}/crates/dangling", 2),
            case("范围:范围内的链接指到仓里范围外拒", "Write", "implementation-writer", f"{work}/crates/kb-link/x.md", 2),
            case("范围:范围内的链接指到范围内放行", "Edit", "implementation-writer", f"{work}/crates/inner/a.rs", 0),
            case("范围:范围外的链接指回范围内放行", "Edit", "implementation-writer", f"{outside_directory}/back/a.rs", 0),
            case("覆盖:仓外的链接指到仓里未跟踪的文件，Write 拒", "Write", None, f"{outside_directory}/untracked-link", 2),
            case("范围:experiment-runner 登记", "Edit", "experiment-runner", f"{work}/research/prompts/e12-preregistration.md", 0),
            # 带轮号的跑前登记（e<号>-preregistration-r<N>.md）同样放行；同一目录里别的文件照拒（WRITE_GUARD_BREAK=scope-new-rows-dropped 下放行那一格红）
            case("范围:experiment-runner 带轮号的跑前登记", "Edit", "experiment-runner", f"{work}/research/prompts/e12-preregistration-r4.md", 0),
            case("范围:experiment-runner research/prompts 下不是跑前登记的文件", "Edit", "experiment-runner", f"{work}/research/prompts/e12-verdict.md", 2),
            case("范围:experiment-runner 三方正文", "Edit", "experiment-runner", f"{work}/research/prompts/_e12-body.md", 2),
            case("范围:experiment-runner replay.sh", "Edit", "experiment-runner", f"{work}/scripts/experiment-apparatus/replay.sh", 0),
            case("范围:experiment-runner mutate.sh", "Edit", "experiment-runner", f"{work}/scripts/mutation/mutate.sh", 2),
            case("范围:experiment-runner 写重跑比对对象登记", "Edit", "experiment-runner", f"{work}/.claude/gate.d/replay-comparison-units.tsv", 0),
            case("范围:experiment-runner 写别的门禁文件越界", "Edit", "experiment-runner", f"{work}/.claude/gate.d/doc-experiments.sh", 2),
            # 实验索引与历史条目的英语、日语版（WRITE_GUARD_BREAK=scope-runner-language-rows-dropped 下前四格红）；别的语言与译名对照表不在范围里
            case("范围:experiment-runner 写英语实验索引", "Edit", "experiment-runner", f"{work}/.claude/kb/experiments.en.md", 0),
            case("范围:experiment-runner 写日语实验索引", "Edit", "experiment-runner", f"{work}/.claude/kb/experiments.ja.md", 0),
            case("范围:experiment-runner 写英语历史条目", "Edit", "experiment-runner", f"{work}/.claude/kb/experiments-history.en.md", 0),
            case("范围:experiment-runner 写日语历史条目", "Edit", "experiment-runner", f"{work}/.claude/kb/experiments-history.ja.md", 0),
            case("范围:experiment-runner 写别的语言的实验索引越界", "Edit", "experiment-runner", f"{work}/.claude/kb/experiments.fr.md", 2),
            case("范围:experiment-runner 写译名对照表越界", "Edit", "experiment-runner", f"{work}/.claude/kb/translation-glossary.en.md", 2),
            case("范围:未登记的项目 agent", "Edit", "unscoped-writer", f"{work}/anything.md", 2),
            case("两道都中时先报覆盖", "Write", "kb-scribe", f"{work}/untracked.md", 2),
            # 四、实验执行员对主工作区 crates/ 下 .rs 的写：拒的那一道要是「绕过先编后换」，不是写范围那一道（出路不同）
            finding_case("先编后换:experiment-runner Edit 主工作区的入库装置", "Edit", "experiment-runner",
                         f"{work}/crates/singlefs-verification/src/bin/e161_x.rs", "绕过先编后换"),
            finding_case("先编后换:experiment-runner Write 新建入库装置", "Write", "experiment-runner",
                         f"{work}/crates/singlefs-verification/src/bin/e999_new.rs", "绕过先编后换"),
            finding_case("先编后换:experiment-runner 改 crates 里 bin 以外的 .rs", "Edit", "experiment-runner",
                         f"{work}/crates/singlefs-core/src/lib.rs", "绕过先编后换"),
            finding_case("先编后换:.. 绕路进 crates 也算", "Edit", "experiment-runner",
                         f"{work}/research/../crates/singlefs-verification/src/bin/e161_x.rs", "绕过先编后换"),
            finding_case("先编后换:仓外的链接指到主工作区的入库装置拒", "Edit", "experiment-runner", f"{outside_directory}/e161-link.rs", "绕过先编后换"),
            finding_case("先编后换:implementation-writer 同一处放行", "Edit", "implementation-writer",
                         f"{work}/crates/singlefs-verification/src/bin/e161_x.rs", None),
            finding_case("先编后换:主 agent 同一处放行", "Edit", None, f"{work}/crates/singlefs-verification/src/bin/e161_x.rs", None),
            finding_case("先编后换:experiment-runner 改草稿目录里的副本放行", "Edit", "experiment-runner",
                         f"{new_root}/runner-x/e161_x.rs", None),
            finding_case("先编后换:experiment-runner 追加 crates/mutations.tsv 放行", "Edit", "experiment-runner",
                         f"{work}/crates/mutations.tsv", None),
            # 三、撇号类角标：五个字符各写死一例（共用字符集少了哪一个，这里就红）
            prime_case("角标:Write 内容含 B2 加一撇（U+2032）", "Write", None, {"content": "取 B2\u2032 的读法\n"}, 2),
            prime_case("角标:Edit new_string 含 P1 加两撇（U+2033）", "Edit", None, {"old_string": "P1", "new_string": "P1\u2033"}, 2),
            prime_case("角标:Write 内容含三撇（U+2034）", "Write", None, {"content": "臂 A\u2034\n"}, 2),
            prime_case("角标:Edit new_string 含 U+02B9", "Edit", None, {"old_string": "丙", "new_string": "丙\u02b9"}, 2),
            prime_case("角标:Write 内容含 U+02BA", "Write", None, {"content": "候选 K9\u02ba\n"}, 2),
            prime_case("角标:MultiEdit 第二处 new_string 含", "MultiEdit", None,
                       {"edits": [{"old_string": "a", "new_string": "b"}, {"old_string": "c", "new_string": "K9\u2032"}]}, 2),
            prime_case("角标:子 agent 在写范围内也判", "Edit", "implementation-writer", {"old_string": "x", "new_string": "G5\u2032"}, 2),
            prime_case("角标:子 agent 同一处不带角标放行（上一例的 2 出自角标这一道）", "Edit", "implementation-writer", {"old_string": "x", "new_string": "G6"}, 0),
            prime_case("角标:ASCII 单引号放行", "Write", None, {"content": "S1' 与 it's\n"}, 0),
            prime_case("角标:只在 old_string 里有（删掉它的改动）放行", "Edit", None, {"old_string": "B2\u2032", "new_string": "B3"}, 0),
            prime_case("角标:MultiEdit 只在 old_string 里有放行", "MultiEdit", None,
                       {"edits": [{"old_string": "B2\u2032", "new_string": "B3"}]}, 0),
            prime_case("角标:读不到共用字符集就拒绝", "Write", None, {"content": "普通内容\n"}, 2,
                       library_path=os.path.join(work, "no-such-lib-forbidden-notations.py")),
        ]
        # 六、语法（续）：.py 在本进程里 ast 解析、不起子进程，.sh 起一个 bash -n；WRITE_GUARD_BREAK=python-syntax-subprocess 下「.py 在本进程里判」一格红
        real_run, started = subprocess.run, []
        def counting_run(command, *run_arguments, **run_keywords):
            started.append(command)
            return real_run(command, *run_arguments, **run_keywords)
        def checker_starts_of(path, content):
            del started[:]
            decided = decide({"tool_name": "Write", "tool_input": {"file_path": path, "content": content}}, work, table, library)
            return [command for command in started if command[0] in ("bash", sys.executable)], decided[1] or ""
        subprocess.run = counting_run
        try:
            python_starts, python_message = checker_starts_of(f"{work}/.claude/gate.d/z.py", "def (:\n")
            shell_starts, _ = checker_starts_of(f"{work}/.claude/hooks/x.sh", "if [ ; then\n  echo\n")
        finally:
            subprocess.run = real_run
        cases += [
            ("语法:写坏的 .py 在本进程里判，不起 python 子进程", 0, len(python_starts)),
            ("语法:写坏的 .py 的拒绝说明点名文件与 SyntaxError", True, ".claude/gate.d/z.py" in python_message and "SyntaxError" in python_message and "python3 的 ast 解析" in python_message),
            ("语法:写坏的 .sh 仍起一个 bash -n", ["bash"], [command[0] for command in shell_starts]),
        ]
        # 共用判据库一次写入只装一次：撇号、钟点、会话号三道都读它；WRITE_GUARD_BREAK=library-reloaded-each-time 下这一格红（装三回）
        NOTATIONS_LIBRARIES.clear()
        NOTATIONS_LIBRARY_LOADS[0] = 0
        decide({"tool_name": "Write", "tool_input": {"file_path": f"{work}/records/library-once.md", "content": "普通内容\n"}}, work, table, library, library)
        cases.append(("库:一次写入判撇号、钟点、会话号三道，共用判据库只装一次", 1, NOTATIONS_LIBRARY_LOADS[0]))
        # 八、old_string 以空白收尾：判的是拒绝那一道的名字；WRITE_GUARD_BREAK=trailing-blank-ignored 时拒绝的几例就红。
        # 不是 .md / .mdx 的目标，钩子收到的 new_string 已去过行尾空白，例子里照那样写
        trailing_finding = "old_string 以空白收尾"
        def trailing_case(label, tool_name, tool_input, want, agent=None, target=None):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=target or f"{work}/records/x.sh")}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library)[2])
        cases += [
            trailing_case("行尾空白:.sh 目标 old_string 以空格收尾", "Edit", {"old_string": "x = 1\ndeclared = ", "new_string": "x = 2\ndeclared ="}, trailing_finding),
            trailing_case("行尾空白:.sh 目标 old_string 以制表符收尾", "Edit", {"old_string": "key\t", "new_string": "key"}, trailing_finding),
            trailing_case("行尾空白:MultiEdit 第二处 old_string 以空格收尾", "MultiEdit",
                          {"edits": [{"old_string": "a", "new_string": "b"}, {"old_string": "python3 - ", "new_string": "python3 -"}]}, trailing_finding),
            trailing_case("行尾空白:.txt 目标也判", "Edit", {"old_string": "declared = ", "new_string": "declared ="}, trailing_finding, target=f"{work}/records/x.txt"),
            trailing_case("行尾空白:.markdown 目标也判（Edit 只在 .md / .mdx 上留行尾空白）", "Edit", {"old_string": "declared = ", "new_string": "declared ="},
                          trailing_finding, target=f"{work}/records/x.markdown"),
            trailing_case("行尾空白:子 agent 写仓外文件也判", "Edit", {"old_string": "a ", "new_string": "b"}, trailing_finding,
                          agent="implementation-writer", target=f"{new_root}/agents/implementation-writer/notes.txt"),
            trailing_case("行尾空白:.md 目标 old_string 以空格收尾放行", "Edit", {"old_string": "x = 1\ndeclared = ", "new_string": "x = 2\ndeclared = "}, None,
                          target=f"{work}/records/x.md"),
            trailing_case("行尾空白:.mdx 目标放行", "Edit", {"old_string": "a ", "new_string": "b "}, None, target=f"{work}/records/x.mdx"),
            trailing_case("行尾空白:大写 .MD 目标放行", "Edit", {"old_string": "a\t", "new_string": "b\t"}, None, target=f"{work}/records/X.MD"),
            trailing_case("行尾空白:.sh 目标 old_string 不以空白收尾放行", "Edit", {"old_string": "x = 1\ndeclared = foo", "new_string": "x = 2\ndeclared = foo"}, None),
            trailing_case("行尾空白:.sh 目标只在中间行有行尾空白放行", "Edit", {"old_string": "x = 1  \ny", "new_string": "x = 3\ny"}, None),
            trailing_case("行尾空白:old_string 以换行收尾、换行前有空白放行", "Edit", {"old_string": "a \n", "new_string": "b\n"}, None),
            trailing_case("行尾空白:只有 new_string 以空白收尾放行（旧判据已撤）", "Edit", {"old_string": "a", "new_string": "b "}, None),
            trailing_case("行尾空白:Write 不判", "Write", {"content": "x = 1 "}, None, target=f"{work}/records/new.sh"),
        ]
        # 五、钟点：样本串用 colon 拼、时区词用 jst / utc 两个变量拼，这份源码里不出现字面钟点与时区词（门禁 doc-text 也扫它）
        colon = ":"
        jst, utc = "J" + "ST", "U" + "TC"
        clock_library = notations_library_path(hook_dir)
        def clock_case(label, target, tool_input, want, agent=None, library_path=clock_library):
            hook_input = {"tool_name": "Edit" if "new_string" in tool_input else "Write",
                          "tool_input": dict(tool_input, file_path=target if os.path.isabs(target) else f"{work}/{target}")}
            if agent:
                hook_input["agent_type"] = agent
            return (label, want, decide(hook_input, work, table, library, library_path)[2])
        cases += [
            clock_case("钟点:日期 时区词 时分x", "records/x.md", {"content": f"用户 2026-09-26 {jst} 15{colon}3x 定\n"}, "描述里写了钟点"),
            clock_case("钟点:日期 时分 时区词 前后", "records/x.md", {"content": f"实十九（2026-09-24 16{colon}50 {utc} 前后落地）\n"}, "描述里写了钟点"),
            clock_case("钟点:时区词 时分，不带日期", ".claude/kb/x.md", {"old_string": "a", "new_string": f"同日 {utc} 14{colon}00 前后：\n"}, "描述里写了钟点"),
            clock_case("钟点:时分 时区词，不带日期", "crates/a.rs", {"content": f"// 跑于 03{colon}44 {jst}\n"}, "描述里写了钟点"),
            clock_case("钟点:带 x 通配的裸时分", "records/x.md", {"content": f"主 agent 22{colon}5x 跑前修订\n"}, "描述里写了钟点"),
            clock_case("钟点:子 agent 也判", "research/prompts/e12-preregistration.md", {"content": f"交回 {jst} 22{colon}0x\n"}, "描述里写了钟点", agent="experiment-runner"),
            clock_case("钟点:只写日期放行", "records/x.md", {"content": "用户 2026-09-26 定\n"}, None),
            clock_case("钟点:仓外草稿目录放行", f"{new_root}/x/progress.md", {"content": f"2026-09-27 22{colon}41 {jst} 编过\n"}, None),
            clock_case("钟点:排除的门禁样本放行", ".claude/gate.d/fixtures/x/red/expect", {"content": f"改于 2026-09-05 00{colon}00\n"}, None),
            clock_case("钟点:排除的检查项自检放行", ".claude/session-checks/kernel-memory-kills", {"content": f"2026-09-25 07{colon}58{colon}02 {utc}\n"}, None),
            clock_case("钟点:排除的 c510 提示放行", "research/prompts/_c510-date-gate-r1-body.md", {"content": f"本机时钟是 {utc}\n"}, None),
            clock_case("钟点:只在 old_string 里有放行", "records/x.md", {"old_string": f"{jst} 15{colon}3x", "new_string": ""}, None),
            clock_case("钟点:读不到判据就拒绝", "records/x.md", {"content": "普通内容\n"}, "描述里写了钟点",
                       library_path=os.path.join(work, "no-such-lib-forbidden-notations.py")),
        ]
        # 五、钟点里的各个判据行：样本串用 glued 把数字与「点」、日期与时段词拆开拼，这份源码里不出现字面写法；
        # 判的是拒绝信息点名的判据行名字，LIB_FORBIDDEN_NOTATIONS_BREAK 关掉哪一条判据行，点名它的那几例就红；
        # 放行的几组分别靠上下文规则、放行标记，LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT / _ALLOW 点名钟点那一形态时它们就红
        def glued(*parts):
            return "".join(parts)
        def clock_shape_case(label, content, want_shape, target="records/x.md"):
            code, message, finding = decide({"tool_name": "Write", "tool_input": {"file_path": f"{work}/{target}", "content": content + "\n"}},
                                            work, table, library, clock_library)
            shape = re.search(r"描述里写了钟点，共 \d+ 处：\n  [^（\n]*（(.+?)）：「", message) if finding == "描述里写了钟点" else None
            return (label, want_shape, shape.group(1) if shape else finding)
        hour_shape, period_shape, period_hour_shape, suffix_shape = "日期或时区后面跟「N 点 / N 时」", "日期或时区后面跟时段词", "时段词后面跟钟点", "「N 点前后」这类钟点"
        date_shape, bare_shape, iso_shape, zone_shape = "日期后面跟钟点", "不挨日期时区的钟点", "ISO 时间戳", "时区词"
        twelve_hour_shape = "十二小时制钟点"
        cases += [
            clock_shape_case("钟点:英文月日后面接十二小时制", glued("交付写 Oct 4 4", f"pm {utc}"), twelve_hour_shape),
            clock_shape_case("钟点:十二小时制隔空格大写", glued("11 ", "AM 截止"), twelve_hour_shape),
            clock_shape_case("钟点:十二小时制带点", glued("交回于 3", "p.m."), twelve_hour_shape),
            clock_shape_case("钟点:以 am / pm 起头的英文词放行", glued("2 AMD 显卡、3 ", "amps、5 ppm、pm2 进程、v1.2", "am"), None),
        ]
        # 七、会话号：样本串用 glued 把目录名与十六进制拆开拼，这份源码里不出现会话号；
        # WRITE_GUARD_DISABLE_SESSION_NAMES=1 或 LIB_FORBIDDEN_NOTATIONS_BREAK=session-names 时拒绝的几例就红，
        # LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=session-names 时带放行标记的那一例就红
        session_shape = "写了会话号"
        cases += [
            clock_shape_case("会话号:标签式（会话 号，日期）", glued("### 配置判法开工（会话 singlefs-", "21，2026-10-01）"), session_shape),
            clock_shape_case("会话号:当主语用的", glued("已发消息，singlefs-", "e1 回话"), session_shape),
            clock_shape_case("会话号:上游三语仓的", glued("SOP 会话 singlefs-ai-sop-zh-", "86 同步来"), session_shape),
            clock_shape_case("会话号:四位十六进制", glued("会话 singlefs-", "e152 在跑"), session_shape),
            clock_shape_case("会话号:crate 名、上游仓目录名、临时目录名放行",
                             glued("改 singlefs-core 与 singlefs-ai-sop-zh；mktemp -d /tmp/singlefs-", "e152.XXXXXX；/tmp/singlefs-", "e152-packages"), None),
            clock_shape_case("会话号放行标记:sh 里的自检输入放行",
                             glued("echo singlefs-", "6a  # session-names:allow 解析 ListAgents 输出的自检输入"), None, target=".claude/scripts/parse.sh"),
            clock_shape_case("钟点:日期 时区词 N 点前后", glued(f"用户 2026-09-28 {jst} 08 ", "点前后要开"), hour_shape),
            clock_shape_case("钟点:时区词 日期 N 点后（时区词那一处先中）", glued(f"本机 {utc} 2026-09-26 23 ", "点后"), zone_shape),
            clock_shape_case("钟点:日期 时区词 N 时许", glued(f"2026-09-16 {utc} 17 ", "时许的快照"), hour_shape),
            clock_shape_case("钟点:不带年份的日期 时区词 N 点前后", glued(f"09-24 {jst} 19 ", "点前后问了四件"), hour_shape),
            clock_shape_case("钟点:日期 时区词 N 点后", glued(f"2026-09-25 {jst} 01 ", "点后弹窗"), hour_shape),
            clock_shape_case("钟点:日期 时区词 时段词", glued(f"交回于 2026-09-27 {jst} ", "上午"), period_shape),
            clock_shape_case("钟点:括号里日期后跟时段词", glued("降为待证（2026-09-13 ", "上午）"), period_shape),
            clock_shape_case("钟点:日期时段词再括号时区词", glued("2026-09-12 ", f"凌晨（{jst}）"), period_shape),
            clock_shape_case("钟点:括号里时区词日期时段词（时区词那一处先中）", glued(f"（{jst} 2026-09-27 ", "早上）"), zone_shape),
            clock_shape_case("钟点:日期时段词前后", glued("满载到 2026-09-28 ", "中午前后"), period_shape),
            clock_shape_case("钟点:标题里日期后括号时段词", glued("### 2026-09-27（", "下午）：E162"), period_shape),
            clock_shape_case("钟点:日期时段词再时分（日期那一处先中）", glued("2026-09-27 ", f"上午 08{colon}32 现跑"), period_shape),
            clock_shape_case("钟点:时段词后跟时分", f"上午 08{colon}32 现跑", period_hour_shape),
            clock_shape_case("钟点:时段词后跟 N 点", glued("晚上 ", "8 点"), period_hour_shape),
            clock_shape_case("钟点:光秃的 N 点前后", glued("19 ", "点前后问了"), suffix_shape),
            clock_shape_case("钟点:裸时分", f"用户 23{colon}10 定", bare_shape),
            clock_shape_case("钟点:裸时分秒", f"交回 14{colon}00{colon}10、续做", bare_shape),
            clock_shape_case("钟点:裸时分秒带毫秒", f"07{colon}58{colon}02.123 起跑", bare_shape),
            clock_shape_case("钟点:时分区间", f"14{colon}00–15{colon}30 之间跑完", bare_shape),
            clock_shape_case("钟点:月-日 时分", f"核查员缺快照（09-24 23{colon}44 那一次）", bare_shape),
            clock_shape_case("钟点:反引号里两位小时的时分", f"跑于 `09{colon}55` 的那一轮", bare_shape),
            clock_shape_case("钟点:日期后面跟带秒带偏移的 diff 头", f"--- a/x.rs\t2026-09-24 16{colon}27{colon}05.398761624 +0000", date_shape),
            clock_shape_case("钟点:stat 的修改时间", f"Modify: 2026-09-24 18{colon}48{colon}58.254848952 +0000", date_shape),
            clock_shape_case("钟点:ISO 时间戳", f"派腿 2026-09-27T09{colon}55{colon}07+00{colon}00", iso_shape),
            clock_shape_case("钟点:不带日期的 T 钟点", f"交回于 …T23{colon}15Z", iso_shape),
            clock_shape_case("钟点:光秃的时区词", f"本机时钟是 {utc}、直接 date 会差一天", zone_shape),
            clock_shape_case("钟点:日期后括号时区词", f"跑于 2026-09-27（{jst}），本机", zone_shape),
            clock_shape_case("钟点:两个时区的名字", f"{jst} 与 {utc} 两个时区", zone_shape),
            clock_shape_case("钟点:时区词 时间", f"{utc} 时间", zone_shape),
            clock_shape_case("钟点:放行标记写在 md 里不认", f"用户 23{colon}10 定  # clock-times:allow 写在 md 里不认", bare_shape),
            clock_shape_case("钟点:放行标记没写理由不认", f"// 跑于 23{colon}10  // clock-times:allow", bare_shape, target="crates/a.rs"),
            clock_shape_case("钟点:第 N 点前后是序号放行", "第 3 点前后两句", None),
            clock_shape_case("钟点:N 点意见放行", "3 点意见", None),
            clock_shape_case("钟点:个数后的点放行", "1000 个崩溃点", None),
            clock_shape_case("钟点:夜间放行", "全量由用户要求或夜间跑", None),
            clock_shape_case("钟点:只写日期放行（Write 整份）", "用户 2026-09-28 定", None),
            clock_shape_case("钟点:不挨日期时区的光秃时段词放行", "同一天下午续跑", None),
            clock_shape_case("钟点:标识符里的时区词放行", f"`{jst}_OFFSET`、`started_{utc}`", None),
        ]
        context_cases = [
            clock_shape_case("钟点上下文:产物行的名字=值放行", f"remount_chosen=2{colon}12 device_chosen=1{colon}16 values=1{colon}16.000,3{colon}12.000", None),
            clock_shape_case("钟点上下文:差分与计数的冒号对放行", f"adjacent_differences=0->1{colon}10,1->2{colon}8 steps_by_changed_leaf_count=1{colon}108,2{colon}36", None),
            clock_shape_case("钟点上下文:字典放行", f"分布 {{0{colon}14, 1{colon}14, 2{colon}16}}", None),
            clock_shape_case("钟点上下文:Python 切片放行", f"r[8{colon}12]=i.to_bytes(4); fields[11{colon}15]; arguments[5{colon}13]", None),
            clock_shape_case("钟点上下文:代码里的时区偏移放行", f'stamp.replace("Z", "+00{colon}00")', None),
            clock_shape_case("钟点上下文:双引号里的代码字面放行", f'            Some("2{colon}11"),', None),
            clock_shape_case("钟点上下文:单引号里的代码字面放行", f"                      printf '2{colon}11' ;;", None),
            clock_shape_case("钟点上下文:百分比放行", f"B Lost Writes **13{colon}54%**", None),
            clock_shape_case("钟点上下文:time -v 的格式说明放行", f"Elapsed (wall clock) time (h:mm:ss or m:ss): 0{colon}03.21", None),
            clock_shape_case("钟点上下文:门禁号行号表格放行", f"| 共用脚本不在 | 21{colon}38 | 31{colon}68 |", None),
            clock_shape_case("钟点上下文:门禁号行号列表放行", f"| kb 目录不在 | —— | 21{colon}39、24{colon}27、27{colon}39 |", None),
            clock_shape_case("钟点上下文:门禁文件名放行", f"门禁 20{colon}09-sample-stage.sh 那一行", None),
            clock_shape_case("钟点上下文:同一行带文件行号的编号对放行", f"触发的那一处（decisions/06-快照实现模型.md{colon}42），06{colon}42 正是没抽到的那一行", None),
            clock_shape_case("钟点上下文:反引号里一位小时的键放行", f"撞键 `3{colon}10`：冷重开是 `2{colon}11`", None),
            clock_shape_case("钟点上下文:产物字段里的 ISO 时间戳放行", f"taken_jst=2026-09-27T01{colon}20{colon}12+09{colon}00", None),
        ]
        allow_cases = [
            clock_shape_case("钟点放行标记:sh 里的 TZ 环境变量放行", f"TZ={utc}-14 date +%F  # clock-times:allow 环境变量写法，符号与东京时间相反", None, target=".claude/scripts/tz.sh"),
            clock_shape_case("钟点放行标记:py 里解析外部通知的正则放行", f'RESETS = re.compile(r"resets (\\d+)(am|pm) \\({utc}\\)")  # clock-times:allow 解析外部通知文本', None,  # account-quota:allow 自证构造钟点放行标记的正则样例
                             target=".claude/hooks/parse.py"),
            clock_shape_case("钟点放行标记:rs 里造时间戳的自检输入放行", f'let stamp = "2026-09-27T09{colon}55{colon}07Z"; // clock-times:allow 造时间戳格式的自检输入', None,
                             target="crates/a.rs"),
        ]
        cases += context_cases + allow_cases
        # 五、ISO 基本格式的紧凑时间戳（运行目录名、日志文件名里连写的那种）：样本串用 glued 把日期与「T 加钟点」拆开拼，这份源码里不出现字面写法；
        # 拒绝的几例判的是拒绝信息点名的形态；WRITE_GUARD_BREAK=compact-timestamp-skipped 或 LIB_FORBIDDEN_NOTATIONS_BREAK=iso-compact-timestamp 时它们红，
        # 放行的「8 位数字后跟 T 但不是时间戳」几例在 LIB_FORBIDDEN_NOTATIONS_BREAK=compact-timestamp-loose 时红
        compact_shape = "ISO 紧凑时间戳"
        compact_sample = glued("运行目录 20261014", "T153000Z")
        compact_cases = [
            clock_shape_case("紧凑时间戳:8 位日期、T、6 位钟点、Z", compact_sample, compact_shape),
            clock_shape_case("紧凑时间戳:8 位日期、T、4 位钟点", glued("日志 20261014", "T1530"), compact_shape),
            clock_shape_case("紧凑时间戳:8 位日期、T、带冒号的时分", glued("跑于 20261014", "T15", colon, "30"), compact_shape),
            clock_shape_case("紧凑时间戳:带连字符的日期加连写钟点", glued("run 2026-10-14", "T153000"), compact_shape),
            clock_shape_case("紧凑时间戳:带小数秒与偏移", glued("20261014", "T153000.5+0900"), compact_shape),
            clock_shape_case("紧凑时间戳:路径里的运行目录名也判", glued("见 logs/run-20261014", "T153000Z/out.log"), compact_shape),
        ]
        compact_lookalike_cases = [
            clock_shape_case("紧凑时间戳:月份越界放行", glued("编号 20261314", "T1530"), None),
            clock_shape_case("紧凑时间戳:钟点越界放行", glued("编号 20261014", "T2560"), None),
            clock_shape_case("紧凑时间戳:年不是 20 开头放行", glued("编号 12345678", "T0000"), None),
            clock_shape_case("紧凑时间戳:钟点只有三位放行", glued("编号 20261014", "T123"), None),
            clock_shape_case("紧凑时间戳:8 位数字全是 9 放行", glued("编号 99999999", "T123456"), None),
            clock_shape_case("紧凑时间戳:前面挨着字母放行", glued("哈希 ab20261014", "T1530"), None),
            clock_shape_case("紧凑时间戳:小写 t 放行", glued("哈希 20261014", "t1530"), None),
        ]
        compact_pass_cases = [
            clock_shape_case("紧凑时间戳:只写日期放行", "运行目录 20261014", None),
            clock_shape_case("紧凑时间戳:.py 里带理由的放行标记放行", glued('stamp = "20261014', 'T153000Z"  # clock-times:allow 造紧凑时间戳的自检输入'), None,
                             target=".claude/hooks/parse.py"),
            clock_case("紧凑时间戳:仓外草稿目录放行", f"{new_root}/x/progress.md", {"content": compact_sample + "\n"}, None),
        ]
        compact_hyphenated_lookalike_cases = [   # LIB_FORBIDDEN_NOTATIONS_BREAK=compact-hyphenated-date-loose 时这两格红（compact-timestamp-loose 下照放行）
            clock_shape_case("紧凑时间戳:带连字符的日期月份越界放行", glued("编号 2026-13-01", "T1234"), None),
            clock_shape_case("紧凑时间戳:带连字符的日期日越界放行", glued("编号 2026-10-32", "T1234"), None),
        ]
        cases += compact_cases + compact_lookalike_cases + compact_pass_cases + compact_hyphenated_lookalike_cases
        # 七里的另一判，批次代号：拒绝的几格判的是拒绝的那一道（「写了批次代号」）；WRITE_GUARD_BREAK=batch-codes-unjudged 或 LIB_FORBIDDEN_NOTATIONS_BREAK=batch-codes 时
        # 拒绝的几格红，库的判据行键一次关一条时点名那一条的格红（文件头）；放行的几格分别靠判据行的边界、上下文规则、排除前缀与只判 .md、.tsv、.txt
        batch_shape = "写了批次代号"
        batch_cases = [
            clock_shape_case("批次代号:数字加字母加短横序号", "24c-1 交回", batch_shape),
            clock_shape_case("批次代号:.tsv 也判", "22c-2\t已合", batch_shape, target="records/x.tsv"),
            clock_shape_case("批次代号:两位数字加字母当名字用", "22a 的接口改了", batch_shape),
            clock_shape_case("批次代号:批次词后面跟一位数字加字母", "截止批次 3c 交回", batch_shape),
            clock_shape_case("批次代号:单位字母的批号挨着批字", "第 10h 批交回", batch_shape),
            clock_shape_case("批次代号:第几批加天干", "第十五批乙二交回", batch_shape),
            clock_shape_case("批次代号:批加数字加天干", "批 10 甲交回", batch_shape),
            clock_shape_case("批次代号:批次词后面的两位代号", "批次 24c 的补丁", batch_shape),
            clock_shape_case("批次代号:配不上对的反引号不开代码段", "孤零零一个 ` 号后面的 22c 交回", batch_shape),
            clock_shape_case("批次代号:批次词后面的三位数字加字母放行", "截止批 123c 交回", None),
            clock_shape_case("批次代号:三位数字加字母挨着批字放行", "123c 批交回", None),
            clock_shape_case("批次代号:两位会话短号放行", "已告诉 87、7a、e2、ed", None),
            clock_shape_case("批次代号:单位、倍数与指数放行", "16k 块、64x 加速、30d 清空、60s 超时、1e-6 的期望", None),
            clock_shape_case("批次代号:路径、标识符与文件行号放行", "path-move-6b/result.md、m3-batch10z-c、a.md:12a、e165", None),
            clock_shape_case("批次代号:不带批次词的候选名放行", "出口 4 甲一、已定项 1 甲、C554 乙、第 1b 步", None),
            clock_shape_case("批次代号:.py 不判", "# 24c-1 交回", None, target=".claude/hooks/parse.py"),
            clock_shape_case("批次代号:.sh 不判", "# 22a 的接口", None, target=".claude/scripts/parse.sh"),
            clock_shape_case("批次代号:变异表放行", "22a\t名字", None, target="crates/mutations.tsv"),
            clock_case("批次代号:仓外草稿目录放行", f"{new_root}/x/progress.md", {"content": "24c-1 交回\n"}, None),
        ]
        batch_context_cases = [   # LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=batch-codes 时这几格红
            clock_shape_case("批次代号上下文:反引号里的放行", "格里写 `10q` 的样子", None),
            clock_shape_case("批次代号上下文:双反引号代码段里的放行", "格里写 ``10q`` 的样子", None),
            clock_shape_case("批次代号上下文:字段值放行", "活槽 = 64f 时", None),
            clock_shape_case("批次代号上下文:算式放行", "13 + 22n 个槽，(w + 31u) / 32u", None),
            clock_shape_case("批次代号上下文:引号与逗号后面的代码字面放行", 'key "22c" 与名单 a,22e', None),
        ]
        cases += batch_cases + batch_context_cases
        # 七里的第三判，任务号、会话短号与批号：拒绝的几格判的是拒绝的那一道（「写了任务号、会话短号或批号」），每格只中一条判据行，
        # 库的判据行键一次关一条时点名那一条的格红（文件头）；放行的几格分别靠判据行的边界、只判 .md、.tsv、.txt、排除前缀与写前写后整份相抵
        numbers_shape = "写了任务号、会话短号或批号"
        verdict_target = "research/prompts/x-main-verification.md"   # 判决：行首「A/B 读数：任务 <号>」一行只在这类文件里放过
        # Edit / MultiEdit 判的是写前、写后两份整文，现文件先放好：fence-open.md 的「尾巴」之前开着围栏，fence-closed.md 的已经收了，multi-edit.md 给 MultiEdit 依次施加；
        # fence-batch.md 的批号在围栏里（删、加、改坏一行围栏让它出来），multi-fence.md 给「先写进围栏」，count-two.md 一处在围栏里一处在外，
        # old-number.md 带一行旧号，old-only.md 的旧号要被删掉，table.md 是带「批次」表头的一张表，edit-target.md 给 Edit 写进新号
        os.makedirs(os.path.join(work, "records"), exist_ok=True)
        for fence_name, fence_text in (("fence-open.md", "前文\n```\n代码\n尾巴\n"), ("fence-closed.md", "前文\n```\n代码\n```\n尾巴\n"),
                                       ("multi-edit.md", "前文\n尾巴\n结尾\n"), ("fence-batch.md", "前文\n```\n那一路第 7 批交回\n```\n"),
                                       ("multi-fence.md", "前文\n尾巴\n"), ("count-two.md", "```\n占位甲\n```\n占位甲\n"),
                                       ("old-number.md", "旧行（任务 21036）\n尾巴\n"), ("old-only.md", "前文（任务 21036）\n"),
                                       ("table.md", "| 批次 | 内容 |\n|---|---|\n| 批 5 | 甲 |\n"), ("edit-target.md", "a\n")):
            with open(os.path.join(work, "records", fence_name), "w", encoding="utf-8") as handle:
                handle.write(fence_text)
        closing_fence_then_text = "```\n\n正文：这件事 (5b) 知道"
        fenced_batch = "```\n那一路第 7 批交回\n```"
        def numbers_multi_edit_case(label, target, edits, want):
            hook_input = {"tool_name": "MultiEdit", "tool_input": {"file_path": f"{work}/{target}", "edits": edits}}
            return (label, want, decide(hook_input, work, table, library, clock_library)[2])
        numbers_cases = [
            clock_shape_case("任务号与批号:括号里的任务号", "调度那一条跑完（任务 21036）", numbers_shape),
            clock_shape_case("任务号与批号:同字后面并列的裸号", "同 20461、20814，已认领", numbers_shape),
            clock_shape_case("任务号与批号:任务号冒号加号", "开关复跑任务号：11964 退 0", numbers_shape),
            clock_shape_case("任务号与批号:.tsv 也判", "复跑\t任务 21036\t退 0", numbers_shape, target="records/x.tsv"),
            clock_case("任务号与批号:Edit 写进的任务号", "records/edit-target.md", {"old_string": "a", "new_string": "跑完（任务 21036）"}, numbers_shape),
            clock_shape_case("任务号与批号:任务与号之间夹全角括号、冒号与井号", "人手任务（5073–5075、5080）、交的任务：14098、任务 #1748", numbers_shape),
            clock_shape_case("任务号与批号:斜杠与短横连着的并列号", "任务 5922/5924 与任务 21036-21040 都退 0", numbers_shape),
            clock_shape_case("任务号与批号:同字与与字后面并列的号", "同 3642/3644 的旧红；同 20461 与 20814；与 20461、20814 相同", numbers_shape),
            clock_shape_case("任务号与批号:单个同字后面的号", "同 21036 的旧红", numbers_shape),
            clock_shape_case("任务号与批号:任务号后接英文冒号", "任务 21036: 退 0", numbers_shape),
            clock_shape_case("任务号与批号:任务号后面跟条件与个别", "任务 21036 条件不满足；任务 21037 个别格退 1", numbers_shape),
            clock_shape_case("任务号与批号:日志文件名与运行目录名里的号",
                             "日志 ~/.local/state/singlefs/task-server/logs/task-0a1b2c3d-12308.attempt-2.log；编译目录 builds/task-0a1b2c3d-5295-3", numbers_shape),
            clock_shape_case("任务号与批号:英文 task 的几种写法", "Four tasks (12579, 12580)、task#21036、task=21036", numbers_shape),
            clock_shape_case("任务号与批号:结局行原文里的 task=", "对照同样退 78（TASK_SERVER_RESULT task=21036 type=command-failed exit=78 attempts=1）", numbers_shape),
            clock_shape_case("任务号与批号:反引号里 task 子命令加号", "对照（`task-server/task log 21036`）同样退 78", numbers_shape),
            clock_shape_case("任务号与批号:反引号里的任务号", "结局行 `任务 21036 退 0` 原样", numbers_shape),
            clock_shape_case("任务号与批号:围栏代码块里的任务号", "```\n任务 21036 退 0\n```", numbers_shape),
            clock_shape_case("任务号与批号:A/B 读数不在行首", "- 甲：零轮，A/B 读数：任务 12400", numbers_shape, target=verdict_target),
            clock_shape_case("任务号与批号:A/B 读数一行在记录里", "A/B 读数：任务 11964", numbers_shape),
            clock_shape_case("任务号与批号:A/B 读数一行里同字并列的号", "A/B 读数：任务 12400；同 20461、20814", numbers_shape, target=verdict_target),
            clock_shape_case("任务号与批号:A/B 读数一行装三个号", "A/B 读数：任务 12400；复跑是任务 12401、门禁是任务 12402", numbers_shape, target=verdict_target),
            clock_shape_case("任务号与批号:引用块里的 A/B 读数", "> A/B 读数：任务 12400", numbers_shape, target=verdict_target),
            clock_shape_case("任务号与批号:长跑拆分一行在 kb 里", "- 长跑拆分：无；任务 21036 在跑", numbers_shape, target=".claude/kb/x.md"),
            clock_case("任务号与批号:旧号那一行新加第二个号照判", "records/old-number.md", {"old_string": "旧行", "new_string": "旧行（任务 21037）"}, numbers_shape),
            clock_shape_case("任务号与批号:括号里数字加字母的会话短号", "这件事 (5b) 知道", numbers_shape),
            clock_shape_case("任务号与批号:挨着会话词的纯数字短号", "会话「(10) 压缩规范」接手", numbers_shape),
            clock_shape_case("任务号与批号:并列的会话短号", "给 (5b)、(10) 发合并请求", numbers_shape),
            clock_shape_case("任务号与批号:括号里短号后面跟会话词", "（b6 推的）、（5b 在修）、（5b、10 在修）", numbers_shape, target=".claude/kb/x.md"),
            clock_shape_case("任务号与批号:括号里短号后面跟范围与判过", "（5b 范围）、（10 判过）", numbers_shape, target=".claude/kb/x.md"),
            clock_shape_case("任务号与批号:判决里括号里的会话短号", "借卡那一批（5b）落主工作区", numbers_shape, target=verdict_target),
            clock_case("任务号与批号:Edit 以收尾围栏起头、插入点前围栏开着", "records/fence-open.md",
                       {"old_string": "尾巴", "new_string": closing_fence_then_text}, numbers_shape),
            numbers_multi_edit_case("任务号与批号:MultiEdit 前一处开了围栏、后一处以收尾围栏起头", "records/multi-edit.md",
                                    [{"old_string": "尾巴", "new_string": "```\n代码"}, {"old_string": "结尾", "new_string": closing_fence_then_text}], numbers_shape),
            clock_case("任务号与批号:Edit 删开头围栏行让批号出了围栏", "records/fence-batch.md", {"old_string": "前文\n```\n", "new_string": "前文\n"}, numbers_shape),
            clock_case("任务号与批号:Edit 在开头围栏前补一行围栏", "records/fence-batch.md", {"old_string": "前文\n", "new_string": "前文\n```\n"}, numbers_shape),
            clock_case("任务号与批号:Edit 把开头围栏的信息串改成带反引号", "records/fence-batch.md", {"old_string": "前文\n```\n", "new_string": "前文\n```a`b\n"}, numbers_shape),
            numbers_multi_edit_case("任务号与批号:MultiEdit 先写进围栏再删开头围栏行", "records/multi-fence.md",
                                    [{"old_string": "尾巴", "new_string": fenced_batch}, {"old_string": "前文\n```\n", "new_string": "前文\n"}], numbers_shape),
            clock_case("任务号与批号:Edit replace_all 一处在围栏里一处在外", "records/count-two.md",
                       {"old_string": "占位甲", "new_string": "那一路第 7 批交回", "replace_all": True}, numbers_shape),
            clock_shape_case("任务号与批号:正文里的批号", "批 6 交回", numbers_shape),
            clock_shape_case("任务号与批号:别的会话的批号", "它的批 105 在修", numbers_shape),
            clock_shape_case("任务号与批号:第几批", "第 6 批交回", numbers_shape),
            clock_shape_case("任务号与批号:批号冒号起头的一行", "- 批 118 量读数：退 0", numbers_shape),
            clock_shape_case("任务号与批号:批井号、区间与并列的第几批", "同批 #2 续做；第 7–13 批；第 105、106 批；批次号 118 量读数", numbers_shape),
            clock_shape_case("任务号与批号:批号后面跟副词只", "批 6 只在副本里改", numbers_shape),
            clock_shape_case("任务号与批号:批号后面跟处理、行为、项目", "它的批 105 处理的是写闸；批 106 行为不对；批 107 项目里改了三份", numbers_shape),
            clock_shape_case("任务号与批号:锚点里的第几批", "批次见 records/x.md#第 1 批", numbers_shape),
            clock_shape_case("任务号与批号:记录里中文数字的第几批", "写段第一批交回", numbers_shape),
            clock_shape_case("任务号与批号:记录里追加第几批", "依赖表追加第 7 批", numbers_shape),
            clock_shape_case("任务号与批号:kb 里同一分句后面的批号", "（seq 批 1 是 8 条、批 10 是 80 条）", numbers_shape, target=".claude/kb/x.md"),
            clock_shape_case("任务号与批号:表格行里的批号在 kb 里", "| 批次 | 内容 |\n|---|---|\n| 它的批 105 在修 | 交回 |", numbers_shape, target=".claude/kb/x.md"),
            clock_shape_case("任务号与批号:记录里表格别的列的批号", "| 批次 | 依赖 |\n|---|---|\n| 批 6 | 批 5 |", numbers_shape),
            clock_shape_case("任务号与批号:没有表头分隔行的竖线行", "| 别的会话的第 105 批还没交回", numbers_shape),
            # 近似写法：括号里换成短号的 mainagent( / subagent( / agent(、「会话」后面不带括号的短号、记录里「批」后面跟的中文数字、
            # 「任务编号 <号>」与「<号> 号任务」、表格里「\\|」（转义的反斜杠加分格的竖线）让批号落进别的列、列表里的引用块「- > A/B 读数：」
            clock_shape_case("任务号与批号:mainagent 与 subagent 括号里的短号", "mainagent(5b) 在修 `.claude/hooks/write-guard.sh`，subagent(c3) 交回", numbers_shape),
            clock_shape_case("任务号与批号:agent 括号里的短号", "agent(5b) 在修；agent(cc) 认领了；问 agent(4a) 回信", numbers_shape),
            clock_shape_case("任务号与批号:会话后面不带括号跟短号", "这一轮剩下的活交给里程碑三收尾那一轮（会话 4a，2026-11-19）；（会话 5b）在修", numbers_shape),
            clock_shape_case("任务号与批号:记录里批后面跟中文数字", "截止批次为三；它的批三在修；批次是十二", numbers_shape),
            clock_shape_case("任务号与批号:任务编号与号任务", "任务编号 12345 退 0；12345 号任务卡住", numbers_shape),
            clock_shape_case("任务号与批号:表格里转义的反斜杠加竖线", "| 批次 | 依赖 |\n|---|---|\n| x\\\\| 第 6 批 |", numbers_shape),
            clock_shape_case("任务号与批号:列表里的引用块里的 A/B 读数", "- > A/B 读数：任务 12345", numbers_shape, target=verdict_target),
            # 拿「<做什么>那一路」的描述指别的会话（判据行 session-descriptions）：调度记录里三句原句与「那一路」后面紧跟动作词的照拒，
            # 按出场次序的 mainagent(A)、subagent(A) 与不跟会话词、动作词的「一路」放行
            clock_shape_case("任务号与批号:那一路会话（调度记录原句：发给两路会话那一句）",
                             "先发给「梳理 subagent」那一路会话，它答不归它、指到它的记录 `records/2026-11-17-subagent账本清欠调度记录.md:383`；"
                             "再发给改检出钩子那一路会话，它认领了，在草稿里把三行点名挪到 `allow-bash-scope-write` 下，等换上工具空出来就换、复跑后发结局行。", numbers_shape),
            clock_shape_case("任务号与批号:那一路会话（调度记录原句：腿跑着时换上那一句）",
                             "检出钩子在腿跑着时可能被改检出钩子那一路会话换上 ⑫ 的头注改动，写判决前照快照倒推。", numbers_shape),
            clock_shape_case("任务号与批号:那一路会话（调度记录原句：问过改检出钩子那一句）",
                             "检出钩子换上前问过改 ㉒ 那一路会话，换完把新 sha256 发给它。", numbers_shape),
            clock_shape_case("任务号与批号:那一路的会话与那一路的 agent",
                             "改写闸那一路的会话交回；判据库那一路的 agent 在跑；那一路的子 agent 认领；那一路 subagent 接手", numbers_shape, target=".claude/kb/x.md"),
            clock_shape_case("任务号与批号:那一路后面跟回信、认领、转达这类动作词",
                             "里程碑四收口的那一路回信；GPU 垃圾回收那一路认领；task-server 那一路转达", numbers_shape),
            clock_shape_case("任务号与批号:那一路后面跟问、报红与说法放行", "单条那一路问复用不看红绿；变异那一路报红；那一路的说法照写", None),
            clock_shape_case("任务号与批号:会话后面跟日期、量词与区间放行", "会话 10-14 已合；会话 42 个子 agent；会话 13 份；挂载会话 30 到 0", None),
            clock_shape_case("任务号与批号:记录里批后面跟中文数字的数量放行", "这批两份补丁；批三个实现员；批一起改；批三方审", None),
            clock_shape_case("任务号与批号:agent 全角括号里的说明放行", "子 agent（推的，没实现）；subagent（2026-09-26）", None),
            clock_shape_case("任务号与批号:这一路放行", "这一路改到的文件都绿了", None),
            clock_shape_case("任务号与批号:那一路径放行", "那一路径下的文件照留", None),
            clock_shape_case("任务号与批号:一路绿放行", "门禁一路绿到收尾", None),
            clock_shape_case("任务号与批号:那一路后面不跟会话词放行", "按块那一路与变异那一路都不起", None),
            clock_shape_case("任务号与批号:mainagent 按出场次序的叫法放行", "mainagent(A) 在改 `.claude/hooks/write-guard.sh`，mainagent(B) 认领了检出钩子", None),
            clock_shape_case("任务号与批号:subagent 按出场次序的叫法放行", "subagent(A) 交回了报告，subagent(B) 在草稿里改判据库", None),
            clock_shape_case("任务号与批号:编号、步号、行号与处数放行", "第 6 步、第 40 行、6 处、E158、D13、C948 照写", None),
            clock_shape_case("任务号与批号:数量放行", "每批 64 条、一批 10 个、批 3 次、批 3 处改动、分批 2", None),
            clock_shape_case("任务号与批号:任务个数放行", "从 4G 起的任务 1180 个", None),
            clock_shape_case("任务号与批号:同字后面跟量词或单位放行", "同 4096 字节，同 2048 个", None),
            clock_shape_case("任务号与批号:不带括号的短号与计数放行", "已告诉 87、7a，长度（20）、Some(22)", None),
            clock_shape_case("任务号与批号:日期与相同字样放行", "任务 2026-11-18 跑完，相同 2048、4096 字节", None),
            clock_shape_case("任务号与批号:不到四位的号放行", "任务 123 的输入", None),
            clock_shape_case("任务号与批号:照出路改写的命令与结局行放行",
                             "对照：主工作区跑同一条（`task-server/task submit -- bash scripts/crash/crash-amplification.sh --item x`）同样退 78（结局行 type=command-failed exit=78）", None),
            clock_shape_case("任务号与批号:.py 不判", "# 任务 21036 跑完", None, target=".claude/hooks/parse.py"),
            clock_case("任务号与批号:只在 old_string 里有放行", "records/old-only.md", {"old_string": "（任务 21036）", "new_string": ""}, None),
            clock_case("任务号与批号:写前文件里已有的旧号不挡这一次写", "records/old-number.md", {"old_string": "尾巴", "new_string": "新的一行"}, None),
            clock_case("任务号与批号:改了旧号那一行、号没动放行", "records/old-number.md", {"old_string": "旧行", "new_string": "改过的旧行"}, None),
            clock_shape_case("任务号与批号:门禁样本放行", "（任务 21036）", None, target=".claude/gate.d/fixtures/x/red/records/x.md"),
            clock_case("任务号与批号:仓外草稿目录放行", f"{new_root}/x/progress.md", {"content": "（任务 21036）\n"}, None),
        ]
        numbers_context_cases = [   # 靠上下文规则放行的前十三格在 LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=task-and-session-numbers 时红；后四格靠围栏放行
            clock_shape_case("任务号与批号上下文:反引号里的会话短号与批号放行", "格里写 `(5b)` 与 `批 105` 的样子", None),
            clock_shape_case("任务号与批号上下文:A/B 读数一行放行", "A/B 读数：任务 11964", None, target=verdict_target),
            clock_shape_case("任务号与批号上下文:派发闸认的 A/B 读数写法放行", "**A/B 读数**：任务 11964\n- A/B读数：task 11964", None, target=verdict_target),
            clock_shape_case("任务号与批号上下文:A/B 读数一行两臂两个号放行", "A/B 读数：任务 12400 与任务 12401", None, target=verdict_target),
            clock_shape_case("任务号与批号上下文:长跑拆分一行放行", "长跑拆分：项 x，任务 21036，用时 8000 秒，拆成两项", None),
            clock_shape_case("任务号与批号上下文:列表里的长跑拆分一行放行", "- 长跑拆分：项 x，任务 21036", None),
            clock_shape_case("任务号与批号上下文:表格行里的批号放行", "| 批次 | 内容 |\n|---|---|\n| 批 6 | 交回 |", None),
            clock_case("任务号与批号上下文:Edit 单改表格里一行、表头在文件里放行", "records/table.md",
                       {"old_string": "| 批 5 | 甲 |", "new_string": "| 批 5 | 甲 |\n| 批 6 | 乙 |"}, None),
            clock_shape_case("任务号与批号上下文:kb 里括号里的步号放行", "几何敏感性：6 槽小环（a6）、一（a1）", None, target=".claude/kb/x.md"),
            clock_shape_case("任务号与批号上下文:kb 里中文数字的第几批放行", "mkfs 写的第一批字节就是它", None, target=".claude/kb/x.md"),
            clock_shape_case("任务号与批号上下文:kb 里批后面跟中文数字放行", "截止批次为三", None, target=".claude/kb/x.md"),
            clock_shape_case("任务号与批号上下文:fsync 攒批的大小放行", "每次 fsync 的记录数（seq 批 1 是 8 条），checkpoint 攒批 64、追加批 10", None,
                             target=".claude/kb/x.md"),
            clock_shape_case("任务号与批号上下文:反引号里拿描述指别的会话放行", "旧写法 `改检出钩子那一路会话` 不再用", None),
            clock_shape_case("任务号与批号上下文:围栏代码块里的会话短号与批号放行", "```\n(5b) 批 105 退 0\n```", None),
            clock_case("任务号与批号上下文:Edit 以围栏起头、插入点前没有开着的围栏放行", "records/fence-closed.md",
                       {"old_string": "尾巴", "new_string": closing_fence_then_text}, None),
            clock_case("任务号与批号上下文:Edit 把批号写进围栏放行", "records/multi-fence.md", {"old_string": "尾巴", "new_string": fenced_batch}, None),
            clock_shape_case("任务号与批号上下文:围栏代码块里拿描述指别的会话放行", "```\n改检出钩子那一路会话\n```", None),
        ]
        cases += numbers_cases + numbers_context_cases
        # 判据库放过的「A/B 读数：」一行与派发闸认的是同一个形：两边的两条正则（连同它们用的 LINE_START）编出来的样子与标志逐个相同
        guard_names = {"re": re}
        with open(os.path.join(hook_dir, "runner-dispatch-guard.sh"), encoding="utf-8") as handle:
            guard_source = handle.read()
        for guard_name in ("LINE_START", "ZERO_ROUND_READING_LINE", "ZERO_ROUND_TASK_NUMBER"):
            guard_line = re.search(rf"^{guard_name} = .*$", guard_source, re.M)
            if guard_line:
                exec(guard_line.group(0), guard_names)
        reading_library = load_notations_library(clock_library)
        cases.append(("任务号与批号:判据库认 A/B 读数一行的两条正则与派发闸的同形",
                      [(guard_names[name].pattern, guard_names[name].flags) if name in guard_names else None
                       for name in ("ZERO_ROUND_READING_LINE", "ZERO_ROUND_TASK_NUMBER")],
                      [(getattr(reading_library, name).pattern, getattr(reading_library, name).flags)
                       for name in ("ZERO_ROUND_READING_LINE", "ZERO_ROUND_TASK_NUMBER")]))
        # 派发闸收、值里另带别的写法的号的那四条与两臂、单臂各一条：判据库认出的读数号（zero_round_reading_numbers）与派发闸的认法
        # （它那两条正则在值里 finditer 出的每一处，换算成行内位置）逐条相同；四条在判决里照拒（另带的那一处不是认出的号），两臂、单臂两条放行
        reading_lines = ("A/B 读数：任务 12400，结局 task=12400 type=succeeded exit=0", "A/B 读数：任务 12400（同 12401、12402 两臂）",
                         "A/B 读数：任务 12400，结局 `TASK_SERVER_RESULT task=12400 type=succeeded exit=0`", "A/B 读数：任务 12400；task-server/task log 12400 末行",
                         "A/B 读数：任务 12400 与任务 12401", "A/B 读数：任务 12400")
        def dispatch_reading_numbers(line):
            reading = guard_names["ZERO_ROUND_READING_LINE"].match(line) if "ZERO_ROUND_READING_LINE" in guard_names else None
            if reading is None or "ZERO_ROUND_TASK_NUMBER" not in guard_names:
                return None
            return [(reading.start("value") + found.start(), reading.start("value") + found.end())
                    for found in guard_names["ZERO_ROUND_TASK_NUMBER"].finditer(reading.group("value"))]
        cases.append(("任务号与批号:判据库认出的 A/B 读数号与派发闸的认法逐条相同",
                      [dispatch_reading_numbers(line) for line in reading_lines],
                      [reading_library.zero_round_reading_numbers(line) for line in reading_lines]))
        cases.append(("任务号与批号:派发闸收而值里另带别的写法的号的四条在判决里照拒、两臂与单臂放行",
                      [numbers_shape] * 4 + [None, None],
                      [clock_shape_case("", line, None, target=verdict_target)[2] for line in reading_lines]))
        # 钟点、会话号逐处全报（行号与总数；LIB_FORBIDDEN_NOTATIONS_BREAK=first-hit-only 下前两格红）；Edit 的行号折算成文件里的行号；
        # 九道判完再报：覆盖、越界与钟点三道一次列全（WRITE_GUARD_BREAK=stop-at-first 下末格红）
        def refusal_of(tool_name, target, tool_input, agent=None):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=f"{work}/{target}")}
            if agent:
                hook_input["agent_type"] = agent
            code, message, finding = decide(hook_input, work, table, library, clock_library)
            return message or "", finding
        two_clocks, _ = refusal_of("Write", "records/x.md", {"content": f"用户 23{colon}10 定\n中间一行\n交回 14{colon}00{colon}10、续做\n"})
        cases.append(("钟点:两处都报（行号与总数）", True, all(fragment in two_clocks for fragment in ("共 2 处", "  第 1 行（", "  第 3 行（"))))
        two_sessions, _ = refusal_of("Write", "records/x.md", {"content": glued("会话 singlefs-", "e152 在跑\n已发消息，singlefs-", "e1 回话\n")})
        cases.append(("会话号:两处都报（行号与总数）", True, all(fragment in two_sessions for fragment in ("共 2 处", "  第 1 行：", "  第 2 行："))))
        edit_lines, _ = refusal_of("Edit", "tracked.md", {"old_string": "a", "new_string": f"x\n用户 23{colon}10 定"})
        cases.append(("钟点:Edit 的行号折算成文件里的行号", True, "  第 2 行（" in edit_lines))
        # 紧凑时间戳：Write、Edit、MultiEdit 三个工具各写一次，拒绝点名形态与命中的那一串（开关见上面紧凑时间戳那一段）
        compact_stamp = compact_sample[len("运行目录 "):]
        for tool_name, tool_input in (("Write", {"content": compact_sample + "\n"}),
                                      ("Edit", {"old_string": "a", "new_string": compact_sample}),
                                      ("MultiEdit", {"edits": [{"old_string": "a", "new_string": "b"}, {"old_string": "c", "new_string": compact_sample}]})):
            compact_message, compact_finding = refusal_of(tool_name, "records/x.md", tool_input)
            cases.append((f"紧凑时间戳:{tool_name} 写进仓里拒绝并点名形态与那一处", True,
                          compact_finding == "描述里写了钟点" and f"（{compact_shape}）：「{compact_stamp}」" in compact_message))
        _, three_labels = refusal_of("Write", "untracked.md", {"content": f"用户 23{colon}10 定\n"}, agent="kb-scribe")
        cases.append(("九道判完再报:覆盖、越界与钟点一次列全", "整份覆盖未跟踪文件、越出写范围、描述里写了钟点", three_labels))
        # 十、往现行账本追加节标题：登记表与账本放进自检现场（WRITE_GUARD_BREAK=ledger-append-ignored 下拒绝的几格红，放行的几格照样绿）
        import contextlib, io
        ledger_text = "# 账本\n\n## 索引\n\n| 节号 | 判定 |\n|---|---|\n| 一 | 做完 |\n\n## 一、甲\n\n正文\n"
        added_heading = "\n## 二、乙\n\n正文"
        os.makedirs(os.path.join(work, ".claude", "gate.d"), exist_ok=True)
        with open(os.path.join(work, LEDGER_REGISTRY), "w", encoding="utf-8") as handle:
            handle.write("current\trecords/ledger.md\t-\t样本里的现行账本，自证用\nsealed\trecords/old.md\t" + "0" * 64 + "\t样本里的封存账本，自证用\n")
        os.makedirs(os.path.join(work, "records"), exist_ok=True)
        for ledger_name in ("ledger.md", "old.md"):
            with open(os.path.join(work, "records", ledger_name), "w", encoding="utf-8") as handle:
                handle.write(ledger_text)
        subprocess.run(["git", "-C", work, "add", "records/ledger.md", "records/old.md"], check=True)
        subprocess.run(["git", "-C", work, "commit", "-qm", "ledgers"], check=True)
        ledger_edit_message, ledger_edit_finding = refusal_of("Edit", "records/ledger.md", {"old_string": "正文", "new_string": "正文" + added_heading})
        cases.append(("账本追加:Edit 往现行账本追加节标题拒", "往现行账本追加节标题", ledger_edit_finding))
        cases.append(("账本追加:MultiEdit 往现行账本追加节标题拒", "往现行账本追加节标题",
                      refusal_of("MultiEdit", "records/ledger.md", {"edits": [{"old_string": "甲", "new_string": "丙"}, {"old_string": "正文", "new_string": "正文" + added_heading}]})[1]))
        cases.append(("账本追加:Write 往现行账本追加节标题拒", "往现行账本追加节标题",
                      refusal_of("Write", "records/ledger.md", {"content": ledger_text + added_heading + "\n"})[1]))
        cases.append(("账本追加:拒绝点名新脚本、账本与出路", True,
                      all(fragment in ledger_edit_message for fragment in ("scripts/kb/ledger-append-section.py", "records/ledger.md", "→ 怎么办", "--body-file"))))
        cases.append(("账本追加:改节号放行", None, refusal_of("Edit", "records/ledger.md", {"old_string": "## 一、甲", "new_string": "## 二、甲"})[1]))
        cases.append(("账本追加:改正文放行", None, refusal_of("Edit", "records/ledger.md", {"old_string": "正文", "new_string": "改过的正文"})[1]))
        cases.append(("账本追加:Write 整份改正文放行", None, refusal_of("Write", "records/ledger.md", {"content": ledger_text.replace("正文", "改过的正文")})[1]))
        cases.append(("账本追加:别的文件里追加节标题放行", None, refusal_of("Write", "records/notes.md", {"content": ledger_text + added_heading + "\n"})[1]))
        cases.append(("账本追加:封存的账本放行", None, refusal_of("Edit", "records/old.md", {"old_string": "正文", "new_string": "正文" + added_heading})[1]))
        bare_root = tempfile.mkdtemp(dir=base)
        del UNJUDGED[:]
        try:
            with contextlib.redirect_stderr(io.StringIO()):
                bare_code = decide_ledger_append({"tool_name": "Edit", "tool_input": {"file_path": f"{bare_root}/records/x.md", "old_string": "a", "new_string": "a" + added_heading}}, bare_root)[0]
        finally:
            shutil.rmtree(bare_root, ignore_errors=True)
        bare_notes = list(UNJUDGED)
        del UNJUDGED[:]
        cases.append(("账本追加:登记表读不到时放行、记一处没判全", (0, True), (bare_code, any("账本追加这一道没判" in note for note in bare_notes))))
        # 走真实入口：把脚本当子进程、从标准输入喂 JSON，防「判定函数对、入口读不到输入」；拒绝要落进检出记录
        script = os.path.join(hook_dir, "write-guard.sh")
        detections = os.path.join(work, "detections.jsonl")
        environment = dict(os.environ, CLAUDE_PROJECT_DIR=work, AGENT_HOOK_DETECTIONS=detections)
        def run_entry(hook_input):
            return subprocess.run(["bash", script], input=json.dumps(hook_input), capture_output=True, text=True, env=environment)
        def stdin_case(label, hook_input, want):
            return (label, want, run_entry(hook_input).returncode)
        cases += [
            stdin_case("stdin:主 agent 写新文件", {"tool_name": "Write", "tool_input": {"file_path": f"{work}/crates/a.rs", "content": "x"}}, 0),
            stdin_case("stdin:覆盖未跟踪文件", {"tool_name": "Write", "tool_input": {"file_path": f"{work}/untracked.md", "content": "x"}}, 2),
            stdin_case("stdin:kb-scribe 越界", {"tool_name": "Edit", "agent_type": "kb-scribe", "tool_input": {"file_path": f"{work}/crates/a.rs"}}, 2),
            stdin_case("stdin:kb-scribe 范围内", {"tool_name": "Edit", "agent_type": "kb-scribe", "tool_input": {"file_path": f"{work}/.claude/kb/x.md"}}, 0),
            stdin_case("stdin:kb-scribe 用 MultiEdit 越界", {"tool_name": "MultiEdit", "agent_type": "kb-scribe",
                                                            "tool_input": {"file_path": f"{work}/crates/a.rs", "edits": [{"old_string": "a", "new_string": "b"}]}}, 2),
            stdin_case("stdin:kb-scribe 用 MultiEdit 范围内", {"tool_name": "MultiEdit", "agent_type": "kb-scribe",
                                                              "tool_input": {"file_path": f"{work}/.claude/kb/x.md", "edits": [{"old_string": "a", "new_string": "b"}]}}, 0),
            stdin_case("stdin:大内容越界", {"tool_name": "Write", "agent_type": "implementation-writer", "tool_input": {"file_path": f"{work}/research/big.md", "content": "y" * 300000}}, 2),
            stdin_case("stdin:实现员写工作根下的报告", {"tool_name": "Write", "agent_type": "implementation-writer",
                                                       "tool_input": {"file_path": f"{new_root}/x/result.md", "content": "x"}}, 0),
        ]
        runner_entry = run_entry({"tool_name": "Edit", "agent_type": "experiment-runner",
                                  "tool_input": {"file_path": f"{work}/crates/singlefs-verification/src/bin/e161_x.rs", "old_string": "a", "new_string": "b"}})
        cases.append(("stdin:experiment-runner 改主工作区的入库装置拒绝", 2, runner_entry.returncode))
        cases.append(("stdin:先编后换的拒绝点名那条脚本与草稿目录", True,
                      all(fragment in runner_entry.stderr for fragment in ("scripts/hosts/compile-then-swap.py", "草稿目录", "→ 怎么办"))))
        # 出路经 task-server 提交、不再教直接起（WRITE_GUARD_BREAK=compile-first-capped-hint 下这一格红）
        cases.append(("stdin:先编后换的出路经 task-server 提交、不写 capped.sh 直接起", True,
                      "task-server/task submit -- python3 scripts/hosts/compile-then-swap.py" in runner_entry.stderr and "capped.sh" not in runner_entry.stderr))
        prime_entry = run_entry({"tool_name": "Edit", "tool_input": {"file_path": f"{work}/crates/a.rs", "old_string": "B2", "new_string": "取 B2\u2032 的读法"}})
        cases.append(("stdin:Edit 写进角标", 2, prime_entry.returncode))
        cases.append(("stdin:角标的拒绝带字符、前后几个字与出路", True,
                      all(fragment in prime_entry.stderr for fragment in ("撇号类角标 \u2032", "取 B2\u2032 的读法", "→ 怎么办", "path-moves.md"))))
        trailing_entry = run_entry({"tool_name": "Edit", "tool_input": {"file_path": f"{work}/records/x.sh", "old_string": "python3 - ", "new_string": "python3 -"}})
        cases.append(("stdin:.sh 目标 Edit old_string 以空白收尾", 2, trailing_entry.returncode))
        cases.append(("stdin:行尾空白的拒绝带收尾那几个字、两条出路", True,
                      all(fragment in trailing_entry.stderr for fragment in ("「…python3 -␠」", "→ 怎么办", "多框到下一个非空白字符", "replace-once.py"))))
        cases.append(("stdin:.md 目标 Edit old_string 以空白收尾放行", 0,
                      run_entry({"tool_name": "Edit", "tool_input": {"file_path": f"{work}/records/x.md", "old_string": "python3 - ", "new_string": "python3 - "}}).returncode))
        recorded = sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
        cases.append(("stdin:七次拒绝都落进检出记录", 7, recorded))
        compact_entry = run_entry({"tool_name": "Write", "tool_input": {"file_path": f"{work}/records/compact.md", "content": compact_sample + "\n"}})
        cases.append(("stdin:Write 写进紧凑时间戳拒绝（退出码 2）", 2, compact_entry.returncode))
        cases.append(("stdin:紧凑时间戳的拒绝点名形态与那一处", True,
                      all(fragment in compact_entry.stderr for fragment in (f"（{compact_shape}）：「{compact_stamp}」", "→ 怎么办", "紧凑写法"))))
        # 没有项目定义的 agent 写仓里：入口拒，拒绝说明点名只许写工作根与交回主 agent 的出路（弄坏开关 builtin-agents-unscoped 下红）
        builtin_entry = run_entry({"tool_name": "Write", "agent_type": "general-purpose", "tool_input": {"file_path": f"{work}/crates/builtin.rs", "content": "x"}})
        cases.append(("stdin:内置 agent 写仓里拒", (2, True),
                      (builtin_entry.returncode, all(fragment in builtin_entry.stderr for fragment in ("只许写工作根", "交回主 agent", "→ 怎么办")))))
        link_entry = run_entry({"tool_name": "Edit", "agent_type": "implementation-writer",
                                "tool_input": {"file_path": f"{work}/crates/escape/hosts", "old_string": "a", "new_string": "b"}})
        cases.append(("stdin:范围内的链接指到范围外拒（退出码 2）", 2, link_entry.returncode))
        ledger_detections = os.path.join(work, "ledger-detections.jsonl")
        ledger_entry = subprocess.run(["bash", script], capture_output=True, text=True, env=dict(environment, AGENT_HOOK_DETECTIONS=ledger_detections),
                                      input=json.dumps({"tool_name": "Edit", "tool_input": {"file_path": f"{work}/records/ledger.md", "old_string": "正文", "new_string": "正文" + added_heading}}))
        cases.append(("stdin:往现行账本追加节标题拒（退出码 2）", 2, ledger_entry.returncode))
        cases.append(("stdin:往现行账本追加节标题的拒绝点名新脚本与出路", True,
                      all(fragment in ledger_entry.stderr for fragment in ("scripts/kb/ledger-append-section.py", "records/ledger.md", "→ 怎么办"))))
        # 十一、工具员就地改主工作区里已存在的脚本：拒的几格判的是拒绝的那一道（只这一道），弄坏开关 WRITE_GUARD_BREAK=script-in-place-allowed 下「整份换上:… 拒」
        # 与「stdin:工具员 Edit 主工作区已存在的 … 拒并点名 hook-swap.py 与出路」几格红，放行与判不了的几格照样绿
        open(os.path.join(work, ".claude", "agents", "tooling-writer.md"), "w").write("---\n")
        swap_only_files = {   # 相对仓根 → (内容, 文件权限)；都 git add 过，Write 整份覆盖那一道不判
            "scripts/mutation/crates-mutation-rows.py": ("def main():\n    return 0\n", 0o644),
            "gpu-verification-server/control": ("#!/usr/bin/env bash\necho stop\n", 0o755),
            ".claude/gate.d/stage.sh": ("#!/usr/bin/env bash\necho stage\n", 0o755),
            ".claude/gate.d/lib/helper.py": ("def main():\n    return 0\n", 0o644),   # 门禁共用库：不是门禁阶段，出路照别的脚本
            ".claude/gate.d/lib-checker.py": ("def main():\n    return 0\n", 0o644),   # 门禁目录这一层 lib- 起头的 .py：门禁共用库，不是门禁阶段
            "scripts/mutation/other-rows.py": ("def main():\n    return 0\n", 0o644),   # 不在要临时服务的登记表里的研究脚本
            ".claude/scripts/tool.py": ("def main():\n    return 0\n", 0o644),
            "task-server/task_lib.py": ("def main():\n    return 0\n", 0o644),
            "prune-server/control": ("#!/usr/bin/env bash\necho stop\n", 0o755),
            "scripts/table.tsv": ("a\tb\n", 0o644),
            "scripts/notes.md": ("说明\n", 0o644),
            "scripts/settings.json": ("{}\n", 0o644),
            "task-server/notes.txt": ("说明\n", 0o644),
            ".claude/gate.d/fixtures/sample-red/stage.sh": ("#!/usr/bin/env bash\necho sample\n", 0o755),   # 还没有门禁阶段 .claude/gate.d/sample-red 的样本目录
            ".claude/gate.d/fixtures/stage.sh/stage-red/expect": ("exit=1\n", 0o644),   # 已有门禁 .claude/gate.d/stage.sh 的样本
            "prune-server/src/protocol.rs": ("pub enum Request {\n    Get,\n}\n", 0o644),
            "gpu-verification-server/src/lib.rs": ("pub fn run() {}\n", 0o644),
            "prune-server/Cargo.toml": ("[package]\nname = \"prune-server\"\n", 0o644),
            "prune-server/Cargo.lock": ("version = 3\n", 0o644),
            "gpu-verification-server/Cargo.toml": ("[package]\nname = \"gpu-verification-server\"\n", 0o644),
        }
        for relative, (content, file_mode) in swap_only_files.items():
            os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
            with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                handle.write(content)
            os.chmod(os.path.join(work, relative), file_mode)
        subprocess.run(["git", "-C", work, "add", *swap_only_files], check=True)
        subprocess.run(["git", "-C", work, "commit", "-qm", "swap-only samples"], check=True)
        os.makedirs(os.path.join(work, ".claude", "hooks"), exist_ok=True)   # 要临时服务的自证登记表样本：脚本出路按它分两种
        temporary_service_table = os.path.join(work, TEMPORARY_SERVICE_TABLE)
        with open(temporary_service_table, "w", encoding="utf-8") as handle:
            handle.write("# 样本\nscripts/mutation/crates-mutation-rows.py\trequired\t# 样本里要临时服务的自证\n.claude/scripts/tool.py\town-server\t# 样本里自己起服务的自证\n")
        compiled_binary = os.path.join(work, "prune-server", "target", "debug", "prune_server")   # 编译产物不进 git（target/ 常被全局忽略），只用 Edit 改，覆盖那一道不判
        os.makedirs(os.path.dirname(compiled_binary))
        with open(compiled_binary, "w", encoding="utf-8") as handle:
            handle.write("binary\n")
        os.chmod(compiled_binary, 0o755)
        copied_script = os.path.join(outside_directory, "copy", "scripts", "mutation", "crates-mutation-rows.py")   # 草稿目录与仓副本：主工作区之外的同名脚本
        os.makedirs(os.path.dirname(copied_script))
        with open(copied_script, "w", encoding="utf-8") as handle:
            handle.write("def main():\n    return 0\n")
        def swap_only_finding(tool_name, relative, tool_input, agent="tooling-writer"):
            hook_input = {"tool_name": tool_name, "tool_input": dict(tool_input, file_path=f"{work}/{relative}")}
            if agent:
                hook_input["agent_type"] = agent
            return decide(hook_input, work, table, library)[2]
        def swap_only_code(hook_input):
            with contextlib.redirect_stderr(io.StringIO()):
                return decide_script_swap_only(hook_input, work)[0]
        def swap_only_message(hook_input):
            with contextlib.redirect_stderr(io.StringIO()):
                return decide_script_swap_only(hook_input, work)[1]
        swap_label = "就地改已存在的脚本"
        cases += [
            ("整份换上:工具员 Edit 主工作区已存在的 scripts/mutation/ 下的 .py 拒", swap_label,
             swap_only_finding("Edit", "scripts/mutation/crates-mutation-rows.py", {"old_string": "return 0", "new_string": "return 1"})),
            ("整份换上:工具员 Edit 主工作区已存在、带执行位无后缀的 gpu-verification-server/control 拒", swap_label,
             swap_only_finding("Edit", "gpu-verification-server/control", {"old_string": "echo stop", "new_string": "echo stop all"})),
            ("整份换上:工具员 Write 整份重写已存在的 .claude/gate.d/ 下的 .sh 拒", swap_label,
             swap_only_finding("Write", ".claude/gate.d/stage.sh", {"content": "#!/usr/bin/env bash\necho stage two\n"})),
            ("整份换上:工具员 MultiEdit 已存在的 .claude/scripts/ 下的 .py 拒", swap_label,
             swap_only_finding("MultiEdit", ".claude/scripts/tool.py", {"edits": [{"old_string": "return 0", "new_string": "return 2"}]})),
            ("整份换上:工具员 Edit 已存在的 task-server/ 下的 .py 拒", swap_label,
             swap_only_finding("Edit", "task-server/task_lib.py", {"old_string": "return 0", "new_string": "return 3"})),
            ("整份换上:工具员 Edit 已存在、带执行位无后缀的 prune-server/control 拒", swap_label,
             swap_only_finding("Edit", "prune-server/control", {"old_string": "echo stop", "new_string": "echo stop all"})),
            # 不拦的几种：新建、不带执行位的数据与文档、fixtures 下的样本、草稿目录与仓副本、别的 agent 类型与主 agent
            ("整份换上:工具员新建 scripts/ 下的 .py 放行", None,
             swap_only_finding("Write", "scripts/mutation/new_tool.py", {"content": "def main():\n    return 0\n"})),
            ("整份换上:工具员改不带执行位的 .tsv、.md、.json、.txt 放行", (None, None, None, None),
             tuple(swap_only_finding("Edit", relative, {"old_string": old, "new_string": old + "x"})
                   for relative, old in (("scripts/table.tsv", "a"), ("scripts/notes.md", "说明"), ("scripts/settings.json", "{}"), ("task-server/notes.txt", "说明")))),
            ("整份换上:工具员改还没有门禁阶段的样本目录（.claude/gate.d/fixtures/sample-red/）里带执行位的样本放行", None,
             swap_only_finding("Edit", ".claude/gate.d/fixtures/sample-red/stage.sh", {"old_string": "echo sample", "new_string": "echo sample two"})),
            # 已有门禁的样本档目录：改已有的、新建的都拒，主 agent 与别的 agent 类型放行；弄坏开关 WRITE_GUARD_BREAK=gate-fixtures-in-place-allowed 下拒的几格红
            ("整份换上:工具员 Write 已有门禁的一份样本拒", swap_label,
             swap_only_finding("Write", ".claude/gate.d/fixtures/stage.sh/stage-red/expect", {"content": "exit=0\n"})),
            ("整份换上:工具员在已有门禁的样本档目录里新建样本拒", swap_label,
             swap_only_finding("Write", ".claude/gate.d/fixtures/stage.sh/stage-new-red/expect", {"content": "exit=1\n"})),
            ("整份换上:主 agent 改已有门禁的样本放行", None,
             swap_only_finding("Edit", ".claude/gate.d/fixtures/stage.sh/stage-red/expect", {"old_string": "exit=1", "new_string": "exit=0"}, agent=None)),
            ("整份换上:实现员改已有门禁的样本不归这一道", 0,
             swap_only_code({"tool_name": "Edit", "agent_type": "implementation-writer",
                             "tool_input": {"file_path": f"{work}/.claude/gate.d/fixtures/stage.sh/stage-red/expect", "old_string": "exit=1", "new_string": "exit=0"}})),
            ("整份换上:工具员改主工作区之外（草稿目录与仓副本）的同名脚本放行", 0,
             swap_only_code({"tool_name": "Edit", "agent_type": "tooling-writer",
                             "tool_input": {"file_path": copied_script, "old_string": "return 0", "new_string": "return 1"}})),
            ("整份换上:实现员改同一份脚本不归这一道", 0,
             swap_only_code({"tool_name": "Edit", "agent_type": "implementation-writer",
                             "tool_input": {"file_path": f"{work}/scripts/mutation/crates-mutation-rows.py", "old_string": "return 0", "new_string": "return 1"}})),
            ("整份换上:主 agent 改同一份脚本放行", None,
             swap_only_finding("Edit", "scripts/mutation/crates-mutation-rows.py", {"old_string": "return 0", "new_string": "return 1"}, agent=None)),
            # 两个服务的 src/：工具员就地给协议枚举加成员、match 还没补全时主工作区编不过，带临时判定库服务的自证都取不到二进制；不看后缀与执行位，一律拒
            ("整份换上:工具员 Edit 主工作区已存在的 prune-server/src/protocol.rs 拒", swap_label,
             swap_only_finding("Edit", "prune-server/src/protocol.rs", {"old_string": "    Get,\n", "new_string": "    Get,\n    Put,\n"})),
            ("整份换上:工具员 Edit 主工作区已存在的 gpu-verification-server/src/ 下的文件拒", swap_label,
             swap_only_finding("Edit", "gpu-verification-server/src/lib.rs", {"old_string": "pub fn run() {}", "new_string": "pub fn run() { }"})),
            ("整份换上:工具员新建 prune-server/src/ 下的文件放行", None,
             swap_only_finding("Write", "prune-server/src/new_module.rs", {"content": "pub fn added() {}\n"})),
            ("整份换上:工具员改 prune-server/target/ 下带执行位的文件放行", None,
             swap_only_finding("Edit", "prune-server/target/debug/prune_server", {"old_string": "binary", "new_string": "binary two"})),
        ]
        source_refusal = swap_only_message({"tool_name": "Edit", "agent_type": "tooling-writer",
                                            "tool_input": {"file_path": f"{work}/prune-server/src/protocol.rs", "old_string": "    Get,\n", "new_string": "    Get,\n    Put,\n"}})
        cases.append(("整份换上:两个服务 src/ 的拒绝给仓副本与补丁的出路", True,
                      bool(source_refusal) and all(fragment in source_refusal for fragment in ("仓副本", "建仓副本", "补丁", "要主 agent 跑的命令", "→ 怎么办"))
                      and HOOK_SWAP_SCRIPT not in source_refusal))
        # 两个服务的构建清单（Cargo.toml、Cargo.lock）：与 src/ 同一个出路；新建与主 agent 放行。弄坏开关 WRITE_GUARD_BREAK=swap-manifests-unseen 下拒的几格与出路那一格红
        cases += [
            ("整份换上:工具员 Edit 主工作区已存在的 prune-server/Cargo.toml 拒", swap_label,
             swap_only_finding("Edit", "prune-server/Cargo.toml", {"old_string": "[package]", "new_string": "[package]\n"})),
            ("整份换上:工具员 Edit 主工作区已存在的 prune-server/Cargo.lock 拒", swap_label,
             swap_only_finding("Edit", "prune-server/Cargo.lock", {"old_string": "version = 3", "new_string": "version = 4"})),
            ("整份换上:工具员 Write 整份重写已存在的 gpu-verification-server/Cargo.toml 拒", swap_label,
             swap_only_finding("Write", "gpu-verification-server/Cargo.toml", {"content": "[package]\nname = \"changed\"\n"})),
            ("整份换上:工具员新建 gpu-verification-server/Cargo.lock 放行", None,
             swap_only_finding("Write", "gpu-verification-server/Cargo.lock", {"content": "version = 3\n"})),
            ("整份换上:主 agent 改 prune-server/Cargo.toml 放行", None,
             swap_only_finding("Edit", "prune-server/Cargo.toml", {"old_string": "[package]", "new_string": "[package]\n"}, agent=None)),
        ]
        manifest_refusal = swap_only_message({"tool_name": "Edit", "agent_type": "tooling-writer",
                                              "tool_input": {"file_path": f"{work}/prune-server/Cargo.toml", "old_string": "[package]", "new_string": "[package]\n"}})
        cases.append(("整份换上:两个服务构建清单的拒绝给仓副本与补丁的出路", True,
                      bool(manifest_refusal) and all(fragment in manifest_refusal for fragment in ("构建清单", "仓副本", "建仓副本", "补丁", "要主 agent 跑的命令", "→ 怎么办"))
                      and HOOK_SWAP_SCRIPT not in manifest_refusal))
        # 脚本的出路按 TEMPORARY_SERVICE_TABLE 分两种：登记成 required 的不加 --verify、自证另交、没有自动回滚；不在表里与登记成 own-server 的照旧加 --verify；表读不到两种都写。
        # 弄坏开关 WRITE_GUARD_BREAK=swap-remedy-always-verify 下 required 那一格与表读不到那一格红
        def swap_remedy_form(relative, old):
            message = swap_only_message({"tool_name": "Edit", "agent_type": "tooling-writer",
                                         "tool_input": {"file_path": f"{work}/{relative}", "old_string": old, "new_string": old + "x"}}) or ""
            return (SWAP_WITH_VERIFY_REMEDY in message, "不加 --verify" in message,
                    "没有自动回滚" in message and "另交一条" in message and TEMPORARY_SERVICE_WRAPPER in message and "--with-temporary-service" in message)
        cases += [
            ("整份换上:登记成 required 的脚本，拒绝说明写不加 --verify、自证换上之后另交一条、没有自动回滚", (False, True, True),
             swap_remedy_form("scripts/mutation/crates-mutation-rows.py", "return 0")),
            ("整份换上:不在登记表里的脚本，拒绝说明照旧写加 --verify 换上", (True, False, False),
             swap_remedy_form("scripts/mutation/other-rows.py", "return 0")),
            ("整份换上:登记成 own-server 的脚本照不在表里办，写加 --verify 换上", (True, False, False),
             swap_remedy_form(".claude/scripts/tool.py", "return 0")),
            ("整份换上:门禁共用库（.claude/gate.d/lib/ 下）不是门禁阶段，照不在登记表里的脚本写加 --verify 换上", (True, False, False),
             swap_remedy_form(".claude/gate.d/lib/helper.py", "return 0")),
        ]
        # 门禁阶段（.claude/gate.d/ 这一层的 .sh）单走一支：不带 --verify、验证是这一道的样本档（经 stage-selftest.sh，有的要包在临时 prune-server 底下）换上之后另交一条、没有自动回滚、
        # 阶段与别的脚本同在一份规格里整份不带 --verify；不读登记表（表读不到也不记没判全）。弄坏开关 WRITE_GUARD_BREAK=gate-stage-remedy-as-script 下这几格红
        def swap_stage_form():
            message = swap_only_message({"tool_name": "Edit", "agent_type": "tooling-writer",
                                         "tool_input": {"file_path": f"{work}/.claude/gate.d/stage.sh", "old_string": "echo stage", "new_string": "echo stage two"}}) or ""
            return (SWAP_WITH_VERIFY_REMEDY in message, "不带 --verify 换上" in message, "不加 --verify" in message,
                    "样本档由主 agent 在换上之后另交一条" in message and STAGE_SELFTEST_SCRIPT in message and "--item <阶段文件名>" in message,
                    "--with-temporary-service" in message and TEMPORARY_SERVICE_WRAPPER in message and SELFTEST_REGISTRY_TABLE in message,
                    "没有自动回滚" in message and "reverse-<时刻>.json" in message,
                    "门禁阶段单放一份规格" in message)
        cases.append(("整份换上:门禁阶段（.claude/gate.d/ 顶层的 .sh）单走一支：不带 --verify、样本档换上之后另交一条、没有自动回滚、阶段单放一份规格",
                      (False, True, False, True, True, True, True), swap_stage_form()))
        del UNJUDGED[:]
        os.rename(temporary_service_table, temporary_service_table + ".off")
        try:
            stage_unreadable_form = swap_stage_form()
        finally:
            os.rename(temporary_service_table + ".off", temporary_service_table)
        stage_unreadable_notes = list(UNJUDGED)
        del UNJUDGED[:]
        cases.append(("整份换上:登记表读不到时门禁阶段照写不带 --verify、不记没判全（不读登记表）", (False, True, False, True, True, True, True, 0),
                      (*stage_unreadable_form, len(stage_unreadable_notes))))
        stage_python_message = swap_only_message({"tool_name": "Edit", "agent_type": "tooling-writer",
                                                  "tool_input": {"file_path": f"{work}/.claude/gate.d/lib-checker.py", "old_string": "return 0", "new_string": "return 1"}}) or ""
        cases.append(("整份换上:.claude/gate.d/ 顶层 lib- 起头的 .py 是门禁共用库、不是门禁阶段，照不在登记表里的脚本写加 --verify 换上", (True, False),
                      (SWAP_WITH_VERIFY_REMEDY in stage_python_message, "不带 --verify 换上" in stage_python_message)))
        del UNJUDGED[:]
        os.rename(temporary_service_table, temporary_service_table + ".off")
        try:
            unreadable_form = swap_remedy_form("scripts/mutation/crates-mutation-rows.py", "return 0")
        finally:
            os.rename(temporary_service_table + ".off", temporary_service_table)
        unreadable_notes = list(UNJUDGED)
        del UNJUDGED[:]
        cases.append(("整份换上:登记表读不到时两种出路都写、记一处没判全", (True, True, True, True),
                      (*unreadable_form, any("读不到" in note and "两种出路都写" in note for note in unreadable_notes))))
        # 判不了照常放行而记一处没判全：带 agent_id 而没有 agent_type 的子 agent 调用；目标所在目录不许进（chmod 000）、stat 失败
        del UNJUDGED[:]
        untyped_code = swap_only_code({"tool_name": "Edit", "agent_id": "a0000000000000ff6",
                                       "tool_input": {"file_path": f"{work}/scripts/mutation/crates-mutation-rows.py", "old_string": "return 0", "new_string": "return 1"}})
        untyped_notes = list(UNJUDGED)
        del UNJUDGED[:]
        cases.append(("整份换上:认不出 agent 类型的子 agent 调用照常放行、记一处没判全", (0, True),
                      (untyped_code, any("认不出 agent 类型" in note for note in untyped_notes))))
        sealed_directory = os.path.join(work, "scripts", "sealed")
        os.makedirs(sealed_directory)
        with open(os.path.join(sealed_directory, "tool.py"), "w", encoding="utf-8") as handle:
            handle.write("def main():\n    return 0\n")
        os.chmod(sealed_directory, 0)
        try:
            sealed_code = swap_only_code({"tool_name": "Edit", "agent_type": "tooling-writer",
                                          "tool_input": {"file_path": os.path.join(sealed_directory, "tool.py"), "old_string": "return 0", "new_string": "return 1"}})
        finally:
            os.chmod(sealed_directory, 0o700)
        sealed_notes = list(UNJUDGED)
        del UNJUDGED[:]
        cases.append(("整份换上:目标 stat 失败照常放行、记一处没判全", (0, True), (sealed_code, any("stat 失败" in note for note in sealed_notes))))
        fixture_refusal_message = swap_only_message({"tool_name": "Edit", "agent_type": "tooling-writer",
                                                     "tool_input": {"file_path": f"{work}/.claude/gate.d/fixtures/stage.sh/stage-red/expect", "old_string": "exit=1", "new_string": "exit=0"}}) or ""
        cases.append(("整份换上:已有门禁的样本的拒绝给草稿目录、补丁与打补丁的出路", True,
                      all(fragment in fixture_refusal_message for fragment in ("stage.sh", "fixtures-base", "fixtures-new", "git diff --no-index", "git apply -p2 --check",
                                                                                "要主 agent 跑的命令", "样本档之前", "→ 怎么办"))))
        del UNJUDGED[:]
        untyped_fixture_code = swap_only_code({"tool_name": "Write", "agent_id": "a0000000000000ff7",
                                               "tool_input": {"file_path": f"{work}/.claude/gate.d/fixtures/stage.sh/stage-red/expect", "content": "exit=0\n"}})
        untyped_fixture_notes = list(UNJUDGED)
        del UNJUDGED[:]
        cases.append(("整份换上:认不出 agent 类型的子 agent 写已有门禁的样本照常放行、记一处没判全", (0, True),
                      (untyped_fixture_code, any("认不出 agent 类型" in note and "样本档目录" in note for note in untyped_fixture_notes))))
        # 走真实入口：上面两回事故的两条写法，拒绝点名 hook-swap.py、「要主 agent 跑的命令」与草稿目录，只这一道（检出记录单用一份）
        swap_only_environment = dict(environment, AGENT_HOOK_DETECTIONS=os.path.join(work, "swap-only-detections.jsonl"))
        for label, relative, old, new in (
                ("stdin:工具员 Edit 主工作区已存在的 scripts/mutation/ 下的 .py 拒并点名 hook-swap.py 与出路", "scripts/mutation/crates-mutation-rows.py", "return 0", "return 1"),
                ("stdin:工具员 Edit 主工作区已存在的 gpu-verification-server/control 拒并点名 hook-swap.py 与出路", "gpu-verification-server/control", "echo stop", "echo stop all")):
            swap_only_entry = subprocess.run(["bash", script], capture_output=True, text=True, env=swap_only_environment,
                                             input=json.dumps({"tool_name": "Edit", "agent_type": "tooling-writer",
                                                               "tool_input": {"file_path": f"{work}/{relative}", "old_string": old, "new_string": new}}))
            cases.append((label, (2, True), (swap_only_entry.returncode,
                                             all(fragment in swap_only_entry.stderr for fragment in (relative, HOOK_SWAP_SCRIPT, MAKE_SWAP_SPEC_SCRIPT, "要主 agent 跑的命令", "草稿目录", "→ 怎么办"))
                                             and "道判定不成立" not in swap_only_entry.stderr)))
        fixture_entry = subprocess.run(["bash", script], capture_output=True, text=True, env=swap_only_environment,
                                       input=json.dumps({"tool_name": "Write", "agent_type": "tooling-writer",
                                                         "tool_input": {"file_path": f"{work}/.claude/gate.d/fixtures/stage.sh/stage-new-red/expect", "content": "exit=1\n"}}))
        cases.append(("stdin:工具员 Write 已有门禁的样本档目录里的新样本拒、点名补丁出路", (2, True),
                      (fixture_entry.returncode, all(fragment in fixture_entry.stderr for fragment in ("stage.sh", "fixtures.patch", "git apply -p2", "→ 怎么办")))))
        stage_entry = subprocess.run(["bash", script], capture_output=True, text=True, env=swap_only_environment,
                                     input=json.dumps({"tool_name": "Edit", "agent_type": "tooling-writer",
                                                       "tool_input": {"file_path": f"{work}/.claude/gate.d/stage.sh", "old_string": "echo stage", "new_string": "echo stage two"}}))
        cases.append(("stdin:工具员 Edit 主工作区已存在的 .claude/gate.d/stage.sh 拒、点名不带 --verify 与样本档换上之后另交一条", (2, True),
                      (stage_entry.returncode, all(fragment in stage_entry.stderr for fragment in ("不带 --verify 换上", "stage-selftest.sh", "没有自动回滚", "→ 怎么办"))
                       and SWAP_WITH_VERIFY_REMEDY not in stage_entry.stderr)))
        # 判不了照常放行而记检出：坏 JSON、JSON 数组各喂一次，检出记录单用一份（开关 unparsable-input-silent 下诊断与检出那几格红）
        unjudged_detections = os.path.join(work, "unjudged-detections.jsonl")
        def run_raw(raw_text, extra_environment=None):
            return subprocess.run(["bash", script], input=raw_text, capture_output=True, text=True,
                                  env=dict(environment, AGENT_HOOK_DETECTIONS=unjudged_detections, **(extra_environment or {})))
        def unjudged_records():
            return [json.loads(line) for line in open(unjudged_detections, encoding="utf-8")] if os.path.exists(unjudged_detections) else []
        broken_entry = run_raw('{"tool_name": "Write", "tool_input": {"file_path": "x.md"')
        broken_records = unjudged_records()
        cases.append(("stdin:坏 JSON 放行（退出码 0）", 0, broken_entry.returncode))
        cases.append(("stdin:坏 JSON 的 stderr 有诊断", True,
                      all(fragment in broken_entry.stderr for fragment in ("write-guard.sh", "不是合法的 JSON 对象", '{"tool_name": "Write"'))))
        cases.append(("stdin:坏 JSON 记一条检出（结局只记、会话号读不到）", (1, "只记", "读不到", True),
                      (len(broken_records), broken_records[-1].get("outcome") if broken_records else None, broken_records[-1].get("session_id") if broken_records else None,
                       bool(broken_records) and all(str(finding).startswith("写闸没判全：") for finding in broken_records[-1].get("findings") or [None]))))
        run_raw('{"session_id": "selftest-session", "agent_type": "implementation-writer", "tool_name": "Edit"')
        cases.append(("stdin:坏 JSON 里按字面找得到的会话号与类型照抄", ("selftest-session", "implementation-writer"),
                      tuple(unjudged_records()[-1].get(key) for key in ("session_id", "agent_type")) if len(unjudged_records()) == 2 else None))
        array_entry = run_raw("[1, 2]")
        cases.append(("stdin:JSON 数组记一条检出", (0, 3), (array_entry.returncode, len(unjudged_records()))))
        # 引导载不全任何一份部分文件（读不到、编不过、截空）：拷一份钩子目录（入口加部分文件），删掉、写坏或截成空文件其中一份，按缺依赖的处置拒（退 2、stderr 点名那一份），不抛错退 1、不静默放行；
        # 截空各一格：第一份（decisions）截空、带 main 的那一份（selftest）截空。读不到部分文件时只放行 Write 写回点名的那一份本身（退 0、记一条检出），Edit 那一份照拒；
        # 开关 part-unreadable-passes 下引导照原样抛错、进程退 1，下面几格红
        bootstrap_hooks = os.path.join(work, "bootstrap-copy", ".claude", "hooks")
        os.makedirs(bootstrap_hooks)
        bootstrap_input = json.dumps({"tool_name": "Write", "session_id": "bootstrap-part", "tool_input": {"file_path": f"{work}/crates/bootstrap-part.rs", "content": "x\n"}})
        def damaged_bootstrap_copy(part_name, damage):
            for copied_name in ("write-guard.sh", "lib_work_root.py") + WRITE_GUARD_PART_NAMES:
                shutil.copy(os.path.join(hook_dir, copied_name), bootstrap_hooks)
            if damage == "missing":
                os.remove(os.path.join(bootstrap_hooks, part_name))
            elif damage == "emptied":
                open(os.path.join(bootstrap_hooks, part_name), "w", encoding="utf-8").close()
            else:
                open(os.path.join(bootstrap_hooks, part_name), "w", encoding="utf-8").write("def broken(:\n")
        for part_name in WRITE_GUARD_PART_NAMES:
            for damage, damage_label in (("missing", "读不到"), ("syntax-error", "编不过"), ("emptied", "截空")):
                damaged_bootstrap_copy(part_name, damage)
                bootstrap_run = subprocess.run(["bash", os.path.join(bootstrap_hooks, "write-guard.sh")], input=bootstrap_input, capture_output=True, text=True, env=environment)
                cases.append((f"引导:{damage_label}部分文件 {part_name} 按拒判（退 2、点名那一份）", (2, True),
                              (bootstrap_run.returncode, f"部分文件 {os.path.join(bootstrap_hooks, part_name)}" in bootstrap_run.stderr and "按拒判" in bootstrap_run.stderr)))
        damaged_bootstrap_copy(WRITE_GUARD_PART_NAMES[0], "missing")
        bootstrap_detections = os.path.join(work, "bootstrap-detections.jsonl")
        written_part = os.path.join(bootstrap_hooks, WRITE_GUARD_PART_NAMES[0])
        for tool_name, expected_exit, expected_records, label in (("Write", 0, 1, "对写回那一份本身的 Write 放行（退 0、记一条检出）"), ("Edit", 2, 0, "对那一份本身的 Edit 照拒（退 2、点名那一份）")):
            part_input = json.dumps({"tool_name": tool_name, "session_id": "bootstrap-part", "tool_input": {"file_path": written_part, "content": "x\n", "old_string": "a", "new_string": "b"}})
            records_before = len(open(bootstrap_detections, encoding="utf-8").readlines()) if os.path.exists(bootstrap_detections) else 0
            bootstrap_run = subprocess.run(["bash", os.path.join(bootstrap_hooks, "write-guard.sh")], input=part_input, capture_output=True, text=True,
                                           env=dict(environment, AGENT_HOOK_DETECTIONS=bootstrap_detections))
            recorded = len(open(bootstrap_detections, encoding="utf-8").readlines()) - records_before if os.path.exists(bootstrap_detections) else 0
            cases.append((f"引导:读不到部分文件时{label}", (expected_exit, expected_records, True), (bootstrap_run.returncode, recorded, f"部分文件 {written_part}" in bootstrap_run.stderr)))
        # ── 九、草稿目录写者锁：SINGLEFS_WORK_ROOT 指到自检自己的工作根，锁与主会话记录都在里面 ──
        # 开关见文件头 --selftest 一段；下面「写者锁:」「stdin:第二个写者」「stdin:持锁者」开头的格各自点名在开关下红
        lock_root = tempfile.mkdtemp(dir=base)
        work_root_before = os.environ.get("SINGLEFS_WORK_ROOT")
        os.environ["SINGLEFS_WORK_ROOT"] = lock_root
        try:
            if write_guard_decisions.writer_lock is None:
                cases.append(("写者锁:库读得到", True, False))
            else:
                prompt_one, prompt_other, prompt_three = "写者锁自检的派发提示甲", "写者锁自检的派发提示乙", "写者锁自检的派发提示丙"
                prompt_four = "写者锁自检的派发提示丁"   # 没有任何回执对得上它
                lock_transcript = os.path.join(lock_root, "main-session.jsonl")
                def make_lock(name, prompt_text):
                    scratch = os.path.join(lock_root, name)
                    write_guard_decisions.writer_lock.create_lock(scratch, write_guard_decisions.writer_lock.new_lock("self-test", prompt_text, "implementation-writer", lock_transcript))
                    return scratch
                def receipt(agent_id, prompt_text, seconds_ago=0):
                    stamped = datetime.fromtimestamp(time.time() - seconds_ago, timezone.utc).isoformat().replace("+00:00", "Z")
                    return json.dumps({"timestamp": stamped, "toolUseResult": {"status": "async_launched", "agentId": agent_id, "prompt": prompt_text}}, ensure_ascii=False)
                for name in ("w1", "w2", "w4", "w5", "w6", "w7/sub", "w9"):
                    os.makedirs(os.path.join(lock_root, name), exist_ok=True)
                make_lock("w1", prompt_one)
                make_lock("w4", prompt_three)
                make_lock("w6", prompt_one)       # 回执对不上的拒绝点名持锁者：持锁者（提示甲）还没写过
                make_lock("w7", prompt_one)       # 嵌套：父目录锁是持锁者甲的
                make_lock("w7/sub", prompt_other)  # 嵌套：子目录锁是对手乙的
                make_lock("w9", prompt_four)      # 没有回执对得上这把锁
                open(os.path.join(lock_root, "w5", write_guard_decisions.writer_lock.LOCK_NAME), "w", encoding="utf-8").write("{不是 JSON")
                for name, dropped in (("w13", "nonce"), ("w14", "dispatched_at"), ("w15", "prompt_sha256")):   # 锁文件在、字段缺一项（开关 unreadable-lock-absent 下这几格红）
                    os.makedirs(os.path.join(lock_root, name))
                    partial = write_guard_decisions.writer_lock.new_lock("self-test", prompt_one, "implementation-writer", lock_transcript)
                    partial.pop(dropped)
                    with open(os.path.join(lock_root, name, write_guard_decisions.writer_lock.LOCK_NAME), "w", encoding="utf-8") as partial_file:
                        json.dump(partial, partial_file)
                os.makedirs(os.path.join(lock_root, "w16", write_guard_decisions.writer_lock.LOCK_NAME))   # 锁的位置是个目录
                for name, broken_claim in (("w22", "{不是 JSON"), ("w23", json.dumps({"claimed_at": 1}))):   # 认领文件在而判不了（开关 broken-claim-absent 下点名那份认领文件的几格红）
                    broken_scratch = make_lock(name, prompt_one)
                    with open(write_guard_decisions.writer_lock.claim_path(broken_scratch, write_guard_decisions.writer_lock.lock_state(broken_scratch)[1]), "w", encoding="utf-8") as claim_file:
                        claim_file.write(broken_claim)
                holder_id, rival_id, zombie_id, first_id, second_id = ("a0000000000000aa1", "a0000000000000bb2", "a0000000000000cc3", "a0000000000000dd4", "a0000000000000ee5")
                with open(lock_transcript, "w", encoding="utf-8") as transcript_file:   # 回执：持锁者（提示甲、锁之后）、对手（提示乙）、旧进程（提示甲、一小时前）；w4 的两个写者没有回执
                    transcript_file.write("\n".join([receipt(rival_id, prompt_other), receipt(zombie_id, prompt_one, seconds_ago=3600), receipt(holder_id, prompt_one)]) + "\n")
                def lock_hook_input(agent_id, relative):
                    return {"tool_name": "Write", "agent_id": agent_id, "agent_type": "implementation-writer", "transcript_path": lock_transcript,
                            "tool_input": {"file_path": os.path.join(lock_root, relative), "content": "x"}}
                def lock_case(label, agent_id, relative, want):
                    return (label, want, decide(lock_hook_input(agent_id, relative), work, table, library)[0])
                def refusal_names_holder():   # 持锁者认领之后才有「持锁者是谁」可写：在 cases 列表里按次序求值
                    message = decide(lock_hook_input(rival_id, "w1/d.md"), work, table, library)[1] or ""
                    return all(fragment in message for fragment in (holder_id, rival_id, "→ 怎么办", "写者锁已判："))
                def bad_lock_refusal_names_lock():
                    message = decide(lock_hook_input(rival_id, "w5/z.md"), work, table, library)[1] or ""
                    return all(fragment in message for fragment in (os.path.join(lock_root, "w5", write_guard_decisions.writer_lock.LOCK_NAME), "判不了谁持锁", "→ 怎么办", "→ 主 agent", "rm "))
                def foreign_message(agent_id, relative):   # 持锁者还没认领的目录里被拒的说明（在 cases 列表里按次序求值，w6、w9 没人认领过）
                    return decide(lock_hook_input(agent_id, relative), work, table, library)[1] or ""
                def foreign_names_expected_holder():
                    rival_message, zombie_message = foreign_message(rival_id, "w6/a.md"), foreign_message(zombie_id, "w6/b.md")
                    return (f"子 agent {holder_id}" in rival_message and "还没有认领文件" in rival_message and rival_id in rival_message and "→ 怎么办" in rival_message
                            and f"子 agent {holder_id}" in zombie_message and zombie_message.count(zombie_id) == 1)
                def foreign_unfound_names_registration():
                    message = foreign_message(rival_id, "w9/a.md")
                    return "主会话记录里没找到它的回执" in message and write_guard_decisions.writer_lock.prompt_digest(prompt_four)[:write_guard_decisions.writer_lock.PROMPT_DIGEST_SHOWN] in message and "→ 怎么办" in message
                def broken_claim_refusal_names_claim():
                    shown = []
                    for name in ("w22", "w23"):
                        broken_scratch = os.path.join(lock_root, name)
                        message = decide(lock_hook_input(holder_id, f"{name}/m.md"), work, table, library)[1] or ""
                        claim_file = write_guard_decisions.writer_lock.claim_path(broken_scratch, write_guard_decisions.writer_lock.lock_state(broken_scratch)[1])
                        shown.append(all(fragment in message for fragment in (claim_file, "判不了谁持锁", "→ 怎么办", "→ 主 agent", "rm ")))
                    return all(shown)
                def claim_owner(name):   # 这个目录的锁此刻的认领：认领者的 agent_id，没有认领交 "absent"，判不了交 "unreadable"
                    scratch = os.path.join(lock_root, name)
                    state, claim = write_guard_decisions.writer_lock.claim_state(scratch, write_guard_decisions.writer_lock.lock_state(scratch)[1])
                    return claim["agent_id"] if state == "ok" else state
                def append_receipt(agent_id, prompt_text):
                    with open(lock_transcript, "a", encoding="utf-8") as transcript_file:
                        transcript_file.write(receipt(agent_id, prompt_text) + "\n")
                def receipt_pending_cases():
                    """w4 的锁登记提示丙：乙（first_id）一直没有回执，甲（second_id）的回执晚落盘，same_prompt_id 是同一份提示的另一次派发。
                    回执读不到的写放行、不认领、记一处没判全（开关 missing-receipt-claims 下退回先到先得，前四格红）；按次序求值。"""
                    import contextlib, io
                    def pending_case(label, agent_id, relative):
                        del UNJUDGED[:]
                        with contextlib.redirect_stderr(io.StringIO()):
                            code = decide(lock_hook_input(agent_id, relative), work, table, library)[0]
                        notes = list(UNJUDGED)
                        del UNJUDGED[:]
                        return (label, (0, "absent", True), (code, claim_owner("w4"), any("回执还没落盘，没认领" in note for note in notes)))
                    same_prompt_id = "a0000000000000ff6"
                    found = [pending_case("写者锁:回执还没落盘的乙先写放行、不认领、记一处没判全", first_id, "w4/x.md"),
                             pending_case("写者锁:真持锁者回执还没落盘时第一次写也放行、不认领", second_id, "w4/y.md")]
                    append_receipt(second_id, prompt_three)
                    found.append(("写者锁:回执落盘之后持锁者写拿到锁", (0, second_id), (decide(lock_hook_input(second_id, "w4/z.md"), work, table, library)[0], claim_owner("w4"))))
                    found.append(lock_case("写者锁:持锁者认领之后，回执一直没落盘的乙写被拒", first_id, "w4/u.md", 2))
                    append_receipt(same_prompt_id, prompt_three)
                    found.append(lock_case("写者锁:认领之后同一份提示的另一次派发写被拒", same_prompt_id, "w4/v.md", 2))
                    found.append(lock_case("写者锁:认领之后持锁者续写放行", second_id, "w4/w.md", 0))
                    return found
                cases += [
                    lock_case("写者锁:提示对不上的子 agent 不能认领", rival_id, "w1/a.md", 2),
                    lock_case("写者锁:同一份提示但派发早于这把锁的旧进程不能认领", zombie_id, "w1/a.md", 2),
                    lock_case("写者锁:持锁者第一次写放行", holder_id, "w1/a.md", 0),
                    lock_case("写者锁:持锁者续写放行", holder_id, "w1/b.md", 0),
                    lock_case("写者锁:持锁者写子目录放行", holder_id, "w1/sub/c.md", 0),
                    lock_case("写者锁:第二个写者被拒", rival_id, "w1/c.md", 2),
                    lock_case("写者锁:第二个写者写子目录也被拒", rival_id, "w1/sub/deeper/e.md", 2),
                    ("写者锁:拒绝说明点名持锁者与出路", True, refusal_names_holder()),
                    *receipt_pending_cases(),
                    lock_case("写者锁:主 agent 不判", None, "w1/f.md", 0),
                    lock_case("写者锁:没有锁的目录不判", rival_id, "w2/x.md", 0),
                    lock_case("写者锁:锁文件不是 JSON 拒", rival_id, "w5/x.md", 2),
                    lock_case("写者锁:锁缺 nonce 拒", holder_id, "w13/x.md", 2),
                    lock_case("写者锁:锁缺 dispatched_at 拒", holder_id, "w14/x.md", 2),
                    lock_case("写者锁:锁缺 prompt_sha256 拒", holder_id, "w15/x.md", 2),
                    lock_case("写者锁:锁的位置是目录拒", holder_id, "w16/x.md", 2),
                    lock_case("写者锁:主 agent 写坏锁的目录不判", None, "w5/y.md", 0),
                    ("写者锁:判不了的锁的拒绝点名那份锁与出路", True, bad_lock_refusal_names_lock()),
                    lock_case("写者锁:认领文件不是 JSON，真持锁者写也拒", holder_id, "w22/x.md", 2),
                    lock_case("写者锁:认领文件缺 agent_id，真持锁者写也拒", holder_id, "w23/x.md", 2),
                    ("写者锁:认领文件判不了的拒绝点名那份认领文件与出路", True, broken_claim_refusal_names_claim()),
                    ("写者锁:认领文件判不了的不另建一份、不改它", ("unreadable", "unreadable"), (claim_owner("w22"), claim_owner("w23"))),
                    lock_case("写者锁:草稿根自己下面直接写不判", rival_id, "result.md", 0),
                    ("写者锁:回执对不上的拒绝点名还没认领的持锁者的 agent_id", True, foreign_names_expected_holder()),
                    ("写者锁:回执对不上又找不到持锁者的回执时，拒绝写锁登记的提示 sha256 前几位", True, foreign_unfound_names_registration()),
                    # 嵌套的草稿目录：w7 的锁是甲的、w7/sub 的锁是乙的，只认最近一层（writer_lock 文件头「嵌套的草稿目录」；开关 holder-ignored、launch-ignored 下第一格红）
                    lock_case("写者锁:父目录锁的持有者写子目录另派给别人持锁的部分被拒", holder_id, "w7/sub/x.md", 2),
                    lock_case("写者锁:子目录锁的持有者写自己的子目录放行", rival_id, "w7/sub/z.md", 0),
                    lock_case("写者锁:子目录锁的持有者写父目录仍被父目录锁拦", rival_id, "w7/y.md", 2),
                    lock_case("写者锁:不持有任何一层锁的第三者写嵌套的子目录被拦", first_id, "w7/sub/q.md", 2),
                    lock_case("写者锁:父目录锁的持有者写父目录放行（认领父目录）", holder_id, "w7/h.md", 0),
                    lock_case("写者锁:父目录认领之后，认领者写别人持锁的子目录仍被拒", holder_id, "w7/sub/x2.md", 2),
                    lock_case("写者锁:父目录认领之后，子目录锁的持有者写自己的子目录仍放行", rival_id, "w7/sub/z2.md", 0),
                    ("写者锁:草稿根之外不判", 0, decide({"tool_name": "Write", "agent_id": rival_id, "agent_type": "implementation-writer", "transcript_path": lock_transcript,
                                                   "tool_input": {"file_path": f"{work}/crates/b.rs", "content": "x"}}, work, table, library)[0]),
                ]
                # 接手换锁（派发闸用 replace_lock）失败不留临时文件：锁内容写不进 JSON（json.dump 写到一半抛 TypeError）、锁的位置是个非空目录（os.replace 抛 OSError）
                # 两种失败各一格，另有一格成功的对照；LIB_WRITER_LOCK_BREAK=replace-temp-kept 下前两格红
                def replace_lock_outcome(name, obstacle):
                    scratch = os.path.join(lock_root, name)
                    os.makedirs(scratch)
                    if obstacle == "directory-in-the-way":
                        os.makedirs(os.path.join(scratch, write_guard_decisions.writer_lock.LOCK_NAME, "keep"))
                    new = write_guard_decisions.writer_lock.new_lock("self-test", prompt_one, "implementation-writer", lock_transcript)
                    if obstacle == "unserializable":
                        new["unserializable"] = object()
                    try:
                        write_guard_decisions.writer_lock.replace_lock(scratch, new)
                        raised = False
                    except (TypeError, OSError):
                        raised = True
                    return raised, sorted(item for item in os.listdir(scratch) if item.endswith(".tmp"))
                cases += [
                    ("写者锁:换锁时锁内容写不进 JSON，抛异常且不留临时文件", (True, []), replace_lock_outcome("w10", "unserializable")),
                    ("写者锁:换锁时锁的位置被目录占着，抛异常且不留临时文件", (True, []), replace_lock_outcome("w11", "directory-in-the-way")),
                    ("写者锁:换锁成功不抛异常也不留临时文件", (False, []), replace_lock_outcome("w12", "none")),
                ]
                # 排他建锁（派发闸用 create_lock）、建认领（try_claim）共用 write_exclusively：内容写不进 JSON（json.dump 写到一半抛 TypeError）时抛异常、
                # 不留临时文件，锁与认领也不落下；LIB_WRITER_LOCK_BREAK=exclusive-temp-kept 下这两格红
                def exclusive_write_outcome(name, which):
                    scratch = os.path.join(lock_root, name)
                    os.makedirs(scratch)
                    new = write_guard_decisions.writer_lock.new_lock("self-test", prompt_one, "implementation-writer", lock_transcript)
                    try:
                        if which == "lock":
                            new["unserializable"] = object()
                            write_guard_decisions.writer_lock.create_lock(scratch, new)
                        else:
                            write_guard_decisions.writer_lock.create_lock(scratch, new)
                            write_guard_decisions.writer_lock.try_claim(scratch, new, object())
                        raised = False
                    except TypeError:
                        raised = True
                    landed = write_guard_decisions.writer_lock.lock_path(scratch) if which == "lock" else write_guard_decisions.writer_lock.claim_path(scratch, new)
                    return raised, sorted(item for item in os.listdir(scratch) if item.endswith(write_guard_decisions.writer_lock.TEMPORARY_SUFFIX)), os.path.lexists(landed)
                cases += [
                    ("写者锁:排他建锁时锁内容写不进 JSON，抛异常、不留临时文件也不留锁", (True, [], False), exclusive_write_outcome("w19", "lock")),
                    ("写者锁:认领时认领内容写不进 JSON，抛异常、不留临时文件也不留认领", (True, [], False), exclusive_write_outcome("w20", "claim")),
                ]
                # 两次接手交错：A 换上锁之后、清理旧认领之前，B 换上自己的锁、B 的子 agent 认领（replace_lock 的 after_replace 造这一刻），A 再清理。
                # 改后 A 只删换锁之前列过的认领：B 的认领还在、第三个子 agent 认领不成，别的进程正在写的认领临时文件也还在；
                # LIB_WRITER_LOCK_BREAK=takeover-sweeps-new-claims（换上之后删掉所有 nonce 不同的认领）下这两格红：
                #     「✗ 自检：写者锁:两次接手交错（A 换锁、B 换锁并认领、A 清理）之后 B 的认领还在，第三个认领不成，旧认领删掉」
                #     「✗ 自检：写者锁:接手不删别的进程正在写的认领临时文件」

                def interleaved_takeover_outcome(name):
                    scratch = os.path.join(lock_root, name)
                    old = write_guard_decisions.writer_lock.new_lock("self-test", prompt_one, "implementation-writer", lock_transcript)
                    write_guard_decisions.writer_lock.create_lock(scratch, old)
                    write_guard_decisions.writer_lock.try_claim(scratch, old, holder_id)   # 换锁之前就在的旧认领
                    in_flight = os.path.join(scratch, f"{write_guard_decisions.writer_lock.CLAIM_PREFIX}0123456789abcdef.4242.abcd{write_guard_decisions.writer_lock.TEMPORARY_SUFFIX}")
                    open(in_flight, "w", encoding="utf-8").close()   # 别的进程正在写、还没链上的认领临时文件
                    lock_a = write_guard_decisions.writer_lock.new_lock("self-test", prompt_other, "implementation-writer", lock_transcript)
                    lock_b = write_guard_decisions.writer_lock.new_lock("self-test", prompt_three, "implementation-writer", lock_transcript)
                    def b_takes_over_and_claims():
                        write_guard_decisions.writer_lock.replace_lock(scratch, lock_b)
                        write_guard_decisions.writer_lock.try_claim(scratch, lock_b, rival_id)
                    write_guard_decisions.writer_lock.replace_lock(scratch, lock_a, after_replace=b_takes_over_and_claims)
                    state, current = write_guard_decisions.writer_lock.lock_state(scratch)
                    survived = claim_owner(name)
                    third = write_guard_decisions.writer_lock.try_claim(scratch, lock_b, zombie_id)
                    return ((state == "ok" and current["nonce"] == lock_b["nonce"], survived, third, os.path.exists(write_guard_decisions.writer_lock.claim_path(scratch, old))),
                            os.path.exists(in_flight))
                takeover_claims, in_flight_kept = interleaved_takeover_outcome("w21")
                cases += [
                    ("写者锁:两次接手交错（A 换锁、B 换锁并认领、A 清理）之后 B 的认领还在，第三个认领不成，旧认领删掉", (True, rival_id, rival_id, False), takeover_claims),
                    ("写者锁:接手不删别的进程正在写的认领临时文件", True, in_flight_kept),
                ]
                # 判不了照常放行而记一处没判全：读不到库、判法自己出错（LIB_WRITER_LOCK_BREAK 设成认不出的值，find_lock_directory 抛 ValueError）；
                # 开关 WRITE_GUARD_BREAK=unjudged-not-recorded 下这两格与下面 stdin 那一格红
                def unjudged_after(prepare, restore, relative):
                    import contextlib, io
                    del UNJUDGED[:]
                    prepare()
                    try:
                        with contextlib.redirect_stderr(io.StringIO()):
                            code = decide(lock_hook_input(rival_id, relative), work, table, library)[0]
                    finally:
                        restore()
                    notes = list(UNJUDGED)
                    del UNJUDGED[:]
                    return code, notes
                saved_library, saved_lock_break = write_guard_decisions.writer_lock, os.environ.get("LIB_WRITER_LOCK_BREAK")
                def restore_lock_break():
                    if saved_lock_break is None:
                        os.environ.pop("LIB_WRITER_LOCK_BREAK", None)
                    else:
                        os.environ["LIB_WRITER_LOCK_BREAK"] = saved_lock_break
                def drop_writer_lock_library():
                    write_guard_decisions.writer_lock = None
                def restore_writer_lock_library():
                    write_guard_decisions.writer_lock = saved_library
                code, notes = unjudged_after(drop_writer_lock_library, restore_writer_lock_library, "w1/u.md")
                cases.append(("写者锁:读不到库时记一处没判全", (0, True), (code, any("读不到 lib_writer_lock.py" in note for note in notes))))
                code, notes = unjudged_after(lambda: os.environ.__setitem__("LIB_WRITER_LOCK_BREAK", "selftest-unknown-mode"), restore_lock_break, "w2/u.md")
                cases.append(("写者锁:自己出错时记一处没判全", (0, True), (code, any("写者锁这一道自己出错" in note for note in notes))))
                lock_environment = dict(environment, SINGLEFS_WORK_ROOT=lock_root)
                def run_lock_entry(agent_id, relative):
                    return subprocess.run(["bash", script], input=json.dumps(lock_hook_input(agent_id, relative)), capture_output=True, text=True, env=lock_environment)
                second_entry = run_lock_entry(rival_id, "w1/g.md")
                cases.append(("stdin:第二个写者被拒（退出码 2）", 2, second_entry.returncode))
                cases.append(("stdin:第二个写者的拒绝点名持锁者、锁的目录与出路", True,
                              all(fragment in second_entry.stderr for fragment in (holder_id, os.path.join(lock_root, "w1"), "→ 怎么办"))))
                cases.append(("stdin:持锁者放行", 0, run_lock_entry(holder_id, "w1/h.md").returncode))
                foreign_entry = run_lock_entry(rival_id, "w6/h.md")
                cases.append(("stdin:回执对不上的拒绝（退出码 2）", 2, foreign_entry.returncode))
                cases.append(("stdin:回执对不上的拒绝点名持锁者的 agent_id 与出路", True,
                              all(fragment in foreign_entry.stderr for fragment in (holder_id, os.path.join(lock_root, "w6"), "→ 怎么办"))))
                cases.append(("stdin:父目录锁的持有者写别人持锁的子目录被拒（退出码 2）", 2, run_lock_entry(holder_id, "w7/sub/h2.md").returncode))
                bad_lock_entry = run_lock_entry(rival_id, "w5/h.md")
                cases.append(("stdin:坏锁拒绝（退出码 2）", 2, bad_lock_entry.returncode))
                cases.append(("stdin:坏锁的拒绝点名那份锁与出路", True,
                              all(fragment in bad_lock_entry.stderr for fragment in (os.path.join(lock_root, "w5", write_guard_decisions.writer_lock.LOCK_NAME), "→ 怎么办", "→ 主 agent"))))
                broken_claim_entry = run_lock_entry(holder_id, "w22/h.md")
                broken_claim_file = write_guard_decisions.writer_lock.claim_path(os.path.join(lock_root, "w22"), write_guard_decisions.writer_lock.lock_state(os.path.join(lock_root, "w22"))[1])
                cases.append(("stdin:坏认领文件拒绝并点名那份认领文件（退出码 2）", (2, True),
                              (broken_claim_entry.returncode, all(fragment in broken_claim_entry.stderr for fragment in (broken_claim_file, "→ 怎么办", "→ 主 agent")))))
                # 回执还没落盘：经钩子入口写，放行、不认领、stderr 一行、检出一条（结局只记）；开关 missing-receipt-claims 下红
                make_lock("w18", prompt_three)
                pending_detections = os.path.join(lock_root, "pending-detections.jsonl")
                pending_entry = subprocess.run(["bash", script], input=json.dumps(lock_hook_input("a0000000000000ab9", "w18/h.md")), capture_output=True, text=True,
                                               env=dict(lock_environment, AGENT_HOOK_DETECTIONS=pending_detections))
                pending_records = [json.loads(line) for line in open(pending_detections, encoding="utf-8")] if os.path.exists(pending_detections) else []
                cases.append(("stdin:回执还没落盘的写放行、不认领，stderr 一行并记一条检出（结局只记）", (0, "absent", True, 1, "只记", True),
                              (pending_entry.returncode, claim_owner("w18"), "回执还没落盘，没认领" in pending_entry.stderr, len(pending_records),
                               pending_records[-1].get("outcome") if pending_records else None,
                               bool(pending_records) and any(str(finding).startswith("写闸没判全：写者锁这一道") and "回执还没落盘，没认领" in str(finding)
                                                             for finding in pending_records[-1].get("findings") or []))))
                lock_error_detections = os.path.join(lock_root, "lock-error-detections.jsonl")
                lock_error_entry = subprocess.run(["bash", script], input=json.dumps(lock_hook_input(rival_id, "w2/h.md")), capture_output=True, text=True,
                                                  env=dict(lock_environment, LIB_WRITER_LOCK_BREAK="selftest-unknown-mode", AGENT_HOOK_DETECTIONS=lock_error_detections))
                lock_error_records = [json.loads(line) for line in open(lock_error_detections, encoding="utf-8")] if os.path.exists(lock_error_detections) else []
                cases.append(("stdin:写者锁自己出错放行并记一条检出（结局只记）", (0, 1, "只记", True),
                              (lock_error_entry.returncode, len(lock_error_records), lock_error_records[-1].get("outcome") if lock_error_records else None,
                               bool(lock_error_records) and any(str(finding).startswith("写闸没判全：写者锁这一道自己出错") for finding in lock_error_records[-1].get("findings") or []))))
        finally:
            if work_root_before is None:
                os.environ.pop("SINGLEFS_WORK_ROOT", None)
            else:
                os.environ["SINGLEFS_WORK_ROOT"] = work_root_before
            shutil.rmtree(lock_root, ignore_errors=True)
        os.unlink(outside.name)
    finally:
        shutil.rmtree(work)
        shutil.rmtree(outside_directory, ignore_errors=True)
    # 第六道判不了照常放行而记一处没判全：Edit 一份在而读不了（chmod 成 000）的脚本，改完之后的内容算不出；开关 WRITE_GUARD_BREAK=syntax-unreadable-unrecorded 下这一格红
    import contextlib, io
    locked_root = tempfile.mkdtemp(prefix="write-guard-locked-")
    try:
        os.makedirs(os.path.join(locked_root, "scripts"))
        locked_script = os.path.join(locked_root, "scripts", "locked.sh")
        with open(locked_script, "w", encoding="utf-8") as handle:
            handle.write("echo a\n")
        os.chmod(locked_script, 0)
        del UNJUDGED[:]
        with contextlib.redirect_stderr(io.StringIO()):
            code = decide_shell_syntax({"tool_name": "Edit", "tool_input": {"file_path": locked_script, "old_string": "echo a", "new_string": "echo b"}}, locked_root)[0]
        notes = list(UNJUDGED)
        del UNJUDGED[:]
        cases.append(("语法:改一份读不了的脚本照常放行、记一处没判全", (0, True), (code, any("读不了" in note and "语法这一次没判" in note for note in notes))))
    finally:
        for directory, _, names in os.walk(locked_root):
            for name in names:
                os.chmod(os.path.join(directory, name), 0o600)
        shutil.rmtree(locked_root, ignore_errors=True)
    failures = [item for item in cases if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide_overwrite() / decide_compile_first() / decide_trailing_blank() / decide_shell_syntax() / decide_scope() / decide_prime_marks() / decide_clock_times() / decide_session_names() / decide_batch_codes() / decide_task_and_session_numbers() / decide_writer_lock() / decide_ledger_append() / decide_script_swap_only() 与入口；"
              "WRITE_GUARD_DISABLE_WRITER_LOCK / LIB_WRITER_LOCK_BREAK / WRITE_GUARD_DISABLE_OVERWRITE / WRITE_GUARD_DISABLE_COMPILE_FIRST / WRITE_GUARD_DISABLE_SYNTAX / WRITE_GUARD_DISABLE_SCOPE / WRITE_GUARD_DISABLE_CLOCK_TIMES / WRITE_GUARD_DISABLE_SESSION_NAMES、"
              "WRITE_GUARD_BREAK=trailing-blank-ignored / scope-root-unanchored / builtin-agents-unscoped / python-syntax-subprocess / library-reloaded-each-time / landing-lexical-only / unparsable-input-silent / unjudged-not-recorded / ledger-append-ignored / script-in-place-allowed / swap-manifests-unseen / swap-remedy-always-verify / gate-stage-remedy-as-script / task-and-session-numbers-unjudged 或 SINGLEFS_WORK_ROOT_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {len(cases)} 种情形）：钩子、门禁、项目脚本、研究脚本目录下写进语法错误的 .sh（Write 与 Edit）与 .py 拒绝、语法对的与别处的放行；"
          "未跟踪的已有文件整份覆盖拒绝，已跟踪 / 不存在 / 仓外 / Edit 放行；主 agent 放行、没有项目定义的 agent（内置类型、定义已删的旧实例）只许写工作根（仓里与仓外别处拒）、范围内放行、"
          "工作根下的报告目录放行、会话临时目录根下（scratchpad 内外）与工作根同名前缀的兄弟目录拒绝，范围外（Edit、Write、MultiEdit）与 .. 绕路与未登记的项目 agent 拒绝，按解开符号链接之后的落点判（范围内的链接指到范围外、经它写不存在的子目录、悬空链接、指到仓里范围外的拒，指到范围内与范围外指回范围内的放行；覆盖与先编后换经链接同样认得出）；experiment-runner 写主工作区 crates/ 下的 .rs（改、新建、.. 绕路）按先编后换拒绝并点名 compile-then-swap.py，"
          "implementation-writer 与主 agent 写同一处、执行员写草稿目录与 crates/mutations.tsv 放行；Write 内容、Edit 与 MultiEdit 的 new_string 里有撇号类角标（五个字符各一例，主 agent 与子 agent 一样）拒绝，"
          "只在 old_string 里有、ASCII 单引号放行，共用字符集读不到拒绝；写进仓里的内容带钟点（日期后、时区词旁、x 通配、日期或时区词后的「N 点 / N 时」与时段词、时段词后的钟点、光秃的「N 点前后」、"
          "不挨日期时区的时分与区间、ISO 与 diff 头 stat 的机器时间、ISO 紧凑时间戳（日期连写或带连字符加连写的钟点，含路径里的运行目录名，Write、Edit、MultiEdit 各一例）、光秃的时区词、英文十二小时制，"
          f"md 里与没写理由的放行标记不认，主 agent 与子 agent 一样）拒绝并点名形态，8 位数字后跟 T 但不是时间戳的 {len(compact_lookalike_cases)} 例（月份与钟点越界、年不是 20 开头、钟点只有三位、前面挨着字母、小写 t）与只写日期、仓外、.py 里带理由的放行标记放行，"
          f"只写日期、序号与个数后的「点」、不挨日期时区的时段词、标识符里的时区词、以 am / pm 起头的英文词、{len(context_cases)} 类长得像钟点的冒号对、"
          f".sh / .py / .rs 里带理由的放行标记 {len(allow_cases)} 例、仓外、排除的样本、hook 自检与 c510 提示、只在 old_string 里有放行，判据读不到拒绝；写进仓里的会话号（标签式、当主语、上游三语仓、长十六进制）拒绝，crate 名、上游仓目录名、临时目录名与 .sh 里带理由的放行标记放行；"
          "目标不是 .md / .mdx 时 Edit 与 MultiEdit 的 old_string 以空格或制表符收尾（.sh、.txt、.markdown，仓里仓外、主 agent 与子 agent 一样）拒绝并给出收窄框选与 replace-once.py 两条出路，"
          ".md / .mdx / .MD 目标、不以空白收尾、只在中间行或换行前有行尾空白、只有 new_string 以空白收尾与 Write 放行；"
          "钟点与会话号逐处列出（行号与总数，Edit 折算成文件里的行号），几道都不成立的一次列全；拒绝都记进检出记录；输入不是合法的 JSON 对象照常放行、stderr 写诊断并记一条只记的检出（会话号读不到写「读不到」，按字面找得到的照抄）；"
          "草稿目录写者锁：提示对不上与派发早于锁的旧进程不能认领，持锁者写与续写（含子目录）放行，第二个写者（含子目录）拒绝并点名持锁者与出路，回执对不上的拒绝点名记录里找到的持锁者（找不到写锁登记的提示 sha256 前几位），"
          "嵌套的子目录另派给别人持锁时，父目录锁的持有者写那个子目录、子目录锁的持有者写父目录与不持有任何一层的第三者都按最近一层的锁拦，"
          "派发回执还没落盘的写（先写的非持锁者、真持锁者自己的第一次写）放行、不认领并记一处没判全（经入口时 stderr 一行、检出一条），回执落盘之后持锁者写才认领，认领之后没回执的与同一份提示的另一次派发被拒，"
          "锁文件在而判不了（不是 JSON、缺 nonce / dispatched_at / prompt_sha256、锁的位置是目录）的拒绝并点名那份锁与出路，认领文件在而判不了（不是 JSON、缺 agent_id）的真持锁者写也拒、点名那份认领文件与出路，"
          "主 agent、没有锁的目录、草稿根自己下面与草稿根之外不判，读不到库与判法自己出错照常放行并记一处没判全；接手换锁时锁内容写不进 JSON、锁的位置被目录占着都不留临时文件，"
          "排他建锁与建认领时内容写不进 JSON 抛异常、不留临时文件也不落下，两次接手交错之后后一次接手的认领还在、第三个认领不成、别的进程正在写的认领临时文件不删；"
          ".py 的语法在本进程里判、不起子进程，共用判据库一次写入只装一次；"
          "往现行账本（拆分提案账本登记表里 current 那一份）追加「## <节号>、」标题（Edit、MultiEdit、Write 各一例）拒绝并点名 ledger-append-section.py 与出路，改节号、改正文、封存的账本、别的文件放行，"
          "登记表读不到时放行并记一处没判全；"
          "工具员用 Write、Edit、MultiEdit 改主工作区 scripts/、.claude/gate.d/、.claude/scripts/、task-server/、prune-server/、gpu-verification-server/ 下已存在的 .sh、.py 与带执行位无后缀的脚本、两个服务 src/ 下已存在的文件与两个服务已存在的 Cargo.toml、Cargo.lock 拒绝，脚本的出路按要临时服务的自证登记表分两种"
          "（经入口的两例点名 hook-swap.py、「要主 agent 跑的命令」与草稿目录），新建、不带执行位的 .tsv / .md / .json / .txt、fixtures 下的样本、主工作区之外（草稿目录与仓副本）、实现员与主 agent 放行，"
          "认不出 agent 类型与 stat 失败的照常放行并记一处没判全；"
          f"写进仓里 .md、.tsv 的批次代号（数字加字母加短横序号、两位数字加字母、挨着批次词的、天干编号的批号）拒绝，{len(batch_cases) + len(batch_context_cases)} 格里"
          "两位会话短号、单位与倍数、路径与标识符、不带批次词的候选名、.py 与 .sh、变异表、仓外与反引号、字段值、算式、引号里的放行；"
          f"写进仓里 .md、.tsv 的任务号（括号里的、同字与与字后面并列的、带冒号与井号的、斜杠短横连着的、英文 task 与 task=、反引号与围栏里的、"
          "不在行首或不在判决里的 A/B 读数一行、kb 里的长跑拆分一行、Edit 写进的）、括号里的会话短号（数字加字母、挨着会话词、括号里跟会话词、并列，"
          "Edit 与 MultiEdit 以收尾围栏起头而插入点前围栏开着的）、批号（正文里的、别的会话的、第几批、批井号、区间与并列、跟副词只的、kb 里表格行的）"
          "与拿描述指别的会话的（调度记录里的三句「那一路会话」、那一路的会话与那一路的 agent）拒绝，"
          f"{len(numbers_cases) + len(numbers_context_cases)} 格里编号与步号、数量、任务个数、中文数字的第几批、不带括号的短号与计数、日期、不到四位的号、锚点、照出路改写的命令与结局行、"
          ".py、只在 old_string 里有、门禁样本、仓外、反引号里的会话短号与批号、判决里行首的 A/B 读数一行（两种派发闸写法）、记录里的长跑拆分一行与表格行里的批号、"
          "kb 里括号里的步号、fsync 攒批的大小、围栏里的会话短号与批号、插入点前围栏已收的放行、"
          "这一路、那一路径、一路绿、不跟会话词的那一路、mainagent(A) 与 subagent(A) 这类按出场次序的叫法、反引号与围栏里拿描述指别的会话的放行")
    return 0

# 弄坏开关与关闭开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（WRITE_GUARD_SELFTEST_MARK 对得上）而下面这些变量有一个非空，
# 读 hook 的 JSON 之前就退 2、点名变量只在自证里认，不管值是什么（这道钩子退 2 是拒，所以外面漏进来的开关不会悄悄放宽写范围）。
# 自证起的子进程的标记（SELFTEST_MARK_VARIABLE、SELFTEST_MARK_CONTENT）定义在入口的引导里：引导在这几份载入之前就要认它。
SELFTEST_ONLY_VARIABLES = ("WRITE_GUARD_BREAK", "LIB_WRITER_LOCK_BREAK", "WRITE_GUARD_DISABLE_OVERWRITE", "WRITE_GUARD_DISABLE_COMPILE_FIRST", "WRITE_GUARD_DISABLE_SCOPE",
                           "WRITE_GUARD_DISABLE_CLOCK_TIMES", "WRITE_GUARD_DISABLE_SESSION_NAMES", "WRITE_GUARD_DISABLE_SYNTAX", "WRITE_GUARD_DISABLE_WRITER_LOCK")

def refuse_break_outside_selftest(arguments):
    set_names = [name for name in SELFTEST_ONLY_VARIABLES if os.environ.get(name)]
    if not set_names or arguments == ["--selftest"] or os.environ.get(SELFTEST_MARK_VARIABLE) == SELFTEST_MARK_CONTENT:
        return None
    for name in set_names:
        print(f"  ✗ write-guard.sh：{name}={os.environ[name]} 是弄坏开关（只给自证用），只在自证（write-guard.sh --selftest）里认，平常跑设了就拒，这一次写没判", file=sys.stderr)
    print(f"     → 怎么办：去掉 {'、'.join(set_names)} 再跑；要证红就跑 {set_names[0]}=<项> bash .claude/hooks/write-guard.sh --selftest", file=sys.stderr)
    return 2


def main():
    """入口的引导在核完部分文件之后调：读 hook 输入、判、交退出码。"""
    hook_dir = sys.argv[1]
    refused = refuse_break_outside_selftest(sys.argv[2:])
    if refused is not None:
        return refused
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    detections_path = os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS
    raw = sys.stdin.buffer.read()
    try:
        hook_input = json.loads(raw.decode("utf-8"))
        if not isinstance(hook_input, dict):
            raise ValueError(f"顶层是 {type(hook_input).__name__}，不是对象")
    except ValueError as error:   # 判不了：照常放行，打诊断并记一条检出（弄坏开关 unparsable-input-silent 下静默放行）
        if os.environ.get("WRITE_GUARD_BREAK") != "unparsable-input-silent":
            identity, text = unparsable_input_entry(raw, f"{type(error).__name__}: {error}")
            note_unjudged(text)
            record_detection(identity, [text], detections_path, "只记")
        return 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or hook_input.get("cwd") or os.getcwd()
    try:
        code, message, finding = decide(hook_input, project_root, os.path.join(hook_dir, "agent-write-scope.tsv"), notations_library_path(hook_dir))
    except Exception as error:   # 判定自己抛异常：照常放行，打诊断并记检出
        note_unjudged(f"判定自己抛异常，这一次放行：{type(error).__name__}: {error}")
        code, message, finding = 0, None, None
    if code:
        print(message, file=sys.stderr)
        record_detection(hook_input, finding, detections_path)
    if UNJUDGED:
        record_detection(hook_input, UNJUDGED, detections_path, "只记")
    return code
