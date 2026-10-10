"""E171（三oracle合谋测试） 比对器与驱动共用的结果行协议：每行以 `E7RESULT ` 起头，收尾 `E7RESULT name=done emitted=N`
（N 把 config 行与 done 行自己都算进去，同 research/scripts/replay.sh 的完整性闸 2）。不带耗时的行两次跑逐字节相同才登记得进 replay；
耗时行以 `E7TIMING ` 起头，不计数、不进入库产物。
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
