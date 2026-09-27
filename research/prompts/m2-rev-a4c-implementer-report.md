# 实审 A4c 报告：准入「按最坏情况计」——每块盘改几片叶、用户数据落得下的槽、压小容量传进挂载准入

时刻：2026-09-26 23:29 – 2026-09-27 00:3x UTC（JST 2026-09-27 08:29 – 09:3x）。规格 `/tmp/claude-1000/impl-rev-a4c/spec.md`。
副本 `/tmp/claude-1000/impl-rev-a4c/copy`（23:29:15Z 取，那一刻在改文件的 sha256 在 `sha256-at-copy.txt`）做改动与证红；探针在另一份副本 `copy-probe` 里（只在副本里的探针函数与用例，源码存 `probes/`）。
往主工作区搬之前，我那 4 份文件在主工作区里与取副本时逐字节相同（`sha256-main-now.txt`，00:00 UTC 核），改动照副本搬进主工作区，搬完逐份 `cmp` 相同。

## 一、结论

1. **第 1 件（一块盘一次空发布改几片叶）：三种情形在今天的代码上都走得到，都量到过一块盘改 3–4 片叶；「每块盘两条叶路径」那一项不是上界，已经量到少扣。**
   4 GiB 两盘、产品容量、随机历史里一次空发布每块盘实写 14 块、之前现算的 ckpt_cost 12（少扣 2）；单元区 2600 槽的盘上实写 12、ckpt_cost 8（少扣 4，那一次整棵分配记录树 4 片叶全改）。
   **停下，式子没改**：上界的不动点不收敛（每多改一片叶，就多写、多换下 D × (高 − 1) 个节点，系数 ≥ 2），能拿准的闭式上界只有「几何上整棵分配记录树」（候选 K1），代价是 4 GiB 每次多扣约 527 块、1 TiB 约 16.7 万块；
   便宜的 K3（这一版现有节点 + 每块盘两条新路径）在量到的全部 3201 次空发布上都够，但证不了（反例的构造在第二节）。候选与量表各格多扣几块在第二节，交主 agent 弹窗问用户。
2. **第 2 件（C545）：今天的代码上造出了那一格，钉成一条用例；「改准入、转成准入先拒」卡在 transaction.rs，停下。**
   单元区 384 槽，崩了再挂，这次挂载里 80 次空发布、把文件顺序写到 6 个单元；写第 7 个单元时每块盘段外成对空槽 0 对、段内 79 对，按字节算的式子每块盘可用 49 槽（需求上界 33 槽），数据单元 0 落点被拒、两块盘逐字节不变。
   卡在哪：发布路径 `prepare_the_version_publish` 先在分配器拷贝上把这次的全部角色（含数据单元）取一遍落点（`settle_the_allocation_record_tree`），取不到就在准入之前交回 `PlacementRefused`，准入排在它后面。
   量到：256 槽那一格式子可用 −14 槽（式子会拒），交回的照样是 `PlacementRefused`。准入一侧加什么判据都轮不到它先判，要动的是 C11b 在改的 `transaction.rs` 的次序；「落得下」还要定算哪几项（第三节）。
3. **第 3 件**：`mount.rs` 两处改成把写入口装的节点容量传进按容量算的读数（新函数 `admission_reading_of_a_writable_mount_with_node_capacities`；第二处经 `push_floor_raises_after_the_row_publish` 多一个参数带过去）。
   **这一改今天等价**：挂载那一路的写入口都是 `PoolWriter::new` 新开的、恒为节点格式容量，全仓没有给挂载装压小容量的入口，造不出红，也就没有「改回去它就红」的变异（加进表里是一条活变异）。真正读不到压小容量的是 `transaction.rs` 那一处，改法在第四节。
4. **另一件**：崩溃枚举那份照今天的代码改钉：会话覆盖写 61 次、第 62 次直接发布被落点拒（探针：第 18、32、47 次式子先拒，第 62、77 次落点被拒，与 A4b 第五节同）；推的那一串 3 次空发布、53 次写，状态数 77 850 ≤ 10⁶，推过之后覆盖写做成。它标了 ignore，全量枚举没跑，留给提交时。
5. 变异：新加 1 行（C545 那一格的用例），副本上证红；主表没改，行在 `mutations-append.tsv`。

**什么现象会推翻它们**：
- 第 1 条：有人证明（不是量）今天的分配器下一次空发布每块盘至多改两片叶——那样第二节那几次量到的 3、4 片叶就该复现不出来；按第二节的复现命令跑不出多于两片叶的，第 1 条的「走得到」不成立。
- 第 2 条：`transaction.rs` 里准入挪到取落点之前（C11b 若动了这一段），第 2 条「卡在次序」就不再成立，要重看。
- 第 3 条：有一条路径能给可写挂载的写入口装上压小容量（`grep -rn set_code_two_tree_node_capacities crates/singlefs-core/src` 出现 `mount.rs`），那时这一改不再等价、要补红。
- 第 4 条：主工作区合进别的会话的改动之后，第 62 次直接发布不再是 `PlacementRefused`（例如第 2 件的次序改了之后会变成 `SpaceAdmissionRefused`）。

## 二、第 1 件：一块盘一次空发布改几片叶——量到的，与候选式子

### 怎么量的

探针 `probes/probe_a4c_leaves.rs`（在 `copy-probe` 里跑，release，经内存包装，`capped.sh 5`）：两块稀疏盘、mkfs → 取号 → 暖机 → 第一个文件，之后按种子抽一段随机历史：
每步 55% 直接发一次空发布（`publish_version`，`file: None`）、15% 覆盖写小文件、20% 顺序写到 1..N 个数据单元、10% 崩了再挂（`PROBE_FILL=1` 时把覆盖写那 15% 也换成顺序写，逼近写满）。
量到的每一次空发布都算（直接发的、崩了再挂之后的暖机、会话与挂载推的抬 F 空发布）：每块盘的叶 = 这次 `rewritten` 里 `AllocationTreeNodeBelowTheRoot` 层级 0 的个数；实写 = 这次全部固定点单元的槽数；
ckpt_cost = 这次之前那一版按今天的式子现算（`checkpoint_cost_of_the_version_to_build_on`）；另记新落点、换下的落点各在哪几片叶，换下的落点不是上一次新写的那几个（「旧的」）时，按上一版的单元表找出它是哪个角色。

命令（`copy-probe` 里，`PROBE_*` 是探针读的环境变量）：

