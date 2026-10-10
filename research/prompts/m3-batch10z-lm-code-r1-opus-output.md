# m3-batch10z-lm-code-r1 攻方腿报告（2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 E1 E2 E3 E7 G1 G2 G4 Y1 Y2 Y3 Y4 R C P Q -->

被判对象是冻结副本 `~/.local/share/singlefs/work/m3-batch10z-lm-code-r1/tree/`（`sha256sum -c` 对 `research/prompts/m3-batch10z-lm-code-r1-snapshot/crates-sha256.txt` 336 行全过）。下面所有数都是攻方副本上的数，不是入库装置上的数。

## 复跑

```
WORK=~/.local/share/singlefs/work/<新目录> bash research/prompts/m3-batch10z-lm-code-r1-opus-model/rerun.sh
```

工作目录仓根。它把冻结副本 rsync 到 `$WORK/copy`、打 `copy-vs-frozen.diff`、release 编、分批并行跑、汇总后与 `runs-summary/` 逐份比（`同：…` / `不同：…`），末尾打 b9 / b10 / b11 的读数行与压测、扫源码两份结果。整条估 10 分钟以上（编译两次加 9 批），我没有整条跑过；里面每一批都在我的草稿目录里分开跑过，结果就是 `runs-summary/` 里那几份。

模型目录 `research/prompts/m3-batch10z-lm-code-r1-opus-model/` 每个文件的 sha256：

```
cc144229753430a9a7ce1a2d4b9d6a4c0603d534e81ddb1f702385d242e5214c  copy-vs-frozen.diff
386118b8073c372f81db25596a3de84d6938a60bbd8655a208116b018233022a  rerun.sh
83c4c5429a8e91365e0bd2c638ee5b026934547bea399aab9828cadfc1e16b66  runs-summary/agg-r-kinds-b5.txt
ed5e2867c8b4a2e512603e8e6c6210264caddfdf89da5cfd06b24ff1fdfff27a  runs-summary/agg-small-pools-b2.txt
6d1d390e327852d51783931fb7ac200760f93611552048fc30581c41e4ce03b7  runs-summary/ledger-early-mark-stress.txt
c188f828e6bf4d71d8653a16847ecdbf6d2ea67c3b0f58d5536a34342859d28f  runs-summary/ledger-random-model.txt
594dfb0b3a9c94fb6a9afc51171f281e585975391b21629b8c831511b3a6346f  runs-summary/mount-counts-b11.txt
4a8234eab8058c46198b3d7c969f39a762ebab5e11ade4b1339c9c3a297cc476  runs-summary/mut-span1-raise.txt
8a2d9b24bc61b412aaa5229177cb870b891af9a17813bfd2024ebc9dda42cc70  runs-summary/pair-tri-nodbl-b3-b4.txt
9b4b43f42cd48c79c4aa70a1225cb5f49d0e6ebcc0289389ce98f4c8b81dab06  runs-summary/q-side-t0-t1.txt
412b7cc1cec4122ef2963bea2b982ed9232f1792f6fbfec657c9f9b0d8dbbfc7  runs-summary/scan-variants.txt
d855b3357da801b78d00fbb0f8b28422761f8bc13c0ded2cfab4393d2f1ef6c5  runs-summary/wedge-b6-b10.txt
057a966c2c4a6ffda34cb2b656592ffa5c6beaa4fdc05d03cd54ade6cc1fd091  src/agg.py
8512cca27a43c67186450598571649897b17f95cb1fb5c98e6eaa8d23cebcaf1  src/pair.py
027cd323ec904b26897d8e679361648c0cc9a24946bc1135f6c89d585308fde5  src/scan_probe.rs
98d830eac6c38271772fe2dc3ff744e5cc700b7c7d93bece5d30bc0fc003436f  src/variants.py
```

