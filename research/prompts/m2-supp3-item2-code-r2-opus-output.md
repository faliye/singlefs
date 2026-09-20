# m2-supp3-item2-code-r2 Opus 攻方腿（M1、M2、M4）

轮：增补 3 第 2 件代码三方第二轮。立场：假设 Z1–Z4 错，在仓副本上施加变异、造历史去打穿。时刻 UTC 2026-09-19 15:40–17:05（JST 2026-09-20 00:40–02:05）。
**这一腿所有数都是仓副本上量的，不是入库装置上的数**；自提的改法只在我的副本上量过、被攻过零轮。

复跑：`research/prompts/m2-supp3-item2-code-r2-opus-model/RERUN.md`；文件清单与 sha256 在同目录 `SHA256SUMS`（87 个文件，`SHA256SUMS` 本身 sha256 `c7d452413309f5988910a35970bdf9aed771c7c9807035aa58dc97cb0921131d`）。
开工时副本 `crates/` 与 `research/prompts/m2-supp3-item2-code-r2-start-snapshot.sha256` 逐个同 sha256（`sha256sum -c` 不是 OK 的行数 0）。

## 各格判定一览

| 格 | 判 | 一句话 |
|---|---|---|
| M1 墙的误红 | 没打中 | 基线在 6 种比重（含 3 种放开用户动作的）、9 批扫法上共 9512 段历史、观察到 59681 次分配记录墙拒，模型对不上 0 次 |
| M1 漏判（非「差一」的误拒） | 没打中（门禁字面）；余量薄 | 5 条 core 变异（串里多算 1、非文件发布多算一个角色、只在抬 F / 只在回退多算一个角色、墙前移 2 条）门禁第四段全红；但「只在抬 F」「只在回退」两条在门禁那 32 个种子里各只红 1 段，按 32 种子一窗切 15 窗，5 / 15、7 / 15 窗一段都不红 |
| M2 回退排除原因只换字段 | 没打中 | 7 条只换 `RollbackCandidateExclusion` 的变异（含三条换进 / 换出「树表 0 条」），四段各自都红 |
| M2 发布落点拒绝只换原因 | **打中** | 分配器用户数据那一处把「每块盘都没有空槽」报成「小盘写满」（m2h）：四段全绿、**整个 workspace 的测试全绿**；门禁四段在 4 GiB 等大盘上一次落点拒绝都走不到 |
| M2 几何上的基线误红 | 没打中（在我能造的几何上） | 两块等大、单元区 384 槽的小盘副本：基线 768 段探针历史、1437 次落点拒绝，模型对不上 0 次；盘不等大模型表达不了，没法造 |
| M2「树表 0 条」算不算候选排除 | 不归我（正推腿那一格） | — |
| M4 跳过 checker 的那一段 | **打中** | 两条只在逼近墙那类历史上走得到的 core 变异（m4a 根环转圈后回收门槛多一代；m4b 分配记录过 600 条后记账已分配少记一槽）：门禁四段全绿、模型与执行器 0 / 256 段判出，池级 checker 在同一批历史上 135 / 256、256 / 256 段判红（不是已知红第 0 条的形态） |

## M1 墙的误红与漏判（没打中）

判据原文（`research/prompts/_m2-supp3-item2-code-r2-body.md` 第 42 行，整行）：

> | M1 墙的误红与漏判 | Z1 在合法历史上会不会误红（数出的条数与实现准入用的量口径不一，例如复用已回收记录之后真写的条数更少、或点名的版本取错），会不会漏掉「差一」以外的误拒（例如准入把某个角色算两次） | 一段合法历史在基线代码上被判对不上；或一条让实现在 ≤ 812 时误拒的 core 变异（不是 W1 那一条）四段都不红 |

### 先读码：模型数的与实现准入用的是不是同一个量

