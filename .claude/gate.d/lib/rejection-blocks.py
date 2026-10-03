"""门禁格内嵌 python 逐块打印拒绝的共用一段：每块是 (摘要, 明细列表, 出路列表)，打成「✗ 摘要」、逐行明细、「→ 出路」。

用它的格（现数：`grep -ln 'rejection-blocks.py' .claude/gate.d/*.sh`）：code-source-discipline 的 reading-discipline、
checker-independence-and-sync 的 layout-checker-sync。两格原来各写一份逐字相同的循环
（.claude/singlefs-ai-sop/rules/sop-first.md「加门禁或钩子之前，先找已有的」第 3 条：只是共用一段逻辑，抽成共用的库）。
这一份只管打印，不带判据；判不判红、退出码由调用的格自己定。

调用（格的内嵌 python 里按路径导入，路径由格经参数传进来）：
    spec = importlib.util.spec_from_file_location("rejection_blocks", <这份文件的路径>)
    rejection_blocks = importlib.util.module_from_spec(spec); spec.loader.exec_module(rejection_blocks)
    rejection_blocks.print_blocks(failures, continuation_indent="     ")

出路列表里以「怎么办」开头的一条打成「     → 怎么办……」，其余各条是它的续行，前面垫 continuation_indent
（两格原来的续行缩进不同：layout-checker-sync 5 个空格，reading-discipline 7 个空格，照原样各传各的）。

弄坏开关 REJECTION_BLOCKS_BREAK=<项>。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（<阶段>/<fixtures 下的样本目录名>，
括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
  REJECTION_BLOCKS_BREAK=summaries-only  只打「✗ 摘要」，不打明细
      checker-independence-and-sync.sh/layout-checker-sync-first-registration-changed-value、
      checker-independence-and-sync.sh/layout-checker-sync-red（输出里找不到「改值 SAMPLE_UNIT_BYTES：16384 → 3…」）；
      code-source-discipline.sh/reading-discipline-red（输出里找不到「crates/singlefs-checker-tier/s…」）
  REJECTION_BLOCKS_BREAK=silent  一块都不打
      checker-independence-and-sync.sh/layout-checker-sync-first-registration-changed-value、
      checker-independence-and-sync.sh/layout-checker-sync-red（输出里找不到「格式常量在这次改动里变了」）；
      code-source-discipline.sh/reading-discipline-red（输出里找不到「C59（种子折叠成同一个状态）：2 处违规没有登记在豁免表里」）
"""
import os

BROKEN = os.environ.get("REJECTION_BLOCKS_BREAK", "")


def print_blocks(failures, continuation_indent="     "):
    """failures：[(摘要, [明细…], [出路…])]，按次序逐块打印；不退出，调用方打印完自己 sys.exit(1)。"""
    if BROKEN == "silent":
        return
    for summary, details, steps in failures:
        print(f"  ✗ {summary}")  # gate-lint:summary
        if BROKEN == "summaries-only":
            continue
        for detail in details:
            print(f"      {detail}")  # gate-lint:detail
        for step in steps:
            print(f"     → {step}" if step.startswith("怎么办") else f"{continuation_indent}{step}")