```text
PROBE_SEEDS=1,2,5,7 PROBE_STEPS=300 PROBE_MAX_UNITS=900 cargo test --offline --release -p singlefs-harness --test probe_a4c_leaves -- --nocapture
PROBE_FILL=1 PROBE_UNIT_AREA=2600 PROBE_SEEDS=5,7,9,11,13,15,17,19 PROBE_STEPS=400 PROBE_MAX_UNITS=200 cargo test ...（同上）
PROBE_UNIT_AREA=2600 PROBE_SEEDS=5,7,9,11 PROBE_STEPS=300 PROBE_MAX_UNITS=300 cargo test ...（同上，不逼满；下表情形 ① 2600 槽那一行出自这一次，那时探针还没算候选式子）
```

每段历史一行汇总（原样；`leaves_hist` 是「一块盘改的叶数最多那块盘的叶数 → 次数」，`under_reserved` 是实写 > ckpt_cost 的次数，`over_k*` 是第四小节候选式子的多扣区间）：

```text
TALLY ua0-s1 empty=222 leaves_hist={1: 201, 2: 18, 3: 3} over_two=3 under_reserved=0 max_diff=0 with_fallback=0 fallback_max_leaves=0 over_k0=[0,15] over_k1=[525,538] over_k3=[8,41] max_new_nodes=2
TALLY ua0-s2 empty=201 leaves_hist={1: 179, 2: 18, 3: 2, 4: 2} over_two=4 under_reserved=2 max_diff=2 with_fallback=0 fallback_max_leaves=0 over_k0=[-2,15] over_k1=[523,538] over_k3=[12,41] max_new_nodes=0
TALLY ua0-s5 empty=200 leaves_hist={1: 168, 2: 28, 3: 2, 4: 2} over_two=4 under_reserved=1 max_diff=2 with_fallback=0 fallback_max_leaves=0 over_k0=[-2,15] over_k1=[523,538] over_k3=[6,47] max_new_nodes=2
TALLY ua0-s7 empty=212 leaves_hist={1: 184, 2: 25, 3: 3} over_two=3 under_reserved=0 max_diff=0 with_fallback=0 fallback_max_leaves=0 over_k0=[0,15] over_k1=[525,538] over_k3=[8,47] max_new_nodes=2
TALLY ua2600-s5 empty=285 leaves_hist={1: 257, 2: 24, 3: 4} over_two=4 under_reserved=4 max_diff=2 with_fallback=3 fallback_max_leaves=2 over_k0=[-2,2] over_k1=[2,6] over_k3=[2,10] max_new_nodes=2
TALLY ua2600-s7 empty=313 leaves_hist={1: 287, 2: 24, 3: 2} over_two=2 under_reserved=2 max_diff=2 with_fallback=4 fallback_max_leaves=3 over_k0=[-2,2] over_k1=[2,6] over_k3=[4,10] max_new_nodes=0
TALLY ua2600-s9 empty=296 leaves_hist={1: 270, 2: 22, 3: 4} over_two=4 under_reserved=4 max_diff=2 with_fallback=3 fallback_max_leaves=3 over_k0=[-2,2] over_k1=[2,6] over_k3=[4,10] max_new_nodes=0
TALLY ua2600-s11 empty=285 leaves_hist={1: 267, 2: 17, 3: 1} over_two=1 under_reserved=1 max_diff=2 with_fallback=4 fallback_max_leaves=1 over_k0=[-2,2] over_k1=[2,6] over_k3=[6,10] max_new_nodes=0
TALLY ua2600-s13 empty=287 leaves_hist={1: 260, 2: 21, 3: 3, 4: 3} over_two=6 under_reserved=6 max_diff=4 with_fallback=5 fallback_max_leaves=4 over_k0=[-4,2] over_k1=[0,6] over_k3=[4,10] max_new_nodes=0
TALLY ua2600-s15 empty=302 leaves_hist={1: 276, 2: 22, 3: 4} over_two=4 under_reserved=4 max_diff=2 with_fallback=7 fallback_max_leaves=2 over_k0=[-2,2] over_k1=[2,6] over_k3=[4,10] max_new_nodes=0
TALLY ua2600-s17 empty=309 leaves_hist={1: 281, 2: 26, 3: 2} over_two=2 under_reserved=2 max_diff=2 with_fallback=3 fallback_max_leaves=1 over_k0=[-2,2] over_k1=[2,6] over_k3=[4,10] max_new_nodes=0
TALLY ua2600-s19 empty=289 leaves_hist={1: 267, 2: 20, 3: 2} over_two=2 under_reserved=2 max_diff=2 with_fallback=0 fallback_max_leaves=0 over_k0=[-2,2] over_k1=[2,6] over_k3=[4,10] max_new_nodes=0
TALLY ua2600-s5 empty=209 leaves_hist={1: 199, 2: 10} over_two=0 under_reserved=0 max_diff=0
TALLY ua2600-s7 empty=227 leaves_hist={1: 216, 2: 11} over_two=0 under_reserved=0 max_diff=0
TALLY ua2600-s9 empty=232 leaves_hist={1: 209, 2: 20, 3: 3} over_two=3 under_reserved=3 max_diff=2
TALLY ua2600-s11 empty=205 leaves_hist={1: 187, 2: 17, 3: 1} over_two=1 under_reserved=1 max_diff=2
```

前 12 行出自前两条命令（`probe-4gib-candidates.log`、`probe-2600-candidates.log`），末 4 行出自第三条（`probe-2.log`，不逼满，没有候选列）。
合计：4 GiB 835 次空发布，3 片叶 10 次、4 片叶 4 次、少扣 3 次（最多 2 块）；2600 槽逼满 2366 次，3 片叶 22 次、4 片叶 3 次、少扣 25 次（最多 4 块），走到回落的 29 次（只数落在聚簇段外的回落）；2600 槽不逼满 873 次，3 片叶 4 次、少扣 4 次（最多 2 块）。

### 三种情形各一段历史（「这一项算了几片」都是每块盘 2 片，分配记录树那一项 = 盘数 × 2 × (高 − 1) + 1）

