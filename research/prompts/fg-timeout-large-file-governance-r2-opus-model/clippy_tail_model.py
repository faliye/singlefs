#!/usr/bin/env python3
"""第 10 问：㉓ 出路里 cargo clippy 的 lint 段（检出钩子 clippy_lint_tail）在 check.sh 读不到、表为空、表的写法变了时给成什么样，
照抄交上去 project-code-guard.sh 的 ⑤（code_discipline_lints、clippy_denied_names）拒不拒，与门禁真跑的 clippy 差几条。

用法：python3 clippy_tail_model.py --frozen <被判的那棵树> --work <新的空目录>
  两份钩子的那几段原样从源文件里取出、在本进程里执行（检出钩子的 detector_break_is_set 恒假、not_judged 只收进列表；
  闸的 CLIPPY_LINT_TABLE_SOURCE 换成每种写法的那一份 check.sh）。门禁真跑的那一份照 bash 自己展开数组（bash -c 定义数组再逐个打出）。
输出：每种写法一行 `CLIPPY name=<写法> …`，末行 `CLIPPY name=done emitted=<行数>`。
  outlet_lints 出路里 -D 点名的个数（不含 warnings）；guard_judges 闸这一刻判不判 ⑤；guard_missing 照抄出路时闸点名缺的；
  gate_minus_outlet 门禁真跑的 -D 里出路没有的；recorded 检出钩子记了几条判不了。
退出码：0 跑完；1 取不出那几段；2 用法错。
"""
import argparse
import json
import os
import re
import subprocess
import sys

EMITTED = 0
HEADER = "#!/usr/bin/env bash\n"
CURRENT = ("CODE_DISCIPLINE_LINTS=(\n  -D clippy::wildcard_enum_match_arm          # 说明\n  -D clippy::allow_attributes_without_reason  # 说明\n"
           "  -D clippy::cast_possible_truncation\n)\n")
VARIANTS = (
    ("table-as-today", CURRENT),
    ("table-empty", "CODE_DISCIPLINE_LINTS=(\n)\n"),
    ("table-readonly-prefix", CURRENT.replace("CODE_DISCIPLINE_LINTS=(", "readonly CODE_DISCIPLINE_LINTS=(")),
    ("table-declare-prefix", CURRENT.replace("CODE_DISCIPLINE_LINTS=(", "declare -a CODE_DISCIPLINE_LINTS=(")),
    ("table-first-item-on-opening-line", CURRENT.replace("CODE_DISCIPLINE_LINTS=(\n  -D", "CODE_DISCIPLINE_LINTS=( -D")),
    ("table-two-lints-on-one-line", CURRENT.replace("  -D clippy::cast_possible_truncation\n", "  -D clippy::cast_possible_truncation -D clippy::cast_sign_loss\n")),
    ("table-attached-deny", CURRENT.replace("  -D clippy::cast_possible_truncation\n", "  -Dclippy::cast_possible_truncation\n")),
    ("table-long-deny", CURRENT.replace("  -D clippy::cast_possible_truncation\n", "  --deny clippy::cast_possible_truncation\n")),
    ("table-rustc-lint-added", CURRENT.replace(")\n", "  -D unsafe_code\n)\n")),
    ("table-closing-paren-indented", CURRENT.replace("\n)\n", "\n  )\n")),
    ("table-quoted-items", CURRENT.replace("-D clippy::cast_possible_truncation", "\"-D\" \"clippy::cast_possible_truncation\"")),
)


def emit(name, **fields):
    global EMITTED
    EMITTED += 1
    print("CLIPPY " + " ".join([f"name={name}"] + [f"{key}={json.dumps(value, ensure_ascii=False)}" for key, value in fields.items()]), flush=True)


def block(source, start_marker, end_marker):
    return source[source.index(start_marker):source.index(end_marker)]


def gate_words(path):
    """门禁那一份照 bash 展开的数组；bash 跑不成交 None。"""
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    found = re.search(r"^(?:readonly |declare -a )?CODE_DISCIPLINE_LINTS=\(.*?^\s*\)", text, re.M | re.S)
    if not found:
        return None
    completed = subprocess.run(["bash", "-c", found.group(0) + "\nprintf '%s\\n' \"${CODE_DISCIPLINE_LINTS[@]}\""], capture_output=True, text=True)
    return completed.stdout.splitlines() if completed.returncode == 0 else None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    os.makedirs(arguments.work, exist_ok=True)
    with open(os.path.join(arguments.frozen, ".claude", "hooks", "bash-command-detector.sh"), encoding="utf-8") as handle:
        detector = handle.read()
    with open(os.path.join(arguments.frozen, ".claude", "hooks", "project-code-guard.sh"), encoding="utf-8") as handle:
        guard = handle.read()
    notes = []
    detector_space = {"os": os, "re": re, "detector_break_is_set": lambda item: False, "not_judged": notes.append}
    guard_space = {"os": os, "re": re}
    try:
        exec(compile(block(detector, "CLIPPY_LINT_TABLE_RELATIVE = ", "def whole_target_build_messages"), "detector_block", "exec"), detector_space)
        exec(compile(block(guard, "CLIPPY_LINT_TABLE_START = ", "def clippy_lint_table_missing"), "guard_block", "exec"), guard_space)
    except (ValueError, SyntaxError) as error:
        emit("setup-error", error=repr(error)[:300])
        print(f"CLIPPY name=done emitted={EMITTED}", flush=True)
        return 1
    cases = [("check-script-missing", None)] + list(VARIANTS)
    for name, table in cases:
        path = os.path.join(arguments.work, f"check-{name}.sh")
        if table is not None:
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(HEADER + table)
        del notes[:]
        tail = detector_space["clippy_lint_tail"](path)
        guard_space["CLIPPY_LINT_TABLE_SOURCE"] = path
        guard_lints = guard_space["code_discipline_lints"]()
        outlet_words = tail.split()[1:] if tail else []
        outlet_denied = guard_space["clippy_denied_names"](outlet_words)
        guard_missing = [] if guard_lints is None else [item for item in ["warnings"] + guard_lints if item not in outlet_denied]
        gate = gate_words(path) if table is not None else None
        gate_denied = guard_space["clippy_denied_names"](gate) if gate else set()
        emit(name, outlet_lints=len([item for item in outlet_denied if item != "warnings"]), guard_judges="yes" if guard_lints is not None else "no",
             guard_missing=guard_missing, gate_lints=len(gate_denied), gate_minus_outlet=sorted(gate_denied - outlet_denied - {"warnings"}),
             recorded=len(notes), verdict="outlet-refused-by-guard" if guard_missing else "outlet-passes-guard")
    print(f"CLIPPY name=done emitted={EMITTED}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
