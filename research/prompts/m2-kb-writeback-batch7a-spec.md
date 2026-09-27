文件：.claude/kb/invariants.md
旧串：
````
| I-7.12 | 系统配置 F 不低于同盘根上的 F | 每块盘分开判：这块盘两槽里自证过（校验和过且 fsid 与本池相同，与取号同一读法，D18（块里携带什么信息） 已定项 11）的系统配置槽中回退下界 F（D22（单元原子性怎么合成） 已定项 9 的字段）的最大值，不低于这块盘根槽里每一条自证过的根带的 F（D22（单元原子性怎么合成） 已定项 7 的字段）。抬 F 那一串先把新 F 写进每块盘的系统配置、过一道屏障，才发第一条带新 F 的根（D16（发布语义） 已定项 1「抬 F 那一串」），所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里。某块盘两槽都自证不过时，那块盘不判；一块盘都判不了时整条报「不适用」。判别力：把「先写系统配置」挪到第一条根之后，崩在两者之间的状态必须红 | 已实现（2026-09-26，池级 checker `walk::check_pool_image` 的 `judge_system_configuration_floor_against_the_roots_on_each_device`：每块有自证过的系统配置槽的盘判一格，那块盘上一条根都没有时那一格成立；两槽都自证不过的盘不判，一块盘都判不了时整条报「不适用」。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant`：步 5 那段历史上抬到上限 11 的镜像一条都不红、I-7.12（系统配置 F 不低于同盘根上的 F） 真被评估过且成立，盘 1 两槽的 F 改回 0 只红它、红在盘 1 txg 16 那条带 11 的根上；变异「不比系统配置里的 F（每块盘恒成立）」「checker 不读系统配置里的 F（读成 0）」（实二）。条款仍是主 agent 2026-09-26 按 SysPre 的写序推的，没三方；层 0 每个崩溃状态都跑池级 checker，按判法每个状态都评估得到它（推的，层 0 没跑） |
````
新串：
````
| I-7.12 | 系统配置 F 不低于同盘根上的 F | 每块盘分开判：这块盘两槽里自证过（校验和过且 fsid 与本池相同，与取号同一读法，D18（块里携带什么信息） 已定项 11）的系统配置槽中回退下界 F（D22（单元原子性怎么合成） 已定项 9 的字段）的最大值，不低于这块盘根槽里每一条自证过的根带的 F（D22（单元原子性怎么合成） 已定项 7 的字段）。抬 F 那一串先把新 F 写进每块盘的系统配置、过一道屏障，才发第一条带新 F 的根（D16（发布语义） 已定项 1「抬 F 那一串」），所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里。某块盘两槽都自证不过时，那块盘不判；一块盘都判不了时整条报「不适用」。判别力：把「先写系统配置」挪到第一条根之后，崩在两者之间的状态必须红 | 已实现（2026-09-26，池级 checker `walk::check_pool_image` 的 `judge_system_configuration_floor_against_the_roots_on_each_device`：每块有自证过的系统配置槽的盘判一格，那块盘上一条根都没有时那一格成立；两槽都自证不过的盘不判，一块盘都判不了时整条报「不适用」。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant`：步 5 那段历史上抬到上限 11 的镜像一条都不红、I-7.12（系统配置 F 不低于同盘根上的 F） 真被评估过且成立，盘 1 两槽的 F 改回 0 只红它、红在盘 1 txg 16 那条带 11 的根上；变异「不比系统配置里的 F（每块盘恒成立）」「checker 不读系统配置里的 F（读成 0）」（实二）。条款仍是主 agent 2026-09-26 按 SysPre 的写序推的，没三方；层 0 每个崩溃状态都跑池级 checker，按判法每个状态都评估得到它（推的，层 0 没跑） |
| I-7.13 | 系统配置池级字段在读者收的范围里 | 任一自证过（magic 与整槽校验和对，`check_system_configuration_slot`）、incompat 位认得的系统配置槽：格式版本 = 1（`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`）、加密类型 = 0（第一版恒关，`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF`）、固定结构槽距 ≥ 4096 字节（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）且槽 1 整槽落在根环基址之前、`physical_block_size` ∈ [457（`ROOT_RECORD_BYTES`）, 槽距]、journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1 且环末端不越过单元区起始槽号（`geometry_of` 的 `fixed_structure_slot_spacing_lies_in_the_format_range` / `physical_block_size_fits_a_root_slot` / `journal_ring_bytes_lie_in_the_supported_range`）。任一盘任一槽不满足其中一项即判红：checker 报违例、不作保，且该池其余不变量一律报「不适用」，与实现整池拒绝挂载一致（用户 2026-09-27 定系统配置越界整池拒）。R（区域数）与 S（每区槽数）越界不在这一条里，仍归「这一槽不可择」 | 已实现（2026-09-27，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`（:46）登记这个编号，`IMPLEMENTED_INVARIANTS`（:54）由此变 47 条；判定 `judge_system_configuration_values_the_reader_accepts`（image.rs:454）在 `walk::check_pool_image`（walk.rs:5573 起）里调用，任一盘任一槽带越界值时该池其余不变量整批报「不适用」并提前返回（walk.rs:5578）；坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 里现在还没有改坏触发它的一份，`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 因此判红，补不补交主 agent 定） |
````
依据：records/2026-09-24-里程碑二收尾调度.md「实审 A3-checker-2 交回并打上；I-7.13 立号（2026-09-27 JST 17:0x）」那一行；research/prompts/m2-rev-a3-checker-2-implementer-report.md 第四节第 1 点；crates/singlefs-checker/src/image.rs、walk.rs 现查

