# m3-unlink-code-r3 云端攻方（Opus）报告（2026-10-03）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 N8 V1 V2 V3 V4 V5 V6 G1 G2 G3 G4 H1 H2 H3 H4 Q2 MU1 MU2 -->

攻击面：正文第三节 K2，重点 N1、N2、N7。原型是冻结副本拷贝里新加的一个 bin crate `attack-r3-probe`（依赖 core、harness、checker 与 checker 档的库，只调它 `crash.rs` 的枚举函数；不是 checker 档包、名字不带 layer0），在三份拷贝上编：丙1 原样（下称 Imm）、`mount.rs` 第 2259 行改回 `HeldUntilFloorTakesEffect`（下称 Held）、我提的零轮改法 G-a（下称 GA）。只跑 harness 档与这个原型；checker 档一条没跑。副本上的数不是入库装置上的数。线程上限开工是 15，跑第一次编译之前收到主 agent 改成 10，此后每条命令都是 `capped.sh 10`。

## 复跑

模型目录 `research/prompts/m3-unlink-code-r3-opus-model/`。整套复跑：`ATTACK_DIR=<空草稿目录> bash research/prompts/m3-unlink-code-r3-opus-model/rerun.sh`（它把冻结副本拷成三份、打 `patches/` 里的补丁、放进 `probe/`、编三份，再按本报告的批次逐批跑；每批前台 1–4 分钟）。`rerun.sh` 是把这一轮实际跑过的命令改写成带目录变量的一份，整份没有从头跑过一遍（见限度）。`scripts/` 里的几份脚本与草稿目录里实际跑的那几份只差两处：草稿目录与仓根写成了 `ATTACK_DIR`、`SINGLEFS_REPO` 两个变量。

