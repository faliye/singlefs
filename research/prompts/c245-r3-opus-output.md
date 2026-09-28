# C245 第三轮 云端攻方腿（Opus）报告

日期：2026-09-28。攻击面：Q2、Q3（先写乙、丙各自最强的一种，在 Q2 消费者清单每一处上造乙 / 丙判错或甲自己判错的状态；另查 Q2 清单有没有漏读者）。不碰 Q1 的逐格算术。

## 复跑

- 模型目录 `research/prompts/c245-r3-opus-model/`：`bash research/prompts/c245-r3-opus-model/rerun.sh <草稿目录>`。它把仓的工作树拷两份（`--exclude target --exclude .git`），一份原样（甲），一份打上 `yi-j-mount.diff`、`yi-j-journal.diff`（乙-J 的落后支），两份都放进探针 `c245_r3_opus_probe.rs`（harness 档测试目标，名字不带 layer0），每条经 `run-with-memory-cap.sh 8G` 与 `capped.sh 4` 跑。
- 我这次跑的副本拷自 HEAD `e5253e8a` 加工作区（2026-09-28）；当时四个被测源文件 sha256 前 16 位：`mount.rs` 085e18385d9e1a00、`transaction.rs` af33d518a58401de、`journal.rs` 661a2d3c638bedf5、`recovery.rs` 2051deb895547221。之后别的会话改了这几份，复跑的数可以不同，先比这四个值。
- 模型目录各文件 sha256（`SHA256SUMS` 原样）：

```
caac58b399a0e5e83876cf2ebf0fdb081d0b69a1bdf10fc2defd27945fd25a2c  c245_r3_opus_probe.rs
9bea753eeee6cb497aebb3d0409d151c546a8241d251006840e5891a7ea20253  rerun.sh
e5b9a8aed820e4f67caf83c2039be5d77480f0305c20f16d5a61cc1e023891fe  yi-j-mount.diff
56004971a4b765ee1defee28c22f6a1013b397e5abd9aaf9fe3bb5b734dbaa1c  yi-j-journal.diff
5363222fc5f803b814e459adb5fc27477d6c928465574f177f6fc712932721da  probe-output.txt
```

- 探针是确定性的（内存稀疏盘、固定内容、没有随机源），跑一次说明的是没有隐藏状态，不是统计上稳定。所有数都是副本上量的，不是入库装置上的数。
- 探针不枚举崩溃状态：每个崩溃状态由录制流里拿掉那几条写直接造出（段内写的子集里的一个），故障直接改内存盘字节（根槽清零、单元一份翻一个字节、记录一份清零、系统配置槽清零）。没有调 `enumerate_layer0`，状态数 0，不占 10⁶ / 10⁷ 的额度。

## 各格判定一览

| 格 | 判定 | 证据形态 |
|---|---|---|
| Q2 漏读者 | 漏了：会话每次用户改动之前的落后支（`mounted_session.rs` 的 `publish_user_change`，每次发布都读 tail）；世代号、F 这两个每次轮换都重写的量的读者（择系统配置、写槽算世代号、F 生效值、checker 与层 0 读 F）；取号那一刻读实例代号；挂载算所选那一版位置时的回落分支 | 全仓 grep，第二节逐处列 |
| Q2-C1 判据 N-配置 | **承重**。丙、乙-朴素在「已确认的发布 + 根槽读不出 + 一份点名单元坏」（两处故障，丙另要一次崩溃）上把已确认的发布当被抛弃；甲拒可写。**判别子观测不到**：乙、丙之下这一格的盘面与「根槽 FUA 之前崩 + 一份单元坏」逐字节相同，而两格该有的结局相反 | 真代码探针 B、B2 量过 |
| Q2-C1 乙的最强一种（乙-J2，靠记录与根槽内容见证） | 同样判错：盘面与上一格相同，任何只看盘面的判法在 X、Y 上给同一个答案；另有三处故障（根槽 + 末条记录两份）甲拒、乙-J2 放行 | 探针 B、E 量过 |
| Q2-C1 甲自己 | 要四处故障（根槽 + 一份单元 + 两块盘最新那槽系统配置）才判错；另有已知残留（根槽 FUA 之后、轮换持久之前崩，C554 Q1）同盘面甲放行，那一格发布没确认，不丢已确认数据 | 探针 D、B2 量过 |
| Q2-C3 落后支（可写挂载逐盘核、挂着之后的入口、会话） | **不承重，而且是甲自己判错的一格**：「轮换那一步没持久（正常崩溃点）」加「这块盘上这一版任一单元的一份坏（单故障）」被判不带所选那一版，可写挂载拒，盘上不变、每次都拒，只读挂载读回数据完好。乙-J（落后看这块盘上有没有那一版末条记录）放行这一格，而钉落后支的现有 39 条用例在乙-J 副本上照样全绿 | 探针 A1、A2 量过；乙-J 原型副本上量过 |
| Q2-C3 丙 | 同甲那一格，窗口从「根槽 FUA 到发布末尾屏障」拉长到「返回之后到下一次发布第一道屏障」 | 盘面与 A1、A2 相同（推的：窗口长短没量） |
| Q2-C3 乙-朴素 | 单故障、不要崩溃就拒可写（tail 永远停在取号那一刻，每块盘都「落后」） | 探针 C 量过 |
| Q2-C2 取号写见证值 | 随 C1：乙-J2 不需要它；乙-朴素、丙下它是唯一 / 滞后的见证 | 推的 |
| 其余读者（世代号、F、实例代号） | 没打中：轮换次数变少不让它们判错；F 那一条甲每次发布都多一次「读坏就写低 F」的机会，要多处故障，没造 | 推的 |
| Q3 | 乙、丙、乙-J、丁的定义与代价在第六节；「每次发布轮换」的依据在 C1 站得住、在 C3 站不住 | 见第六节 |

