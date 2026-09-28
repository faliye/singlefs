# crates/ 代码审阅汇总（singlefs-6c 会话，2026-09-26，只读，未改任何仓内文件）

口径
- 范围：singlefs-format、singlefs-core 全部、singlefs-checker 全部；singlefs-harness 只审 lib.rs / segments.rs / crash.rs / model.rs / model_comparison.rs，其余 harness 源文件与 tests/ 没审。
- 做法：7 个只读审阅员分组逐行读（没跑 cargo），交回后主 agent 对着现行代码逐条核重点项。每条标「主核」= 主 agent 现读代码核过；「审」= 只有审阅员报告，主 agent 没现核。
- 时点：2026-09-26 的工作区。审阅期间 walk.rs、crash.rs、model.rs、model_comparison.rs、fault_injection.rs、layer0_progress.rs 被别的会话改过，行号可能已漂，按引的代码串定位。
- HEAD 对照：除标「仅工作区」的两条外，主核条目的代码串在 HEAD（73ba4a4）里同样存在。
- 欠账表：按关键词 grep 过 .claude/kb/checks-owed.md，只有第 15 条对上 C475；其余没搜到对应条目（关键词搜，不保证穷尽）。
- 分组详细报告：同目录 A-report.md … G-report.md（A format 与编解码；B 分配器/准入/块设备；C 事务层；D 挂载与读路径；E 恢复与树；F checker；G 验证装置）。

逐条正文见发给 singlefs-99 的消息，与本文件相同。