- 实现的准入基数是分配器内存里的记录条数：`crates/singlefs-core/src/transaction.rs` 第 1329 行 `        records_before_this_publish + rewritten.len() * allocator.devices.len();`，一串（写行 + 暖机）逐次加：第 1307 行 `        records_before_this_publish += rewritten.len() * allocator.devices.len();`。
- 模型的放行条件：`crates/singlefs-harness/src/model.rs` 第 1326 行 `                upper_bound_exceeds && true_count_exceeds`，真条数 = 执行器在点名那一版下按 checker 读法数的条数 + 计划里每次按准入口径加的条数。
- 我找的「口径不一」的三处，读码都对得上：① 抬 F 的回收（`reclaim_released_up_to`）不删记录，只进 `reclaimed` 集合，条数不变；复用已回收记录是改写、不加条；罩住的已回收记录删掉那一步在发布里发生，写出去的树就是删过的，下一次准入读的分配器与镜像上那一版同数。② 可写挂载与回退的分配器由所选根 / 目标根的记录重建（`rebuild_from_records`），影子账隔离只动位图、不加记录。③ 挂载的「所选根」在不建崩溃的历史里就是最新根，与模型的 `starting_version` 同一条。
  ⇒ 「复用之后真写的条数更少」这一形只会让镜像上的数与分配器的数**一起**变少，不会分开。准入本身按上界（每个角色每盘一条、不抵扣复用）是收口表第 39 行用户定的保留口径，模型照同一口径加，不算误红。

### 放开用户动作扫基线（误红那一分句）

探针 `scripts/opus_r2_probe.rs`：与门禁第四段同一个执行器（`PerStepChecker::Skipped`），种子与比重从环境变量取；比重除仓里 4 组外另加两组放开用户动作的（`wall_rb`：回退 40% / 抬 F 10% 多；`wall_plain`：只覆盖写与可写挂载），`wall_rb` 另跑 300 步一段。

| 批 | 比重 | 种子 × 步数 | 墙拒次数（观察者数） | 模型对不上 / 新发现 |
|---|---|---|---|---|
| base-wall-observe-0-32 | 门禁第四段那组 | [0,32) × 150 | 326 | 0 |
| base-wall-observe-32-512 | 同上 | [32,512) × 150 | 4221 | 0 |
| base-wall-skip-0-2000 | 同上 | [0,2000) × 150 | 18434 | 0 |
| base-wallrb-skip-0-2000 | wall_rb | [0,2000) × 150 | 2642 | 0 |
| base-wallrb-skip-300-0-1000 | wall_rb | [0,1000) × 300 | 15792 | 0 |
| base-wallplain-skip-0-1000 | wall_plain | [0,1000) × 150 | 13220 | 0 |
| base-broad-skip-150-0-1000 | 快档那组 | [0,1000) × 150 | 313 | 0 |
| base-rollback-skip-150-0-1000 | 回退取样点那组 | [0,1000) × 150 | 20 | 0 |
| base-reuse-skip-150-0-1000 | 复用取样点那组 | [0,1000) × 150 | 4713 | 0 |

原样（`logs/baseline-batch.out`）每批一行 `PROBE SUMMARY … new_findings=0 probe_checker_red=0`，9 行都是 0。

### core 变异（漏判那一分句）

每条都是「实现在真条数 ≤ 812 时误拒」、都不是 W1。门禁 74 号那个测试二进制（四段）与两批探针：

| 变异 | 改了什么（副本上） | 门禁二进制 | 第四段判出段数 | 探针 wall [32,512) | 探针 wall_rb [0,512) |
|---|---|---|---|---|---|
| m1a | 一串准入每次多加 1 条 | 红 | 3 / 32 | 33 | 16 |
| m1b | 不写文件版本的发布（写行、暖机、抬 F 的空发布）准入多算一个角色 | 红 | 4 / 32 | （输出被截，见限度） | 21 |
| m1c | 只在抬 F 路径多算一个角色 | 红 | **1 / 32** | 15 | 8 |
| m1d | 只在回退路径多算一个角色 | 红 | **1 / 32** | 9 | 6 |
| m1e | 墙前移 2 条（`+ 2 >`） | 红（两条用例） | 9 / 32 | — | 36 |

m1c、m1d 在 [32,512) 里判出的种子（`logs/all-logs.tar.gz` 里 `mutant-m1c-*.log`、`mutant-m1d-*.log`）按 32 个一窗切 15 窗，每窗判出段数：m1c `0 1 0 1 2 2 1 1 0 0 0 1 1 2 3`（5 窗为 0），m1d `1 1 0 1 0 1 1 0 2 0 0 1 0 0 1`（7 窗为 0）。
⇒ 门禁写死的 [0,32) 恰好各中 1 段；判据字面「四段都不红」没触发，**没打中**。但这是种子窗的运气：换一窗、或门禁以后挪种子，约一半的窗口看不见「只在回退多算」这一类。不是这一格的打中，记给主 agent 判要不要加种子或加一条钉死的用例（今天 `crates/mutations.tsv` 第 156–164 行施加在 `crates/singlefs-core` 上的是 W1 两条、R1、C368 那一条，没有一条是只在抬 F / 回退路径上误拒的）。