## 一、候选的定义（攻方腿写的，乙、丙都取我能写出的最强一种）

| 名 | 定义 | 最强在哪 |
|---|---|---|
| 甲 | 今天：每次发布末尾 根槽 FUA → 每盘一次系统配置槽轮换（tail = 本次末条计数器）→ 池屏障，屏障做完才返回 | — |
| 丙 | 每次发布照甲轮换，但不等轮换持久就返回；轮换在下一次发布的第一道屏障时持久 | 所有读者照甲不改；唯一的差是「返回之后、下一次屏障之前崩」时最后一次轮换没落 |
| 乙-朴素 | 系统配置只在取号、抬 F、卸载时写；读者照今天读 tail | 用来标出「读者不改就不行」的下界 |
| 乙-J | 乙-朴素的写法，落后支改看记录：这块盘上「那一版末条记录」所在的环槽里读得出、解得开、(实例代号, 计数器) 相同，就算跟得上；否则照今天读单元。每块盘每次核多读一条 4096 字节记录，不写 | 记录在根槽 FUA 之前、过屏障写到每块盘，任何崩溃点上它都不落后于根 |
| 乙-J2 | 乙-J 加见证：环里有带「本次发布末条」标志、(实例, txg) 大于所选那一版的记录，就当有更新的发布（再按根槽内容分两种读法：全 0 算「那次的根没写过」或算读不出）；为真照今天的乙重读一次、仍真拒可写 | 见证用的记录每块盘一份，与系统配置同样分在两块盘上 |
| 丁（攻方腿提的，零轮） | 甲的写法不变，落后支换成乙-J 的判法 | 只动一个读者，不动格式、不动写 |

## 二、Q2 消费者清单（全仓 grep 现查，逐处落点）

每次发布都会变的系统配置量是四个：`journal_tail`、世代号 `slot_generation`（每次轮换加一）、`rollback_floor`（每次轮换按盘上现算的 F 生效值重写）、`journal_instance`（每次轮换重写、只在取号时变）。只数产品代码与 checker；`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` 的 15 处是 E158 装置，按装置列、不判承重。

