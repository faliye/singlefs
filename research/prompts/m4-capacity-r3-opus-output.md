# m4-capacity-r3 攻方腿报告（Opus）

日期：2026-11-17。最终轮。攻击面：「已定」项撑不撑得住（判据格第 1 格）、未定项清单的完整性（第 2 格）、报出的数能不能核（第 3 格）。不重复前两轮已攻过的角度：甲的单盘记录（H1、H2）、记录会变的窗口（G1–G4）、判决的拼接处（M1–M5）、D2（RAID 条带策略） 已定项 13 的单调性（S1、S2）。方案按开工时 `crates/` 的实现谈；每处引文都在主工作区当下的文件上 `grep -nF` 过。

**这条腿没写模型、没建副本、没跑任何项目代码。** 下面的打中与没打中是读码、读条款、算术推的；数一律标「算的」（手算或 shell 算术，没经装置）或「推的」。要坐实的两条，主 agent 跑「要主 agent 跑的命令」一节那两条现成单测就够，不用新原型。

「形态」指正文「问题」一节合并形态第二版的五条（`research/prompts/_m4-capacity-r3-body.md:9`–`:13`）。新编号起新族：P（形态逐句）、Q（未定项清单）、V（数），不接前两轮的字母。

## 各格判定一览

| 格 | 对象 | 结论 | 号 |
|---|---|---|---|
| 1 形态逐句 | 读者句「表项数与 devs 不等整池拒」 | 打中：core 读者手里没有独立于 devs 的「表项数」；唯一看得见表后多出字节的是槽尾全零那一判，它按用户定案只归 checker（C728「恒 0，由 checker 判」），core 照做就撞定案。真正缺的是读表之前判 devs 的上界：今天 core 不判 devs 的值，按 devs 切 8·devs 字节，devs ≥ 451 的自证槽切出槽外 | P1 |
| 1 形态逐句 | 落点句（没给偏移） | 打中（字面冲突 + 实现陷阱）：505 = 489 + 16 只能是追加在 F 之后；而「F 住字段表最后一行」写在代码注释、格式常量文档与 D22 已定项 9 依据里，core 的 F 偏移从合计减 8 推出，合计一改 F 就挪；GPU 两份判器与 checker 坏镜像用例写死 481 / 489 | P2 |
| 1 形态逐句 | 读者句「跨槽…不等整池拒」 | 没打中实质：做得出；要限定在本池 fsid、这一版读者认得的自证槽（incompat 不认得的槽字段表不同） | P3 |
| 1 形态逐句 | 出口句「全部取记录值」 | core 没打中：经 `PoolReader::device_size_in_bytes` 读盘字节数的 8 个调用点（不算转发的包装实现；含实例表清扫、补写闸扫描、journal 扫描、新池排演下限）都被「读者实现」罩住；没罩的是 checker 与 GPU 判器（第二轮 N4 已攻，这一轮不算新打中，只在第 2 格记一条漏项） | P4 |
| 1 形态逐句 | 其余各句（只在 mkfs 写、每条写路抄表、两槽不自证照别的盘比、变大走 D2 已定项 3） | 没打中：写系统配置槽只有 mkfs 与 `write_system_configuration_slot` 两处入口，抄表做得出；与 D22 已定项 9 射程「唯一的盘级字段」不撞（表是池级） | — |
| 2 未定项清单 | 已知四条之外 | 漏六条：表项数 / devs 上界的判法（P1，牵 C728 用户定案）；表在槽内的偏移（P2）；容量表自己归 D22 已定项 26 哪一档（已知清单只写了 devs）；第一轮 K5、K6 第二轮判决没给去向；checker 与 GPU 判器的容量出口不在形态里；「变大放行」怎么报出（fs-design 硬要求 4） | Q1–Q6 |
| 3 数 | 489、505、513、离 512 余量 | 数都对（算的）：今天 489、余 23；devs = 2 为 505、余 7；devs = 3 为 513；表最多 450 项。但 devs = 3 在 `crates/` 不可达，挂在这几个数上的「跨 512 要多枚举一个撕裂维度」在 `crates/` 不成立：崩溃点重放对每次系统配置槽写已取撕裂第三态，切点是新旧不同那一截的中点、不是 512，容量表两写之间不变、进不了那一截。撤回第二轮我的 N5 | V1、V2 |
| 3 数 | 两盘各 175 GiB 到 176 GiB 跨根层级边界 | 对（算的）：边界在每盘 11,527,152 槽（≈ 175.89 GiB），根层级 2 → 3。只一块盘长大时边界不同（另一块 175 GiB 时在 11,664,380 槽 ≈ 177.98 GiB）；1 → 2 那道边界在每盘 68,208 槽（≈ 1.04 GiB），单盘从 1 GiB 长到 71,457 槽（≈ 1.09 GiB）就跨，是便宜的强制进入点；今天跨边界之后可写挂载报层级错（推的），交用户第 3 条「今天的行为没了」说过头 | V3 |
| 岔路清单 | 交用户的三条 | 漏一条（P1 的判法里有一支要改 C728 用户定案）；第 3 条（盘变大用不上）该降成告知或补代价数；第 1、2 条照规则也缺代价数 | 见「岔路清单」 |

