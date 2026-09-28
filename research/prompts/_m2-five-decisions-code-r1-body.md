# 五条定案的实现：代码轮第一轮正文（2026-09-28）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 T1 T2 T3 T4 -->

## 一、题面

用户 2026-09-28 定了五条（原话在 `records/2026-09-28-收口表对齐与三笔欠账复查.md`「用户定案」两节），主 agent 同日照定案改了 `crates/`、补了用例与变异行、写回了 kb。用户同日要求「测试和实现都要走」：这一轮既攻实现（代码做的是不是条款说的、哪一格会错），也攻测试（用例测的是不是条款要的那件事、放不放得过错的实现、造的盘面合不合法、改钉的旧用例丢没丢覆盖）：

| # | 定案 | 条款落点 | 判决 |
|---|---|---|---|
| 1 | C331（择根倒挂压过已确认的写） 取甲：可写挂载判 N-配置 的见证读里有一个系统配置槽读不出或自证不过就判不出、按真；取号那一刻某块盘缺一槽就在第一个取号写之前拒 | D23（journal 的角色与格式） 已定项 14 N-配置续那一段 | `research/prompts/unreadable-at-mount-r2-main-verification.md` L2 |
| 2 | C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算：被抛弃根的账读不出时从中央映射现算引用的落点、内存隔离、计数往上报，映射也现算不成就重读一次再拒可写 | D23（journal 的角色与格式） 已定项 14 影子账那一句 | 同一判决 L3 |
| 3 | 收口表第 ② 行：被抛弃根带的 F 算进 F_生效（实现与 checker 各一份） | D16（发布语义） 已定项 1「生效」那一段 | `research/prompts/abandoned-floor-r1-main-verification.md` M2 |
| 4 | 收口表第 43 行取丁-defer：I-3.1 的遍历并上 F_生效 之下、没被抛弃的根引用着、候选集没走到、最新根账里是已释放且释放代高于 F_生效 的槽 | I-3.1（已分配统计对得上） checker 读法 | 同一判决 M1 |
| 5 | C379（记账的全空聚簇段数与分配器能开的段两个口径） 取 B：两个量分开登记 | D3（空间分配） 已定项 10 ① | 同一判决 M3（只动 kb，没有代码） |

## 二、实现今天的样子（主 agent 的观测，2026-09-28）

腿读代码一律读冻结副本 `/tmp/claude-1000/m2-five-decisions-code-r1/tree/crates/`（sha256 在 `research/prompts/m2-five-decisions-code-r1-snapshot/`）。这一轮改动的项（工作区里别的会话也有没提交的改动，这里只列这一轮的）：