文件：.claude/kb/invariants.md
旧串：
````
池级 checker（`walk::check_pool_image`）判 46 条（数它的命令：`grep -c '^| I-.*已实现' .claude/kb/invariants.md`，与 `crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 长度相等；第一版 23 条，2026-09-16 里程碑「第二个事务」步 3 加 I-3.8（实例表行唯一且低于挂载根），2026-09-17 步 6 加 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），2026-09-18 加 I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变） 与 I-5.4（分配记录罩住的槽互不相交），2026-09-21 加 I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号），2026-09-22 加 I-1.10（码 2 条目宽等于字段表宽）、I-9.6（水位大于两处最大号）、I-9.12（分隔 key 落在孩子区间之外）、I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块），2026-09-23 加 I-8.8（前缀里的事务不被切开） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号），2026-09-24 加 I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读） 与 I-8.9（一次发布的记录序号连续且只有末条带标志），2026-09-25 加过 I-7.10（已停用·回退见证表删了） 与 I-7.11（已停用·见证择根删了）、2026-09-26 随回退见证删掉退役，同日加 I-7.12（系统配置 F 不低于同盘根上的 F），2026-09-27 加 I-1.11（映射 key 与单元头相符）），状态列写「已实现」的就是这 46 条：每条配一份改坏了正好触发它的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`），层 0 的每个崩溃状态都跑它。
````
新串：
````
池级 checker（`walk::check_pool_image`）判 47 条（数它的命令：`grep -c '^| I-.*已实现' .claude/kb/invariants.md`，与 `crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 长度相等；第一版 23 条，2026-09-16 里程碑「第二个事务」步 3 加 I-3.8（实例表行唯一且低于挂载根），2026-09-17 步 6 加 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），2026-09-18 加 I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变） 与 I-5.4（分配记录罩住的槽互不相交），2026-09-21 加 I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号），2026-09-22 加 I-1.10（码 2 条目宽等于字段表宽）、I-9.6（水位大于两处最大号）、I-9.12（分隔 key 落在孩子区间之外）、I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块），2026-09-23 加 I-8.8（前缀里的事务不被切开） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号），2026-09-24 加 I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读） 与 I-8.9（一次发布的记录序号连续且只有末条带标志），2026-09-25 加过 I-7.10（已停用·回退见证表删了） 与 I-7.11（已停用·见证择根删了）、2026-09-26 随回退见证删掉退役，同日加 I-7.12（系统配置 F 不低于同盘根上的 F），2026-09-27 加 I-1.11（映射 key 与单元头相符），同日加 I-7.13（系统配置池级字段在读者收的范围里）），状态列写「已实现」的就是这 47 条：每条配一份改坏了正好触发它的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`），层 0 的每个崩溃状态都跑它。
````
依据：crates/singlefs-checker/src/image.rs:54 的 IMPLEMENTED_INVARIANTS 长度 47（现查）；research/prompts/m2-rev-a3-checker-2-implementer-report.md 结论一节

