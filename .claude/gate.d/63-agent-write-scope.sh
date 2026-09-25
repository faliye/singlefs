#!/usr/bin/env bash
# gate-stage: PreToolUse 七个钩子、SessionStart 的压缩后提示与启动时的 OOM 报告、PostToolUse 的书记官写入后核对注册着、八个项目 hook 自证通过、写范围表与定义双向一致、每个定义带 omitClaudeMd 与「开工先读：」、共用切词模块的函数不在别的 hook 里另写一份
#
# 判据，任一条不成立判红：
#   ① `.claude/settings.json` 的 PreToolUse 里有一条 matcher 同时覆盖 Write 与 Edit、命令指向 `write-guard.sh` 的 hook（写范围与整份覆盖未跟踪文件两道合在里面）；
#   ② `.claude/hooks/write-guard.sh --selftest` 通过（自证里有走真实 stdin 入口的情形）；
#   ③ `.claude/agents/` 里 tools 含 Write 或 Edit 的每个定义，在 `.claude/hooks/agent-write-scope.tsv` 里至少有一条路径模式；
#   ④ 表里每个 agent 名都有定义，每行至少两列；
#   ⑤ PreToolUse 里有一条 matcher 覆盖 Bash、命令指向 `bash-command-detector.sh` 的 hook，且它的 `--selftest` 通过（没超时的等待循环、run_in_background 里又自己放后台记进检出记录、放行：在跑的命令结不结束由主 agent 判断；起看门狗的错误写法——不用 run_in_background、带 `&` / nohup / setsid / disown、输出丢进 /dev/null——执行前拒绝）；
#   ⑥ PreToolUse 里有一条 matcher 覆盖 Agent、命令指向 `runner-dispatch-guard.sh` 的 hook，且它的 `--selftest` 通过（派执行员没点名岔路、续做没写还差的行会被拒；派发提示要子 agent 跑重型测试会被拒；派 kb-scribe、implementation-writer 时要改的文件落在还没判完的三方轮开工快照里会被拒）；
#   ⑦ PreToolUse 里有一条 matcher 覆盖 SendMessage、命令指向 `continuation-guard.sh` 的 hook，且它的 `--selftest` 通过（给最近一次任务通知是 failed 或 killed 的、或已经交回过的子 agent 续做会被拒，上下文多大不拦）；
#   ⑧ `.claude/agents/` 里每个定义的 frontmatter 有 `omitClaudeMd: true`，正文有一行以「开工先读：」开头（不继承 CLAUDE.md 之后，要读的规则全靠这一行点名）；
#   ⑨ PreToolUse 里有一条 matcher 覆盖 Bash、命令指向上游 SOP 的 `claude-hooks/pattern-process-guard.sh` 的 hook，且那个文件在
#     （按模式找进程在执行前拒绝；它的判别力由上游 selftest 的样本管，门禁「门禁自检」阶段跑它，这里只查注册着、文件在）；
#   ⑪ PostToolUse 里有一条 matcher 同时覆盖 Write、Edit、Bash、命令指向 `kb-scribe-followup.sh` 的 hook，且它的 `--selftest` 通过（书记官写 kb 之后按 kb-scribe-followups.tsv 核相关记录跟没跟上）。
#   ⑩ SessionStart 里有一条 matcher 覆盖 compact、一条 matcher 同时覆盖 startup 与 resume（可以是同一条），命令都指向 `session-start.sh`，且 `.claude/hooks/session-start.sh --selftest` 通过
#     （压缩上下文之后补回分支、未提交数、最近的记录与看门狗命令；会话启动或恢复时报近几小时内核因为内存不够杀过的进程，读不到内核日志报「没查成」）。
#   ⑫ PreToolUse 里有一条 matcher 覆盖 Bash、命令指向 `heavy-test-guard.sh` 的 hook，且它的 `--selftest` 通过（重型测试——层 0、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置——子 agent 越出自己那一份就拒，主 agent 与 crash-verifier、gate-triage 不带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request` 也拒）。
#   ⑬ 两份共用模块顶层定义的每个函数名，只许在它自己那一份里定义：`.claude/hooks/` 下别的文件里再 `def <名字>(` 一份就判红——
#     切词模块 `.claude/hooks/lib_shell_words.py`（判 Bash 命令的 hook 手抄一份切词，一份改了另一份不跟，同一条命令一个 hook 认得出、另一个认不出），
#     重型测试判定模块 `.claude/hooks/lib_heavy_tests.py`（heavy-test-guard.sh 与看门狗 research/scripts/agent-watch.py 共用 classify、classify_process、
#     judged_by_name 这些判定，手抄一份就是执行前的闸与进程这一层判得不一样）；名字从模块现读，读不到或一个函数都没有也判红。
#     不算的名字登记在 NOT_SHARED_JUDGMENT（每个 hook 都有自己的 selftest 入口，导入样板 load_sibling_module），登记了而模块里没有这个函数也判红。
#   ⑭ PreToolUse 里有一条 matcher 覆盖 AskUserQuestion、命令指向 `ask-user-claim-guard.sh` 的 hook，且它的 `--selftest` 通过（弹窗问句与选项说明里
#     带断言词——不可能、造不出、从来不、一定、恒为这类——的句子，同一句里既没有出处也没写「推的，没量过」就拒；自证里有走真实 stdin 入口的情形）。
# ③ ④ 的双向比对与读表用共用库 lib-manifest.py，与 50、62、98 号同一份代码。
# 判别力：fixtures/63-agent-write-scope.sh/red 的 .claude/hooks/ 里放一份手抄 shell_tokens 的 hook 与一份手抄 classify_process 的 hook，必须各报出文件与行号；
# green 的 .claude/hooks/ 里放两份从共用模块导入、只写自己判定的 hook（连同被判仓自己那两份模块），必须判绿，成功行数进去的别的文件数要对。
# 自证跑的是这份脚本旁边 ../hooks/ 下的 hook（取自 $0 的目录，不取项目根）：样本仓里放坏的 hook 碰不到它。
#
# 为什么：执行类 agent 越界写，靠定义里一句「只写写范围」拦不住；hook 被删、自证坏了、新加一个能写文件的定义忘了登记，
# 这道闸都会静默消失或静默放行——只有门禁会在它消失时说话（与 46 号同一个道理）。
# 2026-09-17 写 hook 时实测撞过一次静默放行：程序从标准输入读，hook 的 JSON 读不到，判定一律放行，而直接调判定函数的自证全绿。
#
#   bash .claude/gate.d/63-agent-write-scope.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
HOOK="$(cd "$(dirname "$0")/../hooks" && pwd)/write-guard.sh"
cd "$ROOT" 2>/dev/null || exit 2
table_output="$(python3 - "$(cd "$(dirname "$0")" && pwd)/lib-manifest.py" "$(dirname "$HOOK")/lib_shell_words.py" "$(dirname "$HOOK")/lib_heavy_tests.py" <<'PY'
import ast, glob, importlib.util, json, os, re, sys
try:
    spec = importlib.util.spec_from_file_location("lib_manifest", sys.argv[1])
    manifest = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(manifest)
