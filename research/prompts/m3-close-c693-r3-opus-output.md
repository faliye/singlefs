# m3-close-c693-r3 攻方腿（Opus）报告（2026-10-20）

立场：攻方，最终轮。只攻正文第三节 T2、T3、T4；不判 T1；不替主 agent 采纳。前两轮已攻过的角度（读法 B1/B2、U5、按节点数、`w_gens`、宽读 E71 过界、树高只在一点量过、前端逼出墓碑、未知位与预留字节同一条 `None`、对象级 EIO 出 C755 并集）不再攻。没读禁读清单里的文件。

## 复跑命令与文件 sha256

```
# 仓根；T2 一段在 --work 下现拷 crates、打探针与草稿白名单、编 harness 测试目标（4G 包装，capped.sh 31，cargo -j 8）
bash research/prompts/m3-close-c693-r3-opus-model/tools/rerun.sh --repo . --frozen research/prompts/m3-close-c693-r3-snapshot --work <新的草稿目录>
# 末尾三行 name=rerun_compare … identical=1 即复出 out/ 下 t3_step_rows.out、r3probe-today.out、r3probe-whitelist.out
```

rerun.sh 里每条 cargo 自带范围声明并经 `run-with-memory-cap.sh 4G` 起，所以整条不在外面再套包装。这一份 rerun.sh 没有整条跑过（见「这条腿自己的限度」）；out/ 下的文件是草稿目录里逐步跑出来的。

模型目录 `research/prompts/m3-close-c693-r3-opus-model/`：

```
bf726947c37936f65f5931ee8b87c268f21f2191618e8a49afa6795939e2a604  ./apply_probe.py
9e8c4679ebb0c7e8951149fa663ba24f9fefe5aa8390051b666b0566370ec3d7  ./c693_r3_whitelist_probes.rs
5c6b8555b305b7acb3aaa0919b1c07278b2c54de54e36cacedd7617ce41c5a20  ./out/probe-today.log
5d6745f95697d813bacea2f2e47166688f1e67999d0a81aba832c4ea5de2e594  ./out/probe-today-summary.out
02c9f52fa35df0c4d57a6d4db91b677baf82af85a11858c2a85aa848e0935629  ./out/r3probe-today.out
728fe9aa2fb947f5b43c9f402783e499d015ad45275b65f9676502d169b1391b  ./out/r3probe-whitelist.out
0362db2ab93f767fde47946d74535b8ff559728cf14fb623b95d5d9732e4d50d  ./out/t3_step_rows.out
c715d034876144d0b0f8b731ebd7113dd9b321b9a0462af52f425c28c077aff7  ./out/whitelist-regress.out
8e999e69d02895dc02ab5d4298c3e9c1498105caae4deb8bf1e1a6f403c170be  ./summarize_probe.py
d9c29a5ad22800804a782847ca972dc98e3a94362e6bdc6669b2823032ed57b5  ./t3_step_rows.py
231f09e13aface8d3949b9489f0a631e7b3e98a871c4babd958ae426c8d6293e  ./tools/rerun.sh
```

- `apply_probe.py`：在仓副本的五个 F2 退回点（恢复、只读挂载、可写挂载三处，加 harness 挂载透明判官 `read_recovery_fields`）加只记不改的探针，每次走到退回分支记一行：哪一处、失败值、草稿白名单在这一处退不退、F2 判法交回什么、今天实际退到哪。今天的行为一字不变。
- `c693_r3_whitelist_probes.rs`：harness 测试目标（沿用 `common_unwitnessed_publish` 的录法与第二轮那三种盘面），12 格 = 4 种改法 × 3 种盘面，每格打一行 `R3PROBE`（三处结局加挂载透明判官）。
- `summarize_probe.py`：把探针日志按「处 × 失败种类 × 草稿退不退 × 今天退没退」归并。
- `t3_step_rows.py`：T3 的纯算术，读入库产物 `research/results/e97-entry-encoding-single-generation-2026-10-20.out`；弄坏开关 `T3_BREAK=fanout_from_crates` 退 3（跑过）。
- 草稿白名单就是第二轮正推的 `research/prompts/m3-close-c693-r2-sonnet-model/v2/apply_variant.py --variant whitelist`，原样调，没改；打补丁之前三份源码的 sha256 与它钉的相同（它没打 `pristine_sha256_differs`）。

## 各格判定一览

「量过」= 贴得出副本上的原样输出；「推的」= 读码或读条款推的。副本上的数都不算入库装置上的数。