四句（对 M1 整格）：没打中，不适用。

## M2 映射

判据原文（`research/prompts/_m2-supp3-item2-code-r2-body.md` 第 43 行，整行）：

> | M2 映射 | Z2、Z3 的映射会不会把一种拒放成另一种合法的拒，或在合法历史上判对不上；「树表 0 条」算候选排除是不是条款的意思 | 一条只换排除原因或拒绝原因的变异四段都不红；或基线在某种几何（盘不等大、小盘写满）上误红；或条款原文把树表 0 条的根算进候选集而实现报「不是候选」 |

### Z2 回退排除原因：七条只换字段的变异（没打中）

`crates/singlefs-core/src/mount.rs` 第 1205、1220、1230、1238 行四处 `exclusion:` 两两互换（去掉第一轮已有的 R1：被抛弃→低于 F）。门禁二进制原样（`logs/mutant-batch.out`）：

| 变异 | 换法 | 门禁二进制 | 红的测试 |
|---|---|---|---|
| m2a | 低于 F → 被抛弃 | 红（4 条失败） | 快档、复用、回退、逼近墙四段都有「拒绝的理由」对不上 |
| m2b | 树表 0 条 → 低于 F | 红（4） | 四段 |
| m2c | 树表 0 条 → 被抛弃 | 红（4） | 四段 |
| m2d | 不在环里 → 低于 F | 红（4） | 四段 |
| m2e | 低于 F → 树表 0 条 | 红（4） | 四段 |
| m2f | 被抛弃 → 树表 0 条 | 红（4） | 快档、回退、逼近墙三段 + 钉死的被抛弃用例 |
| m2g | 不在环里 → 树表 0 条 | 红（4） | 四段 |

模型给回退的是「所有成立的理由」的集合（`answer_mount_rollback` 按条插入），实现报第一条；换成另一条**也成立**的理由不算错，所以只换顺序的变异不在这里（它不是误报）。回退到树表 0 条的根：快档 125 次、逼近墙段 2 次（`logs/base-run1.log` 的 `Err 成员` 行），这一几何门禁走得到。

### Z3 发布落点拒绝原因：打中

**打中的变异 m2h**（副本上，`crates/singlefs-core/src/allocator.rs` 第 744 行，用户数据那一处）：

```
-                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);
+                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });
```

结果（`logs/mutant-batch2.out`）：门禁二进制 `GATE74-BINARY-EXIT 0`（13 passed），`cargo test --release --workspace` 38 个测试二进制全 `ok`、`EXTRA-EXIT 0`。同形的另两条对照：m2j（同一换法放在提交内生块那一处第 803 行）被 `second_transaction_step_one_overwrite.rs` 的 `publish_running_out_of_space_midway_leaves_the_allocator_as_it_was` 判红；m2i（用户数据那一处「小盘写满」→「每块盘都没有」）被 `second_transaction_supplement_two_unequal_devices.rs` 的 `filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking` 判红；两条门禁二进制都绿。

**为什么四段看不见**：历史的盘写死 4 GiB（`crates/singlefs-harness/src/history.rs` 第 63 行 `pub const HISTORY_DEVICE_BYTES: u64 = 4 << 30;`），四段的 `Err 成员` 表里 `PlacementRefused` 出现 0 次（`grep -c PlacementRefused logs/base-run1.log` 输出 `0`）。胶水那一格 `crates/singlefs-harness/src/model_comparison.rs` 第 172 行 `        PlacementRefusal::NoFreeSlotOnAnyDevice => explained(ModelRefusalReason::UnitAreaWall),`、第 177 行 `        } => ObservedRefusalReason::Unexplained,` 在门禁里一次都没被走到；它只有 `only_a_placement_refused_on_every_device_passes_as_the_unit_area_wall` 那条手搭拒绝值的单元测试，不经分配器。

**在小盘上它看得见**（副本 `small`：两块等大、单元区 384 槽、journal 环 128 MiB，改动见 `scripts/small-copy-history.diff`；`logs/small-batch.out`）：