except OSError as error:
    print(f"  ✗ 读不到共用库 {sys.argv[1]}：{error}")
    print("     → 怎么办：目录与清单的双向比对只有那一份，恢复它，别在阶段里再抄一份。")
    sys.exit(1)
failed = False
settings_path = ".claude/settings.json"
def matcher_covers(matcher, tool):
    if matcher in ("*", ""):
        return True
    if re.fullmatch(r"[A-Za-z0-9_|]+", matcher):
        return tool in matcher.split("|")
    try:
        return re.search(matcher, tool) is not None
    except re.error:
        return False
try:
    entries = (json.load(open(settings_path, encoding="utf-8")).get("hooks") or {}).get("PreToolUse") or []
except Exception as error:
    entries = None
    failed = True
    print(f"  ✗ 读不了 {settings_path}：{error}")
    print("     → 怎么办：修好这份 JSON，再在 hooks.PreToolUse 里注册写范围闸。")
if entries is not None:
    registered = [entry for entry in entries
                  if matcher_covers(entry.get("matcher", ""), "Write") and matcher_covers(entry.get("matcher", ""), "Edit")
                  and any("write-guard.sh" in (hook.get("command") or "") for hook in entry.get("hooks") or [])]
    if not registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册写范围闸：PreToolUse 里没有 matcher 同时覆盖 Write 与 Edit、命令指向 write-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "Write|Edit"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/write-guard.sh"。')
    guard_registered = [entry for entry in entries
                        if matcher_covers(entry.get("matcher", ""), "Bash")
                        and any("bash-command-detector.sh" in (hook.get("command") or "") for hook in entry.get("hooks") or [])]
    if not guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册 Bash 检出 hook：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 bash-command-detector.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "Bash"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/bash-command-detector.sh"。')
    heavy_registered = [entry for entry in entries
                        if matcher_covers(entry.get("matcher", ""), "Bash")
                        and any("heavy-test-guard.sh" in (hook.get("command") or "") for hook in entry.get("hooks") or [])]
    if not heavy_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册重型测试闸：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 heavy-test-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 的 Bash 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/heavy-test-guard.sh"。')
    dispatch_registered = [entry for entry in entries
                           if matcher_covers(entry.get("matcher", ""), "Agent")
                           and any("runner-dispatch-guard.sh" in (hook.get("command") or "") for hook in entry.get("hooks") or [])]
    if not dispatch_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册续派闸：PreToolUse 里没有 matcher 覆盖 Agent、命令指向 runner-dispatch-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "Agent|Task"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/runner-dispatch-guard.sh"。')
    continuation_registered = [entry for entry in entries
                               if matcher_covers(entry.get("matcher", ""), "SendMessage")
                               and any("continuation-guard.sh" in (hook.get("command") or "") for hook in entry.get("hooks") or [])]
    if not continuation_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册续做闸：PreToolUse 里没有 matcher 覆盖 SendMessage、命令指向 continuation-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "SendMessage"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/continuation-guard.sh"。')
    claim_guard_registered = [entry for entry in entries
                              if matcher_covers(entry.get("matcher", ""), "AskUserQuestion")
                              and any("ask-user-claim-guard.sh" in (hook.get("command") or "") for hook in entry.get("hooks") or [])]
    if not claim_guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册弹窗断言闸：PreToolUse 里没有 matcher 覆盖 AskUserQuestion、命令指向 ask-user-claim-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "AskUserQuestion"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/ask-user-claim-guard.sh"。')
    pattern_guard = "singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh"
    pattern_guard_registered = [entry for entry in entries
                                if matcher_covers(entry.get("matcher", ""), "Bash")
                                and any(pattern_guard in (hook.get("command") or "") for hook in entry.get("hooks") or [])]
    if not pattern_guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册按模式找进程的钩子：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 {pattern_guard} 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 的 Bash 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/' + pattern_guard + '"（写法见 .claude/singlefs-ai-sop/rules/command-safety.md「pkill -f / killall 一律禁用」）。')
    elif not os.path.isfile(os.path.join(".claude", pattern_guard)):
        failed = True
        print(f"  ✗ 注册了按模式找进程的钩子，文件 .claude/{pattern_guard} 却不在：会话里每条 Bash 命令都会报钩子错")
        print("     → 怎么办：规范副本旧于 0.0.52 或没装全，按 CLAUDE.md「规范从哪来」那一行重新同步副本、跑 install.sh。")
