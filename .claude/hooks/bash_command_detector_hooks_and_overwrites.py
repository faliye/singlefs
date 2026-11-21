# 检出钩子 bash-command-detector.sh 的 python 部分之一：绕开 git 钩子（㉒）、后台任务输出与单独的 &（④）、覆盖产物（⑤）的写法、解包与打补丁与压缩与就地写的工具（⑭）、mktemp 与写者锁与三方腿（⑫）、follow_simple_command 与 overwrite_steps。
# 入口 bash-command-detector.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 DETECTOR_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
import glob
import importlib.util
import json
import os
import re
import shlex
import subprocess
import sys
from typing import NamedTuple
from __main__ import detector_break_is_set
from bash_command_detector_scope_and_index import (
    HOOK_DIRECTORY, INSERT_ROW_OPTIONS_WITH_VALUE, JUDGING_NOTICES, KNOWN_WRITE_SCRIPTS, LAUNCHER_NAMES, LEDGER_APPEND_SCRIPT, LONE_AMPERSAND, LedgerAppendTarget,
    NON_WRITING_FLAGS, POLL_AFTER, PYTHON_SCRIPT_READ_LIMIT_BYTES, RECORD_APPEND_OPTIONS_WITH_VALUE, RECORD_APPEND_SCRIPT, SELF_DETACH,
    SUBSHELL_AMPERSAND, SWAP_SPEC_SCRIPT, UnknownScriptTarget, UnresolvedScriptTarget, WAIT_COMMAND, WAIT_LOOP, append_detection, bare_setsid_forms,
    first_operand, git_location, git_other_form_clears_index, git_reset_clears_index, git_restore_clears_index, home_expanded,
    in_place_target_in_scratch, last_option_value, not_judged, perl_in_place_files, project_code_runs_library, sed_in_place_targets, shell_layers,
    shell_words, staged_index_state, unbounded_wait_loops, unresolved_script_word, unstaged_hits, wait_loops_in, work_roots, worktree_discard_paths,
    wrapped_commands, write_scope,
)

# ㉒ 绕开 git 钩子的 git commit / git push / git merge / git am / git config，与 git 钩子没装上（文件头 ㉒）：提交与推送前的大文件判法在进仓的 git 钩子里（scripts/gate/git-hooks/ 里的各个钩子，
#   判法 scripts/gate/large-file-git-hook.py），这里不读命令文本判内容，只拦两件事：绕开钩子的写法、钩子没装上（装没装上的判法在 lib_git_hooks.py，与安装脚本共用）
GIT_COMMIT_SHORT_OPTIONS_WITH_VALUE = "mFCct"
GIT_COMMIT_SHORT_OPTIONS_WITH_ATTACHED_VALUE = "Su"
GIT_PUSH_SHORT_OPTIONS_WITH_VALUE = "o"
GIT_ALIAS_DEPTH = 5
HOOKS_BYPASS_WINDOW_VARIABLE = "SINGLEFS_COMMIT_OUTSIDE_WINDOW"   # 用户私有的提交窗口闸认它：窗口外提交设它放行窗口、大文件照判
GIT_HOOKS_INSTALL_COMMAND = "python3 scripts/gate/git-hooks-install.py install"
GIT_HOOKS_LIBRARY = []   # 同目录的 lib_git_hooks.py，第一次用到才导入；导入不了是 [None]
# git 的选项表：全名 → 值的写法（None 不带值，"required" 值贴在 = 后面或是下一个词，"optional" 值只能贴着写）。长选项认全名、唯一前缀的缩写与 --no- 否定，照 git 的 parse-options（expand_long_option）
GIT_COMMIT_LONG_OPTIONS = dict.fromkeys(("all", "allow-empty", "allow-empty-message", "amend", "branch", "dry-run", "edit", "help", "include", "interactive", "long", "no-verify",
                                         "null", "only", "patch", "pathspec-file-nul", "porcelain", "no-post-rewrite", "post-rewrite", "quiet", "reset-author", "short",
                                         "signoff", "status", "verbose", "verify", "ahead-behind"))
GIT_COMMIT_LONG_OPTIONS.update(dict.fromkeys(("author", "cleanup", "date", "file", "fixup", "message", "pathspec-from-file", "reedit-message", "reuse-message", "squash",
                                              "template", "trailer"), "required"))
GIT_COMMIT_LONG_OPTIONS.update({"gpg-sign": "optional", "untracked-files": "optional"})
GIT_PUSH_LONG_OPTIONS = dict.fromkeys(("all", "atomic", "branches", "delete", "dry-run", "follow-tags", "force", "force-if-includes", "help", "ipv4", "ipv6", "mirror", "no-verify",
                                       "porcelain", "progress", "prune", "quiet", "set-upstream", "tags", "thin", "verbose", "verify"))
GIT_PUSH_LONG_OPTIONS.update(dict.fromkeys(("exec", "push-option", "receive-pack", "repo"), "required"))
GIT_PUSH_LONG_OPTIONS.update({"force-with-lease": "optional", "signed": "optional", "recurse-submodules": "required"})
GIT_MERGE_SHORT_OPTIONS_WITH_VALUE = "mFsX"   # git merge 的 -n 是 --no-stat，不是 --no-verify 的短写
GIT_MERGE_LONG_OPTIONS = dict.fromkeys(("abort", "allow-unrelated-histories", "autostash", "commit", "continue", "edit", "ff", "ff-only", "help", "overwrite-ignore", "progress", "quiet", "quit",
                                        "rerere-autoupdate", "signoff", "squash", "stat", "summary", "verbose", "verify", "verify-signatures"))
GIT_MERGE_LONG_OPTIONS.update(dict.fromkeys(("cleanup", "file", "into-name", "message", "strategy", "strategy-option"), "required"))
GIT_MERGE_LONG_OPTIONS.update({"gpg-sign": "optional", "log": "optional"})
GIT_AM_SHORT_OPTIONS_WITH_VALUE = "Cp"   # git am 的 -n 是 --no-verify 的短写（git am -h：-n, --no-verify）；-C <n>、-p <num> 吃值
GIT_AM_LONG_OPTIONS = dict.fromkeys(("abort", "allow-empty", "committer-date-is-author-date", "continue", "ignore-date", "ignore-space-change", "ignore-whitespace", "interactive", "keep",
                                     "keep-cr", "keep-non-patch", "message-id", "no-verify", "quiet", "quit", "reject", "rerere-autoupdate", "resolved", "scissors", "signoff", "skip",
                                     "utf8", "verify", "3way"))
GIT_AM_LONG_OPTIONS.update(dict.fromkeys(("directory", "empty", "exclude", "include", "patch-format", "quoted-cr", "resolvemsg", "whitespace"), "required"))
GIT_AM_LONG_OPTIONS.update({"gpg-sign": "optional", "show-current-patch": "optional"})
GIT_BYPASS_OPTION_TABLES = {"commit": (GIT_COMMIT_LONG_OPTIONS, GIT_COMMIT_SHORT_OPTIONS_WITH_VALUE, GIT_COMMIT_SHORT_OPTIONS_WITH_ATTACHED_VALUE),
                            "push": (GIT_PUSH_LONG_OPTIONS, GIT_PUSH_SHORT_OPTIONS_WITH_VALUE, ""),
                            "merge": (GIT_MERGE_LONG_OPTIONS, GIT_MERGE_SHORT_OPTIONS_WITH_VALUE, "S"),
                            "am": (GIT_AM_LONG_OPTIONS, GIT_AM_SHORT_OPTIONS_WITH_VALUE, "S")}   # 子命令 → (长选项表, 带值的短选项, 值贴在后面的短选项)：认 --no-verify 的几个子命令
GIT_KNOWN_SUBCOMMANDS = frozenset(("status diff log show rev-parse ls-files ls-tree cat-file branch remote config fetch tag describe blame grep help version rev-list shortlog "
                                   "show-ref for-each-ref symbolic-ref merge-base name-rev diff-tree diff-index check-ignore count-objects reflog var ls-remote whatchanged "
                                   "gc fsck rm mv stash cherry-pick checkout switch merge reset restore revert rebase read-tree update-index apply am pull clean "
                                   "sparse-checkout init clone worktree submodule filter-branch update-ref bisect format-patch archive notes commit push add").split())   # 别名盖不住内建子命令：这些不查别名


def git_hooks_library():
    if not GIT_HOOKS_LIBRARY:
        try:
            specification = importlib.util.spec_from_file_location("lib_git_hooks", os.path.join(HOOK_DIRECTORY, "lib_git_hooks.py"))
            module = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(module)
            GIT_HOOKS_LIBRARY.append(module)
        except Exception as error:   # 文件不在、语法错
            not_judged(f"bash-command-detector.sh 读不到 lib_git_hooks.py（{error!r}），git commit / git push 前 git 钩子装没装上这一次没判")
            GIT_HOOKS_LIBRARY.append(None)
    return GIT_HOOKS_LIBRARY[0]


def expand_long_option(word, table):
    """`--名字[=值]` 一个词 → (全名, 是不是 --no- 否定, 值或 None)：全名、唯一前缀的缩写（--no-veri）、--no- 否定都认；缩写有歧义或不在表里交 None。
    弄坏开关 long-option-prefix-unseen 下只认全名。"""
    name, equals, value = word[2:].partition("=")
    names = {}
    for full in table:
        names[full] = (full, False)
        if not full.startswith("no-"):
            names["no-" + full] = (full, True)
    if name in names:
        matched = {names[name]}
    elif not name or detector_break_is_set("long-option-prefix-unseen"):
        return None
    else:
        matched = {names[candidate] for candidate in names if candidate.startswith(name)}
    if len(matched) != 1:
        return None
    full, negated = next(iter(matched))
    return full, negated, (value if equals else None)


def git_option_tokens(arguments, long_options, short_with_value, short_with_attached_value=""):
    """子命令之后的参数里的选项，按出现次序：("long", 全名, 是不是 --no- 否定)、("short", 字母)、("unknown", 词)（不在表里或缩写有歧义的长选项）；`--` 之后不再认。
    带值的选项（表里 required 的长选项不贴着 = 写时、short_with_value 里的短选项）吃掉下一个词（合写时贴在后面的就是值），short_with_attached_value 里的短选项把后面的字母当它的值。"""
    skip_next = False
    for word in arguments:
        if skip_next:
            skip_next = False
        elif word == "--":
            return
        elif word.startswith("--"):
            found = expand_long_option(word, long_options)
            if found is None:
                yield ("unknown", word)
                continue
            name, negated, attached = found
            yield ("long", name, negated)
            if long_options[name] == "required" and attached is None and not negated:
                skip_next = True
        elif word.startswith("-") and len(word) > 1:
            for position, letter in enumerate(word[1:]):
                yield ("short", letter)
                if letter in short_with_value:
                    skip_next = position == len(word) - 2
                    break
                if letter in short_with_attached_value:
                    break


def git_hook_bypass_reasons(global_options, environment, subcommand, arguments, hooks_path_installed=lambda value: False):
    """git commit / git push / git merge / git am 绕开 git 钩子的写法：--no-verify（长选项认唯一前缀的缩写；git merge 的 --no-verify 绕开 pre-merge-commit 与 commit-msg，
    git am 的绕开 pre-applypatch 与 applypatch-msg）、git commit 与 git am 的 -n（合写的 -nm 同样算；git push 的 -n 是 --dry-run、git merge 的 -n 是 --no-stat，不算）、
    全局的 `-c core.hooksPath=…` 与 `--config-env=core.hooksPath=…`、环境里 GIT_CONFIG_KEY_<n> 写成 core.hooksPath 或 GIT_CONFIG_PARAMETERS 里有它；交说明列表（空表是没绕开）。
    `-c core.hooksPath=<值>`、GIT_CONFIG_KEY_<n>=core.hooksPath 加 GIT_CONFIG_VALUE_<n>=<值> 的值经 hooks_path_installed（值 → 是不是安装脚本装的那个钩子目录）判出是装对的，不算绕开；
    `--config-env` 的值是环境变量名、GIT_CONFIG_PARAMETERS 是 git 自己内部的拼法，读不出值，一律算。环境变量 HUSKY=0 这类别的工具的开关不算。
    弄坏开关 bypass-flag-unseen 下不认 --no-verify 与 -n，bypass-config-unseen 下不认改 core.hooksPath 的几种。"""
    reasons = []
    if not detector_break_is_set("bypass-flag-unseen"):
        long_options, short_with_value, short_with_attached_value = GIT_BYPASS_OPTION_TABLES[subcommand]
        for token in git_option_tokens(arguments, long_options, short_with_value, short_with_attached_value):
            if token[0] == "long" and ((token[1] == "verify" and token[2]) or (token[1] == "no-verify" and not token[2])):
                reasons.append(f"git {subcommand} --no-verify")
            elif token[0] == "short" and subcommand in ("commit", "am") and token[1] == "n":
                reasons.append(f"git {subcommand} -n（--no-verify 的短写）")
    if not detector_break_is_set("bypass-config-unseen"):
        position = 0
        while position < len(global_options):
            word = global_options[position]
            value = global_options[position + 1] if word in ("-c", "--config-env") and position + 1 < len(global_options) else word[len("--config-env="):] if word.startswith("--config-env=") else ""
            if (word in ("-c", "--config-env") or word.startswith("--config-env=")) and value.partition("=")[0].lower() == "core.hookspath":
                if not (word == "-c" and hooks_path_installed(value.partition("=")[2])):
                    reasons.append(f"{'-c' if word == '-c' else '--config-env'} core.hooksPath=…")
            position += 2 if word in GIT_GLOBAL_OPTIONS_WITH_VALUE else 1
        for name, value in environment.items():
            key = re.fullmatch(r"GIT_CONFIG_KEY_([0-9]+)", name)
            if key and str(value).lower() == "core.hookspath":
                if not hooks_path_installed(str(environment.get(f"GIT_CONFIG_VALUE_{key.group(1)}", ""))):
                    reasons.append(f"环境变量 {name} 改 core.hooksPath")
            elif name == "GIT_CONFIG_PARAMETERS" and "core.hookspath" in str(value).lower():
                reasons.append(f"环境变量 {name} 改 core.hooksPath")
    return list(dict.fromkeys(reasons))