| 小盘副本 | 门禁二进制第四段（32 × 150） | 探针 wall 256 段 | 探针 wall_plain 256 段 | 探针 wall_rb 256 段 |
|---|---|---|---|---|
| 基线 | 模型对不上 0；`PlacementRefused(NoFreeSlotOnAnyDevice)` 27 + 1 次，全被单元区墙区间接住 | 0 / 256（落点拒绝 293 次） | 0 / 256（1095 次） | 0 / 256（49 次） |
| m2h | 新发现 3（「模型说该成、实现拒了」） | 20 / 256 | 31 / 256 | 3 / 256 |
| m2j | 新发现 3 | 36 / 256 | 123 / 256 | 5 / 256 |

（小盘副本上基线的门禁二进制也红两条：`an_overwrite_that_fills_the_allocation_node_to_exactly_812_records…` 与第四段的「墙一次都没按真条数放行过」——小盘在 812 条之前先写满单元区，是这个几何本身，不是模型对不上。）

四句：
1. 分不分辨臂：这一格只有一个改法（Z3），它分辨的是「改法的映射有没有被门禁验过」，不是两条臂。
2. 被判的系统当时看不看得到：**门禁四段看不到**——4 GiB 等大盘上分配器从不拒落点，任何只换落点拒绝原因的变异四段都必然全绿，这是「判别子观测不到」那一形（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）。但胶水与模型本身看得到：换到小盘，同一套模型、同一张映射就判红，基线不误红。所以病根在取样几何，不在映射。
3. 满足的是判据字面哪个分句：第一分句「一条只换排除原因或拒绝原因的变异四段都不红」。另加一条比字面更重的事实：m2h 在整个 workspace 里也没人判红。
4. 跑前条款的改法在这一格上还中不中：反向接受条款写「按那一格的改法改代码」。第一轮判决第三节第 2 条的改法（带原因、只把容量不够映射成单元区墙）**今天已经做了，对 m2h 不起作用**——映射是对的，缺的是走到它的历史。这一格要另一种改法（见下）。

**我提的改法**（只在我的副本上量过、被攻过零轮）：

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| A. 74 号加第五段：两块等大小盘（单元区几百槽）跑逼近墙那组比重，不跑 checker | m2h、m2j 这类「容量拒绝报成别的」 | 量过（上表，基线 0 / 768 探针段、门禁形态 0 / 32；m2h 3 / 32、m2j 3 / 32）。代价：小盘副本上第四段那组 32 × 150 用时 6.2 秒（`small-base-gate74bin.log` 末段「用时」行；同机 4 GiB 那一段 8.1 秒），副本上量的 |
| B. `mutations.tsv` 补 m2h 一条，点名 A 那一段或一条钉死的用户数据写满用例 | m2h 从此有会红的检查 | 推的（没实现） |
| C. 反方向（「小盘写满」「各盘落点不一致」报成「每块盘都没有」，m2i 的同类）在等大小盘上走不到，仍只靠 `second_transaction_supplement_two_unequal_devices.rs` | 不修；写明 A 对它不起作用 | 推的 |

### 几何上的基线误红（没打中）

等大小盘见上表，0 误红。盘不等大：模型的几何只有一个 `device_size_in_bytes`（`ModelPoolGeometry`），表达不了不等大的池，执行器也只造等大盘；胶水把「小盘写满」映射成 `Unexplained`，于是不等大的池上任何一次合法的「小盘写满」都会判对不上——但今天没有任何一段历史在不等大池上跑模型，这一分句在门禁里走不到。条款上「小盘写满之后设备集合怎么选」本来就没定：`.claude/kb/decisions/02-RAID条带策略.md` 第 111 行 `**全条带写在小盘先写满之后，「当时可写的设备集合」怎么选、条带怎么退化**。`。记为没打中、没法造。

## M4 跳过 checker 的那一段：打中

判据原文（`research/prompts/_m2-supp3-item2-code-r2-body.md` 第 45 行，整行）：

> | M4 跳过 checker 的那一段 | 新取样点不跑 checker，会不会让某类只有 checker 判得出的错在门禁里没人看 | 一条变异只在新取样点那一类历史上走得到、而 checker 判得出、模型判不出，四段都不红 |

「只在新取样点那一类历史上走得到」的依据：另三段跑 checker，根环转一圈之后停在已知红第 0 条（`crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs` 第 282 行 `/// 不跑池级 checker（`PerStepChecker::Skipped`：跑 checker 的历史在根环转过一圈之后停在已知红第 0 条，走不到 812 条），种子与步数见常量。`），所以「根环转过之后」与「分配记录几百条以上」这两类状态只有第四段走得到，而第四段不跑 checker。

