# e171-conclusion-r1 攻方腿（Opus）报告

轮名 e171-conclusion-r1（第一轮），立场：攻方。只攻派给我的攻击面 ①–⑤，G1、G3 为重点。代码引文写冻结副本 `/tmp/claude-1000/e171-conclusion-r1/tree/` 的路径与行号；kb 引文写 kb 文件自己的行号。日期 2026-10-03。

## 各格判定一览

| 格 | 对象 | 攻方判定 | 依据 |
|---|---|---|---|
| G1 | 第 23 行 | 条款读对；这一批三条都放行属实；同一坏实现在「建第 234 个文件」这段可达历史上，O2 现有的 I-9.4 等三条当场红 ⇒ 归「取样点（历史）不敏感」，不是三条的共同盲区 | 量过（副本，变异打进 core） |
| G1 | 第 42 行 | 条款读对；卸载之后再挂，F = 11（基线 17），管理员回退多收 txg 11–16 六个目标，都是 API 交回值；病根是 O1 模型不答卸载（登记读法 6）；O2 的 I-7.9 按定义不判「不抬 F」的带记号根 | 实现一侧量过；模型的答推的 |
| G1 | 第 92 行 | 「写入时间是发布参数」只在里程碑页，决策里没有；没有任何 API 交回写入时间 ⇒ 归「条款没定 + 不可见」，不算共同盲区（认里程碑句为条款时才是真盲区，这一点要主 agent 定） | 读 kb 与码 |
| G1 | 第 242 行 | 写序口径（已定项 7）下不合法、合法性口径（D16（发布语义） 已定项 12）下 replay 之后合法；但「崩溃 → 可写挂载」上 9 / 276 态因见证值拒可写，正确写序下不会拒 ⇒ 归「取样点（历史）不敏感」，补在 O1 的历史 | 量过（实现一侧） |
| G1 | 第 269 行 | 条款读对；「崩溃 → 可写挂载」上流一 6 / 31、流二 46 / 92 态基线挂得上而变异拒 ⇒ 同上 | 量过（实现一侧，逐态差分） |
| G1 | 第 270 行 | 写序口径下不合法、已定项 12 口径下合法；C577 那条性质（见证值 ≥ 末条记录计数器）真被违反的是 5 + 63 态（执行员数 13 + 129）；「崩溃 → 可写挂载」上 0 态因见证值拒；害处要读故障，E171 的域里观测不到；O3 加一句就判得到 | 量过（我的模型） |
| G1 | 第 271 行 | 条款读对；头 ID core 只搬不读、checker 不解 ⇒ 真盲区，但属 O2 少一条单镜像字段核对，不是独立性失败 | 读码，推的 |
| G1 | 第 233 行（丙） | 丙去掉了抓它的那一列（甲乙崩溃判官红，层 0 oracle 与记录核对器也红）；它暴露 D13（验证路线） 已定项 6 表 O3 行「验不到什么」少一句「实现自己的恢复 / 读者代码」 | 读 kb 与码 |
| G2 | 第 15、34 行 | **不是「对这一批等价」**：D3（空间分配） 已定项 10 ③「数据单元取最低的…起点」（第 15 行另违反 ⑤ 的新池新建文件落点表），两行落点都不是最低合格起点 ⇒ 真盲区候选 | 量过（导出目录逐条比落点） |
| G2 | 第 27、112、277 行 | 等价站得住（没打中）；第 27 行另有一段会分开的历史（多记录发布，推的） | 推的 |
| G2 | 第 102、233、265 行 | 没打 | — |
| G3 | O3 的 A(c) | A(c) 取自实现写的根 ⇒ 一致写错的根整类无感（第 42 行是实例）；造出一个只让 O3 红的坏实现（「已返回的调用丢了」，27 / 288 态，O2 0 红、实现恢复 = 盘上最新根、崩溃判官按判据绿）；表里没有这一类 ⇒ 「O3 0 红」不是独立性的证据；D13 那句「只照…文字重新推导」写宽了：恢复规则与 ack 窗口是推导的，根的内容不是（指针一半是边界，标量与写序一半改得动） | 量过 + 读码 |
| G4 | 第 155、161 行与触达 | D = 39、Q1 不变的推算对；缺口射程更大：只改取号 / 暖机 / 卸载交回值的坏实现三条都放行且进不了分母，装置不记交回值，产物答不了 | 推的 |
| G4 | 表的来源 | 37 行里 35 行由专用测试抓（O2 坏镜像族 0 行、O1 模型机器 2 行），多是「一致写错一个值」⇒ Q3「只靠 O2」偏多、O3 独有那一类缺席；Q1 的结论字不受影响 | 量过（计数） |
| G5 | 已定项 6 与 C407 | E171 撑「独立性要验证」，不撑也不推翻「互相独立」；放行原因拆成五种，没有一行是三条共享前提一起错；C407 原样做门禁今天就红，宜改写 | 推的 |

## 复跑与文件摘要

模型目录 `research/prompts/e171-conclusion-r1-opus-model/`。复跑（在副本里，只写给定的工作目录；编译与探针都经 `research/scripts/run-with-memory-cap.sh`，线程数默认 15）：

```
bash research/prompts/e171-conclusion-r1-opus-model/tools/rerun.sh /tmp/claude-1000/<新的工作目录> 15
```

它依次跑：① `witness_after_return.py`（第 270 行两种数法）；② `o3_ack_probe.py`（只让 O3 红的写流模拟）；③ `killer_tests_of_counted_rows.py`；④ `select_states.py` 挑态；⑤ 从冻结副本建仓副本（`Cargo.toml` / `Cargo.lock` 取模型目录里那两份），编两个 harness 档探针 `opus_e171_crash_then_mount_probe`、`opus_e171_history_probe`；⑥ 分片跑「崩溃之后可写挂载」，`summarize_probe.py` 汇总；⑦ 四段短历史；⑧ 把第 23 行变异打进副本 core、重编、建 233 个 inode。⑥ ⑧ 在负载高的机器上各要十几分钟。比对：输出与 `out/` 下同名文件逐行比（`out/probe-summary.out`、`out/witness-after-return.out`、`out/o3-ack-probe.out`、`out/hist-*.tsv`、`out/killer-tests.out`）。

每个文件的 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里）：

