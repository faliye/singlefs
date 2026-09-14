# C322（取号那一步的屏障怎么放没有条款） 第一轮 · Opus 反推腿报告（2026-09-14）

- 立场：假设甲（只补条款、不加屏障）是错的，找它放过的真错（J2、J3）与 J4 的反例；核 P6 那 2 个状态是不是全部；乙也攻。
- 输入：提示 `research/prompts/c322-r1-opus.md`、背景材料 `research/prompts/_c322-r1-background.md`。条款与源码都现查，命令与行号在「一、现查清单」；背景材料附录里抄的 11 段 kb 原文逐字节核过与今天的文件相同（「一、现查清单」末尾那张表）。
- 模型：`research/prompts/c322-r1-opus-model/`。
  - 模型 A `real_stream/`：依赖仓里 `crates/` 下四个 crate（只读、不改），在内存盘上重跑 mkfs → 取号 → 暖机 → 第一个事务，用 harness 的 `writes_and_segments` 切段、逐状态枚举，逐状态读两盘择到的超级块、每盘两槽、根环、journal、持久单元头的写序。
  - `real_stream/src/bin/foreign_disk.rs`：外盘替换那一格（J3）。
  - 模型 B `abstract_paths/`：只用 std，自己实现切段与枚举；把「取号之后本实例的第一次发布」分四种（mkfs 之后第一次可写挂载的暖机空发布、非干净结束之后的恢复写行、实例切换、管理员回退），三条臂各枚举一遍；另有已发布谓词的翻转与两次取号的世代号历史。
  - 产物原样存在同目录：`abstract-paths.out`、`real-stream.out`、`foreign-disk.out`、`layer0-full-run-key-lines.out`（仓里层 0 全量用例的计数行）。复跑命令在「十、复跑」。
- 甲″ 是反推腿攻击途中加的对照臂（取号两盘写完 → 一道屏障，不串行），**不在跑前登记的三条臂里**。它的数只当观测报；要采纳它得另起一轮、跑前登记。

## 〇、结论表

| 问 | 甲 | 乙 | 甲″（对照，未登记） | 细节 |
|---|---|---|---|---|
| P6「恰 2 个」全不全 | 对层 0 那条流是全的：262165 个状态里两盘择到的超级块实例代号不等恰 2 个，持久写集合 {a1@盘0}、{a1@盘1}；两个状态根环只有实例 0、journal 一条记录都没有、恢复择 (0, 0) 读回无文件。撕裂没藏状态：超级块 481 字节住槽的第一个 512 字节扇区，8/8 次超级块写的 [481, 4096) 全 0，按扇区撕开只能是整份旧或整份新。模型 B 用自己的切段在同形流上也数出 2。**但层 0 只有「mkfs 之后第一次可写挂载」这一条路径** | — | — | 二 |
| J1 C322 原句的状态可不可达 | 层 0 那条流里 0 个（只落一盘的 2 个状态里后面的写一个都没持久）；恢复写行 / 实例切换 / 管理员回退三条路径上，同形状态（取号只落一盘、本实例的单元已持久）每条路径 30 个 | 0 | 0 | 三 |
| J2 下一次取号撞号 | **触发**。三条非首次挂载路径各 15 个状态，最小持久写集合 {instance_table_unit@disk0}（取号那两个超级块写一个都没落）；首次挂载那条流 0 个 | 0 | 0 | 四 |
| J3 新条款判成立而镜像是错的 | **触发**。J2 那 15 个状态上，甲改过的 I-7.7（保留 / 删掉「≥ 根环」那半句两种读法）都判成立，今天字面的 I-7.7 也判成立；层 0 流摘掉暖机开头那道屏障后另有 3 个同形状态 | 状态不可达 | 状态不可达 | 五 |
| J3 外盘替换 | 两盘都是各自那个池第一次挂载取的实例 1 ⇒ checker 的 I-7.7 判成立；只有 I-2.1 判红，恢复报 `SuperblocksDisagree`。三条臂一样，不分辨臂 | 同 | 同 | 五 |
| J4 世代号同步 / C322 那条自证 | 层 0 里两盘世代号不等的状态 131078 个，世代号序与实例代号序冲突 0 个 ⇒「只看世代号」那条变异在层 0 上与原式同值。它是取样点不敏感、不是等价：甲 + D22（单元原子性怎么合成） 已定项 16 的逐盘 +1 + 连续两次各只落一盘的取号，16 条历史里 1 条冲突 | 9 条历史里 0 条（前缀序） | 1 / 16，同甲 | 六 |
| 乙的「只落盘 0」 | — | 还在：首次挂载流 1 个、恢复写行路径 1 个；今天字面的 I-7.7 在它上面照样判红 ⇒「I-7.7 字面不改」与乙自己的可达集合对不上 | — | 七 |
| J5 代价 | 0 道额外 FLUSH；但非首次挂载路径 J2 / J3 触发 | 物理上每次取号 +2 道；层 0 状态数 262165 → 262164，E142（第一个事务的干跑） 与八那张登记表的「取号」一行要改 | 物理上每次取号 +1 道；首次挂载路径上它与暖机开头那道紧挨着，录制流里并成一道，E142 与层 0 一个字节不变；条款写成「至少一道完成了的屏障」时首次挂载路径不必另发 | 八 |
| J6 八那一节 | 标题「根槽写路径」罩不住「取号」一行（那一行自己写着「不是根槽写路径」）；更要紧的是「实例切换 / 管理员回退」一行的预想段序列从 [实例表单元 + COW 单元] 开头，取号那几个超级块写压根不在表里——J2 住的正是这一格；非干净结束之后的恢复写行那条路径在表里连一行都没有 | | | 九 |

**按跑前写死的反向接受条款**（背景材料第 37 行：「若任何一条腿证明甲放过一个真错（J2 或 J3 触发，写出镜像）而乙不放过，取乙或丙」）：J2、J3 在甲下触发、在乙下不触发，持久写集合写在「四、J2」⇒ 条款字面指向乙或丙。
反推腿另报一条观测：关掉 J2 / J3 的是「取号之后、本实例第一个非超级块写之前有一道完成了的屏障」，不是串行；乙比甲″ 多买到的只是两盘前缀序（去掉「只落盘 1」、世代号择与实例代号择恒一致），多付一道 FLUSH。
**失败条款不触发**：P6 那 2 个状态的持久写集合与根环、journal 的内容对得上（「二、P6」）。

## 一、现查清单