def git_config_hooks_path_write(arguments):
    """git config 改（设、加、删）core.hooksPath：交 None（不是写：读、--get、--list 这类不算）或只有一个元素的元组：元素是纯设值（`git config core.hooksPath <值>`，
    子命令写法 `git config set core.hooksPath <值>` 同）的那个值，--add、--replace-all、--unset、--unset-all 这几种没有值、元素是 None。弄坏开关 bypass-config-unseen 下不认。"""
    if detector_break_is_set("bypass-config-unseen"):
        return None
    words = [word for word in arguments if word != "--"]
    keys = [position for position, word in enumerate(words) if word.lower() == "core.hookspath"]
    if not keys:
        return None
    flags = {word for word in words if word.startswith("-")}
    if flags & {"--unset", "--unset-all", "--replace-all", "--add"}:
        return (None,)
    if flags & {"--get", "--get-all", "--get-regexp", "--list", "-l"}:
        return None
    for position in keys:
        if position + 1 < len(words) and not words[position + 1].startswith("-"):
            return (words[position + 1],)
    return None


def git_config_core_section_change(arguments):
    """git config --remove-section core、--rename-section core <新名>（长选项的唯一前缀缩写与子命令写法 remove-section、rename-section 同）：core 段连同 core.hooksPath 一起没了或改了名，
    交一句说明；别的段、别的写法交空串。弄坏开关 config-section-unseen 下一概交空串。"""
    if detector_break_is_set("config-section-unseen"):
        return ""
    words = [word.lower() for word in arguments if word != "--"]
    for position, word in enumerate(words[:-1]):
        kind = ("remove-section" if (word.startswith("--rem") and "--remove-section".startswith(word)) or word == "remove-section"
                else "rename-section" if (word.startswith("--ren") and "--rename-section".startswith(word)) or word == "rename-section" else None)
        if kind and words[position + 1] == "core":
            return f"git config {kind} core（core 段连同 core.hooksPath 一起没了或改了名）"
    return ""


def hooks_path_is_installed_value(value, directory, location, environment):
    """core.hooksPath 设成 value（~ 开头的先展开）是绝对路径、解开之后是安装脚本装的那个钩子目录（lib_git_hooks.accepted_directories：主工作树里 HOOKS_DIRECTORY 的绝对路径，
    与 installed_hooks_path 同一个值）：directory 是命令所在的目录；
    相对路径（--git-dir、--work-tree、GIT_DIR 起的 git、链接工作树与稀疏检出的树里找不到它，lib_git_hooks.hooks_path_problems 也不认）、别的仓（没有进仓钩子目录）、目录算不出或不存在、
    --git-dir / --work-tree / GIT_DIR / GIT_WORK_TREE 改了仓的位置、值是空串的都交 False（不当装对）。
    弄坏开关 hooks-path-right-value-refused 下一概交 False，hooks-path-relative-accepted 下相对路径照 git 的规矩从这棵树的顶层目录起算、也算装对的。"""
    if (detector_break_is_set("hooks-path-right-value-refused") or not value or directory is None or not os.path.isdir(directory) or location
            or {"GIT_DIR", "GIT_WORK_TREE"} & set(environment)):
        return False
    library = git_hooks_library()
    if library is None:
        return False
    code, top = library.git_output(directory, "rev-parse", "--show-toplevel")
    if code != 0 or not top or not library.project_checkout(top):
        return False
    expanded = os.path.expanduser(value)
    if not os.path.isabs(expanded) and not detector_break_is_set("hooks-path-relative-accepted"):
        return False
    installed = library.accepted_directories(top)   # 装对的目录：主工作树里进仓钩子目录的绝对路径（一个路径或路径集合都认），两边都解开符号链接再比
    installed = {installed} if isinstance(installed, str) else set(installed)
    return os.path.realpath(expanded if os.path.isabs(expanded) else os.path.join(top, expanded)) in {os.path.realpath(path) for path in installed}


def git_hook_form_judged(subcommand, arguments):
    """git commit / git push 这一次会跑 git 钩子：--help、-h、--dry-run 的 git commit 与 --help、-h 的 git push 不起钩子，不判装没装上。"""
    long_options = GIT_COMMIT_LONG_OPTIONS if subcommand == "commit" else GIT_PUSH_LONG_OPTIONS
    for token in git_option_tokens(arguments, long_options, GIT_COMMIT_SHORT_OPTIONS_WITH_VALUE if subcommand == "commit" else GIT_PUSH_SHORT_OPTIONS_WITH_VALUE,
                                   GIT_COMMIT_SHORT_OPTIONS_WITH_ATTACHED_VALUE if subcommand == "commit" else ""):
        if token[0] == "long" and token[1] in ("help",) + (("dry-run",) if subcommand == "commit" else ()) and not token[2]:
            return False
        if token[0] == "short" and token[1] == "h":
            return False
    return True


def git_hooks_not_installed(directory, location, environment):
    """directory（命令所在的仓里的目录）所在的树自带进仓的钩子目录、而 core.hooksPath 没指到它时的问题说明；别的仓（样本仓、临时仓）、目录算不出或不存在、
    --git-dir / --work-tree / GIT_DIR / GIT_WORK_TREE 改了仓的位置的不判（交空串）。弄坏开关 hooks-not-installed-allowed 下一概不判。"""
    if detector_break_is_set("hooks-not-installed-allowed") or directory is None or not os.path.isdir(directory) or location or {"GIT_DIR", "GIT_WORK_TREE"} & set(environment):
        return ""
    library = git_hooks_library()
    if library is None:
        return ""
    code, top = library.git_output(directory, "rev-parse", "--show-toplevel")
    if code != 0 or not top or not library.project_checkout(top):
        return ""
    problems = library.hooks_path_problems(top)
    return f"{top}：{'；'.join(problems)}" if problems else ""


def git_command_steps(command, start_directory):
    """命令文本里命令位置上的每一条命令（共用模块切出来的，bash -c、命令替换、前缀已剥开），连同启动器（LAUNCHER_NAMES）剥开之后里面那一条，按出现次序。
    弄坏开关 dynamic-launcher-unseen 下不剥启动器（xargs、find -exec 起的 git commit --no-verify 看不见）。"""
    def walk(text, directory, environment, depth):
        for command_at in shell_words.commands_at_command_position(text, directory, environment).commands:
            yield command_at
            if depth >= shell_words.MAXIMUM_NESTING or detector_break_is_set("dynamic-launcher-unseen") or not command_at.words:
                continue
            name = os.path.basename(command_at.words[0])
            if name not in LAUNCHER_NAMES or name == "ssh":   # ssh 起的命令跑在别的机器上，不碰这里的仓；命令词是启动器才去装那份几千行的库
                continue
            library = project_code_runs_library()
            if library is None:
                continue
            inner = library.command_under_launcher(command_at.words, command_at.directory)
            if inner is None:
                continue
            inner_words, inner_directory = inner
            base = command_at.directory
            if inner_directory and base is not None:
                base = os.path.normpath(os.path.join(base, os.path.expanduser(inner_directory)))
            yield from walk(shlex.join(inner_words), base, command_at.environment, depth + 1)

    yield from walk(command, start_directory, None, 0)


def git_alias_value(global_options, subcommand, directory, environment):
    """git 别名 subcommand 的值：命令行 -c alias.<名>=<值> 的先于配置；配置经共用模块的 git_config_commands 照 git 自己的读法取（全局、系统、仓里的，-C 与 --git-dir 跟着带）；
    没有这个别名交 None。"""
    wanted = "alias." + subcommand.lower()
    given, options, position = None, [], 0
    while position < len(global_options):
        word = global_options[position]
        if word == "-c" and position + 1 < len(global_options):
            name, equals, value = global_options[position + 1].partition("=")
            if equals and name.lower() == wanted:
                given = value
            position += 2
        elif word in ("-C", "--git-dir", "--work-tree") and position + 1 < len(global_options):
            options.append((word, global_options[position + 1]))
            position += 2
        else:
            position += 1
    if given is not None:
        return given
    return dict(shell_words.git_config_commands(options, directory, environment)).get(wanted)


def resolved_git_subcommand(global_options, subcommand, arguments, directory, environment):
    """子命令是别名时换成它指的子命令（别名的值照 git 的写法按空白与引号切，接着往下找，最多 GIT_ALIAS_DEPTH 层）：交 (子命令, 参数)。内建子命令（GIT_KNOWN_SUBCOMMANDS）不查别名：
    git 不许别名盖住它们；`!` 开头的 shell 别名、值切不开的、值以选项起头的不再往下认。弄坏开关 alias-unresolved 下不查别名。"""
    if detector_break_is_set("alias-unresolved"):
        return subcommand, arguments
    seen = set()
    for _ in range(GIT_ALIAS_DEPTH):
        if not subcommand or subcommand in GIT_KNOWN_SUBCOMMANDS or subcommand in seen or "$" in subcommand or "`" in subcommand:
            break
        seen.add(subcommand)
        alias = git_alias_value(global_options, subcommand, directory, environment)
        if alias is None or alias.startswith("!"):
            break
        try:
            words = shlex.split(alias)
        except ValueError:
            break
        if not words or words[0].startswith("-"):
            break
        subcommand, arguments = words[0], words[1:] + arguments
    return subcommand, arguments


def git_hook_findings(command, start_directory):
    """㉒：命令位置上的 git commit / git push / git merge / git am / git config 里绕开 git 钩子的写法（git config 改 core.hooksPath 设成装对的值、`-c core.hooksPath=<装对的值>` 不算），
    与 git 钩子没装上的 git commit / git push，交 [(命令的写法, 'bypass' 或 'not-installed', 说明)…]。
    不判 git commit / git push 要带的内容有没有超过单文件上限的文件：那是 git 钩子在提交、推送那一刻判的。弄坏开关 bypass-every-git-command 下只读的 git log、git status
    这类也按 git commit 判。主 agent 与子 agent 一样判。"""
    if "git" not in command:
        return []
    found = []
    for command_at in git_command_steps(command, start_directory):
        words = command_at.words
        if not words or os.path.basename(words[0]) != "git":
            continue
        shown = " ".join(words)[:120]
        global_options, subcommand, arguments = git_global_split(words[1:])
        environment = command_at.environment or {}
        subcommand, arguments = resolved_git_subcommand(global_options, subcommand, arguments, command_at.directory, environment)
        directory, location = git_location(global_options, command_at.directory)
        installed = lambda value: hooks_path_is_installed_value(value, directory, location, environment)
        if subcommand == "config":
            written = git_config_hooks_path_write(arguments)
            if written is not None and not (written[0] is not None and installed(written[0])):
                found.append((shown, "bypass", "git config 改 core.hooksPath"))
            elif git_config_core_section_change(arguments):
                found.append((shown, "bypass", git_config_core_section_change(arguments)))
            continue
        bypass_subcommands = (("commit", "push") + (() if detector_break_is_set("merge-bypass-unseen") else ("merge",))
                              + (() if detector_break_is_set("am-bypass-unseen") else ("am",)))
        if subcommand not in bypass_subcommands and not (detector_break_is_set("bypass-every-git-command") and subcommand and "$" not in subcommand and "`" not in subcommand):
            continue
        reasons = git_hook_bypass_reasons(global_options, environment, subcommand if subcommand in ("push", "merge", "am") else "commit", arguments, installed)
        if reasons:
            found.append((shown, "bypass", "、".join(reasons)))
            continue
        if subcommand in ("commit", "push") and git_hook_form_judged(subcommand, arguments):
            problem = git_hooks_not_installed(directory, location, environment)
            if problem:
                found.append((shown, "not-installed", problem))
    return found


def git_hook_refusal_lines(findings, agent_type):
    """㉒ 的拒绝说明各行（每组两行：原因、出路）：绕开钩子的写法一组、钩子没装上的一组，没有的组不写。"""
    lines = []
    bypass = [finding for finding in findings if finding[1] == "bypass"]
    missing = [finding for finding in findings if finding[1] == "not-installed"]
    if bypass:
        listed = "；".join(f"{shown}（{detail}）" for shown, _kind, detail in bypass)
        lines.append(f"  ✗ {listed}：这样写绕开了 git 的钩子。提交与推送前的大文件判法在进仓的 git 钩子（scripts/gate/git-hooks/ 里的各个钩子，判法 scripts/gate/large-file-git-hook.py）里，"
                     "绕开它，超过单文件上限的文件悄悄进提交、推不上去，事后只能重做提交")
        lines.append(f"     → 怎么办：去掉它（--no-verify、git merge 与 git am 的 --no-verify、git am 的 -n、-n、-c core.hooksPath=…、GIT_CONFIG_* 里的 core.hooksPath、改 core.hooksPath 的 git config、"
                     f"git config --remove-section core 与 --rename-section core）；要把 core.hooksPath 设回钩子目录，设成安装脚本装的那个值（`task-server/task submit -- {GIT_HOOKS_INSTALL_COMMAND}` 装的，主工作树里钩子目录的绝对路径，相对路径照拒）不拒；窗口外提交用环境变量 "
                     f"{HOOKS_BYPASS_WINDOW_VARIABLE}=1 放行用户私有的提交窗口闸（`{HOOKS_BYPASS_WINDOW_VARIABLE}=1 git commit …`，只在用户明说窗口外也要提交时设，大文件照判）。这一道只拒这种写法，不停你在跑的任何东西")
    if missing:
        listed = "；".join(f"{shown}（{detail}）" for shown, _kind, detail in missing)
        lines.append(f"  ✗ {listed}：进仓的 git 钩子没装上，提交与推送前的大文件判法不起作用，超过单文件上限的文件悄悄进提交、推不上去，事后只能重做提交")
        who = (f"把 `task-server/task submit -- {GIT_HOOKS_INSTALL_COMMAND}` 写进报告「要主 agent 跑的命令」一节交回主 agent，由主 agent 交服务" if agent_type
               else f"经任务服务装：`task-server/task submit -- {GIT_HOOKS_INSTALL_COMMAND}`（写 core.hooksPath 到共用的 .git/config、再核一遍；两台读同一份）")
        lines.append(f"     → 怎么办：{who}；装好之后再起这一条。这一道只拒这种写法，不停你在跑的任何东西")
    return lines


