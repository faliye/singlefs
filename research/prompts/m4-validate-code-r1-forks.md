# m4-validate-code-r1 岔路单（2026-10-22）

| 岔路 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 标签 0 配 size > 0 的第一个文件怎么改 size（`crates/singlefs-core/src/transaction.rs` 的 `publish_size_change`、`crates/singlefs-core/src/truncate.rs` 两处断言） | 子：取 txg 之前交类型化拒绝，这一形只能覆盖写；丑：0 个单元的文件变长、N = S、截短都只改记录、标签留 0；寅：读者不再收第一个文件的标签 0 配 size > 0 | 写者有一条路径能自然写出这一形（那样子会挡住真用户，丑或寅才对）；读者不收会让已有的仓内测试与合法镜像判坏 | 查清今天哪条写路径写得出这一形，并在入库装置上复现三种 N 的 panic | 用户已定：丑，写者接得住这一形（2026-10-22 弹窗选「丑：写者接得住这一形 (Recommended)」）；调查员报告 `~/.local/share/singlefs/work/m4-investigate-t5-tag-zero/report.md`：写路径写不出这一形（读码与约 5800 次发布实测 0 次），今天的代码上三种 N 都 panic，另量到重写上段叶会丢掉标签 0 条目 |
| 合法的大 size（K12 上限之内）让冷走读、冷恢复与 RefFS 按 size 整份分配（`crates/singlefs-core/src/recovery.rs` 的 `content.resize(`、`crates/singlefs-reference-filesystem/src/lib.rs` 的 `truncated.resize(`） | 子：三处不交整份字节，交单元内容加尾部洞长；丑：K12 上限收窄到装得下的数；寅：冷读遇超界 size 交类型化拒绝 | 子的 API 改动波及的调用方数（checker 档调用 `recover` 的装置）；丑收窄之后是否挡住真用户会用到的 size | 数清调用方并在入库装置上复现 1 TiB 中止 | 用户已定：这一版照现在的方式读，放到后面的优化里程碑（2026-10-22 原话「这个没必要管就按照现在的方式读。 之后会优化 没有必要现在优化。」「放到后面的优化里程碑里面」） |