**判 checker 看不看得见的装置**：探针 `PROBE_CHECKER=observe`——执行器照 `PerStepChecker::Skipped` 跑（历史不会被已知红第 0 条截停，与门禁第四段同一条路），另在观察者里每一步对镜像跑 `check_pool_image`，把「只有 I-3.1、且记账已分配 > 遍历」那一形（已知红第 0、1 条的共同形态，我没加「根环转过」那个条件，比清单宽）滤掉，记第一处别的违例。基线：门禁那组比重 [0,512) × 150 共 512 段，`probe_checker_red=0`（`logs/baseline-batch.out` 前两行）。

| 变异（副本上） | 改了什么 | 门禁二进制（四段） | 模型 + 执行器（探针 observe 256 段 + wall_rb 512 段） | checker（探针 observe，256 段） |
|---|---|---|---|---|
| m4a | `crates/singlefs-core/src/mount.rs` 第 289 行回收门槛：环里最旧有效根 > 0（根环转过）时取「最旧 + 1」 | 绿（13 passed） | 0 / 256、0 / 512 | 135 / 256（15 / 32 + 120 / 224），第一处都是 I-3.1、记账已分配 **小于** 遍历（例 `盘 0：记账的已分配 Some(2162688)，遍历全部有效根得到 2228224`）；[32,256) 那 120 段第一处出在可写挂载之后 92 段、抬 F 21、回退 7 |
| m4b | `crates/singlefs-core/src/transaction.rs` 第 1749 行记账「已分配」：分配记录过 600 条时少记一槽 | 绿（13 passed） | 0 / 256、0 / 512 | 256 / 256，I-3.1 与 I-5.2 一起红 |

原样：`logs/mutant-batch3.out`（`GATE74-BINARY-EXIT 0`；`PROBE SUMMARY … observe=true: new_findings=0 probe_checker_red=120` / `=224`；wall_rb 两行 `new_findings=0 probe_checker_red=0`）；前 32 段在 `logs/all-logs.tar.gz` 的 `r1-extras/`。m4a 另被 `crates/singlefs-core/src/mount.rs` 的单元测试 `reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root` 判红（整 workspace 跑时 `45 passed; 1 failed`）；**m4b 整个 workspace 全绿**（`logs/all-logs.tar.gz` 里 `r1-extras/mutant-m4b-*.log` 的最后一段）。

m4b 是人工门槛（600 条）造的，形态是「只在大条数上出错的记账 / 编码」；m4a 是真语义错（根环转过之后把最旧有效根还引用的槽回收出去复用），走得到的前提就是根环转过，只有第四段那类历史有。

四句：
1. 分不分辨臂：不涉及臂；打中的是 Z4 本身（这一段不跑 checker）。
2. 系统当时看不看得到：checker 在同一批镜像上看得到（上表）；模型看不到——模型不记落点、不记记账行（第一轮判决第一节 M4 那条设计事实：落点与分配记录归池级 checker 判），m4a 要等回退到被复用槽的那条根再冷启动读回才可能在内容上露出来，512 + 256 段里 0 次。
3. 满足判据字面哪个分句：唯一那一句，四个条件都成立——只在第四段那类历史上走得到（另三段门禁里绿）、checker 判得出、模型判不出、四段都不红。
4. 跑前条款的改法在打中的格上还中不中：条款写「按那一格的改法改代码」，而 Z4 这一格没有写改法（第一轮判决第三节第 1 条只说补新取样点，主 agent 定「接进 74 号」，都是「不跑 checker」的形态）。照今天的形态，两条都还中。

**我提的改法**（只在我的副本上量过、被攻过零轮）：

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| D. 第四段改成跑 checker、但已知红第 0 条那一形不停历史（记下、接着走），别的违例照样停 | m4a、m4b | 量过，就是探针的 observe 模式：基线 0 / 512、m4a 135 / 256、m4b 256 / 256。代价：门禁那组 [32,512) × 150 在 observe 下 261.8 秒（24 线程，`base-wall-observe-32-512.log` 的 `finished in`），不跑 checker 的 [0,2000) × 150 是 504.4 秒，折成每段约 0.55 秒对 0.25 秒；门禁那 32 段约多 2 倍用时，推的 |
| D′. 只在根环没转过的步跑 checker | 都不修（两条都在转过之后） | 推的 |
| E. `mutations.tsv` 补 m4a（点名 D 那一段），不只靠 `reclaim_floor` 的单元测试 | m4a 的端到端判别力 | 推的 |

