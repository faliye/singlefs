"""E171 第 1 次重跑·修订 11 复用库的根目录（占位 H30 的 library_root）怎么取：派到两台里哪一台都是同一个绝对路径
（第 2 次重跑登记 research/prompts/e171-r2-prereg.md 修订 4；「所有任务都要多机可用」，.claude/rules/verification.md 同名一节）。

三层编排（segment_one_units_revision_11.py、rows_orchestrator_revision_11.py、rows_orchestrator_segment_two_revision_11.py）都经这里取库根，
单元 reuse-selfproof 的 `--library <库根>/selfproof` 也就跟着两台相同：
  - H30 没填（没有这一键、null 或空串）：数据根下的缺省 <数据根>/reuse/<库名>，数据根取 .claude/hooks/lib_work_root.py 的 shared_data_path
    （那份文件不改；SINGLEFS_DATA_ROOT 或多机配置的 SHARED_DATA_ROOT，两台同一份），库名同 reuse_library.DEFAULT_LIBRARY_ROOT 的最后一段；
  - 填了绝对路径：照用（规范化）；
  - 填的路径经 `~` 或相对路径取（不是绝对路径）：拒。`~` 在第二台展开成那一台自己的家目录，两台各是一份库。
不建目录、不碰库里的东西；库怎么存归 reuse_library.py。
"""

from __future__ import annotations

import importlib.util
import os

import reuse_library as reuse

HERE = os.path.dirname(os.path.abspath(__file__))
LIB_WORK_ROOT = os.path.join(HERE, "..", "..", "..", ".claude", "hooks", "lib_work_root.py")
SHARED_REUSE_ENTRY = "reuse"
LIBRARY_NAME = os.path.basename(reuse.DEFAULT_LIBRARY_ROOT)


def default_library_root(environment: dict | None = None) -> str:
    """数据根下的缺省库根：shared_data_path("reuse") 下的库名。environment 不给取 os.environ（自证拿它造「第二台」的环境）。"""
    specification = importlib.util.spec_from_file_location("lib_work_root_for_e171_reuse_library_root", LIB_WORK_ROOT)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return os.path.join(module.shared_data_path(SHARED_REUSE_ENTRY, environment), LIBRARY_NAME)


def library_root_of(h30: dict | None, environment: dict | None = None) -> tuple[str, str]:
    """H30 的库根：交回 (库根, 问题)。问题是空串才能用；不是空串时库根是缺省那一个，只给 plan 打出来看，run 照问题退 78。
    写的不是绝对路径（`~` 起头的也算：库不替它展开）就是问题：经 `~` 展开或从当前目录算，两台各是一个路径。"""
    value = (h30 or {}).get("library_root")
    default = default_library_root(environment)
    if value in (None, ""):
        return default, ""
    text = str(value)
    if not os.path.isabs(text):
        return default, (f"H30 library_root={text} 经 ~ 或相对路径取：派到第二台是那一台自己的另一份库；留空取数据根下的缺省 {default}"
                         "（.claude/hooks/lib_work_root.py 的 shared_data_path），或填两台同一的绝对路径")
    return os.path.normpath(text), ""


def require_library_root(h30: dict | None, environment: dict | None = None) -> str:
    """run 用：库根两台同一就交回它，不然抛 reuse_library.RunCondition（调用方退 78）。"""
    root, problem = library_root_of(h30, environment)
    if problem:
        raise reuse.RunCondition(problem)
    return root
