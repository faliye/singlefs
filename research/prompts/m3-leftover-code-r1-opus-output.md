# m3-leftover-code-r1 攻方腿报告（Opus，2026-10-03）

攻击面：正文第三节 K2（重点）、K3、K4、K5。代码读的是冻结副本 `/tmp/claude-1000/m3-leftover-code-r1/tree/`，下文引代码一律写它的绝对路径与它自己的行号。副本上量的数都是副本上的数，不是入库装置上的数。只跑了 harness 档（`singlefs-harness` 的测试目标）与自己的单文件 Rust 模型；checker 档一条没跑。

## 复跑

模型目录 `research/prompts/m3-leftover-code-r1-opus-model/`。在仓根：

```
export OPUS_DIR=/tmp/claude-1000/<新草稿目录>      # 空目录
bash research/prompts/m3-leftover-code-r1-opus-model/setup.sh     # 拷冻结副本、往 mount.rs 末尾加探针、放三份探针用例、存 clean 副本
cd $OPUS_DIR/work && export CARGO_TARGET_DIR=$OPUS_DIR/target
# 探针（K2 ②、K3）：
bash <仓根>/research/scripts/run-with-memory-cap.sh auto bash <仓根>/research/scripts/capped.sh 6 cargo test --offline -p singlefs-harness --test opus_probe_account_vs_mapping -- --nocapture --test-threads 2
bash <仓根>/research/scripts/run-with-memory-cap.sh auto bash <仓根>/research/scripts/capped.sh 6 cargo test --offline -p singlefs-harness --test opus_probe_swallowed_system_configuration_write -- --nocapture
bash <仓根>/research/scripts/run-with-memory-cap.sh auto bash <仓根>/research/scripts/capped.sh 6 cargo test --offline -p singlefs-harness --test opus_probe_k3_outlets -- --nocapture
# 变异（K2 Z4 ①④、Z2 ③④；每个变异一条命令，名字见 mutants.json，用例表见 g_*.list）：
bash <仓根>/research/scripts/run-with-memory-cap.sh 12G bash <仓根>/research/scripts/capped.sh 6 python3 <模型目录>/runner.py <变异名> $OPUS_DIR/mut/mutants.json $(cat $OPUS_DIR/mut/g_wit.list)
# 出路草图下重跑探针：python3 <模型目录>/apply.py r2（或 b-zero），跑上面的探针，跑完 python3 <模型目录>/apply.py --restore
# K4 模型：cp <模型目录>/fold_model.rs . && rustc -O --edition 2021 fold_model.rs && bash <仓根>/research/scripts/run-with-memory-cap.sh auto ./fold_model
```

本会话里与别的腿并行，包装要 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`；改了核心库再跑探针要 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`（峰值表只记了只跑测试的峰值）。挂钟：单个变异过 `g_wit.list`（98 条）约 2 分钟，`g_map.list`（69 条）约 1 分钟，探针各 20 秒到 3 分钟，K4 模型不到 1 秒。

| 文件 | sha256 |
|---|---|
| apply.py | d4b2aa29c1c7e3e7374bb757d19afacfc37f0199e66c5e5050ef393d125f05f6 |
| fold_model.rs | 9d5142f8ddc8bb64eb0ec85b904df1c197b84cc67821fafaa6a01ec6e73c3c52 |
| g_defer.list | 41d2fd6ff4fed8694a7138bdb1133c7dfafe4b1ccb90181959b5af2f863ff7e6 |
| g_map.list | a996fe81bf9bdd4d97f48945f310d74278626d0ac7ec7c8e48b6ee9b99cb4566 |
| g_wit2.list | c84467649aedc93b5238b8c4807f5c9af6ce6655eeb6774df77538ca59f5e874 |
| g_wit.list | d4e05cadfbbab9af3f0157b62bd032a4fcda3d555134fd7ae0767a8678a292c5 |
| mount-probe-append.rs | 88624f9dccbc94a90e4d3db4fa566a139cd001f77a63675406e09b51316e2691 |
| mutants.json | e0cd421efe245ec33fe02ab86479673ff343e4be32d660b2f751ff8256bd01c0 |
| opus_probe_account_vs_mapping.rs | da93c398022cd7f9b46315360383eb7810fd697424ac226a9947f44e99fda543 |
| opus_probe_k3_outlets.rs | 917292c7d1fda2d850141d9e558c713a9185177f290f74f389cf740ac2aba74d |
| opus_probe_swallowed_system_configuration_write.rs | 286534037a2f2e324579bc939f19b88051f40a1cc14f3ac4eff824c15f71f341 |
| runner.py | 4083936f8ab82446e6368add98ef9253f5c65a386fe4cc44378451eb03908acb |
| setup.sh | 949f001fb2cb3e08d4fbdbb8f712aea4c8ce4c895cb236fa1bd21c7bfc6ee7ce |

`logs/` 下是这一轮各次跑的结果行（`grep` 过 `name=`、`FAILED`、`test result` 的那几行），下文引的数都在那里。

## 各格判定一览