`copy-vs-frozen.diff` 改了四份文件，都只在副本里：
- `mount.rs`：两个只看环境变量 `OPUS_LM_ARM` 的臂。`narrow`：「不知道」与「复活集算不出」两支交空（第十二块之前的判法）；`nodbl`：C 那一圈补隔离时，落点的每个槽此刻都占着已分配位（含已释放还在 defer 窗口里的）就跳过（我提的改法，见第一节 H2）。不设就是冻结副本的 tri。
- `allocator.rs`：两个只读探针（隔离位与已分配位同时置着的槽数；一个落点是不是整段占着）。
- `gpu_memory_ledger.rs`：测试模块里加两条攻方用例（随机模型、并发压测），用测试模块已有的假卡。
- 新探针 `crates/singlefs-harness/tests/opus_lm_probe.rs`：起点是上一轮 s2 探针加第十二块实现员的删除动作，加了 Q 一侧三种故障、R 的打包单元两种故障、`S`（一次顺序写 20 个数据单元）、R 还在环里时 C 里 R 那一槽露出来的槽数、挂载那一刻分配器几项计数、逐步跟踪。

取样与预算：盘用 harness 的内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`），每个起点录一次、之后每段从内存拷。先跑一段量挂钟：384 槽一段约 7 秒、1536 槽约 10–14 秒（`runs/t0.log` 一段 7.03 秒），每批按 12–14 件并行、每件 12–16 段定在 4 分钟以内，批不设限时。这一轮没有调层 0 枚举（`enumerate_layer0` / `closed_form_state_count` 都没用），崩溃状态数是 0：故障全是「挂载那一刻读回全零」，崩溃点固定在 R 那次发布落盘之后（见末节限度）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 三态与三种结局 | 没打中 | 6 种 R × 7 组故障 × 两档几何，tri 0 红；R 是回退而只有非谱系片读不出时照样认出回退（42 槽）；单份副本读不出被另一份接住 |
| K1-2 宽护罩不罩得住 | 没打中 | tri 与 nodbl 两臂 R 还在环里时 C 里 R 那一槽露出来的槽数逐段 0；Q 是回退、Q 的根与账读不出的几形都被挂载拒或落到 C554 那一族 |
| K1-3 放的时机与准入 | **打中（两处）** | H1：宽护的量是「现行账里全部已释放的记录」，跟着释放历史涨（1536 槽两块盘挂载那一刻 2242 槽，改前 22），池子在这个进程里卡死：每次写都被准入拒、没有发布、R 那一槽永远写不掉、隔离到进程结束。H2：并进来的已释放落点还在 defer 窗口里的，第九项与「已分配」各扣一次（98 / 116、492 / 634、338 / 2242），1536 槽那一形卡死全由它造成 |
| K2 第十二块判别力 | 没打中（试了一处） | 宽护落点跨度改成 1，入库 harness 文件 4 条认不出用例全红 |
| K3-1 早记丢了之后多给 | 没打中 | 随机模型 5000 种子 × 400 步，账知道丢了之后多给 0 步，早记那一支走到 34986 次 |
| K3-2 在用那一台与重起并发 | 没打中 | 14 线程 × 5000 轮，重起 3564 次、早记 2897 次，没丢的设备记成丢了 0 次，没有断言 |
| K3-3 没有没等成借据的卡 | 没打中（读码，加随机模型旁证） | 账不知道丢了的那一段由重起时 `prepare` 再判驱动那一份；少给集中在丢了的设备「分不出」那两态 |
| K4 第十三块判别力 | **打中** | 两条扫源码用例被改坏行为、照绿的写法骗过：下游交 `&|| { let _ = stop_is_requested; false }`、领块前 `if false && stop_is_requested()`、借之前那一步整个包进 `if false { … }`、在函数体里放一个同名 `device_found_idle_by_a_poll` 恒真；四种在逐字搬运的小程序里照绿，在副本上 `cargo check` 都编得过（前两种连警告都没有） |
| K5-1 账给的大于真能用的 | 没打中 | 同 K3-1；账还不知道设备丢了的那一段数字上多给（53350 步），重起时被 `prepare` 拦 |
| K5-2 卡放得下却拒、并发被压 | 没打中新形 | 随机模型里「卡上真放得下而拒」57479 次，54421 次（95%）在丢了的设备「分不出」那两态（用户已认的只会少借）；另 3058 次在 `DeviceStarted` 态（1569 次没有列出来的没等成借据、1489 次有），这 3058 次的原因我没逐条查 |

## 一、K1 第十二块 tri（core）

### K1-3 打中：宽护的量跟着释放历史涨，池子卡死（H1）

tri 的「不知道」与「复活集算不出」两支都交 `all_released_in_the_current_account()`：现行账里全部 `is_released` 的记录（`crates/singlefs-core/src/mount.rs` 第 2135 行起那个闭包，第 2153、2181 行两处用它，原文「`return all_released_in_the_current_account();`」（`crates/singlefs-core/src/mount.rs:2153`））。这一份不按 F 滤、不按跨度滤，就是此刻账里已释放记录的全部。被抛弃根独占量（第九项）把它们并进隔离集（`.claude/kb/decisions/28-挂载期承诺量.md:27`「只除去现行账里仍分配着的槽」那一句），隔离到「这个进程写掉这条根那一槽」才清（冻结副本 `crates/singlefs-core/src/mount.rs:2118`「宽护那一截与别的 C 落点同一个时机解开（这个进程写掉这条根那一槽时）。」）。

攻击面：历史里先大量释放（覆盖写或顺序写换下旧单元），再让 R 走 tri 的「不知道」支（R 建 inode、R 写的全部单元挂载那一刻读回全零）。已释放记录越多，宽护的量越大。1536 槽盘、22 次顺序写各换 20 个数据单元之后 R 建 inode：

```
arm=tri    alloc_mount=494  free_mount=4208 unblocked_mount=2304 dbl_mount=338 iso_mount=2242 refused_steps=2
arm=nodbl  alloc_mount=2122 free_mount=2580 unblocked_mount=2328 dbl_mount=0   iso_mount=252  refused_steps=0
arm=narrow alloc_mount=2122 free_mount=2580 unblocked_mount=2558 dbl_mount=0   iso_mount=22   refused_steps=0
```
（`runs-summary/mount-counts-b11.txt`，两块盘之和。）tri 挂载那一刻隔离 2242 槽，narrow（改前判法）22 槽，差 2220 槽全是宽护并进来的已释放落点。接着做任何写都被准入拒：60 步的历史里 tri 30 步全 `NoSpaceAfterRaisingTheFloor`、`r_left_at=-`（R 一直在环里）、`iso_end=2242`（`runs-summary/wedge-b6-b10.txt` 里 `label=b9 arm=tri slots=1536 … faults=rall … iso_mount=2242 iso_end=2242 refused_steps=30 r_left_at=-`）。narrow 同历史 `iso_end=0`、池子照常写。

这不是少护、I-7.4 不红（`r=n`，R 不是回退，没有要护的复活单元），方向是多护，但它多护到**把池子锁死**：宽护只在「这个进程写掉 R 那一槽」时清，而写 R 那一槽要先做成一次发布，发布要过准入，准入被这 2242 槽顶住 —— 死锁。tri 本意是认不出回退时宁可多护（弹窗四候选里的候选 2），但把「多护一次挂载读不出的那次发布的复活集」放大成了「多护历史上所有已释放的单元」。

四句自检：
- 分不分辨臂？**分辨**。narrow 同历史 `iso_mount=22 iso_end=0 refused_steps=0`、池子照写；tri `iso_mount=2242 iso_end=2242 refused_steps=30`。两臂只差「认不出时交空还是交全部已释放」。
- 判别子观测不到？不适用：这不是判据判错，是条款（候选 2 的宽护取法）本身把量放大。
- 满足判据的哪个分句？D28 第九项「可写挂载时预隔离的槽 …… 只除去现行账里仍分配着的槽」字面就含「全部已释放」，tri 照它做；代价（隔离量随释放历史线性涨）这一句 kb 里没有，弹窗里给的量是 116–634，没给「先释放一大批再认不出」这一形。
- 跑前条款的改法在这一格还中不中？弹窗四候选里候选 1（留原样记欠账）、候选 4（认不出拒可写）在这一格都不中（候选 1 回到 I-7.4 红但不卡死，候选 4 拒可写、不多护）；候选 3（s2fb，只并释放代晚于 F 的那一截）我没搭臂，推测能把量压到 F 之上那一截、但仍随「F 之上释放多少」涨。

### K1-3 打中：宽护并进来的已释放落点被第九项与「已分配」各扣一次（H2）

并进宽护集的已释放落点里，还在 defer 窗口里的（释放代尚未回收、位图上仍占着）同时算进占着的槽数里（`crates/singlefs-core/src/allocator.rs:259`「占着的槽数：仍分配的加上已释放、还在 defer 窗口里的」，读法甲）与第九项隔离集里。隔离位独立于已分配位（`crates/singlefs-core/src/allocator.rs:578`「隔离一个只被被抛弃根引用的落点：不进已分配、不进 defer、不动空闲计数」），`isolated_slots()`（`crates/singlefs-core/src/allocator.rs:601`）照数，第九项取它（`crates/singlefs-core/src/admission.rs:630` `terms.isolated_for_abandoned_roots_and_unapplied_journal_records,`），「已分配」取 `allocated_slots()`（`crates/singlefs-core/src/admission.rs:582`），可用式两项都减（`.claude/kb/decisions/28-挂载期承诺量.md:19` 的 `可用(d) = 容量(d) − 已分配(d) − … − 被抛弃根独占量(d) − …`）。同一个 defer 槽扣了两次。

探针 `probe_isolated_and_occupied` 数「隔离位与已分配位同时置着」的槽：tri 下 384 槽 98 槽双扣、1536 槽深目标 492 槽双扣，`dbl_mount_released` 与 `dbl_mount` 全相等（都落在已释放记录里）。`runs-summary/pair-tri-nodbl-b3-b4.txt`：`slots=1536 … r=B23 faults=racct,rlin … iso_tri_max=634 iso_nodbl_max=142 dbl_tri_max=492`、`slots=384 … r=b1 faults=racct,rlin … iso_tri_max=116 iso_nodbl_max=18 dbl_tri_max=98`。

这与第十块判决第二节 K2-4「第九项字面只写到比所选那一版新的根的账、没说到并进来的 R_old 账」是同一个病根的延伸：第九项把「宽护并进来的已释放落点」当独占量扣，而这些落点里 defer 那一部分在「已分配」里已经扣过。对照臂 `nodbl`（补隔离时跳过此刻整段都占着已分配位的落点）把 1536 槽那一形的 `iso_mount` 从 2242 压到 252、`dbl_mount` 压到 0，`refused_steps` 从 30 压到 0、`iso_end` 回 0（`runs-summary/mount-counts-b11.txt` 的 `arm=nodbl`；`runs-summary/wedge-b6-b10.txt` 的 `label=b9 arm=nodbl slots=1536 … iso_mount=252 iso_end=0 refused_steps=0 r_left_at=20`）。也就是说第二处（双扣）正是第一处（卡死）量级的主来源：2242 里有 2220 是双扣的 defer 槽。

四句自检：
- 分辨臂？分辨（tri `dbl_mount=492/338`，narrow、nodbl 都 0）。
- 判别子观测不到？不适用。
- 哪个分句？D28 第九项「只除去现行账里仍分配着的槽（那些算在『已分配』里，不在这里再算一次）」—— 这句管的是「仍分配」，没管「已释放但还在 defer 窗口、位图上仍占着」的那一类；defer 槽在「已分配」里（读法甲），在这里又算一次，恰恰违反这句括注的本意。
- 改法在这一格中不中？`nodbl` 这个改法只在我的副本上量过、被攻过零轮：它跳过「此刻整段都占着已分配位」的落点。它对「已释放且已回收、位图上已空」的落点不起作用（那些不在已分配里，本该隔离），对双扣的 defer 槽起作用。它是不是把该护的也漏了，没验（随机模型里 nodbl 的 over 步与 tri 同为「账还不知道丢了」那类、不是真漏，但那是 GPU 模型，core 这边 nodbl 的正确性我没搭 I-7.4 用例核）。

### 这两处与「用户已认的多护」怎么分

正文第二节把「tri 的判法本身与它在『不知道』那一支上多护的那一截」列为报出来不算打中。要分清：
- 纯粹的「多护已释放落点」（候选 2 的定义）本身：在我的取样里不卡死。先释放一大批、再走 tri「不知道」支，`nodbl` 臂（只去掉双扣那一半）隔离 252 槽、`refused_steps=0`、`iso_end=0`，池子照常写。这一截是用户认了的，不算打中。
- **卡死（H1）主要由双扣（H2）造成**：2242 里 2220 是 defer 槽被第九项与「已分配」各扣一次。双扣是记账错误，与第十块判决 K2-4「第九项字面没说到并进来的落点」同族，不是「多护」的本意。去掉双扣（`nodbl`）之后卡死消失。
- 把这两处交主 agent 判是「多护那一截（不算打中）」还是「记账边角（算打中）」。我的判断：H2 是记账错误（defer 槽本在『已分配』里）、落在正文第二节「只攻它的实现与没写到的边角」的射程里，算打中；H1 是 H2 的可见后果。它们是我在副本上量的，没在入库装置上重做。

## 二、K2 第十二块判别力（core）

入库 harness 文件 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 的 4 条认不出用例（`…_lineage_unit_and_its_account_…`、`…_every_unit_it_wrote_…`、`…_root_slot_of_its_target_…`、`…_account_of_its_target_…`）钉的是「挂载那一刻 I-7.4 不红、推过抬 F 复活的槽不空闲」。我造了一处不改认不出判法、只改宽护落点跨度的变异：`placements_a_rollback_publish_revived_released_in_the_current_account` 里 `all_released_in_the_current_account` 闭包把跨度 `u64::from(record.span_slots)` 改成常量 `1`（宽护集的每个落点只隔离起点一个槽）。release 跑入库那份 harness 文件：

```
test units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount ... FAILED
test units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount ... FAILED
test units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount ... FAILED
test units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount ... FAILED
test result: FAILED. 15 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.00s
```
（`runs-summary/mut-span1-raise.txt`。）4 条全红，说明宽护的跨度有测试钉着，这一处没骗过。没打中。数据单元跨度就是 1 槽、以上那条变异对单槽落点无害，之所以还红是因为复活集里有跨度 ≥ 2 的（谱系片之外的单元），这一点正好让它红。

## 三、K3 第十三块 借之前早记丢了（GPU）

列卡、丢没丢、poll 怎么记都在 harness 档（`GpuMemoryLedger::take_in_the_devices_kept_before_booking`，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1443`），卡池那一侧（`gpu_unit_checks.rs:1339` `take_in_the_devices_found_idle`）只接四样。我在 harness 档的测试模块里用它现成的假卡 `CardUnderTheLedger` 搭了两条随机装置，不要卡。