## 要主 agent 跑的命令

两条都是现成单测，核 V2、V3 的前提；这条腿自己没交。

1. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的库测试里的 2 条测试; 范围=2 个测试; 为什么=核系统配置槽写在崩溃点重放里已取撕裂第三态、撕裂镜像按新旧不同那一截的中点切（三方 m4-capacity-r3 攻方 V2）' task-server/task submit -- cargo test -p singlefs-verification --lib -- crash::tests::the_state_plan_with_torn_in_place_overwrites_hands_out_every_landing_combination_once crash::tests::the_torn_image_keeps_the_first_half_of_the_differing_bytes_new_and_the_rest_old --exact`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里「test result: ok. 2 passed」。红了 V2 的前提不成立、V2 撤回。
2. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的库测试里的 1 条测试; 范围=1 个测试; 为什么=核根层级边界按 812 片叶、扇出 169 的算式（三方 m4-capacity-r3 攻方 V3）' task-server/task submit -- cargo test -p singlefs-core --lib -- allocation_record_tree::tests::half_a_slot_at_the_end_of_each_device_does_not_move_the_root_level_across_its_boundary --exact`
   - 在哪起：仓根。第一轮判决记的任务 18750 跑的就是这一条、退 0；主 agent 判定可以直接引那一次，不重跑。
   - 看哪几行：`exit=0`；log 里「test result: ok. 1 passed」。

## 攻击面一：第 1 格，形态逐句

逐句读成要写回的条款：

| 句 | 做得出 | 冲突 |
|---|---|---|
| 落点：逐盘容量表，按设备号 0 到 devs − 1、每项 8 字节 | 做得出；第一版两盘的设备号恒 0、1（mkfs 的 `check_geometry` 要区域归属等于 0 / 1 / 0） | 没给偏移，见 P2 |
| 只在 mkfs 写；每条写路抄择到的表 | 做得出：写系统配置槽只有 mkfs 与 `write_system_configuration_slot`（`crates/singlefs-core/src/transaction.rs:671`）两处入口，轮换、取号、回卷、抬 F、离线出路 α' 都经后一处；它今天按盘表现数写 devs：「device_count: u32::try_from(self.devices.len()).expect("设备数"),」（`crates/singlefs-core/src/transaction.rs:723`），抄表要另带一份 | 句里列的四条路少写了抬 F 与回卷（D22 已定项 16 点名的三种写之一），走同一入口，不算洞 |
| 读者：跨盘、跨槽表不等整池拒 | 做得出 | 「跨槽」要限定，见 P3 |
| 读者：逐盘比、变大放行、两槽不自证照别的盘比 | 做得出 | 不撞 |
| 读者：表项为 0、越界、表项数与 devs 不等整池拒 | 「表项数」做不出 | 见 P1 |
| 出口全部取记录值 | core 做得出 | checker、GPU 判器不在句里，见 P4 |
| 变大走 D2 已定项 3，第一版没有 | 做得出（什么都不做） | 交用户第 3 条的说法见 V3 |

### P1 「表项数与 devs 不等整池拒」：core 没有判别子，照做撞 C728；缺的是读表之前判 devs 的上界（打中）

形态原句：「表项为 0、越界、表项数与 devs 不等整池拒」（`research/prompts/_m4-capacity-r3-body.md:11`）。它来自第二轮辩方那一行，原话是定义加一道判：「读者要求 `1 ≤ devs` 且 `devs × 项宽` 装得进槽内字段表末尾之后的余量，否则整池拒」（`research/prompts/m4-capacity-r2-sonnet-output.md:110`）。形态把「项数 = devs」那个定义改写成了一道判，把真正那道判丢了。

- 表按位置排、项数由 devs 定，槽里没有独立的项数字段。core 读者能拿来比的只有两样：devs，与从 489 起的 8·devs 字节。「表项数 ≠ devs」唯一看得见的形状是表后还多出非零字节，也就是槽尾不全零。
- 槽尾全零那一判，用户定的是只归 checker：C728 的「怎么拦」一格写「恒 0，由 checker 判」（`.claude/kb/checks-owed.md:430`），I-2.4 写系统配置槽尾「core 的读者不读这两段」（`crates/singlefs-checker/invariants/04-校验和.md:9`）。checker 一侧在读槽时取第一个非零字节（`crates/singlefs-checker/src/lib.rs:338`），GPU 判器同一处写死起点：「const SYSTEM_CONFIGURATION_FIELD_TABLE_BYTES: u32 = 489u;」（`crates/singlefs-verification/src/crash_judge_checker.wgsl:6227`）。
- 所以这一句在 core 上要么是空的（没有一种盘面让它单独红，D12 已定项 4 射程要每一条比对项都能被强制走进对不上那一支），要么要 core 去读槽尾，那就推翻 C728 的用户定案、把 I-2.4 那一半从「checker 判」变成「整池拒」。
- 真正缺的那道判：今天 core 读 devs 不判值，`parse_slot` 里就是「let device_count = reader.get_u32();」（`crates/singlefs-core/src/system_configuration.rs:746`），择系统配置只判跨盘相等：「|| previous.immutable.device_count != best_on_device.immutable.device_count」（`crates/singlefs-core/src/recovery.rs:1901`）；checker 也只判相等：「view.device_count == chosen[0].1.device_count,」（`crates/singlefs-checker/src/walk.rs:21030`）。今天 devs 是什么值都不碰内存边界；形态之后要按它切 8·devs 字节，槽 4096、表从 489 起，最多装 (4096 − 489) ÷ 8 = 450 项（算的）。

