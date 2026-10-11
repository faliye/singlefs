## E176 inode 树叶合并三候选的容量与写代价 —— 已跑（两段）

<!-- doc-lint:not-numbers M1 M2 M3 A1 A2 A3 A4 A5 A6 Q1 Q2 Q3 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12 V13 V14 V15 V16 F1 F2 F3 F4 F5 F6 F7 -->

跑前登记 `research/prompts/e176-preregistration.md`（第二段跑完时 sha256 `f7599076bb469dcc483f98cb0f557aca5bc6d79759fe674b4b52f150b8291fa9`，含第十二节「修订」三条：修订 1 补 M3 f = 59 变体的阳性对照与它的公共窗口读法；修订 2 把对拍 bin 从可选特性挪成独立 manifest，见「做了什么」；修订 3 补第二段的三个公共窗口、三个变体的阳性对照与变异 V14–V16，第二段产物之前写）。回答的是岔路单 `research/prompts/m3-3-5-forks.md` 的 M 那一行：inode 树叶容器删到稀疏时怎么回收，C116（叶容器的分裂 / 合并纪律没有会失败的检查） 合并那一半与 C536（inode 树的「左从右借」让容器号条件为假）。

三个候选：M1 只删空即摘（`crates/` 今天）；M2 只做左吸收右（相邻两片合起来 ≤ 233 条就并）；M3 照 D8（核心索引结构） 已定项 6 原文，左吸收右加左从右借（主取样点下限 f = 117、均分借）。装置是独立手写模型 `scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs`，不与 `crates/` 共用代码；M1 那一臂另由对拍 bin `scripts/experiments/e176_crates_oracle/oracle.rs` 逐步比 `crates/singlefs-core/src/inode_tree.rs` 的两个纯函数。

**现状**：

- 第一段跑完：22 条序列（W-R 随机插删 p = 0.55 / 0.75 / 0.95 各五个种子，W-S 按号顺序删三点，W-A 隔号删四点）× 四个臂变体（M1、M2、M3、M3 f = 59）。第六节每一行在三族主取样点上都有判定，第八节标「第一段」的旋钮都有方向相反的一点，岔路单 M 行的够判条件满足。
- 总判定（主窗口，产物 `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out` 第 763–769 行）：Q1r 判「合并买到成倍容量」（W-R p = 0.55 五个种子 R12 都在 0.35 上下）；Q2p 两行（位置记法，Δ2p、Δ3p）判「不翻」，只差 W-R p = 0.75 种子 2 那一点（0.1046、0.1100 略过 0.10）；Q2c 两行（内容记法）与 Q3 两行判「可忽略」。f = 59 窗口六行判定与主窗口相同（第 770–775 行）。
- 敏感性：Q1r 对负载不稳定。同一行在 W-R p = 0.95、W-S 三点、W-A 留 3/4 上 R12 ≥ 0.90（判「代价小」），在 W-A 留 1/233 上 0.0711（判「成倍」）。Q2p 两行在插入概率旋钮上（0.55 对 0.75，种子 2）不稳定。
- 第二段跑完（触发条件「第一段 Q2 或 Q3 有哪一行离门槛不到 0.05」成立：Q2p 两行在 W-R p = 0.75 五个种子上在 0.0724 … 0.1100 之间；主 agent 点名跑）。同 22 条序列加三个变体：M3 借到刚好够下限（`M3_just_enough`）、M2 先查左边（`M2_left_first`）、M3 先查左边（`M3_left_first`），各开一个公共窗口（`just_enough` 窗口 = M1、M2、M3_just_enough；`left_first` 窗口 = M1、M2_left_first、M3_left_first；修订 3）。总判定（产物 `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out` 第 1137–1148 行）：
  - 先查左边窗口六行的判定与主窗口相同：Q2p 两行「不翻」（W-R p = 0.75 种子 2 上 Δ2p 0.1046、Δ3p 0.1111），Q2c、Q3 四行「可忽略」。
  - 借到刚好够下限窗口：Q2p 两行「不翻」；Q2c Δ3c、Q3 两行由主窗口的「可忽略」变成「不翻」。W-S 两个顺序删点上 Δ3c 0.5018、W-A 留 1/233 上 0.4568，都 < 1.00。三个窗口里没有一行判到「显著」，Q2p 也没有在哪个窗口判到「可忽略」。
  - 敏感性：「借多少」旋钮在 15 对上判定不同，都是主窗口「可忽略」、借到刚好够下限「不翻」：Δ3p 在 W-R p = 0.75 种子 1、4、5 上各 1 对，涉及 M3 的四行在 W-S 两个升序删点与 W-A 留 1/233 上各 4 对；「先查哪一对」旋钮 132 对判定全相同，其中 99 对两点值逐位相等，挪门槛自证做不了。
- 停机条款没打中（两段各跑一次）：对拍 bin 在 22 条序列上逐步比到 M1 被拒，零处不同；本地常量 233、135 与 `crates/` 回比相等。第二段那次对拍用改过的源码重编，产物与第一段那份逐字节相同。作废条款没打中：第一段乙类锚点 26 格全对，第二段 29 格全对；第一段 44 个、第二段 66 个公共窗口三条臂的操作流前缀指纹全相同；两段各跑两遍（8 个线程与 1 个线程），sha256 相同。丙类断言一条没红（任何一条红都会让装置当场 panic、不出收尾行）。第二段的主窗口与 M1、M2、M3 三条臂的 396 行与第一段逐字相同，第一段用改过的源码重跑，与入库那份逐字节相同。

确定性模型：两遍逐字节相同说明的是没有隐藏状态、线程数不影响输出，不是统计上稳定；证据强度靠钉绝对值的锚点、阳性对照、对拍与变异表。

### 做了什么

- **模型**：叶容器表按叶序存容器号与按号升序的记录号。插入照 `crates/` 今天的落法：最右片不满就追加，满了就末尾分裂出新片、容器号 = 这条的号；新建之后片数 > 135 就拒，树不变。删除：从装着它的那片删掉，删空就摘（三臂相同）；M2 摘完查新相邻的那一对、没删空就从那片起查右邻与左邻两对，和 ≤ 233 就左吸收右，并到没有一对装得下；M3 这次一次都没并、那片没删空、记录数 < f 且有右邻时，从右邻借最小的 ⌊(\|R\| − \|X\|) ÷ 2⌋ 条，借完让右邻与它右邻那一对照 M2 的规则查。
- **两种重写记法**：位置记法 = 内容变了的叶，加上第一个被摘掉的位置及其右边每一片留下的叶（`crates/` 的重写清单就是这样，`inode_tree.rs` 第 308–311 行）；内容记法 = 只数内容变了、发布之后还在的叶。「一次 unlink」只数最后一批那一次发布。
- **逐步同走**：一条序列的四个臂变体一步一步同走，每条臂各用一个生成器实例、各算各的操作流 SHA-256；公共窗口 = 从头到窗口里三条臂最早一次不接受操作之前。主窗口三条臂是 M1、M2、M3；f = 59 窗口三条臂是 M1、M2、M3 f = 59（修订 1 第 2 行）。每条臂每次发布之后跑丙类断言：片内升序、每片 1 … 233 条、容器号 ≤ 片内最小号、沿叶序容器号严格递增、相邻两片 key 区间不交叠；I-9.4（容器号不超最小 key） 第三句在 M1、M2 上断言、在 M3 上计数；M2、M3 相邻两片之和 ≥ 234；内容记法 M1、M2 每次 ≤ 1 片、M3 ≤ 2 片。
- **负载**：W-R 用 splitmix64（种子即初值），每一步取 53 位均匀数 u，活集合空或 u × 100 < p 百分数 × 2^53 就插入（按整数比，p 取精确十进制），否则删活集合第 next() mod 长度 个、swap_remove。W-S 一轮插 1000 条再删最小（或最大）的 D 个。W-A 一轮插 A 条连号，再按号升序删掉这一批里登记的那一份。每条臂最多吃 2 × 10⁷ 个操作；第一段 88 条臂都在上限内被拒，没有「未撞到」。
- **对拍**：对拍 bin 把每条序列在 M1 被拒之前的每一步同时喂给模型 M1 与 `crates/` 的 `write_records_into_leaf_containers` / `remove_record_from_leaf_containers`，逐步比片数、每片（容器号、记录数、最小号、最大号）、位置记法重写清单逐项、内容记法、摘片数、被拒的那一步与拒的格。它以 `#[path]` 引同一份模型源码，用同一目录下自己的 `Cargo.toml` 编（自成一个 workspace，以路径依赖引 `crates/singlefs-core`、`crates/singlefs-format`，只读）。没挂在 `scripts/experiments/Cargo.toml` 的可选特性上，原因是可选的路径依赖照样要 cargo 读到 `../../crates` 下的 manifest，而 `scripts/mutation/mutate.sh` 只把 `research/` 拷进临时目录编译，那样整个包每一张变异表的基线都编不过（登记修订 2，第一次跑变异表时撞上）。
- **对拍的判别力**（登记第九节末）：在草稿目录的副本上给模型打开 V1（M1 删空摘片时位置记法不记右边各片），对拍在 W-S 主取样点第 1233 个操作报出第一处不同，与登记写的一样。两段各做一次，第二段那份与第一段逐字节相同。读数在「对拍」一节。
- **第二段**（登记第五节第二段那一行、修订 3）：同一份模型加 `segment-two` 入口，22 条序列各喂六个臂变体（M1、M2、M3、M3_just_enough、M2_left_first、M3_left_first），开三个公共窗口（`main`、`just_enough`、`left_first`）。M3_just_enough 借 f − \|X\| 条（借到刚好 117）；M2_left_first、M3_left_first 删完之后先查左邻与 X 那一对，再查 X 与右邻，下限与借多少取主旋钮。「借多少」旋钮比主窗口与 `just_enough` 窗口里涉及 M3 的四行，「先查哪一对」旋钮比主窗口与 `left_first` 窗口的六行。Q1r 不在这两个旋钮上判（登记第六节把它定成 M2 取主旋钮），M2_left_first 被拒时的活数只当读数报。

### 复跑

```
# 仓根；模型在 research/（不依赖 crates/），线程数读 SINGLEFS_THREAD_CAP（capped.sh 设），只影响挂钟、不影响输出
cd /home/fy5090/code/singlefs/research
bash scripts/hosts/capped.sh 8 cargo build --release --manifest-path scripts/experiments/Cargo.toml --bin e176-inode-leaf-merge-capacity-and-rewrites
bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 8 ./target/release/e176-inode-leaf-merge-capacity-and-rewrites segment-one
bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 8 ./target/release/e176-inode-leaf-merge-capacity-and-rewrites segment-two
# 登记的复跑（scripts/experiments/replay.sh 两行：segment-one、segment-two）
cd .. && bash scripts/experiments/replay.sh E176
# 对拍（独立 manifest，以路径依赖只读引 crates/；replay.sh 不编它）
cd research
bash scripts/hosts/capped.sh 8 cargo build --release --manifest-path scripts/experiments/e176_crates_oracle/Cargo.toml --target-dir target
bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 8 ./target/release/e176-inode-leaf-merge-crates-oracle
# 单测 22 条与变异表 16 条
bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 8 cargo test --release --bin e176-inode-leaf-merge-capacity-and-rewrites
MUTATE_JOBS=4 MUTATE_CARGO_JOBS=2 MUTATE_MEMORY_MAX=4G bash scripts/mutation/mutate.sh e176-inode-leaf-merge-capacity-and-rewrites scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs mutations/e176_inode_leaf_merge_capacity_and_rewrites.tsv
```

第一段约 16 秒（8 个线程）、23 秒（1 个线程）；第二段约 19 秒（8 个线程）、25 秒（1 个线程，都含内存包装）；对拍约 23 秒（8 个线程）。产物：