| 查什么 | 命令 | 结果（行号） |
|---|---|---|
| D23（journal 的角色与格式） 已定项 16 | `grep -n '已定项 16' .claude/kb/decisions/23-journal的角色与格式.md` | 只命中第 695 行（索引表那一行；它写的「正文见 D23「已定项 16」」指向的正文小节不存在，与 P1 一致） |
| D22（单元原子性怎么合成） 已定项 16 | `grep -n '已定项 16' .claude/kb/decisions/22-单元原子性怎么合成.md` | 第 232 行（索引行）、第 1003 行（正文小节标题），正文 1003–1019 |
| D16（发布语义） 已定项 7 / 8 | `grep -n '^### 已定项 7\|^### 已定项 8' .claude/kb/decisions/16-发布语义.md` | 已定项 8 在第 205 行、已定项 7 在第 285 行 |
| I-7.7（超级块实例代号不低于根环） | `grep -n 'I-7.7' .claude/kb/invariants.md` | 表格行在第 53 行 |
| C322 | `grep -n '^| C322 ' .claude/kb/checks-owed.md` | 第 301 行 |
| 实例表的行什么时候写、已发布谓词、取号的写法 | `grep -rn '写行\|恢复行' .claude/kb/decisions/*.md` | `.claude/kb/decisions/18-块里携带什么信息.md` 第 879 行（在第 775 行「### 已定项 11」小节里；背景材料没抄它，附录整行抄） |
| 八那张登记表的三行 | `grep -n '实例切换 / 管理员回退\|第一次可写挂载取号\|^| 空发布' .claude/kb/first-txn-layout.md` | 第 389（取号）、390（空发布）、392（实例切换 / 管理员回退）行；小节标题第 378 行 |
| 运行时决策路径不许遍历 | `grep -n '运行时决策路径' .claude/rules/fs-design.md` | 第 23 行 |
| 实现的择超级块 | 读 `crates/singlefs-core/src/recovery.rs` | 191–229：逐盘取世代号大的槽，各盘只比 fsid 与设备数，返回第一块盘那一份 |
| 实现的取号 | 读 `crates/singlefs-core/src/transaction.rs` | 49：`SUPERBLOCK_GENERATION_AT_INSTANCE_ACQUISITION: u64 = 2`；174–178：发布之后的世代号 = txg + 2；180–192：`acquire_instance` 由调用方传「上一个实例代号」，一次 `RotateSuperblockSlots` 给每块盘写同一个世代号（124–147） |
| checker 的 I-7.7 | 读 `crates/singlefs-checker/src/walk.rs` | 528–536：各盘择到的超级块实例代号全相等；564–575：都 ≥ 根环最大实例代号。`crates/singlefs-checker/src/image.rs` 131–171：每盘两槽择世代号大的；**全仓 checker 不比两盘超级块的 fsid**（walk.rs 527 取 `chosen[0]` 的几何当全池几何） |
| 层 0 的切段与枚举 | 读 `crates/singlefs-harness/src/crash.rs` | 289–328 切段（屏障关段、FUA 写关自己那一段）；594–640 枚举（前面的段全持久 + 当前段任意真子集 + 全持久那一个）；30–62 扇区粒度 512 |
| 层 0 把 I-7.7 钉成 2 | 读 `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs` | 107 行 `let expected_violated = if invariant == "I-7.7" { 2 } else { 0 };`；129–132 行段序列 `[2, 2, 1, 2, 2, 1, 18, 2, 1, 2]` |
| 录制流把相邻屏障记成一道 | `grep -n '连续几道屏障之间没有任何写' crates/singlefs-harness/src/lib.rs` | 第 83 行 |
| 层 0 全量现跑 | `cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture`（与门禁 54 号同一条命令） | 6 passed、170.26 s；计数行三行原样存进 `c322-r1-opus-model/layer0-full-run-key-lines.out`，关键两段整行抄在「二、P6」 |

背景材料附录与今天的 kb 逐字节核对（命令：`diff <(sed -n 'A,Bp' kb 文件) <(sed -n 'C,Dp' research/prompts/_c322-r1-background.md)`，11 段全部 `SAME`）：

| kb 文件:行 | 背景材料:行 |
|---|---|
| `23-journal的角色与格式.md:248-698` | 320-770 |
| `23-journal的角色与格式.md:1204-1241` | 776-813 |
| `22-单元原子性怎么合成.md:46-92` | 819-865 |
| `22-单元原子性怎么合成.md:1003-1019` | 871-887 |
| `16-发布语义.md:205-245` | 893-933 |
| `16-发布语义.md:285-306` | 939-960 |
| `invariants.md:40-65` | 966-991 |
| `first-txn-layout.md:28-84` | 997-1053 |
| `first-txn-layout.md:378-399` | 1059-1080 |
| `checks-owed.md:301` | 1086 |
| `checks-owed.md:119` | 1092 |

⇒ 下面引这 11 段时只写「kb 文件:行」，原文就是背景材料附录里那一份；背景材料没抄的（D18 第 879 行、fs-design.md 第 23 行）在附录整行抄。

## 二、P6：层 0 那 2 个状态是不是全部

**正推**。模型 A 用仓里同一串调用重录 mkfs 之后那条流、用 harness 的 `writes_and_segments` 切段，写表 33 条、段序列与层 0 用例钉的一致；每条写落盘之后带着的实例代号全是 1（`carried_instances`）。取号那两个超级块写（a1@盘0、a1@盘1）独占第 0 段，第 0 段的真子集只有 {}、{a1@盘0}、{a1@盘1}；过了第 0 段，之后每一个超级块写都带实例 1，两盘不可能再不等。产物 `c322-r1-opus-model/real-stream.out` 整行抄：

```text
ORIGINAL segments=[2, 2, 1, 2, 2, 1, 18, 2, 1, 2] writes=33 carried_instances=[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
ORIGINAL states=262165 instances_unequal=2 generations_unequal=131078 generation_instance_order_conflicts=0 generation_only_choice_differs=0
UNEQUAL persisted=[a1@disk0] chosen(generation,instance)=[(2, 1), (1, 0)] root_ring_instances={0} journal_instances={} recovery=NoFile { root: (InstanceGeneration(0), CheckpointTxg(0)) } checker_I-7.7=Some(Violated("各盘择到的超级块实例代号不相等：[1, 0]"))
UNEQUAL persisted=[a1@disk1] chosen(generation,instance)=[(1, 0), (2, 1)] root_ring_instances={0} journal_instances={} recovery=NoFile { root: (InstanceGeneration(0), CheckpointTxg(0)) } checker_I-7.7=Some(Violated("各盘择到的超级块实例代号不相等：[0, 1]"))
```

仓里层 0 全量用例现跑的计数行（`c322-r1-opus-model/layer0-full-run-key-lines.out` 整行抄）：

```text
LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=9 verification_failed=0 journal_differing=3 exhaustive=true
CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-1.3=262165/0 I-1.4=262165/0 I-1.6=262165/0 I-1.7=262165/0 I-2.1=262165/0 I-2.3=262165/0 I-2.4=262165/0 I-2.5=262165/0 I-3.1=4/0 I-5.1=262165/0 I-5.2=4/0 I-7.1=262165/0 I-7.2=262165/0 I-7.6=262165/0 I-7.7=262165/2 I-7.8=262165/0 I-9.1=4/0 I-9.2=4/0 I-9.4=4/0 I-9.7=4/0 I-9.10=4/0 I-9.13=4/0
CHECKER_FIRST {"I-7.7": "各盘择到的超级块实例代号不相等：[1, 0]"}
```

⇒ 两个状态的持久写集合就是 P6 说的那两个；根环实例集合 `{0}`、journal 实例集合 `{}`、恢复择 (0, 0) 读回无文件、checker 的 I-7.7 判红。**失败条款（「那 2 个状态里有一个根环或 journal 里已有实例 1 的东西」）不触发。**

**反推**：假设「恰 2 个」是切段口径造出来的，本该看到下面几样之一，逐样查了，都没有：

| 假设的漏法 | 本该看到什么 | 看到的 |
|---|---|---|
| 后面某一段里还有两盘实例代号不等的状态 | 模型 A 全量 262165 个状态里 `instances_unequal` > 2 | `instances_unequal=2`（上面那行 `ORIGINAL states=262165 …`） |
| 撕裂态被「并进没持久」藏掉了（超级块是原地覆写，撕开的槽校验和不过、盘退回另一个槽，那不等于「旧字节还在」） | 某次超级块写撕开之后留下一个既不是旧也不是新的槽 | 超级块 481 字节住 4096 字节槽的第一个 512 字节扇区，产物 `SUPERBLOCK_PADDING superblock_writes=8 bytes_481_to_4096_all_zero=8`；mkfs 的超级块同一个 `to_slot`（`crates/singlefs-core/src/superblock.rs` 146 行断言 481）。按 512 字节扇区撕开只能留下整份旧的或整份新的 ⇒ 并进「没持久 / 持久」是精确的，不是近似 |
| 两盘崩溃时点不同步 | 某个状态里盘 0 的写属于更后的段而盘 1 的更早段没持久 | 屏障对两盘都发（`transaction.rs` 149–153），后一段的写在前一段全部持久之后才发出；段内任意子集已经含跨盘的所有组合 |
| mkfs 那 13 步没枚举（层 0 从 mkfs 之后起，八那张表 mkfs 一行写「不在（G19）」） | mkfs 中途两盘实例代号不等 | mkfs 写的全是实例 0（`make_filesystem.rs` 273–291），不可能不等 |