### K3-1 / K5-1 账多给：随机模型

`opus_lm_random_ledger_model`：一张假卡上随机走借之前那一步、借、取设备（丢了重起）、建、还（随机没等成）、让设备丢、放掉丢了的设备。每步后拿账给的（`card_rooms_mebibytes_now`）比卡上真能再给的（空闲减驱动还要长的、减活着借据没建的），多给就记。5000 种子 × 400 步：

```
OPUS_LM_LEDGER seeds=5000 steps=400 bookings=352533 refusals=91662 under_refusals=57479 early_marks_seen=34986 overfull_seeds=0 over_steps=0 worst_over_mebibytes=0 over_steps_while_the_ledger_does_not_know_the_loss=53350
```
（`runs-summary/ledger-random-model.txt`。）`over_steps=0`：账知道设备丢了（早记过，或重起时 `prepare_the_driver_share_before_starting_the_device` 判过）之后，没有一步账给的大于卡上真能再给的。`early_marks_seen=34986`：借之前那一步真的在设备丢了、有没等成借据时记了账，这一支走到了。`overfull_seeds=0`：没有一个种子的账让假卡物理上超过它的总量。

唯一数字上「多给」的是 `over_steps_while_the_ledger_does_not_know_the_loss=53350`：设备刚丢、账还没被告知（还没走到借之前那一步、也还没重起）那一瞬间，账按旧读数算、给得比真能给的多。这一段由重起时 `prepare` 现读一次拦（`crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 1341 行 `driver_share_of_a_new_device_fits_now`）：那一刻放不下就不起、交显存不够。这正是这一块要补的 K4-2 的反面——早记把这一段缩短，没加长。没打中多给。

### K5-2 少给：按账的状态分类

`under_refusals=57479`（卡上真放得下而账拒）按账此刻的状态分：

```
OPUS_LM_UNDER_BY_STATE {"DeviceLostNotStartedAgain/kept_listed=0": 12270, "DeviceStarted/kept_listed=0": 1569, "DeviceStarted/kept_listed=1": 1489, "DeviceStartedAgainWhileTheLostDeviceRemains/kept_listed=0": 37491, "DeviceStartedAgainWhileTheLostDeviceRemains/kept_listed=1": 4660}
```
（`runs-summary/ledger-random-model.txt`。）54421 次（95%）在 `DeviceLostNotStartedAgain` 与 `DeviceStartedAgainWhileTheLostDeviceRemains` 两态——丢了的设备连它的缓冲可能还占着卡，这个进程占着的分不出哪一份是谁的，照「读不到实际占着的」算 0（`crates/singlefs-harness/src/gpu_memory_ledger.rs:215` `| DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => return 0,`），只会少借。这是 D24 口径下「宁可少借、不多给」，用户已认的方向，不算打中。剩 3058 次在 `DeviceStarted`：1489 次有列出来的没等成借据（它的缓冲可能还在卡上、照全额记，合理），1569 次没有——这 1569 次的成因我没逐条查（随机模型里借据建了多少由随机定，可能是借了没建满、账按全额记），单列给主 agent，不当打中。

### K3-2 / K3-3 并发：早记与重起抢

`opus_lm_early_lost_mark_racing_the_restart`：几条线程在一张假卡上同时走借之前那一步、借、取设备（`DeviceStartedOnDemand::started_or_start`，丢了重起）、随机让设备丢、还。核两样：没丢的设备从没被账记成丢了；丢了、还在用、有没等成借据的那一台，再走一次借之前那一步总会被记上。

```
OPUS_LM_STRESS threads=8 rounds=2000 bookings=16000 restarts=826 restart_refused=0 early_marks_seen=560 wrongly_lost=0 lost_and_still_in_use_with_kept_after_one_more_pass=false
OPUS_LM_STRESS threads=14 rounds=5000 bookings=70000 restarts=3564 restart_refused=0 early_marks_seen=2897 wrongly_lost=0 lost_and_still_in_use_with_kept_after_one_more_pass=false
```
（`runs-summary/ledger-early-mark-stress.txt`。）`wrongly_lost=0`：14 线程 5000 轮、重起 3564 次里，没有一次把没丢的设备记成账上丢了那一堆（`take_in_that_the_device_was_found_lost` 的 `device_in_use == Some(self.device)` 守卫，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1676`，卡池看到重起之前留着的那一台时不动新的一台）。收尾那一行 `false`：这台丢了、还在用的设备上没等成的借据，收尾再走一次借之前那一步之后不再记着。没打中把新设备记成丢了、也没打中丢了的记不上。