| 格 | 攻的那一步 | 判定 | 量过 / 推的 |
|---|---|---|---|
| T2-甲 | 草稿白名单三处对同一份字节分法不同 | **打中**：树表单元的父指针校验和对、节点自述的载荷校验和不对（内容拒收）时，草稿补丁让恢复拒、只读挂载退到 R 之前一版、可写挂载拒，三处落三个结局（今天三处都退到 R 之前一版） | 量过（副本 harness，没见证的两种盘面） |
| T2-乙 | 草稿只改了五个 F2 退回点里的三个 | **打中**：harness 挂载透明判官没改，草稿白名单下在没见证的内容拒收盘面上断言失败（panic）；GPU 判器 `crash_judge_recovery.wgsl` 同样按「走读失败」就退 | 量过（harness 判官，6 格 panic）+ 推的（WGSL，读码） |
| T2-丙 | 零轮组合规则「任何一处看到内容被拒，三处都不退」 | **打中**：三处走读次序不同、各自停在第一个失败上，同一份盘上一处先看到「字节丢了」、另一处先看到「内容被拒」；草稿白名单下恢复拒、只读挂载退。组合规则要的「任何一处看到」在只读挂载那一刻观测不到 | 量过（混合盘面，今天三处都退 / 草稿下分家） |
| T2-丁 | 今天靠「内容拒收也退」才挂上的合法状态 | **没打中**（在 harness 档 48 个测试目标上）：探针记到的 266 次退回分支里，今天真的退了的全是 `UnitUnreadable` / `MappingStillUnreadable`；非白名单失败（`InvariantViolated`、`FileVersionWithoutAnyJournalRecord`、`deleted_inodes` 那一条 `UnitMalformed`）的每一次 F2 判法都交回 `None`。崩溃点重放的状态没跑（交主 agent，命令在后面） | 量过（副本，探针）；崩溃状态没量 |
| T2-戊 | 白名单写成对可写挂载那一处的改动 | **打中**（判别子观测不到）：可写挂载建上一版时实例表读不出与解不开并成一个 `InstanceTableMalformed`，那一处分不出「字节丢了」与「内容被拒」 | 推的（读码） |
| T3-甲 | 改后条文 ①⑤「四个网格点上每点彼此相同」 | **打中**（漏变量的全称）：加上台阶行条数 s，`64 × 8` 窄读在 s = 371..915 时四档 key 宽的树高不同 | 量过（入库产物上的算术） |
| T3-乙 | 改后条文 ④「最多多写同比例的叶」 | **打中**：两块盘、台阶行 463..476 级时，22 字节 key 每次发布写 3 个记账节点、21 字节写 1 个，不是「同比例」（2.8%） | 量过（算术，扇出取正推的数） |
| T3-丙 | 改后条文 ③ 的百分数 | 小错：`8192 × 64` 22 字节臂写 376%，照同一条文别的数用的取整法（截断）是 375%；「旧口径同一格」只对 `8192 × 64` 成立 | 量过（产物行） |
| T4-甲 | 两种检查形态 | 不打中两者之一：两种都是排空那一刻的运行时断言，罩住的 key 集合相同（受这次前端条目影响的 key）；⑤ 是盘上判据、只罩水位那一个统计量，先于前端就能判。⑤ 的「按 (统计量, 树) 分」要等 checker 分树，而正推把 ⑤ 放在里程碑四 | 推的 |
| T4-乙 | 删头 H-1 与 D18 已定项 24 第 5 条 | **打中**（比第二轮更早可达的一段历史）：单头时做的管理员回退留下的是不带树 ID 的 24 字节谱系记录，之后建克隆、销毁原来的头，H-1 让原头的台阶行离开而那条记录只增不删，按头判时归不到任何一个活头。冲突不只是 H-1 的：克隆继承谱系记录时 H-3（永留）照样撞 | 推的 |
| T4-丙 | 两个落点的前提 | **打中**（前提漏列）：删头台阶行落在里程碑十四二·2，而它要的「按头的谱系记录字节形态」（D18:582 自认没定）在里程碑十四里一处都没写（`grep -c 谱系` 为 0） | 量过（grep） |

## T2　改法甲（F2 白名单）

被攻的对象：草稿补丁的三处条件，原文是 「whitelist    F2 只为「字节丢了」退：UnitUnreadable、MappingMiss、MappingStillUnreadable（只读挂载的 UnitMalformed 也算，它含载荷校验和不对）」(`research/prompts/m3-close-c693-r2-sonnet-model/v2/apply_variant.py:7`)；第二轮正推自己写了没量的反方向风险 「**没量到的是反方向的风险**」(`research/prompts/m3-close-c693-r2-sonnet-output.md:207`)。

装置：草稿目录里的仓副本（主工作区 `crates/` 现拷，`recovery.rs`、`mounted_read.rs`、`mount.rs` 三份 sha256 与草稿钉的相同）。harness 测试目标经 `run-with-memory-cap.sh 4G`、`capped.sh 31`、`cargo -j 8`（`-j 31` 与 `-j 16` 编 harness 时撞了 4G 上限，包装退 250，那两次没有输出可用；没加大上限，改成少几个并行编译）。

### T2-甲　同一份字节，草稿三处分法不同

盘面：R 的树表单元里 inode 条目一个字节改掉，节点不重建（节点自述的载荷校验和不对），根槽与 R 那条记录里点名它的位置条目校验和照改后的字节重算。单元两条位置都读得到与父指针校验和相符的那份，所以这是「字节读到了、读者不认」，按白名单的本意该不退。三处怎么报：恢复报 「/// 单元读到了、解不出来（哪一个、哪一条）。」(`crates/singlefs-core/src/recovery.rs:522`) 那一种；只读挂载报 「/// 读回来的单元解不开（头坏了、载荷校验和不对、补齐非零）。」(`crates/singlefs-core/src/mounted_read.rs:189`) 那一种。草稿把后者算进「字节丢了」，前者不算。

原样输出（`out/r3probe-today.out` 与 `out/r3probe-whitelist.out`，这一改法的三行）：