| 文件 | 是什么 |
|---|---|
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out` | 第一段：config、锚点与阳性对照、22 条序列 × 四个臂变体、总判定、敏感性（991 行，sha256 `fe3a322978d570c3a6db180ac6e10d00a9e744c9b16b403f4e49105ef5f58dc4`；8 个线程与 1 个线程两遍逐字节相同；第二段改过的源码重跑一次，仍逐字节相同） |
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out` | 第二段：config、锚点与阳性对照（47 格，登记答案 29 格）、22 条序列 × 六个臂变体 × 三个窗口、总判定、两个旋钮的敏感性（1370 行，sha256 `8906bfd7d87a516abbe61c74c410dfc00d9c967b57fbad916f08f14c21e60737`；8 个线程与 1 个线程两遍逐字节相同） |
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle.out` | 第一段的对拍：常量回比、`crates/` 单测三种形态回比、22 条序列逐步比到 M1 被拒（28 行，sha256 `c0a6635e1cab890f1b9dc134df1fb43e53adc2ba2c993f59bbaef981f8c69ae3`；与改 manifest 之前在草稿目录跑的那一次逐字节相同） |
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle-2026-10-08.out` | 第二段的对拍（用第二段的源码重编）：与第一段那份逐字节相同（sha256 `c0a6635e1cab890f1b9dc134df1fb43e53adc2ba2c993f59bbaef981f8c69ae3`） |
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1.out` | 第一段对拍的判别力：模型上打开 V1 之后的对拍输出（草稿目录副本上编的，判决行故意是 false） |
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1-2026-10-08.out` | 第二段对拍的判别力（第二段的源码上打开 V1）：与第一段那份逐字节相同（sha256 `29e6ab1ed77318939f8d377b7c90728c7a4028a24771d0e8aad080c71f909bb3`） |
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-unit-tests.out` | 第一段的单测输出（14 个） |
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-unit-tests-2026-10-08.out` | 第二段的单测输出（22 个，今天这份源码） |
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-mutations.out` | 第一段 `mutate.sh` 整张表（13 条）的输出（stderr 头一行在前） |
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-mutations-2026-10-08.out` | 第二段 `mutate.sh` 整张表（16 条，今天这份源码与变异表）的输出（stderr 头一行在前） |

### 判决（整行抄自产物）

`grep -n 'name=verdict'` 列出的全部判决行：

```
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:33:E7RESULT name=verdict scope=positive_controls anchors_registered=26 anchors_matching=26 all_registered_anchors_match=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:760:E7RESULT name=verdict scope=window_prefixes windows=44 windows_with_equal_prefixes=44 all_window_prefixes_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:761:E7RESULT name=verdict scope=f6_m1_reached points=22 m1_not_reached_points=0 m1_reached_on_every_point=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:762:E7RESULT name=verdict scope=f5_m3_borrows_on_main_points main_point_m3_borrows=5930 m3_borrowed_on_some_main_point=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:763:E7RESULT name=verdict scope=row row=Q1r window=main points=22 judged_points=22 verdict=merge_buys_double_capacity
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:764:E7RESULT name=verdict scope=row row=Q2p_delta2 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:765:E7RESULT name=verdict scope=row row=Q2p_delta3 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:766:E7RESULT name=verdict scope=row row=Q2c_delta2 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:767:E7RESULT name=verdict scope=row row=Q2c_delta3 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:768:E7RESULT name=verdict scope=row row=Q3p_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:769:E7RESULT name=verdict scope=row row=Q3c_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:770:E7RESULT name=verdict scope=row row=Q2p_delta2 window=floor59 points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:771:E7RESULT name=verdict scope=row row=Q2p_delta3 window=floor59 points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:772:E7RESULT name=verdict scope=row row=Q2c_delta2 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:773:E7RESULT name=verdict scope=row row=Q2c_delta3 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:774:E7RESULT name=verdict scope=row row=Q3p_delta32 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:775:E7RESULT name=verdict scope=row row=Q3c_delta32 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:990:E7RESULT name=verdict scope=sensitivity_self_proof same_pairs=200 unstable_pairs=14 moved_to_unstable=162 equal_values_not_separable=38 every_separable_same_pair_moved_to_unstable=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=22 constants_equal=true crates_unit_test_shapes_equal=true all_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=15 constants_equal=true crates_unit_test_shapes_equal=false all_equal=false
```

前两份产物的判决行没有 false 或 not_run 字段，违例、不匹配、歧义、失败类的整数计数都是 0。第三份是判别力自证，`all_equal=false` 与 `crates_unit_test_shapes_equal=false` 是登记预期内的：模型上打开了 V1，对拍应当报出不同。`verdict=no_flip` 不是「没过」：它是第六节「其余 ⇒ 不翻」那一档的判定。

第二段那一次（对拍、判别力与第二段产物）`grep -n 'name=verdict'` 列出的全部判决行：

```
research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle-2026-10-08.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=22 constants_equal=true crates_unit_test_shapes_equal=true all_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1-2026-10-08.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=15 constants_equal=true crates_unit_test_shapes_equal=false all_equal=false
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:49:E7RESULT name=verdict scope=positive_controls anchors_registered=29 anchors_matching=29 all_registered_anchors_match=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1128:E7RESULT name=verdict scope=window_prefixes windows=66 windows_with_equal_prefixes=66 all_window_prefixes_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1129:E7RESULT name=verdict scope=f6_m1_reached points=22 m1_not_reached_points=0 m1_reached_on_every_point=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1130:E7RESULT name=verdict scope=row row=Q1r window=main points=22 judged_points=22 verdict=merge_buys_double_capacity
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1131:E7RESULT name=verdict scope=row row=Q2p_delta2 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1132:E7RESULT name=verdict scope=row row=Q2p_delta3 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1133:E7RESULT name=verdict scope=row row=Q2c_delta2 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1134:E7RESULT name=verdict scope=row row=Q2c_delta3 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1135:E7RESULT name=verdict scope=row row=Q3p_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1136:E7RESULT name=verdict scope=row row=Q3c_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1137:E7RESULT name=verdict scope=row row=Q2p_delta2 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1138:E7RESULT name=verdict scope=row row=Q2p_delta3 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1139:E7RESULT name=verdict scope=row row=Q2c_delta2 window=just_enough points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1140:E7RESULT name=verdict scope=row row=Q2c_delta3 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1141:E7RESULT name=verdict scope=row row=Q3p_delta32 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1142:E7RESULT name=verdict scope=row row=Q3c_delta32 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1143:E7RESULT name=verdict scope=row row=Q2p_delta2 window=left_first points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1144:E7RESULT name=verdict scope=row row=Q2p_delta3 window=left_first points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1145:E7RESULT name=verdict scope=row row=Q2c_delta2 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1146:E7RESULT name=verdict scope=row row=Q2c_delta3 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1147:E7RESULT name=verdict scope=row row=Q3p_delta32 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1148:E7RESULT name=verdict scope=row row=Q3c_delta32 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1369:E7RESULT name=verdict scope=sensitivity_self_proof same_pairs=205 unstable_pairs=15 moved_to_unstable=74 equal_values_not_separable=131 every_separable_same_pair_moved_to_unstable=true
```

第二段的对拍那份没有 false 字段；判别力那份的两个 false 照第一段的理由是预期内的。第二段产物的判决行没有 false 或 not_run 字段，`m1_not_reached_points=0`。主窗口那几行（第 1130–1136 行）与第一段第 763–769 行判定相同。`equal_values_not_separable=131` 不是「没过」：它是挪门槛自证做不了的对数，两点值逐位相等、门槛挪不进两点之间（登记第八节没写这种情形怎么报，这里单列，不并进别的计数）。

### 锚点与阳性对照（整行抄自 `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out` 第 2–32 行）

登记第七节甲、乙类全部格，加修订 1 的 f = 59 变体（A2 … A6 只报不判，`matches=unregistered`）。空间放大那一格（不设 135 上限的 M1 建满 10⁶ 再删九成）在第 29 行，`ratio=9.98` 与 D8（核心索引结构） 已定项 6 末尾「9.98 倍」那一句相同，F3 没打中。阳性对照 F1 没打中：A2 上 M2 活数 31 455 > M1 的 367，A3 上 M3 借 1 次，A5 上 M2 并 1 次。

```
E7RESULT name=anchor cell=A1 arm=M1 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M2 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M3 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M3_floor59 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A2 arm=M1 refused_insert=31688 live=367 containers=135 merges=0 borrows=0 matches=true
E7RESULT name=anchor cell=A2 arm=M2 refused_insert=62776 live=31455 containers=135 merges=134 borrows=0 matches=true
E7RESULT name=anchor cell=A2 arm=M3 refused_insert=62776 live=31455 containers=135 merges=134 borrows=918 matches=true
E7RESULT name=anchor cell=A2 arm=M3_floor59 refused_insert=62776 live=31455 containers=135 merges=134 borrows=58 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M1 content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=true
E7RESULT name=anchor cell=A3 arm=M2 content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=true
E7RESULT name=anchor cell=A3 arm=M3 content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=174/176 matches=true
E7RESULT name=anchor cell=A3_boundary arm=M3 leaf0_largest=291 leaf1_smallest=292 matches=true
E7RESULT name=anchor cell=A3 arm=M3_floor59 content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M1 content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1 matches=true
E7RESULT name=anchor cell=A4 arm=M2 content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1 matches=true
E7RESULT name=anchor cell=A4 arm=M3 content=240 position=242 containers=3 merges=1 borrows=7 third_sentence_false=0 sizes=233/233/1 matches=true
E7RESULT name=anchor cell=A4 arm=M3_floor59 content=234 position=236 containers=3 merges=1 borrows=1 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A5 arm=M1 content=234 position=234 containers=5 merges=0 borrows=0 third_sentence_false=0 sizes=233/116/116/233/1 matches=true
E7RESULT name=anchor cell=A5 arm=M2 content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1 matches=true
E7RESULT name=anchor cell=A5 arm=M3 content=241 position=243 containers=4 merges=1 borrows=7 third_sentence_false=0 sizes=233/232/233/1 matches=true
E7RESULT name=anchor cell=A5 arm=M3_floor59 content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M1 content=233 position=233 containers=2 merges=0 borrows=0 third_sentence_false=0 sizes=116/117 matches=true
E7RESULT name=anchor cell=A6 arm=M2 content=233 position=233 containers=1 merges=1 borrows=0 third_sentence_false=0 sizes=233 matches=true
E7RESULT name=anchor cell=A6 arm=M3 content=240 position=240 containers=1 merges=1 borrows=7 third_sentence_false=0 sizes=233 matches=true
E7RESULT name=anchor cell=A6 arm=M3_floor59 content=233 position=233 containers=1 merges=1 borrows=0 third_sentence_false=0 sizes=233 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M3_just_enough content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=117/233 matches=true
E7RESULT name=anchor cell=A5 arm=M2_left_first content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1 matches=true
E7RESULT name=anchor cell=C1 arm=M1_unlimited leaves=4292 live=100000 ideal=430 ratio=9.98 matches=true
E7RESULT name=anchor cell=crates_insert_500 arm=M1 sizes=233/233/34 container_numbers=1/234/467 matches=true
E7RESULT name=anchor cell=crates_insert_31456 arm=M1 refused_inode=31456 refused_containers=136 capacity=135 matches=true
E7RESULT name=anchor cell=crates_drop_middle arm=M1 position_rewritten=[1] dropped=1 matches=true
```

### 读数：Q1（被拒时的活数与 R12 = L_M1 ÷ L_M2）

读法 1：一条臂第一次有插入被拒时，最后一个被接受的版本里的记录条数。88 条臂都被拒（产物第 761 行 `m1_not_reached_points=0`；M2、M3 也都 `stop=refused`），没有「上界」点。列：序列、L_M1、L_M2、L_M3（Q1b，附带）、L_M3 f = 59、R12、判定。命令（仓根）与原样输出：

```
awk '/name=arm_run/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} live[v["sequence"],v["arm"]]=v["live_at_stop"]} /name=point_verdict/ && / window=main / && / row=Q1r /{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} print v["sequence"], live[v["sequence"],"M1"], live[v["sequence"],"M2"], live[v["sequence"],"M3"], live[v["sequence"],"M3_floor59"], v["value"], v["verdict"]}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out
```