| 情形 | 哪一段历史（复现） | 那一次 | 每块盘实改的叶 | 实写 / ckpt_cost | 怎么多出来的 |
|---|---|---|---|---|---|
| ① 开放段装不下，开下一个全空段、那一段在别的叶 | 2600 槽，不逼满，种子 11 第 73 步 | txg 85，直接发的空发布 | 61、62、63（3 片） | 10 / 8（少扣 2） | 新落点先在开放段尾 51517–51519（叶 63），段满了开最低全空段 50176（叶 61）接着放 50176–50182；换下的旧落点（不是上一次新写的）在叶 62、63 |
| ① 同上（4 GiB） | 4 GiB，种子 1 第 184 步 | txg 216 | 61、62、68（3 片） | 12 / 12（没少扣，别的项多扣吃掉了） | 开放段 50304 尾 50365–50367（叶 62），段满开 55744（叶 68）；换下的旧落点（不是上一次新写的）在叶 61 |
| ② 没有全空段，回落到最低空槽、一个单元一处 | 2600 槽，逼满，种子 13 第 154 步 | txg 181，会话推的抬 F 空发布 | 61、62、63、64（4 片，整棵树） | 12 / 8（少扣 4） | 新落点 52671、50261、50279、50301、50494、51074、51076、51078–51081、51107 散在 4 个段里（回落取最低空槽）；换下的旧节点（盘 0 那一份；盘 1 那一份在相邻槽）：叶 61 的节点在 51793（叶 63）、叶 62 / 63 的节点在 52649 / 52650（叶 64） |
| ② 同上（暖机） | 同一段历史第 22 步 | txg 28，崩了再挂之后的暖机 | 61–64（4 片） | 12 / 8（少扣 4） | 挂载之后聚簇段集合是空的、没有全空段：暖机的单元回落到 50176、50177、50245–50250、52648–52651 |
| ③ 换下的上一版节点很久以前落在别的叶 | 4 GiB，种子 2 第 132 步 | txg 150，直接发的空发布 | 61、62、67、69（4 片） | 14 / 12（少扣 2） | bump 游标在段 50304 里跨过叶 61 的末槽（新落点 50343–50356）；叶 62 的节点（两块盘各一个）上一次写在 55003 / 55008（叶 67）、叶 67 的节点在 56645 / 56652（叶 69）——换下叶 62 的节点弄脏叶 67，叶 67 的节点重写又弄脏叶 69，一条链 |

那几次的原样行在 `probe-roles-4gib.log`、`probe-roles-2600.log`、`probe-roles-2600-m1.log`、`probe-2.log`、`probe-1.log`（`grep '^OVER\|^RELEASED-OLD'`）。

**走不到的**：单元区 240 / 256 / 384 槽的三档窄盘（`HistoryDeviceWidth`）走不到多于两片叶——单元区从槽 50176 起、只有 240–384 槽，落在叶 61（到 50343）与叶 62 两片里，每块盘一共就两片叶。窄盘上今天的式子与候选 K1 同值（5）。

### 为什么不动点收不了口

一次空发布改的叶 ⊆ 新落点所在的叶 ∪ 换下的落点所在的叶。改一片叶，它的叶节点与祖先（每块盘 高 − 1 个）都要重写：每个重写的节点既多一个新落点，又换下一个旧落点（旧落点可以在任何一片叶里，情形 ③）；
回落时新落点也可以各在一片叶里（情形 ②）。于是「改的叶数 ≤ 新落点叶数 + 换下落点叶数」这一式里，右边随左边以系数 D × (高 − 1)（两块 4 GiB 盘是 4）涨，迭代不收敛，只停在整棵树的节点数上。
要一个比整棵树小的上界，得用分配器的局部性（没有回落、换下的节点都在最近几段里），而这两样今天的代码都不保证——上表 ② ③ 就是反例。

### 候选式子（分配记录树那一项；中央映射树那一项照今天的式子、按「分配记录树那一项 + 记账树节点数」算删插，所以跟着变）

- **K0（今天的，A4b）**：盘数 × 2 × (高 − 1) + 1。上面量到少扣，不是上界。
- **K1（几何上整棵树，拿得准）**：1 + Σ_盘 Σ_{层级 L = 0 … 根层级 − 1}（这块盘单元区 [s, e) 在层级 L 上罩到的位置数 = ⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1，S_L = 812 × 169^L）。
  一次发布重写的节点都在这一版之后的树里，而位置寻址的树每块盘在单元区里至多这么多个位置，所以恒是上界；不读这一版的状态、不依赖 ckpt_cost，O(盘数 × 树高) 算完。代价随盘容量线性涨（约单元区槽数 ÷ 812）。
- **K3（这一版现有节点 + 每块盘两条新路径，便宜、证不了）**：(这一版分配记录树根之下的节点数) + 盘数 × 2 × (高 − 1) + 1。现有节点数从 `TransactionOutput::allocation_record_tree` 取、O(1)。
  量到的 3201 + 450 次空发布一次都没超（多扣最少 2）；一次空发布新建的节点最多 2 个（`max_new_nodes`）。
  证不了的地方：新建节点只来自落进「没有记录的叶」的新落点，而回落可以把一个个单元落进不同的这种叶——这要求那些叶里没有全空段（例如每段都有被影子账隔离、却不在当前账里的槽，或单元区尾巴不满一段的那几槽），推的，没造出来；一次空发布的固定点多于 64 块、要开两个以上新段时同理。

量表各格（`admission_checkpoint_cost_per_device_paths.rs` 的 8 格，探针版在 `copy-probe` 里给每一行多打候选值，改动在 `probes/measurement-cells-candidates.diff`；多扣 = ckpt_cost − 那一次每块盘实写，区间是那一格全部空发布）：

| 格 | 空发布 | K0 ckpt / 多扣 | K1 ckpt / 多扣 | K3 ckpt / 多扣 | 每块盘被扣（13c + 8 槽，c 取格内最大）K0 → K1 → K3 |
|---|---|---|---|---|---|
| two-1GiB（高 2） | 60 | 8 / 0–2 | 44 / 36–38 | 10–12 / 2–6 | 112 → 580（9 MiB，0.89%）→ 164 |
| two-4GiB（高 3） | 60 | 12 / 2–4 | 537 / 527–529 | 16–18 / 6–10 | 164 → 6989（109 MiB，2.67%）→ 242 |
| two-64GiB | 60 | 12 / 2–4 | 10347 / 10337–10339 | 16–18 / 6–10 | 164 → 134519（2102 MiB，3.21%）→ 242 |
| two-1TiB（高 4） | 30 | 16 / 4–6 | 167310 / 167298–167300 | 22–24 / 10–14 | 216 → 2175038（33985 MiB，3.24%）→ 320 |
| 4 GiB、文件 300 单元、映射树 2 层 | 60 | 16 / 5–7 | 539 / 528–530 | 22–24 / 11–15 | 216 → 7015（110 MiB）→ 320 |
| 4 GiB、映射压到 4-8（只供测试） | 60 | 24–26 / 11–16 | 890–892 / 877–882 | 30–36 / 17–26 | 346 → 11604 → 476 |
| 4 GiB、映射压到 3-3（只供测试） | 60 | 33–41 / 17–29 | 1073–1079 / 1057–1067 | 43–53 / 27–41 | 541 → 14035 → 697 |
| 4 GiB、映射 2-2 记账 8-3（只供测试） | 60 | 171–189 / 141–163 | 144991–145529 / 144961–145503 | 253–324 / 223–298 | 2465 → 1891885（大过盘）→ 4220 |
| 随机历史 4 GiB（第一小节，835 次） | — | 多扣 −2–15 | 多扣 523–538 | 多扣 6–47 | — |
| 随机历史 2600 槽逼满（2366 次） | — | 多扣 −4–2 | 多扣 0–6 | 多扣 2–10 | — |