def git_hook_refusal(command, start_directory, agent_type):
    """git_hook_findings 的拒绝说明各行；写说明那一步出任何错折成判不了：记一条检出、不拒这一道，别的道照常判、照常拒。弄坏开关 git-hook-messages-unguarded 下不接异常，allow-git-hook-guard 下一概放行。"""
    if detector_break_is_set("allow-git-hook-guard"):
        return []
    findings = git_hook_findings(command, start_directory)
    if not findings:
        return []
    if detector_break_is_set("git-hook-messages-unguarded"):
        return git_hook_refusal_lines(findings, agent_type)
    try:
        return git_hook_refusal_lines(findings, agent_type)
    except Exception as error:
        not_judged(f"bash-command-detector.sh 没写成 git commit / git push 绕开钩子的拒绝说明（{error!r}），这条命令照常执行")
        return []

# run_in_background 里等本会话后台任务输出文件（<会话目录>/tasks/<id>.output）的循环：等自己起的后台任务要结束本轮等完成通知，不写轮询
TASK_OUTPUT_PATH = re.compile(r"(?:^|/)tasks/[^/\s]+\.output\b")

def waits_on_task_output(loop_commands):
    return any(TASK_OUTPUT_PATH.search(word) for words in loop_commands for word in words if words)

def background_task_output_wait_loops(text):
    """命令位置上的等待循环、条件或循环体里点名了 tasks/<id>.output 的，外面有没有 timeout 都算。"""
    return [loop for layer in shell_layers(text, honor_timeout=False) for loop in wait_loops_in(layer.commands, waits_on_task_output)]

def subagent_background_refusal(agent_type, run_in_background):
    """⑩ 子 agent（输入里有 agent_type）用 run_in_background 起命令就交 True（task-server/task wait 也算）；主 agent 与前台命令交 False。"""
    if detector_break_is_set("disable-check") or detector_break_is_set("allow-subagent-background"):
        return False
    return bool(agent_type) and run_in_background is True

def wait_loop_refusal(command, run_in_background):
    """前台（run_in_background 不是 true）的命令里有没超时的等待循环，就交回认出的循环（每个循环开头那条简单命令）；否则交 []。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-foreground-wait-loop")):
        return []
    refuse_every = detector_break_is_set("refuse-every-wait-loop")
    if run_in_background is True and not refuse_every:
        if detector_break_is_set("allow-background-task-output-wait"):
            return []
        return background_task_output_wait_loops(command)
    return unbounded_wait_loops(command, honor_timeout=not refuse_every)

# run_in_background 里后面没有 wait 的单独 &：逐层按记号找以单独的 & 收尾的作业，看之后同一层、同一对圆括号里有没有 wait
WORDS_THAT_ONLY_CLOSE_A_COMPOUND = {"done", "fi", "esac", "}"}

def jobs_with_endings(tokens):
    """一层的记号按命令分隔符切成一条条简单命令，交回 [(词, 收尾的分隔符或 None, 所在的圆括号组)]。
    圆括号组是从外到里每一层开括号的序号（顶层是空元组），一对圆括号是一个子 shell；词由共用模块的 simple_commands 取
    （重定向连同目标去掉）。数组赋值 `名字=(…)` 的括号也当一组：括号成对，组的层次不乱，元素里既没有 & 也不会被当成同组的 wait。
    没有词的只留以 & 收尾的那种（`(…) &`）。"""
    jobs, piece, groups, opened_count = [], [], [()], 0
    for is_operator, text in tokens:
        if not (is_operator and text in shell_words.COMMAND_SEPARATORS):
            piece.append((is_operator, text))
            continue
        words = shell_words.simple_commands(piece)[0]
        if words or text == "&":
            jobs.append((words[0] if words else [], text, groups[-1]))
        piece = []
        if text == "(":
            opened_count += 1
            groups.append(groups[-1] + (opened_count,))
        elif text == ")" and len(groups) > 1:
            groups.pop()
    words = shell_words.simple_commands(piece)[0]
    if words:
        jobs.append((words[0], None, groups[-1]))
    return jobs

def is_wait_command(words):
    """剥掉开头的关键字与前缀之后命令词是 wait，且不带参数、或带了进程号（`wait "$pid"`）；只带选项的（`wait -n`）不算。"""
    skipped = shell_words.skip_prefixes(words)
    if skipped.command_position is None or words[skipped.command_position] != "wait":
        return False
    arguments = words[skipped.command_position + 1:]
    return not arguments or any(not argument.startswith("-") for argument in arguments)

def background_job_label(words):
    """拒绝信息里认出的作业：它的词；只剩 done / fi / } 这类收尾词的，是整个复合命令放了后台。"""
    if not [word for word in words if word not in WORDS_THAT_ONLY_CLOSE_A_COMPOUND]:
        return "（前面那个复合命令：圆括号、花括号、循环或 if）"
    return " ".join(words)[:120]

def wait_coverage(words, wait_functions):
    """一条命令等了几个之前放后台的作业：不带参数的 wait、调用里写着不带参数 wait 的函数（Z14）交 None（全等）；`wait <进程号>…` 交进程号个数；
    `wait -n`（只带选项）、`tail --pid`、`while|until … kill -0` 轮询（文件头 ② 末那一句，Z12、Z13）各交 1；别的交 0。
    弄坏开关 wait-forms-unseen 下 wait -n、tail --pid、kill -0 轮询、函数里的 wait 都不算；partial-wait-allowed 下带进程号的 wait 照全等算。"""
    skipped = shell_words.skip_prefixes(words)
    keywords = words[:skipped.command_position] if skipped.command_position is not None else words
    if skipped.command_position is None:
        return 0
    name, arguments = words[skipped.command_position], words[skipped.command_position + 1:]
    forms_seen = not detector_break_is_set("wait-forms-unseen")
    if name == "wait":
        pids = [argument for argument in arguments if not argument.startswith("-")]
        if not arguments:
            return None
        if pids:
            return None if detector_break_is_set("partial-wait-allowed") else len(pids)
        return 1 if forms_seen else 0
    if forms_seen and name in wait_functions:
        return None
    if forms_seen and name == "tail" and any(argument.startswith("--pid") for argument in arguments):
        return 1
    if forms_seen and any(keyword in ("while", "until") for keyword in keywords) and name == "kill" and arguments[:1] == ["-0"]:
        return 1
    return 0


def wait_function_names(tokens):
    """一层记号里定义的、函数体里有一条不带参数的 wait 的函数名（`w() { wait; }`、`function w { wait; }`）。"""
    names = set()
    for index, (kind, value) in enumerate(tokens):
        name, body = None, None
        if kind is False and value == "function" and index + 1 < len(tokens) and tokens[index + 1][0] is False:
            name, body = tokens[index + 1][1], index + 2
            if tokens[body:body + 2] == [(True, "("), (True, ")")]:
                body += 2
        elif kind is False and tokens[index + 1:index + 3] == [(True, "("), (True, ")")]:
            name, body = value, index + 3
        if name is None or tokens[body:body + 1] != [(False, "{")]:
            continue
        depth, cursor = 0, body
        while cursor < len(tokens):
            if tokens[cursor] == (False, "{"):
                depth += 1
            elif tokens[cursor] == (False, "}"):
                depth -= 1
                if depth == 0:
                    break
            cursor += 1
        for body_words in shell_words.simple_commands(tokens[body + 1:cursor])[0]:
            if body_words and body_words[-1:] != ["}"] and wait_coverage([word for word in body_words if word != "{"], set()) is None:
                names.add(name)
    return names


def unwaited_background_jobs(text):
    """整条命令每一层（顶层、bash -c 的代码、喂给 shell 的 heredoc 正文、命令替换）里以单独的 & 收尾、之后同一层同一对圆括号里没等到的作业，交回每个作业的说法。
    逐层按次序数：不带参数的 wait（或调用里有它的函数）等齐之前的全部；`wait <进程号>` 等它写了几个号就几个（两个 & 只等一个的，剩下那个照没等到拒）；
    `wait -n`、`tail --pid`、`kill -0` 轮询各等一个（Z12、Z13），先等先起的那个。"""
    found = []
    for layer in shell_layers(text):
        jobs = jobs_with_endings(layer.tokens)
        wait_functions = set() if detector_break_is_set("wait-forms-unseen") else wait_function_names(layer.tokens)
        pending = {}
        for words, ending, group in jobs:
            if ending == "&":
                pending.setdefault(group, []).append(words)
                continue
            covered = wait_coverage(words, wait_functions)
            if covered is None:
                pending[group] = []
            elif covered:
                pending[group] = pending.get(group, [])[covered:]
        for waiting in pending.values():
            found += [background_job_label(words) for words in waiting]
    return list(dict.fromkeys(found))

def unwaited_ampersand_refusal(command, run_in_background):
    """run_in_background 为 true 的命令里有以单独的 & 收尾、之后同一层没有 wait 的作业，就交回认出的作业；否则交 []。前台不拒。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-unwaited-ampersand")):
        return []
    if run_in_background is not True:
        return []
    return unwaited_background_jobs(command)

# ⑤ 整份覆盖 research/results/ 下未跟踪的产物：按命令位置认会整份覆盖文件的写法，跟着 cd 与这条命令里赋过的变量算目标
OVERWRITING_REDIRECTS = {">", ">|", "&>", ">&"}   # >> 与 &>> 是追加；>& 后面跟 fd 号或 - 时是复制、关闭 fd，不写文件
APPENDING_REDIRECTS = {">>", "&>>"}                # 只有 ⑨（every_write）认：追加也改了那一份
READ_WRITE_REDIRECT = "<>"                         # 只有 ⑭（service_writes）认：读写打开、不截断，往原 inode 上从头改写
FILE_DESCRIPTOR_WORD = re.compile(r"^(?:\d+-?|-)$")
ASSIGNED_VARIABLE = re.compile(r"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))")
# 目标词里这一刻算不出的段：变量（含 ${…} 的各种展开与特殊参数）、花括号展开；命令替换另由共用模块的 substitution_span 认
BRACE_PIECE = re.compile(r"\{[^{}/]*(?:,|\.\.)[^{}/]*\}")   # 目标词里的花括号展开（`{a,b}`、`{1..2}`）：重定向目标上的单列一种拒绝说明
UNKNOWN_PIECE = re.compile(r"\$\{[^}]*\}|\$[A-Za-z_][A-Za-z0-9_]*|\$[0-9@*#?$!-]|\{[^{}/]*(?:,|\.\.)[^{}/]*\}")
GLOB_CHARACTERS = set("*?[")
RESULTS_DIRECTORY_WORDS = ("research", "results")
# cp / mv / install：命令名 → 要带一个值的短选项；带值的长选项（不写 = 时值是下一个词）
COPY_SHORT_OPTIONS_WITH_VALUE = {"cp": "tS", "mv": "tS", "install": "tSgmo"}
COPY_LONG_OPTIONS_WITH_VALUE = {"--target-directory", "--suffix", "--no-preserve", "--sparse", "--group", "--mode", "--owner", "--strip-program"}
GIT_GLOBAL_OPTIONS_WITH_VALUE = {"-C", "-c", "--git-dir", "--work-tree", "--namespace", "--config-env"}
GIT_ADD_OPTIONS_THAT_DO_NOT_KEEP_BYTES = {"-n", "--dry-run", "-u", "--update", "-N", "--intent-to-add", "-p", "--patch", "-i", "--interactive", "-e", "--edit", "--refresh"}
RESULTS_PRESERVING_KINDS = {"git add", "mv"}   # rm 掉的旧字节没留住，⑤ 不认

class OverwriteCandidate(NamedTuple):
    form: str                            # 认出的写法：>、>|、&>、>&、tee、cp、mv、install、dd、truncate；⑤ 与 ⑦ 共用同一份（PythonCode 另判，只给 ⑦ 用）
    target: str                          # 目标词（这条命令里前面赋过值的变量已代入）
    directory: str | None                # 那一刻的当前目录；cd 算不出来是 None，按仓库根解析
    sources: tuple = ()                  # cp / mv / install 的源
    into_directory: bool | None = False  # True：-t 给的目录；None：看目标是不是已有目录、以 / 结尾；False：目标就是写的那个文件
    replaces_inode: bool = False         # 先删掉目标或换成链接再写（cp --remove-destination、-l、-s）：⑦ 不拦，⑤ 照样算覆盖
    appends: bool = False                # tee -a / --append（只在 every_write 下交，form 照写 tee）：⑲ 认它是追加
    links_sources: bool = False          # cp -l、不带 -s 的 ln（只在 service_writes 下交）：目标是源的硬链接，往目标写就是往源写，⑭ 另判源

class PreservingStep(NamedTuple):
    """同一条命令里让旧字节留住的一步：git add 的路径（进了 git）、mv 挪走的源（换了名字还在）；排在它后面再写这些路径不算覆盖。
    rm 删掉的路径也记一步（kind 为 rm）：只给 ⑦ 用，之后写那个名字是一个新 inode；⑤ 不认它，旧字节并没有留住。"""
    words: tuple
    directory: str | None
    kind: str = "git add"                # git add、mv、rm

class PythonCode(NamedTuple):
    """喂给 python 的一段代码（`python3 -c` 的，或喂给它的 heredoc 正文）与跑它那一刻的当前目录：只给 ⑦ 用。"""
    code: str
    directory: str | None

class CopyArguments(NamedTuple):
    operands: list                 # 非选项参数：源在前，目标在最后（给了 -t 时全是源）
    target_directory: str | None   # -t / --target-directory 给的目录
    treats_target_as_file: bool    # -T / --no-target-directory
    keeps_existing_target: bool    # -n / --no-clobber / --update=none…：已有的目标不被整份盖掉
    creates_directories: bool      # install -d / --directory：参数全是要建的目录
    replaces_target_inode: bool = False  # cp --remove-destination、-l / --link、-s / --symbolic-link：目标先删掉或换成链接，不在原 inode 上写
    makes_backup: bool = False     # -b / --backup[=…]：已有的目标先换个名字留底，目标这个名字照样是新写进去的（⑤ ⑦ 照旧当「不盖掉」，⑭ 当写）；ln 的 -b 同
    links_hard: bool = False       # cp -l / --link：目标是源的硬链接
    links_symbolic: bool = False   # cp -s / --symbolic-link、ln -s / --symbolic：目标是指向源的符号链接

