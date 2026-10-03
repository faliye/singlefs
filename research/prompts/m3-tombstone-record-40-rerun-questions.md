# 墓碑记录 40 字节下重跑 E83、E102：问题单（2026-10-03）

墓碑记录今天是 40 字节（D18（块里携带什么信息） 已定项 24；`.claude/kb/layout/03-third-txn.md` 第 11、18 行，`format-const: TOMBSTONE_RECORD_BYTES = 40`；实现 `crates/singlefs-format/src/lib.rs`）。E83（墓碑的粒度）与 E102（单元类登记表与打包记录单元）的装置按 56 字节算（`research/e7-index-bench/src/bin/e83_tombstone_grain.rs` 第 50 行、`e102_unit_class_registry.rs` 第 75 行），门禁 code-source-discipline 的 format-constants 格判红。改装置里的常量就是新实验，照 `.claude/singlefs-ai-sop/rules/test-discipline.md`「实验开跑之前，答案不许已经存在」重跑。

| 问题 | 为哪个决定而量 | 翻面观测 | 量到哪算够 | 状态 |
|---|---|---|---|---|
| E83 在 40 字节下，「每条记录独占单元」与「打包共享单元」两种粒度的物理写量各多少 | D18（块里携带什么信息） 已定项 8「删除写墓碑单元」的粒度选择 | 40 字节下打包共享单元不再比独占单元少写，或差距小到选择要换 | 两种粒度在装置原有的取样点上各出一行，单测钉绝对值 | 够判（2026-10-04，E83 第 2 次，不翻） |
| E102 在 40 字节下，打包容量 cap(h, 40) 与 E102 正文里依赖 56 字节的那几格（打包容量、下游数、混装错分数）各是多少；正文里写死 56 的反向接受条款按什么改 | D18（块里携带什么信息） 已定项 11 定案提案 | 40 字节下登记表的某一处映射或打包容量让提案 P8 不成立 | 正文每一处依赖记录宽度的数都在 40 下重算并钉住，与 `format-const: TOMBSTONE_CONTAINER_RECORDS = 815` 对得上 | 够判（2026-10-04，E102 第 2 次，P8 不翻、815 对得上） |