文件：.claude/kb/invariants.md
旧串：
````
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节与指针头部的 MAC 16 + nonce 12（D19（块指针的结构与宽度预算） 已定项 3）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；指针头部 MAC / nonce 那一半的池级判定在 A3-checker-2 里做，还没落地） |
````
新串：
````
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节与指针头部的 MAC 16 + nonce 12（D19（块指针的结构与宽度预算） 已定项 3）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；池级走读在 `walk::judge_unit_header`（`crates/singlefs-checker/src/walk.rs:649` 起）另判单元头偏移 4 的格式版本 = 1（walk.rs:695，不认得就判该结构损坏、不按今天的字段表往下解）与 29 字节预留位全 0（walk.rs:730）；走读跟随的每条指针在 `judge_pointer_mac_and_nonce_are_zero`（`crates/singlefs-checker/src/image.rs:619`）判头部 MAC 16 + nonce 12 全 0，四个入口分别是码 2 节点指针（walk.rs:790）、inode 内部条目子指针（walk.rs:1440）、实例表链指针（walk.rs:1766）、extent 数据指针（walk.rs:1916），只判不断、指针指向的单元照样走下去，2026-09-27 实审 A3-checker-2；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`） |
````
依据：crates/singlefs-checker/src/walk.rs:649,695,730,790,1440,1766,1916 与 image.rs:619 现查；research/prompts/m2-rev-a3-checker-2-implementer-report.md 第三节第 1、2 行

文件：.claude/kb/invariants.md
旧串：
````
| I-1.10 | 码 2 条目宽等于字段表宽 | 任一码 2 索引节点，头里自述的**条目宽**（D18（块里携带什么信息） 已定项 7 的自描述三段之一：key 宽 1 + 条目数 2 + 条目宽 2，偏移表在 D18（块里携带什么信息） 已定项 18、载荷布局在 D8（核心索引结构） 已定项 11）**等于**它那棵树登记的条目字段表宽度：extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108（key 22 + 子指针 86）。条目宽是**盘上读来的值**，`index_node_view` 只判了它 ≥ key 宽（extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」——不判就按字段表的固定偏移切，切片当场越界（增补 3 第 6 件 panic 面普查第二族，C476（盘上内容可控时走得到的 panic 有 21 处） R1 / R3 / R4 / R13）。条目宽为 0 时条目数必须为 0（`声明长度 = 条目数 × 条目宽` 在宽 0 时恒成立，挡不住它），否则判该节点损坏（2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做）。判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：**先于**按字段表解条目。⚠️ **中央映射树不判**：它的条目宽由 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树） 定，那一条还开着；**树表条目（200 字节）两边都没写**，而走读照样按固定偏移切树表节点的条目——要不要把射程扩到它，账在 C476（盘上内容可控时走得到的 panic 有 21 处） | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::walk_tree_table_entry`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；条目宽为 0 时条目数非 0 那一条池级判定在 A3-checker-2 里做，还没落地） |
````
新串：
````
| I-1.10 | 码 2 条目宽等于字段表宽 | 任一码 2 索引节点，头里自述的**条目宽**（D18（块里携带什么信息） 已定项 7 的自描述三段之一：key 宽 1 + 条目数 2 + 条目宽 2，偏移表在 D18（块里携带什么信息） 已定项 18、载荷布局在 D8（核心索引结构） 已定项 11）**等于**它那棵树登记的条目字段表宽度：extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108（key 22 + 子指针 86）。条目宽是**盘上读来的值**，`index_node_view` 只判了它 ≥ key 宽（extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」——不判就按字段表的固定偏移切，切片当场越界（增补 3 第 6 件 panic 面普查第二族，C476（盘上内容可控时走得到的 panic 有 21 处） R1 / R3 / R4 / R13）。条目宽为 0 时条目数必须为 0（`声明长度 = 条目数 × 条目宽` 在宽 0 时恒成立，挡不住它），否则判该节点损坏（2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做）。判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：**先于**按字段表解条目。⚠️ **中央映射树不判**：它的条目宽由 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树） 定，那一条还开着；**树表条目（200 字节）两边都没写**，而走读照样按固定偏移切树表节点的条目——要不要把射程扩到它，账在 C476（盘上内容可控时走得到的 panic 有 21 处） | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::walk_tree_table_entry`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；条目宽为 0 时条目数非 0 那一条 2026-09-27 实审 A3-checker-2 落地：`index_node_view_judging_a_zero_entry_width`（`crates/singlefs-checker/src/walk.rs:747`）在解码失败理由是 `Verdict::EntryWidthZeroWithEntries`、且这棵树登记了条目字段表（`EntryFieldTableRegistration::Registered`）时报违例；树表条目与中央映射树不在射程内，只记走读失败；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`） |
````
依据：crates/singlefs-checker/src/walk.rs:747 现查；research/prompts/m2-rev-a3-checker-2-implementer-report.md 第三节第 4 行