```
02befec7f52278bfdb15188d72c481f23029a6f634e7f55beb33088e6cec2eef  apply_mutation.py
f8aa9a8f90b66c7abdc0dbc79a752c227bccb47c7303356a9ebed4fb7ede2e3b  killer_tests_of_counted_rows.py
d09fc905941aec0be2b28a6cf00708665c4877a86c80990ecfe1744c54499f2c  o3_ack_probe.py
38fdee3f1a722438a40d720071263dca4b548c17216427af63bfda41effe70dc  opus_e171_crash_then_mount_probe.rs
b8f99d608887d46ba45ac214eb0944630733939c8bef66e8851c605b590522a4  opus_e171_history_probe.rs
2179634646c83752538145a0122637222c49782c63c08ac2587cec3615490ba7  out/ack-o3-verdicts.tsv
917e3ca4dbcbadbc6dd35fea0b9a0d874b421cd3080f4f4b662c52b5c93d8cd2  out/ack-probe.tsv
4ab845c8987d7b497fd98beee7955166eedd87086b15b492af7d85d3cbabf6f7  out/baseline-calibration-partial.tsv
59b8f06fd9fab9c3f8aa5e17e447bfe977ae04b59ae9f01d4fabc1a7bdb9f1d3  out/baseline-calibration-summary.out
c0afa2645b7303aab8d029045434457b4d1931ec6acbb4fe27808a9e2d6db012  out/hist-h1-1.tsv
9f7e5ed6e08e7fa694aebbab611677e6981ec905b4044e6ba0ce655072170e29  out/hist-h1-2.tsv
6d19b49e5df281fead91feefbf6f683915b19102490cb19f82b5e86b863b50cc  out/hist-h1-3.tsv
5b15b18032098f0dcd00769e918eb1f02fb57a6e81480ce9c5cfec425c8d64fa  out/hist-h1-4.tsv
4115b73ce848de8bf09fd8a0f2f8962287150bc82d7f6621550640c004763d40  out/hist-h2-1.tsv
186d466f4c2be1d8a3291327b0ace6503f9747d8dc73666ffff225c69bb6205e  out/killer-tests.out
bed12aa6f8ee969475ace135fafa5885eab9555b5ccd7becbcfebb58575feadf  out/o3-ack-probe.out
285cd91e7c3b25a88698904e8bcb4d7eca0edec320a65e139ee5073cb1bd6da5  out/placement-row15.out
e033f839713b6e480dfdb743fb89614438845cc30e6f737cdbc6b687aa07541e  out/placement-row34.out
16b4b0409de23f93e5860252b041efc6ef1c3ac0717b382c29fbac91d5d1cbdd  out/probe-summary.out
a999b88fa5b3854e7a7dd8dafca976d3ea3e9685a5840980e126ba4b247f186b  out/sel-probe.tsv
6aa5c8af1f1ddfa3a789ea78b867e9af9427b0ed70d105e81e7444f25fd85507  out/witness-after-return.out
ce29fdd071958e2e4aff5dbdad14ef4176aa1b694a9b70f99d0f254b9bb4fae1  run_crash_probe_batch.sh
7ae95f48716a9e6e98dc7b42b08ebda04f3d61b624ef21537f308d7bfe0d3e0e  select_states.py
f9d6531f05a5f80d6b3a5a0eba5979371cf0dbe24482eb23e4e4a80ba7b8f6ae  summarize_probe.py
8a159d4e0eb724c59ef6d40bba3f4bb0e1cf2c24f72529932b616df131aa0e34  tools/rerun.sh
896d5d2055c1b4139b9e7d84a9aca5ef1ee4ad3abc6db4cdf6739237f837c7d8  witness_after_return.py
8ee46ee73ba30d367cb9f4ad2c0350c73a259dfa14bf0baf60d1202da7409c4b  workspace-Cargo.lock
4fc67af798ffb9786d37c0ef04b5f3651ad2100f252cff1fa5280a6a7a34c3c7  workspace-Cargo.toml
```
## G1　真盲区候选逐行（攻击面 ①）

口径：「状态非法」照执行员自己的三分口径（取到的态里有条款不许的）；「三条看不看得到」分两层：这一批两条流上（E171 测的），与在同一个坏实现上把历史往后接一小段、由用户决定的那几步放开（攻方补测的）。后一层打中的，算「取样点（历史）不敏感」，同第 102、265 行在执行员口径里的处置，不算三条 oracle 的共同盲区。

### 第 23 行（inode 叶容器号 1 → 0）：条款读得对；不是三条的共同盲区，O2 在多建 233 个 inode 的历史上就红