```
wr_p055_seed1 8070 22452 22116 22452 0.3594 merge_buys_double_capacity
wr_p055_seed2 8177 22422 21909 21995 0.3647 merge_buys_double_capacity
wr_p055_seed3 7978 22190 21867 22190 0.3595 merge_buys_double_capacity
wr_p055_seed4 7933 22823 21820 22771 0.3476 merge_buys_double_capacity
wr_p055_seed5 8083 22396 22260 21912 0.3609 merge_buys_double_capacity
wr_p075_seed1 21360 24167 23551 24167 0.8838 no_flip
wr_p075_seed2 21173 24427 24264 24427 0.8668 no_flip
wr_p075_seed3 21378 24225 24225 24225 0.8825 no_flip
wr_p075_seed4 20994 24299 23980 24299 0.8640 no_flip
wr_p075_seed5 21157 24325 24163 24325 0.8698 no_flip
wr_p095_seed1 29865 29865 29865 29865 1.0000 no_merge_capacity_cost_small
wr_p095_seed2 29810 29811 29811 29811 1.0000 no_merge_capacity_cost_small
wr_p095_seed3 29802 29803 29803 29803 1.0000 no_merge_capacity_cost_small
wr_p095_seed4 29815 29815 29815 29815 1.0000 no_merge_capacity_cost_small
wr_p095_seed5 29821 29821 29821 29821 1.0000 no_merge_capacity_cost_small
ws_a1000_d500_smallest_first 31245 31245 31245 31245 1.0000 no_merge_capacity_cost_small
ws_a1000_d950_smallest_first 31319 31319 31284 31278 1.0000 no_merge_capacity_cost_small
ws_a1000_d500_largest_first 31455 31455 31455 31455 1.0000 no_merge_capacity_cost_small
wa_keep1of2_a1000 17132 27054 25464 27054 0.6333 no_flip
wa_keep1of2_a2330 16773 31455 31455 31455 0.5332 no_flip
wa_keep1of233_a2330 2236 31455 31081 31081 0.0711 merge_buys_double_capacity
wa_keep3of4_a1000 24606 25478 25478 25478 0.9658 no_merge_capacity_cost_small
```

Q1r 的总判定是「合并买到成倍容量」：主取样点 W-R p = 0.55 五个种子 R12 在 0.3476 … 0.3647，都 < 0.50。对 M2 / M3 不利的那一半同样在表里：W-S 三点、W-R p = 0.95 五个种子、W-A 留 3/4 上 R12 ≥ 0.90（不合并的容量代价小），W-S 三点上 M1 与 M2 被拒时的活数逐个相同（顺序删把整片删空，M1 照样摘掉）。W-A 留 1/2（主取样点）是 0.6333，落在「不翻」。M3 被拒时的活数在 12 条序列上小于 M2、10 条相等、0 条大于（W-R p = 0.55 五个种子 21 820 … 22 260 对 22 190 … 22 823）：借在这组负载上没有多换来容量。数法（仓根）与原样输出：

```
awk '/name=arm_run/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} live[v["sequence"],v["arm"]]=v["live_at_stop"]; seq[v["sequence"]]=1} END{less=0; equal=0; more=0; for(s in seq){ if (live[s,"M3"]+0 < live[s,"M2"]+0) less++; else if (live[s,"M3"]+0 == live[s,"M2"]+0) equal++; else more++ } print "m3_less_than_m2=" less, "equal=" equal, "m3_more_than_m2=" more}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out
m3_less_than_m2=12 equal=10 m3_more_than_m2=0
```

### 读数：Q2、Q3（主窗口里每次 unlink 平均重写片数之差）

Δ = (臂甲的和 − 臂乙的和) ÷ 窗口里最后一批的次数，44 个窗口里最后一批最少 1643 次（`grep 'name=window ' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out | sed 's/.*unlinks=\([0-9]*\).*/\1/' | sort -n | head -1` 输出 `1643`），没有「样本不足」。负值是那条臂比 M1 少重写。命令（仓根）与原样输出：

```
awk '/name=point_verdict/ && / window=main / && !/ row=Q1r /{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} out[v["sequence"]]=out[v["sequence"]] " " v["row"] "=" v["value"]; if(!(v["sequence"] in seen)){seen[v["sequence"]]=1; order[++n]=v["sequence"]}} END{for(i=1;i<=n;i++) print order[i] out[order[i]]}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out
```

```
wr_p055_seed1 Q2p_delta2=-0.0260 Q2p_delta3=-0.0157 Q2c_delta2=0.0012 Q2c_delta3=0.0111 Q3p_delta32=0.0103 Q3c_delta32=0.0099
wr_p055_seed2 Q2p_delta2=-0.0393 Q2p_delta3=-0.0301 Q2c_delta2=0.0013 Q2c_delta3=0.0115 Q3p_delta32=0.0092 Q3c_delta32=0.0102
wr_p055_seed3 Q2p_delta2=-0.0230 Q2p_delta3=-0.0131 Q2c_delta2=0.0012 Q2c_delta3=0.0111 Q3p_delta32=0.0099 Q3c_delta32=0.0099
wr_p055_seed4 Q2p_delta2=-0.0172 Q2p_delta3=-0.0076 Q2c_delta2=0.0013 Q2c_delta3=0.0118 Q3p_delta32=0.0097 Q3c_delta32=0.0105
wr_p055_seed5 Q2p_delta2=-0.0219 Q2p_delta3=-0.0128 Q2c_delta2=0.0013 Q2c_delta3=0.0108 Q3p_delta32=0.0091 Q3c_delta32=0.0095
wr_p075_seed1 Q2p_delta2=0.0881 Q2p_delta3=0.0724 Q2c_delta2=0.0000 Q2c_delta3=0.0047 Q3p_delta32=-0.0158 Q3c_delta32=0.0047
wr_p075_seed2 Q2p_delta2=0.1046 Q2p_delta3=0.1100 Q2c_delta2=0.0000 Q2c_delta3=0.0038 Q3p_delta32=0.0054 Q3c_delta32=0.0038
wr_p075_seed3 Q2p_delta2=0.0796 Q2p_delta3=0.0822 Q2c_delta2=0.0000 Q2c_delta3=0.0040 Q3p_delta32=0.0026 Q3c_delta32=0.0040
wr_p075_seed4 Q2p_delta2=0.0985 Q2p_delta3=0.0786 Q2c_delta2=0.0000 Q2c_delta3=0.0041 Q3p_delta32=-0.0199 Q3c_delta32=0.0041
wr_p075_seed5 Q2p_delta2=0.0935 Q2p_delta3=0.0936 Q2c_delta2=0.0000 Q2c_delta3=0.0046 Q3p_delta32=0.0001 Q3c_delta32=0.0046
wr_p095_seed1 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed2 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed3 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed4 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed5 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
ws_a1000_d500_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0299 Q2c_delta2=0.0000 Q2c_delta3=0.0342 Q3p_delta32=0.0299 Q3c_delta32=0.0342
ws_a1000_d950_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0300 Q2c_delta2=0.0000 Q2c_delta3=0.0343 Q3p_delta32=0.0300 Q3c_delta32=0.0343
ws_a1000_d500_largest_first Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of2_a1000 Q2p_delta2=0.0050 Q2p_delta3=0.0055 Q2c_delta2=0.0000 Q2c_delta3=0.0016 Q3p_delta32=0.0004 Q3c_delta32=0.0016
wa_keep1of2_a2330 Q2p_delta2=0.0172 Q2p_delta3=0.0172 Q2c_delta2=0.0001 Q2c_delta3=0.0001 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of233_a2330 Q2p_delta2=0.0191 Q2p_delta3=0.0457 Q2c_delta2=0.0004 Q2c_delta3=0.0275 Q3p_delta32=0.0265 Q3c_delta32=0.0271
wa_keep3of4_a1000 Q2p_delta2=0.0003 Q2p_delta3=0.0003 Q2c_delta2=0.0003 Q2c_delta3=0.0003 Q3p_delta32=0.0000 Q3c_delta32=0.0000
```

两种记法下的结论各写一句，不合成一句：

- **位置记法（`crates/` 今天的记法）**：Δ2p、Δ3p 总判定「不翻」。主取样点上两行都 ≤ 0.10（W-R p = 0.55 五个种子是负的，−0.0393 … −0.0076：M1 删空摘片时右边各片按位置左移全部重写，单次可到 133 片，M2 并得早、摘片时右边剩得少）。不利于「合并可忽略」的那一半：W-R p = 0.75 五个种子 Δ2p 0.0796 … 0.1046、Δ3p 0.0724 … 0.1100，种子 2 两行都略过 0.10。那一族 M1 在窗口里摘过的只有最右那一片（位置记法单次最大 1，产物里 M1 的 `position_largest=1`），M2、M3 的并把右边各片挪了位置（单次最大 90 … 105）。
- **内容记法**：Δ2c、Δ3c 总判定「可忽略」，最大的是 W-S 两点的 Δ3c（0.0342、0.0343）与 W-A 留 1/233 的 0.0275；Δ2c 最大 0.0013。
- **Q3（M3 − M2）**：两种记法都「可忽略」，最大 0.0343（W-S 顺序删 950 那一点，内容记法）。借在主取样点上真的发生了（F5 判决行 `main_point_m3_borrows=5930`），平均到每次 unlink 不到 0.035 片。

主取样点的窗口重写分布（整行抄自同一份产物；直方图五格是 0、1、2、3–9、10–134 片各几次）：

