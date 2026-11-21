# m4-close-c1040-c1041-code-r1：判决（2026-11-18）

正文 `research/prompts/_m4-close-c1040-c1041-code-r1-body.md`，开工快照 `research/prompts/m4-close-c1040-c1041-code-r1-snapshot/`。代码轮第一轮，这一题没有前几轮。攻的是 C1040、C1041 两批合入与用户新加的文件区间交集长度证明。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/m4-close-c1040-c1041-code-r1-sonnet-output.md`，模型 `research/prompts/m4-close-c1040-c1041-code-r1-sonnet-model/` | C1041 四格成立；C1040 可写挂载三入口与 mkfs 成立，只读挂载没判与 D12 第 69、75、103 行不成立；证明成立，两处 None 支 `== 0` 没有专属必红行；八格六格成立；已知红测试判「测试判法要换」 |
| 攻方（Opus） | `research/prompts/m4-close-c1040-c1041-code-r1-opus-output.md`，三份测试草稿在它的副本里 | 六处打中：只读挂载做决定时看不到探测值；会话发布用户改动的入口不比探测值；一槽两格同错时 core 与池级 checker 点名不同的格；类型一格不成立；`mount_transparency` 那一读与 core 不同判（现有流走不到）；证明漏两行必红。已知红测试判「测试判法要换」 |

核查：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-model-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-close-c1040-c1041-code-r1 --out ~/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-model-rerun/m4-close-c1040-c1041-code-r1-tree --repository /home/fy5090/code/singlefs`，任务 20437 退 0。
- `python3 scripts/agents/cite-check.py research/prompts/m4-close-c1040-c1041-code-r1-sonnet-output.md research/prompts/m4-close-c1040-c1041-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-model-rerun/m4-close-c1040-c1041-code-r1-tree --background research/prompts/_m4-close-c1040-c1041-code-r1-background.md`，退 0，末行「✓ 核了 130 处引文、139 处裸行号，对上 97 处，没判 172 处（其中裸行号没有引文可比 116 处；逐处列在上面）」。判决用到的行，主 agent 在主工作区现读过（下面各条写了文件:行）。
- 正推腿的模型拷到 `~/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-model-rerun/sonnet-model`，三份文件 sha256 与腿报的相同；任务 20423 退 0，「name=model_result failures=0」「name=rerun_matches_saved_output ok=yes」。腿在交回之前不经服务直接跑过一次这个模型（报告「没做什么」），判决只引 20423。
- 攻方腿的副本拷到 `~/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-model-rerun/attack-copy`，与主工作区 `crates/` 只差三份测试（两份新草稿、一份装置加了探测调用链打印并换了两条期望），主 agent 逐份看过 diff：
  - 任务 20434 退 0，「test result: ok. 1 passed」：换成 `(盘数, 0)`、`(2 × 盘数, 0)` 的两条期望走通；第 1、2 次探测的调用链里是 `singlefs_core::mount::physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration`，第 3 次起是 `singlefs_core::mounted_session::rehearse_the_push_then_the_remainder_of_the_change`。服务另报这一条用例的 GPU 判段本次未跑（NOT_RUN_NO_GPU_ADAPTER），它在探测次数那两条断言之后，不影响这里的读数。
  - 任务 20435 退 101：「ATTACK_READ_ONLY recorded=4096 probed_on_device_0=512 result_is_ok=true」，「ATTACK_READ_ONLY recorded=512 probed_on_device_1=4096 result=None」；后一条草稿预测只读挂载会因根槽不对齐挂不上，实际挂上了，草稿那条 `assert!` 红。两个方向只读挂载都挂上。
  - 任务 20436 退 0：「ATTACK_TWO_REFUSED core=Err(RootRingSlotsPerRegionOutOfRange { device: DeviceIdentity(0), out_of_range: RootRingSlotsPerRegionOutOfRange { declared_slots_per_region: 17, minimum: 4, maximum: 16 } })」，池级 checker 四槽都报 `UnitGeometryFieldDisagrees(UnitBytesNotTheFormatConstant)`。
