# m3-leaf-death-r2 辩方腿（Sonnet）报告（2026-10-08）

批次：`records/2026-10-01-删除与并发事务.md#删除线第 47 批`。立场：辩方，复核第一轮判决 `research/prompts/m3-leaf-death-r1-main-verification.md` 的 K1（候选二、三、四出局；F2 判打中；D8 已定项 6 不整条推翻；均分借）。只读冻结树 `~/.local/share/singlefs/work/m3-leaf-death-r1/tree/` 与 kb。

本报告分段落盘（每段不超过 150 行）。状态：完成。K1 每一项的结论在下面第一张表，各节给量、四句与推翻条件，「没做什么」在最后。

## 各格判定一览

「量过」= 在本腿模型 `research/prompts/m3-leaf-death-r2-sonnet-model/` 上跑出的（第一轮攻方腿 E177 装置拷贝加改动，不是入库装置，数不进 kb）；「推的」= 按条款或代码推、没跑。下表里的节号指本报告下面的节。

| 判决条目（`research/prompts/m3-leaf-death-r1-main-verification.md` 第二节 K1 与第四、五节） | 辩方判定 | 一句话依据 | 什么现象会推翻它 |
|---|---|---|---|
| 候选二「只落一次」出局 | **站得住** | 合并历史上同样复活：inode r 119–209（量过，第 1 节）；E177 459 原样 | 写侧有条款保证零条目版本活过同组每个旧版，且 G-环 上 r 峰值 0 |
| 候选二「一直留在树里」出局 | **站得住，但理由比判决写的窄** | 配「树持有零条目版本的槽」的读法加数据单元门，E177 51 段与合并 64 段全部五个名字空间 r = m = 0（量过）；剩下的只有 I-9.13 与 D8 条文（第 1 节） | I-9.13 与 D8「不落盘」被用户改成允许零条目叶，且重建能放置零条目叶（没量） |
| 候选二「字面不罩合并被吸收的右半」 | **够不着**（这一句不是打中） | 合并配零条目版本留叶序里，被吸收的右半同样写零条目版本，r = m = 0（量过，第 1 节） | — |
| 候选三 (a)「借意图那次的墓碑，早于 / 不晚于死亡 txg 才不收」 | **只中那两种读法** | 墓碑按 (号, 出生代) 在所有版本里一律不收（不比版本诞生代号），加供出规则与数据单元门（`TSZ`）：r 全 0，m 只在 unlink 途中（量过，第 2 节） | 重建读侧必须按版本诞生代号比墓碑死亡 txg（kb 现在没定读侧） |
| 候选三 (b)「最后一批另写一条撞 I-9.20」 | **够不着，但单条挪到最后一批撞 I-9.18** | 今天意图那次已写的一条够用；把那一条的写挪到最后一批（FSZ）r = m = 0，但撞 I-9.18 与 D18 已定项 24 第 2 条（第 2 节） | 用户改 I-9.18 与 K-U2 |
| 候选三 (c)「放自己的区，合并后分配记录 / 映射 r 峰值 5」 | **够不着**（加供出规则不中） | TSZ 在合并 64 段上分配记录、映射 r = 0（量过，第 2 节） | 供出规则在有崩溃状态的扫描上不成立（本腿没有崩溃状态） |
| 候选三整体「出局」 | **站不住（改成条件出局）** | (a)(b)(c) 三个具体打中在最强形态上都不中，剩一格 unlink 途中的 m，与 X3 同源（第 2 节） | 用户或主 agent 把 E177 真值改成「有谱系墓碑的对象从意图那次起不算活」则回到集合；扫盘重建必须逐字复原已发布版本则回到原判决 |
| 候选四「判别子观测不到」 | **站得住** | 写路径带墓碑（`crates/` 今天）后，两段历史读得出的码 1 / 码 3 单元含墓碑容器仍逐个相同 9 / 9（量过，第 3 节） | 重建读侧能读到 A、B 之间不同的单元（例如最后一批另写东西） |
| 「G-环 退回更旧版本是同一个病」 | **站得住** | T 族、C2HoldZ 的 r_out 全 0；只落一次 459（量过） | — |
| X1「只剩候选一，不交选择，交认这个方向」 | **站不住** | 候选三 / 四的唯一剩余格（unlink 途中的 m）就是 X3 第一支的代价，两题同源（第 2 节） | 用户在 X3 已选「最后一批才生效」 |
| F2 判打中 | **站得住（读法层面），字面触发没中** | 登记 F2 两个触发条件都没满足；判打中靠 D8 条文加 `crates/` 自己的注释（第 4 节） | 删空不算合并的退化形 |
| D8 已定项 6「不整条推翻、改字面」 | **站得住，但只在取候选一时** | 取候选三 / 四（T 族）时那句被整句换掉；而 `crates/` 今天已经写整对象墓碑（第 4 节） | — |
| 均分借 | **站得住（带一条 0.13% 的例外）** | 换成有序删的 36 格，均分借重写的叶没有一格多于刚好够；E176 22 条序列里一条（`wa_keep1of2_a1000`）多 0.13%；只有「搬动的记录条数」这个量翻（第 5 节） | 用「搬了多少条记录」当代价 |

## 复跑与文件

模型目录 `research/prompts/m3-leaf-death-r2-sonnet-model/`：源码 = `research/prompts/m3-leaf-death-r1-opus-model/` 的拷贝（E177 装置 + 第一轮攻方的 `leaf_death.rs`）加本轮改动（`src/leaf_death.rs`；`src/main.rs` 只多 `borrowed_records` 字段与 4 个模式名）。不依赖 `crates/` 与 `scripts/experiments/Cargo.toml`；`SINGLEFS_REPO` 只用来找 `scripts/run-with-memory-cap.sh`。新增：