**校验（独立路子）**：模型 B 不用 harness，自己切段、自己枚举，把首次挂载那条流抽象成（种类, 盘, 实例代号），甲臂数出 `superblocks_unequal=2`（`abstract-paths.out` 第 1 行）。两条路子只共用「屏障关段、FUA 写关自己那一段」这一句切段规则（它来自 D13（验证路线） 已定项 4），不共用代码。

⚠️ **射程**：「恰 2 个」只对层 0 那条流成立。那条流只有「mkfs 之后第一次可写挂载」这一条路径——八那张登记表「实例切换 / 管理员回退」一行（first-txn-layout.md:392）的层 0 枚举列逐字是「不在」，非干净结束之后的可写挂载（恢复写行）在表里没有行。J1、J2、J3 打中的全在这几条路径上。

## 三、J1：C322 原句的状态可不可达

层 0 那条流里 **0 个**。产物 `real-stream.out` 整行抄：

```text
ORIGINAL exactly_one_acquisition_write=2 exactly_one_acquisition_write_and_any_later_write=0 replacement_check_violations=0
```

「只落一盘」的 2 个状态里第 2 条以后的写一条都没持久：暖机第一条记录 w1 在第 1 段，枚举走到第 1 段时第 0 段已经全持久。⇒ C322 要的那条判定在今天的流里**没有输入**（与 C124（回退行与重放下界没有会红的检查） 那句「按今天的条文没有输入」同形）。

在另三条路径上同形状态可达。模型 B（`abstract-paths.out` 第 4–6 行，恢复写行路径；实例切换第 7–9 行、管理员回退第 10–12 行逐字相同）：

```text
PATH first_publish=mount_after_unclean_end(recovery_row_publish) arm=jia(no_barrier) segments=[6, 2, 1, 2, 2, 1, 2] barriers=4 states=78 superblocks_unequal=32 literal_I-7.7_violations=32 jia_I-7.7_violations(keep_root_clause)=0 replacement_check_violations=15 reuse=15 reuse_while_literal_I-7.7_holds=15 reuse_while_jia_I-7.7_holds(keep_root_clause)=15 reuse_while_jia_I-7.7_holds(drop_root_clause)=15 first_reuse_persisted=Some(["instance_table_unit@disk0"])
PATH first_publish=mount_after_unclean_end(recovery_row_publish) arm=yi(serial_disk0_barrier_disk1_barrier) segments=[1, 1, 4, 2, 1, 2, 2, 1, 2] barriers=6 states=32 superblocks_unequal=1 literal_I-7.7_violations=1 jia_I-7.7_violations(keep_root_clause)=0 replacement_check_violations=0 reuse=0 reuse_while_literal_I-7.7_holds=0 reuse_while_jia_I-7.7_holds(keep_root_clause)=0 reuse_while_jia_I-7.7_holds(drop_root_clause)=0 first_reuse_persisted=None
PATH first_publish=mount_after_unclean_end(recovery_row_publish) arm=jia_double_prime(both_then_one_barrier) segments=[2, 4, 2, 1, 2, 2, 1, 2] barriers=5 states=33 superblocks_unequal=2 literal_I-7.7_violations=2 jia_I-7.7_violations(keep_root_clause)=0 replacement_check_violations=0 reuse=0 reuse_while_literal_I-7.7_holds=0 reuse_while_jia_I-7.7_holds(keep_root_clause)=0 reuse_while_jia_I-7.7_holds(drop_root_clause)=0 first_reuse_persisted=None
```

甲那一行 `superblocks_unequal=32`：取号两写与四个单元写同在一段（`segments=[6, …]`），只落一盘的 2 × 2⁴ = 32 个里有 2 个是单元一个没落（{acquire_superblock@disk0}、{acquire_superblock@disk1}），其余 30 个都有本实例单元持久（算术）。乙、甲″ 下单元在关掉取号的那道屏障之后，这类状态 0 个。

## 四、J2：下一次取号拿到一个盘上已有单元带着的实例代号（甲下触发）

### 甲 ① 的前提只在一条路径上成立

甲 ① 写的是「暖机第一次空发布开头那道屏障就是持久点（本实例的第一个非超级块写排在它之后）」。括号里那句只在 mkfs 之后第一次可写挂载成立：那条路径取号之后接的是空发布，空发布的第一步就是屏障（first-txn-layout.md:390「屏障 [w1 空记录 × 2 盘] 屏障 [w2 根 FUA] …」）。空发布开头那道屏障其实是 D16（发布语义） 已定项 7「COW 单元/节点 → 屏障 → journal 记录 → …」里单元集为空时剩下的那一道——单元集不空，屏障就排在单元写之后。

另三条路径上，取号之后本实例的第一次发布先写单元：

- D18（块里携带什么信息） 第 879 行（已定项 11，附录整行抄）逐字：「**行怎么写**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2）：按实例代号唯一、后写覆盖，只在恢复与回退时写、与第一个新根同一次发布——任何非干净结束的实例在下次恢复写行」「实例切换（D23（journal 的角色与格式） 已定项 14 的注）同崩溃行；每次写行 COW 重写整条链」。
- first-txn-layout.md:392 那一行的预想段序列逐字：「[实例表单元 + COW 单元] 屏障 [记录] 屏障 [根 FUA] [超级块槽]，**之后接新实例的暖机**」。

⇒ 非干净结束之后的可写挂载、实例切换、管理员回退，本实例的第一个非超级块写是实例表单元（外加恢复要重写的固定点单元、切换要重发的单元、回退那次发布的 COW 单元），它们排在第一道屏障**之前**。甲不另加屏障，取号那两个超级块写就与这些单元写同在一段（模型 B `segments=[6, …]`）。

**能不能把写行那次发布挪到暖机之后、让甲 ① 重新成立**：按 D18 第 879 行自己的谓词不能（推理，按条文，没建模）。「i < i_now 且无行 ⇒ 已发布」：暖机那几个新根挂上之后、写行之前，上一个非干净结束的实例还没有行，它号 > W 的孤儿单元全部判已发布——「与第一个新根同一次发布」这一句是承重的。

### 构造（恢复写行路径；实例切换、管理员回退同形）

- **前态**（两盘）：第一个事务写完之后掉电，实例 1 非干净结束。超级块槽 0 = 世代号 4 / 实例 1、槽 1 = 世代号 5 / 实例 1；根环最大 (实例 1, txg 3)；journal 里是实例 1 的记录。
- **第二次挂载**：恢复择 (1, 3)；取号 = max(超级块 1, 根环 1) + 1 = 2，两盘各写一次超级块槽（D22（单元原子性怎么合成） 已定项 16：世代号 6，落槽 6 mod 2 = 0）；接着第一次发布（txg 4）先写实例表单元（行 (1, 3, 0)）两份、恢复要重写的固定点单元两份，然后才是屏障 → 记录 → 屏障 → 根槽 FUA → 超级块。甲下崩溃段 = {acquire_superblock@disk0, acquire_superblock@disk1, instance_table_unit@disk0, instance_table_unit@disk1, rewritten_fixed_point_node@disk0, rewritten_fixed_point_node@disk1}。
- **崩溃状态 Z 的持久写集合**（最小的一个）：{instance_table_unit@disk0}。取号那两个超级块写一个都没落；盘上多了一个写序实例代号 2 的实例表单元（落在按所选根的账是空闲的槽上，没有根引用它）。
- **第三次挂载**：两盘超级块都是实例 1、根环最大实例 1 ⇒ 甲的规则 max(全部超级块, 根环) + 1 = **2**，正是 Z 里那个单元的写序实例代号 ⇒ **J2 触发**。

模型 B 在三条路径上各数出 15 个这样的状态（四个单元写的非空子集 × 两个超级块写都没落），乙、甲″ 下 0 个（`abstract-paths.out` 第 4–12 行的 `reuse=` 字段；甲那一行的 `first_reuse_persisted=Some(["instance_table_unit@disk0"])` 就是 Z）。首次挂载那条流三条臂都是 0（第 1–3 行）。

