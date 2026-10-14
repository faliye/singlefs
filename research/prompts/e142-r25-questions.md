# E142（新池新建文件的干跑） 第 25 次重跑的问题单（2026-10-12）

第 24 次（登记 `research/prompts/e142-r24-prereg.md`，问题单 `research/prompts/e142-r24-questions.md`，主产物 `research/results/e142-new-pool-file-creation-dry-run-r24-main-2026-10-05.out`）之后 `crates/` 变了两处装置没跟上：mkfs 每盘多清一段 [0, 根环基址)（D22（单元原子性怎么合成） 已定项 8 第 3 条，用户 2026-10-02 定 W1，`crates/singlefs-core/src/make_filesystem.rs` 每盘清零循环的第一句）；退役树加进 `crates/`（树表写 12 条、树 ID 水位 65549、记账标签 6 含退役项，`.claude/kb/checks-owed.md` C774（E142 装置没追平退役树） 那一行点名的常量与函数）。用户 2026-10-12 定：装置追平、重跑第 25 次，与门禁 checker-tier-qemu-device-streams 改成按登记行比 mkfs 行两样都做。只写问题与候选，不写倾向、不写已有的数。

| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| 1 | 装置照 D22（单元原子性怎么合成） 已定项 8 第 3 条补上每盘 [0, 根环基址) 整段清零之后，产物 `name=segments path=mkfs` 那一行与布局表八「mkfs 种根」登记行（`scripts/crash/check-segment-registry.py --print-mkfs-segments-line` 打出的那一行）逐字相同吗 | 相同 / 不同（哪一个字段） | 两行 diff 非空，或 `operations=` `segments=` `closed_form=` `kinds=` 四样任一不同 | 第 25 次主产物的那一行与登记行比对一次 | 开着 |
| 2 | 装置照 C774（E142 装置没追平退役树） 补上退役树（树表条目、树 ID 水位、记账标签 6 的退役项）之后，装置与 `crates/` 在主点上区域逐字节、字段表逐行、标签 6 两侧相等吗 | 全等 / 哪一处不等 | 区域比对任一区域 `sha256=` 不等；`name=layout_field … equal=false` 任一行；标签 6 两侧的数不等 | 第 25 次主产物三处比对各一次 | 开着 |
| 3 | 第 24 次已够判的几样在第 25 次的装置与 `crates/` 上还成立吗：装置不剩旧值、判决字段与第 24 次逐个相同、整张变异表全抓 | 都成立 / 哪一样不成立 | `residual_old_values` 非零；判决字段任一不同；变异表有一条没红或无效 | 第 25 次产物与变异表各跑一次 | 开着 |