- 条款：「每个容器号都是某条记录首次插入时的 inode 号」（`.claude/kb/decisions/08-核心索引结构.md:205`），容器号 0 不在其中，执行员读对了。
- 变异点 `container: inode.0,` → `container: 0,`（「container: inode.0,」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-core/src/inode_tree.rs:244`））是「新开一片容器」那一支，第一片叶与之后每次末尾分裂都走它。
- 攻方补测（量过，副本）：把第 23 行的变异打进副本的 core，拿 E171 导出的第 23 行流一全持久态当起点，可写挂载之后 `publish_new_inodes` 建 233 个 inode（用户建第 234 个文件），盘上第二片叶容器号仍是 0，池级 checker（O2，未改）当场红三条。原样输出（模型目录 `out/hist-h2-1.tsv`）：

```
row23_s1_m23core	inodes	step=new_inodes_1	ok containers=0:233,0:1	recover=2,6 o2_red=I-1.1:树_65537（种类_2）的根：key_宽_/_key_区间与条目不符（Err(KeysNotStrictlyAscending)）|I-9.4:沿叶序容器号不严格递增：容器_0_之后是容器_0|I-9.12:第_1_条的分隔_key_0_不大于上一条的_0
```

  同一段历史在基线 core 上（参照）：`base_s1	inodes	step=new_inodes_1	ok containers=1:233,234:1	recover=2,6 o2_red=-`。判它的是 I-9.4「沿叶序容器号严格递增」（`.claude/kb/invariants.md:290`）。
- 判：这一批（一个文件、不分裂）上三条都放行属实；同一个坏实现在「多建文件」这段可达历史上被 O2 现有的不变量抓到。归「取样点（历史）不敏感」，补的是历史取样（inode 数跨过一片叶），不是 oracle。单片叶上容器号 0 的那些态本身，今天三条都判不到（推的：要判就得一条「从没分裂过的叶，容器号 = 叶里最小 inode 号」的新不变量，没写、没量）。

### 第 42 行（卸载把 F 抬到现行那一版带的 F）：条款读得对；经 API 两步就看得到，缺的是 O1 模型里的卸载

- 条款：D16（发布语义） 已定项 1「正常卸载」一行「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md:42`）；射程「卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版」（`.claude/kb/decisions/16-发布语义.md:57`）。变异点「let rollback_floor = current_file_version.root.checkpoint_txg;」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-core/src/mount.rs:2190`）。
- 攻方补测（量过，副本、基线 core）：拿基线与第 42 行流二（卸载之后）的全持久态各做一次可写挂载，再把管理员回退的目标逐个放开扫（实例 0–4 × txg 1–22，110 个目标，每个从挂载之后那一刻的拷贝起）。原样输出（模型目录 `out/hist-h1-1.tsv`、`out/hist-h1-2.tsv`）：

```
base_s2	remount_rollback	step=mount	ok instance=3 new_root=3,22 floor=17 chosen=2,19 effective=2,19	recover=3,22 o2_red=-
base_s2	remount_rollback	step=rollback_sweep	tried=110 accepted=2,17;2,18;2,19;3,20;3,21;3,22
row42_s2	remount_rollback	step=mount	ok instance=3 new_root=3,22 floor=11 chosen=2,19 effective=2,19	recover=3,22 o2_red=-
row42_s2	remount_rollback	step=rollback_sweep	tried=110 accepted=2,11;2,12;2,13;2,14;2,15;2,16;2,17;2,18;2,19;3,20;3,21;3,22
```

  卸载之后再挂，挂载交回的根带 F = 11（基线 17）；管理员回退多收 6 个条款不许的目标（txg 11–16）。两处都是 API 交回的值，O1-对拍比 `ObservedRoot` 的 F、比回退的成否——前提是模型答得出卸载（登记读法 6 写明模型没有卸载的答）。
- O2：I-7.9 带卸载记号那一支只判上界，而且只判「抬 F 的那一条根」：「它带的 F 比同一实例里 txg 比它小的最新那条根带的高」（`.claude/kb/invariants.md:63`）。变异下卸载那串根带的 F 与前一条相同，按定义不是「抬 F 的那一条」，这条不变量根本不判它。O3：A(c) 就是这一行自己写的 F = 11 的根（见 G3）。
- 判：这一批上三条都放行属实；但不是 O2 / O3 才补得上的盲区，病根是 O1 一列的已知缺口（模型不答卸载），E171 的流又止于卸载。补在 O1：模型给出卸载的答（带记号的根、F = 现行那一版的 txg），O1-对拍在卸载那次调用上就比得出（推的，没实现）。D13（验证路线） 已定项 6 表 O1 行「验不到什么」没写卸载，这一处不是「已承认」。

### 第 92 行（覆盖写照抄上一版的写入时间）：不是条款，也不可见；归「条款没定」，不归共同盲区

- 「写入时间是发布参数」只在里程碑页：「写入时间是发布参数」（`.claude/kb/milestone/02-second-txn.md:103`），`01-first-txn.md` 第 103 行是「现状」一段里的括注；`grep -rn '写入时间' .claude/kb/decisions/` 零命中。
- 实现里没有任何读路径把写入时间交回调用方（`grep -n 'write_time' crates/singlefs-core/src/mounted_read.rs` 零命中；`write_time_seconds` 只出现在 `inode_tree.rs`、`mounted_session.rs`、`records.rs`、`transaction.rs` 的写一侧），没有 stat 一类 API。
- D13（验证路线） 已定项 6 的三条各自的判据来源：O1「POSIX 可见语义：内容、大小、目录结构」（`.claude/kb/decisions/13-验证路线.md:161`），而这个实现没有让它可见的 API；O2 判 invariants，没有一条管写入时间；O3「只照 decisions 与 invariants 的文字重新推导」（`.claude/kb/decisions/13-验证路线.md:163`），两处都没有这一句。三条都没被派到这件事上。
- 判：里程碑页的验收句不是三条 oracle 的判据来源；这一行归「条款没定（决策里没有）+ 不可见」，记成规约欠账（要么在 D8（核心索引结构） 已定项 6 写明取值并给出读得到它的 API，再由 O1 判；要么写明这 24 字节不进判定），不算「三条的共同盲区」。按执行员的另一种读法（认里程碑句为条款）它才是真盲区；两种读法的分辨事实是「kb 里的验收句是不是 D13（验证路线） 已定项 6 所说的『规约』」，这一点要主 agent 定。

### 第 242、270 行（写序类）：执行员数的「不合法的态」是写序口径；按 D16（发布语义） 已定项 12 的合法性口径，这些态 replay 之后合法

- 两行依的都是 D16（发布语义） 已定项 7 的写序：「系统配置槽在根槽持久之后再更新」（`.claude/kb/decisions/16-发布语义.md:181`）、「fsync 等系统配置轮换持久之后才返回」（同一行）。这是实现的义务，执行员按「照条款的写序造不出来」数态。
- 同一份 kb 给中间状态的**合法性**另有一条：「根 + journal 任意前缀，replay 之后合法」（`.claude/kb/decisions/16-发布语义.md:264`）。按这一条，一个态合不合法看 replay 之后落到哪，不看它的字节是不是正确写序造得出来。O1-崩溃判官与 O3 判的正是这一口径（O3：「landed_is_allowed = recovery.landed is not None and any(version.projection == recovery.landed for version in allowed.allowed)」（`/tmp/claude-1000/e171-conclusion-r1/tree/research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/executor_core.py:127`）），两条在这两行的每一态上都判「在 / 绿」。
- 第 270 行条款的害处由 C577 写明：钉的性质是「发布返回后每个崩溃点见证值 ≥ 末条记录计数器」（`.claude/kb/checks-owed.md:582`）。攻方在导出态上照这一性质重数（`witness_after_return.py`，冻结 O3 的读写流与建镜像代码，量过）：

```
name=witness_after_return label=base1 states=60 executor_style=0 witness_below_ack_rotation=0 versions_with_rotation=3 examples=-
name=witness_after_return label=base2 states=516 executor_style=0 witness_below_ack_rotation=0 versions_with_rotation=19 examples=-
name=witness_after_return label=r270s1 states=56 executor_style=8 witness_below_ack_rotation=5 versions_with_rotation=3 examples=state=29 batch=1 seg=6 witness=1 needed=2 ack=2;state=31 batch=1 seg=6 witness=1 needed=2 ack=2;state=32 batch=1 seg=6 witness=1 needed=2 ack=2
name=witness_after_return label=r270s2 states=460 executor_style=79 witness_below_ack_rotation=63 versions_with_rotation=19 examples=state=29 batch=1 seg=6 witness=1 needed=2 ack=2;state=31 batch=1 seg=6 witness=1 needed=2 ack=2;state=32 batch=1 seg=6 witness=1 needed=2 ack=2
```

  批 1 + 批 2 合计：见证值真落后的态流一 5、流二 63（基线 0）；执行员式的数法在我这一版上是 8 / 79（执行员报 13 / 129，切段配对不同，没对齐到逐态）。差额是「一块盘的轮换落了、另一块没落」的态：见证值取全池最大，这类态见证值不落后。
- 攻方补测「崩溃之后可写挂载」（量过，副本、基线 core 做挂载；第 270 行取上面执行员式挑出的批 1 + 批 2 全部 87 态；第 242 行取「同一次发布的系统配置写落了、它后面的根槽写没持久」的态，批 1 + 批 2 全部：流一 35 态、流二 241 态（流二分两趟跑，批 2 的 40 态标 `r242s2b2`、批 1 的 201 态标 `r242s2b1`））。按拒的原因汇总（`summarize_probe.py`）：

```
name=mount_after_crash label=r270s1 mount=ok o2_green=True digest_ok=true count=5
name=mount_after_crash label=r270s1 mount=refused:SomeSystemConfigurationSlotUnreadOrUnverified o2_green=True digest_ok=true count=3
name=mount_after_crash label=r270s2 mount=ok o2_green=True digest_ok=true count=43
name=mount_after_crash label=r270s2 mount=refused:SomeSystemConfigurationSlotUnreadOrUnverified o2_green=True digest_ok=true count=36
name=mount_after_crash label=r242s1 mount=ok o2_green=True digest_ok=true count=12
name=mount_after_crash label=r242s1 mount=refused:AgainstTheRecordAtTheWitnessedCounter o2_green=True digest_ok=true count=3
name=mount_after_crash label=r242s1 mount=refused:SomeSystemConfigurationSlotUnreadOrUnverified o2_green=True digest_ok=true count=20
name=mount_after_crash label=r242s2b1 mount=ok o2_green=True digest_ok=true count=82
name=mount_after_crash label=r242s2b1 mount=refused:AgainstTheRecordAtTheWitnessedCounter o2_green=True digest_ok=true count=2
name=mount_after_crash label=r242s2b1 mount=refused:AgainstTheSelectedVersionsLastRecord o2_green=True digest_ok=true count=2
name=mount_after_crash label=r242s2b1 mount=refused:SomeSystemConfigurationSlotUnreadOrUnverified o2_green=True digest_ok=true count=115
name=mount_after_crash label=r242s2b2 mount=ok o2_green=True digest_ok=true count=38
name=mount_after_crash label=r242s2b2 mount=refused:AgainstTheRecordAtTheWitnessedCounter o2_green=True digest_ok=true count=1
name=mount_after_crash label=r242s2b2 mount=refused:AgainstTheSelectedVersionsLastRecord o2_green=True digest_ok=true count=1
```

  `SomeSystemConfigurationSlotUnreadOrUnverified` 是撕裂了系统配置槽写的态，基线上同一种拒照样有（「限度」第 3 条），不算这两行的害处。去掉它：第 270 行 87 态里 0 态因见证值拒；第 242 行 276 态里 9 态因见证值拒（流一第 13、17、62 态，流二第 13、17、105、109、454、491 态，拒因是 `AgainstTheRecordAtTheWitnessedCounter` 或 `AgainstTheSelectedVersionsLastRecord`）——例：流一第 13 态（落地向量 `22222000…`）暖机的记录与一块盘的系统配置写落了、根槽 FUA 没落，恢复停在 (0,0)，见证值 1 指着实例 1 的记录 (1,1) ⇒ 判真、重读仍真、拒可写。按 D16（发布语义） 已定项 7 的写序，系统配置写不会先于它见证的根落盘，这样的拒在正确实现上不会出现。
- 第 270 行的害处要第二件事才显：C577 管的是「最新根一时读不出、恢复靠见证值知道丢了东西」，那是读故障；第 242 行 `crates/mutations.tsv` 点名抓它的测试在 `fault_injection_fast_tier`（写故障，C381 的前提），而它在纯崩溃域里另有上面这 9 态的可见害处。
- 判：两行的态在执行员的写序口径下不合法、在已定项 12 的合法性口径下 replay 之后合法（O1-崩溃判官、O3 判的就是这一口径，绿得对）。分开说：第 242 行在「崩溃 → 可写挂载」这段两步可达的历史上看得到（9 / 276 态拒可写，基线写序下不会拒），与第 269 行同类，归「取样点（历史）不敏感」，补在 O1 的历史（崩溃态上可写挂载、模型给答）；第 270 行在这段历史上 0 态可见，害处只在读故障下显，E171 的域（只造崩溃）里观测不到。分辨这两种判法的是「D13（验证路线） 已定项 6 的『规约允许的集合』指字节（写序可达）还是 replay 结果」：O3 的登记读法 2、3 取的是后者。其余归属：第 270 行的性质 O3 本来就能判（它有 ack(c)，加一句「见证值 ≥ 已确认那一版轮换写带的 tail」，攻方在自己的模型上量过：基线 0 态、第 270 行 5 + 63 态，被攻过零轮）；第 242 行的写序是录制流上的静态性质，在 O3 读流时按 D16（发布语义） 已定项 7 核一遍写序即可（推的，没实现）；它的害处归故障注入（D13（验证路线） 已定项 3 的 RefFS 调用点）。不算「三条的共同盲区」，算「三条按已定项 12 判、写序义务没有人判」。

### 第 269 行（取号见证值取成槽世代号）：条款读得对；崩溃之后可写挂载就看得到（拒可写），E171 的崩溃判官只做只读读回

- 条款：「mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）。变异点「.map(|system_configuration| system_configuration.quantities.journal_tail)」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-core/src/transaction.rs:719`）。
- 同一节写见证值的用处：读阶段之后「仍为真就在取号之前拒可写挂载」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）。变异把见证值抬高，崩在「取号写落了、下一次轮换还没落」之间的态上，下一次可写挂载会以为有一版更新的发布读不出而拒。
- 攻方补测（量过，副本、基线 core 做挂载；态取 E171 第 269 行导出里镜像与基线同号态不同的那些，批 1 + 批 2，流一 31、流二 92，落地向量与基线同号态逐个核过相同；基线同号态各跑一遍当参照）。逐态差分原样输出：