```
39:E7RESULT name=window_rewrites sequence=wr_p055_seed1 window=main arm=M1 unlinks=35448 content_sum=35406 position_sum=38633 content_mean=0.9988 position_mean=1.0898 content_largest=1 position_largest=127 content_histogram=42/35406/0/0/0 position_histogram=1/35406/0/0/41
40:E7RESULT name=window_rewrites sequence=wr_p055_seed1 window=main arm=M2 unlinks=35448 content_sum=35447 position_sum=37712 content_mean=1.0000 position_mean=1.0639 content_largest=1 position_largest=46 content_histogram=1/35447/0/0/0 position_histogram=1/35320/1/25/101
41:E7RESULT name=window_rewrites sequence=wr_p055_seed1 window=main arm=M3 unlinks=35448 content_sum=35798 position_sum=38076 content_mean=1.0099 position_mean=1.0741 content_largest=2 position_largest=48 content_histogram=1/35096/351/0/0 position_histogram=1/34971/351/24/101
72:E7RESULT name=window_rewrites sequence=wr_p055_seed2 window=main arm=M1 unlinks=37793 content_sum=37743 position_sum=41605 content_mean=0.9987 position_mean=1.1009 content_largest=1 position_largest=130 content_histogram=50/37743/0/0/0 position_histogram=1/37743/0/2/47
73:E7RESULT name=window_rewrites sequence=wr_p055_seed2 window=main arm=M2 unlinks=37793 content_sum=37792 position_sum=40119 content_mean=1.0000 position_mean=1.0615 content_largest=1 position_largest=45 content_histogram=1/37792/0/0/0 position_histogram=1/37657/3/27/105
74:E7RESULT name=window_rewrites sequence=wr_p055_seed2 window=main arm=M3 unlinks=37793 content_sum=38176 position_sum=40467 content_mean=1.0101 position_mean=1.0708 content_largest=2 position_largest=45 content_histogram=1/37408/384/0/0 position_histogram=1/37278/385/28/101
105:E7RESULT name=window_rewrites sequence=wr_p055_seed3 window=main arm=M1 unlinks=36583 content_sum=36535 position_sum=39784 content_mean=0.9987 position_mean=1.0875 content_largest=1 position_largest=133 content_histogram=48/36535/0/0/0 position_histogram=3/36535/0/0/45
106:E7RESULT name=window_rewrites sequence=wr_p055_seed3 window=main arm=M2 unlinks=36583 content_sum=36580 position_sum=38942 content_mean=0.9999 position_mean=1.0645 content_largest=1 position_largest=48 content_histogram=3/36580/0/0/0 position_histogram=3/36447/2/28/103
107:E7RESULT name=window_rewrites sequence=wr_p055_seed3 window=main arm=M3 unlinks=36583 content_sum=36942 position_sum=39303 content_mean=1.0098 position_mean=1.0744 content_largest=2 position_largest=49 content_histogram=3/36218/362/0/0 position_histogram=3/36088/363/27/102
138:E7RESULT name=window_rewrites sequence=wr_p055_seed4 window=main arm=M1 unlinks=37574 content_sum=37524 position_sum=40519 content_mean=0.9987 position_mean=1.0784 content_largest=1 position_largest=131 content_histogram=50/37524/0/0/0 position_histogram=2/37524/0/1/47
139:E7RESULT name=window_rewrites sequence=wr_p055_seed4 window=main arm=M2 unlinks=37574 content_sum=37572 position_sum=39871 content_mean=0.9999 position_mean=1.0611 content_largest=1 position_largest=46 content_histogram=2/37572/0/0/0 position_histogram=2/37436/3/31/102
140:E7RESULT name=window_rewrites sequence=wr_p055_seed4 window=main arm=M3 unlinks=37574 content_sum=37966 position_sum=40235 content_mean=1.0104 position_mean=1.0708 content_largest=2 position_largest=44 content_histogram=2/37178/394/0/0 position_histogram=2/37045/395/32/100
171:E7RESULT name=window_rewrites sequence=wr_p055_seed5 window=main arm=M1 unlinks=37279 content_sum=37228 position_sum=40474 content_mean=0.9986 position_mean=1.0857 content_largest=1 position_largest=127 content_histogram=51/37228/0/0/0 position_histogram=4/37228/0/1/46
172:E7RESULT name=window_rewrites sequence=wr_p055_seed5 window=main arm=M2 unlinks=37279 content_sum=37275 position_sum=39658 content_mean=0.9999 position_mean=1.0638 content_largest=1 position_largest=47 content_histogram=4/37275/0/0/0 position_histogram=4/37139/3/28/105
173:E7RESULT name=window_rewrites sequence=wr_p055_seed5 window=main arm=M3 unlinks=37279 content_sum=37630 position_sum=39997 content_mean=1.0094 position_mean=1.0729 content_largest=2 position_largest=47 content_histogram=4/36920/355/0/0 position_histogram=4/36788/356/27/104
534:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=main arm=M1 unlinks=30500 content_sum=30370 position_sum=39229 content_mean=0.9957 position_mean=1.2862 content_largest=1 position_largest=133 content_histogram=130/30370/0/0/0 position_histogram=0/30370/0/6/124
535:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=main arm=M2 unlinks=30500 content_sum=30370 position_sum=39229 content_mean=0.9957 position_mean=1.2862 content_largest=1 position_largest=133 content_histogram=130/30370/0/0/0 position_histogram=0/30370/0/6/124
536:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=main arm=M3 unlinks=30500 content_sum=31413 position_sum=40142 content_mean=1.0299 position_mean=1.3161 content_largest=2 position_largest=133 content_histogram=0/29587/913/0/0 position_histogram=0/29457/913/6/124
633:E7RESULT name=window_rewrites sequence=wa_keep1of2_a1000 window=main arm=M1 unlinks=16500 content_sum=16500 position_sum=16500 content_mean=1.0000 position_mean=1.0000 content_largest=1 position_largest=1 content_histogram=0/16500/0/0/0 position_histogram=0/16500/0/0/0
634:E7RESULT name=window_rewrites sequence=wa_keep1of2_a1000 window=main arm=M2 unlinks=16500 content_sum=16500 position_sum=16583 content_mean=1.0000 position_mean=1.0050 content_largest=1 position_largest=4 content_histogram=0/16500/0/0/0 position_histogram=0/16467/0/33/0
635:E7RESULT name=window_rewrites sequence=wa_keep1of2_a1000 window=main arm=M3 unlinks=16500 content_sum=16526 position_sum=16590 content_mean=1.0016 position_mean=1.0055 content_largest=2 position_largest=4 content_histogram=0/16474/26/0/0 position_histogram=0/16441/38/21/0
```

### 轨迹（第八节）

被谓词消费的量是叶容器片数（片数 > 135 就拒）。每条臂报片数峰值、等于 135 的发布次数、期末片数；填充率最小值与被拒前最后一版；M3 的 I-9.4（容器号不超最小 key） 第三句为假的相邻对数峰值、为正的发布次数、期末值。填充率最小值在 88 条臂上都是 0.0043（= 1 ÷ 233，第一次建 inode 那一版），这一列只说明起点，不说明删到稀疏时的样子；删到稀疏时的样子看被拒前最后一版那一列。主取样点的轨迹行（整行抄）：

```
34:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=78967 operations_accepted=78966 live_at_stop=8070 live_over_31455=0.2566 containers_peak=135 publishes_at_135=651 containers_final=135 fill_smallest=0.0043 fill_final=0.2566 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=fe4304e70876af50f0fa91b8f2c75791c72d36710fdc199dddfffe13d732a1f6
35:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=230361 operations_accepted=230360 live_at_stop=22452 live_over_31455=0.7138 containers_peak=135 publishes_at_135=961 containers_final=135 fill_smallest=0.0043 fill_final=0.7138 merges=429 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=d71ba0162292de7d49f990dd26d0a8a0ca1a8264ebf95e4f3c6f7f38339e3711
36:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=226987 operations_accepted=226986 live_at_stop=22116 live_over_31455=0.7031 containers_peak=135 publishes_at_135=774 containers_final=135 fill_smallest=0.0043 fill_final=0.7031 merges=427 borrows=1020 i94_third_false_peak=45 i94_third_false_positive_publishes=224603 i94_third_false_final=41 consumed_prefix_sha256=068dd9dcc4c0fcf20ceeeba91e5e38383cc4e6cb4a34b17db8b8d9e8c6bfb60b
37:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=230361 operations_accepted=230360 live_at_stop=22452 live_over_31455=0.7138 containers_peak=135 publishes_at_135=961 containers_final=135 fill_smallest=0.0043 fill_final=0.7138 merges=429 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=d71ba0162292de7d49f990dd26d0a8a0ca1a8264ebf95e4f3c6f7f38339e3711
67:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=83764 operations_accepted=83763 live_at_stop=8177 live_over_31455=0.2600 containers_peak=135 publishes_at_135=414 containers_final=135 fill_smallest=0.0043 fill_final=0.2600 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=5afa3f6043a5318a7b34d32dfb6f6026d7cef5496eb8025df9220a2318f62fb7
68:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=231163 operations_accepted=231162 live_at_stop=22422 live_over_31455=0.7128 containers_peak=135 publishes_at_135=869 containers_final=135 fill_smallest=0.0043 fill_final=0.7128 merges=422 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=583feaff08554eee9716e557803ce97d7cf45dcb040d06b8c32d311c5bbd29c3
69:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=226290 operations_accepted=226289 live_at_stop=21909 live_over_31455=0.6965 containers_peak=135 publishes_at_135=609 containers_final=135 fill_smallest=0.0043 fill_final=0.6965 merges=430 borrows=1029 i94_third_false_peak=44 i94_third_false_positive_publishes=221873 i94_third_false_final=40 consumed_prefix_sha256=f05ae2f3871ea2821bd23aa97983f8503deb14b6ba96a2e97415ba0a6785dc09
70:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=227128 operations_accepted=227127 live_at_stop=21995 live_over_31455=0.6993 containers_peak=135 publishes_at_135=418 containers_final=135 fill_smallest=0.0043 fill_final=0.6993 merges=434 borrows=1 i94_third_false_peak=1 i94_third_false_positive_publishes=481 i94_third_false_final=0 consumed_prefix_sha256=828525d844500cf89c5b0d5c924022bad7ae3605871882afed2fd3bff2a23f01
100:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=81145 operations_accepted=81144 live_at_stop=7978 live_over_31455=0.2536 containers_peak=135 publishes_at_135=860 containers_final=135 fill_smallest=0.0043 fill_final=0.2536 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=fa40f6fa0318543033abd9c7fb1815326ad7f6e44f60abb0f39d5b89e6e4c056
101:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=226565 operations_accepted=226564 live_at_stop=22190 live_over_31455=0.7055 containers_peak=135 publishes_at_135=517 containers_final=135 fill_smallest=0.0043 fill_final=0.7055 merges=406 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=761e85f85e52f879b3ddfac416f02a96e4f142cbea8fb99e92ae78fcc6aed648
102:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=223438 operations_accepted=223437 live_at_stop=21867 live_over_31455=0.6952 containers_peak=135 publishes_at_135=473 containers_final=135 fill_smallest=0.0043 fill_final=0.6952 merges=406 borrows=950 i94_third_false_peak=39 i94_third_false_positive_publishes=219845 i94_third_false_final=36 consumed_prefix_sha256=79d49208dbd51d8c5a97528d345a18f1dc75948794454ee27c3ffb98820663bf
103:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=226565 operations_accepted=226564 live_at_stop=22190 live_over_31455=0.7055 containers_peak=135 publishes_at_135=517 containers_final=135 fill_smallest=0.0043 fill_final=0.7055 merges=406 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=761e85f85e52f879b3ddfac416f02a96e4f142cbea8fb99e92ae78fcc6aed648
133:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=83082 operations_accepted=83081 live_at_stop=7933 live_over_31455=0.2522 containers_peak=135 publishes_at_135=672 containers_final=135 fill_smallest=0.0043 fill_final=0.2522 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=b1cb5e13c8eb2872837027dd4e6efced911dbd51ee052f0db59e4d2af111a6c3
134:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=229232 operations_accepted=229231 live_at_stop=22823 live_over_31455=0.7256 containers_peak=135 publishes_at_135=936 containers_final=135 fill_smallest=0.0043 fill_final=0.7256 merges=408 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=799af22c99b01dad0a9069d884d81e6fff284b13b3e22814ccb768912e660b65
135:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=220027 operations_accepted=220026 live_at_stop=21820 live_over_31455=0.6937 containers_peak=135 publishes_at_135=453 containers_final=135 fill_smallest=0.0043 fill_final=0.6937 merges=395 borrows=981 i94_third_false_peak=43 i94_third_false_positive_publishes=215205 i94_third_false_final=42 consumed_prefix_sha256=b07328935fa974782873a50ce2936b459454b7acc1da713b58cfa434a8af04ea
136:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=228432 operations_accepted=228431 live_at_stop=22771 live_over_31455=0.7239 containers_peak=135 publishes_at_135=402 containers_final=135 fill_smallest=0.0043 fill_final=0.7239 merges=408 borrows=1 i94_third_false_peak=1 i94_third_false_positive_publishes=347 i94_third_false_final=0 consumed_prefix_sha256=d90c6c10aeb892b21485fa66808ae4f65c172cc475802204e88d7e881205c0cb
166:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=82642 operations_accepted=82641 live_at_stop=8083 live_over_31455=0.2570 containers_peak=135 publishes_at_135=411 containers_final=135 fill_smallest=0.0043 fill_final=0.2570 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=0a565b850da4ed7e69f6217e4518839373fad02f0a377d28380a93eed1138447
167:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=224797 operations_accepted=224796 live_at_stop=22396 live_over_31455=0.7120 containers_peak=135 publishes_at_135=408 containers_final=135 fill_smallest=0.0043 fill_final=0.7120 merges=407 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=16eb9a1342ae8c1dada946306ff2bd89c0746725dbffec13a982ac78576fcf56
168:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=223717 operations_accepted=223716 live_at_stop=22260 live_over_31455=0.7077 containers_peak=135 publishes_at_135=696 containers_final=135 fill_smallest=0.0043 fill_final=0.7077 merges=399 borrows=1001 i94_third_false_peak=42 i94_third_false_positive_publishes=220737 i94_third_false_final=38 consumed_prefix_sha256=dd7c20e7d9320424e738ef15abe21786782c370a3fee1dce44f36e49d6b01301
169:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=220315 operations_accepted=220314 live_at_stop=21912 live_over_31455=0.6966 containers_peak=135 publishes_at_135=411 containers_final=135 fill_smallest=0.0043 fill_final=0.6966 merges=388 borrows=1 i94_third_false_peak=1 i94_third_false_positive_publishes=574 i94_third_false_final=0 consumed_prefix_sha256=beba41c2b4e574d2fb2cb0b69d3bec5bbe75bb30d0a377f771f132484466afe1
529:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M1 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
530:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M2 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
531:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M3 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=130 borrows=913 i94_third_false_peak=1 i94_third_false_positive_publishes=44919 i94_third_false_final=1 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
532:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=130 borrows=131 i94_third_false_peak=1 i94_third_false_positive_publishes=22321 i94_third_false_final=1 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
628:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M1 stop=refused refused_operation=50133 operations_accepted=50132 live_at_stop=17132 live_over_31455=0.5447 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.5447 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=254c551cd054829735b28c412d02a379d6a0645758e962936459248375911f66
629:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M2 stop=refused refused_operation=80055 operations_accepted=80054 live_at_stop=27054 live_over_31455=0.8601 containers_peak=135 publishes_at_135=528 containers_final=135 fill_smallest=0.0043 fill_final=0.8601 merges=106 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=0146ed25b25641ce21fcb425ea93c20cf1918917204687cd3a9548be1fdfaf20
630:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M3 stop=refused refused_operation=74465 operations_accepted=74464 live_at_stop=25464 live_over_31455=0.8095 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.8095 merges=81 borrows=36 i94_third_false_peak=19 i94_third_false_positive_publishes=71807 i94_third_false_final=18 consumed_prefix_sha256=2e2bb7b45ee7a2dd79d0d34d754b9252faa3222ad1addb3a58b0f9d714d352e9
631:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=80055 operations_accepted=80054 live_at_stop=27054 live_over_31455=0.8601 containers_peak=135 publishes_at_135=528 containers_final=135 fill_smallest=0.0043 fill_final=0.8601 merges=106 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=0146ed25b25641ce21fcb425ea93c20cf1918917204687cd3a9548be1fdfaf20
```