def substitute_assigned_variables(word, variables):
    """`$名字`、`${名字}` 换成这条命令里前面赋过的值；没赋过的变量、命令替换、特殊参数照留。"""
    return ASSIGNED_VARIABLE.sub(lambda match: variables.get(match.group(1) or match.group(2), match.group(0)), word)

def tee_files(arguments):
    """tee 的参数：带 -a / --append 的交 []（追加），否则交它写的每个文件（`-` 是标准输出，不算）。"""
    files, options_ended = [], False
    for argument in arguments:
        if options_ended or argument == "-" or not argument.startswith("-"):
            if argument != "-":
                files.append(argument)
        elif argument == "--":
            options_ended = True
        elif argument.startswith("--"):
            if len(argument) >= 3 and "--append".startswith(argument):
                return []
        elif "a" in argument[1:]:
            return []
    return files

def parse_copy_arguments(name, arguments):
    """cp / mv / install / ln（⑭ 另认）的参数按 GNU 的选项规矩切开（选项可以排在文件名后面，`--` 之后全是文件名）。"""
    operands, target_directory, treats_target_as_file, keeps_existing_target, creates_directories = [], None, False, False, False
    replaces_target_inode = makes_backup = links_hard = links_symbolic = False
    short_options_with_value = COPY_SHORT_OPTIONS_WITH_VALUE.get(name, "tS")   # ln 与 cp、mv 一样 -t、-S 带值
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        position += 1
        if argument == "--":
            operands += arguments[position:]
            break
        if argument == "-" or not argument.startswith("-"):
            operands.append(argument)
            continue
        if argument.startswith("--"):
            option, has_value, value = argument.partition("=")
            if option in COPY_LONG_OPTIONS_WITH_VALUE and not has_value:
                value = arguments[position] if position < len(arguments) else ""
                position += 1
            if option == "--target-directory":
                target_directory = value
            elif option == "--no-target-directory":
                treats_target_as_file = True
            elif option == "--no-clobber" or (option == "--update" and value.startswith("none")):
                keeps_existing_target = True
            elif option == "--backup":
                makes_backup = True
            elif option == "--directory" and name == "install":
                creates_directories = True
            elif option in ("--remove-destination", "--link", "--symbolic-link") and name == "cp":
                replaces_target_inode = True
                links_hard = links_hard or option == "--link"
                links_symbolic = links_symbolic or option == "--symbolic-link"
            elif option == "--symbolic" and name == "ln":
                links_symbolic = True
            continue
        letters = argument[1:]
        for index, letter in enumerate(letters):
            if letter in short_options_with_value:
                value = letters[index + 1:]
                if not value:
                    value = arguments[position] if position < len(arguments) else ""
                    position += 1
                if letter == "t":
                    target_directory = value
                break
            if letter == "T":
                treats_target_as_file = True
            elif letter == "n" and name != "ln":   # ln 的 -n 是 --no-dereference，不是不盖已有的目标
                keeps_existing_target = True
            elif letter == "b":
                makes_backup = True
            elif letter == "d" and name == "install":
                creates_directories = True
            elif letter in "ls" and name == "cp":
                replaces_target_inode = True
                links_hard = links_hard or letter == "l"
                links_symbolic = links_symbolic or letter == "s"
            elif letter == "s" and name == "ln":
                links_symbolic = True
    return CopyArguments(operands, target_directory, treats_target_as_file, keeps_existing_target, creates_directories, replaces_target_inode,
                         makes_backup, links_hard, links_symbolic)

def git_added_paths(arguments):
    """git 的参数：是 `git [全局选项] add …` 且真把内容放进 git（不是 -n、-u、-N、-p 这类）时，交回 (-C 给的目录列表, 加进去的路径)；否则交 None。
    `-A` / `--all` 不带路径时加的是整个工作树，交 ["."]（按 -C 与当前目录算，仓库根上跑时就是整个仓）。"""
    position, change_directories = 0, []
    while position < len(arguments) and arguments[position].startswith("-"):
        option = arguments[position].partition("=")[0]
        has_value_inline = "=" in arguments[position]
        position += 1
        if option in GIT_GLOBAL_OPTIONS_WITH_VALUE and not has_value_inline:
            if option == "-C" and position < len(arguments):
                change_directories.append(arguments[position])
            position += 1
    if arguments[position:position + 1] != ["add"]:
        return None
    paths, adds_everything, options_ended = [], False, False
    for argument in arguments[position + 1:]:
        if options_ended or not argument.startswith("-"):
            paths.append(argument)
        elif argument == "--":
            options_ended = True
        elif argument in GIT_ADD_OPTIONS_THAT_DO_NOT_KEEP_BYTES:
            return None
        elif argument in ("-A", "--all"):
            adds_everything = True
    if not paths and adds_everything:
        paths = ["."]
    return change_directories, paths

# ⑭ 解包（tar -x）与打补丁（patch、git apply）：包里、补丁里写了什么这一刻不读，按「往哪个目录里写」判，目录是服务源码目录或包着已有服务源码的上级就拒（ServiceSourceView.touches）
TAR_SHORT_OPTIONS_WITH_VALUE = "bCfFgHIKLNTVX"
TAR_LONG_OPTIONS_WITH_VALUE = {"--file", "--exclude", "--exclude-from", "--files-from", "--transform", "--xform", "--use-compress-program", "--strip-components",
                               "--owner", "--group", "--mode", "--newer", "--newer-mtime", "--index-file", "--label", "--suffix", "--listed-incremental",
                               "--to-command", "--format", "--checkpoint-action", "--blocking-factor"}
PATCH_SHORT_OPTIONS_WITH_VALUE = "bBdDFioprVzY"
PATCH_LONG_OPTIONS_WITH_VALUE = {"--strip", "--input", "--reject-file", "--prefix", "--suffix", "--fuzz", "--define", "--version-control", "--basename-prefix", "--basename-suffix"}
GIT_APPLY_OPTIONS_THAT_DO_NOT_WRITE = {"--check", "--stat", "--numstat", "--summary", "-h", "--help"}

def tar_extraction_directory(arguments):
    """tar 的参数：是解包（-x、--extract、--get，合写的 -xzf 与不带横线的 xzf 同）时交回解进哪个目录（-C / --directory 照写的顺序累计，没给是 "."）；不是解包交 None。
    解到标准输出的（-O、--to-stdout、--to-command）不写盘，也交 None；弄坏开关 service-source-stdout-extract-judged 下照解进目录判。"""
    extracting, to_stdout, directory, position = False, False, ".", 0
    if arguments and not arguments[0].startswith("-") and re.fullmatch(r"[A-Za-z0-9]+", arguments[0]):   # 不带横线的老写法：tar xzf 包
        extracting = "x" in arguments[0]
        to_stdout = "O" in arguments[0]
        position = 1
        for letter in arguments[0]:
            if letter in TAR_SHORT_OPTIONS_WITH_VALUE:
                value = arguments[position] if position < len(arguments) else ""
                position += 1
                if letter == "C":
                    directory = os.path.join(directory, value)
    while position < len(arguments):
        argument = arguments[position]
        position += 1
        if argument == "--":
            break
        if argument.startswith("--"):
            option, has_value, value = argument.partition("=")
            if option in ("--extract", "--get"):
                extracting = True
            elif option in ("--to-stdout", "--to-command"):
                to_stdout = True
                if option == "--to-command" and not has_value:
                    position += 1
            elif option == "--directory":
                if not has_value:
                    value = arguments[position] if position < len(arguments) else "."
                    position += 1
                directory = os.path.join(directory, value)
            elif option in TAR_LONG_OPTIONS_WITH_VALUE and not has_value:
                position += 1
            continue
        if argument.startswith("-") and argument != "-":
            letters = argument[1:]
            for index, letter in enumerate(letters):
                if letter == "x":
                    extracting = True
                elif letter == "O":
                    to_stdout = True
                elif letter in TAR_SHORT_OPTIONS_WITH_VALUE:
                    value = letters[index + 1:]
                    if not value:
                        value = arguments[position] if position < len(arguments) else ""
                        position += 1
                    if letter == "C":
                        directory = os.path.join(directory, value)
                    break
    if to_stdout and not detector_break_is_set("service-source-stdout-extract-judged"):
        return None
    return directory if extracting else None

def patch_target(arguments):
    """patch 的参数：交回 (写法, 目标词)：就地打补丁的目标是 -d / --directory 给的目录（没给是 "."），-o / --output 给了文件的补丁结果只写进那个文件；
    带位置参数的原文件（`patch 原文件 补丁`、`patch -i 补丁 原文件`）时补丁只改那一份文件（在 -d 给的目录里找它），照那一份判；--dry-run 什么都不写，交 None。
    弄坏开关 service-source-patch-operand-ignored 下不看位置参数的原文件，照目录判。"""
    directory, output, operands, position = ".", None, [], 0
    while position < len(arguments):
        argument = arguments[position]
        position += 1
        if argument == "--dry-run":
            return None
        if argument.startswith("--"):
            option, has_value, value = argument.partition("=")
            if option in ("--directory", "--output"):
                if not has_value:
                    value = arguments[position] if position < len(arguments) else ""
                    position += 1
                if option == "--directory":
                    directory = value
                else:
                    output = value
            elif option in PATCH_LONG_OPTIONS_WITH_VALUE and not has_value:
                position += 1
            continue
        if argument.startswith("-") and argument != "-":
            letters = argument[1:]
            for index, letter in enumerate(letters):
                if letter in PATCH_SHORT_OPTIONS_WITH_VALUE:
                    value = letters[index + 1:]
                    if not value:
                        value = arguments[position] if position < len(arguments) else ""
                        position += 1
                    if letter == "d":
                        directory = value
                    elif letter == "o":
                        output = value
                    break
            continue
        if argument != "-":
            operands.append(argument)
    if output:
        return "patch -o", output
    if operands and not detector_break_is_set("service-source-patch-operand-ignored"):
        return "patch 原文件", os.path.join(directory, operands[0])
    return "patch", directory

# ⑭ 另认的删除、挪走与原地改写（service_writes）：unlink、find -delete 与 find -exec 里对找到的路径写的命令、rsync --delete 与 --remove-source-files、
#   gzip 一类把原文件换成别的文件的、`<>` 重定向、shred、sort -o、gawk -i inplace、unzip；各条按命令名认，参数按各命令自己的选项规矩切
COMPRESSOR_NAMES = {"gzip", "gunzip", "bzip2", "bunzip2", "xz", "unxz", "lzma", "unlzma", "compress", "uncompress", "pigz", "unpigz", "zstd", "unzstd", "lz4", "unlz4"}
COMPRESSORS_KEEPING_INPUT = {"zstd", "unzstd", "lz4", "unlz4"}   # 默认留着原文件，带 --rm 才删；别的默认压完、解完就删原文件，带 -k / --keep 才留
COMPRESSOR_KEEP_LETTERS = set("cktlhV")                          # -c 写标准输出、-k 留原文件、-t 只测、-l 只列、-h、-V：原文件都不动
COMPRESSOR_KEEP_OPTIONS = {"--stdout", "--to-stdout", "--keep", "--test", "--list", "--help", "--version"}
COMPRESSOR_SHORT_OPTIONS_WITH_VALUE = "STMFCpbIoD"
COMPRESSOR_LONG_OPTIONS_WITH_VALUE = {"--suffix", "--threads", "--memlimit", "--format", "--check", "--processes", "--blocksize", "--iterations", "--output", "--dict"}
SHRED_SHORT_OPTIONS_WITH_VALUE = "ns"
SHRED_LONG_OPTIONS_WITH_VALUE = {"--iterations", "--size", "--random-source"}
AWK_SHORT_OPTIONS_WITH_VALUE = "FfvilEe"
AWK_LONG_OPTIONS_WITH_VALUE = {"--include", "--load", "--file", "--source", "--field-separator", "--assign", "--exec"}
AWK_ASSIGNMENT_OPERAND = re.compile(r"[A-Za-z_][A-Za-z0-9_]*=")
RSYNC_REMOVING_SOURCE_OPTIONS = {"--remove-source-files", "--remove-sent-files"}

def plain_operands(arguments, short_options_with_value="", long_options_with_value=()):
    """选项之后的文件参数：`--` 之后全是文件，带值的选项连值一起跳过（short_options_with_value 是带值的短选项字母，合写的 `-n3` 值在后面），单独一个 `-` 不算文件。"""
    operands, position, options_ended = [], 0, False
    while position < len(arguments):
        argument = arguments[position]
        position += 1
        if options_ended or argument == "-" or not argument.startswith("-"):
            if argument != "-":
                operands.append(argument)
        elif argument == "--":
            options_ended = True
        elif argument.startswith("--"):
            option, has_value, _ = argument.partition("=")
            if option in long_options_with_value and not has_value:
                position += 1
        else:
            for index, letter in enumerate(argument[1:]):
                if letter in short_options_with_value:
                    if not argument[2 + index:]:
                        position += 1
                    break
    return operands

def compressor_removed_files(name, arguments):
    """gzip 一类的参数：原文件会被换成压缩或解压后的文件（压缩器 COMPRESSORS_KEEPING_INPUT 之外默认删原文件，那几个带 --rm 才删）时交回要处理的文件；
    带 -c、-k、-t、-l、-h、-V 这类原文件不动的参数，或不带 --rm 的 COMPRESSORS_KEEPING_INPUT，交 []。"""
    removes = name not in COMPRESSORS_KEEPING_INPUT
    position, options_ended = 0, False
    while position < len(arguments) and not options_ended:
        argument = arguments[position]
        position += 1
        if argument == "--":
            options_ended = True
        elif argument.startswith("--"):
            option, has_value, _ = argument.partition("=")
            if option in COMPRESSOR_KEEP_OPTIONS:
                return []
            removes = removes or option == "--rm"
            if option in COMPRESSOR_LONG_OPTIONS_WITH_VALUE and not has_value:
                position += 1
        elif argument.startswith("-") and argument != "-":
            for index, letter in enumerate(argument[1:]):
                if letter in COMPRESSOR_KEEP_LETTERS:
                    return []
                if letter in COMPRESSOR_SHORT_OPTIONS_WITH_VALUE:
                    if not argument[2 + index:]:
                        position += 1
                    break
    return plain_operands(arguments, COMPRESSOR_SHORT_OPTIONS_WITH_VALUE, COMPRESSOR_LONG_OPTIONS_WITH_VALUE) if removes else []