注意 D 的滤法比已知红第 0 条宽（没要求根环转过），这是装置的限度：一条只在转圈前、只让「记账 > 遍历」的变异会被它放过。

## 没打中的形状（试过什么、取样多大）

- M1 误红：读码找了三处口径可能分开的地方（回收不删记录、复用改写不加条、罩住的已回收记录删除；挂载 / 回退重建分配器的来源；可写挂载的所选根），都对得上。扫法见 M1 那张表：6 种比重、9 批、9512 段、59681 次墙拒，0 次对不上。没造崩溃（执行器不建崩溃；崩溃之后 `mount_writable` 的 `effective_root` 可能不是最新根，那一形这里走不到）。
- M1 漏判：5 条 core 变异（m1a–m1e），门禁二进制都红；m1c、m1d 的种子窗余量薄，已在上面写明。
- M2 回退：7 条只换字段的变异，四段都红。只换检查次序的变异没试：模型收「所有成立的理由」，换成另一条成立的理由不是误报，按字面不该红。
- M2 几何：等大小盘（单元区 384 槽）三种比重各 256 段 + 门禁形态 32 段，0 误红；1280 槽那一版在 812 条之前写不满单元区，没用。盘不等大没造（模型表达不了）。

## 这条腿自己的限度

- 全部是仓副本上的数，不进 kb；入库装置上要重做才能引。
- **一批作废的日志**：小盘副本第一次跑完 m2j 之后，我改 `history.rs`（只动 harness）再跑「基线」，`scripts/mutate.py` 还原 `allocator.rs` 时用的是 `.orig` 的旧 mtime，cargo 把带 m2j 的 `singlefs-core` 产物当成新鲜的留着——那批「等大小盘上基线报『小盘写满』」是 m2j，不是基线。发现后加了 `os.utime`、摸过源文件重编，重跑的 `logs/small-batch.out` 才是上表的数；作废的在草稿目录 `logs/stale-small/`，没进 tar 包。仓副本 `mut` 上第一至三批每条变异都改 `singlefs-core` 里的一个文件、整 crate 重编，没有吃到旧产物；`repo` 副本从没施加过变异。
- 第一批变异的 extra 用 `| head -40` 截输出，cargo 吃到 SIGPIPE 退 101：m1b、m1e 的第一条探针、m4a / m4b 的第二条探针报 `EXTRA-EXIT 101` 而没有汇总行。m4a / m4b 已在第三批不截输出重跑（上表的数取那一批）；m1b 的 wall [32,512) 没重跑，表里写「输出被截」。
- 探针的已知红滤法比清单宽（不看根环转没转），见 M4 末句。
- 小盘副本除了盘大小还把 journal 环从默认缩到 128 MiB（不然 mkfs 要求盘 ≥ 4 × 环），这是几何上的另一处改动。
- m4b 的 600 条门槛是人造的；它证的是「大条数才露的记账错第四段没人看」这一类，不是某个真实缺陷。
- 本机负载：开工时 `ps` 没有 `qemu-system`、`vm-bench.sh`、`e152-*`、`fio`、别的 `cargo` / `gate.sh`；全程只跑我自己起的 cargo，`nice -n 19`。

## 没做什么

- M3（报告里的数、`_ =>` 通配臂、kb 旧成员名）与 M2 里「树表 0 条按条款算不算候选排除」：按分工不归我，没核。
- 没跑门禁 59 号整表、没跑 `gate.sh`；没在入库装置上重做任何一个数。
- 没把改法 A、D 实现进 `crates/`：只在副本上以探针形态量过；B、C、D′、E 是推的。
- 不等大盘上的模型对拍没造（`ModelPoolGeometry` 只有一个盘大小）。
- 没抽第二次样：每个结论只跑过一轮（变异与探针都是确定性的，同种子重跑逐项相同是门禁里 `the_same_seed_runs_to_the_same_outcomes_and_the_same_bytes_twice` 那条测试管的，我没另证）。

**什么现象会推翻这份报告的打中**：M2——有人指出一条经分配器、在 m2h 下会红的现有测试（我跑的是 `cargo test --release --workspace`，不含 `#[ignore]`）；或 m2h 这种换法按条款也算合法（小盘写满与每块盘都满在等大盘上是同一件事）。M4——另三段在门禁配置下对 m4a / m4b 红；或模型 / 执行器在别的种子窗里判出它们；或主 agent 判第四段不跑 checker 本来就被判据接受（判据写的是「四段都不红」即打中）。
