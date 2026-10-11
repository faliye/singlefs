# m3-formal-code-b25-r1 云端攻方（Opus）报告（2026-10-05）

轮：第 25 批代码轮第一轮。攻击面 K1、K2、K3（重点 K1、K2）。前一轮判决：无。所有数都是攻方副本上量的（冻结副本 rsync 到草稿目录，不在冻结副本里编、写），不算入库装置上的数。

## 复跑

在主仓根下（整条脚本经内存包装，脚本里不再套包装；段之间互不依赖）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 6 \
  bash research/prompts/m3-formal-code-b25-r1-opus-model/tools/rerun.sh <草稿目录> k1-targeted k1-long k1-bound k1-rollback-mutations k1-container-number k2-device
bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 6 \
  bash research/prompts/m3-formal-code-b25-r1-opus-model/tools/rerun.sh <草稿目录> k1-sweep
```

- `k2-device` 一段（186 秒）、`k1-bound` 一段与 `k1-container-number` 一段（含 `many_inodes_write_and_read`，76 秒）在攻方自己的草稿目录复跑过（经包装都退 0，读数与下文逐项同）。别的段各 1–3 分钟；`k1-sweep`（1296 段历史）在本机负载 15–48 下约 15–20 分钟，**超过前台时限**，单独起。
- 1296 段扫描当时用的是 `opus_k1_leaf_container_histories.rs` 加 `On`（覆盖写）与 `K1_CHECKER` 两样之前的那一版、`run_batch.sh` 加 `K1_BATCH_NO_INNER_CAP` 之前的那一版；两样都只多认一种操作字母、多一个环境开关，扫描用到的字母与路径没变。

模型目录 `research/prompts/m3-formal-code-b25-r1-opus-model/` 每个文件的 sha256：

```
8032742d149459f9af41f76b1564d0f0da942208d929c70c1fe5d576fa1a288d  ./histories-big3000.tsv
004f6061a05c6730beb59ae553367900f07d8720bb29bc0a24a1cee0b18691a9  ./histories-container-number.tsv
1316b2987a84b698d049159bcb898670333131eedac2a9deb114d4ea93d6087b  ./histories-long.tsv
2a21149d002a6897a1ceef592689902e9bbd0010ff50ccde81c1bbbcdbfefae6  ./histories-overwrite-only.tsv
06649f6d4755e64bc60c5768790bb98d004378886461a5610a058fb11781540c  ./histories-rollback-mutation.tsv
2c788c75d5d1cfec8640d22c094d04ef05761931d226eae60bf9e8e5dfcdec35  ./histories-targeted.tsv
1c321bbced4be520ba2d5c7647dacd2d296ae01a97eaf954f55cf8b64c76b224  ./opus_k1_leaf_container_histories.rs
766c4d0799d11b16f509df197035eeb18834ef69c1e8e3084b52335ca1308881  ./tools/gen_sweep.py
bef1eac015e9bbb161cf748a024587554ad1fe91330b9367a39d66b52345d7ad  ./tools/mutate_core_container_number.py
e584eb86570bcc4b0d8506e357f7ec12fd39b39c4a78e43b05751d6a9f46fb0b  ./tools/mutate_rollback.py
21fb589d74f9bbb2a7eadd94edb7a3505300d889a0b1ec051e111c07fac78dbd  ./tools/patch_checker_fault.py
25f4330ad6455b2b3fdde6019411c3aa95d497f95d37d5044763c17c0e00369a  ./tools/patch_device_dump.py
8c22e1b9ee57fe8095f9ba28676a166718141f6a0e88008868bd3b909dd9bf2c  ./tools/patch_device_no_root.py
8c6b52e5d3ec860afd08481b3a4719bf5a549302cec05bb1752d32225dc7d2b7  ./tools/patch_model_bound_print.py
e5a95d7da854996ec547318e515048d170dfe7e3cd25723cf2aec06e20da81e6  ./tools/rerun.sh
c9c73bbbbc7020b6aa9a091294deeec85c5bf1693b574fec47b7d8c17da1d546  ./tools/run_batch.sh
```

## 各格判定一览

| 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|
| K1-a 模型 ⌊k ÷ 233⌋ 与 core 的切叶规则 | 没打中（一次抽样） | 量过 | 1296 段「固定前缀 + 用户动作放开扫」与 26 段定向历史全部 `Completed`，模型与实现每一步都对得上 |
| K1-b 第二片之后管理员回退那一支 | 打中「没有东西盯着」 | 量过（副本变异） | 回退那一支的两行各删一行，25-1 新用例照绿；我的三段历史各红其中一行，`U C*468 R2 C*3` 两行都红 |
| K1-c 超过 135 片模型 panic | 打中：那一支走不到，先撞的是另一个 panic | 量过 | 任意一条时间线上第 1277 次操作左右，模型在 `model.rs:1562` 加法溢出 panic（只建 inode、只覆盖写、中间挂载各一段都一样），离 135 片（31 456 次建 inode）远得很；这个溢出不是 25-1 引入的 |
| K1-d 胶水认错叶容器编号而 `agrees` 为真 | 位置序号：没打中（推的）；容器号：O1 看不见（设计内），别处罩着 | 推的 / 量过 | 位置序号认错会把不同的分配代对到同一角色上，判得出；core 把分裂出的容器号写错（坏实现），O1 一路绿、25-1 用例照绿，`many_inodes_write_and_read` 4 条红 2 条 |
| K1-e 25-1 三行变异各只红点名那条 | 没量（推的） | 推的 | 三行都只动 ≥ 234 次建 inode 的历史；harness 档别的用例没有建到 234 个的，推只红点名那条 |
| K2-a 同类而判定不同的态（真实代码） | 没打中（一次抽样） | 量过 | 对拍那一形截到第 3 次调用前 6 个写（4608 态，72 × 2⁶）逐态判：144 类里没有一类内部的恢复落点、两个崩溃子列、RefFS 一票、O2 判定有一样不同 |
| K2-b 分错的类让对拍与类核对 ①–④ 全放过（V17–V19 之外） | 打中 | 量过（副本注入故障） | 「四样输入」少一样：判态那条路上 O2、不看 journal 的那遍恢复、记录核对器任一个 panic，两个崩溃子列都变成「恢复 panic」，而类核对 ③ 只记看 journal 那遍恢复的读；O2 逐个读落了的 C 写的槽。给 O2 注入「第 2 对镜像写只落了盘 0」时 panic：登记的对拍（k = 2）与主取样点 ①–④ 全 0、`s13=false`，q5 那一行与不注入时逐字同；k = 6 的对拍 1152 态不同 |
| K2-c 288 态对拍覆盖面 | 打中（覆盖窄） | 量过 | 那 2 个 C 写是同一个单元的两盘镜像（第 41、42 号写，同一偏移）：288 态只罩 24 个 C 写里一个单元的四种落法 |
| K2-d `q5_not_run=no_root_write` 那一支 | 打中：走不到 | 量过（副本模拟条件） | 主取样点没有根写时，对拍那一形（同一次驱动、截得更早）也没有，对拍逐态 panic、走 `s13=equivalence_pairing` 退 3；认定第 6 条要的「不进 S13」那一出口走不到 |
| K2-e S13 与 `class_landing_mismatches` 的计法 | 没打中 | 推的（读码）+ 量过（基线全 0） | 计法与登记修订 2、认定第 3 条一致 |
| K3 装置 28 行变异的三分 | 没打中（推的） | 推的 | 读码没找到等价或取样点不敏感的；V17 的敏感全靠用例里「模型停在第 3 次调用」那一遍（真实流上 `crash_judge_not_applicable_states=0`，量过） |

## K1　25-1：O1 跨叶容器

### K1-a 两边的切叶规则：没打中（一次抽样）

- core 的规则：新记录的 inode 号大于树里每个 key 才追加，追加到最右那片还装得下就放进去，否则在末尾新建一片。`crates/singlefs-core/src/inode_tree.rs:232`：「let rightmost = rightmost_container_with_room(&containers);」；中间插入当场拒，`crates/singlefs-core/src/inode_tree.rs:225`：「InodeTreeWriteRefusal::InsertWouldNotLandInTheRightmostLeaf {」。
- 模型的规则：`crates/singlefs-harness/src/model.rs:197`：「let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;」，`records_before` 是这一版里「删完第一个文件之后建了几条」，回退时随目标那一版的 `contents` 一起取回。
- 两者相同的前提：树里的记录只经「末尾追加」变多、从不变少（左边每片都满 233 条）。模型罩的历史里能让记录变少的只有 unlink 第一个文件那一次，执行器在删完之后不再调 unlink（`MissingPrecondition::CurrentVersionWithoutTheFirstFile`）；回退把条数连同树一起带回目标那一版。所以「删除之后条数变少」在这套历史里不可达，没有一种形状能让两边的切法分开。
- 实测（攻方副本，`opus_k1_leaf_container_histories.rs`，每段历史从起点 `AfterFirstFile` 跑、每一步都由模型判）：
  - 定向 26 段（`histories-targeted.tsv`）：分裂前后的回退（回退到 233 条、235 条、跨第三片的 466 条）、回退两次、跨实例回退、挂载（多实例）、冷启动、抬 F 再回退、回退到环外（拒）、零单元发布（前提不满足）、建到 470 条（三片）。26 段全部 `ending=Completed`。
  - 用户动作放开扫（照 `.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做` 那一条的做法，只固定前缀）：前缀 `U C*233`、`U C*234`、`U C*466`、`U C*467`（分裂前一步与分裂那一步），后缀在 {C, R1, R2, M, X, K, F1} 上全枚举 3 步（233、234 两个前缀）或 2 步（466、467），另在 {C, R1, M, X} 上全枚举 4 步（233、234）。去重后 1296 段，全部 `ending=Completed`（`grep -vc 'ending=Completed'` 为 0）。其中前缀之后第一步就是回退且做成的（`p234` 前缀 162 段、`p467` 前缀 14 段）、回退之后再建 inode 的（63 段、2 段）、挂载之后再建的（`p233`/`p234` 各 50 段）。
  - 取样怎么缩的：历史长度由前缀定（233–467 次建 inode），候选动作 7 种、后缀 2–4 步；每段 1.1–8 秒（负载 12–48），1296 段分 18 批跑完（另有 2 批作废重切，见「限度」），每批 6 件并行，每批 22–184 秒。没有崩溃态（K1 只走 O1 对拍，不枚举崩溃态）。
- 推翻条件：一段由用户动作（不经故障注入）造出的、在第一个文件删完之后让 inode 树记录数变少或出现中间插入的合法历史；或扫描里出现一段 `NewFinding`。

### K1-b 第二片之后管理员回退那一支：打中「没有东西盯着」（量过，副本变异）

模型回退那一支对「最左那片之外的叶容器」做两件事：`crates/singlefs-harness/src/model.rs:1746`：「role_written_at.retain(|role, _| {」（丢掉现行那一版的）与 `crates/singlefs-harness/src/model.rs:1749`：「role_written_at.extend(」（换上目标那一版的）。`tools/mutate_rollback.py` 在副本上各删一件，跑 25-1 新用例与我的四段历史（`histories-rollback-mutation.tsv`），原样输出（`ms` 与计数列删了，别的照抄）：

```
-- drop-retain
test result: ok. 1 passed; 0 failed; …（25-1 新用例）
K1ROW name=T01 steps=236 … ending=NewFinding sig=ModelDisagreement { aspect: "单元的分配代" } at=Operation(235) panic=None violations=[]
K1ROW name=T02 steps=241 … ending=Completed
K1ROW name=T03 steps=238 … ending=NewFinding sig=ModelDisagreement { aspect: "单元的分配代" } at=Operation(237) panic=None violations=[]
K1ROW name=T13 steps=470 … ending=NewFinding sig=ModelDisagreement { aspect: "单元的分配代" } at=Operation(469) panic=None violations=[]
-- drop-extend
test result: ok. 1 passed; 0 failed; …（25-1 新用例）
K1ROW name=T01 steps=239 … ending=Completed
K1ROW name=T02 steps=238 … ending=NewFinding sig=ModelDisagreement { aspect: "单元的分配代" } at=Operation(237) panic=None violations=[]
K1ROW name=T03 steps=241 … ending=Completed
K1ROW name=T13 steps=470 … ending=NewFinding sig=ModelDisagreement { aspect: "单元的分配代" } at=Operation(469) panic=None violations=[]
-- none
K1ROW name=T01/T02/T03/T13 … ending=Completed（四段都是）
```

（T01 = `U C*234 R1 C*3`，T02 = `U C*236 R1 C*3`，T03 = `U C*236 R3 C*3`，T13 = `U C*468 R2 C*3`。）

- 判读：25-1 新用例只建、不回退，回退那一支的两行各删一行它都照绿；25-1 的三行变异也都不动这一支。实现员自报「有代码没测试」成立，而且这一支今天零变异覆盖。
- 两行要不同的取样点：丢掉「丢掉现行的」那一件，要「现行那一版的右侧叶多于目标那一版」（T01：两片回到一片）；丢掉「换上目标的」那一件，要「目标那一版有右侧叶」（T02：两片回到两片，叶 1 的分配代要换回目标那一版的）。T13（三片回到两片）一段两件都红。
- 处置建议（**只在我的副本上量过、被攻过零轮**）：加一条 harness 档用例跑 T13 的形状（`U`、建 468 个、回退到根环第 3 新那条、再建 3 个，判历史 `Completed`、回退做成、回退之后三次建都做成），另加两行变异照 `drop-retain`、`drop-extend` 的改法、点名这条新用例。用例时长：T13 在 release 下 6–9 秒（负载 15–30），debug 推约 5–10 倍（推的，没量 debug）。
- 推翻条件：一条已有的 harness 档用例（不是 25-1 新用例）在 `drop-retain` 或 `drop-extend` 下红。我只跑了 25-1 新用例与自己的四段，别的 151 个测试二进制没跑。

### K1-c 实现员自报 ①（超过 135 片模型 panic）：那一支走不到，先撞的是 `model.rs:1562` 的溢出（量过）

- 两边的界本身对得上：模型 `crates/singlefs-harness/src/model.rs:201`：「position_from_the_leftmost < leaf_containers_one_root_holds,」（第 31 456 次建 inode 那一条落第 135 片时 panic），core `crates/singlefs-core/src/inode_tree.rs:259`：「if containers.len() > capacity {」（同一次拒）。同一次、同一个扇出 135。
- 但模型走不到那里。长历史原样输出（`histories-long.tsv`、`histories-big3000.tsv`）：

```
K1ROW name=OV1400 steps=1277 applied=1277 tail=["A", "A", "A", "A"] matched=1278 alloc_compared=23004 roots=1278 ms=24782 ending=NewFinding sig=Panic { location: "crates/singlefs-harness/src/model.rs:1562" } at=Operation(1277) panic=Some(CapturedPanic { location: "crates/singlefs-harness/src/model.rs:1562", message: "attempt to add with overflow" }) violations=[]
K1ROW name=CR1300 steps=1277 applied=1277 tail=["A", "A", "A", "A"] matched=1279 alloc_compared=31350 roots=1279 ms=12129 ending=NewFinding sig=Panic { location: "crates/singlefs-harness/src/model.rs:1562" } at=Operation(1277) panic=Some(CapturedPanic { location: "crates/singlefs-harness/src/model.rs:1562", message: "attempt to add with overflow" }) violations=[]
K1ROW name=CRM steps=1276 applied=1276 tail=["A", "A", "A", "A"] matched=1278 alloc_compared=31338 roots=1279 ms=29293 ending=NewFinding sig=Panic { location: "crates/singlefs-harness/src/model.rs:1562" } at=Operation(1276) panic=Some(CapturedPanic { location: "crates/singlefs-harness/src/model.rs:1562", message: "attempt to add with overflow" }) violations=[]
K1ROW name=BIG3000 steps=1277 applied=1277 tail=["A", "A", "A", "A"] matched=1279 alloc_compared=31350 roots=1279 ms=31465 ending=NewFinding sig=Panic { location: "crates/singlefs-harness/src/model.rs:1562" } at=Operation(1277) panic=Some(CapturedPanic { location: "crates/singlefs-harness/src/model.rs:1562", message: "attempt to add with overflow" }) violations=[]
```

  （OV1400 = 起点之后只覆盖写 1400 次；CR1300 = unlink 之后只建 inode 1300 次；CRM = 建 600、可写挂载、再建 700；BIG3000 = 建 3000。）
- 溢出的那一行：`crates/singlefs-harness/src/model.rs:1562`：「occupied_slots_before_the_two_trees」（下一行接 `+ (mapping_nodes - 1) * …`）。病根是映射树节点数上界沿来路按比例往上滚：`crates/singlefs-harness/src/model.rs:683`：「+ 2 * (entries / smallest_half_of_a_split(central_mapping_tree_leaf_entry_capacity()));」把这一版的**全部**条目数（不是这次插入的条数）每次都加进上一版的上界，`crates/singlefs-harness/src/model.rs:1863`：「occupied_slots_upper_bound_per_device: previous.occupied_slots_upper_bound_per_device」再把它累进占槽上界。副本上打点（`tools/patch_model_bound_print.py`，只读、不改判法；OV1400 那一段，原样摘行）：

```
OPUSBOUND txg=100 mapping_nodes_upper_bound=1649 occupied_upper_bound=89389
OPUSBOUND txg=300 mapping_nodes_upper_bound=511975 occupied_upper_bound=18532503
OPUSBOUND txg=500 mapping_nodes_upper_bound=143316547 occupied_upper_bound=5159568055
OPUSBOUND txg=1100 mapping_nodes_upper_bound=3140296983829407 occupied_upper_bound=113050691418244955
OPUSBOUND txg=1279 mapping_nodes_upper_bound=486338242148574403 occupied_upper_bound=17508176717349128455
```

  每次发布约乘 1.029（txg 100 → 300 乘 310 倍），占槽上界在 txg 1280 前后越过 u64。这是 `.claude/kb/checks-owed.md:538`：「| C682 | 模型的映射树节点数上界只涨不落 |」那一笔的同一个病，C682 写的后果是「推的…没接模型跑随机历史」；这里量到了更硬的一个后果：任何一条时间线上约 1277 次发布之后模型 panic。txg 100 时占槽上界已是 89 389 槽，4 GiB 盘单元区 211 968 槽，推 txg 130 前后上界就越过单元区，C682 说的「撞单元区墙判不了」在那之后都成立（推的，没造错拒变异）。
- 与 25-1 的关系：溢出不是 25-1 引入的（只覆盖写的 OV1400 一样溢出）；25-1 之前建到第 234 个 inode 就撞胶水断言，长的建 inode 历史根本跑不到这里。25-1 两处注释说的射程因此要收：`crates/singlefs-harness/src/model_comparison.rs:680`：「叶容器数到一个根装得下的 135 片之前（三万多次建 inode）不拒，」说的是实现，没错；模型那一侧实际罩到约 1276 次建 inode（第 6 片），不是三万多次。
- 四句：① 不分辨「25-1 对不对」（没有 25-1 也溢出），分辨的是「实现员自报 ① 的三种处置」；② 系统看得到（溢出就在模型自己的算术里）；③ 满足的是派发「合法历史上 `agrees` 为假（误红）」那一分句的同族——不是判成对不上，是模型 panic、执行器记 `NewFinding`；④ 三种处置在这一格上：「模型答拒」——在第 135 片答拒碰不到这一格，历史在第 6 片就停了，**对这一格不起作用**；「加用例」——要建 31 456 个 inode，跑不到；「记账」——唯一碰得到的，并进 C682（把「模型 panic 于约 1277 次发布」作为量到的后果写进去）。
- 处置建议：自报 ① 记账，不改模型那条 `assert!`（它守的是今天到不了的一格，留着不害事）；C682 那一行补量到的后果与复跑命令（`rerun.sh <目录> k1-long k1-bound`）。另：随机历史几百步、层 0 与 E171 的流都远短于 1277 次发布，今天的门禁与产物不受影响（推的：按它们的历史长度推，没逐个查）。
- 推翻条件：同一棵树上，一段 1300 次以上发布的历史模型不 panic；或打点显示上界不随发布次数按比例涨。

### K1-d 胶水认错叶容器编号而 `agrees` 仍为真

- **位置序号：没打中（推的）**。胶水只做 `crates/singlefs-harness/src/model_comparison.rs:80`：「ModelUnitRole::inode_leaf_container_at(u64::from(index.0))」，模型比的是两个方向的角色集合加每个角色每块盘一条、仍分配、分配代等于模型记的（`crates/singlefs-harness/src/model.rs:3222`：「let handed_in: BTreeSet<ModelUnitRole> =」，`crates/singlefs-harness/src/model.rs:3255`：「.all(|record| !record.is_released && record.generation == *written_at);」）。交回的位置序号错了，要么多出 / 少了一个角色（判红），要么两片的分配代对调；只有两片分配代相同才放得过，而模型罩的历史里每次建 inode 只写一片、回退不重写叶，任两片的分配代不同。推翻条件：一段模型罩的历史上两片叶容器同一次发布写出。
- **容器号（身份里的 8 字节）：O1 看不见，25-1 用例也看不见（量过，副本坏实现）**。`tools/mutate_core_container_number.py` 把 core 末尾分裂那一处 `crates/singlefs-core/src/inode_tree.rs:244`：「container: inode.0,」改成 `inode.0 - 1`，跑 `U C*237`（与 25-1 新用例同形）：

```
-- checker off
K1ROW name=CN237 steps=238 applied=238 tail=["A", "A", "A", "A"] matched=240 ending=Completed
-- checker on
K1ROW name=CN237 steps=235 applied=235 tail=["A", "A", "A", "A"] matched=237 ending=NewFinding sig=CheckerViolations { invariants: ["I-9.4", "I-9.12"] } at=Operation(234) panic=None violations=[("I-9.4", "左容器 1 的最大 key 234 不小于右容器号 234"), ("I-9.12", "第 1 条的分隔 key 234 不大于上一个孩子的最大 key 234")]
```

  O1 只比分配代，看不到容器号、分隔 key 与记录内容，这是 O1 的设计射程，不算胶水的错；但 25-1 新用例关了每步 checker（`crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs:42`：「per_step_checker: PerStepChecker::Skipped,」），随机历史又建不到 234 个 inode，所以这个坏实现在 O1 这一路（25-1 用例、随机历史）上全绿。它没漏到 harness 档之外：同一个副本、同一处改动下，`many_inodes_write_and_read` 经发布路径一次建过 233 条、跑池级 checker 并逐项比分裂前的身份，4 条里红 2 条（`creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold`、`publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish`，`test result: FAILED. 2 passed; 2 failed`）。同一个坏实现在第一片的容器号上也错了一位（第一片同样走这一支），每步跑 checker 的那一段到第 234 步才红，第一片那一位 O2 没判出来（I-9 不判容器号与首条记录的关系，射程内）。
- 判读：容器号这一格由 `many_inodes_write_and_read` 罩着（量过），不归 O1；K1-d 在「胶水认错位置序号」与「O1 罩不罩容器号」两问上都没有要 25-1 改的。

### K1-e 25-1 三行变异：推的，没跑

三行（理想模型 /234、胶水 `.min(0)`、最左那片也记重写）都只在建到第 234 个 inode 才和原式分开；harness 档里经模型建 inode 的用例只有 25-1 新用例建过 233 个（别的 `HistoryOperation::CreateInode` 用例是随机历史，几百步建不满），推只红点名那条。没跑 harness 档整套，推翻条件：哪一行变异下另一个测试二进制也红。

## K2　E171 装置：Q5 候选二

副本上编出装置（`cargo build --release -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes`，17 秒），`confirmed-loss-form` 模式（主取样点）跑一遍 7 秒。没改装置时的四行（`k2-device` 段 `base`，原样）：

```
E7RESULT name=q5_equivalence_pairing in_flight_writes_kept=2 states=288 product_of_the_choices=288 class_states=288 classes=144 mismatching_states=0 crash_judge_not_applicable_states=0 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 refinements=0 moved_last_segment_positions=[] stopped_at_the_state_limit=false s13=false in_flight_writes=2 in_flight_write_kinds=unit_write:2 keyed_last_segment_positions=[0,1,2,3,4] class_last_segment_positions=[5,6] judged_states_planned=144 state_limit=10000000
E7RESULT name=q5_equivalence sampling_point=main_before_the_first_barrier_of_call_3 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 refinements=0 moved_last_segment_positions=[] stopped_at_the_state_limit=false s13=false
E7RESULT name=q5 sampling_point=main_before_the_first_barrier_of_call_3 states=1207959552 last_segment_writes=29 last_segment_tearable_writes=2 product_of_the_choices=1207959552 a10_states_equal_the_product=true lost_the_confirmed_version=150994944 reference_filesystem_red=150994944 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 not_lost_states=1056964608 not_lost_o1_red=0 states_without_a_root=0 derived_by=equivalence_classes judged_states=144
E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 s3_o2_judgement_mismatches=0 states=144 s13=false
```

这是攻方副本上的数，不是 E171 第 1 次重跑的产物（那一段的产物在禁读清单里，没读）。态数：这一趟实际照原路径判的是 288 + 144 态，加类核对 ③ 的 72 个代表；下面 k = 6 的对拍 4608 态。都远低于约 10⁶；这一条腿装置跑合计照原路径判过的崩溃态 21 888 个（六种跑法各跑两遍：base、dump-k2、fault-k2 各 288 + 144，dump-k6 4608 + 144，fault-k6 4608，noroot 288；另每趟 72 个代表的记读恢复不算判态），K1 的原型不枚举崩溃态。

### K2-a 真实代码上同类而判定不同的态：没打中（一次抽样，量过）

`tools/patch_device_dump.py` 给副本装置加两样只读探针（不改判法）：对拍那一形截到第 3 次调用前 k 个写（`E171_OPUS_PAIRING_K`），逐态打一行类键、恢复落点、两个崩溃子列、RefFS 一票与 O2 判定。k = 6：4608 态、144 类，按类分组之后（恢复落点、O1-崩溃判官、RefFS、O2）四样在每一类里都只有一种取值，`matches_class=false` 0 态，O2 4608 态全是 `Some([])`。k = 2 同样（288 态）。罩到的是 24 个 C 写里前 3 个单元（6 个写）的全部落法；后 9 个单元没罩。推翻条件：k 更大或换一组 C 写之后有一类里出现两种取值。

### K2-b 分错的类让对拍与类核对 ①–④ 全放过：打中（量过，副本注入故障）

**病根（读码）**：登记说两个崩溃子列「吃的输入只有四样」（`research/prompts/e171-r1-prereg.md:713`：「- 为什么这样分得开：O1 两个崩溃子列在一态上吃的输入只有四样」），类核对 ③ 只记看 journal 那一遍恢复读过的字节（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8040`：「singlefs_core::recovery::recover(&reader, singlefs_core::recovery::JournalPolicy::Consult)」）。而照原路径判一态时，两个崩溃子列还取决于第五样：判这一态的整条路上有没有别的东西 panic。