内存包装：上限 1G–4G，同会话另一条腿也在跑，设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 与 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`。

```
7a4a6e32468d81d5cb9ab27a0584bb42c48b2aed935bb715c922d692004c5afc  out/enum-h2-held.log
1626930b5b9bcfb0e741bbb3daee811268834d3ed52b0c66db2a90b2051b1605  out/enum-h2-imm.log
16d4fb25227543904e0d35acc84ad4f13c383128e03ac6fed7438dc32bafeee9  out/enum-h3-held.log
7eb7ef8d6cd41861a4dfb41a1930bf50a5c97cf0a84adf8d62854d14876faf51  out/enum-h3-imm.log
2a6d1ec4b8900ac167c5953d00fd5a78d48e56ee35b54adbae4d0f24109e92fa  out/enum-x-imm.log
2bc60d44b7f81eb501c70468ce586323bfefe0036ae30f20be2b13c6338760a8  out/h2-held.log
06fdeafd0fe01d9c244da9898452520d888810a4b3f6308d9ed4810144bbee40  out/h2-imm.log
e3a48162bc78b906a320c50d89d7576c366712ae9b895be6e0c6604a1fe8508b  out/h4-held.log
abea49a48f0d67dea7f06ff0c000e0ecbdbbbb62788ca38289f8e00400243b1b  out/h4-imm.log
0b2ae932c065add469935e97939d009f91fc237db6c54018d91c0b108199deaa  out/hits-compare.txt
a442171934aa3ad24d838eaeb64bc9a8f4bc9617a6cee0935509de42efd313e0  out/hit-shapes.txt
6d217ea08571cb0541f0fcc3d28cd8cf0f2e54a189fe6e674fa7f466dc0c2aa4  out/imm-intersections.txt
639928b79963f93c26b43e14669b937f72d967bc6d585f5934e34d49d80e3b22  out/n2-narrow-summary.txt
c82a4379a7ef988589aec3da12b1aaf88b97c81c89714d961a0249fe31c7fdc0  out/n2-remount-240.log
76ad6c53484036f6c2c7d0f4a7fb3a626dae50712972c6ca52ee1b183884f653  out/n2-remount-summary.txt
61c935e8aeb83b04d5011ffd2e4d2e98355b099f5669087740262fa99313770e  out/n5-narrow.log
e23fc71adb37232199a0a88a184c14af4775fa3731cec593cd047f8535cf16f7  out/n7-4g.log
4e3906427727fbaef90f6cf57ecc04d5bd70ea1a39e4a97d08ed0d6d2353124f  out/sweeps-summary.txt
2a333265a576578b428429522db51870c1583f041cf755727358d77fd71ab624  out/v2-tests.log
0ad88817f39457972974932443bd94789b61ad54ff51dfc9fbbbb6021d98f83b  out/x-held-detail.log
e82a54a9912f3d16dacd37eb53467b6b862cef38c690596ce31883662ed12066  out/x-imm-detail.log
10cb87ba6215e65ca22b517629cfcad2e2dcfe15c7ebb303e6d37acb5ef60016  patches/ga-reuse-only-below-every-newest-root.patch
eb933ea23e918c56ab58c3b618620b9cf2a3dd01ec5c73c3bdd1cf5aa686a3a6  patches/held-before-bing1.patch
d052843a2a303bda06eb3107f19053904c2cd15bab68248b345c962331771874  patches/workspace-member.patch
9e8077a06491981a3b551efcf3e5f9607ba48def3d01dc69a176ddc1a3b68401  probe/Cargo.toml
87b597d714cadd00adc3319416503e4b05b6d068bd924eebbc17005988223c6e  probe/main.rs
3b66766e87031a362b945aabc058cb11df7d2f2cdaa66f88f427b85dd26beba9  rerun.sh
8ea1dc1a6815f6b9ff8355be8d3f1bffc31fbda81f507e4f66e1911a2e7e97b9  scripts/build-held.sh
65636ba08b16fe0adbc0238a348337ddb8535899bf1ce0b391fe67d7e4cadef1  scripts/build.sh
cafe3836c3340d242869a2551b98e40f9e47c766b722a594c3d1c8e5ee1655e4  scripts/build-v2.sh
0d59e91b4d9929ef3f644bff4ef68ce65d5742c5db479a0b7079be7bd2f391a8  scripts/compare.py
a21e422a8c7b0e1e22af97e5d839bbc2661b3db8b42bdf8b4e838509d28d6d75  scripts/hit-batch.sh
4fb2cfc11848296c2bc15999d10ddb1f8a2525aa992bd8b5bba9b8fa02194069  scripts/n2-batch.sh
3b0a25f7b8f5be7f9ac15e92bb29f9e213e3c12a7b5b9d02674db9414b9de1d4  scripts/n2sum.sh
f3b187bcdb4dc3bd220b351ca50f7ebb19c63795cb91dc44512685e9116a3e80  scripts/run.sh
4bb4a85a2b371520b992cd866d05e2d898ea0b4f1f0fa53bf673c28f72d0bb43  scripts/summarize.sh
77e590062b869075994f192557002eac2da6bada12eb4dc6dda49c7229df3cbf  scripts/sweep-batch.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| N1 (a) X 第 1404 行那一份盘面（256 槽、覆盖写 2、14 单元、崩在第 1 批之后），丢盘 0、只剩盘 1 | 交集为空（量过） | txg 19 引用的槽 ∩ 卸载写的槽：0 个；层 0 枚举域 8223 个状态逐个算，0 | 第一节 1.1 |
| N1 (b) 同一刀上改之前那一版 | 那一刀摆不出来（量过） | Held 在这份盘面上卸载在预演里就拒、一个写都不发；它停在卸载之前的盘面，丢盘 0 时池级 checker 红 7 条（含那六条） | 第一节 1.2，原样抄在那里 |
| N1 放开扫：丙1 写掉仍可能被择中的根引用的槽 | **打中（量过）** | 窄盘网格 1900 个起点里 174 个、卸载—再挂循环 186 个里 13 个、4 GiB 盘循环 400 个里 1 个有交集；丢掉住着现行那一版的根的那块盘之后，恢复择到另一块盘上最新的根，那条根引用的单元已被卸载的空发布写掉；Held 与 GA 在同样的起点上交集都是 0 | 第一节 1.3 |
| N1 放开扫：读不读得到被写掉的单元 | **打中（量过）** | 丙1 下剩下那块盘只读挂载失败（`MappingStillUnreadable`）：窄盘 37 对（起点 × 丢的盘），4 GiB 盘上活文件打不开 1 对；同一刀 Held 在 672 个可比刀上全读得出；层 0 枚举域里 h2 那一份 Imm 6158 / 8223 个丢盘 0 的状态读不出，Held 0 | 第一节 1.4 |
| N1 判据字面（第五节那一条） | 字面判「单盘镜像本身的事」，但判别子看不见（见第一节 1.6） | 交集非空的 2098 个可比刀上，Held 同一刀红的条数与 Imm 相同的 1641 个、不同的 457 个（Imm 多红 I-3.11）；那六条在单盘镜像上卸载之前就红，名字层面盖不住丙1 的差别 | 第一节 1.5、1.6 |
| N1 两块盘都在 | 没打中（量过） | 前缀刀 3 族 3 个实现全部 0 交集；层 0 枚举 3 份 × 两个实现，池级 checker 0 违例 | 第一节 1.7 |
| N1 卸载—再挂循环 | **打中（量过）** | 同上第三行：做完一次卸载、再挂、覆盖写之后的第二次卸载照样写掉另一块盘上最新的根引用的槽，4 GiB 盘上那一份是活文件的 extent 叶与 inode 叶 | 第一节 1.3 |
| N2 写账 | 没打中（量过，一次抽样） | 74132 个注错点（窄盘 72372 + 只崩了再挂 1760），每个带账的错按它那一处的读法都满足「落下的写 − 取号两次 = 写账合计」；份数对不上的全是 r2 H4 那一形（根槽已落、之后的系统配置轮换写失败） | 第二节 |
| N2 两处口径 | 量到、不分辨这一批 | 写行之后那一处与续做那一处推成过抬 F 再报错时，两种读法给不同的合计，只有对的那一处的读法对得上；错值本身不带它出自哪一处（X 报告 Q2 自报过） | 第二节 |
| N2 取界（准入与续做都推） | 没打中（量过） | harness 里那份 `FloorRaisesInTheMountWrites::of_the_mount_output` 在 1550 个起点上与真值逐个相等，其中 21 个两边都推 | 第二节 |
| N7 映射树上界会不会低于真节点数 | 没打中（量过） | 300 段历史 5497 次发布，按真条目数算的两种上界（沿来路带 / 上一版真节点数）都不低于真节点数 | 第三节 |
| N7 只涨不落的上界 | **打中松紧（量过）**，判别力的后果是推的 | 沿来路带的上界最高到真节点数的 333 倍；300 段里 8 段，单这一项按每次发布加进占槽上界的槽数累计就越过 4 GiB 盘「允许撞单元区墙」的门槛 | 第三节 |
| N7 分批与实现 | 没打中（读代码） | 模型与实现同一个式子（每批 144、从尾部、剩下不多于 144 时是最后一批） | 第三节 |
| N3–N6、N8 | 没攻 | 时间给了 N1、N2、N7 | 「这条腿自己的限度」 |

## 一、N1 丙1 的崩溃安全

被判的那一句：`/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-core/src/mount.rs:2259`「RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,」，它的理由写在 `/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-core/src/mount.rs:2255`「/// 引用它们的那几条根都已低于 F_生效。挂载续做推满仍放不下之后卸载靠这一样做成（槽被续做吃光，扣着的话这一串在预演里就取不到落点）。」。回收门槛是 `/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-core/src/mount.rs:1278`「oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))」：卸载抬到现行那一版的 txg C，释放代 ≤ C 的槽全部回收、当场可发（`/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-core/src/mount.rs:2765`「entry.reuse_of_the_reclaimed_slots(),」）。释放代为 C、C − 1 的槽，正是 C − 1、C − 2 那几条根引用的。