def sort_output_files(arguments):
    """sort 的 -o 文件、--output 文件（`-oFILE`、`--output=FILE` 同）：排完序写进去的文件。"""
    files, position = [], 0
    while position < len(arguments):
        argument = arguments[position]
        position += 1
        if argument == "--":
            break
        if argument.startswith("--"):
            option, has_value, value = argument.partition("=")
            if option == "--output":
                if not has_value:
                    value = arguments[position] if position < len(arguments) else ""
                    position += 1
                if value:
                    files.append(value)
        elif argument.startswith("-") and argument != "-":
            for index, letter in enumerate(argument[1:]):
                if letter in "kStTo":
                    value = argument[2 + index:]
                    if not value:
                        value = arguments[position] if position < len(arguments) else ""
                        position += 1
                    if letter == "o" and value:
                        files.append(value)
                    break
    return files

def awk_in_place_files(arguments):
    """gawk、awk 的参数：带 -i inplace（`-iinplace`、`--include inplace`、`--include=inplace` 同）时交回被就地改写的文件（位置参数里除了程序文本和 var=值 的赋值）；不带交 []。"""
    in_place, has_program_option, operands, position, options_ended = False, False, [], 0, False
    while position < len(arguments):
        argument = arguments[position]
        position += 1
        if options_ended or argument == "-" or not argument.startswith("-"):
            operands.append(argument)
        elif argument == "--":
            options_ended = True
        elif argument.startswith("--"):
            option, has_value, value = argument.partition("=")
            if option in AWK_LONG_OPTIONS_WITH_VALUE and not has_value:
                value = arguments[position] if position < len(arguments) else ""
                position += 1
            in_place = in_place or (option == "--include" and value.startswith("inplace"))
            has_program_option = has_program_option or option in ("--file", "--source", "--exec")
        else:
            for index, letter in enumerate(argument[1:]):
                if letter in AWK_SHORT_OPTIONS_WITH_VALUE:
                    value = argument[2 + index:]
                    if not value:
                        value = arguments[position] if position < len(arguments) else ""
                        position += 1
                    in_place = in_place or (letter == "i" and value.startswith("inplace"))
                    has_program_option = has_program_option or letter in "fEe"
                    break
    if not in_place:
        return []
    return [operand for number, operand in enumerate(operands) if (has_program_option or number > 0) and not AWK_ASSIGNMENT_OPERAND.match(operand)]

def unzip_extraction_directory(arguments):
    """unzip 的参数：解压（不是 -l、-v 列、-t 测试、-p、-c 写标准输出、-z 看注释、-Z 的 zipinfo）时交回解进哪个目录（-d 给的，没给是 "."）；不是解压交 None。"""
    directory, position = ".", 0
    while position < len(arguments):
        argument = arguments[position]
        position += 1
        if argument == "--":
            break
        if argument.startswith("--") or not argument.startswith("-") or argument == "-":
            continue
        letters = argument[1:]
        for index, letter in enumerate(letters):
            if letter in "lvtpczZ":
                return None
            if letter == "d":
                directory = letters[index + 1:]
                if not directory:
                    directory = arguments[position] if position < len(arguments) else "."
                    position += 1
                break
    return directory

def rsync_removal_steps(arguments, directory):
    """rsync 的 --delete（含 --delete-before 这一族，与 --del）：目的地里源里没有的文件被删；--remove-source-files（--remove-sent-files）：拷完删源，等于挪走；
    -n / --dry-run 什么都不做。弄坏开关 service-source-rsync-removal-unseen 下不认这两样。"""
    if detector_break_is_set("service-source-rsync-removal-unseen"):
        return []
    dry_run = any(argument == "--dry-run" or (argument.startswith("-") and not argument.startswith("--") and "n" in argument[1:]) for argument in arguments)
    operands = [argument for argument in arguments if not argument.startswith("-")]
    if dry_run or len(operands) < 2:
        return []
    steps = []
    if any(argument == "--del" or argument.startswith("--delete") for argument in arguments):
        steps.append(OverwriteCandidate("rsync --delete", operands[-1], directory))
    if any(argument in RSYNC_REMOVING_SOURCE_OPTIONS for argument in arguments):
        steps += [OverwriteCandidate("rsync --remove-source-files", source, directory) for source in operands[:-1]]
    return steps

def find_start_paths(arguments):
    """find 的起点目录：全局选项（-H -L -P、-D 选项、-O 级别）之后、第一个以 - 起头或是 ( ! ) , 的词之前的词；一个都没写是当前目录。"""
    position = 0
    while position < len(arguments) and (arguments[position] in ("-H", "-L", "-P") or arguments[position].startswith("-O")):
        position += 1
    if position < len(arguments) and arguments[position] == "-D":
        position += 2
    start = position
    while position < len(arguments) and not (arguments[position].startswith("-") or arguments[position] in ("(", ")", "!", ",")):
        position += 1
    return arguments[start:position] or ["."]

def find_path_word():
    """find 的 -exec 里 {} 在 lib_project_code_runs.py 里写成的占位词（读不到它时是空串）。"""
    library = project_code_runs_library()
    return getattr(library, "FIND_PATH_WORD", "") if library is not None else ""

def git_apply_directory(arguments):
    """git 的参数：是 `git [全局选项] apply …` 且真往工作区写（不是 --check、--stat、--numstat、--summary，也不是只写索引的 --cached）时，交回它在哪个目录里打补丁
    （-C 照写的顺序累计，没给是 "."）；否则交 None。"""
    position, directory = 0, "."
    while position < len(arguments) and arguments[position].startswith("-"):
        option = arguments[position].partition("=")[0]
        has_value_inline = "=" in arguments[position]
        position += 1
        if option in GIT_GLOBAL_OPTIONS_WITH_VALUE and not has_value_inline:
            if option == "-C" and position < len(arguments):
                directory = os.path.join(directory, arguments[position])
            position += 1
    if arguments[position:position + 1] != ["apply"]:
        return None
    rest = arguments[position + 1:]
    if any(argument in GIT_APPLY_OPTIONS_THAT_DO_NOT_WRITE for argument in rest):
        return None
    if "--cached" in rest and not any(argument in ("--index", "--3way", "-3") for argument in rest):
        return None
    return directory

def truncate_files(arguments):
    """truncate 的参数：交回它改的每个文件（-s / -r / -o 带值的选项跳过）。"""
    files, position, options_ended = [], 0, False
    while position < len(arguments):
        argument = arguments[position]
        position += 1
        if options_ended or not argument.startswith("-") or argument == "-":
            files.append(argument)
        elif argument == "--":
            options_ended = True
        elif argument in ("--size", "--reference"):
            position += 1
        elif not argument.startswith("--"):
            letters = argument[1:]
            value_letter = next((index for index, letter in enumerate(letters) if letter in "sr"), None)
            if value_letter is not None and value_letter == len(letters) - 1:
                position += 1
    return files

def in_place_editor_files(name, arguments):
    """sed -i / perl -i 改的文件（⑧ ⑨ ⑫ 用）；不带 -i 的交 []。sed 沿用 ⑧ 的 sed_in_place_targets，perl 的开关簇按 perl_in_place_files 认；
    两样都不交照文字就落在 /tmp/ 或工作根下的（in_place_target_in_scratch）。"""
    if name == "sed":
        return sed_in_place_targets(arguments)
    return [path for path in perl_in_place_files(arguments) if not in_place_target_in_scratch(path)]

MKTEMP_SUBSTITUTION = re.compile(r"^(?:\$\(\s*mktemp\s+([^()]*)\)|`\s*mktemp\s+([^`]*)`)$")

MKTEMP_CALL = r"(?:\$\(\s*mktemp\s+[^()]*\)|`\s*mktemp\s+[^`]*`)"
# `$(cd $(mktemp …) && pwd -P)`（`pwd`、`pwd -L`、`;` 代 `&&`、反引号写法）：整段、第 1 或 2 组是里面那条 mktemp 命令替换
MKTEMP_INSIDE_PWD = re.compile(r"^(?:\$\(\s*cd\s+\"?(" + MKTEMP_CALL + r")\"?\s*(?:&&|;)\s*pwd(?:\s+-[LP])?\s*\)"
                               r"|`\s*cd\s+\"?(" + MKTEMP_CALL + r")\"?\s*(?:&&|;)\s*pwd(?:\s+-[LP])?\s*`)$")
MKTEMP_IN_WORD = re.compile(r"\$\(\s*cd\s+\"?" + MKTEMP_CALL + r"\"?\s*(?:&&|;)\s*pwd(?:\s+-[LP])?\s*\)|" + MKTEMP_CALL)
SCRATCH_OWNER_IDENTITY = {}   # ⑫ 判一条命令时由 bash_scope_refusal 设成 hook 输入里的 agent_id、session_id、transcript_path；别处调 overwrite_steps 时空着
WRITER_LOCKS = []             # 同目录的 lib_writer_lock.py，第一次用到才导入；导入不了是 [None]

def writer_locks_library():
    if not WRITER_LOCKS:
        try:
            import lib_writer_lock
            WRITER_LOCKS.append(lib_writer_lock)
        except Exception as error:   # 文件不在、语法错：认不出自己的草稿目录，带目录的 mktemp 模板照旧算不出
            not_judged(f"bash-command-detector.sh 读不到 lib_writer_lock.py（{error!r}），mktemp 模板在不在自己的草稿目录里这一次没判")
            WRITER_LOCKS.append(None)
    return WRITER_LOCKS[0]

def own_scratch_directory(directory):
    """directory 落在这个子 agent（SCRATCH_OWNER_IDENTITY）认领的草稿目录里：工作根下第一层那个目录的归属文件 .owner 写的会话是 hook 输入的会话，
    而且离 directory 最近的写者锁归它（认领文件写的是它的 agent_id；还没有认领文件时，主会话记录里它的派发回执与锁对得上）。只读，不建认领文件。
    弄坏开关 mktemp-template-owner-unchecked 下工作根下的目录一律算。"""
    agent_id, session_id = SCRATCH_OWNER_IDENTITY.get("agent_id"), SCRATCH_OWNER_IDENTITY.get("session_id")
    if not agent_id or work_roots is None:
        return False
    directory = os.path.normpath(directory)
    for root in work_roots.scratch_roots():
        root = root.rstrip("/")
        if not directory.startswith(root + "/"):
            continue
        if detector_break_is_set("mktemp-template-owner-unchecked"):
            return True
        first_level = os.path.join(root, directory[len(root) + 1:].split("/")[0])
        try:
            owner_lines = open(os.path.join(first_level, ".owner"), encoding="utf-8").read().splitlines()
        except OSError:
            return False
        if not session_id or f"session: {session_id}" not in [line.strip() for line in owner_lines]:
            return False
        locks = writer_locks_library()
        lock_directory = locks.find_lock_directory(os.path.join(directory, "probe"), [root]) if locks is not None else None
        lock = locks.read_lock(lock_directory) if lock_directory is not None else None
        if lock is None:
            return False
        claim = locks.read_claim(lock_directory, lock)
        if claim is not None:
            return claim["agent_id"] == agent_id
        receipt = locks.launch_receipt(agent_id, SCRATCH_OWNER_IDENTITY.get("transcript_path"))
        return (receipt is not None and receipt[1] == lock.get("prompt_sha256")
                and receipt[0] >= lock["dispatched_at"] - locks.LAUNCH_SLACK_SECONDS)
    return False

def writer_lock_refusal(raw_path):
    """只带 Read、Bash 的定义（SCRATCH_OWNER_IDENTITY["read_only"]）写工作根下的草稿目录：离落点最近的写者锁（lib_writer_lock.find_lock_directory）在、
    持锁的不是这个子 agent（writer_verdict，与写闸第九道同一份）的拒，交一句说明；锁在而判不了谁持锁的也拒（与写闸同一口径）；离落点最近的没有锁而第一层目录的 .owner 是别的会话的也拒（other_session_owner）。
    自己持锁、没有锁（也不是别的会话的）、落点解不开、读不到写者锁库的交 None；回执还没落盘（unclaimed，不认领）的交 None 并记一条检出（弄坏开关 scope-unclaimed-receipt-unrecorded 下不记）。
    弄坏开关 scope-writer-lock-unchecked 下不判。"""
    agent_id = SCRATCH_OWNER_IDENTITY.get("agent_id")
    if not SCRATCH_OWNER_IDENTITY.get("read_only") or not agent_id or work_roots is None or detector_break_is_set("scope-writer-lock-unchecked"):
        return None
    locks = writer_locks_library()
    if locks is None:
        return None
    try:
        landings = write_scope.landing_forms(raw_path)
    except OSError:
        return None
    for landing in landings:
        scratch = locks.find_lock_directory(landing, work_roots.scratch_roots(), any_entry=True)
        if not scratch:
            other_owner = other_session_owner(landing)
            if other_owner:
                return other_owner
            continue
        state, lock = locks.lock_state(scratch)
        if state == "absent":
            continue
        if state == "unreadable":
            return f"写者锁：草稿目录 {scratch} 的写者锁在、却判不了谁持锁（{lock}）"
        verdict, holder = locks.writer_verdict(scratch, lock, agent_id, SCRATCH_OWNER_IDENTITY.get("transcript_path") or "")
        if verdict == "unclaimed" and not detector_break_is_set("scope-unclaimed-receipt-unrecorded"):
            noted = f"bash-command-detector.sh 读不到这个子 agent 的派发回执（主会话记录里没找到它，或记录读不了），草稿目录 {scratch} 的写者锁认不了谁持锁，这一次 Bash 里的写放行、没认领"
            if noted not in JUDGING_NOTICES:
                not_judged(noted)
        if verdict in ("holder", "unclaimed"):
            continue
        return f"写者锁：草稿目录 {scratch} 的写者锁由{'子 agent ' + holder if holder else '派发提示对得上那把锁的子 agent'}持有，不是这个子 agent"
    return None

