#!/usr/bin/env python3
"""E203（准入之前扫类的步代价）量的那一路：在拷来的实现上只插计数或只插计时，不改任何判定。

这个文件是入口：放变异检查、弄坏开关的判断与 `main`；其余在同目录 `e203_admission_scan_cost_counters_<部分>.py` 四份（行表与读数模块文字、
插入的编辑表、套用与拷副本、自证），入口开头把它们里的名字原样再导出（`e206_fixed_point_cost_counters.py`、`e207_strict_scan_steps_counters.py` 按 `e203.<名字>` 取）。

跑前登记 research/prompts/e203-preregistration.md 第五节「装置写在哪」第 1 条。

用法：
  python3 e203_admission_scan_cost_counters.py --build counts|timing --source <仓根> --into <工作目录> [--diff-out <文件>]
  python3 e203_admission_scan_cost_counters.py --check-anchors <树>      只核锚点、打每个锚点的行号，不写；锚点全对之后再不起编译地判一遍：插完的每行读数语句要落在语句层（前一个字符是 ; { }、最内层括号是 {，不在表达式、实参表、链式调用中间），改过的 .rs 括号要配平，不过退 3、打 name=insertion_problem / name=balance_problem 行
  python3 e203_admission_scan_cost_counters.py --selftest [--histories <历史用例>]   用一棵小的假树证每个分支（不编实现）；
                                                                         另核历史用例里意图那次规模读数的取法（不给取本目录那一份）
  python3 e203_admission_scan_cost_counters.py --mutation-check <变异表>   逐条改坏、跑自证，没红的报 missed
  python3 e203_admission_scan_cost_counters.py --write-fake-source <新目录>   写一棵小的假仓根（入口脚本的自证用）
退出码：0 成；2 用法错或文件不在；3 有锚点命中不是恰好一次（一个字节不写，登记停机条款第 1 条）；4 自证有一格不对。

--build counts：只插计数与段切换，不插计时（模块 e203_scan_counts）。
--build timing：只插计时与段切换，不插计数（模块 e203_scan_timing），并去掉副本里 harness 的 lookup-work-counters 特性。
两种构建的副本里都有 `pub use … as e203_scan;`，历史用例只写 e203_scan::…，两边同一份源文件。
每个锚点在它那个函数体里必须恰好命中一次，否则退 3。
同一条命令被打断（内存上限杀、手动停）之后重起就是续跑：副本目录里先写开拷标记，拷完、改完才写完成标记（内容是输入键）。
再起时：完成标记的键与这一次的输入键相同，原样用这份副本；有开拷标记而没有完成标记（半拷）或键不同，清掉重拷；
非空而没有开拷标记（不是本脚本拷的）拒绝、不清，退 2。
弄坏开关（只给自证用，平常跑时不认；入口脚本的自证也经它用到）：
  E203_RUN_BREAK=copy-resume-off  不认完成标记与开拷标记，非空目录一律拒（回到不能续跑的写法）
      a_finished_copy_with_the_same_key_is_reused、a_finished_copy_whose_inputs_changed_is_cleared、a_copy_after_the_clearing_has_the_new_content、
      a_half_copy_is_cleared_and_copied_again、a_half_copy_ends_up_complete_after_the_second_try、
      a_copy_without_the_done_marker_is_not_trusted、a_begun_marker_cut_off_while_it_was_written_is_still_ours
      （失败行句首 name=selftest_case case=<这些名字> result=FAILED）
"""
import os
import pathlib
import sys

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, SCRIPT_DIRECTORY)  # 拆出的各部分在入口旁边；`-I` 起的进程不带脚本目录，这里补上
from e203_admission_scan_cost_counters_row_table_and_module_template import *  # noqa: E402,F401,F403
from e203_admission_scan_cost_counters_insertion_edits import *  # noqa: E402,F401,F403
from e203_admission_scan_cost_counters_applying_edits_and_copying_the_tree import *  # noqa: E402,F401,F403
from e203_admission_scan_cost_counters_selftest import *  # noqa: E402,F401,F403