- 零条目版本（候选二）配合并：容器留在叶序里、合并只在有记录的片之间做（`src/leaf_death.rs` 的 `dismiss_container_at`、`right_neighbour_of`）；
- 读法 `C2Hold` / `C2HoldZ`（候选二最强形态）、`T` / `TS` / `TSZ`（候选三 / 四最强形态）、`FSZ`（墓碑的写挪到最后一批）；
- 模式 `r2-e177`、`r2-merges`（1280 件）、`r2-sweep`（72 件，均分借换前提）、`r2-rollback`、`r2-indistinguishable`。

回归：改动后的模型对第一轮攻方腿已有的读法，E177 110 行聚合逐行相同、合并 280 行逐行相同（合并里第一轮的 20 行 M1 + 只落一次本轮没跑，不算）。
复跑（仓根，8 个线程；每阶段经内存包装，build 6G、其余 4G；交回前在空目录 `~/.local/share/singlefs/work/m3-leaf-death-r2-sonnet/rerun1/` 里从头跑通一次，另在 `rerun2/` 里重跑 e177、sweep、rollback、indist、merges-0 五段，排序后 sha256 与 `expected/sorted-output-sha256.txt` 逐个相同）：

```
M=research/prompts/m3-leaf-death-r2-sonnet-model; S=scripts; O=<空目录>; export SINGLEFS_REPO=$PWD
bash $S/run-with-memory-cap.sh 6G bash $M/rerun.sh $O build
bash $S/run-with-memory-cap.sh 4G bash $M/rerun.sh $O e177 sweep rollback indist
for k in 0 1 2 3 4 5 6 7; do bash $S/run-with-memory-cap.sh 4G bash $M/rerun.sh $O merges-$k; done
bash $S/run-with-memory-cap.sh 4G bash $M/rerun.sh $O summarize
bash $M/compare.sh $O
```

各阶段挂钟（机器忙，同时有别的会话的编译与模型）：build 4 秒；e177 + sweep + rollback + indist 约 70 秒；merges 每批 160 件 3.2–3.7 分钟，八批合计约 28 分钟。`expected/` 是 `rerun1/` 的 `summary/` 拷下来的，`rerun1/` 上跑 `compare.sh` 是同义反复，不算核对；独立的核对是 `rerun2/` 里 e177、sweep、rollback、indist、merges-0 五段排序后的 sha256 与 `expected/sorted-output-sha256.txt` 逐个相同（上面已写）。

## 1　候选二（零条目版本落盘）

### 1.1　最强形态是什么

第一轮攻方腿只量了两种写法，且零条目版本只配「不合并」（`research/prompts/m3-leaf-death-r1-opus-model/src/leaf_death.rs` 里 `assert_eq!(policy.merge, MergeKind::M1, …)`）。本轮补三处：