try:
    session_entries = (json.load(open(settings_path, encoding="utf-8")).get("hooks") or {}).get("SessionStart") or []
except Exception:
    session_entries = None  # 读不了这份 JSON 的那一条上面已经报过、已判红
if session_entries is not None:
    compact_reminder_registered = [entry for entry in session_entries
                                   if matcher_covers(entry.get("matcher", ""), "compact")
                                   and any("session-start.sh" in (hook.get("command") or "") for hook in entry.get("hooks") or [])]
    if not compact_reminder_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册压缩后提示 hook：SessionStart 里没有 matcher 覆盖 compact、命令指向 session-start.sh 的一条")
        print('     → 怎么办：在 hooks.SessionStart 里加一条 matcher "startup|resume|compact"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/session-start.sh"。')
    oom_report_registered = [entry for entry in session_entries
                             if matcher_covers(entry.get("matcher", ""), "startup") and matcher_covers(entry.get("matcher", ""), "resume")
                             and any("session-start.sh" in (hook.get("command") or "") for hook in entry.get("hooks") or [])]
    if not oom_report_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册启动时的 OOM 报告 hook：SessionStart 里没有 matcher 同时覆盖 startup 与 resume、命令指向 session-start.sh 的一条")
        print('     → 怎么办：在 hooks.SessionStart 里把 session-start.sh 那一条的 matcher 写成 "startup|resume|compact"（会话崩了再起是 startup 或 resume，这时要先看内核 OOM 杀过谁）。')
