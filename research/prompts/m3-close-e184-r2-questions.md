# E184（常规层成本计数） 第 2 次跑前登记修订问题单（2026-10-24）

上游写序变化（取号写两轮、C755 每次可写挂载问一次 F2）之后，装置 `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs` 的钉值表已照新段形重钉（新池 11 → 12 段、建快照 15 → 16 段、多记录 19 → 20 段，`~/.local/share/singlefs/work/m3-close-b16-e184-e183-pins/result.md` 第一、三节），而跑前登记 `research/prompts/e184-preregistration.md` 第七节锚 A1、A2、A5、A7 与第十一节作废条款 X3（`SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST` 16842807）写的还是改前的数；第一段整份产物（18 件长跑）没在新段形上重跑。

| 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 登记第七节锚 A1、A2、A5、A7 与第十一节 X3 的数在新段形下各取什么 | 甲：照新段形现算的数改登记（修订记录写改了什么、依据哪份读数、时点在第一段重跑之前，原数原样保留）；乙：登记原数不动，另立重跑登记、只换数 | 现算的新数与装置 selfcheck 现录值逐个相同（不同则装置或现算有一边错） | 每个锚与 X3 都有新旧两数、来源与现算式子，装置 selfcheck 退 0 | 够判（2026-10-24，`research/results/e184-regular-layer-cost-counts-segment-one-r2-verdict-comparison-2026-10-24.out`） |
| 第一段整份产物在新段形上判定词翻不翻（T1–T4、A1–A6、B1 触发 F6） | 不翻：重跑之后判定词逐格同改前；翻：某一格换判定词 | 重跑产物里任一格判定词与改前产物不同 | 第一段 18 件在新段形上重跑完、逐格比过 | 够判（2026-10-24，`research/results/e184-regular-layer-cost-counts-segment-one-r2-verdict-comparison-2026-10-24.out`） |