```
name=row269_vs_baseline stream=s1 baseline=ok row269=ok count=10
name=row269_vs_baseline stream=s1 baseline=ok row269=refused:AgainstTheRecordAtTheWitnessedCounter count=3
name=row269_vs_baseline stream=s1 baseline=ok row269=refused:Undecidable count=3
name=row269_vs_baseline stream=s1 baseline=refused:SomeSystemConfigurationSlotUnreadOrUnverified row269=refused:SomeSystemConfigurationSlotUnreadOrUnverified count=15
name=row269_vs_baseline stream=s2 baseline=ok row269=ok count=16
name=row269_vs_baseline stream=s2 baseline=ok row269=refused:AgainstTheRecordAtTheWitnessedCounter count=3
name=row269_vs_baseline stream=s2 baseline=ok row269=refused:AgainstTheSelectedVersionsLastRecord count=40
name=row269_vs_baseline stream=s2 baseline=ok row269=refused:Undecidable count=3
name=row269_vs_baseline stream=s2 baseline=refused:SomeSystemConfigurationSlotUnreadOrUnverified row269=refused:SomeSystemConfigurationSlotUnreadOrUnverified count=30
```

  基线挂得上、变异拒可写的态：流一 6 / 31，流二 46 / 92（拒在取号之前，盘上不变；`NewerStateStillUnreadableAfterOneReread`）。O2 在这 123 态上 0 条红（同一份输出的 `o2_red`）。
- 判：这一批的态 replay 之后合法（崩溃判官绿属实），但「崩溃 → 可写挂载」这段两步可达的历史上，实现交回的结局与基线不同（基线挂上、变异拒）。O1-对拍比可写挂载的结局（`answer_mount_writable`）；E171 的流里没有「崩溃之后再可写挂载」这一步，O1-崩溃判官只做只读读回（`observed_read_back_after_a_crash`）。归「取样点（历史）不敏感」：补在 O1 的历史里加「崩溃态上可写挂载」并让模型给答。⚠️ 基线本身在撕裂了系统配置轮换的态上就拒可写（见「这条腿自己的限度」第 3 条，C331 的已知形），差分只能逐态比，不能看「拒没拒」。

### 第 271 行（deleted_inodes 的树表条目头 ID 写 0）：条款读得对；字段谁都不读，补在 O2 是一条字段核对

- 条款：「`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537）」（`.claude/kb/decisions/08-核心索引结构.md:250`）。
- 读者一侧：core 解出头 ID 只是原样搬（「head_identifier: entry.head_identifier,」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-core/src/mount.rs:5507`）），`grep -rn 'head_identifier' crates/singlefs-checker/src/` 零命中。没有任何 API 交回它、没有任何判定读它：这个字段今天写进去就没人看，接什么历史都观测不到（推的：按读者一侧零引用推，没造 unlink 历史跑变异，因为头 ID 在 unlink 路径上同样只搬不读）。
- 判：真盲区，但不是「三条的共同盲区」那种独立性失败，是 O2 的定义域里少一条字段核对（单个镜像、字段值由 D8（核心索引结构） 已定项 8 的表写死），不需要第二个输入。补在 O2：树表条目头 ID 照已定项 8 的表核（零轮）。D13（验证路线） 已定项 6 表 O2 行「验不到什么」写的是「一个块被覆盖成另一份合法内容」（`.claude/kb/decisions/13-验证路线.md:162`），头 ID 0 不是合法内容，不在已承认的那一类里。