历史（推的）：

- 前缀：mkfs 两盘。
- 故障：两盘四槽的 devs 都改成 1000、整槽校验和重封（崩溃不会造出这个，CRC 碰撞不在模型里；`crates/singlefs-verification/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 这一族造的就是这种盘面，要求拒而不 panic）。
- 用户动作放开扫：只读挂载、可写挂载、`recover`。
- 今天：可写挂载由盘表比对拒（「盘数不是 devs」那一项）；只读挂载不读 devs，照挂。形态照字面实现、不另判 devs 的：三条入口读表都从 489 切 8000 字节，越过 4096 的槽，`ByteReader` 下标越界 panic。可写挂载那一条要看比表在比盘表之前还是之后，只读那一条一定先读表。

四问：

1. 分不分辨臂：分辨。甲只在本盘槽里固定 8 字节、不按 devs 切，没有这一格；只中乙。
2. 系统当时看不看得到判别子：「表项数」看不到（没有独立字段，槽尾 core 按定案不读）；devs 看得到，只是今天不判。
3. 满足判据哪一分句：第 1 格「做不出」（独立的表项数判不出）与「与别的已定项冲突」（用槽尾判就撞 C728 用户定案），两个分句都满足。
4. 跑前条款给的改法在这一格中不中：前两轮判决没给这一格的改法。我的改法（零轮，只在推演上过）：删掉「表项数与 devs 不等」；换成「读表之前判 devs：第一版 devs = 2（或 1 ≤ devs ≤ 450），不在整池拒」，排在比表、比盘表之前；表后 [489 + 8·devs, 4096) 照 C728 由 checker 判 I-2.4，端点随 devs 移（I-2.4 正文已有「往这两段余量里加字段时，同一次改动改 checker 的端点」那一句）。

推翻条件：主 agent 现查出 core 某处已经判槽尾全零、或判 devs 的值域（我只查了 `recovery.rs`、`system_configuration.rs`、`mount.rs`、`instance_table_sweep.rs` 与 checker 的 `walk.rs`、`lib.rs` 里 devs 的用处）。

### P2 落点句没给偏移：追加在 F 之后撞「F 住字段表最后一行」，而 core 的 F 偏移从合计推（打中，字面冲突 + 实现陷阱）

505 = 489 + 16 只在表追加到现行字段表之后时成立，也就是排在回退下界 F（481 起 8 字节）之后。三处已写成「F 是最后一行」：

- 代码注释：「回退下界 F 住字段表最后一行、journal 实例代号之后」（`crates/singlefs-core/src/system_configuration.rs:83`），紧接着那一行把 F 的偏移从合计推出来：「const ROLLBACK_FLOOR_OFFSET: u64 = SYSTEM_CONFIGURATION_BYTES - ROLLBACK_FLOOR_FIELD_BYTES;」（`crates/singlefs-core/src/system_configuration.rs:85`）。
- 格式常量文档：「系统运行量末尾是回退下界 F 8 字节，偏移 481 起，481 → 489」（`crates/singlefs-format/src/lib.rs:275`），值是「pub const SYSTEM_CONFIGURATION_BYTES: u64 = 489;」（`crates/singlefs-format/src/lib.rs:276`）。
- D22 已定项 9 依据：「F 放在字段表末尾、偏移 481 起是本规格按「追加在系统运行量末尾」取的」（`.claude/kb/decisions/22-单元原子性怎么合成.md:235`）。

陷阱：实现照形态把合计改成 505，core 的 F 偏移自动挪到 497（落在盘 1 那一项里），写侧 `to_slot` 收尾断言的也是这个合计（「.assert_position(SYSTEM_CONFIGURATION_BYTES, "系统配置");」，`crates/singlefs-core/src/system_configuration.rs:551`）；GPU 判器两份写死 F 在 481：「c.floor = rd64(base, 481u);」（`crates/singlefs-verification/src/crash_judge_common.wgsl:1062`，`crash_judge_checker.wgsl` 第 176 行同一句）；checker 坏镜像用例按合计减 8 找 F：「usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_BYTES - 8)」（`crates/singlefs-verification/tests/checker_known_bad_images.rs:5886`）。core 与判器各读一个 F，F 那几条 I-7.12 一类的判就分叉。另一种放法（插在系统运行量之前）不挪 F，但挪 469 / 477 / 481 三个在 checker、GPU 判器与字节表里写死的偏移，合计照样是 505。

四问：① 不分辨（甲在 489 追加 8 字节同撞）；② 不适用；③ 第 1 格「与别的已定项冲突」：字面撞 D22 已定项 9 依据那一句与代码里的两处现状说明，做得出；④ 改法（零轮）：形态写明「表追加在 489 起，F 仍在 481」，同一批把 F 偏移写死成 481（或从「固定段合计 489」推），把格式常量拆成「固定段 489」与「表 8·devs」两个量，GPU 两份判器的 489u 与那条坏镜像用例同改；D22 已定项 9 依据那一句改成「F 在固定段末尾」。

### P3 「跨槽」要限定（没打中实质）

今天跨盘只比各盘择到的那一槽（`recovery.rs:1901`）。形态的「跨槽」把每块盘没择到的那一槽也拉进来比，做得出（取号写系统配置之前就逐盘读两槽：`verified_system_configuration_slots`）。要写明只比本池 fsid、incompat 位这一版认得的自证槽：incompat 不认得的槽字段表是另一套，按 D22 已定项 16 本来就不可择、不核偏移，拿它的 489 起那几字节比表会把一个可择的池判成整池拒。

### P4 出口句：core 全被「读者实现」罩住；没罩的是 checker 与 GPU 判器（N4 已攻，这一轮只核落点）

形态原句：「容量读数的出口（建空闲图、读者实现、落点越界判、单元区起点判、分配记录树读侧与根层级）全部取记录值」（`research/prompts/_m4-capacity-r3-body.md:12`）。core 里读盘字节数的现查结果：`BlockDevice::size_in_bytes` 在测试之外的直接调用只在建空闲图（`crates/singlefs-core/src/mount.rs:3242`）、mkfs、`PoolReader` 的切片实现（`recovery.rs:552`）与块设备读写的范围护栏几处，别的都经 `PoolReader::device_size_in_bytes`；后者的调用点有实例表清扫（`instance_table_sweep.rs:417`）、补写闸与离线出路的单元区扫描（`system_configuration_repair.rs:59`）、两处 journal 扫描（`recovery.rs:5274`、`crates/singlefs-core/src/mount.rs:7000`）、新池排演下限（`recovery.rs:6779`）、落点越界与单元区起点（`recovery.rs:2847`、`:2886`）、分配记录树读侧（`allocation_record_tree.rs:156`）。「读者实现」换成记录值，这几处一起换，句子罩得住（N2 说的护栏变恒过是第二轮的事，不重攻）。

句子外面的是 checker：根层级「.map(|device| self.reader.device_bytes(*device).unwrap_or(0) / SLOT_BYTES)」（`crates/singlefs-checker/src/walk.rs:5274`）与「.map(|device| reader.device_bytes(*device).unwrap_or(0) / SLOT_BYTES)」（`crates/singlefs-checker/src/walk.rs:7156`），另有 `walk.rs` 第 5734、5851、11592、20550、20722、22288 行按镜像字节数取盘尾；GPU 判器同（第二轮 N4 引了 `crash_judge_common.wgsl:2631`）。盘长大跨根层级边界之后，core 按记录写树、checker 按镜像大小算层级，合法池被 checker 判红。这一格第二轮 N4 已攻、判决记成验收；我只在第 2 格 Q5 记它没进形态、也没进未定清单。

## 攻击面二：第 2 格，未定项清单

已知四条：N1 根层级的设备集合；只读拒不拒（交用户）；与「设备描述符表第一版不写」的字面关系（交用户）；devs 在 D22 已定项 26 的归档（欠账）。漏的六条：

| 号 | 漏的未定项 | 依据 | 归谁定 |
|---|---|---|---|
| Q1 | 读表之前怎么判 devs、「表项数」那一句留不留（P1）：三支——加一个独立的项数字段（每份多付字段宽，505 跟着变）；core 判槽尾全零（推翻 C728 用户定案「恒 0，由 checker 判」）；只判 devs 的值域、删「表项数」那一句、槽尾照 C728 由 checker 判 | P1 | 第二支动用户定案，交用户；第一、三支主 agent 可定 |
| Q2 | 表在槽内的偏移，与 F 是不是还「住字段表最后一行」（P2） | P2；F 的放法本身「被攻过零轮」（D22 已定项 9 依据那一句） | 主 agent |
| Q3 | 容量表自己归 D22 已定项 26 哪一档。已知清单只写 devs；第二轮 T1 说容量表与 devs 同形，判决只把 devs 记成欠账。代码里四档的字节数是写死的断言（`system_configuration.rs` 第 558 到 577 行：不可变一档必须 389），表进不可变一档就变成 389 + 8·devs，断言要改；进别的档撞第一轮 K3 | T1、K3 | 与 devs 那笔欠账并成一笔 |
| Q4 | 第一轮 K5（C1040 那一行字面要互比）、K6（D12 已定项 4「为什么」一列越位）的去向。第一轮判决写它们「不随形态消失，第二轮逐条判」（「K5（C1040 那一行字面要互比）」，`research/prompts/m4-capacity-r1-main-verification.md:30`）；第二轮判决没提这两条；这一轮正文第 5 问的改句清单只列 D12 已定项 4 与 5、D22 已定项 9 与 15、D2 已定项 3，没有 C1040 那一行。那一行今天写「两盘容量不互比（只有分配记录越过单元区末尾时拒」与「与系统配置记录的值、另一块盘的值比」（`.claude/kb/checks-owed.md:642`），D12 已定项 4 进度一格也写「两盘容量不互比」 | K5、K6 | 主 agent（改句清单补 C1040 行与 D12 已定项 4 进度格） |
| Q5 | checker 与 GPU 判器的容量出口不在形态的出口句里（P4），未定清单也没有；第二轮判决把 N4 记成验收、没记成形态的一句 | P4、N4 | 主 agent |
| Q6 | 「变大放行」怎么报出。fs-design 硬要求第 4 条「分支必须可观测：运行时看得出走了哪一条」（`.claude/rules/fs-design.md:80`）；形态写了放行、有效容量取记录值，没写挂载结果里报不报「这块盘比记录大」。正推第一轮提过一个报出字段，合并形态没带上 | fs-design 硬要求 4 | 主 agent |

改句清单另要连带的（不是未定，是第 5 问清单的漏项）：I-2.4 正文里「系统配置槽 [489, 4096)」那一段的端点（`crates/singlefs-checker/invariants/04-校验和.md:9`）；`layout/01-first-txn.md` 写清单里 m4、a1、w3、w6、t15 五行的「489 / 4096 槽」与合计那一段，连同它的英、日两份译本；D22 已定项 9「⚠️ 现值 489 没跨 512」那一条（见 V2）。

没漏的（试过、判不算漏）：表项的键（位置排还是带设备号）——第二轮判决已定位置排、零轮（「表项按设备号 0 到 devs − 1 排」那一条）；第二轮辩方给带设备号那一支挂的代价是「当场跨 512」，见 V2，那条代价不成立，所以位置排与带设备号只差 8 字节，不值一条岔路。

## 攻击面三：第 3 格，报出的数能不能核

### V1 489、505、513、离 512 的余量：数对；devs = 3 在 crates/ 不可达（核得上）

- 今天的字段表：格式常量「pub const SYSTEM_CONFIGURATION_BYTES: u64 = 489;」（`crates/singlefs-format/src/lib.rs:276`），写侧收尾按它断言（`system_configuration.rs:551`）；`layout/01-first-txn.md` 第 160 行的逐字段偏移按行累加到 489（算的：自举头到 devs 147，槽世代号 155，整槽校验和 187，加密预留到 301，几何段到 433，可调值到 469，tail 477，实例代号 481，F 489）。离 512 余 23（算的）。
- 加表：devs = 2 时 489 + 2 × 8 = 505，余 7；devs = 3 时 489 + 3 × 8 = 513；表最多 ⌊(4096 − 489) ÷ 8⌋ = 450 项（算的）。
- devs = 3 在 `crates/` 今天造不出来：mkfs 内层入口断言「"第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）"」（`crates/singlefs-core/src/make_filesystem.rs:607`）；内存里的区域归属定长 3 项「pub region_devices: [DeviceIdentity; 3],」（`crates/singlefs-core/src/system_configuration.rs:304`）；D22 已定项 16 第 3 句「多于两块盘换 P 要另立格式决策」（`.claude/kb/decisions/22-单元原子性怎么合成.md:355`）；D2 已定项 18「第一版加盘准入拒绝让 w ≥ 3 可达的设备集合变化，devs 上限 2，直到条带表开线」（`.claude/kb/decisions/02-RAID条带策略.md:302`）。所以「devs = 3 起 513」说的是另一次格式决策之后的盘面，不是这一题的盘面。

核的命令：上面三处 `grep -nF` 与 shell 算术，没经装置。

### V2 「跨 512 要多枚举一个撕裂维度」在 crates/ 不成立；撤回第二轮我的 N5（打中，挂在数上的代价核不上）

D22 已定项 9 的原话：「代价知情接受：崩溃点重放要多枚举一个撕裂维度」（`.claude/kb/decisions/22-单元原子性怎么合成.md:219`）与「现值 489 没跨 512」（同一份文件第 221 行）。第二轮我的 N5 与第二轮辩方都拿 505 / 513 去对这条代价。现查崩溃点重放：

- 系统配置槽写已经是可撕裂的原地覆写：判它的函数在单元写之外只看两样——写长过不过一个扇区、罩住的范围里原来有没有东西（「if is_copy_on_write || write.length_in_bytes() <= SECTOR_BYTES {」，`crates/singlefs-verification/src/crash.rs:2343`；系统配置槽那一臂「| StepKind::SystemConfigurationSlot」在第 2340 行，归非单元写）。槽写 4096 字节、mkfs 已在两槽写过东西，所以 mkfs 之后每一次系统配置槽写都取第三态。单测把这一点钉住了：基镜像那一槽有旧内容的系统配置槽写、与它重叠的后一次系统配置槽写都在可撕裂的清单里（「assert_eq!(tearable.write_indexes(), vec![0, 3, 4]);」，`crates/singlefs-verification/src/crash.rs:3195`）。
- 撕裂镜像不按 512 切，按新旧不同那一截的中点切：「新旧不同的那一截（第一个不同的字节到最后一个不同的字节）前一半是新的」（`crates/singlefs-verification/src/crash.rs:2366`）。
- 形态下容量表两次写之间不变，进不了「新旧不同的那一截」；那一截今天是槽世代号到 F（147 到 489）。

所以：字段表是 489、505 还是 513，重放给每次系统配置槽写的状态数都是同一个 3，撕裂镜像的字节也一样。真设备上（`physical_block_size` = 512）跨 512 只对两次写之间会变的字段有后果，容量表在第一版不变；它要变得等 D2 已定项 3 那条事务，那时才是要付的代价。

四问：① 不分辨（甲、乙都不碰）；② 不适用；③ 第 3 格「数能不能核」：数核得上，挂在数上的代价核不上，D22 已定项 9 那两句与实现不符；④ 改法：D22 已定项 9 改句不写「现值 505 没跨 512、devs = 3 起跨」这一类句子，改成「表在两次写之间不变，跨不跨 512 不改变撕裂之后的可择性；崩溃点重放对每次系统配置槽写已取撕裂第三态」，那两句的现状另记一笔欠账。**撤回第二轮我的 N5**（「乙吃掉 512 撕裂边界前的大半余量」那一条代价）：它称的代价在 `crates/` 不存在。

推翻条件：「要主 agent 跑的命令」第 1 条红；或主 agent 现查出另有一条崩溃点重放的路按 512 扇区切系统配置槽写（我只读了 `crash.rs` 的 `TearableInPlaceOverwrites` 与 `torn_image_of_in_place_overwrite`，GPU 判器那一侧的撕裂没读）。

### V3 「两盘各 175 GiB 到 176 GiB 跨根层级边界」：对；精确边界、单盘长大的边界与今天跨边界的行为（核得上，另有两处补充）

算式：根层级是最小的 L ≥ 1，使各盘「绝对槽数 ÷ 第 L − 1 层一个孩子罩的槽数」向上取整之和 ≤ 169（`crates/singlefs-core/src/allocation_record_tree.rs:105`–`:113`，「cells <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT」在第 112 行）；叶罩「pub const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;」（`crates/singlefs-format/src/lib.rs:163`），扇出「pub const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;」（`crates/singlefs-format/src/lib.rs:169`）；绝对槽数是字节 ÷ 16384（`crates/singlefs-core/src/allocator.rs:528`）。

| 情形 | 边界（每盘槽数） | 折合 | 算式（算的） |
|---|---|---|---|
| 两盘等大，层级 2 → 3（第一轮正推那句） | 11,527,152 槽在 2、多一槽到 3 | ≈ 175.89 GiB | 812 × 169 = 137,228；84 × 137,228 = 11,527,152；175 GiB = 11,468,800 槽，每盘 84 格共 168；176 GiB = 11,534,336 槽，每盘 85 格共 170 > 169 |
| 一块 175 GiB 不动、另一块长大，层级 2 → 3 | 长大那块超过 11,664,380 槽 | ≈ 177.98 GiB | 169 − 84 = 85；85 × 137,228 |
| 两盘等大，层级 1 → 2 | 68,208 槽在 1、多一槽到 2 | ≈ 1.04 GiB | 84 × 812；现成单测钉住：「let slots_of_eighty_four_leaves = ALLOCATION_RECORD_TREE_LEAF_SLOTS * 84;」（`crates/singlefs-core/src/allocation_record_tree.rs:977`），断言「盘尾那半个槽不算：两盘各 84 格，根在第 1 层」（同一份文件第 987 行） |
| 一块 1 GiB 不动、另一块从 1 GiB 长大，层级 1 → 2 | 长大那块到 71,457 槽 | ≈ 1.09 GiB | 1 GiB = 65,536 槽 → 81 格；169 − 81 = 88；88 × 812 = 71,456 |

第一轮正推的原话「两盘等大，175 GiB 时根层级 2，176 GiB 时 3」（`research/prompts/m4-capacity-r1-sonnet-output.md:24`）核得上。两处补充：

- 「变大跨边界」的验收不必造 176 GiB 的盘：最后一行那个 1 GiB → 1.09 GiB 的单盘长大就跨 1 → 2（1 GiB 的盘要把 journal 环长设在四分之一即 256 MiB 以内），内存稀疏盘上的事（推的，没量挂钟）。崩溃点重放的历史只有两档盘宽：4 GiB（每盘 2 格，层级 2，要跨 2 → 3 得把一块长到三百多 GiB）与几千槽的小盘（`crates/singlefs-verification/src/history.rs:187` 写最窄那档「1408 + 1007 + 240 = 2655 槽」，每盘 4 格，层级 1）；N4 要加的「mkfs 之后把一块盘扩大再接着写」那段历史取小盘那档、把一块长到超过 (169 − 4) × 812 = 133,980 槽（≈ 2.04 GiB，算的；稀疏盘不占内存）就跨 1 → 2。
- 今天跨边界之后的行为（推的，没跑）：读侧几何按读者报的字节数现算（`recovery.rs:4461` 的「&AllocationRecordTreeGeometry::of_reader(reader),」），盘上的树是按长大之前的层级写的；读到根节点时「if header.level != self.geometry.level_of(node) {」（`crates/singlefs-core/src/allocation_record_tree.rs:709`）报「"分配记录树节点的层级不是它的位置规定的那一层",」（同一份文件第 711 行）。可写挂载与 `recover` 都读这棵树，所以都拒，拒因是层级错、不说容量；只读挂载不读分配记录树（`mounted_read.rs` 里没有分配记录树几何），照挂。形态之后三种入口都挂上、长出来的那部分不用。

所以交用户的第 3 条（第一轮判决「这件今天的行为没了」，`research/prompts/m4-capacity-r1-main-verification.md:40`）说过头：今天盘长大只在不跨层级边界时用得上，跨了就可写挂载报层级错拒；形态把这一格从「看长了多少，可能挂不上」变成「一律挂得上、多出的不用」。

四问（只问第二条补充）：① 不分辨甲乙（两者都取记录值）；分辨「形态」与「今天」；② 今天的读者看得到探测值、看不到记录，判别子缺的正是记录；③ 第 3 格「数能不能核」的连带：数核得上，第一轮判决给这个数配的「今天的行为」读法核不上；④ 不涉改法，涉交用户那一条怎么写（见下节）。

推翻条件：「要主 agent 跑的命令」第 2 条红；或主 agent 造「两盘 1 GiB mkfs、盘 1 扩到 71,457 槽、可写挂载」这一段，看到挂载成功（那样「今天跨边界拒」不成立）。

## 岔路清单：交用户的漏了哪条、多了哪条

现有三条（第二轮判决「交用户的」，`research/prompts/m4-capacity-r2-main-verification.md:44`–`:46`）：① 乙与「设备描述符表第一版不写」的字面关系；② 容量变小时只读拒不拒；③ 盘变大的部分要等显式几何再求值事务。

- 漏一条：**P1 的判法里那一支（core 判槽尾全零）要推翻 C728 的用户定案**。主 agent 取第一或第三支就不交；取第二支必须交，候选写清三支各自付什么（多字段宽 / 推翻「由 checker 判」/ 删一句）。我推荐第三支（零轮）。
- 第 ③ 条该降级：它不是两条臂都可取的岔路，是知情告知；照 V3 改写成「今天盘长大只在不跨根层级边界时用得上，跨了可写挂载拒；形态之后一律挂得上、多出的部分等 D2 已定项 3 那条事务」。要留成岔路，另一支是「第一版做只罩盘变大的最小几何再求值」，那就照「岔路交用户定之前，每条路要有代价数」（`.claude/rules/three-way-inference.md:51`）先量代价。
- 第 ①、② 条照同一条规则也缺代价数：① 的两支是条款改字，代价可以写成「改哪几句」，不必建计数模型；② 的两支（拒只读 / 只读照挂并报出）第二轮 S2 只给了推的理由，没有数，交之前要么补一个计数（缩小的盘上有多少单元只剩一份），要么在岔路单上写明「代价没量，原因」。
- 没有多余的：三条都是用户定案或用户可见行为的事。N1（第 4 问）与 devs 归档（欠账）不交用户，照正文。

## 撤回

- 第二轮我的 N5（「乙吃掉 512 撕裂边界前的大半余量（代价，没人称过）」）撤回：那条代价在 `crates/` 的崩溃点重放里不存在（V2）。连带撤回 N5 里「跨过最迟随条带表那条线来，届时崩溃点重放的撕裂维度与这张表一起付」那半句。
- 第二轮我在 M2 改法里写的「变长放在字段表末尾不挪别的偏移」收窄：不挪别的偏移成立，但挪 F 的推法（P2），要同一批把 F 的偏移与合计拆开。

## 改法各修哪一格

全部「推的」：按代码与条款推，没实现、没跑，没有任务号；只在我的推演上过、被攻过零轮。

| 改法 | P1 | P2 | P3 | V2 | V3 / 交用户第 3 条 |
|---|---|---|---|---|---|
| 删「表项数与 devs 不等」；读表之前判 devs（第一版 = 2，或 1..450），排在比表之前；槽尾照 C728 由 checker 判、端点随 devs 移 | 修（推的）：不再 panic、不撞 C728 | 不涉 | 不涉 | 不涉 | 不涉 |
| 表追加在 489、F 写死在 481；格式常量拆成「固定段 489」与「表 8·devs」；GPU 两份判器的 489u、坏镜像用例同改 | 不涉 | 修（推的） | 不涉 | 不涉 | 不涉 |
| 「跨槽」限定到本池 fsid、incompat 认得的自证槽 | 不涉 | 不涉 | 修（推的） | 不涉 | 不涉 |
| D22 已定项 9 改句不写「跨 512」代价，另记那两句与实现不符的欠账 | 不涉 | 不涉 | 不涉 | 修（推的，待主 agent 跑第 1 条命令） | 不涉 |
| 交用户第 3 条改写成告知，带上「今天跨根层级边界可写挂载拒」；验收用 1 GiB → 1.09 GiB 或小盘 → 2.04 GiB 两种几何 | 不涉 | 不涉 | 不涉 | 不涉 | 修说法（推的，今天的行为没跑） |

## 没打中的形状

- 写系统配置槽的入口：全仓 `crates/*/src` 里 `.to_slot()` 的产品调用只有 mkfs 与 `write_system_configuration_slot`（验证包 `crash_judge_tables.rs` 两处是造坏镜像），抄表做得出；离线出路用调用方参数那一处是第二轮 M4 与辩方条件二，不重攻。
- 表项按交进来的身份取还是按盘上自述的本盘设备号取：只读挂载今天不核本盘设备号（`own_device_numbers_on_disk_differing_from_the_identities_handed_in` 只在可写挂载与会话入口调），交进来的身份对调时容量比对会拿错项；但身份一对调，三个根环区域都按错的盘读、读不到根，挂载照样失败，只是拒因可能先报成「容量变小」。挂不上，不算打中；拒因的次序交实现时照 N3 那一格一起排。
- 记录单位：表记整槽数，探测是字节；盘缩小不到一个 16 KiB 槽时整槽数不变、放行，分配器同样按整槽，不漏。
- 设备号与位置：第一版两盘的身份恒 {0, 1}（mkfs 要区域归属等于 0 / 1 / 0，`make_filesystem.rs` 第 558 行那一判），「按设备号 0 到 devs − 1 排」做得出；掉盘换盘之后设备号不连续是另一次格式决策的事（第二轮辩方已提），不攻。
- 出口句在 core 上：8 个经读者取字节数的调用点逐个对过（P4），读者实现一换全罩。journal 扫描两处取 min(环长, 盘尾 − 环起点)，记录 ≥ 环末，取记录与取探测结果相同。
- 与 D2 已定项 4 a「不卡容量」、D2 已定项 2「各盘不必等大」、D22 已定项 9 射程「唯一的盘级字段」：表是池级、逐盘一项，都不撞。
- 取样范围：判据第 1 格逐句（五条）；第 2 格对照前两轮判决、两轮四份腿报告的打中与改法逐条找去向；第 3 格的三个数各用 shell 算术与现成单测核。

## 这条腿自己的限度

- 没写模型、没建副本、没跑项目代码。P1 的 panic、V3 的「今天跨边界可写挂载拒」都是推的。坐实的最小原型（主 agent 定要不要派，都不涉崩溃枚举）：
  - P1：在实现了形态而没加 devs 判的副本上，两盘四槽 devs 写 1000、重封校验和，只读挂载看是 panic 还是拒。今天的代码上这一格没有对象（今天不按 devs 切）。
  - V3：今天的代码上，两盘 1 GiB（环长 ≤ 256 MiB）mkfs、写一版、把盘 1 的内存稀疏盘换成 71,457 槽的同内容盘，可写挂载、只读挂载、`recover` 各一次，看拒因。
- V2 只读了 CPU 侧枚举的撕裂判法；GPU 判器拿到的崩溃状态是不是同一张写表造的，没核。
- 第 2 格「漏项」只对照了前两轮判决、两轮四份腿报告与这一轮正文；这一轮正推腿报告与实现员副本照禁读清单没读，它们里面有没有已经答掉其中几条，我不知道。
- C728 那一行与 I-2.4 正文是长行，我只核了引到的那几个短语。

## 没做什么

- 不判正推那几格（第 4、5 格没攻）；不替主 agent 采纳；没有副本上的数；没往统一任务服务交任务，判决要引的数没复跑（两条命令在「要主 agent 跑的命令」一节）。
- 没改 kb、代码与条款。

## 清理

草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-capacity-r3-attack` 里只有认领时的 `.owner`、`.writer-lock`、一份 `grep` 输出 `size-calls-core.txt` 与 cite-check 的输出；没建仓副本、编译目录与模型目录，没有要删的。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m4-capacity-r3-opus-output.md` 末行（交回前现跑，写这一节之前那一次）：「✓ 核了 33 处引文、27 处裸行号，对上 33 处，没判 27 处（其中裸行号没有引文可比 27 处；逐处列在上面）」。