`grep 'name=arm_run' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out | grep -o 'fill_smallest=[0-9.]*' | sort | uniq -c` 输出 `88 fill_smallest=0.0043`。

累计重写差 S(t) = Σ(臂 − M1)，t 走主窗口里的最后一批（主取样点，整行抄）。W-R p = 0.55 上位置记法的期末值是负的（M2、M3 比 M1 少重写），峰值是正的：S 先涨到正值，之后被 M1 摘片那几次的大重写（单次可到 133 片）拉成负数；只报期末值会漏掉前面那一段，摘片落在窗口哪一段没有逐步记。

```
42:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M2-M1 notation=position peak=122 positive_single_differences=127 final=-921
43:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M2-M1 notation=content peak=41 positive_single_differences=41 final=41
44:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M3-M1 notation=position peak=297 positive_single_differences=476 final=-557
45:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M3-M1 notation=content peak=392 positive_single_differences=392 final=392
75:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M2-M1 notation=position peak=192 positive_single_differences=135 final=-1486
76:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M2-M1 notation=content peak=49 positive_single_differences=49 final=49
77:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M3-M1 notation=position peak=450 positive_single_differences=514 final=-1138
78:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M3-M1 notation=content peak=433 positive_single_differences=433 final=433
108:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M2-M1 notation=position peak=56 positive_single_differences=133 final=-842
109:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M2-M1 notation=content peak=45 positive_single_differences=45 final=45
110:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M3-M1 notation=position peak=163 positive_single_differences=492 final=-481
111:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M3-M1 notation=content peak=407 positive_single_differences=407 final=407
141:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M2-M1 notation=position peak=25 positive_single_differences=135 final=-648
142:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M2-M1 notation=content peak=48 positive_single_differences=48 final=48
143:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M3-M1 notation=position peak=168 positive_single_differences=527 final=-284
144:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M3-M1 notation=content peak=442 positive_single_differences=442 final=442
174:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M2-M1 notation=position peak=39 positive_single_differences=136 final=-816
175:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M2-M1 notation=content peak=47 positive_single_differences=47 final=47
176:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M3-M1 notation=position peak=198 positive_single_differences=486 final=-477
177:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M3-M1 notation=content peak=402 positive_single_differences=401 final=402
537:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M2-M1 notation=position peak=0 positive_single_differences=0 final=0
538:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M2-M1 notation=content peak=0 positive_single_differences=0 final=0
539:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M3-M1 notation=position peak=913 positive_single_differences=913 final=913
540:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M3-M1 notation=content peak=1043 positive_single_differences=1043 final=1043
636:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M2-M1 notation=position peak=83 positive_single_differences=33 final=83
637:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M2-M1 notation=content peak=0 positive_single_differences=0 final=0
638:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M3-M1 notation=position peak=90 positive_single_differences=59 final=90
639:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M3-M1 notation=content peak=26 positive_single_differences=26 final=26
```

### 几何 / 旋钮敏感性（第八节）

每个旋钮的主取样点（或参照点）与方向相反的点在七行上各比一次：判定相同记 `two_points_same`，不同记 `unstable`。负载旋钮按点配对（W-R 按种子配，0.75 那一点也配了一次）；留 1/233 只在 A = 2330 上有，所以拿 W-A 留 1/2、A = 2330 当参照；f 旋钮在 22 条序列上各比主窗口与 f = 59 窗口里涉及 M3 的四行；记法旋钮在 7 个主取样点上比 Q2p 与 Q2c、Q3p 与 Q3c。判别力自证：两点同判时把一个门槛挪到两点的值之间，判定必须转成不同。按旋钮数（命令在仓根跑，原样输出；第三列是判定比较，第四列是挪门槛之后，`-` 表示两点本来就不同、不做自证）：

```
awk '/name=sensitivity/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} key=v["knob"] " " v["result"] " " (v["moved_result"]==""?"-":v["moved_result"]); count[key]++} END{for(k in count) print count[k], k}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out | sort -k2,2 -k3,3
```

```
7 batch_size_2330 two_points_same unstable
32 borrow_floor_59 two_points_same equal_values_cannot_be_separated
56 borrow_floor_59 two_points_same unstable
2 deleted_fraction_0.95 two_points_same equal_values_cannot_be_separated
5 deleted_fraction_0.95 two_points_same unstable
3 deletion_order_largest_first two_points_same equal_values_cannot_be_separated
4 deletion_order_largest_first two_points_same unstable
28 insert_probability_0.75 two_points_same unstable
7 insert_probability_0.75 unstable -
30 insert_probability_0.95 two_points_same unstable
5 insert_probability_0.95 unstable -
6 kept_fraction_1of233 two_points_same unstable
1 kept_fraction_1of233 unstable -
6 kept_fraction_3of4 two_points_same unstable
1 kept_fraction_3of4 unstable -
1 rewrite_notation_content two_points_same equal_values_cannot_be_separated
20 rewrite_notation_content two_points_same unstable
```

两点同判的 200 对里 162 对挪门槛之后转成不同；挪门槛自证做不了的对数是 38（两点的值逐位相等、门槛挪不进两点之间，第 990 行 `equal_values_not_separable=38`；登记第八节没写这种情形怎么报，单列，不并进别的计数）：f 旋钮 32 对（8 条序列 × 4 行：W-R p = 0.95 五个种子、W-S 降序、W-A 留 1/2 A = 2330、W-A 留 3/4 上 M3 与 M3 f = 59 都一次没借，轨迹行 `borrows=0`）、W-S 两个旋钮 5 对与记法旋钮 1 对（两点都是 0.0000 或 1.0000）。这 38 对只能写「两点值相同」，判别力没有自证。

判定不同的 14 对（整行抄）：

```
776:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed1 main_value=0.3594 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed1 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
783:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed2 main_value=0.3647 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed2 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
790:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed3 main_value=0.3595 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed3 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
797:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed4 main_value=0.3476 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed4 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
804:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed5 main_value=0.3609 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed5 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
811:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed1 main_value=0.3594 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed1 opposite_value=0.8838 opposite_verdict=no_flip result=unstable
818:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed2 main_value=0.3647 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed2 opposite_value=0.8668 opposite_verdict=no_flip result=unstable
819:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q2p_delta2 main=wr_p055_seed2 main_value=-0.0393 main_verdict=negligible opposite=wr_p075_seed2 opposite_value=0.1046 opposite_verdict=no_flip result=unstable
820:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q2p_delta3 main=wr_p055_seed2 main_value=-0.0301 main_verdict=negligible opposite=wr_p075_seed2 opposite_value=0.1100 opposite_verdict=no_flip result=unstable
825:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed3 main_value=0.3595 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed3 opposite_value=0.8825 opposite_verdict=no_flip result=unstable
832:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed4 main_value=0.3476 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed4 opposite_value=0.8640 opposite_verdict=no_flip result=unstable
839:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed5 main_value=0.3609 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed5 opposite_value=0.8698 opposite_verdict=no_flip result=unstable
860:E7RESULT name=sensitivity knob=kept_fraction_3of4 row=Q1r main=wa_keep1of2_a1000 main_value=0.6333 main_verdict=no_flip opposite=wa_keep3of4_a1000 opposite_value=0.9658 opposite_verdict=no_merge_capacity_cost_small result=unstable
867:E7RESULT name=sensitivity knob=kept_fraction_1of233 row=Q1r main=wa_keep1of2_a2330 main_value=0.5332 main_verdict=no_flip opposite=wa_keep1of233_a2330 opposite_value=0.0711 opposite_verdict=merge_buys_double_capacity result=unstable
```

Q1r 对插入概率（0.55 对 0.95、0.55 对 0.75 两个方向五个种子都翻）与留下比例（1/2 对 3/4、1/2 对 1/233 两个方向）都不稳定；Q2p 两行只在插入概率 0.55 对 0.75 的种子 2 上不稳定。Q2c、Q3 两种记法在每个旋钮上都两点同判。

### 第二段：锚点与阳性对照

第二段产物第 2–32 行与第一段第 2–32 行逐字相同（`diff <(sed -n 2,32p research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out) <(sed -n 2,32p research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out)` 没有输出）。第 33–48 行是三个变体新加的 16 格（修订 3：A1 按登记答案判，其余只报不判），整行抄：

```
E7RESULT name=anchor cell=A1 arm=M3_just_enough refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M2_left_first refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M3_left_first refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A2 arm=M3_just_enough refused_insert=62776 live=31455 containers=135 merges=134 borrows=15197 matches=unregistered
E7RESULT name=anchor cell=A2 arm=M2_left_first refused_insert=62776 live=31455 containers=135 merges=134 borrows=0 matches=unregistered
E7RESULT name=anchor cell=A2 arm=M3_left_first refused_insert=62776 live=31455 containers=135 merges=134 borrows=918 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M2_left_first content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M3_left_first content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=174/176 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M3_just_enough content=349 position=351 containers=3 merges=1 borrows=116 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M2_left_first content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M3_left_first content=240 position=242 containers=3 merges=1 borrows=7 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A5 arm=M3_just_enough content=350 position=352 containers=4 merges=1 borrows=116 third_sentence_false=0 sizes=233/232/233/1 matches=unregistered
E7RESULT name=anchor cell=A5 arm=M3_left_first content=241 position=243 containers=4 merges=1 borrows=7 third_sentence_false=0 sizes=233/232/233/1 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M3_just_enough content=349 position=349 containers=1 merges=1 borrows=116 third_sentence_false=0 sizes=233 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M2_left_first content=233 position=233 containers=1 merges=1 borrows=0 third_sentence_false=0 sizes=233 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M3_left_first content=240 position=240 containers=1 merges=1 borrows=7 third_sentence_false=0 sizes=233 matches=unregistered
```

先查左边在登记的手造格上与主旋钮的读数逐格相同（A2 … A6，M2、M3 各自对比），这几格分不出这个旋钮；分得出它的是单测里那组三片 117 / 117 / 117（修订 3 第 5 行，变异 V14 让它红）。借到刚好够下限在 A4 … A6 上借 116 次、主旋钮借 7 次（推的，没逐步看：每次只借到 117 条，下一次删就又低于下限、再借）。