- 主工作区复现已知红测试，任务 20424 退 101：「left: (2, 0)」「right: (0, 0)」，在 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:3798`。
- 列项：任务 20421 退 0，八行新必红与三个新 proof 项都列得出；任务 20422 退 0（第 4 次，前 3 次撞内存上限），以「C1040：」「C1041：」起头的项 17 条，「锚点失效 0 行」。
- 核不动：攻方 3.2「记录 512、换上探测 4096 的盘，会话发布写了才拒」没建用例（要先有一版带文件的现行版本），是推的；攻方 1.4 `mount_transparency.rs:2423` 那一读与 core 不同判，现有流里四格都是常量、走不到，没建用例；GPU 判器两个内核两条腿都没读。

## 二、判决

打中、现查坐实的：

1. **只读挂载不比探测值。** D12（目标介质） 已定项 4 比对表第一行：「| `physical_block_size` | **拒绝挂载**（可写与只读都拒） |」（`.claude/kb/decisions/12-目标介质.md:75`），用户已定「只读也判」（`records/2026-11-17-里程碑四收口续做.md` 第 31 行）。`grep -n 'probe_physical_block_size' crates/singlefs-core/src/mounted_read.rs crates/singlefs-core/src/recovery.rs` 零命中；任务 20435 两个方向只读挂载都挂上。只读挂载只拿 `&dyn PoolReader`，`PoolReader` 没有探测接口（实现员批 2 报告「停下交主 agent 的设计问题」第 1 条）。
2. **kb 状态句与代码不符。** `.claude/kb/decisions/12-目标介质.md:69`「探测到的 physical_block_size 已与记录比（可写与只读都拒；mkfs 时参数与探测不同也拒）」与第 103 行「代码已做这一比（可写与只读都拒；mkfs 时参数与探测不同也拒）」，只读那一半不成立。
3. **注释引错条款。** `crates/singlefs-core/src/mount.rs:764` 把 D12 已定项 4 第一行写成「对不上 ⇒ 拒绝可写挂载」，第 75 行写的是「拒绝挂载（可写与只读都拒）」，「拒绝可写挂载」是第 76 行设备容量那一行的；`crates/singlefs-core/src/make_filesystem.rs:216` 写「可写与只读挂载逐盘比探测值都会拒」，只读那一半与第 1 条不符。
4. **同一个量两种类型。** `crates/singlefs-core/src/make_filesystem.rs:220`–`:221`：`probed_physical_block_size: PhysicalBlockSizeInBytes` 与 `physical_block_size_in_the_parameters: u32`，`:615` 比的时候拆 newtype；`mount.rs` 的新成员同形（攻方引 `:769`、`:7628`）。根子是 `crates/singlefs-core/src/system_configuration.rs:299` `pub physical_block_size: u32` 本来就是裸整数，这一批新加的两个错误成员没有用 newtype。检查单「类型与名字」不成立。
5. **会话发布用户改动的入口不比探测值，推抬 F 那一支比。** `crates/singlefs-core/src/mounted_session.rs:2964` 起只比盘表身份，`:2984`、`:2994` 起核可见与本盘设备号，再核落后支，都不探测；推抬 F 时经 `push_one_floor_raise_within_the_admission_budget`（攻方引 `:6328`）走 `caller_inputs_agreeing_with_the_disk`，那里含探测（任务 20434 第 1、2 次探测的调用链）。`mounted_session.rs:2928`–`:2929` 的文档写循环里推的每一串「再核一遍三样」，C1040 之后那一核是四样。哪一边对，条款没写：D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」不含探测值，D12 已定项 4 管的是挂载时。交用户（「交用户的」第 1 条）。
6. **已知红测试判「测试判法要换」。** 两条腿一致；任务 20434 坐实两次探测来自推抬 F 那一支的核，换成 `(盘数, 0)`、`(2 × 盘数, 0)` 两条期望整条走通。附条件：第 5 条用户定「会话里只在挂载时比」，这一条反过来（推抬 F 那一支去掉探测，测试照原期望）。
7. **证明漏两行必红。** 两条腿一致：`crates/singlefs-replay-prefix/src/file_range_proofs.rs:171` 与 `:221` 两处 None 支新加的 `range_intersection_length(...) == 0` 没有自己的必红行；它们与紧挨的 `forall` 等价，去掉 `forall` 不让这一格红。规格、两条组合引理、见证都成立：后置条件没有空真，两条引理每半边前提单删都有反例（正推模型 20423 量过），八行新必红按落点各红在点名的那一格（列项 20421；落点是推的，Verus 这一轮没复跑，20208 是用户那一趟的结局）。

成立、没打中的：

- C1041 四格：偏移、值与判法在 core（`recovery.rs:1421`、`:1533`）、池级 checker（`image.rs:1066`、`:1233`）、验证包两处（`verification_read.rs:94`、`memory_pool.rs:758`）与字段表一致；整池拒的结局两边相同。一槽两格同错时点名的格不同（任务 20436），I-7.13 对根环起点那一格已有「照两边各自的判定次序钉」的先例（`crates/singlefs-checker/invariants/02-根环.md` 第 20 行），不判缺陷，这一种组合补一条钉住两边各自点名的用例。
- C1040 可写挂载：`device_table_disagreeing_with` 的两处调用点（`mount.rs:981` 的 `of_a_pool_on_disk` 与参数核那一处）罩着可写挂载与挂着之后收盘表的入口，都在第一个写之前。mkfs：`make_filesystem.rs:649` 的判在第一次写之前（两腿一致）。
- 验证包两处直接调池级 checker 的 `unit_geometry_field_that_disagrees`：两处都在审计一侧，与 core 不共用代码，不算「审计与被审计用同一段代码」（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」）。代价是验证包与池级 checker 用同一个算法，记一笔，不改。
- 检查单「走不到的理由二次查实」：报告「没有新加 `assert!`」按字面成立；`system_configuration.rs:677` 起新加 4 处 `writer.assert_position`，只在改代码、字段表偏移挪动时触发，任何输入都不触发（两腿一致），成立。「不兼容旧代码与旧格式」「不新建判定库」「所有任务多机可用」「测试优先多线程」成立。「带 --item 的命令与项」：17 行与 11 个新项都列得出（20421、20422），成立；已知红测试那个目标没被合入后验证跑到，是 `scripts/gate/merge-and-verify.sh:80` 只按补丁改到的路径挑目标、挑不出被行为改变牵连的目标（攻方读的），另立一笔，不归这一批代码。「新崩溃流多罩了什么」：这一批没有新流；写系统配置槽的只有 `to_slot`、产品代码里没有第二个写者（两腿 grep 一致），探测值与参数不同的盘上做 mkfs 或可写挂载的流 grep 范围里没有、全量没追（推不出，按打中办：修补批里实现员列全造盘时探测值不取参数的装置）。

推不出、交用户的：D22（单元原子性怎么合成） 已定项 2「mkfs 按池内 `max(physical_block_size)` 划」与代码「参数与每块盘探测值都相等」在异构盘上不一致（第一版两盘同构，两腿都推不出谁管），见「交用户的」第 2 条。

有打中，照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」改完再攻一轮，第二轮派辩方腿复核这份判决。

## 三、改法

修补批（一个实现员，以 (5b) 借卡那一批落主工作区之后的样子为基线；那条已知红测试并进同一批）：

1. 只读挂载择完系统配置逐盘比探测值，不等就拒，点名盘、探测值、记录值：`PoolReader`（`crates/singlefs-core/src/recovery.rs:129`）加必须实现的探测方法（批 2 报告的加法甲，编译器逼每个读者表态，照 `.claude/singlefs-ai-sop/rules/code-discipline.md`「类型：让非法状态写不出来」；带默认体交 `None` 的加法乙让包装读者静默不判，不取）；`MemoryPool`、`CrashImage` 记每块盘 mkfs 时的块宽。`crates/singlefs-verification/src/judging_units_entries.rs` 的登记与管它的会话协调。验收：记录 512 探测 4096、记录 4096 探测 512 各一条只读挂载被拒，各配一行变异。零轮。
2. 两个新错误成员里参数值与记录值改成 `PhysicalBlockSizeInBytes`。零轮。
3. `mount.rs:764` 与 `make_filesystem.rs:216` 的注释照第 75 行与改后的实现改。
4. 已知红测试：等「交用户的」第 1 条定了再改，两种结局各写在第二节第 6 条。
5. 一槽 S 越界与四格同错时两边各自点名的格，补一条钉住的用例（攻方任务 20436 那份草稿可作起点）。量过（20436）。
6. 证明补两行必红：`file_range_proofs.rs:171`、`:221` 两处 `== 0` 各改成 `== 1` 一行，红在 `bytes_in_unit` 与 `checked_bytes_in_indexed_unit`。这份是用户的文件，交用户定由谁补（「交用户的」第 3 条）。零轮。
7. kb：D12 第 69、103 行状态句改成只读那一半没做、欠在 C1040，与第 1 条实现一起改回。

另立、不进这一批：`scripts/gate/merge-and-verify.sh` 挑测试目标只按补丁改到的路径，挑不出被行为改变牵连的目标；登在 `records/2026-11-17-subagent账本自上而下清欠.md` 第二百七十四节，随当周治理那一批评估。

## 交用户的

1. 挂着之后会话发布用户改动的入口要不要也比探测值。甲：每次用户改动入口都比（入口补上，红测试期望改成 `(盘数, 0)`、`(2 × 盘数, 0)`）；乙：只在挂载时与收盘表的那几个入口比，会话里认定盘表不换（推抬 F 那一支去掉探测，红测试照原期望，`mounted_session.rs:2928` 的文档对上）。依据：第二节第 5、6 条；攻方推甲下「记录 512、换上探测 4096 的盘」在会话里写了才拒（推的，没量过）。
2. D22 已定项 2「mkfs 按池内 `max(physical_block_size)` 划」与实现「参数与每块盘探测值都相等，否则拒」在异构盘上不一致。第一版两盘同构、今天显不出；条款改成「都相等」，还是留到加第二套布局时再定。
3. 证明那两行必红（`file_range_proofs.rs:171`、`:221`）由谁补：并进主 agent 的修补批，还是用户那一路自己补。

## 这一轮被判过的文件

- crates/singlefs-checker/src/image.rs：判了 `unit_geometry_field_that_disagrees` 与 `read_system_configuration_slot` 里四格那一判的偏移、值与位置，与 core 同结局；一槽 S 与四格同错时点名单元大小（任务 20436），与 core 不同，判不是缺陷、补钉住的用例。
- crates/singlefs-core/src/make_filesystem.rs：判了 mkfs 参数与探测值那一判在第一次写之前（成立）；新错误成员参数值用裸 `u32`（打中，改法第 2 条）；第 216 行注释说只读挂载会拒（打中，改法第 3 条）。
- crates/singlefs-core/src/mount.rs：判了 `physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration` 罩住可写挂载与收盘表的入口（成立）；第 764 行注释引错 D12 已定项 4（打中）；新成员记录值用裸 `u32`（打中）；推抬 F 那一支经 `caller_inputs_agreeing_with_the_disk` 探测，交用户第 1 条。
- crates/singlefs-core/src/recovery.rs：判了 `unit_geometry_fields_of_the_format` 的次序与四个成员（成立）；`PoolReader` 没有探测接口，只读挂载那一比落不下（打中，改法第 1 条）。
- crates/singlefs-core/src/system_configuration.rs：判了四格偏移常量、`to_slot` 新加的 4 处 `assert_position` 任何输入都不触发（成立）、`unit_geometry_fields_recorded_in_the_slot` 只切字节。
- crates/singlefs-verification/src/memory_pool.rs：判了 `.filter` 只让候选变大、不放宽任何槽（成立），直接调池级 checker 函数在审计一侧（成立）；`MemoryPool` 不记块宽，改法第 1 条要补。
- crates/singlefs-verification/src/verification_read.rs：判了 `parse_system_configuration` 四格一判与池级 checker 同判（成立）；经 `mount_transparency.rs:2423` 那一读与 core 不同判，现有流走不到（攻方 1.4，记一笔）。
- crates/singlefs-verification/tests/core_review_geometry_back_chain_and_empty_inode.rs：判了造盘探测块宽取参数那一处改法与三条几何用例的期望不变（成立）。
- crates/singlefs-verification/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs：判了穷举 `match` 新成员并进 panic 那一臂（成立）。
- crates/singlefs-verification/tests/make_filesystem_refuses_a_physical_block_size_parameter_other_than_the_probed_one_before_any_write.rs：判了三条用例核录制流 0 步、两盘逐字节不变与阳性对照（成立）。
- crates/singlefs-verification/tests/memory_pool_scan_candidates_do_not_take_the_geometry_of_a_slot_zero_carrying_a_refused_unit_geometry_field.rs：判了四格各改一份、阳性对照先核候选为空（成立）。
- crates/singlefs-verification/tests/system_configuration_unit_geometry_fields_are_refused_by_core_and_the_pool_checker.rs：判了四格各一条、core 两条挂载路与池级 checker 同判（成立）；一槽两格同错那一种组合补在改法第 5 条。
- crates/singlefs-verification/tests/writable_mount_refuses_a_device_whose_probed_physical_block_size_differs_from_the_system_configuration.rs：判了两个方向各一条与阳性对照（成立）；只钉了 `mount_writable` 一个入口（正推读的），改法第 1 条的只读两条照这份写。
- crates/singlefs-replay-prefix/src/file_range_proofs.rs：判了 `range_intersection_length` 与两条组合引理、见证成立，后置条件不空真；两处 None 支 `== 0` 没有专属必红行（打中，改法第 6 条）。
- crates/singlefs-replay-prefix/src/README.md：判了新加那一段的射程与证明一致（只涉字节区间与返回长度）。
- crates/singlefs-replay-prefix/verus-must-red-worlds.tsv：判了八行新必红的落点与三行 `proof_errors` 改 2（成立），少两行（改法第 6 条）。
- crates/mutations.tsv：判了 C1040、C1041 追加的 17 行（快照 `mutations-c1040-c1041-rows.tsv`），列项 20422 得 17 条、锚点失效 0 行，变异 19266「抓到 17 条、没红 0 条」。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D12（目标介质） 已定项 4 | 不影响 | 2026-11-18 不受影响：条款不动；第 69 行实现边界状态句只读那一半与代码不符，随修补批改 |
| D12（目标介质） 已定项 5 | 不影响 | 2026-11-18 不受影响：条款不动；第 103 行状态句同上 |
| D18（块里携带什么信息） 已定项 11 | 不影响 | 2026-11-18 不受影响：会话入口要不要另比探测值交用户（交用户第 1 条），定了再回看 |
| D22（单元原子性怎么合成） 已定项 2 | 不影响 | 2026-11-18 不受影响：「按池内 max 划」与实现在异构盘上的不一致交用户（交用户第 2 条），第一版两盘同构 |
