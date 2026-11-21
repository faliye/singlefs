#!/usr/bin/env python3
"""第 9 问：检出钩子开关名单三处一致那一格（break_switch_name_sets、break_switch_set_mismatches）对文件头写法变化判成什么。

用法：python3 name_sets_model.py --frozen <被判的那棵树>
  从被判的检出钩子源文件里原样取出 BREAK_SWITCH_KNOWN_TUPLE 到「弄坏开关只在自证里认」注释之前那一段代码（两个函数与它们的正则），
  在本进程里执行（detector_break_is_set 换成恒假：不开弄坏开关），再拿同一份源文件的几种改写喂 break_switch_set_mismatches。
输出：每种改写一行 `NAMES name=<改写> mismatches=<条数> detail=<前两条>`，末行 `NAMES name=done emitted=<行数>`。
结局读法：改写后的源文件里开关确实三处一致而 mismatches>0 是误判（响红）；确实不一致（有开关只在一处、或文档写着不存在的开关）而 mismatches=0 是漏判（静默）。
退出码：0 跑完；1 取不出那一段或某种改写没改上（锚串不在）；2 用法错。
"""
import argparse
import json
import os
import re
import sys

EMITTED = 0


def emit(name, **fields):
    global EMITTED
    EMITTED += 1
    print("NAMES " + " ".join([f"name={name}"] + [f"{key}={json.dumps(value, ensure_ascii=False)}" for key, value in fields.items()]), flush=True)


def load_checker(source):
    start = source.index("BREAK_SWITCH_KNOWN_TUPLE = re.compile")
    end = source.index("# 弄坏开关只在自证里认：自证起的子进程靠这个变量")
    namespace = {"re": re, "detector_break_is_set": lambda item: False}
    exec(compile(source[start:end], "break_switch_block", "exec"), namespace)
    return namespace["break_switch_set_mismatches"]


def replaced(source, old, new, count=1):
    if source.count(old) < count:
        raise ValueError(f"锚串不在：{old[:80]}")
    return source.replace(old, new, count)


def line_starting(source, prefix):
    for line in source.split("\n"):
        if line.startswith(prefix):
            return line
    raise ValueError(f"没有以 {prefix[:60]} 起头的行")


def variants(source):
    """(改写名, 改写后的源文件, 改写后开关是不是确实三处一致)。"""
    push_line = line_starting(source, "#   BASH_COMMAND_DETECTOR_BREAK=allow-large-file-push  ")
    all_line = line_starting(source, "#   BASH_COMMAND_DETECTOR_BREAK=commit-all-flag-unseen  ")
    all_text = all_line[len("#   BASH_COMMAND_DETECTOR_BREAK=commit-all-flag-unseen  "):]
    found = []
    found.append(("baseline", source, True))
    two_on_one = replaced(replaced(source, push_line + "\n", push_line.replace("=allow-large-file-push  ", "=allow-large-file-push,commit-all-flag-unseen  ") + "\n"), all_line + "\n", "")
    found.append(("table-two-items-comma-on-one-line", two_on_one, True))
    either = replaced(replaced(source, push_line + "\n", push_line.replace("=allow-large-file-push  ", "=allow-large-file-push 或 BASH_COMMAND_DETECTOR_BREAK=commit-all-flag-unseen  ") + "\n"), all_line + "\n", "")
    found.append(("table-two-items-or-on-one-line", either, True))
    found.append(("table-name-in-backquotes", replaced(source, all_line, "#   BASH_COMMAND_DETECTOR_BREAK=`commit-all-flag-unseen`  " + all_text), True))
    found.append(("table-name-on-continuation-line", replaced(source, all_line, "#   BASH_COMMAND_DETECTOR_BREAK=\n#     commit-all-flag-unseen  " + all_text), True))
    found.append(("table-two-space-indent", replaced(source, all_line, "#  BASH_COMMAND_DETECTOR_BREAK=commit-all-flag-unseen  " + all_text), True))
    found.append(("header-prose-line-looks-like-entry", replaced(source, all_line, all_line + "\n#   git-commit-a  这一行是说明文字，不是开关"), True))
    read_anchor = "def detector_break_is_set(item):"
    single_quoted = replaced(source, read_anchor, "def probe_reads_undocumented_switch():\n    return detector_break_is_set('probe-undocumented-switch')\n\n" + read_anchor)
    found.append(("code-read-single-quoted-undocumented", single_quoted, False))
    through_name = replaced(source, read_anchor, "PROBE_SWITCH = \"probe-undocumented-switch\"\ndef probe_reads_undocumented_switch():\n    return detector_break_is_set(PROBE_SWITCH)\n\n" + read_anchor)
    found.append(("code-read-through-a-name-undocumented", through_name, False))
    bash_equal = replaced(source, "HOOK_DIR=", "[[ \",${BASH_COMMAND_DETECTOR_BREAK:-},\" == *,probe-undocumented-switch,* ]] && true\nHOOK_DIR=")
    found.append(("bash-layer-equal-form-undocumented", bash_equal, False))
    removed = source
    for old, new in (("\"commit-all-flag-unseen\", ", ""), (all_line + "\n", ""),
                     ("state[\"all\"] and not detector_break_is_set(\"commit-all-flag-unseen\")", "state[\"all\"]")):
        removed = replaced(removed, old, new)
    found.append(("switch-removed-but-still-in-section-list", removed, False))
    return found


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    arguments = parser.parse_args()
    hook = os.path.join(arguments.frozen, ".claude", "hooks", "bash-command-detector.sh")
    with open(hook, encoding="utf-8") as handle:
        source = handle.read()
    try:
        mismatches_of = load_checker(source)
        cases = variants(source)
    except (ValueError, KeyError, SyntaxError) as error:
        emit("setup-error", error=repr(error)[:300])
        print(f"NAMES name=done emitted={EMITTED}", flush=True)
        return 1
    for name, text, consistent in cases:
        found = mismatches_of(text)
        if consistent:
            verdict = "correct" if not found else "false-mismatch"
        else:
            verdict = "missed-silently" if not found else "caught"
        emit(name, mismatches=len(found), truly_consistent="yes" if consistent else "no", verdict=verdict, detail=found[:2])
    print(f"NAMES name=done emitted={EMITTED}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
