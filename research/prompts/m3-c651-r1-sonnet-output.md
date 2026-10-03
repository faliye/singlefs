# m3-c651-r1 云端正推腿报告（Sonnet，三方论证第一轮，2026-10-02）

腿：`three-way-forward`；格：V1–V3（正文第三节），分工表「云端正推」一行。代码按冻结副本 `/tmp/claude-1000/m3-c651-r1/tree/`（行号取自副本，`crates/` 路径下的引文用 `--root` 指到副本核）；
kb 行号取自仓里 kb 文件自己，不是背景材料。没读攻方腿与 m3-formal-r5 的任何产出。
模型与探针：`research/prompts/m3-c651-r1-sonnet-model/`（README 写了跑法；探针在副本的拷贝里编译与跑，没动冻结副本）。

## 〇、前提复核（实现今天的样子，对着副本逐条现查）

背景材料第二节的几条主 agent 观测，逐条核：

- checker 的槽 1 偏移 ③ 取 4096：`crates/singlefs-checker/src/image.rs:589` 「.unwrap_or(FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES);」，读槽 1 那一句 `crates/singlefs-checker/src/image.rs:590` 「let slot_one = read_system_configuration_slot(reader, device, spacing);」。一致。
- 实现的 ① ② 与 checker 相同：`crates/singlefs-core/src/recovery.rs:1084` 「.or(slot_spacing_recorded_by_the_first_mountable_slot_zero)」；③ 逐档：`crates/singlefs-core/src/recovery.rs:1090` 「None => slot_one_found_by_trying_every_slot_spacing_the_format_allows(reader, device)?,」。一致。
- 逐档的档范围与档数：`crates/singlefs-core/src/recovery.rs:972` 「for candidate_spacing in (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES..=largest_spacing)」，上界 `crates/singlefs-core/src/recovery.rs:715` 「region_start(0).0 - SYSTEM_CONFIGURATION_SLOT_BYTES」，档数 `crates/singlefs-core/src/recovery.rs:951` 「上下界之间 (1 MiB − 4096 − 4096) ÷ 512 + 1 = 2033 档，每档读一次。」。我用命令数了一遍：

```
$ python3 -c "print((1048576-4096-4096)//512+1)"
2033
```

