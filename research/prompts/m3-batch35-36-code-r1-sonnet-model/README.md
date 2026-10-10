# m3-batch35-36-code-r1 正推腿（Sonnet）的模型目录

仓路径从环境变量取：`SINGLEFS_REPO` = 冻结副本根（`~/.local/share/singlefs/work/m3-batch35-36-code-r1/tree`），`SINGLEFS_MAIN_REPO` = 主仓根（只取 `research/scripts/` 里的内存包装与线程上限脚本）。

| 脚本 | 做什么 | 前台耗时 |
|---|---|---|
| `rerun.sh <空目录>` | R1 sha256 20 份；R2 变异表静态判（每行原文命中次数）与新增行数；R3 基线 harness 用例（core lib 10 条、harness 6 个二进制、known_bad 3 组 5 条）；R4 变异行逐条现跑（W 的 15 行 + 增补 2 第 61 行 + 第 33 批 T 的 16 行 + V 的 16 行，checker 档 9 行不跑）；R5 组合变异（T 的「只认带末条标志」行单独存活、与 C_ACCOUNT 那行一起红） | 约 2.5 分钟 |
| `rerun-adhoc.sh <空目录>` | R6 `protected.extend(...)` 整句删掉跑 core lib 与 10 个相关 harness 二进制，预期 nfailed=0 | 约 2 分钟 |
| `compare.sh <目录甲> <目录乙>` | 两边都有的 `static/baseline/mutations/combo/adhoc.txt` 排序、去耗时后逐行比 | 秒 |
| `crc_identity.py` | 独立实现 CRC-32C，验 V3 拼读公式 crc(A‖B) = (crc(A‖0) ^ crc(0)) ^ crc(B)；预期末行 `CRC_IDENTITY OK trials=6` | 秒 |

`expected/` 是交回前在空目录从头跑出的输出（rerun.sh 跑了两遍，两遍 compare 全同）。`mutrun.py`、`combo.py`、`adhoc.py`、`static_anchor.py` 是上面几个脚本调的零件。
没经这里复跑的：checker 档与 GPU（对拍文件、`crash_judge_*.wgsl` 的变异行、钉值）——子 agent 不跑，见报告「要主 agent 跑的命令」。