而恢复择根不看 F：`/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-core/src/recovery.rs:1260`「/// 三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的（D22（单元原子性怎么合成） 已定项 7 择新；」。第一版区域归属 0 / 1 / 0，根轮流落两块盘；丢掉住着 C 的那块盘，剩下那块盘上最新的根是 C − 1 或 C − 2，恢复就择它。

### 造法（用户决定的几步全部放开）

- 起点四族：窄盘（不补足环窗口、准入关着起池；单元区 176–352 槽步长 16、unlink 之前覆盖写 0–10 次、文件 1/2/4/8/14 个单元、意图之后崩在第 0/1/3/7/13 批之后，重开可写挂载，准入判着、批大小 1）；窄盘循环（窄盘起点卸载做完、再挂、覆盖写 0/1/2/4 次，判第二次卸载）；普通盘（384/256/240 槽与 4 GiB 四档，覆盖写 0–15 次、崩了再挂 0–2 次、每次之后覆盖写 0/1/3 次）；普通盘循环（同上再卸载一次、再挂、覆盖写 0/1/2/4/8 次）。
- 每个起点在会话上录一次卸载（不注错），切段；崩溃状态两种取：前缀刀（录制流前 n 个写持久，n = 0 … 全长，全部起点都跑），层 0 枚举域（`enumerate_layer0_selecting_versions_observing_each_state`，每段都展开，含原地覆写的撕裂态；只对挑出来的 5 份跑，见 1.4）。
- 每个状态按三种丢盘各判一遍：两块都在、丢盘 0、丢盘 1。判：恢复择到哪条根 R、系统配置里的 F、这个状态里持久了的单元写罩住的槽 ∩「R 引用的槽」、只读挂载再把每个活文件从头读一遍的结局、池级 checker 红哪几条；两块都在的前缀刀上再可写挂载一次，挂载之后再跑 checker。
- 「R 引用的槽」三条读法，互不共用代码：读法三（判交集用）是卸载之前分配器的记录里那个槽「已释放、释放代 > R」且卸载之前盘上那个槽的单元头诞生代号 ≤ R；读法二是只留 R 一条根的镜像上恢复加只读挂载加整读实际读过的单元区槽；读法一是同一份镜像上池级 checker 读过的单元区槽（它还读已释放记录罩的槽，偏大，只作参考）。对照：每个 (丢盘, R) 上读法二读到的槽有多少落在读法三认的全集里，记在各日志的 CONTROL 行。

### 1.1 (a) X 那一份盘面上的交集：空

前缀刀：X 那一刀（先写系统配置 2 个写 + 第一次空发布 12 个单元写）是 n = 14，`research/prompts/m3-unlink-code-r3-opus-model/out/x-imm-detail.log:72`「CUT n=14 roots_landed=0 next=journal_record lost=dev0 chosen=Some(19) sc_floor=Some(21) inter_records=0 inter_reads=0 inter_checker_reads=3 whole_read=same checker=I-1.1+I-2.1+I-4.8+I-7.2+I-7.4+I-7.6 remount=-」。读法三的交集 0、读法二 0；读法一的 3 个是 checker 读的已释放记录罩的槽，不是 txg 19 引用的。层 0 枚举域（8223 个状态）上丢盘 0、择到 txg 19 的 4107 个状态交集全是 0：`research/prompts/m3-unlink-code-r3-opus-model/out/enum-x-imm.log:214`「ENUM states=8223 observed=8223 violations=8223 failed_states=0 checker_violated={}」（`violations` 是枚举器自己的版本 oracle：这份盘面上文件删除中、版本表的内容对不上，与这里无关；`checker_violated={}` 是两块盘都在时池级 checker 0 违例）。

交集的槽号清单：空，计数 0。

### 1.2 (b) 改之前那一版在同一刀上：摆不出那一刀

