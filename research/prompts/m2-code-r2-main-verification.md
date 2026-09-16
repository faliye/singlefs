# 发布 B（覆盖写 + 释放）那批代码三方对抗第二轮：主 agent 核实（2026-09-16）

第一轮判决 `research/prompts/m2-code-r1-main-verification.md` 第四节按跑前条款要「X1 / X2 / X4 打中 ⇒ 再攻一轮」。第二轮换攻击面：攻方腿只攻四处改法本身（`m2-code-r2-opus.md`），辩方腿复核第一轮每一格的判决（`m2-code-r2-sonnet.md`），本地腿攻空闲独立计数与释放经映射两处（`m2-code-r2-local.md`，英文，两次抽样）。材料：`_m2-code-r2-diff.md`（从提交 d2aeb7d 起 crates/ 的全部 diff + 两个新测试全文，2167 行）；条款附录沿用 `_m2-code-r1-background.md`。

被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/unit.rs`、`crates/singlefs-harness/src/crash.rs`；用例 `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`。

## 一、本地腿（s1 215 词、s2 275 词；字词损坏闸两次都过）

| 问 | s1 | s2 | 主 agent 核 |
|---|---|---|---|
| 1（空闲独立计数能不能骗过 I-5.2 与 I-3.1） | NO HIT：「the sum consistently equals the unit area size … because the checker directly uses these counters」 | HIT：假设释放路径「incorrectly increments free_slots and decrements allocated_slots」，两个计数一起漂而「I-3.1 holds」 | 两次相反。s2 的构造是一条对称变异（两个计数一加一减），I-5.2 确实骗得过——但它说 I-3.1 仍成立是错的：I-3.1 拿记账的已分配与环里全部有效根引用的并集比，已分配少 10 槽当场红（第一轮攻方腿的 M-X2 正是这条变异，副本上 I-3.1 红）。s1 的「the checker directly uses these counters」也不准：checker 读的是盘上记账行，不是内存计数。记「不稳定、没打中」；有用的一句是「对称变异只靠 I-3.1 拦」，写进判决第三节 |
| 2（经上一版映射查落点会不会查错） | NO HIT：上一版的映射节点是同一次发布写的，释放时那些单元仍分配着 | HIT：假设「the mapping node was written with an incorrect slot for a key」（写者 bug）⇒ 释放错的槽 | 两次相反。s2 的前提是映射节点本身写错，那是写者的 bug、不是释放路径的；而且这种镜像 checker 会红（映射条目的位置项带单元校验和，指错槽读不到对应的单元）。记「不稳定、没打中」 |
| 3（两盘不同槽的合法历史） | NO HIT | NO HIT | 两次一致：D2（RAID 条带策略） 已定项 10 两盘同槽，第一版没有两盘异槽的形态。没打中 |

## 二、辩方腿（Sonnet，`m2-code-r2-sonnet-output.md` 81 行；六条变异在副本上各跑一遍）

| 格 | 报告怎么判 | 主 agent 核 | 处置 |
|---|---|---|---|
| 第一轮判决一、二、三节的二十格技术判断 | 全部「判对」，X2 那一格「比判决更强」：摘掉减法之后 checker 的 I-5.2 与空闲绝对值断言两条路都红 | 与入库装置上的读数一致（`I-5.2 … Violated("盘 0：空闲 Some(3472883712) + 已分配 Some(376832) ≠ 单元区 Some(3472883712)")`） | 无 |
| X1-① 与本地腿 X2 第二次抽样的处置 | **处置不合条款**：条款「X1–X4 打中 ⇒ 改代码、补一条会红的用例、再攻一轮」，第四节只写了字、引了既有的 C283（准入失败时不先推发布就报 ENOSPC） / I-5.3（报出的空闲都兑现得了），没有新代码、没有新用例，也没有明写偏离 | **核实，判对**：条款原文没有例外；第一轮判决第四节确实没写「这里按 X6 待遇」 | 第一轮判决补第五节明写偏离与理由（谓词要步 5 的两样东西）；补弱形态用例 `released_placements_are_not_handed_out_again_before_reclaim_exists`，变异「`mark_released` 清位图」会红；步 5 还 |
| X5-① 的处置（立 C374（释放代与树表诞生 txg 只有验收断言盯着） + 收窄措辞） | 「门槛较低，勉强够」：没有新增独立对照，只是账记得更准 | 与第一轮判决说的一致，条款对 X5 只要「补对照」，立不立不变量步 3 前定 | 无 |
| 六条变异 | 各红在判决说的那条断言 / checker 上 | 与入库装置上的六次变异读数逐条相同 | 无 |
| `newest_persisted_root_txg` 改名 | 报告核出攻方腿写的旧名是修前的，改名是改法的一部分 | 对 | 无 |

辩方腿小结：技术判断零判错；打中的是处置与跑前条款之间的落差，已按上表补齐。