### 撞号坏在哪：同一批孤儿从「判损坏」翻成「已发布」

D18 第 879 行逐字：「**作废一个 checkpoint 必须换实例代号**：当前实例没有行，谓词对它只能问「诞生代号 ≤ 挂载根 txg」，重发同一个号一发布，诞生代号等于它的孤儿全部判已发布——这是失败表能封闭的依据」。甲下的撞号就是「重发同一个号」。模型 B 按那一行的四支谓词逐字实现，喂 Z 里那类孤儿（码 1：写序 (2, 1)、诞生 txg 4；码 2 / 3：实例 2、诞生 txg 4），产物整行抄：

```text
PREDICATE orphan=OrphanUnit { instance: 2, transaction: 1, birth_txg: 4, class: DataUnitCodeOne } mount_root(1,3)=Corrupt jia_reused_instance2_root_txg4=Published jia_reused_instance2_root_txg5=Published fresh_instance3_root_txg4=Unpublished fresh_instance3_root_txg5=Unpublished
PREDICATE orphan=OrphanUnit { instance: 2, transaction: 0, birth_txg: 4, class: NodeOrPackedCodeTwoOrThree } mount_root(1,3)=Corrupt jia_reused_instance2_root_txg4=Published jia_reused_instance2_root_txg5=Published fresh_instance3_root_txg4=Unpublished fresh_instance3_root_txg5=Unpublished
```

- 撞号之前（挂载根 (1, 3)）：「i > i_now ⇒ 判损坏」。
- 甲撞号之后（复用的实例 2，行只写到 [1, 2) = (1, 3, 0)，根 (2, 4)）：「i == i_now ⇒ b ≤ 挂载根的 txg」⇒ **已发布**。同一批字节，谓词翻面。
- 超级块已带 2 时（乙、甲″ 下单元落了盘就一定如此）：取号得 3，行写到 [1, 3)，含 (2, 3, 0) ⇒ 未发布，正确。

⇒ I-1.2（块头写序已发布） 的扫描方向依据「判不成已发布者必是未发布的撕裂事务垃圾」在这批孤儿上反过来了：它们是垃圾、却判成已发布。
⚠️ **没建模的一半**：把谓词翻面走到「扫描重建复活了一份错的版本」，要第二次尝试写出的内容与孤儿不同、身份相同。同一实现在同一个盘面上重做恢复，写出的多半逐字节相同、落在同一个槽上把孤儿盖掉；实例切换重发的是还没返回的调用者的数据，崩溃之后那些调用者没了，第二次尝试写的是别的东西——那一格不同。这一步是推理，模型只量了谓词翻面。

### 三条臂共有的一个缺口（不分辨臂）：世代号规则与「全部超级块」的读法

- D22（单元原子性怎么合成） 已定项 16 字面是「槽世代号从 1 起、每写一次 +1」（每盘各数）；实现一次 `RotateSuperblockSlots` 给每块盘写同一个世代号（`transaction.rs` 124–147），取号那次恒写 2（第 49 行常量）、发布之后恒写 txg + 2（174–178）。这两个常量只对第一个实例对：第二次挂载的取号写世代号 2、而盘上已有世代号 5，「每盘两槽里择世代号大的」看不见这次取号。
- 模型 B `GENERATION … rule=TodayConstantTwoAtAcquisition` 三行：甲 16 条两次取号历史里 15 条、乙 9 条里 8 条、甲″ 16 条里 15 条，有某个槽带着比「按世代号择到的」更大的实例代号。
- 甲 ② 写「max(全部超级块, 根环) + 1」，D18 第 879 行写「max(这次挂载独占打开成功的那个集合里各超级块的代号, …)」。若「各超级块」读成每盘择到的那一份，而世代号照今天的常量走，取号之后单元已落盘（乙、甲″ 下有屏障也一样）再崩，下一次取号照样撞号。⇒ 不论取哪条臂，都要写死两句：「全部超级块」= 每盘两槽里全部自证过的槽；之后的取号世代号怎么取（逐盘 +1 还是全池最大 + 1）。

## 五、J3：新条款判成立、而镜像是错的

### J3-1：J2 那 15 个状态上，今天的 I-7.7 与甲改过的 I-7.7 都判成立（甲下触发）

`abstract-paths.out` 第 4 行（整行已抄在「三、J1」）里，甲那一格：`reuse=15 reuse_while_literal_I-7.7_holds=15 reuse_while_jia_I-7.7_holds(keep_root_clause)=15 reuse_while_jia_I-7.7_holds(drop_root_clause)=15`。拿最小的 Z 看：两盘超级块都是实例 1 ⇒ 甲改过的条款走「相等」那一支、成立；「≥ 根环里任一根」1 ≥ 1、成立；今天的字面同样成立。而盘上有一个写序实例代号 2 的单元，下一次取号就撞上它（「四、J2」）。⇒ **J3 触发**，镜像就是 Z（持久写集合 {instance_table_unit@disk0}）。乙、甲″ 下 Z 不可达（`reuse=0`）。

### J3-2：层 0 那条流摘掉甲 ① 点名的那道屏障

甲 ① 说暖机第一次空发布开头那道屏障是取号的持久点。模型 A 把它摘掉（mkfs 之后第 3 步，`removed_step=2` 从 0 数），取号两写与 w1 两写并成一段再枚举。`real-stream.out` 第 8–12 行整行抄：

```text
NO_WARM_UP_LEADING_BARRIER removed_step=2 segments=[4, 1, 2, 2, 1, 18, 2, 1, 2] writes=33
NO_WARM_UP_LEADING_BARRIER states=262174 equality_violations=8 jia_I-7.7_violations=6 replacement_check_violations=3 reuse=3 reuse_while_jia_I-7.7_holds=3 checker_states_in_first_segment=16 checker_I-7.7_violations_in_first_segment=8
NO_BARRIER_STATE persisted=[w1@disk0] superblock_instances=(0, 0) record_instances={1} checker_I-7.7_violated=false jia_I-7.7_holds=true next_acquisition=1 reused=true
NO_BARRIER_STATE persisted=[w1@disk1] superblock_instances=(0, 0) record_instances={1} checker_I-7.7_violated=false jia_I-7.7_holds=true next_acquisition=1 reused=true
NO_BARRIER_STATE persisted=[w1@disk0,w1@disk1] superblock_instances=(0, 0) record_instances={1} checker_I-7.7_violated=false jia_I-7.7_holds=true next_acquisition=1 reused=true
```

那 3 个状态（{w1@盘0}、{w1@盘1}、{w1@盘0, w1@盘1}）两盘超级块都是实例 0、journal 里有实例 1 的记录，下一次取号得 1 ⇒ 撞号；今天 checker 的 I-7.7 不红（它在同一段里红的是另外 8 个「只落一个 a1」的状态），甲改过的 I-7.7 成立。⇒ 两盘相等时，今天的条款与甲的条款都不看 journal、不看单元，「记录跑到超级块前面」这类错一格都拦不住。

### 为什么不能靠把取号的 max 扩到单元来补

取号的 max 若要带上单元写序，就得在挂载时读遍每个单元头——`.claude/rules/fs-design.md` 第 23 行（附录整行抄）把运行时决策路径的遍历判成「**不许**，且代价不许随盘容量增长」。journal 恢复本来就全环扫（D23（journal 的角色与格式） 已定项 3），带上它不多花；单元带不上。⇒ 补法只能是写序：取号的超级块写先持久、本实例的单元后写（推理）。

### 「取号写超级块全或无」在掉电下哪条臂都做不到

