# unreadable-at-mount-r2 云端攻方（Opus）报告：Y1、Y2（2026-09-28）

攻击面：正文「Y1　连挂轨迹」「Y2　零轮变体在第一轮网格上的数」。全部数只在冻结副本的拷贝上量过（副本上的数，不是入库装置上的数）。
甲-读错、(b)-从树表现算 两份改法是我按正文第一节的定义写的一种实现：只在我的模型上量过、被攻过零轮。

## 复跑

模型目录 `research/prompts/unreadable-at-mount-r2-opus-model/`。一条命令从冻结副本重建五份副本（今天 `base`、甲 `jia`、甲-读错 `jiaeio`、(a) `a`、(b)-从树表现算 `b`）、打 diff、编译、跑网格与轨迹原型、汇总：

```
bash research/scripts/capped.sh 16 bash research/scripts/run-with-memory-cap.sh 16G bash research/prompts/unreadable-at-mount-r2-opus-model/rerun.sh /tmp/claude-1000/unreadable-at-mount-r2/tree <新的工作目录>
```

- 第一轮的 `common-mod.rs.diff`、`candidate-jia.diff`、`candidate-a.diff` 与两张网格用例原样从 `research/prompts/unreadable-at-mount-r1-opus-model/` 取，不拷进这个目录。
- `common-mod-memory.rs.diff`：`tests/common/mod.rs` 的镜像从文件改成内存稀疏盘（`Recorded = RecordingBlockDevice<SparseBlockDevice>`，「重开」从带内容的录制流重建）。改完之后今天、甲、(a) 三份副本上两张网格的逐行输出与第一轮 `runs/` 里的逐行相同（`name=c331grid`/`c331v4` 行 1959 行、`name=c393grid` 行 98 行，排序后 `diff` 为空），这一改动不改结局。
- `candidate-jiaeio-over-jia.diff`（打在甲之上）、`candidate-b-over-a.diff`（打在 (a) 之上）是这一轮新写的两份。
- `r2-opus-trajectory-proto/` 是 Y1 的轨迹原型（`[workspace]` 成员加一行，`cargo build -p r2-opus-trajectory-proto`，参数 `torn|single_slot|c393_single|c393_double`）。
- 我这一次实际是分批跑的：`run-y2.sh`（网格，顺序跑，每条经内存包装）与 `run-y1-batch.sh`（轨迹，两批并行，整条脚本经内存包装、每件写 `.rc`）；批的原样输出在 `runs/run-y2.out`、`runs/batch1.out`、`runs/batch2.out`。
- 汇总：`c331-summary.txt`、`c393-summary.txt` 是第一轮的 `summarize_c331.py`、`summarize_c393.py` 在 `runs/` 上的输出；`trajectory-summary.txt` 是 `summarize_trajectories.py` 的输出。

挂钟（这一次）：C331 网格每份 150–209 秒，C393 网格每份 8–10 秒；轨迹原型每件 170–405 秒（`runs/*-traj-*.log` 末行 `name=r2traj-total … seconds=`）。开跑前先单跑一件 `base:torn`（352 秒）估的：全量 5 份 × 4 部分约 20 件，并行 14 件、5 件两批，估 15 分钟以内，没缩取样。

模型目录每个文件的 sha256（`cd` 到模型目录后 `find . -type f | sort | xargs sha256sum`）见本报告末节「模型目录 sha256」。

## 各格判定一览

| 格 | 形态 | 结果 | 判定 |
|---|---|---|---|
| Y1 撕裂轮换 | 甲 | 层 0 枚举到的 13 个状态里，D 的两写有一写撕裂的 5 个状态（含第一轮三格）：6 次挂载 × 6 种用户动作全只读，拒可写的每一次录制流 0 步、两块盘逐字节不变，撕裂那一槽 6 次之后仍自证不过 | **量实：一次断电之后永远只读**（N=6 以内，量过） |
| Y1 撕裂轮换 | 甲-读错 | 同 13 个状态：第 1 次就可写，第 1 次挂载之后四槽全自证过 | 不只读，第 1 次靠取号那一写把撕裂那一槽盖掉走出 |
| Y1 单槽持久读坏 | 甲-读错 | 盘 0 一槽系统配置设备报读错（`DeviceError`），别的一概读得出、C 已确认：6 次 × 6 种动作全只读，盘上不变 | **打中：单故障之后永远只读**（今天、(a)、(b) 全可写） |
| Y1 单槽持久读坏 | 甲 | 同一格（`DeviceError` 与 `Zeros` 都）：永远只读 | 同上，甲另外在 `Zeros` 形也永远只读 |
| Y1 账持续读不出 | (a) | 树表 / 分配记录树根 / 其下节点 / 两样，`single`、`double` 两个起点，两种读回，两种坏扇区模型：全部 6 次 × 6 种动作只读，盘上不变，被抛弃根一直在根环里 | **量实：永远只读** |
| Y1 账持续读不出 | (b)-从树表现算 | 分配记录树那几格：第 1 次就可写、之后每次都可写；被抛弃根在第 3 或第 4 次挂载之后离开根环（每次写 3 次、或正常卸载收尾的），每次 0 或 1 次写又崩溃收尾的在 6 次以内没离开；根环里时它的单元一个没被盖、I-7.4 一次都没红；树表那几格：同 (a) 永远只读 | 分配记录树那一指称走得出；树表那一指称永远只读 |
| Y2 C331 | 甲-读错 | 1959 段：丢写 120（**全是 `Zeros` 形**：`AllFour` 56、`NewestOnBoth` 64），多拒 284（全是 `DeviceError` 形）；撕裂三格不拒 | 按跑前判据进表；另见 Y1 那一格 |
| Y2 C331 V4 | 甲-读错 的取号扩展 | V4 144 段里丢写 10，全是 `Zeros` 形；今天丢 20，`DeviceError` 那 10 段罩住 | 只罩 `DeviceError` 那一半 |
| Y2 C393 | (b)-从树表现算 | 98 段：复用 0、拒可写 14（全是树表读不出的格），(a) 拒的 28 格里 14 格改成可写且不复用 | 复用 0，进不进表看 Y3 |
| 旁及 | 坏扇区不重映射（`Sticky`）时 | (b) 一条轨迹：被抛弃根离开根环之后，它那个读不出的槽回到空闲池、被新单元占用，第 4 次起可写、只读都挂不上 | 不分辨臂（推的），单列 |

## Y1　连挂轨迹

### 1. 起点、用户动作与记什么

- **撕裂轮换（甲、甲-读错 的格）走层 0 的枚举域**：原型在内存盘上造第一轮 `standard` 那段历史（A (1,3)；实例 2 写行 4、暖机 5、B (2,6)、C (2,7)，全部确认），再在同一个进程里发 D (2,8) 并录下这条流。D 的流切成 4 段（`runs/base-traj-torn.log` 原样行）：