def other_session_owner(landing):
    """落点所在的工作根下第一层目录没有写者锁、而归属文件 .owner（scripts/hosts/work-directory-ownership.sh 的三行：session、created、purpose，一行不多一行不少）写的会话不是 hook 输入的会话：
    交一句说明（别的会话的目录，没有写者锁也不许写）；没有归属文件、归属是这个会话的、归属文件写得不对（认不出是谁的）、hook 输入里没有会话号交 None。
    弄坏开关 scope-owner-only-directory-unjudged 下一概交 None。"""
    session_id = SCRATCH_OWNER_IDENTITY.get("session_id")
    if not session_id or work_roots is None or detector_break_is_set("scope-owner-only-directory-unjudged"):
        return None
    landing = os.path.normpath(landing)
    for root in work_roots.scratch_roots():
        root = root.rstrip("/")
        if not landing.startswith(root + "/"):
            continue
        first_level = os.path.join(root, landing[len(root) + 1:].split("/")[0])
        owner_path = os.path.join(first_level, ".owner")
        if os.path.islink(owner_path) or not os.path.isfile(owner_path):
            return None
        try:
            with open(owner_path, encoding="utf-8") as handle:
                text = handle.read()
        except (OSError, ValueError):
            return None
        lines = text.split("\n")[:-1] if text.endswith("\n") else text.split("\n")
        fields = {line.partition(":")[0]: (line.partition(": ")[2] if ": " in line else line) for line in lines}
        if len(lines) != 3 or not all(fields.get(key) for key in ("session", "created", "purpose")) or fields["session"] == session_id:
            return None
        return f"归属：草稿目录 {first_level} 的归属文件 .owner 写的会话是 {fields['session']}，不是这个会话，目录里又没有写者锁，别的会话的目录不许写"
    return None

# ⑫ 三方腿（定义 frontmatter 写了 leg: three-way 的类型，lib_three_way_legs.py）往 research/prompts/ 写，只许落在这一轮自己的报告与模型目录：轮名取派发回执里提示的行首「报告：research/prompts/<轮>-<模型>-output.md」那一行
#   （<轮>-<模型> 就是报告名去掉 -output.md；第二次同立场腿的报告名里带 -s2）。写范围表只认形态（research/prompts/*-opus-output.md），别的轮的报告与模型目录也在形态里，靠这一步收窄。
LEG_PROMPTS_PREFIX = "research/prompts/"
LEG_REPORT_LINE = re.compile(r"^\W*报告[:：]")
THREE_WAY_LEGS_LIBRARY = []     # 同目录的 lib_three_way_legs.py，第一次用到才导入；导入不了是 [None]
LEG_REPORT_BASES = {}           # (agent_id, 记录路径) → (报告名去掉 -output.md, None) 或 (None, 读不到的原因)：一条命令里几处写只读一遍主会话记录

def three_way_legs_library():
    if not THREE_WAY_LEGS_LIBRARY:
        try:
            specification = importlib.util.spec_from_file_location("lib_three_way_legs", os.path.join(HOOK_DIRECTORY, "lib_three_way_legs.py"))
            module = importlib.util.module_from_spec(specification)
            specification.loader.exec_module(module)
            THREE_WAY_LEGS_LIBRARY.append(module)
        except Exception as error:   # 文件不在、语法错
            not_judged(f"bash-command-detector.sh 读不到 lib_three_way_legs.py（{error!r}），三方腿往 research/prompts/ 写这一次没收窄到这一轮")
            THREE_WAY_LEGS_LIBRARY.append(None)
    return THREE_WAY_LEGS_LIBRARY[0]

def is_three_way_leg(repository_root, agent_type):
    legs = three_way_legs_library()
    return legs is not None and agent_type in legs.three_way_leg_types(repository_root)

def leg_report_base():
    """这个子 agent 的派发回执（主会话记录里 status 是 async_launched、agentId 对得上的那一条）里提示的行首「报告：」那一行点名的报告 research/prompts/<名>-output.md 的 <名>；
    交 (名, None)，读不到交 (None, 原因)：hook 输入里没有 agent_id 或记录、记录读不了、没找到回执、提示里没有这一行。"""
    agent_id, transcript_path = SCRATCH_OWNER_IDENTITY.get("agent_id"), SCRATCH_OWNER_IDENTITY.get("transcript_path")
    if (agent_id, transcript_path) not in LEG_REPORT_BASES:
        LEG_REPORT_BASES[(agent_id, transcript_path)] = read_leg_report_base(agent_id, transcript_path)
    return LEG_REPORT_BASES[(agent_id, transcript_path)]

def read_leg_report_base(agent_id, transcript_path):
    locks, legs = writer_locks_library(), three_way_legs_library()
    if not agent_id or not transcript_path or not os.path.isfile(transcript_path):
        return None, "hook 输入里没有 agent_id 或主会话记录"
    if locks is None or legs is None:
        return None, "读不到写者锁库或三方腿类型库"
    try:
        found = subprocess.run(["grep", "-F", "-e", agent_id, transcript_path], capture_output=True, text=True, timeout=locks.TRANSCRIPT_GREP_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired):
        return None, "主会话记录读不了"
    for line in found.stdout.splitlines():
        if '"async_launched"' not in line:
            continue
        try:
            result = json.loads(line).get("toolUseResult")
        except (ValueError, AttributeError):
            continue
        if not (isinstance(result, dict) and result.get("status") == "async_launched" and result.get("agentId") == agent_id and isinstance(result.get("prompt"), str)):
            continue
        for prompt_line in result["prompt"].splitlines():
            mention = legs.LEG_REPORT_MENTION.search(prompt_line) if LEG_REPORT_LINE.match(prompt_line) else None
            if mention:
                return mention.group(0)[len(LEG_PROMPTS_PREFIX):-len("-output.md")], None
        return None, "派发提示里没有行首「报告：research/prompts/<轮>-<模型>-output.md」一行"
    return None, "主会话记录里没找到这个子 agent 的派发回执"

def leg_round_problem(name):
    """name 是三方腿要写的 research/prompts/ 下的路径（相对那一层）：是这一轮自己的报告 <名>-output.md、模型目录 <名>-model 或它里面的文件时交 None，别的交一句说明；
    回执读不到（leg_report_base）时不收窄、交 None，并记一条检出（放行但必记检出）。"""
    base, reason = leg_report_base()
    if base is None:
        noted = f"bash-command-detector.sh 读不到三方腿的派发回执里「报告：」一行（{reason}），research/prompts/ 下这一次写只照写范围表判，没收窄到这一轮的报告与模型目录，照常执行"
        if noted not in JUDGING_NOTICES:
            not_judged(noted)
        return None
    if name in (f"{base}-output.md", f"{base}-model") or name.startswith(f"{base}-model/"):
        return None
    return f"三方腿只许写这一轮自己的报告 {LEG_PROMPTS_PREFIX}{base}-output.md 与模型目录 {LEG_PROMPTS_PREFIX}{base}-model/（轮名取自派发回执里「报告：」那一行），别的轮的报告与模型目录不是它的"

def mktemp_template_directory(value, directory=None):
    """`$(mktemp -d <模板>)`（或反引号写法）不带 -t、模板所在的目录（带目录的写法，或相对 directory 的）落在这个子 agent 认领的草稿目录里时，
    交回它建的目录（那个目录下一个名字）；别的写法、模板里还有 $ 与通配、目录不在自己的草稿目录里交 None。弄坏开关 mktemp-scratch-template-unknown 下一概交 None。"""
    if detector_break_is_set("mktemp-scratch-template-unknown"):
        return None
    match = MKTEMP_SUBSTITUTION.match(value.strip())
    if not match:
        return None
    arguments = (match.group(1) if match.group(1) is not None else match.group(2)).split()
    options = [argument for argument in arguments if argument.startswith("-")]
    operands = [argument for argument in arguments if not argument.startswith("-")]
    if len(operands) != 1 or not all(option == "--directory" or re.fullmatch(r"-[dq]+", option) for option in options):
        return None
    if not any(option == "--directory" or "d" in option for option in options) or re.search(r"[$`*?\[{]", operands[0]):
        return None
    template = shell_words.resolve_path(directory, os.path.expanduser(operands[0]))
    if template is None:
        return None
    parent = os.path.dirname(template)
    return os.path.join(parent, "mktemp-directory") if own_scratch_directory(parent) else None

def mktemp_inside_pwd(value):
    """`$(cd $(mktemp …) && pwd -P)`（`pwd`、`pwd -L`、`;` 代 `&&`、反引号写法同）：交回里面那条 mktemp 命令替换；不是这个形状交 None。
    弄坏开关 mktemp-nested-substitution-unknown 下一概交 None。"""
    if detector_break_is_set("mktemp-nested-substitution-unknown"):
        return None
    match = MKTEMP_INSIDE_PWD.match(value.strip())
    return (match.group(1) or match.group(2)) if match else None

def unquoted(word):
    """整个词被一对同样的引号包着的，去掉那一对。"""
    return word[1:-1] if len(word) >= 2 and word[0] == word[-1] and word[0] in "'\"" else word

def mktemp_option_directory(value, directory=None):
    """`$(mktemp -d -p <目录>)`、`--tmpdir=<目录>`（可再带一个不含 / 的模板名；`-p` 可与 -d、-q、-t 合写）：<目录>（相对 directory 的也算）落在这个子 agent 认领的草稿目录里时，
    交回它建的目录（那个目录下一个名字）；<目录> 带变量与通配、在 /tmp、仓里、别的子 agent 或别的会话的草稿目录里、别的选项、模板名带 / 都交 None。
    弄坏开关 mktemp-option-directory-unknown 下一概交 None。"""
    if detector_break_is_set("mktemp-option-directory-unknown"):
        return None
    match = MKTEMP_SUBSTITUTION.match(value.strip())
    if not match:
        return None
    arguments = (match.group(1) if match.group(1) is not None else match.group(2)).split()
    option_directory, rest, index = None, [], 0
    while index < len(arguments):
        argument = arguments[index]
        if re.fullmatch(r"-[dqt]*p", argument):
            if index + 1 >= len(arguments):
                return None
            option_directory = arguments[index + 1]
            if len(argument) > 2:
                rest.append(argument[:-1])
            index += 2
        elif argument.startswith("--tmpdir="):
            option_directory = argument[len("--tmpdir="):]
            index += 1
        else:
            rest.append(argument)
            index += 1
    if option_directory is None:
        return None
    options = [argument for argument in rest if argument.startswith("-")]
    operands = [unquoted(argument) for argument in rest if not argument.startswith("-")]
    if len(operands) > 1 or not all(option == "--directory" or re.fullmatch(r"-[dqt]+", option) for option in options):
        return None
    if not any(option == "--directory" or (not option.startswith("--") and "d" in option) for option in options):
        return None
    if operands and ("/" in operands[0] or re.search(r"[$`*?\[{]", operands[0])):
        return None
    option_directory = unquoted(option_directory)
    if not option_directory or re.search(r"[$`*?\[{]", option_directory):
        return None
    parent = shell_words.resolve_path(directory, os.path.expanduser(option_directory))
    if parent is None:
        return None
    return os.path.join(parent, "mktemp-directory") if own_scratch_directory(parent) else None

def mktemp_created_directory(value, directory=None):
    """一条 mktemp 命令替换（可套在 `$(cd $(mktemp …) && pwd -P)` 里，mktemp_inside_pwd）建出来的目录，落在这个子 agent 认领的草稿目录里时交回它的位置：
    `-p <目录>`、`--tmpdir=<目录>` 指的（mktemp_option_directory），或不带 -t 的模板所在的（mktemp_template_directory）；别的交 None。"""
    inner = mktemp_inside_pwd(value)
    if inner is not None:
        value = inner
    return mktemp_option_directory(value, directory) or mktemp_template_directory(value, directory)

def mktemp_substitutions_expanded(word, directory):
    """目标词里直接写的 `$(mktemp -d <模板>)`、`$(mktemp -d -p <目录>)`（可套在 `$(cd … && pwd -P)` 里），建的目录在自己认领的草稿目录里的，换成它建的目录（mktemp_created_directory）；别的照留。"""
    return MKTEMP_IN_WORD.sub(lambda match: mktemp_created_directory(match.group(0), directory) or match.group(0), word)

def mktemp_directory_value(value, directory=None):
    """赋的值是 `$(mktemp -d)`（或反引号写法，可带 -t 与不含 / 的模板）时，交回它建的目录落在哪：${TMPDIR:-/tmp} 下一个名字，仓外；
    不带 -t 的模板在这个子 agent 认领的草稿目录里时，交回草稿目录里的一个名字（mktemp_template_directory，directory 是相对模板的当前目录）；
    `-p <目录>`、`--tmpdir=<目录>` 指到自己认领的草稿目录里的同样（mktemp_option_directory），不带值的 `--tmpdir` 同 -t；套在 `$(cd $(mktemp …) && pwd -P)` 里的按里面那条算（mktemp_inside_pwd）；
    别的写法（不带 -d 的、-p 与 --tmpdir 指的目录不在自己的草稿目录里的、模板在别处的）交 None，变量照旧算不出。弄坏开关 mktemp-directory-unknown 下 ${TMPDIR:-/tmp} 那一种交 None。"""
    in_scratch = mktemp_created_directory(value, directory)
    if in_scratch is not None:
        return in_scratch
    if detector_break_is_set("mktemp-directory-unknown"):
        return None
    match = MKTEMP_SUBSTITUTION.match((mktemp_inside_pwd(value) or value).strip())
    if not match:
        return None
    arguments = (match.group(1) if match.group(1) is not None else match.group(2)).split()
    options = [argument for argument in arguments if argument.startswith("-")]
    operands = [argument for argument in arguments if not argument.startswith("-")]
    bare_tmpdir_known = not detector_break_is_set("mktemp-option-directory-unknown")
    if not all(option == "--directory" or (option == "--tmpdir" and bare_tmpdir_known) or re.fullmatch(r"-[dtq]+", option) for option in options):
        return None
    makes_directory = any(option == "--directory" or (not option.startswith("--") and "d" in option) for option in options)
    uses_template_in_temporary = any((not option.startswith("--") and "t" in option) or option == "--tmpdir" for option in options)
    if not makes_directory or len(operands) > 1 or (operands and (not uses_template_in_temporary or "/" in operands[0])):
        return None
    temporary = os.environ.get("TMPDIR") or "/tmp"
    return os.path.join(temporary if os.path.isabs(temporary) else "/tmp", "mktemp-directory")