- 零条目版本配 M2 / M3 均分 / M3 刚好够：容器留在叶序里，合并只在有记录的片之间做；被吸收的右半同样写零条目版本。
- 读法 `C2Hold`：零条目版本是树持有的活单元，记录算死，但它的槽与映射 key 照收。第一轮的 `C2` 把它的槽也丢了，E177 上留下 m = 3：「ring M1_zero_persist C2 allocation_records histories=51 r_peak_max=300 r_out_peak_max=276 histories_r_pos=37 m_peak_max=3 histories_m_pos=48」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-e177.txt:66`）。这一格第一轮判决没有列，是读法漏洞，不是候选的病。
- 读法 `C2HoldZ`：`C2Hold` 加第一轮攻方腿的数据单元门（数据单元按重建出的 inode 记录的出生代与 size 判）。

### 1.2　量（读法 S 是今天的读法；全部 ring，G-宽 同形，见 `expected/agg-*.txt`）

只落一次（OneShot）：

- E177 51 段：「ring M1_zero_oneshot C2 inode histories=51 r_peak_max=459 r_out_peak_max=459 histories_r_pos=3 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-e177.txt:60`）。
- 合并 64 段 M3 均分：「ring M3half_zero_oneshot C2 inode histories=64 r_peak_max=119 r_out_peak_max=119 histories_r_pos=25 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-merges.txt:200`）；M2 下 209。第一轮只在 M1 上量了 459，合并下同一个病。

一直留（Persist）加 `C2HoldZ`：

- E177：「ring M1_zero_persist C2HoldZ inode histories=51 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-e177.txt:79`）与同组分配记录「ring M1_zero_persist C2HoldZ allocation_records histories=51 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-e177.txt:75`）。
- 合并 M3 均分：「ring M3half_zero_persist C2HoldZ allocation_records histories=64 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-merges.txt:215`）。
- 不加数据单元门的 `C2Hold`：「ring M3half_zero_persist C2Hold allocation_records histories=64 r_peak_max=199 r_out_peak_max=199 histories_r_pos=64 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-merges.txt:210`）：这 199 是文件变短 / 覆盖写那一族（C109 的 K2），候选一不加数据单元门也是同一个数，不是候选二特有。

五个名字空间（inode、`deleted_inodes`、extent、分配记录、中央映射）× 两个几何 × E177 51 段与合并 64 段 × M2 / M3 均分 / M3 刚好够：`C2HoldZ` 的 r、m 全 0（`expected/agg-*.txt` 里 `C2HoldZ` 各行）。

### 1.3　一直留剩下的账

- I-9.13 为假的版本：E177 51 段 20003 次发布里 1439 个（7.2%）；合并 M3 均分 64 段 110365 次发布里 12161 个（11.0%），M2 下 14042 个（12.7%）：「ring M3half_zero_persist runs=64 publishes=110365 merges=114 borrows=148 borrowed_records=2562 zero_versions=127 retirements=0 versions_with_an_empty_container=12161 containers_final=895 empty_containers_final=31 containers_peak_max=22 empty_peak_max=3」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/counts-merges.txt:7`）。零条目叶一旦留下就在之后每一版里。
- 第一轮说「判红是推的」：现在有 checker 侧的事实。池级 checker 对每片类型 2 叶判「self.judgements.judge("I-9.13", !records.is_empty(), || {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:3056`），已有的坏镜像表里就有一项把类型 2 容器记录数改成 0 要求只红 I-9.13：「// 类型 2 容器的记录数与声明长度都改成 0。」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:1292`）。kb 给这条的理由是「空叶在重建时没有 key 范围」（`.claude/kb/invariants.md:301`）；零条目叶在叶序里怎么放置（只剩容器号）本腿没有建重建模型，没量。
- 叶只增不减：E177 一直留下终态 96 片叶，其中零条目 60 片，不留 36 片：「ring M1_zero_persist runs=51 publishes=20003 merges=0 borrows=0 borrowed_records=0 zero_versions=63 retirements=0 versions_with_an_empty_container=1439 containers_final=96 empty_containers_final=60 containers_peak_max=5 empty_peak_max=3」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/counts-e177.txt:3`）。回收门（E104 的 P5）两条路径都要，第一版都不回收。
- 合并规则要加一条例外：`pair_fits_in_one_container` 对「零条目叶 + 任意一片」恒为真，不排除零条目叶，M2 / M3 下一次发布就把桩吸收掉（等于退役）。模型里是 `right_neighbour_of` 跳过空叶（推的，读 `src/leaf_death.rs` 即见）。
- 合并目的落空：合并本来是收回叶，留桩之后叶数几乎不降（合并 64 段终态叶 895 对 863，上面那行与 `counts-merges.txt:5`）。

### 1.4　回退（手造盘，量过）

零条目版本不被带到新线上（不像随现行那一版走的退役树），只要读侧按谱系滤单元：「name=r2_rollback_demo candidate=two_zero_version_persist tombstones_filtered_by_lineage=none units_filtered_by_lineage=true resurrected=0 missed=0 missed_inodes=[]」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/rollback.txt:4`）；不滤时「… units_filtered_by_lineage=false resurrected=0 missed=2 missed_inodes=[234, 235]」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/rollback.txt:3`）。所以第一轮判给候选一的「必须带死亡 txg」这条额外要求，候选二没有。（单元谱系滤是 C615 的读侧，三个候选都要。）

### 1.5　四句

- 分不分辨臂：只落一次与一直留分辨。一直留与候选一在 r、m 格上**不分辨**（都 0），只在 I-9.13、叶数、回收上分。
- 判别子看不看得到：看得到——零条目版本读得出时读侧认得；只落一次的病是它被盖掉之后读侧没东西读。
- 满足的是判据字面哪一句：只落一次——「仍复活」；一直留——不是 r、m 那句，是「不撞别的 I 类判据」（正文第五节第一条的后半句）。
- 改法在打中的格上还中不中：「读法认树持有的零条目版本 + 数据单元门」在 r、m 格上不中；I-9.13 照中，改法只能是改 I-9.13 与 D8「不落盘」。

### 1.6　判

**判决站得住**，但读判决的人要知道它现在只靠 I-9.13 一条（外加叶不降与回收门），不再是「两腿独立打中」。第一轮把「一直留」判出局时没有量过它配读法之后的 r、m（零），也没有量过合并。同一个衡量尺下：第一轮攻方腿自己的候选一也是在「记录放模型专用区、不占槽」的理想化下量的（`research/prompts/m3-leaf-death-r1-opus-model/src/leaf_death.rs:7`），载体不被根持有就会像只落一次那样被盖（推的）——候选一的载体账与候选二的 I-9.13 账是同一种账（推的）（各改一处 kb 条文），不该一个记「出局」、一个记「待第二轮给载体」。

什么现象会推翻它：用户同意改 I-9.13 与 D8「记录数为 0 的类型 2 容器不落盘」，且重建能放置零条目叶；或候选一的载体账（树表条目、I-9.23 合法集合、新不变量）比这一条更贵。

## 2　候选三（类型 2 记录死亡写墓碑）与候选四（只改读侧）：T 族

### 2.1　最强形态是什么

第一轮三个打中，各自依赖一个**可选的**选择：

- (a) 借意图那次的墓碑，只有两种读法：墓碑死亡 txg 与「供出版本的诞生代号」比，早于才不收（复活 3）、不晚于就不收（误判死亡 1）。比较本身不是 kb 条款：kb 把「扫盘重建怎么读墓碑」留给 K-W5b：「扫盘重建怎么读墓碑与谱系记录不在这里定，记欠账（K-W5b」（`.claude/kb/decisions/18-块里携带什么信息.md:579`）。(inode, 对象出生代) 已发布不复用，墓碑指认的是整个对象，不比版本。
- (b) 最后一批另写一条墓碑 ⇒ I-9.20。支持候选三的人不会另加一条：`crates/` 今天意图那次已经写：「tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/transaction.rs:5997`），D18 已定项 24 定的也是「死亡 txg = 意图那次发布的 checkpoint_txg」（`.claude/kb/decisions/18-块里携带什么信息.md:573`）。
- (c) 记录级死亡不说容器死了 ⇒ 合并后被吸收右半的槽复活。但被吸收右半的每条记录在左半有一份更新的拷贝；容器版本只有供出了至少一条没死的记录才算活单元，不需要容器级记录。

本轮的读法（都是读侧，盘上零新增）：

- `T`：今天意图那次已写的整对象墓碑，按 (号, 对象出生代) 在所有版本里一律不收，不比版本诞生代号；一片里一条都不收的容器整片不收；被点名对象的数据单元不收。
- `TS`：`T` 加供出规则：同号取诞生代号最大的版本供出，容器版本只有供出了至少一条没压掉的记录才算活单元。
- `TSZ`：`TS` 加第一轮攻方腿的数据单元门。
- `FSZ`：墓碑的**写**挪到最后一批（一条，不是再加一条）：死亡 txg = 记录离开树的那次发布；其余同 `TSZ`。

### 2.2　量

第一轮的两种比法（对照，不是本轮新读法）：「ring M1_dropped C3_intent_lt inode histories=51 r_peak_max=3 r_out_peak_max=2 histories_r_pos=48 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-e177.txt:25`）与「ring M1_dropped C3_intent_le inode histories=51 r_peak_max=1 r_out_peak_max=0 histories_r_pos=10 m_peak_max=1 histories_m_pos=51」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-e177.txt:20`）。

`T` 族，E177 51 段：

- 「ring M1_dropped TSZ inode histories=51 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=1 histories_m_pos=51」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-e177.txt:55`）：inode r = 0，m = 1，51 段都有。
- 「ring M1_dropped TSZ extent histories=51 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=300 histories_m_pos=35」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-e177.txt:54`）与分配记录「ring M1_dropped TSZ allocation_records histories=51 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=301 histories_m_pos=49」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-e177.txt:51`）：r = 0，m 是 unlink 途中那个对象的数据单元被意图那次的墓碑提前压掉。

合并 64 段（M3 均分）：

- 第一轮攻方腿的 `C3Z` 在这里的分配记录上 r = 3：「ring M3half_dropped C3Z allocation_records histories=64 r_peak_max=3 r_out_peak_max=3 histories_r_pos=41 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-merges.txt:166`）；加了供出规则的 `TSZ`：「ring M3half_dropped TSZ allocation_records histories=64 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=181 histories_m_pos=35」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-merges.txt:191`）与 inode「ring M3half_dropped TSZ inode histories=64 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=1 histories_m_pos=64」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-merges.txt:195`）。M2 / M3 刚好够 / G-宽 同形：`T` 族所有读法 × 所有政策 × 两个几何的 inode r 峰值最大 0，`TSZ` 五个名字空间 r 全 0（`expected/agg-*.txt`，聚合最大值我用脚本数的：E177 与合并两张表里 `TSZ` 行的 r_peak_max 最大值都是 0）。
- 不加数据单元门的 `TS`（对照）分配记录 r = 198：「ring M3half_dropped TS allocation_records histories=64 r_peak_max=198 r_out_peak_max=198 histories_r_pos=60 m_peak_max=181 histories_m_pos=35」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-merges.txt:186`）——文件变短 / 覆盖写那一族，候选一不加门也是同一个数，是 K2 的事，不是候选三的病。

m 全部落在 unlink 途中（意图到最后一批之间）：`T` / `TSZ` 在 inode 树上 m 为正的扫描 111754 次、其中在 unlink 途中 111754 次：「T M1_dropped inode m_scans=111754 m_scans_in_unlink=111754 r_scans=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/window.txt:65`）；「cells=120 cells_with_m_scans_outside_unlink=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/window.txt:121`）。这些历史是 unlink 密集的：E177 与合并两批 M1 历史合计 130286 个版本里 55877 个（42.9%）落在某个 unlink 途中：「window_share geometry=ring policy=M1_dropped reader=T inode_tree_m_positive_scans=55877 publishes=130286」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/window.txt:123`）。真实负载里 unlink 途中的占比不会这么高，这个占比是历史的性质，不是候选的性质。

`FSZ`（墓碑的写挪到最后一批）：E177 与合并全部 r = m = 0：「ring M1_dropped FSZ inode histories=51 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-e177.txt:35`）、「ring M3half_dropped FSZ allocation_records histories=64 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-merges.txt:171`）。

### 2.3　回退（手造盘，量过）

墓碑树随现行那一版（带着被回退掉那一段写的），读侧按死亡 txg 滤谱系——这就是 I-9.18 对墓碑已定的读法，不加字段：「name=r2_rollback_demo candidate=T_whole_object_tombstone tombstones_filtered_by_lineage=true units_filtered_by_lineage=true resurrected=0 missed=0 missed_inodes=[]」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/rollback.txt:2`）；不滤时误判死亡 2：「name=r2_rollback_demo candidate=T_whole_object_tombstone tombstones_filtered_by_lineage=false units_filtered_by_lineage=false resurrected=0 missed=2 missed_inodes=[234, 235]」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/rollback.txt:1`）。

### 2.4　四句

- 分不分辨臂：`TSZ` 与候选一在 r 格上都 0；m 格上分辨（候选一 + 数据单元门 m = 0，`TSZ` 在 unlink 途中 m > 0）。`FSZ` 与候选一不分辨。
- 判别子看不看得到：unlink 途中那一版与删空之后的那一版，读侧拿到的输入逐条相同（第 3 节）——答「死」就是 `T` 族，答「活」就是第一轮的 (a) 复活那一支；二选一，没有第三个答案。
- 满足的是判据字面哪一句：剩下的 m 满足「误判死亡（活记录被判死）」，而被判死的那条记录 nlink = 0 且在谱系上有墓碑；I-9.19 的标题与判据用的正是「墓碑不指活对象」：「谱系上的墓碑不指活对象」（`.claude/kb/invariants.md:307`）。按 kb 自己的词，有谱系墓碑的对象不是活对象（读法，推的）；按 E177 的真值（记录还在树里就算活），它是。**哪个口径对，是 X3，不是候选三自己的缺陷。**
- 改法在打中的格上还中不中：(a) 的 r 格不中、(b) 不写第二条所以不碰 I-9.20、(c) 加供出规则不中；`FSZ` 把写挪到最后一批，m 也不中，但那一形撞 I-9.18：「删除队列里的号有谱系上的墓碑」（`.claude/kb/invariants.md:306`）——队列里的号在意图到最后一批之间没有墓碑——也撞 D18 已定项 24 第 2 条的死亡 txg。

### 2.5　判

**第一轮对候选三的判决：(a)、(b)、(c) 三个具体打中在最强形态上都不中，唯一剩下的是 unlink 途中的 m（inode 1、extent 300、分配记录 / 映射 301），所以「出局」站不住；它改成「条件出局」。** 这一格就是判决第四节 X3 第一支的代价（「意图那次：unlink 途中漏活 m 峰值 inode 1、extent 300（量）」），判决在 X1 里把它当候选三的缺陷、在 X3 里又把它当用户的选择，同一个量两处口径。X1「只剩候选一」要和 X3 一起问：

| X3 的答案 | 候选三 / 四（T 族） | 候选一 |
|---|---|---|
| 意图那次生效 | 盘上零新增，代价 = unlink 途中的 m | 另有载体、读法、回退规则的账 |
| 最后一批才生效 | 读侧做不到（第 3 节）；写侧要把墓碑挪到最后一批（`FSZ`，r = m = 0），撞 I-9.18、D18 已定项 24 第 2 条与 K-U2 | 不受影响 |

按跑前写死的判据（正文第五节第一条）严格读：`T` 族没有在第一轮打中的**全部**格上不中（m 那格还在），所以候选三不自动回到候选集；这里交的是「X1 不能先于 X3 单独写成『只剩候选一』」。

什么现象会推翻它：用户或主 agent 把 E177 真值改成「有谱系墓碑的对象从意图那次起不算活」（则 m 格消失）——推翻的是「出局」；或扫盘重建必须逐字复原已发布的版本（则 m 格是缺陷）——推翻的是「条件出局」，回到判决原样。供出规则本腿只在干净发布之后的扫描上量过，没有崩溃状态。

## 3　候选四（只改读侧）：判别子在 `crates/` 今天的写路径上还成立吗

第一轮攻方腿的两段历史（A：unlink 只发到意图再发一次空发布；B：发完意图与最后一批）是在**不写墓碑**的写路径上造的：「let mut simulation = Self::start(geometry, WritePath::WithoutTombstones, PRODUCT_UNLINK_BATCH_DATA_UNITS, None, false);」（`research/prompts/m3-leaf-death-r1-opus-model/src/leaf_death.rs:409`）。今天 `crates/` 意图那次写整对象墓碑，读侧能多一个输入。本轮把同一对历史放到带墓碑的写路径（`WritePath::WithTombstones`，`r2-indistinguishable` 模式）上：读得出的码 1 / 码 3 单元（含墓碑容器）逐个相同，A 活、B 死：「name=r2_indistinguishable write_path=with_tombstones geometry=ring prefix_overwrites=40 inode_count=700 x=700 latest_txg_a=61 latest_txg_b=61 code1_code3_readable_units_a=67 code1_code3_readable_units_b=67 tombstone_units_a=1 tombstone_units_b=1 units_identical=true x_alive_a=true x_alive_b=false」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/indistinguishable.txt:6`）；九行（G-宽、G-环、G-环 前缀先覆盖写 40 次 × n = 234、467、700）全是 `units_identical=true x_alive_a=true x_alive_b=false`（`grep -c 'units_identical=true x_alive_a=true x_alive_b=false' research/prompts/m3-leaf-death-r2-sonnet-model/expected/indistinguishable.txt` 输出 `9`）。

- 分不分辨臂：分辨（候选一在 B 里多写一条，A、B 的输入不再相同）。
- 判别子看不看得到：看不到。墓碑在 A、B 里同一个单元、同一个死亡 txg。
- 满足的是判据字面哪一句：答「死」= 第 2 节的 `T` 族（m 在 unlink 途中），答「活」= 第一轮的 (a) 复活那一支。
- 改法在打中的格上还中不中：要分开就得在最后一批多写东西（回到候选一 / 二 / `FSZ`）。

**判：第一轮对候选四的判决站得住。** 补两点：(1) 它现在是在 `crates/` 今天的写路径上成立的，不是只在无墓碑路径上；(2) 候选四在这里**没有独立的形态**——读侧能做的唯一非平凡答法就是 `T` 族，它的取舍就是第 2 节那张 X3 表。

什么现象会推翻它：重建读侧能读到 A、B 之间不同的单元，例如最后一批另写一个单元（那就是别的候选）。

## 4　F2 判打中；D8 已定项 6「不整条推翻、改字面」

### 4.1　F2

登记 F2 的触发有两个：「执行员按第十三节命令 C10 在 kb 里查到一条要素……而 R-S 没用它」和「或 C4 不再是 0 行」（`research/prompts/e177-preregistration.md:484`）。实验页记的是字面没打中：「字面没打中」（`.claude/kb/experiments/177-逐树不写墓碑的复活数.md:248`），C4（`grep -rn '退役记录' crates --include=*.rs | wc -l`）今天仍是 0 行。所以**字面触发两个条件都没满足**；判决判「打中」靠的是实质读法，不是登记字面。这个读法站得住，依据四处，都是原文：

- `crates/` 自己把删空叫左吸收右的特例：「/// - 那一片删空了 ⇒ 整片不落盘（「记录数为 0 的类型 2 容器不落盘」，D8 已定项 6：左吸收右的退化形——右边那片是空的），」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:277`）。
- D8 对合并写的是：「保留左半身份，右半退役走 COW 释放路径」（`.claude/kb/decisions/08-核心索引结构.md:171`），并要求「发布前必须合并掉」（同一行）。
- D18 已定项 11 登记退役记录：「容器合并时右半退役、重生时旧身份退役」（`.claude/kb/decisions/18-块里携带什么信息.md:300`）。
- 这条登记没有「第一版」三个字，所以执行员那条 grep 命中不了；但删空发生在第一版单头干净历史里（E177 的 F1 全族），右半退役跟着发生。

反方读法：删空是 `crates/` 写路径里的特例、不是合并，则退役记录不是「第一版就该生效」的要素，F1 的复活就是 D8 已定项 6 在 R-S 下成立不了的性质，对 D8 已定项 6 的措辞该是「推翻」。这是读条文的分歧，不是能量出来的；判决选前一种有四处原文，**站得住**。影响只有 E177 实验页的措辞（「在缺退役记录的 R-S 下复活」），不影响候选集合。

什么现象会推翻它：用户或 D8 作者说明删空不是合并。

### 4.2　D8 已定项 6「不整条推翻、改字面」

被判的那句是：「类型 2 的记录死亡不写墓碑，死亡由现行叶版本体现，择现行版本压在 C113」（`.claude/kb/decisions/08-核心索引结构.md:169`）。

- 取候选一时：这句保留，补「容器身份消失写退役记录」——判决的「两层」说法成立。
- 取 `T` 族（候选三 / 四）时：这句被整句换掉（记录死亡由墓碑体现），不是改字面。
- 这句的前半「不写墓碑」在 `crates/` 里**今天已经不成立**：意图那次写整对象墓碑（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/transaction.rs:5997`），D18 已定项 24 说明它在意图那次写：「第一版在意图那次写」（`.claude/kb/decisions/18-块里携带什么信息.md:212`）。所以判决的「现行叶版本管记录死亡、退役记录管容器身份消失」缺了第三层——已经在写的整对象墓碑；「两层合起来经量站得住」应该写成三层。

**判：站得住，但要加两个限定**——只在取候选一时；且层数是三不是二。什么现象会推翻它：X1 / X3 的答案选 `T` 族。

## 5　均分借：两腿的量是不是同一个前提

### 5.1　两腿的前提

- 攻方腿：E177 装置，64 段随机 / 集中删，只有 1 号有数据；量的是最后一批那次发布重写的叶：「ring M3half_dropped runs=64 … containers_rewritten_in_removals=46232 … borrows=158」（`research/prompts/m3-leaf-death-r1-opus-model/expected/counts-merges.txt:4`）与「ring M3just_dropped runs=64 … containers_rewritten_in_removals=47315 … borrows=1237」（`research/prompts/m3-leaf-death-r1-opus-model/expected/counts-merges.txt:5`）。差 2.3%。
- 正推腿：E176 装置，22 条序列，全是只有记录、没有数据的 inode，量每次 unlink 重写的叶：「E7RESULT name=window_rewrites sequence=ws_a1000_d950_smallest_first window=main arm=M3 unlinks=575700 content_sum=592977 position_sum=757638 content_mean=1.0300」（`research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:843`）对「… window=just_enough arm=M3_just_enough unlinks=575700 content_sum=862096 position_sum=1026775 content_mean=1.4975」（`research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:858`）。差 45%。
- 所以不是同一个前提：删序不同（随机 / 有序）、发布粒度不同（只数最后一批 / 每次 unlink）、窗口不同。共同点是都没有数据文件挪动，且新号只落最右的叶（「插入永远落在最右的叶」，`.claude/kb/decisions/08-核心索引结构.md:171`），中间的叶只经历删。

### 5.2　换前提（本腿 `r2-sweep`：有序删）

36 格 = n0 ∈ {700, 1400} × 删序 {从小到大、从大到小、隔两个删一个} × 删 {50%, 80%, 95%} × 每删 7 个建一批 / 不建，只配 M3 均分与刚好够：

- 重写的叶：均分没有一格多于刚好够，合计 29153 对 34767：「cells 36 cells_where_half_rewrites_more_leaves_than_just 0 cells_where_half_moves_fewer_records_than_just 0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/sweep.txt:38`）、「hb=531 jb=6145 hm=6785 jm=6145 hr=29153 jr=34767」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/sweep.txt:39`）。借的次数 531 对 6145。
- 翻面的只有一个量——搬动的记录条数：12 格均分搬得更多（合计 6785 对 6145；合并 64 段上 2606 对 1237，`expected/counts-merges.txt:5`、`expected/counts-merges.txt:8`），24 格相同，没有一格更少。把一次借的代价记成「搬了多少条 140 字节的记录」就会翻，记成「重写了几个 32 KiB 的叶 / 几刀写」（E176 的口径、层 0 枚举崩溃点的口径）不翻。按本腿的数：多重写 5614 个叶对少搬 640 条记录，差约三个数量级。
- 删序为从大到小且不建时没有借（0/0），两个借法逐格相同。
- 合并 64 段 × 2 个几何共 128 件里，均分重写的叶有 0 件多于刚好够、76 件相同（对 `rerun1/merges-*.out` 的 `containers_rewritten_in_removals` 逐件比，一次性脚本，没入 `expected/`）。
- 正推腿的 E176：22 条序列里 21 条均分 ≤ 刚好够，一条例外：「E7RESULT name=window_rewrites sequence=wa_keep1of2_a1000 window=main arm=M3 unlinks=16500 content_sum=16526 position_sum=16590 content_mean=1.0016 position_mean=1.0055 content_largest=2 position_largest=4 content_histogram=0/16474/26/0/0」（`research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:941`）对「E7RESULT name=window_rewrites sequence=wa_keep1of2_a1000 window=just_enough arm=M3_just_enough unlinks=16500 content_sum=16504 position_sum=16587 content_mean=1.0002 position_mean=1.0053 content_largest=2 position_largest=4 content_histogram=0/16496/4/0/0」（`research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:956`）：均分在 16500 次 unlink 里多重写 22 个叶（+0.13%）。判决和正推报告的「均分没有输」要加这一条例外。

### 5.3　两腿口径的一处不一致

判决写「最坏频度界两种一样」又写「每对并之前借 ≤ 7 次对 ≤ 116 次」。前一句是攻方腿的说法，没写前提，数的是「每次删最多借一次」；后一句是正推腿的界，数的是「纯删时每对并之前借几次」。单位不同（每次删 / 每对），前提不同（没写 / 纯删），一起写进判决会被读成同一个界。新号只落最右的叶，所以 7 这个界对中间的叶对成立，最右一对因为被建抬高可以再借（推的）。本腿没有量最坏频度（`r2-sweep` 是平均），这一条留「推的」。

### 5.4　判

**站得住，带一条例外。** 重写叶口径上均分在本腿 36 格、攻方腿 128 件里没有一处多于刚好够，正推腿 22 条序列里只有 `wa_keep1of2_a1000` 一条多 0.13%；翻面只在「搬记录条数」口径，本项目的代价口径是单元（崩溃点按写请求数）。「只有 1 号有数据」这一点本腿模型也给不了别的 inode 数据，换不了；推的：有数据的 unlink 在最后一批前多出若干次只重写本片的发布，借带来的额外重写绝对数不变、相对占比变小。

什么现象会推翻它：代价口径改成按记录条数计；或出现一个取样点，均分借的叶重写多于刚好够（本腿 36 格与攻方腿 128 件里没有，正推腿 22 条序列里有一条 0.13%）。

## 6　第一轮打中的格 × 最强形态（汇总，数都是量过的）

| 第一轮打中的格 | 第一轮的数 | 最强形态下 | 还中吗 |
|---|---|---|---|
| 二·只落一次·E177 inode 复活 | r 459 | 459（`agg-e177.txt:60`） | 中 |
| 二·只落一次·合并下 inode 复活 | 第一轮没量 | M3 均分 119、M2 209（`agg-merges.txt:200`、`agg-merges.txt:115`） | 中 |
| 二·一直留·I-9.13 为假的版本 | 1439 | 1439（同一个数）；合并下 12161 / 110365 | 中（唯一一格） |
| 二·一直留·r、m | inode r 0，分配记录 m 3（读法漏洞，判决没列） | `C2HoldZ` 五个名字空间全 0 | 不中 |
| 三·(a)·早于才不收 | r 3 | `TSZ` inode r 0 | 不中 |
| 三·(a)·不晚于就不收 | m 1，F8 另有 r 1 | `TSZ` m 1（unlink 途中），F8 r 0 | m 中，r 不中 |
| 三·(b)·最后一批另写一条 | I-9.20 红 | 不写第二条；`FSZ` 把一条挪到最后一批 r = m = 0 | 不中 I-9.20，中 I-9.18 |
| 三·(c)·合并后分配记录 / 映射复活 | r 5（wide M2） | 「wide M2_dropped TSZ allocation_records histories=64 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=181 histories_m_pos=37」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-merges.txt:416`）；对照第一轮的「wide M2_dropped C3Z allocation_records histories=64 r_peak_max=5 r_out_peak_max=5 histories_r_pos=41 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/agg-merges.txt:391`） | r 不中，m 中（同 (a)） |
| 四·A、B 读得出的单元相同 | 9 / 9 | 带墓碑的写路径上 9 / 9 | 中 |

盘上新增写（量过的次数，每次的单元数是推的）：候选一 = 退役记录 E177 63 次、合并 M3 均分 123 次（`expected/counts-e177.txt:1`、`expected/counts-merges.txt:5`）；候选二一直留 = 零条目版本 63 次 / 127 次，每次一个 32 KiB 单元，不回收；`T` 族 = 0；`FSZ` = 0（把已有的一条挪位置）。

## 7　限度

- 模型是 E177 装置的拷贝，读侧是手写的择现行，不是 `crates/` 的扫描重建（`crates/` 没有）；只有干净发布之后的扫描，没有崩溃状态、没有坏镜像、只有单头。供出规则、零条目叶的放置、墓碑读侧都没在崩溃状态上量。
- 墓碑、零条目版本与退役记录同第一轮攻方腿一样放在模型专用区或叶序里、不占另外的树根；没量载体的树表、根、回收。
- 取样同第一轮：E177 51 段 × G-环 / G-宽（G-紧 没跑）、合并 64 段（一种生成器，种子 1–8）；「没打中」都是一次抽样，本腿没有 `s2` 样本。`r2-sweep` 的有序删 36 格是新造的，只配 M3 两种借法。
- 只有 1 号有数据，住最左那片；带数据文件被借走 / 被吸收的那一形（K3 的格）本腿没做。
- 所有数是副本上的，不进 kb；要引，由主 agent 在入库装置上重做（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节）。

## 没做什么

- 没替主 agent 撤销或维持判决，只交复核结果；没判 X2、X3、X4 的取舍。
- 没读禁读清单里的文件（第二轮攻方腿的报告与模型、`m3-w3-r1` 那一轮）；`ps` 里偶然看到别的会话的命令行，没读它们的文件。
- 没跑重型测试（checker 档、层 0、QEMU、变异整表）；没编译、没跑 `crates/` 里的代码；没动 `crates/`、kb、门禁。
- 没建重建模型（零条目叶放置、N1 分隔 key），没量崩溃状态，没量克隆头 / 快照，没量 K2 区间墓碑、N1、N2（不归本腿）。
- 没跑 `s2` 样本；「T 族 r = 0」「`C2HoldZ` r = m = 0」是一次抽样的量（模型确定，同输入同输出，但读侧是手写的）。
- `rerun2/` 只重跑了 e177、sweep、rollback、indist、merges-0 五段（排序后 sha256 逐个相同），merges-1 到 merges-7 与 summarize 只在 `rerun1/` 跑过一次。

## 草稿与副本

- 草稿目录 `~/.local/share/singlefs/work/m3-leaf-death-r2-sonnet/`：保留 `rerun1/*.out`、`rerun1/summary/`、`rerun2/*.out`、`runs/`（开发中的 r2-merges 分批产物）、`dev/`（开发用源码与补丁脚本）、`opus-expected/`（第一轮攻方腿 `expected/` 的拷贝，做回归用）、`r2-*.out`。
- 自己建的编译目录：`target-dev/`、`rerun1/target/`、`rerun2/target/`，交回前已删；仓副本没建。

## 模型目录每个文件的 sha256

`(cd research/prompts/m3-leaf-death-r2-sonnet-model && find . -type f | sort | xargs sha256sum)`：

```
381b9724cbe0861f817bc961b18fc47dfe1bc6763442309112d696fe9d56fa94  ./Cargo.lock
e1d65102dec434420f80b4bae77e42bffe38f0755eeb6918e66eafc8fd4cb05f  ./Cargo.toml
9963cb32ed35f64b0b53ccadf0adeda0d414765ee81e23b07a1d377c7c7489b2  ./compare.sh
19507c8fe3074bdb7f5de626437199a501587dd10adffcb2e506c6b2f27d489c  ./expected/agg-e177.txt
63f6c6157b0251731fe245618110c0a9e7d83f6503f26f86093f5e9facb95fe6  ./expected/agg-merges.txt
43d2857d623c407ac4ad32c6ea9619e3f846182e98e4a9896c9d168d634af246  ./expected/counts-e177.txt
0f417ea8c74c11a459f8992f57409da8d3f2ab83bbaf74f66bfd214050e4ab3d  ./expected/counts-merges.txt
749d4648c528ebdbc9aa885cbc4c24e73eacad60e9149f62d2d6b6ffe7c36afc  ./expected/indistinguishable.txt
143c6f40bdea8008b3063ae8621021d418faa2f30dc22092accc56d035a7d01a  ./expected/rollback.txt
04816739a317430b88e94d628fa9531a464e9685ba5889dd387579cf1b400cc0  ./expected/sorted-output-sha256.txt
5f4b4a69a99a9bc0fe2e64a531bd74da8c9331ab46dcb06265fc0617ebdba664  ./expected/sweep.txt
24f38310d8711a21a9e6dc3e2a70cb7f705c8d28971510340c0e463a6199d519  ./expected/window.txt
7886bad964bdd92cf9f8fd09d86ee24a3f75bd3c65c768577f627b34ea7d7e3a  ./README.md
ada347bd24a9e60ec3b33449f971388d235f21eea83739ee299c07a97d82adc5  ./rerun.sh
c88de285faa5c55de9d3925753a80e0132ba3cf863a9fce684d61daae7eaa922  ./src/leaf_death.rs
167b9e1f2a460acab15850648565abdd37ac67cdb509f86b05e82df82141a000  ./src/main.rs
ea9ca4c27ca7bb2445b23ff7188790b5b28b7e45c48df72f1f58d4bec159fd0f  ./tools/agg.awk
ecb808becf7ea4159ad8ee7927ebf1c5613d0182d3f2ab64da6fb0ec9c589859  ./tools/counts2.py
4f3efbe850942e06591f2d77b7f70275683e8e088fb665026e5df48e05fc8a69  ./tools/counts.awk
0371d1ca38705f2ccac7eb7ffc90e960aabe0b28b0197a5927910b7bfe517e63  ./tools/sweep.py
427e85718c77627c03bbc3d3cabbf9eee1f243a3894d93e2106fbf433e63ffc1  ./tools/window.py
```

## cite-check 末行

`python3 scripts/cite-check.py research/prompts/m3-leaf-death-r2-sonnet-output.md`：

```
  ✓ 核了 53 处引文，对上 53 处，没判 0 处（逐处列在上面）
```
