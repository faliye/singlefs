# m3-batch31-code-r1 云端攻方（Opus）报告

<!-- doc-lint:not-numbers K1 K2 K3 N1 N2 N3 N4 N5 N6 F-L0 F-W F-IT O3 A1 A2 A3 A4 B1 B2 C1 C2 D1 D2 E1 E2 F-KEY F-LEN -->

轮名 m3-batch31-code-r1（代码轮第一轮），攻击面：正文第三节 K2（重点 N1、N2）与 K2 末两条的第二次抽样。只跑 harness 档，GPU 不跑（读 wgsl、用 CPU 算期望）。合法状态多红那一格（要层 0）不攻、不跑。

## 各格判定一览

| 格 | 判定 | 节 |
|---|---|---|
| K2 N1 F-L0 主形（墓碑、谱系，层级 1，类型段 0，孩子不往下走、每份照判 I-2.1） | 没打中：CPU 量 150 形（墓碑、谱系各 3 种类型段 × 25 种副本组合）与期望模型逐行相同；GPU 读码同支 | 一 |
| K2 N1 层级 0 的根下类型段 0 | 没打中：`node.level <= 1` 把层级 0 一起罩住，CPU 量 50 形与层级 1 同判；GPU `n.level <= 1u` 同支 | 一 A4 |
| K2 N1 孩子一份读不出、一份对得上 | 没打中：读不出那一份照红 I-2.1，F-L0 自己照记走读失败 | 一 A1 |
| K2 N1 盘 0 / 盘 1 | 打中一处（弱，不是第 31 批引入）A3：类型段 0 那一支「子单元是码 2」小句只看第一份 16 KiB 校验和对得上的副本，(c16, node) 与 (node, c16) 判 I-9.23 / I-9.17 一红一绿；判据字面两读，交主 agent | 一 A3 |
| K2 N1 判 I-2.1 的长度读法 | 条款缺字 A2：F-L0 按头里类标签的长度比，I-2.1 那一行列举「不往下走的几支」时没有它；代码与用户 2026-10-06 定案同向，不算代码打中 | 一 A2 |
| K2 N1 合法状态多红 | 不攻、不跑（要层 0，归提交时） | — |
| K2 N2 | 没打中：`pending_level == NONE` 只在读根那一次成立；CPU 量 6 形，GPU 读码同判 | 二 |
| K2 N3 | 没打中（读码）：最终状态里走读内核不记 61，候选根 R 停在第 100 片合并，扫描内核读 R 自己的 257 片走到 37 / 38 | 三 |
| 观察 O3 | **打中**：容器树容器、inode 叶「走过」按第 0 条位置条目认；盘 0 那条位置条目指到这一遍先走过的单元、盘 1 那份是带 I-1.7 违例的容器时 I-1.7 报成立，同一形改成盘 1 那条指过去时红。CPU 量 18 形，GPU 读码同形。改法 F-KEY（只在我的模型上量过、被攻过零轮） | 四 |
| CPU 与 GPU 判得不一样 | 没打中（读码；F-零槽 那一形照正文第一节不攻） | 五 |
| 第二次抽样：m3-batch30 F-W CPU（inode / 墓碑 / 谱系，层级 1，盘 0 / 盘 1） | 没打中：三棵树两种盘序同红 | 六 |
| 第二次抽样：m3-batch30 F-W、F-IT 的 GPU 读码 | 没打中 | 六 |
| 第二次抽样：m3-batch29 L1 一份读不出 | 没打中 | 六 |
| 第二次抽样：m3-batch29 L2 链长与候选集里外 | 没打中（读码） | 三、六 |
| 附带量到：inode 类型段 7、两份各自证成码 2 与码 3 | 盘序一翻 I-3.1、I-3.11、I-5.1 跟着翻；是 C739 那一形（已登记），不算新打中 | 六 |

## 复跑

```
SINGLEFS_REPO=~/.local/share/singlefs/work/m3-batch31-code-r1/tree SINGLEFS_TOOLS=<仓根> \
  bash research/prompts/m3-batch31-code-r1-opus-model/rerun.sh <工作目录> base
#   一次前台跑不完时分三次，各点一条臂：base、mut-fl0、fix-key（同一个工作目录）
bash research/prompts/m3-batch31-code-r1-opus-model/compare.sh <工作目录>   # 末行 SAME 1349 行
```

- `SINGLEFS_TOOLS` 是包装脚本另给的变量（冻结副本里没有 `research/`），写在 `rerun.sh` 文件头。编译 6G、跑 4G 经 `research/scripts/run-with-memory-cap.sh`，线程经 `research/scripts/capped.sh`（`THREADS`，默认 10）；每条臂自己的编译目录 `<工作目录>/target-<臂>`；一条一条起 cargo。
- 三条臂：`base`（冻结副本原样，加探针）、`mut-fl0`（删掉 F-L0 那一处调用，即 `crates/mutations.tsv` 第 31 批 F-L0 那行的替换：期望模型的比对会不会红的自证）、`fix-key`（攻方改法 F-KEY）。`base` 与 `fix-key` 另跑整个 `checker_known_bad_images` 目标。
- 本机一次从头跑通：工作目录 `~/.local/share/singlefs/work/m3-batch31-code-r1-opus/rr4/`（空目录起，三次前台调用各一条臂），`compare.sh` 末行原样「SAME 1349 行」。挂钟：base 约 2.5 分钟、mut-fl0 约 1 分钟、fix-key 约 2.7 分钟（本机负载 32 核占满时量的）。
- 模型文件 sha256：