def remember_working_directory(variables, directory):
    """cd 之后 $PWD 就是那个目录（算得出时记进 variables，后面的目标代入）；算不出时把之前记的作废。弄坏开关 pwd-variable-unexpanded 下不记。"""
    if detector_break_is_set("pwd-variable-unexpanded"):
        return
    if directory is None:
        variables.pop("PWD", None)
    else:
        variables["PWD"] = directory

def scope_launcher_inner_text(name, arguments, directory):
    """⑫ 用：eval 的代码（参数照 eval 的样子用空格接起来）与启动器（xargs、find -exec、parallel、flock、/usr/bin/time 这类，lib_project_code_runs.py 的 command_under_launcher）
    里面那条命令，交回一段命令文本，照命令接着判写到哪；xargs、find、parallel 读进来接进命令的词是没展开的占位词（LAUNCHER_INPUT_WORDS），落点照算不出拒。
    不是这几种的交 None。弄坏开关 scope-launcher-writes-unseen 下一律交 None（这几种起的写看不见）。"""
    if detector_break_is_set("scope-launcher-writes-unseen"):
        return None
    return launcher_inner_text(name, arguments, directory)

def launcher_inner_text(name, arguments, directory):
    """scope_launcher_inner_text 的判法本体（不看弄坏开关）；⑭ 认 find -exec 里写找到的路径也用它。"""
    if name == "eval":
        return " ".join(arguments) if arguments else None
    if name not in LAUNCHER_NAMES:   # 与 lib_project_code_runs.launcher_names() 同一份名单（自证「启动器名单与 lib_project_code_runs 的一致」）；不是启动器的不装那个库
        return None
    library = project_code_runs_library()
    if library is None:
        return None
    inner = library.command_under_launcher([name, *arguments], directory)
    if inner is None:
        return None
    inner_words, inner_directory = inner
    prefix = f"cd {shlex.quote(inner_directory)} && " if inner_directory else ""
    return prefix + " ".join(word if word in launcher_input_words() else shlex.quote(word) for word in inner_words)

def launcher_input_words():
    """xargs、find、parallel 读进来接进命令的词在 lib_project_code_runs.py 里写成的占位词（读不到它时是空的）。"""
    library = project_code_runs_library()
    if library is None:
        return ()
    return tuple(getattr(library, name) for name in ("XARGS_INPUT_WORD", "FIND_PATH_WORD", "PARALLEL_INPUT_WORD") if hasattr(library, name))

def command_pieces(tokens):
    """一层的记号按命令分隔符切成一段段：一段至多一条简单命令，连同它的重定向与替换正文；只有重定向、没有词的也成一段
    （`(…) > 文件`、`{ …; } > 文件` 右括号之后那段，共用模块的 simple_commands 不交它）。数组赋值 `名字=(…)` 的括号留在段里。
    交回 (段, 管道标记)，两个列表一一对应：标记为真的段，标准输入是前一段经 `|` 或 `|&` 送来的（`cat <<'EOF' | python3 -` 里 python3 那一段）。"""
    pieces, piped, after_pipe, piece, position = [], [], False, [], 0
    while position < len(tokens):
        is_operator, text = tokens[position]
        if (is_operator is False and shell_words.ARRAY_ASSIGNMENT_START.match(text)
                and tokens[position + 1:position + 2] == [(True, "(")]):
            end = position + 2
            while end < len(tokens) and tokens[end] != (True, ")"):
                end += 1
            piece += tokens[position:end + 1]
            position = end + 1
            continue
        if is_operator and text in shell_words.COMMAND_SEPARATORS:
            if piece:
                pieces.append(piece)
                piped.append(after_pipe)
            after_pipe = text in ("|", "|&") or (after_pipe and not piece and text == "\n")   # 管道符之后换行，下一段还是管道后一段
            piece = []
        else:
            piece.append(tokens[position])
        position += 1
    if piece:
        pieces.append(piece)
        piped.append(after_pipe)
    return pieces, piped

PRINTF_FORMAT_SLOT = re.compile(r"%[system_configuration]")

def piped_output_bodies(words, stdin_bodies):
    """一条命令交给管道的文本（包成 HeredocBody，管道后一段当它的标准输入）：cat（没有文件操作数）原样交出它读到的标准输入，echo、printf 交出自己的参数；
    别的命令交 ()——它们的输出算不出。stdin_bodies 是这条命令自己的标准输入（heredoc 或更前一段经管道送来的）。"""
    name, arguments = None, []
    for _, _, name, arguments in wrapped_commands(words):
        pass
    if name == "cat":
        operands = [argument for argument in arguments if not argument.startswith("-") or argument == "-"]
        return tuple(stdin_bodies[-1:]) if all(operand == "-" for operand in operands) else ()
    if name == "echo":
        start = next((index for index, argument in enumerate(arguments) if not re.fullmatch(r"-[neE]+", argument)), len(arguments))
        return (shell_words.HeredocBody("<<", True, " ".join(arguments[start:]).replace("\\n", "\n").replace("\\t", "\t") + "\n"),)
    if name == "printf" and arguments:
        values = iter(arguments[1:])
        text = PRINTF_FORMAT_SLOT.sub(lambda _: next(values, ""), arguments[0]).replace("\\n", "\n").replace("\\t", "\t")
        return (shell_words.HeredocBody("<<", True, text),)
    return ()

def target_pattern(word, unknown_mark="*"):
    """目标词里这一刻算不出的段（没赋过值的变量、命令替换、花括号展开）换成 unknown_mark（默认 *）：交回 (样式, 有没有算不出的段, 有没有通配)。
    unknown_mark 给一个不会出现在路径里的字符时，样式里还分得出哪些是算不出的段、哪些是原本的通配。"""
    pieces, position, has_unknown_piece, has_glob = [], 0, False, False
    while position < len(word):
        if word.startswith("$(", position) or word[position] == "`":
            position = shell_words.substitution_span(word, position)[0]
            pieces.append(unknown_mark)
            has_unknown_piece = True
            continue
        unknown = UNKNOWN_PIECE.match(word, position)
        if unknown:
            pieces.append(unknown_mark)
            has_unknown_piece = True
            position = unknown.end()
            continue
        has_glob = has_glob or word[position] in GLOB_CHARACTERS
        pieces.append(word[position])
        position += 1
    return "".join(pieces), has_unknown_piece, has_glob

def is_inside(path, directory):
    """path 在 directory 里面（不含 directory 自己）；按字面比、解开符号链接之后比，有一边在里面就算。"""
    return any(os.path.normpath(candidate).startswith(os.path.normpath(parent) + os.sep)
               for candidate, parent in ((path, directory), (os.path.realpath(path), os.path.realpath(directory))))

def destination_words(candidate, base):
    """这一步写到的目标词：重定向与 tee 是目标本身；cp / mv / install 写进目录时是 目录/每个源的文件名（源里的通配按这一刻的文件展开，
    一个都展不开的 cp 会报错、什么都不写；源里有算不出的段，文件名换成 *）。"""
    into_directory = candidate.into_directory
    if into_directory is None:
        pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(candidate.target))
        into_directory = candidate.target.endswith("/") or (
            not has_unknown_piece and not has_glob and os.path.isdir(shell_words.resolve_path(base, pattern)))
    if not into_directory:
        return [candidate.target]
    names = []
    for source in candidate.sources:
        pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(source))
        if has_unknown_piece:
            names.append("*")
        elif has_glob:
            names += [os.path.basename(match.rstrip("/")) for match in sorted(glob.glob(shell_words.resolve_path(base, pattern)))]
        else:
            names.append(os.path.basename(source.rstrip("/")))
    return [f"{candidate.target.rstrip('/')}/{name}" for name in names]

def resolve_destination(word, base, repository_root):
    """目标词按 base 解开：交回 (确定的绝对路径, [])；有算不出的段或通配时交回 (None, 样式在 research/results/ 下对得上的已有文件)，
    样式落不进 research/results/ 的交 (None, [])。`"$变量"/research/results/…` 这类变量在前的，从 research/results/ 那一段起按仓库根接。"""
    results_directory = os.path.join(repository_root, *RESULTS_DIRECTORY_WORDS)
    pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
    path = shell_words.resolve_path(base, pattern)
    if not has_unknown_piece and not has_glob:
        return path, []
    anchor = pattern.find("/".join(RESULTS_DIRECTORY_WORDS) + "/")
    if anchor > 0 and "*" in pattern[:anchor] and not is_inside(path, results_directory):
        path = os.path.join(repository_root, pattern[anchor:])
    if not is_inside(path, results_directory):
        return None, []
    return None, sorted(match for match in glob.glob(path) if os.path.isfile(match))

def preserved_paths(step, base):
    """PreservingStep 里的路径按 base 解开（通配按这一刻的文件展开，有算不出的段的不算）。"""
    paths = []
    for word in step.words:
        pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
        if has_unknown_piece:
            continue
        resolved = shell_words.resolve_path(base, pattern)
        paths += glob.glob(resolved) if has_glob else [resolved]
    return [os.path.normpath(path) for path in paths]

def is_preserved(path, preserved):
    """path 是同一条命令里前面 git add 过、或 mv 挪走了的那个文件（或在那样一个目录里面）。"""
    return any(os.path.normpath(path) == kept or is_inside(path, kept) for kept in preserved)

GIT_LS_FILES_RUNS = [0]   # 起了几次 git ls-files，自证数它

def run_git_ls_files(repository_root, arguments):
    GIT_LS_FILES_RUNS[0] += 1
    return subprocess.run(["git", "-C", repository_root, "ls-files", *arguments], capture_output=True, text=True, timeout=20)

def git_tracks(repository_root, path):
    """`git ls-files --error-unmatch`：进过 git 交 True，没进交 False，git 自己出错（不是仓、路径在仓外、超时）交 None。
    一份一份查，只给弄坏开关 per-path-git-query 用；正常走 tracked_verdicts 一次查完。"""
    try:
        completed = run_git_ls_files(repository_root, ["--error-unmatch", "--", path])
    except (OSError, subprocess.TimeoutExpired):
        return None
    return {0: True, 1: False}.get(completed.returncode)

def tracked_names(repository_root, paths):
    """一次 git ls-files 查这几份文件，交回进了 git 的那些（规范化的绝对路径集合）；git 自己出错交 None。"""
    try:
        completed = run_git_ls_files(repository_root, ["-z", "--full-name", "--", *paths])
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode != 0:
        return None
    top = os.path.realpath(repository_root)
    return {os.path.normpath(os.path.join(top, name)) for name in completed.stdout.split("\0") if name}

def tracked_verdicts(repository_root, paths):
    """几份已有的文件各自进没进 git，一次 git ls-files 查完 → {路径: True（进了）/ False（没进）/ None（git 自己出错）}。
    符号链接按链接本身判（只解析它所在的目录），与 git ls-files --error-unmatch 同义。"""
    paths = list(dict.fromkeys(paths))
    if not paths:
        return {}
    if detector_break_is_set("per-path-git-query"):   # 弄坏开关：一份一份查
        return {path: git_tracks(repository_root, path) for path in paths}
    tracked = tracked_names(repository_root, paths)
    if tracked is None:
        return dict.fromkeys(paths)
    return {path: os.path.normpath(os.path.join(os.path.realpath(os.path.dirname(path)), os.path.basename(path))) in tracked for path in paths}

def untracked_among(repository_root, paths):
    """几份已有的文件里没进 git 的，一次 git ls-files 查完；git 自己出错交 None。"""
    if not paths:
        return []
    tracked = tracked_names(repository_root, paths)
    if tracked is None:
        return None
    return [path for path in paths if os.path.normpath(os.path.realpath(path)) not in tracked]

def puts_itself_in_background(command):
    """检出 ②：放出追踪的那几种写法（SELF_DETACH，③ 拒了的不再记）与没等到的单独 &。单独的 & 与 ④ 同一套数法（unwaited_background_jobs 按层、按圆括号逐个数
    wait_coverage）：每个 & 记了 $!、之后逐个 `wait "$pN"` 的算等齐（主 agent 的 `( … ) & p1=$!; ( … ) & p2=$!; wait $p1; wait $p2` 不记）；
    引号里当数据写的 & 不算。弄坏开关 background-ampersand-by-regex 下按正则数（引号里的 & 照记，几个 & 而 `wait "$pid"` 逐个等的照记）。"""
    stripped = shell_words.strip_data_heredocs(command)
    if SELF_DETACH.search(stripped):
        return True
    if not detector_break_is_set("background-ampersand-by-regex"):
        return bool(unwaited_background_jobs(command))
    command = stripped
    if SUBSHELL_AMPERSAND.search(command):
        return True
    ampersands = list(LONE_AMPERSAND.finditer(command))
    for match in ampersands:
        rest = command[match.end():]
        if POLL_AFTER.search(rest):
            continue
        waits = [found.group(1).strip() for found in WAIT_COMMAND.finditer(rest)]
        if any(arguments == "" for arguments in waits):
            continue
        if len(ampersands) == 1 and any(arguments and not arguments.startswith("-n") for arguments in waits):
            continue
        return True
    return False

def findings_for(command, run_in_background=False):
    if detector_break_is_set("disable-check"):
        return []
    findings = []
    executed = command if detector_break_is_set("keep-heredoc-bodies") else shell_words.strip_data_heredocs(command)
    if WAIT_LOOP.search(executed) and not re.search(r"\btimeout\b", executed):
        findings.append("没有 timeout 的等待循环")
    if run_in_background and puts_itself_in_background(command) and not detector_break_is_set("disable-self-background"):
        findings.append("run_in_background 里又自己放后台（nohup / setsid / disown / 后面没有 wait 的 &）：完成通知当场发出，跑完的那个不会叫醒你")
    if run_in_background and not detector_break_is_set("bare-setsid-unrecorded"):
        findings += [f"run_in_background 里 {form} 不带 --wait：它起的那一刻若是进程组组长（包装 exec 过去、bash -c 只有这一条），setsid 就 fork 一份、自己马上退出，"
                     "完成通知当场发出；写成 setsid --wait（-w）" for form in bare_setsid_forms(command)]
    return findings