读法：K1 在 4 GiB 以上每块盘多扣约 2.7–3.2% 的盘（挂载期承诺量按 c_max 乘 12 放大），1 TiB 每块盘约 33 GiB；压小容量那一格里映射树那一项跟着爆，扣的比盘还大。K3 在产品容量格多扣 2–15 块。
2600 槽那格 K1 有一次多扣 0（整棵树 4 片叶全改那一次，实写正好等于 K1）。

K1、K3 都没写进 `admission.rs`：前者代价远超用户定案时看到的量，后者证不了；选哪一个（或改分配器、让一次空发布的落点与换下的节点不散出去）是设计判断，交主 agent 弹窗问用户。
D28（挂载期承诺量） 已定项 4 的新句因此这一轮没有：式子没变。A4b 报告第二节给书记员的那句「射程」里「这一格没定」照旧成立，可以补上「量到过少扣：4 GiB 随机历史少扣 2 块，2600 槽的盘少扣 4 块，三种情形都走得到」。

## 三、第 2 件：C545 那一格——造出来了，「转成准入先拒」卡在 transaction.rs

### 造出来的那一格（探针，不改式子）

探针 `probes/probe_a4c_c545.rs`：两块小盘，第一个文件之后崩了再挂，这次挂载里直接推 N 次空发布，再把文件顺序写得一个单元一个单元地长（直接调 `publish_sequential_write`、不经会话推抬 F），直到第一次被拒；被拒时打出式子的可用（槽）、上一次做成那次的需求（槽）、每块盘成对空槽在这次挂载开过的聚簇段外 / 段内各几对。原样：

```text
C545 width=UnitAreaOf384Slots empties=0 empty_refused=None units=10 ADMISSION short=[(0, 22, 35), (1, 22, 35)] pairs=[(0, 33, 28), (1, 33, 28)]
C545 width=UnitAreaOf384Slots empties=10 empty_refused=None units=8 ADMISSION short=[(0, 18, 31), (1, 18, 31)] pairs=[(0, 52, 8), (1, 52, 8)]
C545 width=UnitAreaOf384Slots empties=20 empty_refused=None units=7 ADMISSION short=[(0, 20, 29), (1, 20, 29)] pairs=[(0, 33, 29), (1, 33, 29)]
C545 width=UnitAreaOf384Slots empties=30 empty_refused=None units=7 ADMISSION short=[(0, 22, 27), (1, 22, 27)] pairs=[(0, 22, 42), (1, 22, 42)]
C545 width=UnitAreaOf384Slots empties=40 empty_refused=None units=7 ADMISSION short=[(0, 24, 29), (1, 24, 29)] pairs=[(0, 13, 51), (1, 13, 51)]
C545 width=UnitAreaOf384Slots empties=50 empty_refused=None units=7 ADMISSION short=[(0, 16, 29), (1, 16, 29)] pairs=[(0, 11, 49), (1, 11, 49)]
C545 width=UnitAreaOf384Slots empties=60 empty_refused=None units=7 ADMISSION short=[(0, 22, 29), (1, 22, 29)] pairs=[(0, 15, 49), (1, 15, 49)]
C545 width=UnitAreaOf384Slots empties=80 empty_refused=None units=6 PLACEMENT unit=Data(DataUnitIndexInFile(0)) refusal=NoFreeSlotOnAnyDevice available=[(0, 49), (1, 49)] demand_of_last=23 pairs=[(0, 0, 79), (1, 0, 79)] clusters={SlotNumber(50304), SlotNumber(50368), SlotNumber(50432), SlotNumber(50496)}
C545 width=UnitAreaOf256Slots empties=0 empty_refused=None units=6 ADMISSION short=[(0, 14, 27), (1, 14, 27)] pairs=[(0, 35, 24), (1, 35, 24)]
C545 width=UnitAreaOf256Slots empties=10 empty_refused=None units=3 ADMISSION short=[(0, 15, 21), (1, 15, 21)] pairs=[(0, 50, 11), (1, 50, 11)]
C545 width=UnitAreaOf256Slots empties=20 empty_refused=None units=1 ADMISSION short=[(0, -13, 15), (1, -13, 15)] pairs=[(0, 48, 0), (1, 48, 0)]
C545 width=UnitAreaOf256Slots empties=30 empty_refused=None units=1 ADMISSION short=[(0, -14, 15), (1, -14, 15)] pairs=[(0, 24, 24), (1, 24, 24)]
C545 width=UnitAreaOf256Slots empties=40 empty_refused=None units=1 PLACEMENT unit=Data(DataUnitIndexInFile(0)) refusal=NoFreeSlotOnAnyDevice available=[(0, -14), (1, -14)] demand_of_last=6 pairs=[(0, 0, 48), (1, 0, 48)] clusters={SlotNumber(50304), SlotNumber(50368)}
C545 width=UnitAreaOf256Slots empties=50 empty_refused=None units=1 ADMISSION short=[(0, -12, 17), (1, -12, 17)] pairs=[(0, 23, 26), (1, 23, 26)]
C545 width=UnitAreaOf256Slots empties=60 empty_refused=None units=1 ADMISSION short=[(0, -16, 15), (1, -16, 15)] pairs=[(0, 31, 16), (1, 31, 16)]
C545 width=UnitAreaOf256Slots empties=80 empty_refused=None units=1 ADMISSION short=[(0, -12, 17), (1, -12, 17)] pairs=[(0, 7, 41), (1, 7, 41)]
```