- 判态那条路上除了看 journal 的恢复，还跑三样：`crates/singlefs-checker-tier/src/crash.rs:1234`：「let ignored = recover(image, JournalPolicy::Ignore);」、`crates/singlefs-checker-tier/src/crash.rs:1247`：「let pool_checker = check_pool_image(image);」、`crates/singlefs-checker-tier/src/crash.rs:1248`：「let records = check_records(image, consulted.effective_root);」，外加观测函数里的 O2、第二遍恢复与 RefFS。
- 其中任一个 panic，整个态记成 panic 的态（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3820`：「panicking_states.push((next_ordinal, panic_text(&*payload)));」），两个崩溃子列都写成恢复 panic（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3728`：「crash_judge: CrashJudgeVerdict::RecoveryPanicked,」）。
- O2 逐个读落了的 C 写的槽：崩溃镜像交给 checker 的候选单元槽就是落了的那几个单元写的槽（`crates/singlefs-harness/src/memory_pool.rs:746`：「if *is_persisted && write.device.0 == device && write.kind == StepKind::UnitWrite {」），checker 三处扫描都从它取（`crates/singlefs-checker/src/walk.rs:3199`：「let mut slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {」、`crates/singlefs-checker/src/walk.rs:3310`：「let unit_slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {」、`crates/singlefs-checker/src/walk.rs:6118`：「let slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {」）。所以 O2 的走法随 C 落了哪几个而变，恰在 ③ 的读集之外。
- 类核对 ① 的装置实现是在判过的态上用四样输入重算、与原路径比（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8327`：「let recomputed = crash_columns_of_the_four_inputs(」）：判过的态只有代表（C 全不落）与 C 全落，第五样在这两种态上不 panic 就永远看不见。

**造法**：`tools/patch_checker_fault.py` 在副本的 `check_pool_image` 开头注入一条只在环境变量下生效的 panic——第 2 对镜像写（末段第 43、44 号写，槽 50253）只落了盘 0 那一份时 panic。这一形不在代表上（C 全不落）、不在 C 全落上、也不在登记的 288 态对拍里（对拍只截到第 41、42 号写）。原样输出（`k2-device` 段的 `fault-k2`、`fault-k6`）：

```
-- fault-k2 rc=0
E7RESULT name=q5_equivalence_pairing in_flight_writes_kept=2 states=288 product_of_the_choices=288 class_states=288 classes=144 mismatching_states=0 crash_judge_not_applicable_states=0 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 refinements=0 moved_last_segment_positions=[] stopped_at_the_state_limit=false s13=false in_flight_writes=2 in_flight_write_kinds=unit_write:2 keyed_last_segment_positions=[0,1,2,3,4] class_last_segment_positions=[5,6] judged_states_planned=144 state_limit=10000000
E7RESULT name=q5_equivalence sampling_point=main_before_the_first_barrier_of_call_3 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 refinements=0 moved_last_segment_positions=[] stopped_at_the_state_limit=false s13=false
E7RESULT name=q5 sampling_point=main_before_the_first_barrier_of_call_3 states=1207959552 last_segment_writes=29 last_segment_tearable_writes=2 product_of_the_choices=1207959552 a10_states_equal_the_product=true lost_the_confirmed_version=150994944 reference_filesystem_red=150994944 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 not_lost_states=1056964608 not_lost_o1_red=0 states_without_a_root=0 derived_by=equivalence_classes judged_states=144
E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 s3_o2_judgement_mismatches=0 states=144 s13=false
-- fault-k6 rc=3
E7RESULT name=q5_equivalence_pairing in_flight_writes_kept=2 states=4608 product_of_the_choices=4608 class_states=4608 classes=144 mismatching_states=1152 crash_judge_not_applicable_states=0 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 refinements=0 moved_last_segment_positions=[] stopped_at_the_state_limit=false s13=false in_flight_writes=6 in_flight_write_kinds=unit_write:6 keyed_last_segment_positions=[0,1,2,3,4] class_last_segment_positions=[5,6,7,8,9,10] judged_states_planned=144 state_limit=10000000
E7RESULT name=verdict s1_failed_checks=0 s13=equivalence_pairing q5_not_run=equivalence_pairing_failed
OPUSSTATE ordinal=16777535 class=0:at_least_one landing=22222222222222222222222222222222222200000002000 consulted_root=None crash_judge=recovery_panic reference_filesystem=recovery_panic o2=Some(["pool_checker_panic"]) matches_class=false recovery_panic=Some("攻方注入：O2 在槽 50253 的镜像对只落了盘 0 那一份时 panic")
```

（`in_flight_writes_kept=2` 是那一行照抄的常量，k = 6 那一趟实际截到 6 个写，看同一行的 `in_flight_writes=6`。）

- 判读：注入故障之后，登记的对拍与主取样点的 ①–④ 全 0、`s13=false`，q5 那一行与不注入时逐字相同；而照原路径判，「第 43 号写落、第 44 号写没落」的态（每种 K 取法下 2²² 态，共 72 × 2²² = 301 989 888 态）两个崩溃子列都该是恢复 panic。其中 K 取法落在「已确认的丢了」那 1/8 上的 37 748 736 态，q5 行把它们记成 RefFS 红（推的：按类结构算，没逐态判）。k = 6 的对拍逐态判出 1152 态不同（72 × 16），S13 触发。
- 四句：① 只打候选二，不打候选一、三、四、五（它们判的都是原路径上的真实态）；② 装置看得到——要么也记那几样的读，要么让 O1 两列不随它们的 panic 变；③ 满足的是登记 `research/prompts/e171-r1-prereg.md:715`：「  - 类核对 ①（输入清单）：」那一条要罩的「多出上面四样之外的」，以及派发「V17–V19 之外，还有没有一种分错它们全放过」；S13 的两个触发观测（`q5_equivalence` 计数、对拍 `mismatching_states`）都是 0，属「判别子观测不到」那一形：① 只在判过的态上重算，第五样在那两种态上恒同；④ 登记给的出路是「交到的 C 写移进键、细分」，只看 ③ 的读集，**对这一格不起作用**。
- 真实代码上它中不中：K2-a 量到 k ≤ 6 的 4608 态里 O2 没有 panic、判定都是 `Some([])`；后 9 个单元的落法没量。所以这一格今天是「方法上的洞」，不是「已知某态判错」。

几个改法各修哪一格（每格「量过」贴副本上的输出、「推的」没实现）：

| 改法 | 修 K2-b 吗 | 代价 | 标 |
|---|---|---|---|
| 甲：判态那条路上把 O2、`check_records`、不看 journal 的那遍恢复的 panic 各自接住、单列，不让它们改写 O1 两列（照 `observation_of_a_panicking_state` 里 O2 已经 `catch_unwind` 的做法）；O1 两列只随看 journal 那遍恢复 panic | 修：四样输入的说法这才成立，③ 记的读集就是 O1 两列的全部读 | 改装置的判态路径（登记修订），阳性对照要重跑 | 推的 |
| 乙：③ 也记那三样的读 | 修，但候选二随之不可行：O2 按构造读每一个落了的 C 写的槽，24 个 C 写全会移进键，2²⁴ × 72 远超上限 10⁷，走 S13 | 0 行装置改动的思路，结果是候选二停机 | 推的 |
| 丙：对拍从 k = 2 加到 k = 6（罩前 3 个单元） | 只修「坏形落在前 3 个单元里」的情形；本例量过能抓（1152 态不同） | 对拍 4608 态，副本 release 41–55 秒（量过） | 量过 |
| 丁：对拍按单元成对取，罩 12 个单元每一对的「只落一边」 | 修「单个单元镜像拆开」这一族，罩不到多个单元组合起来才 panic 的 | 推约 72 × (1 + 12 × 3) = 2664 态（推的） | 推的 |

以上改法**只在我的副本上量过（丙）或只是推的（甲、乙、丁），被攻过零轮**。推翻条件：照原路径判态时，O2 / `check_records` / 不看 journal 的恢复 panic 不会让 O1 两列变成恢复 panic（读码看错了），或 checker 候选槽不含落了的 C 写。

### K2-c 288 态对拍罩的是哪几个写：只罩一个单元的两份镜像（量过）

对拍截到第 3 次调用的前 2 个写（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7699`：「const EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT: usize = 2;」）。探针打出的末段写（k = 6 那一趟，原样摘前 11 行里的 C 部分）：

```
OPUSWRITE index=41 device=0 offset=822181888 length=32768 kind=unit_write call=Some(3)
OPUSWRITE index=42 device=1 offset=822181888 length=32768 kind=unit_write call=Some(3)
OPUSWRITE index=43 device=0 offset=823345152 length=16384 kind=unit_write call=Some(3)
OPUSWRITE index=44 device=1 offset=823345152 length=16384 kind=unit_write call=Some(3)
```

前 2 个写是同一个单元在两块盘上的镜像（同一偏移）。所以 288 态对拍罩的是「一个单元的四种落法 × 72 种 K 取法」，罩不到两个不同单元之间的任何组合；登记写它「每一跑也做一遍」、有一态不同走 S13，实际判别力只到第一对镜像。这一格是 K2-b 能漏过去的直接原因之一（改法丙、丁）。推翻条件：换一条流、前 2 个写不是镜像对（本例不成立）。

### K2-d `q5_not_run=no_root_write` 那一支：走不到，登记要的出口被对拍抢先（量过，副本模拟条件）

- 主取样点在 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7471`：「if let Some(why) = &export.not_exported {」走 `q5_not_run=no_root_write`；`not_exported` 只在 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:4478`：「export.not_exported = Some("no_root_write_to_judge".to_string());」一处置成有值。这一段只在「登记写的那种情形」（截出来的写流里没有根槽写）下进得去，这一点对。
- 但在它之前先跑对拍（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7338`：「let pairing = equivalence_pairing_of(&pairing_stream);」，`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7354`：「if pairing.stops_at_s13() {」）。对拍那一形与主取样点同一次驱动、截得更早（截在第 3 次调用前 2 个写之后），主取样点没有根写时它也没有；对拍不查根写，逐态判时 `crates/singlefs-checker-tier/src/crash.rs:1231`：「let root_persisted = image.persisted[judged_root_index];」越界 panic，每态记成 panic 的态，代表上的记读恢复照常交回报告，④ 前一半对不上，S13。
- `tools/patch_device_no_root.py` 让副本里 Q5 那一形的 `judged_root_index` 取 `usize::MAX`（只模拟条件，写流不变），原样输出（`k2-device` 段 `noroot`）：

```
-- noroot rc=3
E7RESULT name=q5_equivalence_pairing in_flight_writes_kept=2 states=288 product_of_the_choices=288 class_states=288 classes=144 mismatching_states=216 crash_judge_not_applicable_states=0 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=72 derived_class_mismatches=72 class_landing_mismatches=0 refinements=0 moved_last_segment_positions=[] stopped_at_the_state_limit=false s13=true in_flight_writes=2 in_flight_write_kinds=unit_write:2 keyed_last_segment_positions=[0,1,2,3,4] class_last_segment_positions=[5,6] judged_states_planned=144 state_limit=10000000
E7RESULT name=verdict s1_failed_checks=0 s13=equivalence_pairing q5_not_run=equivalence_pairing_failed
```

  stderr 里是 `index out of bounds: the len is 51 but the index is 18446744073709551615`（`crates/singlefs-checker-tier/src/crash.rs:1231:41`）。
- 判读：登记「主 agent 认定」第 6 条（`research/prompts/e171-r1-prereg.md:862`：「6. 「没有根写可判」那一支在这条流上走不到」）要的是那一形记 `q5_not_run=no_root_write`、不进 S13；装置今天在那一形上走的是 `s13=equivalence_pairing`，字面又回到 S13。真实流上有根写（第 38 号写），所以今天的产物不受影响。
- 四句：① 不分辨候选；② 装置看得到（对拍之前就能查 `judged_root_index`）；③ 满足的是认定第 6 条「不进 S13、不进判定」那一句的反面；④ 认定第 6 条给的改法（主取样点那一处记 `no_root_write`）在这一格上**不起作用**，因为控制流到不了那里。
- 改法（零轮，推的）：`run_confirmed_loss_form` 在对拍之前先看主取样点那一形的 `judged_root_index`，`usize::MAX` 就直接记 `q5_not_run=no_root_write` 退 3；或对拍那一形没有根写时记同一个出口。推翻条件：副本模拟之外，存在一条流使主取样点没有根写而对拍那一形有。

### K2-e S13 与 `class_landing_mismatches` 的计法：没打中（读码 + 基线全 0）

- `class_landing_mismatches` 拿观测里独立算出的落地串（`landing_vector_of`）去核代表与「C 全落」那一态的数字串，与分法用的混合进制是两条路，能抓「代表序号算错」。基线上 0（量过）。
- S13（主取样点）看的是细分到上限、读集相交剩余、①②④ 与 `class_landing_mismatches`，与认定第 3 条「`class_landing_mismatches` 算进 S13」一致；`refinements` 不进 S13，与登记「细分是照登记做」一致。A10 的「各类态数之和」只在对拍里进 S13，主取样点交比对器两份各算，与登记第七节 A10「两份各算，不读对方的计数」一致。
- 没打中的形状：找「代表或 C 全落那一态不在它的类里」——两种序号都由同一个 `ConfirmedLossPartition` 造，而 `class_landing_mismatches` 用独立的落地串核，造不出放过的。

## K3　E171 装置：其余改动（推的，没跑任何 checker 档测试与变异）

- 28 行（`grep -c '^E171R1 装置' crates/mutations.tsv` 在冻结副本里是 28）逐行读了原文与替换文，没有找到在所有输入上同值的（等价）；取样点敏感性按用例名与登记写的取样点推：
  - V17（键去掉「C 至少落一个」）：真实流上模型没停在第 3 次调用，`crash_judge_not_applicable_states=0`（上面 base 那一行，量过），这一位在真实流的每一态上都不改任何一列——登记修订 2 已写明要靠用例里「模型停在第 3 次调用」那一遍取样，用例名 `equivalence_classes_reproduce_every_state_with_and_without_the_model_stopped_at_the_in_flight_call` 带了那一遍。照「取样点不敏感」那一类的判据，敏感性全压在那一遍上；没跑，推它敏感。
  - V18（③ 恒判不相交）：用例只改写表里那一项的落点（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:10166`：「stream.writes[first_in_flight_write].offset = stream.writes[journal_record_write].offset;」），② 按原操作分种类、不随之变；用例断言移进键的位置（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:10170`：「pairing.classification.moved_positions,」）与细分 1 次，V18 下这两条都红。推敏感（没跑）。
  - V4、V11：登记修订 2 第 2.4 节写的取样点与用例名一致，推敏感。
  - 其余 22 行照抄原装置或第九节，取样点与原装置相同，没新攻。
- K2-b 的病根在 V17–V19 之外，28 行里没有一行动「判态路径上别的东西 panic」这一格；照改法甲改了之后要补一行（例如让 O2 的 panic 照旧改写 O1 两列），点名对拍或一条新用例。

## 没打中的形状

| 试过的形状 | 取样范围 | 结局 |
|---|---|---|
| K1：删完之后让 inode 树条数变少 | 模型罩的历史里没有入口（unlink 只删第一个文件，删完之后执行器不调）；读码 | 造不出 |
| K1：中间插入（新号不大于树里最大 key） | 回退之后再建（63 + 2 段扫描、T01–T04、T13、T14–T16）；core 在这一形上拒，模型没有这条理由 | 一次都没出现：回退之后的号仍大于树里每个 key |
| K1：回退跨分裂点、回退两次、跨实例回退、回退到环外 | 定向 26 段 + 1296 段扫描 | 全部 `Completed` |
| K1：崩溃恢复抛弃分裂那一版的根 | 每一处 `X`（扫描里 834 处：622 次被拒、212 次前提不满足） | 一次都没做成：恢复抛弃根在可写挂载那一步被拒（`MountError::NewerStateStillUnreadableAfterOneReread`），模型同判；分裂那一版的根被抛弃之后再建这一形没罩到 |
| K1：多实例 | `M` 在扫描里 403 + 403 + 14 + 14 次，挂载之后再建 102 段；`U C*100 M C*100 M C*40 M C*2` | 全部 `Completed` |
| K1：位置序号放过 | 读码 | 推的，没造坏实现 |
| K2：同类而判定不同（真实代码） | k = 2（288 态）、k = 6（4608 态）逐态判 | 每一类四样取值都唯一 |
| K2：全持久态与末段其余态的 RefFS 崩溃时刻不同 | 读码：末段没被屏障关上，两者都算在「全部步都返回之后」 | 不成立 |
| K2：代表或 C 全落那一态序号算错而 `class_landing_mismatches` 放过 | 读码 | 造不出 |

## 这一条腿自己的限度

- 只抽了一次样（一条攻方腿）。K1-a、K2-a、K2-e 与 K3 的「没打中」照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不算没打中。
- 所有数都在攻方副本上量（副本 = 冻结副本的 `crates/` 逐字节拷贝，加下面列的改动）：K1 的原型测试是新加的一个 harness 档测试文件；K1-b、K1-c 打点、K1-d 改了副本里的 `model.rs` 或 `inode_tree.rs`（每次跑完还原、`cmp` 与冻结副本相同）；K2 改了副本装置（两处只读探针、一处模拟条件）与副本 checker（一处注入 panic，都只在环境变量下生效）。副本上的数要在入库装置上重做才能引。
- K2-b 的坏形是**注入的故障**，不是真实代码里找到的 panic；它证的是「对拍与 ①–④ 罩不到这一类输入」，不是「Q5 的读数今天错了」。
- K1 的历史只走 4 GiB 盘、`SpaceAdmission::JudgedByTheFormula`、`AfterFirstFile` 起点；小盘与 `AfterMakeFilesystem` 起点没扫。扫描的动作集合没含覆盖写（删完之后执行器不调）与零单元发布以外的写行操作。
- 扫描当时有两批（第 6、7 批，12 件）被我外面套的 `timeout` 杀掉、整批作废（定义要求跑批不设限时，这一步违了，记在这里），重切成 72 件 6 行一件补跑；1296 段每段恰好出现一次（名字去重数 0）。

## 要主 agent 跑的命令

- 复跑：本报告开头两条（`k1-sweep` 那一条超过前台时限，用 run_in_background 起）。
- 想把 K1-b 的结论引进 kb：在入库装置上照 T13 加用例与两行变异之后，跑 `bash scripts/mutation/mutate.sh` 那两行（harness 档）。
- 想坐实 K2-b 是不是只是方法上的洞：在入库装置上把对拍 k 临时加到 6（或照改法丁按单元成对取），跑一遍主取样点的 `confirmed-loss-form`，看 `mismatching_states`。这一步是判决要不要引的事，不归攻方腿。

## 没做什么

- 没跑任何 checker 档测试（装置 44 条内联单测）、没跑 `crates/mutations.tsv` 任何一行（含 25-1 三行与装置 28 行）、没跑门禁；K1-e、K3 的结论都是推的。
- 没读 E171 第 1 次重跑这一段的产物（禁读）；K2 的数全是我在副本上跑出来的。
- 没跑 harness 档整套（151 个测试二进制）；`drop-retain` / `drop-extend` 与容器号坏实现下只跑了点名的几个测试。
- K2 没逐态罩后 9 个单元的落法（2²⁴ 量级，候选一的代价）；K2-b 的「37 748 736 态」是按类结构算的，没逐态判。
- 没判正推腿那几格，没替主 agent 采纳。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-formal-code-b25-r1-opus-output.md --root scripts/experiments/target/e171-r1-frozen-tree --root . --background research/prompts/_m3-formal-code-b25-r1-background.md` 末行：

```
  ✓ 核了 39 处引文，对上 39 处，没判 0 处（逐处列在上面）
```

草稿目录 `/tmp/claude-1000/m3-formal-code-b25-r1-opus/` 里自己建的仓副本与编译目录（`tree/`、`target/`、`rerun1–3/` 下的 `tree/` 与 `target/`）交回前都删了；留着的是日志与装置导出（`h/`、`sweep/`、`q5/`、`rerun1–3/out/`，合 17 MB），复跑照开头的命令另建。