```
name=r2traj-segment stream=d_publish segment=0 writes=28 kinds=28 writes, kinds {UnitWrite}
name=r2traj-segment stream=d_publish segment=1 writes=2 kinds=28:JournalRecord@d0+16805888,29:JournalRecord@d1+16805888
name=r2traj-segment stream=d_publish segment=2 writes=1 kinds=30:RootRecordFua@d0+7348224
name=r2traj-segment stream=d_publish segment=3 writes=2 kinds=31:SystemConfigurationSlot@d0+4096:tearable,32:SystemConfigurationSlot@d1+4096:tearable
name=r2traj-plan stream=d_publish_after_segment_0 writes=5 segments=3 closed_form_state_count=8 layer0_state_count_with_torn=13 whole_d_publish_layer0_state_count_with_torn=268435468 d=(2,8)
name=r2traj-enumerated stream=d_publish_after_segment_0 states_enumerated=13 states_observed=13 red_states=0
```

  整条流 2.68×10⁸ 个状态，超过约 10⁶，不跑。我缩的是历史长度：第 0 段（28 次 COW 单元写）整段当作已持久并进起点，剩下 3 段 5 次写交 `enumerate_layer0_selecting_versions_observing_each_state` 每段都展开（原地覆写取三态），13 个状态一个不落，每个状态在五份副本的产品代码上各跑一遍轨迹。缩前 268435468 个状态、缩后 13 个；丢掉的是「D 的单元写只落了一部分」那些状态，在那些状态里 D 的根、记录与系统配置轮换都没发，盘上见证 C 的系统配置槽不动（推的：按段序推，没跑）。第一轮手造的三格就在这 13 个里：`torn_d0` = `landing=111T0`，`torn_both` = `111TT`，`torn_d0_d1_persisted` = `111T1`（`landing` 按 5 次写逐个写：1 新、0 旧、T 撕裂，原型按盘上字节比出来的）。另两个撕裂状态 `1110T`、`1111T` 是盘 1 撕、盘 0 没持久或持久，第一轮没造。这一腿原型全量合计 13 × 5 = 65 个状态（`name=r2traj-total … layer0_states_enumerated_by_this_run=13`，五份副本各 13）。
- **单槽持久读坏**：C 确认返回之后的盘面（`name=r2traj-start start=single_slot sc=d0[g10t7,g9t6]d1[g10t7,g9t6]`），盘 0 较旧那一槽或最新那一槽一直读坏，读回块设备错或全 0。这一格第一轮甲的 719 格多拒里就有（「一段读故障的多拒」），这一轮给它连挂。
- **被抛弃根的账持续读不出（(a)、(b) 的格）**：第一轮 `c393_candidates_grid.rs` 的两个起点原样（`single` 一条被抛弃根 (2,8)；`double` 两条 (2,8)、(4,14)），账（树表、分配记录树根、分配记录树根之下一个节点、树表与分配记录树根两样）每次读都坏，读回块设备错或全 0；`double` 上分别坏 C、D、两条都坏（树表、分配记录树根两种）；另各有一条账读得出的对照。
- **坏扇区之后被写过怎么办**两种都跑：`Sticky`（写了照样读坏，不重映射）、`HealsOnWrite`（被写罩到就好，盘写时重映射）。读故障是设备层的，盘上字节没坏；拒可写之后一次写都没有，两种模型在拒了的轨迹上逐行相同。
- **用户动作放开扫**，只固定起点与故障：每次挂载先试可写；做成了就发 k 次覆盖写（k = 0、1、3），再按两种之一收尾（进程崩溃 / 正常卸载 `unmount`）；可写被拒就试只读挂载。6 种组合 × 每条轨迹 6 次挂载（N = 6）。
- **每次挂载记**：可写成没成（拒因前 110 字）；拒了的，可写那一试之后录制流多了几步（`ops_by_writable_attempt`）、两块盘逐字节变没变，只读挂载成没成、择到哪条根、之后又多几步、变没变；每次之后盘上四个系统配置槽的世代号与 tail（自证不过记 `X`）；被抛弃根还在不在根环（`readable_roots` 里有没有它）、它那次发布写出的单元被盖了几个、池级 checker 的 I-7.4。