try:
    post_entries = (json.load(open(settings_path, encoding="utf-8")).get("hooks") or {}).get("PostToolUse") or []
except Exception:
    post_entries = None  # 读不了这份 JSON 的那一条上面已经报过、已判红
if post_entries is not None:
    followup_registered = [entry for entry in post_entries
                           if all(matcher_covers(entry.get("matcher", ""), tool) for tool in ("Write", "Edit", "Bash"))
                           and any("kb-scribe-followup.sh" in (hook.get("command") or "") for hook in entry.get("hooks") or [])]
    if not followup_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册书记官写入后的核对 hook：PostToolUse 里没有 matcher 同时覆盖 Write、Edit、Bash、命令指向 kb-scribe-followup.sh 的一条")
        print('     → 怎么办：在 hooks.PostToolUse 里加一条 matcher "Write|Edit|Bash"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/kb-scribe-followup.sh"。')
table_path = ".claude/hooks/agent-write-scope.tsv"
patterns_by_agent, malformed = {}, []
if os.path.isfile(table_path):
    for line_number, line, fields in manifest.table_rows(table_path):
        if len(fields) < 2 or not fields[0].strip() or not fields[1].strip():
            malformed.append(f"第 {line_number} 行：{line}")
            continue
        patterns_by_agent.setdefault(fields[0].strip(), []).append(fields[1].strip())
else:
    failed = True
    print(f"  ✗ 没有 {table_path}")
    print("     → 怎么办：建这张表：agent 名、路径模式、出处三列，用制表符分隔，按每个能写文件的定义的「写范围」一节转写。")
writers = []
missing_omit, missing_basis = [], []
for path in sorted(glob.glob(".claude/agents/*.md")):
    whole = open(path, encoding="utf-8").read()
    head = whole.split("\n---", 1)[0]
    if not any(line.strip() == "omitClaudeMd: true" for line in head.split("\n")):
        missing_omit.append(os.path.basename(path)[:-3])
    if not any(line.startswith("开工先读：") for line in whole.split("\n")):
        missing_basis.append(os.path.basename(path)[:-3])
    tools_line = next((line for line in head.split("\n") if line.startswith("tools:")), "")
    tools = {tool.strip() for tool in tools_line[len("tools:"):].split(",")}
    if tools & {"Write", "Edit"}:
        writers.append(os.path.basename(path)[:-3])
if malformed:
    failed = True
    print("  ✗ 写范围表里这些行不到两列、或 agent 名与路径模式有空的：")  # gate-lint:summary
    for entry in malformed:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：每行至少写 agent 名与路径模式两列，用制表符分隔。")
# ③ 有 Write 或 Edit 的定义要在表里登记；④ 表里的名字要有定义。两个方向的盘上集合不同，各给各的判定。
unscoped, ghosts = manifest.two_way(writers, list(patterns_by_agent),
                                    exists=lambda name: os.path.isfile(f".claude/agents/{name}.md"))
if unscoped:
    failed = True
    print("  ✗ 这些定义的 tools 里有 Write 或 Edit，写范围表里却没有登记——hook 会把它们的每次写都拒掉：")  # gate-lint:summary
    for name in unscoped:
        print(f"     {name}")  # gate-lint:detail
    print(f"     → 怎么办：照它定义里「写范围」一节，在 {table_path} 给它加路径模式。")
