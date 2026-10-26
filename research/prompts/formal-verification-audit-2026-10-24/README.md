# 形式化验证审计证据（2026-10-24）

正式报告：[形式化验证严格复核](../../../records/2026-10-24-形式化验证严格复核.md)。

- `initial/`：首次审核的 opaque 恒假前提与单行 requires 探针、Verus 原始 JSON 和 stderr。
- `deep/`：深入审核的源码指纹、当前核快照、漏登记模块与反转选段比较的变异副本、Verus 原始输出、最小门禁复现脚本。
- `deep/REPORT.md`：迁入 records 前的原始报告；正式阅读入口为上方 records 链接。

证据从原临时目录逐字节归档，没有重新运行验证。`deep/gate_probe.py` 与 `deep/gate_probe_false.py` 是当轮的一次性复现脚本；其门禁输出当时只显示于会话，未另存原始 stdout，结论记录在报告中，不能把报告当作原始输出文件。

复现时先将所需脚本复制到新的临时目录，将脚本中 `root=Path('/home/fy5090/code/singlefs')` 改成待核工作区的绝对路径，使用登记的 Verus 安装，并按仓库的范围声明与内存包装规则执行。脚本会在自己的目录创建副本或最小工程，不要直接在本冻结证据目录运行。`probe.py` 复验当前核及两个变异；`gate_probe_false.py` 复现已登记函数调用漏登记错误引理后被 proof 格判绿。当前源码可能已变化，历史结果只对应 `deep/source-sha256.json` 中的指纹。

原临时目录 `/tmp/singlefs-formal-deep-audit-wgxnn8_h` 与 `/tmp/singlefs-formal-writing-audit-58eqtqxk` 未删除，保留供当轮核对；报告链接不依赖它们。