- `crates/singlefs-core/src/mount.rs`：`WitnessedCounterComparison` 加成员 `SomeSystemConfigurationSlotUnreadOrUnverified`；`SelectedVersionAgainstTheWitness::witnesses_a_publish_newer_than_the_selected_version` 那一支判真；`newer_publish_witness`（逐盘读两槽，缺一槽判那一支）；`StillUnreadableAfterOneReread` 加成员 `AccountOfAnAbandonedRoot`；`isolate_slots_referenced_only_by_abandoned_roots`（改成交 `Result`，读不出先从映射现算、再重读、再拒）；新函数 `placements_referenced_by_root_computed_from_the_mapping`；调用处 `rebuilt_allocator`、`raise_the_floor_through`（抬 F 时拒就把分配器换回原样）。
- `crates/singlefs-core/src/transaction.rs`：`self_verified_system_configurations_of_every_device`（缺一槽就拒）；`InstanceAcquisitionFailed` 与 `ExpectedInstanceAcquisitionFailed` 的成员改名 `DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`。
- `crates/singlefs-core/src/recovery.rs`：`effective_rollback_floor_of_the_roots_read`（被抛弃根带的 F 并进来）。
- `crates/singlefs-checker/src/walk.rs`：`check_pool_image` 里 F_生效 并上被抛弃根的 F、I-3.1 的遍历并上 defer 槽；新函数 `deferred_slots_referenced_only_below_the_floor_per_device`。
- `crates/singlefs-harness/src/history.rs`、`model_comparison.rs`：新成员的匹配臂。
- `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` 孤记录那一条：系统配置槽 0 不再写坏一个字节，改成退回 mkfs 那一份（甲 之下写坏就是撕裂轮换）。
- 用例：新文件 `a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs`、`an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs`、`raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`、`an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs`；`tests/common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`（不再清零见证槽，写回轮换之前那一代）与新加的 `abandon_the_third_version_by_a_crash_before_its_rotation`；改钉的用例在 `acquisition_writes_the_witnessed_journal_tail_…`、`rollback_by_a_forward_publish.rs`、`corrupt_on_disk_content_is_refused_instead_of_panicking.rs`、`entries_after_mount_refuse_swapped_or_behind_devices.rs`、`fsync_drop_and_devices_without_the_selected_version.rs`。
- `crates/mutations.tsv`：新加 6 行（名以「C331 甲」「C393 (b)-从映射现算」「收口表第 43 行丁-defer」「收口表第 ② 行算进」起头），`research/scripts/prove-red.sh` 6 条都抓到；改锚点 8 行。
- 没做：随机历史的已知红清单（`crates/singlefs-harness/src/history.rs` 的 `KNOWN_RED_FORMS`）里收口表第 43 行那一条还在，没删；C331 要的多次挂载崩溃点重放里的注入没做（C331 仍欠）。

## 三、格

### Z1　条款对实现、条款对测试（云端正推）
逐条核五条定案的条款原文（kb 文件自己的行号）与上面列的每一项：代码做的是不是条款说的，有没有条款说了而代码没做、代码做了而条款没说的；新写的四份用例与改钉的五份用例测的是不是条款要的那件事、断言钉的是条款里的哪一句（`.claude/singlefs-ai-sop/rules/engineering-philosophy.md`「人的注意力该花在哪」第 1 条），条款里有而没有用例钉的逐句列出。另答一问：甲 之下逐盘核里 `SelectedVersionLackingOnDevice::NoSelfVerifiedSystemConfiguration` 那一支（`crates/singlefs-core/src/mount.rs`）经可写挂载走不到了（见证读缺一槽先拒，`fsync_drop_and_devices_without_the_selected_version.rs` 的空盘两格都改钉成见证拒），它该留着当防御、还是删掉，条款怎么说。

### Z2　哪一格会错（云端攻方）
在冻结副本的拷贝上造反例，至少攻这几处：① `newer_publish_witness` 的「缺一槽」在 mkfs 之后、取号写之后、撕裂轮换之后、只剩一块盘可见时各判什么，有没有该可写而被拒成永远只读之外的新形；② `placements_referenced_by_root_computed_from_the_mapping` 现算出的落点集与账里的逐个相同吗（多层映射树、打包单元、实例表多片、树表 0 条那一版）；多了会多隔离、少了会复用；③ `isolate_slots_referenced_only_by_abandoned_roots` 改成交 `Result` 之后，抬 F 那条路上被拒时分配器换回原样的次序对不对、有没有写过盘；④ `deferred_slots_referenced_only_below_the_floor_per_device` 会不会把「该回收而没回收」的错盖住（造一个抬 F 该回收而实现没回收的变异，看 I-3.1 还红不红）；⑤ 被抛弃根带的 F 并进 F_生效 之后，有没有合法历史上回退或抬 F 被误拒。只跑 harness 档。