两次独立的设备写，掉电时哪条臂都有「两盘不等」的状态：`abstract-paths.out` 第 1–3 行（首次挂载路径）甲 `superblocks_unequal=2`、乙 `1`、甲″ `2`。只有丙（一次原子写认号）能让它全或无。D18 第 879 行的「全或无」讲的是 I/O 错（「一次 I/O 错要重试到 T_retry 用尽才算失败；用尽后全或无判失败时先把已经写出的那几份回卷成旧代号」），而且那里已经写着一条例外：「回卷期间盘上出现的「同一集合内代号不等」是 I-7.7（超级块实例代号不低于根环） 的一条显式例外，下一次挂载按 max + 1 修复」。invariants.md 第 53 行的 I-7.7 没带这条例外 ⇒ **今天两处已经不一致**；甲 ② 是同一条例外的掉电版，应当与它写在同一处。

### 反推腿给的替代检查（候选形态，不是判决）

> ① 盘上每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘择到的超级块实例代号里最大的那个；② 各盘超级块实例代号不等时，较大的那个不出现在任何根记录、journal 记录、单元写序里。

- 判别力：模型 B 的 `replacement_check_violations`（第 1–12 行）甲在三条非首次挂载路径上各 15、首次挂载 0，乙与甲″ 在全部四条路径上 0；模型 A 层 0 那条流 `replacement_check_violations=0`，摘掉暖机开头那道屏障之后 3——两个模型里它都恰好在撞号的状态上红。
- 与「不撞号」的关系（代数）：下一次取号 = max(超级块, 根环) + 1 不撞号 ⇔ max(超级块, 根环) ≥ 盘上任一根 / 记录 / 单元的实例代号。保留今天「超级块 ≥ 根环」那半句时，左边就是 max(超级块)，两者等价于 ①。
- 输入：checker 的扫描方向已经逐个读单元头（`crates/singlefs-checker/src/walk.rs` 475–503 读码 2 头算 I-7.8），带上写序的实例代号不要新格式。

### J3-3：外盘替换（三条臂一样，不分辨臂）

把池 A 的盘 1 换成另一个池（fsid 不同、同样 mkfs → 第一次挂载 → 暖机 → 第一个事务）的盘 1。`foreign-disk.out` 整行抄：

```text
FOREIGN case=home_pool_untouched I-7.7=Some(Holds) violated=[] recovery=FileRead root=(InstanceGeneration(1), CheckpointTxg(3)) bytes=3000
FOREIGN case=disk1_replaced_by_foreign_pool_disk1 I-7.7=Some(Holds) violated=[I-2.1: 实例表单元 在盘 1 槽 50176 的那一份与位置条目里的校验和对不上] recovery=Failed root=None failure=SuperblocksDisagree
```

每个池第一次挂载都取 1（D23（journal 的角色与格式） 已定项 16），两盘实例代号「相等」是巧合；checker 不比两盘超级块的 fsid（`image.rs` 131–171 逐盘择槽，`walk.rs` 527 行拿 `chosen[0]` 的几何当全池几何），恢复比（`recovery.rs` 219–224 报 `SuperblocksDisagree`）。这一格让三条臂一起中，按 evidence-discipline「打中不分辨臂」不拿它判臂，记成 C322 之外的一笔：checker 缺「两盘超级块 fsid 与设备数相符」这一判。

## 六、J4：两盘世代号同不同步；C322 那条自证构造得出吗

### 两盘世代号不恒同步，但层 0 里从不与实例代号反序

- `ORIGINAL states=262165 … generations_unequal=131078 generation_instance_order_conflicts=0 generation_only_choice_differs=0`（整行抄在「二、P6」）。
- 131078 的来历（算术，与产物对得上）：第 0 段 2 个（a1 只落一盘）、第 3 段 2 个（w3 只落一盘）、第 6 段 2 × 2¹⁶ = 131072 个（w6 只落一盘 × 16 个单元写的任意子集）、第 9 段 2 个（t11 只落一盘）。
- 同一步给两盘写的是同一个 (世代号, 实例代号)，实例代号只在取号那一步变 ⇒ 两盘择到的超级块谁的世代号大，谁的实例代号就不小；世代号相等就是同一步写的，实例代号也相等。⇒ C322 那条变异（择超级块只看世代号、不比实例代号）在层 0 的每个状态上与原式同值，**层 0 里构造不出会红的状态**。

### 它是取样点不敏感，不是等价变异

按 `.claude/rules/mutation-sampling.md` 的三分问一句「换一个取样点它还同值吗」：模型 B 在两次取号的历史上找到了不同值的取样点。`abstract-paths.out` 第 15–23 行整行抄：

```text
GENERATION arm=jia(no_barrier) rule=PerDiskIncrement histories=16 generation_only_choice_below_highest_chosen=1 some_slot_instance_above_generation_choice=1 first_example=Some(([true, false], [false, true], [[SlotContent { generation: 6, instance: 2 }, SlotContent { generation: 5, instance: 1 }], [SlotContent { generation: 6, instance: 3 }, SlotContent { generation: 5, instance: 1 }]]))
GENERATION arm=jia(no_barrier) rule=PoolWideHighestPlusOne histories=16 generation_only_choice_below_highest_chosen=0 some_slot_instance_above_generation_choice=0 first_example=None
GENERATION arm=jia(no_barrier) rule=TodayConstantTwoAtAcquisition histories=16 generation_only_choice_below_highest_chosen=0 some_slot_instance_above_generation_choice=15 first_example=None
GENERATION arm=yi(serial_disk0_barrier_disk1_barrier) rule=PerDiskIncrement histories=9 generation_only_choice_below_highest_chosen=0 some_slot_instance_above_generation_choice=0 first_example=None
GENERATION arm=yi(serial_disk0_barrier_disk1_barrier) rule=PoolWideHighestPlusOne histories=9 generation_only_choice_below_highest_chosen=0 some_slot_instance_above_generation_choice=0 first_example=None
GENERATION arm=yi(serial_disk0_barrier_disk1_barrier) rule=TodayConstantTwoAtAcquisition histories=9 generation_only_choice_below_highest_chosen=0 some_slot_instance_above_generation_choice=8 first_example=None
GENERATION arm=jia_double_prime(both_then_one_barrier) rule=PerDiskIncrement histories=16 generation_only_choice_below_highest_chosen=1 some_slot_instance_above_generation_choice=1 first_example=Some(([true, false], [false, true], [[SlotContent { generation: 6, instance: 2 }, SlotContent { generation: 5, instance: 1 }], [SlotContent { generation: 6, instance: 3 }, SlotContent { generation: 5, instance: 1 }]]))
GENERATION arm=jia_double_prime(both_then_one_barrier) rule=PoolWideHighestPlusOne histories=16 generation_only_choice_below_highest_chosen=0 some_slot_instance_above_generation_choice=0 first_example=None
GENERATION arm=jia_double_prime(both_then_one_barrier) rule=TodayConstantTwoAtAcquisition histories=16 generation_only_choice_below_highest_chosen=0 some_slot_instance_above_generation_choice=15 first_example=None
```

- 甲 / 甲″ + 逐盘 +1（D22（单元原子性怎么合成） 已定项 16 的字面）：第一次取号只落盘 0（盘 0 槽 0 = 世代 6 / 实例 2），第二次取号只落盘 1（盘 1 按自己最新的世代 5 + 1 = 6 落槽 0，实例 3）⇒ 两盘择到 (6, 2) 与 (6, 3)，世代号相等、实例代号不等，只看世代号择到 2。16 条两次取号历史里 1 条。
- 乙：9 条里 0 条——盘 0 先写先刷，盘 0 的世代号与实例代号都不会比盘 1 小。
- 全池最大 + 1 的世代号规则：三条臂都 0 条（落后的那块盘直接跳到同一个世代号）。
- ⇒ 会红的取样点要同时满足：甲或甲″、逐盘 +1、连着两次取号各只落一盘。层 0 只有一次挂载，够不着；要在层 0 之上加「多次挂载的历史」才造得出。
- 伤不伤正确性：甲的取号取全部超级块的 max，择到哪一份只影响 tail 与几何（两盘相同），不进取号。今天挂载路径没写（`transaction.rs` 180–192 的 `acquire_instance` 由调用方传「上一个实例代号」，`recovery.rs` 191–229 返回第一块盘那一份）；哪天两者接成「择到的那一份 + 1」，这一格就承重。