```
a7809fc650578fa3d3e409b0b20b0b94d068a76fb5ca5eabea3dce402f751c1a  probe_kbi.rs
c29979d888d29c1cb34b6abc382a969e7c8f2cabaa7074afceefdbba51bec12a  expect_model.py
3acbca23c88f871dff0c08bdf5b41d2e57c090277dbdaf697b8da0b488708df9  make_arm.py
9202c85717ddb888dfc54d5af3f48a44bd1cc7b6089adbfc625c119017fed2c9  rerun.sh
01abdcea96ca5750c407227389c4d9fa9ff9c46da0112467c1d4f90a55867452  compare.sh
34e906b6bc2dd36b591ce02519eeb410faf67a8650f56e7b614cce0ba9d2dffa  baseline/summary.txt
```

- 探针 307 行 PROBE：类型段 × 副本矩阵 225（三棵树 × {rec 不改、t0 改 0 并清身份、t7 改不认得的 7} × 25）、层级 0 的根 52、O3 21、extent 上段根 6、干净镜像 3。副本五种状态：`bad` 位置条目校验和翻一字节；`gone` 槽改到盘尾之外、读不出；`c16` 校验和换成容器前 16 KiB 的 CRC（头里仍是码 3）；`node` 那块盘上孩子的槽写进这棵树根（码 2）的原样字节、校验和换成它的 CRC；`ok` 不动（码 3 容器，按 32 KiB 自证得过）。
- 取样没缩：全部探针一次跑完（本机 base 臂探针部分约 30 秒），不涉及崩溃枚举。


下文 `T` 指冻结副本根 `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree`；引代码写全路径。PROBE 行一律从 `~/.local/share/singlefs/work/m3-batch31-code-r1-opus/rr4/base.probe`（本机从头跑通那一次）原样抄，与 `research/prompts/m3-batch31-code-r1-opus-model/baseline/summary.txt` 里同一行逐字相同。

## 一、N1（F-L0）

### A1　主形：层级 1、类型段 0、孩子不往下走，每份副本照判 I-2.1

