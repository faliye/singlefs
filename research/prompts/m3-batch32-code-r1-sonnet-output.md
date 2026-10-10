# m3-batch32-code-r1 云端正推（Sonnet）报告（2026-10-07）

立场：云端正推，格 K1（P1–P7）与 K3。从条款、判决格推到代码行、测试与变异行，再与冻结副本逐格比；一样就写一样，不一样把两边整行并排抄。
引代码写冻结副本全路径加行号，引 kb 与判决写那份文件自己的行号；方案按 `crates/` 今天的实现来谈。线程上限 5 → 4 的消息收到于第一次编译之前，编译与测试都按 4 起。
GPU 一侧只读码（`.wgsl`、`crash_judge_tables.rs`），不编译 checker-tier、不跑 GPU；GPU 那一侧的「会给什么」用 CPU 一侧的现成读者与我写的 u32 移植模型算期望，标「推的」。

## 〇、我这一腿读过 / 跑过的

| 事 | 命令或位置 | 观测 |
|---|---|---|
| 被判 9 份与快照一致 | `cd <冻结副本> && head -9 research/prompts/m3-batch32-code-r1-snapshot/crates-sha256.txt \| sha256sum -c -` | 9 行都是 `OK`（`walk.rs`、`crash_judge_checker.wgsl`、`crash_judge_common.wgsl`、`crash_judge_checker_scan.wgsl`、`crash_judge_tables.rs`、`crash_judge_dispatch.rs`、`checker_known_bad_images.rs`、对拍文件、`mutations.tsv`） |
| 冻结副本上 harness 档基线 | `rerun.sh <工作目录> baseline`（`cargo test -p singlefs-harness --test checker_known_bad_images -- <七个子串>`，经内存包装，编译 6G、跑 4G，线程 4） | 末行 `test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 16.49s`（这一批新加的 14 条加同名子串带出的 F-W 一族 16 条） |
| 变异表第 3078–3089 行锚点 | `rerun.sh … anchors`（`apply_mutation.py anchors` 加 `gpu_named_tests_exist.py`） | 12 行都是 `anchor_hits=1`、点名的测试是 `#[test] fn`、参数筛得到点名的测试；GPU 6 行 `--exact` 名等于点名名 |
| 我的探针与模型 | `rerun.sh … probe i49 model mutations broad extra` | 见各节；模型目录 `research/prompts/m3-batch32-code-r1-sonnet-model/` |
| 空目录从头复跑 | `SINGLEFS_REPO=<冻结副本> bash …/rerun.sh <新空目录 run2> all`，再 `compare.sh` 比 run1 与 run2 | `real 9m9.430s`；`diff` run1 的汇总与 run2 的汇总无输出（两份 sha256 都是 `65c581f50b57862ea711de8df30d37a5cd77f5d3cdde71ca045c9964bca1b2ee`，230 行）；汇总存为模型目录的 `expected-compare.txt` |

说明：这一轮机器负载很高（`uptime` 看到过 load average 157）。E5、E8 两行第一次被内存包装的挂钟上限截断（退出码 253，`ran` 不满 30），我删掉那两行重跑，报告里用的是完整跑完的那一次；其余阶段（probe、i49、mutations、broad、extra 别的行）都是一次跑完。

## 一、K1 / P1　F-KEY：「走过」的键

### 1.1 CPU 与 GPU 三处「走过」是不是同一个落点