if ghosts:
    failed = True
    print("  ✗ 写范围表里登记的这些 agent 在 .claude/agents/ 里没有定义：")  # gate-lint:summary
    for name in ghosts:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：改成现在的定义名，或删掉这几行。")
if missing_omit:
    failed = True
    print("  ✗ 这些定义的 frontmatter 没有 omitClaudeMd: true——每次派发都白带约 10 万 token 的 CLAUDE.md 与规则：")  # gate-lint:summary
    for name in missing_omit:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：在它的 frontmatter 里加一行 omitClaudeMd: true，开工先读那一句指到 .claude/agent-common.md「规则怎么读」。")
if missing_basis:
    failed = True
    print("  ✗ 这些定义没有「开工先读：」一行——不继承 CLAUDE.md 之后，它要读的规则没有地方点名：")  # gate-lint:summary
    for name in missing_basis:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：在正文里加一行「开工先读：」，点名它干活要读的规则文件与小节。")
# ⑬ 两份共用模块顶层定义的函数名从模块现读（这份阶段旁边 ../hooks/ 下那一份），再扫被判仓 .claude/hooks/ 下它之外的每个文件
# 不算的名字：不是共用的判定，别的 hook 里有同名的一份不算手抄；登记了而模块里没有的判红（不起作用的排除项）
NOT_SHARED_JUDGMENT = {
    "lib_heavy_tests.py": {
        "selftest": "每个 hook 都有自己的 --selftest 入口，同名不是抄的判定",
        "load_sibling_module": "按文件路径导入同目录模块的样板，不是判定",
    },
}
# (模块文件名, 叫法, 这份阶段旁边那一份的路径, 手抄一份会怎样, 该怎么办, 模块里该有什么)
SHARED_MODULES = [
    ("lib_shell_words.py", "共用切词模块", sys.argv[2],
     "手抄的切词会分叉：同一条命令一个 hook 认得出、另一个认不出",
     "从 `.claude/hooks/lib_shell_words.py` 导入，不另写一份（照 bash-command-detector.sh 开头按文件路径 importlib 导入的写法）；那份不够用就改它，再跑两个 hook 的 --selftest。",
     "切词、切简单命令、认命令位置、剥前缀、跟 cd 这几个函数要定义在 lib_shell_words.py 里，两个 hook 从它导入。"),
    ("lib_heavy_tests.py", "共用重型测试判定模块", sys.argv[3],
     "手抄的判定会分叉：执行前的闸 heavy-test-guard.sh 与看门狗 agent-watch.py 对同一条命令判得不一样",
     "从 `.claude/hooks/lib_heavy_tests.py` 导入，不另写一份（照 heavy-test-guard.sh 开头按文件路径 importlib 导入的写法）；那份不够用就改它，再跑它与 heavy-test-guard.sh 的 --selftest。",
     "classify、classify_process、judged_by_name 这些判定要定义在 lib_heavy_tests.py 里，heavy-test-guard.sh 与看门狗从它导入。"),
]
hooks_directory = ".claude/hooks"
shared_counts = []   # [(模块文件名, 查的函数个数, 查了几个别的文件)]
for module_file, module_title, module_path, divergence, fix, required in SHARED_MODULES:
    try:
        module_tree = ast.parse(open(module_path, encoding="utf-8").read(), module_path)
    except (OSError, SyntaxError, ValueError) as error:
        failed = True
        print(f"  ✗ 读不了{module_title} {module_path}：{error}")
        print(f"     → 怎么办：恢复 .claude/hooks/{module_file}；{fix}")
        continue
    function_names = [node.name for node in module_tree.body if isinstance(node, ast.FunctionDef)]
    if not function_names:
        failed = True
        print(f"  ✗ {module_title} {module_path} 里一个顶层函数都没有：这一条没有名字可查")
        print(f"     → 怎么办：{required}")
        continue
    not_judgment = NOT_SHARED_JUDGMENT.get(module_file, {})
    stale_exclusions = sorted(set(not_judgment) - set(function_names))
    if stale_exclusions:
        failed = True
        print(f"  ✗ NOT_SHARED_JUDGMENT 给 {module_file} 登记了模块里没有的函数：{'、'.join(stale_exclusions)}（不起作用的排除项会让人以为那几个名字被绕开了）")
        print("     → 怎么办：从 63 号的 NOT_SHARED_JUDGMENT 里删掉这几项，或者核一下是不是模块里的函数改了名。")
    checked_names = [name for name in function_names if name not in not_judgment]
    module_in_scanned_repo = os.path.normpath(os.path.join(hooks_directory, module_file))
    redefinition = re.compile(r"^[ \t]*def[ \t]+(" + "|".join(map(re.escape, checked_names)) + r")[ \t]*\(", re.M)
    redefinitions, scanned_hook_files = [], 0
    for directory, subdirectories, file_names in os.walk(hooks_directory):
        subdirectories[:] = sorted(name for name in subdirectories if name != "__pycache__")
        for file_name in sorted(file_names):
            path = os.path.normpath(os.path.join(directory, file_name))
            if path == module_in_scanned_repo:
                continue
            try:
                content = open(path, encoding="utf-8").read()
            except (OSError, UnicodeDecodeError):
                continue
            scanned_hook_files += 1
            for match in redefinition.finditer(content):
                redefinitions.append(f"{path}:{content.count(chr(10), 0, match.start()) + 1}  def {match.group(1)}")
    shared_counts.append((module_file, len(checked_names), scanned_hook_files))
    if redefinitions:
        failed = True
        print(f"  ✗ {module_title} .claude/hooks/{module_file} 里的函数，在 .claude/hooks/ 下别的文件里又定义了一份（{divergence}）：")  # gate-lint:summary
        for entry in redefinitions:
            print(f"     {entry}")  # gate-lint:detail
        print(f"     → 怎么办：{fix}")