```
# 今天
change=NodePayloadChecksumWrongButLocationChecksumRecomputed face=CrashBetweenTheRootOfRAndItsRotation recovery=file_read:version_before_r read_only=ok:version_before_r writable=ok:version_before_r mount_transparency_judge=Some("version_before_r")
change=NodePayloadChecksumWrongButLocationChecksumRecomputed face=RConfirmedButItsRotationSlotsReadBackZero recovery=file_read:version_before_r read_only=ok:version_before_r writable=ok:version_before_r mount_transparency_judge=Some("version_before_r")
change=NodePayloadChecksumWrongButLocationChecksumRecomputed face=SystemConfigurationWitnessedR recovery=refused:UnitMalformed { what: "树表单元" } read_only=refused:Open(UnitMalformed { what: "树表单元", error: PayloadChecksumMismatch }) writable=refused:Recovery(UnitMalformed { what: "树表单元" }) mount_transparency_judge=Some("r")
# 草稿白名单
change=NodePayloadChecksumWrongButLocationChecksumRecomputed face=CrashBetweenTheRootOfRAndItsRotation recovery=refused:UnitMalformed { what: "树表单元" } read_only=ok:version_before_r writable=refused:Recovery(UnitMalformed { what: "树表单元" }) mount_transparency_judge=panicked:assertion `left == right` failed: （判官的断言消息，截断）
change=NodePayloadChecksumWrongButLocationChecksumRecomputed face=RConfirmedButItsRotationSlotsReadBackZero recovery=refused:UnitMalformed { what: "树表单元" } read_only=ok:version_before_r writable=refused:Recovery(UnitMalformed { what: "树表单元" }) mount_transparency_judge=panicked:assertion `left == right` failed: （判官的断言消息，截断）
change=NodePayloadChecksumWrongButLocationChecksumRecomputed face=SystemConfigurationWitnessedR recovery=refused:UnitMalformed { what: "树表单元" } read_only=refused:Open(UnitMalformed { what: "树表单元", error: PayloadChecksumMismatch }) writable=refused:Recovery(UnitMalformed { what: "树表单元" }) mount_transparency_judge=Some("r")
```

今天三处在没见证的两种盘面上都静默落到 R 之前一版（与第二轮量过的未知位一形同）；草稿白名单下恢复拒、只读挂载落到 R 之前一版、可写挂载拒。C755 并集判退要的是三处落同一版，草稿让它们分家。

打中之后四句：① 分辨：分辨「草稿三处的条件」与「三处同一分法」两种写法（在这两格上前者分家、后者不分）。② 系统看不看得到：看得到，三处都拿到了失败值，只是归类不同。③ 满足的分句：草稿第 7 行括注「只读挂载的 UnitMalformed 也算，它含载荷校验和不对」；载荷校验和不对而父指针校验和对，正是「字节读到了、读者不认」，括注把它归错了一边。④ 改法：「只读挂载的 `UnitMalformed` 归内容拒收」修这两格（推的：只读挂载那一支的父指针校验和不对已经走 `Walk(UnitUnreadable)`，量过的 `NodePayloadChecksumWrong…` 三行里见证面报 `Open(UnitMalformed { … PayloadChecksumMismatch })`）。

### T2-乙　五个 F2 退回点，草稿改了三个

F2 判法在仓里有五个调用点：恢复、只读挂载、可写挂载，加 harness 的挂载透明判官 「walk_to_file(reader, root, &mut mapping_fallbacks).is_ok()」(`crates/singlefs-harness/src/mount_transparency.rs:225`) 与 GPU 判器 「let fallback = newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(cfg_choice.cfg, chosen.root, effective);」(`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:58`)。后两处只问「走读失败没有」，草稿没碰。

量过：草稿白名单下 `read_recovery_fields(…, TheRecoveredVersion)` 在没见证的内容拒收盘面上断言失败——上面三组改法里没见证的 6 格都是 `mount_transparency_judge=panicked:assertion …`（`out/r3probe-whitelist.out`），今天同 6 格是 `Some("version_before_r")`（`out/r3probe-today.out`）。WGSL 那一处是读码推的（没跑 GPU 判器）。

打中之后四句：① 分辨：分辨「只改三处」与「五处一起改」。② 系统看不看得到：判官拿得到失败值（`walk_to_file` 交回 `Err(failure)`，今天被 `.is_ok()` 丢掉）；WGSL 走读内核交回 `walked.failure`（「let walked = walk_to_file(root_to_walk, cfg_choice.cfg, expected_version_index(root_to_walk.instance, root_to_walk.txg));」(`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:46`)），分不分得出「字节丢了」要看内核的失败码表，没核。③ 满足的分句：正推第二轮 V2.3 已写「harness 的 `mount_transparency.rs` 那一处同样要把失败值留下再交，否则判官和实现分叉」，草稿量具没做，30 条已有 F2 用例没走到这一形，所以是绿的。④ 改法：甲的实现批必须把判官与 WGSL 一起改，验收加「判官在内容拒收盘面上不 panic」；只改三处在这 6 格上不起作用（量过）。崩溃点重放的判官就是这两处之一，草稿那三处一改、判官不改，重放里一旦出现内容拒收的没见证状态就是判官红，不是实现红。

### T2-丙　组合规则要的「任何一处看到」观测不到

正推第二轮的零轮组合规则 「组合规则（零轮，推的）：并集判退里，**任何一处看到的失败是读者拒收的内容，三处都不退，各自报自己看到的失败**」(`research/prompts/m3-close-c693-r2-sonnet-output.md:205`)。

