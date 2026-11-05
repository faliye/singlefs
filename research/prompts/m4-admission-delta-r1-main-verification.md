# m4-admission-delta-r1 判决：准入之前的步骤怎么做到不随总数涨（主 agent，2026-11-05，第一轮）

<!-- doc-lint:not-numbers A1 A2 A3 A4 A5 A6 N1 N2 H1 H2 H3 H4 H5 H6 H7 H8 H9 L1 K1 -->

正文 `research/prompts/_m4-admission-delta-r1-body.md`，背景材料 `research/prompts/_m4-admission-delta-r1-background.md`。腿：正推（Sonnet）`research/prompts/m4-admission-delta-r1-sonnet-output.md`，攻方（Opus）`research/prompts/m4-admission-delta-r1-opus-output.md`（模型 `m4-admission-delta-r1-opus-model/`），攻方第二次抽样 `research/prompts/m4-admission-delta-r1-opus-s2-output.md`（模型 `m4-admission-delta-r1-opus-s2-model/`）。代码读的是冻结副本 `~/.local/share/singlefs/work/m4-admission-delta-r1/repo/`（正文写成那一刻的主工作区 `crates/`、`Cargo.toml`、`Cargo.lock`，复跑前补进路径包 `prune-server`），产品代码 6 份的 sha256 在它旁边的 `crates-sha256.txt`。

## 一、核查

- 开工快照：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1-judge -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-admission-delta-r1 --out /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1-judge/m4-admission-delta-r1-tree --repository /home/fy5090/code/singlefs`（任务 6503）：「清单点名 4 份，倒推出 3 份、sha256 对上 3 份；倒推不出 1 份」。倒推不出的是 `.claude/kb/experiments/199-发布照抄上一版的代价.md`（快照之后 E199 执行员改过，HEAD 里没有）；三份腿报告只有正推第 222 行引它一处，引的原文「准入之前与准入之后两段各报各的」在此刻那一份里逐字还在（`grep -c` 1）。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-admission-delta-r1-sonnet-output.md research/prompts/m4-admission-delta-r1-opus-output.md research/prompts/m4-admission-delta-r1-opus-s2-output.md --root /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo --background research/prompts/_m4-admission-delta-r1-background.md`：「✓ 核了 109 处引文、193 处裸行号，对上 82 处，没判 220 处」；不带 `--root`（仓根，kb 那几份在这里）：「对上 108 处，没判 194 处（其中裸行号没有引文可比 191 处）」。两次都没有对不上的。
- 复跑（主 agent 经服务交，都不是重型，都在各自草稿目录里拷冻结副本跑，不碰主工作区）：
  - 攻方模型（任务 6265）：「name=full_root_leaf_one_insert leaf_entries=1 height_before=1 height_after=2 rewritten=3 literal_bound=1」（leaf_entries 取 2、3、8、64、255 各一行，都是 rewritten=3 literal_bound=1）；「name=keys_times_height_sweep capacities=5 histories_per_capacity=5 publishes_per_history=400 publishes_judged=9975 literal_bound_exceeded=1139 largest_excess_over_the_literal_bound=28 corrected_bound_exceeded=0」；「name=one_key_publish_output_size total_keys=100000 height=4 nodes_before=3124 nodes_in_the_returned_shape=3124 origins_returned=3124 rewritten=4 replaced=4 carried_at_a_new_position=0 input_key_set=100001」；K1 那一带四种几何（2×4GiB、2×64GiB、2×1TiB、2×16TiB）各一行 `name=k1_band`，例「name=k1_band geometry=2x4GiB slots_per_device=262144 root_level=2 … k1=529 … band_blocks_at_least=512 band_per_device_mib=8.0」；「test result: ok. 3 passed; 0 failed」。
  - 第二次抽样模型：第一趟（任务 6287）两趟都编不过「failed to get `prune-server` as a dependency」，是主 agent 冻副本时漏了路径包，补进副本后重交（任务 6394）：今天四条全「ok」，套上 A3 模拟之后「a_write_past_the_journal_limit_on_a_pool_short_by_the_formula_is_refused_for_its_records_without_raising_the_floor ... FAILED」，「name=rerun_summary today_rc=0 patched_rc=101 failed_in_patched=1 reorder_failed_in_patched=1」「name=rerun_verdict expected=yes」。
- 核不动的：无。

## 二、逐格判定（第一轮：打中的先挂起，三轮里多数打穿才算）