### 第 233 行（丙里全放行）：丙是去掉了抓它的那一列的读法；它暴露的是 O3「验不到什么」少写一句

- 甲、乙里 O1-崩溃判官红（流一 3 态、流二 42 态），层 0 那份 oracle 与记录核对器也红（`layer0_or_record_checker_red=true`，实验页第 157 行的产物行）。丙只取 O1-对拍，按定义不看崩溃之后的恢复。
- 按 D13（验证路线） 已定项 6 字面的分工，O1「崩溃之后允许看到的集合归 RefFS（已定项 3）」（`.claude/kb/decisions/13-验证路线.md:161`）：实现自己的恢复代码出错，字面上的三条确实都看不到——O2 判恢复之前的镜像，O3 跑它自己的恢复、从不跑实现的恢复。表里 O1 行「验不到什么」写了「崩溃语义」，O3 行只写「规约本身可能写错——必须配阳性对照才有意义」（`.claude/kb/decisions/13-验证路线.md:163`），没写「实现自己的恢复代码」。
- 判：不是验证体系的盲区（崩溃判官、层 0 oracle、记录核对器都抓到），是 D13（验证路线） 已定项 6 表里 O3 一行少一句「验不到：实现自己的恢复 / 读者代码（它只跑自己的恢复）」。这一类在 37 行里有 8 行（27、108、112、115、233、237、277、281，错在实现的读者或恢复一侧），O3 对它们按构造永远不红。

## G2　「对这一批等价」与「取样点不敏感」（攻击面 ②）

### 第 15、34 行不是「对这一批等价」：D3（空间分配） 已定项 10 写死了数据单元取最低的合格起点

- 执行员判「落点条款没定」所用的检索是 `grep -rn '起扫\|单元区起点'`，漏了 D3（空间分配） 已定项 10：「③ 数据单元取最低的、起点 32768 对齐、两槽都空、整跨度都过合取的起点」（`.claude/kb/decisions/03-空间分配.md:204`），同一项还把新池新建文件上的落点写成表：「⑤ 在新池新建文件上的落点（开放段 [50240, 50304)，bump 游标从 50240 起）」（`.claude/kb/decisions/03-空间分配.md:206`），其中「| t2 | extent 树根兼叶（码 2，16 KiB） | 11 | 50240 | 0 |」（`.claude/kb/decisions/03-空间分配.md:210`）。
- 第 15 行（量过，导出目录逐条比，槽号 = 偏移 ÷ 16384）：基线流一用户数据在槽 50180（两槽），extent 根在 50240；第 15 行用户数据在 50240、extent 根在 50304，七个提交内生块整体后移 64 槽。原样输出（`awk` 取流一第 22–45 条 op、盘 0、槽号:跨度）：

```
== baseline-run1
22:50180:2 24:50240:1 26:50242:2 28:50244:1 30:50245:1 32:50246:1 34:50247:1 36:50248:1 38:50249:1 40:50250:1 42:50251:1 44:50252:1 
== rows-s4/row-15/dump
22:50240:2 24:50304:1 26:50306:2 28:50308:1 30:50309:1 32:50310:1 34:50311:1 36:50312:1 38:50313:1 40:50314:1 42:50315:1 44:50316:1 
```

  第 22 条 op 之前两边写流逐条相同（执行员读数：第一处不同在第 22 条），分配状态相同；基线在同一状态上取槽 50180，变异跳过它取了更高的 50240，不是最低的合格起点，违反 ③（「50180 合格」靠的是基线实现的取法，推的）；extent 根落在 50304，与 ⑤ 的表（50240）逐字不符。
- 第 34 行（量过，同法，流二第 498–537 条 op）：基线抬 F 之后那次发布的用户数据落在槽 50176（复用回收的槽），第 34 行落在 50196。原样输出：

```
== baseline-run1
498:50176:2 500:50439:1 502:50440:2 ...
== rows-s4/row-34/dump
498:50196:2 500:50439:1 502:50440:2 ...
```

  第 498 条 op 之前两边写流逐条相同（执行员读数）；抬 F 之后那几槽按 D16（发布语义） 已定项 1「| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根) |」（`.claude/kb/decisions/16-发布语义.md:33`）已是可再分配，基线在同一状态上取了 50176，变异取 50196，不是最低的合格起点（同上，「50176 合格」靠基线的取法，推的）。
- 判：两行在「择到新版」的那些态里（执行员自己数的：第 15 行流一批 1 12 态、流二 356 / 112 态；第 34 行流二 60 / 14 态）镜像不在 ③（第 15 行另加 ⑤）写得出的集合里，照执行员的三分口径归**真盲区候选**，不归「对这一批等价」。三条各自为什么放行，执行员的那一列成立（O1 不比落点、O2 只判记账自洽、O3 投影不带落点）。补在哪：O2 可以写一条「这一版新分配的数据单元起点 = 按上一版（候选根里比它旧的那条）的分配状态算出的最低合格起点」（单个镜像里有上一版的根，推的，没写没量；开放段只住内存，② 那一合取在单镜像上要另想）。这一判让 Q1 的真盲区候选多两行，对 Q1「有」的结论字没有影响。

### 第 27、112、277 行：「对这一批等价」站得住；第 27 行另有一段会分开的历史

- 三行在这一批里与基线不同的态都是「调用没返回」的态（执行员 `returned_check.py` / `extra_checks_f4.py` 的读数），新旧两版都在已定项 12 的集合里。攻方没找到反例（没另跑）。
- 第 27 行（末条标志读反）会分开的历史，执行员写的是「已返回、提交点只在记录里」；另有一段更近的（推的，没跑）：一次发布跨两条以上记录（点名单元超过一条记录的 67 项，`publish_new_inodes` 的注释写明会跨条），记录都落了、根没落的态上，变异把中间那条（不带末条标志）读成末条，只施加半次发布——这是已定项 4「一次发布整体施加或整体不施加」不许的，且实现恢复落到一版模型没提交过的根，O1-崩溃判官的 ②「择到的根要是模型提交过的某一版」会红。这一行在这一批里等价成立，换历史就被现有的一列抓到，不是盲区。

### 第 102、233、265 行记「不稳定」：没打（没找到与执行员读数相反的事实）

## G3　O3 的 A(c) 取自实现写出的根（攻击面 ③）

### 它推导什么、不推导什么（读码）