- 384 槽、80 次空发布那一格就是 C545：长到 6 个单元都做成，第 7 个单元时段外成对空槽 0 对、段内 79 对，式子每块盘可用 49 槽，数据单元 0 落点被拒。
- 256 槽、40 次空发布那一格：式子可用 **−14** 槽（式子会拒），交回的却是 `PlacementRefused`——发布路径先取落点、后判准入的直接证据。

### 钉成的用例（主工作区，已搬）

`crates/singlefs-harness/tests/second_transaction_admission_raises_the_floor_before_refusing.rs` 新加
`a_write_whose_free_slot_pairs_are_all_inside_this_mounts_cluster_segments_is_refused_by_placement_before_any_write_while_the_byte_formula_admits`（测试函数在第 211 行；它用的三个辅助函数在第 102、135、167 行）：
384 槽、崩了再挂、80 次空发布、顺序写到 6 个单元；断言每块盘成对空槽（段外, 段内）=（0, 79）、按字节算的式子每块盘可用 ≥ 需求上界（6 个单元那次新写的槽 + 2 + ckpt_cost）、写第 7 个单元交回 `PlacementRefused { Data(0), NoFreeSlotOnAnyDevice }`、两块盘逐字节不变。
它钉的是今天的样子（C545 欠账「验收」那一栏前半句：准入放行之后分配失败时走拒绝那一条、盘上不变），不是规格要的「准入先拒」。同一份文件里 `an_admitted_overwrite_…` 的文档注释原来写「发布路径先判准入，`PlacementRefused` 只在准入放行之后才报得出来」，与代码的次序相反，我改成照实写（第 62 次那一格数不变）。
规格给的新文件名 `admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs` 说的是改完之后的行为，这一轮做不到，没建。

### 卡在哪（行号现查）

`crates/singlefs-core/src/transaction.rs`（C11b 在改，我没碰）：
- 第 4825 行 `let settled = settle_the_allocation_record_tree(`：在分配器的拷贝上把释放与这次全部角色的取落点走一遍（第 5028 行 `allocate_placement_for_role(&mut rehearsal, *identity, plan.txg)?;`，数据单元排在最前），取不到就带 `?` 交回 `PlacementRefused`；
- 第 4859 行 `&admission_reading_before_a_publish(allocator, previous),`：准入在那之后才判。

所以发布路径上「准入放行、落点取不到」字面上不会发生：凡是走到准入的，拷贝上的落点已经全取到了；取不到的根本走不到准入。准入一侧（`admission.rs`）加一项「用户数据落得下的槽对」，在今天的次序下永远轮不到它拒，改了也是一条证不了红的活变异，所以我没加。

### 要改成「准入先拒」需要的几样（交主 agent；推的，没实现）

1. `transaction.rs`（C11b）：`prepare_the_version_publish` 里在 `settle_the_allocation_record_tree` 之前，先对「有普通分配、准入判着」的发布按这次的数据单元个数（计划里就有）判一次「每块盘段外成对空槽 ≥ 数据单元数」，不够交回 `SpaceAdmissionRefused`（会话照旧推抬 F 再判）。
2. `admission.rs`：读数里每块盘多一项「用户数据落得下的槽对数」、需求里多一项「数据单元个数」，`admit_on_every_device` 逐盘两样都判。
3. `allocator.rs`：给出每块盘段外成对空槽数。现数要扫整个单元区（代价随盘容量涨，`.claude/rules/fs-design.md`「记账是事务的副产品」那一节不许）；按段增量维护「整对都空的槽对数」，读时总数减这次挂载开过的几段，是 O(开过的段数)——隔离、扣住、回收、放开扣住四处都要跟着维护。
4. 还要定的（条款没写）：元数据那一侧落不落得下要不要也进准入（两槽的 inode 叶容器要成对、开新段会把段外的空槽对关给用户数据、各盘要同槽）；抬 F 回收后「扣住」的槽计数上是空闲、分配器不发，式子没扣（`admission.rs` `AdmissionReading::of_allocator` 文档第 311 行那条 ⚠️）。只补数据单元那一项，「放行之后不因空间不够失败」仍不成立。
5. 另一处要主 agent 看的（推的，没量 `df`）：这一格里段内 79 对空槽在这次挂载里对用户数据关着（D3（空间分配） 已定项 8 第 2 条），`df` 若把它们算空闲，这次写报 ENOSPC 就是 D3（空间分配） 已定项 9 第 1 条的假性 ENOSPC；准入先拒只改交回的成员，不改这一点。

## 四、第 3 件：压小容量传进挂载那两处的准入——改了，今天等价

改动（主工作区，行号现查）：
- `crates/singlefs-core/src/admission.rs`：第 840 行 `admission_reading_with_the_switch_reserve_of` 多一个参数 `node_capacities`，ckpt_cost 改调 `checkpoint_cost_of_the_version_to_build_on_with_node_capacities`；
  第 866 行 `admission_reading_before_a_publish` 签名不变（`transaction.rs` 第 4859 行在调），传 `FromTheNodeFormat`；第 881 行 `admission_reading_of_a_writable_mount` 签名不变（`second_transaction_supplement_two_admission_formula.rs` 第 543 行在调，那份不在我的清单里），转调新的第 896 行 `admission_reading_of_a_writable_mount_with_node_capacities`。
- `crates/singlefs-core/src/mount.rs`：第 3043 行（取号之前那一判）传 `pool.code_two_tree_node_capacities()`；第 3298 行取写行与暖机那几次写入口的容量，经 `push_floor_raises_after_the_row_publish` 新参数（第 3386 行）带到第 3394 行（推抬 F 之后再判）。
  那个函数因此 8 个参数，clippy `too_many_arguments` 红，照仓里别处的写法在第 3376 行加 `#[allow(clippy::too_many_arguments, reason = …)]`（函数签名因此挪到第 3380 行）。

**为什么造不出红、没有变异**（命令原样，主工作区）：

```text
$ grep -rn 'set_code_two_tree_node_capacities' crates/singlefs-core/src
crates/singlefs-core/src/transaction.rs:218:    pub fn set_code_two_tree_node_capacities(&mut self, capacities: CodeTwoTreeNodeCapacities) {
$ grep -rn 'PoolWriter::new' crates/singlefs-core/src/mount.rs crates/singlefs-core/src/mounted_session.rs
crates/singlefs-core/src/mounted_session.rs:152:    let mut writer = PoolWriter::new(parameters, devices);
crates/singlefs-core/src/mount.rs:1999:    let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
crates/singlefs-core/src/mount.rs:3009:    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
crates/singlefs-core/src/mount.rs:4238:    let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
```