### 层 0 里会红的两个替代自证

`real-stream.out` 第 5 行整行抄：

```text
ORIGINAL next_acquisition_collides_with_instance_on_disk: max_over_all_superblocks_rule=0 first_disk_chosen_rule=1 generation_only_choice_rule=0 first_disk_example=[a1@disk1]
```

| 自证 | 变异 | 检查 | 层 0 里绿 → 红 |
|---|---|---|---|
| (a) | 摘掉暖机第一次空发布开头那道屏障（甲 ① 点名的持久点） | 「五、J3」替代检查 ① | 0 → 3（`real-stream.out` 第 9 行 `replacement_check_violations=3`）；今天 checker 的 I-7.7 在这 3 个状态上不红，当不了这条自证的检查 |
| (b) | 下一次取号改用「第一块盘择到的那一份」（`recovery.rs` 今天返回的就是它）+ 根环，不取全部超级块的 max | 下一次取号不撞盘上任何实例代号（每盘两槽 + 根 + 记录 + 单元） | 0 → 1（`first_disk_chosen_rule=1`，状态 [a1@disk1]）；只看世代号的那一版仍是 0（`generation_only_choice_rule=0`） |
| C322 原形 | 择超级块只看世代号 | 同 (b) | 层 0 里 0 → 0；只在多次挂载的历史上会红（模型 B 1 / 16） |

(a) 与 D16（发布语义） 已定项 7 那条「摘掉根槽前的屏障必须红」同形，适合当主自证；(b) 管的是取号拿哪个数，当第二条。两条都要先把替代检查 ① 或「不撞号」判定做进 checker 或层 0 的计数，才有检查可跑。

## 七、乙：「只落盘 0」还在不在；乙买到了什么

- **还在**。`abstract-paths.out` 第 2 行（首次挂载路径，乙）`superblocks_unequal=1 literal_I-7.7_violations=1`，第 5 行（恢复写行路径，乙）同样是 1。那个状态的持久写集合是 {acquire_superblock@disk0}：盘 0 先写、先刷，盘 1 还没写就掉电。今天字面的 I-7.7「各超级块的实例代号相等」在它上面判红 ⇒ 乙的「I-7.7 字面不改」与乙自己的可达集合对不上：层 0 用例第 107 行的 2 会变成 1，变不成 0。
- 乙买到的：
  1. 去掉「只落盘 1」，两种不等的形状剩一种。
  2. 本实例的单元一定排在两盘超级块都持久之后 ⇒ J2、J3 关掉（四条路径 `reuse=0`）。这一样甲″ 也买到。
  3. 两盘前缀序：盘 0 永远不落后于盘 1 ⇒ 不论哪种世代号规则，只看世代号择到的都不比最大实例代号小（`GENERATION arm=yi…` 三行都是 0）。反面是 C322 原形那条自证在乙的整个可达域上都与原式同值、永远造不出会红的状态，只能用「六、J4」的 (a)(b)。
- 乙没买到的：I-7.7 的相等（掉电下做不到）；世代号规则那个缺口（今天的常量 2 下 9 条历史里 8 条照样看不见取号，「四、J2」末尾）。
- 乙比甲″ 多付的：每次取号多一道 FLUSH；首次挂载那条流的段序列从 `[2, 2, 1, 2, 2, 1, 18, 2, 1, 2]` 变成 `[1, 1, 2, 1, 2, 2, 1, 18, 2, 1, 2]`，层 0 状态数 262165 → 262164（算术：1 + Σ(2^|段| − 1)），E142（第一个事务的干跑） 要重跑，八那张表「取号」一行与层 0 用例 129–132 行的段序列断言要改。

## 八、J5：代价

模型 B 各路径的屏障道数（录制流记法：相邻屏障记成一道，`crates/singlefs-harness/src/lib.rs` 第 83 行）：首次挂载 甲 4 / 乙 5 / 甲″ 4；恢复写行、实例切换、管理员回退 甲 4 / 乙 6 / 甲″ 5（`abstract-paths.out` 第 1–12 行的 `barriers=` 字段）。

| 臂 | 每次取号额外 FLUSH（物理） | 首次挂载那条流 / 层 0 | 格式 | E142 与八那张表 |
|---|---|---|---|---|
| 甲 | 0 | 不变，262165 | 不改 | 不改；但非首次挂载路径 J2 / J3 触发 |
| 乙 | 2 | 段序列变，262164 | 不改 | E142 重跑；「取号」一行 `2` → `1+1`；层 0 用例段序列断言改 |
| 甲″（未登记） | 1；条款写成「取号之后、本实例第一个非超级块写之前至少一道完成了的屏障」时，首次挂载路径不必另发（暖机开头那道就算） | 不变（那道屏障与暖机开头那道相邻，录制流里本来就记成一道） | 不改 | 首次挂载那几行不改；「实例切换 / 管理员回退」一行与要新增的「非干净结束之后的可写挂载」一行得把取号写与那道屏障写进段序列 |
| 丙 | 看那一次原子写的形态 | 变 | 改 | 全改 |

## 九、J6：八那一节

- 标题（first-txn-layout.md:378）是「根槽写路径的段序列登记表」；「取号」那一行（:389）自己写着「不是根槽写路径」，:382 那段正文也在解释为什么一条不是根槽写路径的东西登记在这里。标题罩不住这一行，字面对不上。
- 更要紧的是缺行：「实例切换 / 管理员回退」一行（:392）的预想段序列从「[实例表单元 + COW 单元]」开头，取号那几个超级块写不在里面；非干净结束之后的可写挂载（恢复写行）在表里没有行。:382 又写「别处引提交步骤一律链到这一节，不另抄」⇒ J2 住的那几条路径上，取号的持久点在全仓没有可以落的地方。
- 反推腿的读法（不是判决）：改成不带「根槽」限定的名字，例如「提交步骤的段序列登记表（根槽写路径与取号）」；「实例切换 / 管理员回退」一行开头补上取号写与它后面那道屏障；加一行「非干净结束之后的可写挂载」。

## 十、复跑

模型是确定性的（没有 I/O、没有随机源、没有并发），跑一遍与跑 N 遍给的信息一样，证据强度不来自轮数。判别力靠对照：模型 A 的替代检查在原样流上 0、摘掉屏障之后 3；模型 B 的撞号在乙 / 甲″ 上 0、在甲上 15；模型 B 首次挂载甲那一格的 2 与层 0 的 2 由两套独立代码得到。两个模型都没有变异表（不在 `research/e7-index-bench/src/bin/` 下，门禁 33 号不管它们），这是一处欠着的证据强度。

```bash
S=<临时目录>; rsync -a research/prompts/c322-r1-opus-model/ "$S/c322-model/"
cd "$S/c322-model/abstract_paths" && CARGO_TARGET_DIR="$S/target-abstract" cargo run --release -q             # 产物 abstract-paths.out（23 行）
cd "$S/c322-model/real_stream" && CARGO_TARGET_DIR="$S/target-real" cargo run --release -q --bin c322-r1-opus-real-stream   # 产物 real-stream.out（12 行，本机约 19 秒）
cd "$S/c322-model/real_stream" && CARGO_TARGET_DIR="$S/target-real" cargo run --release -q --bin foreign_disk               # 产物 foreign-disk.out（2 行）
cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture        # 仓根跑；layer0-full-run-key-lines.out 是它输出里 LAYER0 / CHECKER / CHECKER_FIRST 那三行
```

构建放在临时目录，是为了不在 `research/prompts/` 下生成 `target/` 与 `Cargo.lock`；模型 A 按绝对路径依赖仓里四个 crate（`real_stream/Cargo.toml`）。

## 十一、什么现象会推翻这些结论

