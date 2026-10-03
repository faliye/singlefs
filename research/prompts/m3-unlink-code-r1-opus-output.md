# m3-unlink-code-r1 攻方腿（Opus）报告（2026-10-03）

攻击面：正文第三节 K1–K6，重点 K2、K3、K6；正文第二节「已知、不用再攻的」三条不攻。前几轮判决：无。
所有数都在我自己的草稿副本 `/tmp/claude-1000/m3-unlink-code-r1-opus/tree`（从冻结副本 rsync，改动见模型目录 `copy-changes.patch`）上量，不是入库装置上的数。
代码引文的行号对着冻结副本；被引的 8 份 crates 文件与主仓逐字节相同（sha256 前 16 位两边一致，命令与输出在第七节）。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-unlink-code-r1-opus-model/`：
- `harness-tests/*.rs` 拷进副本的 `crates/singlefs-harness/tests/`；`opus-proto-crash-states/` 拷成副本的 `crates/opus-proto-crash-states/`，`src/main.rs` 第 7 行的 `#[path]` 改成副本里 `tests/common_admission/mod.rs` 的绝对路径；`copy-changes.patch` 打进副本（工作区成员加这一个包；`mount.rs` 加一条只在环境变量 `OPUS_ARM_MOUNT_WRITABLE_ON_CONTINUATION_SHORT` 设了时才走的原型臂，只 E8b 用）。
- harness 档原型（工作目录：副本根）：
  `RUN_WITH_MEMORY_CAP_PARALLEL_REASON=<理由> nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 cargo test --release -p singlefs-harness --test <目标> -- --nocapture --test-threads 1`
  目标：`opus_proto_deletion_slots`（E1，产物是 debug 跑的）、`opus_proto_fill_then_unlink`（E2）、`opus_proto_continuation_error_accounting`（E3、E8、E8c；E8b 要设上面那个环境变量）、`opus_proto_deletion_heavy_histories`（E4，种子与步数取 `OPUS_FIRST_SEED`、`OPUS_SEEDS`、`OPUS_OPS`）、`opus_proto_directed_unlink`（E5）、`opus_proto_walk_to_the_landing_wall`（E7）、`opus_proto_create_after_unlink`（E10）。
- 崩溃状态原型 E6（不在 checker 档包里，名字不带 layer0；全量 69671 态，每片约 1.16 万态）：
  先枚举，`OPUS_SHARD=<i>/6 OPUS_PHASE=enumerate OPUS_DUMP_DIR=<草稿目录> … cargo run --release -q -p opus-proto-crash-states -- c1`（i = 0…5）；
  再逐片重挂，`OPUS_PHASE=remount OPUS_RANGE=<起>,<止> OPUS_PERSISTED_FILE=persisted<i>of6.txt OPUS_DUMP_DIR=<同上> … -- c1`。
- 每份文件的 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里跑）：

```
cd09cd8dadac89a0ce383a5b538e9671396d2762470208c80d9d68608aef1894  copy-changes.patch
6db4850f7422e27a7319a7eed55de1819f73f235e5a2ddb57e03dfd521706297  harness-tests/opus_proto_continuation_error_accounting.rs
e95647b305c2f80c72a6c37a4ee80f4376582036da2749797b8a2bcc9c66323f  harness-tests/opus_proto_create_after_unlink.rs
651d4fa32aaa11f37193ada04aa7e839279400c73a2e019a6e68bc68c0a42b3f  harness-tests/opus_proto_deletion_heavy_histories.rs
5d4b358564f1703574350ac0266c8aeca452a25bb6e0a51162493d7c9147b684  harness-tests/opus_proto_deletion_slots.rs
4be6def2c6de025869d35c291f225a67fc49cfc661406038e8e41344a808b76f  harness-tests/opus_proto_directed_unlink.rs
6cdb17329fc939a5f7fe7854f986f8974607b95b69a0b08b7b3b985877c27ac8  harness-tests/opus_proto_fill_then_unlink.rs
af80a67d545bcdaff350076190af39496f90cc0997fdbd60ef629dc9f7a065df  harness-tests/opus_proto_walk_to_the_landing_wall.rs
a1dd78261a69a1f0a62e9794e3a3487bb598c5a412876c8fd8372be0240b22b0  opus-proto-crash-states/Cargo.toml
7460a442512e296f92501654ff4d99b1b6ab81c803b57f431ee75eac810f385d  opus-proto-crash-states/src/main.rs
efba86ac0f9ad686b30e2cabbfbe2b12024220c57a3ed86fcd1c1d6d3bdbefc2  outputs/e10.log
a240b233cc989c71af894a9bac9b07918205a6dc081b871249b9367fb1ee99ab  outputs/e1.log
f89df24270c7953e3f6634f1c5fa5f0cb680a8e20ba90b5dad353cbee911a0d9  outputs/e2.log
01c6ecc268c5fea607f78f26de2d271b10fff601d6922960da726f77504bf278  outputs/e3.log
7f8a15676bd7ae9e810d3daa8ce8454521a33c43ed2aa83f8ba5162fc74ed208  outputs/e4-240-b2.log
23bcadb57544d51a3a0be9f1af4efdd669c4ae8dd5a475ed6c23b9f63c80e1b5  outputs/e4-240-probe.log
0d86572d0e050f44c3349bbb41c1d8563a30ebcbc2ff7de8234f78dae84d8217  outputs/e4-384-b1.log
b0b2b6413baa0ee57f156fd0a39817bf7e9ae83e1436b78e792ebe0f7d6969aa  outputs/e4-384-b2.log
a17196e1996af040e6d88ef281a45e5e180cf14f3d1eb14e1debb0c12bc4d344  outputs/e4-probe.log
e0a563d073c8636e375a38f5ff6cd8286cb22d840ecf0cd4c9ff6e649e727b01  outputs/e5.log
f370bf4d73dda2c86543d33be497955dca4b48cfe1c9484830dc27bff3d01dba  outputs/e6-c1-r0a-detail.log
0c24628220eb970bf7e33931ce2c58d33210b2424dab46e7ea3287d734ab3fc4  outputs/e6-c1-r0.log
1b09e4c93f51af9df0b42935427b67fc44b04a4968bb46f04270db91f9c4bc24  outputs/e6-c1-r1.log
1683f8b263b205859fa320d67641763e1c99827d2fdb987cc85c58cbd5b86420  outputs/e6-c1-r2.log
ee457178e1704c82cd829ae8ffa07acebb72ec391a95fe5aec817b6dc1a67acf  outputs/e6-c1-r3.log
a2cf711b70cc7194e724a4b020cad01c02a832ba77583ad37301d603ad059c1e  outputs/e6-c1-r4a.log
a84bc8300a2282e79926ff0fbe39ab6b40cb6767f7e2a3cf6d95857e48cef455  outputs/e6-c1-r4b.log
50b1be3813211fd3ee82951e4b271de261390961f6ccb8a0260d847a3b4340d0  outputs/e6-c1-r5a.log
77815bbd571763db7039816283afe7c8513627d63ccc76c4f50cdfee4a5f3e51  outputs/e6-c1-r5b.log
92b4905b7bd63c2e792d90171c13cb054fb967d5eac94fd1396c849cffd4a28a  outputs/e6-c1-s0.log
14bede7bb3776e9346a7954592cf5bf0654abce1d91c78fa364370f386a1e4d8  outputs/e6-c1-s1.log
2e3a8d47979fa25136310205ec2933a9b2d0df2142d9d7dccddefe9e1ada89b3  outputs/e6-c1-s2.log
bfc4afb9658dc5794fa22f5dcf9cc9f79f56b13ad76476cac447c6fa13c73a47  outputs/e6-c1-s3.log
8e7334b9d0ad9c71f5a4509a1d85b782e72885d86b4fdd9bf51a665db84ed2c9  outputs/e6-c1-s4.log
158d886955be559f8dcccd560caf17fd458216087de349a0601c442d7f559e93  outputs/e6-c1-s5.log
57dcdfd3962cb5557872f8f8279acb8167e2b295ba3068aaa5867ca15fe56ba2  outputs/e7.log
e3d9200658d06ba53622f1b5f1005142600f8c519eb9223bade41546a5b4a569  outputs/e8b.log
0765e73d174ea5e04fd4118e6312701eab276057e3373344a02e4ade6feb9e9a  outputs/e8c.log
b20b70f868a3e5b7dccaa1c0d3070acecb67f69d242eea393eb93dfd0a506b4d  outputs/e8.log
```

## 各格判定一览

| 格 | 编号 | 判定 | 一句话 | 量在哪 |
|---|---|---|---|---|
| K2 | A1 | 打中（实现与它自己写的契约不符） | 挂载续做推满仍不够、交回 `MountError::Publish` 时，错里一份已落盘发布的写账都没有：副本上这次挂载落盘 13 次、设备一层 333 次写调用，错里记 0 | E3 |
| K2 | A2 | 打中（会话侧同一类） | 回退到意图那一版之后覆盖写、顺序写、unlink：会话先走空落盘 4 次，再拒成 `NoFileVersionToChange`，拒绝里不带这 4 版与推的抬 F | E5a |
| K6 | A3 | 打中（覆盖缺口，A2 的后果） | 随机历史执行器在现行版本有删除中的文件时一律不调用户改动，F1「会话里每次用户改动先走空」在随机历史上一次都没被模型比过 | 代码、E4 计数 |
| K2 | A4 | 打中（推的，checker 档没跑） | 崩溃注入：没崩过的那次挂载在续做里交回错时取不出「续做之前发了几次」，续做那几步的写全归暖机段，`Continuation` 段摆不到二次崩溃 | 代码 |
| K6 | A5 | 打中（钉值丢失） | Q-T 改钉把钉 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 的唯一用例改成了它的反面；HEAD 里测试有 3 处引用，冻结副本的测试 0 处；副本上这条拒绝照样走得到，「一个写都不发」仍成立 | git、E8c |
| K2 | A6 | 打中不分辨臂 | `mount.rs` 文档「下一次挂载接着走」在仓里那条续做用例的窄盘上不成立：之后每次挂载都在取号前被落点拒；把续做改成「挂载照样做成可写」再卸载，同样卡死。病根是准入关着造的盘面 | E8、E8b |
| K6 | A7 | 覆盖缺口（不是实现缺陷） | 模型的文件恒是一个数据单元，非最后一批的删除批（只改尾叶那一支）在随机历史里从没出现过 | 代码 |
| K2 | N1 | 没打中 | 续做推的抬 F 那一段 69671 个崩溃状态：恢复 0 oracle 违例、checker 0 红；逐个重挂 69647 个做成、队列走空、挂载之后 checker 0 红；20 个是撕裂的系统配置轮换（已定只剩只读挂载）；4 个同 A6 | E6 |
| K3 | N2 | 没打中 | W 上界：58 种配置里每次删除类发布每盘新占槽 W' 最大 20（4 GiB）、15（小盘），W 是 82–368、30 | E1 |
| K3 | N3 | 没打中（准入开着） | 准入开着的历史里删除类发布一次都没被空间拒：126 段填满后 unlink（含崩在中间重挂），22 段写到式子放得下的最大再 unlink，512 段偏删除随机历史 | E2、E7、E4 |
| K3 | O1 | 观测（代价，不算缺陷） | 落点那一判对删除类发布要「普通分配 + ckpt_cost（分配记录树按 K1 整棵树）」：4 GiB 上 ckpt_cost 540–546 槽，实写 ≤ 20 槽；小盘 13 对 ≤ 15 | E1、代码 |
| K3 | O2 | 观测（代价） | unlink（意图 + 最后一批）之后准入式子的可用(d) 反降 20–24 槽/盘（三档小盘 18 格都是），环窗口照扣全额 | E10 |
| K3 | O3 | 小出入（推的） | 墓碑预付里的 H 取内存里那一版的形状，条款写的是从墓碑树根节点码 2 头现读；两者同值 | 代码 |
| K2 | O4 | 小出入（推的） | `publish_unlink_intent` 遇队列不空是 panic、不是类型化错误；产品路径（会话先走空）走不到 | 代码 |

E 编号与模型目录里的用例对应：E1 删除类发布新占槽，E2 填满后 unlink，E3/E8/E8b/E8c 续做交回错之后，E4 偏删除随机历史，E5 定向历史，E6 崩溃状态枚举，E7 写到最大再 unlink，E10 删后建 inode。

## 一、K2 续做、走空与推抬 F

### A1　续做推满仍不够交回的错里没有已落盘发布的写账（打中）

历史（E3，副本上跑）：仓里 `admission_raises_the_floor_before_refusing.rs` 那条「续做推满仍不够」用例的盘面——两块单元区 256 槽、不为环窗口补足的窄盘，建池时准入关着；覆盖写 2 次，顺序写把第一个文件长到 14 个单元；进程里发意图与 1 批（批大小 1）后进程退出；重开可写挂载（准入判着、批大小 1）。每一步许它的那一句：
- 续做与会话共用走空，`mount_writable` 在写行与暖机之后走到空：「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己」（`.claude/kb/decisions/08-核心索引结构.md:463`）。
- 续做里一批被落得下那一判拒：「删除类发布（意图、各批、挂载里的续做批）与管理员回退也不判式子，删除类发布的落点那一判照判」（`.claude/kb/decisions/28-挂载期承诺量.md:36`）。
- 推满仍不够交回错：「return Err(walk_failed(refusal));」（`crates/singlefs-core/src/mount.rs:4400`），而 `walk_failed` 里写死「writes_of_persisted_publishes: Vec::new(),」（`crates/singlefs-core/src/mount.rs:4375`）。

量到的（`outputs/e8.log`，原样）：

```
test e3_writes_handed_back_by_a_mount_whose_continuation_stays_short ... E3 cause=SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_devices: [DeviceShortOfDemand { device: DeviceIdentity(0), available: AvailableBytesOnOneDevice(147456), demand: BytesOnOneDevice(245760) }, DeviceShortOfDemand { device: DeviceIdentity(1), available: AvailableBytesOnOneDevice(147456), demand: BytesOnOneDevice(245760) }] })
E3 publishes_landed_by_the_failed_mount=13 writes_of_persisted_publishes.len()=0 accounted_write_calls(before+persisted+failed)=0+0+0 device_level_write_calls_excluding_barriers=333
```

这次挂载落盘 13 次（写行、暖机、续做 11 步，只读挂载读到的 txg 差），设备一层 333 次写调用（不含屏障；其中取号那两次系统配置槽写按契约不在错里），错里三格合计 0。

四句：
1. 分不分辨臂：不分辨 L4 的哪一臂（交回错或做成可写）；它是代码与它自己写的契约不符，局部可改。
2. 系统当时看得到吗：看得到。同一个函数里续做发成的每一步已经收进「continuation.steps_published.extend(steps_published);」（`crates/singlefs-core/src/mount.rs:4363`），每一版带自己的写账；写行、暖机与挂载推的抬 F 在 `establish_instance` 手里。
3. 满足的是哪一句的字面：`PublishSequenceFailed` 字段文档「失败之前这一串里已经落盘的发布各自的写，按先后（可写挂载写行那次在最前）；第一次就失败时是空的。」（`crates/singlefs-core/src/mount.rs:727`），与它的理由「与失败那一次落盘阶段已记的写（增补 2 收口表第 58 行：不交出来，设备一层数到的写与程序交得出的账对不上）。」（`crates/singlefs-core/src/mount.rs:719`）。同一函数的文档另写了「推了的那几串不在错里」（`crates/singlefs-core/src/mount.rs:4318`），与第 727 行互相矛盾。
4. 改法在打中的格上还中不中：改法甲（我提的，推的、没实现、被攻过零轮）——错里按 txg 次序交出写行、暖机、挂载推的抬 F、续做每一步与续做推的每一串的写账，连同推满仍不够的停因；它正修这一格。改法乙（把第 4318 行那句写回 kb、契约改成「续做那一段不交」）也让字面一致，但第 719 行的理由（设备一层与程序账对不上）照样成立，算不上修。

### A2　会话先走空之后拒成 `NoFileVersionToChange`，拒绝不带走空落盘的那几版（打中）

历史（E5a，副本上跑，4 GiB 两块盘）：第一个文件顺序写成 4 个单元，会话装批大小 1；会话 unlink（意图 + 4 步走完）；挂着时回退到意图那一版（`deleted_inodes` 回来一条）；再发一次用户改动。量到的（`outputs/e5.log`，原样）：

```
test e5a_user_changes_after_rolling_back_to_the_intent ... E5a after-rollback-to-intent queue=[InodeNumber(1)] change=overwrite -> Err(NoFileVersionToChange: carries no versions); publishes_landed_on_disk=4 current_member=WithTreesAndNoFile checker=0
E5a after-rollback-to-intent queue=[InodeNumber(1)] change=create -> Ok(versions=5, raises=0); publishes_landed_on_disk=5 current_member=WithFile checker=0
E5a after-rollback-to-intent queue=[InodeNumber(1)] change=unlink -> Err(NoFileVersionToChange: carries no versions); publishes_landed_on_disk=4 current_member=WithTreesAndNoFile checker=0
E5a after-rollback-to-intent queue=[InodeNumber(1)] change=seqwrite -> Err(NoFileVersionToChange: carries no versions); publishes_landed_on_disk=4 current_member=WithTreesAndNoFile checker=0
```

覆盖写、顺序写、unlink 三种都是先走空落盘 4 次，再拒；拒绝成员不带那 4 版，也不带推的抬 F（这里没推）。代码自己写明了：「走空一个写都不发；走空把带文件的一版走成文件集合为空的，那几步已落盘），再拒这次改动。」（`crates/singlefs-core/src/mounted_session.rs:95`）；拒在「return Err(UserChangeRefused::NoFileVersionToChange);」（`crates/singlefs-core/src/mounted_session.rs:390`）。同一枚举的别的成员都带「被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）。」（`crates/singlefs-core/src/mounted_session.rs:128`）。池级 checker 绿：盘面没错，错的是交回的账。

四句：不分辨臂；系统看得到（走空交回的 `PublishedInThisChange` 就在手里，第 390 行之前刚赋给 `published_in_this_change`）；字面满足的是 A1 第 3 句同一条理由（第 719 行），会话这一侧没有单独的契约句，判据是同一枚举里别的成员都交；改法：让这一种拒绝也带 `PublishedInThisChange`（推的、零轮），它正修这一格，并且解掉 A3。

### A4　崩溃注入：挂载在续做里交回错时，续做的写归进暖机段（打中，推的）

`mount_and_publish_once_on_the_crash_image` 取「续做之前发了几次」只在挂载做成时有：「let publishes_before_the_deletion_queue_continuation = mounted.as_ref().ok().map(|mounted| {」（`crates/singlefs-checker-tier/src/crash_injection.rs:1605`）；分段函数的文档写明「从它起是续做（没崩过的那一次挂载被拒、交不出这个数时 None：写行之后都算暖机）。」（`crates/singlefs-checker-tier/src/crash_injection.rs:2194`），代码「(_, Some(_) | None) => WritableMountPhase::WarmUp,」（`crates/singlefs-checker-tier/src/crash_injection.rs:2209`）。A1 的历史正是这一形：挂载交回错之前续做已落盘 11 步，这 11 步的写在崩溃注入里全算暖机段，`WritableMountPhase::Continuation` 那一段一个二次崩溃点都摆不到。

四句：不分辨臂；系统当时看不到——挂载交回的 `MountError::Publish` 不带写行、暖机与续做各自的发布数（与 A1 同根），注入器只能退回「全算暖机」；字面打中的是正文 K2 那一问「崩溃注入 Continuation 那一段认得出续做的写」；改法：A1 的改法甲让错里带按次序的各次发布，注入器就能在错的那一支同样切出续做段（推的，checker 档没跑，零轮）。这一格只按代码推，没有在 checker 档上跑出来（重型，不归我）。

### A6　「下一次挂载接着走」在仓里那条用例的窄盘上不成立（打中不分辨臂）

`continue_the_deletion_queue_until_it_is_empty` 的文档：「三种都是已经发成的那几步与推成的那几串留在盘上，下一次挂载接着走。」（`crates/singlefs-core/src/mount.rs:4319`）。同一份盘面（A1）上连挂四次（E8，`outputs/e8.log`，原样）：

```
test e8_mounting_again_after_the_continuation_stayed_short ... E8 attempt 1: Publish(PublishSequenceFailed { cause: SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_devices: [DeviceShortOfDemand { device: DeviceIdentity(0), available: AvailableBytesOnOneDevice(14745; publishes_landed=13
E8 attempt 2: PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { instance_to_acquire: InstanceGeneration(3), publish_index: 0, warm_up_publishes_planned: 1, unit: TreeTable, refusal: NoFreeSlotOnAnyDevice }; publishes_landed=0
E8 attempt 3: PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { instance_to_acquire: InstanceGeneration(3), publish_index: 0, warm_up_publishes_planned: 1, unit: TreeTable, refusal: NoFreeSlotOnAnyDevice }; publishes_landed=0
E8 attempt 4: PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { instance_to_acquire: InstanceGeneration(3), publish_index: 0, warm_up_publishes_planned: 1, unit: TreeTable, refusal: NoFreeSlotOnAnyDevice }; publishes_landed=0
```

第一次失败的挂载自己吃掉了最后的空槽（13 次发布、F 没动），之后每次挂载在取号之前就被落点拒，池只剩只读。对照臂（E8b，副本上把推满仍不够改成「挂载照样做成可写、队列留着」，只在设了环境变量时走）：挂上之后正常卸载，卸载那一串在预演里就落不下，再挂同样在取号前被拒（`outputs/e8b.log`）：

```
test e8b_arm_mount_writable_then_unmount_then_mount_again ... E8b first mount (arm): queue=1 continuation_steps=11
E8b unmount: RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { refused_publish_in_the_sequence: 2, publishes_in_the_sequence: 2, cause: PlacementRefused { unit: AccountingTree, refusal: NoFreeSlotOnAnyDevice
E8b second mount: PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { instance_to_acquire: InstanceGeneration(3), publish_index: 0, warm_up_publishes_planned: 1, unit: TreeTable, refusal: NoFreeSlotOnAnyDevice }
```

四句：不分辨臂（两臂一样卡死）；病根在两臂共用的前提——盘面是准入关着造的，式子的空间保证不在；准入开着的历史里我没走到这一形（N3）。字面打中的只是第 4319 行那句无条件的「下一次挂载接着走」。改法：把那句收成「准入开着时」（推的、零轮），或另立一笔账记「准入关着造的盘上续做交回错之后池只剩只读」。

### N1　续做推的抬 F 那一段的崩溃状态（没打中）

装置（E6，`opus-proto-crash-states`，副本上跑）：A1 那份盘面但只崩在意图之后（0 批），重开可写挂载（准入判着、批大小 1）全程录流；这次挂载续做 14 步，第 11 步之后被落得下那一判拒、推一串两次空发布的抬 F 再接着走。整条挂载流的闭式状态数 107376761，超过 10⁶，不跑；取「推之前那一步落盘之后」起、到推的那一串最后一次根槽写为止那一段（录制流 [434, 490)，42 次写、10 段），连撕裂的原地覆写共 69671 态，按层 0 的枚举域每一段都展开（`enumerate_layer0_in_state_slices_or_one_shard`，分 6 片）。推完之后续做的下一步不在段里：把它带上是 16846898 态，超过 10⁶。

```
E6 C1 narrow256 P2 U14 crash-after-intent uncrashed mount: continuation_steps=14 continuation_raises=[2] whole_mount_stream writes=463 segments=78 closed_form_states=107376761
E6 C1 narrow256 P2 U14 crash-after-intent piece publishes=["step", "cont-raise", "cont-raise"] ops=[434,490) writes=42 segments=10 closed_form=69651 states_with_torn=69671
```

每一片的枚举（恢复之后 oracle 不给版本：这一段每个根下面都没有文件，读出内容即违例；池级 checker 由枚举器自己跑）：6 片合计 69671 态，oracle 违例 0、checker 判红 0、恢复结局全是 NoFile（`outputs/e6-c1-s0.log` … `e6-c1-s5.log`，每片一行 `enumerated states=… oracle_violations=0 … checker_violated_states=0`）。

每个状态再可写挂载一次（批大小 1、准入判着），挂载之后的镜像跑池级 checker（`outputs/e6-c1-r*.log`）：69647 个做成、队列走空、挂载之后 checker 0 红；20 个 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`；4 个 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`。
- 那 20 个里逐个看过 10 个（`outputs/e6-c1-r0a-detail.log`，第 0 片前 2000 态），持久位都落在写表第 42 位起的撕裂项上；另 10 个只看了成员名。这是已定的代价：「撕裂的系统配置轮换也落在这里：代价是那之后只剩只读挂载（同一判决 L1）。」（`crates/singlefs-core/src/mount.rs:439`）。
- 那 4 个的持久位只在段首两次写之内（含全不持久，即「推之前那一步刚落盘就崩」），与 A6 同一形：准入关着造的窄盘，空槽被续做吃光。
- 取样范围：一种盘面（256 槽窄盘、覆盖写 2 次、14 单元、崩在意图之后），一段（推的那一串）。没取：会话里推的抬 F（仓里 checker 档已有覆盖写那一形的枚举）、续做的删除批本身（单段就超 10⁶）。本腿原型全量合计 69671 态，远在 10⁷ 之内。

### K2 其余几问（按代码读，没打中）

- 挂载续做与会话走空：底下是同一个 `walk_the_deletion_queue_until_it_is_empty`；推抬 F 的外层循环是两份（`mounted_session.rs` 一份、`mount.rs` 一份），A1、A2 的差别正出在这两份外层上。
- 「每一次被拒的删除类发布各算一次准入」：两处都在这一趟走空有一步落盘时清零，「publishes_pushed_in_this_admission = 0;」（`crates/singlefs-core/src/mounted_session.rs:561`、`crates/singlefs-core/src/mount.rs:4361`），与文档「这一趟走空有一步落了盘，下一次被拒的就是后面那一次发布，这次准入已推的数从 0 数起。」（`crates/singlefs-core/src/mounted_session.rs:534`）一致。E2、E7、E4 里删除类发布一次都没被空间拒，这一条在我的取样里没被执行到（只有 A1、N1 那份准入关着造的盘面推过一串）。
- `deleted_inodes` 至多一条 key：会话先走空保证它；直接调公开的 `publish_unlink_intent` 遇队列不空是断言，「"发意图之前队列已走空：任何一版的 deleted_inodes 至多一条 key（D8 已定项 16 第 6 条）"」（`crates/singlefs-core/src/transaction.rs:5770`），不是类型化错误（O4）。条款那一句「多于一个根装得下那一支拒成类型化错误」管的是 2031 条以上那一支，这里不违字面；产品路径走不到，只记一笔。
- 推满仍不够时盘上留下的几版：意图版与若干批之后的版，都是不崩也会经过的中间态；E5、E6 上 checker 全绿。

## 二、K3 准入

### N2　W 上界（没打中）

E1：两块 4 GiB 盘与 384 槽那一档小盘，批大小 K ∈ {1, 3, 144}，文件 1–600 个单元，先覆盖写 0 或 10 次，经会话 unlink；每一次删除类发布按分配记录数「这一版里分配代 = 这次 txg、未释放」的槽，逐盘取最大，与实现算的 W（`ring_window_on_one_device` ÷ (3S + 1)）比。58 种配置，`exceed=true` 0 次（`grep -c 'exceed=true' outputs/e1.log` 输出 0）。W' 最大：4 GiB 上 20（W 取 82、86、368），小盘上 15（W = 30）。最大那几行（`outputs/e1.log`，截前 200 字）：

```
E1 base width=FourGibibytes K=144 U=600 P=0 steps=6 raises=0 W=368 ring=9200 ckpt=546 maxW'=20 exceed=false per_step=[16, 17, 20, 20, 19, 12]
E1 base width=FourGibibytes K=1 U=300 P=10 steps=301 raises=0 W=82 ring=2050 ckpt=542 maxW'=19 exceed=false per_step=[16, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 17, 16, 17, 17, 17, 17, 17, 17, 17, 19
E1 base width=UnitAreaOf384Slots K=1 U=80 P=10 steps=81 raises=0 W=30 ring=750 ckpt=13 maxW'=15 exceed=false per_step=[15, 13, 11, 13, 13, 13, 13, 13, 13, 13, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 1
```

取样的限度：单元都是一次顺序写连着落的，分配记录树每批只碰一两片叶；W 里「每层 min(N_ℓ, 2·(K + 26))」那一项要单元散在很多片叶上才逼得近，我没造出散布的落点（覆盖写 10 次也没散开）。「24 = 普通分配 16 + 固定点根 4 + 映射树根以下 4」那一拆（「pub const DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST: u64 = 24;」，`crates/singlefs-core/src/admission.rs:298`）在分配记录树每层改很多片叶时，映射树根以下改的叶会不会多于 4，我没量到那一格。

### N3　准入开着时删除类发布被空间拒（没打中）

- E2：三档小盘 × K ∈ {1, 2, 144} × 覆盖写 0/5/20 次 × 先不先重挂，文件每次多一个单元长到式子拒，再经会话 unlink：54 段全做成、推抬 F 0 次；同样的盘面在进程里发意图与 j ∈ {0, 1, 3, 10, 30, 60} 批后崩、重挂续做：72 段全挂上、队列走空。126 段全部 checker 0 违例。
- E7：同三档小盘，一次顺序写写到式子放得下的最大（384 槽 120 单元、256 槽 57、240 槽 49；先重挂的少几个），再在进程里一步一步发删除、遇空间拒就停：18 段没有一段被拒；会话一路做完的 4 段也是 0 次推抬 F（`outputs/e7.log`，原样）：

```
E7S width=UnitAreaOf384Slots K=1 units=120 unlink=Ok(versions=121, raises=0, refusals=[])
E7S width=UnitAreaOf384Slots K=2 units=120 unlink=Ok(versions=61, raises=0, refusals=[])
E7S width=UnitAreaOf240Slots K=1 units=49 unlink=Ok(versions=50, raises=0, refusals=[])
E7S width=UnitAreaOf240Slots K=2 units=49 unlink=Ok(versions=26, raises=0, refusals=[])
```

384 槽那段 121 次删除类发布，按 E1 量的每次 11–15 槽推，合计比单元区（1199 槽）还多，会话内的回收接得上（推的，没逐次数回收）。
- E4 的 512 段随机历史里 `refusals` 一栏没有一次落在 unlink 上（第四节）。

### O1　落点那一判对删除类发布要的槽（观测）

删除类发布有普通分配就判落点：「let units_landing_is_judged = match allocator.space_admission() {」（`crates/singlefs-core/src/transaction.rs:6230`），要的提交内生块槽 = 普通分配那一半 + ckpt_cost，ckpt_cost 里分配记录树按整棵树：「拿 ckpt_cost 的最坏数」（`crates/singlefs-core/src/admission.rs:758`）。E1 上 4 GiB 的 ckpt_cost 是 540–546 槽，删除类发布实写每盘 ≤ 20 槽，要的是实写的约 27 倍；小盘上 13 对 11–15。按 K1 式子推（没量）：单元区越大倍数越大，两块 1 TiB 盘时每块盘的叶约 8.3 万片，ckpt_cost 约 16.6 万槽，而 W 在 K = 144 时约 710。

它会不会误拒：落点那一判留出的 ckpt_cost 正是式子里扣的 checkpoint 保留池，所以准入开着时删除只动得了环窗口、切换预留与墓碑预付那几截；N3 的取样里这几截从没被用完。拒了要推抬 F，推满仍不够才报错（A1、A6 那一形）。结论：倍数是真的，我没造出准入开着时因它误拒的历史；推翻条件是找到一段准入开着、删除批实写放得下而落点那一判拒、推满仍不够的历史。

### O2　unlink 之后准入可用量反降（观测）

E10：三档小盘，覆盖写 0/3/6/10/20/40 次之后同一份盘面分两路：甲 unlink 第一个文件（意图 + 最后一批）再建 4 个 size 0 的 inode；乙不删、直接建 4 个。18 格里 unlink 都让式子的可用(d) 掉了 20–24 槽/盘（15 格 20、1 格 22、2 格 24），掉到 0 或负的有 5 格；这几格建 inode 都是推一串抬 F 之后做成，乙路在同样的格上也推过一串。原样（`outputs/e10.log`）：

```
E10 width=UnitAreaOf384Slots overwrites_done=20 available_before=[1, 1] | keep: creates=["Ok(raises=1)", "Ok(raises=0)", "Ok(raises=0)", "Ok(raises=0)"] | unlink=Ok(raises=0) available_after_unlink=[-21, -21] creates=["Ok(raises=1)", "Ok(raises=0)", "Ok(raises=0)", "Ok(raises=0)"]
```

机理（按条款推）：环窗口是常驻扣项，「- **环窗口**（常驻扣项）：删除各批新写的兜底」（`.claude/kb/decisions/28-挂载期承诺量.md:29`）；删除那几次的新写进了已分配，环窗口照扣全额，删掉的单元又被最近 4 个状态钉着，所以删除自己的新写先从用户那一份里出。E4 里 240 槽那两批建 inode 被拒 38 + 101 次（`refusals` 栏里 `NoSpaceAfterRaisingTheFloor` 共 38 + 104 次），模型都认（区间里的墙）。它是不是缺陷取决于条款要不要「删除之后的可用量不低于删除之前」，条款没这一句；记成代价交主 agent。

### O3　墓碑预付里的 H 从哪读（小出入，推的）

条款：「H 是现行那一版墓碑树的内部层数（零根与只有根时 0），从墓碑树根节点码 2 头现读」（`.claude/kb/decisions/03-空间分配.md:52`）。实现取这次发布算出来的内存形状：「tombstone_tree_internal_levels: resolved.tombstone_tree.internal_levels(),」（`crates/singlefs-core/src/transaction.rs:4112`），即根层级减一。两者在一致的版本上同值；条款写「现读」是照 ckpt_cost 的先例防内存与盘上分叉，这里没走那一步。没造出两者不等的历史。

## 三、K6 模型与随机历史

### A3　F1「会话里每次用户改动先走空」在随机历史上从没被模型比过（打中，覆盖缺口）

执行器遇现行版本有删除中的文件就不调覆盖写、unlink 与建 inode：「/// 现行版本的 `deleted_inodes` 不空（unlink 的一批被拒之后、回退到删除中的一版之后）：覆盖写与 unlink 经会话先走空，」（`crates/singlefs-harness/src/history.rs:1333`），理由写在「模型比不了已落盘的那几版——执行器不调。」（`crates/singlefs-harness/src/history.rs:1335`）。所以会话侧的先走空只在写死的用例里走（`admission_raises_the_floor_before_refusing.rs` 那几条与 E5a 这种），随机历史里走空只在挂载续做那一处被比。E4 的 512 段里这一形出现了 4 次，4 次都被跳过（`outputs/e4-240-probe.log` 一次、`outputs/e4-384-b2.log` 三次，`not_applicable` 栏里「现行版本有删除中的文件」）。
四句：不分辨臂；执行器看得到 `deleted_inodes` 不空，跳过是因为 A2 那个拒绝成员交不出走空的几版；字面打中的是正文 K6「还有没有一种版本被比较链跳过」；改法：A2 的改法让那一种拒绝也带发成的几版，执行器就能照 `settle_user_change_comparing_every_version_published` 的做法逐版比（推的、零轮）。

### A5　Q-T 改钉之后，取号之前预演取不到落点那一拒没有用例钉了（打中，钉值丢失）

```
$ git grep -c 'PlacementRefusedBeforeAcquisitionMountAdmissionUndecided' HEAD -- crates/singlefs-harness/tests crates/singlefs-checker-tier/tests
HEAD:crates/singlefs-harness/tests/random_histories.rs:3
$ grep -rlc 'PlacementRefusedBeforeAcquisitionMountAdmissionUndecided' <冻结副本>/crates/singlefs-harness/tests <冻结副本>/crates/singlefs-checker-tier/tests | wc -l
0
```

HEAD 里钉它的是 `writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged`（逐字段钉成员、录制流一步都不许有）；这一批照 Q-T「乙 照现值改钉并改名」把它改成了 `eighteen_empty_overwrites_on_240_slot_devices_with_the_admission_skipped_leave_room_for_a_writable_mount_with_one_warm_up`，钉的是反面（挂载做成）。冻结副本里引这个成员的只剩 `src/`（mount.rs、history.rs、model_comparison.rs、mount_transparency.rs）与一个 checker 档装置，没有用例。注释说落点墙那一格「只靠写死的那 5 条用例罩着」，那 5 条罩的是发布路径与抬 F 的落点，不是可写挂载取号之前那一判。
副本上这一拒照样走得到，「一个写都不发、盘面不变」也仍成立（E8c，`outputs/e8c.log`，原样，我的打印截了前 160 字）：

```
test e8c_the_refusal_before_acquisition_writes_nothing ... E8c second mount: Some(PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { instance_to_acquire: InstanceGeneration(3), publish_index: 0, warm_up_publishes_planned: 1, unit; recorded_operations=0 images_identical=true
```

四句：不分辨臂；不是实现缺陷，是判别力丢了——今天把那一拒改成先写后拒，没有用例会红；字面打中的是正文 K6「比重表换表会不会让别的组的旧钉值悄悄失效」；改法：用 A1 那份窄盘（续做交回错之后再挂）补一条钉成员与「录制流 0 步、镜像逐字节相同」的用例（E8c 就是它的草稿，推的、零轮）。

### A7　模型只罩一个数据单元的文件（覆盖缺口，不是缺陷）

「模型的文件只有一个数据单元，删除中的那一版的下一步就是最后一批，之后这一版有树而文件集合为空。」（`crates/singlefs-harness/src/model.rs:1452`）。随机历史里的覆盖写都在一个单元以内，所以「只改尾叶」的删除批（L1 那条新的 extent 分支，正文写它被攻过零轮）在随机历史里一次都不出现，模型与实现在多批删除上分不分叉没有随机取样。E1 的 58 种配置（最多 600 单元、批 1/3/144）、E5 定向历史上 checker 全绿，只说明盘面合法，不是模型对拍。

### N4　偏删除的随机历史（没打中）

E4 自己起的比重「偏删除」（带文件那张表 unlink 25、回退 15、抬 F 10、可写挂载 8、崩溃恢复抛弃根 6；删完之后那张表回退 30、建 inode 20、可写挂载 15、崩溃恢复抛弃根 10），准入判着、每一步之后跑池级 checker：

| 盘 | 种子 | 每段步数 | 段数 | 跑完 | 新发现 | unlink 做成 |
|---|---|---|---|---|---|---|
| 4 GiB（BROAD 比重） | [7300000000, 7300000008) | 40 | 8 | 8 | 0 | 7 |
| 240 槽 | [7300000000, 7300000016) | 40 | 16 | 16 | 0 | 14 |
| 240 槽 | [7300002000, 7300002048) | 40 | 48 | 48 | 0 | 43 |
| 384 槽 | [7300000016, 7300000056) | 40 | 40 | 40 | 0 | 33 |
| 384 槽 | [7300001000, 7300001400) | 60 | 400 | 400 | 0 | 342 |

合计 512 段、439 次 unlink、0 个新发现。偏删除比重在 4 GiB 上没跑（每 8 段 40 步要 186 秒，排在交回之后）。比重表换表那一问：只有带「unlink 之后那张表」的两组（BROAD、REUSE）换了生成估计；TOWARD_THE_UNIT_AREA_WALL 带了 unlink 却没给那张表，unlink 之后照「开着、没有文件」抽，抽到的第一个文件被拒（`FirstFileVersionOnAVersionThatAlreadyHasAFile`），那几步白跑，它的钉值已按 Q-T 改过；没加 unlink 的两组生成逐项不变（按代码读）。

## 四、自己提的改法（都只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 标 | 依据 |
|---|---|---|---|
| 甲：续做交回错时按 txg 次序交出写行、暖机、挂载推的抬 F、续做每一步与续做推的每一串的写账，连同停因 | A1；连带 A4 | 推的（没实现、没跑） | A1 第 2 句：数据在函数手里 |
| 乙：`NoFileVersionToChange`（或新成员）带上会话走空交回的 `PublishedInThisChange` | A2；连带 A3 | 推的 | A2 第 2 句 |
| 丙：补一条钉「可写挂载取号之前预演取不到落点、录制流 0 步、镜像逐字节相同」的用例，盘面用 A1 那份窄盘续做交回错之后 | A5 | 量过（副本上的原样输出见 A5 的 E8c 那一行） | E8c |
| 丁：把「下一次挂载接着走」收成「准入开着时」 | A6 | 推的 | E8、E8b |
| 对照臂：续做推满仍不够时挂载照样做成可写 | A6 | 量过（E8b 原样输出见 A6）：对这一格不起作用，卸载那一串在预演里就落不下 | E8b |

## 五、没打中的形状

- 续做推的抬 F 那一段的每个崩溃状态，恢复与重挂之后 checker 判红（N1：69671 态 0 红）。
- 删除类发布实写超过 W（N2：58 种配置，W' ≤ 20）。
- 准入开着时删除类发布被落得下那一判拒到推满仍不够（N3：126 + 22 段与 512 段随机历史，0 次）。
- 会话把带文件的一版走空之后读者答错、checker 判红（E5：回退到意图那一版之后四种用户改动、删非第一个 inode 走到删空、第一个文件活着时先建 inode 再删，全绿；`outputs/e5.log`）。
- 模型与实现在偏删除随机历史上分叉（N4：512 段 0 个新发现）。
- 推抬 F 的预算在续做那一趟里数错（按代码读一致；取样里没有被执行到，见 K2 其余几问）。

## 六、这条腿自己的限度

- 取样只在两块盘、内存稀疏盘、三档小盘与 4 GiB 上；分配记录树每层被删除批改很多片叶那一格我没造出来（N2 的限度）。
- 准入开着的历史里我一次都没走到删除被空间拒，所以「准入开着时续做推满仍不够」那一支（L4 的挂载那一半）只有准入关着造的盘面上的样本（A1、A6、N1），推不到产品路径。
- 崩溃状态只枚举了续做推的那一串（一段、一种盘面）；删除批本身单段就超过 10⁶，没跑；会话推的抬 F 没另跑。
- A4 只按代码推，checker 档（崩溃注入）不归我跑。
- E1 的产物是 debug 跑的：那一条 `cargo test` 超过前台 240 秒被工具转到了后台（不是我有意放后台），我没停它、让它跑完了（341 秒），之后的原型一律 release、前台 235 秒以内分批跑。
- 跑法上同一会话里另有一条腿在跑，内存包装要我设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 才放行；E6、E8b 之后几条另设了 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`（峰值表里记的是只算状态数就退出那一次的峰值）。退出码 251 / 252 那几次命令一行都没跑，我照提示补了理由重跑。
- 「没打中」都是一次抽样，照规则还要再抽一次才记。

## 七、核对用的命令与输出

被引的 8 份 crates 文件在主仓与冻结副本逐字节相同（sha256 前 16 位，左主仓、右冻结副本）：

```
crates/singlefs-core/src/mount.rs 746dcb5088aeaf55 746dcb5088aeaf55
crates/singlefs-core/src/mounted_session.rs ad06913ee2853b57 ad06913ee2853b57
crates/singlefs-checker-tier/src/crash_injection.rs 01fa5151e7c452c6 01fa5151e7c452c6
crates/singlefs-core/src/admission.rs e0390d392e4b956c e0390d392e4b956c
crates/singlefs-core/src/transaction.rs 2400421618917917 2400421618917917
crates/singlefs-harness/src/history.rs fa0858cc850ab479 fa0858cc850ab479
crates/singlefs-harness/src/model.rs d93e8ab98b753cda d93e8ab98b753cda
crates/singlefs-harness/tests/random_histories.rs 12b769f67e893175 12b769f67e893175
```

## 八、没做什么

- 正推、辩方那几格不判；不替主 agent 采纳；副本上的数不算入库装置上的数。
- K1、K4、K5 没专门攻（派发提示把重点放在 K2、K3、K6）；E5 顺带碰到 K4 的「回退到删除中的一版」与「删非第一个 inode 走到删空」，都绿。
- 没跑 checker 档、层 0 用例、崩溃注入、变异表；没跑仓里的随机历史慢档（E4 是我自己的种子区间与比重）。
- 偏删除比重在 4 GiB 上、与 TOWARD_THE_UNIT_AREA_WALL 那一组带 unlink 的取样，没跑。
- 交回前删掉了草稿副本 `/tmp/claude-1000/m3-unlink-code-r1-opus/tree`（含编译目录）与 E6 的落盘中间文件 `/tmp/claude-1000/m3-unlink-code-r1-opus/e6-dump-c1`；复跑要的全部在模型目录里。草稿目录里留着原始日志（与模型目录 `outputs/` 同一份）。
