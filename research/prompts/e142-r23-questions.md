# E142（新池新建文件的干跑） 第 23 次重跑的问题单（2026-10-05）

<!-- doc-lint:not-numbers P8G -->

第 22 次（登记 `research/prompts/e142-r22-prereg.md` 与 `research/prompts/e142-r22-s2-revision.md`，主产物 `research/results/e142-new-pool-file-creation-dry-run-r22-s2-main-2026-10-05.out`、P8G 导出 `research/results/e142-r22-p8g-crates-dump-2026-10-05.out`）在主点与 P8G 两个几何上都答「装置与 `crates/` 29 个区域逐字节相同」。之后用户 2026-10-05 定系统配置槽偏移 433 那 4 字节 T_time 的编码单位改成毫秒、默认 5000（判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2 第三行），第 30 批 30-3 照改 `crates/singlefs-core/src/system_configuration.rs`（补丁 `/tmp/claude-1000/m3-formal-30-3/patch/crates.patch`）；装置 `scripts/experiments/e142_new_pool_file_creation_dry_run.rs` 第 1524 行仍写 `writer.put_u32(5); // T_time 秒`。第 22 次的登记与产物冻结不动。

| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| 1 | 装置只把第 1524 行 T_time 那一个常量改成毫秒（5 → 5000、注释改毫秒）之后，装置独立算出的字节与 30-3 合入后的 `crates/` 写出的，在主点几何上 29 个区域逐字节相同吗（含反向链） | 相同 / 哪几个区域不同 | 比对行有一行 `equal=false`，或 `unequal` 大于 0 | 主点那一段比对行全数出来 | 开着 |
| 2 | 同一改动之后，P8G 几何（两块盘、每盘 8 GiB，第 22 次第二段那一个）上装置与 `crates/` 的导出 29 个区域逐字节相同吗、判决字段翻不翻面 | 相同且不翻面 / 哪几个区域不同或哪几个判决字段翻面 | `P8G_SUMMARY` 里 `unequal` 大于 0，或判决字段与第 22 次第二段有一个不同 | P8G 那一段比对行与判决字段全数出来 | 开着 |
| 3 | 第 22 次已够判的几样在第 23 次还成立吗：装置写出的字节与 `layout/01` 字段表逐字段相等（174 行）、装置里不剩旧值、判决字段与第 22 次逐个相同、整张变异表全抓 | 都成立 / 哪一样不成立 | `name=layout_field … equal=false` 有一行；`residual_old_values count` 大于 0；判决字段有一个翻面；变异有没红或无效 | 四样各数出来 | 开着 |