| 结论 | 推翻它的观测 |
|---|---|
| J2 / J3 在甲下触发 | 有条款让非首次挂载路径上第一次发布的单元写之前先有一道屏障（那时这几条路径上甲就是甲″）；或 D18 第 879 行「与第一个新根同一次发布」改成暖机在前、写行在后（前提回来，但「i < i_now 且无行 ⇒ 已发布」的窗口打开，推理）；或证明恢复写行、切换、回退的第一次发布不写任何单元 |
| 撞号的害处 | 证明第二次尝试一定逐字节重写孤儿、落在同一个槽上（那样谓词翻面没有可观测的后果）；实例切换重发的是没返回的调用者的数据，反推腿推不出这一条 |
| P6「恰 2 个」全 | 设备的撕裂粒度小于 512 字节（`crash.rs` 第 23 行 `SECTOR_BYTES = 512` 是假设；D22（单元原子性怎么合成） 已定三的方法论标注把撕裂形态的推理记成本机证伪不了，附录整行抄）：那时一次超级块写能撕出校验和不过的槽、盘退回另一个槽，状态要重枚举 |
| J4 的敏感取样点 | D22 已定项 16 的「每写一次 +1」被定成全池一个号（实现今天的写法）：模型 B 全池最大 + 1 那三行都是 0，敏感取样点消失，C322 原形那条自证在三条臂下都造不出 |

## 附录：正文引到、背景材料附录没抄或需要单独对照的 kb 原文（整行抄）

命令一律是 `sed -n 'Np' 文件`。正文里「」内的 kb 引文都是这些整行里的原样子串。

**`.claude/kb/decisions/18-块里携带什么信息.md:879`**（在第 775 行「### 已定项 11」小节里）

```markdown
四元组 (出生树 0, 类型 4, 容器号 = 片序号从 0 起, 出生代 0)；出生树 0 与 D5（快照 / 空间记账机制） 给「全池 / 无归属」行的约定同一个数；不属于任何树、不进容器索引，由根记录直接持有物理指针（D22（单元原子性怎么合成） 已定项 7）。一片 ⌊(32768 − 136) / 88⌋ = 370 条记录（含链指针，数据行 369；136 是含 nonce / MAC / 算法类型预留位的码 3 头，D18（块里携带什么信息） 已定项 14 / 已定项 16）；mkfs 写出一片空表，记录数 1（唯一一条是「无下一片」的链指针记录）。**行怎么写**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2）：按实例代号唯一、后写覆盖，只在恢复与回退时写、与第一个新根同一次发布——任何非干净结束的实例在下次恢复写行：给 [所选根的实例, 新实例) 每个实例写 (i, **所选根的 checkpoint_txg**, **属于实例 i 的、被这次重放施加的最大事务号**)——jsn = 实例代号 32 位 + 计数器 48 位（D23（journal 的角色与格式） 已定项 9），重放按 jsn 严格连续、断号即止 ⇒ 任何一次重放都跨不过实例边界 ⇒ **同一次恢复 / 切换写出的那批行里，只有所选根那个实例的 W 可能非 0，其余恒 0**（限定在「同一次写出的那批」上：后来的恢复只给 [新的所选根实例, 新实例) 写行、不回头改旧行，所以表上允许有多行 W ≠ 0，它们来自不同次的恢复或切换）（E104（扫描重建的现行版本判定） `recovery_own_txns_global_w` 世界：把一个全局量写给范围里的每一个实例时，恢复实例自己的孤儿复活 3）——恢复恒选最新可读且自证通过的根，它可以不是最新发布过的根（槽坏了就退一格，之后的记录接不上就一条都不施加、W = 0）；一条都没施加的写 (i, 所选根 txg, 0)；回退写 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)，回退行在 kind 0 记录的 flags 字节里置 bit0 标「回退」、**不许被后来的恢复覆盖**（落在 R_old 上的那次恢复会给 [所选根的实例, 新实例) 每个实例写行，正好盖到它；C124（回退行与重放下界没有会红的检查））；实例切换（D23（journal 的角色与格式） 已定项 14 的注）同崩溃行；每次写行 COW 重写整条链。W 是精确的前缀：事务号在实例内单调、记录按事务号顺序追加（第一版串行提交下恒成立），一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加（D23（journal 的角色与格式） 已定项 7 的注）。**已发布谓词（全局）**：i_now = 挂载根的实例，单元写序 (i, n)、诞生代号 b——i < i_now 且无行 ⇒ 已发布；有行 (i, T_pub, W)：码 1 ⇒ b ≤ T_pub ∨ n ≤ W（在所选根里，或被重放施加；E104（扫描重建的现行版本判定） `lost_root` 世界：T_pub 按字面取在飞 txg − 1 时槽坏掉的那个 checkpoint 整个判成已发布，复活 2 + 错 1），码 2 / 3 ⇒ b ≤ T_pub（所选根之后的固定点单元一律未发布，恢复实例重写它们；那 6 字节事务号在码 2 / 3 上不参与任何判定，I-1.8（归并后版本全序） 对码 3 的全序键是 (诞生代号, 实例代号)）；i == i_now ⇒ b ≤ 挂载根的 txg；i > i_now ⇒ 判损坏。「无行 ⇒ 已发布」只对同一时间线成立。**回收**：行可删当且仅当整轮清扫对**池中每一个落点**都得出了判定（读成功并按行判、已抹头、已被覆盖）且其中没有该实例的未发布单元、那次清扫里没有任何一次读失败（读不到不等于不存在，扇区维），且全部设备在线（设备维），且根环里没有该实例发布的根。量词取「池中每一个」不取「该实例写过的每一个」：已抹头与已被覆盖的落点说不出它原来属于谁，按后者写的话要枚举的集合与要销毁的信息是同一份（被抛弃的根留在环里时，它的行是回退候选判据的输入，E104（扫描重建的现行版本判定）：不看根环则被抛弃的根整批回到候选集）；zoned 第一版不回收、链长无上界（C121（zoned 上实例表链长无上界））。**表不可读时**：只读挂载；扫描重建停在级 1——表没了就一条行都没有，全部旧实例的全部单元落进「无行 ⇒ 已发布」，被回退抛弃的整段时间线也在内，所以不是记歧义。实例表单元自己不走 I-1.2（块头写序已发布） 的谓词：它的现行版本由根记录持有的物理指针唯一确定，指针指不到或校验不过就是表不可读。**作废一个 checkpoint 必须换实例代号**：当前实例没有行，谓词对它只能问「诞生代号 ≤ 挂载根 txg」，重发同一个号一发布，诞生代号等于它的孤儿全部判已发布——这是失败表能封闭的依据（D23（journal 的角色与格式） 已定项 14 的注）。**实例代号**住超级块（E100（超级块的三段几何） 段四那 4 字节）。可写挂载的顺序：先判这次能不能可写（可写设备数够 w 的下限 ∧ **独占打开池中过半的设备**——任意两个过半集合相交，每设备的独占打开才成为池级互斥；这条与它的代价（4 盘池只剩 2 块可写时只能只读）是 D2（RAID 条带策略） 已定项 13 定的，不是本项 ∧ 实例表可读 ∧ 实例切换的预留拿得到），再取新代号 = max(**这次挂载独占打开成功的那个集合**里各超级块的代号, 根环里全部根记录的实例代号) + 1 并写进**那个集合**里的每一份超级块、**全或无**（写超级块与数据单元写同一个取向：一次 I/O 错要重试到 T_retry 用尽才算失败；用尽后全或无判失败时**先把已经写出的那几份回卷成旧代号**，回卷不成才只读挂载——回卷期间盘上出现的「同一集合内代号不等」是 I-7.7（超级块实例代号不低于根环） 的一条显式例外，下一次挂载按 max + 1 修复；取号的写入集合与 max 的取值集合取同一个——写「全部可见」而只独占一部分时，集合外那些盘写不写得进去不由这个挂载说了算，两个挂载能互相逼成只读。**过半是准入门槛，不是写入范围**：写入集合取独占打开成功的全部设备，只写过半会把集合外的健康盘落下、它们的代号从此是旧的，与真正错过取号的盘分不开。**「可见」= 独占打开成功且超级块读得通**；I-7.7（超级块实例代号不低于根环） 的相等限定到同一个集合），之后才动任何单元；只读挂载不取号；从 1 起、0 无效。**前提**：同一时刻只有一个可写挂载——由过半独占打开保证；多主机共享存储上两个可写挂载会算出同一个代号、谓词分不出它们，随 D9（加密） 已定项 8 那一类宣布不防。取号那一刻两路见证不完整可见时不额外拦截，与整卷回滚同格；I-7.6（根环区域落在互不相同的盘上） 只在 devs ≥ R 时让根环成为独立见证，第一版 (R = 3, devs = 2) 不成立。错过取号的盘分两类：**打开过又掉了、且缺号期间池里确实发布过**的，回归时整盘作废、只读到重同步完成；**从未被这次挂载独占打开的**，或缺号期间池里一次发布都没有的，不作废——它没有分叉的物理可能（第八轮反推腿 3.1 / 7.2）；第一版 2 盘、掉一块只能只读挂载 ⇒ 分叉盘回归不可达；D2（RAID 条带策略） 已定项 3 / 4 允许在线加盘 ⇒ 第三块盘一加就可达，C120（分叉盘回归的判定与重同步）在允许加第三块盘之前到期，与过半规则同时。
```

