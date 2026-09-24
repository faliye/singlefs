# 实现员交回：checker 这一批（收口表第 44 行 I-3.10、第 48 行 C504、第 54 行候选 b）

接手的是撞会话额度中断的那一位（交接摘要 `/tmp/claude-1000/impl-checker-batch/handover.md`）。它已落盘的改动没有重做：副本先同步到主工作区 2026-09-23 22:13 UTC 的现状（`crates/` 与 `.claude/` 都同步了，HEAD 同为 3b60f09），再把它的补丁重放上去，全部复核一遍；这之后只改了两处名字（见「接手之后改了什么」），另外补了 5 行变异。22:56 主工作区又变了一批（`crates/mutations.tsv` 末尾多 7 行、`crash.rs`、`bad_disk_input.rs`、`first_transaction_step_seven_layer0.rs`、`second_transaction_step_zero_layer0.rs` 等 9 份改了、新增一份 `second_transaction_step_three_acquisition_barrier_layer0.rs`），22:57 再同步一次、补丁照样重放无冲突，check.sh、门禁 54 号全量加快档、59 号本批 18 行、登记给我的几个阶段、C504 复核都在 22:57 这一版上重跑了一遍（第八节「22:57 那一版上的复跑」）；第四节逐条证明会红那一轮是 22:13 那一版上跑的。时刻都是 UTC。

## 一、三件逐条

| 件 | 做到没有 | 依据 |
|---|---|---|
| 第 44 行 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） | **做成套**：池级 checker 判定、只红它的坏镜像、射程 ③ 的用例、8 行变异（第四节第 1–4、14–17 行） | 实现 `crates/singlefs-checker/src/walk.rs` 第 1557 行 `birth_txg_in_a_usable_unit_header`、第 1577 行 `allocation_record_node_pointers_of_the_candidate_versions`、第 1623 行 `judge_allocation_generations_against_unit_births`，第 2936 行接进 `check_pool_image`；`image.rs` 第 37 行 `IMPLEMENTED_INVARIANTS` 39 → 40 条。坏镜像 `checker_known_bad_images.rs` 第 2252 行 `known_bad_images_of_the_allocation_generation`（发布 B 之后 B 的数据单元两盘各一条未释放记录，分配代 4 改回 3，单元不动），用例第 2281 行只红 I-3.10、其余逐项与干净镜像相同 |
| 第 48 行 C504（树表条目宽在走读里无守卫，今天没坏法打得到） | **这一批开工前主工作区里已经有了，本批没改一行**，只在同步后的副本上复核 | 守卫在 `walk.rs` 第 522 行（`walk_tree_table_entry` 第一步，两处切片之前）；坏法 `crates/singlefs-harness/src/bad_disk_input.rs` 的 `narrow_the_entry_width_of_the_tree_table_unit`；用例 `second_transaction_supplement_three_bad_disk_input.rs` 第 576 行；变异 `crates/mutations.tsv` 第 300 行。这几处都是主工作区未提交的改动（`bad_disk_input.rs` 未跟踪），开工快照 16:28 里就有，是谁改的我查不到。复核结果见第八节「C504 复核」 |
| 第 54 行候选 b（候选集并集补上「由记录施加出来、根槽从没落盘的那一版」） | **做到**：两半验收都有用例 | 实现 `walk.rs` 第 2577 行 `VersionAppliedOnlyByRecords`、第 2610 行 `versions_applied_only_by_records`（四条同时成立才算一版，见第五节）、第 441 行 `walk_version_applied_only_by_records`、第 2854 行起并进遍历并按版本各判一格 I-7.4 / I-4.8；I-3.1 机理标识加一段「并进遍历的由记录施加出来的版本 N 个」（第 3019 行）。残留记录那条流：`second_transaction_step_zero_layer0.rs` 的钉死值从 `[("I-3.1", 12)]` 改成 `[]`，同步后的副本上绿；纯泄漏两份坏镜像（Z3-A、Z3-B）与仓里原有那份 I-3.1 坏镜像照样只红 I-3.1 |

## 二、写过的文件

补丁 `/tmp/claude-1000/impl-checker-batch/checker.patch` 只含下面 6 份（全部在 `crates/`，没碰 `litmus/`；相对主工作区 22:57 的现状生成。其中 `first_transaction_step_seven_layer0.rs`、`second_transaction_step_zero_layer0.rs` 两份主工作区 22:56 也改过，改的是别的段落，补丁的增删行与 22:13 那一版逐行相同，只是上下文与行号偏了）：