### 第二段读数：Q2、Q3（三个窗口）

主窗口那一组数与「读数：Q2、Q3」一节第一段的表逐字相同（第二段产物里主窗口的 396 行与第一段同名行逐字相同）。另两个窗口，命令（仓根，`W` 取 `just_enough` 或 `left_first`）与原样输出：

```
awk -v W=just_enough '/name=point_verdict/ && $0 ~ (" window=" W " ") {delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} out[v["sequence"]]=out[v["sequence"]] " " v["row"] "=" v["value"]; if(!(v["sequence"] in seen)){seen[v["sequence"]]=1; order[++n]=v["sequence"]}} END{for(i=1;i<=n;i++) print order[i] out[order[i]]}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out
```

`just_enough` 窗口（M1、M2、M3_just_enough）：

```
wr_p055_seed1 Q2p_delta2=-0.0260 Q2p_delta3=0.0177 Q2c_delta2=0.0012 Q2c_delta3=0.0454 Q3p_delta32=0.0437 Q3c_delta32=0.0442
wr_p055_seed2 Q2p_delta2=-0.0393 Q2p_delta3=0.0255 Q2c_delta2=0.0013 Q2c_delta3=0.0661 Q3p_delta32=0.0648 Q3c_delta32=0.0648
wr_p055_seed3 Q2p_delta2=-0.0230 Q2p_delta3=0.0381 Q2c_delta2=0.0012 Q2c_delta3=0.0616 Q3p_delta32=0.0611 Q3c_delta32=0.0604
wr_p055_seed4 Q2p_delta2=-0.0172 Q2p_delta3=0.0377 Q2c_delta2=0.0013 Q2c_delta3=0.0567 Q3p_delta32=0.0550 Q3c_delta32=0.0554
wr_p055_seed5 Q2p_delta2=-0.0219 Q2p_delta3=0.0376 Q2c_delta2=0.0013 Q2c_delta3=0.0637 Q3p_delta32=0.0595 Q3c_delta32=0.0624
wr_p075_seed1 Q2p_delta2=0.0881 Q2p_delta3=0.1106 Q2c_delta2=0.0000 Q2c_delta3=0.0192 Q3p_delta32=0.0225 Q3c_delta32=0.0192
wr_p075_seed2 Q2p_delta2=0.1046 Q2p_delta3=0.1282 Q2c_delta2=0.0000 Q2c_delta3=0.0216 Q3p_delta32=0.0236 Q3c_delta32=0.0216
wr_p075_seed3 Q2p_delta2=0.0796 Q2p_delta3=0.0957 Q2c_delta2=0.0000 Q2c_delta3=0.0141 Q3p_delta32=0.0161 Q3c_delta32=0.0141
wr_p075_seed4 Q2p_delta2=0.0985 Q2p_delta3=0.1096 Q2c_delta2=0.0000 Q2c_delta3=0.0186 Q3p_delta32=0.0111 Q3c_delta32=0.0186
wr_p075_seed5 Q2p_delta2=0.0935 Q2p_delta3=0.1068 Q2c_delta2=0.0000 Q2c_delta3=0.0266 Q3p_delta32=0.0133 Q3c_delta32=0.0266
wr_p095_seed1 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed2 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed3 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed4 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed5 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
ws_a1000_d500_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.4975 Q2c_delta2=0.0000 Q2c_delta3=0.5018 Q3p_delta32=0.4975 Q3c_delta32=0.5018
ws_a1000_d950_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.4975 Q2c_delta2=0.0000 Q2c_delta3=0.5018 Q3p_delta32=0.4975 Q3c_delta32=0.5018
ws_a1000_d500_largest_first Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of2_a1000 Q2p_delta2=0.0050 Q2p_delta3=0.0053 Q2c_delta2=0.0000 Q2c_delta3=0.0002 Q3p_delta32=0.0002 Q3c_delta32=0.0002
wa_keep1of2_a2330 Q2p_delta2=0.0172 Q2p_delta3=0.0172 Q2c_delta2=0.0001 Q2c_delta3=0.0001 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of233_a2330 Q2p_delta2=0.0191 Q2p_delta3=0.4749 Q2c_delta2=0.0004 Q2c_delta3=0.4568 Q3p_delta32=0.4558 Q3c_delta32=0.4564
wa_keep3of4_a1000 Q2p_delta2=0.0003 Q2p_delta3=0.0003 Q2c_delta2=0.0003 Q2c_delta3=0.0003 Q3p_delta32=0.0000 Q3c_delta32=0.0000
```

`left_first` 窗口（M1、M2_left_first、M3_left_first）：

```
wr_p055_seed1 Q2p_delta2=-0.0248 Q2p_delta3=-0.0147 Q2c_delta2=0.0012 Q2c_delta3=0.0113 Q3p_delta32=0.0101 Q3c_delta32=0.0101
wr_p055_seed2 Q2p_delta2=-0.0393 Q2p_delta3=-0.0288 Q2c_delta2=0.0013 Q2c_delta3=0.0108 Q3p_delta32=0.0105 Q3c_delta32=0.0096
wr_p055_seed3 Q2p_delta2=-0.0230 Q2p_delta3=-0.0128 Q2c_delta2=0.0012 Q2c_delta3=0.0108 Q3p_delta32=0.0102 Q3c_delta32=0.0096
wr_p055_seed4 Q2p_delta2=-0.0172 Q2p_delta3=-0.0091 Q2c_delta2=0.0013 Q2c_delta3=0.0116 Q3p_delta32=0.0081 Q3c_delta32=0.0103
wr_p055_seed5 Q2p_delta2=-0.0233 Q2p_delta3=-0.0134 Q2c_delta2=0.0013 Q2c_delta3=0.0106 Q3p_delta32=0.0100 Q3c_delta32=0.0093
wr_p075_seed1 Q2p_delta2=0.0881 Q2p_delta3=0.0864 Q2c_delta2=0.0000 Q2c_delta3=0.0045 Q3p_delta32=-0.0017 Q3c_delta32=0.0045
wr_p075_seed2 Q2p_delta2=0.1046 Q2p_delta3=0.1111 Q2c_delta2=0.0000 Q2c_delta3=0.0042 Q3p_delta32=0.0066 Q3c_delta32=0.0042
wr_p075_seed3 Q2p_delta2=0.0796 Q2p_delta3=0.0822 Q2c_delta2=0.0000 Q2c_delta3=0.0040 Q3p_delta32=0.0026 Q3c_delta32=0.0040
wr_p075_seed4 Q2p_delta2=0.0985 Q2p_delta3=0.0875 Q2c_delta2=0.0000 Q2c_delta3=0.0041 Q3p_delta32=-0.0110 Q3c_delta32=0.0041
wr_p075_seed5 Q2p_delta2=0.0935 Q2p_delta3=0.0936 Q2c_delta2=0.0000 Q2c_delta3=0.0046 Q3p_delta32=0.0001 Q3c_delta32=0.0046
wr_p095_seed1 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed2 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed3 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed4 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed5 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
ws_a1000_d500_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0299 Q2c_delta2=0.0000 Q2c_delta3=0.0342 Q3p_delta32=0.0299 Q3c_delta32=0.0342
ws_a1000_d950_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0300 Q2c_delta2=0.0000 Q2c_delta3=0.0343 Q3p_delta32=0.0300 Q3c_delta32=0.0343
ws_a1000_d500_largest_first Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of2_a1000 Q2p_delta2=0.0050 Q2p_delta3=0.0055 Q2c_delta2=0.0000 Q2c_delta3=0.0016 Q3p_delta32=0.0004 Q3c_delta32=0.0016
wa_keep1of2_a2330 Q2p_delta2=0.0172 Q2p_delta3=0.0172 Q2c_delta2=0.0001 Q2c_delta3=0.0001 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of233_a2330 Q2p_delta2=0.0191 Q2p_delta3=0.0457 Q2c_delta2=0.0004 Q2c_delta3=0.0275 Q3p_delta32=0.0265 Q3c_delta32=0.0271
wa_keep3of4_a1000 Q2p_delta2=0.0003 Q2p_delta3=0.0003 Q2c_delta2=0.0003 Q2c_delta3=0.0003 Q3p_delta32=0.0000 Q3c_delta32=0.0000
```

两种记法、两个旋钮的结论各写一句，不合成一句：

- **先查左边、位置记法**：Δ2p、Δ3p 仍判「不翻」，W-R p = 0.75 种子 2 上 0.1046、0.1111 过 0.10。主取样点 W-R p = 0.55 上仍是负的（−0.0393 … −0.0091）。这个旋钮让两行的值动得最多的一处是 0.014（p = 0.75 种子 1 的 Δ3p 0.0724 → 0.0864），判定不变（数法：对每条序列、Δ2p 与 Δ3p 两行取 `left_first` 窗口减主窗口的绝对值，取最大，`awk` 读 `name=point_verdict` 行，输出 `max_abs_change 0.014 wr_p075_seed1 Q2p_delta3 0.0724 -> 0.0864`）。
- **先查左边、内容记法与 Q3**：四行仍「可忽略」，最大 0.0343（W-S 升序删 950 那一点，与主窗口相同）。
- **借到刚好够下限、位置记法**：Δ3p 仍「不翻」，但过 0.10 的点从一个变四个（W-R p = 0.75 种子 1、2、4、5：0.1106、0.1282、0.1096、0.1068），W-S 两个升序删点到 0.4975，W-A 留 1/233 到 0.4749；主取样点 W-R p = 0.55 上 0.0177 … 0.0381。没有一点到 1.00。
- **借到刚好够下限、内容记法与 Q3**：Δ3c 与 Q3 两行由「可忽略」变「不翻」。不利于「借的写代价可忽略」的那一半：W-S 主取样点（升序删 500）上 Δ3c 0.5018、Q3 两行 0.4975 / 0.5018，W-A 留 1/233 上 0.4558 … 0.4568。没有一点到 1.00，「显著」一侧没有打中。
- Δ2p、Δ2c 两行在 `just_enough` 窗口里与主窗口逐个相同（同一对臂 M1、M2）。

轨迹与借的次数（每条臂被拒时的活数；三个 M3 变体借的次数；两个 M2 变体并的次数），命令（仓根）与原样输出：

```
awk '/name=arm_run/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} s=v["sequence"]; if(!(s in seen)){seen[s]=1; order[++n]=s} live[s,v["arm"]]=v["live_at_stop"]; br[s,v["arm"]]=v["borrows"]; mg[s,v["arm"]]=v["merges"]} END{print "sequence M1 M2 M3 M3_just_enough M2_left_first M3_left_first | borrows M3 M3_just_enough M3_left_first | merges M2 M2_left_first"; for(i=1;i<=n;i++){s=order[i]; print s, live[s,"M1"], live[s,"M2"], live[s,"M3"], live[s,"M3_just_enough"], live[s,"M2_left_first"], live[s,"M3_left_first"], "|", br[s,"M3"], br[s,"M3_just_enough"], br[s,"M3_left_first"], "|", mg[s,"M2"], mg[s,"M2_left_first"]}}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out
```