`PoolWriter::new` 起步恒是 `FromTheNodeFormat`（`transaction.rs` 第 205 行），挂载与会话那几处只 new、不装开关，core 里没有一处调 `set_code_two_tree_node_capacities`：挂载那两处的容量今天恒是节点格式，改前改后同一个数。
「改回去传 `FromTheNodeFormat`」是等价变异，加进 `crates/mutations.tsv` 会在 59 号上活着，所以没加。规格要的「先红后改」这一件做不到，照实写在这里。

**真正读不到压小容量的那一处**（`transaction.rs` 第 4859 行，归 C11b，我没改）的改法：把
`&admission_reading_before_a_publish(allocator, previous),` 换成
`&admission_reading_with_the_switch_reserve_of(allocator, previous, InstanceRowsOfTheSwitchReserve::OfTheNextWritableMount, pool.code_two_tree_node_capacities()),`
（两样都已是 `admission` 的 `pub`，`use` 里加上）。它的红：写入口装压小容量（如 `capped((477, 150), (2, 2))`）的池上逼到准入紧的那一刻，按节点格式算的 ckpt_cost 比那一版空发布实写少、保留池少扣——这条用例我没造（落在 `transaction.rs` 上才有意义）。

## 五、另一件：崩溃枚举那份改钉（`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`）

照今天的代码现算（探针 `probes/probe_a4c_direct_refusals.rs`：每次会话覆盖写之前，在盘面与分配器的拷贝上直接调发布路径覆盖写一次），原样：

```text
OVERWRITE 18 direct_before=SpaceAdmissionRefused session=Ok pushed_by=["admission"]
OVERWRITE 32 direct_before=SpaceAdmissionRefused session=Ok pushed_by=["admission"]
OVERWRITE 47 direct_before=SpaceAdmissionRefused session=Ok pushed_by=["admission"]
OVERWRITE 62 direct_before=PlacementRefused(Data(DataUnitIndexInFile(0)), NoFreeSlotOnAnyDevice) session=Ok pushed_by=["placement"]
OVERWRITE 77 direct_before=PlacementRefused(Data(DataUnitIndexInFile(0)), NoFreeSlotOnAnyDevice) session=Ok pushed_by=["placement"]
```

改钉：会话覆盖写 `1..=61`，第 62 次直接调发布路径断言 `PlacementRefused`；注释与断言消息里「第 51 次」换成「第 62 次」，「准入放行」换成照实的「数据单元落点被拒（发布路径算定形状时在分配器的拷贝上就取不到，在准入与任何写之前返回）」，文档注释补一句第 18、32、47 次是式子先拒。
用例后半没动。按改后的前缀只算数、不枚举（探针 `probes/probe_a4c_crash_count.rs`，把用例截在状态数断言之后），原样：

```text
CRASH-COUNT states=77850 writes=53 root_writes=3 publishes=3
CRASH-COUNT overwrite-after-raise-ok
```

推的那一串 3 次空发布、53 次写（根槽写 3 次与空发布数相等），状态数 77 850 在 10⁶ 以内，推过之后覆盖写做成——用例在枚举之前的断言都过得去。全量枚举（每个状态恢复、再挂、池级 checker）没跑：标了 ignore 的崩溃枚举，归提交时的崩溃验证员；它不在名字带 layer0 的二进制里、已标 ignore，登记 `crash-case:` 那一行照旧（我没加新的崩溃枚举用例）。

## 六、变异行与证红

写在草稿目录（主表没改；主 agent 用 `research/scripts/apply-writer-patch.py` 的格式打）：`/tmp/claude-1000/impl-rev-a4c/mutations-append.tsv`，1 行，六段。原样（制表符分隔）：

```text
实审 A4c C545 那一格（D3 已定项 8 第 2 条 / 已定项 10 ②）：用户数据的候选落点不排除这次挂载开过的聚簇段（段外没有成对空槽时第 7 个单元落进段里、写做成）	crates/singlefs-core/src/allocator.rs	            if !inside_cluster_segment\n                && self.is_free(SlotNumber(candidate))	            let _ = inside_cluster_segment;\n            if self.is_free(SlotNumber(candidate))	-p singlefs-harness --test second_transaction_admission_raises_the_floor_before_refusing -- a_write_whose_free_slot_pairs	a_write_whose_free_slot_pairs_are_all_inside_this_mounts_cluster_segments_is_refused_by_placement_before_any_write_while_the_byte_formula_admits
```

- 名字：`实审 A4c C545 那一格（D3 已定项 8 第 2 条 / 已定项 10 ②）：用户数据的候选落点不排除这次挂载开过的聚簇段（段外没有成对空槽时第 7 个单元落进段里、写做成）`。
- 改坏哪一行 → 哪条断言红：`crates/singlefs-core/src/allocator.rs` 第 593 行 `if !inside_cluster_segment` 那两行换成不判聚簇段 →
  `second_transaction_admission_raises_the_floor_before_refusing.rs` 第 261 行起那条 `assert!(matches!(&refused, Err(PublishError::PlacementRefused { Data(0), NoFreeSlotOnAnyDevice })))` 红，消息原样 `顺序写到 7 个单元：数据单元 0 取不到落点：Ok(CheckpointTxg(91))`（第 7 个单元落进段里、写做成）。同一次跑（过滤到这一条）没有别的测试红。
- 证红：副本 `/tmp/claude-1000/impl-rev-a4c/copy`（主表那一行追加在副本的 `crates/mutations.tsv` 末尾），`PROVE_RED_LOG_DIRECTORY=/tmp/claude-1000/impl-rev-a4c/prove-red-logs bash research/scripts/capped.sh 5 bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-rev-a4c/copy --memory 8G singlefs-harness '<上面那个名字>'`，判定两行原样：

```text
实审 A4c C545 那一格（D3 已定项 8 第 2 条 / 已定项 10 ②）：用户数据的候选落点不排除这次挂载开过的聚簇段（段外没有成对空槽时第 7 个单元落进段里、写做成）	抓到	a_write_whose_free_slot_pairs_are_all_inside_this_mounts_cluster_segments_is_refused_by_placement_before_any_write_while_the_byte_formula_admits 红了（日志 /tmp/claude-1000/impl-rev-a4c/prove-red-logs/001.log）
✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了
```

  基线（不改源码、同一组参数）：`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.68s`（`prove-red-logs/baseline.log`），基线红集为空；改坏之后：`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.70s`（`prove-red-logs/001.log`）。