文件：.claude/kb/decisions/09-加密.md
旧串：
````
**射程**：预留的是已定项 12 的四样，加密关闭时全 0、所有卷照留、不挪作他用（D19（块指针的结构与宽度预算） 已定项 3）。`crates/` 里今天没有加密代码：写路径把这些位写 0（`PointerHead::write_to`、`RootRecord` 与 `JournalRecord` 的写出、`SystemConfiguration::to_slot`、单元头的 29 字节预留）；core 读路径今天判单元头 29 字节全 0（I-2.4（头校验和覆盖范围））、系统配置加密类型恒 0、两处读者（多层码 2 树内部条目、journal 新根段指针）的指针头部 MAC / nonce 恒 0，非 0 一律判损坏（实审 A3a）；这两处读者之外还没换上的指针读者、系统配置其余结构常量、journal 记录自己的算法类型 / nonce / MAC 预留，读路径仍跳过。池级 checker 判单元头 29 字节全 0（`check_unit`）；池级码 1 / 码 3 与指针头部在 A3-checker-2 里做。
````
新串：
````
**射程**：预留的是已定项 12 的四样，加密关闭时全 0、所有卷照留、不挪作他用（D19（块指针的结构与宽度预算） 已定项 3）。`crates/` 里今天没有加密代码：写路径把这些位写 0（`PointerHead::write_to`、`RootRecord` 与 `JournalRecord` 的写出、`SystemConfiguration::to_slot`、单元头的 29 字节预留）；core 读路径今天判单元头 29 字节全 0（I-2.4（头校验和覆盖范围））、系统配置加密类型恒 0、两处读者（多层码 2 树内部条目、journal 新根段指针）的指针头部 MAC / nonce 恒 0，非 0 一律判损坏（实审 A3a）；这两处读者之外还没换上的指针读者、系统配置其余结构常量、journal 记录自己的算法类型 / nonce / MAC 预留，读路径仍跳过。池级 checker 判三种码单元头 29 字节全 0与格式版本认得、及走读跟随的指针头部 MAC / nonce 全 0，均归 I-2.4（头校验和覆盖范围）；系统配置格式版本、加密类型越出这一版读者收的范围，归新立的 I-7.13（系统配置池级字段在读者收的范围里）。
````
依据：crates/singlefs-checker/src/walk.rs（judge_unit_header）、image.rs（judge_pointer_mac_and_nonce_are_zero、judge_system_configuration_values_the_reader_accepts）现查；research/prompts/m2-rev-a3-checker-2-implementer-report.md 第三节