if failed:
    sys.exit(1)
pattern_count = sum(len(patterns) for patterns in patterns_by_agent.values())
print("TABLE_OK", len(writers), pattern_count, *(count for _, function_count, file_count in shared_counts for count in (function_count, file_count)),
      "、".join(name for names in NOT_SHARED_JUDGMENT.values() for name in names))
PY
)"; table_rc=$?
printf '%s\n' "$table_output" | grep -v '^TABLE_OK '
if [[ $table_rc -ne 0 ]]; then exit 1; fi
read -r _ writer_count pattern_count shared_function_count scanned_hook_file_count heavy_function_count heavy_scanned_file_count not_shared_judgment_names <<<"$(printf '%s\n' "$table_output" | grep '^TABLE_OK ')"
selftest_output="$(bash "$HOOK" --selftest 2>&1)"; selftest_rc=$?
if [[ $selftest_rc -ne 0 ]]; then
  echo "  ✗ write-guard.sh 的自证没过："
  printf '%s\n' "$selftest_output" | sed 's/^/    /'   # gate-lint:detail
  echo "     → 怎么办：修 .claude/hooks/write-guard.sh 的 decide_scope() 或入口，再跑 --selftest 看它转绿。"
  exit 1
fi
guard_output="$(bash "$(dirname "$HOOK")/bash-command-detector.sh" --selftest 2>&1)"; guard_rc=$?
if [[ $guard_rc -ne 0 ]]; then
  echo "  ✗ bash-command-detector.sh 的自证没过："
  printf '%s\n' "$guard_output" | sed 's/^/    /'   # gate-lint:detail
  echo "     → 怎么办：修 .claude/hooks/bash-command-detector.sh 的 findings_for() 或入口，再跑 --selftest 看它转绿。"
  exit 1
fi
dispatch_output="$(bash "$(dirname "$HOOK")/runner-dispatch-guard.sh" --selftest 2>&1)"; dispatch_rc=$?
if [[ $dispatch_rc -ne 0 ]]; then
  echo "  ✗ runner-dispatch-guard.sh 的自证没过："
  printf '%s\n' "$dispatch_output" | sed 's/^/    /'   # gate-lint:detail
  echo "     → 怎么办：修 .claude/hooks/runner-dispatch-guard.sh 的 decide() 或入口，再跑 --selftest 看它转绿。"
  exit 1