### Z4　测试（云端攻方，与 Z2 同一条腿、另成一节报告）
① 对每份新用例与改钉的用例，写至少一种「看起来对、其实错」的实现（不是变异表里已有的那 6 行与改锚点的 8 行），看用例放不放得过：例如见证读只查盘 0、只查世代号大的那一槽；从映射现算少算打包单元或实例表多片；defer 槽不看释放代、或把被抛弃根引用的也并进来；F_生效 只在 checker 一侧并。放得过的，写出最短的那一种。
② 造盘面的三处改动合不合法：`tests/common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`（把见证槽写回成世代号 g − 2、内容拿 g − 1 那一槽的）、`acquisition_writes_the_witnessed_journal_tail_…` 的「落后一次轮换」（世代 3 拿世代 4 的内容改号）、`writable_mount_of_a_formatted_pool.rs` 的孤记录那一格（槽 0 退回 mkfs 那一份）：各自是不是一段真崩溃留得下的盘面；不是的，它钉住的行为会不会因此是假的。`an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs` 直接改根记录的 F，判它测到的是不是条款那一句。
③ 改钉的五份旧用例（`rollback_by_a_forward_publish.rs` 撕裂树表那一条、`corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 指针在单元区之下那一条、`entries_after_mount_refuse_swapped_or_behind_devices.rs` 只剩一槽那一条、`fsync_drop_and_devices_without_the_selected_version.rs` 的见证拒因与空盘两格、`rollback_by_a_forward_publish.rs` 分配记录越出单元区那一条、`writable_mount_of_a_formatted_pool.rs` 孤记录那一条、`acquisition_writes_…` 丢最新槽那两条）：改钉前它们守住的那件事，改钉之后还有没有用例守着；没有的逐条列出。
④ 变异表：新 6 行各自红的是不是点名的那条用例、替换文是不是真把行为退回了旧的一种（不是另一种错）。

### Z3　算术与判定表（本地攻方）
按写死的事实表逐格推：行是故障形态（撕裂轮换、一槽持续读错、取号那一刻缺一槽、被抛弃根树表读不出、分配记录树读不出、两者都读不出且映射为空），列是 {可写挂载的结局, 取号的结局, 影子账隔离几槽, abandoned_roots_unreadable}；每格写依据的事实表行号（代码行号以冻结副本为准）。与云端攻方不重叠：本地腿不跑代码。

## 四、分工

| 腿 | 格 | 立场 | 理由 |
|---|---|---|---|
| 云端正推（Sonnet） | Z1 | 条款对实现 | 代码轮第一轮，没有前一轮代码判决可复核 |
| 云端攻方（Opus） | Z2、Z4 | 在副本上真跑：实现的反例与测试放得过的错实现 | 要真跑；两节分开报 |
| 本地攻方 | Z3 | 按事实表逐格推 | 与云端攻方分开：云端跑、本地推 |

本地腿：攻（代码轮第一轮，不派辩方；本地攻方只按事实表推 Z3 判定表，与云端攻方真跑的 Z2 攻击面不重叠）

## 五、跑前写死的判据

- Z2 造出一条合法历史上丢写、复用被抛弃根引用的单元、或 checker 在合法状态上红 ⇒ 打中，改法并进下一批，这一轮判决写明。
- Z2 ④ 那个「该回收而没回收」的变异下 I-3.1 不红 ⇒ 记成丁-defer 的已知盲区，登记欠账，不推翻用户定案。
- Z1 判出条款与代码对不上 ⇒ 以条款为准改代码；条款自己没写清的，交用户。
- Z4 ① 写出一种错实现、新用例与改钉用例全绿 ⇒ 打中，补用例（或收紧断言）并进下一批。
- Z4 ② 判出某处盘面不是真崩溃留得下的、而它钉住的行为只在那种盘面上成立 ⇒ 打中，那条用例改造法。
- Z4 ③ 列出改钉之后没人守的旧行为 ⇒ 逐条判那件事在新定案下还该不该守，该守的补用例。
- Z3 与 Z2 同一格不同的，主 agent 现查坐实哪一边。

交付：腿的报告写 `research/prompts/m2-five-decisions-code-r1-<腿名>-output.md`，分段落盘；攻方的副本改法与用例放 `research/prompts/m2-five-decisions-code-r1-opus-model/`。引 kb 条款写 kb 文件自己的行号。方案按 `crates/` 今天的实现来谈。