## 四、K4 第十三块判别力（GPU）：两条扫源码用例被骗过

两条扫源码用例不要卡：`gpu_unit_checks.rs:2408` `the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking`（逐字核 `take_in_the_devices_found_idle` 函数体里五段文本各恰好一处、取设备之后三个闭包不多不少）与 `crash_amplification.rs:4849` `every_layer_handed_the_stop_request_hands_it_on_or_asks_it`（核六层收叫停参数的函数各把它整名用上一次、层序对）。我把这两条用例的判法逐字搬进一个小程序 `src/scan_probe.rs`（读的源码换成命令行给的文件），对冻结副本原件与几种变体跑（`src/variants.py` 生成变体）：

```
SCAN_GREEN crash variants/crash-0-original.rs
SCAN_RED   crash variants/crash-1-row-never-stop.rs
SCAN_GREEN crash variants/crash-2-fool-never-stop-naming-it.rs
SCAN_GREEN crash variants/crash-3-fool-never-ask-naming-it.rs
SCAN_GREEN gpu   variants/gpu-0-original.rs
SCAN_RED   gpu   variants/gpu-1-row-never-lost.rs
SCAN_GREEN gpu   variants/gpu-2-fool-never-call-the-ledger.rs
SCAN_GREEN gpu   variants/gpu-3-fool-poll-always-idle.rs
```
（`runs-summary/scan-variants.txt`。）原件绿、变异表里那两行（`crash-1` 把 `check_blocks_where_until` 往下交的叫停换成 `&|| false`、`gpu-1` 把丢没丢闭包换成 `|_started| false`）红 —— 这两条是实现员登记的变异，确实红。但四种「改坏行为、照绿」的写法骗过了它：