三处走读的次序不同，各自停在第一个失败上：恢复先读实例表 「read_unit_via_locations(reader, &root.instance_table.locations, data_unit_bytes)?;」(`crates/singlefs-core/src/recovery.rs:6266`)、再读树表；只读挂载在映射根非空时先读中央映射树 「    let mapping_tree = read_code_two_tree(」(`crates/singlefs-core/src/mounted_read.rs:407`)、后读树表 「    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;」(`crates/singlefs-core/src/mounted_read.rs:434`)。

历史：R 的树表 inode 条目带一个不认识的 flags 位（将来的写者合法写下的），同时 R 那次发布新写的中央映射树根两份读回全零（F2 本来要接的那一形）。原样输出（`UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero` 三行）：

```
# 今天
change=UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero face=CrashBetweenTheRootOfRAndItsRotation recovery=file_read:version_before_r read_only=ok:version_before_r writable=ok:version_before_r mount_transparency_judge=Some("version_before_r")
change=UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero face=RConfirmedButItsRotationSlotsReadBackZero recovery=file_read:version_before_r read_only=ok:version_before_r writable=ok:version_before_r mount_transparency_judge=Some("version_before_r")
change=UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero face=SystemConfigurationWitnessedR recovery=refused:UnitMalformed { what: "树表条目" } read_only=refused:Open(Walk(UnitUnreadable { slot: SlotNumber(1541) })) writable=refused:Recovery(UnitMalformed { what: "树表条目" }) mount_transparency_judge=Some("r")
# 草稿白名单
change=UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero face=CrashBetweenTheRootOfRAndItsRotation recovery=refused:UnitMalformed { what: "树表条目" } read_only=ok:version_before_r writable=refused:Recovery(UnitMalformed { what: "树表条目" }) mount_transparency_judge=panicked:assertion `left == right` failed: （判官的断言消息，截断）
change=UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero face=RConfirmedButItsRotationSlotsReadBackZero recovery=refused:UnitMalformed { what: "树表条目" } read_only=ok:version_before_r writable=refused:Recovery(UnitMalformed { what: "树表条目" }) mount_transparency_judge=panicked:assertion `left == right` failed: （判官的断言消息，截断）
change=UnknownFlagsBitRebuiltWithTheMappingRootOfRReadingBackZero face=SystemConfigurationWitnessedR recovery=refused:UnitMalformed { what: "树表条目" } read_only=refused:Open(Walk(UnitUnreadable { slot: SlotNumber(1541) })) writable=refused:Recovery(UnitMalformed { what: "树表条目" }) mount_transparency_judge=Some("r")
```

草稿下只读挂载看到的是 `Walk(UnitUnreadable)`、退了；恢复与可写挂载看到的是 `UnitMalformed { what: "树表条目" }`、拒了。组合规则要只读挂载知道「别处看到了内容被拒」，可只读挂载在映射根那一步就停了，没读到树表；单独起一次只读挂载时也没有「别处」。

打中之后四句：① 分辨：分辨「各处按自己第一个失败分类」与「读完并集、按全部失败分类」。② 系统看不看得到：**看不到**——判别子观测不到那一形：只读挂载做决定那一刻手里只有映射根读不出这一个失败。③ 满足的分句：组合规则的「任何一处看到」。④ 改法：要么规则改成「每一处都读完 C755 的并集、收全部失败再分类」（读法变了：今天的走读第一个失败就停），要么写明「各处按自己第一个失败分类，混合盘面三处可以分家」并认下。两样都是我提的、被攻过零轮。改判据之后是新一轮的事。

### T2-丁　今天有没有合法状态靠「内容拒收也退」才挂上（没打中，范围写在下面）

探针（`apply_probe.py`，只记不改）打在今天的代码上，跑了 harness 档 48 个测试目标（`targets` 取「调恢复或两种挂载、又带读回全零 / 没见证 / 退回 / 见证字样」的 35 个，加内容拒收一族、崩溃镜像再挂一族与本腿的 `c693_r3_whitelist_probes`；`random_histories` 与 `checker_known_bad_images` 两个超过单条前台时限没跑）。归并（`out/probe-today-summary.out` 头两行与「今天退了而草稿不退」的全部行）：

```
name=probe_lines parsed=266 unparsed=0
name=probe_cell site=judge_read_recovery_fields failure_kind=UnitMalformed{树表单元} draft_whitelist_retreats=false today=see_recovery events=3
name=today_retreats_but_draft_whitelist_does_not site=read_only failure_kind=TreeTableEntryMalformed test=unknown_flags_confirmed_zero
name=today_retreats_but_draft_whitelist_does_not site=read_only failure_kind=TreeTableEntryMalformed test=unknown_flags_crash
name=today_retreats_but_draft_whitelist_does_not site=recovery failure_kind=UnitMalformed{树表单元} test=payload_checksum_confirmed_zero
name=today_retreats_but_draft_whitelist_does_not site=recovery failure_kind=UnitMalformed{树表单元} test=payload_checksum_crash
name=today_retreats_but_draft_whitelist_does_not site=recovery failure_kind=UnitMalformed{树表条目} test=unknown_flags_confirmed_zero
name=today_retreats_but_draft_whitelist_does_not site=recovery failure_kind=UnitMalformed{树表条目} test=unknown_flags_crash
name=today_retreats_but_draft_whitelist_does_not site=writable failure_kind=Recovery(UnitMalformed){树表单元} test=payload_checksum_confirmed_zero
name=today_retreats_but_draft_whitelist_does_not site=writable failure_kind=Recovery(UnitMalformed){树表单元} test=payload_checksum_crash
name=today_retreats_but_draft_whitelist_does_not site=writable failure_kind=Recovery(UnitMalformed){树表条目} test=unknown_flags_confirmed_zero
name=today_retreats_but_draft_whitelist_does_not site=writable failure_kind=Recovery(UnitMalformed){树表条目} test=unknown_flags_crash
```

「今天退了而草稿不退」的 10 行全在本腿造的内容拒收盘面上（`unknown_flags_*`、`payload_checksum_*`），已有用例里一行都没有。已有用例里走到退回分支、失败不在白名单里的（`grep` 归并表的 `draft_whitelist_retreats=false today=none` 行）：恢复 `InvariantViolated` 4 次（I-1.1，`the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads` 的单盘幸存镜像）、可写挂载 `InvariantViolated` 9 次（I-2.5，位置条目不升序一族）、`FileVersionWithoutAnyJournalRecord` 1 次、`deleted_inodes` 那一条 `UnitMalformed` 1 次，F2 判法每一次都交回 `None`（系统配置见证了，或判不出），今天也没退，草稿不改它们的结局。

草稿白名单在已有用例上的回归（`out/whitelist-regress.out`，20 个目标：探针记到退回分支的 17 个、正推那 5 个 F2 目标里另外两个、本腿的目标）：20 个目标 163 条用例 0 条红，今天同一批也是 0 条红。

结论的范围：harness 档的这一批目标，不是崩溃点重放的枚举域。单盘幸存镜像那 4 次说明「崩溃之后的盘、走读失败在白名单之外」的状态是存在的（I-1.1），只是那里 F2 判法交回 `None`；同一形若落在没见证的盘面上，今天会退、草稿会拒。崩溃点重放里有没有这样的状态，本腿没量（照派发提示不跑），命令写在「要主 agent 跑的命令」。

什么现象会推翻「没打中」：崩溃点重放带探针跑出一行 `C693F2 … draft_whitelist_retreats=false … retreated=(…)`，且那个状态的判官判它合法。

### T2-戊　可写挂载那一处分不出实例表是读不出还是解不开（推的）

可写挂载建上一版时，实例表读不出与解不开并成一个失败 「.map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?;」(`crates/singlefs-core/src/mount.rs:1282`)。草稿在可写挂载那一处只认包在 `MountError::Recovery` 里的白名单三种，恢复的走读只读实例表的第 0 片（T2-丙 引的 6266 行），可写挂载沿链读全（`mount.rs` 里 `rebuild_previous_version` 的文档注释写了只解第 0 片的坑）。所以 R 那次发布重写过实例表（写行那次发布）、表多于一片、第 0 片之外的一片两份读回全零（字节丢了，F2 本来要接的那一形）时：今天可写挂载拿到 `InstanceTableMalformed`、退到上一版，草稿拒；恢复不读那一片、落在 R（这一处分家今天就有，是 C755 并集要收的）。草稿下可写挂载那一处把字节丢了当成内容拒收。

打中之后四句：① 分辨：分辨「草稿可写挂载那一处的条件」与「先把 `InstanceTableMalformed` 拆成读不出 / 解不开再分类」。② 系统看不看得到：**看不到**——那一处拿到的失败值里已经没有这一位（判别子观测不到）。③ 满足的分句：白名单「字节丢了 = 单元两条位置都读不到校验和相符的那份」。④ 改法：先拆错误成员再分类（推的，零轮）。没造这一形：要让 R 是一次可写挂载的写行发布，本腿的录法（`record_one_publish_after_overwrites`）录的是覆盖写，实例表与上一版共用，造不出来。

### T2 几个改法各修哪一格

全部是本腿提的，只在本腿的副本上量过、被攻过零轮。

| 改法 | 修哪一格 | 在打中的格上 | 标 |
|---|---|---|---|
| 只读挂载的 `OpenPoolForReadFailure::UnitMalformed` 归内容拒收（三处同一分法：父指针校验和对就是内容） | T2-甲 | 没见证两格三处同拒 | 推的 |
| 判官 `read_recovery_fields` 与 WGSL 判器同批分失败种类 | T2-乙 | 6 格判官不再 panic | 推的（panic 量过，改后没跑） |
| 组合规则改成「读完 C755 并集、收全部失败再分类」，或写明「各处按第一个失败分类，混合盘面可分家」认下 | T2-丙 | 混合盘面两格 | 推的 |
| `InstanceTableMalformed` 拆成读不出 / 解不开 | T2-戊 | 实例表读回全零那一形 | 推的 |
| 第二轮正推的草稿原样（对照） | T2-甲、乙、丙 | 甲、丙两格三处分家，乙 6 格 panic | 量过 |

## T3　D5 五句的改后条文

被攻的对象：第二轮正推 V3.2 的五条（`research/prompts/m3-close-c693-r2-sonnet-output.md` 第 239–253 行），加第二轮判决要补的「台阶行条数也决定树高」。第二轮攻方 V3 已经打中「树高句漏了台阶行条数」；这一格攻的是：五条里哪几句在补上这个变量之后数对不上、哪几句是在 s（台阶行条数）的一个取样点上说的全称。②（E71 那一条）逐数对过正推 `out/v3_checks.out` 与产物，没找到对不上的数，不再列。

装置：`t3_step_rows.py`，叶扇出取入库产物 `cell_side_by_side` 行（1087 / 740 / 603 / 542），窄读行数取同一行的 `groups`；crates 几何的 21 / 22 字节叶容量取正推 `out/v3_checks.out` 那一行（491 / 477），内部 150。只算叶层 1 → 2 那一道门槛（产物里没有 E97 几何的内部扇出，2 → 3 那一道没算）。原样输出 `out/t3_step_rows.out`：

```
name=t3_e97_leaf_fanout widths=[7, 14, 19, 22] fanouts=[1087, 740, 603, 542]
name=t3_e97_narrow_heights_differ_with_step_rows trees=64 devs=8 rows_without_steps=172 step_rows_where_four_widths_differ=371..915 count=545
name=t3_e97_narrow_heights_differ_with_step_rows trees=1024 devs=64 rows_without_steps=2372 step_rows_where_four_widths_differ=none count=0
name=t3_e97_narrow_heights_differ_with_step_rows trees=8192 devs=8 rows_without_steps=16428 step_rows_where_four_widths_differ=none count=0
name=t3_e97_narrow_heights_differ_with_step_rows trees=8192 devs=64 rows_without_steps=16708 step_rows_where_four_widths_differ=none count=0
name=t3_crates_one_more_key_byte devices=2 step_rows=463 rows=478 nodes_key21=1 nodes_key22=3 extra_nodes=2
name=t3_crates_one_more_key_byte devices=2 step_rows=476 rows=491 nodes_key21=1 nodes_key22=3 extra_nodes=2
name=t3_crates_one_more_key_byte_worst ratio=3.000 step_rows=463 rows=478 nodes_key21=1 nodes_key22=3 stated_ratio_bound=1.028
name=t3_done
```

### T3-甲　①⑤「四个网格点上每点彼此相同」是 s = 0 那一点上的全称

① 原文 「上每点彼此相同（窄读 1、2、2、2；宽读 2、3、3、3）」(`research/prompts/m3-close-c693-r2-sonnet-output.md:241`)，⑤ 原文 「的树高在登记的四个网格点上彼此相同、格点之间不处处相同」(`research/prompts/m3-close-c693-r2-sonnet-output.md:253`)。E97 的行集里没有台阶行（窄读 `64 × 8` 是 172 行，产物 `cell_side_by_side` 行）。台阶行随建 inode 的发布一直涨，D5 自己写了门槛 「代价：一行 34 字节；每次发布多写的单元，台阶行到第 462 行为止是 0，第 463 行起 +2，第 36 074 行起 +4」(`.claude/kb/decisions/05-快照-空间记账机制.md:55`)。同一个登记网格点 `64 × 8` 窄读，再加 s 级台阶行：s 在 371 到 915 之间（545 个取值），7 字节 key 一片叶装得下、22 字节要两层，四档树高不同。另三个网格点窄读在这一道门槛上没有这样的 s（行数已过最大的叶扇出），2 → 3 那一道没算。

打中之后四句：① 不判臂。② 不适用。③ 满足的分句：①⑤ 的「每点彼此相同」「彼此相同」，没带 s。④ 改法：第二轮判决给的「树高一句补上台阶行条数」只补在「树高」那一句上不够，「每点彼此相同」这句要么带「台阶行 0 级时」，要么删掉、只留「随 key 宽、行数（含台阶行）走」。

### T3-乙　④「最多多写同比例的叶」在一片叶装满的边上不成立

④ 原文 「在一片叶装得下的池上不多写节点，多叶的池上最多多写同比例的叶」(`research/prompts/m3-close-c693-r2-sonnet-output.md:251`)。两块盘（池级 3 + 每盘 6 = 15 行）加 s 级台阶行，s 在 463 到 476 之间：21 字节 key 一片叶（1 个节点），22 字节 key 两片叶加一个根（3 个节点），每次发布多写 2 个节点、3 倍，不是 2.8%。这正是 D5:55 自己写的「第 463 行起 +2」那一段：多出的 1 字节把这道门槛提前了 14 级。

打中之后四句：① 不判臂。② 不适用。③ 满足的分句：「最多多写同比例的叶」的「最多」。④ 改法：改成「叶数向上取整，跨过一片叶装满的那一道门槛时多一层（多 2 个节点）；两块盘上 22 字节比 21 字节早 14 级台阶跨过」，或只写 2.8% 是叶容量的比、不写写放大。

### T3-丙　③ 的两处小错

③ 原文 「`8192 × 64` 占 337%（22 字节臂 376%）过界」(`research/prompts/m3-close-c693-r2-sonnet-output.md:249`)。22 字节臂这一格 80 740 352 字节 ÷ 21 474 836 = 3.7598（正推 `out/v3_checks.out` 那一行 `k22_permille_of_budget=3759`），同一条文里 47%（10 240 000 ÷ 21 474 836 = 0.4768）用的是截断，照同一取法这里是 375%。另：「旧口径同一格 19 字节臂 11 419 648 字节、53%」接在「`8192 × 8` 与 `8192 × 64`（并列）」之后，而产物里旧口径最紧那一格只有 `8192 × 64`：「name=narrow_tightest question=Q-E97-4 arm=acct_wide_19B old_node_bytes=11419648 old_grids=8192x64 old_percent=53 old_permille=531 old_over=0 new_node_bytes=458752 new_grids=8192x8,8192x64 new_percent=2 new_permille=21 new_over=0」(`research/results/e97-entry-encoding-single-generation-2026-10-20.out:229`)。不分辨臂，措辞改。

另一处推的：③ 末句 「付的是窄读那一档的上界」(`research/prompts/m3-close-c693-r2-sonnet-output.md:249`) 没带 ② 已带的「台阶行除外」；台阶行条数不受「统计量数 × 树数 × 设备数」那个上界管，③ 说「上界」要连同 ② 的例外一起说。

### T3 几个改法各修哪一格

| 改法 | 修哪一格 | 在打中的格上 | 标 |
|---|---|---|---|
| 第二轮判决的「树高一句补台阶行条数」只补在树高句 | T3-甲 | ①⑤「彼此相同」照旧在 s = 371..915 上错 | 量过（算术） |
| ①⑤ 的「彼此相同」带「台阶行 0 级时」或删去 | T3-甲 | 不再说全称 | 推的，零轮 |
| ④ 改成「跨叶满门槛多一层」 | T3-乙 | s = 463..476 那 14 级说对 | 推的，零轮 |
| ③ 改 375%、「同一格」改成 `8192 × 64` | T3-丙 | 对上产物行 | 量过（产物行） |

## T4　墓碑缩题后的检查与删头台阶行

这一格全是读条款与读码推的，没造 checker、没造多头路径（`crates/` 今天没有克隆与销毁头）。

### T4-甲　两种检查形态比

被比的两样：正推 「2. **I-9.30 加 ⑤：同一 (统计量, 树) 下的台阶行按代段升序 H 严格递增**」(`research/prompts/m3-close-c693-r2-sonnet-output.md:294`) 加排空对拍（参考重放，结果比「受影响的每个 key」）；攻方 「会红的形是反方向的一句」(`research/prompts/m3-close-c693-r2-opus-output.md:242`)（前端里每个 key 到达序最后一条在叶上兑现）。

| 坏实现 | 排空对拍 | 前端 → 叶 | ⑤ | 今天的 I-3.13 |
|---|---|---|---|---|
| 朴素取最大吞掉墓碑（水位统计量） | 红 | 红 | 红（正推模型量过第 2 版红） | 绿（台阶行豁免） |
| 朴素取最大吞掉墓碑（别的记账统计量，非台阶行） | 红 | 红 | 不判 | 红（留下的行代段早于 txg）推的 |
| 反向索引、LRU 两棵前端树吞墓碑 | 红 | 红 | 不判 | 不判 |
| 排空多改了一个前端里没有的 key | 不判（只比受影响的 key） | 不判 | 不判 | — |
| 写者今天就多照抄一级等 H 的台阶（不经前端） | 不判（没有前端） | 不判 | 红 | 绿 |

推的结论：两种前端检查罩的 key 集合相同（都只看这次前端条目碰到的 key），拦的坏实现相同；第四行要靠已有的「叶 ⊆ 前端」那一方向（C383、D8 已定项 12 的检查）补，两者都要连同它一起写。⑤ 不需要前端、今天就能判，罩的只是水位一个统计量。误红：⑤ 在单头上推不出误红（第二轮攻方「没打中的形状」已试回退、崩溃回退）；有克隆头之后 ⑤ 必须按 (统计量, 树) 分（建克隆那次给新头的那一级与源头最后一级 H 可以相等），而 checker 今天不分树 「`crates/` 今天没有克隆路径，这一判不按树段分头」(`crates/singlefs-checker/src/walk.rs:13074`)。正推把 ⑤ 放在里程碑四八·1「**检查 ⑤（I-9.30 台阶 H 严格递增）不依赖前端**」(`research/prompts/m3-close-c693-r2-sonnet-output.md:316`)：单头期不误红，但它的「按树分」要跟里程碑十四一·1 的分树一起改，这一笔依赖两边都没写。

### T4-乙　删头 H-1 与 D18 已定项 24 第 5 条：比第二轮更早可达的一段历史

D18 已定项 24 第 5 条的谱系记录 「随现行那一版、只增不删」、记录是 T_rb 8 + T_old 8 + H_cur 8 = 24 字节、没有树 ID，且 「按头之后谱系记录的字节形态没定（推的，第一版单头）。」(`.claude/kb/decisions/18-块里携带什么信息.md:582`)。

第二轮攻方的历史要「对克隆头做管理员回退」，那一步今天没有实现。更早可达的一段：
1. 单头期（今天就能做）：管理员回退一次，追加一条 24 字节谱系记录 (T_rb, T_old, H_cur)，H_cur 是原头 A 的水位。
2. 里程碑十四一·1：从 A 建克隆 C（C 的台阶行从建克隆那次起，代段都晚于 T_rb）。
3. 里程碑十四二·2：销毁 A，按 H-1 那次发布不再照抄 A 的台阶行。
4. 谱系记录只增不删，第 1 步那条还在；I-9.30 按头判它时它不带树 ID、归不到哪个头；归给活头 C 时，C 的台阶行里代段 ≤ T_rb − 1 的是空集，最大水位不等于 H_cur ⇒ 红。合法历史上误红。

不只是 H-1 的冲突：第 2 步之后如果 C 继承 A 的谱系记录（克隆继不继承谱系没定），不删 A 也是同一个红；H-3（永留 A 的台阶行）在「按头判、记录归给 C」时同样红。第二轮攻方说 `per_tree_all`（H-3）「反而是三者里唯一不撞这一条的」(`research/prompts/m3-close-c693-r2-opus-output.md:250`)，在这段历史上不成立（推的）。

打中之后四句：① 分辨：不分辨 H-1 / H-2 / H-3——三者在「记录归给 C」时一起红，病根在共用的前提（谱系记录不带头、按头的形态没定）。② 系统看不看得到：checker 看不到——第 1 步那条记录的字节里没有头。③ 满足的分句：D18:582「随现行那一版、只增不删」与「有克隆头时这一判与台阶行都按头」。④ 改法：先定「第一版 24 字节谱系记录在有克隆头之后归哪个头」（例：归建池时那个头的树 ID，头被销毁时这几条连同移出 / 不再判），再在 H-1 / H-3 之间选；只在 H-1 上补「被删头的谱系记录去向」不够，C 继承那一支还在。零轮。

### T4-丙　两个落点的前提

- 删头台阶行 → 里程碑十四二·2「| 二·2 | 销毁头：分叉点闸、分批续做、释放边界 |」(`.claude/kb/milestone/14-fourteenth-txn.md:50`)：它要的「按头的谱系记录形态」（T4-乙 ④）在里程碑十四里一处都没写——`grep -c 谱系 .claude/kb/milestone/14-fourteenth-txn.md` 输出 `0`（本腿现跑）。一·1 压着的 C765（I-9.29 不分树）也不是谱系。前提没满足、也没登记。
- 墓碑折叠 → 里程碑八一·1：前提是前端本身，正推与第二轮攻方都写了；本腿没找到漏列的前提（没逐条核里程碑八的依赖图）。
- ⑤ → 里程碑四八·1：见 T4-甲 末句，单头期成立，按树分那一步的依赖没写。

## 没打中的形状

- **T2：已有用例里靠「内容拒收也退」才挂上的状态。** 48 个 harness 目标、266 次退回分支（`out/probe-today-summary.out`），没有；`random_histories`、`checker_known_bad_images` 没跑（超过前台单条时限）。崩溃点重放的枚举域没跑。
- **T2：草稿白名单让已有用例变红。** 20 个目标 163 条 0 条红（`out/whitelist-regress.out`）。
- **T2：见证了 R 的盘面。** 四种改法在见证面上今天与草稿三处都拒（`out/r3probe-*.out` 的 `SystemConfigurationWitnessedR` 行），不丢数据，不分家。
- **T3 ②（E71）。** 逐数对过 `out/v3_checks.out`，没找到对不上的。
- **T4：⑤ 在单头合法历史上误红。** 推不出（台阶认法「值与最后一级不同才算」、水位只增）。

## 这条腿自己的限度

- 副本上的数，不是入库装置上的数；`c693_r3_whitelist_probes.rs` 的三种盘面沿用 `common_unwitnessed_publish`，只造了 inode 条目与树表单元两样，没造别的树、别的单元。
- `tools/rerun.sh` 没整条跑过（编 harness 一次要 3 到 4 分钟、子 agent 前台单条限 4 分钟，逐步跑的）；out/ 下三份要比的文件是草稿目录里同一套步骤逐步跑出来的。`out/whitelist-regress.out` 里本腿目标那一行是加混合盘面之前的 9 格那一次；12 格那一次是 `out/r3probe-whitelist.out`（单线程跑，避免输出交错）。
- 编译与跑的并行度：`capped.sh 31` 设了上限，cargo 另给 `-j 8`（`-j 31`、`-j 16` 编 harness 撞 4G，包装退 250）。几次 44 秒限时里没跑完的目标（`exit=124`）之后单独重跑过，全过。
- T2-戊、T4 全是推的；T3 只算了叶层 1 → 2 那一道门槛。
- 一次抽样：「没打中」的几格按规则不算「没打中」，要第二次抽样才算。

## 要主 agent 跑的命令

1. 崩溃点重放里 F2 实际退回的失败种类（T2-丁 没量的那一半；轻型，归主 agent）。在一份仓副本根里：
   ```
   python3 -I research/prompts/m3-close-c693-r3-opus-model/apply_probe.py --repo <副本根>
   C693_PROBE_LOG=<日志> SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=crash-point-replay.sh --quick（加 F2 探针的副本）; 范围=快档登记的项; 为什么=C693 第三轮 T2 记崩溃状态上 F2 退回分支的失败种类，核草稿白名单的反方向风险' bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/crash/crash-point-replay.sh --quick
   python3 -I research/prompts/m3-close-c693-r3-opus-model/summarize_probe.py <日志>
   ```
   看 `name=today_retreats_but_draft_whitelist_does_not` 有没有行：有就是草稿白名单会把一个今天退回挂上的崩溃状态改成拒，推翻 T2-丁。探针只在 CPU 路径上记（GPU 判器不经它）；录入段复用库里已有结论的块不会再走恢复，要记全须让这一趟不复用（怎么关复用照 `scripts/crash/crash-point-replay.sh` 文件头，本腿没核）。
2. 复跑本腿：`bash research/prompts/m3-close-c693-r3-opus-model/tools/rerun.sh --repo . --frozen research/prompts/m3-close-c693-r3-snapshot --work <新的草稿目录>`（里面每条 cargo 自带声明与 4G 包装；编 harness 要几分钟，主 agent 用后台起）。看末尾三行 `name=rerun_compare … identical=1`。

## 没做什么

- 不判 T1 与正推、辩方那几格；不替主 agent 采纳；没碰 kb、主工作区 `crates/`、入库装置与入库产物。
- 没跑重型测试、没跑 checker 档、没跑崩溃点重放（轻型也没跑）；没调 `enumerate_crash_states`（这一轮的打中都是靶向盘面，不是枚举来的；崩溃状态那一半交主 agent）。
- 没造：可写挂载写行发布作 R 的盘面（T2-戊）、克隆与销毁头（T4）、GPU 判器的跑法（T2-乙 WGSL 那一半）。
- 草稿：仓副本 `/home/fy5090/.local/share/singlefs/work/m3-close-c693-r3-opus/repo`（连编译目录）交回前删；日志（`runs/`、`*.log`、`probe-today.log`）留在草稿目录，没入库——要用的数已进模型目录 `out/`。

## cite-check 末行

```
$ python3 scripts/agents/cite-check.py research/prompts/m3-close-c693-r3-opus-output.md --root . --background research/prompts/_m3-close-c693-r3-background.md
  ✓ 核了 25 处引文、1 处裸行号，对上 25 处，没判 1 处（其中裸行号没有引文可比 1 处；逐处列在上面）
```

没判的一处是 T2 开头引 `apply_variant.py` 第 7 行那句（引文里带「」，脚本认不出），原文那一行本腿 `grep -nF` 现取过。
