"""E200（修订6近似树重跑） 各脚本共用的结果行协议。

出处：照抄 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/result_lines.py（提交 59ce5a865 那一版，sha256
7f273b61a1b8f2bbdaa37898d57a007595bea320dbe9a8a6df0b4e357fa30de5），只改了这一段文件头：E171（三oracle合谋测试） 研究一侧那份有按 sha256 的钉链，
E200 不载入、不改它（登记 research/prompts/e200-preregistration.md 第五节 5.4）。

每行以 `E7RESULT ` 起头，收尾 `E7RESULT name=done emitted=N`（N 把收尾行自己也算进去，同 scripts/experiment-apparatus/replay.sh 的完整性闸 2）。
"""

from __future__ import annotations

import os
import sys


class ResultLines:
    """往一个新文件（已存在就拒绝）或标准输出写结果行；值里的空白换成下划线，按空格切字段时不会切错。"""

    def __init__(self, output_path: str | None):
        if output_path is not None and os.path.exists(output_path):
            raise FileExistsError(f"产物 {output_path} 已存在，不覆盖")
        self.handle = open(output_path, "x", encoding="utf-8") if output_path is not None else sys.stdout
        self.emitted = 0

    def result(self, body: str) -> None:
        self.emitted += 1
        self.handle.write(f"E7RESULT {body}\n")
        self.handle.flush()

    def timing(self, body: str) -> None:
        self.handle.write(f"E7TIMING {body}\n")
        self.handle.flush()

    def finish(self) -> None:
        self.emitted += 1
        self.handle.write(f"E7RESULT name=done emitted={self.emitted}\n")
        self.handle.flush()
        if self.handle is not sys.stdout:
            self.handle.close()


def field(value: object) -> str:
    """结果行里的一个值：空白换成下划线，空串写 `-`。"""
    text = str(value)
    return "".join("_" if character.isspace() else character for character in text) or "-"