- `crash-2`：`judge_my_blocks_on_gpu_cards` 往下交的叫停换成 `&|| { let _ = stop_is_requested; false }`（永不叫停，但闭包体里提了一次 `stop_is_requested` 这个名字）。用例数「整名用上几次」，这个假用法让它数到 ≥ 1，绿。行为上这一层再也不叫停下游。
- `crash-3`：领块前 `if stop_is_requested() {` 换成 `if false && stop_is_requested() {`（短路，永不 break，但名字还在）。同样骗过「整名用上一次」。
- `gpu-2`：`take_in_the_devices_found_idle` 整个函数体包进 `if false { … }`（借之前那一步永不调账）。用例只核函数体里那几段文本各出现一次，`if false {` 包在外面不影响文本计数，绿。卡池借之前再也不告诉账留着的设备丢没丢——正是这一块要补的 K4-2 行为被关掉。
- `gpu-3`：函数体开头放一个同名 `fn device_found_idle_by_a_poll<T>(_polled: &T) -> bool { true }`，遮住外层那个。poll 的结局永远「队列空了」。用例核的那段文本（`device_found_idle_by_a_poll(&std::panic::catch_unwind(…))`）原样还在，绿。

四种在副本上都编得过（`$WORK/copy` 加 `.cargo/config.toml` 后 `cargo check --offline -p singlefs-checker-tier --features gpu,verdict-store --lib`）：`crash-2`、`crash-3` 连警告都没有（名字被用到了），`gpu-2`、`gpu-3` 各只报一条没用的函数/导入警告（变异跑道不带 `-D warnings`，第十块判决第二节 K5「下游换成永不叫停 cargo check 只报 unused variable 警告」同一条）。

