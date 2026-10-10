# 第 10z 批第四块代码轮第一轮：正推腿（Sonnet）报告（2026-10-08）

轮名 m3-batch10z-d-code-r1，立场正推。分到的格：K1（1-3）、K2（1-3）、K3 全部。
冻结副本 `~/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/`（下称「冻结副本」；引代码的路径相对它，`cite-check.py --root` 指它）。
我的草稿目录 `~/.local/share/singlefs/work/m3-batch10z-d-code-r1-sonnet/`，模型目录 `research/prompts/m3-batch10z-d-code-r1-sonnet-model/`（探针源码、变异表、原始输出，README.txt 写了复跑命令）。
只跑了 harness 档（`cargo test -p singlefs-harness --test …`、`-p singlefs-core --lib`），经 `run-with-memory-cap.sh 12G` 与 `capped.sh 6`；checker 档一条没跑。开工时 `ps` 看到别的会话的 cargo（opus-probe 等），没有 `qemu-system`、`vm-bench.sh`、`fio`、`e152-file-system-benchmark`，照常加 `nice -n 19` 跑。

## 0. 基线与方法

- 冻结副本被判文件的 sha256 与 `research/prompts/m3-batch10z-d-code-r1-snapshot/crates-sha256.txt` 逐行相同（`sha256sum -c … | grep -vc ': OK'` 输出 0）。
- 基线（副本原样）：`singlefs-core --lib` 退 0，`test result: ok. 202 passed; 0 failed`；新用例文件 `ok. 8 passed`；LR2 文件 `ok. 14 passed`（14 线程 16 秒）；另五个相关目标（admission_raises_the_floor_before_refusing 等）全 ok；慢档 `raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable` `ok. 1 passed`（`logs/base-*.log`、`logs/k1-slow.log`）。
- 量法：入库装置 `crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs`（R 那块盘）与新用例文件的 `pool_whose_newest_root_was_abandoned_by_a_crash_recovery`（C 那块盘）造盘，各加一层「读报错 / 读回全零」可开关的盘包装；会话里覆盖写，隔几步跑一次池级 checker（只看 I-7.4 与 I-4.8），再卸载、再跑一次。另在分配器上数「被引用单元里 `is_free` 的个数」（每个跨度槽都查，不只第一槽）。
- 探针能分辨改前改后：撤掉 10z-d（对冻结副本 mount.rs、allocator.rs 反向打实现员的两份 diff，15 个 hunk 全成，副本 `repo-pre`）后同一探针 H1 根槽读错一形在第 13 次覆盖写被拒，现行副本跑完 60 次不拒（见 K2-1）。

## 1. K1　定案做到没有

### K1-1　H1：抬 F 那一串每一处读根环都换了吗

条款（kb 现查）：D16 第 37 行末段。

- 「同一串读根环时被抛弃根的根槽读坏、重读仍坏不拒，照根环表里记着的落点进影子账、不进上限」（`.claude/kb/decisions/16-发布语义.md:37`）。
- 抬 F 那一串在 core 里读根环的地方，按 `grep -n 'ring_slots_known_to_hold_a_root_by' crates/singlefs-core/src/mount.rs` 数是三处用「知道住着根的槽」：算上限、抬 F 重算影子账与回收门槛、管理员回退判候选。前两处换了，第三处没动（H1 要的）。