```
$ git apply --numstat checker.patch   （在主工作区跑）
4	4	crates/singlefs-checker/src/image.rs
309	6	crates/singlefs-checker/src/walk.rs
661	2	crates/singlefs-harness/tests/checker_known_bad_images.rs
5	4	crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
22	8	crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs
7	6	crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
```

`git diff --stat -- crates litmus` 在主工作区与副本上都混着别的会话几十份未提交的改动，分不出谁的，不贴；以上面的 numstat 为准。

动到的已有测试（不是新测试）：
- `checker_known_bad_images.rs` 的 I-4.2 坏镜像：原来只把数据单元头里的诞生代号改成 4，现在分配记录树里那个落点的分配代也跟着改成 4（两盘各一条）。不改的话 I-3.10 跟着一起红，判别力算不到 I-4.2 头上。
- 「每条已实现的不变量都有坏镜像」那条用例的链上加了 `known_bad_images_of_the_allocation_generation` 与 `known_bad_images_of_a_pure_leak`。
- `first_transaction_step_seven_layer0.rs`、`second_transaction_step_three_formatted_pool.rs`、`second_transaction_step_zero_layer0.rs`：登记「哪些不变量只在新根下评估 / 报不适用 / 必须评估到」的清单加 I-3.10。`second_transaction_step_three_formatted_pool.rs` 另立 `NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE`：写行那次发布之后（树表 0 条、根记录直接持有一片分配记录树，C512（树表 0 条的一版上被换下的单元记在哪））I-3.10 真被评估过，其余 16 条照旧不适用。

`crates/mutations.tsv` 没动（派发要求新行不写进它）。新行 18 行在 `/tmp/claude-1000/impl-checker-batch/mutations-append.tsv`，逐行名字见第四节。

## 三、接手之后改了什么

1. 副本 `crates/`（连同 `.claude/`）同步到主工作区现状后重放补丁：`git apply --check` 6 份全过，没有冲突。先存了一份：`drafts/mine-before-sync/`。
2. 门禁 12 号（不许用角标）：补丁里 `checker_known_bad_images.rs` 一行注释与变异表第 13 行的名字都写着判决里的「候选 d」加一撇；主工作区里那份判决已经把它改名成 **f**（`research/prompts/m2-placement-falsepositive-r1-main-verification.md` 第 25、48 行），两处跟着改成「候选 f」。补丁与变异表里现在一个角标字符都没有（门禁 12 号在带着补丁、变异表末尾追加了这 18 行的副本上判绿，见第八节）。
3. 补了 5 行变异（变异表第 14–18 行）：根记录直接持有的那片分配记录树不读、三类单元头诞生代号偏移各错一处、候选 b 第 ② 条去掉。

## 四、每条新测试怎么证明会红

做法：副本 `drafts/prove/repo`（同步到主工作区现状 + 补丁，自带 target）里逐条改坏一处，跑点名测试所在的**整个**测试二进制（`--no-fail-fast`、不加过滤），记红了哪些测试，跑完从原件拷回并 `touch`；每个涉及的二进制先在不改动的副本上跑一遍当基线。脚本 `drafts/scripts/prove_red.py`，结果 `drafts/prove2/prove-results.tsv`，每条的整段输出 `drafts/prove2/prove-red-logs/row-N.log`。三个二进制（`checker_known_bad_images`、`second_transaction_step_zero_layer0`、`second_transaction_step_three_formatted_pool`）的**基线红集都是空的**。`crates/singlefs-checker/src/` 里没有 `debug_assert`（`grep -c` 三份文件都是 0），红的都是测试自己的断言。跑于 22:19–22:38。

「行号」是补丁之后 `walk.rs` 里被改的那一行；「断言」是点名那条测试红在哪一行、消息开头；「同红」是同一个二进制里一起红的测试条数（含点名那条）。