```
sequence M1 M2 M3 M3_just_enough M2_left_first M3_left_first | borrows M3 M3_just_enough M3_left_first | merges M2 M2_left_first
wr_p055_seed1 8070 22452 22116 22391 22416 22163 | 1020 6212 1024 | 429 428
wr_p055_seed2 8177 22422 21909 22306 22493 21909 | 1029 6774 989 | 422 424
wr_p055_seed3 7978 22190 21867 22277 22293 21757 | 950 6298 931 | 406 409
wr_p055_seed4 7933 22823 21820 22317 22823 22308 | 981 6273 1012 | 408 408
wr_p055_seed5 8083 22396 22260 22096 22396 21704 | 1001 6615 944 | 407 407
wr_p075_seed1 21360 24167 23551 24167 24167 24011 | 53 222 52 | 20 20
wr_p075_seed2 21173 24427 24264 24427 24427 24264 | 42 245 51 | 22 22
wr_p075_seed3 21378 24225 24225 24388 24225 24225 | 50 172 50 | 22 22
wr_p075_seed4 20994 24299 23980 24299 24299 23980 | 56 249 51 | 23 23
wr_p075_seed5 21157 24325 24163 24325 24325 24163 | 53 322 53 | 23 23
wr_p095_seed1 29865 29865 29865 29865 29865 29865 | 0 0 0 | 0 0
wr_p095_seed2 29810 29811 29811 29811 29811 29811 | 0 0 0 | 1 1
wr_p095_seed3 29802 29803 29803 29803 29803 29803 | 0 0 0 | 1 1
wr_p095_seed4 29815 29815 29815 29815 29815 29815 | 0 0 0 | 0 0
wr_p095_seed5 29821 29821 29821 29821 29821 29821 | 0 0 0 | 0 0
ws_a1000_d500_smallest_first 31245 31245 31245 31245 31245 31245 | 913 15174 913 | 0 0
ws_a1000_d950_smallest_first 31319 31319 31284 31275 31319 31284 | 17277 286396 17277 | 4 4
ws_a1000_d500_largest_first 31455 31455 31455 31455 31455 31455 | 0 0 0 | 0 0
wa_keep1of2_a1000 17132 27054 25464 27054 27054 25464 | 36 4 36 | 106 106
wa_keep1of2_a2330 16773 31455 31455 31455 31455 31455 | 0 0 0 | 130 130
wa_keep1of233_a2330 2236 31455 31081 31081 31455 31081 | 179568 2951927 179568 | 29004 29004
wa_keep3of4_a1000 24606 25478 25478 25478 25478 25478 | 0 0 0 | 33 33
```

132 条臂都被拒（`grep name=arm_run research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out | grep -c 'stop=refused'` 输出 132）。借到刚好够下限在 W-S 主取样点上借 15 174 次、主旋钮 913 次；W-S 主取样点窗口里它的内容记法直方图是 0 片 0 次、1 片 15 326 次、2 片 15 174 次（第 809 行，整行抄）：

```
809:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=just_enough arm=M3_just_enough unlinks=30500 content_sum=45674 position_sum=54403 content_mean=1.4975 position_mean=1.7837 content_largest=2 position_largest=133 content_histogram=0/15326/15174/0/0 position_histogram=0/15196/15174/6/124
```

M2_left_first 被拒时的活数只是读数，Q1r 不在这个旋钮上判（登记第六节）：它在 W-R p = 0.55 上 22 293 … 22 823，与 M2 主（22 190 … 22 823）同一个量级。

### 第二段：两个旋钮的敏感性

按旋钮数（仓根，原样输出；第三列是判定比较，第四列是挪门槛之后，`-` 表示两点本来就不同、不做自证）：

```
awk '/name=sensitivity/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} key=v["knob"] " " v["result"] " " (v["moved_result"]==""?"-":v["moved_result"]); count[key]++} END{for(k in count) print count[k], k}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out | sort -k2,2 -k3,3
```

```
32 borrow_amount_just_enough two_points_same equal_values_cannot_be_separated
41 borrow_amount_just_enough two_points_same unstable
15 borrow_amount_just_enough unstable -
33 check_order_left_first two_points_same unstable
99 check_order_left_first two_points_same equal_values_cannot_be_separated
```

两点同判的 205 对里 74 对挪门槛之后转成不同；挪门槛自证做不了的对数是 131（两点值逐位相等；第 1369 行 `equal_values_not_separable=131`，单列，不并进别的计数）。按序列数（`awk` 读 `moved_result=equal_values_cannot_be_separated` 的敏感性行，按旋钮与主点序列名计数）：「借多少」32 对 = 8 条序列 × 4 行，正是 M3 与 M3_just_enough 都一次没借的 8 条（W-R p = 0.95 五个种子、W-S 降序、W-A 留 1/2 A = 2330、W-A 留 3/4，借的次数见「第二段读数：Q2、Q3（三个窗口）」一节轨迹与借的次数那张表）；「先查哪一对」99 对 = 14 条序列六行全等（W-S 三点、W-A 四点、W-R p = 0.95 五个种子、p = 0.75 种子 3 与 5）共 84 对，加 W-R p = 0.55 五个种子与 p = 0.75 种子 1、2、4 上部分行相等共 15 对。这 131 对只能写「两点值相同」，判别力没有自证。判定不同的 15 对全在「借多少」旋钮上（第 1169、1181、1185、1209–1216、1229–1232 行；`grep -n 'name=sensitivity.* result=unstable' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out` 列出，整行抄四行为例）：

```
1169:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2p_delta3 main=wr_p075_seed1@M3 main_value=0.0724 main_verdict=negligible opposite=wr_p075_seed1@M3_just_enough opposite_value=0.1106 opposite_verdict=no_flip result=unstable
1209:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2p_delta3 main=ws_a1000_d500_smallest_first@M3 main_value=0.0299 main_verdict=negligible opposite=ws_a1000_d500_smallest_first@M3_just_enough opposite_value=0.4975 opposite_verdict=no_flip result=unstable
1210:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2c_delta3 main=ws_a1000_d500_smallest_first@M3 main_value=0.0342 main_verdict=negligible opposite=ws_a1000_d500_smallest_first@M3_just_enough opposite_value=0.5018 opposite_verdict=no_flip result=unstable
1230:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2c_delta3 main=wa_keep1of233_a2330@M3 main_value=0.0275 main_verdict=negligible opposite=wa_keep1of233_a2330@M3_just_enough opposite_value=0.4568 opposite_verdict=no_flip result=unstable
```

Q2p、Q2c、Q3 对「借多少」不稳定（涉及 M3 的四行，在 W-R p = 0.75、W-S 升序删、W-A 留 1/233 上翻）；对「先查哪一对」两点同判。

### 对拍（停机条款，整行抄）

`research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle.out` 全文：

```
1:E7RESULT name=oracle_constants model_leaf_record_capacity=233 crates_inode_leaf_records=233 model_root_node_leaf_container_capacity=135 crates_leaf_containers_one_root_node_holds=135 equal=true
2:E7RESULT name=oracle_crates_unit_test_shape model=sizes=233/233/34,container_numbers=1/234/467 crates=sizes=233/233/34,container_numbers=1/234/467 equal=true
3:E7RESULT name=oracle_crates_unit_test_shape model=refused_inode=31456,refused_containers=136,capacity=135 crates=refused_inode=31456,refused_containers=136,capacity=135 equal=true
4:E7RESULT name=oracle_crates_unit_test_shape model=position_rewritten=[1],dropped=1 crates=position_rewritten=[1],dropped=1 equal=true
5:E7RESULT name=oracle sequence=wr_p055_seed1 operations_compared=78967 unlinks_compared=35448 m1_refused_operation=78967 first_difference=none
6:E7RESULT name=oracle sequence=wr_p055_seed2 operations_compared=83764 unlinks_compared=37793 m1_refused_operation=83764 first_difference=none
7:E7RESULT name=oracle sequence=wr_p055_seed3 operations_compared=81145 unlinks_compared=36583 m1_refused_operation=81145 first_difference=none
8:E7RESULT name=oracle sequence=wr_p055_seed4 operations_compared=83082 unlinks_compared=37574 m1_refused_operation=83082 first_difference=none
9:E7RESULT name=oracle sequence=wr_p055_seed5 operations_compared=82642 unlinks_compared=37279 m1_refused_operation=82642 first_difference=none
10:E7RESULT name=oracle sequence=wr_p075_seed1 operations_compared=42169 unlinks_compared=10404 m1_refused_operation=42169 first_difference=none
11:E7RESULT name=oracle sequence=wr_p075_seed2 operations_compared=42502 unlinks_compared=10664 m1_refused_operation=42502 first_difference=none
12:E7RESULT name=oracle sequence=wr_p075_seed3 operations_compared=42241 unlinks_compared=10431 m1_refused_operation=42241 first_difference=none
13:E7RESULT name=oracle sequence=wr_p075_seed4 operations_compared=42701 unlinks_compared=10853 m1_refused_operation=42701 first_difference=none
14:E7RESULT name=oracle sequence=wr_p075_seed5 operations_compared=42524 unlinks_compared=10683 m1_refused_operation=42524 first_difference=none
15:E7RESULT name=oracle sequence=wr_p095_seed1 operations_compared=33152 unlinks_compared=1643 m1_refused_operation=33152 first_difference=none
16:E7RESULT name=oracle sequence=wr_p095_seed2 operations_compared=33173 unlinks_compared=1681 m1_refused_operation=33173 first_difference=none
17:E7RESULT name=oracle sequence=wr_p095_seed3 operations_compared=33191 unlinks_compared=1694 m1_refused_operation=33191 first_difference=none
18:E7RESULT name=oracle sequence=wr_p095_seed4 operations_compared=33180 unlinks_compared=1682 m1_refused_operation=33180 first_difference=none
19:E7RESULT name=oracle sequence=wr_p095_seed5 operations_compared=33190 unlinks_compared=1684 m1_refused_operation=33190 first_difference=none
20:E7RESULT name=oracle sequence=ws_a1000_d500_smallest_first operations_compared=92246 unlinks_compared=30500 m1_refused_operation=92246 first_difference=none
21:E7RESULT name=oracle sequence=ws_a1000_d950_smallest_first operations_compared=1184620 unlinks_compared=576650 m1_refused_operation=1184620 first_difference=none
22:E7RESULT name=oracle sequence=ws_a1000_d500_largest_first operations_compared=92456 unlinks_compared=30500 m1_refused_operation=92456 first_difference=none
23:E7RESULT name=oracle sequence=wa_keep1of2_a1000 operations_compared=50133 unlinks_compared=16500 m1_refused_operation=50133 first_difference=none
24:E7RESULT name=oracle sequence=wa_keep1of2_a2330 operations_compared=47064 unlinks_compared=15145 m1_refused_operation=47064 first_difference=none
25:E7RESULT name=oracle sequence=wa_keep1of233_a2330 operations_compared=67197 unlinks_compared=32480 m1_refused_operation=67197 first_difference=none
26:E7RESULT name=oracle sequence=wa_keep3of4_a1000 operations_compared=40607 unlinks_compared=8000 m1_refused_operation=40607 first_difference=none
27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=22 constants_equal=true crates_unit_test_shapes_equal=true all_equal=true
28:E7RESULT name=done emitted=28
```

判别力自证：模型上打开 V1 之后（`research/results/e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1.out`），对拍在 7 条序列上报出第一处不同，W-S 主取样点正是第 1233 个操作、`crates/` 的重写清单长 4、模型 0，与登记第九节末写的相同；`crates/` 单测「删空中间一片」那一格也对不上。另 15 条序列（W-R p = 0.75 / 0.95、W-S 降序、W-A 四点）在 M1 被拒之前没有摘过非最右的一片，位置左移那一支在那几条上没被对拍走到。整行抄：

```
4:E7RESULT name=oracle_crates_unit_test_shape model=position_rewritten=[],dropped=1 crates=position_rewritten=[1],dropped=1 equal=false
20:E7RESULT name=oracle sequence=ws_a1000_d500_smallest_first operations_compared=1233 unlinks_compared=233 m1_refused_operation=none first_difference=operation=1233 Unlink(233) 位置记法重写清单不同：crates [0, 1, 2, 3]（长 4），模型 []（长 0）
27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=15 constants_equal=true crates_unit_test_shapes_equal=false all_equal=false
```

第二段开跑之前，对拍与判别力各用第二段的源码重编重跑一次：`research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle-2026-10-08.out` 与第一段的对拍产物 `cmp` 逐字节相同（判决行 `all_equal=true`），`research/results/e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1-2026-10-08.out` 与第一段的判别力产物 `cmp` 逐字节相同（V1 副本退 1，同样在 W-S 主取样点第 1233 个操作报出不同）。对拍只比 M1，第二段的三个变体不在它的射程里。

### 单测与变异