- 没加的变异：第 3 件的挂载两处是等价变异（第四节）；崩溃枚举那份只改钉值，点名它的既有一行（`crates/mutations.tsv` 第 777 行「崩在准入推的那一串空发布中间…」，参数不带 `--ignored`、那条用例标了 ignore）照旧，留给 59 号，我没在副本里跑它。

## 七、第 4 步那几样（主工作区，00:0x UTC；开跑前 `ps` 看到别的会话一条 `cargo test --offline -p singlefs-harness --lib` 在跑，没有性能测量；等锁多久没量）

动到的测试二进制与 core 单测（`nice -n 19 bash research/scripts/capped.sh 5 bash research/scripts/run-with-memory-cap.sh 8G bash /tmp/claude-1000/impl-rev-a4c/run-main-mine.sh`，debug），各末行原样：

```text
second_transaction_admission_raises_the_floor_before_refusing: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 90.72s
second_transaction_crash_inside_the_floor_raise_pushed_by_the_session: test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
admission_checkpoint_cost_per_device_paths: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 16.47s
singlefs-core --lib: test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

`admission_checkpoint_cost_per_device_paths` 我没改，它点名的函数在 `admission.rs` 里，一起跑了。崩溃枚举那份唯一的用例标了 ignore（0 跑、1 ignored），全量没跑。副本里前两个二进制与 core 单测先跑过一次，同样全绿（`touched-*.log`）。

`cargo fmt --all -- --check`：退 1，差异全在 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（C11b 的文件，67 处），我那 4 份文件没有差异：

```text
$ grep '^Diff in' main-fmt.log | sed -E 's/:[0-9]+:$//' | sort | uniq -c
     67 Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
```

`cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `check.sh` 那 7 条：第一次红在我加的第 8 个参数（`too_many_arguments`，`mount.rs` 第 3376 行），加了 `#[allow(…, reason)]`；第二次整仓跑红在别人的文件上（原样节选）：

```text
error: `mut allocator` shadows a previous, unrelated binding
    --> crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:3312:13
error: `ranges` shadows a previous, unrelated binding
    --> crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:7244:17
clippy exit=101
```

只对我动到的目标再跑（同一组 lint）：`cargo clippy -p singlefs-core --all-targets` 退 0；`cargo clippy -p singlefs-harness --test second_transaction_admission_raises_the_floor_before_refusing --test second_transaction_crash_inside_the_floor_raise_pushed_by_the_session --test admission_checkpoint_cost_per_device_paths` 退 0（`main-clippy-core.log`、`main-clippy-mine.log`）。

`cargo build --offline --all-targets`：退 101，红在 B3a-3c 的 layer0 文件上（原样）：

```text
error[E0425]: cannot find function `second_content` in this scope
   --> crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split_layer0.rs:231:27
error: could not compile `singlefs-harness` (test "second_transaction_supplement_two_tree_split_layer0") due to 1 previous error
build exit=101
```

`cargo build --offline -p singlefs-core --all-targets` 退 0；我那三个 harness 目标上面编过、跑过。

登记给我的门禁阶段（`stage-owners.tsv`，逐个 `nice -n 19 bash .claude/gate.d/<文件>`，74 号 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G`），退出码与判定行原样：

```text
33-mutation-tables.sh exit=0
  ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1106 条的原文各命中源码一次；没有同名实验二进制、锚点不在本阶段射程的表 6 张：research/mutations/e125_zoned_wp.tsv research/mutations/e129_tear_injector.tsv research/mutations/e129_thin_neighbour.tsv research/mutations/e129_thin_rmw.tsv research/mutations/e158_arms.tsv research/mutations/e158_r3_arm_mutations.tsv （本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）
53-format-const-placeholders.sh exit=0
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））
92-layout-checker-sync.sh exit=0
  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，106 个格式常量里变了 16 个（checker 在同一次改动里跟了 16 个，按滞后表放行 0 个），都不欠 checker 跟进
94-checker-implementation-disjoint.sh exit=0
  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`（别名引进来的 0 个）；共享模块 1 份源码的正文 286 行里没有分支与循环（`#[cfg(test)]` 标着的项 313 行不扫）
93-feature-bits.sh exit=0
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））
89-closeout-row27-preconditions.sh exit=77
  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）
74-model-differential.sh exit=1
  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）
```

- 33 号查的 `crates/mutations.tsv` 是主表（1106 条，我没改）；我那一行在草稿里，打进去之后要再判一次。
- 89 号退 77（本次未跑），按没判写。
- 74 号红：`second_transaction_supplement_three_random_history` 22 过、2 红，红的是
  `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`。
  不是我这一轮带来的：在没有我第 3 件改动的副本（`copy-probe`，23:45Z 快照）上 release 同一个二进制，同样这两条红、`test result: FAILED. 22 passed; 2 failed; 2 ignored`（`baseline-random-history.log`）；A4b 报告第七节记的也是「22 过、2 红，基线就红」，调查员报告第 1、2 条（被抛弃根引用的单元被重新发出去）。我没修。

## 八、交主 agent 的设计问题（停在那一处，没自己定）

1. **第 1 件选哪个上界**（弹窗问用户）：K1（整棵树，拿得准；4 GiB 以上每块盘多扣约 2.7–3.2% 的盘，1 TiB 每块盘约 33 GiB）、K3（现有节点 + 每块盘两条新路径；产品容量各格多扣 2–15 块，量到的全部空发布都够，但回落落进「没有记录的叶」时证不了）、
   还是改分配器让一次空发布碰的叶有界（例如空发布不回落、换下旧节点时顺带重写落在别处的叶节点），再按那个界写式子。三样都要改 D28（挂载期承诺量） 已定项 4 的句子，书记员那一句等选定之后再写。
   在选定之前，今天的式子（K0）在第二节那几段历史里少扣 2–4 块：保留池不够时那一次空发布的固定点取不到落点，发布路径在任何写之前拒（`PlacementRefused`），不写坏盘，但「保留池保证 checkpoint 自己的固定点写得出去」这一句不成立。
2. **第 2 件的次序**：准入要在取落点之前判，改的是 `transaction.rs` `prepare_the_version_publish`（第 4825 行 settle 在前、第 4859 行准入在后），C11b 在改那份文件。排给谁、什么时候排，主 agent 定；另要定「落得下」算哪几项（第三节第 4 条），与这一格在 `df` 上是不是假性 ENOSPC（第三节第 5 条，推的）。
3. **第 3 件**：`transaction.rs` 第 4859 行那一处的改法在第四节，归 C11b；挂载那两处我改了但等价、没有红，要不要留由主 agent 定（留着的好处只是读数与写入口读同一档容量，将来挂载若装压小容量不用再找这两处）。
4. `checks-owed.md` C545 行「准入放行之后固定点分配仍可能失败」的说法：发布路径上取落点在准入之前，字面上走不到「准入放行之后」，走到的是「准入没判到就被落点拒」。是否要改那一行的措辞，交书记员（我写不了 kb）。

## 九、这一轮写过的文件

主工作区（`crates/`，用 Edit 改，改完逐份与副本 `cmp` 相同）：

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-core/src/admission.rs` | 第 3 件：`admission_reading_with_the_switch_reserve_of` 多 `node_capacities` 参数；新 `admission_reading_of_a_writable_mount_with_node_capacities`；另两个读数函数签名不变、传节点格式 |
| `crates/singlefs-core/src/mount.rs` | 第 3 件：两处读数传写入口的容量；`push_floor_raises_after_the_row_publish` 多一个参数与 `#[allow(clippy::too_many_arguments, reason)]`；一处文档注释 |
| `crates/singlefs-harness/tests/second_transaction_admission_raises_the_floor_before_refusing.rs` | 第 2 件：新用例 1 条与 3 个辅助函数；`an_admitted_overwrite_…` 文档注释里「发布路径先判准入」改成照实写；`use` 跟着加 |
| `crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs` | 另一件：51 → 62 改钉（`1..=61`、注释、断言消息） |