- 「只收自证过、自述槽距正好等于所在偏移」：`crates/singlefs-core/src/recovery.rs:986` 「if recorded_spacing != candidate_spacing {」；「几档都收得到时取世代号最大的、相同取偏移小的」：`crates/singlefs-core/src/recovery.rs:989` 「if highest_generation_found.is_none_or(|found| {」（后面是严格大于，档按偏移升序走，同号时先收到的、即偏移小的留下）。一致。
- 槽距上界的「第一版常量」那半句在 D22 里就有：`.claude/kb/decisions/22-单元原子性怎么合成.md:333` 「逐档找槽 1 那一处还没有哪一槽可读，仍用第一版常量」。一致。
- 背景材料第二节说「io_min 大于 4096 的设备今天是否存在于任何测试里，由腿自己数」。数法与原样输出（副本里）：

```
$ cd /tmp/claude-1000/m3-c651-r1/tree && grep -rn 'io_min' crates/singlefs-harness/tests/ | wc -l
38
```

  38 行里造出槽距 8192 的池并走到「槽 0 弄坏再找槽 1」的有三处：`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:497` 「fn slot_one_is_found_by_trying_every_spacing_when_no_slot_zero_in_the_pool_verifies() {」（只判 core，两盘槽 0 都坏）、
  `crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs:836` 「fn slot_one_is_looked_for_at_the_slot_spacing_another_devices_slot_zero_records() {」（只判 core，一盘槽 0 坏）、
  `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs:1222` 「fn a_device_whose_slot_zero_does_not_verify_has_its_slot_one_looked_for_at_the_spacing_another_device_records(」（只判 checker，一盘槽 0 坏）。
  没有一条用例在「全池槽 0 都坏、槽距 ≠ 4096」上同时判 core 与 checker：core 那份文件不引 checker（`grep -rln singlefs_checker` 对它零命中）。
- 背景材料没写、而这一格要连同改的读法还有两份（今天各自照写了一遍）：层 0 GPU 判器 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:671` 「fn slot_one_by_trying_every_spacing(device: u32) -> SysConfig {」；它同号时的比较在 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:686` 「if (!have_best || lt64(best.generation, s.generation)) {」，是同一个逐档读法；
  O3 执行器的 Python 读法，是第三种：`research/e7-index-bench/src/bin/e170_o3_spec_executor/system_configuration.py:91` 「# 槽 0 自证不过：槽 1 在哪 kb 没写（G03）。暂代读法：从 4096 起按扇区一档往上找到 1 MiB − 4096，」，只看本盘、没有 ②，收到第一个就停（`research/e7-index-bench/src/bin/e170_o3_spec_executor/system_configuration.py:102` 「slots.append(candidate)」后面是 `break`），不比世代号。

什么现象会推翻这一节：副本里 `crates/` 的行不是上面这些；或 `grep -rn 'io_min'` 在冻结清单对应的树上数出别的数。

## 一、V1　槽距从哪来

### 1.1 条款原文（每句我在被引文件里 `grep -nF` 过一次；长行引的是整句，不是整行）

- D22 已定项 16 第一段：`.claude/kb/decisions/22-单元原子性怎么合成.md:331` 「系统配置槽：每盘恒 2 个槽，槽 i 的设备内偏移 = i × 固定结构槽距（D2（RAID 条带策略） 已定项 19），槽世代号从 1 起（mkfs 写下的两槽世代号都是 1）、每写一次 +1，下一次写的槽 = 世代号 mod 2，择槽取校验和过且世代号最大的（逐盘计：取号、发布末尾的轮换、取号全或无失败时的回卷三种系统配置写都写这块盘上自证过的槽里最大的世代号 + 1）。」
- D22 已定项 16 第 1 句（槽距上界）：`.claude/kb/decisions/22-单元原子性怎么合成.md:333` 「判固定结构槽距的上界用同一槽自述的根环起点」。
- D22 已定项 16 第 5 句（读者判槽距，越界即拒）：`.claude/kb/decisions/22-单元原子性怎么合成.md:337` 「读者判固定结构槽距：≥ 4096，且槽 1 整槽落在同一槽自述的根环起点之前；`physical_block_size` ∈ [根记录宽 457, 槽距]。」
- D2 已定项 19：`.claude/kb/decisions/02-RAID条带策略.md:313` 「**定案**：槽距 = **4096 向上取整到 mkfs 时探测的 `io_min` 的整数倍**」；同行的两道挂载闸 `.claude/kb/decisions/02-RAID条带策略.md:313` 「**(a) 实际 `io_min` ≤ 记录的槽距**」；射程里「还没实现」：`.claude/kb/decisions/02-RAID条带策略.md:315` 「挂载时拿实际 `io_min` 与记录的槽距比对那一步还没实现」。
- D22 已定项 9（池级字段）：`.claude/kb/decisions/22-单元原子性怎么合成.md:210` 「内容是**池级**的，每盘存一份副本，两槽同内容；唯一的盘级字段是「本盘设备号 4」，不是每块盘能配不同的值。」
- D22 已定项 8（容错承诺）：`.claude/kb/decisions/22-单元原子性怎么合成.md:178` 「容错承诺是单故障：每盘一槽系统配置坏在承诺里；」；同一项第 1 条 `.claude/kb/decisions/22-单元原子性怎么合成.md:174` 「系统配置每盘放一份。」
- D23 已定项 3：`.claude/kb/decisions/23-journal的角色与格式.md:125` 「**tail 语义上是派生态**」；`.claude/kb/decisions/23-journal的角色与格式.md:127` 「恢复不许先信 tail 再往前走：必须先全环扫描、逐条验证，再择最长合法前缀。」
- I-8.5：`.claude/kb/invariants.md:94` 「池里至少有一块盘的两个系统配置槽里有一个槽自证过（magic 与整槽校验和对）：tail 住这些槽」，同行 `.claude/kb/invariants.md:94` 「这个集合就是全部 tail 见证者」「全池皆废判红；单盘全废不判违例」「对着只读恢复判」。
- 现有「槽 1 在哪」的文字只有描述：`.claude/kb/layout/01-first-txn.md:160` 「槽距住在槽自己里，读槽 1 之前要先有一个读得出槽距的槽 0。」与 `.claude/kb/layout/01-first-txn.md:160` 「③ 这一格 checker 与实现的读法不同，哪个为准没有条款定。」；C651 行 `.claude/kb/checks-owed.md:520` 「两种读法哪个为准没有条款定」。
  ① ② 两步（本盘槽 0 的槽距、别盘槽 0 的槽距）也只在 layout:160 的描述与 `crates/singlefs-checker/src/image.rs` 的注释里，D22 已定项 16 的条款句里没有——这一句要写的是整条链 ① ② ③，不只 ③。

核「①② 两步在 kb 条款里没有」：

```
$ cd .claude/kb && grep -rn '第一块槽 0\|别的盘槽 0\|槽 0 可用\|槽 0 可择\|池里别的盘' --include=*.md . | cut -c1-60
layout/01-first-txn.md:160:**系统配置槽 0 与槽 1 的位置**：槽宽都是 4096；槽 0 在设备内偏移 0，槽 1 在设
$ grep -n '槽 0 自证不过\|槽 0 都' decisions/22-单元原子性怎么合成.md decisions/02-RAID条带策略.md | wc -l
0
```

### 1.2 三个候选各自与条款句的相容性

候选的定义照正文第三节：V1-常量 = checker 今天的 ③；V1-逐档 = 实现今天的 ③；V1-拒 = 全池槽 0 都不可用就整池拒挂载（今天 `crates/` 里没有）。
另有一个我想到、`crates/` 里做不到的候选 V1-探测（挂载时探得的 io_min 按 D2 已定项 19 的式子算槽距）：`crates/singlefs-core/src/recovery.rs:1081` 「的式子拿挂载时探得的 io_min 算，要 `PoolReader` 交得出探测值，今天交不出。」；
而且 D2 已定项 19 的闸 (a) 是实际 io_min ≤ 记录的槽距（不是相等），实际 io_min 比 mkfs 时小的设备上按式子算出的槽距会小于记录的，所以它最多是逐档的下界，不是答案。不单列一行。

| 候选 | D22 已定项 16 的「槽 i 的设备内偏移 = i × 固定结构槽距」（`.claude/kb/decisions/22-单元原子性怎么合成.md:331`） | D22 已定项 16 第 5 句的槽距范围 | D2 已定项 19 | D23 已定项 3 / I-8.5 | D22 已定项 8 的容错承诺（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`） |
|---|---|---|---|---|---|
| V1-常量 | 槽距恰为 4096 的池相容；槽距 > 4096 的池读的是 4096，不是 1 × 自述槽距，**冲突** | 相容（4096 在范围里） | 与「槽距 = 4096 向上取整到 io_min 的整数倍」冲突：D2 19 允许 > 4096，常量把它们一律当 4096 | 槽距 > 4096 的池：槽 1 自证过而 I-8.5 判红，**冲突**（实测，见 1.3 的 S2） | 槽距 4096 的池覆盖；槽距 > 4096 的池不覆盖 |
| V1-逐档 | 相容：只收自述槽距 == 所在偏移的槽，正是「偏移 = 1 × 自述槽距」 | 相容：档上界 = 第一版根环起点 − 4096，D22 已写「逐档找槽 1 那一处还没有哪一槽可读，仍用第一版常量」；根环起点 ≠ 64 的槽整池拒，所以上界不会放宽 | 相容：下界 = 公式下限 4096，步长 512 对 mkfs 产出的池完备（见下） | 相容：槽 1 自证过即进见证者集合（实测 S2 core 择到） | 相容 |
| V1-拒 | 条款没写「拒」；但「每盘恒 2 个槽 … 择槽取校验和过的」下，一个自证过的槽 1 被整个忽略 | 相容（不读任何偏移） | 相容 | I-8.5 判的是字节（`.claude/kb/invariants.md:94` 「magic 与整槽校验和对」），与槽在哪个偏移无关；槽 1 自证过而判「全池皆废」，**冲突** | **冲突**：全池每盘槽 0 都坏就是「每盘一槽系统配置坏」，在承诺里；拒挂载把承诺内的状态变成不可挂载 |

步长 512 对 mkfs 产出的池完备，这条是推的，依据：mkfs 判每个根槽与系统配置槽的偏移是判定宽度 max(physical_block_size, io_min) 的整数倍（`crates/singlefs-core/src/make_filesystem.rs:343` 「.find(|(_, offset)| !offset.0.is_multiple_of(decision_width_in_bytes))」），
区域 0 槽 0 在 1 MiB（`crates/singlefs-core/src/root_ring.rs:96` 「ROOT_RING_BASE_SLOT * SLOT_BYTES + region * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES,」；用例也这样写 `crates/singlefs-harness/tests/mkfs_refuses_geometries_whose_root_slots_do_not_align.rs:135` 「"pbs 512 + io_min 3072：该报区域 0 槽 0 在 1 MiB 处不是 3072 的整数倍，实际 {:?}",」），
所以判定宽度整除 2^20、是 2 的幂；physical_block_size ≥ 根记录宽 457（`crates/singlefs-format/src/lib.rs:195` 「pub const ROOT_RECORD_BYTES: u64 = 457;」）⇒ 判定宽度 ≥ 512；槽 1 偏移 = 槽距是判定宽度的整数倍 ⇒ 是 512 的整数倍。
读者收的槽距范围并不要求 512 的倍数（`crates/singlefs-core/src/recovery.rs:761` 一带只判 ≥ 4096 与上界），所以一个槽距写成 4100 的池读者收、逐档找不到；mkfs 产不出它（我没造这个池，只读了代码，推的）。

### 1.3 状态 × 候选的结局表

今天的 checker 是 V1-常量、今天的 core 是 V1-逐档，所以两列「实测」是在同一份镜像上两边各自的输出（探针 `research/prompts/m3-c651-r1-sonnet-model/c651_probe.rs`，输出 `research/prompts/m3-c651-r1-sonnet-model/c651_probe.out`，副本拷贝里经 `run-with-memory-cap.sh` 与 `capped.sh 8` 跑，退出码 0，`test result: ok. 1 passed`）。V1-拒今天没有实现，那一列是推的。
「槽 0 弄坏」= 翻槽 0 里偏移 2048 的一个字节，整槽校验和不过；「残留」= 同一块盘上更早一次 mkfs 留下的槽 1（mkfs 只写本次的两槽，见 1.4）。

| # | 状态（两盘池） | V1-常量（checker 实测） | V1-逐档（core 实测） | V1-拒（推） |
|---|---|---|---|---|
| S1 | 槽距 4096，两盘槽 0 都坏，槽 1 好 | 挂：可择槽 盘0 盘1 各 1 个，I-8.5 成立 | 挂：`Ok(盘0 槽距4096 世代1)` | 拒 |
| S2 | 槽距 8192（io_min 8192），两盘槽 0 都坏，槽 1（在 8192）好 | **无可择槽**，旁路报告「盘 0、盘 1 都没有自证过的槽」，**I-8.5 判红**「池里 2 块盘的系统配置槽一个都没自证过：tail 没有见证者」 | 挂：`Ok(盘0 槽距8192 世代1)` | 拒 |
| S3 | 槽距 8192，只盘 0 槽 0 坏 | 挂（② 借盘 1 的槽距）：盘 0 一个可择槽、盘 1 两个，I-8.5 成立 | 挂：`Ok(盘0 槽距8192 世代1)` | 挂（有可用槽 0，不进「全池没有」那一格） |
| S4 | 槽距 8192，两盘槽 0 都坏，4096 处各有一份旧池残留（槽距 4096、别的 fsid、世代 1），8192 处新池槽 1 好 | 择到旧池：可择槽「槽距 4096 世代 1」 | 择到旧池：`Ok(盘0 槽距4096 世代1 fsid"singlefs-old-poo")`（同号取偏移小） | 拒 |
| S5 | 同 S4，但盘 1 的槽 1 也坏、残留只在盘 1 | 盘 0 无槽，盘 1 择残留；I-8.5 成立 | `Err(SystemConfigurationsDisagree)` | 拒 |
| S6 | 槽距 4096，两盘槽 0 都坏，8192 处各有一份旧池残留（槽距 8192、别的 fsid、世代 9），4096 处新池槽 1 好 | 择到新池（读的是 4096，碰巧对） | 择到旧池：`Ok(盘0 槽距8192 世代9 fsid"singlefs-old-poo")` | 拒 |
| S7 | 两盘的两个槽都坏 | 拒（推：`NoValidSystemConfiguration`） | 拒（推，同） | 拒 |
| S8 | 同 S4，但残留与新池同 fsid（同一个 fsid 重做 mkfs）、世代 1（`research/prompts/m3-c651-r1-sonnet-model/c651_probe_c.out`） | 择残留：可择槽「槽距 4096 世代 1」，I-8.5 成立 | 择残留，**恢复照常成功**：`Ok(盘0 槽距4096 世代1 fsid"singlefs-new-poo")`，`recover` 报 `NoFile { root: (InstanceGeneration(0), CheckpointTxg(0)) }`，用的是槽距 4096 的几何，真池是 8192 | 拒 |

S2 那一行的「core 照常挂、checker 判不出」就是 C651 说的两边读法不同；S1 说明同一句话在槽距 4096 的池上两边没有差别。
再有一件事要摆在桌面上：E170 的两条层 0 流（槽距都是 4096）的取样态里有 S1 这一形。我用执行器的建镜像模块把批 1 每个取样态的槽 0 与 4096 处槽 1 数了一遍（`research/prompts/m3-c651-r1-sonnet-model/count_slot0_b.py`，原样输出在 `research/prompts/m3-c651-r1-sonnet-model/count_slot0_b.out`），摘与 G03 有关的几行：

```
new_pool_file_creation devices [0, 1] states 82
  batch=1 槽0自证过的盘数=0/2 槽1(4096)自证过的盘数=2/2 态数=2
  batch=1 槽0自证过的盘数=1/2 槽1(4096)自证过的盘数=2/2 态数=8
fixed_script_through_e_then_unmount devices [0, 1] states 890
  batch=1 槽0自证过的盘数=0/2 槽1(4096)自证过的盘数=2/2 态数=12
  batch=1 槽0自证过的盘数=1/2 槽1(4096)自证过的盘数=2/2 态数=48
```

2 + 8 = 10、12 + 48 = 60，与 E170 页写的 G03 命中数对得上：`.claude/kb/experiments/170-O3规约执行器第一版.md:122` 「name=gap_hits batch=1 stream=new_pool_file_creation G01=53 G02=53 G03=10」、`.claude/kb/experiments/170-O3规约执行器第一版.md:123` 「name=gap_hits batch=1 stream=fixed_script_through_e_then_unmount G01=397 G02=397 G03=60」。
所以 G03 撞的 70 态里 14 态（2 + 12）是 S1 形（每盘槽 0 都不自证、槽 1 两盘都好），56 态是一盘槽 0 坏。V1-拒对这 14 个崩溃态（不是介质损坏，是批 1 取样出的崩溃点）给「整池拒挂载」。这是实测的计数；这 14 态在 V1-拒下该算什么，条款没说。
（这份计数用的是执行器的 `slot_is_self_verified`，它不核 magic（G01），所以数的是「整槽校验和过」，与 I-8.5 的「magic 与整槽校验和对」略宽，核 magic 只会让「自证过」的态更少、不会更多——这一句是推的。）

### 1.4 逐档的「自述等于所在偏移」挡不挡旧残留（正文 V1 点名的问题）

答：只挡一半。它挡的是「残留槽自述的槽距 ≠ 它所在偏移」：`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:544` 「fn a_self_verifying_slot_whose_recorded_spacing_is_not_its_own_offset_is_not_taken() {」钉的就是这一半。
不挡「残留槽自述的槽距 == 它所在偏移」，而这一种正是同一块盘上换槽距重做 mkfs 留下的：旧池槽距 4096，它的槽 1 在 4096、自述 4096；新池槽距 8192，mkfs 不清 4096 处。S4、S6 两行是实测。
择到残留之后恢复走到哪（`research/prompts/m3-c651-r1-sonnet-model/c651_probe_b.out`，同样的镜像加一行 `recover`）：S4、S6 报 `Failed { root: None, failure: NoValidRoot }`（残留池的根不在，拒；错的只是错误名，没挂错）；S5 报 `Failed { root: None, failure: SystemConfigurationsDisagree }`。
S8（残留与新池同 fsid）不同：`recover` 照常成功、用槽距 4096 的几何去读一个槽距 8192 的池，checker 同样把它当可择槽、I-8.5 成立。这是 `crates/singlefs-core/src/recovery.rs:955` 注释里「按旧池的几何挂」的实测，且 V1-常量与 V1-逐档都躲不开；之后可写挂载按错的槽距写会怎样我没量（可写挂载那条路径我没走）。
残留的来源有一句现成的原话：`crates/singlefs-core/src/recovery.rs:953` 「风险：同一块盘上更早一次 mkfs 用别的槽距写下的槽 1，这一次 mkfs 不清（它只写本次的两槽）」，
mkfs 的写清单里我没找到哪一句要写 [4096, 槽距) 这一段，写零的只有根环三个区域与 journal 环：`.claude/kb/decisions/22-单元原子性怎么合成.md:176` 「mkfs 要把第 0 代根种进全部根环区域，并把三个根环区域整段写 0」。
三个候选在残留上的排名：V1-拒不会择错（它什么都不择）；V1-常量与 V1-逐档各在一个方向上择错（S4 两个都错；S6 只有逐档错）。**没有哪个候选单靠「读哪个偏移」就能防残留**，防残留要的是别的信息（fsid 对账、mkfs 清残留区），那是 V2 与「零轮改法」的事（下面 1.6）。

另外三处读代码读到的、没实测的：

- 逐档循环里 `mountable_slot_or_refusal(...)?` 在偏移比对之前：`crates/singlefs-core/src/recovery.rs:975` 「match mountable_slot_or_refusal(」，比对在 `crates/singlefs-core/src/recovery.rs:986` 「if recorded_spacing != candidate_spacing {」。
  一个自证过、池级字段越界（格式版本、根环起点等）的残留槽落在某一档上，会在比对之前就把整池拒掉（`SystemConfigurationValueRefused`，整池拒是 D22 已定项 16 第 5 句的「越界即拒」）。我没造这个镜像，读代码推的。
- 逐档是按盘各扫各的：`crates/singlefs-core/src/recovery.rs:1090` 的调用点在逐盘循环里，S5 实测过一块盘择到残留、另一块择到别的，随后 `crates/singlefs-core/src/recovery.rs:1120` 「return Err(RecoveryFailure::SystemConfigurationsDisagree);」整池拒。D22 已定项 9 说槽距与 fsid 是池级字段，扫到一块盘之后别的盘不借它的槽距（② 只借槽 0 的）。
- 逐档的上界是第一版常量：`crates/singlefs-core/src/recovery.rs:950` 「槽 1 整槽落在根环基址（[`region_start`] 的区域 0，1 MiB）之前」，这一点与 D22 已定项 16 同一句里的「逐档找槽 1 那一处还没有哪一槽可读，仍用第一版常量」一致，不是缺口。

### 1.5 按正文第五节的判据写 V1 的结论

判据 1：只有一个候选与条款文字相容 ⇒ 写成条款候选句。按 1.2 的表：V1-常量在槽距 > 4096 的池上与 D22 已定项 16、I-8.5 冲突；V1-拒与 D22 已定项 8 的容错承诺冲突，I-8.5 的字面也与它不合；只有 V1-逐档在三格里都不冲突。
所以按判据 1，V1 只有逐档一个候选与条款文字相容。（判决归主 agent；我这里只给相容性与两处我认为条款没说的缺口：残留、V2。）

候选句（我起草，没被攻过；放进 D22 已定项 16，连 ① ② 一起写，因为 ① ② 今天在 kb 里只有 layout 的描述）：

> 系统配置槽 1 的设备内偏移 = 这一槽自述的固定结构槽距，读者按这个次序找：① 这块盘的槽 0 可择时，取它自述的槽距；② 否则取池里按设备次序第一块槽 0 可择的盘自述的槽距；③ 全池没有一块盘的槽 0 可择时，每块盘在 4096 到第一版根环起点 − 4096 之间每 512 字节一档各读一槽，只收自证过、可择、且自述的槽距正好等于所在偏移的槽；某块盘一档都收不到，按这块盘两槽都无效处置（D22 已定项 8 第 1 条：跳过这块盘）。

对实现的改动（行数是减出来或估的，估的标「推」）：

| 方案 | 改哪一边 | 行数 |
|---|---|---|
| 采逐档（这句） | core：不改（`crates/singlefs-core/src/recovery.rs:962` 起的 56 行已是它）；checker：`crates/singlefs-checker/src/image.rs:589` 处全池没有槽 0 时改调一个新的逐档函数、再把 I-8.5 与旁路报告改成读它 | 推：checker +35 左右；GPU 判器 0（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:671` 起 32 行已是它）；O3 执行器 `research/e7-index-bench/src/bin/e170_o3_spec_executor/system_configuration.py:91` 起 14 行改成 ② + 逐档（推：+10 左右）；kb：D22 已定项 16 加一句、layout:160 的 ③ 段改、C651 还清 |
| 采常量 | core：删 `crates/singlefs-core/src/recovery.rs:940` 到 1017 共 78 行，`crates/singlefs-core/src/recovery.rs:1090` 那一臂改成取 4096，删/改 `core_review_tree_table_duplicates_and_slot_one_search.rs` 里 9 处 `flip_slot_zero_on_every_device` 调用所在用例（`grep -n` 数出）；GPU 判器删 32 行与一处调用；checker 0 | 减出来：core −78、GPU −32，加改测试；D2 已定项 19 要补一句「槽距 > 4096 的池全池槽 0 都坏时不可挂」，且 I-8.5 判红对槽距 > 4096 的池成为条款的一部分 |
| 采拒 | core：`crates/singlefs-core/src/recovery.rs:1090` 那一臂改成返回一个新的 `RecoveryFailure`，删 78 行；GPU 同；checker：`crates/singlefs-checker/src/image.rs:589` 处改成不读槽 1，I-8.5 与旁路报告改读法 | 推：core −78 + 6，GPU −32 + 4，checker +4；D22 已定项 8 的「每盘一槽系统配置坏在承诺里」要加一条例外，I-8.5 的「全池皆废」要改义 |

### 1.6 零轮改法（我提的，没被攻过，只记下）

- mkfs 把 [4096, 槽距) 区间写零（D22 已定项 8 第 3 条，`.claude/kb/decisions/22-单元原子性怎么合成.md:176`，已经对根环与 journal 环这样做，同一个理由）：把 S4、S6 的残留来源掐掉，不碰读法。代价：mkfs 多一段写零，槽距 4096 的池 0 字节。
- 逐档收到的候选与别的盘的 fsid 对账、盘间槽距对账：S5 那种「一盘择残留、一盘择好的」就由整池拒改成取多数/取与别盘一致的。条款没有写；代价大于上一条。
- 一个盘上收到两个以上不同槽距的候选时整池拒（V2 的 V2-歧义即拒，见第二节）。

什么现象会推翻 V1 这一节的结论：
1. 找到一个条款句，在 D22 / D2 / D23 / invariants 里明说「全池槽 0 都不可用时整池拒挂载」或「只认 4096」（我 `grep` 过 D22、D2 的相关词零命中，见 1.1 末的命令；没有对全仓 kb 做过同样的全文搜）；
2. 探针在副本上的 S1–S6 在换一份冻结副本或加了别的提交之后给不同的输出；
3. 有一份已定的、把「全池每盘槽 0 都坏」划到容错承诺之外的条款（D22 已定项 8 现在只把「每盘各坏一槽 + 最新根一时读不出」划出去）。

## 二、V2　几档都收得到时取哪一槽

只对 V1-逐档有意义。

### 2.1 依据是哪一句条款：没有，要新写

- 「世代号最大」那一句是择槽的条款，但它的比较对象是同一块盘上同一个池的两个槽：`.claude/kb/decisions/22-单元原子性怎么合成.md:331` 「择槽取校验和过且世代号最大的（逐盘计：取号、发布末尾的轮换、取号全或无失败时的回卷三种系统配置写都写这块盘上自证过的槽里最大的世代号 + 1）」。
  逐档里「几档都收得到」的几个候选不是同一个池的两个槽（槽距是池级字段，`.claude/kb/decisions/22-单元原子性怎么合成.md:210`；一个池只有一个槽距），世代号在两个池之间没有可比性：世代号只是一个池里「每写一次 +1」的计数（同一行 331）。这一句是我的读法（推）；条款没写比较对象能不能跨槽距。
- 「世代号相同取偏移小」全仓只在描述实现的那一段里出现一次，不是条款：

```
$ grep -rn '世代号相同取偏移小' .claude/kb --include=*.md | wc -l
1
```

  那一处是 `.claude/kb/layout/01-first-txn.md:160` 「几档都收得到时取世代号最大的，世代号相同取偏移小的。」（描述实现；同一行的末句说「哪个为准没有条款定」）。
- 实现里这两条的位置：core `crates/singlefs-core/src/recovery.rs:989` 「if highest_generation_found.is_none_or(|found| {」（严格大于，同号时先到的、偏移小的留下）；GPU 判器 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:686` 「if (!have_best || lt64(best.generation, s.generation)) {」同一个读法；O3 执行器的暂代读法不比世代号、收到第一个（偏移最小）就停（第〇节引过的 `system_configuration.py:102`）。三份读法在「同号」上一致、在「不同号」上执行器与另两份不同。

### 2.2 V2 只在候选 ≥ 2 档时才起作用，而干净状态没有这种事

一个池只有一个槽距，mkfs 本次只写本次的两槽，所以干净状态下逐档最多收到一档（本池的槽 1）。≥ 2 档只在残留（1.4 的 S4、S6、S8）或手造镜像里出现。V2 的全部后果因此都落在残留状态上。

### 2.3 候选与结局（残留状态上，S4/S6/S8 是 1.3 的行，实测）

| 候选 | 句子 | S4（旧池世代 1） | S6（旧池世代 9） | S8（同 fsid） | 干净状态 |
|---|---|---|---|---|---|
| V2-今天 | 世代号最大，同号取偏移小 | 择旧池，恢复 `NoValidRoot` 拒 | 择旧池，恢复 `NoValidRoot` 拒 | 择旧池，恢复成功、几何错（4096 当 8192） | 一档，无关 |
| V2-歧义即拒 | 收到两档以上、自述槽距不同，整池拒 | 拒 | 拒 | 拒 | 一档，无关 |

两点：
1. V2-今天在 S4、S6 上今天就是拒（错在错误名与拒的位置：`NoValidRoot` 而不是「找到两个互不一致的槽」），在 S8 上挂错几何；V2-歧义即拒在三格上都拒、不挂错。这一列是实测加推（V2-歧义即拒没有实现，S4/S6/S8 上拒是按定义推的）。
2. V2-歧义即拒的代价在没测的另一个方向：残留世代号比新池小、新池槽 1 好时，今天取世代号大的 = 新池、能挂；V2-歧义即拒会拒。这一格我没造镜像量，是推的；它换来的是 S8 不挂错几何。
   代价行数（推）：core `crates/singlefs-core/src/recovery.rs:972` 起的循环收成一个 `Vec`、多于一个不同槽距就返回错 +8 行；GPU 判器同形 +8 行；执行器 +5 行。没有任何一边需要删代码。

「V2-与别盘对账」（逐档收到的候选与别盘的 fsid、槽距对账）我没单列：S8 的残留与新池同 fsid，对账分不出；S5 型可以救，但条款没有，也没实现，归 1.6 的零轮改法。

什么现象会推翻 V2 这一节：D22、D2、D23、layout 里找到一句明写「槽 1 候选之间怎么选」的条款（我 `grep` 了「取偏移小」「世代号相同」，只命中描述实现的 layout:160）；或 S4、S6、S8 在别的冻结副本上输出不同。

## 三、V3　checker 与实现照同一句之后怎么验

### 3.1 这张坏镜像用哪个装置造、进哪一档

三个可选位置，我按今天 `crates/` 里有什么逐个看：

1. **harness 档（非 checker 档）——够用，今天缺的就是它。** 同一份镜像、core 与 checker 一起判的形态已经有现成的：`crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs:3` 「这里在同一份镜像上两边一起判」。
   造镜像的 helper 在 core 那份文件里（私有）：`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:421` 「fn formatted_image_with_minimum_input_output_bytes(minimum_input_output_bytes: u32) -> MemoryPool {」，加同一文件的 `flip_slot_zero_on_every_device`（第 430 行）。
   我的探针 `research/prompts/m3-c651-r1-sonnet-model/c651_probe.rs` 就是这个用例的草稿：造 io_min 512 / 8192 两个池，弄坏槽 0，同一份镜像交 core 的 `choose_system_configuration` 与 checker 的 `verified_system_configuration_slots`、`check_pool_image`（I-8.5），打印，没有断言。
   落成用例时的比较键：（有没有可择槽、择到哪块盘、槽距、世代号）两边必须相同。今天在这一键上两边不同的格：S2（core 择到、checker 无可择槽且 I-8.5 红）、S6（core 择旧池、checker 择新池）。S1、S3、S4、S8 两边今天相同（S4、S8 是同样地错）。
   进的档：`cargo test -p singlefs-harness --test <新文件>`，是 harness 档，不是 checker 档，子 agent 能跑；我没有把它落成入库用例（不在我的写范围），草稿在副本拷贝里跑过，输出见 `research/prompts/m3-c651-r1-sonnet-model/c651_probe.out`。
2. **checker 档的坏盘输入**：`crates/singlefs-checker-tier/src/bad_disk_input.rs:224` 「pub const EVERY_DAMAGE_KIND: [DamageKind; 18] = [」里 18 种坏法没有「全池槽 0 弄坏」，而且这份文件对 io_min 的命中数是 0：

```
$ grep -c 'minimum_input_output\|io_min' crates/singlefs-checker-tier/src/bad_disk_input.rs
0
```

   所以要在那里造，得同时加 io_min 这根轴和一种新坏法；那是 checker 档，我不跑。C651 那一行原话是「坏盘输入里造」，指的是这一处；我建议主 agent 先落 1，再决定要不要加 2（这是我的意见，不是条款）。
3. **层 0 崩溃流（E170 的两条流、GPU 判器）**：今天分不出 V1-常量与 V1-逐档。依据：两条流取样态的槽 1 都在 4096 处自证（`count_slot0_b.out` 里「槽1(4096)自证过的盘数」那一列）；这两条流用的 mkfs 参数的槽距没有逐个核（`crates/singlefs-checker-tier/src/crash_injection.rs:807` 的 `fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,` 说明它跟着参数走，值我没查）；所以这一句只写「推」。要让层 0 流覆盖槽距 > 4096，得另给一条 io_min 8192 的流，是重档工作。

### 3.2 O3 严格跑法照这一句写的那一步

执行器里今天的读法：`research/e7-index-bench/src/bin/e170_o3_spec_executor/system_configuration.py:77` 「def locate_self_verified_slots(image: SparseCrashImage, device_identity: int,」只收一块盘（调用处 `research/e7-index-bench/src/bin/e170_o3_spec_executor/system_configuration.py:164` 一处），所以它没有 ②；撞 G03 的那一行：`research/e7-index-bench/src/bin/e170_o3_spec_executor/system_configuration.py:93` 「gaps.record_hit(spec_gaps.GAP_WHERE_SYSTEM_CONFIGURATION_SLOT_ONE_LIES_WHEN_SLOT_ZERO_IS_UNVERIFIED)」。
照 1.5 的候选句，这一步是（执行器只读 kb，所以这一句要先进 D22 已定项 16，不能只在我的报告里）：
1. 签名从「一块盘」改成「池」：先对每块盘判槽 0 是否自证、可择，记下第一块槽 0 可择的盘的槽距（② 要这个）；
2. 本盘槽 0 可择：槽 1 在它自述的槽距（与今天相同）；本盘不可择而池里有一块可择：槽 1 在那一块的槽距（新增，① ② 两步）；全池都没有：逐档，每 512 字节一档，收自证过、可择、自述槽距 == 所在偏移的槽，多档按 V2 定的句取（今天是收第一个就停）；
3. 删掉 `record_hit` 那一行与 `spec-gaps.tsv` 里 G03 一行（`research/e7-index-bench/src/bin/e170_o3_spec_executor/spec-gaps.tsv:4`），严格跑法重跑批 1。期望：G03 命中从 10 + 60 降到 0；这 70 态里 14 态走第三条（S1 形）、56 态走第二条（一盘槽 0 坏），其余命中的另两个欠一句 G01、G02 与这一句无关。
这一步我没有跑（执行器与严格跑法不在我的写范围，也不是我的格），写的是「该改哪三处」。

什么现象会推翻 V3 这一节：harness 的 `system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 的造镜像方式（`common::build_pool`）造不出 io_min 8192 的池（我用的是另一个文件的 helper，没用 `common`）；或 E170 两条流的 mkfs 参数里有 io_min > 4096（我没逐个核）；或执行器 `locate_self_verified_slots` 在 `system_configuration.py:164` 之外还有调用处（`grep -nF` 只命中这两行）。

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V1-常量 × D22 已定项 16 / D2 已定项 19 / I-8.5 | 冲突（槽距 > 4096 的池）；槽距 4096 的池一致 | 槽距 8192、两盘槽 0 都坏时 checker 读 4096，判不出槽 1 且 I-8.5 判红，而 core 择到了（实测 S2） |
| V1-逐档 × 同上 | 一致 | 只收自述槽距 == 所在偏移的槽 = 「槽 i = i × 自述槽距」；上界用第一版常量，D22 已写；S2 实测 core 择到 |
| V1-拒 × D22 已定项 8 容错承诺 / I-8.5 字面 | 冲突 | 全池每盘槽 0 都坏正是「每盘一槽系统配置坏」，在承诺里；E170 流批 1 有 14 态是这一形（实测计数） |
| V1 汇总（判据 1） | 只有逐档与条款文字相容 | 候选句在 1.5（腿起草，没被攻过），连 ① ② 一起写；改动行数表在 1.5 |
| V1 残留 | 规则没说 | 三个候选都躲不开同 fsid 残留：S8 实测恢复成功、几何错；mkfs 不清 [4096, 槽距) 是来源，1.6 记了两个零轮改法 |
| V2 | 规则没说 | 「世代号最大」是同池两槽之间的择槽条款，「同号取偏移小」全仓只有 layout:160 一处描述；要新写，两个候选与结局在 2.3 |
| V3 装置与档 | 有现成同形用例，缺的是 io_min 8192 的这一格 | harness 档一条用例即可，草稿 `c651_probe.rs` 已跑出今天两边在 S2、S6 上不同；checker 档坏盘输入没有这种坏法且无 io_min 轴；层 0 流分不出两个候选 |
| V3 O3 严格跑法 | 要改三处 | 执行器签名从一块盘改成池、加 ②、③ 收多档按 V2 取，删 G03；这一句要先进 D22 已定项 16，执行器只读 kb |

## 五、没做什么、复核不了的

- 不替主 agent 采纳，也不出判决：1.5 的候选句与 2.3 的两个候选都是「相容性 + 结局」，选哪个、要不要交用户由主 agent 判；不判攻方腿的格。
- 没有把 V3 的用例落成入库文件：不在写范围，草稿与输出在模型目录，README 有跑法。
- 没跑 checker 档任何测试，没跑 E170 执行器的严格跑法（也没改执行器）；`count_slot0_b.py` 只读执行器的建镜像模块与 `research/target/e170-dumps`。
- 没量的（读代码推的）：残留槽带越界值时逐档在偏移比对之前整池拒（1.4）；槽距写成 4100 这类读者收而 mkfs 产不出的池逐档找不到（1.2）；V2-歧义即拒在「残留世代号小于新池」那一格的代价（2.3）；S8 之后可写挂载按错槽距写会怎样（没走可写挂载）。
- 核不了的：E170 两条流 mkfs 参数的槽距值（只核了取样态里槽 1 都在 4096；`crates/singlefs-checker-tier/src/crash_injection.rs:807` 说明槽距跟着参数走，没有逐流查参数）；对全仓 kb 做「整池拒 / 只认 4096」这类词的全文搜（只 grep 了 D22、D2 的相关词与 `取偏移小`）；冻结副本与工作区当前 `crates/` 是否逐字节相同（只在副本上读、跑）。
- 探针与计数脚本是我自己的模型：它们在冻结副本的拷贝里跑，不与入库装置共用代码以外的东西；按 `.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」，上面的实测数主 agent 引之前要在入库用例里重做一次。
- 包装器 `research/scripts/run-with-memory-cap.sh` 往 `research/scripts/memory-peaks.tsv` 追加了我这几条命令的峰值行（它自己写的，不是我动的）。
- 草稿目录 `/tmp/claude-1000/m3-c651-r1-forward/`：探针原始输出 `probe2.out`、`probe_b.out`、`probe_c.out`、两份计数输出与计数脚本；编译用的仓副本 `/tmp/claude-1000/m3-c651-r1-forward/tree/` 含 `target/`，交回前删掉（探针源码与输出已进模型目录）。