def record(hook_input, detections_path, extra_findings=()):
    """检出就追加一行；返回写了几行。永远不拦命令。extra_findings 是入口另外判出、只记不拦的（⑤ 目标算不出、git 判不了的，判不了的几样）。"""
    tool_input = hook_input.get("tool_input") or {}
    try:
        findings = findings_for(tool_input.get("command") or "", bool(tool_input.get("run_in_background")))
    except Exception as error:   # 判的时候抛了：照常执行，stderr 报一句、记进这一行检出
        message = not_judged(f"bash-command-detector.sh 没判成没超时的等待循环与后台里又放后台（{error!r}），这条命令照常执行")
        findings = [] if detector_break_is_set("failure-unrecorded") else [message]
    findings += list(extra_findings)
    if not findings:
        return 0
    append_detection(hook_input, findings, detections_path)
    return 1

def python_script_bodies(command, start_directory, script_roots=None):
    """⑧ 读的 python 脚本：命令位置上 `python3 <脚本文件> …`（共用模块交 launcher 为 python 的那一条，bash -c、前缀与包装里的照认）的脚本，
    路径此刻算得出（cd、前面赋过的变量、~、$HOME、$SINGLEFS_WORK_ROOT 代入）、是读得到的普通文件、落在 script_roots 的某一个底下（不给是工作根）、
    不超过 PYTHON_SCRIPT_READ_LIMIT_BYTES 的，交 [(写出来的脚本路径, 跑它时的当前目录, 正文)]。算不出、读不到、在别处的（仓里的定点写入脚本这类）不交。
    弄坏开关 python-script-body-unread 下一份都不交；python-script-any-location-read 下不看落在哪个根底下。"""
    if detector_break_is_set("python-script-body-unread"):
        return []
    roots = tuple(script_roots) if script_roots is not None else (tuple(work_roots.scratch_roots()) if work_roots else ())
    bodies = []
    for command_at in shell_words.commands_at_command_position(command, start_directory).commands:
        if command_at.launcher != "python" or not command_at.words:
            continue
        written = command_at.words[0]
        expanded = shell_words.expand_known_variables(work_roots.expand_written_path(written) if work_roots else written, command_at.variables or {})
        if expanded is None or re.search(r"[*?\[{]", expanded):
            continue
        path = shell_words.resolve_path(command_at.directory, os.path.expanduser(expanded))
        if path is None or not os.path.isfile(path):
            continue
        if not detector_break_is_set("python-script-any-location-read") and not any(
                is_inside(path, root) or is_inside(os.path.realpath(path), root) for root in roots):
            continue
        try:
            if os.path.getsize(path) > PYTHON_SCRIPT_READ_LIMIT_BYTES:
                continue
            with open(path, encoding="utf-8", errors="replace") as handle:
                bodies.append((written, command_at.directory, handle.read()))
        except OSError:
            continue
    return bodies

def is_main_workspace_crate_source(path, repository_root):
    return path.endswith(".rs") and is_inside(path, os.path.join(repository_root, "crates"))

def ledger_append_root(step, repository_root):
    """LedgerAppendTarget 要写的那个仓的根 → (根, None)；算不出 → (None, 原因)。--root 给的照它（取 realpath，同脚本）；没给的是脚本路径解开符号链接之后往上三层
    （脚本自己按 realpath(__file__) 的 scripts/kb/ 上两层认仓根）；直接执行只剩脚本名的按 repository_root（判的这个仓）。"""
    word = step.root if step.root is not None else step.script
    if word is None:
        return repository_root, None
    shown = f"--root {word}" if step.root is not None else f"脚本路径 {word}"
    pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(home_expanded(word)))
    if has_unknown_piece or has_glob:
        return None, f"{shown} 里有这一刻算不出的段（没赋过值或来源不明的变量、命令替换、通配）"
    if step.directory is None and not os.path.isabs(pattern):
        return None, f"前面有 cd 到这一刻算不出的目录，{shown} 落在哪算不出"
    path = os.path.realpath(pattern if os.path.isabs(pattern) else os.path.join(step.directory, pattern))
    return (path if step.root is not None else os.path.dirname(os.path.dirname(os.path.dirname(path)))), None

def known_write_script_steps(name, arguments, directory, variables):
    """命令是 KNOWN_WRITE_SCRIPTS 里的定点写脚本（python3 起它的，或直接执行它的）时，它要写的文件各交一个 OverwriteCandidate；读不出要写哪个文件的交 UnknownScriptTarget。
    name 是命令词的名字（basename；直接执行 `$S …` 时就是 `$S`，代入之后再取名字）。脚本词先按 variables（这条命令里前面赋过的值）代入再取名字比；
    python3 起的脚本词代入之后还有算不出的段、第一个不是选项的参数在、又不带不写的参数的，交 UnresolvedScriptTarget（落点由 bash_scope_verdict 判）。
    弄坏开关 scope-known-scripts-unseen 下一概交 []；scope-swap-spec-out-unseen 下不认 make-swap-spec.py 的 --out；scope-script-variable-unexpanded 下脚本词不代入、也不交 UnresolvedScriptTarget。"""
    if detector_break_is_set("scope-known-scripts-unseen"):
        return []
    substitute = not detector_break_is_set("scope-script-variable-unexpanded")
    launched_by_python = bool(shell_words.PYTHON_NAME.match(name))
    if launched_by_python:
        invocation = shell_words.python_invocation(arguments)
        if invocation.code_string is not None or invocation.module is not None or invocation.script_index is None or invocation.script_index >= len(arguments):
            return []
        script_word, rest = arguments[invocation.script_index], arguments[invocation.script_index + 1:]
    else:
        script_word, rest = name, arguments
    expanded_word = substitute_assigned_variables(script_word, variables) if substitute else script_word
    script = os.path.basename(expanded_word)
    if script not in KNOWN_WRITE_SCRIPTS:
        if substitute and launched_by_python and unresolved_script_word(home_expanded(expanded_word)):
            rest = [substitute_assigned_variables(argument, variables) for argument in rest]
            target = first_operand(rest)
            if target and not any(argument in NON_WRITING_FLAGS for argument in rest):
                return [UnresolvedScriptTarget(f"python3 {script_word}", target, directory)]
        return []
    rest = [substitute_assigned_variables(argument, variables) for argument in rest]
    if any(argument in NON_WRITING_FLAGS for argument in rest):
        return []
    if script == LEDGER_APPEND_SCRIPT:
        # 不带目标文件参数：写的是仓根下登记表 current 那一份账本；弄坏开关 scope-ledger-append-unseen 下不认它
        if detector_break_is_set("scope-ledger-append-unseen"):
            return []
        root = last_option_value(rest, "--root")
        return [LedgerAppendTarget(script, root, expanded_word if launched_by_python and root is None else None, directory)]
    if script == RECORD_APPEND_SCRIPT:
        # 写的是 <记录>（第一个不是选项的参数，--root 的值不算）；弄坏开关 scope-record-append-unseen 下不认它，scope-record-append-root-as-target 下把 --root 的值当记录
        if detector_break_is_set("scope-record-append-unseen"):
            return []
        options_with_value = () if detector_break_is_set("scope-record-append-root-as-target") else RECORD_APPEND_OPTIONS_WITH_VALUE
        record = first_operand(rest, options_with_value)
        return [OverwriteCandidate(script, record, directory)] if record else []
    if script == SWAP_SPEC_SCRIPT:
        # 写的是 --out 那一份规格（`--out 路径`、`--out=路径` 两种写法）；弄坏开关 scope-swap-spec-out-unseen 下不认它
        if detector_break_is_set("scope-swap-spec-out-unseen"):
            return []
        out = last_option_value(rest, "--out")
        return [OverwriteCandidate(script, out, directory)] if out else []
    if script == "replace-batch.py":
        specification = first_operand(rest)
        path = shell_words.resolve_path(directory, os.path.expanduser(specification)) if specification else None
        try:
            with open(path, encoding="utf-8") as handle:
                files = [item["file"] for item in json.load(handle) if isinstance(item, dict) and isinstance(item.get("file"), str)]
        except (OSError, TypeError, ValueError, KeyError):
            files = []
        if not files:
            return [UnknownScriptTarget(script, f"{script} 的规格 {specification or '（没给）'} 这一刻读不出要改哪些文件（不存在、不是 JSON 数组或没有 file）")]
        return [OverwriteCandidate(script, file, directory) for file in files]
    # 弄坏开关 scope-insert-row-absent-literal-as-target 下 --absent-literal 不当带值的选项，它的值被当成目标文件
    insert_row_options = INSERT_ROW_OPTIONS_WITH_VALUE - {"--absent-literal"} if detector_break_is_set("scope-insert-row-absent-literal-as-target") else INSERT_ROW_OPTIONS_WITH_VALUE
    target = first_operand(rest, insert_row_options if script == "insert-row.py" else ())
    return [OverwriteCandidate(script, target, directory)] if target else []

def service_directory_copy_hits(view, candidate, base):
    """cp -r / rsync 一步（OverwriteCandidate，form 是 cp、rsync、rsync --inplace）里，源是已存在的目录、目标是已存在的目录时会盖掉的已有服务源码（ServiceSourceView.overwritten_by_directory_copy）；
    源或目标带算不出的段、带通配的目标交 []，源带通配的逐个展开。弄坏开关 service-source-directory-copy-unseen 下一律交 []。"""
    if detector_break_is_set("service-source-directory-copy-unseen") or candidate.form not in ("cp", "rsync", "rsync --inplace") or not candidate.sources:
        return []
    target_text, target_unknown, target_glob = target_pattern(os.path.expanduser(candidate.target))
    if target_unknown or target_glob:
        return []
    destination = shell_words.resolve_path(base, target_text)
    if destination is None or not os.path.isdir(destination):
        return []
    hits = []
    for source in candidate.sources:
        source_text, source_unknown, source_glob = target_pattern(os.path.expanduser(source))
        if source_unknown:
            continue
        resolved = shell_words.resolve_path(base, source_text)
        if resolved is None:
            continue
        if candidate.form == "cp":
            contents_only = source.endswith("/.") or source == "." or candidate.into_directory is False
        else:
            contents_only = source.endswith("/") or source.endswith("/.") or source == "."
        for source_path in (sorted(glob.glob(resolved)) if source_glob else [resolved]):
            if os.path.isdir(source_path):
                hits += view.overwritten_by_directory_copy(source_path, destination, contents_only and not source_glob)
    return hits

def git_global_split(arguments):
    """git 之后的参数切成 (全局选项, 子命令, 子命令的参数)；全局选项里认不出的照放进去。没有子命令交 (…, None, [])。"""
    index = 0
    while index < len(arguments):
        word = arguments[index]
        if word in GIT_GLOBAL_OPTIONS_WITH_VALUE:
            index += 2
        elif word.startswith("-"):
            index += 1
        else:
            return arguments[:index], word, arguments[index + 1:]
    return arguments, None, []

def whole_index_reset_refusal(command, start_directory):
    """⑮：命令位置上（lib_shell_words 切，cd 与 git -C 跟着算）清整个索引的 git 命令，暂存区非空或判不了时交回认出的写法；主 agent 与子 agent 一样判。
    弄坏开关 allow-whole-index-reset 下一概放行。"""
    if detector_break_is_set("allow-whole-index-reset"):
        return []
    found = []
    for command_at in shell_words.commands_at_command_position(command, start_directory).commands:
        words = command_at.words
        if not words or os.path.basename(words[0]) != "git":
            continue
        global_options, subcommand, arguments = git_global_split(words[1:])
        unresolved = False
        if subcommand and ("$" in subcommand or "`" in subcommand) and not detector_break_is_set("git-subcommand-unexpanded-allowed"):
            # 子命令写成变量、命令替换（`x=reset; git $x`）：前面赋过字面值的照展开再判，展开不了的认不出是不是清索引，照清索引的那一类拒
            expanded = shell_words.expand_known_variables(subcommand, command_at.variables or {})
            if expanded is None:
                unresolved = True
            else:
                subcommand = expanded
        other_forms = not detector_break_is_set("whole-index-other-forms-unseen")
        if not unresolved and subcommand not in ("reset", "restore") and not (other_forms and subcommand in ("stash", "rm", "checkout")):
            continue
        directory, location = git_location(global_options, command_at.directory)
        if unresolved:
            clears = True
        elif subcommand == "reset":
            clears = git_reset_clears_index(arguments, directory, location)
        elif subcommand == "restore":
            clears = git_restore_clears_index(arguments)
        else:
            clears = git_other_form_clears_index(subcommand, arguments)
        if not clears:
            continue
        state = staged_index_state(location, directory)
        if state == "空":
            continue
        shown = " ".join(words)[:120]
        found.append(shown if state == "有" else f"{shown}（目录算不出，暂存区空不空判不了）")
    return list(dict.fromkeys(found))

def worktree_discard_refusal(command, start_directory):
    """⑱：命令位置上（lib_shell_words 切，cd 与 git -C 跟着算）带路径撤工作区改动的 git 命令，点名的路径里有没暂存的改动、或判不了时交回认出的写法；
    主 agent 与子 agent 一样判。弄坏开关 allow-path-discard 下一概放行。"""
    if detector_break_is_set("allow-path-discard"):
        return []
    found = []
    for command_at in shell_words.commands_at_command_position(command, start_directory).commands:
        words = command_at.words
        if not words or os.path.basename(words[0]) != "git":
            continue
        global_options, subcommand, arguments = git_global_split(words[1:])
        if subcommand not in ("checkout", "restore", "stash", "clean"):
            continue
        directory, location = git_location(global_options, command_at.directory)
        discard = worktree_discard_paths(subcommand, arguments, directory, location)
        if discard is None:
            continue
        shown = " ".join(words)[:120]
        paths = [shell_words.expand_known_variables(path, command_at.variables or {}) for path in discard.paths]
        problem = (discard.unresolved or ("路径写成展开不了的变量或命令替换" if None in paths else "")
                   or ("目录算不出" if directory is None else ""))
        if problem:
            found.append(f"{shown}（{problem}，判不了点名的路径有没有没暂存的改动）")
            continue
        hits = unstaged_hits(discard, [os.path.expanduser(path) for path in paths], location, directory)
        if hits is None:
            found.append(f"{shown}（git 起不来，判不了点名的路径有没有没暂存的改动）")
        elif hits:
            kind = "没跟踪的文件" if discard.untracked else "没暂存的改动"
            found.append(f"{shown}（有{kind}：{'、'.join(hits[:5])}{'…' if len(hits) > 5 else ''}）")
    return list(dict.fromkeys(found))