CPU 侧读单元与认落点由同一个方法交回，三处共用：「fn read_a_unit_to_walk_and_the_placement_it_was_read_from(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:1248`）。它调 「pub fn read_referenced_unit_with_the_location_it_was_read_from(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/image.rs:1517`），后者每条位置条目都判 I-2.1、交回第一份对得上的那一条位置条目；所以「读回的落点」就是第一份校验和对得上的那一份的落点。
「走过」的插入键，原样（前四条是这一批 F-KEY 改的四个落点：容器树内部孩子、inode 叶容器、inode 内部孩子、extent 数据单元，正文说的「三处」是容器、inode 孩子、数据单元三类；最后一条是早先改的实例表链）：
- 容器树内部条目的孩子：「if !self.visited_units.insert(placement_it_was_read_from) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:2640`）
- inode 叶容器：「if !self.visited_units.insert(placement_it_was_read_from) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:2947`）
- inode 内部孩子：「if !self.visited_units.insert(placement_it_was_read_from) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:3067`）
- extent 数据单元（第五处，同方法）：「if !self.visited_units.insert(placement_it_was_read_from) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:3435`）
- 实例表链（早先改的，不是这一批）：「if !self.visited_units.insert(placement_it_was_read_from) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:3235`）

GPU 侧：`w_read_referenced_unit` 每次先把标志清零、第二条位置条目那一份顶上时置位（「w_read_back_from_the_second_location = true;」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1310`）），`w_visit_the_copy_read_back` 按标志选落点（「fn w_visit_the_copy_read_back(p: Ptr) -> bool {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1318`））。三处调用与 CPU 三处一一对应：extent 数据单元（「if (!w_visit_the_copy_read_back(p)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1863`））、inode 叶容器（「if (!w_visit_the_copy_read_back(child)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2751`））、容器树内部条目的孩子（「if (!w_visit_the_copy_read_back(child)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3288`））。GPU 没有 inode 内部孩子这一处：走读内核只接层级 1 的 inode 树（`w_walk_inode_root`），CPU 那一处 GPU 没有对应物，不是漏改。
标志的有效范围：读与访问之间没有别的 `w_read_referenced_unit` 调用（我逐处看了这三处读到访问之间的代码：只有判头、判身份、`w_failures` 与 `continue`，没有递归也没有再读），读码一致。

**判定**：同一个落点（读回、自证过的那一份的落点）；CPU 与 GPU 一致。推翻它的现象：某一处在读与访问之间多出一次 `w_read_referenced_unit`（GPU）或换了读法（CPU）。

### 1.2 没被改的那几把键，该不该跟着改

没改的键与它们今天的落点，原样：
- 父单元：「fn start_walking_below(&mut self, unit: (u32, u64)) -> Option<(u32, u64)> {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:1216`）；inode 内部孩子那一处往下走时传的是第 0 条位置条目的落点：「self.walk_inode_internal_node(&view, tree, child_placement, &mut leaves_in_leaf_order);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:3135`）。
- 诞生代号那张表：「self.birth_txg_of_walked_units.insert(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:1021`），键在函数里写死成 `pointer.locations[0]`。
- 合并：三处都是 `merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement)`（如 「self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:2641`）），`child_placement` 取第 0 条位置条目。
- inode 叶序：「inode_leaf_order_below_a_walked_unit: BTreeMap<(u32, u64), Vec<InodeLeafContainerInLeafOrder>>,」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:397`），写与读都按 `child_placement`。
GPU 侧同一批：诞生代号写在「走过的单元表」里那一行，按第 0 条位置条目的落点找行（「let visited_index = w_visited_index_in_the_main_walk(p.device0, p.slot0);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1421`）），合并也是 `w_merge_below(child.device0, child.slot0)`（「w_merge_below(child.device0, child.slot0);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3289`））。

**读码推出的后果（推的，下面 1.3 用探针坐实一半）**：
1. 读回的落点就是第 0 条时（合法状态都是这样，见 1.4），新旧键相同，什么都不变。
2. 读回的是第 1 条那一份时，`visited_units` 按第 1 条的落点登记，而诞生代号、父单元、合并、叶序仍按第 0 条的落点登记。第 0 条的落点若正好是另一个单元 X 的落点（攻方 loc0 那一形），这四张表里 X 的那一行被新单元 C 覆盖或并进：
   - 诞生代号：`birth_txg_of_walked_units[X]` 被 C 的诞生代号盖掉；回退目标那条非最新候选根再走到 X 时，「if let Some(birth_txg) = self.birth_txg_of_walked_units.get(&unit).copied() {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:1285`）按这个数判 I-4.9。条款那一行：「按落点认，读落点上单元头里的诞生代号」（`.claude/kb/invariants.md:185`），X 落点上的头是 X 自己的头，所以按 C 的数判是误红；这是主 agent 正文第二节那一形，我读码独立推到同一处。
   - 父单元：C 孩子的落点（若有）记在 `placements_referenced_below_a_walked_unit[X]` 下面，与 X 自己的孩子并成一堆，之后别的版本经 X 合并时把 C 的孩子也并进来。容器是叶，没有孩子，这一项只在 inode 内部孩子那一处有牙。
   - 叶序：`inode_leaf_order_below_a_walked_unit[X]` 同样被覆盖，限 inode 叶容器 / 内部孩子两处。
3. 第 0 条那一份对不上、第 0 条的落点又没有别的单元占着时：CPU 的诞生代号表、父单元、合并、叶序都按第 0 条的落点记、也按它取，彼此一致；`visited_units` 按第 1 条的落点登记，只与它自己配对。GPU 把诞生代号写进「走过的单元表」按第 0 条找到的那一行，那一行不存在（没按第 0 条走过），记不下，合并时同一个键也找不到：CPU 判 I-4.9、GPU 不判（实现员报告停下交主 agent 第 1 条的第二形，读码同）。


### 1.3 探针与候选改法（I-4.9 那一形，P1 的后半）

探针 `probe_of_the_sonnet_leg_for_batch32_i49`（`probe_append.rs`，追加在副本里 harness 测试文件 `checker_known_bad_images.rs` 的末尾，只读、不断言）：把这一批 F-KEY 三形的造法（`image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width`）各做两种盘序，打红集与 I-4.9 的说明。没改的冻结副本上，原样（`i49.txt` 里 `i49-base` 的那 6 行）：
```
PROBE i49-base lineage named_walked_node_in_location=0 red=["I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-4.9", "I-5.1", "I-7.4"] I-4.9=Violated:别的版本先走过的盘 0 槽 50244 的单元：诞生代号 9 大于引用它的候选根的 txg 3
PROBE i49-base lineage named_walked_node_in_location=1 red=["I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"] I-4.9=Holds
```
墓碑与 inode 两形两种盘序红集都是 `["I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"]`、I-4.9 都是 `Holds`（推的：这两形的镜像里没有非最新的候选根再走到被盖住的那个落点，I-4.9 报成立不是因为键对、是没人去查；我没有造出那一形）。
只有谱系 loc0 那一形多红 I-4.9，与实现员报告与主 agent 正文第二节一致。「说明点名盘 0 槽 50244」：就是被先走过的 inode 树根 X 的落点。X 的单元头里的诞生代号不晚于回退目标那条根，说明里的「诞生代号 9」是新容器 C 的数（谱系容器是这一版新写的）。结论：误红，原因就是 1.2 第 2 点的诞生代号表被覆盖。

候选改法 F1（「一个单元的身份是它被读回的那一份的落点」）：诞生代号、合并、父单元、叶序四把键都按读回那一份的落点。我在副本上只做了容器那一处的两处替换（`extra_mutations.tsv` 第 9、10 行：合并键改成 `placement_it_was_read_from`；诞生代号那一处把两条位置条目对调之后交给原函数），再跑同一支探针，原样（`i49.txt` 里 `i49-f1` 的谱系两行）：
```
PROBE i49-f1 lineage named_walked_node_in_location=0 red=["I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-3.5", "I-4.8", "I-5.1", "I-7.4"] I-4.9=Holds I-3.5=Violated:相邻两版 (实例 {2}、txg 8) → (实例 2、txg 9)（NoHole）：后一版引用、前一版不引用的单元盘 0 槽 50244（用户可见树 None）诞生代号 3 不大于前一版的 txg，也不在一棵整棵继承下来的用户可见树里（根指针出生 txg 小于前一版的 txg、子树里读得出的诞生代号都不大于它）
PROBE i49-f1 lineage named_walked_node_in_location=1 red=["I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"] I-4.9=Holds I-3.5=Holds
```
墓碑与 inode 两形在 F1 下红集不变。结果：F1 只改容器那一处，谱系 loc0 的 I-4.9 不再红，**但 I-3.5 从成立变成红**。I-3.5 的出现规则按 `unit_of_each_placement`（落点 → 第 0 条位置条目的单元）取诞生代号。下面是我对出来的（推的，依据是 `i49-base` 里 I-3.5 成立与 `i49-f1` 里说明点名的数 3）：改前那个被盖掉的数（9，即 I-4.9 说明里的「诞生代号 9」）恰好让「后一版引用、前一版不引用的单元」那一条 (a)「诞生代号 > 前一版 txg（8）」过得去，改后取到 X 自己的数 3，(a) 不成立，红。也就是说，I-4.9 的误红与 I-3.5 的「成立」是同一个被盖掉的数的两面，只改诞生代号表的键会把一个误判换成另一个。
F1 要连带核的一处（探针已经坐实，下面是读码给的原因）：`unit_of_each_placement` 与 I-3.5 的出现规则按第 0 条位置条目的落点取诞生代号（「unit_of_each_placement: BTreeMap<(u32, u64), (u32, u64)>,」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:434`）），诞生代号表若改按读回的落点存，I-3.5 在读回第 1 条的单元上取不到诞生代号（「头读不出的不判」）：要么 `unit_of_each_placement` 也按读回的落点重映，要么诞生代号两处键各存一份（第 0 条的落点没被别的单元占着时）。这是 F1 的第二个落点，不是 F-KEY 三处本身的缺陷，写在这里是因为「该不该跟着改」不能只改一把键。

### 1.4 F-KEY 会不会让合法状态多红或漏判（盘 0 那份没落、盘 1 落了）

读码推出，两条互相独立的理由：
1. **只在读回的是第 1 条时键才变，而那时 I-2.1 已经红**：「judgements.judge("I-2.1", matches,」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/image.rs:1530`）对每条位置条目都判，「if matches && first_good.is_none() {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/image.rs:1536`）只取第一份对得上的。第 0 条对得上时读回的就是它、键与改之前相同；第 0 条对不上则 I-2.1 当场红。所以 F-KEY 不可能把一个 I-2.1 成立的状态判红，也不可能把它漏成成立（键不变）。多红只能出现在 I-2.1 已经红的状态上，且只多红 I-1.3、I-1.7、出生身份那几条（因为这一份不再被误认成「走过」）。
2. 合法崩溃状态里根引用的单元两份都已落盘：发布的持久顺序是 「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）」（`.claude/kb/decisions/16-发布语义.md:195`），两份单元写都在第一道屏障之前（读码推的，两份单元写属同一批单元写）；「盘 0 那份没落、盘 1 落了」的单元只可能是还没被任何根引用的新单元，走读走不到它。实现员报告同一句（推的，没量）。

推翻它的现象：一个 I-2.1 全成立的层 0 状态，在 F-KEY 之前后判定集合不同（只有读回的落点与第 0 条不同才会，而那时 I-2.1 红，所以不存在）。我没有跑层 0（要 checker 档），这一条是读码推的。
GPU 同一理由：`w_read_referenced_unit` 对两条位置条目都 `w_judge(INVARIANT_I_2_1, …)`，标志只在第 0 条没对上、第 1 条对上时置位（「if (matches1 && first.version == NONE) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1308`））。

### 1.5 K1 / P1 判定

- 三处「走过」的键：CPU 与 GPU 一致（读回的那一份的落点）；`read_index_node` 与 `w_read_index_node` 都没改（「let placement = (pointer.locations[0].device, pointer.locations[0].slot);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:1504`）；GPU 同形 「if (!w_visit(p.device0, p.slot0)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1513`）），代码注释说「两处就按两个落点认同一件事」（「只改这里的键，两处就按两个落点认同一件事」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:1503`））——同一句对三处改过的键也成立：它们的「走过」与合并、诞生代号、父单元、叶序按两个落点认同一件事，这是 1.2 的冲突。
- 没改的键逐把说（读码推，1.3 的探针坐实诞生代号那一把）：

| 键 | 该不该跟着改 | 理由 |
|---|---|---|
| 诞生代号表 `birth_txg_of_walked_units` | 该 | 条款写「按落点认，读落点上单元头里的诞生代号」，头是读回那一份的头；按第 0 条记会盖掉别的单元的数（I-4.9 误红，实测） |
| 合并（`merge_placements_below_a_unit_walked_by_an_earlier_version`） | 该 | 与「走过」是同一件事的两端：先走过的按读回的落点登记，再被引用时要按同一个落点找回来；代码自己的注释就这么说（`read_index_node` 那一条） |
| 父单元（`start_walking_below` 的键） | 该，且必须与合并同键 | 合并拿它当「这个单元下面的落点」的表键；不同键会合并到别的单元的孩子（只在 inode 内部孩子一处有孩子，其余两处是叶，没有牙） |
| inode 叶序（`inode_leaf_order_below_a_walked_unit`） | 该，读写同键 | 同上；只在 inode 叶与 inode 内部孩子两处用 |
| `unit_of_each_placement`（I-3.5 把两份副本折成一个单元） | 随诞生代号表一起处理 | 探针：只改诞生代号表的键，I-3.5 由成立变红（1.3） |

  跟着改不是改一行，要成套换，再把 I-3.5 的取数一起核。
- 字段文档已经过时：「走过的被引用单元（按位置条目第 0 条的落点认，与 `visited_units` 同一个键）→ 头里的诞生代号」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:427`），这一批之后 `visited_units` 对三处已不是第 0 条的落点。

## 二、K1 / P2　F-ID-b：「自证过」的判据

### 2.1 CPU 与 GPU 是不是同一个判据，与 F-W 认别类那一处是不是同一个

| 一项 | CPU | GPU |
|---|---|---|
| 头怎么读 | 按最短的单元（16 KiB）读头取类标签：「reader.read(location.device, offset, node_bytes())?[UNIT_HEADER_CLASS_TAG_OFFSET];」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:4519`） | `read_at(…, NODE_BYTES)` 取 `rd8(first_copy.base, 6u)`：「let first_copy = read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3089`） |
| 类标签没登记 | `unit_bytes_of_the_class(…)?` 交 None，这一份不算：「let unit_bytes = unit_bytes_of_the_class(MappingKeyUnitClass::of(class_tag_in_the_header))?;」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:4520`） | 码 1 / 2 / 3 之外 `w_unit_verifies_at_the_length_of_its_class` 恒 false：「fn w_unit_verifies_at_the_length_of_its_class(s: Slice, unit_class: u32, checksum: u32) -> bool {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3046`） |
| 按哪种长度比校验和 | 码 2 16 KiB、码 1 / 码 3 32 KiB（`unit_bytes_of_the_class`），与位置条目里的校验和比 | 同（`slice_crc_matches` 码 2 比 c0、码 1 / 3 比 c1 且要落点有 32 KiB） |
| 两份的次序 | 位置条目顺序，第一份自证过的就停：`child.locations.iter().find_map` | 第 0 条先、第 1 条后，第一份自证过的就返回 |
| 哪份都没自证过 | `.or_else(…read_unit_without_judging(…, node_bytes()))`，取第一份按 16 KiB 对得上的头里的类标签（「read_unit_without_judging(self.reader, &child, node_bytes())」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:2415`）） | `read_unit_via_locations(child, NODE_BYTES)`，同一读法（「let child_unit = read_unit_via_locations(child, NODE_BYTES);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3213`）） |

F-W 认别类那一处（`child_unit_class_other_than_the_entry_type_names`，「fn child_unit_class_other_than_the_entry_type_names(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:4477`））与新函数用的是同一个谓词：头里类标签登记、按那一类的长度读回的整单元校验和对得上。GPU 侧三个函数（`w_child_unit_class_other_than_the_entry_type_names`、`w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`、新函数）共用 `w_unit_verifies_at_the_length_of_its_class` 一个函数，是同一个判据；CPU 侧这个谓词在三个函数里各写一遍（`child_unit_class_other_than_…`、`class_tag_of_the_first_copy_…`、`some_copy_of_the_child_verifies_as_…`，后者按类型段那一类的长度读、再比头里的类标签，等价），今天逐行等价，但没有共用代码，改一处会漂（读码，推的）。

调用结构也同形：先问 F-W（有没有一份自证成码 2 之外的一类；有、且另有一份自证成码 2 时身份红、往下走码 2 那一份；没有时身份红、不往下走），再问类型段 0 那一支认「孩子是码 2」：CPU 「class_tag_of_the_first_copy_verifying_at_the_length_of_its_class(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:2410`），GPU 「var class_of_the_child_of_a_type_zero_entry = w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3211`）。

**判定**：同一个判据；与 F-W 认别类同一个谓词。推翻它的现象：某一类标签（码 0、4–255）在 CPU 与 GPU 上被一边当自证过、一边不当；或码 1 / 3 的校验和长度在两边取得不同。

### 2.2 两份副本的组合两两扫（两种盘序）

探针 `probe_of_the_sonnet_leg_for_batch32_tomb` 与 `…_lineage`（`probe_append.rs`）：墓碑树与谱系树各自根（层级 1）第一条条目改成类型段 0、身份三段清零；孩子两份副本各取六种状态之一：
S3 自证成码 3（原样）、S2 自证成码 2（盘上那一槽头 16 KiB 写成这棵树根的字节、校验和换成它 16 KiB 的 CRC）、C16 头里是码 3 而校验和只对得上前 16 KiB、BAD 校验和翻位、UNREAD 盘身份不在池里、ZERO 槽指着没写过的 16 KiB 而校验和是全零 16 KiB 的 CRC（F-零槽之后读出全零）。6 × 6 × 2 棵树 = 72 格。
模型 `p2_matrix_model.py` 照读码写：自证过的别类与自证过的码 2 并存时 F-W 判红；否则孩子的类取第一份自证过的，没有就取第一份 16 KiB 对得上的，再没有读不出；红 = 读得出且不是码 2。
结果：72 格的「条目身份那一条（I-9.23 / I-9.17）红不红」与模型逐格相同（`PROBE p2 … identity_red=`；比对脚本的输出：`tomb 36 cells, 与模型不同的格: []`、`lineage 36 cells, 与模型不同的格: []`）。两棵树的 36 格网格相同，R 是红、点是不红，行是位置条目 0、列是位置条目 1：
```
          S3     S2    C16    BAD UNREAD   ZERO
    S3     R      R      R      R      R      R
    S2     R      .      .      .      .      .
   C16     R      .      R      R      R      R
   BAD     R      .      R      .      .      R
UNREAD     R      .      R      .      .      R
  ZERO     R      .      R      R      R      R
```
读法：
1. 网格对称（互换位置条目 0 与 1，每一格同色）：F-ID-b 要的「不随盘序翻面」成立，C16 与 S2 并存（A3 那一对）两种盘序都不红。
2. 红集（不只是身份那一条）互换之后有 5 对不同，全部只差 I-2.5，来自我的造法：UNREAD 把盘身份写成 7，排在位置条目 0 时 I-2.5（位置条目按设备身份严格升序）红，排在 1 时不红；与 F-ID-b 无关。
3. 两份都没自证过的格（C16、BAD、UNREAD、ZERO 之间的 16 格）里，今天含 C16 或 ZERO 的 12 格红（取第一份 16 KiB 对得上的头里的类标签，码 3 或码 0 都不是码 2），BAD 与 UNREAD 之间 4 格不红（读不出、这一小句不判）。正文第一节说这一形用户 2026-10-07 定「当读不出、这一小句不判」、第 33 批改：改完这 12 格全部翻成不红；这是第 33 批要改的格数，不是这一批的缺陷。
4. 两份各自证成不同类（S3 与 S2）今天红（F-W 那一支，身份判红再往下走码 2 那一份）；两份各自证成码 3 之外的两种别类（例如码 1 与码 3）的组合，我的造法里没有码 1 的单元（墓碑镜像里没有现成的数据单元槽），没扫；那一形的槽数登记随盘序变，是 「认孩子的类时两份副本各自证成不同类，取哪一类、按哪一类登记槽数没有条款」（`.claude/kb/checks-owed.md:588`） 的 ①，仍开着，与这一批无关。

### 2.3 F-ID-b 各处钉在哪（变异）

CPU 变异表第 3079 行红且只红点名那一条（见第三节表）。我另加三行（`extra_mutations.tsv`）：
- E8「自证判据不看长度（按 16 KiB 比，头里的类标签对就算）」：红，倒下的是这一批 F-ID-b 的 4 条（墓碑 / 谱系 × 层级 1 / 层级 0），见第三节。F-ID-b 要的「按那一类的长度」钉住了。
- E7「取最后一份自证过的副本而不是第一份」：存活。没有测试区分「第一份」与「任一份」，因为两份各自证成不同类时 F-W 先判红，结论与盘序无关；这一点正是 C739 ① 开着的地方，不是这批测试的漏。
- E6「去掉两份都没自证过时照改之前读」：存活。没有一条测试钉那 12 格的今天读法，第 33 批改它时要自带测试。

**判定**：一致；与 F-W 同一谓词；72 格与模型相同。推翻它的现象：层级 ≥ 2 的树里类型段 0 的条目上，两份副本组合的结论随盘序变（G3 两条只钉了一种组合，我的网格只在层级 1 的根上扫，层级 ≥ 2 没扫）。

## 三、K1 / P3　测试与变异

### 3.1 这一批新加的 14 条 CPU 坏镜像（冻结副本里，基线 30 条全过）与对拍流 3 条

`rerun.sh … baseline` 的末行：`test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 16.49s`（这一批新加的 14 条加同名子串带出的 F-W 一族 16 条）。

每条测试怎么钉、被哪些变异红掉（「表行」是 `crates/mutations.tsv` 的行号，「E」是我在副本上另加的变异，`extra_mutations.tsv`；「宽筛」指我用同一组 7 个子串筛 30 条跑那一行变异，`mutations-broad.txt`，每行都是整份 30 条里红了哪几条）：

| 测试（冻结副本行） | 钉的那一读 | 红掉它的变异 |
|---|---|---|
| 「fn a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13399`） | 容器树内部孩子的「走过」（loc0 / loc1 两种盘序，I-1.7 红在记录宽、红集 loc1 逐字、loc0 与 loc1 同批） | 表行 3078（只红它）、E5、E1 |
| 「fn an_inode_leaf_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13417`） | inode 叶容器的「走过」 | E3（只红它）、E1、宽筛 3078 |
| 「fn a_lineage_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13437`） | 谱系容器的「走过」；I-4.9 列进「允许不同」（`allowed_to_differ`）：两种结果都过，没钉 | E5、E1、宽筛 3078 |
| 4 条 F-ID-b（墓碑 / 谱系 × 层级 1 / 层级 0，行 13556、13570、13586、13601） | 认「孩子是码 2」只看自证过的副本；红集恰好 `["I-2.1", "I-4.8", "I-7.2", "I-7.4"]`，两种盘序 | 表行 3079（点名那条）、宽筛 3079（4 条全红）、E8（4 条全红） |
| 「fn a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13620`） | F-L0 在层级 0 的根下照判 I-2.1，红集恰好四条 | 表行 3080（只红它） |
| G1 墓碑 「fn a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13699`）、谱系（行 13721） | 类型段 0 那一支往下走码 2 那一份；I-7.2 红在「的节点有类型段 0 的条目」那一句 | 表行 3081 = S1（点名那条）、宽筛 3081 |
| G1 inode 「fn an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13746`） | inode 类型段 0 那一支往下走；I-1.1 红在「inode 树的内部节点」 | 表行 3082 = S2（只红它：行自己的 6 条里红 1 条） |
| G2 「fn an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13767`） | 头里类标签是同长度别类（码 1）时不算「自证成类型段那一类」，不往下走 | 表行 3083 = S4（只红它） |
| G3 两条（行 13955、13961） | 层级 2、F-W 往下走那一份子树 | 表行 3081 = S1（两条都红） |

红集原样（行自己的 6 条参数，`mutations.txt` 的 `RESULT table` 行，只摘 `failed=`）：3078 `failed=1`（点名那条）、3079 `failed=1`、3080 `failed=1`、3081 `failed=3`（G1 墓碑加 G3 两条）、3082 `failed=1`、3083 `failed=1`；六行都 `verdict=RED`；红的都是该行参数选到的、与它改的那一读有关的几条（3081 另红 G3 两条，别的行只红点名那一条）。变异表第 3078–3083 行六条 CPU 变异我都在冻结副本的拷贝上现跑过，全红。

对拍流 3 条（GPU，读码，没上卡）：
- F-KEY 「fn the_gpu_judge_agrees_with_the_cpu_when_the_first_location_of_a_tombstone_container_with_a_wrong_record_width_names_a_node_walked_before(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10374`）：要 CPU 红 I-1.7、I-2.1，每个状态 GPU 与 CPU 逐位相同；只有墓碑容器那一处（`w_walk_container_tree_root`）有流，inode 叶容器与 extent 数据单元两处 GPU 调用点没有流，变异行 3084 改的是三处共用的 `w_visit_the_copy_read_back`，只有墓碑那条流会看见。
- F-ID-b 「fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_checked_at_sixteen_kibibytes_beside_a_node(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10457`）：要 CPU 红 I-2.1、I-7.2，逐位相同；「I-9.23 不红」靠 CPU 与 GPU 逐位相同加 CPU 一侧的 harness 测试那一条，对拍流自己没写 `!contains("I-9.23")`；变异 3085 把 GPU 改成 `NONE`，GPU 红 I-9.23、与 CPU 不一致，流会红。
- F-零槽 「fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_with_the_checksum_of_zeros(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:10530`）：槽 190000，16 KiB 全零的 CRC。读 16 KiB 的读者、盘中间的缺口、32 KiB 那条全零单元；**16 KiB 那条全零单元（盘尾）与槽号回绕（第四节 4.2）没有流**，变异 3089 把整个 `select(0u, 1u, …)` 换成 `NONE`，不区分 `0u` 与 `1u`，把 `1u` 改成 `0u` 的变异会存活（推的：只有指着盘上最后一槽、校验和是全零 32 KiB 的 CRC 的位置条目才区分两条——走读不往下读的孩子那一支「16 KiB 与 32 KiB 两种长度都比」，32 KiB 那条会误对上，CPU 读不到——流里没有这样的条目）。
变异表 GPU 六行（3084–3089）我只核了静态（`anchors.txt` 的 `GPU-ANCHOR` 六行：锚点各命中一次、点名的测试是 `#[test] fn`、`--exact` 的名字等于点名的名字）；红不红要上卡，归提交时。

### 3.2 S2、S4、G4 各自钉不钉得住它要钉的那一读

- **S2**（行 3082）：把 inode 树那一处的 F-W 门改成只在类型段 2 才走：红，且只红 G1 inode 那一条（`failed=1`）。钉住了「inode 类型段 0、另一份自证成码 2 时照类型段那一支往下走码 2 那一份」。
- **S4**（行 3083）：认「另一份自证过」不看头里的类标签：红，且只红 G2 那一条（`failed=1`）。钉住了「按类型段那一类的长度比校验和对得上、头里却是同长度别类时不算」。
- **G4**（`for invariant_of_the_checksum_family in ["I-2.1", "I-4.8", "I-7.4"]`，「for invariant_of_the_checksum_family in ["I-2.1", "I-4.8", "I-7.4"] {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:12111`））：**钉不住「F-W 往下走时读那一份按同一长度比校验和」这一读**。E1 把容器树 / inode / 数据单元三处的读改成「不判 I-2.1、只取第一份校验和对得上的」（`extra.txt` 第 1 行）：红的是三条 F-KEY（它们的红集里要 I-2.1），G4 所在的那 5 条 F-W 一族（`…beside_a_copy_verifying_as_a_node_reddens_…`）一条不红：它们要的 I-2.1 一族另有来路（中央映射条目读同一片容器时那一份也对不上，harness 里那句注释自己写了）。实现员报告停下交主 agent 第 4 条同一件事，这里是它的实测。

### 3.3 没被钉住的几处（实测，不是推的）

| 变异（我的，`extra_mutations.tsv`） | 改了什么 | 结果 | 含义 |
|---|---|---|---|
| E2 | F-KEY 只退 extent 数据单元那一处 | 存活（30 条全过） | extent 那一处没有坏镜像钉 |
| E4 | F-KEY 只退 inode 内部孩子那一处 | 存活 | inode 内部孩子（层级 ≥ 2 的 inode 树）那一处没有坏镜像钉；GPU 本来没有这一处 |
| E6 | 去掉两份都没自证过时照改之前读 | 存活 | 第二节 2.2 第 3 点那 12 格今天读法没有测试钉；第 33 批改它时自带测试 |
| E7 | 取最后一份自证过的副本而不是第一份 | 存活 | 「第一份」与「任一份」没有测试区分（C739 ①） |
| E1 | 三处读不判 I-2.1 | 红 3 条 F-KEY，F-W 一族不红 | G4 钉不住那一读（3.2） |
| E3、E5、E8 | 分别退 inode 叶、容器树两处、认自证只比 16 KiB | 都红，E3 红 1 条、E5 红 2 条（墓碑与谱系）、E8 红 4 条 | F-KEY 的容器树与 inode 叶两处、F-ID-b 的长度读法钉住了 |

I-4.9 那一形：谱系 F-KEY 测试把 I-4.9 放进「允许不同」，没钉它红不红（注释自己写「交主 agent 定，这里不钉它」）。第一节 1.3 的 F1 落地时，这条测试仍过（`allowed_to_differ` 吃掉两种结果）；要钉就得另加一条「I-4.9 成立」的断言，连同 I-3.5 那一格的取舍（1.3）。

### 3.4 变异行点名的测试与参数筛不筛得到

`anchors.txt`（`apply_mutation.py anchors` 与 `gpu_named_tests_exist.py`）原样：12 行都是 `anchor_hits=1`；CPU 六行 `named_is_a_test=yes named_selected=yes`，每个参数子串各命中一个测试名（`per_filter=1,1,1,1,1,1`，3081 多一个 `below_a_level_one_node_whose` 命中 2 条）；GPU 六行 `exact_arg_equals_named=yes`、同名测试只有一个。
一个读码提醒：六条 CPU 行的参数列表相同（3081 多一个子串），都只选 6 条里的点名那条与另五条；谱系的 F-KEY、F-ID-b、G1 与层级 0 那几条没有在任何一行的参数里，它们被红不红只能靠我的宽筛看到（3.1 表里）。这不违反变异表的要求（点名的那条红即可），只是谱系那几条今天没有一行变异单独保它们。

**判定**：测试期望对条款写全；S2、S4 钉住；G4 钉不住 F-W 那一读；F-KEY 在 extent 与 inode 内部孩子两处、F-ID-b 在回落读法两处没有测试钉；I-4.9 那一形没钉。推翻「E2 / E4 没钉」的现象：存在一条 harness 测试在 E2 或 E4 下红（我跑的 30 条里没有；整份 `checker_known_bad_images` 是主 agent 在合入验证里跑的 188 条，没人在 E2 / E4 下跑过，我只跑了 30 条）。

## 四、K1 / P4　F-零槽：GPU 交回全零单元

### 4.1 GPU 交回全零单元的条件，与 CPU 读出全零的条件

CPU 一侧（稀疏读者）读出全零的条件：请求按扇区对齐、`offset + length` 不越过盘尾（「match offset.0.checked_add(length_in_bytes) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/src/memory_pool.rs:132`），越过交 None），范围里没有任何写过的扇区（基线写过的扇区加这条流枚举写表里每次写的范围，没写过的读出全 0，「没写过的读出全 0；屏障什么都不做（内存里没有缓存）。」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/src/memory_pool.rs:147`））。表这一侧「碰过的范围」的口径：基线写过的扇区加枚举写表里每次写的范围（「for sector in sparse.written_sectors().keys() {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:462`）），单元区里取与单元区起点的交（「let unit_overlap = range.start.max(unit_area_start)..range.end;」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:485`）），按 16 KiB 槽建落点。
GPU 一侧交回全零单元的条件，原样（`zero_unit_version_over_a_slot_nothing_wrote`，「fn zero_unit_version_over_a_slot_nothing_wrote(before: u32, offset: vec2<u32>, length: u32) -> u32 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:502`））：
- 「let first_slot = (offset.x >> SLOT_SHIFT) | (offset.y << (32u - SLOT_SHIFT));」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:504`）
- 「if (((offset.x & (NODE_BYTES - 1u)) | (offset.y >> (32u - SLOT_SHIFT))) != 0u || first_slot < location_word(before, 9u)」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:505`）
- 「|| location_word(before, 10u) < first_slot + ((length + NODE_BYTES - 1u) >> SLOT_SHIFT)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:506`）
- 「return select(0u, 1u, first_slot >= location_word(before, 11u));」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:509`）
表一侧每个落点之后的那一段（`unit_slots_nothing_wrote_after_each_location`，「pub fn unit_slots_nothing_wrote_after_each_location(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:1121`））：起点是「这个落点之后的头一个整槽」与单元区起点的较大者（「let first_slot = first_slot_after_the_location.max(unit_area_start_slot);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:1145`）），终点是同盘下一个落点所在的槽（不含）或盘尾（「.map_or(device_end_slot, |next| next.offset / slot_bytes())」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:1150`））。落点表在 「locations.sort_by_key(|location| (location.device, location.offset));」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:254`） 按（盘，偏移）排好，GPU 的二分与这个次序一致。

读码对应的一一关系，逐条：
1. 表里的「落点」= 基线或枚举写碰过的 16 KiB 槽（各种非单元落点另算）；一个单元槽落点标 32 KiB 长（盘尾 16 KiB），但它只因为那一个 16 KiB 槽被写过才建，下一槽没人写就是零（「JudgeLocationKind::UnitSlot => location.offset / slot_bytes() + 1,」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:1138`）：缺口从下一槽起算）。两边「盖住」的口径一致。
2. 对齐：GPU 要槽对齐，CPU 只要扇区对齐。指针里的槽号乘 16 KiB 必然槽对齐，所以走读与扫描内核从指针取的读（`slot_to_bytes`）碰不到这个差；journal 记录等 4096 字节读在单元区之前，不进这一支。
3. 盘尾：缺口终点 ≤ 盘尾，`first_slot + n ≤ 终点` 保证整段在盘内；最后一个整 32 KiB 装不下的槽起交 16 KiB 那条（`first_slot >= word 11`），32 KiB 读在盘尾由 `first_slot + 2 > 终点` 拒掉。与 CPU 的 `end <= device_size` 对得上。
4. **槽号的 32 位回绕（读码找到的一支与 CPU 不同，下面 4.2）**：两处 u32 算术没有挡住槽号 ≥ 2³² 或 `first_slot + n` 溢出。

### 4.2 打中：槽号回绕，GPU 交回全零单元而 CPU 读不到（推的，没上卡）

读码：「let first_slot = (offset.x >> SLOT_SHIFT) | (offset.y << (32u - SLOT_SHIFT));」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:504`） 是槽号的低 32 位。注释想挡的是「槽号装不进 32 位」，写成 `offset.y >> (32u - SLOT_SHIFT)`，即 `offset.y >> 18`。槽号是 48 位（高 16 位加低 32 位），偏移 = 槽号 × 2¹⁴，所以 `offset.y` = 槽号高位 × 2¹⁴ + (槽号低位 >> 18)；`offset.y << 18` 把 `offset.y` 的第 14 位及以上丢掉，`first_slot` 实际只留槽号的低 32 位。槽号 ≥ 2³² 当且仅当 `offset.y >> 14 != 0`；代码写的 `offset.y >> 18` 只在槽号 ≥ 2³⁶ 时才拦。所以槽号在 [2³²，2³⁶) 的偏移通过这道检查，`first_slot` 回绕成槽号的低 32 位。另一处：「|| location_word(before, 10u) < first_slot + ((length + NODE_BYTES - 1u) >> SLOT_SHIFT)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:506`）里 `first_slot + n` 是 u32 加法：从槽 2³² − 1 起读 1 槽或 2 槽、从槽 2³² − 2 起读 2 槽，和回绕成 0 或 1，`缺口终点 < 和` 恒假，检查放行；这一支槽号还没到 2³²，不靠上面那道检查的缺陷，也不要求缺口非空（空缺口两个字都是 0，`first_slot < 0` 同样假）。其中从 2³² − 1 起读 16 KiB 一例能走到交回全零单元（版本 1，16 KiB 那条）：32 KiB 读在同一处被「长度大于单元长度」那一句拦住（模型输出里只有 16 KiB 的行）。
指针里槽号是 6 字节（`rd48`：「return vec2<u32>(rd32(base, o), rd16(base, o + 4u));」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:228`）），可以写成 2³² 以上；坏镜像造得出。CPU 对同一个槽：稀疏读者按设备契约越过盘尾交 None（「Some(end) if end <= device_size_in_bytes => Ok(()),」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/src/memory_pool.rs:133`）），那一份读不出、I-2.1 红。
我的探针（CPU 一侧，原样，`probe_of_the_sonnet_leg_for_batch32_zero_slot`；4 GiB 盘，槽 0–262143）：
```
PROBE p4 zero-slot-crc slot190000 red=["I-3.1", "I-3.11", "I-9.23"]
PROBE p4 zero-slot-crc slot262143-last-slot-of-the-disk red=["I-3.1", "I-3.11", "I-9.23"]
PROBE p4 zero-slot-crc slot262144-one-past-the-disk red=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.4", "I-9.23"]
PROBE p4 zero-slot-crc slot2p32+190000 red=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.4", "I-9.23"]
PROBE p4 zero-slot-crc slot2p33+190000 red=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.4", "I-9.23"]
PROBE p4 zero-slot-crc slot2p36+190000 red=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.4", "I-9.23"]
```
造法：墓碑树根第一条条目类型段 1 → 7，第 1 条位置条目的槽写成表头那一列的值、校验和写成全零 16 KiB 的 CRC（与这一批对拍流 F-零槽同一造法）。CPU：槽 190000 与盘上最后一槽（262143）读出全零、I-2.1 成立；槽 262144 与槽 ≥ 2³² 的都读不到、I-2.1 红。
GPU 一侧我用 `wgsl_zero_unit_model.py` 把 `read_at` 与 `zero_unit_version_over_a_slot_nothing_wrote` 逐位移植成 u32 回绕语义（落点表第 9–11 字照 `unit_slots_nothing_wrote_after_each_location`），对「层 0 流的形」（单元只写在单元区开头几槽，别处没写过）原样输出：
```
MODEL zero-unit[small] point slot=4294967295 offset=(4294950912,16383) 16KiB read_at=('ZERO', 1) cpu=UNREADABLE
MODEL zero-unit[small] point slot=4295157296 offset=(3112960000,16384) 16KiB read_at=('ZERO', 0) cpu=UNREADABLE
MODEL zero-unit[small] point slot=8590124592 offset=(3112960000,32768) 16KiB read_at=('ZERO', 0) cpu=UNREADABLE
MODEL zero-unit[small] point slot=68719666736 offset=(3112960000,262144) 16KiB read_at=('NONE',) cpu=UNREADABLE
MODEL zero-unit[small] point slot=190000 offset=(3112960000,0) 16KiB read_at=('ZERO', 0) cpu=ALL_ZERO
```
也就是：槽 2³² − 1、2³² + 190000、2³³ + 190000 上 GPU 交回全零单元（`ZERO`），CPU 读不到；2³⁶ + 190000 被 `offset.y >> 18` 那一道挡住（NONE）。在槽号落在最后一个落点之后那一段（层 0 流里单元都写在低槽）的盘上，2³² + n（n 在缺口里）原样回绕；换成单元写得靠盘尾的「宽」表（模型里 `wide`），回绕的 n 落不进缺口，只剩 2³² − 1 与 2³⁶ − 1 两个溢出点（输出里「越界误认 盘 0 长度 16384：槽 [4294967295, 68719476735]」）。
后果（读码）：位置条目指着槽 2³² + 190000、校验和是全零 16 KiB 的 CRC 的那一份，CPU 读不出（I-2.1 红）而 GPU 读出全零、I-2.1 成立，再按类标签 0 往下判：与 3.1 的 F-零槽对拍流同一条造法，只把槽号加 2³²，GPU 会少红 I-2.1、I-4.8、I-7.4，CPU 与 GPU 判得不一样（读码，没上卡）。
这一支不是合法状态：槽号 ≥ 2³² 的指针只会出现在坏镜像里，不会出现在合法崩溃状态里，所以「合法流与现有对拍流结论不变」那句仍然成立（只有坏镜像才碰）；但它是一条 CPU 与 GPU 判得不一样的坏镜像。
推翻它的现象：GPU 上用那条流的造法把槽号加 2³²，GPU 的 I-2.1 仍然红（说明有别处挡住了，我没读到）；或 `read_at` 的调用方在 `slot_to_bytes` 之前就把槽号限在盘内（`grep -h 'read_at(' crash_judge_*.wgsl | grep -v 'fn read_at' | wc -l` 数出 27 处调用，其中 20 处是 `slot_to_bytes(指针里读出的 48 位槽号)`（`grep -c 'slot_to_bytes('` 同一管道），都直接交，没有一处先把槽号限在盘内）。
改法（读 `crates/` 今天的实现）：在 `zero_unit_version_over_a_slot_nothing_wrote` 里把 `offset.y >> (32u - SLOT_SHIFT)` 换成 `offset.y >> SLOT_SHIFT`（挡槽号 ≥ 2³²），再把 `location_word(before, 10u) < first_slot + n` 写成不溢出的 `location_word(before, 10u) - first_slot < n`（先有 `first_slot < location_word(before, 10u)` 才减）；加一条 2³² + 190000 的对拍流与两行变异（各改回去一处）。这是改法，不是我的判决。

### 4.3 实现员列的「改了之后还留着的分叉」两形对不对

实现员报告原样两形：「读 32 KiB 时第一槽没写过、第二槽有落点（CPU 读出前半全零、后半是那个落点的字节，GPU 照旧读不到）；单元区之前没写过的 journal 环槽、根环之外的槽（CPU 读零，GPU 读不到；派发只要单元区）」。
模型输出（`MODEL zero-unit[small] fork-1 / fork-2`，原样）：
```
MODEL zero-unit[small] fork-1 槽 1092（没写过，下一槽 1093 是落点）读 16384：GPU=('ZERO', 0) CPU=ALL_ZERO
MODEL zero-unit[small] fork-1 槽 1092（没写过，下一槽 1093 是落点）读 32768：GPU=('NONE',) CPU=OVERLAPS_A_LOCATION
MODEL zero-unit[small] fork-2 单元区之前的槽 1 读 16384：GPU=('NONE',) CPU=ALL_ZERO
```
两形都对：第一形由 `first_slot + 2 > 缺口终点` 拒掉；第二形由落点第 9 字 ≥ 单元区起点拒掉。除这两形外，读码还有第三形（4.2），以及一个不影响结论的：缺口里不按槽对齐的读 GPU 一律读不到，CPU 能读（扇区对齐就行），指针碰不到。模型对全部 (槽, 长度) 的统计，原样：
```
MODEL zero-unit[small] CPU 读出全零而 GPU 读不到（单元区之前）：4340 个 (盘, 槽, 长度)
MODEL zero-unit[small] GPU 交回全零单元而 CPU UNREADABLE：14 个 (盘, 槽, 长度)
MODEL zero-unit[small] 一致：都是全零：1044202 个 (盘, 槽, 长度)
```
（`GPU 交回全零单元而 CPU 不是全零` 只出现 `UNREADABLE` 一类，不出现「盖住落点的内容」一类：缺口内读出的全零与 CPU 一致。）

### 4.4 表里两条全零单元的检查字算得对不对

构造：「let zero_unit_lengths: [u64; ZERO_UNIT_VERSIONS] = [DATA_UNIT_BYTES, NODE_BYTES];」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:1280`），每条调的是与每一个真落点版本同一个函数 `version_checks(JudgeLocationKind::UnitSlot, 字节)`（「let checks = version_checks(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:1282`）），所以 `[前 16 KiB 的 CRC, 前 32 KiB 的 CRC（只有 16 KiB 时 0）, 标志位, 单元布局检查]` 与一个内容全零的真落点完全相同，CPU 读出全零之后各个读者的判法对它们不变。版本表字段次序与别的版本一致（「versions.extend([0, 0, checks[0], checks[1], checks[2], checks[3], NONE, 0]);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:1286`）：组合 0、字节起点 0、四个检查字、等值掩码起点 NONE、组合高字 0），字节表最前面放 32 KiB 零字（「version_bytes.extend(bytes_to_words(&zero_unit));」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:1288`）），真落点的字节起点之后在它后面接（「index(version_bytes.len()),」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:1311`）），落点的版本起点已经把这两条算进去（`index(versions.len() / VERSION_WORDS)` 在零单元推入之后取）。等值掩码起点 NONE 这一字只有恢复内核读（`grep -n 'T_VERSIONS' crash_judge_*.wgsl`：读第 6 字的两处都在 `crash_judge_recovery.wgsl`），它读的是写所在落点的版本（`version_of_location`），读不到零单元（读码：「let equal_first = tab(T_VERSIONS, version * VERSION_WORDS + 6u);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:138`） 的 `version` 来自写所在的落点）。
我另核全零 16 / 32 KiB 的 CRC 就是 CPU 读者用的那个函数：探针 `probe_of_the_sonnet_leg_for_batch32_crc` 打 core 的 `crc32_castagnoli`，与 Python 独立实现逐位比（探针打 `PROBE crc zeros16384=94640b85 zeros32768=bc43baad`，`crc32c_check.py` 打 `MODEL crc zeros16384=94640b85 zeros32768=bc43baad`，两边逐位相同），F-零槽那条流 CPU 一侧槽 190000 的 I-2.1 成立（上面第一行）也说明 CPU 读者比的是同一个值。
`JUDGE_TABLES_VERSION` 3 → 4（「pub const JUDGE_TABLES_VERSION: u32 = 4;」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:41`））：表的字布局变了（版本表前两条、落点第 9–11 字），抬版本是对的。表这一侧没有 Rust 单元测试直接测 `unit_slots_nothing_wrote_after_each_location` 与零单元那两条（`grep -rn` 只在 `crash_judge_tables.rs` 本身出现，没有任何测试文件点名），它们只靠 GPU 对拍那一条流在卡上跑到。

**判定**：GPU 交回全零的条件与 CPU 的不是一一对应：GPU 是 CPU 的子集（槽对齐、单元区、整段落在一个缺口里），缺口里读出的全零与 CPU 一致，实现员列的两形对；另外有第三形（槽号回绕，GPU 比 CPU 宽，4.2），那一形我标「打中」，要上卡坐实。检查字：与真落点同一个函数，没有发现算错。推翻（检查字）的现象：表里零单元的检查字与同一个函数在 32 KiB 全零字节上现算的不同（我没有跑 checker-tier，复核不了，只读码）。

## 五、K1 / P5–P7　照字面核

### P5　扫描内核说明

判决那一行（m3-batch29-code-r1 K1 L2）的结论：「两句**留作兜底**：它们在攻方那一形上走得到」（`research/prompts/m3-batch29-code-r1-main-verification.md:25`）。
改后的说明（`crash_judge_checker_scan.wgsl`）：「走满先记它自己的判不了（站点号 61），扫描内核不跑；这里两处落在候选根的链途经走读内核已走过的片、链长过上限的那一形（走读内核走到已走过的片」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:12`）。
逐句核：
- 「走读内核先把根环里每条自证过的根的链按同一个上限走一遍、走满先记站点 61」：走读内核的链长常量就是 61 号站点（「const UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (61u << 8u);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1608`））。
- 「走读内核走到已走过的片就停、不记 61」：「if (!w_visit(device_read_from, slot_read_from)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1643`）后接 `w_merge_below` 与 `break`，没有记判不了。
- 「这里两处」就是 37 / 38 两个站点常量（「const UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (37u << 8u);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:17`）、「const UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (38u << 8u);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:20`）），与判决「它们在攻方那一形上走得到」同一件事。
判决里「走读内核阶段循环每条根都走自己的链」（候选集外的根各自走完整条链、走满记 61）与说明里「根环里每条自证过的根」一致。改前的说明（`research/prompts/_m3-batch32-code-r1-diff.md` 里被替换的那两行）说「走读内核哪天不再走某一类根时的接手（读码推的，没有一条流落到这里）」，与判决「正推与攻方都对，主 agent 正文第二节那一句错」冲突，改后消除。
**判定**：一致。推翻它的现象：走读内核有一类根的链不按上限走（例如根环里自证不过的根）而说明说「每条自证过的根」。

### P6　钉值

钉值原样（`crash_judge_dispatch.rs`）：「("recovery", recovery_kernel_source(), 319_127, 80_873),」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1347`）、「380_969,」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1351`）、「329_198,」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1357`）。每一对「表达式数 + 余量」都等于预算 「pub const LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL: u64 = 400_000;」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:246`）（`python3 -c 'print(319127+80873, 380969+19031, 329198+70802)'` 原样输出 `400000 400000 400000`）。注释里的 naga 版本 `=30.0.1` 与 `Cargo.lock` 一致（`name = "naga"` 下一行 `version = "30.0.1"`，第 452–453 行）。
这三个数是展开后的表达式数，要编译 checker-tier 的内核才能量（本轮不跑 checker-tier 的编译），我复核不了；读码：三个内核都经 `read_at` 的新分支多了东西，走读内核另多了 F-KEY 与 F-ID-b 的调用点，增量 +2 415、+2 737、+5 439 的次序（走读内核最大）合理。
一条读码提醒（不是缺陷）：预算注释写「已知编得过的最大那个的一倍半左右…编不编得过的真实边界在 27 万与 98 万之间，没量过」，扫描内核今天 380 969、余量 19 031，约比「已知编得过」的 27 万多 41%；每次再长都要先量。
**判定**：算术一致（和为预算）；数本身复核不了（要编 checker-tier）。

### P7　对拍文件

删掉的两样与替换：
- 设了额度变量就判红的那一支与辅助函数 `the_card_pool_or_report_not_run`：全文已没有它的引用（`grep -n 'the_card_pool_or_report_not_run'` 零命中），41 处调用换成 `gpu_card_pool_or_report_not_run`（diff 里 `-…the_card_pool_or_report_not_run(` 42 行 = 41 处调用加 1 处定义；`+…gpu_card_pool_or_report_not_run(` 41 行；文件里这个名字出现 43 次 = 41 处调用加 import 加文件头的说明）。
- 常量改引：「usable_cards_or_report_not_run, GpuCardPool, CARD_QUOTA_ENVIRONMENT_VARIABLE,」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:113`）；定义在 「pub const CARD_QUOTA_ENVIRONMENT_VARIABLE: &str =」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:954`），值就是被删的那个字符串（`"SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES"`），并且 `pub use` 导出（「GpuCardPool, CARD_QUOTA_ENVIRONMENT_VARIABLE,」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:923`））。
- 变量仍有两个读法：卡池里读（`open_the_card_pool`，「process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1279`），认不出就 panic），量速度那一条用例里读（「process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:215`）），辅助函数 `card_quota_mebibytes_from_the_value_of_the_variable` 仍被用（「let quota = card_quota_mebibytes_from_the_value_of_the_variable(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:214`）），没有变成死代码。
- 文件头那半句：「显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（量速度那条用例当那张卡的额度；卡池里的读法在 `…CARD_QUOTA_ENVIRONMENT_VARIABLE`）」；卡池一侧的说明是「每张卡账的上限（MiB）…不是每条用例的上限」（「不是每条用例的上限，借得比它多的照常判「放不下」。」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:953`））；两处说的是同一件事。
没编译（checker-tier 的编译不归我）：能读到的只有名字对得上、没有悬空引用；编译过不过归提交时主 agent。
**判定**：一致（字面）。推翻它的现象：对拍文件编译出悬空名字或「未使用」警告（我读码没找到）。

## 六、K3　被判的 9 份文件，每份一格

| 文件（冻结副本） | 这一轮在它上面判到的格 | 判定 |
|---|---|---|
| `crates/singlefs-checker/src/walk.rs` | P1 三处键与共用方法（1.1）；P1 没改的四把键（1.2，探针 1.3）；P2 新函数与回落读法（二）；F-KEY 与 F-ID-b 各变异行（三） | 一致，附一处冲突：四把没改的键与「走过」按两个落点认同一件事（1.2）；字段文档过时（1.5） |
| `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | P1 三处调用与标志（1.1）；P2 新函数与分支（2.1）；四把没改的键（1.2）；变异行 3084–3088 的锚点与点名测试 | 与 CPU 一致；诞生代号「第 0 条没走过」一形与 CPU 不同（1.2 第 3 点，推的） |
| `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` | P4 `read_at` 与 `zero_unit_version_over_a_slot_nothing_wrote`（四） | **冲突**：槽号回绕，GPU 比 CPU 宽（4.2，推的） |
| `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` | P5 说明（五） | 一致 |
| `crates/singlefs-checker-tier/src/crash_judge_tables.rs` | P4 落点第 9–11 字与两条全零单元（4.1、4.4）；`JUDGE_TABLES_VERSION` | 一致，附：没有 Rust 单元测试直接测这两处，只有 GPU 流 |
| `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` | P6 钉值（五） | 算术一致（和为 400 000）；展开后的表达式数复核不了 |
| `crates/singlefs-harness/tests/checker_known_bad_images.rs` | P3 14 条新测试、G4 断言、基线 30 条过、变异红集（三） | 一致，附：G4 钉不住 F-W 那一读；extent 与 inode 内部孩子那两处没有坏镜像；I-4.9 没钉 |
| `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | P3 三条对拍流；P7 辅助函数与常量（五） | P7 字面一致；三条流读码一致（没上卡）；F-零槽流没有 16 KiB 条与回绕的流 |
| `crates/mutations.tsv` | 第 3078–3089 行十二行（3.4） | 一致：锚点各命中一次、点名的测试筛得到；CPU 六行全红且只红点名那几条；GPU 六行只核了静态 |

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 P1 三处「走过」的键 CPU 与 GPU | 一致 | 都按读回那一份的落点认；GPU 没有 inode 内部孩子那一处 |
| K1 P1 没改的四把键（父单元、诞生代号、合并、叶序） | 冲突 | 该跟着改，但成套跟：探针上 F1 只改容器那一处，谱系 loc0 的 I-4.9 误红消失、I-3.5 变红（`i49.txt`） |
| K1 P1 合法状态多红 / 漏判 | 一致 | 键只在第 0 条那份对不上时才变，那时 I-2.1 已红（读码，层 0 没跑） |
| K1 P2 同一判据 | 一致 | CPU 与 GPU、与 F-W 认别类都是「登记的类标签加按那一类长度的整单元校验和对得上」 |
| K1 P2 两份副本两两扫 | 一致 | 72 格与模型逐格相同；网格对称（不随盘序翻面）；回落那 12 格是第 33 批要改的 |
| K1 P3 期望写全 | 一致 | 14 条新测试与 3 条对拍流的断言对得上条款 |
| K1 P3 S2、S4 | 一致 | 各红且只红点名那一条 |
| K1 P3 G4 | 冲突 | 钉不住 F-W 往下走时那一份按同一长度读的那一读（E1 存活） |
| K1 P3 F-KEY 的 extent、inode 内部孩子两处；F-ID-b 回落读法 | 规则没说 / 没钉 | E2、E4、E6、E7 存活，没有测试钉 |
| K1 P3 变异行筛得到 | 一致 | 12 行锚点各命中一次、点名的测试筛得到 |
| K1 P4 条件一一对应 | 冲突 | GPU 是 CPU 的子集加一支回绕：槽 2³² − 1 与 [2³²，2³⁶) 里落在缺口内的槽，GPU 交回全零而 CPU 读不到（推的，没上卡） |
| K1 P4 检查字 | 一致 | 与真落点同一个函数；全零 16 / 32 KiB 的 CRC 与 Python 独立实现逐位相同；表这一侧无单元测试 |
| K1 P4 实现员列的两形 | 一致 | 两形都对；多出第三形（回绕） |
| K1 P5 | 一致 | 说明与判决 L2 那一行一致 |
| K1 P6 | 复核不了 | 三对数之和都是 400 000，展开后的表达式数要编 checker-tier |
| K1 P7 | 一致 | 41 处调用换名、常量改引、没有悬空引用；编译归提交时 |
| K3 九份文件 | 见第六节 | 每份一格 |

## 八、要主 agent 跑的命令

1. 4.2 的回绕（要上卡，归提交时的 GPU 对拍）：在 F-零槽那条流（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 里 `record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_a_slot_nothing_wrote`）另起一条流：只把第 1 条位置条目里写进去的槽号加 2³²，`assert_nothing_in_the_flow_wrote_the_slot` 那一句仍核 190000（槽号加了 2³² 之后基镜像读不到，那一句会 panic）；再起一条槽 2³² − 1 的。期望：我的读码对，就有 GPU 与 CPU 不一致（GPU 少红 I-2.1、I-4.8、I-7.4）。用 `--features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 经 `gpu-quiesce-run.sh` 跑（归主 agent，我不跑）。
2. P6 钉值：`crash_judge_dispatch::tests::each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget`（快档库单测，归提交时）。
3. 变异表 GPU 六行（3084–3089）红不红：归提交时的 checker-tier-crates-mutation-replay。
4. 我的复跑：`SINGLEFS_REPO=<冻结副本> bash research/prompts/m3-batch32-code-r1-sonnet-model/rerun.sh <空工作目录> all`（harness 档，编译 6G、跑 4G，线程 4；空目录从头 `all` 我这边 9 分 9 秒（load average 20–30），负载高时要分几次调用，每个阶段能单独重跑，见脚本头；跑完 `compare.sh <工作目录>` 的输出应与模型目录的 `expected-compare.txt` 逐行相同）；`compare.sh <工作目录>` 整理。

## 九、没做什么

- 没编译、没跑 checker-tier（`singlefs-checker-tier` 的库、测试与 GPU）；P6 的表达式数、GPU 六行变异、三条新对拍流、P7 的编译都没验。
- 没跑层 0 与崩溃枚举（checker 档）：1.4 的「合法状态不多红」是读码推的。
- 没替攻方做 K2：I-4.9 一形的改法候选只给了 F1 一处的实测，没有扫 loc0 / loc1 之外的形，没有扫 GPU 三个内核各调用点与 CPU 的逐一对；P2 的网格只在层级 1 的根上，层级 ≥ 2 没扫。
- 没给判决，没采纳或否决任何格。
- 单次抽样：这一轮我只抽了一次；「没打中」的几格（P1 合法状态、P2 网格）按 `three-way-inference.md` 还差一次同立场抽样才记「没打中」。
- 草稿与产物在仓外，没有入库（探针输出、变异原始输出、两次复跑的汇总都是中间证据，不是实验产物）：`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1-sonnet/run1/out/`（我的第一次跑，含 `i49.txt`、`probe.txt`、`mutations.txt`、`mutations-broad.txt`、`extra.txt` 与各条的 `.raw`）、`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1-sonnet/run2/out/`（空目录从头复跑）；两次的汇总 `compare-run1.txt`、`compare-run2.txt` 同目录上一层。编译目录 `run1/target`、`run2/target` 与仓副本 `run1/tree`、`run2/tree` 我交回前删了（复跑用模型目录的 `rerun.sh` 从头来，9 分钟）。
