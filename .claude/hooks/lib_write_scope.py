"""写范围表（agent-write-scope.tsv）的读法与判法：write-guard.sh（Write / Edit / MultiEdit）、bash-command-detector.sh（Bash 里的写）与 runner-dispatch-guard.sh（派发提示里点名的路径）用同一份，不各抄一份。

表的写法在 agent-write-scope.tsv 文件头：三列用制表符分隔（agent 名、路径模式、出处），路径模式相对仓根，以 / 开头的是绝对路径，** 跨目录、* 不跨目录，
以 $SINGLEFS_WORK_ROOT/ 开头的由 lib_work_root.py 展开成工作根与过渡期旧根各一条。模式里只有 ** 与 * 两个元字符，别的字符一律 re.escape。

  load_scopes(table_path) -> {agent 名: [路径模式…]}                                读表；读不了抛 OSError
  expanded_scope_patterns(patterns, work_roots, anchored=True) -> [展开后的模式…]    work_roots 是 lib_work_root 模块，读不到时给 None（$ 起头的模式丢掉，按拒判）
  glob_to_regex(pattern) -> 编译好的正则
  path_in_scope(expanded_patterns, absolute, project_root) -> bool                  absolute 是规范化的绝对路径；落在仓外时只认 / 起头的模式
  landing_forms(raw_absolute, lexical_only=False) -> [落点…]                        一个写的目标最终落在哪：规整 .. 并逐段解开符号链接（目标不存在时解到最深的已存在那一层再接上剩下的段）
  landings_out_of_scope(expanded_patterns, raw_absolute, project_root, lexical_only=False) -> [不在范围里的落点…]
                                                                                    落点的每一种认法都要在范围里；仓根与绝对模式的字面前缀也解开符号链接再比。解不开抛 OSError

bash-command-detector.sh ⑫ 的 bash_scope_verdict 与 write-guard.sh 第二道都调 landings_out_of_scope，落点一份判法；
它的弄坏开关 WRITE_GUARD_BREAK=landing-lexical-only 经 lexical_only 参数交进来（退回只按字面规整）。

弄坏开关不在这里：三个入口各有各的（write-guard.sh 的 WRITE_GUARD_BREAK=scope-root-unanchored 经 anchored 参数交进来；runner-dispatch-guard.sh 的
RUNNER_DISPATCH_GUARD_BREAK=scope-library-unread 当作读不到这份库）。
"""
import os
import re


def load_scopes(table_path):
    scopes = {}
    for line in open(table_path, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) >= 2 and fields[0].strip() and fields[1].strip():
            scopes.setdefault(fields[0].strip(), []).append(fields[1].strip())
    return scopes


def expanded_scope_patterns(patterns, work_roots, anchored=True):
    """写范围表里的模式：$SINGLEFS_WORK_ROOT/ 起头的展开成工作根与过渡期旧根各一条；读不到 lib_work_root.py（work_roots 是 None）时这样的模式丢掉（按拒判）。"""
    expanded = []
    for pattern in patterns:
        if pattern.startswith("$"):
            expanded += work_roots.expand_scope_pattern(pattern, anchored) if work_roots else []
        else:
            expanded.append(pattern)
    return expanded


def glob_to_regex(pattern):
    parts, index = [], 0
    while index < len(pattern):
        if pattern.startswith("**", index):
            parts.append(".*"); index += 2
        elif pattern[index] == "*":
            parts.append("[^/]*"); index += 1
        else:
            parts.append(re.escape(pattern[index])); index += 1
    return re.compile("^" + "".join(parts) + "$")


def path_in_scope(expanded_patterns, absolute, project_root):
    """规范化的绝对路径 absolute 命中展开后的某条模式：相对模式比相对仓根的写法（路径在仓外时没有，不比），绝对模式比绝对路径。"""
    root = os.path.normpath(project_root)
    relative = os.path.relpath(absolute, root) if absolute == root or absolute.startswith(root + os.sep) else None
    for pattern in expanded_patterns:
        target = absolute if pattern.startswith("/") else relative
        if target is not None and glob_to_regex(pattern).match(target):
            return True
    return False


def landing_forms(raw_absolute, lexical_only=False):
    """一个写的目标（绝对路径，可以没规整过）最终落在哪几种认法下：照 shell 逐段解开符号链接的（`链接/..` 回到链接指向那一层的上级），
    与先规整 `..` 再解开的（工具先规整路径再打开的走法）。目标或它往上几层不存在时，os.path.realpath 解到最深的已存在那一层、再接上剩下的段；
    悬空的链接照它指向的路径算。lexical_only 时只交字面规整的那一种（弄坏开关用：改前的认法）。解不开（读不了链接）抛 OSError。"""
    if lexical_only:
        return [os.path.normpath(raw_absolute)]
    return list(dict.fromkeys((os.path.realpath(raw_absolute), os.path.realpath(os.path.normpath(raw_absolute)))))


def resolved_pattern(pattern):
    """绝对模式的字面前缀（第一个 * 之前那一段里最后一个 / 之前的目录）解开符号链接之后的写法：模式里的根（工作根、旧根）本身是链接时，解开了的落点也对得上。"""
    star = pattern.find("*")
    static = pattern if star == -1 else pattern[:star]
    cut = static.rfind("/")
    if cut <= 0:
        return pattern
    return os.path.realpath(pattern[:cut]) + pattern[cut:]


def landings_out_of_scope(expanded_patterns, raw_absolute, project_root, lexical_only=False):
    """目标的每一种落点（landing_forms）都要命中展开后的某条模式才算在范围里；交回不在范围里的那几种落点（空列表就是都在）。
    相对模式比相对仓根的写法，仓根照字面与解开符号链接各认一次；绝对模式照字面与字面前缀解开符号链接各认一次。lexical_only 时只认字面（改前的样子）。"""
    roots = [os.path.normpath(project_root)] if lexical_only else list(dict.fromkeys((os.path.normpath(project_root), os.path.realpath(project_root))))
    patterns = list(expanded_patterns) if lexical_only else list(dict.fromkeys(
        list(expanded_patterns) + [resolved_pattern(pattern) for pattern in expanded_patterns if pattern.startswith("/")]))
    return [landing for landing in landing_forms(raw_absolute, lexical_only)
            if not any(path_in_scope(patterns, landing, root) for root in roots)]