「只读挂载不写盘」这一句我没当前提：每一次只读挂载前后都比了录制流步数与两块盘的全部扇区（`MemoryPool` 相等），结论见下一节。附带一句代码现状（观测，不是条款）：只读挂载的入口只收一个读者（「pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, MountReadOnlyFailure> {」（`crates/singlefs-core/src/mounted_read.rs:553`）），类型上拿不到写口。

### 2. 结果（`trajectory-summary.txt`，每格是 6 种用户动作下出现过的模式串与次数；W 可写、R 只读、X 都挂不上）

撕裂轮换，五份副本（今天、甲、甲-读错、(a)、(b)），D 的两写有撕裂的 5 个状态：

| 状态 | 今天 | 甲 | 甲-读错 | (a) | (b) |
|---|---|---|---|---|---|
| 5 `111T0`（第一轮 `torn_d0`） | WWWWWW×6 | **RRRRRR×6**，撕裂槽始终 `X` | WWWWWW×6 | WWWWWW×6 | WWWWWW×6 |
| 7 `1110T` | WWWWWW×6 | **RRRRRR×6** | WWWWWW×6 | WWWWWW×6 | WWWWWW×6 |
| 8 `111TT`（第一轮 `torn_both`） | WWWWWW×6 | **RRRRRR×6** | WWWWWW×6 | WWWWWW×6 | WWWWWW×6 |
| 9 `1111T` | WWWWWW×6 | **RRRRRR×6** | WWWWWW×6 | WWWWWW×6 | WWWWWW×6 |
| 11 `111T1`（第一轮 `torn_d0_d1_persisted`） | WWWWWW×6 | **RRRRRR×6** | WWWWWW×6 | WWWWWW×6 | WWWWWW×6 |

没撕裂的 8 个状态五份副本全是 WWWWWW×6。甲那 5 格 30 条轨迹：`refusals_left_the_disk_unchanged=true`、`every_system_configuration_slot_verifies_from_mount=none`（汇总行原样字段）。

单槽持久读坏（C 已确认，别的都读得出）：

| 槽 × 读回 | 今天 | 甲 | 甲-读错 | (a) | (b) |
|---|---|---|---|---|---|
| 盘 0 较旧槽 × `DeviceError` | WWWWWW×6 | RRRRRR×6 | **RRRRRR×6** | WWWWWW×6 | WWWWWW×6 |
| 盘 0 最新槽 × `DeviceError` | WWWWWW×6 | RRRRRR×6 | **RRRRRR×6** | WWWWWW×6 | WWWWWW×6 |
| 盘 0 较旧槽 × `Zeros` | WWWWWW×6 | RRRRRR×6 | WWWWWW×6 | WWWWWW×6 | WWWWWW×6 |
| 盘 0 最新槽 × `Zeros` | WWWWWW×6 | RRRRRR×6 | WWWWWW×6 | WWWWWW×6 | WWWWWW×6 |

`Sticky` 与 `HealsOnWrite` 两种模型在这四行上结果相同（各 6 条轨迹）。

被抛弃根的账持续读不出（`single` 16 格 + `double` 24 格，已含两种读回 × 两种坏扇区模型，另各一条对照；`jia` 那一列造不出起点，见第 5 节）：

| 账 | 今天 | 甲-读错 | (a) | (b) |
|---|---|---|---|---|
| 树表（`single`、`double` 的 C / D / 两条） | W×6，被抛弃根的单元在根环里被盖、I-7.4 红 | 同今天 | **RRRRRR×6** | **RRRRRR×6** |
| 树表与分配记录树根两样（`single`） | 同上 | 同今天 | **RRRRRR×6** | **RRRRRR×6** |
| 分配记录树根（`single`、`double` 的 C / D / 两条） | 同上 | 同今天 | **RRRRRR×6** | WWWWWW×6，复用 0、I-7.4 不红 |
| 分配记录树根之下一个节点（`single`） | 同上 | 同今天 | **RRRRRR×6** | `HealsOnWrite`：WWWWWW×6；`Sticky`：5 条 WWWWWW、1 条 WWWXXX（见第 4 节） |
| 对照（账读得出） | WWWWWW×6 | 同 | WWWWWW×6 | WWWWWW×6 |

今天与甲-读错两列在这 42 个起点上是同一份轨迹（`runs/base-traj-c393_*.log` 与 `runs/jiaeio-traj-c393_*.log` 去掉末行 `name=r2traj-total` 之后 `diff` 为空），「根环里被盖」的行数：今天 624 行、甲-读错 624 行、(a) 0 行、(b) 0 行（数的是 `abN=(…):in_ring=true:overwritten=` 后面非 0 的挂载行，两份 c393 部分合计 1512 行）。

### 3. 原样行

甲在撕裂轮换 `111TT`（第一轮 `torn_both`）上，第 1 次与第 6 次挂载（每次写 3 次、正常卸载那一种用户动作；别的五种逐字相同，因为可写一次都没做成）。甲-读错同一状态第 1 次（`runs/jia-traj-torn.log`、`runs/jiaeio-traj-torn.log`）：

```
jia: name=r2traj start=torn_rotation state=8 landing=111TT policy=k3Unmount mount=1 writable=refused[NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { fir] ops_by_writable_attempt=0 disk_changed_by_writable_attempt=false read_only=ok(2,8) ops_after_read_only=0 disk_changed_by_read_only=false bad_ranges_left=0 after: sc=d0[g10t7,X]d1[g10t7,X]
jia: name=r2traj start=torn_rotation state=8 landing=111TT policy=k3Unmount mount=6 writable=refused[NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { fir] ops_by_writable_attempt=0 disk_changed_by_writable_attempt=false read_only=ok(2,8) ops_after_read_only=0 disk_changed_by_read_only=false bad_ranges_left=0 after: sc=d0[g10t7,X]d1[g10t7,X]
jiaeio: name=r2traj start=torn_rotation state=8 landing=111TT policy=k0Crash mount=1 writable=ok instance=3 effective=(2,8) isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=0 published=[] end=crash bad_ranges_left=0 after: sc=d0[g12t9,g13t10]d1[g12t9,g13t10]
```

甲-读错 走出来靠的是可写挂载自己的写：取号、写行、暖机各轮换一次系统配置，第 1 次挂载之后四槽都是新世代（`g12`、`g13`），撕裂那一槽被盖掉；一次都不写的用户动作（k = 0、崩溃收尾）同样走出。

甲-读错 在单槽持久读坏（盘 0 较旧槽、`DeviceError`、写时重映射）上第 1 次与第 6 次（`runs/jiaeio-traj-single_slot.log`）：

```
jiaeio: name=r2traj start=single_slot slot=OlderOnDevice0 read_back=DeviceError after_a_write=HealsOnWrite policy=k3Unmount mount=1 writable=refused[NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { fir] ops_by_writable_attempt=0 disk_changed_by_writable_attempt=false read_only=ok(2,7) ops_after_read_only=0 disk_changed_by_read_only=false bad_ranges_left=1 after: sc=d0[g10t7,g9t6]d1[g10t7,g9t6]
jiaeio: name=r2traj start=single_slot slot=OlderOnDevice0 read_back=DeviceError after_a_write=HealsOnWrite policy=k3Unmount mount=6 writable=refused[NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { fir] ops_by_writable_attempt=0 disk_changed_by_writable_attempt=false read_only=ok(2,7) ops_after_read_only=0 disk_changed_by_read_only=false bad_ranges_left=1 after: sc=d0[g10t7,g9t6]d1[g10t7,g9t6]
```

盘上四槽字节都自证得过（`sc=` 里没有 `X`），坏的只是盘 0 那一槽的读；写时重映射的盘也救不了它，因为拒在取号之前，一次写都没发（`bad_ranges_left=1` 从头到尾没变）。

(a) 与 (b) 在「分配记录树根持续读不出」（`single`、`DeviceError`、写时重映射，每次写 3 次、正常卸载）上（`runs/a-traj-c393_single.log`、`runs/b-traj-c393_single.log`）：

```
a: name=r2traj start=c393-single which=[0] account=AllocationRecordTreeRoot read_back=DeviceError after_a_write=HealsOnWrite bad_ranges=2 policy=k3Unmount mount=6 writable=refused[NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot { abandoned_root: RollbackTarget { instance: ] ops_by_writable_attempt=0 disk_changed_by_writable_attempt=false read_only=ok(3,10) ops_after_read_only=0 disk_changed_by_read_only=false bad_ranges_left=2 after: sc=d0[g12t9,g13t10]d1[g12t9,g13t10] ab0=(2,8):in_ring=true:overwritten=0/13 I-7.4=Holds
b: name=r2traj start=c393-single which=[0] account=AllocationRecordTreeRoot read_back=DeviceError after_a_write=HealsOnWrite bad_ranges=2 policy=k3Unmount mount=1 writable=ok instance=4 effective=(3,10) isolated=[(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)] abandoned_roots_unreadable=1 published=[14,15,16] end=unmount_ok(txg 19) bad_ranges_left=2 after: sc=d0[g24t19,g23t18]d1[g24t19,g23t18] ab0=(2,8):in_ring=true:overwritten=0/13 I-7.4=Holds
b: name=r2traj start=c393-single which=[0] account=AllocationRecordTreeRoot read_back=DeviceError after_a_write=HealsOnWrite bad_ranges=2 policy=k3Unmount mount=3 writable=ok instance=6 effective=(5,28) isolated=[(DeviceIdentity(0), 16), (DeviceIdentity(1), 16)] abandoned_roots_unreadable=1 published=[32,33,34] end=unmount_ok(txg 37) bad_ranges_left=2 after: sc=d0[g46t37,g45t36]d1[g46t37,g45t36] ab0=(2,8):in_ring=false:overwritten=6/13 I-7.4=Holds
b: name=r2traj start=c393-single which=[0] account=AllocationRecordTreeRoot read_back=DeviceError after_a_write=HealsOnWrite bad_ranges=2 policy=k3Unmount mount=4 writable=ok instance=7 effective=(6,37) isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=0 published=[41,42,43] end=unmount_ok(txg 46) bad_ranges_left=2 after: sc=d0[g56t45,g57t46]d1[g56t45,g57t46] ab0=(2,8):in_ring=false:overwritten=6/13 I-7.4=Holds
```

(b) 第 3 次挂载结束时被抛弃根的根槽已被盖掉（`in_ring=false`）、它那次发布写出的 13 个单元里 6 个已被盖，I-7.4 不红（轨迹原型只在每次挂载结束时看一次，这一次挂载里两件事的先后没逐次发布核；逐次发布核的是 Y2 的网格，(b) 在那里到 txg 31 复用 0）；第 4 次挂载时影子账已经看不到它，隔离归 0、`abandoned_roots_unreadable=0`。(a) 在同一格第 6 次仍拒，被抛弃根一直在根环里。

### 4. 旁及：坏扇区不重映射时，离开根环之后那一槽回到空闲池

(b) 在「分配记录树根之下一个节点」持续读坏、坏扇区不重映射（`Sticky`）、每次写 3 次并正常卸载的那一条轨迹上，第 3 次挂载之后被抛弃根离开根环，第 4 次起可写、只读都挂不上（`runs/b-traj-c393_single.log`）：

```
name=r2traj start=c393-single which=[0] account=AllocationRecordTreeNodeBelowTheRoot read_back=DeviceError after_a_write=Sticky bad_ranges=2 policy=k3Unmount mount=4 writable=refused[Recovery(UnitUnreadable { slot: SlotNumber(50335) })] ops_by_writable_attempt=0 disk_changed_by_writable_attempt=false read_only=err(Open(Walk(UnitUnreadable { slot: SlotNumber(50335) }))) ops_after_read_only=0 disk_changed_by_read_only=false bad_ranges_left=2 after: sc=d0[g46t37,g45t36]d1[g46t37,g45t36] ab0=(2,8):in_ring=false:overwritten=6/13 I-7.4=Holds
```

读坏的那一槽离开隔离之后被第 3 次挂载里的发布拿去放了现行那一版的单元（推的：按第 4 次读不回的槽号推，没逐次发布记分配），两份都落在不重映射的坏扇区上，现行那一版就读不回来。`Zeros` 读回同形（各 3 行 `read_only=err`）。今天与甲-读错同一格同一动作没撞上（`X_mounts=0`，两份 c393 部分 1512 行里数的），那是因为今天从第 1 次挂载起就不隔离那一槽、它被别的东西占了还是没被占取决于分配次序——**这一格不分辨臂**（推的：凡是「读不出的槽迟早回到空闲池」的形态都会碰上，只是早晚不同；没为今天另造一段历史去撞）。它说的是另一件事：分配器不记得哪些槽读坏过。写时重映射的模型下同一格 6 条轨迹全 W。

### 5. jia 副本上 C393 起点造不出

第一轮的起点造法（`common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`）要在每块盘清零最新那一槽系统配置之后可写挂载一次，甲把这一形当判不出拒了，原型在造起点那一步 panic（`runs/jia-traj-c393_single.log`、`runs/jia-traj-c393_double.log`，`b1-13.rc`、批 2 第 5 件退出码 101）。所以 (a)、(b) 那几格没有甲一列；甲在这几格上的轨迹与今天相同是推的（甲只动系统配置那一判，不动影子账）。这一点本身也是一格：**「每块盘一槽系统配置坏」加一次崩溃恢复，甲拒可写**，与撕裂同形。

### 6. 打中之后的四句

| 打中 | 分不分辨臂 | 系统当时看不看得到判别它的东西 | 满足判据字面的哪一个分句 | 跑前条款给的改法在这几格上还中不中 |
|---|---|---|---|---|
| 甲：撕裂轮换之后永远只读（5 个层 0 状态 × 6 种动作） | 分：只有甲只读，今天、甲-读错、(a)、(b) 第 1 次就可写 | 看得到「这一槽自证不过」，看不到它是撕裂还是静默回坏（第一轮 K1 的判别子问题）；`111T0`、`1110T` 两个状态里另一块盘两槽都自证过，那块盘上就有 C 的见证（推的，见第 Y2 节第 3 条），甲没用它 | 正文第五节第一条「一个形态在 Y1 上连挂 N 次都只读、且盘上一直不变 ⇒ 记「一次故障之后永远只读」量实」 | 条款没给改法。甲-读错 在这 5 格上走得出（量过）；甲-读错-每盘 见第 Y2 节第 3 条 |
| 甲-读错：单槽持久设备读错之后永远只读 | 分：今天、(a)、(b) 全可写 | 看得到：挂载那一刻盘 1 两槽都读得出、自证过，被确认过的发布的见证在盘 1 上也有一份（推的：每次轮换每块盘各写一槽、轮换之后屏障才确认，D16 已定项 7；没在副本上造「盘 0 读错那一槽里有比盘 1 更新的见证」的状态去反证）。判的是「这一槽读错」这件事，而判别它要的东西在另一块盘上——判据写宽了，不是判别子观测不到 | 同一条「永远只读」；另碰正文第五节甲-读错那一条「与甲并列进交用户的表，各报丢写、多拒、永远只读与否」里的「永远只读与否」这一栏：是 | 条款没给改法；甲-读错-每盘 在这一格上的数见第 Y2 节第 3 条 |
| (a)：账持续读不出之后永远只读（40 格 × 6 种动作，40 格已含两种坏扇区模型） | 分辨 (a) 与 (b)：分配记录树那几格 (b) 可写；树表那几格不分辨，(b) 同样永远只读 | 看得到：重读一次仍读不出。树表那一格的判别子（树表内容）盘上没有第二个来源；但**落点**有：根记录直接指着树表、实例表与中央映射树，映射树的条目点名其余每个单元（量过：账读得出时这样现算的落点集与账里的逐个相同，见第 Y2 节第 2 条） | 同一条「永远只读」 | (b) 修分配记录树那几格（量过），不修树表那几格（量过）；(b)-从映射现算 见第 Y2 节第 2 条 |
| (b)：树表持续读不出之后永远只读 | 不分辨 (a)、(b) | 同上一行 | 同一条 | 同上一行 |

## Y2　零轮变体在第一轮网格上的数

### 1. 甲-读错（`c331-summary.txt` 原样行；今天、甲两份的输出与第一轮逐行相同，见「复跑」）

```
today=base scenarios=1959 verdicts={'refused': 814, 'writable_no_loss': 905, 'writable_LOST_both': 18, 'writable_LOST_older_acknowledged': 222}
== candidate=jia scenarios=1959 key_mismatch=0 verdicts={'refused': 1725, 'writable_no_loss': 234}
   lost_cells=0 over_refused_cells=719 today_lost_now_not_lost=240
== candidate=jiaeio scenarios=1959 key_mismatch=0 verdicts={'refused': 1190, 'writable_no_loss': 649, 'writable_LOST_both': 9, 'writable_LOST_older_acknowledged': 111}
   lost_cells=120 over_refused_cells=284 today_lost_now_not_lost=120
   lost by prefix: {'long': 48, 'torn_both': 14, 'standard': 48, 'v4': 10}
   over_refused by prefix: {'long': 120, 'standard': 120, 'torn_both': 12, 'torn_d0': 32}
```

按读回拆（对 `runs/jiaeio-c331_candidates_grid.log` 与 `runs/base-c331_candidates_grid.log` 按键对齐数的，脚本就是第一轮 `summarize_c331.py` 的 `load` 加一个按 `read_back` 的计数，结果照抄）：

- 丢写 120 格：`Zeros` 120、`DeviceError` 0；按系统配置那一组：`NewestOnBoth` × `Zeros` 64、`AllFour` × `Zeros` 56。今天丢的 240 格里 `DeviceError` 120、`Zeros` 120，甲-读错 把 `DeviceError` 那 120 格全挡住（拒 92、可写不丢 28），`Zeros` 那 120 格照丢；甲-读错 丢而今天不丢的格：0。
- 多拒 284 格：全是 `DeviceError`（甲的 719 格里 `DeviceError` 361、`Zeros` 358）。按系统配置那一组：`NewestOnDevice0` 82、`NewestOnBoth` 80、`OlderOnDevice0` 70、`BothOnDevice1` 34、`AllFour` 18。最短的 1 段读故障：盘 0 一槽（较旧或最新）`DeviceError`，持续、只在第 5 次读起（取号那一读）或只在两遍见证读上坏，别的都读得出（`c331-summary.txt` 里甲-读错 `over_refused: fewest_fault_ranges=1` 下列的格）。
- 撕裂三格（`slots=None hidden=None`）：今天 `writable_no_loss`，甲 `refused`，甲-读错 `writable_no_loss`（三格原样同上一行的对齐结果）。
- 最短丢写反例 4 段读故障：两块盘最新那一槽读回全 0（`NewestOnBoth`、`Zeros`）加 C 的根与它盘 0 那一份数据单元读不出（V3 的 `Zeros` 形），`standard` 与 `long` 两个起点、各种时机都丢（`c331-summary.txt` 里 `lost: fewest_fault_ranges=4` 下列的格）。

照跑前判据第二条：丢写只在 `Zeros` 形、撕裂三格不拒 ⇒ 与甲并列进交用户的表。「永远只读与否」那一栏按 Y1：撕裂轮换走得出、单槽持久设备读错永远只读。

**V4 那一支**：甲-读错 的取号扩展（取号那一读有一槽设备报读错就拒）在 V4 144 段里丢写 10，全是 `Zeros` 形（今天 20：`DeviceError` 10、`Zeros` 10）。罩住 `DeviceError` 那一半，`Zeros` 那一半罩不住：取号那一读把全 0 的槽当自证不过、照今天取另一槽的 tail 6 写进去。

### 2. (b)-从树表现算（`c393-summary.txt` 原样行）

```
TOTAL base: cells=98 {'ok_no_reuse': 28, 'REUSE': 70}
TOTAL a: cells=98 {'ok_no_reuse': 70, 'refused': 28}
TOTAL b: cells=98 {'ok_no_reuse': 84, 'refused': 14}
```

- 复用 0。(a) 拒的 28 格里，(b) 拒 14 格（全是树表读不出：`TreeTable` 12 格、`TreeTableAndAllocationRecordTreeRoot` 2 格），另 14 格（分配记录树根 12 格、分配记录树根之下节点 2 格）可写不复用，隔离数与读得出的对照一样（`single` 14/14、`double` 30/30）。今天可写不复用的 28 格 (b) 一格没拒。
- 「记录也读不出」那一形（第一轮 (d) 在其上复用 16 段）：(b) 不读 journal 记录，分配记录树那几格照样可写不复用；树表那几格照 (a) 拒（`c393-summary.txt` 里 `rec=TheAbandonedPublish`、`rec=EveryRecordOfItsInstance` 的行）。
- 现算的落点集核过：副本 b 在账读得出时把现算的与账里的比一次（`SINGLEFS_R2_B_COMPARE`），网格一遍 62 次，62 次逐个相同（`runs/b-c393_candidates_grid.log`，`grep -c` 数的）：

```
     40 name=r2-b-compare root=(2,8) account=26 computed=26 only_in_account=Some([]) only_in_computed=Some([])
     22 name=r2-b-compare root=(4,14) account=30 computed=30 only_in_account=Some([]) only_in_computed=Some([])
```

- 我的实现是正文定义的一种读法（`candidate-b-over-a.diff`）：账读不出时，树表读得出、不是 0 条，就取根记录直接指着的实例表链（认得出的每一片）、树表、中央映射树的每个节点，再加映射树里每条映射条目的落点（数据单元、打包单元 2 槽，索引节点 1 槽）；只进内存隔离表与根环表，不写盘。现算不成才照 (a) 重读一次、仍不成就拒。我没照字面「从树表走读」去走四棵树：分配记录树的根读不出时，它下面的节点从树上走不到，只有映射树里有（推的：没实现那种走法，按树的结构推）。

### 3. 我提的收窄：甲-读错-每盘（只在我的模型上量过、被攻过零轮）

定义（`candidate-jiaeiodev-over-jiaeio.diff`，打在甲-读错之上）：见证读里**每一块盘**都有一槽设备报读错才判不出（按真）；有一块盘两槽都读得出，就照今天取全部自证过的槽的 max。取号那一读同一条：每一块盘都有一槽读错才拒，一份自证过的都没有照今天拒。

依据（推的）：每次发布的轮换在每块盘各写一槽、轮换之后屏障才向调用方确认（D16 已定项 7，第一轮判决 K1 引的同一条），所以任何被确认过的发布，它的见证在每块盘上都有一份；只要有一块盘两槽都读得出，那块盘上就有最新被确认的见证。只有每块盘都缺一槽时才可能把它漏掉——那正是 C583 那一格的双故障形，用户定过不在容错射程（「用户 2026-09-27 定不在容错射程（D22（单元原子性怎么合成） 已定项 8 容每盘一槽坏是单故障承诺）」（`.claude/kb/checks-owed.md:462`））。

量出来的（副本 `jiaeiodev`，`runs/jiaeiodev-c331_candidates_grid.log`、`runs/jiaeiodev-traj-torn.log`、`runs/jiaeiodev-traj-single_slot.log`；汇总行出自第一轮 `summarize_c331.py` 加这一份副本的输出 `c331-summary-with-dev.txt`）：

```
== candidate=jiaeiodev scenarios=1959 key_mismatch=0 verdicts={'refused': 1004, 'writable_no_loss': 835, 'writable_LOST_both': 9, 'writable_LOST_older_acknowledged': 111}
   lost_cells=120 over_refused_cells=98 today_lost_now_not_lost=120
```

- 丢写 120 格，与甲-读错 的丢写格是同一个集合（全是 `Zeros`，V4 里 10 格）：收窄没多丢一格。
- 多拒 98 格，是甲-读错 那 284 格的子集，全是两块盘都有一槽设备读错的形（`NewestOnBoth` 80、`AllFour` 18）；只有一块盘有槽读错的 186 格（盘 0 一槽 `NewestOnDevice0` 82、`OlderOnDevice0` 70，盘 1 两槽 `BothOnDevice1` 34）不再拒，这 186 格在今天可写且不丢。
- Y1：撕裂轮换 13 个状态、单槽持久读坏 8 格（两种读回 × 两槽 × 两种坏扇区模型），6 种用户动作下全是 WWWWWW（`trajectory-summary` 里 `jiaeiodev` 一列，汇总见 `traj-dev.txt`）。
- 没量的：两块盘各坏一槽而且是持久读错的那一格上的轨迹（它会永远只读：拒在取号之前、一次写都没有，推的）；读回全 0 那一侧照旧放过，收窄碰不到。

### 4. 我提的放宽：(b)-从映射现算（只在我的模型上量过、被攻过零轮）

定义（`candidate-bmap-over-b.diff`，打在 (b) 之上）：去掉「树表读得出、不是 0 条」那一道前提，树表自己的落点直接取根记录里那条指针；映射树读不出、解不开（或映射根是空指针）才照 (a)。理由：现算要的是这条根引用了**哪些槽**，不是树表的**内容**；树表是权威态、内容没有第二个来源（第一轮 K3 引的「| **权威态** | **单元 + 记账 + 根** |」（`.claude/kb/decisions/21-权威态与派生态的分界.md:188`）那一行），但被抛弃根的树表内容用不着，隔离只要落点。

量出来的（副本 `bmap`，`c393-summary-with-bmap.txt`、`traj-bmap.txt`）：

```
TOTAL bmap: cells=98 {'ok_no_reuse': 98}
```

- 98 格全可写、复用 0；现算与账逐个相同的核对 62 次（与副本 b 同样两行，`runs/bmap-c393_candidates_grid.log`）。
- Y1：`single` 16 格 + `double` 24 格 + 两条对照，各 6 种动作，全 WWWWWW，只有 (b) 那一格 `Sticky` 坏扇区的旁及形照样 WWWXXX（`DeviceError`、`Zeros` 各一条）；根环里被盖 0 行、I-7.4 红 0 行。树表那几格 (a)、(b) 永远只读，这一版第 1 次就可写。
- 没量的：映射树也读不出（再加一段读故障）时它落回 (a)、永远只读（推的）；与已定分项相容不相容不归我判（Y3）。

## 几个改法各修哪一格

「量过」指 `runs/` 里这一份副本的原样结局；「推的」按代码推、没跑。

| 改法 | 撕裂轮换（5 状态） | 单槽持久设备读错 | 单槽持久读回全 0 | V1/V3 `DeviceError` 丢写 | V1/V3 `Zeros` 丢写 | V4 `DeviceError` | V4 `Zeros` | 两盘各一槽读错 |
|---|---|---|---|---|---|---|---|---|
| 今天 | 可写（量过） | 可写（量过） | 可写（量过） | 丢（量过） | 丢（量过） | 丢（量过） | 丢（量过） | 丢或可写（量过：网格里的格） |
| 甲 | 永远只读（量过） | 永远只读（量过） | 永远只读（量过） | 拒（量过） | 拒（量过） | 拒（量过） | 拒（量过） | 拒（量过） |
| 甲-读错 | 可写（量过） | **永远只读（量过）** | 可写（量过） | 拒（量过） | **丢（量过）** | 拒（量过） | **丢（量过）** | 拒（量过） |
| 甲-读错-每盘（我的，零轮） | 可写（量过） | 可写（量过） | 可写（量过） | 拒（量过） | **丢（量过）** | 拒（量过） | **丢（量过）** | 拒；持久的话永远只读（推的） |

| 改法 | 树表持续读不出 | 分配记录树持续读不出 | 被抛弃那次发布的记录也读不出 | 复用（98 段） |
|---|---|---|---|---|
| 今天 | 可写、复用（量过） | 可写、复用（量过） | 可写、复用（量过） | 70（量过） |
| (a) | 永远只读（量过） | 永远只读（量过） | 永远只读（量过） | 0（量过） |
| (b)-从树表现算 | 永远只读（量过） | 可写、不复用，被抛弃根第 3–4 次挂载离开根环（量过） | 按账那一格同上两列（量过） | 0（量过） |
| (b)-从映射现算（我的，零轮） | 可写、不复用（量过） | 可写、不复用（量过） | 可写、不复用（量过） | 0（量过） |

## 没打中的形状

- 「只读挂载写盘」：五份副本加我的两份，所有拒可写之后的只读挂载，录制流 0 步、两块盘逐字节不变（`refusals_left_the_disk_unchanged=true`，全部 `r2traj-summary` 行里 `=false` 的 0 行，`grep -c` 数的）。拒可写那一试本身同样 0 步。取样：5 个起点族、6 种用户动作、每条 6 次挂载。
- 甲、(a) 的「永远只读」里找出口：试过每次先试可写再只读、k = 0/1/3 次写、崩溃或正常卸载收尾、坏扇区写时重映射或不重映射，6 次挂载里一次都没走出，盘上一个字节没变。没试的出口：管理员回退（入口 `roll_back_by_a_forward_publish` 要调用方交进分配器与现行那一版的 `TransactionOutput`，这两样只有可写挂载交得出，推的：只读下调不到，没在副本上试）、从只读 remount 成可写（C286 那一格「只读之后 remount 成可写算不算一次可写挂载、许不许，全仓没有一句」，代码里没有这个入口）、mkfs 重建（丢数据，不算出口）。
- 甲-读错 在 `DeviceError` 形上的丢写：1959 段里 0 格；V4 144 段里 0 格。
- (b) 的复用：98 段、Y1 252 条 c393 轨迹（42 个起点 × 6 种动作，`single`、`double` 两个起点族）里 0 格；它把被抛弃根的落点算全了（62 次逐个相同）。
- 撕裂轮换的全部层 0 状态（D 流第 0 段之后）：五份副本 + 甲-读错-每盘，只有甲在撕裂那 5 个状态上只读。

## 这条腿自己的限度

- **层 0 枚举只跑了 D 流第 0 段之后那 3 段**：整条 D 流 268435468 个状态，超过约 10⁶ 不跑；缩历史长度之后 13 个状态、全展开。这一腿原型全量合计 13 × 7 份副本 = 91 个状态（每份 `layer0_states_enumerated_by_this_run=13`），远在约 10⁷ 之内。第 0 段里的崩溃点（D 的单元写落了一部分）没有逐状态走，那些状态上系统配置没被动过是按段序推的。
- **单槽持久读坏、被抛弃根的账持续读不出都是设备故障，不是崩溃状态**：只在起点上加，轨迹里的挂载与发布都不在中途崩溃（「崩溃收尾」是在一次挂载做完、k 次写都确认之后才崩）。一次挂载中途崩溃之后的轨迹没跑。
- N = 6；「永远只读」量到的是 6 次以内。只读的那些轨迹每一次都在取号之前拒、盘上 0 步，第 7 次起的盘面与第 1 次逐字节相同，结论外推到任意次是按「盘面没变、判定是盘面的函数」推的。
- 只在一组几何上量：两块盘、区域归属 [0, 1, 0]、4 GiB 盘、默认环长、每区 8 个根槽、单文件。
- 读故障只有「块设备错」「读回全 0」两种；坏扇区只有「写了照样坏」「被写罩到就好」两种模型。
- 甲-读错、(b)-从树表现算 是我对正文定义的一种实现：甲-读错 的「设备报读错」取 `PoolReader::read` 交 `None`；(b) 用中央映射树现算，不是字面的逐棵树走读（第 Y2 节第 2 条末尾）。别的读法没量。
- `jia` 副本上 C393 起点造不出（第 Y1 节第 5 条），甲在 (a)、(b) 那几格上没有一列。
- `common-mod-memory.rs.diff` 改了共用的搭建（镜像放内存）；三份第一轮副本上两张网格逐行对得上第一轮，但轨迹原型没有文件镜像的对照版可比。
- 副本上的数不是入库装置上的数；要引，得在入库装置上重做。

## 没做什么

- 没判 Y3、Y4；没替主 agent 采纳。甲-读错-每盘、(b)-从映射现算 两份是我提的，只在我的模型上量过、被攻过零轮。
- 没跑重型测试：没跑 checker 档包的测试、层 0 用例与门禁；原型里调的是 checker 档库的 `enumerate_layer0_selecting_versions_observing_each_state` 与计数函数，流是原型自己录的。
- 管理员回退、只读 remount 成可写这两条出口没试（入口不存在或调不起，见「没打中的形状」）。
- 「每块盘各坏一槽、持久读错」那一格的轨迹没跑（甲-读错-每盘 会在其上永远只读，推的）。
- 交回前删了：仓副本 `/tmp/claude-1000/unreadable-at-mount-r2/opus/pristine`（9.6M）、`base`、`jia`、`jiaeio`、`jiaeiodev`、`a`、`b`、`bmap`（各 9.7M）；编译目录 `target-a`（556M）、`target-b`（560M）、`target-base`（556M）、`target-jia`（555M）、`target-jiaeio`（557M）、`target-bmap`（466M）、`target-jiaeiodev`（467M）；临时目录 `tmp`（4.0K）。草稿目录里留着 `runs/`（与模型目录 `runs/` 同一批日志）与 `scratch/`（脚本、汇总、构建日志），复跑要的全部文件在模型目录里。

## 模型目录 sha256

`cd research/prompts/unreadable-at-mount-r2-opus-model && find . -type f | sort | xargs sha256sum`：

```
0f94f4eb80ae0c987c8cf681b0746f9d7a86cd870c3ba55c4ff319ff9748bade  ./c331-summary.txt
e880d83e7e995cb35bd4af9ca40951cf9910e144b2bd9f764e107ff2b4573887  ./c331-summary-with-dev.txt
461160a7b919fc2c9b4230bd7c04ef3134cd8006b7dcfd4d310d7a9aaaf9e3ea  ./c393-summary.txt
8bd4f8873f6791961341bfe3ca6e32a2ecdc76e7859b10fe26b6a8c38984851a  ./c393-summary-with-bmap.txt
81eda2ca26aa9794b5197a441a07bc9b2667f7f50b55d04353449beeca1c4c57  ./candidate-bmap-over-b.diff
bc2b3c486ecc11399372ff0ba1500b4601eb175421b3e34630c5d8eb25c5b15c  ./candidate-b-over-a.diff
3d64b9c490583dcd06ff873739fe51af90dec27fe3c2c02923dfe3b15f55a40b  ./candidate-jiaeiodev-over-jiaeio.diff
ebda52c049dcfb8cf696749541cac7fc3af1081d4c9c74503c19ac116b372d6c  ./candidate-jiaeio-over-jia.diff
9c5b02846b0b30233b3f96f72d9c3b95b5db6f4ae44f52c3321e5013fc1b4561  ./common-mod-memory.rs.diff
083d33e24f81d95b2bdf69f93ad617485f182a194fb64092db97a16fe51a4ba9  ./r2-opus-trajectory-proto/Cargo.toml
c88ebebfc5ef8caf154b9eef87a67d676105142466c8db1dd360b2083854d37c  ./r2-opus-trajectory-proto/src/main.rs
1a8a3e7ca03d9f9e13e593b646228650b4c78c69f5d0a8496df85a453f529eb7  ./rerun.sh
fa7f01b00e09a4446f9cc2333a409ddc91f828cd15306d37842faf6bc2e69929  ./run-batch3.sh
f266ecac47eb45a645095432d020b32d51ebe89cf5b4ce683ebe13427fc5cf2f  ./run-batch4.sh
c1ab3a9f8ceb73d4c498b276439018799f835cf805a394bc0024120c1db6e6a5  ./runs/a-c393_candidates_grid.log
65b91a4487ec6fc7ebdecc00e7c1f3f749a9e91aee682231db7422322c913516  ./runs/a-traj-c393_double.log
d3ce217dbea774d5a81269eb78f223edc7a1673ca2cf7829deecc105ffa833ae  ./runs/a-traj-c393_single.log
4a7e5958d7031e09dd0e7d0ec46e36208b96d50e2c08b5f7651c8bc7579e2561  ./runs/a-traj-single_slot.log
6d29e2776093a086287e669da2151138b1c9abce0f13ebcb4c10674ed3339a59  ./runs/a-traj-torn.log
a185171dcc3af861cb4592107aac52fc486df84a7b5fa2877757c3dcb627ef9a  ./runs/base-c331_candidates_grid.log
eeac9bec5b43fc783d2170366985942172c4fc58f479551584bff6b47ffdbfa5  ./runs/base-c393_candidates_grid.log
8cd437e4486b8c7d4a21f0ada92b73b8a6463d2343210719afe8e50416c1f0a9  ./runs/base-traj-c393_double.log
0f2c50d71c937bf14d8ce4d268c0782f12d501866d716e77f927354976c6af65  ./runs/base-traj-c393_single.log
76bdac53e51161cf5f1c62af69b847f506b9d95fc1c929acda84ca274da75033  ./runs/base-traj-single_slot.log
a2eaf8cf10db38d37aebaa25aa0c843802d40a84a812d41530ab7db836ef45a7  ./runs/base-traj-torn.log
c4d94c0ef082fcdd7f12c7df6249b7a8816fea75264293ae1e44d584d1dc6cbf  ./runs/batch1.out
d84cb912f532a75fbcbe312d24cc88927d7d925dae502a20c73e74400fb77ec2  ./runs/batch2.out
e8e37dd6558e7feff41640e37b20417f68db10eea76349be0f6c296c43a4b580  ./runs/batch3.out
60f92a4e31b5e68a3a1e44f66fe251b11c1cc43b2ad9434c85284bfe68fae851  ./runs/batch4.out
37a21466fd14e5e7d132f3923353c56b726e8ef415f65f43f6460509fa865707  ./runs/b-c393_candidates_grid.log
c553f6c8ba0079ef940303797637cf53856fa510f376f4c75c29881c970a66ba  ./runs/bmap-c393_candidates_grid.log
71e8ff45992c6e8c5e62f747aeae5ef6116988bf435edb066076cb69599665c0  ./runs/bmap-traj-c393_double.log
e890eb047645c8f43cda0abba043c354bc8d9843b981881f12a36f08e3e854f9  ./runs/bmap-traj-c393_single.log
4ef2a6b5905592accbe94e528fd4828551228421e7997dcdbe8fd0be39701f84  ./runs/b-traj-c393_double.log
58d6c1121e8a497dc8cdd76631148efd3af74fae37b99dfd7f199dd8e9083537  ./runs/b-traj-c393_single.log
5072f147ac87792a09d43d71105ee42c15003bdaa7a2ec294298858c4acc6a03  ./runs/b-traj-single_slot.log
ec0097f23a9d3022bfefc3375d9b623c05b13b187f35fd1d5faa9a86c10d8f4c  ./runs/b-traj-torn.log
16b3d221faadc0e9cce304947effd345d789f2685b19036ff5b7d488cc3a0a22  ./runs/jia-c331_candidates_grid.log
7bdb8d9ce87a9460e1b33cf0baf40f48a0e56c18ec3c346379f728c65c096768  ./runs/jiaeio-c331_candidates_grid.log
ced2cd5148f34c29f0fc2b2a6075a8f3f077c795f199354a97bce95e3e779fc2  ./runs/jiaeiodev-c331_candidates_grid.log
37e1266fb0dbf2feea876c1db14a4a0c731f55ffc35eba6574df7eaf9c716abe  ./runs/jiaeiodev-traj-single_slot.log
921448fa20df92d3a2e9936de099f95edefb3e54dfae7dd94c38e7289537fef4  ./runs/jiaeiodev-traj-torn.log
89c0c483e464114b0ef6ef5051b1df774a0fffa089634209749583d61aa38567  ./runs/jiaeio-traj-c393_double.log
ae5691739189ce10b7462972d44f6779fa00d93a6e95ebee27429e65904e52d3  ./runs/jiaeio-traj-c393_single.log
f3bcef48759f3d062d52632b409e08e477b83c540d662e3f1f2a432c94276733  ./runs/jiaeio-traj-single_slot.log
488ab3c19829dcadfbd8a8aab25a13dc6346348b1fb9de3be47251f6108b513c  ./runs/jiaeio-traj-torn.log
362d1bdffcfb9c1bf63d32d0b674a2efb8bac048c4d6ab59037741b397612b7b  ./runs/jia-traj-c393_double.log
a5992740719e48afa4f935cdf726da83faa84aafd8ce68850b52e5214337a30c  ./runs/jia-traj-c393_single.log
1ae1001be100b02e18080dd616afc76d4ccfd8a0546c3f0fa6f6e2e4957ccc72  ./runs/jia-traj-single_slot.log
ce0f0c22dc0db045139529d3b7ffcda2cbeda93469884cb1dde549bc09405e3f  ./runs/jia-traj-torn.log
e214416e98f6e7723aa9cb1514f2d9c0fb73c495b6a0d8291e76adac3561fa3f  ./runs/run-y2.out
262ca8998e70a0076b680455c0cbaeb81bc12702a7379e3333a78ea55f0b77af  ./run-y1-batch.sh
024cc21716241b8819f0cf2b30d7b039f9b9e0683aa786e2cc1a5ec4b5dd1f7f  ./run-y2.sh
9712508262fbb7f4a981f321da6ea5f72d48ff7d0a392016c9084fe48799dbad  ./summarize_trajectories.py
ff2bb51a59ae0e8109aa836cfedffa3bc44058ad3ea003c66b17c0f25f5f8b39  ./traj-bmap.txt
8c2246de25707f0a777124a0a62ef059cd8ce9c688f77deddc2ad5c5845b500d  ./traj-dev.txt
eb25ac95d6477cef8408d4a71f4d0c70f8bda58ae3d30c43a261091f7fd6716d  ./trajectory-summary.txt
```
