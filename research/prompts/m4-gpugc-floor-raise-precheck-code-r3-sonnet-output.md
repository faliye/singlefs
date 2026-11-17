# m4-gpugc-floor-raise-precheck-code-r3：云端正推腿（Sonnet）报告（2026-11-10）

背景材料 `research/prompts/_m4-gpugc-floor-raise-precheck-code-r3-background.md`，正文、改动全文、附录同名后缀。冻结副本 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3/repo`（下称冻结副本；`mounted_session.rs` sha256 前缀 `4d58410438a9781d`，与仓内工作区此刻那一份、与 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-snapshot/crates-sha256.txt` 的记录一致）。
本腿没写模型，只写了两个只读数数脚本（下面第零节）；没往统一任务服务交任务，没起 cargo。

## 零、要主 agent 跑的命令与文件 sha256

只读数数脚本（从第二轮两份探针已有的输出里重新数出本报告 T3 引的全部数；不编译、不写文件，秒级）：

```
task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3-sonnet/derive_t3.py /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-main-check/opus-run/out /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-main-check/sonnet-run/out
```

| 文件 | sha256 |
|---|---|
| `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3-sonnet/derive_t3.py` | `f40dc233f7b9d96dfed135353ce3f6899707b0aef27c95ea8279e61c3af3aa0e` |
| `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3-sonnet/derive_existing.py` | `e8d9033736ec6eee5bbb3b3171afeec2d23d841ae89bf3f82a26718c7416af76` |
| 被数的攻方输出 `opus-run/out/probe.out` | 前缀 `8e72cdeed7477443`（与第二轮判决第 18 行引的前缀一致） |
| 被数的辩方输出 `sonnet-run/out/side-by-side.txt` | 前缀 `4ea3eb75a0cdfbb3`（与第二轮判决第 15 行引的前缀一致） |

第二条（辩方已有用例里聚簇段那一项离翻面多远）：

```
task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3-sonnet/derive_existing.py /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-main-check/sonnet-run/out
```

我本机直接跑的输出（原样）：

```
槽对那一判的先算行=116  其中聚簇段那一项>0 的=58  旧上界<需求的=110
聚簇段那一项>0 且旧上界已判放不下的行=56
  那一项折成字节最大=1015808  带上它之后离需求最近还差=208044032 字节
  翻面（旧上界<需求<=新上界）的行=0
```

第一条脚本我本机直接跑的输出（原样，T3 各数都出自这里）：

```
== 攻方：全量 probe.out
cells(A=..B=.. 各行之和)=50198  探测点(probe-point 各行之和)=282  C 那一圈罩着的探测点=61
slot-pair-refusal-calls=49610  占 cells 的 98.8%
vain-series(格数，不是串数)=18105  vain-empty-publishes=41026  每格平均空发布=2.27
vain 格里试过卸载重挂的=1645（占 vain 格 9.1%）  其中写进=0
stuck 格里试过卸载重挂的=610  其中写进=0  C 圈罩着的探测点数x每点上限=61x10=610
含 'outlet-lost' 的 COUNT 键数=0（0 = 那一支一次都没跑到，不是跑了读成 0）
vain 末态是先算又判出放不下（推过一串）且 C 圈罩着的格=500（outlet_lost_after_a_series 的候选格，没有一格试过重挂）
remount 之后段外槽对多出来的数减去段内槽对数（逐盘，各桶之和）：负=2690 零=140 正=1680
自证 arm-a=zero：stuck 重挂试=90 vain 重挂试=0 含 outlet-lost 的键数=0 vain-series=0 arms-differ=0
自证 arm-a=huge：stuck 重挂试=90 vain 重挂试=299 含 outlet-lost 的键数=0 vain-series=4140 arms-differ=7044
```

```
== 辩方：并排与先算行
格数按形： {'natural': 100, 'natural-warm': 99, 'forms': 1} 合计 200
   123 格  三次结局=('NoSpaceAfterRaising', 'NoSpaceAfterRaising', 'NoSpaceAfterRaising')  重挂=REMOUNT_WRITE_OK
    40 格  三次结局=('NoSpaceAfterRaising', 'NoSpaceAfterRaising', 'NoSpaceAfterRaising')  重挂=REMOUNT_WRITE_REFUSED
    25 格  三次结局=('NoSpaceEven', 'NoSpaceEven', 'NoSpaceEven')  重挂=REMOUNT_WRITE_REFUSED
    12 格  三次结局=('WRITTEN',)  重挂=None
SUMMARY cells=200 same=200 lost_path_in_old=0 recovered_in_new=0 white_push_new=40 inside_positive_cells=200
natural-new: 先算行=768 按判分={'FormulaAvailableAgainstDemand': 768}
warm-new: 先算行=1030 按判分={'FormulaAvailableAgainstDemand': 1030}
forms-new: 先算行=8 按判分={'FormulaAvailableAgainstDemand': 8}
natural-old: 先算行=768 按判分={'FormulaAvailableAgainstDemand': 768}
warm-old: 先算行=1030 按判分={'FormulaAvailableAgainstDemand': 1030}
forms-old: 先算行=8 按判分={'FormulaAvailableAgainstDemand': 8}
existing-accept: 先算行=18 按判分={'FormulaAvailableAgainstDemand': 14, 'SlotPairsOutsideTheClusterSegmentsForDataUnits': 4}
existing-wide: 先算行=430 按判分={'FormulaAvailableAgainstDemand': 427, 'SlotPairsOutsideTheClusterSegmentsForDataUnits': 2, 'test': 1}
existing-runtime: 先算行=196 按判分={'FormulaAvailableAgainstDemand': 86, 'SlotPairsOutsideTheClusterSegmentsForDataUnits': 110}
```

（`'test': 1` 是 `existing-wide.out` 第 261 行：cargo 测试输出的「test 名 ...」字样插进了先算行中间，那一行的判是 `FormulaAvailableAgainstDemand`，不影响别的数。）

## T1　撤回是不是干净

**结论：干净。** 先算判法（`SlotsAFloorRaiseCouldRelease` 到 `push_one_floor_raise_for_a_refusal_for_space`、三处调用、`UserChangeRefused` 新成员及其两个结构、`AcceptRefused` 的读法、判法模块 10 条用例）与第一轮被判那一版逐字节相同；第二轮 122 批补丁里留下的只有文档「九种」一处。没有残留的字段、参数、`expect`、用例字段。

