# E173（上限卡在读坏槽的三种处置） 第 2 次重跑：问题单（2026-11-13）

不是为岔路建的实验，照岔路单的格式写。只写问题与候选定义，不写倾向、不写已有的数。

起因：复跑登记表 `scripts/experiment-apparatus/replay.sh` 第 2921 行（`E173|@driver_e173_checkpoint_cost||e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out|exact`）在今天的代码上与留存产物对不上：抬 F 之前先算加了拒绝成员 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`（`crates/singlefs-core/src/mounted_session.rs`），之后又换了槽对那一判的上界并加了合取 7 那一形（判决 `research/prompts/m4-gpugc-precheck-b-a-code-r3-main-verification.md`）。

| 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 一、留存产物与今天的代码差在哪 | ① 只差被拒的结局名（原来报推满仍放不下的格，今天有的改报先算判出放不下）；② 除了结局名，还有计数、写数、屏障数、空发布次数这类量变了 | 逐行比今天现跑的输出与留存产物：有没有一行的数值字段不同 | 那一条驱动在今天的代码上跑完，与留存产物逐行比，列出不同的行与不同的字段 | 开着 |
| 二、E173 的结论要不要改 | ① 问题单第 1 行那一条判语（按 ckpt_cost 现算的代码读法与条款读法不同、只代码读法打中的格数）照旧；② 判语里的格数或哪一臂打中变了 | 重跑之后判语里引的每个数与实验页现有的是否相同 | 第一行够判之后，对着实验页「结论」逐个数核 | 开着 |
