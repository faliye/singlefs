"""绿样本：模块与函数的文档串里写到这几种写法，都是说明，不是代码。

gate 基准有上游时取上游的 merge-base，推完之后取 HEAD~1，GATE_BASE 给了取它；
改动范围经 gate_changed_paths 与 gate_added_lines 取，列路径的 git ls-files、git diff --name-only 都带选项。
"""
import subprocess


def tracked_names(root):
    """列出跟踪的文件。

    基准不对时看 merge-base 取到的是哪一个，--name-status 列出来的路径与仓里逐字对得上。
    """
    return subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z"], cwd=root, capture_output=True).stdout
