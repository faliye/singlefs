# m4-c860-r1 正推腿报告（Sonnet，2026-10-19）：扫描重建认「删除中」凭哪一侧

## 0　开工核对与复跑

开工快照 `research/prompts/m4-c860-r1-snapshot/`：被判的四份代码文件 `sha256sum -c` 全 OK；kb 九份里 `.claude/kb/decisions/08-核心索引结构.md` 对不上（别的会话在开工之后改过它：文件头加了语言链接行、树种类码登记表与快照树表条目几处，与已定项 16 无关）。下面引这份文件的行号按写报告时现查的，不按快照。

```
$ sha256sum -c research/prompts/m4-c860-r1-snapshot/tree-sha256.txt
crates/singlefs-checker/src/rebuild.rs: OK
crates/singlefs-checker/src/walk.rs: OK
crates/singlefs-checker-tier/src/crash_judge_checker_scan_rebuild.wgsl: OK
crates/singlefs-harness/tests/scan_rebuild_reads_lineage_and_retirement_records.rs: OK
$ sha256sum -c research/prompts/m4-c860-r1-snapshot/kb-sha256.txt | grep -v ': OK$'
.claude/kb/decisions/08-核心索引结构.md: FAILED
```

C860 镜像补丁与实现员报告核过：`~/.local/share/singlefs/work/m4-c860-writer/patch/crates.patch` 的 sha256 `d5ac0e9a6954a3c3bcf5a0e11770517326ad75d5130d6799b20e1010ddd93e05`、`report.md` 的 sha256 `2e01ecbbda016f2bd1e3f8798b6e4a708cdbcb1fe7a0ac4d11230219c7518184`，都与派发提示给的一致；该补丁对主工作区此刻的树 `git apply --check` 通过。

### 模型目录与复跑

模型目录 `research/prompts/m4-c860-r1-sonnet-model/`。模型不是独立重写的计数模型，而是在**仓的真实现上**加一个环境变量开关（`SONNET_RECOGNITION`）切五种认法，再用仓里已有的 harness 脚手架造世界、数 I-1.13 判什么；开关补丁与世界矩阵都在模型目录里，不进 `crates/`。

复跑（整条超过前台 240 秒，我没整条跑过；各步在我的草稿副本里分开跑过，输出在 `out/`）：

```
bash scripts/hosts/run-with-memory-cap.sh 12G bash research/prompts/m4-c860-r1-sonnet-model/tools/rerun.sh \
  --repo <仓根> --frozen <主工作区此刻的树的拷贝> --work <新的草稿目录> [--model research/prompts/m4-c860-r1-sonnet-model]
# 再要已有用例回归：同一条命令前加 SONNET_RERUN_REGRESS=1
```

| 文件 | sha256 |
|---|---|
| tools/rerun.sh | 2485138c98f6d843ab262f0a6b1bb1343079a8fb698a66eb4fcbc80b89f65567 |
| patch/c860-writer-crates.patch（实现员那份，原样拷） | d5ac0e9a6954a3c3bcf5a0e11770517326ad75d5130d6799b20e1010ddd93e05 |
| patch/recognition-variants.patch（`rebuild.rs`、`walk.rs` 的开关） | 94c4d083ad98f1434c97e270ec01f3479f0f972c4ac33465d520f54014f93326 |
| patch/matrix.rs（世界矩阵，附在 lineage 测试文件末尾） | bf1a1f034b4b89a0bd0d13da416fa16d10b38c44adf636946722e2e69539ad24 |
| out/matrix-jia.log | 9ea859f43dd530a6a7858e6cd8d692e64b5de525485dd02bf533d0229e579865 |
| out/matrix-yi.log | 2eb4714375a144e0789dc83c880f5c7b0e3551242e5d04bd829fe92fcf3699d0 |
| out/matrix-bing.log | 5b5f2ddceddd661ef5ea30b6b9ea32f69bb798b2ac88a6a1e3acdc83443a9a74 |
| out/matrix-ding.log | eac76d544cd5e3d6e5e508166749ac71edb8ad7fe91175af6bce236ef88e2a6f |
| out/matrix-ding2.log | 53da52a97802f971651b9b09f589e533229221f5858c1c1ec615aeb8dd351b22 |
| out/matrix-trace-jia.log | 2c488fdd15e06516cb9f9a6c58a9a794a049fff803f5c17be6f226c49609c1e9 |
| out/random-jia.log | da1c50a96aecb2d47f8c729c0b18e39e3c329d5933f70dd07f3a2d3b50f94447 |
| out/random-yi.log | 89077d764cf107246fda2f586acc6c6af9c4fee80e66f7bb106588060a346c80 |
| out/random-bing.log | d48901aff60075e7d7179ba00f60f77b1d8fb2de2886834cfc293ec2bdc71167 |
| out/random-ding.log | 00b1d071dbcb4d647ab4f09ab9eb48fe717aa48f83acd2652c8722c78fdc9d75 |
| out/random-ding2.log | d866e83ab73fe5188983f33e989a8e4f56d125f82f16ae005dc797dd0975b81c |
| out/regress-summary.txt | 24e882be5cd129a2e6144567858ff778dd9911b7767d3a51500d0ad284f2621f |