文件：.claude/kb/decisions/22-单元原子性怎么合成.md
旧串：
````
- 自举头有写入者身份 20（实现标识 16 字节 ASCII 零补齐 + 版本 4 字节 u32）与校验和算法标识 1（0 无效、1 CRC-32C）；几何段有 mkfs 时的 `physical_block_size` 4 与扩展点声明值 N 4（宽度都按字节计）。读者遇到系统配置或单元头的格式版本不是 1，一律拒收那一槽 / 那一个单元，不按今天的字段表往下解（主 agent 2026-09-27 定，池级判定在 A3-checker-2 里做）。
````
新串：
````
- 自举头有写入者身份 20（实现标识 16 字节 ASCII 零补齐 + 版本 4 字节 u32）与校验和算法标识 1（0 无效、1 CRC-32C）；几何段有 mkfs 时的 `physical_block_size` 4 与扩展点声明值 N 4（宽度都按字节计）。读者遇到系统配置或单元头的格式版本不是 1，一律拒收那一槽 / 那一个单元，不按今天的字段表往下解（主 agent 2026-09-27 定；池级 checker 判定已落地：单元头格式版本归 I-2.4（头校验和覆盖范围），系统配置格式版本归 I-7.13（系统配置池级字段在读者收的范围里），实审 A3-checker-2）。
````
依据：crates/singlefs-checker/src/walk.rs:695（judge_unit_header 判格式版本）、image.rs:454（judge_system_configuration_values_the_reader_accepts）现查；research/prompts/m2-rev-a3-checker-2-implementer-report.md 第三节第 2 行

文件：.claude/kb/decisions/22-单元原子性怎么合成.md
旧串：
````
- **读者择系统配置时判根槽宽与固定结构槽距**：固定结构槽距 ∈ [4096, 1 MiB − 4096]（上界是槽 1 整槽落在根环基址之前，`root_ring::region_start(0)` 减系统配置槽宽 4096）；根槽宽（`physical_block_size`）∈ [根记录宽 457 字节（已定项 7）, 槽距]；越界整池拒绝挂载，与每区槽数 S 越界同一个处置（实审 A3a 落地：`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts`，`SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange` / `PhysicalBlockSizeOutsideTheRootSlotBounds`）。
````
新串：
````
- **读者择系统配置时判根槽宽与固定结构槽距**：固定结构槽距 ∈ [4096, 1 MiB − 4096]（上界是槽 1 整槽落在根环基址之前，`root_ring::region_start(0)` 减系统配置槽宽 4096）；根槽宽（`physical_block_size`）∈ [根记录宽 457 字节（已定项 7）, 槽距]；越界整池拒绝挂载，与每区槽数 S 越界同一个处置（实审 A3a 落地：`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts`，`SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange` / `PhysicalBlockSizeOutsideTheRootSlotBounds`）；池级 checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、该池其余不变量一律报不适用（实审 A3-checker-2 落地：`crates/singlefs-checker/src/image.rs` 的 `judge_system_configuration_values_the_reader_accepts`）。
````
依据：crates/singlefs-checker/src/image.rs:454、walk.rs:5573-5585 现查；用户 2026-09-27 JST 14:0x 定「整池拒」，records/2026-09-24-里程碑二收尾调度.md

文件：.claude/kb/decisions-history/2026-09.md
旧串：
````
## 历史版本