| 处 | 位置 | 做了什么 |
|---|---|---|
| 准入算上限 | 「!ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)」（`crates/singlefs-core/src/mount.rs:3596`） | 传进上限的「知道住着根的槽」滤掉根环表记成被抛弃根的槽 |
| 抬 F 重算影子账与回收门槛读根环 | 「!ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)」（`crates/singlefs-core/src/mount.rs:3870`） | 重读仍坏就拒的槽集合同样滤掉；读坏的这几槽记着的落点交回 |
| 影子账补隔离 | 「for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad {」（`crates/singlefs-core/src/mount.rs:1630`） | 照记着的落点补隔离 |
| 卸载 | 与抬 F 同一个 `raise_the_floor_through`（`RaiseFloorEntry::Unmount`），不另读 | 随上两处 |
| 管理员回退 | 「&ring_slots_known_to_hold_a_root_by(allocator),」（`crates/singlefs-core/src/mount.rs:7588`） | 没动 |

判定：**做到**。实测三条：

1. 被抛弃根的根槽读错：现行副本不拒，撤掉 10z-d 的副本拒。同一探针 `h1_ring_error_after_mount`：
   - 撤掉 10z-d：`被拒=Some((13, "FloorRaiseFailedWhilePushingForSpace(… RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { ring_slot: RootRingSlot { region: 2, slot: 2 }, …`（`logs/probe-pre.log`）。
   - 现行：`会话写至多60:完成60次 被拒=None`，卸载 `做成(计数0)`（`logs/probe-h1-b.log` 与 `logs/probe-k23-a.log`）。
2. 有效根槽照拒（两处各验）：
   - 卸载：新用例 `unmount_after_the_root_ring_slot_of_a_valid_root_turns_unreadable_is_still_refused_before_any_write` 绿；我的变异 `rr-all-exempt`（拒集换成空）把它打红。
   - 准入：新文件两条都绿而旧目标红——变异 `ceil-all-exempt` 在 `admission_raises_the_floor_before_refusing`（`known_root_slot_still_bad_after_one_reread_refuses_the_floor_raise_before_any_write`）、`root_ring_slot_reread_outcomes_…`（`a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write`）红。
   - H2 那块盘（R 记有效根）根槽读错：第 12 次覆盖写准入拒、卸载拒 `RootRingSlotKnownToHoldARoot`（`logs/probe-h2-ow_root.log`，10 组全这样）——条款字面（有效根的槽照拒）。
3. 管理员回退没变：`k11_admin_rollback_with_abandoned_slot_bad`，被抛弃根的根槽读错，`roll_back_by_a_forward_publish` 返 `Err(CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot { ring_slot: RootRingSlot { region: 2, slot: 2 }, first_reading: Unreadable, reread: Unreadable }))`；故障关着时 `Ok(())`（`logs/probe-k11.log`）。

**有一形漏（做到但漏一处调用方）**：公开函数 `rollback_floor_ceiling`（「pub fn rollback_floor_ceiling<Device: BlockDevice>(」，`crates/singlefs-core/src/mount.rs:3092`）本身照旧口径，它的调用方照旧传「知道住着根的槽」整集：
- checker 档 `on_device_modes.rs` 的预算上限：「&ring_slots_known_to_hold_a_root_by(allocator),」（`crates/singlefs-checker-tier/src/on_device_modes.rs:336`）。
- 实测：被抛弃根的根槽读错，用这一写法调 `rollback_floor_ceiling` 返 `Err(RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { ring_slot: RootRingSlot { region: 2, slot: 2 }, … })`，同一时刻 `raise_rollback_floor` 本身（目标取当前 F，因为上限那一步 Err 没给出数）返 `Ok(0)`（`logs/probe-k11.log`，`k11_public_ceiling_function_with_abandoned_slot_bad`）。
- 实现员报告说公开 API 一个字没改，是有意；但 `on_device_modes.rs` 这个调用方在这一形上仍拒。影响面是 checker 档的装置模式（上限先算一遍再交 `raise_rollback_floor`），不是产品路径。
- 推翻它：checker 档 `raise-rollback-floor` 模式没有任何一条在注入被抛弃根的根槽读错的状态下跑；我没跑 checker 档，这一形在 checker 档上的结局是推的（用同一函数、同一参数在 harness 档复现了返回值）。

实现员自定「H1 读坏的被抛弃根槽不重读」（背景 §二）。读 `readable_roots_for_raising_the_floor` 实现：这几槽不进「重读仍坏就拒」的集合，读一次，读坏就交记着的落点；读得出就走读出来的那条根（取表里记着的落点）。结局相同，与实现员报告说的一致。推翻条件：构造一形「第一次读坏、重读读得出」的被抛弃根槽，两种结局的盘不同（我没造；理由：被抛弃根的落点两条路都取表里同一份，造出来也不会不同）。

### K1-2　H2：认同条件有没有一形认错

条款（D16 第 37 行）：「在那一槽记着同一条根（那一槽记的是有效根且 txg 相同）时」（`.claude/kb/decisions/16-发布语义.md:37`）；「C 那一圈记着的落点算、不读账、读不出不拒，C 也没记着的照今天读盘」（`.claude/kb/decisions/16-发布语义.md:37`）。

代码：
- `ValidRoot` 与进程内重读读出有效根那一类只比 txg：「} => *recorded_checkpoint_txg == checkpoint_txg,」（`crates/singlefs-core/src/allocator.rs:1167`）。
- 两遍里只有一遍自证过的有效那一半比 (实例代号, txg)：「) == (instance, checkpoint_txg)」（`crates/singlefs-core/src/allocator.rs:1175`）。
- 认同之后交那一槽 C 的整份：「.cloned()」（`crates/singlefs-core/src/allocator.rs:1190`）；`None` 的出口在 1185-1187 行（认不同）与 `.get(&ring_slot)` 没有这一槽（1189）。
- 调用处只对按实例表被抛弃、且根环表没记着落点的根借：「.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(」（`crates/singlefs-core/src/mount.rs:4131`）。

逐形：

| 形 | 判定 | 依据 |
|---|---|---|
| 同一槽差一整圈（txg 差 3S）的另一次发布 | **做到**（认同挡住）；借来的 C 那一份是超集 | `target_for_publish` 里 region = txg % 3、slot = (txg / 3) % S，同槽的 txg 差 3S。`ValidRoot` 比 txg、一遍自证类比 (实例, txg)，差一圈的 R 不借。库单测 `placements_named_by_unapplied_journal_records_for_a_root_recorded_valid_answer_only_for_the_same_root_on_its_ring_slot` 钉；我的变异 `h2-valid-compare-ignore-txg`（有效根类不比 txg）、`h2-onepass-compare-txg-only`（一遍自证类只比 txg）、`h2-slot-off-by-one`、`h2-allow-badatmount`、`h2-allow-abandoned-occupant`、`h2-drop-valid-reread-class` 各把这一条库单测打红（`k3/out/*.txt`，`core rc=101`）。那一槽 C 里带着两次发布的落点时借整份：多隔离几个本已被 C 隔离的槽，只会多护。 |
| 只比 txg、不比实例（有效根类） | **做到，认错走不到（推的）** | 槽上记的有效根与盘上那一条是同一条（这个进程没写过那一槽；写了就换成这个进程写的有效根，C 在那一槽的落点同时解开，见「C 的全部落点只在这个进程写掉那条记录所属 txg 对应的根环槽、那次发布取完落点之后解开」（`.claude/kb/decisions/23-journal的角色与格式.md:392`），借不到）。同一槽同 txg 不同实例的两条根要在同一时刻都在这一槽上，做不到。推翻：造出「槽上记的有效根与抬 F 读出的被抛弃根同 txg 不同实例、C 还没解开」。没造出。 |
| 那一槽 C 只记着别的发布、没记着 R（实现员推的那一形） | **条件造不出（推的）**，没造 | 要 R 与 R'（txg 差 3S）的记录都在水位之上、R 的记录与账都不在 C 里、R' 的在：水位之上要留 ≥ 3S 次发布。384 槽小盘 3S = 24（探针 `diag_bz_refusal` 列根环里的根，一圈 24 条：`roots_in_ring=[(2, 24), (1, 3), (1, 6), …]` 共 24 项，`logs/probe-diag.log`）。R' 的根没落盘而 R 与 R'−1…R+1 的根都落了，则 R+1…R'−1 共 3S−1 条根已把所选根 P 那一槽盖掉，P 不在环里、不能被择。造不出的前提是「根环只留最近 3S 条根、所选根必须在环里」。推翻：造出水位之上相隔 3S 个 txg 的两次发布的记录同时可读，且择根仍落在更旧的 P。 |
| 那一槽 C 里什么都没有（`None`）→ 照今天读盘 | **做到** | 库单测同上；变异 `h2-unrecorded-C-gives-empty`（C 在那一槽没条目时交空串）把它打红；端到端：LR2 文件 `an_unmount_meeting_an_abandoned_root_not_in_the_root_ring_table_whose_account_is_unreadable_recomputes_it_from_the_mapping`（读盘、计数 1）与慢档 `raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable` 都绿（慢档我跑了：`ok. 1 passed; … finished in 6.36s`）。 |
| C 在那一槽有条目但是空 `Vec` 时 | **一形没人钉（推的）** | `named_by_ring_slot.entry(…).or_default().extend(placements)` 在 `placements` 过滤后为空时留下一个空条目（`crates/singlefs-core/src/mount.rs:1821`），借来的就是 `Some(空)`、不读账。记录点名与新根段指针一律落在单元区内时这不发生；我没造出能让过滤后为空的记录，推翻：造出这样一条记录。 |

端到端用例的一处弱：H2 用例拿「借来的」当期望来核「不空闲」——`.expect("C 那一圈在 R 那一槽记着 R（R 的记录点名的、R 的账里的落点）");`（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:841`）与「&named_on_the_ring_slot_of_r,」（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:863`）是同一份。借来的份被削成一半（变异 `h2-first-half-only`）、并上全部槽（`h2-union-of-all-slots`）时这条用例仍绿，只有库单测红。我拿 R 自己写出的单元（`recorded.unit_writes()`）当期望量过（K2-2），那一步用例里没有。

### K1-3　e158 单测改完，原来要钉的那件事还钉着吗

原来钉的是「抬 F 那一步的影子账真的读到了注入的故障」：旧断言是 `abandoned_roots_unreadable > 0`（diff 的减号行），名字 `mount_writable_then_raise_floor_reaches_the_injected_fault`（`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:22862`）。

- 新断言：「(Some(0), Some(0)),」（`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:22921`）加「记着的落点一槽都不空闲」。
- 判定：**钉的件事换了，原件不再钉**。新的「抬 F 不读被抛弃根的账、照记着的落点隔离」钉着：回到读盘，注入后计数变 1，红。原来的「注入的读故障确实打到了抬 F 这一遍」没有任何断言：注入落空（偏移不对、故障没武装上）时新单测一样绿。`FaultedOperationAttempt` 只带 `abandoned_roots_unreadable`、`error_debug`、`error_member`、`device_write_bytes` 四项（结构定义在 `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:1848`），没有故障命中数。
- 补位：读盘那一支（计数 1、从映射现算）转由 harness 档两条钉，我跑了：LR2 文件整份 `ok. 14 passed`，慢档 `ok. 1 passed`。
- 我没法跑 e158 单测（checker 档，按约定不跑）；改动本身我读了 diff，没编。实现员报告说它 `--no-run` 编过。
- 推翻「注入落空时新单测照绿」：在单测里把 `fault_targets` 的偏移换成越界值，看它是否仍绿——checker 档，归主 agent。

## 2. K2　I-7.4（近 K 代块未被复用）与 I-4.8（近 K 代根校验和自洽）

量法：K1 §0 那一段。「空闲」= 分配器上 `is_free`（未分配、未隔离、未被抬 F 扣住），按跨度每个槽查。「险步」= 被抛弃根还在根环里的某一步里，它引用的单元有空闲的。红 = 池级 checker 报 I-7.4 或 I-4.8。

### K2-1　H1 不进上限、照记着的落点隔离：有没有一形被抛弃根引用的单元掉出影子账、被发出去

C 那块盘（`pool_whose_newest_root_was_abandoned_by_a_crash_recovery`，C = 实例 2 的 txg 8，挂载那一刻记着 22 个落点）。探针 `h1_run`：可写挂载（故障关着，根环表记着 C 的 22 个落点），挂上之后故障开着，会话覆盖写 60 次、每 3 步跑一次 checker，C 离开根环（被这个进程的发布覆写）之后撤掉故障，再顺序写大文件写到 `NoSpaceAfterRaisingTheFloor`（写满），卸载，再跑 checker。命令：

`CARGO_TARGET_DIR=… bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 6 cargo test --offline -p singlefs-harness --test k2d_probe -- --nocapture --test-threads 6 h1_ k23_baseline`（输出 `logs/probe-h1-c.log`，`test result: ok. 5 passed; … finished in 29.02s`）

| 形 | 表里还记着 C 时空闲>0 的步 | 逐步 checker 首个违例 | 写后违例 | 卸载 | 卸载后违例 |
|---|---|---|---|---|---|
| C 的根槽读错 | `[]` | None | `[]` | 做成（计数 0） | `[]` |
| C 的根槽读回全零 | `[]` | None | `[]` | 做成（计数 0） | `[]` |
| C 的根槽与账都读错 | `[]` | None | `[]` | 做成（计数 0） | `[]` |
| C 的账读错（10y 已有的形） | `[]` | None | `[]` | 做成（计数 0） | `[]` |
| 基线（无故障） | `[]` | None | `[]` | 做成（计数 0） | `[]` |

五行一样。「空闲(写后)=22」只在 C 已被覆写离开根环（`表里还记着C=Some(false)`；探针在第 17 次覆盖写之前发现表里不再记 C）之后出现，那时它引用的单元按条款可回收，checker 不红。更早的一版探针（200 步、无写满，`logs/probe-h1-b.log`）同样：五形都 200 步不拒、`违例(写后)=[]`。

判定：**没造出被发出去的一形**。限定：只量了 C 这一块盘（实例 2 的 txg 8 被抛弃根，22 个落点，含跨度 2 的）；多条被抛弃根同时在环里、被抛弃根的落点与另一条被抛弃根重叠，我没造。

继承的一处债，不是这一批的错：新补隔离那一圈用的去重键只取起点槽、不带跨度，与已登记的 C750 同一处。实测它在这个池上没触发。

一处用例盲区（K3 展开）：新补隔离那一圈传进 `isolate_abandoned` 的跨度若被改成 1（变异 `loop-span1`），十条相关 harness 用例全绿；同时我的探针在 C 还在环里的第 13、14、15 步各看到 2 个落点空闲（`险步=[(13, 2), (14, 2), (15, 2)]`，`k3/out/loop-span1.probe.log`），因为用例核「不空闲」只查每个落点的起点槽：「.is_free(placement.slot)」（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:417`）。那一次历史里空闲的单元没在 C 离开之前被写掉，checker 没红。

### K2-2　H2 借来的 C 那一份罩不罩得住 R 引用的全部单元

R 那块盘（`common_unwitnessed_publish`：覆盖写 5 次后录一次覆盖写 R=(实例 1, txg 9)，P=(实例 1, txg 8)，崩在 R 根槽落盘之后）。探针 `h2_run`：可写挂载 M1 里让 R 的某一组单元读回全零（逐组扫，覆盖写 R 10 组、顺序写三单元 R 13 组；组 = 同一单元的各份），择根退到 P、R 记成有效根；再叠三种读零：账三棵树的根单元（az）、R 那次发布的全部记录（rz）、两样都零（bz）；挂上之后 R 的账读错 / R 的根槽读错 / 无故障；会话覆盖写 45 次、每 3 步（或有空闲时）核一次 R 的单元是否空闲或被覆盖、跑 checker，卸载后再核。「不在 C 里」= R 写出的单元（覆盖写 20 个、顺序写 26 个，各含两盘的份）里没被 C 在 R 那一槽借来的那份罩住的个数。

汇总命令 `python3 sumh2.py logs/probe-h2-<名>.log`（`research/prompts/m3-batch10z-d-code-r1-sonnet-model/probe/sumh2.py`），再 `grep -c` 数：

| 批 | 例数 | 择 P（R 被抛弃）且 C 借来 | 其中不在 C 里>0 | 险步非空或有违例 | 卸载做成 |
|---|---|---|---|---|---|
| 覆盖写 / 账读错 | 10 | 9 | 0 | 0 | 10 |
| 覆盖写 / 账与映射根挂载时读零（az） | 10 | 10 | 0 | 0 | 10 |
| 覆盖写 / R 的记录挂载时读零（rz） | 10 | 9 | 0 | 0 | 10 |
| 顺序写 / 账读错 | 13 | 10 | 0 | 0 | 13 |
| 顺序写 / az | 13 | 13 | 0 | 0 | 13 |
| 顺序写 / rz | 13 | 10 | 0 | 0 | 13 |
| 覆盖写 / R 的根槽读错 | 10 | 9 | 0 | 0 | 0（全拒，见下） |

合计借来的 C 罩住 R 全部单元的例子 61 个（9+10+9+10+13+10），一例也没有不在 C 里的单元，险步 `[]`，checker 不红。每批里「择 P」之外的那几例是 M1 没退到 P（那组单元 R 走得通），R 仍是有效根，不走 H2。

- 借来的份比 R 自己的单元多：覆盖写批借 70 或 26 个落点（按零读的是哪一组；az 批全是 26，rz 批全是 22），顺序写批借 100 / 44（az 44，rz 28），R 的 20（顺序写 26）个单元都在里面。
- R 的根槽读错（R 记有效根）：第 12 次覆盖写准入拒 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，卸载拒 `RootRingSlotKnownToHoldARoot`，10 例全这样（条款字面：有效根的槽照拒）。
- **罩不住的一形（C 里没有 R）**：bz（R 的记录、账三个根单元都读回全零，映射根在账里一起零）。C 在 R 那一槽什么也没有，借来 `None`，不走 H2；顺序写 13 组全部：
  - 挂载后 R 的 26 个单元里 10 个空闲；
  - 第 1、2、3 步 R 在环里时空闲数 8、6、4，被覆盖数 18、20、22；
  - 第 3 步 checker：`I-7.4: 被抛弃的根（实例 1、txg 9）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 …`（`logs/probe-h2-sq_bz.log`）。
  - 这一形是 D23 已声明的代价：「现算不成不隔离、不拒（主 agent 2026-10-08 定）」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）。H2 没改它：撤掉 10z-d 的副本在这一形上我没单独跑，但挂载那一遍（`rebuilt_allocator`）这一批只多传了一个 `&[]`。
  - 探针里这些例子第 13 步拒是我的读故障盖到了被重新分配的单元（R 的账单元已被新写入占用），不是产品行为。
- 覆盖写批的 bz（`ow_bz`）在我的 115 秒超时里没跑完（`rc=124`），不引。
- 没造到的：C 里有 R 的一部分而缺另一部分（既读不全记录又读不全账、映射又读得出不齐）。挂载那一遍对这一形有别的出口（映射现算、凭据、拒可写，D23 第 392 行），H2 借的是同一份 C，不比挂载时更差。推翻：造出 C 里有 R 的部分落点、借来之后 R 的某个单元空闲而 checker 红。

判定：**借来的 C 罩得住 R 的全部单元**（C 里有 R 时 61/61）；罩不住的一形是 C 里没有 R，借不到，不是 H2 造成。

### K2-3　实现员推的那一形（挂载时被抛弃根的根槽读回全零，它的单元谁都不护）

造出来了。探针 `k23_*`：C 那块盘，可写挂载那一刻 C 的根槽读回全零（或读错；或根槽与账一起读回全零），挂载做成之后故障撤掉（或一直开着），会话覆盖写 60 次、每 3 步跑 checker，再写满、卸载。五形全一样（`logs/probe-k23-a.log`）：

- 根环表不记 C（`挂载记着=None`）；
- C 的 22 个落点挂载后 20 个空闲（`挂后空闲=20`）；
- 第 3 步 checker 红：

```
I-7.4: 被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 1416 的那一份与位置条目里的校验和对不上；数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的
```

- 之后 C 被这个进程覆写离开根环，写后与卸载后 checker 不红。

红在 C 还在环里的那几步。撤掉 10z-d 的副本逐字相同：`logs/probe-pre.log` 里 `K2-3 根槽挂载时读回全零(之后恢复)`、`K2-3b`、`K2-3c` 等五形同样 `挂载记着=None 挂后空闲=20` 与第 3 步同一句 I-7.4，基线行相同。**不是这一批造成的，也不是这一批加重的**。

算不算 D16 已定的代价：
- D16 第 37 行我读了全行，没找到一句把「挂载那一刻被抛弃根的根槽读坏 / 读回全零，它的单元没人护」写成代价。它写的读回全零是回收门槛钉在可能住过的最旧 txg（护已释放的槽），这里 C 的单元挂载后在现行账里本就是空闲的，钉不到。
- 最近的是 checks-owed 的 C747（较新发布 R 在挂载那一刻整个看不见，R 的单元没人护）：「判别子观测不到，判决定这一形不修」（`.claude/kb/checks-owed.md:596`）。K2-3 是它的被抛弃根版：挂载那一刻看不见，进程内表里没有它。条款没为这一版单独写，归主 agent 判（并入 C747 / 另立一格 / 修）。
- 推翻「造得出」：在模型目录的探针上复跑 `k23_` 五条，任一条 `挂后空闲=0` 且逐步 checker 不红。对照（根槽读得出）是同一份日志里的基线行：`挂载记着=Some(22) 挂后空闲=0`。

## 3. K3　测试判别力

量法：在副本 `repo`（底 = 冻结副本）上逐条施加变异（`k3/mut.py`，原文须在文件里恰好命中一次），跑三段，再还原（`k3/run.sh`，`trap` 还原）：①`cargo test -p singlefs-core --lib`；②两份新 harness 档文件整份（`raise_floor_takes_…` 8 条 + `an_abandoned_roots_unreadable_…` 14 条）；③（加 `WIDE=1` 时）另五个相关目标（`admission_raises_the_floor_before_refusing`、`raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount`、`root_ring_slot_reread_outcomes_…`、`writable_mount_keeps_a_released_slot_named_by_an_unapplied_record_isolated_until_its_root_ring_slot_is_overwritten`、`raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window`）。基线三段全绿（§0）。每条变异的摘要 `k3/out/<名>.txt`。26 条自造变异。还原由 `mut.py restore` 断言（新文本恰好命中一次才换回）；全部做完后 `cmp` 核 `mount.rs`、`allocator.rs` 与冻结副本逐字节相同（`revert-h2-or-else` 的还原断言认不出空替换串，手工 `cp` 回冻结副本的 `mount.rs`，再 `cmp` 相同）。

我造的变异不照实现员那 15 行（它们的证红我没重做）。分三组：

**（一）把这一批的每一处原样改回去**（4 条）：全部被新文件红住。

| 名 | 改回哪一处 | 红的 |
|---|---|---|
| `revert-ceil-filter` | 准入算上限的拒集不滤被抛弃根的槽（`mount.rs:3589-3598`） | `session_writes_after_the_root_ring_slot_of_an_abandoned_root_…` 一条（卸载不走上限，所以 unmount 那条不红：符合预期） |
| `revert-rr-filter` | 抬 F 重算那一遍的拒集不滤（`mount.rs:3860-3872`） | 卸载与会话写两条 |
| `revert-new-loop` | 新补隔离那一圈 `.take(0)`（`mount.rs:1630`） | 卸载与会话写两条 |
| `revert-h2-or-else` | 去掉 `.or_else(借 C)`（`mount.rs:4131`） | `unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_…` 一条 |

**（二）改窄 / 改歪**（余下 22 条，其中 `loop-span1`、`loop-no-exemption` 放在（三））：

| 名 | 改了什么 | 库单测 | 新文件 | 另五个目标 | 判 |
|---|---|---|---|---|---|
| `ceil-inverted` | 准入拒集只留被抛弃根的槽 | 绿 | 红（会话写） | — | 拦住 |
| `ceil-all-exempt` | 准入拒集清空（有效根槽也不拒） | 绿 | **绿** | 红 3 条（`known_root_slot_still_bad_after_one_reread_refuses_the_floor_raise_before_any_write`、`the_seeded_history_whose_ceiling_read_of_the_genesis_root_slot_fails_once_refuses_the_raise_above_the_ceiling`、`a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write`） | 旧目标拦住，新文件不钉「准入时有效根槽读坏照拒」 |
| `rr-all-exempt` | 抬 F 重算拒集清空 | 绿 | 红（有效根槽卸载仍拒） | — | 拦住 |
| `h1set-includes-valid` | 「记成被抛弃根的槽」把有效根槽也算进去 | 绿 | 红（同上） | — | 拦住 |
| `h1set-includes-reread-valid` | 同，加进去的是进程内重读读出有效根的类 | 绿 | **绿** | 红 1 条（`a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write`） | 旧目标拦住 |
| `h1set-includes-badatmount` | 同，加进去的是挂载时读坏的类 | 绿 | 绿 | 未跑 | 推等价：这一类本不在「知道住着根的槽」里，加进去只多一个空落点 |
| `loop-first-only` | 新圈只处理记着的第一个落点 | 绿 | 红两条 | — | 拦住 |
| `loop-skip-already-isolated-by-slot-only` | 新圈跳过跨度>1 的落点 | 绿 | 红两条 | — | 拦住 |
| `rr-placements-only-if-no-root-read` | 交回的落点每槽只取第一个 | 绿 | 红两条 | — | 拦住 |
| `rr-placements-never-when-read-error-first-slot` | 区域 2 的槽不交落点 | 绿 | 红两条 | — | 拦住（我造得太巧：C 就在区域 2） |
| `h2-valid-compare-ignore-txg` | 有效根类不比 txg | **红 1 条** | 绿 | — | 库单测拦住，端到端不红 |
| `h2-onepass-compare-txg-only` | 一遍自证类只比 txg | 红 1 条 | 绿 | — | 同上 |
| `h2-drop-valid-reread-class` | 去掉进程内重读读出有效根这一类 | 红 1 条 | 绿 | — | 同上 |
| `h2-allow-badatmount` | 挂载时读坏的槽也借 | 红 1 条 | 绿 | — | 同上 |
| `h2-allow-abandoned-occupant` | 记成被抛弃根的槽也借 | 红 1 条 | 绿 | — | 同上 |
| `h2-unrecorded-C-gives-empty` | C 在那一槽没条目时交空串（不读盘） | 红 1 条 | 绿 | — | 同上 |
| `h2-union-of-all-slots` | 借来的是 C 全部槽的并 | 红 1 条 | 绿 | — | 同上 |
| `h2-first-half-only` | 借来的只取前一半 | 红 1 条 | 绿 | — | 同上 |
| `h2-slot-off-by-one` | 按 txg+1 的槽取 C | 红 1 条 | 红 H2 用例 | — | 拦住 |
| `raise-borrow-for-all-roots` | 借 C 不限被抛弃的根 | 绿 | 绿 | 绿 | 推等价：只有被抛弃的根才消费借来的 |

九条 `h2-*` 的库单测都是同一条：`placements_named_by_unapplied_journal_records_for_a_root_recorded_valid_answer_only_for_the_same_root_on_its_ring_slot`。H2 认同条件的默认档钉全压在这一条上；端到端用例只钉「槽错了借不到」（`h2-slot-off-by-one`），不钉借来的份对不对（K1-2 末段）。

**（三）照绿的**（26 条里 4 条绿，其中 2 条我判等价）：

1. **`loop-span1`（新补隔离那一圈把跨度改成 1）——真照绿**。库单测、新文件 22 条、另五个目标全绿（`k3/out/loop-span1.txt`：`core rc=0`、`raise rc=0`、`wide rc=0`）。探针（`logs`/`k3/out/loop-span1.probe.log`）在这个变异下看到 C 还在环里时 2 个落点空闲（`险步=[(13, 2), (14, 2), (15, 2)]`），池级 checker 在那一段历史里没红（`逐步首个违例=None`）。原因：用例核「不空闲」只查起点槽（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:417`），checker 要空闲的单元真被写掉才红。我的探针 `free_among` 查跨度内每个槽，所以看得见。
2. **`loop-no-exemption`（新圈不豁免候选根 / 现行账引用的槽）——良性照绿**：多隔离，不会把单元发出去，但 D28 第九项写的是「只除去现行账里仍分配着的槽」（`.claude/kb/decisions/28-挂载期承诺量.md:27`），不豁免会让同一个槽在「已分配」与第九项里数两遍。新用例没核准入读数的第九项，没钉。
3. `h1set-includes-badatmount`、`raise-borrow-for-all-roots`：判等价（上表），没有观测能分辨。

另三处「改回去照绿」的不是源码变异，是测试本身：
- e158 单测（checker 档，没跑）：注入落空时新断言一样绿（K1-3）。
- H2 端到端用例的期望取自被测方法的同一调用（K1-2 末段）。
- 新用例核空闲只查起点槽（上面第 1 条）。

没覆盖的 K3 项：实现员那 15 行变异我没重做；全工作区 185 个 harness 目标我只跑了 7 个（两份新文件 + 五个相关目标）；checker 档（e158 单测、变异表锚点整表核、层 0 流）一条没跑。

## 4. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 | 做到，有一形漏 | 抬 F 那一串的两处读根环（算上限、重算影子账）都换了，有效根槽照拒、管理员回退没变，三点实测；公开函数 `rollback_floor_ceiling` 与 checker 档 `on_device_modes.rs:336` 的调用方仍按旧口径拒（实测返 Err），产品路径不走它 |
| K1-2 | 做到 | 差一整圈、只比 txg、两样都没记着，三形认同条件都挡得住或走不到；「C 只记别的发布」要水位之上留 ≥ 3S 次发布，造不出（推的）；一处没人钉：C 那一槽有空条目时借到 `Some(空)`；端到端 H2 用例的期望取自被测方法本身 |
| K1-3 | 钉的件事换了，原件不再钉 | 新断言钉「不读账、照记着的落点隔离」；原来钉的「注入的故障确实打到了抬 F 那一遍」没有断言，注入落空新单测照绿；读盘那一支转由 LR2 文件 + 慢档钉（我跑了，绿）；e158 单测本身没跑 |
| K2-1 | 没造出被发出去的一形 | C 块盘五形 × 60 步 + 写满 + 卸载，被抛弃根还在环里时 0 个空闲、checker 不红；撤掉 10z-d 的副本同探针在 H1 一形第 13 次覆盖写被拒（探针分辨得出）；新圈的跨度没有用例钉（K3） |
| K2-2 | 罩得住（C 里有 R 时） | 借来的 C 罩住 R 全部单元 61/61，险步空、checker 不红；C 里没有 R 的 bz 形（记录、账、映射根全零）C 借不到，13/13 第 3 步 I-7.4 红，是 D23 第 392 行声明的代价，H2 没碰 |
| K2-3 | 造得出，非这一批造成 | 挂载那一刻 C 的根槽读回全零或读错：C 不入表，22 个落点 20 个空闲，第 3 步 I-7.4 红；撤掉 10z-d 的副本逐字相同；D16 第 37 行没写，最近是 C747，归主 agent |
| K3 | 一形真照绿，其余拦住 | 26 条自造变异：红 22（含 4 条把这一批原样改回去的）、绿 4：`loop-span1` 真绿，`loop-no-exemption` 良性绿，另两条判等价；另有 3 处测试本身的弱点 |

## 5. 没做什么

- checker 档的任何测试（`singlefs-checker-tier` 测试二进制、e158 单测、层 0 流、变异表整表锚点）一条没跑。e158 单测改完能不能编、能不能过，我没验。
- 实现员那 15 行变异的证红没重做。
- 185 个 harness 目标里只跑了 7 个；其余没跑，不知道有没有别的目标因 H1 / H2 改期望。
- 不替主 agent 采纳或出判决；K2-3 算不算已定代价、`rollback_floor_ceiling` 调用方要不要换，只报现象。
- `ow_bz` 一批超时（`rc=124`）没出结果，不引。
- 没造：多条被抛弃根同时在环里的 H1；C 里有 R 的一部分的 H2；水位之上相隔 3S 的两次发布。

## 6. 要主 agent 跑的命令

- e158 单测（checker 档）：`cargo test --offline -p singlefs-checker-tier --bin e158_root_choice_repair mount_writable_then_raise_floor_reaches_the_injected_fault`，经内存包装；看 `test result` 行。
- 判 K1-3 的「注入落空照绿」：同一单测里把 `fault_targets` 的偏移换成越界值再跑一次，看是否仍绿。
- 判 `on_device_modes.rs:336`：checker 档 `raise-rollback-floor` 模式在被抛弃根的根槽读错的状态下跑一次，看是否拒。

## 7. 草稿产物与清理

- 已删（我自己建的编译目录与仓副本，探针源码、变异表、原始输出已拷进 `research/prompts/m3-batch10z-d-code-r1-sonnet-model/`，要复跑从冻结副本重拷一份即可）：
  `/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1-sonnet/repo`（冻结副本拷贝，变异与探针都在这里做）、
  `/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1-sonnet/repo-pre`（撤掉 10z-d 的副本）、
  `/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1-sonnet/target`（3.8 GiB）、
  `/home/fy5090/.local/share/singlefs/work/m3-batch10z-d-code-r1-sonnet/target-pre`（1.1 GiB）。
- 留着的：同目录下的 `*.log`、`probe/`、`k3/`（变异表与 `k3/out/`）、`sumh2.py`、`progress.md`、`spec.json`（后者是改本报告引文路径用的替换规格）。
- 仓内只写了本报告与模型目录；没碰 `crates/`、冻结副本（变异全在副本的拷贝里，还原后与冻结副本 `cmp` 相同）。
- 期间 `ps` 见别的会话在跑 cargo（opus-probe、checker-tier 的 release 测试），没有性能测量（`qemu-system`、`vm-bench.sh`、`fio`）；我的命令都加了 `nice -n 19`，内存包装退出码都是 0，没有 250–254。
- 中途有一次我自己把 `run.sh` 套在 `timeout 80` 里被杀，留下一份已施加的变异与一个在等内存准入的 cargo；我先用 `mut.py restore` 还原并 `cmp` 核过冻结副本一致，再去停那个等准入的进程，`proc.py stop` 回「已经不在了」、`ps` 核过那串进程已退出，之后 `run.sh` 里换成脚本自己的阶段超时并加 `trap` 还原。那两条变异（`ceil-inverted`、`rr-all-exempt`）之后各整条重跑，表里的结果是重跑的。