准备步骤（拷树、打两份补丁、附矩阵、补 `mod` 行）我另在一个空目录里按 `rerun.sh` 里同样的命令跑过，三份结果文件与草稿副本逐字节相同（`cmp` 无输出）。

## 1　结论摘要（正推，下面各节逐格给依据）

- 今天的认法（下称甲）与它的三个改法（乙、丙、丁，定义见 K1）在**合法盘**上没有差别：14 个造出来的合法世界、1736 个随机历史状态（其中 241 个在删过文件之后）、30 个调池级 checker 的 harness 测试目标里的既有用例，五种读法下 I-1.13 的结局相同（K2）。
- 四种读法只在一类坏盘上分出高低：「扫描一侧有墓碑压着的记录、遍历方向一侧没有这条记录、别的表都对上」。造得出的只有 C860 那一形（两处坏法：抹退役记录加补回映射条目），甲判成立，乙、丙、丁判红（K3）。同一张镜像上 I-9.35、I-3.9、I-3.11 判违例（实现员观测，我在副本里复现）；根环转过一圈之后 I-9.35 不再判（它只在父版读得出时判），I-3.1、I-3.9、I-3.11 仍判（K3 的 X10）。
- 判决 M5 的字面没写「删除中」由哪一侧认，也没写遍历方向没有这条记录时扫描一侧该不该去掉；I-1.13 定义列写的是「inode 记录集合」逐项相同，状态列才复述 M5（K4）。
- 推荐乙：只在扫描一侧去记录这一步加上「遍历方向也有同号同出生代的记录」，改动最小、与 M5 字面最近，合法盘零误红，C860 那一形判红。丁（集合比对）多拦的几形已由 I-9.5 / I-9.18 / I-9.19 拦，且让 7 条既有的「只红某一条」隔离用例转红，不推荐（K3、K5）。
- 乙要改 `rebuild.rs` 一处、`walk.rs` 一处、GPU 内核一处，既有用例转红一条，变异行三处要动；代价估算在 K5。
- 本腿没造出、也没排除的：一个单一 bug（退役这一步整个被跳过）造成的镜像，根环转过之后 I-9.35 不判、I-3.x 记账一致，这时只有乙、丙、丁能红。这是甲与乙之间唯一实质的岔路，写进岔路单（第 7 节）。

## 2　K1　候选认法的定义

先说清两侧各自读什么。扫描一侧：`rebuild.rs` 在当前版本 inode 叶容器里收记录进 `inode_records`，在谱系上有死亡 txg 的整对象墓碑里收 `tombstoned_objects`；遍历方向一侧：`walk.rs` 从最新根的 inode 树读 `walked.inode_records`。

甲（今天）。扫描一侧把 (号, 对象出生代) 同时在自己的记录和墓碑集合里的对象认作删除中，集合名 `objects_being_deleted`：

