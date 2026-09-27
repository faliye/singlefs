"""defs-gatebatch-m2-r1 云端攻方（Opus）探针的共用部分：按路径导入被判的 admission.py，拿它自证里现成的假工具链、假 cargo 与小仓搭法。
被判的文件只读、只拷进临时目录，不改原件。每个探针的格写成「期望的行为」：打中的格在今天的代码上不成立，探针退 1。"""
import importlib.util
import os
import sys

REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
DEFAULT_ADMISSION = os.path.join(REPOSITORY, "research/scripts/admission.py")
DEFAULT_STAGE = os.path.join(REPOSITORY, ".claude/gate.d/54-layer0-replay.sh")
DEFAULT_SCRIPTS = os.path.join(REPOSITORY, "research/scripts")
DEFAULT_HOOK = os.path.join(REPOSITORY, ".claude/hooks/heavy-test-guard.sh")


def load_admission(path):
    spec = importlib.util.spec_from_file_location("admission_under_test", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def option(name, default):
    """--name <值>，没给取 default。"""
    arguments = sys.argv[1:]
    if name in arguments and arguments.index(name) + 1 < len(arguments):
        return os.path.abspath(arguments[arguments.index(name) + 1])
    return default


class Cells:
    """逐格记：kind 是 ATTACK（打中则这一格不成立）或 CONTROL（对照，今天应当成立）。"""

    def __init__(self, title):
        self.title = title
        self.rows = []

    def expect(self, kind, label, holds, detail):
        self.rows.append((kind, label, bool(holds), detail))
        state = "holds" if holds else "BROKEN"
        print(f"{kind}\t{state}\t{label}\t{detail}")

    def finish(self):
        broken_attacks = [row for row in self.rows if row[0] == "ATTACK" and not row[2]]
        broken_controls = [row for row in self.rows if row[0] == "CONTROL" and not row[2]]
        print(f"SUMMARY {self.title}: cells={len(self.rows)} attack_cells_broken={len(broken_attacks)} "
              f"control_cells_broken={len(broken_controls)}")
        # 对照格破了说明装置本身不对，退 2；只有攻击格破了（打中），退 1；全成立退 0
        if broken_controls:
            return 2
        return 1 if broken_attacks else 0
