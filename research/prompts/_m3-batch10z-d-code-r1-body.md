# 根环实现第 10z 批第四块：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 H1 H2 R C M7 M11 LR2 -->

## 一、题面

这一轮攻第 10z 批第四块：照第 10z 批第二块代码轮判决 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第三节用户定案 H1、H2 改抬 F 那一串，连同那份判决第二节的 K2-3（e158 单测）与 K3。

用户 2026-10-08 弹窗定：
- H1「被抛弃根的槽不拒」：抬 F 那一串读根环时，被抛弃根的根槽读坏不拒，照根环表里记着的落点进影子账、不进上限；有效根的槽照今天，读坏重读仍坏就拒。
- H2「用 C 那一圈记着的落点」：根环表里没记着那条被抛弃根的落点、而 C 那一圈在那一槽记着同一条根时，照 C 的落点算、不读账、读不出不拒；C 也没记着的照今天读盘。

条款：`.claude/kb/decisions/16-发布语义.md` 已定项 1「根槽这一次读坏」那一行（第 37 行末两句是 H1、H2）；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 回退段与「可写挂载时的预隔离」；`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 1 第九项。

改动全文：`research/prompts/_m3-batch10z-d-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-d/patch/report.md`。

| # | 改动 | 落点 |
|---|---|---|
| A | H1：准入算上限、抬 F 重算影子账、卸载那一串里，根环表记成被抛弃根的槽读坏不拒，按没有根算、不进上限，影子账照记着的落点补隔离；有效根槽照拒；管理员回退那一遍不改 | `crates/singlefs-core/src/mount.rs` |
| B | H2：`RootRingOccupancy::placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot`，认同条件槽记有效根且 txg 相同（两遍只有一遍自证那一类再比实例代号） | `crates/singlefs-core/src/allocator.rs`、`mount.rs` |
| C | e158 内联单测改期望：抬 F 做成、计数 0、记着的落点不空闲 | `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` |
| D | K3：新用例断言记着的落点不空闲；H1 三条、H2 一条、M7、M11 各一条、两条库单测；m5、m6、m8、m20 各一条默认档会红；LR2 文档句改现状；变异追加 15 | 两份 harness 测试、`crates/mutations.tsv` |

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/`，取在第四块合入之后（底含删除线第 43 批）；被判文件 sha256 在 `research/prompts/m3-batch10z-d-code-r1-snapshot/crates-sha256.txt`。
- 合后验证（CPU 部分，`~/.local/share/singlefs/work/m3-r1/merge-10z-d/verify-summary.txt`）：core lib `202 passed`；harness 档四个包 185 个测试二进制全 ok；fmt、clippy 退 0；E171 装置单编退 0、sha 前 12 位 ab35105f861d 不变；e158 装置内联单测 `93 passed; 2 failed`，10z-d 改的那一条过了，红的两条在撤掉 10z-d 的副本上照样红、归删除线（不在这一轮被判范围，腿报出来不算打中）。碰卡的 checker 档 lib 与 GPU 测试排在 E174 之后。
- 主 agent 合入时改过一行变异：第 3326 行（M11 / m19）的原文落在删除线第 43 批抽出的共用函数 `placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread` 里，两处 `return Ok(…)` 改成 `return Some(…)`，重证红 `✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了`。
- 实现员自定、被攻过零轮的：H1 读坏的被抛弃根槽不重读（理由：重读读出来也是同一条根、同一份落点）；`ValidRoot` 不带实例代号，这一类只比 txg。
- 实现员自己报的、推的、没造的：
  - C 那一圈只按槽归组，同一槽差一整圈的另一次发布的落点会一起借来；那一槽 C 只记着别的发布、没记着 R 时认同条件挡不住；
  - 挂载那一刻被抛弃根的根槽读回全零时，它的单元在现行账里是空闲的，影子账与 C 那一圈都不护它们。
- E171 装置约束（形式化线 2026-10-08）：`DeviceFreeMap`、`Placement`、`PoolAllocator`、`RootRingOccupancy`、`raise_rollback_floor` 的名字与签名不动，装置 match 的枚举不加成员。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　定案做到没有
1. H1：抬 F 那一串每一处读根环都换了吗？有效根槽读坏是不是照拒？管理员回退那一遍是不是没变？
2. H2：认同条件有没有一形认错（差一整圈、那一槽 C 只记着别的发布、只比 txg 不比实例）？两样都没记着时是不是照今天读盘？
3. e158 单测改完，原来要钉的那件事还钉着吗？

### K2　I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽）
1. H1 不进上限、照记着的落点隔离：有没有一形被抛弃根引用的单元掉出影子账、被发出去？
2. H2 借来的 C 那一份罩不罩得住 R 引用的全部单元？罩不住的那一截会不会被发出去？
3. 实现员推的那一形（挂载时被抛弃根槽读回全零，它的单元谁都不护）造不造得出、算不算 D16 已定的代价？

### K3　测试判别力
新用例与变异能不能拦住这一批每一处被改回去？有没有一形「改回去」照绿？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