四句自检：
- 分辨臂？这是判别力格，问的是用例拦不拦得住改回去。原件绿、行为改坏（crash-2/3、gpu-2/3）也绿 ⇒ 这四种改坏都不被用例拦。
- 判别子观测不到？不适用。
- 哪个分句？两条用例都按「文本在不在、名字出现几次」判，不按「值真的往下传、真的被 break 用」判。`crash` 那条用 `whole_name_uses_of_the_stop_request`（`crash_amplification.rs:4830`，只数整名出现），`gpu` 那条数文本片段出现次数（`gpu_unit_checks.rs:2451` `body_without_comments_and_spaces.matches(expected).count()`）。一个提了名字但永不生效的假用法、一个 `if false` 外包、一个同名遮蔽，都满足文本判据、不满足行为。
- 改法在这一格中不中？要拦住这四种得换成行为判据（崩溃放量真造一次叫停、看每层真停；借之前真让设备丢、看账真改）——那要起设备或崩溃枚举，是重型测试，这一轮不跑；换判据是新一轮。这是「判别子观测不到」那一类：扫源码这层看得到的只有文本，看不到行为。

打中。实现员自己在 `gpu_unit_checks.rs` 那条用例的文档里写了「列卡、丢没丢、poll 的结局怎么记都在 harness 档（不要卡就测得到）」，harness 档那几条行为用例（`a_lease_kept_on_a_device_found_lost_before_booking_…`、`a_device_found_lost_with_its_buffers_still_on_the_card_…`、`a_device_found_lost_after_its_restart_…`）确实是行为判据、我没骗过它们（随机模型与压测也印证账的行为对）。被骗过的是**卡池那一侧「真的把这四样接进账」**这一道只有扫源码用例守着：`gpu-2`（永不调账）、`gpu-3`（poll 恒真）把卡池那一侧改瘫，harness 档那几条照样绿（它们直接调账、不经卡池），只有扫源码用例该拦、而拦不住。