- 「.filter(|object| tombstoned_objects.contains(object))」（`crates/singlefs-checker/src/rebuild.rs:760`）
- 「inode_records.remove(inode);」（`crates/singlefs-checker/src/rebuild.rs:763`）
- 「if tombstoned_objects.contains(&(inode, object_birth)) {」（`crates/singlefs-checker/src/rebuild.rs:777`）：墓碑压着的对象的数据单元不认领，不分记录还在不在，这一句不属于「删除中」的认法，四种读法都不动它。
- 遍历方向一侧只在自己有同号、同出生代的记录时才去：「.contains(&(**inode, read_u64(record, INODE_RECORD_OBJECT_BIRTH_OFFSET)))」（`crates/singlefs-checker/src/walk.rs:14929`），去记录「walked.inode_records.remove(inode);」（`crates/singlefs-checker/src/walk.rs:14934`），去该号全部数据单元集合项与那些项的映射条目「walked.central_mapping.remove(key);」（`crates/singlefs-checker/src/walk.rs:14946`）。

乙。认法在甲之上加一条：遍历方向一侧也有同号、同出生代的记录才算删除中。两侧各去掉什么：

- 扫描一侧：只有遍历方向有同号同出生代记录时才去它的记录；否则保留，于是「inode 记录集合」差一处。
- 遍历方向一侧：同甲（记录、该号的数据单元集合项、映射条目）。
- 实现上的区别只有一处：扫描一侧去记录的那一步从 `rebuild.rs` 挪到 `walk.rs` 里遍历方向去记录的同一个循环（实现员的对照改法、我的 `yi` 开关都是这样写）。

丙。记录不去，单元与映射去：

- 扫描一侧不去记录，遍历方向一侧也不去记录；两侧的 inode 记录照常逐项比（记录原样，宽与字节）。
- 数据单元集合项与映射条目的去法同甲（遍历方向按「扫描认出的集合 ∩ 遍历方向有同号同出生代记录」去）。
- 理由：删除中的对象，两侧读的是同一片叶容器里的同一条记录，合法历史上记录必然逐字节相同，去掉它没有必要；真正会不同的只有单元与映射（分批释放到一半，墓碑已判单元死，遍历方向的 extent 树里还有）。

丁（集合比对）。两侧各自认，认出的集合必须相同：

- 扫描一侧：墓碑集合 ∩ 自己的记录（同甲的 `objects_being_deleted`）。
- 遍历方向一侧：自己的记录里 nlink = 0 的 (号, 对象出生代)，依据 D8 已定项 16 第 3 条「队列 = inode 树里 nlink = 0 的号集合」（`.claude/kb/decisions/08-核心索引结构.md:486`）。
- 两个集合不等本身判 I-1.13 违例（多一张「删除中的集合」比对表）；相等时各去各的单元与映射，记录照比（同丙）。
- 变体丁（改报不适用）：集合不等、且 I-9.5 / I-9.18 / I-9.19 之一已违例时，I-1.13 报不适用并点名它，照 `INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED` 的先例（「pub const INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED」，`crates/singlefs-checker/src/walk.rs:14801`）；这个先例的取用处是「if let Some((_, reason)) = INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED」（`crates/singlefs-checker/src/walk.rs:14862`）。

考虑过没实现的：把 I-9.35 加进上面那张「遍历方向已损坏」表，让 I-9.35 红时 I-1.13 报不适用。它把 C860 那一形从「成立」改成「不适用」，I-1.13 照样不红，比乙弱，不作候选。

判定（K1）：规则没说。M5 的原句没有写由哪一侧认（K4 并排抄）。

推翻 K1 这一节的现象：`crates/` 里另有别处认删除中的代码（我只 grep 到 `rebuild.rs`、`walk.rs`、GPU 内核三处，命令：`grep -rn 'objects_being_deleted' crates --include=*.rs --include=*.wgsl`，命中 `rebuild.rs` 120、757、762、828、1551、1587，`walk.rs` 14928，wgsl 1147、1257、1269、1300、1302，共三个文件）。

## 3　K2　合法盘上会不会误红

### 推出来的

四种读法在**合法盘**上的差，只可能出在下面这个集合 R 上：扫描一侧有 (N, b) 的记录、谱系上有压着它的墓碑、遍历方向的 inode 树里没有同号同出生代的记录。四种读法对不在 R 里的删除中对象处理相同（两侧都有记录：甲、乙去掉，丙、丁留下比，两侧记录是同一片叶里的同一条，逐字节相同）。所以问「哪一种在哪一形上误红」就是问「合法历史上 R 会不会非空」。

R 非空要求扫描一侧选中的那一版叶容器，遍历方向最新根下的 inode 树不引用，或引用的版本里没有这条记录。合法历史里让扫描一侧选的叶与树引用的叶不同的机制只有三个，每个都有一道专门的滤：

- 叶删空退役：扫描一侧按退役树不认领（`rebuild.rs:727` 起「if let Some(death_txg) = retired_leaf_containers」那一支）。退役记录的死亡 txg 不在谱系上（挂着回退过）时，叶在遍历方向的树里是活的，记录两侧都有。
- 多版叶：同一叶身份的多个物理版本取诞生代号在谱系上、写序键最大的一版（`rebuild.rs` 的 `latest_of_the_ordered_versions` 那一步），回退抛弃的那一段里诞生的版本不进。
- 墓碑树不按谱系滤版本、按死亡 txg 滤：死亡 txg 在谱系上才算。

另一个常被问的合法形——「多个文件同时在删除中」——造不出来：D8 已定项 16 第 6 条「任何一版里 `deleted_inodes` 至多一条 key」（`.claude/kb/decisions/08-核心索引结构.md:489`），核心层在发意图之前断言队列已走空：「发意图之前队列已走空：任何一版的 deleted_inodes 至多一条 key（D8 已定项 16 第 6 条）」（`crates/singlefs-core/src/transaction.rs:9445`）。我第一次造两个文件交错删，就是在这句断言上 panic 的（矩阵第一版的 L03–L05，日志里 `发意图之前队列已走空`）。所以合法历史里同一时刻 `objects_being_deleted` 至多一个元素，「多文件交错」只有「删序不同」（先删谁）。

删除中的对象 N 的最后一批「删 N 的 inode 记录」与队列、单元释放在同一次发布里（D8 已定项 16 第 2 条：「最后一批释放 N 剩下全部单元、删 N 的 inode 记录、删 `deleted_inodes` 那条 key、把 N 的条目从上段去掉（不写标签 0），同一次发布」，`.claude/kb/decisions/08-核心索引结构.md:485`）。所以不存在「记录没了、单元还在」的合法中间态（推的，依据是这一条加上量到的 L01–L06 都没有这一形；I-4.7 把「extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号」也算删除中，见 `crates/singlefs-checker/invariants/06-事务与崩溃一致性.md:13`，那是它防坏盘的口径）。

### 量到的（每个世界的造法在 `patch/matrix.rs`，输出在 `out/matrix-*.log`）

「认出」一列来自 `out/matrix-trace-jia.log` 的 `TRACE scan_recognized=…`，空表示这个世界里没有对象被认作删除中（它只验证「没有凭空多出的 R」，不验证认法）。

| 世界 | 造法 | 认出 | 甲 | 乙 | 丙 | 丁 | 丁（改报不适用） |
|---|---|---|---|---|---|---|---|
| L01 | 三单元文件，意图之后 | {(1,3)} | 成立 | 成立 | 成立 | 成立 | 成立 |
| L02 | 同上，第一批之后 | {(1,3)} | 成立 | 成立 | 成立 | 成立 | 成立 |
| L03 | 同上，另有活着的空 inode 2 | {(1,3)} | 成立 | 成立 | 成立 | 成立 | 成立 |
| L04 | 先把空 inode 2 删到底，再开始删 1 号 | {(1,3)} | 成立 | 成立 | 成立 | 成立 | 成立 |
| L05 | 先把 1 号删到底，再开始删空 inode 2 | {(2,5)} | 成立 | 成立 | 成立 | 成立 | 成立 |
| L06 | 233 个空 inode 之后删 234（右片叶只装这一条），意图之后 | {(234,4)} | 成立 | 成立 | 成立 | 成立 | 成立 |
| L07 | 同上删到底（叶删空退役） | 空 | 成立 | 成立 | 成立 | 成立 | 成立 |
| L08 | 删 2 号到底（左片吸收右片） | 空 | 成立 | 成立 | 成立 | 成立 | 成立 |
| L09 | 三单元文件意图加一批，挂着回退到意图那版 | {(1,3)} | 成立 | 成立 | 成立 | 成立 | 成立 |
| L10 | 删 234 到底，回退到删之前 | 空 | 成立 | 成立 | 成立 | 成立 | 成立 |
| L11 | 删 234 到底，回退到意图那版（退役记录的死亡 txg 不在谱系上） | {(234,4)} | 成立 | 成立 | 成立 | 成立 | 成立 |
| L12 | 建快照后删 234 到底 | 空 | 成立 | 成立 | 成立 | 成立 | 成立 |
| L13 | 建快照后三单元文件意图加一批 | {(1,3)} | 成立 | 成立 | 成立 | 成立 | 成立 |
| L14 | 回退抛弃的那段里诞生又被删的对象 | 空 | 成立 | 成立 | 成立 | 成立 | 成立 |

- 14 个世界里 9 个真有对象被认作删除中，5 个是「已删完或回退越过」，只验证没有凭空多出来的 R。L11 是 R 最容易非空的合法形（叶有退役记录但不在谱系上），五种读法都成立。
- 随机历史计数（既有慢档用例，每种读法一遍）：1736 个状态全是「成立」，删过文件之后的 241 个全是「成立」，违例 0、不适用 0：

```
$ grep -h SCAN_REBUILD_TALLY_SUMMARY out/random-{jia,yi,bing,ding,ding2}.log
SCAN_REBUILD_TALLY_SUMMARY histories=56 states=1736 states_after_a_deletion=241 violated=0
（五份相同）
```

- 已有用例回归（30 个调池级 checker 的 harness 测试目标，三段各 10 个，五种读法各一遍；`out/regress-summary.txt`）：甲 0 条红；乙、丙各红 1 条，丁（红）红 7 条，丁（改报不适用）红 1 条。红的是：
  - 五种读法里除甲外都红：`erasing_the_retirement_record_reddens_the_scan_rebuild_after_the_right_leaf_retired`，红在它自己断言「不出现 inode 记录集合」那一句（「!detail.contains("「inode 记录集合」") && !detail.contains("「每个 inode 的数据单元集合」"),」，`crates/singlefs-harness/tests/scan_rebuild_reads_lineage_and_retirement_records.rs:244`）；这句断言就是照甲写的。
  - 只有丁（红）另红 6 条「只红某一条」的隔离用例（队列与墓碑相关：`a_queue_number_whose_inode_record_has_nlink_one_reddens_the_deletion_queue_invariant`、`an_inode_record_with_nlink_zero_outside_the_queue_reddens_only_the_deletion_queue_invariant` 等），因为丁在这些镜像上让 I-1.13 也红。

未覆盖的合法形（照实写）：最后一批之前那一版（只剩一个单元，与 L02 同形，没有单独造）；挂载续做（续做的起点就是上面这些崩溃态的盘，没有单独造）；逐崩溃点的全枚举是重型（checker 档崩溃点重放，我不能跑，见「要主 agent 跑的命令」）。

判定（K2）：一致——五种读法在已造的合法世界、随机历史与既有用例上结局相同，唯一差别是既有用例里那一条按甲写死的断言。推导说 R 在合法盘上恒空，量到的与它相容，但推导里「合法历史上让扫描选的叶与树引用的叶不同的机制只有三个」是我读代码得的，没有穷举证明。

推翻它的现象：任何一条合法历史（崩溃点重放的某个崩溃态、挂载续做、回退、快照）上 `TRACE scan_recognized` 含 (N, b) 而遍历方向的 inode 树没有 (N, b)，且别的表都对上——那是乙、丙、丁的误红。

## 4　K3　坏盘上会不会放过

### 量到的

造法在 `patch/matrix.rs`；「其余违例」是同一镜像上除 I-1.13 外判违例的不变量（甲读法下取的，其余读法下这一列相同）。

| 世界 | 坏法 | 甲 | 乙 | 丙 | 丁 | 丁（改报不适用） | 其余违例 |
|---|---|---|---|---|---|---|---|
| X01 | C860 镜像：抹退役记录，再把退役叶最后一版的映射条目补回最新根的中央映射叶 | 成立 | 违例 | 违例 | 违例 | 违例 | I-3.9、I-3.11、I-9.35 |
| X10 | 同 X01，镜像做在删完之后再发 30 次建 inode（根环转过一圈）的池上 | 成立 | 违例 | 违例 | 违例 | 违例 | I-3.1、I-3.9、I-3.11（I-9.35 不再判） |
| X02 | 只抹退役记录 | 违例（中央映射差 1 处） | 违例 | 违例 | 违例 | 违例 | I-9.35 |
| X03 | 抹谱系记录（挂着回退过的池） | 违例 | 违例 | 违例 | 违例 | 违例 | 无 |
| X04 | 2 号删除中，墓碑改指活的 3 号 (号, 出生代) | 成立 | 成立 | 成立 | 违例 | 不适用 | I-9.18、I-9.19 |
| X05 | 2 号删除中，墓碑改指不存在的号 | 成立 | 成立 | 成立 | 违例 | 不适用 | I-9.6、I-9.18 |
| X06 | 2 号删除中，墓碑出生代差一 | 成立 | 成立 | 成立 | 违例 | 不适用 | I-9.10、I-9.18、I-9.29、I-9.30 |
| X07 | 三单元文件意图之后把 nlink 改回 1（活对象带墓碑） | 成立 | 成立 | 成立 | 违例 | 不适用 | I-9.5、I-9.19 |
| X08 | 三单元文件意图之后抹掉它的记录，留单元 | 违例 | 违例 | 违例 | 违例 | 违例 | I-9.5 |
| X09 | 墓碑多一条逐字相同的重复 | 成立 | 成立 | 成立 | 成立 | 成立 | I-9.20 |

对着 K3 点名的几形：

- 退役记录丢失：单独坏它（X02），四种读法都在「中央映射」红，因为扫描一侧把那片退役叶当现行、多出它的码 3 容器 key；已有用例钉着。要让 inode 记录成为唯一的差，得再补回映射条目（X01）。
- 谱系记录丢失（X03）：四种读法同红，红在「中央映射」（扫描一侧没了谱系，选中被回退抛弃的版本）。
- 遍历方向丢了一条删除中对象的记录（X08）：这个造法让两侧一起丢（记录在叶里，两侧读同一片叶），单元还在遍历方向的 extent 树里，五种读法都红在单元集合。没有单元的删除中对象（空 inode）丢记录这一形，我没造出来（`drop_the_record_of_the_first_file_in_the_leaf_at_and_keep_its_data_units` 断言叶里是 1 号与 2 号，不适用空 inode 的叶）；推的：这一形记录两侧都没有，五种读法都成立，由 I-9.5 拦（队列有号、树里没有记录，I-9.18 对这一形报判不了，I-9.5 才红）。
- 扫描一侧多出一条被墓碑压着的旧记录：X01、X10。这是唯一把甲与其余分开的一形。
- 墓碑本身坏了或多了：X04–X07、X09。甲、乙、丙全部成立，丁（红）红，丁（改报不适用）报不适用；这几形已经由 I-9.5、I-9.18、I-9.19、I-9.20 等判红（表最后一列）。

### 「兜住算不算够」

- X01 上兜住它的三条里，I-9.35 的判法明写「父版在回退候选集里读得出才判」（`crates/singlefs-checker/invariants/09-inode树结构.md:47`，行里的原句：「父版在回退候选集里读得出才判」）：根环转过一圈之后它不判，X10 里已经看到 I-9.35 不在红名单里。
- X10 里还红着的 I-3.1、I-3.9、I-3.11，红因都是那次「补回映射条目」让一个已释放的槽又被中央映射引用。这是镜像构造带进来的，不是「退役这一步被跳过」本身的后果。
- 所以「退役这一步整个被跳过」（不写退役记录、不摘映射条目、槽仍记已分配）的单一 bug 镜像，在根环转过一圈之后：I-9.35 不判，I-3.x 记账与引用一致，推的结论是只有 I-1.13（乙、丙、丁）能红。我没造出这张镜像（要同时改记账行与分配记录，超出本轮的预算），所以这一句是**推的**。这是甲与乙之间唯一实质的岔路，写进第 7 节的岔路单。
- I-1.13 的判别力是不是独立要的：从定义看要。它是扫描方向与遍历方向两套读法互验的那一条（`crates/singlefs-checker/invariants/01-块自描述.md:18` 里的「扫描方向与遍历方向共享的只有格式解析」），把它的唯一一处不对称留给别的不变量兜，就是让互验的判别力依赖那几条不变量的存续与适用范围。这是设计判断，不是量到的数。

判定（K3）：冲突——甲在 X01、X10 上放过，乙、丙、丁判红；同一镜像的其余违例在 X01 上三条齐全、在 X10 上少一条（I-9.35）。

推翻它的现象：（一）在 X01 同形的镜像上，乙或丙判成立（说明我的开关实现与「乙」的定义对不上）；（二）找到一张合法历史镜像上乙红（K2 的推翻条件）；（三）找到另一张镜像，甲红而乙成立（则乙不是严格更严）。后两条我都没找到：矩阵里没有甲红而乙成立的世界。

## 5　K4　判决 M5 的文意与 I-1.13 的定义

并排（左：条款；右：今天的实现，同一句话的两面）。

- M5 判决：「两侧都去掉它的映射、单元与记录，别的对象照比；必红世界：别的文件删除中途改坏一条映射，I-1.13 红。GPU 扫描重建内核（C821 那份）同步改。」（`research/prompts/m3-multi-file-unlink-r1-main-verification.md:41`）
- I-1.13 状态列复述：「两侧都去掉它的映射、数据单元与记录，别的对象照比」（`crates/singlefs-checker/invariants/01-块自描述.md:18`）
- 遍历方向一侧的实现注释：「别的对象照比；出生代对不上的号不去掉（两侧照比、对不上就红）。」（`crates/singlefs-checker/src/walk.rs:14922`）
- C860 自己写的缺口：「遍历方向那一侧只摘出生代对得上的号，两侧不对称判决没写到」（`.claude/kb/checks-owed.md:574`）

读法：

- M5 与定义列都没写「删除中」由哪一侧认，也没写「遍历方向没有这条记录」时扫描一侧去不去；「两侧都去掉」在甲、乙、丙里都可以读成真（甲：扫描一侧认了就两侧去；乙：两侧都有才两侧去；丙：单元与映射两侧去，记录不去）。所以 M5 的字面不蕴含其中任何一种。判定：**规则没说**。
- 定义列写 inode 记录集合是「inode 号 → 当前版本 inode 叶容器里那条记录原样」（「inode 记录集合（inode 号 → 当前版本 inode 叶容器里那条记录原样）」，`crates/singlefs-checker/invariants/01-块自描述.md:18`），没有在定义里给删除中的对象开例外；例外只出现在状态列复述 M5 的那一句里。字面上最近的是丙（记录始终进比较），但 M5 判决自己的字面是「去掉记录」，所以丙要改判决的字面，乙与甲不用。
- 乙会不会让 M5 的必红世界（别的文件删除中途改坏一条映射）的判法变化：不会。该用例 `moving_a_mapping_location_of_another_file_while_the_first_file_is_half_deleted_reddens_the_scan_rebuild` 在甲、乙、丙、丁、丁（改报不适用）下都是 `ok`（`out/regress-summary.txt` 的 c3 段，五份日志里各一行 `... ok`）。乙只改「遍历方向没有这条记录」的对象，必红世界里那个被删的文件两侧都有记录。

判定（K4）：规则没说（并有一处小冲突：定义列字面最近丙，状态列/M5 判决字面是「去掉记录」，最近甲、乙）。

推翻它的现象：M5 判决正文（第三节除第 41 行之外）另有一句点名了「删除中」的认法或写了遍历方向没有记录时怎么办。我只读了这份判决的第 41 行（grep 定位），没有读它的其余部分，所以这条结论只对第 41 行成立。

## 6　K5　改法与代价

以下按 `crates/` 今天的实现（快照 sha256 与 `tree-sha256.txt` 一致的四份文件）列；行数全是估的（推的）：我的开关补丁 `recognition-variants.patch`（116 行）把去记录那步包进了环境变量判断，行数不代表产品形态，没有拿它当实测。

| 处 | 乙 | 丙 | 丁（集合比对） |
|---|---|---|---|
| `rebuild.rs` 去记录那步（`rebuild.rs:762` 起的循环） | 删 3 行，改上面 2 行注释 | 同乙 | 同乙 |
| `rebuild.rs` 单测 `a_tombstoned_object_that_still_has_its_record_is_left_out_of_both_sides_as_a_deletion_in_progress`（断言在 `rebuild.rs:1592`） | 改 1 处断言：记录现在留在 `tables.inode_records`；注释 2 行（约 4 行） | 同乙 | 同乙 |
| `walk.rs` 遍历方向去记录那步（`walk.rs:14934`）及上面的注释 | 加 1 行 `rebuilt.inode_records.remove(inode);`、`let rebuilt` 改 `let mut rebuilt`、注释 2 行（约 4 行） | 删 `walked.inode_records.remove(inode);` 一行与注释 2 行（约 3 行） | 加遍历方向自己认的集合、比两个集合、报因（约 30 行；若要「改报不适用」再加约 15 行） |
| GPU 内核 `crash_judge_checker_scan_rebuild.wgsl:1347` 那个 `continue` 与 `:1284` 的记成比过 | 把 `:1347` 的 `continue` 改成「也要遍历方向有同号同出生代记录」，加一个小函数（约 12 行） | 删 `:1347` 的 3 行与 `:1284` 的 1 行（约 4 行） | 另写一遍遍历方向记录里 nlink = 0 的集合并与 `scan_rebuild_tombstoned_object_table` 派生的集合比，加结局码（约 40 行） |
| 既有用例 | `erasing_the_retirement_record_reddens_the_scan_rebuild_after_the_right_leaf_retired`（`scan_rebuild_reads_lineage_and_retirement_records.rs:244` 那句断言）改成「`inode 记录集合` 差 1 处、key inode 234」（约 3 行） | 同乙 | 同乙，另加 6 条隔离用例要改（丁红）或 1 条（改报不适用，仅同乙那一条） |
| C860 新用例 | 实现员的 `a_retired_leaf_left_in_the_walked_mapping_without_its_retirement_record_leaves_only_the_deleted_inode_record_and_reddens_the_scan_rebuild`：去掉 `#[ignore]`，改文档注释（约 10 行） | 同乙 | 同乙，断言改成集合不同的报因 |
| 变异行（`crates/mutations.tsv`） | `:4854` 那行（去记录那步）搬到 `walk.rs` 新的一行；`:4958` 那行（认删除中恒不命中）它的杀手用例是上面那条被改的用例，要重核还杀不杀得掉；新增一行「改回甲」（恒去扫描一侧记录）由 C860 用例杀；GPU 的 `:4885`–`:4888` 四行逐行重核（约 3 行） | 同乙，`:4854` 那行删掉 | 同乙，再加集合比对的变异行 |
| kb | `crates/singlefs-checker/invariants/01-块自描述.md:18` 那句、C860 一行（`.claude/kb/checks-owed.md:574`）改还清；C860 引的 C821 那行核一次 | 同乙，另要改 M5 判决字面的引用 | 同乙，另加「删除中集合」一句 |
| 合计（估） | 池级 checker 约 10 行，GPU 约 12 行，用例约 15 行，变异 3 行 | 池级约 6 行，GPU 约 4 行，用例约 15 行，变异 2 行 | 池级约 35 到 50 行，GPU 约 40 行，用例约 25 行，变异 4 行 |

池级与 harness 的改动在 harness 档能验（`cargo test -p singlefs-harness`，不碰 checker 档）。GPU 内核的改动要对拍：`crates/singlefs-checker-tier/tests/crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker.rs` 是 checker 档的重型测试，我不能跑（见「要主 agent 跑的命令」）。

判定（K5）：不适用「一致 / 冲突」；规则没说什么，这一格是工程估算。估算来源的缺口：行数没有改出来量过（我改的是带开关的试验分支，不是产品形态）；变异行是否还杀得掉没有重放（变异表我没跑）。

推翻它的现象：产品形态的乙改完之后，`git diff --stat` 超过上面合计的两倍，或对拍测试要改的不止上面列的用例。

## 7　岔路单（要交用户的，候选、翻面观测、够判条件三列，不带倾向）

| 候选（各自的定义） | 翻面观测（量出什么会让选择换一边） | 够判条件（量到哪一步可以交用户定） |
|---|---|---|
| 岔路 A：保持甲，还是改乙。甲 = 扫描一侧见「记录 + 墓碑」就认删除中，遍历方向有同号同出生代记录才去它的；乙 = 遍历方向也有同号同出生代记录，扫描一侧才去它的记录 | 找到一张合法历史镜像（崩溃点重放的崩溃态、挂载续做、回退、快照里的一个）上，扫描一侧认出 (N, b) 而遍历方向 inode 树里没有 (N, b)，别的表都对上：乙误红，选甲。反过来，造出「退役这一步整个被跳过」的单一 bug 镜像，根环转过一圈之后 I-9.35 不判、I-3.x 一致，乙红而甲成立：选乙 | 两头各量一次：（一）在崩溃点重放（重型）里把乙的开关打开，数合法崩溃态上的 I-1.13 误红是不是 0；（二）造出上面那张单一 bug 镜像并量 I-9.35、I-3.x、I-1.13 各判什么。（一）量过且为 0、（二）量过，这条岔路就够判 |
| 岔路 B：乙还是丙。丙 = 删除中的对象的记录始终进比较（两侧都不去记录），单元与映射的去法同甲 | 找到合法历史上同一对象两侧的记录字节不同（我没找到，两侧读同一片叶）：选乙；或找到只有丙红的坏盘：选丙。另外丙要把判决 M5 的字面「去掉记录」改掉，这是文字上的代价，不是观测 | 在乙、丙之间要先问 M5 的字面能不能改；技术上（一）的崩溃点重放对丙也打开开关量一遍，误红 0 即够判 |
| 岔路 C：要不要让 I-1.13 自己认删除中（丁，遍历方向用 nlink = 0 认，两侧集合必须相等） | 找到一张 I-9.5、I-9.18、I-9.19 都不红、而两侧集合不同的镜像：丁多拦一形，选丁。我造的 X04–X07 上这三条都红 | 先造出那张镜像；造不出（我试的墓碑改指、墓碑抹号、墓碑出生代差一、nlink 改回 1 四形都被这三条拦着）就不够判，也不必判 |

## 8　要主 agent 跑的命令（我不能跑的重型，以及整条复跑）

1. 乙或丙产品形态改完之后的 GPU 对拍（checker 档，重型）：`SINGLEFS_HEAVY_TESTS=user-request`（或提交时 `=commit`）加范围声明，`cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker`，工作目录仓根，内存上限与线程数取登记值；跑完看 `test result` 一行，失败数应为 0（这份测试文件里造了意图之后、删一半旁有别的文件的镜像，`pool_with_the_first_file_half_deleted_beside_inode_two` 在该文件第 552 行）。
2. 岔路 A（一）的崩溃点重放：`bash scripts/crash/crash-point-replay.sh --item <含删除中崩溃态的项>`（`--list-items` 先列），在把乙的开关打开的副本上跑；我没有把开关接到 checker 档，也不能跑它。
3. 变异行：`checker-tier-crates-mutation-replay.sh --quick <根>` 轻型核心部分，在改完的树上跑；看汇总行的现判、复用数和有没有红行。我没有改变异表。
4. 整条复跑 `tools/rerun.sh`：超过前台 240 秒，我没整条跑过；它的各步在草稿副本里分开跑过（编译、矩阵、随机历史、回归三段），准备步骤另在空目录里 `cmp` 过。复跑前先把主工作区此刻的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default` 拷成冻结副本。

## 9　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 | 规则没说 | 甲、乙、丙、丁四种认法各自的两侧去什么、不去什么写清了；M5 原句没写由哪一侧认 |
| K2 | 一致 | 14 个合法世界、1736 个随机历史状态、30 个测试目标里的既有用例上五种读法结局相同；R 在合法盘上恒空是推的，量到的与它相容 |
| K3 | 冲突 | C860 那一形（X01）与根环转过之后的同形（X10）甲判成立、乙丙丁判红；别的坏法五种读法同红，或由 I-9.x 拦、只有丁在其上额外红 |
| K4 | 规则没说 | M5 判决与定义列都没写认法；定义列字面最近丙，M5 字面最近甲、乙；乙不改 M5 必红世界的判法 |
| K5 | 估算（不适用一致/冲突） | 乙：池级约 10 行、GPU 约 12 行、用例约 15 行、变异 3 行，既有用例转红 1 条；丙更少；丁最多且让 6 条隔离用例转红 |
| 推荐 | 乙 | 改动最小、与 M5 字面最近、合法盘零误红、C860 那一形判红；被攻过零轮 |

## 10　没做什么

- 不替主 agent 采纳或出判决；「推荐乙」是本腿的正推结论，不是判决。
- 没有跑任何重型：checker 档包的测试、GPU 对拍、崩溃点重放、变异整表、形式化验证都没跑。harness 档的测试在草稿副本里跑过（编译与测试经 `capped.sh 31` 与内存包装，范围声明都带了）。
- 没有改 `crates/`、`.claude/kb/`、`crates/mutations.tsv`；所有改动在草稿副本与模型目录的补丁里。
- 没有改 GPU 内核，K5 里 GPU 的行数全是估的；GPU 一侧「甲读法的结局与 Rust 逐位相同」我只读了代码（`crash_judge_checker_scan_rebuild.wgsl:1258`、`:1269`、`:1347`），没跑对拍，所以 GPU 一侧的数全是没量。
- 没有造「退役这一步整个被跳过」的单一 bug 镜像（要同时改记账行与分配记录），所以岔路 A 的翻面观测（二）没量。
- 没有穷举崩溃点：合法盘上「R 恒空」靠推导加 14 个造出的世界、随机历史与既有用例，不是对每个崩溃态的枚举。
- 没有读完判决 `m3-multi-file-unlink-r1-main-verification.md`，只读了第 41 行。
- `tools/rerun.sh` 整条没跑过（见第 8 节第 4 条）。
- 开工快照的一份 kb（`08-核心索引结构.md`）在开工之后被别的会话改过，我引它的行号按写报告时现查的。

### 删了哪些（草稿目录 `~/.local/share/singlefs/work/m4-c860-r1-forward/`）

交回前删了：`copy/`（仓副本，35 MB）、`target/`（编译目录，9.2 GB）、`preptest2/`（验证 `rerun.sh` 准备步骤用的空目录副本，69 MB）、`diffwork/`（出补丁用，2.3 MB）。留下的是日志（`*.log`、`*.exit`）、`regress*.sh`、`build-all.sh`、`prep-test.sh`、`model-src/` 与 `progress.md`；模型目录里的 `patch/` 与 `out/` 是入库那一份。