**`.claude/kb/decisions/23-journal的角色与格式.md:695`**

```markdown
16. **mkfs 写出的实例代号是几、第一次可写挂载取几（2026-09-13，用户定案）：mkfs 写 0，0 = 「mkfs、尚无实例」，不是有效实例；第一次可写挂载按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 取 max(超级块, 根环) + 1 = 1，并写进每一份超级块（一次超级块槽写，世代号 +1）之后才动单元。mkfs 写出的单元写序 (0, 0)、第 0 代根实例代号 0；D18（块里携带什么信息） 已定项 11 那句「从 1 起、0 无效」读成「实例从 1 起，0 保留给 mkfs」。** 正文见 D23（journal 的角色与格式）「已定项 16」。 **状态：已定。**
```

**`.claude/rules/fs-design.md:23`**

```markdown
| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 | 不是慢，是**在被问到的那一刻没有答案**。准入控制要「进门前先算最坏情况」，而「释放空间这个操作本身不需要申请空间」也压在同一个数上 |
```

**`.claude/kb/decisions/22-单元原子性怎么合成.md:87`**（已定三的方法论标注）

```markdown
⚠️ **方法论标注**：本文件关于撕裂形态的推理**在本机无法证伪**——
```

**`.claude/kb/invariants.md:53`**

```markdown
| I-7.7 | 超级块实例代号不低于根环 | 可写挂载独占打开成功的那个集合里（它至少是过半，过半是准入门槛不是写入范围），各超级块的实例代号相等，且 ≥ 根环里任一根记录的实例代号（必要条件，不充分；取号写超级块全或无，C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的取号规则）。只在 devs ≥ R 时根环才是超级块之外的独立见证，第一版 R = 3 > devs = 2 时两路见证的失败域是同一对盘 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **2026-09-14 层 0 实测**：取号那两个超级块写只落了一块盘的 2 个崩溃状态里两份实例代号不等、判红——C322（取号那一步的屏障怎么放没有条款） 还开着，层 0 的测试把这个数钉成 2 |
```

**`.claude/kb/first-txn-layout.md:378`、`:382`、`:389`、`:390`、`:392`**

```markdown
## 八、根槽写路径的段序列登记表（C316（提交步骤的登记位有四处且互不相同） 的登记位，2026-09-13 立）
一条线的提交协议由它全部根槽写路径的录制流段序列定（D13（验证路线） 已定项 4 切段：屏障与 FUA 写切段，段内任意整写子集），等价类按这张表的同构分：段边界的位置相同、每段里出现的步骤种类**集合**相同（D17（实现分层与第三方管道） 已定项 2）；一步重复几次（设备数、单元数）是参数不进判据。别处引提交步骤一律链到这一节，不另抄。步骤种类只有五种：写单元（含码 3 容器、实例表单元）、写 journal 记录、根槽 FUA 写、超级块槽原地覆写、屏障。表里的段序列逐字来自 E142（第一个事务的干跑） 产物的 `name=segments` 五行（`path=mkfs / instance_acquisition / warm_up / transaction / post_mkfs_stream`；取号那一行不是根槽写路径，登记在这里是因为整条流按录制流切、它的两个写落在流的开头一段），单测 `registered_segment_sequences_match_every_recorded_path` 钉住它们；表与产物之间的逐字比对由门禁 52 号做（C316（提交步骤的登记位有四处且互不相同） 2026-09-13 已还清）。
| 第一次可写挂载取号（实例代号 0 → 1，不是根槽写路径） | [a1 超级块槽 × 2 盘，世代号 2] ⇒ `2`，2 次操作、4 个崩溃状态，种类 `[superblock_slot×2]`；不另加屏障，靠暖机第一次空发布开头那道屏障收段（C322（取号那一步的屏障怎么放没有条款）） | 一（a1）；D23（journal 的角色与格式） 已定项 16；产物 `name=segments path=instance_acquisition` | 在（第六次跑起，整条流开头那一段） |
| 空发布（暖机，第一版 2 次） | 屏障 [w1 空记录 × 2 盘] 屏障 [w2 根 FUA] [w3 超级块槽 × 2 盘]（第二次同型，w4–w6）⇒ 两次合起来 `2+1+2+2+1+2`，种类 `[journal_record×2,barrier×2]\|[root_record_fua]\|[superblock_slot×2,barrier]\|[journal_record×2,barrier]\|[root_record_fua]\|[superblock_slot×2]` | 零；D16（发布语义） 已定项 8；产物 `name=segments path=warm_up` | 在（第四次跑起） |
| 实例切换 / 管理员回退 | **预想**（没写成字节）：[实例表单元 + COW 单元] 屏障 [记录] 屏障 [根 FUA] [超级块槽]，**之后接新实例的暖机**（D16（发布语义） 已定项 8：新实例的根覆盖两块盘之前连推空发布，与第一次挂载同型）——与普通发布 + 空发布同型，多的是内容不是步骤 | D23（journal 的角色与格式） 已定项 14；D16（发布语义） 已定项 8 | 不在 |
```

**`.claude/kb/checks-owed.md:301`**

```markdown
| C322 | 取号那一步的屏障怎么放没有条款 | D23（journal 的角色与格式） 已定项 16 定第一次可写挂载先把实例代号写进每一份超级块、之后才动单元，没说这两个写与之后的暖机之间要不要屏障；E142（第一个事务的干跑） 第六次跑按最少屏障取（不另加，靠暖机第一次空发布开头那道屏障收段），取号那一段因此不是根槽写路径、[first-txn-layout.md](first-txn-layout.md) 八那一节的标题罩不住它 | 崩溃点重放里「取号写了一盘、另一盘没到、暖机的记录已持久」这个状态要有一条判定（实例代号不一致的两份超级块怎么择）；判别力自证：把择超级块改成只看世代号不比实例代号，检查必须由绿转红 | D23（journal 的角色与格式） 已定项 16 补一句屏障放哪；八那一节要不要改名。⚠️ **2026-09-14 用户 18 问定案没有碰这一格**（那一轮的 第 8 问 只定了根环区域归属 0 / 1 / 0 与暖机次数 2，取号那两个写与暖机之间要不要屏障一个字没说）⇒ 这一笔原样开着 ⚠️ 2026-09-14 池级 checker 在层 0 里判出这一格：取号只落了一块盘的 2 个崩溃状态里 I-7.7（超级块实例代号不低于根环） 判红，层 0 的测试把这个数钉成 2（还清时要改那条断言） | 2026-09-13 E142（第一个事务的干跑） 第六次跑 G21 |
```