## 五、没打中的形状

- **K1-1 三态判错**：6 种 R（覆盖写、建 inode、顺序写、回退浅、回退深、删除）× 7 组故障（账三指针、谱系片、全部单元、非谱系打包单元、按序第 0/1/2 片打包单元、R_old 根槽、R_old 账）× 两档几何（384 / 1536）。tri 在每一格 0 红（`runs-summary/agg-r-kinds-b5.txt`）。想造「该不知道判成肯定不是」：让 R 的某一片打包单元读不出而别的几片都读得出、里面没有谱系记录——tri 仍判「不知道」（`racct,rnonlin`、`racct,rpk0/1/2` 都把 iso 抬到 116，与 racct 的 42 不同，说明没被当成「肯定不是」）。想造「该肯定不是判成不知道」白多护：R 不是回退（建 inode / 顺序写 / 覆盖写）而某片打包单元读不出，tri 判「不知道」多护（18→116）——这是用户认了的「不知道那一支多护」，不是判错。R 是回退而只有非谱系片或某一片打包单元读不出：tri 照样从谱系片认出回退、交 42 槽（`racct,rpk0/1/2` 在 `r=b1/b3` 下 iso=42，与认得出同数），没被「一片读不出就当不知道」带偏——因为谱系片这一片读得出、认出了就不看别的（冻结副本第 2272 行「认出了回退就不看别的几片」）。
- **K1-2 宽护罩不住 R 经别处继承的单元**：tri 与 nodbl 两臂，R 还在环里时 C 里 R 那一槽露出来的槽数（三种挡位都没置、发得出去的）逐段为 0（`src/pair.py` 的 `tri_exposed`/`nodbl_exposed` 两列在 b3/b4 全 0）。想复刻第十块攻方「丙并中间」那一形（R 与所选那一版之间还有 Q、Q 的单元只在 Q 那一槽）：把回退当 Q（`mid=b1`、Q 是回退）、R 建 inode、Q 的根与账读回全零——挂载落到 C554 那一族（Q 看不见，不是 tri 的事）或被恢复拒（`q-side-t0-t1.txt`：`cfgq,qroot,qlast,racct` 两形 `iso_end=30`、`hit=false`，Q 侧账或谱系片读不出时 `mount=Err:Recovery(UnitUnreadable…)`）。没造出 tri 的宽护漏掉 R 该护的单元。
- **K3 多给 / 并发**：见第三节，随机模型 5000 种子、压测 14 线程 5000 轮都没造出账知道丢了之后多给、也没造出并发把新设备记成丢了或丢了的记不上。
- **K5-2 卡放得下却整趟弃卡 / 并发被压**：随机模型里借不到就 continue、不弃整趟、不压并发（模型没模拟「配了 GPU 放不下判红」那一条，那要 `lease_card_or_report_not_run` 的 panic 路径，没搭）。