### 2026-09-27（其十五）
````
新串：
````
## 历史版本

### 2026-09-27（其十八）：D9（加密） 已定项 10、D22（单元原子性怎么合成） 已定项 2 / 已定项 9：I-7.13 立号，checker 补系统配置池级越界判定

> 快查·改前：D9（加密） 已定项 10 射程末句写「池级 checker 判单元头 29 字节全 0（`check_unit`）；池级码 1 / 码 3 与指针头部在 A3-checker-2 里做」；D22（单元原子性怎么合成） 已定项 9 那句写「读者遇到系统配置或单元头的格式版本不是 1，一律拒收那一槽 / 那一个单元，不按今天的字段表往下解（主 agent 2026-09-27 定，池级判定在 A3-checker-2 里做）」；已定项 2 那条「读者择系统配置时判根槽宽与固定结构槽距……越界整池拒绝挂载」没提 checker 报哪条不变量；`invariants.md` 没有 I-7.13 这一行，池级 checker 判 46 条。
>
> 快查·改后：D9（加密） 已定项 10 射程改成现值：三种码单元头 29 字节全 0与格式版本认得、及走读跟随的指针头部 MAC / nonce 全 0，均归 I-2.4（头校验和覆盖范围）；系统配置格式版本、加密类型越界归新立的 I-7.13（系统配置池级字段在读者收的范围里）。D22（单元原子性怎么合成） 已定项 9 那句改成「池级 checker 判定已落地：单元头格式版本归 I-2.4（头校验和覆盖范围），系统配置格式版本归 I-7.13（系统配置池级字段在读者收的范围里）」；已定项 2 那条补一句「池级 checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、该池其余不变量报不适用」。`invariants.md` 新增 I-7.13（系统配置池级字段在读者收的范围里），池级 checker 判定条数 46 → 47。