fi
continuation_output="$(bash "$(dirname "$HOOK")/continuation-guard.sh" --selftest 2>&1)"; continuation_rc=$?
if [[ $continuation_rc -ne 0 ]]; then
  echo "  ✗ continuation-guard.sh 的自证没过："
  printf '%s\n' "$continuation_output" | sed 's/^/    /'   # gate-lint:detail
  echo "     → 怎么办：修 .claude/hooks/continuation-guard.sh 的 decide() 或入口，再跑 --selftest 看它转绿。"
  exit 1
fi
session_start_output="$(bash "$(dirname "$HOOK")/session-start.sh" --selftest 2>&1)"; session_start_rc=$?
if [[ $session_start_rc -ne 0 ]]; then
  echo "  ✗ session-start.sh 的自证没过："
  printf '%s\n' "$session_start_output" | sed 's/^/    /'   # gate-lint:detail
  echo "     → 怎么办：修 .claude/hooks/session-start.sh 的 reminder() / oom_report() / read_kernel_journal() 或入口，再跑 --selftest 看它转绿。"
  exit 1
fi
followup_output="$(bash "$(dirname "$HOOK")/kb-scribe-followup.sh" --selftest 2>&1)"; followup_rc=$?
if [[ $followup_rc -ne 0 ]]; then
  echo "  ✗ kb-scribe-followup.sh 的自证没过："
  printf '%s\n' "$followup_output" | sed 's/^/    /'   # gate-lint:detail
  echo "     → 怎么办：修 .claude/hooks/kb-scribe-followup.sh 的 decide() / touched_paths() 或登记表 kb-scribe-followups.tsv，再跑 --selftest 看它转绿。"
  exit 1
fi
heavy_output="$(bash "$(dirname "$HOOK")/heavy-test-guard.sh" --selftest 2>&1)"; heavy_rc=$?
if [[ $heavy_rc -ne 0 ]]; then
  echo "  ✗ heavy-test-guard.sh 的自证没过："
  printf '%s\n' "$heavy_output" | sed 's/^/    /'   # gate-lint:detail
  echo "     → 怎么办：修 .claude/hooks/heavy-test-guard.sh 的 heavy_uses() / classify() / refusal_reason() 或入口，再跑 --selftest 看它转绿。"
  exit 1
fi
claim_guard_output="$(bash "$(dirname "$HOOK")/ask-user-claim-guard.sh" --selftest 2>&1)"; claim_guard_rc=$?
if [[ $claim_guard_rc -ne 0 ]]; then
  echo "  ✗ ask-user-claim-guard.sh 的自证没过："
  printf '%s\n' "$claim_guard_output" | sed 's/^/    /'   # gate-lint:detail
  echo "     → 怎么办：修 .claude/hooks/ask-user-claim-guard.sh 的 sentences_of() / without_code_and_quotations() / assertion_spans() / has_provenance() 或入口，再跑 --selftest 看它转绿。"
  exit 1
fi
echo "  ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸与弹窗断言闸注册着、自证通过，按模式找进程的上游钩子注册着、文件在，会话开始 hook（压缩后提示与启动时的 OOM 报告）注册着、自证通过，书记官写入后的核对 hook 注册着、自证通过，表与定义一致（${writer_count} 个有 Write 或 Edit 的定义、${pattern_count} 条路径模式），共用切词模块的 ${shared_function_count} 个函数只在 lib_shell_words.py 里定义（查了 .claude/hooks/ 下另外 ${scanned_hook_file_count} 个文件），共用重型测试判定模块的 ${heavy_function_count} 个函数只在 lib_heavy_tests.py 里定义（查了 .claude/hooks/ 下另外 ${heavy_scanned_file_count} 个文件；${not_shared_judgment_names} 不算，见 NOT_SHARED_JUDGMENT）"