| 格 | 判定 | 一句话 | 量过 / 推的 |
|---|---|---|---|
| K2 Z2 ① 缺一槽在各状态下判什么 | 没打中「永远只读之外的新形」；永远只读那一形多一个成因（就是 C600，见 K3） | 设备吞掉 mkfs 写系统配置槽的那一次写（每盘两次里任一次），之后每一次可写挂载都拒，mkfs 同一个进程里的取号也拒：空池永远只读；吞别的系统配置写（取号写、轮换写）不拒 | 量过 |
| K2 Z2 ② 现算落点与账逐个相同吗 | 没打中；树表 0 条那一版的永远只读是条款认过的代价，附一条零轮改法 | 152 条根环里的根逐条比：带文件的 107 条两臂逐个相同（含两层映射树、2000 个 inode、unlink 之后、两片实例表）；树表 0 条的 45 条现算一律交 `None` | 前半量过，后半推的 |
| K2 Z2 ③ 抬 F 被拒时分配器换回的次序 | 没打中（实现对）；但没有用例走到这一支 | 换回在任何写之前；删掉换回那一句的变异过 69 条用例全绿 | 量过 |
| K2 Z2 ④ defer 槽盖不盖「该回收没回收」 | 没打中 | 抬 F 一槽不回收的变异下 I-3.1 照红（7 条用例红） | 量过 |
| K2 Z2 ⑤ 被抛弃根的 F 让回退 / 抬 F 误拒 | 没打中 | 合法历史里被抛弃根带的 F 不高于系统配置里的 F（SysPre），算进与不算进生效值相同 | 推的 |
| K2 Z4 ① 测试放得过的错实现 | 打中 4 种 | 「见证读与取号只看槽 1」过 286 条；「现算只认实例表第 0 片」过 69 条；「抬 F 影子账被拒不换回分配器」过 69 条；「defer 把被抛弃根引用的也并进来」过 56 条 | 量过 |
| K2 Z4 ② 造盘面合不合法 | 打中 1 处（用例自己写明） | 孤记录那一格的盘面真崩溃留不下，只有说谎的设备留得下 | 读代码 |
| K2 Z4 ③ 改钉之后没人守的旧行为 | 没做 | 见「限度」 | — |
| K2 Z4 ④ 变异表新 6 行 | 没打中 | 6 行各自红点名的用例；第 1415 行的替换文退回的是「拒可写」不是旧的「跳过」，但旧的「跳过」另造一个变异也红同 5 条 | 量过 |
| K3 C600 两条出路 | 只交结局 | C600 在副本上确定性复现；今天的 D23 已定项 14 写的判法是 R2，代码还是甲；R2 草图下 C600 这一族全部能挂，代价是「纯形」那一族照条款认的丢写（量到一条：已确认的发布被新实例的行抛弃） | 量过 |
| K4 写零折叠 | 没打中 | 切块与 `write_zeroes_in_chunks` 同式；20 万轮随机到达里合法乱序全折、丢一段全不折、块间对调全不折 | 量过（单文件模型） |
| K5 对接上游第三轮之后 | 没打中；一处口径不一（拒的一侧） | 384 槽几何三样逐项核着；白名单按引入者全记；数值去空白四入口一致，取值类变量不去空白 | 读代码 |

## K2 Z2：实现哪一格会错

### ① 见证读「缺一槽」在各状态下判什么

判法在 `/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:4676`：「< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE」（每块盘两槽里自证过的少于两份就判不出、按真），取号那一侧同式，`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/transaction.rs:697`：「< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE」。

造法：故障注入的说谎设备（`InjectedFault::WriteIsSwallowed`），只挑系统配置两槽那一段（偏移小于两倍槽距），按盘数第 n 次那样的写吞掉一次，n 从 1 扫到 8（mkfs 起步）或 4（只做过 mkfs 起步），两块盘各扫一遍，用户动作两种（重开、覆盖写再重开），只固定「吞哪一次写」这一个故障。探针 `opus_probe_swallowed_system_configuration_write.rs`，结果行在 `logs/c600-mkfs.log.lines`、`logs/c600-probe.log.lines`。

只做过 mkfs 起步、接着可写挂载（每盘第 1 次是 mkfs 的 1 MiB 整段清零，第 2、3 次是 mkfs 写槽 0、槽 1，第 4 次是挂载自己的取号写）：

```
device=0 n=1 fired=["dev0 off0 len1048576 ending=Completed 
device=0 n=2 fired=["dev0 off0 len4096 Refused { member: \"MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion) ending=NewFinding 
device=0 n=3 fired=["dev0 off4096 len4096 Refused { member: \"MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion) ending=NewFinding 
device=0 n=4 fired=["dev0 off0 len4096 ending=Completed 
device=1 n=1 fired=["dev1 off0 len1048576 ending=Completed 
device=1 n=2 fired=["dev1 off0 len4096 Refused { member: \"MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion) ending=NewFinding 
device=1 n=3 fired=["dev1 off4096 len4096 Refused { member: \"MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion) ending=NewFinding 
device=1 n=4 fired=["dev1 off0 len4096 ending=Completed
```

`NewFinding` 的签名是「模型说该成、实现拒了」，与 C600 的条目逐字同一形（`.claude/kb/checks-owed.md:463`：「吞写之后可写挂载被拒而模型说该成」）。mkfs 同一个进程接着取号、暖机、第一个文件起步时，吞第 2、3 次在起点段就停：取号拒成 `DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`（`logs/c600-probe-device0-n2.full`）。两种起步、两种用户动作、两块盘共 32 段里，拒的恰是 mkfs 写槽的那 8 段；吞取号写、轮换写的 24 段都照常挂上（被吞的槽里留着旧的那一份，仍自证得过）。

判：Z2 ① 问的是「永远只读之外的新形」，这里没有；永远只读那一形多了一个成因：mkfs 的槽写被吞，那一槽是 mkfs 清零留下的全 0，往后没有任何写会去重写它（可写挂载与取号都在第一个写之前拒），池里一个文件都没有就只剩只读。C600 用旧种子在今天的副本上复现不出来了（`SINGLEFS_FAULT_INJECTION_FIRST_SEED=7463871032432355222` 那一段报「新发现 0」，`logs/c600.log.lines`；生成器在删除批加了两类操作，同一个种子抽到的已不是那段历史），这里给出的是不靠种子的造法。其余几个状态：撕裂轮换照条款 L1 的代价拒（`a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs` 已钉）；只剩一块盘可见在读见证之前就被盘数那一核拒，走不到这一判。

四句：分辨臂（K3 的两条出路在这一族上结局不同，见 K3）；判别子：挂载那一刻看得见那一槽是全 0、另一块盘两槽都是 mkfs 那一份，但「读回全 0」也是 harness 自己的暂时读错形（`UnreadableRangeReadBack::Zeros`），只看「全 0」分不出这一形与危险的那一形（K3 的 H4）；字面：Z2 ①「该可写而被拒成永远只读」那一类本身；改法：见 K3 表。