- 改前：D9（加密） 已定项 10 射程末句写「池级 checker 判单元头 29 字节全 0（`check_unit`）；池级码 1 / 码 3 与指针头部在 A3-checker-2 里做」；D22（单元原子性怎么合成） 已定项 9 那句写「……不按今天的字段表往下解（主 agent 2026-09-27 定，池级判定在 A3-checker-2 里做）」；已定项 2 那条只到「越界整池拒绝挂载，与每区槽数 S 越界同一个处置」，没提 checker 报哪条不变量；`invariants.md` 没有 I-7.13 这一行，第 12 行写「判 46 条」。
- 改后：D9（加密） 已定项 10 射程改写：池级 checker 判三种码单元头 29 字节全 0与格式版本认得、及走读跟随的指针头部 MAC / nonce 全 0，均归 I-2.4（头校验和覆盖范围）；系统配置格式版本、加密类型、固定结构槽距、`physical_block_size`、journal 环长越出这一版读者收的范围，归 I-7.13（系统配置池级字段在读者收的范围里）。D22（单元原子性怎么合成） 已定项 9 那句改成「……不按今天的字段表往下解（主 agent 2026-09-27 定；池级 checker 判定已落地：单元头格式版本归 I-2.4（头校验和覆盖范围），系统配置格式版本归 I-7.13（系统配置池级字段在读者收的范围里），实审 A3-checker-2）」；已定项 2 那条追加「池级 checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、该池其余不变量一律报不适用（实审 A3-checker-2 落地：`crates/singlefs-checker/src/image.rs` 的 `judge_system_configuration_values_the_reader_accepts`）」。`invariants.md` 新增一行 I-7.13（系统配置池级字段在读者收的范围里），判据是任一自证过的系统配置槽格式版本 = 1、加密类型 = 0、固定结构槽距 ≥ 4096 且槽 1 落在根环基址之前、`physical_block_size` ∈ [457, 槽距]、journal 环长 ÷ 4096 ÷ F ≥ 1 且环末端不越过单元区起点；任一盘任一槽不成立即判红、该池其余不变量报不适用（R、S 越界不在这一条里，仍归「这一槽不可择」）；池级 checker 判定条数 46 → 47（`crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS`：新增常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`，:46/:54）。
- 依据：实审 A3-checker-2 实现员报告 `research/prompts/m2-rev-a3-checker-2-implementer-report.md` 第四节第 1、2 点；用户 2026-09-27 JST 14:0x 定「系统配置字段越界整池拒（推荐）」，原话在 `records/2026-09-24-里程碑二收尾调度.md`「实审 A3-checker-2 交回并打上；I-7.13 立号（2026-09-27 JST 17:0x）」那一行。

### 2026-09-27（其十五）
````
依据：research/prompts/m2-rev-a3-checker-2-implementer-report.md 第四节；records/2026-09-24-里程碑二收尾调度.md「实审 A3-checker-2 交回并打上；I-7.13 立号（2026-09-27 JST 17:0x）」那一行

文件：.claude/kb/invariants.md
旧串：
````
## 历史版本

### 2026-09-27（立 I-1.11
````
新串：
````
## 历史版本

### 2026-09-27（立 I-7.13（系统配置池级字段在读者收的范围里）；I-2.4（头校验和覆盖范围） 与 I-1.10（码 2 条目宽等于字段表宽） 状态列改成现值；已实现 46 → 47）

- 实审 A3-checker-2（`research/prompts/m2-rev-a3-checker-2-implementer-report.md`）交回并打上，用户 2026-09-27 JST 14:0x 定「系统配置字段越界整池拒」（`records/2026-09-24-里程碑二收尾调度.md`「实审 A3-checker-2 交回并打上；I-7.13 立号（2026-09-27 JST 17:0x）」那一行）。
- I-7.13（系统配置池级字段在读者收的范围里） 立，状态已实现：判据是任一自证过的系统配置槽格式版本 = 1、加密类型 = 0、固定结构槽距 ≥ 4096 且槽 1 落在根环基址之前、`physical_block_size` ∈ [457, 槽距]、journal 环长 ÷ 4096 ÷ F ≥ 1 且环末端不越过单元区起点；任一盘任一槽不成立即判红、整池不作保（该池其余不变量报不适用），R、S 越界不在这一条里、仍归「这一槽不可择」。判定 `crates/singlefs-checker/src/image.rs` 的 `judge_system_configuration_values_the_reader_accepts`（:454），`walk::check_pool_image`（walk.rs:5573 起）调用；坏镜像还没补，`checker_known_bad_images.rs` 那条「清单里每条都要有坏镜像」的断言暂时判红，交主 agent 派实现员补。已实现条数 46 → 47，与 `IMPLEMENTED_INVARIANTS` 相等。
- I-2.4（头校验和覆盖范围）：状态列改成现值——池级走读在 `walk::judge_unit_header`（walk.rs:649 起）判三种码单元头的格式版本 = 1、29 字节预留位全 0；走读跟随的每条指针在 `judge_pointer_mac_and_nonce_are_zero`（image.rs:619）判头部 MAC / nonce 全 0，四个入口（码 2 节点指针、inode 内部条目子指针、实例表链指针、extent 数据指针）都已接上；改前状态列写「指针头部那一半的池级判定在 A3-checker-2 里做，还没落地」。
- I-1.10（码 2 条目宽等于字段表宽）：状态列改成现值——`index_node_view_judging_a_zero_entry_width`（walk.rs:747）在解码失败理由是「条目宽 0 而条目数非 0」、且这棵树登记了条目字段表时报违例；树表与中央映射树不在射程内，只记走读失败。改前状态列写「条目宽为 0 时条目数非 0 那一条池级判定在 A3-checker-2 里做，还没落地」。

### 2026-09-27（立 I-1.11
````
依据：research/prompts/m2-rev-a3-checker-2-implementer-report.md 第三、四节；crates/singlefs-checker/src/walk.rs、image.rs 现查