| 表行 | 改坏什么 | 行号 | 点名的测试红在哪条断言 | 同红 |
|---|---|---|---|---|
| 1 | I-3.10 的比较恒成立 | 1666 | `checker_known_bad_images.rs:2294`「判红的该只有 I-3.10」（坏镜像上一条都不红了） | 1 |
| 2 | 射程 ① 放宽：已释放的记录也判 | 1649 | `:2285`「发布 B 之后的干净镜像上 I-3.10 要真被评估过且成立」 | 19 |
| 3 | 射程 ② 改读末槽 | 1657 | `:2294`「判红的该只有 I-3.10」（末槽上读不出可用的头，坏记录不判） | 1 |
| 4 | 射程 ③ 放宽：头校验和不过也取诞生代号 | 1570 | `:2688`「起点槽的头校验和不过的那条记录不判……」 | 1 |
| 5 | 候选 b 整个拿掉（记录施加的版本不并进遍历） | 2860 | `:2570`「接在只由记录施加出来的那一版之后的合法镜像上一条都不许红」 | 2 |
| 6 | 同上，点名层 0 残留记录那条流 | 2860 | `second_transaction_step_zero_layer0.rs:368`「I-3.1 判违例的状态数」left 12 / right 0，差 638976 − 573440 = 65536 字节 | 1 |
| 7 | 候选 b 第 ① 条放宽：不看实例表的行 | 2632 | `:2618`「挂载之前那一刻一条都不许红」 | 1 |
| 8 | 第 ③ 条去掉：低于 F 的也并进 | 2640 | `:2769`「F 抬过 (1, 5) 之后一条都不许红」 | 1 |
| 9 | 第 ④ 条去掉：环转过去也并进 | 2641 | `:2831`「环转过一整圈之后只有 I-3.1 那一形已知红」 | 1 |
| 10 | 按版本那一格 I-7.4 恒成立 | 2869 | `:2652`「那一版的树表被抹掉，I-7.4 要红」 | 1 |
| 11 | 按版本那一格 I-4.8 恒成立 | 2875 | `:2652`「那一版的树表被抹掉，I-4.8 要红」 | 1 |
| 12 | 判决 Z3 候选 c：I-3.1 改成「记账 ≥ 遍历」 | 3031 | `:2384`「判红的该只有 I-3.1」（纯泄漏哑了）；同红里有 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`，即仓里原有那份 I-3.1 坏镜像也哑了 | 5 |
| 13 | 判决 Z3 候选 f：I-3.1 改成「记账 − 遍历 ≤ defer 行」 | 3031 | 同第 12 行 | 5 |
| 14 | I-3.10 不读根记录直接持有的那片分配记录树（C512） | 1589 | `second_transaction_step_three_formatted_pool.rs:434`「写行那次发布之后：I-3.10 要真被评估过且成立」 | 1 |
| 15 | 码 1 诞生代号偏移 75 → 83 | 1562 | `:2285`「发布 B 之后的干净镜像上 I-3.10 要真被评估过且成立」 | 19 |
| 16 | 码 2 诞生代号偏移 52+2k → 60+2k | 1565 | 同第 15 行 | 19 |
| 17 | 码 3 诞生代号偏移 73 → 81 | 1567 | 同第 15 行 | 19 |
| 18 | 候选 b 第 ② 条去掉：根槽读得出的那一版也再并进一次 | 2637 | `:2602`「机理标识要报出那一版并进了遍历」，实际报「并进遍历的由记录施加出来的版本 5 个」 | 2 |
| C504 | 主表第 300 行（不是本批的行，复核用）：守卫那一步去掉 | 522 | 见第八节「C504 复核」 | — |

变异名逐行（`mutations-append.tsv`，六段制表符分隔，与 `crates/mutations.tsv` 同形）：

1. 增补 2 收口第 44 行：I-3.10 的比较恒成立（未释放记录的分配代与单元头里的诞生代号不比）
2. 增补 2 收口第 44 行：I-3.10 射程 ① 放宽（已释放的记录也拿释放代去比单元头的诞生代号）
3. 增补 2 收口第 44 行：I-3.10 射程 ② 改读末槽（跨两槽的记录不再取起点槽那个单元头）
4. 增补 2 收口第 44 行：I-3.10 射程 ③ 放宽（起点槽的头校验和不过也照样取诞生代号来比）
5. 增补 2 收口第 54 行（候选 b）：由记录施加出来、根槽从没落盘的那一版不并进遍历（合法镜像上 I-3.1 红回来）
6. 增补 2 收口第 54 行（候选 b）：由记录施加出来的那一版不并进遍历（层 0 残留记录那条流 12 个状态在 I-3.1 上红回来）
7. 增补 2 收口第 54 行（候选 b）第 ① 条放宽：不看实例表里有没有这个实例的行，环里带提交标记的记录都算施加过
8. 增补 2 收口第 54 行（候选 b）第 ③ 条去掉：低于回退下界 F 的那一版也并进遍历
9. 增补 2 收口第 54 行（候选 b）第 ④ 条去掉：环里最旧的有效根已比那一版新（它换下的单元已可回收）也并进遍历
10. 增补 2 收口第 54 行（候选 b）：由记录施加出来的那一版引用的单元被复用或抹头时 I-7.4 不判（按版本那一格恒成立）
11. 增补 2 收口第 54 行（候选 b）：从由记录施加出来的那一版出发的遍历读不出单元时 I-4.8 不判（按版本那一格恒成立）
12. 增补 2 收口第 54 行判决 Z3 的候选 c（记账 ≥ 遍历）：纯泄漏坏镜像必须照样红
13. 增补 2 收口第 54 行判决 Z3 的候选 f（记账 − 遍历 ≤ defer 行）：纯泄漏坏镜像必须照样红
14. 增补 2 收口第 44 行：I-3.10 不读树表 0 条那一版根记录直接持有的那片分配记录树（C512（树表 0 条的一版上被换下的单元记在哪））
15. 增补 2 收口第 44 行：I-3.10 取码 1 单元头的诞生代号偏移错位（75 读成 83）
16. 增补 2 收口第 44 行：I-3.10 取码 2 单元头的诞生代号偏移错位（52 + 2k 读成 60 + 2k）
17. 增补 2 收口第 44 行：I-3.10 取码 3 单元头的诞生代号偏移错位（73 读成 81）
18. 增补 2 收口第 54 行（候选 b）第 ② 条去掉：根槽读得出的那一版也从它的记录再并进一次遍历

第 5、6 行改的是同一处、点名不同的测试；第 12、13 行改的是既有的 I-3.1 判定那一行（`walk.rs` 第 3031 行，不是本补丁的代码），它们钉的是「判决里被否掉的两个改法一旦落地，这几条用例当场红」。

## 五、候选 b 落成了什么：「由记录施加出来、根槽从没落盘的那一版」怎么认

判决（`research/prompts/m2-placement-falsepositive-r1-main-verification.md` Z3 一节）只说「并集补上记录施加出来的版本」，没给单张镜像上怎么认。实现里（`walk.rs` 第 2610 行 `versions_applied_only_by_records`）是四条同时成立：

| 条 | 判据 | 为什么这样认 | 钉它的用例 / 变异 |
|---|---|---|---|
| ① 它被施加过 | 记录带提交标记，且最新根指着的实例表里有这个实例的行 (i, Ti, Wi)、记录的 checkpoint_txg ≤ Ti | 写行取的 Ti 是恢复之后有效根的 txg；没有这个实例的行就还没有哪次恢复施加它（残留记录那条流里「链接得到残留记录」的 19 个状态就是这一格） | `a_journal_record_that_no_mount_has_applied_yet_is_not_walked_and_every_invariant_holds`；变异第 7 行 |
| ② 它的根槽读不出 | 根环里自证过的根没有一条是 (i, 这个 txg) | 读得出的由根环那一路走 | `a_version_applied_only_by_its_journal_record_is_walked…` 的机理标识断言；变异第 18 行 |
| ③ 不低于回退下界 | txg ≥ 最新根带的 F | 与候选根同一条 | `…leaves_the_walk_once_the_rollback_floor_is_raised_past_it`；变异第 8 行 |
| ④ 它换下的单元还没到回收的时候 | 根环里没被实例表判抛弃的根里，txg 最小的那条比它小 | 回收门槛是 max(F_生效, 环里最旧有效根)（`crates/singlefs-core/src/mount.rs` 第 499–506 行，「有效」也只滤被抛弃的根，与这里同一口径）；③ 加 ④ 推得出它换下的单元释放代高于门槛 | `…leaves_the_walk_once_the_root_ring_has_turned_past_it_and_a_mount_reclaimed_its_units`；变异第 9 行 |

一版按 (实例代号, checkpoint_txg) 认，两块盘各一份镜像记录取先读到的那一份。走法：从记录新根段里的树表指针（0..86）与映射根指针（86..172）走下去，判定与候选根同一套，引用进同一个并集；I-7.4 / I-4.8 每一版各判一格（`erasing_a_unit_only_the_version_applied_by_its_journal_record_references_reddens_the_reuse_invariants`，变异第 10、11 行）。I-3.10 也读这几版树表里的分配记录树。

两半验收：
- **误报消失**：`a_version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_a_leak_on_top_still_reddens_it` 造的就是那段历史（A → B，实例 1 再发 (1, 5) 只落单元与记录、根槽与系统配置槽一个没落 → 可写挂载择 (1, 4)、施加记录、写行 (1, 5, 3)、写行发布 txg 6 → 暖机 txg 7），与 c381-r1 那个纯断电同形；合法镜像一条不红，I-3.1 真被评估过且成立。层 0 残留记录那条流钉死值 12 → 0。
- **改完仍会红**：同一份镜像上再泄漏一槽，只红 I-3.1，机理标识报「并进遍历的由记录施加出来的版本 1 个」；纯泄漏两份（Z3-A 只泄漏、Z3-B 泄漏加 defer 行跟着抬高）只红 I-3.1、别的逐项与干净镜像相同；判决里否掉的 c、f 两个改法做成变异第 12、13 行，一落地这几条当场红，同红里有仓里原有那份 I-3.1 坏镜像所在的用例。

⚠️ `…root_ring_has_turned_past_it…` 那条用例上 **I-3.1 本来就红**（环转过一整圈之后记账多于遍历，机理标识里最老的自证过的根已高过 F），与 (1, 5) 那一版无关，是收口表第 ② 行那一族、随机历史的已知红第 0 条。用例钉的是现状：只红 I-3.1、红在那一形、那一版没有并进遍历。不是认下来的行为。

## 六、`.claude/kb/invariants.md` 的措辞（交主 agent，我没改 kb）

I-3.10 那一行（主工作区现在是第 136 行，派发时说的第 135 行已经挪了一行）状态列，建议把「未实现」换成：

> 已实现（2026-09-23，池级 checker `crates/singlefs-checker` 的 `walk::judge_allocation_generations_against_unit_births`；读回退候选集里每一版的分配记录树——根环里候选根的（树表里种类 3 那一条）、树表 0 条那一版根记录直接持有的那一片（C512（树表 0 条的一版上被换下的单元记在哪））、由记录施加出来的那几版的（同 I-3.1（已分配统计对得上） 那一格）；同一条记录（盘、槽、代）只判一次；射程 ③ 按记录逐条跳过，一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）

这段措辞里「读回退候选集里每一版」是实现替条款做的一个选择，条款正文写的是「任一未带已释放标志的分配记录」，见第七节第 1 条；主 agent 不认这个选择的话，这段要跟着改。

I-3.1 那一行的「checker 读法」注，候选 b 落地之后「实际遍历所有引用」多了一格，建议在「2026-09-17 起……」那一句后面接一句：

> 2026-09-23 起回退候选集另并进「由记录施加出来、根槽从没落盘的那一版」（用户定候选 b，增补 2 收口表第 54 行）：环里带提交标记的 journal 记录 (i, T)，最新根的实例表里有 i 的行且 T ≤ Ti、根环里没有 (i, T) 的根、T ≥ F、环里最旧的未被抛弃的根 txg < T；从那条记录新根段里的树表指针与映射根指针走下去（`walk::versions_applied_only_by_records`）。

I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽） 两行写的都是「回退候选集里每一个根」，实现里由记录施加出来的那几版也各判一格（第五节），这两行要不要跟着写一句，一并交主 agent。

## 七、要主 agent 定的

1. **I-3.10 读哪些分配记录**。条款写「任一未带已释放标志的分配记录」；实现只读回退候选集（加上面两类）里的分配记录树，不读被实例表判抛弃的根与低于 F 的根。理由：低于 F 的根那棵账里未释放的记录，它罩住的槽在 F 抬过之后可以合法地被回收复用、换了诞生代号，照「任一」去判会在合法镜像上红。候选集里不会出这种事：回收门槛是 max(F_生效, 环里最旧有效根)，候选根的 txg 不低于这两个数，它未释放的记录不可能已被回收。**推翻条件**：一段合法历史里，候选集里某条根的分配记录树有一条未释放记录，它的槽上已经是另一个单元（诞生代号不同）——层 0 全量与随机历史快档都没有撞到（第八节）。另外 checker 用的是最新根自己带的 F，不是跨盘的 F_生效，一块盘的载体根坏掉之后两者可以不同，这是 I-3.1 那一行已经写着的同一个缺口，I-3.10 跟着它。
2. **候选 b 那一版的四条认法（第五节表）是实现推出来的**，判决没写。尤其 ④ 只是一个充分条件：④ 不成立而它换下的单元其实还没回收（释放代 > 环里最旧有效根 ≥ 这一版的 txg）时，checker 不并进这一版、遍历少算，I-3.1 会红（露出来，不会藏住泄漏）。要不要走一轮三方核这四条，由主 agent 定。
3. **记录新根段里没有的两样**（`walk.rs` 第 435–440 行那段注）：实例表指针、「树表 0 条那一版的分配记录树的根」。实现照新根段走、这两样不走：实例表那一样由它施加在其上的那一版走（那一版在候选集里时）；第二样在带文件的一版上恒全零。条款没写的两格是「那一版不在候选集里」与「由记录施加出来的是树表 0 条的一版」。后一格今天走得到：暖机与写行的空发布也写带提交标记的记录，新根段照上一版的根（`crates/singlefs-core/src/transaction.rs` 第 566 行暖机那段、第 649–665 行写行那段文档注释：写行另写实例表与分配记录树两个单元，新根段里的树表、映射根仍照抄上一版），那样的一版从记录走下去的就是上一版的树表，并进来不多算也不少算；写行那次发布写的那片分配记录树节点在它的根槽没落盘时谁也不引用，checker 与分配器是不是都不算它，**我没有用例核过**。这一格没有定案之前照现在这样走：少算只会让 I-3.1 红出来，不会让它变绿。
4. **收口表第 48 行 / C504 的状态**：实现、坏法、用例、变异第 300 行都在主工作区（未提交），不是本批写的。收口表第 48 行与 `.claude/kb/checks-owed.md` C504 那一行现在还写着「实现待做」「检查仍欠」，改不改由主 agent 定。

## 八、check.sh 与门禁

开跑前 `ps` 看过：没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；别的会话的 `cargo test --all`、`cargo test --release --bin e142-first-txn-dry-run` 与另一个实现员的批跑一直在，负载 22:42 时 154（32 核）。本批每一份副本各用自己的 target，没有等锁。

**check.sh，在副本 `repo`（主工作区现状 + 补丁）上原样跑**：`exit 1`，停在第一步，末尾原样：

```
     emit_result(&format!(
  ✗ 格式不合规
     → 怎么办： 跑 cargo fmt --all 让它自己改完，再重跑本脚本。
```

没排版的只有两份：`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`（83 处）与 `e158_root_choice_repair.rs`（32 处），都是主工作区里别的会话未跟踪的实验 bin，不在补丁里。补丁那 6 份 `cargo fmt --check` 不报。

为了让后几步跑得到，另起副本 `drafts/checkcopy/repo`：先只对那两份跑 `rustfmt`，clippy 又在它们身上红（e158 两处 `#[allow]` 没写 reason、一处 items after test module、一处 `ok` 同名遮蔽；e156 一处 `mounted` 同名遮蔽，共 5 处，全在这两份里），于是把这两份挪出去（`drafts/checkcopy/removed-bins/`）再跑。22:15–22:26，`exit 0`，四步原样：

```
══ cargo fmt --check ══
  ✓ 格式通过
══ cargo clippy ══
  ✓ clippy 通过
══ cargo build ══
  ✓ 构建通过
══ cargo test ══
  ✓ 单测通过
```

`test result` 50 行全是 ok。涉及的几个二进制：`checker_known_bad_images` 22 passed；`second_transaction_step_zero_layer0` 8 passed、1 ignored；`second_transaction_step_three_formatted_pool` 10 passed；`first_transaction_step_seven_layer0` 5 passed、1 ignored；`second_transaction_supplement_three_bad_disk_input` 9 passed、1 ignored；`second_transaction_supplement_three_random_history` 18 passed、2 ignored（其中跑池级 checker、要求「只停在已知红那几形」的几条都绿，也就是 I-3.10 与候选 b 在这几段随机历史上没有让 checker 在已知红之外判红；哪几段每一步都跑 checker 以门禁 74 号文件头为准，我没逐段核）。

**登记给实现员的门禁阶段**，在副本 `drafts/gatecopy/repo`（`repo` 加 `crates/mutations.tsv` 末尾追加本批 18 行）上跑，末行与退出码原样：

| 阶段 | 退出码 | 末行 |
|---|---|---|
| 33-mutation-tables.sh | 0 | `  ✓ 145 个实验二进制都有成形的变异表，1540 条变异的原文各命中源码一次；crates/mutations.tsv 348 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）` |
| 53-format-const-placeholders.sh | 0 | `  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））` |
| 92-layout-checker-sync.sh | 0 | `      第一条纯 SSD 布局线：.claude/kb/layout/02-second-txn.md` |
| 94-checker-implementation-disjoint.sh | 0 | `    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 59 号复跑` |
| 93-feature-bits.sh | 0 | `  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 1 位、登记表分出去 1 位；扫了 46 个 .rs，认出 2 处 feature bit 常量、解出位号 1 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:26，值 `INCOMPAT_FIRST_SSD_LINE_BIT` 不是位掩码字面量））` |
| 89-closeout-row27-preconditions.sh | **77（本次未跑，不是通过）** | `      alloc-basis 第三轮 ③：扣住配今天的 checker 放过「扣住位被撤掉」那份坏镜像：扣住位只住内存、盘上没有落点，没有可扫的对象` |
| 12-no-prime-marks.sh（不归我，因为改了名字顺手跑） | 0 | `     没扫的：二进制 7 份；上游副本 .claude/singlefs-ai-sop/ 下 0 份（只能在上游改）`（上一行是「✓ 查了 1963 份文本文件，没有角标写法」，行尾括号里列着那五个字符，这里不抄） |

**门禁 59 号，只跑本批 18 行**（副本 `drafts/gate59b/repo`，`crates/mutations.tsv` 只留这 18 行，`SINGLEFS_GATE_FULL=1 GATE_MUTATION_WORKERS=4`，22:38:45–22:40:21，`exit 0`），末行原样：

```
  ✓ crates 变异表复跑：18 条变异各自红在点名的测试上（原文都恰好命中一次；工作进程动态领活——谁先跑完谁再领下一条，不按条数预先切片；输出按表的行号排序、与进程数无关）
```

前 18 行逐条「✓ <变异名>：<测试名> 红了」，全文 `drafts/gate59b/gate59.log`。

**门禁 54 号**，在 `drafts/gatecopy/repo` 上先跑 `--full`（22:27:21–22:55:37，`exit 0`）再跑快档（`SINGLEFS_GATE_FULL=1`，`exit 0`）。快档的判定行原样：

```
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 8 条通过、1 条 ignored
  ✓ 全绿标记与这批输入的内容哈希相同（9a11ec66942f21c9…，93 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-23T22:55:37Z，标记里的计数行原样：
```

全量两条流都 `violations=0`、`checker_violations=0`、`exhaustive=true`；I-3.10 在第一个事务那条流 `I-3.10=4/0`，两次发布那条流 `I-3.10=1842252/0/262161`（评估过 / 违例 / 不适用；不适用的 262161 个与 I-3.1、I-5.4 同数，是最新根下面还没有分配记录树的那些状态）。全文 `drafts/gatecopy/gate54-full.log`、`gate54-quick.log`。这份全绿标记写在副本自己的 `.git` 里，输入哈希含追加了 18 行的变异表，与主工作区对不上，主工作区的 54 号要主 agent 收尾时自己跑。

**C504 复核**（主表第 300 行，副本 `drafts/prove/repo`，22:13 那一版，22:40–22:46；22:57 那一版上的复跑见本节末尾）：基线上 `second_transaction_supplement_three_bad_disk_input` 9 passed、1 ignored，「清单外的 panic 0 次」；把守卫那一步改成 `if false && …` 之后同一二进制红 3 条，`bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites` 红在 `second_transaction_supplement_three_bad_disk_input.rs:168`（left 11 / right 0，「清单外的 panic 11 次」，都在 `crates/singlefs-checker/src/lib.rs:154`：range start index 10 out of range for slice of length 8），`a_tree_table_narrower_than_a_registered_entry_…` 红在同文件第 598 行。C504「怎么拦」列要的判别力自证（panic 计数由 0 变正）在主工作区现状上成立。

**门禁 74 号**（登记给实现员，`drafts/gatecopy/repo`，22:55:52–22:56:17，`exit 0`），末行原样：

```
      随机历史：小盘上逼近单元区墙的取样点：模型对拍 4713 步：该拒而拒 643、区间里拒 329、该成而成 3741；比过根 4325 条、分配记录 77196 条、冷启动内容 59 次、抬 F 上限 167 次；回退到 txg = F_生效 > 0 的根做成 0 次；分配记录墙按镜像上的真条数放行 0 次；单元区墙按区间放行 310 次
```

首行是「✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 5 段）：」，全文 `drafts/gatecopy/gate74.log`。

**22:57 那一版上的复跑**（主工作区 22:56 又变了一批之后，副本重新同步、补丁重放，下面每一样都在这一版上重跑）：

| 跑了什么 | 时段 | 结果 |
|---|---|---|
| check.sh 原样（`repo`） | 23:19 前后 | `exit 1`，末尾三行与上面逐字相同（`✗ 格式不合规`，仍卡在 e156、e158 两份上：83 处、32 处） |
| check.sh（`checkcopy`，挪走 e156、e158） | 22:58:25–23:18:30 | `exit 0`，四步 `✓ 格式通过`、`✓ clippy 通过`、`✓ 构建通过`、`✓ 单测通过`；`test result` 51 行全是 ok；`checker_known_bad_images` 22 passed、`second_transaction_step_zero_layer0` 8 passed 1 ignored、`second_transaction_step_three_formatted_pool` 10 passed、`first_transaction_step_seven_layer0` 5 passed 1 ignored、`second_transaction_supplement_three_bad_disk_input` 9 passed 1 ignored、`second_transaction_supplement_three_random_history` 18 passed 2 ignored |
| 门禁 54 号 `--full` 再快档（`gatecopy`） | 22:58:32–23:15:59 | 两次都 `exit 0`；全量两条流 `states=262165 closed_form=262165`、`states=2104413 closed_form=2104413`，`violations=0`、`checker_violations=0`、`exhaustive=true`，I-3.10 仍是 `4/0` 与 `1842252/0/262161`；快档首行与上面逐字相同，第二行是「✓ 全绿标记与这批输入的内容哈希相同（6ac97033d5b71e01…，94 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-23T23:15:44Z，标记里的计数行原样：」 |
| 门禁 12、33、53、92、93、94、89、74 号（`gatecopy`） | 23:15:59–23:16:59 | 退出码依次 0、0、0、0、0、0、77、0；12、53、92、93、94 号末行与上面逐字相同；33 号末行只差条数：「crates/mutations.tsv 355 条的原文各命中源码一次」；74 号末行与上面逐字相同 |
| 门禁 59 号，本批 18 行（`gate59b`） | 22:58:38–23:01:12 | `exit 0`，末行与上面逐字相同（「18 条变异各自红在点名的测试上」） |
| C504 复核（主表第 300 行，`prove`） | 23:18:58–23:19:51 | 基线 9 passed、1 ignored、「清单外的 panic 0 次」；守卫去掉之后红 3 条：`second_transaction_supplement_three_bad_disk_input.rs:204`（left 11 / right 0，「清单外的 panic 11 次」，`crates/singlefs-checker/src/lib.rs:154`：range start index 10 out of range for slice of length 8）、第 634 行、第 446 行 |

第四节那 18 行逐条跑整个二进制的证明没有在 22:57 这一版上重跑；59 号在这一版上把 18 行各自点名的测试都判红了。

## 九、补丁对主工作区现状的 `git apply --check`

`checker.patch` 定稿之后在主工作区跑，2026-09-23 23:21:12 UTC，原样：

```
Checking patch crates/singlefs-checker/src/image.rs...
Checking patch crates/singlefs-checker/src/walk.rs...
Checking patch crates/singlefs-harness/tests/checker_known_bad_images.rs...
Checking patch crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs...
Checking patch crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs...
Checking patch crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs...
apply-check exit 0
```

`mutations-append.tsv` 18 行的锚点在「主工作区现状 + 补丁」上各命中一次（`drafts/scripts/check_anchors.py`，「核完，不是恰好一次的 0 行」）；主表 22:57 那一版的 342 行在同一份树上也都恰好一次。补丁与变异表里角标字符 0 个（`grep -cP` 数的）。

```
3e8e99daf65c5350d0041851e20de5bebead15c5d4ba830105a9825eea7116cc  /tmp/claude-1000/impl-checker-batch/checker.patch
5be561c8d819ebe55c3f754110078659161ce3a71c0f6b8e4f14b596b667f78f  /tmp/claude-1000/impl-checker-batch/mutations-append.tsv
```

推翻条件：主工作区在 23:21 之后再动这 6 份文件里补丁碰到的那几段，`git apply --check` 会失败，要重放；动 `walk.rs` 里变异表那 18 行锚点所在的行，锚点会失配（门禁 33 号先红）。

## 十、没做什么

- 没走三方对抗；没提交、没暂存；没改 kb（I-3.10、I-3.1、I-7.4、I-4.8 几行的措辞与收口表第 44、48、54 行、C504 行的状态都交主 agent，见第六、七节）。
- `crates/mutations.tsv` 没动；18 行新变异只在 `mutations-append.tsv`，要主 agent 追加。
- 主工作区的门禁 54 号全量、59 号整表没跑（只在副本上跑了 54 号全量加快档、59 号本批 18 行）；层 0、QEMU、herd7 与整张 crates 变异表归 `crash-verifier`。
- check.sh 在主工作区现状上过不了第一步，卡在别的会话的两份未跟踪实验 bin（e156、e158）上，没替它们排版或修 clippy；四步全绿是在把那两份挪走的副本上跑出来的。
- 第七节第 3 条「写行那次发布写的那片分配记录树节点在它的根槽没落盘时，checker 与分配器是不是都不算它」没有用例核。
- 第 48 行 C504 本批没写一行代码，只复核。
- 本批新加的都是 checker 判定与坏镜像，没有新开层 0 流，没有新的崩溃点重放用例。