### ② 现算的落点集与账里的逐个相同吗

探针 `opus_probe_account_vs_mapping.rs`：在副本的 `mount.rs` 末尾加两个只读探针函数（`mount-probe-append.rs`），对根环里每条可读根同时调账那一臂 `placements_referenced_by_root` 与现算那一臂 `placements_referenced_by_root_computed_from_the_mapping`，按 (盘, 槽, 跨度) 的集合逐个比。历史：内存稀疏盘上第一个文件；可写挂载；顺序写 320、150、330 个数据单元（映射条目过 294，中央映射树长成两层）；重开；建 2000 个 inode（多片叶容器，打包单元）；覆盖写成 1 个单元；顺序写 310 个单元；unlink 第一个文件（意图加全部分批，墓碑树、谱系树、`deleted_inodes`）；重开。另一条：连着 375 次取号之后崩溃，下一次可写挂载写出两片实例表，再覆盖写三次。

```
    107 tree_table_empty=Ok(false) mapping_root_empty=false
     45 tree_table_empty=Ok(true) mapping_root_empty=true
name=opus-probe-pages pages=2
name=opus-probe-total-two-pages differing_roots=0
name=opus-probe-total differing_roots=0
```

（`logs/run-probe3.log.lines`。）带文件的 107 条根两臂逐个相同，每条根的落点数从 26 到 724；树表 0 条的 45 条，现算一律交 `None`，账那一臂交 4 个落点。

没打中「多隔离、少复用」。树表 0 条那一版现算交 `None` 是写明的（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:1408`：「映射根是空指针的 mkfs 第 0 代、树表 0 条的一版也在内」），后果是一条树表 0 条的被抛弃根只要树表两份读不出，可写挂载就永远拒；这一形条款认过（`.claude/kb/decisions/23-journal的角色与格式.md:408`：「C393（被抛弃根的账读不出时修复没有条款） 从映射也现算不成的那一形同样只剩只读挂载」），已有用例按「拒」钉（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:562`：「它是树表 0 条的一版、从中央映射也现算不成」）。可补的一点：152 条根里「树表 0 条」与「映射根是空指针」逐条同真同假，所以现算那一臂在映射根是空指针时可以直接交账那一臂树表 0 条分支算的那几样（实例表各片、树表指针、分配记录树各节点，全在根记录里），不必读树表。只在我的模型上量过那个对应关系，改法本身被攻过零轮、没实现没跑（推的）。

没造的一形（推的）：隔离的单元（释放时读盘核对不上、记录留在已分配而映射条目已去掉）在账里、不在映射里，账读不出走现算时这几个槽不隔离。它们不被这条被抛弃根的任何数据引用，复用它们不丢这条根的数据；没造，没量。

### ③ 抬 F 那条路上影子账被拒时的次序

