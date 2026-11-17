# 绿样本：自证函数体里设基准变量喂共用取法、在临时仓里取基准核对，是造的输入，不判
import os
import subprocess


def base_shape_selftest(root):
    saved = os.environ.pop("GATE_BASE", None)
    os.environ["GATE_BASE"] = "HEAD~1"
    try:
        expected = subprocess.run(["git", "-C", root, "merge-base", "HEAD", "@{upstream}"], capture_output=True, text=True).stdout
        listed = subprocess.run(["git", "-C", root, "ls-files"], capture_output=True, text=True).stdout
    finally:
        os.environ.pop("GATE_BASE", None)
        if saved is not None:
            os.environ["GATE_BASE"] = saved
    return expected, listed