## 六、这条腿自己的限度

- 所有数都在攻方副本上，不进 kb。`nodbl`（core）、随机模型与压测（GPU）都是我搭的，被攻过零轮；`nodbl` 的正确性（会不会把该护的漏了）我没搭 I-7.4 用例核，只量了它消掉双扣、不卡死。
- 崩溃状态没枚举：故障全是「挂载那一刻读回全零」，崩溃点固定在 R 那次发布落盘之后（探针 `run_one` 把写流截到 R 的根槽写），没走层 0 的 `enumerate_layer0`，所以「R 那次发布写到半途崩」的那些崩溃状态一个都没覆盖。这一轮只攻合法恢复态下的宽护量与记账，不攻崩溃点正确性。
- GPU 没起真卡：随机模型与压测用假卡 `CardUnderTheLedger`，`is_lost`、`found_idle_by_a_poll`、丢设备都是模型里的布尔，不是 wgpu 真丢设备；真卡上 poll 丢了的设备 panic、`nvidia-smi` 那 1.3 毫秒窗口都没碰（用户已认）。
- K4 的四种骗过写法只证了「扫源码用例拦不住」与「副本上编得过」，没证「真跑崩溃放量 / 真起卡池时下游真不叫停、卡池真不告诉账」——那要重型测试，没跑；是推的后果（文本判据下行为可被改坏）。
- H1 卡死那一形我按「先释放一大批再认不出」造，释放用顺序写 `S`（一次 20 个数据单元）堆出来；真实负载里认不出回退（谱系片或 R_old 读不出）与「刚释放一大批」同时发生的概率我没估。
- 没跑任何门禁阶段、没跑 checker 档、没编 E171、没跑入库 harness 文件的全量（只跑了带那一处跨度变异的那一次，4 红）。

## 七、没做什么（固定）

不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数。草稿目录 `~/.local/share/singlefs/work/m3-batch10z-lm-code-r1-opus/` 里的仓副本 `copy/`（带 `.cargo/config.toml`）、编译目录 `target/`、日志 `runs/`、辅助脚本 `src/`、扫源码变体 `scan/` 交回后删，删法见交回消息。