`raise_the_floor_through` 先克隆分配器、核参数与盘表、算生效 F、判上限、算这一串的 txg、读根环，然后才调影子账；影子账交 `Err` 时换回克隆（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/mount.rs:2562`：「*allocator = allocator_before_the_raise;」）再返回，这之前没有写（第一个写是之后的系统配置写）。次序是对的。没有用例走到这一支：删掉换回那一句的变异（`raise-no-restore`）过 `g_map.list` 69 条全绿（`logs/mut-raise-no-restore.log.lines`：「name=raise-no-restore cases=69 failed=0」）。后果只是这一次抬 F 多留几个隔离位（只增不减，清在被抛弃根离开根环那一次），不会复用。归 Z4 ①。

### ④ defer 槽会不会盖住「该回收而没回收」

变异 `raise-no-reclaim`（抬 F 那一串回收门槛写成 0，一槽都不回收）过 `g_defer.list` 56 条，7 条红，其中点名的那一条红在 I-3.1：

```
name=raise-no-reclaim cases=56 failed=7
```

（`logs/mut-raise-no-reclaim.log.lines`；失败说明里是「记账的已分配 Some(2572288)，遍历全部有效根得到 2326528（其中隔离豁免 0、F 之下仍在 defer 的 163840）」，摘在 `logs/mut-raise-no-reclaim.failed.excerpt`。）defer 那一并只收「释放代高于 F_生效」的槽（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker/src/walk.rs:4649`：「if record.is_released && record.generation > effective_rollback_floor {」），该回收的是释放代不高于 F 的，两者不交，所以盖不住。没打中。另一个变异 `raise-reclaim-only-to-floor`（回收只到新 F、不看环里最旧有效根）56 条全绿：抬 F 之前每次发布已按同一门槛回收过，这一截在抬 F 那一刻已是空的，两者行为相同（推的）。

### ⑤ 被抛弃根带的 F 算进 F_生效 之后，回退或抬 F 被误拒

没打中，推的、没跑。理由：带新 F 的根落盘之前，这一串先把新 F 写进每块盘的系统配置（SysPre），之后每次轮换写的 F 取盘上现算的生效值、只增不减；所以一条被抛弃根带的 F，在没有叠加读错的历史里已经在系统配置那一半里，算进与不算进 F_生效 相同，不会多拒一次回退或抬 F。只在「系统配置两槽与带最高 F 的根同时读不出」那一形上两者不同，那一形本身已在 `transaction.rs` 写系统配置槽那一段的注释里记为「推的，没造过」。抬 F 的上限按每块盘最新的有效根取，被抛弃根至多比落到的那一版高一个 txg（单次崩溃只抛弃最新那一条，再早的根系统配置见证过、C554 乙会拒），新实例写行与暖机之后上限就越过它。

## K2 Z4：测试放不放得过错的实现

### ① 错实现逐个过用例

每个变异单独施加在副本上（每处替换恰好命中一次，跑完从 clean 副本恢复），用例按条并行、各 1 线程，`#[ignore]` 的慢档没跑。用例表：`g_wit.list`（C331 甲那几份，8 个目标 98 条）、`g_wit2.list`（另外凡是写系统配置槽的 17 个目标 188 条）、`g_map.list`（C393 与被抛弃根那几份，6 个目标 69 条）、`g_defer.list`（defer 与抬 F，6 个目标 56 条）。不变异时 98 条与 56 条全绿（`logs/mut-base-wit.log.lines`、`logs/mut-base-defer.log.lines`），69 条那 6 个目标在串行基线里全绿（`logs/base-cargo-serial.lines`）。

| 变异 | 错在哪 | 用例 | 红几条 | 判 |
|---|---|---|---|---|
| slot1-only-both | 见证读与取号都只看每块盘的槽 1，槽 0 坏了照样判「都读得出」 | g_wit 98 + g_wit2 188 | 0 + 0 | **放过** |
| w-first-device-only | 见证读只看盘表第一块盘 | 4 个目标 22 条 | 1 | 抓到 |
| acq-first-device-only | 取号只看第一块盘 | 同上 | 4 | 抓到 |
| w-newest-only | 见证读只看世代号是不是全池最大 | g_wit 98 | 15 | 抓到（红在落后盘那几条，是多拒不是少拒） |
| map-no-packed | 现算少算打包单元 | g_map 69 | 5 | 抓到 |
| map-first-instance-page-only | 现算只认实例表第 0 片 | g_map 69 | 0 | **放过** |
| map-no-mapping-nodes | 现算少算中央映射树自己的节点 | g_map 69 | 5 | 抓到 |
| map-no-tree-table | 现算少算树表 | g_map 69 | 5 | 抓到 |
| map-not-counted | 现算成了不计进 `abandoned_roots_unreadable` | g_map 69 | 5 | 抓到 |
| raise-no-restore | 抬 F 那条路影子账被拒时不换回分配器 | g_map 69 | 0 | **放过** |
| defer-ignores-generation | defer 不看释放代 | g_defer 56 | 6 | 抓到 |
| defer-includes-abandoned | defer 把被抛弃根引用的也并进来 | g_defer 56 | 0 | **放过** |

最短的那几种（替换文在 `mutants.json` 同名键下）：

- slot1-only-both：`newer_publish_witness` 里「缺一槽」改成「这块盘偏移一个槽距处那一槽读不出或自证不过」，`self_verified_system_configurations_of_every_device` 同改。全部写坏系统配置槽的用例坏的都是世代 5 那一槽，5 对 2 取余落在槽 1，所以看不出只查了槽 1。撕裂轮换落在偶数世代时坏的是槽 0：这个错实现照样可写挂载、照样取号，C331 甲要挡的那一形放过去一半。
- map-first-instance-page-only：现算那一臂只取实例表链的第 0 片。用例的池都只有一片实例表。实现本身是对的：两片实例表的池上两臂逐个相同（上一节 ② 的第二条历史）。
- raise-no-restore：没有用例让抬 F 那一串在影子账上被拒（被抛弃根的账与映射重读之后都读不出）。
- defer-includes-abandoned：放得过，但我没造出两者判得不同的合法状态：被抛弃时间线上新分配的槽不在现行那一版的账里，共用祖先的槽若在现行账里是「已释放、释放代高于 F」，引用它的那一版（txg ≥ F）还在环里就已走过（推的）。

### ② 造盘面的三处改动合不合法

- `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：见证槽写回成世代 g − 2、内容取 g − 1 那一槽的。真崩溃在那一槽里留的是世代 g − 2 的真内容，它的 tail、实例代号、F 都不高于 g − 1 那一槽；读系统配置的几处要么取各槽的最大值、要么取世代最大的一槽，两种盘面给出同一个结果。按读法等价，没打中（推的，没逐处跑）。
- `acquisition_writes_…` 的「落后一次轮换」（世代 3 拿世代 4 的内容改号）：同一个道理，落后那块盘的最大 tail 与真盘面相同，没打中（推的）。
- `writable_mount_of_a_formatted_pool.rs` 孤记录那一格：用例自己写明盘面真崩溃留不下（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:222`：「造出来的盘面：取号写之后才有记录，真崩溃留不下这一形」）。取号写与记录写之间有屏障，记录持久而取号写没持久只有说谎的设备做得出（上面 ① 的吞写就是这一种）。它钉的「从 txg 2、jsn 2 起」只在这种盘面上成立：真崩溃里取号写持久、记录持久、根没持久时，恢复由记录重建那一版（D23（journal 的角色与格式） 已定项 15），走不到孤记录这一支。按旧正文 Z4 判据算打中，改法二选一：把它改成说谎设备的造法（`WriteIsSwallowed` 吞取号写），或在注释与用例名里写明它测的是「说谎设备留下的盘面」。

### ③ 改钉之后没人守的旧行为

没做，见「限度」。

### ④ 变异表新 6 行

6 行逐行施加在副本上，跑各自点名的测试目标（`logs/mut-row141*.log.lines`）：

| 行 | 点名的目标 | 红 |
|---|---|---|
| 1413 C331 甲 挂载 | a_writable_mount_refuses_… | 3 条里 2 条 |
| 1414 C331 甲 V4 取号 | acquisition_writes_… | 8 条里 2 条 |
| 1415 C393 (b)-从映射现算 | an_abandoned_roots_unreadable_… | 11 条里 5 条 |
| 1416 收口表第 43 行丁-defer | raising_the_floor_into_the_gap_… | 2 条里 1 条 |
| 1417 收口表第 ② 行算进（实现） | an_abandoned_roots_rollback_floor_… | 2 条里 1 条 |
| 1418 收口表第 ② 行算进（checker） | 同上 | 2 条里 1 条 |

每行都红点名的那一条。第 1415 行的替换文让现算一律交 `None`，行为退回的是「读不出就重读、再拒可写」，不是 M1 之前的「只计数、跳过、一个槽都不隔离」；另造的变异 `c393-old-skip`（读不出就跳过）红同样 5 条（`logs/mut-c393-old-skip.log.lines`：「name=c393-old-skip cases=11 failed=5」），所以旧行为照样有用例守着。其余 5 行的替换文退回的就是旧行为。没打中。

## K3　C600：两条出路各自的结局

### 今天的条款怎么写

D23（journal 的角色与格式） 已定项 14 今天写的判法已不是甲：`.claude/kb/decisions/23-journal的角色与格式.md:392`：「这里的实现还是 2026-09-28 取甲的那一版，欠 C331（择根倒挂压过已确认的写）」。它指过去的 R2 在 D22（单元原子性怎么合成） 已定项 8：`.claude/kb/decisions/22-单元原子性怎么合成.md:178`：「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过」，同一行还写了 R2 认下的代价：「全池读回一个值的纯形丢写 120 格」。

所以「C600 那种拒算不算允许的结局」按两份文字答案不同：按 2026-09-28 甲的写法是允许的（缺一槽就判不出、按真、重读仍缺就拒）；按今天 D23 指向的 R2，C600 那块盘没有报读错（那一槽读得回，只是全 0、自证不过），不是判不出，c_见证 取自证过的那几槽、等于 0，判据为假，该挂上——今天代码的拒不是条款的结局，是代码落后于条款（C331 还欠着）。下面两条出路的「改挂载的判法」我按 R2 起草（`mutants.json` 的 `r2`），另起一个「全 0 的槽当没写过」的草图（`b-zero`）对照；两份草图都只在我的副本上量过，被攻过零轮。

### 造的几段历史

- C600 一族：只做过 mkfs 起步，mkfs 写槽 0 或槽 1 的那一次写被设备吞掉（两块盘各两次），再可写挂载、覆盖写、可写挂载。同上面 Z2 ①。
- H4 一族（`opus_probe_k3_outlets.rs`）：内存盘上第一个文件之后覆盖写两次（B 落在 (1,4)，C 落在 (1,5)，两次发布都正常返回、系统配置轮换都写完），进程退出；下一次可写挂载时 C 的根槽、C 的数据单元两份读回全 0，另按格让每块盘见证 C 的那一槽读回全 0 或报读错，整次挂载（含重读那一遍）都这样，挂载之后照读。读回全 0 是 harness 自己用的暂时读错形（`UnreadableRangeReadBack::Zeros`）。

结果（`logs/k3-h4-base.log.lines`、`logs/k3-h4-r2.log.lines`、`logs/k3-bzero.log.lines`、`logs/c600-mkfs.log.lines`、`logs/c600-mkfs-r2.log.lines`，下面是对这几份跑的 `grep`/`sed` 的原样输出，N配置拒 是 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion` 的缩写）：

```
== k3-h4-base.log.lines
name=opus-k3-h4 zero_witness_on=[0, 1] REFUSED N配置拒 … comparison: SomeSystemConfigurationSlotUnreadOrUnverified
name=opus-k3-h4 zero_witness_on=[0] REFUSED N配置拒 … comparison: SomeSystemConfigurationSlotUnreadOrUnverified
name=opus-k3-h4 zero_witness_on=[] REFUSED N配置拒 … comparison: AgainstTheSelectedVersionsLastRecord
name=opus-k3-h4 witness_modes=[(0, DeviceError), (1, DeviceError)] REFUSED N配置拒 … comparison: SomeSystemConfigurationSlotUnreadOrUnverified
name=opus-k3-h4 witness_modes=[(0, DeviceError)] REFUSED N配置拒 … comparison: SomeSystemConfigurationSlotUnreadOrUnverified
name=opus-k3-h4 witness_modes=[(0, ReadsBackZeros), (1, DeviceError)] REFUSED N配置拒 … comparison: SomeSystemConfigurationSlotUnreadOrUnverified
== k3-h4-r2.log.lines
name=opus-k3-h4 zero_witness_on=[0, 1] MOUNTED newest_before=(1,5) effective_root=(1,4) new_instance=2
name=opus-k3-h4 zero_witness_on=[0] REFUSED N配置拒 … comparison: AgainstTheSelectedVersionsLastRecord
name=opus-k3-h4 zero_witness_on=[] REFUSED N配置拒 … comparison: AgainstTheSelectedVersionsLastRecord
name=opus-k3-h4 witness_modes=[(0, DeviceError), (1, DeviceError)] REFUSED N配置拒 … comparison: SomeSystemConfigurationSlotUnreadOrUnverified
name=opus-k3-h4 witness_modes=[(0, DeviceError)] REFUSED N配置拒 … comparison: AgainstTheSelectedVersionsLastRecord
name=opus-k3-h4 witness_modes=[(0, ReadsBackZeros), (1, DeviceError)] REFUSED N配置拒 … comparison: SomeSystemConfigurationSlotUnreadOrUnverified
== k3-bzero.log.lines
name=opus-k3-h4 zero_witness_on=[0, 1] MOUNTED newest_before=(1,5) effective_root=(1,4) new_instance=2
name=opus-k3-h4 zero_witness_on=[0] REFUSED N配置拒 … comparison: AgainstTheSelectedVersionsLastRecord
name=opus-k3-h4 zero_witness_on=[] REFUSED N配置拒 … comparison: AgainstTheSelectedVersionsLastRecord
== c600-mkfs-r2
      8 ending=Completed
== c600-mkfs (甲)
      4 ending=Completed
      4 ending=NewFinding
== k3-bzero c600
      8 ending=Completed
```

R2 草图与 b-zero 草图下挂上的那一格，新实例写的行是 (实例 1, 所选 txg 4)，C (1,5) 按新实例表判成被抛弃：一次已经正常返回、系统配置也见证过的发布被丢掉（`k3-h4-r2.log.lines` 那一行的 `rows=` 字段）。

### 两条出路并排

| 历史 | 今天（甲） | 出路一：模型认这种拒，代码不动 | 出路二：挂载改判（R2 草图） | 对照：b-zero 草图 |
|---|---|---|---|---|
| C600 一族（mkfs 写槽被吞，8 段） | 4 段拒、4 段挂上；拒的那 4 段永远只读（空池） | 同左；大档回到 0 条新发现 | 8 段全挂上 | 8 段全挂上 |
| 吞取号写、轮换写（mkfs 起步 4 段、第一个文件起步 24 段） | 全挂上 | 同左 | mkfs 起步那 4 段挂上；另外 24 段没在草图下跑 | 同 R2 |
| H4，两块盘的见证槽都读回全 0 | 拒 | 同左 | **挂上，丢 C**（R2 自己认的纯形丢写） | **挂上，丢 C** |
| H4，一块盘的见证槽读回全 0 | 拒 | 同左 | 拒（另一块盘的槽见证 C） | 拒 |
| H4，见证槽都不坏 | 拒 | 同左 | 拒 | 拒 |
| H4，两块盘的见证槽都报读错 | 拒 | 同左 | 拒（判不出） | 没跑 |
| H4，一块盘报读错 | 拒 | 同左 | 拒（另一块盘两槽都自证过、见证 C） | 没跑 |
| H4，一块读回全 0、一块报读错 | 拒 | 同左 | 拒（判不出） | 没跑 |
| 撕裂轮换（一槽内容坏、设备不报错） | 永远只读（条款 L1 的代价，用例钉着） | 同左 | 挂上（钉甲的用例里 2 条翻） | 坏的内容不是全 0 时仍拒（推的） |
| 一槽持续报读错、没有一块盘两槽都自证过 | 永远只读 | 同左 | 永远只读（条款认） | 永远只读 |
| 现有钉甲的用例（`g_wit.list` 98 条） | 全绿 | 全绿 | 11 条翻（`logs/mut-r2-wit.log.lines`）：钉「缺一槽就拒」的 5 条按 R2 本来该翻（撕裂、丢一槽之后挂上或取号）；空盘、盘 1 每次读都报错那 2 条仍拒、拒因成员变了；`acquisition_refuses_a_device_whose_witness_slots_…` 的 3 条是我的草图在取号一侧把「某块盘一份都读不出就拒」那一条可见性核一起删了造成的，不是 R2 的结局；`writable_mount_of_a_formatted_pool` 那 1 条（取号之前的暂时读错）没细看 | 没跑 |

R2 草图取号一侧少了可见性核那个缺口，不影响 H4 与 C600 两族的结局：这两族里每块盘都至少有一槽自证过。

各自会不会把别的形压成永远只读或漏判倒挂：

- 出路一不新增倒挂，也不新增永远只读；它把 C600 一族、撕裂轮换、一槽持续坏三种永远只读都留着。要认得窄：32 + 8 段单次吞写里只有吞 mkfs 写槽的那几段拒，模型若写成「吞过系统配置写就许拒」，以后轮换写被吞也开始拒的回归就看不见了。它与今天 D23 的文字不一致（文字已是 R2）。
- 出路二按 R2 改：C600 一族与撕裂轮换不再永远只读；永远只读只剩「有设备报读错、又没有一块盘两槽都自证过」那一形。代价是 H4 两块盘都读回全 0 那一格从「拒」变成「挂上、丢一次已确认的发布」：这是 R2 写明认下的纯形丢写，这里给出一段具体历史（要同一次挂载里 C 的根槽、C 的两份数据单元、两块盘的见证槽共 5 处同时读回全 0，重读那一遍也一样）。b-zero 在这一格上与 R2 同一个结局，另外把「全 0」与「内容坏」分开判，没有 R2 好的地方（推的），不推荐。

判别子：挂载那一刻 C600（全 0 槽、另一块盘两槽都是 mkfs 那一份、没有读错）与 H4 两块盘都读回全 0（全 0 槽、没有读错）看得见的差别是「另一块盘两槽都自证过，且两槽内容相同、tail 为 0」。只靠「有没有读错」（R2）分不开这两格在丢写上的差别；一个更窄的改法是「全池自证过的槽都是 mkfs 写的那一份（tail 0、实例代号 0）时，缺的那一槽至多是一次取号写，判据为假」，它让 C600 挂上、不碰 H4。只在纸上推过，没实现没跑，被攻过零轮。

## K4　写零折叠块内乱序

切法：判器 `/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/device_log.rs:282`：「let chunk_start = fill_start + chunk_index * ZERO_FILL_CHUNK_BYTES;」，块尾取 `min(块起点 + 块长, 段尾)`；后端 `/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/block_device.rs:364`：「let this_chunk = (length - written).min(length_of(zeros));」，暂存区长 `min(段长, 块长)`。两边都从段的起点起每 4 MiB 一块、最后一块到段尾，逐字一致。

模型：`fold_model.rs` 把判器里 `fold_declared_zero_fills`、`tile_of_zeros_starting_at`、`zero_fill_chunk_containing`、`entries_tiling_one_zero_fill` 四个函数用 `sed` 原样抽出来（冻结副本第 238–353 行），只换了块长（16，逻辑与块长无关）与全 0 的哈希。随机 20 万轮：段起点 0–39、段长 1–70，按后端切块，每块切成 1–4 段、块内随机次序、块间按序。

```
name=k4-spanning-ahead folded_into_one=false
name=k4-fold-model rounds=200000 chunk=16 p1_legal_not_folded=0 p2_drop_still_folded=0 p3_cross_chunk_swap_folded=0/154085 p4_extra_piece_equals_expected=0 p5_two_fills_not_two_events=0 in_order_merged_across_chunks_not_folded=0
```

（`logs/k4-model.log.lines`。）逐问：

- 块内乱序之外丢一块：丢任意一段，折不出整段（p2 = 0）。丢掉的那一段只可能由紧接着的、同一处的另一条全 0 写补上；那条写在程序一侧是另一件事，折完之后两边件数差一，逐项比对照样红，只是报的位置挪到了后面那一件（推的）。
- 按序跨块那条旧行为与新规则打不打架：按序的跨块段照样折（最后一项 = 0）；块与块之间对调一律不折（p3 = 0）；相邻两段清零各自折成一件（p5 = 0）。唯一不折的合法到达形是页缓存回写把前一块的后半与后一块的前半合成一条、先于前一块前半到（`k4-spanning-ahead` 那一行）。只有不经直接 I/O 写零时块与块之间才不保序；虚机档的写零走不走页缓存那一臂我没查到，结局是误红不是放过（推的）。
- 长度 0 之外不终止的输入：解析只在扇区数大于 0 时出写事件（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/device_log.rs:124`：「if sector_count > 0 {」），所以没有长度 0 的写；内层每一轮吃掉一条或返回，外层每一轮下标至少加一。没找到。

没打中。

## K5　对接上游第三轮之后

- 窄一断言：断言的就是「照 journal 恢复基镜像落到的根 = 基镜像里（txg，实例）最大的根」，与定案那一句同义。认「基镜像里的根」的写法是看每个写过的 512 字节扇区开头是不是根 magic（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:434`：「bytes.len() >= ROOT_RECORD_IDENTITY_END && bytes[..ROOT_MAGIC.len()] == ROOT_MAGIC」），magic 只有 4 字节（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-core/src/root_record.rs:11`：「pub const ROOT_MAGIC: [u8; 4] = *b"SFSR";」），不验自证。一个用户数据扇区恰好以 `SFSR` 起头、后面 12 字节又读成更大的（txg，实例）时会误拒：随机内容每扇区约 2^-32，登记流里用的几种确定性内容（步长 7、31 的取模序列）造不出这四个字节（推的，没跑登记流：那在 checker 档）。没打中。
- 白名单与逐个引入者判：闭包按模块引用收、停在 `history.rs`，引它的每一个文件都记成引入者（不是只记第一个）；库模块按「`crates/<包>/src/` 下」认（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:940`：「parts.next() == Some("crates") && parts.next().is_some() && parts.next() == Some("src")」）。经一份没登记的库模块间接引入：那份库模块自己就是引入者，判 `UnregisteredLibraryModule`；经一份登记给别的用例的库模块：登记按（用例，库模块）配对，照样拦。能放过去的只有 `tests/` 下共用的搭建模块直接引（判 `TestTargetFile`），这是定案写明放行的那一类。没打中；判它的用例在 checker 档，没跑，只读代码。
- 去空白与写 0：数值变量四个入口走两份解析，两份都先去前后空白（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/numeric_environment_variable.rs:51`：「match text.trim().parse::<Number>() {」，`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/history.rs:4735`：「match text.trim().parse::<Number>() {」）；种子数、步数、注入次数、短段写数四类计数都取非零类型，写 0 都拒。口径一致。不一致的在取值类变量：比重、checker、盘宽、空间准入按原文逐字比、不去空白（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/src/history.rs:4778`：「.find(|choice| environment_value_of(*choice) == value)」），崩溃注入的短段写数 `none` 也是原文比（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs:3099`：「let draw = if short_segment_writes == SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE {」），`" 11"` 照收而 `" none"`、`" broad"` 拒。拒的一侧，不会悄悄跑错档；读代码，没跑。
- 抬 F 搭建拆分之后的池几何：harness 档的使用者仍经 `start_after_the_first_file(device_width, …)`，参数、盘宽都取 `HistoryDeviceWidth` 那一档，与拆分前同一个来源；抄的那份 384 槽常数（384、815、6 MiB）只给 checker 档那一条用，起 384 槽那一档时逐项核参数、盘宽与几何三样（`/tmp/claude-1000/m3-leftover-code-r1/tree/crates/singlefs-harness/tests/common_admission/mod.rs:77`：「"common_admission_pool 抄的 384 槽那一档几何（槽距、环长、每区根槽数）与 history.rs 的 UnitAreaOf384Slots 相同"」），`FixedGeometry` 只有这三个字段。没打中。

## 打中之后的四句（Z4 的四处与 Z2 ① / K3 那一形）

| 打中 | 分辨臂吗 | 判别子当时看得到吗 | 满足的字面 | 改法在这几格上还中吗 |
|---|---|---|---|---|
| slot1-only 放过 286 条 | 分辨：对的实现在槽 0 坏时拒，错的照挂 | 看得到：把偶数世代那一槽（槽 0）写坏就分得开 | `research/prompts/_m2-five-decisions-code-r1-body.md:63`：「Z4 ① 写出一种错实现、新用例与改钉用例全绿 ⇒ 打中，补用例（或收紧断言）并进下一批。」 | 补一条坏槽 0 的挂载用例、一条坏槽 0 的取号用例，这个变异就红（推的） |
| map-first-instance-page-only 放过 69 条 | 分辨：两片实例表的池上两臂隔离数不同 | 看得到：池要先长到两片 | 同上 | 照 `instance_table_page_full.rs` 的连取 375 次号把池推到两片，再跑一遍 C393 那几臂（推的） |
| raise-no-restore 放过 69 条 | 分辨：拒之后分配器上多出的隔离位 | 看得到：比拒前拒后的 `isolated_slots_per_device` 一类计数 | 同上 | 挂着时让一条被抛弃根的账与映射都读不出、推一次抬 F，断言拒且分配器与拒前相同（推的） |
| defer-includes-abandoned 放过 56 条 | 我没造出两者判得不同的合法状态 | — | 同上 | 可能是等价变异；不补也行，补的话要先造出那一格（推的） |
| 孤记录那一格的盘面 | 不涉臂 | 用例自己写明 | `research/prompts/_m2-five-decisions-code-r1-body.md:64`：「Z4 ② 判出某处盘面不是真崩溃留得下的、而它钉住的行为只在那种盘面上成立 ⇒ 打中，那条用例改造法。」 | 改成吞取号写的说谎设备造法，或写明它测的是说谎设备留下的盘面（推的） |
| C600 一族（Z2 ① 永远只读的又一成因） | 分辨 K3 两条出路 | 「全 0、没报读错」看得到；但它与 H4 两盘都读回全 0 在这一点上相同，只靠它分不开 | 正文 K3 那一格本身；`research/prompts/_m3-leftover-code-r1-body.md:52`：「打中 ⇒ 主 agent 现查坐实，改法进下一批」 | 出路一不改结局；R2 草图在 C600 上 8/8 挂上（量过），在 H4 两盘全 0 上丢 C（量过）；「全池只剩 mkfs 那一份」的窄改法只推过 |

## 这条腿提的改法（都只在我的模型上量过或只推过，被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 补坏槽 0（偶数世代）的挂载与取号用例 | Z4 ① slot1-only | 推的：没写用例；变异放过 286 条是量过的 |
| 补两片实例表上的 C393 用例 | Z4 ① map-first-instance-page-only | 推的；两片实例表上两臂逐个相同是量过的（`logs/run-probe3.log.lines`） |
| 补抬 F 一串影子账被拒的用例 | Z4 ① raise-no-restore | 推的 |
| 孤记录用例改造法或改名 | Z4 ② | 推的 |
| 现算那一臂在映射根是空指针时交树表 0 条那几样落点 | Z2 ② 树表 0 条被抛弃根树表读不出时的永远只读 | 推的；「树表 0 条 ⟺ 映射根是空指针」在 152 条根上逐条成立是量过的 |
| 挂载与取号改判成 R2 | K3 出路二 | 草图量过（C600 8/8 挂上、H4 六格里一格丢写、钉甲的 98 条翻 11 条），草图取号一侧少了可见性核，要补 |
| 全池自证过的槽都是 mkfs 那一份时判据为假 | K3 C600 一族，不碰 H4 | 推的，没实现 |

## 没打中的形状

- Z2 ② 两臂比对：152 条根（带文件 107、树表 0 条 45）；形状覆盖两层中央映射树（每盘 354 个落点以上）、2000 个 inode 的多片叶容器、unlink 之后的墓碑树与谱系树、两片实例表、三个实例。没覆盖：隔离的单元（释放时核对不上那一类）、多层分配记录树、多层记账树（产品容量下要 80 块盘）。
- Z2 ③ 次序：读代码；变异只证明没用例守。
- Z2 ④：两个回收变异，56 条用例。
- Z2 ⑤：只推，没造。
- Z4 ①：上面表里 12 个变异，用例 4 张表共 8 + 17 + 6 + 6 个目标。
- Z4 ④：6 行逐行施加，各跑点名的目标；外加旧行为的「跳过」变异。
- K3：C600 一族 32 + 8 段单次吞写（两种起步、两种用户动作、两块盘、n 扫到 8 或 4）；H4 六格；三套判法（甲、R2 草图、b-zero 草图）。吞的只扫了系统配置那一段的写，吞根槽写、单元写没扫。
- K4：单文件模型 20 万轮，五条性质加一个页缓存形；没跑判器自己的单元测试（checker 档）。
- K5：只读代码。

## 这条腿自己的限度

- 「没打中」都是一次抽样，照 `.claude/rules/three-way-inference.md` 要再抽一次才记。
- 变异只跑了非 `#[ignore]` 的用例；`rollback_by_a_forward_publish.rs` 22 条、`reuse_after_raising_the_floor.rs` 13 条、`fsync_drop_…` 1 条、`admission_raises_the_floor_before_refusing.rs` 1 条慢档没跑，四处「放过」可能被这些慢档抓到。
- Z4 ③（改钉的旧用例丢了哪些覆盖）没做：要对着 9 月 28 日那一版逐条比改钉前后的断言，时间放在了 K2 的真跑上。
- K3 的 R2 与 b-zero 都是我的草图，不是条款作者的实现；R2 草图在取号一侧把「某块盘一份都读不出就拒」也删了，3 条用例翻是草图造成的。H4 要同一次挂载里 5 处同时读回全 0，我没估它的概率。
- 副本上量的数都要在入库装置上重做才能引。
- checker 档一条没跑：K4 的判器单元测试、K5 的白名单用例与窄一断言所在的登记流都在那里。

## 要主 agent 跑的命令（可选，都没跑）

- 四处「放过」的变异连同慢档再跑一遍（每个变异估 20–40 分钟，放后台）：工作目录 `$OPUS_DIR/work`（先跑 `setup.sh`），环境 `CARGO_TARGET_DIR=$OPUS_DIR/target`，`python3 <模型目录>/apply.py <变异名> && bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 6 cargo test --offline -p singlefs-harness --test rollback_by_a_forward_publish --test reuse_after_raising_the_floor --test admission_raises_the_floor_before_refusing --test fsync_drop_and_devices_without_the_selected_version -- --include-ignored; python3 <模型目录>/apply.py --restore`，变异名取 `slot1-only-both`、`map-first-instance-page-only`、`raise-no-restore`、`defer-includes-abandoned`。看 `test result` 行：有 `failed` 不为 0 的，那个变异就不算「放过」。

## 没做什么

- 没判 K1、没替正推腿核条款对实现；K3 不替用户选。
- 没跑 checker 档、重型测试、QEMU；没跑 `#[ignore]` 的慢档。
- 没改冻结副本与主工作区；改动都在 `/tmp/claude-1000/m3-leftover-code-r1-opus/work`。
- Z4 ③ 没做；Z2 ⑤ 只推没造；K5 只读代码。
- 吞写只扫了系统配置那一段；R2 草图没在「吞取号写、轮换写」那 24 段上跑。

## 草稿目录

`/tmp/claude-1000/m3-leftover-code-r1-opus/`：交回前删掉了仓副本 `work/`、编译目录 `target/`、`clean/` 与 K4 模型的二进制；留着各次跑的原始日志与 `mut/out-*/`（每个变异逐条的结局与失败输出），模型目录 `logs/` 里是从它们 `grep` 出来的结果行。要重跑照「复跑」一节用 `setup.sh` 重建。
