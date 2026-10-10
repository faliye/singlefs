# E142（新池新建文件的干跑） 第 24 次重跑的问题单（2026-10-05）

<!-- doc-lint:not-numbers N22R P8G -->

第 23 次（登记 `research/prompts/e142-r23-prereg.md`，问题单 `research/prompts/e142-r23-questions.md`，报告 `/tmp/claude-1000/e142-r23-run/report.md`，主产物 `research/results/e142-new-pool-file-creation-dry-run-r23-main-2026-10-05.out`）第 2 行够判；第 1、3 行的守门 Q142.114（「改动之外什么都没动」）按登记字面打中 F23d：N22R 到主产物的归一化 diff 除登记点名的三组（`device_region_bytes`、`p8g_device_region_bytes`、`journal_header_write_digest`）外，`write_list_row`、`p8g_write_list_row` 各变 2 行。第 23 次的登记与产物冻结不动，照字面记第 1、3 行不够判。

| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| 1 | 守门收成「写清单那两组只许 `sha256=` 一列变、且每一行的新值等于同一次写的 `device_region_bytes`（或 `p8g_device_region_bytes`）那一行的 `sha256=`」之后，装置（第 23 次那一份，只改过第 1524 行）与 30-3 合入后的 `crates/` 在主点几何上 29 个区域逐字节相同吗（含反向链） | 相同 / 哪几个区域不同 | 比对行 `unequal` 大于 0；或守门在收过的形态下仍打中 | 主点比对行与守门全数出来 | 开着 |
| 2 | 同一组输入下，第 23 次已够判的几样还成立吗：字段表 174 行逐字段相等、装置不剩旧值、判决字段与第 22 次第二段逐个相同、整张变异表全抓 | 都成立 / 哪一样不成立 | `name=layout_field … equal=false` 有一行；`residual_old_values count` 大于 0；判决字段翻面；变异有没红或无效 | 四样各数出来 | 开着 |