Held 在 X 那一份盘面上卸载在预演里被拒、一个写都不发：`research/prompts/m3-unlink-code-r3-opus-model/out/x-held-detail.log:2`「unmount=Err RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { refused_publish_in_the_sequence: 2, publishes_in_the_sequence: 2, cause: PlacementRefused」。它留下的是卸载之前的盘面；那份盘面丢盘 0、只剩盘 1 时池级 checker 原样：

- `research/prompts/m3-unlink-code-r3-opus-model/out/x-held-detail.log:4`「CUT n=0 roots_landed=0 next=end lost=dev0 chosen=Some(19) sc_floor=Some(0) inter_records=0 inter_reads=0 inter_checker_reads=0 whole_read=same checker=I-1.1+I-2.1+I-3.1+I-4.8+I-7.2+I-7.4+I-7.6 remount=-」
- 七条的原文在同一份日志第 6–12 行（`DETAIL n=0 lost=Some(0)` 开头），例：`research/prompts/m3-unlink-code-r3-opus-model/out/x-held-detail.log:7`「DETAIL n=0 lost=Some(0) I-2.1: 实例表单元 在盘 0 槽 1600 的那一份与位置条目里的校验和对不上」、`research/prompts/m3-unlink-code-r3-opus-model/out/x-held-detail.log:12`「DETAIL n=0 lost=Some(0) I-7.6: 根环区域的设备 [0, 1, 0]：不同值 2、最重的盘背 1 个（上界 3）」。

所以那六条在卸载一个字都没写的单盘镜像上已经红（I-2.1 那一条读的是丢掉的盘 0 上那一份）；Imm 在 n = 0（同一份盘面）上丢盘 0 也红这七条，逐字相同（两份日志里 `DETAIL n=0 lost=Some(0)` 那七行 `diff` 无输出、退出码 0）。X 那一刀上，第五节的判据给出：交集为空 ⇒ 那六条是单盘镜像本身的事。

### 1.3 放开扫：交集非空的形（打中）

| 族 | 实现 | 一行汇总（原样） |
|---|---|---|
| 窄盘 | Imm | `research/prompts/m3-unlink-code-r3-opus-model/out/sweeps-summary.txt:1`「narrow imm: starts=1900 start_failed=1400 unmount_raised=1900 unmount_err=0 starts_with_intersection=174 intersection_slot_lines_sum=288 start_lost_pairs_with_old_root_read_failure=37 such_cuts=728」 |
| 窄盘 | Held | `research/prompts/m3-unlink-code-r3-opus-model/out/sweeps-summary.txt:2`「narrow held: starts=1900 start_failed=1400 unmount_raised=1648 unmount_err=252 starts_with_intersection=0」 |
| 窄盘 | GA | `research/prompts/m3-unlink-code-r3-opus-model/out/sweeps-summary.txt:3`「narrow ga: starts=1900 start_failed=1400 unmount_raised=1900 unmount_err=0 starts_with_intersection=0」 |
| 窄盘循环 | Imm / Held / GA | 第 4–6 行：186 个里 13 个有交集、3 对读不出 / 147 个里 0 / 186 个里 0 |
| 普通盘循环 | Imm / Held / GA | 第 7–9 行：400 个里 1 个有交集、1 对读不出 / 0 / 0 |
| 普通盘 | Imm | 第 10 行：553 个里 0 |

`start_failed` 是参数组合本身造不出来的（批数不少于单元数时第 N 批报 `UnlinkOfAnInodeThatIsNotALiveRecord`，窄盘挂载取号之前被拒 60 个等），三个实现同一份起点。交集按起点逐个列在 `research/prompts/m3-unlink-code-r3-opus-model/out/imm-intersections.txt`（188 个起点，每个两行：START 行与交集槽号、槽上的单元、卸载之前那条记录的释放代）。被写掉的单元有 extent 树叶（树 65536 层 0）、inode 叶容器（码 3 打包单元）、`deleted_inodes` 树节点（树 65537）、删除中文件的数据单元、映射树节点（树 65540）。例（h2，窄盘 176 槽、覆盖写 0、4 单元、崩在第 3 批之后；现行那一版 txg 14，丢盘 0 时盘 1 上最新的是 txg 13）：

`research/prompts/m3-unlink-code-r3-opus-model/out/h2-imm.log:172`「INTERSECTION_BY_RECORDS lost=dev0 root_txg=13 count=3 slots=["(1,1411)=node(tree=TreeIdentifier(65537),level=1,birth=8) released_gen=Some((true, 14))", "(1,1414)=data(DataUnitIdentity { tree: TreeIdentifier(65536), object: 1, object_birth: CheckpointTxg(3), anchor_offset: 0 },birth=4) released_gen=Some((true, 14))"…」

### 1.4 读得到被写掉的单元（打中）

h2 那一份第 n = 8 刀（先写系统配置 2 个写 + 第一次空发布 6 个单元写），丢盘 0：

- Imm：`research/prompts/m3-unlink-code-r3-opus-model/out/h2-imm.log:58`「CUT n=8 roots_landed=0 next=unit_write lost=dev0 chosen=Some(13) sc_floor=Some(14) inter_records=1 inter_reads=1 inter_checker_reads=1 whole_read=differs[ro:Open(Walk(MappingStillUnreadable { slot: SlotNumber(1411) }))] checker=I-1.1+I-2.1+I-4.8+I-7.2+I-7.4+I-7.6+I-7.9 remount=-」
- Held（同一份起点卸载做得成、写数相同，同一刀）：`research/prompts/m3-unlink-code-r3-opus-model/out/h2-held.log:58`「CUT n=8 roots_landed=0 next=unit_write lost=dev0 chosen=Some(13) sc_floor=Some(14) inter_records=0 inter_reads=0 inter_checker_reads=0 whole_read=same checker=I-1.1+I-2.1+I-4.8+I-7.2+I-7.4+I-7.6+I-7.9 remount=-」

`whole_read=same` 是剩下那块盘只读挂载、整读每个活文件的结局与卸载之前同一块盘上的逐项相同（走到的根 txg 与内容哈希）；`differs[ro:…]` 是只读挂载本身失败。

层 0 枚举域（每段展开，8223 个状态）上 h2 按丢的盘分（从 `ENUMV` 行按丢的盘、择到的根加出来的数，命令：`grep ^ENUMV <日志> | awk '{split($2,a,"="); c=a[2]; k=$3" "$4; if ($0 ~ /whole_read=differs\[(ro|open|read)/) f[k]+=c; if ($0 ~ /inter_records=[1-9]/) h[k]+=c; t[k]+=c} END{for (k in t) print k, t[k], h[k]+0, f[k]+0}'`）：Imm 丢盘 0、择到 txg 13 的 8214 个状态里 6158 个只读挂载失败；Held 同样 8214 个里 0 个。两个实现两块盘都在时 checker 0 违例：`research/prompts/m3-unlink-code-r3-opus-model/out/enum-h2-imm.log:277`「ENUM states=8223 observed=8223 violations=8223 failed_states=0 checker_violated={}」、`research/prompts/m3-unlink-code-r3-opus-model/out/enum-h2-held.log:277`「ENUM states=8223 observed=8223 violations=8223 failed_states=0 checker_violated={}」。h3（窄盘 176 槽、覆盖写 5、1 单元、崩在第 1 批之后，12330 个状态）：Imm 丢盘 1、择到 txg 15 的 4107 个状态里 2051 个有交集、2051 个只读挂载失败，Held 0。

4 GiB 盘（随机历史默认那一档）的卸载—再挂循环，活文件（h4：第一个文件之后崩了再挂 1 次、卸载、再挂、覆盖写 2 次，再卸载；现行 txg 12，丢盘 0 时盘 1 上最新的是 txg 10）：

- Imm 第 18 刀：`research/prompts/m3-unlink-code-r3-opus-model/out/h4-imm.log:59`「CUT n=18 roots_landed=0 next=unit_write lost=dev0 chosen=Some(10) sc_floor=Some(12) inter_records=1 inter_reads=1 inter_checker_reads=1 whole_read=differs[open:ExtentTreeWalk(MappingStillUnreadable { slot: SlotNumber(50240) })] checker=I-1.1+I-2.1+I-4.8+I-7.2+I-7.4+I-7.6+I-7.9 remount=-」——只读挂载做成，打开活文件时 extent 树走不下去。
- Imm 第一次空发布的单元写写完（n = 22）：`research/prompts/m3-unlink-code-r3-opus-model/out/h4-imm.log:84`「CUT n=22 roots_landed=0 next=journal_record lost=dev0 chosen=Some(10) sc_floor=Some(12) inter_records=2 inter_reads=2 inter_checker_reads=2 whole_read=differs[ro:Open(Walk(MappingStillUnreadable { slot: SlotNumber(50242) }))] checker=I-1.1+I-2.1+I-4.8+I-7.2+I-7.4+I-7.6+I-7.9 remount=-」
- Held 第一次空发布的单元写写完（它这一次写 16 个单元，n = 18）：`research/prompts/m3-unlink-code-r3-opus-model/out/h4-held.log:58`「CUT n=18 roots_landed=0 next=journal_record lost=dev0 chosen=Some(10) sc_floor=Some(12) inter_records=0 inter_reads=0 inter_checker_reads=0 whole_read=same checker=I-1.1+I-2.1+I-4.8+I-7.2+I-7.4+I-7.6+I-7.9 remount=-」

这一份的录制流 Imm 52 个写、Held 44 个，状态数 Imm 2097168（超过约 10⁶，层 0 枚举没跑，只跑前缀刀）。

### 1.5 同一刀上两个实现的池级 checker（174 个交集起点逐刀比）

174 个起点在 Imm 与 Held 上各跑一遍前缀刀，每刀三种丢盘都开 checker。同一刀 = 同一份起点、同一个 n、同一种丢盘，只在 Held 卸载做得成且写数与 Imm 相同的起点上比（32 个起点 Held 卸载在预演里拒，15 个写数不同，不比）。`research/prompts/m3-unlink-code-r3-opus-model/out/hits-compare.txt` 原样：

- `research/prompts/m3-unlink-code-r3-opus-model/out/hits-compare.txt:5`「comparable_cuts_with_imm_intersection=2098」
- `research/prompts/m3-unlink-code-r3-opus-model/out/hits-compare.txt:3`「  ..same_checker_names_as_held=1641」
- `research/prompts/m3-unlink-code-r3-opus-model/out/hits-compare.txt:1`「  ..checker_names_differ_from_held=457」
- `research/prompts/m3-unlink-code-r3-opus-model/out/hits-compare.txt:4`「comparable_cuts_imm_old_root_read_failure=672」
- `research/prompts/m3-unlink-code-r3-opus-model/out/hits-compare.txt:2`「  ..held_read_ok_at_same_cut=672」
- 名字不同的例：`research/prompts/m3-unlink-code-r3-opus-model/out/hits-compare.txt:16`「EXAMPLE (('176', '5', '1', '1'), 8, 'dev1', 'I-1.1+I-2.1+I-3.11+I-4.8+I-7.2+I-7.4+I-7.6+I-7.9', 'I-1.1+I-2.1+I-4.8+I-7.2+I-7.4+I-7.6+I-7.9')」（前一个是 Imm，后一个是 Held：Imm 多红 I-3.11）

名字相同的那 1641 刀上，checker 的原文也不同：h1（窄盘 320 槽、覆盖写 0、14 单元、崩在第 0 批之后）第 18 刀丢盘 0，I-7.2 那一条 Imm 多了「数据单元两份都读不到对得上的」「映射条目指的单元两份都读不到对得上的」两段，Held 没有（两份日志 `h1-imm-detail.log`、`h1-held-detail.log` 在草稿目录，没进模型目录；同形的 h2 第 8 刀上 Imm 多「树 65537（种类 2）的根 两份都读不到对得上的」，草稿里 `diff` 的原样输出：`< DETAIL n=8 lost=Some(0) I-7.2: 最新的根走不完：树 65537（种类 2）的根 两份都读不到对得上的；分配记录树（树 65538）的根：内部条目指不到一个合法的孩子，这一支走不下去；映射条目指的单元两份都读不到对得上的` 对 `> DETAIL n=8 lost=Some(0) I-7.2: 最新的根走不完：分配记录树（树 65538）的根：内部条目指不到一个合法的孩子，这一支走不下去`）。

### 1.6 第五节那条判据怎么落、打中之后的四句

第五节的分句：交集非空「而且改之前那一版在同一刀上不红那几条」⇒ 打穿；「交集为空，或者改之前那一版同样红」⇒ 单盘镜像本身的事。

- 对正文点名的那一份（X 第 1404 行的盘面）：交集为空 ⇒ 字面落在后一句。
- 对放开扫出来的形：交集非空；「那几条」指那六条（I-1.1、I-2.1、I-4.8、I-7.2、I-7.4、I-7.6），它们在卸载一个字都没写之前的单盘镜像上就红（1.2 那一行；I-2.1 的原文读的是丢掉的那块盘上那一份），同一刀 Held 同样红 ⇒ 字面同样落在后一句。分得开两个实现的是 Imm 多出的 I-3.11（457 刀）、I-7.2 原文里多出的几段、以及只读挂载读不读得出（672 刀），都不在「那几条」的名字里。

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到 | 满足的是哪一句 | 跑前给的改法在这几格上还中不中 |
|---|---|---|---|---|
| 丙1 写掉另一块盘上最新的根引用的单元（1.3、1.4） | 分辨：Held 与 GA 在同一批起点上交集都是 0、读得出（1.3 表、1.4） | 看得到：抬 F 那一串手里有根环读出来的每条根（`roots`）与区域归属（`device_of_txg`），每块盘上最新的有效根 txg 就在里面；GA 只用这两样（见第四节） | 判据字面：X 那一份交集为空；放开扫的形「改之前那一版同样红那几条」——字面不判丙1。这是「判别子观测不到」那一形：判据拿 checker 的条名区分，而那六条在单盘镜像上卸载之前就饱和，条名分不开两个实现；分得开的是只读挂载结局与 I-3.11 | 第五节只给「交用户 / 记欠账」，没有改法；我提的 GA 在这几格上不中（量过），回到 Held 也不中但丢掉丙1 要的那 252 个卸载 |

另一种读法摆在这里交主 agent 判：D16 已定项 1「生效」那一行把系统配置里的 F 算进 F_生效（`.claude/kb/decisions/16-发布语义.md:39`「| 生效 | F 另记进每块盘的系统配置（「抬 F 那一串」那一行写明先写后抬）；恢复后生效值 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 被抛弃时间线上的根所带 F 的最大值, 系统配置里读得出的 F 的最大值)；系统配置里读得出的取每块幸存盘两槽里全部自证过（校验和过且 fsid 与本池相同）的槽，与取号式子同一取法（D18（块里携带什么信息） 已定项 11） |」），按候选集「txg ≥ F_生效」读，剩下那块盘上那条 txg 13 / 19 / 10 的根本来就不是回退候选，Held 下它读得出也只是恢复择根不看 F 的结果；另一方面掉一块盘之后按盘数降级是 I-7.6 那一行写明的承诺（`.claude/kb/invariants.md` 第 60 行，整行太长不抄，只指路）。两种读法下丙1 的后果不同：前一种下它不改变「剩下那块盘没有合法的根」，后一种下它把「退 2 代、读得出」变成「读不出」。条款没写恢复在系统配置 F 高于剩下那块盘上全部根时择什么。

### 1.7 两块盘都在：没打中

前缀刀上「两块都在」那一种交集 0（全部族、三个实现）；层 0 枚举的 5 份（X、h2、h3 各 Imm，h2、h3 各 Held）`checker_violated={}`。两块都在时恢复择的是现行那一版或卸载那一串更新的根，回收的槽释放代 ≤ C，C 不引用它们。前缀刀上再可写挂载一次之后 checker 红的有 4 刀，都是 I-7.9，Held 在同一份起点同一刀上也红（见「顺手看到」），不归丙1。

## 二、N2：写账与取界

### 注错扫（没打中，一次抽样）

造法：窄盘起点停在挂载之前（同第一节的窄盘族：单元区 176/208/240/272/304/336 槽、覆盖写 0/3/6/9 次、文件 1/4/14 个单元、崩在第 0/1/3 批之后，可造的 185 个），另加只崩了再挂的窄盘（240 槽那一档，第一个文件之后崩了再挂 6/7/8/9/10/12 次，判下一次挂载；写行之后那一处推两串抬 F 的形就在这里）。可写挂载（准入判着、批大小 1）先不注错录一遍数清写调用总数 W，再对 n = 1 … W 每个写调用各注一次块设备错，两种：只这一次失败、这一次与之后每一次都失败（掉电）。每个注错点比：错里的写账合计 vs 设备一层落下的写 − 取号两次；错里的已落盘份数 vs 落下的根槽写数。`FloorRaiseFailedAfterTheMountsPublishes` 按契约文档有两种读法（`/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-core/src/mount.rs:510`「/// + 写行之后那一处另加 `floor_raises` 各串的各次空发布（续做那一处它们已在 `writes_of_persisted_publishes` 里，不另加）+ `cause` 里那一串自己的写账。」），两种都算，份数同样两种都算。

结果：`research/prompts/m3-unlink-code-r3-opus-model/out/n2-narrow-summary.txt:19`「starts=189 clean_ok=185 fault_points=72372 unledgered=0」，只崩了再挂那一族 6 个起点 1760 个注错点（`research/prompts/m3-unlink-code-r3-opus-model/out/n2-remount-summary.txt`）。

- 写账：每一个带账的错，至少一种读法等于「落下的写 − 取号两次」；没有一个注错点两种都对不上（汇总里没有 `match=none` 一格）。两种注错方式各一份、数相同：`Publish`（写行、暖机与续做里的一步，含 V2 加进去的先写系统配置那几步）每种 28129 + 2634 个点，V1 包的 `FloorRaiseFailedAfterTheMountsPublishes` 每种 4205 + 90 + 542 + 12 + 6 + 198 个点，写账全对上。
- 份数：对不上的全部是「失败的那个写是系统配置槽写、根槽写已落、根槽写数比份数多 1」，例 `research/prompts/m3-unlink-code-r3-opus-model/out/n2-narrow-summary.txt:17`「  power Publish match=contract persisted_ok=false failing_write_kind=system_configuration_slot roots_minus_persisted=1 = 2634」——r2 H4 那一形（V4 只改注释），不是这一批引入的。
- 落下的写多于取号两次而错里不带账的：0（`unledgered=0`）。取号那一步被拒的 370 个点落下的写不超过 2。

### 两处口径（量到，X 报告 Q2 已自报）

写行之后那一处推过抬 F 之后再有一串报错，只有「写行」读法对上：`research/prompts/m3-unlink-code-r3-opus-model/out/n2-remount-240.log` 里 `match=row_reading` 的那几格（崩了再挂 7–12 次的 5 个起点，每个起点每种注错方式 45 + 6 + 2 个点）；续做那一处推过之后再报错，只有「续做」读法对上（窄盘那一族每种注错方式 90 + 12 + 6 个点）；两处都还没推成一串时两种读法相等。错值里没有一项说明它出自哪一处：一个拿到 `FloorRaiseFailedAfterTheMountsPublishes` 的调用方要自己知道是哪一处，才算得出合计。今天入库的调用方（`crash_injection.rs` 的划段）只看各份写的结构种类、不算合计，读错这一口径的消费者今天没有（推的：在冻结副本 `crates/` 下 `grep -rln writes_of_persisted_publishes` 找到的读它的入库代码里，`crash_injection.rs` 第 1790 行那一支只划段；`new_pool_file_creation_on_device.rs` 第 843 行把这个成员并进不划段的那一大支；别的命中是测试、`mount.rs` 与 `transaction.rs` 自己，没逐个读完）。不分辨这一批，记一笔。

### 取界：准入与续做都推（没打中）

harness 里那份取界（`singlefs_harness::mount_transparency::FloorRaisesInTheMountWrites::of_the_mount_output`，N5 搬进来的）在窄盘网格（176–320 槽 10 档 × 覆盖写 0–10 × 1/2/4/8/14 单元 × 崩在第 0/1/3/7/13 批之后）上，与真值（写表里第一个所带 F 高于取号那一次的系统配置槽写，偏移 481）逐个比：984 个推过的起点逐个相等（准入推 958、续做推 5、两边都推 21），566 个没推的都交 `NotPushed`，0 个 `NotLocatedInTheMountWrites`（`research/prompts/m3-unlink-code-r3-opus-model/out/n5-narrow.log` 的 `N5TALLY` 行，没有一行 `N5MISS`）。r2 我在抄进来的那一份上做过同类，这一轮换成入库的那一份、换了网格。

## 三、N7：模型的映射树上界与分批

### 上界低不低于真节点数（没打中）

造法：4 GiB 两块盘，第一个文件之后 14 次用户改动：七成是顺序写到 2–1500 个单元（跨 1–11 片下段叶），三成是一个单元以内的覆盖写，最后一次 unlink（经会话，产品批大小）；300 个种子。每一次发布（会话交回的 `versions_published` 与推的抬 F 的空发布，按 txg 排）取实现这一版中央映射树的真条目数 E 与真节点数 N，算两种上界：沿来路带的（上一版的上界，模型的读法甲）与按上一版真节点数的，都用 `singlefs_harness::model::central_mapping_tree_nodes_upper_bound`。10 批合计 5497 次发布，两种上界都没有一次低于 N（各批 `N7SUMMARY` 行 `violations=0`，例 `research/prompts/m3-unlink-code-r3-opus-model/out/n7-4g.log:52`「N7SUMMARY publishes=542 violations=0 max_nodes=11 worst_ratio_bound_carried_over_nodes=285.00」）。真节点数最多 11。条目数这里用的是实现的真条目数，不是模型自己现算的那个（`central_mapping_entries_other_than_the_allocation_record_tree` 私有，没接）：判的是上界那段推理，不是模型算的条目数。

### 只涨不落：松到什么程度（打中松紧，后果是推的）

沿来路带的上界从不往下走（`/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-harness/src/model.rs:1651`「previous.central_mapping_tree_nodes_upper_bound.max(1),」），而真树在一次覆盖写把文件缩回一个单元之后回到一个节点。最松的一处：`research/prompts/m3-unlink-code-r3-opus-model/out/n7-4g.log:119`「N7SLACK seed=75 unlink txg=27 entries=66 nodes=1 bound_carried=333 bound_tight=3」。

模型每次发布把「映射树上界 − 1」个节点各一槽加进占槽上界（`/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-harness/src/model.rs:1692`「occupied_slots_upper_bound_per_device: previous.occupied_slots_upper_bound_per_device」，沿来路累加），撞单元区墙那一种拒绝按占槽上界 × 64 > 单元区槽数判允许（`/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-harness/src/model.rs:2491`「planned.occupied_slots_per_device * CLUSTER_SEGMENT_SLOTS」）。4 GiB 盘单元区 211968 槽，门槛 3312 槽。只把映射树那一项（Σ(上界 − 1)）累加，300 段里 8 段在第 22–25 次发布时单这一项就过了门槛（那时按真节点数累计 117–133 槽），例 `research/prompts/m3-unlink-code-r3-opus-model/out/n7-4g.log:36`「N7CROSS seed=17 publish=25 mapping_slots_carried=3338 mapping_slots_actual=126 threshold_slots=3312」：同一段历史按真节点数只有 126 槽。过了门槛之后，那段历史里之后每一次发布被实现拒成撞单元区墙，模型都判「允许」——一个在盘几乎空着时就错拒的实现，在这几段历史里比不出来。

推的部分：模型的占槽上界本来就沿来路累加全部写过的槽（数据单元也在里面），一段写了几千个单元的历史不靠映射树这一项也会过门槛；映射树这一项是在那之上额外加的、且只增不减。我没接模型本身跑随机历史（执行器的 `random_histories` 没跑），所以「模型在哪一步开始允许」「改钉值之前那一版在同样的历史上从哪一步开始允许」都没量；也没造一个错拒的变异去证漏判。

### 分批（没打中，读代码）

实现：`/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-core/src/unlink.rs:125`「let data_units_kept = data_units_remaining - data_units_per_batch;」，产品批大小是 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`（144），剩下不多于一批时是最后一批。模型：`/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-harness/src/model.rs:651`「const MODEL_DELETION_BATCH_DATA_UNITS: u64 = EXTENT_TREE_LOWER_LEAF_DATA_UNITS;」、`/tmp/claude-1000/m3-unlink-code-r3/tree/crates/singlefs-harness/src/model.rs:1941`「let step = if data_units_remaining > MODEL_DELETION_BATCH_DATA_UNITS {」。同一个常量、同一个比较、都从尾部；空文件两边都按 1 个单元算。分得开两者的只有挂载用了只供测试的批大小（执行器不装那个开关）。

## 四、改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| GA：卸载那一格只把「释放代 ≤ 每块盘上最新有效根的 txg 取小」的槽当场放开，其余照准入那一格扣到生效（`patches/ga-reuse-only-below-every-newest-root.patch`，在 `raise_the_floor_through` 回收那一处，用它手里已有的 `roots`、`device_of_txg`、`abandoned_by_table`） | 1.3、1.4：丢任何一块盘之后，剩下那块盘上最新的根不引用被当场放开的槽 | 量过（副本）：窄盘 1900 个起点交集 0、卸载 0 个被拒（`research/prompts/m3-unlink-code-r3-opus-model/out/sweeps-summary.txt:3`「narrow ga: starts=1900 start_failed=1400 unmount_raised=1900 unmount_err=0 starts_with_intersection=0」），两族循环 186 + 400 个起点交集 0（第 6、9 行）；入库两份 harness 用例在 GA 副本上全过：`research/prompts/m3-unlink-code-r3-opus-model/out/v2-tests.log:28`「test result: ok. 21 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.40s」、`research/prompts/m3-unlink-code-r3-opus-model/out/v2-tests.log:47`「test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s」。推的：它靠「恢复在一块盘上择那块盘上最新的自证根」，那条根自己坏了（介质错）时往更旧的根退，GA 不罩；层 0 枚举没在 GA 上跑 |
| GB：`mount.rs` 第 2251–2255 行那段文档改写：回收的槽被写掉时引用它们的根低于 F_生效，但恢复择根不看 F（`recovery.rs` 第 1260 行），丢盘之后照样会择到它们 | 文档与恢复行为对不上 | 推的 |
| GC：补一条用例：卸载那一串第一次空发布的单元写完之后丢掉住着现行那一版的根的盘，剩下那块盘只读挂载并整读活文件，判与卸载之前同一块盘上读出的相同（h2、h4 两形） | X 第 1404 行那条用例只比池级 checker 的条名，那六条在单盘镜像上饱和，比不出丙1 | 判法在原型上量过（Imm 红、Held 与 GA 绿）；做成入库用例是推的 |
| GD：条款补一句：恢复在系统配置的 F 高于剩下那几块盘上全部根时择什么（择那块盘上最新的根照旧读出旧版，还是按 F_生效 拒） | 1.6 那两种读法之间的空档 | 推的；交主 agent / 用户 |
| GE：模型的映射树上界加一个与来路无关的上界取小（例如按这一版条目数从空树建起的那一个再加上收缩留下的非空节点数的界），或注明占槽上界在长历史上判不了单元区墙 | 第三节「只涨不落」 | 推的；没实现、没跑 |

N2 那两处口径（第二节）我不提改法：今天没有算合计的消费者。

## 没打中的形状

- 两块盘都在：前缀刀全部族、三个实现交集 0；层 0 枚举 5 份 checker 0 违例（1.7）。
- 普通盘（不经卸载—再挂循环）553 个起点交集 0：卸载的空发布把单元放进聚簇段里没用过的槽，没碰刚回收的那几个。
- N2 写账：74132 个注错点没有一个两种读法都对不上；份数对不上的全是 r2 H4 那一形。
- N2 / N5 取界：1550 个起点 0 个对不上、0 个取不出。
- N7 上界低于真节点数：5497 次发布 0 次。
- N7 分批：读代码，同一个式子。

## 这条腿自己的限度

- 一次抽样；没打中的几格照规则要再抽一次。
- 层 0 枚举只跑了 5 份（X、h2、h3 各 Imm，h2、h3 各 Held），合计 8223 × 3 + 12330 × 2 = 49329 个状态；其余起点只跑前缀刀（层 0 枚举域的一个子集）。h4（4 GiB 循环）一段就是 2097168 个状态，超过约 10⁶，没跑，只跑了前缀刀。
- 「R 引用的槽」三条读法都是我定的；读法二（恢复加整读实际读到的槽）在 X 那一份上比读法三多读 11 个槽（`research/prompts/m3-unlink-code-r3-opus-model/out/x-imm-detail.log:153`「CONTROL lost=Some(0) root_txg=19 by_records=27 read_slots=21 read_slots_within_by_records=10 checker_slots=488 checker_slots_within_by_records=27」），多出来的我推是恢复按 journal 记录点名验证时读的后几版的单元（没逐个核），判交集用读法三；读法三若漏认 R 引用的槽，交集只会少算，所以 X 那一份另用读法二核过：前缀刀与层 0 枚举域的 `ENUMV` 行里 `inter_reads` 全是 0（`grep -c` 计 0 行）。
- 整读只读活文件；删除中的文件（窄盘族多数起点）只读挂载本身要走的树，读不到它的数据单元。
- 「同一刀」按同一个 n 认；Held 写数不同的 15 个起点不比，h4 那一份按「第一次空发布的单元写写完」对齐（Imm n = 22、Held n = 18）。
- 第一节 1.5 里 h1 两份日志与草稿里那次 `diff` 的输出没进模型目录（在草稿目录 `out/` 下）。
- `hits-imm-*` 那一批 10 件并行、估时 2 分钟，跑了 4 分 12 秒，超过前台 240 秒被转成后台，我没停它、等它跑完（读了它的完成通知）；之后的批都拆小到前台跑得完。Held 那一批第一次有 3 件包装退 252（内存排不上，命令没跑），单独重跑了那 3 件。
- `rerun.sh` 是把实际跑过的命令改写成参数化的一份，整份没有从头跑过。
- N3–N6、N8 没攻；N7 没接模型本身、没跑执行器的随机历史。

## 顺手看到、不在这一轮判的

- 两块盘都在、卸载那一串某一次空发布的记录已落而根没落的刀上，再可写挂载一次之后池级 checker 红 I-7.9（窄盘 240 槽、覆盖写 3、14 单元、崩在第 1 批之后，第 15、16 刀；Held 同一份起点同一刀同样红）：不分辨丙1，没查原因。
- 单盘镜像上池级 checker 在卸载之前就红六七条（I-2.1 读的是丢掉那块盘上的一份），checker 没有「掉了一块盘」的读法，单盘恢复的判定今天只能靠恢复与只读挂载的结局。

## 没做什么

- K1、K3 不归这条腿；N3–N6、N8 没攻。
- 没跑 checker 档、没跑整包 harness；只在 GA 副本上跑了两份 harness 测试目标。
- 副本上的数不进 kb；交回前删掉草稿目录里的三份仓副本与三个编译目录（见交回）。

## 要主 agent 跑的命令

没有必须的。要核 N1 的数，在新的空草稿目录里 `ATTACK_DIR=<目录> bash research/prompts/m3-unlink-code-r3-opus-model/rerun.sh`（工作目录任意；内存上限脚本里写死 1G–4G、线程 10；全套估 40–60 分钟，要放后台的话由主 agent 起）。看 `summarize.sh` 那一段的十行、`compare.py` 的输出与 `out/enum-h2-*.log` 的 `ENUMV` 行。
