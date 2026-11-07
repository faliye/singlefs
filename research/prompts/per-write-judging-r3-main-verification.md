# 崩溃状态逐写独立判：第三轮判决（最终轮，2026-11-06）

正文：`research/prompts/_per-write-judging-r3-body.md`。两条腿：攻方 `research/prompts/per-write-judging-r3-opus-output.md`，辩方 `research/prompts/per-write-judging-r3-sonnet-output.md`。前两轮判决：`research/prompts/per-write-judging-r1-main-verification.md`、`research/prompts/per-write-judging-r2-main-verification.md`。

## 核查怎么做的

- 两腿各写一套探针，主 agent 在同一份冻结副本（`/home/fy5090/.local/share/singlefs/work/per-write-judging-r3-attack/frozen`）上各复跑一次：
  - 攻方：任务 8703（第 4 次尝试退 0），产物拷进 `research/prompts/per-write-judging-r3-opus-model/out/`；
  - 辩方：C1（普查、臂、阳性对照），任务 8704（第 4 次尝试退 0），产物在 `/home/fy5090/.local/share/singlefs/work/per-write-judging-r3-defense-rerun/`。辩方的逐态全量几层（24 段合计 4,945,477,608 态，单进程推 35 天以上）没交。
- 引文：两腿报告的 cite-check 末行分别是「✓ 核了 36 处引文、2 处裸行号，对上 36 处」「✓ 核了 77 处引文、0 处裸行号，对上 77 处」（腿自己跑的）；判决只引下面主 agent 现查的产物行。

## 量过的（两套探针一致）

- 第二轮 E6 在真实流上的命中全是宽度误报。攻方产物：`PROBE_E6_FLOW label=unmodified segments=36 unit_segments=7 segments_with_r2_e6=5 segments_with_class_width_e6=0 segments_with_exact_start_e6=0 segments_with_pointer_e6=0 …`。按点名项的类取读者真读的宽度后，一段都不命中。
- 收窄判据下的态数，两套探针逐数相同：
  - 攻方：`… narrowed_passing=7 … reduced_total_narrowed=471`（删除流）；对拍三条流 355、378、404。
  - 辩方：`PROBE_E6N_FLOW label=unmodified segments=36 unit_segments=7 passing_old=2 passing_e6n=7 engine_total=1463812286 reduced_total_old=1379926285 reduced_total_e6n=471`；对拍三条流 `reduced_total_e6n=355`、378、404。
- 反例臂：收窄判据在 `record_before_units`（反例 B）与 `late_unit`（反例 A）上各拒 1 段（攻方 `narrowed_passing=6`；辩方 `PROBE_E6N_FLOW label=record_before_units … passing_e6n=6`）。
- 合成规则：攻方 `late_unit` 小宇宙 1024 态，`group_delta_red_differs=0 group_delta_items_differ=0`，并合成 `group_union_red_differs=256`。按单元副本组判要用增减合成，并合成不行。
- 阳性对照（两组同时被盖，判据不收的形状）：`PROBE_EXHAUST_CONTROL pairs=91 skipped=0 states=1456 pairs_union_items_differ=78 states_union_items_differ=243 pairs_delta_items_differ=78 states_delta_items_differ=243`，比较在这一臂上有牙。

## 没堵上的（两腿都报）

1. **比较在今天的流上没有牙**：真实流每个状态都绿时，任何合成（连只判 S=∅ 的退化候选）都与全组合逐态相同。要分辨，得拿 `crates/mutations.tsv` 里会让崩溃态变红的变异去比逐写判与全组合（攻方提议，没实现）。
2. **E5（D5：要 S=∅ 那一遍的读迹）没有实现**：库里没有记读包装，第二轮辩方写过不能据此降态。
3. **E3「落在单元区」那半句没量**。
4. 收窄判据（攻方叫 E7、辩方叫 E6n）在码 2 那一截比第二轮 E6 松，照「臂的定义也在跑前写死之列」记一次输、当新臂：**被攻过零轮**。

## 结论

三轮到此为止。量出来的是：收窄判据下逐写独立判能把删除流从约 14.6 亿态降到 471 态、对拍三条流降到 355、378、404 态；合成必须用按单元副本组的增减合成；两条反例臂各被拒 1 段。但「判据收下的段上逐写判与全组合等价」这句全称结论，在今天干净的实现上分不出真假（没堵上的第 1 条），还缺 E5 的实现与 E3 的量。

交用户定。主 agent 的建议：照收窄判据与增减合成实现逐写判，但落地之前把没堵上的三条做成会红的检查——
- 拿 crates 变异逐条比逐写判与全组合（会让崩溃态变红的变异上两者必须同判），任何一条不同就判红；
- 引擎实现 D5 的读迹，判据管不到或读迹缺的段整段退回全组合；
- 量 E3「落在单元区」；
三条都绿之前，逐写判只当「快路」，门禁仍以全组合为准。

## 交用户的表

| 项 | 状态 | 说明 |
|---|---|---|
| 收窄判据（E7 / E6n） | 被攻过零轮 | 两套探针量出删除流 471、对拍 355/378/404；反例两臂各拒 1 段 |
| 合成规则 | 增减合成（量的） | 并合成在 late_unit 小宇宙 256 态不同 |
| 等价性 | 今天分辨不出 | 要拿 crates 变异比，没实现 |
| E5（读迹） | 没实现 | 判据管不到就整段退回 |
| E3「落在单元区」 | 没量 | — |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） | 备料 | 2026-11-06 不受影响：三轮没有立判据，判法不改，等用户定 |