### 做法（都是只读命令，输出原样）

第一轮被判的那一版整份在 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1-main-check/tree/crates/singlefs-core/src/mounted_session.rs`（下称 r1 副本，sha256 前缀 `080bd6bc4e3d18f5`，与 `research/prompts/m4-gpugc-floor-raise-precheck-code-r1-snapshot/crates-sha256.txt` 记的 `mounted_session.rs` 那一行一致；8335 行）。冻结副本里那一份 8461 行。两份整份 `diff`：

```
$ diff <r1 副本> <冻结副本> | grep -E '^[0-9]' | wc -l
34
$ diff <r1 副本> <冻结副本> | grep -E '^[0-9]' | tr '\n' ' '
404c404,405 562a564,576 2899c2913 3033c3047 3215c3229,3230 3364c3379 4237a4253,4256 4256,4260c4275,4278 4267,4270c4285 4345c4360 4410,4411c4425,4427 4413c4429,4483 4420,4422c4490 4501,4519d4568 4681c4730,4731 4684c4734,4745 4691,4693c4752 4933,4937c4992,4998 4959,4960c5020 4962c5022,5028 4964c5030 5015d5080 5021c5086,5092 5025c5096 5154c5225 5161c5232,5233 5164c5236 5449,5455d5520 5533,5534c5598,5601 5546c5613,5614 5779,5780c5847,5852 5781a5854,5856 5784,5790c5859,5860 5791a5862,5917
$ diff <r1 副本> <冻结副本> | grep -E '^[<>]' | grep -nE 'FloorRaise|floor_raise|SlotsA|NoSpaceEven|cluster_segments|inside_the|unblocked_slot|抬 F'
19:<     /// 一次整理发布被空间拒就暂停（撤 R 排他、意图留在盘上）、不推抬 F。发布都返回之后发 discard（判决 G5）。
20:>     /// 一次整理发布被空间拒就暂停：发一次只删意图的发布丢掉意图、连同 R 排他一起撤，不推抬 F，下一趟从头签发。发布都返回之后发 discard（判决 G5）。
209:< /// 整理的一次发布没发成（活意图在盘上）：空间不够就暂停——撤 R 排他、意图留着（攻方 K-k），不推抬 F（判决 G8 g1）；别的原样交回。
213:> /// 不推抬 F（判决 G8 g1）。别的拒绝原样交回，什么都不写。
```

34 个改动段里：第 3 段（r1 第 2899 行 → 冻结副本第 2913 行）是文档「八种」改「九种」；其余 33 段都落在 r1 副本第 404 到 5791 行之间的整理（compaction）暂停与停下的代码和文档上，是别的会话在同一份文件里的改动，与先算无关——上面第三条命令把这 33 段里碰到先算字眼的行全捞了，只剩两条整理文档里说「不推抬 F」的字样。r1 副本第 5792 行以后（先算的全部代码、`AcceptRefused` 的读法、用例模块）一个改动段都没有，只整体后移 126 行。

块哈希（同一段在两份文件里的行区间，取这段文字的 sha256 前 12 位）：

| 块 | r1 副本行 | 冻结副本行 | 哈希 | 相同 |
|---|---|---|---|---|
| 模块头文档（先算那两行） | 3-9 | 3-9 | `127cd7a30866` | 相同 |
| `UserChangeRefused` 到新成员的两个结构、宽护之前 | 571-680 | 585-694 | `cd9d1b1d1a71` | 相同 |
| `SlotsAFloorRaiseCouldRelease` 到 `no_space_even_…` 构造函数（含 `of_the_device`、上界、三个函数） | 5815-5942 | 5941-6068 | `db387f7e07ae` | 相同 |
| `push_one_floor_raise_for_a_refusal_for_space` 整个函数 | 5943-6092 | 6069-6218 | `e2b78a3de36b` | 相同 |
| `AcceptRefused` 的文档与 `floor_raises_pushed_by_this_accept` 的 match | 6475-6512 | 6601-6638 | `211f72f225ea` | 相同 |
| 判法用例模块 `everything_a_floor_raise_could_release_tests` | 8059-8304 | 8185-8430 | `15a8425f7db8` | 相同 |

再把第一轮 diff 里 `mounted_session.rs` 那一段（`research/prompts/_m4-gpugc-floor-raise-precheck-code-r1-diff.md`）的 417 条加行、6 条减行逐行对冻结副本核：非空加行在冻结副本里缺的 0 条，减行在冻结副本里仍在的 0 条（只读 python 比对，输出 `plus 417 minus 6` 与两个 0）。

### 逐处落到冻结副本的文件:行

| 处 | 冻结副本现在 | 与第一轮那一版 |
|---|---|---|
| 模块头文档点名新成员 | 「（`UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`）。」（`crates/singlefs-core/src/mounted_session.rs:7`） | 相同 |
| 新成员 | 「NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(」（`crates/singlefs-core/src/mounted_session.rs:646`） | 相同 |
| 新成员的结构 | 「pub struct NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {」（`crates/singlefs-core/src/mounted_session.rs:676`）与「pub struct DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {」（`crates/singlefs-core/src/mounted_session.rs:688`） | 相同 |
| `publish_user_change` 文档「九种」 | 「[`UserChangeRefused`] 的九种。」（`crates/singlefs-core/src/mounted_session.rs:2913`），枚举 `pub enum UserChangeRefused {`（`crates/singlefs-core/src/mounted_session.rs:585`）数得出 9 个成员 | **不同（唯一一处）**：第一轮是「八种」，第 122 批改成「九种」，撤回留着；9 个成员是 `DeviceTableOtherThanTheOneOfTheMount`、`NoFileVersionToChange`、`DevicesWithoutASelfVerifiedSystemConfiguration`、`OwnDeviceNumbersDifferFromTheDeviceTable`、`DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`、`Publish`、`NoSpaceAfterRaisingTheFloor`、`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`、`FloorRaiseFailedWhilePushingForSpace`，数对 |
| 槽数结构 | 「struct SlotsAFloorRaiseCouldRelease {」（`crates/singlefs-core/src/mounted_session.rs:5943`），字段只有 `deferred`、`isolated`、`free_but_blocked` 三个槽数 | 相同 |
| 读分配器 | 「fn of_the_device(device_map: &DeviceFreeMap) -> Self {」（`crates/singlefs-core/src/mounted_session.rs:5954`），一个参数 | 相同（聚簇段参数已无） |
| 上界 | 式子那一判「short.available.0 + bytes_of_slots(slots.deferred) + bytes_of_slots(slots.isolated),」（`crates/singlefs-core/src/mounted_session.rs:5977`）；槽对那一判「slots.deferred + slots.isolated + slots.free_but_blocked;」（`crates/singlefs-core/src/mounted_session.rs:5982`）加「+ bytes_of_slots(slots_that_could_complete_a_pair * slots_of_a_data_unit),」（`crates/singlefs-core/src/mounted_session.rs:5987`）；提交内生块「JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,」（`crates/singlefs-core/src/mounted_session.rs:5990`） | 相同（槽对那一判里没有聚簇段那一项） |
| 恰好等于需求照推 | 「(upper_bound.0 < i128::from(short.demand.0)).then_some(」（`crates/singlefs-core/src/mounted_session.rs:6000`） | 相同 |
| 先算的位置 | 「if !devices_short.is_empty() {」（`crates/singlefs-core/src/mounted_session.rs:6082`）在 `push_one_floor_raise_for_a_refusal_for_space` 开头、`refusal_is_short_of_space` 判与推串之前 | 相同 |
| `accept` 读法 | 「UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) => {」（`crates/singlefs-core/src/mounted_session.rs:6636`） | 相同 |
| 判法模块 | 「mod everything_a_floor_raise_could_release_tests {」（`crates/singlefs-core/src/mounted_session.rs:8185`），模块内 `#[test]` 10 条（`grep -n '#\[test\]'` 在 8185 到 8431 之间数得 10 个；第 11 个是后一个模块的），与第一轮同 | 相同 |
| `compaction_plan.rs` 的文档 | 「这笔算术不算 `accept` 被空间拒时当场推的抬 F 那几串」（`crates/singlefs-core/src/compaction_plan.rs:1534`）；该文件与 r1 副本只有 5 个改动段（609、630、637、757、840 行起），全在整理暂停的文档与 `pause_the_live_intent`、`resume_the_live_intent` 两个函数的删除上，与这一行无关 | 相同 |

### 残留检查（零命中）

```
$ grep -rn 'inside_the_cluster_segments\|slot_pairs_a_remount_frees\|an_unblocked_slot_pair_inside' crates --include=*.rs
（只有 3 行：singlefs-verification 的 admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs 里本来就有的 free_slot_pairs_outside_and_inside_the_cluster_segments_of_this_mount 助手函数，和 crates/singlefs-core/src/allocator.rs 第 580 行的局部变量 pairs_inside_the_cluster_segments；都不是第 122 批的）
$ grep -rn 'unblocked_slot_pairs_inside' . --include=*.rs --include=*.tsv --include=*.md --include=*.toml   # 冻结副本
（无输出）
```

先算区间（冻结副本第 5941 到 6218 行）里 `cluster_segments`、`BTreeSet` 都没有出现；`use std::collections::{BTreeMap, BTreeSet};`（`crates/singlefs-core/src/mounted_session.rs:23`）仍有别处在用（第 4789 行起），所以撤回没有留下没人用的 `use`。

### T1 续：与 D16 已定项 10 射程那一段逐句对

`.claude/kb/decisions/16-发布语义.md` 的已定项 10 标题在第 269 行，「**射程**」那一段是第 275 行（单独一长行）。下面把与先算有关的句子逐句拆开，每句的引文都是第 275 行里的原文，对着冻结副本的行。

| 句（kb 第 275 行） | 冻结副本里对应的 | 判 |
|---|---|---|
| 「`accept` 被空间拒时自己抬 F（用户 2026-11-09 定）：与 `publish_user_change` 同一个推法、同一个上限，推满仍放不下才拒」（`.claude/kb/decisions/16-发布语义.md:275`） | `accept` 里调的就是同一个函数：「let pushed = push_one_floor_raise_for_a_refusal_for_space(」（`crates/singlefs-core/src/mounted_session.rs:7030`），`publish_user_change` 那一边是「push_one_floor_raise_for_a_refusal_for_space(」（`crates/singlefs-core/src/mounted_session.rs:3795`） | 对得上 |
| 「抬 F 之前先算（用户 2026-11-09 定）：推抬 F 放得出的全放出来也放不下就一串都不推，报 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`」（`.claude/kb/decisions/16-发布语义.md:275`） | 先算在函数开头、推串之前：「if !devices_short.is_empty() {」（`crates/singlefs-core/src/mounted_session.rs:6082`）里 `return Err(no_space_even_with_everything_a_floor_raise_could_release(`，构造成员 `UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(Box::new(`（第 6046 行） | 对得上 |
| 「头一次被拒就判出时盘上逐字节不变（推过一串之后、或 unlink 意图发成之后走空才判出的，盘上有这次调用写的东西）」（`.claude/kb/decisions/16-发布语义.md:275`） | 提前返回发生在 `push_one_floor_raise_within_the_admission_budget` 之前，一次空发布都没发；成员把调用方传进来的 `published_in_this_change` 原样带上（`no_space_even_with_everything_a_floor_raise_could_release` 的参数，第 6036 行起）：头一次被拒时它是空的，推过一串或走空发过版时非空；`accept` 那一边读它：「UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) => {」（`crates/singlefs-core/src/mounted_session.rs:6636`） | 对得上 |
| 「先算罩 `accept`、`publish_user_change` 与走空三处」（`.claude/kb/decisions/16-发布语义.md:275`） | 调 `push_one_floor_raise_for_a_refusal_for_space` 的恰是三处：`publish_the_change_raising_the_floor_for_space` 里（第 3795 行，`publish_user_change` 走它）、走空函数里（`crates/singlefs-core/src/mounted_session.rs:6195`）、`accept`（第 7030 行）。`grep -nF 'push_one_floor_raise_for_a_refusal_for_space('` 在冻结副本里恰这三处调用（加定义第 6069 行） | 对得上 |
| 「不罩改动发成之后另发的搬迁批」（`.claude/kb/decisions/16-发布语义.md:275`） | 搬迁批自己的函数（第 2403 行起）直接调「push_one_floor_raise_within_the_admission_budget(」（`crates/singlefs-core/src/mounted_session.rs:2461`），不经先算 | 对得上 |
| 「也不判提交内生块那一判（照推）」（`.claude/kb/decisions/16-发布语义.md:275`） | 「JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,」（`crates/singlefs-core/src/mounted_session.rs:5990`），`device_short_even_…` 用 `?` 把 `None` 当成放得下（第 5999 行） | 对得上 |
| 「短缺种类混着来时可能推过一串才判出」（`.claude/kb/decisions/16-发布语义.md:275`） | 三处调用都在循环里，每次被拒都重新进先算、按那一次拒绝自己的判（`judged_short_by`）算上界，所以第一次是提交内生块那一判（`None`、照推）、推过一串后换成别的判而判出，是代码本来的样子 | 对得上（读码推的，没量过） |
| 「式子那一判的上界是可用加 defer 加隔离」（`.claude/kb/decisions/16-发布语义.md:275`） | 「short.available.0 + bytes_of_slots(slots.deferred) + bytes_of_slots(slots.isolated),」（`crates/singlefs-core/src/mounted_session.rs:5977`） | 对得上 |
| 「短在段外槽对那一判同样先算」（`.claude/kb/decisions/16-发布语义.md:275`） | 槽对那一判有分支（第 5980 行起），上界 = 可用 + (defer + 隔离 + 扣住) × 一个数据单元的槽数换算（第 5982、5987 行）。kb 没写这个式子，只写「同样先算」，没有冲突 | 对得上（kb 没写式子，不算缺） |
| 「上界恰好等于需求照推」（`.claude/kb/decisions/16-发布语义.md:275`） | 严格小于才判出：「(upper_bound.0 < i128::from(short.demand.0)).then_some(」（`crates/singlefs-core/src/mounted_session.rs:6000`） | 对得上 |
| 「先算之前量到的连续被拒推串的代价：每次 `series=2 empties=6`，一次白写 `writes=40 barriers=16 written_bytes=500736`」（`.claude/kb/decisions/16-发布语义.md:275`） | 是改动之前的实测读数，不对着代码 | 不对代码，不判 |

kb 射程没有写、也没有冲突的一处：先算判出放不下时报 `NoSpaceEven…`，没有 `wide_protection_at_the_ceiling`，不带出路；而已定项 1 那一行写的出路是「准入推抬 F 停在上限、C 那一圈这一刻还隔离着槽时」（`.claude/kb/decisions/16-发布语义.md:39`）才报的，先算判出放不下的这一形没推到上限，两句不冲突，但「先算先于宽护那一报」没有一句条款。这就是第二轮判决登的欠账（判出放不下时 C 那一圈罩着、卸载重挂写得进），本轮 T3 说明它为什么还没有数据。

另一处 kb 没写的到达面：推环预演（算「要出路」时）经 `MountedSession::publish_the_change_raising_the_floor_for_space` 在分配器拷贝上再发一次这次改动（`crates/singlefs-core/src/mounted_session.rs:1870`），先算在预演里同样起作用：推完环仍被拒时预演交回「Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {」（`crates/singlefs-core/src/mounted_session.rs:1907`）。我没追这个结局在列出路时怎么用（推的），只记下先算会在这个入口里起作用，与射程不冲突，只是三处调用点之外还有这个入口在跑先算。

**T1 判定：撤回干净，先算判法与第一轮被判那一版逐处相同，只多文档「九种」一处，kb 射程逐句对得上。**
什么会推翻它：冻结副本此刻的 `mounted_session.rs` sha256 不是 `4d58410438a9781d…`（我比的不是被判的那份）；或 r1 副本不是第一轮被判的那份（它的 sha256 与 r1 快照记的 `080bd6bc4e3d18f5…` 对不上）；或用别的写法比整份文件（行尾、空白）出现我这份 `diff` 没看出的差。

## T3　第二轮判决第一节两份探针的读数口径

读的是两份模型源码（只读）：攻方 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-main-check/opus-model`（下称攻方模型）、辩方 `…/sonnet-model`（下称辩方模型），数出自两份输出 `opus-run/out/probe.out`、`sonnet-run/out/*`（第零节脚本重数过）。判决原文行号指 `research/prompts/m4-gpugc-floor-raise-precheck-code-r2-main-verification.md` 的行。

两份探针都把第二轮冻结副本（带聚簇段那一项）当 A/新臂，把「那一项读成 0」当 B/旧臂；旧臂就是这一轮撤回之后的上界（r3 的 `most_a_floor_raise_could_make_available` 少了那一项，其余相同，读码见 T1）。

### 逐项一行

| 项 | 数的是 X（源码:行） | 判决用它说 Y（判决行） | 对得上？ |
|---|---|---|---|
| `vain_series_by_the_inside_term=18105` | 探测点上每个 g 一格：A 臂没写进且推了至少一串、B 臂头一次就判放不下的格数。「let vain = !a.published() && a.series > 0 && b.fired_first_round();」（`opus-model/probe/src/shapes.rs:466`），累加「tally.add("vain-series", 1);」（`opus-model/probe/src/shapes.rs:487`），印成「self.count("vain-series"),」（`opus-model/probe/src/tally.rs:54`）。单位是格：一格里 A 至少推一串，41026 次空发布 ÷ 18105 格 = 2.27；g 是探测点上按步长扫的（「let stride = ((high - low) / SIZES_PER_PROBE_POINT).max(1);」`opus-model/probe/src/shapes.rs:441`），不是真实写的分布 | 第 28 行「白推 18105 串、41026 次空发布」 | **对得上**（作为「撤回的那一项会让先算改推白串」这件事成立）；口径松两处：18105 是格数不是串数（串数不小于它，没单数），也不是出现频率 |
| `vain_remount_lets_it_through=0` | vain 格里**被抽到**做卸载重挂、且重挂后同一个写发成的格数。能抽到的条件是「if follow_the_way_out && !a.published() && remounts_left > 0 {」（`opus-model/probe/src/shapes.rs:508`），额度「const REMOUNTS_PER_PROBE_POINT: u32 = 10;」（`opus-model/probe/src/shapes.rs:55`）；重挂「match arm_a.unmount_then_mount() {」（`opus-model/probe/src/shapes.rs:525`）后「let d = publish_observed(&mut arm_a, change, InsideTerm::AsCoded);」（`opus-model/probe/src/shapes.rs:529`），写进才记「self.count("vain-series remount-retry published"),」（`opus-model/probe/src/tally.rs:58`）。1645 次试，0 次写进 | 第 28 行「卸载重挂让它写进 0 次」 | **对得上一半**：分母是 1645（占 vain 格的 9.1%），不是 18105；抽的是每个探测点 g 从小往大的头 10 个可跟格，这些最接近旧上界，也最可能写进，所以 0/1645 是偏有力的负证据；但判决那句没写分母 |
| `outlet_lost_first_round=0` | A 臂头一次被拒就是先算判放不下、探测点上 C 那一圈罩着、**并且这一格排到了卸载重挂**、重挂后写进的格数：「let lost_with_the_c_ring = a.kind == NO_SPACE_EVEN && holding > 0;」（`opus-model/probe/src/shapes.rs:468`），只在上面第 508 行那个带额度的分支里记（`opus-model/probe/src/shapes.rs:540`、`opus-model/probe/src/shapes.rs:546`），印成「self.count("outlet-lost-first-round remount-retry published"),」（`opus-model/probe/src/tally.rs:62`）。这个数出在**主跑**（A 臂 = `as-coded`，即带聚簇段那一项），不是「读成 0」那一臂 | 第 29 行「那一截读成 0 的一臂（第一轮修补之前的判法）上 `outlet_lost_first_round=0`，`stuck_reports=7730`、`c_ring_holding_at_probe=61`……第一轮采纳的理由在取样里没出现」 | **对不上**：两层。一，7730 和 61 是主跑（A 臂带那一项）的数；读成 0 的那一臂是另一次 62 个配置的自证（`stuck_reports=1079`、`c_ring_holding_at_probe=9`）。二，这个 0 不是「试了没丢」，是**那一支一次都没跑到**：见下一节 |
| `published_where_the_old_bound_fired=0` | A 臂写进了而 B 臂头一次就判放不下的格数：「let published_where_the_old_bound_fired = a.published() && b.fired_first_round();」（`opus-model/probe/src/shapes.rs:467`），印成「self.count("published-where-the-old-bound-fired"),」（`opus-model/probe/src/tally.rs:63`） | 第 31 行「上界算小、误拒推了本来写得进的写……两份取样都没撞上」 | **对得上一半**：只覆盖 (旧上界, 新上界] 这一窗里（A 的上界不小于 B，A 推了才可能写进）：窗内 18105 格 A 推了都没写进，窗内误拒确是 0；窗外 A 也判放不下的格，探针没有「不带先算的真发布」做对照，旧上界在那里误不误拒这个数看不见。「两份取样」里另一份是第一轮任务 11937，不是辩方探针（辩方没数这一项） |
| `recovered_in_new=0` | 辩方并排脚本里 `lost_path_in_old` 的子集：「if has_the_precheck_refusal(o) and remount_ok:」（`sonnet-model/tools/side_by_side.py:71`）才进「counts["lost_path_in_old"] += 1」（`sonnet-model/tools/side_by_side.py:72`），其中新臂不再被先算拒的才记「counts["recovered_in_new"] += 1」（`sonnet-model/tools/side_by_side.py:74`）。`lost_path_in_old=0` 时它必为 0 | 第 22 行「新那一项在真分配器上改结局的格全是白推（`recovered_in_new=0`、`outlet_kept_by_the_inside_term=0`、`vain_remount_lets_it_through=0`），没有一格换来写进或出路」 | **对不上（空真）**：200 格里新旧结局逐字相同（「if show(n) == show(o):」`sonnet-model/tools/side_by_side.py:68`，`same=200`），没有一格是「新那一项改了结局」的格，谈不上它们「全是白推」；原因见下面「辩方探针的先算全走式子那一判」 |

### 读成 0 的那几项：装置走不走得到

**攻方 `outlet_lost_first_round`、`outlet_lost_after_a_series`：走不到。**
- 全量输出里含 `outlet-lost` 的计数键 0 个（脚本输出「含 'outlet-lost' 的 COUNT 键数=0」）。`tally.rs` 把每个出现过的键都印出来（`Tally::print` 遍历 `counts`），键没有就是那一支一次都没执行：不是「试了 N 次读成 0」。
- 为什么没执行：这一支只在带额度的重挂分支里（「if follow_the_way_out && !a.published() && remounts_left > 0 {」，`opus-model/probe/src/shapes.rs:508`），额度每个探测点 10 次。C 那一圈罩着的探测点 61 个，卡宽护的格（`stuck`）重挂试了 610 次 = 61 × 10，额度整个被它们吃光；自证 `arm-a=zero` 是 9 个点 × 10 = 90、`arm-a=huge` 同样 90。g 是从小往大扫的（「let mut g = low;」`opus-model/probe/src/shapes.rs:447`，「g += stride;」`opus-model/probe/src/shapes.rs:571`），卡宽护的格在中段，先算判放不下的格在大 g 一端，额度到不了。
- 候选格存在、一次没试：vain 格里末态是「先算又判放不下（推过一串）、C 圈罩着」的 500 格（脚本输出，对应 `outlet_lost_after_a_series` 的条件）。头一次被拒就判放不下且 C 圈罩着（`outlet_lost_first_round` 的条件）的格数，输出里不分 `holding`，数不出；有的上限是 crash 家族「A=NoSpaceEven、没推串」的 450 格（含 `holding=false` 的探测点），「推的，没量过」。

**攻方 `outlet_kept_by_the_inside_term=0`、`vain_remount_lets_it_through=0`：走得到，但整份输出里没有一格「重挂之后写得进」的正对照。** 条件「if b.fired_first_round() && d.published() {」（`opus-model/probe/src/shapes.rs:536`）要卸载重挂写进；卡宽护格重挂 610 次 0 写进，vain 格 1645 次 0 写进，自证 `huge` 臂 299 次 0 写进，三份输出里「重挂后写进」的计数从来没有非 0 过。攻方自证只验了 vain 这一个数（`huge` 臂 4140、`zero` 臂 0），出路那几个数没有任何一道自证。装置能不能造出「重挂写得进」这一形，攻方输出里没有证据。

**辩方 `recovered_in_new` / `lost_path_in_old`：走不到——先算全程走式子那一判，聚簇段那一项根本没被读。**
- 聚簇段那一项只在槽对那一判里加：r2 副本（主 agent 核查树，`~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-main-check/tree`，`mounted_session.rs` sha256 前缀 `5104b9b5e4089e26`）里式子那一判是「JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand => {」（`tree/crates/singlefs-core/src/mounted_session.rs:5985`），只加 defer 与隔离；那一项在槽对那一判的分支里：「slots.unblocked_slot_pairs_inside_the_cluster_segments;」（`tree/crates/singlefs-core/src/mounted_session.rs:5996`）、「slot_pairs_a_remount_frees_from_the_cluster_segments * slots_of_a_data_unit,」（`tree/crates/singlefs-core/src/mounted_session.rs:6000`）。
- 辩方 200 格的先算行共 1806 条（`natural` 768、`warm` 1030、`forms` 8），`judged=` 全是 `FormulaAvailableAgainstDemand`，槽对那一判 0 条（脚本输出）。新旧每格的上界字节逐字相同（例：`demand=770048 … bound=3637172 old_bound=Some(3637172)`，`sonnet-run/out/natural-new.out` 里的先算行）。
- 所以辩方 `same=200`、`recovered_in_new=0`、`white_push_new=40` 都与新那一项无关：`white_push_new` 数的是「新臂推了串、三次都没写进、重挂也写不进」，旧臂在同样格里结局逐字相同；`inside_positive_cells=200` 数的是各盘 `in=` 字段（这块盘上段内空槽对数，「if any(inside not in ("0", "") and any(part != "0" for part in inside.split("/")) for _, _, inside in n["attempts"]):」`sonnet-model/tools/side_by_side.py:77`）大于 0，只说那一项有值，不说它进了判。
- 辩方能给的是别的东西：123 格先算没拦、推串三次没写进、卸载重挂之后写进（`REMOUNT_WRITE_OK`），是「出路这一形装置造得出来」的正对照；25 格先算（式子那一判）拦下，且这 25 格重挂也写不进：式子那一判的上界在这一形上没吃掉出路。这是关于**式子那一判**的，不是关于被撤回的槽对那一项。
- 另外 `forms` 那三形（读回全零，宽护卡住最典型的一形）只出了 1 格：「fn probe_sweep_forms() {」（`sonnet-model/probe/probe_tests.rs:15`）本想扫 3 形 × 7 个大小（`probe_scenario.rs` 第 145 行起的 `probe_sweep_of_the_forms`），第一形跑完第一个大小就撞同名镜像 panic（`sonnet-run/out/forms-new.out` 第 18 行），格数 100 + 99 + 1 = 200（脚本输出）。判决第一节提到了 panic，没提它让 `forms` 只剩 1 格。

**辩方已有用例（`existing-*`）上「翻面 0」：**槽对那一判的先算行 116 条，聚簇段那一项大于 0 的 58 条；其中旧上界已判放不下的 56 条，那一项折成字节最大 1015808，带上它之后离需求最近还差 208044032 字节（第零节第二条脚本输出）——用例的需求（256 MiB）离上界很远，没有一条落在那一项能翻面的窗里。所以「翻面 0」不是「窗里没翻」，是「没有用例待在窗里」。

**`published_where_the_old_bound_fired`：走得到、窗内有效。**（见逐项表）

### 两份探针共用的前提

| 前提 | 攻方 | 辩方 | 影响 |
|---|---|---|---|
| 只经 `MountedSession::publish_user_change` 发写 | 「.publish_user_change(&mut self.devices, change)」（`opus-model/probe/src/pool.rs:177`） | 场景助手同一个：「let outcome = session.publish_user_change(&mut self.devices, change);」（`crates/singlefs-verification/tests/common_admission_pool/mod.rs:306`） | 先算三处调用里 `accept` 那一处（`crates/singlefs-core/src/mounted_session.rs:7030`）两份都没走；推环预演那个入口也没走 |
| 出路只建模成「正常卸载 + 再可写挂载」 | 「pub fn unmount_then_mount(&mut self) -> Result<(), String> {」（`opus-model/probe/src/pool.rs:209`） | 「if let Err(refusal) = pool.crash_and_mount_writable() {」（`sonnet-model/probe/probe_scenario.rs:74`）前面先 `unmount` | kb 写的另一条出路（推环，`.claude/kb/decisions/16-发布语义.md:39`）两份都没试 |
| 挂载用公式准入 | 「SpaceAdmission::JudgedByTheFormula,」（`opus-model/probe/src/pool.rs:200`） | 「SpaceAdmission::JudgedByTheFormula,」（`crates/singlefs-verification/tests/common_admission_pool/mod.rs:255`，在 `crash_and_mount_writable` 里） | 两份都没看另一种准入 |
| 不带读故障 | 「不带任何读故障」（`opus-model/probe/src/shapes.rs:2`） | 自然崩溃两形不带；读回全零那一形只出 1 格 | 读坏根槽引出的宽护卡住，基本在两份取样之外 |
| 都是 384 槽一档的小盘几何 | 注释「验证包 384 槽小盘那一档同一个数」（`opus-model/probe/src/pool.rs:30`），再加 0/64/256 槽的盘宽 | 场景 `parameters_of_the_384_slot_small_devices` | 盘型单一；攻方多了盘宽三档 |
| 「旧」臂都是在第二轮冻结副本上把那一项乘 0 / 置 0 | `InsideTerm::Zero => without_inside,`（`opus-model/probe/core_probe_module.rs:84`） | 「* u64::from(std::env::var_os("PROBE_OLD_BOUND").is_none()),」（`sonnet-model/tools/patch_the_copy.py:29`） | 两份都没有在撤回之后的 r3 冻结副本上跑过；与 r3 等价靠读码（T1），没跑 |

两份取样**互补地看不见同一格**：攻方 98.8% 的格（49610 ÷ 50198）里槽对那一判在场，聚簇段那一项真的在起作用，但它没有「重挂写得进」的正对照；辩方有正对照（123 格），但它的先算全走式子那一判，那一项从没被读。没有一格同时是「槽对那一判在判」「那一项改了结局」「C 圈罩着」「卸载重挂写得进」。

### T3 小结：能撑什么、撑不了什么

- 撑得住：撤回的那一项会让先算对槽对那一判多放行、推白串（攻方 18105 格，且自证 `huge` 臂 4140、`zero` 臂 0 对得上）。这是第二轮判决撤回的理由，读数撑得起。
- 撑不住：「撤回不丢出路」（判决第 29 行）——攻方 `outlet_lost_*` 那一支一次没跑；「新那一项改结局的格全是白推、没有一格换来写进或出路」（判决第 22 行）里辩方那一半是空真。出路这一格，两份读数合起来是**没有数据**，不是「没撞上」。
- 所以判决里「取样没撞上」「这一份取样里没有它要修的格」两句，按读数应改说「装置没走到」；欠账登记宜写成「撤回之后，先算判出放不下时 C 那一圈罩着、卸载重挂写得进的那一格没有任何有效读数」。这是读数口径的说法，采不采纳由主 agent 定。

什么会推翻 T3 的结论：（一）`probe.out` 里出现含 `outlet-lost` 的计数键（我数是 0 个，那么分支其实跑过）；（二）stuck 重挂试数不等于探测点数 × 额度（610 ≠ 61 × 10），那么额度被吃光的解释不成立；（三）辩方 `natural/warm/forms` 的先算行里有一条 `judged=SlotPairs…`（我数是 0 条）；（四）r2 副本里聚簇段那一项出现在式子那一判的分支里（我读到的在槽对那一判里，r2 副本第 5996、6000 行）。

## 检查单八格

**1. 走不到的理由二次查实：成立。**
撤回之后先算区间（冻结副本第 5941 到 6218 行）里还剩 `.expect(` 两处：「.expect("没挡的槽都没置分配位，分配位没置的都算空闲：没挡的槽数不大于空闲槽数"),」（`crates/singlefs-core/src/mounted_session.rs:5961`）与「.expect("拒绝里的盘来自这个分配器：准入的两份读数都按它的盘建");」（`crates/singlefs-core/src/mounted_session.rs:6026`）。第 122 批唯一新增的那一处（「段外槽对是全部槽对减去那几段各自的数」）：`grep -nF '段外槽对是全部槽对减去' crates/singlefs-core/src/mounted_session.rs` 零命中。区间外 8185 行起的判法模块只有一处 `panic!`（「panic!("交回的就是全放出来也放不下那个成员");」`crates/singlefs-core/src/mounted_session.rs:8404`），是用例里的断言，同第一轮。
两处各自的走不到理由我重读了一遍：
- 第 5961 行：`unblocked_slots` 只在 `set_blocking_bit` 里随三种位增减，计数的是三种位都没置的槽（`crates/singlefs-core/src/allocator.rs:544`、`:545`）；分配位只在两处改：置位处 `free_slots -= span`（`crates/singlefs-core/src/allocator.rs:941`、`:951`），清位处 `free_slots += span`（`crates/singlefs-core/src/allocator.rs:847`、`:854`）；`free_slots` 初值是单元区全部槽数（`crates/singlefs-core/src/allocator.rs:477`）。所以「分配位没置」的槽数恒等于 `free_slots`，三种位都没置的是它的子集，减法不会下溢。读码推的，没跑。
- 第 6026 行：拒绝里的盘来自准入读数，读数按分配器建（`admission_reading_before_a_publish` 一族的参数是 `allocator: &PoolAllocator`，`crates/singlefs-core/src/admission.rs:1706` 起），而先算拿的 `allocator` 与被拒那一次判的是同一份（三处调用都在同一个循环里传同一个 `allocator`，推串不增减盘）。读码推的，没跑。
调用点没变：`of_the_device(` 一处定义（第 5954 行）一处调用（第 6029 行）；`push_one_floor_raise_for_a_refusal_for_space` 三处调用（第 3795、6195、7030 行）与第一轮相同（T1 的块哈希：这一区间一个改动段都没有）。

**2. 类型与名字：成立。**
`SlotsAFloorRaiseCouldRelease` 只剩三个字段，都是槽数：`deferred`、`isolated`、`free_but_blocked`（「free_but_blocked: u64,」`crates/singlefs-core/src/mounted_session.rs:5950`；定义在第 5943 行）；槽对字段已无，第二轮攻方说的「槽对与槽共用裸 `u64`」题面消失。`of_the_device` 只收一个 `&DeviceFreeMap`。

**3. 不兼容旧代码与旧格式：成立。**
本轮改动只删（`research/prompts/_m4-gpugc-floor-raise-precheck-code-r3-diff.md` 全是删行与把文档/函数头改回第一轮的写法）。先算区间 5937 到 6220 行里没有 `cfg(`、环境变量、`compat`、`legacy`、`deprecated`、兼容分支；仅有的两个 `#[allow(` 是 `push_one_floor_raise_for_a_refusal_for_space` 与走空函数的 `too_many_arguments`，在第一轮那一版里就有（块哈希同）。

**4. 不新建判定库：成立。**
改动只涉及 `mounted_session.rs` 与 `crates/mutations.tsv`（删一行）。`diff` 主 agent 撤回前留的备份 `m4-gpugc-floor-raise-precheck-r2-revert/mutations.tsv.before`（7580 行）与冻结副本的 `crates/mutations.tsv`（7579 行）只差 `7580d7579`，那一行是「抬 F 先算：槽对那一判的上界不算聚簇段里没挡的槽对」。仓内没有新文件我看得出（改动全文只有这一个文件）；没有查整仓的未跟踪文件，「推的，没量过」。

**5. 所有任务多机可用：成立。**
本轮没有新用例。判法模块 10 条用例（`#[test]` 在第 8185 到 8431 行之间数得 10）只用内存里的结构体：区间里没有文件、环境变量、主机名、线程、`static`（`grep -nE 'thread|spawn|serial|ignore|test_threads|cfg\(|env::|Mutex|static |\.db|sqlite|\.tsv|File::|std::fs'` 在第 8185 到 8430 行只命中两处：`DataUnitIndexInFile::FIRST` 里的 `File::`，和下一个模块头上的 `#[cfg(test)]`，都不是文件或线程）。

**6. 测试优先多线程：成立。**
没有新用例，也没有线程数的写死或覆盖。运行库两条崩溃循环用例上的不并行理由还在：「不切片并行：每次挂载接着上一次崩溃留下的盘，次序本身就是被测对象，没有可切的区间」（`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1093`）与同样一句（`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1101`）；该文件与仓内工作区那一份逐字节相同（`diff` 无输出）。

**7. 带 --item 的命令与项：成立。**
- 被删的变异行不在了：`crates/mutations.tsv` 里「抬 F 先算」开头的行现在 21 条（第 7477 到 7497 行），比删之前少那一条。
- 剩下 21 条的第三列（改前原文）在 `mounted_session.rs` 里各恰好命中一次，第五列的命令点名的 17 个测试名（判法模块 10 个、`accept_raises_the_floor_for_space` 4 个、运行库 3 个）各恰有一个 `fn` 定义（读码 + `grep -c "fn <名>"` 都是 1）。
- 别的登记没有点名被删的用例：`grep -rIl 'an_unblocked_slot_pair_inside_the_cluster_segments' .`（排除 `.git`、`target`）只命中 `research/prompts/` 与 `records/` 下的议论文字，`crates/`、`scripts/`、`.claude/` 零命中；`.claude/gate.d/crash-point-replay-items.tsv` 里也没有先算相关的项。

**8. 新崩溃流多罩了什么：成立（本轮没有新崩溃流，不需要罩）。**
改动只删，没有新的崩溃流；运行库那两条崩溃循环用例没动（上一格）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 撤回是否干净（对第一轮被判那一版） | 成立：逐处相同 | 先算全部代码、新成员、`accept` 读法、10 条用例与第一轮逐字节同；唯一差别是文档「九种」（第 2913 行），无残留字段、参数、`expect`、用例字段 |
| T1 与 D16 已定项 10 射程逐句 | 成立 | 11 句里 10 句对得上，1 句是量值不对代码；kb 没写「先算先于宽护那一报」，与射程不冲突，是已登的欠账 |
| T3 `vain_series_by_the_inside_term=18105` | 对得上（口径松） | 数的是格不是串、也不是出现频率；撑得起「撤回的那一项会让先算推白串」 |
| T3 `vain_remount_lets_it_through=0` | 对得上一半 | 0 出在 1645 次抽到的重挂里（vain 格的 9.1%），判决没写分母；抽的是最可能写进的小 g |
| T3 `outlet_lost_first_round=0` | 对不上 | 数出在带那一项的主跑而不是读成 0 的一臂；那一支一次没跑（计数键 0 个，重挂额度被卡宽护格吃光：610 = 61 × 10） |
| T3 `published_where_the_old_bound_fired=0` | 对得上一半 | 只覆盖（旧上界, 新上界]这一窗，窗内成立；窗外没有不带先算的对照 |
| T3 `recovered_in_new=0` | 对不上（空真） | 它是 `lost_path_in_old=0` 的子集；辩方 1806 条先算行全走式子那一判，聚簇段那一项从没被读，新旧 200 格逐字相同 |
| T3 读成 0 的几项装置走不走得到 | 攻方 `outlet_lost_*` 走不到、`outlet_kept`/`vain_remount` 走得到但无正对照；辩方 `recovered_in_new`/`lost_path_in_old` 走不到 | 两份取样互补地看不见同一格；出路这一格两份合起来是没有数据 |
| 检查单：走不到的理由二次查实 | 成立 | 先算区间剩 2 处 `expect`（第 5961、6026 行），第 122 批那处已无，理由重读成立，调用点没变 |
| 检查单：类型与名字 | 成立 | 只剩三个槽数字段，槽对字段已无 |
| 检查单：不兼容旧代码与旧格式 | 成立 | 本轮只删，区间里没有兼容分支、环境变量、`cfg` |
| 检查单：不新建判定库 | 成立 | 只动 `mounted_session.rs` 与 `crates/mutations.tsv`（删一行）；未跟踪文件整仓没查 |
| 检查单：所有任务多机可用 | 成立 | 没有新用例，10 条用例只用内存结构体 |
| 检查单：测试优先多线程 | 成立 | 没有新用例；运行库两条崩溃循环用例的不并行理由还在（第 1093、1101 行） |
| 检查单：带 --item 的命令与项 | 成立 | 变异表剩 21 条，原文各命中一次、点名的 17 个测试都在；别的登记没点名被删的用例 |
| 检查单：新崩溃流多罩了什么 | 成立（不需要） | 改动只删，没有新崩溃流 |

## 没做什么

- 没判 T2（未定项清单的完整性，是攻方腿的主攻格），也没替主 agent 采纳或出判决；T3 里「判决那两句宜改说装置没走到」是读数口径的说法，采不采纳由主 agent 定。
- 没往统一任务服务交任务，没起 cargo，没重跑第二轮的两份探针：T3 的数全是读已有输出（`opus-run/out`、`sonnet-run/out`）加读模型源码，数数脚本的命令在第零节「要主 agent 跑的命令」；判决要引的数没有复跑。
- 没写模型，没设计能补上出路那一格的新探针。要补的话至少要：每类可跟格单独给额度（不让卡宽护格吃光 C 圈罩着的探测点）、造出一个槽对那一判在判而「重挂写得进」的正对照、再有一道只在撤回之后的 r3 冻结副本上跑的对照；这三样我没有起编译就写不稳，不在本轮写。
- 没在 r3 冻结副本上跑过任何测试或探针；判法模块 10 条绿（任务 11966）、clippy（任务 11967）是主 agent 的读数，我没复核。
- 没追推环预演（`crates/singlefs-core/src/mounted_session.rs:1870` 起）里 `ChangeStillRefusedAfterThePush` 在列出路时怎么用。
- 没查整仓的未跟踪文件（检查单「不新建判定库」只看了改动全文与变异表）。
- 草稿目录里没有自己建的仓副本与编译目录；留下的只有 `derive_t3.py`、`derive_existing.py`（第零节）。