单测 22 个，今天这份源码（`grep -c '^test .* \.\.\. ok$' research/results/e176-inode-leaf-merge-capacity-and-rewrites-unit-tests-2026-10-08.out` 输出 22；同一份文件里 `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s`）。第一段那 14 个（`research/results/e176-inode-leaf-merge-capacity-and-rewrites-unit-tests.out`）里钉绝对值的有：全部锚点格（26 格登记答案逐字比）、A1 与 A2 的 M1 另写成加法式、SHA-256 三个公开测试向量、四种负载生成器的前几步、丙类断言五种坏形态各自会红、门槛的边界取值、挪门槛自证、按序列分线程与单线程逐字相同、W-R 主取样点上三条臂操作流前缀相同。第二段加的 8 个：第二段的窗口表与三个变体的策略逐个钉死、第二段锚点（登记格 29、总 47）、三片 117 / 117 / 117 上先查右边与先查左边的末态与两种记法（M2、M3 各两条，共四条）、第二段六个臂变体分线程与单线程逐字相同、第二段敏感性行的旋钮与行逐个钉死并核对先查左边那一点取的是 `left_first` 窗口；随机长跑丙类断言那条单测加了 M3_left_first。

变异表 `scripts/mutation/tables/e176_inode_leaf_merge_capacity_and_rewrites.tsv` 16 条（登记第九节 V1–V13，修订 3 补 V14–V16；锚点按今天的装置源码取，每条恰好命中一次）：抓到 16、无效 0、没红 0；`💥`、`⚠️`、`⏱`、`🧱` 各 0；收尾「计数：内存撞顶 0 条（上限 4G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条」与「已还原，基线仍全绿」（`research/results/e176-inode-leaf-merge-capacity-and-rewrites-mutations-2026-10-08.out` 第 19、20 行；第一段那次 13 条全抓在 `research/results/e176-inode-leaf-merge-capacity-and-rewrites-mutations.out`）。V4 改的是序列与锚点格共用的 `live_inodes_at_refusal`；V13 在 W-R 主取样点上让作废断言当场红；V9 由丙类「相邻两片 key 区间不交叠」当场红；V14 由三片 117 / 117 / 117 那两条先查左边的单测抓到，V15 由 A3 借到刚好够下限那一格登记答案抓到，V16 由第二段窗口表那条单测抓到。

16 条全抓只说明写了断言的那几处被变异测过，不说明模型每一行都测过：判定与敏感性的算术（`point_verdict`、`overall_verdict`、`moved_threshold_verdicts`）由单测钉边界，变异表里没有一条动它们；第二段敏感性取哪个窗口（`segment_two_sensitivity_lines` 里按窗口名找下标）由单测核一点，没有变异。

### 岔路表

| 岔路行 | 状态 | 依据（产物里算出它的命令） | 还差什么；剩下的量能不能让它翻面 |
|---|---|---|---|
| M 行翻面观测第一句：M1 撞上 135 片时的活数 | 已够判 | `grep 'name=arm_run' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out` 的 `live_at_stop`；Q1r 那条 `awk`（「读数：Q1」一节）；第一段总判定第 763 行，第二段主窗口第 1130 行判定相同 | 不差。登记的量两段都跑完，没有剩下的量。第二段没动 M1 与 M2 主（第二段主窗口与 M1、M2、M3 三条臂的 396 行与第一段逐字相同），Q1r 判定不变；Q1r 对负载不稳定（敏感性一节）是已量到的性质，不是缺的量 |
| M 行翻面观测第二句：M2、M3 每次 unlink 比 M1 多重写的叶片数 | 已够判。位置记法：Δ2p、Δ3p 在三个窗口都「不翻」。内容记法：Δ2c 三个窗口都「可忽略」；Δ3c 与 Q3 两行在主窗口、`left_first` 窗口「可忽略」，在 `just_enough` 窗口「不翻」 | 第一段总判定第 764–775 行；第二段总判定第 1131–1148 行；「第二段读数：Q2、Q3」一节那条 `awk`（`W` 取 `just_enough`、`left_first`） | 不差。第二段是登记的最后一段，跑完了；没有剩下的量。位置记法在两个旋钮上都没翻到「可忽略」或「显著」（`just_enough` 窗口 Δ3p 最大 0.4975、`left_first` 窗口 Δ3p 最大 0.1111）。「借多少」旋钮让 Δ3c 与 Q3 由「可忽略」翻成「不翻」，15 对判定不同（「第二段：两个旋钮的敏感性」一节），记「不稳定」；三个窗口里没有一点到 1.00 |
| M 行够判条件：三族负载上两个量都量到 | 已够判 | 第一段产物第 761 行 `m1_reached_on_every_point=true`、第 760 行 `all_window_prefixes_equal=true`；第二段产物第 1129 行 `m1_reached_on_every_point=true`、第 1128 行 `all_window_prefixes_equal=true`；第一段 88 行、第二段 132 行 `name=arm_run`，都 `stop=refused` | 不差 |

### 它答不了的

- 只数叶的分组与重写片数：不建内部节点的分隔 key 路由（M3 借走的号按分隔 key 会被路由到右片，这是改 I-9.4（容器号不超最小 key） 判据时一起答的问题）、不建崩溃点重放（C116（叶容器的分裂 / 合并纪律没有会失败的检查） 的另一半）、不建 GPU 判法要改的工作量。岔路 M 的选择因此还落在这几样没量的东西上。
- 一次发布一个操作；建 inode 成批、unlink 的意图那次与中间各批（原地改一条，三臂都只重写那一片）不进比较。根在每一次最后一批发布里三臂都重写，不进比较。
- 22 条序列是枚举点，不是定义域；W-R 只有三个 p，W-A 只有四种留法。Q1r 在这组负载上随 p 与留下比例翻面，别的负载形态（比如长时间稳态、成批删一个目录）在哪一侧没量过。
- M3 只量了下限两档（117、59）、借多少两档（均分、借到刚好够下限）、先查哪一对两档（右边、左边），三个旋钮没有交叉（借到刚好够下限与先查左边都只在 f = 117 上量）；别的借法（比如借到别的比例）没量。「借多少」让 Δ3c 与 Q3 翻面，那一档之间的取值在哪一侧没量过。
- 「先查哪一对」的敏感性里 99 对两点值逐位相等、挪门槛自证做不了；那几条序列上这个旋钮是不是根本没走到两对都装得下的那一支，没有逐步计数，读不出来。
- 每条臂最多 2 × 10⁷ 个操作，两段都没有臂撞到这个上限；W-A 留 1/233 上 M2 与 M2_left_first 吃了 13 547 775 个操作才被拒（第二段两段里最多的一条）。
- 填充率最小值一列在第一段 88 条、第二段 132 条臂上都是第一次建 inode 那一版的 1 ÷ 233，不说明稀疏程度。
- 「复跑」一节里那条 `replay.sh` 命令今天退 2、一行都不跑：复跑脚本整表先做表的形状检查与准入检查，第二段写这一页时卡在 E175（产品路径上 inode 号水位退回与重发号）那一行第 4 列写成了带斜杠的路径（第一段写页时卡在 E142（新池新建文件的干跑） 那两行的准入键冲突），都与 E176（inode 树叶合并三候选的容量与写代价） 无关；两段的产物都是手工跑同一条命令两遍、逐字节比的。

### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 独立手写模型 | 叶容器表只存号；插入末尾分裂、删空即摘、M2 并、M3 借，两种记法按这次发布动过的容器号与第一个摘片位置算。M1 与路径 2 比的是 22 条登记序列上 M1 被拒之前的每一步（枚举点，不是定义域：负载取登记的三族 22 点）；锚点格与路径 3 比 | `scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs:155 insert`、`:278 remove`、`:321 rewrites_of_this_publish` | 一片 233 条、根扇出 135（本地常量，值抄自 `crates/` 并在路径 2 那边回比）；位置记法的定义（读 `crates/singlefs-core/src/inode_tree.rs` 第 308–311 行得来）；负载生成器（对拍时两边吃同一个生成器的输出） |
| 2 | `crates/` 的两个纯函数 | 今天实现的 M1：`write_records_into_leaf_containers` 追加 / 末尾分裂、片数 > 135 拒；`remove_record_from_leaf_containers` 删空摘片、右边各片全进重写清单 | `crates/singlefs-core/src/inode_tree.rs:181 write_records_into_leaf_containers`、`:286 remove_record_from_leaf_containers` | `INODE_LEAF_RECORDS`（`crates/singlefs-format/src/lib.rs:110`）、`leaf_containers_one_root_node_holds`（`inode_tree.rs:109`）；对拍时的负载生成器取自路径 1 |
| 3 | 设计员的独立参照（Python） | 同样的三条臂，只在登记第七节的六个手造场景与空间放大那一格上算 | 无实现：只有抄本，在 `research/prompts/e176-preregistration.md` 第 500 行起的 `anchors.py` 代码块（草稿目录那份不入库） | 233、135；位置记法的同一个定义；它在 X 删空摘掉之后不查新相邻那一对（登记第十三节写明，六个格不受影响） |

跨路径的比对断言：路径 1 对路径 2 钉在对拍 bin `scripts/experiments/e176_crates_oracle/oracle.rs:80 compare_one_sequence`（逐步比，第一处不同就停）；路径 1 对路径 3 钉在单测 `every_registered_anchor_matches_the_independent_reference`（同一份源码第 2154 行，26 格登记答案逐字比；第二段另有 `segment_two_anchors_judge_a1_for_the_three_variants_and_only_report_the_rest` 比 29 格）。共用项问句：233、135 若错，三条路径一起错，挡不住「kb 与实现一起取错」；位置记法三条路径同出一处读法，若那读法本身不是 `crates/` 想要的，三条一起错；负载生成器只在路径 1 里有一份，对拍证明不了生成器照登记生成（那一半靠单测里四种负载的前几步钉住）。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 6 | 支撑 | 2026-10-08 改了：用户定「M3 连借一起做」（岔路单 `research/prompts/m3-3-5-forks.md` M 那一行），借取均分（`research/prompts/m3-leaf-death-r1-main-verification.md` 第二节 K3、`research/prompts/m3-leaf-death-r2-main-verification.md` 第二节「均分借」）；已定项 6 的合并那一句改成左吸收右与左从右借都做、借取均分，依据段引了 E176（inode 树叶合并三候选的容量与写代价）：合并买到的容量（Q1r）与多重写的叶片（Q2、Q3）是选 M3 的代价数，借到刚好够下限那个窗口让内容记法的多重写由「可忽略」变「不翻」。空间放大那一格与条款的 9.98 相同（F3 没打中） |

## 历史版本

### 2026-10-08：第二段已跑

- **改前**：状态「部分已跑（第一段；第二段的触发条件成立，待点名）」；第二段三个变体没跑，「它答不了的」写「借到刚好够下限与先查左边归第二段，没跑」；单测 14、变异 13；`scripts/experiments/replay.sh` 登记一行（`segment-one`）。
- **改后**：状态「已跑（两段）」；正文补第二段（22 条序列 × 六个臂变体 × 三个窗口）的锚点、两个窗口的 Q2 / Q3 读数、轨迹与借的次数、两个旋钮的敏感性与判决行点名；第一段那 38 对两点值相等的写成「挪门槛自证做不了的对数」；单测 22、变异 16（修订 3 补 V14–V16）；对拍与判别力用第二段的源码各重跑一次，与第一段逐字节相同；`replay.sh` 加一行 `segment-two`；D8（核心索引结构） 已定项 6 一行仍是备料，回看重写。
- **依据**：`research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out` 第 1137–1148 行（两个新窗口的总判定）、第 1369 行 `E7RESULT name=verdict scope=sensitivity_self_proof same_pairs=205 unstable_pairs=15 moved_to_unstable=74 equal_values_not_separable=131 every_separable_same_pair_moved_to_unstable=true`；跑前登记第十二节修订 3。

### 2026-10-08：建页，第一段已跑

跑前登记装置写之前写成（`research/prompts/e176-preregistration.md`）；模型单测跑完之后、产物之前补第十二节修订 1（f = 59 变体的阳性对照与公共窗口读法）；第一次跑变异表撞上可选路径依赖让 `mutate.sh` 的副本编不过，补修订 2（对拍 bin 改成独立 manifest）。第一段跑完，第二段的触发条件成立、待主 agent 点名。