| # | 读者 | 读的量 | 背景材料第二节列了没有 |
|---|---|---|---|
| C1 | `mount.rs` `newer_publish_witness`：「.map(|slot| slot.quantities.journal_tail)」（`crates/singlefs-core/src/mount.rs:4157`），判据 N-配置 | tail（全池最大） | 列了 |
| C2 | `transaction.rs` `highest_journal_tail_of`（`crates/singlefs-core/src/transaction.rs:671` 起）：取号写与回卷写带的见证值 | tail | 列了 |
| C3a | `devices_without_the_selected_version`（可写挂载取号之前）经 `units_of_the_version_missing_on_a_device_behind_it`：「if newest_system_configuration_journal_tail >= version_journal_position {」（`crates/singlefs-core/src/mount.rs:3124`） | (实例, tail)，按世代号取最新一槽 | 列了 |
| C3b | `devices_behind_the_current_version` 经 `caller_inputs_agreeing_with_the_disk`（`crates/singlefs-core/src/mount.rs:3481` 起：正常卸载、抬 F、管理员回退） | 同 C3a | 列了 |
| C3c | 会话每次用户改动之前：「let devices_behind = devices_behind_the_current_version(」（`crates/singlefs-core/src/mounted_session.rs:284`） | 同 C3a | **漏了**：它是唯一一个随发布频率读 tail 的读者 |
| C4 | 择系统配置每盘取世代号大的那槽：「if one.quantities.slot_generation > zero.quantities.slot_generation {」（`crates/singlefs-core/src/recovery.rs:1013`）；C3 两处按世代号取最新一槽（`crates/singlefs-core/src/mount.rs:3075`、`crates/singlefs-core/src/mount.rs:3167`）；写槽时世代号 = 最大 + 1（`transaction.rs` `write_system_configuration_slot`） | 世代号 | 漏了 |
| C5 | F 生效值：「.map(|system_configuration| system_configuration.quantities.rollback_floor)」（`crates/singlefs-core/src/recovery.rs:1732`）；层 0 的 F 界（`crates/singlefs-checker-tier/src/crash.rs:318`）；checker I-7.12 | F | 漏了 |
| C6 | 取号的号：「.map(|system_configuration| system_configuration.quantities.journal_instance)」（`crates/singlefs-core/src/transaction.rs:620`）；checker I-7.7「.map(|(view, _)| view.journal_instance)」（`crates/singlefs-checker/src/walk.rs:2541`） | 实例代号 | 漏了（它每次轮换重写、值只在取号时变） |
| C7 | 每次轮换写之前现算整池 F：「let pool_effective_floor = effective_rollback_floor_rereading_unreadable_reads_once(」（`crates/singlefs-core/src/transaction.rs:374`），每块盘一次、读整个根环与最新根的实例表 | 读根环，写 F | 漏了（写路径上的读者，也是甲独有的每次发布读开销） |
| — | checker 只解析 tail、不判：「journal_tail: read_u64(slot, SYSTEM_CONFIGURATION_TAIL_OFFSET),」（`crates/singlefs-checker/src/lib.rs:285`）；恢复路径零命中；harness `fault_injection.rs` 两处读实例代号（测试脚手架） | — | 不承重 |

顺带看到、没往下攻的一处：所选那一版的末条记录两份都读不出时，挂载算「所选那一版的位置」回落到环里计数器最大的那条记录（「.or_else(|| records.values().max_by_key(|record| record.counter))」（`crates/singlefs-core/src/mount.rs:4329`）），那条可以比所选那一版新；落后支拿它比。只在所选那一版带末条标志的记录两份都读不出、N-配置 又没拒可写时走得到，没造。

## 三、Q2-C1 判据 N-配置：承重，乙、丙判错，判别子在乙、丙之下观测不到

历史（探针 `build_history`，两块内存稀疏盘、e142 参数 pbs 512）：mkfs → 取号 1 → 暖机 → 第一个文件 txg 3 → 覆盖写 txg 4 → 覆盖写 txg 5。txg 5 就是被攻的「最后一次发布 P」。

两格：

- **X**（P 已返回、fsync 已确认）：P 的根槽读不出（清零），P 的数据单元盘 1 那一份坏一个字节。两处故障。根读不出，择根落到 txg 4；重放 P 的记录时点名单元两份没有都验过，不施加，所选那一版是 txg 4。
- **Y**（P 的根槽 FUA 之前崩，没确认）：P 的数据单元盘 1 那一份坏。一次崩溃加一处故障。

该有的结局：X 不许把已确认的 P 当被抛弃（它的单元会被再发出去）——甲的做法是重读一次仍读不出就拒可写；Y 放行、接 txg 4（P 没确认，落回去合 「fsync 等系统配置轮换持久之后才返回」（`.claude/kb/decisions/16-发布语义.md:179`））。

探针 B、B2、D、E 的原样输出（`probe-output.txt`，甲副本）：

```
probe=B-identity x_equals_y_under_jia=false under_bing=true under_yi_naive=true
probe=B state=X-jia writable_mount=RefusedByTheWitnessOfANewerPublish
probe=B state=X-bing writable_mount=Mounted { effective_instance: 1, effective_txg: 4, new_instance: 2 }
probe=B state=X-yi-naive writable_mount=Mounted { effective_instance: 1, effective_txg: 4, new_instance: 2 }
probe=B state=Y-jia-or-bing writable_mount=Mounted { effective_instance: 1, effective_txg: 4, new_instance: 2 }
probe=B state=Y-yi-naive writable_mount=Mounted { effective_instance: 1, effective_txg: 4, new_instance: 2 }
probe=B-yi-j2 zero_slot_means_never_written=false witness_on_x=true witness_on_y=true
probe=B-yi-j2 zero_slot_means_never_written=true witness_on_x=false witness_on_y=false
probe=B2 bing_writable_mount=Mounted { effective_instance: 1, effective_txg: 4, new_instance: 2 } faults_lifted=[true, true] read_only_after_lift=Ok((7, false, true))
probe=D state=X+sc0 faults=3 jia=RefusedByTheWitnessOfANewerPublish
probe=D state=X+sc1 faults=3 jia=RefusedByTheWitnessOfANewerPublish
probe=D state=X+sc0+sc1 faults=4 jia=Mounted { effective_instance: 1, effective_txg: 4, new_instance: 2 }
probe=E faults=3 jia=RefusedByTheWitnessOfANewerPublish yi_naive=Mounted { effective_instance: 1, effective_txg: 4, new_instance: 2 } yi_j2_witness(zero=ambiguous,zero=never)=[false, false]
```

读法：

- `B-identity`：按读出的字节比，X 与 Y 在丙之下（X 拿掉 P 的轮换写）逐字节相同，在乙-朴素之下（取号之后一次系统配置写都没有）也相同；甲之下不同——差的就是 P 那次轮换写的 tail。乙、丙之下根槽 FUA 之后盘上一个写都没有，「P 的根写过又读不出」与「P 的根从没写」只差在根槽里，而那一槽正是故障抹掉的。
- 丙、乙-朴素在 X 上可写挂载落到 txg 4、取号 2。B2 再把两处故障撤掉（两处都没被新实例盖到，`faults_lifted=[true, true]`）：只读挂载落到 txg 7（新实例），读回 txg 4 的内容（`false, true` = 不等于 txg 5 的内容、等于 txg 4 的）。丙之下 txg 5 已向调用者确认过，这就是丢已确认的数据。
- 乙-J2 两种根槽读法在 X、Y 上各给同一个答案（盘面相同）：全 0 算读不出时 X 对、Y 拒（Y 的故障是持久坏，每次都拒）；全 0 算没写过时 Y 对、X 放行（丢已确认的 P）。任何只看盘面的判法都一样。
- D：甲在 X 上要再坏掉两块盘各自最新那槽系统配置（共四处）才放行；只坏一块盘的仍拒。
- E：乙-J2 丢已确认发布的另一形只要三处（P 的根槽 + P 末条记录两份，记录读不出重放本来就接不上），甲在同一格拒。

四句（X / Y 这一对）：

1. 分辨臂：分辨。甲拒 X、放 Y；丙、乙-朴素、乙-J2 两种读法都在其中一格判错（量过）。
2. 被判的系统当时看不看得到判别子：甲看得到（系统配置 tail，写在根槽 FUA 之后）；乙、丙看不到——盘面逐字节相同（量过）。这是 `evidence-discipline.md`「判别子观测不到」那一型，打在乙、丙身上：不是改读者能修的。
3. 满足判据字面的哪个分句：跑前条款第一条「Q2 里至少有一处「承重」且那一处在乙、丙下给得出判错的具体状态」——C1 满足「具体状态」；「乙、丙的代价没有比甲省下一个数量级」是 Q1 的格，我没算。
4. 跑前条款给的改法在这几格上还中不中：条款没给改法。我能想到的乙侧改法（乙-J2 两种读法）在这一对上照样中；另一个方向「每次发布轮换但只在一块盘上写」（推的，没实现）不改变「FUA 之后有一个写」这件事，X / Y 分得开，但见证只剩一份。

**故障数与射程**：X 是两处故障（根槽一时读不出 + 一份点名单元读不出），丙另要一次崩溃。它正是 C577 那一格（用户 2026-09-27 为它加了发布返回前那道屏障）。而 C583 那一行写着双故障形「用户 2026-09-27 定不在容错射程」（`.claude/kb/checks-owed.md:463`）。X 算不算射程内，要主 agent 按这两条定案判；我只报故障数，不替它判。

**甲自己在 C1 上的残留**（已知，C554 Q1）：B2 那份盘面在甲之下就是「P 的根槽 FUA 已落、两块盘轮换都没持久时崩」再加 X 的两处故障，甲放行、P 被抛弃、根读不出影子账隔离不到它的单元。甲之下 P 没确认，丢它合条款，代价是之后 I-7.4 红（没在探针里跑 checker，推的，与 C583 同形）。乙-J2（全 0 算读不出）在根槽「读报错」语义下会拒这一格：P 的记录在，根槽读不出。这是乙-J2 唯一比甲多挡住的一格（推的，探针的故障都是清零，没造「读报错」语义）。

## 四、Q2-C3 落后支：甲自己判错，乙-J 不判错，每次发布轮换在这一处不承重

条款：D18 已定项 11 取号之前逐盘核那一段写「放行两种：只落后、单元都在」（`.claude/kb/decisions/18-块里携带什么信息.md:314`），另一种是「系统配置跟得上、只缺几份单元（单份坏」（`.claude/kb/decisions/18-块里携带什么信息.md:314`）。代码同一判：「只落后、单元都在的不算：发布的根落盘之后、系统配置轮换之前崩了」（`crates/singlefs-core/src/mount.rs:647`）。两种各自放行，两样同时出现（落后 **且** 缺）就判「不带所选那一版」。要读的单元是整版的：「带文件的一版就是重建出来的 `units`（全部角色」（`crates/singlefs-core/src/mount.rs:937`），不只是最后一次发布写的那几个。

造的状态（探针 A1）：同一段历史，P（txg 5）的根槽 FUA 已落，盘 1 上 P 的轮换写没持久（段内子集里的一个：根槽 FUA 切段，轮换两写在同一段）；盘 1 上 P 的数据单元那一份坏一个字节。一次正常崩溃 + 一处单份坏。

```
probe=A1 state=rotation-lost-on-dev1+dev1-data-copy-corrupted jia_first=RefusedByDevicesWithoutTheSelectedVersion { devices: [1], selected_txg: 5 } jia_second=RefusedByDevicesWithoutTheSelectedVersion { devices: [1], selected_txg: 5 } disk_unchanged=true read_only_txg_and_content_ok=Ok((5, true)) yi_j_devices_without=[]
probe=A1-control rotation_lost_only=Mounted { effective_instance: 1, effective_txg: 5, new_instance: 2 } corrupted_copy_only=Mounted { effective_instance: 1, effective_txg: 5, new_instance: 2 }
probe=A1-stale jia=RefusedByDevicesWithoutTheSelectedVersion { devices: [1], selected_txg: 5 } yi_j_devices_without=[1]
probe=A2 state=both-rotations-lost+data-copy-corrupted-on-dev0 jia=RefusedByDevicesWithoutTheSelectedVersion { devices: [0], selected_txg: 5 }
probe=A2 state=both-rotations-lost+data-copy-corrupted-on-dev1 jia=RefusedByDevicesWithoutTheSelectedVersion { devices: [1], selected_txg: 5 }
probe=C faults=1 yi_naive=RefusedByDevicesWithoutTheSelectedVersion { devices: [1], selected_txg: 5 } jia=Mounted { effective_instance: 1, effective_txg: 5, new_instance: 2 }
```

读法：

- 甲拒可写，盘上一个字节没动（`disk_unchanged=true`），第二次照样拒；只读挂载读回 txg 5 的内容完好。只读挂载不写，这块盘的系统配置永远停在那里：这个池从此只能只读，直到有人重同步。两个对照格（只丢轮换、只坏一份）各自都挂得上，与现有用例 `device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies`、`current_device_one_with_one_corrupted_copy_still_mounts_writable` 同结局；后者的注释自己写着「这一格交主 agent 定，见报告」（`crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs:1916`）。叠在一起这一格没有用例钉。
- A2：两块盘轮换都没持久时两块盘都落后，单份坏落在哪块盘上都拒那块。
- 潜伏坏那一支（坏的是 txg 3 写下、txg 5 那一版还引用着的单元）这段历史里没走到：覆盖写每次重写整条文件路径，`A1-latent` 一行都没打出来（历史里没有 txg 3 与 txg 5 共用的非空单元）。按 `mount.rs:937` 的「全部角色」，更长的历史里整版任一单元的潜伏坏都算（推的）。
- 乙-J 在同一盘面放行（`yi_j_devices_without=[]`：盘 1 带着 txg 5 的末条记录）；旧快照那一格（盘 1 停在 txg 3）甲、乙-J 都拒（`[1]`）。
- 把乙-J 的判法打进副本的 `mount.rs`（`yi-j-mount.diff`，只改 `units_of_the_version_missing_on_a_device_behind_it` 里「落后」那一判），钉落后支与「可见」支的现有用例在乙-J 副本上全绿，探针的 A1、A2、C 三条按预期变红（不再拒）：

```
behind-yij-entries_after_mount_refuse_swapped_or_behind_devices: test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.49s
behind-yij-fsync_drop_and_devices_without_the_selected_version: test result: ok. 16 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 75.51s
behind-yij-acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.13s
behind-yij-entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
behind-yij-c245_r3_opus_probe: test result: FAILED. 4 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.87s
```

  （1 条 ignored 是 `fsync_drop…` 里原有的耗时用例，没跑。乙-J 副本上仍在写每次发布的轮换，所以它量的是「落后支不读 tail」这一半，不是整个乙。）
- 乙-朴素（探针 C）：不要崩溃，一份坏就拒——读者不改的乙在这一处比甲还差。

四句：

1. 分辨臂：分辨。甲、丙、乙-朴素拒；乙-J、丁放行（量过）。
2. 系统当时看不看得到判别子：看得到——盘 1 上 P 的记录在（记录在根槽 FUA 之前过屏障），挂载的读阶段本来就扫了环。今天的判法不用它，用的是写在 FUA 之后、在这个崩溃点上必然滞后的 tail。
3. 满足的分句：它不是「乙、丙判错」，是「甲自己也判错」：盘面落在条款字面「不带」的定义里，拒是照字面；错在判据——两种放行各自写了，没写它们的交集。按 `evidence-discipline.md` 的表，这是「判据自己写错」、打中分辨臂的那一型。
4. 改法在这几格上还中不中：丁（甲 + 落后支看记录）在 A1、A2 上不中（副本量过）；乙-J 同。「落后就拒」（今天的另一候选，取舍记在 D18 已定项 11 射程）在 A1 上照样中，而且只丢轮换不坏单元也拒（推的）。

**这一处对 Q2 / Q4 的意思**：落后支读 tail 是因为 tail 恰好每次发布都写，不是因为它需要一个写在 FUA 之后的量；换成记录更准。它不能当「每次发布轮换」的依据。

## 五、其余读者（C2、C4–C7）：没打中

| 读者 | 试了什么 | 结果 |
|---|---|---|
| C2 取号写见证值 | 乙-J2 下见证改由记录出，取号写的 tail 没有读者；乙-朴素、丙下它是 C1 唯一 / 滞后的见证 | 不独立承重，随 C1（推的） |
| C4 世代号 | 轮换次数少了，择槽、写槽算世代号都不因此判错；新槽读不出时写槽落回同一槽位，内容更新，不伤（推的） | 没打中 |
| C5 F | F 只在抬 F 时变，乙之下照写；每次轮换带「写之前现算的 F」反而给甲每次发布一次写低 F 的机会：代码自己注明「这一种叠加故障算得低（推的，没造过）」（`crates/singlefs-core/src/transaction.rs:373`）。要把全部带高 F 的槽与根同时读坏，多处故障，没造 | 没打中；方向不利于甲（推的） |
| C6 实例代号 | 值只在取号时变，乙照写 | 没打中 |
| C7 轮换前现算 F | 甲独有的每次发布读开销：每块盘一次、读整个根环与最新根的实例表链（推的，没量） | 进第六节代价，不是判错 |

## 六、Q3：候选、Q2 各处要改什么、会不会出新的错判、代价（代价只给次数，逐格数归 Q1）

| 候选 | C1 | C3 | 新的错判状态 | 每次发布比甲少 / 多（推的，按 `transaction.rs` 的步骤数，没量） |
|---|---|---|---|---|
| 甲 | 两处故障不判错，四处才判错（量过） | 崩溃 + 一处单份坏就永久拒可写（量过） | — | 基准：每盘一次系统配置槽写（4096 字节整槽）、末尾一道池屏障、轮换前每盘现算一次 F 读根环 |
| 丙 | X 上丢已确认的发布（量过）；判别子在盘上没有，改读者修不了 | 同甲，窗口拉长到返回之后、下一次发布第一道屏障之前 | X | 写不少；少末尾那道屏障的等待（序点少一个，下一次发布第一道屏障放行它） |
| 乙-朴素 | X 上丢已确认的发布（量过） | 单份坏、不要崩溃就拒（量过） | X、单份坏 | 每次发布少两次系统配置写、少一道屏障、少 C7 的读 |
| 乙-J + 乙-J2 | X / Y 必错一格（量过）；三处故障丢已确认的发布（量过） | 不判错（副本量过） | X 或 Y；三处故障那一格 | 同乙-朴素；会话每次改动多读每盘一条记录，少读每盘两槽系统配置 |
| 丁（甲 + 落后支看记录） | 同甲 | 不判错（副本量过） | 没发现 | 同甲 |

要改的 Q2 处：丙不改读者（改了也没用，判别子不在盘上）；乙-朴素不改读者就在 C1、C3 两处判错；乙-J 改 C1、C2、C3a/b/c；丁只改 C3a/b/c 共用的那一个函数。

## 七、改法各修哪一格

| 改法 | A1 / A2（C3 交集） | C（乙-朴素单份坏） | X（C1） | Y（C1） | 三处故障（E） |
|---|---|---|---|---|---|
| 丁 = 甲 + 落后支看记录（`yi-j-mount.diff`） | 修（量过，乙-J 副本上 A1、A2 不再拒，现有 39 条用例全绿） | 不适用（甲不写成乙） | 不动（推的） | 不动（推的） | 不动（推的） |
| 乙-J2，全 0 算读不出 | — | — | 修（量过） | 错拒（量过） | 不修（量过） |
| 乙-J2，全 0 算没写过 | — | — | 不修（量过） | 修（量过） | 不修（量过） |

改法都只在我的模型与副本上量过，被攻过零轮。丁那一行只量了钉落后支的 4 个测试目标与探针，没跑整个 harness，也没跑 checker 档。

## 没打中的形状

- C1 上找单故障（含崩溃 + 一处）让乙、丙判错的形状：X 至少两处——根读不出之后，同一实例里重放会从记录把 P 接回来，除非有一份点名单元读不出或记录读不出；零单元的 P 不点名单元、重放必成。跨实例边界那一支（新实例的根全读不出）要两块盘的根区一起坏。都没造出一处的。
- C3 上找乙-J 的新错判：乙-J 要把「落后」判成真，得这块盘上那条记录读不出，那是一处故障，再叠一份单份坏才拒——与甲「最新那槽系统配置读不出 + 单份坏」同为两处，没找到乙-J 独有的少故障形。没量，推的。
- 潜伏单份坏（旧单元）：这段历史里没有 txg 3 与 txg 5 共用的非空单元，这一支零取样。
- 取样范围：一条固定历史（到 txg 5、一个实例、每次发布一条记录、pbs 512）；故障只取清零 / 翻一个字节；没有「读报错」语义，没有多记录发布、没有第二个实例、没有回退、没有抬 F。

## 这条腿自己的限度

- 乙、丙没有实现成产品代码，是按录制流拿掉写造出的盘面；乙-J 只实现了落后支一半。乙之下的写次数、屏障位置与这里的盘面是否处处一致，只按「系统配置写只剩取号那两写」推。
- 判射程（两处故障算不算承诺内）没做：C577 与 C583 两条用户定案方向不同，交主 agent。
- Q1 的逐格数没算；第六节的代价是按代码数步骤，推的。
- 没跑 checker 档、门禁、层 0；探针在副本上跑，数不是入库装置上的数。

## 草稿目录

删了（交回之前）：`/tmp/claude-1000/c245-r3-opus/repo`（371M，甲的仓副本）、`/tmp/claude-1000/c245-r3-opus/repo-yij`（371M，乙-J 的仓副本）、`/tmp/claude-1000/c245-r3-opus/target`（1.2G）、`/tmp/claude-1000/c245-r3-opus/target-yij`（1.2G）。留着 `/tmp/claude-1000/c245-r3-opus/` 下的日志、`.rc` 与 `src/`（共 344K，不是仓副本、也不是编译目录）；要入库的探针与输出已拷进模型目录。