- 版本表：「版本表 = 基镜像上的现行那一版（版本 0，执行器在基镜像上自己恢复得出）加录制流里每一次根槽写按流序各一版。」（`/tmp/claude-1000/e171-conclusion-r1/tree/research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py:3`）；每一版的投影直接取实现写下的那 512 字节：「versions.append(PublishedVersion(len(versions), projection_of_root_bytes(prepared.new_content),」（`/tmp/claude-1000/e171-conclusion-r1/tree/research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py:61`）。允许集合「A(c) = 版本表里流序落在 [ack(c), begun(c)] 的各版」（同一文件第 91 行）；判定是投影相等（`executor_core.py:127`，见 G1）。投影只有六样，「卸载记号与根记录其余字段不进投影」（`/tmp/claude-1000/e171-conclusion-r1/tree/research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/root_ring.py:23`）。
- 由此 O3 从 kb 文字重新推导的是：择系统配置、择根、journal 前缀（含点名单元校验）这几条**恢复规则**，与「已返回的调用不许丢、没开始的发布不许出现」的**窗口** [ack, begun]。它**不**推导：每一版根该带什么值（F、水位、txg 序列、指针）、根之外的任何内容、写序、实现自己的恢复代码。
- 推论（读码推的，第 42 行是一个实测例）：实现在全部根槽写上一致地写错投影里的任一字段，只要它自己的记录新根段与根一致、写序不丢已返回的调用，O3 在每一态上都判「在」。第 42 行就是这一形：A(c) 里的那几版 F = 11，恢复落到的也是 F = 11，「在」。

### 只让 O3 红的坏实现（造出来了，量过；写流层面模拟，推的等价于三处代码变异）

- 坏实现：「暖机的第二次空发布不等持久就返回」——拿掉它的记录与根槽之间那道屏障、根槽写不带 FUA、轮换之后那道屏障也拿掉（写的字节与基线逐字节相同，只改屏障与写种类）；历史截到下一次调用（新池新建文件的 publish）的第一组单元写为止。崩溃态：末段合并出的 7 次写整写子集全枚举、可撕裂的两次写另加撕裂态，2^5 × 3^2 = 288 态，一个不落；前面各段与基线相同（E171 基线三条全绿），不重判。
- 原样输出（`o3_ack_probe.py`，冻结 O3 的代码）：

```
name=o3_ack_probe tail_writes=7 torn_in_tail=2 states=288 o3_strict_not_in=27 o3_provisional_not_in=27 spec_gap=0 anomalies=1
```

  同一批 288 态再喂 Rust 探针（副本、基线 core、未改的池级 checker），按「O3 严格判定 / 实现只读恢复走到的根 / O2 / 盘上已持久的最新根槽（txg,实例）/ 崩溃之后可写挂载」汇总（`summarize_probe.py`），O3 红的 27 态：

```
name=ack_probe count=3 o3_strict=not_in recover=1,1 o2_red=- persisted_txg_inst=1,1 mount=ok digest_ok=true
name=ack_probe count=9 o3_strict=not_in recover=1,1 o2_red=- persisted_txg_inst=1,1 mount=refused:AgainstTheSelectedVersionsLastRecord digest_ok=true
name=ack_probe count=15 o3_strict=not_in recover=1,1 o2_red=- persisted_txg_inst=1,1 mount=refused:SomeSystemConfigurationSlotUnreadOrUnverified digest_ok=true
```

  这 27 态里：暖机已返回（ack(c) = 第二次空发布那一版 (1,2)），而它的记录与根都没落、下一次调用的单元写落了。O3 判「不在」；实现恢复走到 (1,1)，O2 0 条红；盘上已持久的最新根槽是 txg 1、实例 1，O1-崩溃判官的 ③「盘上已持久的最新根槽是 (T, i) 时，择到的根按 (txg, 实例) 不许比它旧」（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/singlefs-harness/src/model.rs:2614`）成立，(1,1) 是暖机交出的、模型提交过的一版，读回没有文件——按这四条判据推，崩溃判官判绿（推的：判据按代码逐条推，没在模型上跑）；O1-对拍比的是调用交回的值，写种类与屏障不改交回值（推的）。只有 O3 红。
- 这说明两件事。① O3 有一类别人判不到的：**已确认的调用丢了**（崩溃判官只跟「盘上已持久的」比，不跟「已返回的」比）。② E171 的表里没有这一类：`crates/mutations.tsv` 里 `singlefs-core/src` 下变异名带「少一道屏障 / 没有屏障 / 不过屏障」的 13 行（`awk` 现数），进了 281 行的只有第 270 行（轮换之后那道）；记录前后、单元写与记录之间那几道一行都没进（281 行的行名 `grep -E '屏障|FUA'` 只命中第 106、195、242、266、270 行，106、195、266 与屏障数无关）。进了的第 242、270 行，已返回的那一版都还在（两行的根槽 FUA 都在返回之前落盘），journal 重放也补得回。所以 Q3 里「没有一行只靠 O3」、Q1 里「O3 0 红」都不是关于 O3 独立性的证据，这一表根本没测到 O3 独有的那一类。

### 设计边界还是缺陷

- D13（验证路线） 已定项 6 给 O3 的判什么是「崩溃点截断后的每个磁盘状态是否落在规约允许的集合里」、独立性来自「只照 decisions 与 invariants 的文字重新推导」（`.claude/kb/decisions/13-验证路线.md:163`）。实现出来的 O3 重新推导的是恢复规则与 ack 窗口，「允许的集合」里每一版的内容取自实现。按字面，这一句写宽了：O3 对「实现写了一个错的根」整类无感，是这个实现选择（登记读法 2、3）带来的，不是 kb 文字推导不出来——标量字段（txg 序列、实例代号、F、树 ID 水位）按已定条款从调用序列推得出（例：卸载之后 F = 现行那一版的 txg）；指针字段推不出（要第二份完整实现），那一半是设计边界。
- 判：一半是边界（指针、内容），该写进「验不到什么」；一半是缺陷（标量字段与写序），改得动。改法见文末改法表（都被攻过零轮）。

## G4　触达定义与测量方法（攻击面 ④⑤）

### 第 155、161 行：执行员的「D = 39、Q3 只靠 O1 多两行、Q1 不变」对；缺口的射程比「这两行」大

- 那两行的推算没找到反例（两行只有 O1-对拍红，`name=row_matrix` 读数，没另跑）。
- 读法 4 的三样（写流、调用结局、恢复报告）之外，坏实现还能只改「成功调用交回的值」。O1-对拍只在**有模型答的调用**上比交回值；取号、暖机、卸载三种调用是 `not_modelled`（基线流二 `call-outcomes.tsv` 15 次调用里 3 次、流一 3 次里 2 次）。一个只改这三种调用交回值的坏实现：读法 4 记「未触达」、O1-对拍不比、O2 / O3 看不到（写流不变）——三条都放行，而且连分母都进不去。E171 装置不记成功调用的交回值，238 行未触达里有没有这样的，产物答不了（推的；未触达行里名字提到取号 / 暖机 / 卸载的有 21 行，`awk` 现数，逐行看过名字，多数是失败路径或空池、环转一圈这类流里没有的前提，没逐行判）。
- 判：⑤ 的担心成立一半：缺口不会让「已计入分母」的读数变（计入的行至少触达一样），但会让一类「三条都放行、交回值可见」的坏实现落进未触达、不被看见。改法：装置记每次成功调用交回的值（至少 `not_modelled` 的三种），触达多一样（推的，零轮）。

### 坏实现表取自 `crates/mutations.tsv`：偏向谁

- 37 行计入分母的行，各自被 `crates/mutations.tsv` 点名的哪条测试抓（`killer_tests_of_counted_rows.py`，量过）：点名 O1 模型对拍机器的 2 行（11、108），点名 O2 坏镜像族 `checker_known_bad_images` 的 0 行；其余 35 行是逐条手写的专用测试（harness 档 21、core 与 replay-prefix 的单测 8、checker 档 6，含抓第 270 行的 `a_publish_returns_only_after_the_system_configuration_rotation_is_durable`、抓第 242 行的 `fault_injection_fast_tier`）。全放行的 13 行（丙）全在这 35 行里。
- 偏向：表里的行是「项目历史上某条专用测试钉过的一个字节、一个字段、一道屏障」，多数是**一致写错一个值**的变异。这一形对 O1（不读盘）与 O3（A(c) 取自实现）按构造放行，只剩 O2 有没有那一条不变量——所以 Q1 的「有」偏向「有」、Q3 的「只靠一条」偏向「只靠 O2」（23 行里 13 行只靠 O2）。反方向：O3 独有的那一类（已返回的调用丢了，见 G3）表里 0 行，Q3 量不到「只靠 O3」。Q1 的结论字（≥ 1 行全放行）不受偏向影响；Q3 与「O3 0 红」的读数受，不能拿来说 O3 的独立性。
- 域外推：两条流、一个文件、不分裂、不 unlink、流止于卸载、崩溃之后只做只读读回、不造读写故障。G1 里第 23、42、269 行在这一域外一步就被看见（多建文件 / 卸载之后再挂 / 崩溃之后可写挂载），第 242、270 行的害处要读写故障。Q1 的「有」里，一部分是这个域的边，不是 oracle 之间的共同盲区。

## G5　D13（验证路线） 已定项 6 与 C407 怎么改（攻方的读法）

- 依据能引 E171，只引到「三条确有一齐放行的坏实现」这一层（Q1 甲乙各 12 行，攻方加第 15、34 行之后更多）；不能引成「三条之间有共谋式的共同盲区、独立性被否」：按 G1、G2 拆开，放行的原因分五种——O1 的已知模型缺口（第 42 行，卸载不答）、历史取样的边（第 23 行多建文件、第 242 与 269 行崩溃之后可写挂载）、O2 少一条字段 / 落点不变量（第 271、15、34 行）、写序义务按已定项 12 没人判且害处要读故障（第 270 行）、条款没定且不可见（第 92 行）；没有一行是「三条因为共享同一个前提而一起错」。实验页「影响的决策」那一行宜写「支撑『独立性要验证』这一句（测到了放行），不支撑也不推翻『三条互相独立』」。
- 「回头补 oracle」今天落成：O1 模型答卸载、历史里加「崩溃态上可写挂载」与「多 inode」；O2 加树表条目头 ID 核对、新分配数据单元的落点核对；O3 加「见证值 ≥ 已确认那一版的轮换」与录制流写序核对，标量根字段改由调用序列推（全部零轮，见改法表）；故障注入接写序类。
- O3 那一行「验不到什么」要加两句：「实现写出的根的内容（A(c) 取自实现的根槽写，投影之外与投影里一致写错的都判不到）」「实现自己的恢复与读者代码（它只跑自己的恢复）」。
- C407：它要的门禁形态「已知坏实现至少一条红、关一条仍有别的腿红」，按今天的产物第二半句不成立（Q3 甲乙各 23 行只靠一条），照原样做成门禁当天就红。宜改写成两件：① 门禁只钉「已知坏实现每一行至少一条红」，表取 E171 判过的那几行加攻方的「已返回的调用丢了」那一形（O3 一列要有一行只靠它）；② 「关一条仍有别的腿红」降为测量，不当门禁。（推的，零轮。）

## 打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）

「臂」在这一轮指执行员三分的三类与正文 G1 列的三个改法（O1 的 `ObservedRoot` 加字段、O2 加一条不变量、O3 投影加字段）。

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足判据字面的哪一个分句 | 正文给的三个改法在打中的格上还中不中 |
|---|---|---|---|---|
| 第 15、34 行不等价（G2） | 分辨：把两行从「对这一批等价」挪到「真盲区候选」，别的行不动 | 看得到：落点就在镜像里，上一版的分配状态在同一镜像的旧根里（O2 推得出，推的） | 三分口径「真盲区＝取到的态里有条款不许的，三条都放行」：条款是 D3（空间分配） 已定项 10 ③ / ⑤ | O1 加字段：不中（落点不进 `ObservedRoot`，加了也没有模型答）；O2 加不变量：中（推的）；O3 加投影：不中（落点不在根里） |
| 第 23 行在多 inode 历史上 O2 红（G1） | 不分辨三类，分辨「盲区」与「取样点（历史）不敏感」 | 看得到：O2 现有的 I-9.4 | 第 102、265 行那类「取样点不敏感」的同形，只是取样的是历史不是崩溃态 | 三个改法都不用：加历史即中（量过） |
| 第 42 行经 API 可见（G1） | 分辨：病根落在 O1 一列（模型缺卸载），不是 O2 / O3 | O1 看得到交回的 F（`ObservedRoot` 带根的 F），缺的是模型的答 | 三分口径「真盲区」字面成立（这一批）；归因到 O1 已知缺口 | O1 加字段：不中（字段已在，缺的是答）；O2 加不变量（I-7.9 带记号那一支改等号）：中（推的）；O3 投影加字段：不中（F 已在投影里，A(c) 取自实现） |
| 第 242、269 行崩溃之后可写挂载拒（G1） | 分辨：从「真盲区」挪到「取样点（历史）不敏感」 | 看得到：可写挂载的结局是 API 交回值 | 同上一行第三列 | 三个改法都不中（O1 字段、O2 不变量、O3 投影都不管可写挂载的结局）；中的是「O1 历史里加崩溃之后可写挂载」（量过实现一侧的差，模型一侧推的） |
| 第 270 行的态按已定项 12 合法（G1） | 不分辨三类；打中的是判据：「不合法」用的是写序口径 | 三条都看不到写序（O1 不碰盘、O2 判单镜像、O3 按已定项 12 判 replay 结果） | 打中的是三分口径里「条款不许的态」的「条款」是哪一条：已定项 7（写序义务）还是已定项 12（合法性） | O1 / O2 / O3 那三个改法都不中；中的是 O3 加「见证值 ≥ 已确认那一版的轮换」（量过，我的模型上） |
| 第 92 行不是条款（G1） | 分辨：从「真盲区候选」挪到「条款没定」 | 看不到：没有 API 交回写入时间，单镜像不知道发布参数 | 「条款不许」的「条款」不成立（决策里没有） | 三个改法都不中 |
| 第 271 行谁都不读（G1） | 不分辨三类（仍是真盲区）；分辨「独立性失败」与「O2 少一条字段核对」 | O2 看得到：头 ID 就在树表条目里 | 「真盲区」字面成立 | O2 加不变量：中（推的）；另两个不中 |
| O3 只红的坏实现（G3） | 分辨：给 Q3 补上「只靠 O3」那一类（这一表 0 行） | 看得到：O3 有 ack(c) | 不对着 E171 的判据；对着 D13（验证路线） 已定项 6 O3 行「判什么」 | 三个改法都不涉及（不是盲区，是 O3 独有的判别力） |

## 改法表（全部只在我的模型上量过或只是推的，被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| O1：模型给卸载的答（带记号的根、F = 现行那一版的 txg），流二在卸载那次调用上对拍 | 第 42 行 | 推的。量过的只是实现一侧：卸载之后再挂 F = 11（基线 17）、回退多收 txg 11–16 六个目标（`out/hist-h1-1.tsv`、`hist-h1-2.tsv` 原样贴在 G1） |
| O1：历史里加「崩溃态上可写挂载」，模型给答 | 第 269、242 行 | 实现一侧量过（第 269 行 6 / 31 + 46 / 92 态、第 242 行 9 / 276 态因见证值拒可写）；模型一侧的答推的。⚠️ 基线在撕裂系统配置槽的态上也拒（C331 已知形），模型的答要先与 R2 / 甲的取舍对齐，否则基线就红 |
| O1：历史里加「多 inode（跨一片叶）」 | 第 23 行 | 量过：第 23 行变异打进 core 后建 233 个 inode，O2 现有 I-1.1、I-9.4、I-9.12 红；基线同段历史 0 条红 |
| O2：树表条目头 ID 照 D8（核心索引结构） 已定项 8 的表核 | 第 271 行 | 推的（没写、没量） |
| O2：新分配数据单元起点 = 按上一版分配状态算出的最低合格起点（D3（空间分配） 已定项 10 ③） | 第 15、34 行 | 推的（没写、没量；开放段只住内存，② 那一合取在单镜像上怎么判没想好） |
| O2：I-7.9 带卸载记号那一支改成「F 等于根环里 txg 比它小的最新那条不带记号的根的 txg」，且不限于「抬 F 的那一条」 | 第 42 行 | 推的（没写、没量；根环转过一圈、前一条不带记号的根已被盖掉时怎么判没想好） |
| O3：加一句「见证值 ≥ 已确认那一版（ack(c)）轮换写带的 tail」 | 第 270 行 | 量过（`witness_after_return.py`，我的模型）：基线 0 / 576 态，第 270 行 5 / 56 + 63 / 460 态 |
| O3：读录制流时按 D16（发布语义） 已定项 7 核每次发布的写序 | 第 242、270 行 | 推的 |
| O3：根的标量字段（txg 序列、实例代号、F、水位）改由调用序列按条款推，不取实现写的 | 第 42 行（及同形） | 推的 |
| D13（验证路线） 已定项 6 表 O3 行「验不到什么」加两句（实现写出的根的内容；实现自己的恢复与读者代码） | 第 42、233 行与 8 行读者类 | 文字改动，不需量 |
| D8（核心索引结构） 已定项 6 写明写入时间取值并给读得到它的 API，或写明不进判定 | 第 92 行 | 规约改动 |
| E171 装置记成功调用的交回值（至少取号、暖机、卸载），读法 4 的触达多一样 | ⑤ 那一类未触达行 | 推的 |

## 没打中的形状

- 第 27、112、277 行（等价）：在这一批里找与基线不同而「调用已返回」的态，没找到（照执行员的读数，没另跑）；第 27 行另给了一段会分开的历史（多记录发布，推的）。
- 第 102、233、265 行（不稳定）：没找到与执行员读数相反的事实，没打。
- 第 155、161 行的推算（D = 39、Q1 不变）：没找到反例。
- 第 270 行在「崩溃之后可写挂载」上：挑出的 87 态（批 1 + 批 2 全部执行员式的态）里，因见证值拒的 0 态；拒的 39 态都是撕裂系统配置槽那一种，基线上同样有。纯崩溃域里没造出它的可见害处。
- 第 242 行在 O3 上：想过让 O3 红（系统配置先落、根没落的态上 ack 窗口外），没成：根槽 FUA 都在返回之前，A(c) 罩得住，75 态没另跑 O3（E171 已跑，`strict_in`）。
- 「只让 O3 红」的单处代码变异：拿掉轮换之后那道屏障（第 270 行）、只把根槽改成不带 FUA、只拿掉记录与根之间那道屏障，三种单处改法按切段推都会被 journal 重放或同段的屏障补回已返回的那一版（推的，没逐个跑），所以造的是三处合起来的写流（G3）。
- 第 271 行：没造 unlink 历史跑变异——读者一侧对头 ID 零引用（core 只搬、checker 不解），推得出接什么历史都看不到；这是推的，没跑。

取样范围：E171 导出目录里第 15、23、34、42、242、269、270 行与基线的批 1、批 2（态数见各节）；短历史四段（卸载之后再挂并扫 110 个回退目标、建 233 个 inode、unlink 一个、以及第 23 行变异下建 233 个 inode）；写流模拟一段（288 态）。

## 这条腿自己的限度

1. 全部数是副本上的：冻结副本 `/tmp/claude-1000/e171-conclusion-r1/tree/` 的 crates，加仓里 2026-10-03 那一刻的 `Cargo.toml` / `Cargo.lock`（冻结副本里没有这两份，拷进了模型目录，摘要见开头），探针是我写的 harness 档测试目标，不是入库装置。照规则，要引进 kb 先在入库装置上重做。
2. 崩溃态没有照层 0 全枚举：两条流的层 0 全量是 16,777,260 与 1,662,648,564 态（E171 基线产物 `full_states`），超过每段历史约 10⁶ 的上限，按规则不跑；用的是 E171 自己取样的批 1、批 2 里按挑法挑出的态，挑出来的一个不落（第 242 行流二先跑批 2、后补批 1，两趟 15 片，实跑约 11 分钟与 5 分钟）。写流模拟那一段（G3）是我自己造的历史，末段 288 态全枚举，前面各段与基线相同、没重判。全部原型合起来判过的态：288 + 408 + 201 + 220（基线校准，中途停掉的那一批跑完的部分：全量 576 态那一批在负载下估要近半小时，停掉改成按挑法挑态）+ 53（试跑）+ 2（计时小样）= 1172 态，远低于 10⁷。
3. 校准：基线自己在撕裂了系统配置槽写的态上就拒可写挂载（`comparison:SomeSystemConfigurationSlotUnreadOrUnverified`；基线校准 220 态里流一 20 / 60、流二 44 / 160）。这与 D23（journal 的角色与格式） 已定项 14 里「这里的实现还是 2026-09-28 取甲的那一版，欠 C331」相符（`.claude/kb/decisions/23-journal的角色与格式.md:392`），射程句「撕裂的系统配置轮换（普通断电就造得出，被撕裂的那次发布没确认过）不再因判不出拒可写挂载」（`.claude/kb/decisions/23-journal的角色与格式.md:408`）写的是取 R2 之后的样子。我的「崩溃之后可写挂载」差分因此只按逐态比、只数见证值那几种拒。
4. O1-崩溃判官在写流模拟（G3）上是按判据逐条推的绿，没在理想模型上跑（模型要跟着同一段调用推，模拟出来的写流没有对应的模型答）；O1-对拍「交回值不变」也是推的。
5. 「模型会答该成」（第 42、242、269 行补在 O1 的那几条）是推的；量过的只是实现一侧的交回值与基线不同。
6. 一次抽样：这是第一轮攻方的一条腿，「没打中」的格按规则不支撑任何结论。
7. 机器负载：编译与探针都在负载 60–90 的机器上 `nice -n 19` 跑，耗时数不当性能数用。

## 没做什么

- 没跑任何 checker 档测试、没起 E171 装置、没改仓里任何文件（写的只有本报告与模型目录）。
- 没造第 271 行的 unlink 历史，没在模型上跑「崩溃之后可写挂载」的模型答，没实现改法表里任何一条。
- 没判正推、辩方那几格，没替主 agent 采纳。