def mutation_check(table):
    """对变异表逐条跑：把原文换成替换文写到临时目录，跑 --selftest，必须退非 0 才算抓到。

    每行三段（名、原文、替换文）改的是本脚本（入口与拆出的各部分合起来，原文在这几份文件里合计恰好命中一次，改的是命中的那一份）；
    第四段写 `histories` 的改的是历史用例 `e203_admission_scan_cost_histories.rs`，本脚本原样、自证经 --histories 读改过的那一份。
    自证一律经 --histories 指到历史用例（改过的或仓里那一份），临时目录里的本脚本旁边没有历史用例。"""
    import subprocess
    import tempfile
    directory_of_the_script = pathlib.Path(__file__).resolve().parent
    script_sources = {name: (directory_of_the_script / name).read_text(encoding="utf-8") for name in SCRIPT_FILE_NAMES}
    sources = {"script": "".join(script_sources.values()), "histories": HISTORIES.read_text(encoding="utf-8")}
    missed = 0
    for line in pathlib.Path(table).read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        name, original, replacement = fields[:3]
        target = fields[3] if len(fields) == 4 else "script"
        if target == "histories":
            # 历史用例是 Rust：表里的 \n（反斜杠加 n）还原成换行，好写跨行的原文
            original, replacement = original.replace("\\n", "\n"), replacement.replace("\\n", "\n")
        # 改本脚本的那几行：原文与替换文逐字取自本脚本的源码（源码里的 \n 是反斜杠加 n 两个字符），不做转义还原
        source = sources[target]
        if source.count(original) != 1:
            print(f"name=counters_mutation id={name} result=anchor_lost hits={source.count(original)}")
            missed += 1
            continue
        with tempfile.TemporaryDirectory() as directory:
            histories = pathlib.Path(directory) / "histories.rs"
            for name, text in script_sources.items():
                (pathlib.Path(directory) / name).write_text(text.replace(original, replacement) if target == "script" else text, encoding="utf-8")
            mutated = pathlib.Path(directory) / SCRIPT_FILE_NAMES[0]
            histories.write_text(source.replace(original, replacement) if target == "histories" else sources["histories"], encoding="utf-8")
            outcome = subprocess.run([sys.executable, "-I", str(mutated), "--selftest", "--histories", str(histories)],
                                     capture_output=True, text=True, check=False)
        caught = outcome.returncode != 0
        missed += 0 if caught else 1
        print(f"name=counters_mutation id={name} result={'caught' if caught else 'missed'} exit={outcome.returncode}")
    print(f"name=counters_mutation_done missed={missed}")
    return 0 if missed == 0 else 4


def break_switch_is_allowed(argv):
    """弄坏开关只在自证里认：命令行就是 --selftest，或入口脚本的自证设了 E203_SELFTEST_MARK 指到它建的标记文件（内容对得上）。别处设了拒。"""
    if not os.environ.get("E203_RUN_BREAK") or "--selftest" in argv:
        return True
    mark = os.environ.get("E203_SELFTEST_MARK")
    try:
        return bool(mark) and pathlib.Path(mark).read_text(encoding="utf-8").strip() == "e203-selftest"
    except OSError:
        return False


def main(argv):
    if not break_switch_is_allowed(argv):
        print("  ✗ E203_RUN_BREAK 只给自证证红用，这一趟不是自证起的\n"
              "     → 怎么办：去掉 E203_RUN_BREAK 再跑；要证红就跑 E203_RUN_BREAK=<项> python3 -I e203_admission_scan_cost_counters.py --selftest", file=sys.stderr)
        return 2
    args, index = {}, 0
    while index < len(argv):
        if argv[index] in ("--selftest",):
            args[argv[index]] = True
            index += 1
        elif argv[index].startswith("--") and index + 1 < len(argv):
            args[argv[index]] = argv[index + 1]
            index += 2
        else:
            print(__doc__, file=sys.stderr)
            return 2
    if "--selftest" in args:
        return selftest(args.get("--histories", HISTORIES))
    if "--check-anchors" in args:
        return check_anchors(args["--check-anchors"])
    if "--write-fake-source" in args:
        write_fake_source(args["--write-fake-source"])
        return 0
    if "--mutation-check" in args:
        return mutation_check(args["--mutation-check"])
    if all(key in args for key in ("--build", "--source", "--into")):
        return build(args)
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