没写：`crates/singlefs-core/src/allocator.rs`（第 2 件没走到要暴露落得下的槽那一步）、`crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs`（第 1 件式子没改）、规格给名的新测试文件（第三节）、`crates/mutations.tsv`（行在草稿里）。
`crates/mutations.tsv` 要追加的 1 行在 `/tmp/claude-1000/impl-rev-a4c/mutations-append.tsv`（变异名见第六节）。
我这 4 份文件相对取副本时那一版的改动另存成 `/tmp/claude-1000/impl-rev-a4c/record-of-main-edits/crates.patch`（`git apply --check --reverse` 在主工作区上干净：**已经在主工作区里，不要再打**），逐份增删：admission.rs +30 −7、mount.rs +28 −10、admission_raises… +185 −5、crash_inside… +6 −5。

途中（约 00:10 UTC）主工作区被别的会话 `git stash`（`stash@{0}: On master: 同步远端前的工作区（2026-09-27 09:2x JST）`）再弹回：HEAD 换成 `ac927812`，这之间 `.claude/hooks/heavy-test-guard.sh` 带着合并冲突标记、我的 Bash 被那道 hook 拒了一阵；弹回之后我那 4 份文件与副本逐字节相同（上面 `cmp`），改动现在在暂存区里（弹回时连同别人的一起进了 index），所以 `git diff --stat -- crates litmus` 是空的。两样都原样：

```text
$ git diff --stat -- crates litmus | wc -l
0
$ git diff --cached --stat -- crates litmus | tail -n 1
 94 files changed, 31283 insertions(+), 11454 deletions(-)
$ git diff --cached --stat -- crates litmus | grep -E 'admission.rs|/mount.rs'
 crates/singlefs-core/src/admission.rs              |  659 +-
 crates/singlefs-core/src/mount.rs                  | 2766 ++++++--
```

（94 份里除上表 4 份，都是别的会话的改动；两份测试文件是未跟踪的新文件，不在 `--stat` 里。）弹回之后 `.claude/hooks/heavy-test-guard.sh` 里仍有 6 行冲突标记（`grep -c '^<<<<<<<\|^>>>>>>>'`），我没碰。
第七节的测试、clippy、门禁都是在 stash 之前跑的；弹回之后我那 4 份与跑的时候逐字节相同，`git diff 73ba4a4 HEAD -- crates litmus` 为空（这一次同步没带进 crates 的改动），没重跑。

## 十、没做什么

- 没走三方对抗；没提交；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`，都没跑。全量 `cargo test`、`check.sh`、`gate.sh` 整轮没跑。
- 第 1 件：式子没改（停下）；K1、K3 都没写进 `admission.rs`；量表那份测试没加新格——第二节那几段历史只在探针里（`probes/probe_a4c_leaves.rs`，要进仓得先定式子，否则就是一条红着的用例）。随机历史只跑了 4 GiB 与单元区 2600 槽两档，1 GiB、64 GiB、1 TiB 没跑随机历史；三块盘那一格没重量。
- 第 2 件：准入没改（卡在 `transaction.rs`）；`allocator.rs` 没动；规格给名的新测试文件没建；`df` 在那一格报多少没量。
- 第 3 件：造不出红、没有变异（等价）；`transaction.rs` 那一处没改（C11b）。
- 另一件：崩溃枚举全量没跑（标了 ignore），只按改后的前缀算了状态数；点名它的第 777 行变异没跑。
- 门禁 74 号红的那两条（被抛弃根那一族，调查员第 1、2 条）没修，不在这一轮；89 号退 77 没判。
- 探针产物没进 `research/results/`：我的写范围只有 `crates/`、`litmus/` 与草稿目录；要留由主 agent 拷。都在 `/tmp/claude-1000/impl-rev-a4c/`：
  探针源码 `probes/`（4 份 `.rs` 与两份只在副本里的改动 `admission-probe-function.diff`、`measurement-cells-candidates.diff`），
  量到的原样 `probe-1.log`、`probe-2.log`、`probe-3.log`、`probe-4gib-candidates.log`、`probe-2600-candidates.log`、`probe-cells-candidates.log`、`probe-roles-*.log`、`probe-c545-1.log`、`probe-direct-refusals.log`、`probe-crash-count.log`，
  门禁与测试日志 `gate-*.log`、`main-*.log`、`touched-*.log`、`baseline-random-history.log`、`prove-red-logs/`。

## 十一、清理

仓副本 `/tmp/claude-1000/impl-rev-a4c/copy` 与 `/tmp/claude-1000/impl-rev-a4c/copy-probe`（各带自己的 `target/`）交回前删掉；别的编译目录没建。
留着的：上面那些日志与 `probes/`、`record-of-main-edits/`、`mutations-append.tsv`、`progress.md`、几份跑命令用的小脚本（`run-*.sh`）、本报告——都不是编译目录或仓副本。