| 格 | 判定 | 依据 |
|---|---|---|
| 正文第二节的前提 | 正文写错 | 正推第 0 节与攻方 H1 独立对上：`judge_the_publish_before_any_read_or_write` 里两道准入，落点判（`transaction.rs:12523` `admit_the_units_landing_on_every_device`）与式子判（`:12621` `admit_on_every_device`）；多层码 2 树规划只经 `resolve`（唯一调用点 `:13063`，在 `settle_the_allocation_record_tree` 里，`:12536` 才调）到达，夹在两道之间。正文与实现员报告把它算成落点判之前。「准入之前」照 D8（核心索引结构） 已定项 16 射程读，要连两道一起算，不只第一道 |
| A1 随总数涨的步 | 正推列全（推的，代码读出来的） | 正推表一（落点判前）与表二（两道之间、每轮重跑、至多 64 轮）；正文没点名而列出的：别的文件逐个造计划、extent 上段 `previous_upper_nodes.contains` 线性、墓碑树退役树整份克隆再拷（攻方 H8 同报，「真正无界的是墓碑和退役容器的整份照抄」）、`unit()` 线性找四个根头（`transaction.rs:4351`）、记账树全量规划与不收缩的台阶行、聚簇段求和；两道之间每轮克隆分配器（`:13098`）、全记录分叶（`allocation_record_tree.rs:372`）、deadlist 的两个 `BTreeMap`。inode 叶容器拷贝今天封顶（至多 135 片、每片至多 233 条，`inode_tree.rs:1065`、`:157`） |
| 四条候选共用的洞 | 打中（两次抽样都报，不分辨臂） | 攻方 H5 与第二次抽样 H4：ckpt_cost 的记账树一项在落点判的参数里全量规划（`admission.rs:1339`、`:1349`、`:1358` 与 `code_two_tree.rs:651`），inode 号水位台阶行每建一次 inode 多一行、永不删，随历史无界涨；四条候选都罩不到。正推同报（记账树不是差量，D8（核心索引结构） 已定项 1 整批换代）。另立一笔账，不拿它判臂 |
| A2 规划按差量 | 没打穿（两次抽样），但做完仍随总数涨 | 正推：分裂、摘空、长高、降低、整理搬迁的重写节点集合差量与全量逐个相同（推的），要小心回放次序、同一次发布既出又进的 key（第二次抽样 H6：中央映射树 key 集合是 `mapped_units ∪ 留存条目`，差量要在并集上算）、`flatten` 位置编号平移（第二次抽样 H7 同报）；攻方 H4：做完固定点每一轮仍整份拷分配器、按叶分组比两遍全部记录（`transaction.rs:13098–13121`）。A2 的定义没罩到固定点 |
| A3 准入判挪到差量上界 | 打中（两次抽样都打中，第二次模型现量坐实） | 攻方 H7 与第二次抽样 H1：A3 改了「拒成哪一种」——今天记录超上限、合取 7、码 2 树拒都排在式子之前、原样交回、盘上不变，A3 下先被式子拒、会话推抬 F 写盘；任务 6394 现量坐实（今天绿、A3 模拟下恰好那一条红）。攻方 H3：字面式子「key 数 × 树高」不是上界，任务 6265 现量：9975 次发布里超过 1139 次、根叶装满再插一把重写 3 个节点而式子给 1。攻方 H2 与第二次抽样 H3：分配记录树拿得准的上界只有 K1，判它会多拒一带（任务 6265 的 `k1_band` 四行），不判它固定点就留在准入之前。第二次抽样 H2：按 ckpt_cost 数记录条数会把今天照发的写拒掉。正推另报：A3 乙与 D28（挂载期承诺量） 已定项 1 需求、时序两句字面冲突（第二次抽样 H5 同报）|
| A4 只换表示 | 打中（攻方 H6、正推都判不够） | 规划器收全量 key、深拷整棵上一版形状、交回整棵新形状，与上一版怎么表示无关；任务 6265 现量：「total_keys=100000 … nodes_in_the_returned_shape=3124 origins_returned=3124 rewritten=4」，交回的形状随总数涨、重写只有 4 |
| A5 碰到的条款 | 正推列出 | A2 只改实现（记账树严格不随总数涨要动 D8（核心索引结构） 已定项 1）；A3 甲要补定 C573；A3 乙要改 D28（挂载期承诺量） 已定项 1 两句；A4 只改实现 |
| 新候选 | 零轮 | 攻方 A6「差量规划连分配器差量预演」；正推 N1「只修落点判的前置并重述射程」、N2「A3 乙加精确复核做断言」 |
| 第二次抽样 L1 | 交下一轮核 | D28（挂载期承诺量） 的 W 给分配记录树每层取 2·(K+26)，是一个按差量的上界，与 K1 那句注释「收不了口」对不上 |
| 攻方 H9 | 采 | A1–A4 既是候选名又是格名，下一轮正文分开 |

## 三、下一轮（第二轮）

- 正文改：第二节的前提改成「两道准入」；候选改名不与格名撞；候选集合取 A2（把固定点与 key 并集写进定义）、A6、N1、N2 与 A3 的哪一种变体能躲开「拒成哪一种」那一格（不是原样的 A3：它这一轮打中的那一格已经现量坐实）；记账树台阶行那一个洞另立一格，单问「它有没有一种做法不动 D8（核心索引结构） 已定项 1」。
- 第二轮分工照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：攻第一轮的修补本身（A2 补上固定点之后、A6、N1、N2）/ 产生这些结论的方法（攻方两份模型的取样：`keys_times_height_sweep` 只扫 5 种容量、每种 5 段历史；A3 模拟只在一处挪了式子判）/ 从未被看过的地方（墓碑树退役树整份照抄、`unit()` 线性找根头这几处正文没点名的）。辩方腿复核这一轮 A3、A4 两格的判定够不够得着、是不是同样打中所有替代方案。
- 代价数归 E199（发布照抄上一版的代价）：照抄与现算重建字节由它量；这一轮正推列的「扫」类的行 E199 不量（正推第 222 行），第二轮正文写明哪一条候选的代价要另建计数模型。

## 四、交用户

| 事 | 出处 | 被攻过几轮 |
|---|---|---|
| 这一轮不交用户定：四条候选里 A3、A4 打中，A2 没打穿但定义不全，新候选零轮；照规则再攻两轮再交 | 本判决第二节 | 一轮 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-11-05 不受影响：这一轮只判候选，没有定案；射程「准入之前」照字面要连两道准入一起算，这是读法，条款不改 |
| D8（核心索引结构） 已定项 1 | 不影响 | 2026-11-05 不受影响：记账树台阶行那一个洞记下，要不要动它留到第二轮 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-05 不受影响：A3 改了「拒成哪一种」，这一轮挂起，条款不改 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-11-05 不受影响：A3 乙与它两句字面冲突，挂起 |