被判的那一句：`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2499`：「if node.level <= 1 {」，下一行起调共用函数；GPU `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3180`：「if (n.level <= 1u) {」，`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3183`：「w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);」。

取样：墓碑树（`image_after_the_intent_of_a_three_unit_file`）、谱系树（`image_after_rolling_back_to_the_first_root`）、inode 树（`build_pool`）的根都是层级 1；根第一条条目按 rec / t0 / t7 改类型段，两条位置条目各取五种状态之一（上面「复跑」一节），25 种组合全扫，盘 0、盘 1 是位置条目 0、1（两盘同槽）。

期望不从 CPU 输出抄：`expect_model.py` 按 kb 那几行与读码写下的分支规则（类型段那一类、自证成哪一类、F-W、F-L0 每份按头里类标签的长度比、F-L0 恒记走读失败、写进 `node` 字节的那块盘上中央映射条目指的容器对不上）算 I-2.1、I-4.8、I-7.2、I-7.4 与身份那一条红不红，再与 PROBE 行比。它也是 GPU 那一侧的期望：GPU 这几支读码同形（下一段）。结果（`rr4/*.model` 末行原样）：

```
base.model:MODEL agree=257 disagree=0 skipped=18
mut-fl0.model:MODEL agree=221 disagree=36 skipped=18
fix-key.model:MODEL agree=257 disagree=0 skipped=18
```

`skipped` 是 inode 类型段 0 往下走的那 18 形（`walk_inode_internal_child`，不在模型里）。`mut-fl0` 那 36 行都是墓碑、谱系类型段 0 走 F-L0 那几形：CPU 少红 I-2.1，比对路径会红。⚠️ 模型是我看过墓碑 t0 那 25 行之后写的，是一致性核对，不是盲测。

几行原样（墓碑；谱系同形）：

```
PROBE matrix tree=tomb level=1 type=t0 c0=bad c1=bad red=[I-2.1,I-4.8,I-7.2,I-7.4] i2_1=nogo i7_2=fl0 i1_7=Holds
PROBE matrix tree=tomb level=1 type=t0 c0=gone c1=node red=[I-2.1,I-4.8,I-7.2,I-7.4] i2_1=nogo i7_2=fl0 i1_7=Holds
PROBE matrix tree=tomb level=1 type=t0 c0=node c1=gone red=[I-2.1,I-4.8,I-7.2,I-7.4] i2_1=nogo i7_2=fl0 i1_7=Holds
```

`i2_1=nogo` 是 I-2.1 第一条违例说明点名「走读不往下读的孩子」；`i7_2=fl0` 是 I-7.2 的说明里有 F-L0 那句走读失败。`gone`/`node` 两种盘序同红：读不出那一份照红 I-2.1（另一份 `node` 写进容器的槽，中央映射那一处也红 I-2.1，模型把它算进去了），走读失败由 F-L0 自己记。

GPU 读码：`w_copy_matches_its_location_checksum_at_the_length_its_header_names` 读不出交 false（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3085`：「if (copy.version == NONE) {」），登记码 1–3 按那一类的长度（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3089`：「if (class_in_the_header >= 1u && class_in_the_header <= 3u) {」），没登记的两种长度都试，与 CPU `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:4486`：「None => vec![node_bytes(), data_unit_bytes()],」同判；对不上加 `w_mismatches`，主走读与候选根按它判 I-4.8、I-7.4，与 CPU 按 I-2.1 违例次数判同义。F-W 之后落到类型段 0 那一支、层级 ≤ 1 时两边都先判身份、再每份照判、再记一次走读失败。GPU 的读者对盘上碰过的每个 16 KiB 槽都立一个落点（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:496`：「for slot in unit_slots {」），只有没碰过的槽读不出——那是 F-零槽，正文第一节写明这一轮不攻。

判定：没打中。什么会推翻：一条 GPU 对拍流上 F-L0 那一形 GPU 与 CPU 违例位不同（提交时主 agent 跑的那四条对拍）。

### A2　长度读法：F-L0 这一支不在 I-2.1 那一行的列举里

`c16`（头里是码 3、校验和是前 16 KiB 的 CRC，按 32 KiB 自证不过、不算另一类）落到 F-L0：按头里那一类（码 3）的 32 KiB 比，对不上，I-2.1 红：

```
PROBE matrix tree=tomb level=1 type=t0 c0=c16 c1=node red=[I-2.1,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=nogo i7_2=fl0 i1_7=Holds
```

（I-2.1 第一条点名的是 `c16` 那一份，`i2_1=nogo`。）按类型段 0 说的那一类（码 2，16 KiB）比就对得上。I-2.1 那一行（`.claude/kb/invariants.md:130`，整行抄在附录）写「不往下走孩子的几支」按头里类标签的长度比，括注列的是三棵树类型段不认得与孩子自证成另一类两种；F-L0 是类型段认得（0）、孩子不是另一类、只因层级 ≤ 1 不往下走，括注里没有。代码选的是头里类标签的长度，与用户 2026-10-06 定的「不往下走的那两支按头里类标签说的长度」同向，所以我不记代码打中，记条款缺字：写回 I-2.1 时把 F-L0 这一支补进括注（或写明它按类型段那一类的长度）。层级 ≥ 2 时同一份副本走 `read_index_node`，按 16 KiB 读回比（推的，没在层级 2 的镜像上量）。

四句：不分辨第 31 批的改与不改（第 30 批这一支不判 I-2.1，也就没有长度问题）；判别子看得见（头读得出）；满足的是「条款没写」，不是误红、漏判、漏红任一句；改法只有写回，碰得到这一格。

### A3　盘序翻面：类型段 0 那一支「子单元是码 2」只看第一份 16 KiB 对得上的副本（弱打中，交主 agent）

```
PROBE matrix tree=tomb level=1 type=t0 c0=c16 c1=node red=[I-2.1,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=nogo i7_2=fl0 i1_7=Holds
PROBE matrix tree=tomb level=1 type=t0 c0=node c1=c16 red=[I-2.1,I-4.8,I-7.2,I-7.4] i2_1=nogo i7_2=fl0 i1_7=Holds
PROBE matrix tree=lineage level=1 type=t0 c0=c16 c1=node red=[I-2.1,I-4.8,I-7.2,I-7.4,I-9.17] i2_1=nogo i7_2=fl0 i1_7=Holds
PROBE matrix tree=lineage level=1 type=t0 c0=node c1=c16 red=[I-2.1,I-4.8,I-7.2,I-7.4] i2_1=nogo i7_2=fl0 i1_7=Holds
PROBE l0root tree=tomb level=0 type=t0 c0=c16 c1=node red=[I-2.1,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=nogo i7_2=fl0 i1_7=Holds
PROBE l0root tree=tomb level=0 type=t0 c0=node c1=c16 red=[I-2.1,I-4.8,I-7.2,I-7.4] i2_1=nogo i7_2=fl0 i1_7=Holds
```

同一对副本只换盘，身份那一条（墓碑 I-9.23、谱系 I-9.17）一红一绿。出处：`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2374`：「let child_class = read_unit_without_judging(self.reader, &child, node_bytes())」，它取第一份 16 KiB 校验和对得上的（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:4376`：「pointer.locations.iter().find_map(|location| {」）；GPU 同形 `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3176`：「let child_unit = read_unit_via_locations(child, NODE_BYTES);」，CPU 与 GPU 一起翻，不是分叉。批量扫的 275 行（矩阵与层级 0 的根）里同一对副本换盘红的集合不同的只有 6 对，墓碑、谱系类型段 0 这 4 对之外，一对是 inode 类型段 0 往下走那一支（读回的是第一份 16 KiB 对得上的，盘序一换读回的那一份跟着换，红的集合跟着换；同一个读者读法），一对是 C739（第六节）。核对命令 `python3 ~/.local/share/singlefs/work/m3-batch31-code-r1-opus/sym.py <run.log>`（草稿目录里的辅助脚本，不进模型），末行原样「pairs-asym 6 rows 275」。

四句：
1. 分不分辨臂：不分辨第 31 批的改与不改（这一小句第 30 批就在，F-L0 没碰它）；分辨「按第一份读回的判」与两份都看的两种读法（下面 F-ID-a、F-ID-b，推的，没实现）。
2. 判别子看得见：两份都读得出，checker 当时两份都能看。
3. 判据字面：两读。读法甲——「子单元」就是读者读到的那一份：core 的读者同样取第一份校验和对得上的（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-core/src/recovery.rs:371`：「pub(crate) fn read_unit_via_locations<Reader: PoolReader + ?Sized>(」），(node, c16) 时 core 读到的就是码 2，I-9.23 报成立不是漏判。读法乙——I-9.23 那一行（`.claude/kb/invariants.md:311`，附录整行）「孩子头里的类与类型段说的那一类不一致时」那一段把两份副本都算进「孩子」，(node, c16) 里盘 1 那份头里是码 3，条目照判损坏，报成立就是漏判（口径同 m3-batch22-code-r1 正文第五节「漏判」那一句）。先例：m3-batch26-code-r1 判决第二节 K2 H1 H-V 把「结论随那一份在哪块盘翻面」当打中（那一形两份都自证得过；这一形 `c16` 那份自证不过，所以只算弱）。
4. 改法：F-ID-a 两份里任一份按 16 KiB 对得上、头里类标签不是 2 就判红（两种盘序都红）；F-ID-b 只认按自己那一类长度自证过的副本（`c16` 不算，两种盘序都绿）。两个都碰得到这 4 对；都是推的，没实现、没跑，被攻过零轮。

### A4　层级 0 的根

```
PROBE l0root tree=tomb type=rec c0=ok c1=ok untouched red=[] i2_1=- i7_2=- i1_7=Holds
PROBE l0root tree=lineage type=rec c0=ok c1=ok untouched red=[] i2_1=- i7_2=- i1_7=Holds
```

墓碑树、谱系树根头里层级 1 → 0（重封、补到根），条目不动：一条都不红。根之下直接是容器、根却说自己是层级 0，CPU 不判（`walk_container_tree_root` 不看根的层级），GPU 同样不看（`w_walk_tree_table_entry` 读根用 key_range_reading 0 之后直接进 `w_walk_container_tree_root`）。I-1.1 那一行（`.claude/kb/invariants.md:26`）对这两棵树根的层级没有写法，按字面不算漏判，记观察。层级 0 的根下类型段 0 的 50 形与层级 1 逐形同判（`MODEL` 那 257 行里含这 50 行），F-L0 的说明照写「层级 0 的节点有类型段 0 的条目：孩子会是层级 0 的码 2 节点」——层级 0 时那句话的后半句不对（孩子该是层级 −1），只是说明文字。

## 二、N2（GPU extent 上段根豁免）

读码：`pending_level` 只在进循环前是 NONE（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2184`：「var pending_level = NONE;」），往孩子走时改成父层级减一（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2314`：「pending_level = level - 1u;」），层级来自头里一个字节、父层级 ≥ 1 才有孩子，所以之后再不会是 NONE；读孩子失败回到父帧时不重进根。于是 `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2191`：「if (pending_level == NONE) {」只在读根那一次成立，与 CPU 只在 `position` 为 `None` 时用那一种读法一一对应（根由 `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2146`：「self.walk_extent_upper_node(&entry[14..100], None, tree);」进，孩子由 `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:3678`：「self.walk_extent_upper_node(child_pointer_bytes, Some(child), tree);」进）。豁免条件两边逐项相同：CPU `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:1499`：「(KeyRangeReading::PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller, 0)」加 `if view.entries.is_empty()`，GPU `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1515`：「} else if (key_range_reading == 3u && n.level == 0u && n.entry_count == 0u) {」，两边都只判 key 宽。

CPU 量（三个单元的文件 unlink 到底之后 extent 上段根兼叶、零条目，`PROBE-INFO extent-root level=0 entries=0 key_width=24 entry_width=113`；根头改一处、重封、补到根）：

```
PROBE extent shape=empty-l0-legal red=[] i2_1=- i7_2=- i1_7=Holds
PROBE extent shape=empty-l1-entry113 red=[I-1.1,I-1.10] i2_1=- i7_2=- i1_7=Holds
PROBE extent shape=empty-l1-entry110 red=[I-1.1] i2_1=- i7_2=- i1_7=Holds
PROBE extent shape=empty-l2-entry110 red=[I-1.1] i2_1=- i7_2=- i1_7=Holds
PROBE extent shape=empty-l0-entry112 red=[I-1.10] i2_1=- i7_2=- i1_7=Holds
PROBE extent shape=empty-l0-keywidth-tag red=[I-1.11,I-1.2,I-1.4,I-3.10,I-4.8,I-7.2,I-7.4] i2_1=- i7_2=other i1_7=Holds
```

GPU 期望（读码，用上面的 CPU 结果对）：层级 ≥ 1 而零条目——`key_range_reading` 是 3 但层级不是 0，落到 `w_check_internal_node_separators`，零条目判不成立 → I-1.1；`w_position_and_entry_width_hold` 里层级就是根自己的层级，条目宽不是 110 时 I-1.10 红并停：与 CPU 两行同（`entry113` 红 I-1.1、I-1.10，`entry110` 只红 I-1.1）。层级 0 零条目而 key 宽不对：两边都只比 `schema_key_width` 与头里的 key 宽，都红 I-1.1；改 key 宽这一字节同时挪了头的长度，CPU 在头那一步就判用不了（上面最后一行），GPU 的 `w_judge_unit_header` 判同一批头字段（这一格没逐字段对，推的）。

判定：没打中。什么会推翻：「删到最后一批」那条对拍流上 GPU 仍在第 156–164 个状态多红 I-1.1（提交时主 agent 跑的对拍），或一个 extent 上段根层级 ≥ 1 的状态 GPU 走了豁免。

## 三、N3（候选根自己那张实例表链 257 片、最新根的链途经它第 100 片）

读码走一遍最终状态（全部写都落了）：
1. 走读内核上下文 0 先走最新根（实例 3 最后一次暖机的根）的链：T 第 0 片 → 第 1 片（槽 199000）→ R 那条链的第 100 片。第 100 片先按读回的那一份标走过（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1624`：「if (!w_visit(device_read_from, slot_read_from)) {」），再判身份：容器号 100 不是链上第 2 片（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1647`：「let identity_holds = is_zero64(p.birth_tree) && p.record_type == 4u && eq64(p.container, page_index) && is_zero64(p.container_birth);」），I-1.1 红、记走读失败、停。只走了 3 片。
2. 候选根 R（实例 3 写行那次，txg 比最新根小；按最新根读到的第 0 片的行判有效——这一句推的，没逐行核那张表）在同一上下文里走自己的链：第 0–99 片，第 100 片已走过 → 合并、停。101 片，不到 256，`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1728`：「if (page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED) {」不成立，不记 61。实例 3 别的暖机根指 T，T 第 0 片已走过，立刻合并。
3. 扫描内核读一条根自己那张表的两处都逐片判身份（容器号等于片序号）再往下读：R 那张 257 片身份都对，读满 256 片还有下一片 → 站点 38（`r_instance_rows`）或 37（`r_instance_pages_after_first`），见 `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1131`：「for (var step = 0u; step < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; step = step + 1u) {」；最新根那条链读到第 2 片身份不对就停，到不了上限。

与 CPU 同序：CPU 也是先按读回那一份插走过（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:3174`：「if !self.visited_units.insert(placement_it_was_read_from) {」）、后判身份。崩溃在 R 落了、后面的暖机根没落的那几个状态，R 是最新根，走读内核走满 257 片记 61，扫描内核不跑，是那条用例允许的原因字。

判定：没打中（读码，GPU 没跑）。什么会推翻：那条对拍流上一个状态都没有 37 / 38（提交时跑）。正文问的「实例 3 写行那条候选根在不在回退候选集」：在（实例表行判有效、txg ≥ F）；「走读内核会不会先记 61」：最终状态不会。

## 四、观察 O3：「走过」按第 0 条位置条目认 —— 打中

### 造法（`probe_kbi.rs` 的 `o3_probe`）

在干净镜像上：① 树根第一条条目的孩子（墓碑容器 / 谱系容器 / inode 叶容器）头里记录宽减 1、重封、新校验和补到根（只这一处是 I-1.7 违例，对照行 `variant=none`）；② 再把那条条目子指针第 `collide` 条位置条目的槽改成这一遍先走过的一个码 2 节点 X 的槽（盘不动：位置条目 0 在盘 0、1 在盘 1，两盘同槽，I-2.5 照旧成立）。三种 `variant`：`fw` 校验和是 X 那 16 KiB 的 CRC（那一份自证成码 2，另一份是码 3 容器，走 F-W）；`bad` 校验和是坏的；`badself` X 就是这棵树自己的根、校验和坏的。X 在墓碑、谱系那两棵取 inode 树根（`x_kind=2`），inode 那棵取 extent 树根（`x_kind=1`），`badself` 取自己的根。`collide=loc0` 与 `collide=loc1` 是同一形把「指错的那条」放在盘 0 还是盘 1。

### 量到的（冻结副本，`base` 臂，墓碑原样；谱系、inode 同形，见 `rr4/base.probe` 里 `PROBE o3` 那 21 行）

```
PROBE o3 tree=tomb variant=none collide=none x_kind=2 red=[I-1.7] i2_1=- i7_2=- i1_7=Violated
PROBE o3 tree=tomb variant=bad collide=loc0 x_kind=2 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4] i2_1=read i7_2=- i1_7=Holds
PROBE o3 tree=tomb variant=bad collide=loc1 x_kind=2 red=[I-1.7,I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4] i2_1=read i7_2=- i1_7=Violated
PROBE o3 tree=tomb variant=badself collide=loc0 x_kind=10 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4] i2_1=read i7_2=- i1_7=Holds
PROBE o3 tree=tomb variant=badself collide=loc1 x_kind=10 red=[I-1.7,I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4] i2_1=read i7_2=- i1_7=Violated
PROBE o3 tree=tomb variant=fw collide=loc0 x_kind=2 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4,I-9.23] i2_1=read i7_2=- i1_7=Holds
PROBE o3 tree=tomb variant=fw collide=loc1 x_kind=2 red=[I-1.7,I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4,I-9.23] i2_1=read i7_2=- i1_7=Violated
```

三棵树 × 三种 variant 共 9 对，`loc0` 那一半 I-1.7 一律 `Holds`、`loc1` 那一半一律 `Violated`（`grep 'PROBE o3' rr4/base.probe` 数：`i1_7=Holds` 9 行，全是 `collide=loc0`）。别的红项两种盘序基本同红（I-3.1、I-3.11、I-5.1 是指错的那条位置条目按容器的两槽登记进了 X 的槽、真容器在那块盘上没人引用；谱系 `loc0` 另多 I-3.5）。

机理：容器那一支 `read_referenced_unit` 交回第一份校验和对得上的——盘 0 那份对不上，交回的是盘 1 那份（带违例的容器）；「走过」却按第 0 条位置条目认（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2597`：「let child_placement = (child.locations[0].device, child.locations[0].slot);」，`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2598`：「if !self.visited_units.insert(child_placement) {」）。盘 0 那条指的是先走过的 X，插不进去，当成「别的版本走过」合并、返回，读回来的容器一个字节都不判（I-1.3、出生身份、I-1.7 都在这之后）。inode 叶同形（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:2821`：「let child_placement = (child.locations[0].device, child.locations[0].slot);」），`read_index_node` 同形（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:1473`：「let placement = (pointer.locations[0].device, pointer.locations[0].slot);」，这一处我没单独造形，F-KEY 臂顺手改了）。实例表链那一处早改成按读回那一份认（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:3170`：「let placement_it_was_read_from = (」，m3-batch25 攻方 H-b），别处没跟。不是 F-W 才有：`bad`、`badself` 两种不走 F-W，照样中。

GPU 读码同形：容器那一支 `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3246`：「if (!w_visit(child.device0, child.slot0)) {」，inode 叶 `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2732`：「if (!w_visit(child.device0, child.slot0)) {」，`w_read_index_node` `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1494`：「if (!w_visit(p.device0, p.slot0)) {」；`w_read_referenced_unit` 同样交回第一份对得上的。按 CPU 算的期望：GPU 在 `loc0` 那 9 形上同样不置 I-1.7、`loc1` 那 9 形置；CPU 与 GPU 一起漏，不分叉。core 的读者同样取第一份校验和对得上的（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-core/src/recovery.rs:371`：「pub(crate) fn read_unit_via_locations<Reader: PoolReader + ?Sized>(」），它读到的正是盘 1 那份带违例的容器——checker 放过的恰好是读者会撞上的那一份。

### 四句

1. 分不分辨臂：分辨。今天的写法（按第 0 条位置条目认走过）在 `loc0` 9 形上 I-1.7 报成立；F-KEY 臂（下面）18 形全红。
2. 判别子看得见：看得见。读的时候就知道读回的是哪一条（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/image.rs:1517`：「pub fn read_referenced_unit_with_the_location_it_was_read_from(」，实例表链已经在用）。
3. 满足的分句：正文第五节指的 m3-batch22-code-r1 正文第五节「漏判」那一句——镜像违反 `invariants.md` 那一行的字面而 checker 报成立：I-1.7（`.claude/kb/invariants.md:32`）判记录宽那一步对「任一打包记录单元」，被条目引用、读者会读到的那片容器记录宽不是登记的宽，checker 报 `Holds`。镜像本身已经因 I-2.1 等是坏的，漏的是它里面另一处违例，且漏不漏随指错的那条在哪块盘翻面（O3 要的「盘序不同判得不同」）。
4. 改法在打中的格上还中不中：见下表。

| 改法 | 墓碑 `loc0` 3 形 | 谱系 `loc0` 3 形 | inode `loc0` 3 形 | 别的格 |
|---|---|---|---|---|
| F-KEY：容器、inode 孩子、`read_index_node` 三处「走过」按读回那一份的落点认（合并、父单元等别的键照旧按第 0 条，同实例表链那一处的写法） | 量过：I-1.7 `Violated` | 量过：I-1.7 `Violated`；`bad`、`fw` 两形另多红 I-4.9（`loc1` 那一半没有；原因没查） | 量过：I-1.7 `Violated` | 量过：PROBE 其余 286 行与 `base` 臂逐行相同；整个 `checker_known_bad_images` 187 条全过（`rr4/fix-key.kbi` 末行原样「test result: ok. 187 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out」） |
| GPU 三处 `w_visit` 同改（`w_read_referenced_unit` 要交回读的是哪一条，实例表链那一处已有现成写法） | 推的 | 推的 | 推的 | 推的；没跑 |
| 不改代码、只把「两份副本落点不同时按哪一份认走过」写进条款 | 不碰这几格（今天照样漏） | 同 | 同 | — |

F-KEY 臂的几行原样（墓碑）：

```
PROBE o3 tree=tomb variant=bad collide=loc0 x_kind=2 red=[I-1.7,I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4] i2_1=read i7_2=- i1_7=Violated
PROBE o3 tree=tomb variant=badself collide=loc0 x_kind=10 red=[I-1.7,I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4] i2_1=read i7_2=- i1_7=Violated
PROBE o3 tree=tomb variant=fw collide=loc0 x_kind=2 red=[I-1.7,I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4,I-9.23] i2_1=read i7_2=- i1_7=Violated
```

F-KEY 只在我的模型上量过、被攻过零轮；`extent` 的数据单元指针那一处（`walk.rs` 第 3373 行同一句键）与 inode 往下走的内部孩子没造形、没改。什么会推翻这一格：在冻结副本上 `collide=loc0` 那几形 I-1.7 报 `Violated`（比如主 agent 复跑时某处先把 X 的走过插成了别的键），或主 agent 认定 I-1.7 的「任一」不含「被引用、读者会读到、但另一份落点被别的单元占着」的那一片。

## 五、CPU 与 GPU 判得不一样的一形

没找到（读码）。逐支对过的：F-L0（第一节 A1）、类型段 0 那一支的身份小句（A3，两边同取第一份，一起翻）、F-W 两处（第六节）、extent 上段根（第二节）、实例表链（第三节）、O3 那三处走过的键（第四节，两边一起漏）。我起先疑 GPU `read_at` 只认落点起点、读不到一个 32 KiB 单元的后一半槽，查表之后不成立：盘上碰过的每个 16 KiB 槽都单独立落点（第一节 A1 末段那一句），只有没碰过的槽读不出，那是 F-零槽，不攻。什么会推翻：提交时第 31 批那几条对拍流或第 30 批 F-W 那几条有一个状态违例位不同。

## 六、第二次抽样

### E1　m3-batch30 F-W 的 CPU 一侧（inode / 墓碑 / 谱系，层级 1，盘 0 / 盘 1）

一份按类型段那一类自证过（`node` 对类型段 0，`ok` 对类型段 = 记录类型）、另一份自证成别类，两种盘序：

```
PROBE matrix tree=tomb level=1 type=rec c0=node c1=ok red=[I-2.1,I-4.8,I-7.4,I-9.23] i2_1=read i7_2=- i1_7=Holds
PROBE matrix tree=tomb level=1 type=rec c0=ok c1=node red=[I-2.1,I-4.8,I-7.4,I-9.23] i2_1=read i7_2=- i1_7=Holds
PROBE matrix tree=lineage level=1 type=rec c0=node c1=ok red=[I-2.1,I-4.8,I-7.4,I-9.17] i2_1=read i7_2=- i1_7=Holds
PROBE matrix tree=lineage level=1 type=rec c0=ok c1=node red=[I-2.1,I-4.8,I-7.4,I-9.17] i2_1=read i7_2=- i1_7=Holds
PROBE matrix tree=inode level=1 type=rec c0=node c1=ok red=[I-2.1,I-4.8,I-7.4,I-9.2] i2_1=read i7_2=- i1_7=Holds
PROBE matrix tree=inode level=1 type=rec c0=ok c1=node red=[I-2.1,I-4.8,I-7.4,I-9.2] i2_1=read i7_2=- i1_7=Holds
PROBE matrix tree=tomb level=1 type=t0 c0=node c1=ok red=[I-2.1,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=read i7_2=fl0 i1_7=Holds
PROBE matrix tree=tomb level=1 type=t0 c0=ok c1=node red=[I-2.1,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=read i7_2=fl0 i1_7=Holds
PROBE matrix tree=inode level=1 type=t0 c0=node c1=ok red=[I-1.1,I-1.2,I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.2,I-7.4,I-9.2] i2_1=read i7_2=other i1_7=Holds
PROBE matrix tree=inode level=1 type=t0 c0=ok c1=node red=[I-1.1,I-1.2,I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.2,I-7.4,I-9.2] i2_1=read i7_2=other i1_7=Holds
```

两种盘序同红；不多红 I-3.1、I-3.11、I-5.1（类型段 = 记录类型那几行），墓碑类型段 0 那一形走到 F-L0（`fl0`）。inode 类型段 0 那一形往下走的是写进叶容器槽的那份树根原样字节，它头里的层级、出生身份对不上，红 I-1.1、I-1.2 等是这份坏镜像自己的（读的就是那一份）。没打中。

### E2　F-W、F-IT 的 GPU 读码

F-W：GPU 两份都看、交回第一份认出的另一类（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3049`：「if (w_unit_verifies_at_the_length_of_its_class(first_copy, class_in_the_first_header, child.sum0)) {」起两段），与 CPU `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:4438`：「if Some(class_in_the_header) != class_the_entry_type_names」同；「另有一份是类型段那一类」两份都看（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3070`：「fn w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child: Ptr, class_the_entry_type_names: u32) -> bool {」，CPU `/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:4461`：「child.locations.iter().any(|location| {」）。容器树两处、inode 树一处的分支次序与 CPU 同（GPU inode 树只接层级 1 类型段 2，别的落判不了，第 31 批那条对拍流钉它）。F-IT：GPU 实例表链每片判 I-1.2 用码 3 偏移 73、93、103（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1645`：「w_judge(INVARIANT_I_1_2, eq64(rd64(s.base, 73u), pointer.birth_txg) && rd32(s.base, 93u) == pointer.instance」），CPU 同偏移（`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:983`：「birth_txg: read_u64(unit, 73),」，`/home/fy5090/.local/share/singlefs/work/m3-batch31-code-r1/tree/crates/singlefs-checker/src/walk.rs:986`：「tail: TailOfTheBirthIdentity::BirthSequence(read_u32(unit, 103)),」），都只判 I-1.2、不判 I-4.2。没打中。

### E3　m3-batch29 L1：一份读不出

`gone` 对每种另一份、三种类型段、三棵树都扫了（25 组合里含 `gone` 的 9 组），与期望模型逐行相同；两种盘序同红（第一节 A3 那 6 对之外没有翻面的）。例（类型段 7）：

```
PROBE matrix tree=tomb level=1 type=t7 c0=gone c1=ok red=[I-2.1,I-3.1,I-3.11,I-4.8,I-7.4,I-9.23] i2_1=nogo i7_2=- i1_7=Holds
PROBE matrix tree=tomb level=1 type=t7 c0=ok c1=gone red=[I-2.1,I-3.1,I-3.11,I-4.8,I-7.4,I-9.23] i2_1=nogo i7_2=- i1_7=Holds
PROBE matrix tree=tomb level=1 type=t7 c0=c16 c1=gone red=[I-2.1,I-3.1,I-3.11,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=nogo i7_2=nogo2 i1_7=Holds
PROBE matrix tree=tomb level=1 type=t7 c0=gone c1=c16 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-7.2,I-7.4,I-9.23] i2_1=nogo i7_2=nogo2 i1_7=Holds
```

（I-3.1、I-3.11 是读不出那条位置条目指到盘尾之外、引用按它登记、那块盘上的真容器没人引用，这份坏镜像造法带来的。）读不出那一份照红 I-2.1，另一份对得上就不记走读失败（`i7_2=-`），两份都对不上记（`nogo2`）。GPU：读不出交 false（第一节 A1）。没打中。合法状态多红那一格不攻（层 0）。

### E4　m3-batch29 L2：链长与候选集里外

读码，见第三节：候选集里的根与最新根同一上下文，链途经走过的片就合并；候选集外的根（被抛弃、F 之下）各走各的、走满记 61；扫描内核两处逐片判身份、读满记 37 / 38。没打中（读码，GPU 没跑）。

### 附带：C739 那一形的盘序翻面（不算新打中）

```
PROBE matrix tree=inode level=1 type=t7 c0=node c1=ok red=[I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4,I-9.2] i2_1=read i7_2=- i1_7=Holds
PROBE matrix tree=inode level=1 type=t7 c0=ok c1=node red=[I-2.1,I-4.8,I-7.4,I-9.2] i2_1=read i7_2=- i1_7=Holds
```

inode 类型段 7、两份各自证成码 2 与码 3：inode 树先问孩子自证成哪一类、取第一份认出的那一类定登记槽数（码 2 一槽 / 码 3 两槽），盘序一换 I-3.1、I-3.11、I-5.1 跟着翻；墓碑、谱系类型段 7 按层级定槽数，两种盘序同红。这正是 I-9.2 那一行点名的 C739（两份各自证成不同类时取哪一类、按哪一类登记槽数没有条款），已登记，记一个量到的形状给它，不算这一轮的打中。

## 七、没打中的形状（试过什么、取样多大）

| 形状 | 取样 | 结果 |
|---|---|---|
| F-L0 每份照判 I-2.1，五种副本状态两两组合 | 墓碑、谱系各 25 形（层级 1）+ 各 25 形（层级 0 的根） | 与期望模型逐行相同；只 (c16, node) 那一对翻面（A3） |
| 类型段 = 记录类型、类型段 7 两支同一组副本 | 三棵树 × 2 × 25 = 150 形 | 与模型相同；inode 类型段 7 一对翻面归 C739 |
| inode 类型段 0（往下走，F-W 之后） | 25 形 | 不在模型里；两种盘序只 (c16, node) 一对不同，读者读法 |
| extent 上段根：零条目层级 0 / 1 / 2、条目宽 110 / 112 / 113、key 宽字节 | 6 形 | CPU 与 GPU 读码同判 |
| GPU 读不到 32 KiB 单元后一半槽 | 读码 | 不成立：碰过的槽都是落点 |
| 实例表链 257 片、候选集里 / 外 | 读码 | 与对拍用例的期望一致 |
| 干净镜像 | 3 份（墓碑、谱系、inode 那三份起点） | 一条都不红 |

## 八、这条腿自己的限度

- GPU 一侧全是读码推的，没在卡上跑；第 31 批那几条 GPU 对拍、GPU 那 8 行变异归提交时。
- 合法状态多红（层 0 崩溃枚举）按派发不攻、不跑；我跑的只有 3 份干净镜像与 `checker_known_bad_images` 原有 174 条（全过），说明不了合法流上 F-L0 走不走得到。
- 期望模型是看过墓碑类型段 0 那 25 行之后写的，不是盲测；它与 CPU 共用我对分支的理解，只有 `mut-fl0` 那条臂证明比对会红。
- O3 只造了容器、inode 叶两处（`read_index_node` 那一处只在 F-KEY 臂里改了、没单独造形；extent 数据单元指针那一处没碰）；F-KEY 在谱系 `loc0` 两形多红 I-4.9，原因没查。
- A3 的两个改法 F-ID-a、F-ID-b 没实现、没跑。
- 所有数都是冻结副本的拷贝上量的；副本上的数不算入库装置上的数。层 0 状态数：本腿没调 `enumerate_layer0` / `enumerate_layer0_versions`，0 个状态。

## 九、没做什么

- 不判正推那几格（K1、K3）；不替主 agent 采纳；没改仓里任何 crates 文件。
- 没跑 GPU、没跑 checker 档任何测试、没跑门禁阶段。
- 第 31 批正文第一节列的不判项（F-零槽、扫描内核第 12 行那句说明、展开数钉值、m3-batch30 判决第三节、GPU 对拍与变异）没攻。

## 草稿与编译目录

草稿目录 `~/.local/share/singlefs/work/m3-batch31-code-r1-opus/`。交回前删了自己建的仓副本与编译目录（`copy/`、`target/`、`rr1/`–`rr3/` 整个、`rr4/` 里三份 crates 副本与三个 `target-*`）；留下 `rr4/` 里的 `*.probe`、`*.model`、`*.kbi`、`*.log`、`summary.txt`（第一节起引的 PROBE 行出处）与辅助脚本 `sym.py`。这些是攻方草稿，不入库；复跑以模型目录为准。

## 附录：kb 原行（`.claude/kb/invariants.md` 自己的行号，整行抄）

第 130 行（I-2.1）与第 311 行（I-9.23）：

```
| I-2.1 | 校验和与内容匹配 | 任一被引用的块，其校验和与内容匹配。按多长去比（父条目的类型段说的那一类与孩子头里的类标签不一致时）：走读往下走孩子的那一支，按父条目类型段说的那一类的长度读回比；不往下走孩子的几支（墓碑、谱系、inode 三棵树的类型段不认得、孩子自证成另一类），每条位置条目按那一份头里类标签说的那一类的长度比，类标签不在登记表里或头读不出的，16 KiB 与 32 KiB 两种长度都对不上才算不匹配。同一块、同一对校验和，两支可以判得不同（用户 2026-10-06 定保持，`research/prompts/m3-batch29-code-r1-main-verification.md` 第四节） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判——不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22（刚释放的块立即重分配） 的必红） |
| I-9.23 | 墓碑树条目身份与子头相符 | 墓碑树（D18（块里携带什么信息） 已定项 24）每个内部条目的类型段 ∈ {0, 1}：0 ⇒ 子单元是码 2（根长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；1 ⇒ 子单元是码 3，子头 (出生树, 打包记录类型, 容器号, 容器出生代) 与身份引用四段逐字相等、子指针头部出生树 == 身份引用出生树、子容器打包记录类型 == 1。孩子头里的类与类型段说的那一类不一致时（条目照判损坏之后）怎么往下认：两份副本里有一份头里类标签是类型段说的那一类（类型段 0 是码 2 节点，类型段 1 是码 3 容器）、按那一类长度的整单元校验和与它的位置条目对得上，就照类型段那一支往下走那一份，引用登记的槽数按那一类；另一份按同一长度读回比校验和，对不上照红 I-2.1（校验和与内容匹配），I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用） 跟着；一份都没有的不往下走，每份副本照 I-2.1（校验和与内容匹配） 那一行的长度读法判。两份各自证成两种不同的别类时取哪一类还没有条款，记在 C739（认孩子的类时两份副本各自证成不同类，取哪一类、按哪一类登记槽数没有条款）（`research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1、4 条，`research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M1 那一行）。身份不符单独判定，不再记走读失败；与 I-1.6（类标签登记且三处相等）、I-1.7（打包容器合法与判定顺序） 重合的小句仍留作合取。与 I-9.2（条目身份与子头相符） 同形（墓碑正推报告 `research/prompts/m3-tomb-r1-sonnet-output.md` 第 241 行 N1；根长高之后类型段 0 那一支是主 agent 按 K-R3-3 补的，被攻过零轮） | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
```
