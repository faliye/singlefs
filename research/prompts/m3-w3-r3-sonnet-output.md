# m3-w3-r3 正推腿（Sonnet）报告：K1 代价表、K2 Y1 两种取法

草稿目录 `~/.local/share/singlefs/work/m3-w3-r3-sonnet/`；模型目录 `research/prompts/m3-w3-r3-sonnet-model/`；冻结树 `~/.local/share/singlefs/work/m3-w3-r1/tree/`。
（正文分段追加；末尾有「交回前核对」。）

轮名 m3-w3-r3；背景 `research/prompts/_m3-w3-r3-background.md`（正文 `research/prompts/_m3-w3-r3-body.md`）；前两轮判决 `research/prompts/m3-w3-r1-main-verification.md`、`research/prompts/m3-w3-r2-main-verification.md`；批次 `records/2026-10-01-删除与并发事务.md#删除线第 47 批`。
立场：正推。只答 K1、K2；K3 归攻方腿，不碰。冻结树 `/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/`（只读）。
这一份里的数全是抽象模型、冻结树副本上的探针和纯算术，不是入库装置上的数；我提的形态没有一个写进 `crates/`；进 kb 之前要在入库装置上重做。

## 〇、先看这几条

1. 三个形态在同一组历史、同一套口径上放齐了（第三节 3.1–3.5）。V1 臂（回退取目标那一版的水位）上，C2aL、C3rL、C4g 对「现有 I-9.29、I-9.6 都不红」的事件与持久镜像都漏 0；C2i 单独漏持久镜像 3.8%。三者的差别不在这一格，在三处：覆盖面（C4g 管不到恢复时少算的那一形，V3 臂漏 12 056 / 76 552）、历史写错时红不红（C4g 单存储，C2aL、C3rL 有互核）、代价（第三节）。
2. 代价表（3.7、3.10）的头条数：C4g 每次发布 0 个单元、回退那次谱系记录多 8 字节；C3rL 每个建 inode 的发布恒多 2 个单元（4 次写，崩溃状态 ×16）；C2aL 台阶行 ≤ 462 条时 0 个单元，之后 +2 个、36 074 条之后 +4 个。记账树只往右追加时每片叶只装一半（叶从中间切），所以 463 而不是 477 就长高。
3. Y1（第四节，并排表在 4.4）：取法一（维持）下，把跳升吸收进历史的做法（C2aL 的台阶行，谱系带 cur 的值）不放过真重发；把「环里有被抛弃的根带着这个水位就放过」写成判据（容许跳升）会放过 422 / 9 534 个 V1 + 历史不写的真重发，而且 create 台阶那一形仍误报 81 686 张。取法二（只取没被抛弃的根）：在含「算水位那一遍读不出 cur 自己的根」的字母表里 P 臂一张都不退，C143 那一格没退回去；环里那一半在时间线上恒为空，改法是 `mount.rs` 里加一道滤。
4. 前两轮的数：两套抽象模型（辩方腿、攻方腿）与我并进后的模型在同一组历史上逐数相等（3.6），可以并排；装置上的数（E175 的 G-long、F2）与模型的数不是同一个空间，不能并排。攻方腿第二轮报告里「A + H 两个最强形 336805 / 336805」与它自己的产物对不上（C2aL 是 169763 / 336805）。

## 一、复跑与模型文件

模型目录 `research/prompts/m3-w3-r3-sonnet-model/`。在仓根，`W` 取一个空目录；每段一条前台命令（编译 6G、跑 4G，线程 6，都经 `scripts/hosts/run-with-memory-cap.sh`；只跑 harness 档）：

```
M=research/prompts/m3-w3-r3-sonnet-model; W=<空目录>
for p in build-model model-main model-defense model-mutation model-mutation2-p model-mutation2-v1 model-k2 model-blind model-tolring model-deep \
         probe-build probe-run probe-compare cost cost-check tables compare-r2 compare-r2opus compare; do SINGLEFS_REPO=$PWD bash $M/rerun.sh $W $p; done
```

- `timeline-model/`：抽象时间线模型（不依赖 `crates/`）。它是 m3-w3-r2 辩方腿的模型（`research/prompts/m3-w3-r2-sonnet-model/timeline-model/`）加三样：m3-w3-r2 攻方腿的候选（C2aL、C3rL 的最强形与 I-9.30 草案的几种读法、五种新变异）；Y1 的两个臂（`ProductWithoutAbandonedRoots` 取法二、`RollbackRingOnly` 控制臂）与一个字母（`rollback-blind`：算水位那一遍读不出 cur 自己的根）；「容许跳升」的三个候选（名字带 `-tol`）。
- 对账：老候选 25 个、22 个文件的每个字段与辩方腿的 expected 逐数相等（`rerun.sh compare-r2`）；攻方腿的 18 个文件、1 299 格与攻方腿的 expected 逐数相等（`rerun.sh compare-r2opus`）。命令输出在 3.6 末。
- `probe/`：两段只接在冻结树**副本**末尾的测试探针（`probe-build` 拷树、核清单、接上、编 `singlefs-core` 的 lib 测试），调真的 `plan_the_tree_after_this_publish` 与 `after_a_container_is_opened_at_the_right_end`，逐发布打重写节点数、树高。`cost/planner.py` 是规划器的 Python 复刻，`probe_compare.py` 与探针逐行对账，`--red` 先弄坏复刻（切在 ⌊n÷2⌋）必须判红。`cost/cost_model.py` 出代价表里的算术；`cost/check_constants.py` 把用到的 38 个格式常量与规划规则逐个在冻结树里 `grep -nF`。
- `expected/`：这一轮在草稿目录里跑出来的输出（去掉 W3TRACE 行、排序）；交回前在空目录从头跑通一次，末行在末尾「交回前核对」贴。

## 二、实现今天的样子（我读的路径；引句行号都是现查）

- 回退的水位。环里全部读得出的根都进来，不按实例表滤：`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7738`：「let ring_roots = readable_roots(」，`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7748`：「let inode_number_watermark = highest_inode_number_watermark_in_the_ring(」，与现行那一版内存里的取 max：`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7755`：「ring.max(current.inode_number_watermark())」。树 ID 水位同样取环里全部根：`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7747`：「.fold(current.root.tree_identifier_watermark, u64::max);」。
- 读不出的根不算：`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7456`：「Ok(RebuiltVersion::WithoutFile) | Err(_) => None,」。
- 回退在这之前已经读了 cur 的实例表，读不出就在任何写之前拒：`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7633`：「let current_table = instance_table_chain_of_root(&*devices, &current.root)」，下一行 `/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7634`：「.map_err(|_unreadable_or_malformed| RollbackError::CurrentInstanceTableMalformed)?」。判被抛弃的函数已经有，候选集用它：`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1542`：「pub fn root_is_abandoned_by_the_instance_table(」，判法在 `/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1561`：「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」。
- 记账树每发布整树重装，不是在上一版上改：`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/transaction.rs:4879`：「fn accounting_entries_of_this_publish(」现算这一代的 15 行，`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/transaction.rs:8082`：「let accounting_built = build_multi_level_tree(」。规划器先删上一版有而这一版没有的 key、再插这一版有而上一版没有的：`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/code_two_tree.rs:665`：「for deleted in previous_keys.difference(keys_after_this_publish) {」；叶装不下时从中间切：`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/code_two_tree.rs:450`：「let right_keys = keys.split_off(keys.len().div_ceil(2));」。
- 容器树（谱系树的形态）根满长高、末尾分裂、左边的节点一个字节不动：`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/container_tree.rs:286`：「pub fn after_a_container_is_opened_at_the_right_end(&self, entries_per_node: usize) -> Self {」。
- 格式常量：谱系记录 `/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:119`：「pub const LINEAGE_RECORD_BYTES: u64 = 16;」，记账条目 `/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:145`：「pub const ACCOUNTING_ENTRY_BYTES: u64 = 34;」，树表条目数 `/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:188`：「pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 11;」，池级 checker 已实现的不变量数 `/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/image.rs:98`：「pub const IMPLEMENTED_INVARIANTS: [&str; 84] = [」，崩溃状态闭式 `/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-harness/src/segments.rs:201`：「/// E77（发布的持久顺序） 的闭式：崩溃状态数 = 1 + Σ(2^|段| − 1)，|段| 按写数算。」。
- `crates/` 里没有发号历史（台阶行、发号记录）：第一轮正推已查（`research/prompts/m3-w3-r1-sonnet-output.md` 第 0.2 节的 grep，零命中）；三个形态的写者与读者都是「以后加在哪一处」。

## 三、K1：三个形态放在同一组历史、同一套口径上（抽象模型，数全是模型上量的）

### 3.1 定义：谁存什么、单镜像上怎么判红

| 形态 | 存什么、谁写 | 单镜像上判红 | 载体 |
|---|---|---|---|
| C2aL | 台阶行 (g, H)：每次水位变了的发布留一行（建 inode 的发布、回退跳升那次），随根 COW、回退从 cur 带；谱系记录多带 H_cur（cur 的水位） | I-9.30 草案 ① 水位 ≥ 每行 H、② 树里每条 (号 n, 出生代 b) 不存在一行 H > n 且 b > g、③ 水位大于初值时有一行 H ≥ 水位且 g ≤ txg；再加每条谱系记录 max{H : g ≤ T_cur} 等于 H_cur 且 H_cur ≠ 0 | 记账树（要开例外）；或容器树 |
| C3rL | 根下只追加的 (号, 出生代)，随根 COW、回退从 cur 带；谱系记录多带 count = cur 那一版发号记录的条数 | 同号两个出生代；记录里有号 ≥ 水位；最新树里每条记录都在记录里；每条谱系记录 #{b ≤ T_cur} 等于 count 且 count ≠ 0 | 谱系树同形的容器树 |
| C4g | 谱系记录多带 W = 回退前 cur 的水位（16 → 24 字节），随现行那一版、回退那次追加一条 | 水位 < W，或最新树里有 (n, b) 满足 n < W 且 b > T_rb | 谱系树（已有） |
| C2i | 每个实例 (最高水位, 第一次到它的代) | 水位 < 高水位，或有 (n, b) 满足 n < 高水位 且 b > 那个代（每个实例各判） | 记账树 |
| C2ig | C2i 与 C4g 各存各的，红一处即红 | 任一红 | 两处 |

出处（定义是前两轮腿给的，我没改）：「每次水位变了的发布留一行」（`research/prompts/m3-w3-r2-opus-output.md:89`）；「谱系记录多带 count = cur 那一版发号记录的条数」（`research/prompts/m3-w3-r2-opus-output.md:90`）；「谱系记录多带 (回退前 cur 的水位 W)，16 → 24 字节」（`research/prompts/m3-w3-r2-sonnet-output.md:62`）；「同 C2d，每个实例各判一次」（`research/prompts/m3-w3-r2-sonnet-output.md:60`）；「C2i 与 C4g 各存各的，红一处即红」（`research/prompts/m3-w3-r2-sonnet-output.md:63`）。

### 3.2 「洞」：V1 臂（回退取目标那一版的水位）上，现有检查都不红的事件与持久镜像，候选漏几个

「洞」= 现有 I-9.29t（含谱系上的墓碑）与 I-9.6t（含墓碑那一半）都不红。事件 = 同一条时间线上重发的那一步；持久镜像 = 事件之后被重发的对象还活在最新根里的镜像。格里写「漏的个数 / 分母」：

| 历史（环 × 长 × 字母表） | 洞事件分母 | 持久洞分母 | C2aL | C3rL | C4g | C2ig | C2i（事件 + 持久） |
|---|---|---|---|---|---|---|---|
| 环 4 × 7 × 12 个 | 138372 | 352888 | 0/138372 + 0/352888 | 0/138372 + 0/352888 | 0/138372 + 0/352888 | 0/138372 + 0/352888 | 0/138372 + 13486/352888 |
| 环 6 × 7 × 12 个 | 54264 | 158272 | 0/54264 + 0/158272 | 0/54264 + 0/158272 | 0/54264 + 0/158272 | 0/54264 + 0/158272 | 0/54264 + 5952/158272 |
| 环 8 × 7 × 12 个 | 44704 | 109108 | 0/44704 + 0/109108 | 0/44704 + 0/109108 | 0/44704 + 0/109108 | 0/44704 + 0/109108 | 0/44704 + 3914/109108 |
| 环 12 × 7 × 12 个 | 44704 | 109108 | 0/44704 + 0/109108 | 0/44704 + 0/109108 | 0/44704 + 0/109108 | 0/44704 + 0/109108 | 0/44704 + 3914/109108 |
| 环 4 × 8 × 8 个（含删除、墓碑、崩溃抛弃、实例切换） | 121336 | 440672 | 0/121336 + 0/440672 | 0/121336 + 0/440672 | 0/121336 + 0/440672 | 0/121336 + 0/440672 | 0/121336 + 29002/440672 |
| 环 6 × 6 × 12 个 | 2696 | 5908 | 0/2696 + 0/5908 | 0/2696 + 0/5908 | 0/2696 + 0/5908 | 0/2696 + 0/5908 | 0/2696 + 218/5908 |

出处（环 4 × 7 一行，其余各文件同一位置）：分母 「events_where_existing_checks_silent=138372 … residual_where_existing_checks_silent=352888」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-0.out:181`）；C2aL 「candidate=C2aL false_positive=489394 … event_missed_where_existing_checks_silent=0 … residual_missed_where_existing_checks_silent=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-0.out:7`）；C3rL 「candidate=C3rL false_positive=0 … event_missed_where_existing_checks_silent=0 … residual_missed_where_existing_checks_silent=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-0.out:24`）；C4g 「candidate=C4g false_positive=0 … event_missed_where_existing_checks_silent=0 … residual_missed_where_existing_checks_silent=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-0.out:31`）；C2ig 「candidate=C2ig false_positive=0 … event_missed_where_existing_checks_silent=0 … residual_missed_where_existing_checks_silent=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-0.out:21`）；C2i 「candidate=C2i false_positive=0 … event_missed_where_existing_checks_silent=0 … residual_missed_where_existing_checks_silent=13486」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-0.out:20`）。
环 6、环 8、深一层的分母与漏数照同一个文件位置读：「events_where_existing_checks_silent=54264 … residual_where_existing_checks_silent=158272」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-1.out:179`）；「events_where_existing_checks_silent=44704 … residual_where_existing_checks_silent=109108」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-2.out:179`）；「events_where_existing_checks_silent=121336 … residual_where_existing_checks_silent=440672」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-deep-2.out:181`）。

读法：① 三个形态在洞上没有差别：环 4、6、8、12 与深一层，洞事件与持久洞都漏 0；C2i 单独漏持久镜像（环 4 上 13486/352888 = 3.8%，环越长占比越小但不到 0）。② 洞里「回退之后没挂载过」的事件占 25838/138372，三个形态在这一类上同样漏 0——它们的历史都不靠环里还留着回退前的根（C1b 靠这个，第二轮已出局）。③ 这一格是前两轮四套模型能并排的那一格（见 3.6）。

### 3.3 覆盖面：恢复时水位少算 1（V3 臂）与发布不加新建个数（V2 臂）

| 臂 | 洞事件分母 | 持久洞分母 | C2aL | C3rL | C4g | C2ig | C2i |
|---|---|---|---|---|---|---|---|
| V3（恢复时新实例的水位比所选根的少 1，环 4 × 7） | 76552 | 190236 | 0/76552 + 0/190236 | 0/76552 + 0/190236 | 12056/76552 + 149416/190236 | 0/76552 + 0/190236 | 0/76552 + 0/190236 |

出处：「events_where_existing_checks_silent=76552 … residual_where_existing_checks_silent=190236」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-4.out:235`）；C4g 「candidate=C4g false_positive=10218 … event_missed_where_existing_checks_silent=12056 … residual_missed_where_existing_checks_silent=149416」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-4.out:31`）；C3rL 「candidate=C3rL false_positive=0 … event_missed_where_existing_checks_silent=0 … residual_missed_where_existing_checks_silent=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-4.out:24`）；C2aL 「candidate=C2aL false_positive=1525996 … event_missed_where_existing_checks_silent=0 … residual_missed_where_existing_checks_silent=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-4.out:7`）。

读法：C4g 只管回退引起的水位退回，V3 上漏 12056/76552 个洞事件（15.7%）与 149416/190236 张持久洞镜像；C2aL、C3rL（每次水位变了 / 每个新 inode 都留痕）与 C2i、C2ig 都漏 0。这是第二轮辩方腿量过的那一形（它报 12 056 / 76 552），我并进来的 C2aL、C3rL 两行是新的。V3 不是产品路径能走到的（阳性对照），所以这一格问的是「将来别的写错水位的缺陷」要不要也被抓到，不是今天的洞。
V2 臂（发布不加新建个数）：现有 I-9.6 在每个 V2 事件上都红，洞是 0（「events_where_existing_checks_silent=0 … residual_where_existing_checks_silent=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-4.out:141`））；C3rL 是唯一抓得到 V2 事件的（漏 0），C2aL、C4g、C2i 漏全部 1523216。第二轮判决已判 V2 不在 W3 射程里，这里只记数，不算形态的分。

### 3.4 合法镜像上的误报（P 臂；V1、V3 臂的 FP 列不读）

| 历史 | 合法镜像 | 其中合法的跨被抛弃时间线重发 | C2aL | C3rL | C4g | C2i | C2ig |
|---|---|---|---|---|---|---|---|
| 环 4 × 7，从不清扫 | 10836269 | 48356 | 0 | 0 | 0 | 0 | 0 |
| 环 4 × 7，清扫阈值 0 | 10964563 | 35042 | 0 | 0 | 0 | 0 | 0 |
| 环 8 × 7，清扫阈值 0 | 11444772 | 71380 | 0 | 0 | 0 | 0 | 0 |
| 环 4 × 8 × 8 个字母 | 5184012 | 65700 | 0 | 0 | 0 | 0 | 0 |
| 合计 | 38429616 | 220478 | 0 | 0 | 0 | 0 | 0 |

出处：「images=10836269 legal_images=10836269 … reissues_across_abandoned_roots=48356」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-0.out:57`）；「images=10964563 legal_images=10964563 … reissues_across_abandoned_roots=35042」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-1.out:57`）；「images=11444772 legal_images=11444772 … reissues_across_abandoned_roots=71380」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-5.out:54`）；「images=5184012 legal_images=5184012 … reissues_across_abandoned_roots=65700」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-deep-0.out:57`）；候选行同一位置（`candidate=C2aL false_positive=0` 起）。

V1 臂上 C2aL 的 FP 列非 0（环 4 上 489 394）不是合法历史判红：「candidate=C2aL kind=false_positive ops=[create,rollback1,rollback1]」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-0.out:69`）——V1 的第一次回退把水位压低一格，台阶行记下了这一次，第二次回退又回到高的那条根，镜像本身没有重发，但台阶行里留着前一次退回的那一行（H 比前面的低），谱系那一对的判据（max{H : g ≤ T_cur} 等于 H_cur）对不上就红。这是历史里粘着的前一次退回，不是合法历史上的误报；它在 P 臂上不出现（上表）。

### 3.5 历史写错或不写：八种变异下，现有检查都不红的 9 534 个真重发事件里候选漏几个（V1 臂，环 4 × 6）

变异（每一种都在同一个回退 / 恢复步骤上改写历史，别的步骤照对）：A = 回退那次历史与谱系那一对都取 R_old 的（等于攻方腿的 A + H）；B = 历史只照抄这一版自己的；C = 历史不写；H = 谱系那一对取 R_old 的、历史照对；Z = 谱系那一对写 0；R = 回退跳升那一行不写；D = 台阶行记成发布之前的水位；Mh = 新实例的历史从被抛弃的根带。定义出处：「回退那次，根下的发号历史（C2*、C3r）与谱系里那份「回退前水位」都取 R_old 那一版的」（`research/prompts/m3-w3-r3-sonnet-model/timeline-model/src/main.rs:32`）；「谱系那一对 (H_cur, count) 取 R_old 那一版的，历史本身照对」（`research/prompts/m3-w3-r3-sonnet-model/timeline-model/src/main.rs:38`）；「崩溃恢复抛弃最新那条根时，新实例的历史从被抛弃的那条根带」（`research/prompts/m3-w3-r3-sonnet-model/timeline-model/src/main.rs:46`）。

| 候选（存几处） | V1+A | V1+B | V1+C | V1+H | V1+Z | V1+R | V1+D | V1+Mh |
|---|---|---|---|---|---|---|---|---|
| C2aL（两处：台阶行 + 谱系 H_cur） | 9534 | 9534 | 0 | 0 | 0 | 0 | 0 | 0 |
| C3rL（两处：发号记录 + 谱系 count） | 9534 | 9534 | 0 | 0 | 0 | 0 | 0 | 0 |
| C4g（一处：谱系 W） | 9534 | 9534 | 9534 | 9446 | 9534 | 0 | 0 | 0 |
| C2i（一处） | 9534 | 9534 | 9534 | 0 | 0 | 0 | 0 | 0 |
| C2ig（两处，各判各的，不互核） | 9534 | 9534 | 9534 | 0 | 0 | 0 | 0 | 0 |

分母每一列都是 9534。出处：「events_where_existing_checks_silent=9534」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-mutation-5.out:291`）（V1 + C 一列；别的列同一位置）；C4g 在 V1 + C 上 「candidate=C4g false_positive=0 … event_missed_where_existing_checks_silent=9534」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-mutation-5.out:31`）；C3rL 「candidate=C3rL false_positive=0 … event_missed_where_existing_checks_silent=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-mutation-5.out:24`）；C4g 在 V1 + H 上 「candidate=C4g false_positive=0 … event_missed_where_existing_checks_silent=9446」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-mutation2-5.out:31`）。

读法：
- 有互核的两个（C2aL、C3rL）在 C、H、Z、R、D、Mh 六种上一个真重发都不漏；单存储的 C4g 在 C 与 Z 上漏全部、H 上漏 9 446，因为谱系那一份自己写错或不写时没有第二处可比。这就是 r2 辩方腿说的单个存储做不到互核，数在这里。
- A 与 B 上三个形态都漏全部 9 534：A 是历史与谱系同取 R_old、两处一起错，互核看不出；B 是历史只剩这一版自己的、没东西可比。两处互核只防「一处错」。
- C2i 在这八列里 A、B、C 漏全部、别的列 0：它不读谱系，H、Z、R、D、Mh 改的不是它的存储，不构成它的检出。
- P 臂（没有真重发，只看改过的镜像红不红）：C2aL 在 A 上 169763/336805 张被改过的镜像不红，C3rL 336805/336805，C4g 336805/336805；完整的 P 臂八列在 `research/prompts/m3-w3-r3-sonnet-model/expected/report-tables.out` 的表三。出处 「candidate=C2aL false_positive=0 … mutated_history_undetected=169763 … mutated_images=336805」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-mutation-0.out:7`）。

### 3.6 前两轮四套模型的数能不能并排

四套 = 第一轮攻方腿的抽象模型 + 第二轮辩方腿的抽象模型（又改自第一轮攻方腿）+ 第二轮攻方腿的抽象模型（改写）+ 两处装置（第一、二轮正推 / 辩方腿的 E175 装置影子，第二轮攻方腿的原型）。

| 对比项 | 结论 | 出处 |
|---|---|---|
| 三套抽象模型的历史空间（环 4、长 7、12 个字母、V1 臂） | 同一个：images、events、residual_images 逐数相等，可以并排 | 第一轮攻方腿 「images=10836269 legal_images=7914147 events=340776 events_where_I-9.29_holds=144904 residual_images=443952」（`research/prompts/m3-w3-r1-opus-model/expected/model-main-2.out:1`）；第二轮攻方腿 「images=10836269 legal_images=0 events=340776 residual_images=443952」（`research/prompts/m3-w3-r2-opus-model/expected/model-main-2.out:1`）；第二轮辩方腿 「images=10836269 legal_images=7914147 events=340776 events_where_I-9.29_holds=144904」（`research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-0.out:107`）；这一轮 「images=10836269 legal_images=7914147 events=340776 events_where_I-9.29_holds=144904 … residual_images=443952」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-2.out:181`） |
| 逐格对账 | 这一轮并进后的模型对辩方腿 22 个文件、每个文件 441 格（25 个老候选）0 格不同；对攻方腿 18 个文件 1 299 格 0 格不同 | `rerun.sh compare-r2`、`compare-r2opus` 的末行，见本节末「命令输出」 |
| 「C2a」不是同一个东西 | 第一轮与辩方腿的 C2a 每个发布留一行（含空发布、挂载）；攻方腿的 C2aL / I-9.30 的台阶行只在水位变了的发布留一行。数不能直接比，要比的话比 C2aL 那一列 | 老写法 「upsert(&mut records.generation_rows, txg, watermark);」（`research/prompts/m3-w3-r3-sonnet-model/timeline-model/src/main.rs:236`）；新写法 「if watermark != watermark_before {」（`research/prompts/m3-w3-r3-sonnet-model/timeline-model/src/main.rs:233`） |
| 「合法镜像」的口径 | 辩方腿的 legal 不看臂（V1 臂上也有「合法」），攻方腿只在 P 臂数；V1、V3 臂的 FP 列两边都不读 | 攻方腿 V1 臂 legal_images=0（上一行引句），辩方腿 7914147 |
| 「洞」口径 | 只有辩方腿与这一轮有（现有检查都不红的事件、持久镜像）；第一、二轮攻方腿只有「候选单独红不红」，不能直接比洞 | 「events_where_existing_checks_silent=138372」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-0.out:181`） |
| 装置上的数（E175 的 G-long、F2） | 与模型不是同一个空间（32 / 64 段真代码历史，事件 476 / 336 个），不能与模型的数并排，只能在装置之间比；而且 c2a、c3 的影子用的是装置攒下的全部根账，第一轮正推自己写了是上界：「c2all、c3 的 1 934 / 1 934 是上界」（`research/prompts/m3-w3-r1-sonnet-output.md:122`）；C4g、C2i 的影子是真规则（回退那次写 W、每实例留高水位）。第二轮辩方腿的装置数：「c4g 476 6 2 3234 221 4070」（`research/prompts/m3-w3-r2-sonnet-model/expected/device-G-long-V1.summary:12`） |
| 第二轮攻方腿的原型（层 0 全域 65 548 个状态） | 是第三个空间：一次建 inode 的发布的崩溃状态，不与上面两处并排；我不能复核（层 0 枚举是 checker 档，不跑） | 它自己的汇总 「w3a_cross_abandoned_reissue_images=131072」（`research/prompts/m3-w3-r2-opus-model/expected/ab-merge-of-20.out:7`） |

**报出来的数核不对的一处**：攻方腿第二轮报告第五节的表把变异 A + H 在 P 臂上写成两个最强形各「336805 / 336805」（「| A+H 历史与谱系那一对都取 R_old（双变异） | 两个最强形 | 336805 / 336805；336805 / 336805 | 26410；26410 |」（`research/prompts/m3-w3-r2-opus-output.md:202`）），它自己的产物里 C2aL 是 169763 / 336805：「candidate=C2aL-strongest … mutated_history_undetected=169763 … mutated_images=336805」（`research/prompts/m3-w3-r2-opus-model/expected/model-mutation-10.out:14`），C3rL 才是 336805 / 336805：「candidate=C3rL-strongest … mutated_history_undetected=336805 … mutated_images=336805」（`research/prompts/m3-w3-r2-opus-model/expected/model-mutation-10.out:15`）。这一轮的模型给同一个数：「candidate=C2aL … mutated_history_undetected=169763 … mutated_images=336805」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-mutation-0.out:7`）。V1 臂那一列 26410 与产物对得上（「candidate=C2aL-strongest … event_missed=26410」（`research/prompts/m3-w3-r2-opus-model/expected/model-mutation-11.out:14`））。结论「A + H 两处一起错互核看不出」仍成立（V1 + A 上两者都漏全部 9 534），只是 P 臂上 C2aL 能红约一半被改过的镜像，不是零。

命令输出（最终那次空目录复跑，`rerun.sh compare-r2`、`compare-r2opus` 的末几行，原样）：
```
$ ls rerun2/out/compare-r2-*.txt | wc -l
22
$ cat rerun2/out/compare-r2-*.txt | sort | uniq -c
     22 compare-r2 比了 441 格（25 个老候选），不同 0 格
$ bash $M/rerun.sh $W compare-r2opus
compare-r2opus 比了 18 个文件 1299 格，不同 0 格
```

### 3.7 每次发布多写的字节与单元、崩溃状态数、盘上增长（算术；规划器那一半对着真规划器的探针核过）

**怎么算的。** 记账树每次发布整树重装（「let accounting_built = build_multi_level_tree(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/transaction.rs:8082`）），规划器按「上一版有这一版没有的先删、这一版有上一版没有的再插」改形状，叶与内部节点装不下都从中间切，路径上碰过的节点整条重写。今天一代 15 行里 14 行（统计量 1–11）每发布换代，落在最左叶；水位那一行（统计量 12，最大的统计量）每发布换代，落在最右叶。台阶行留在水位那一组里，只往右追加；所以只有一片叶时每发布重写 1 个节点，分成两片之后最左叶与最右叶都要重写，加根，是 3 个，每长高一层再多 2 个。
核对：`cost/planner.py` 是规划器的复刻；探针把真的 `plan_the_tree_after_this_publish` 接在冻结树副本末尾，逐发布打重写数与树高。对账 「probe-compare 比了 9077 行，不同 0 行」（`research/prompts/m3-w3-r3-sonnet-model/expected/probe-compare.out:10`）（9 056 行 = 容量压小的三档各 3 000 个发布 + 产品容量的两档各 28 个采样发布，21 行是先一次装进 H 个台阶行再接着走的四个点）；先弄坏复刻（切在 ⌊n÷2⌋）必须红：「probe-compare 比了 3000 行，不同 2969 行」（`research/prompts/m3-w3-r3-sonnet-model/expected/probe-compare-red.out:12`）。
真规划器在产品容量（叶 477、内部 150）下的几个点：「publish=464 rewritten=3 height=2」（`research/prompts/m3-w3-r3-sonnet-model/expected/probe-bulk.out:3`）（第一次分叶）；「publish=36075 rewritten=6 height=3」（`research/prompts/m3-w3-r3-sonnet-model/expected/probe-bulk.out:10`），下一发布稳态 「publish=36076 rewritten=5 height=3」（`research/prompts/m3-w3-r3-sonnet-model/expected/probe-bulk.out:11`）；「publish=2742511 rewritten=9 height=4」（`research/prompts/m3-w3-r3-sonnet-model/expected/probe-bulk.out:20`），稳态 「publish=2742512 rewritten=7 height=4」（`research/prompts/m3-w3-r3-sonnet-model/expected/probe-bulk.out:21`）。长高那一发布多重写 1–2 个新节点，下面的表按稳态写。

**表 A：记账树（C2aL 的台阶行；C2i 的实例行也在这里）。** 「行数」= 已留下的台阶行数（C2aL）或实例数（C2i）。每个节点一个 16 KiB 单元，两份副本各写一次：

| 行数 | 树高 | 每发布重写的记账节点 | 比今天（1 个）多的单元 | 多的写 | 每发布写的字节（含两份副本） |
|---|---|---|---|---|---|
| ≤ 462 | 1 | 1 | 0 | 0 | 32 768（今天） |
| 463 – 36 073 | 2 | 3 | +2 | +4 | 98 304（+64 KiB） |
| 36 074 – 2 742 509 | 3 | 5 | +4 | +8 | 163 840（+128 KiB） |
| 2 742 510 – 208 431 645 | 4 | 7 | +6 | +12 | 229 376（+192 KiB） |
| ≥ 208 431 646 | 5 | 9 | +8 | +16 | 294 912（+256 KiB） |

出处：「accounting-height-thresholds history_rows_when_height_2=463 history_rows_when_height_3=36074 history_rows_when_height_4=2742510 history_rows_when_height_5=208431646」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:6`）；稳态 「accounting-steady-rewritten h1=1 h2=3 h3=5 h4=7 h5=9」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:10`）；右边缘模拟对复刻 「edge-model-vs-replica leaf_cap=477 internal_cap=150 publishes=40000 heights_compared=39999 different=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:5`）。463 而不是 477：叶从中间切，只往右追加时左半留 239 条不再碰，所以每片叶只装一半：「accounting-storage-per-step-row entry_bytes=34 leaf_rows_when_appending=239 bytes_per_row_one_copy=68.6 bytes_per_row_two_copies=137.1」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:22`）。
`singlefs-core` 里叶 477 条、内部 150 个孩子的算式：叶 (16384 − 159) ÷ 34，内部 (16384 − 159) ÷ 108，来自 「pub const ACCOUNTING_ENTRY_BYTES: u64 = 34;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:145`） 与 「pub const ACCOUNTING_INTERNAL_ENTRY_BYTES: u64 = 108;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:148`）、「pub const ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES: u64 = 159;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:71`）；`check_constants.py` 查了 38 项常量与规则在冻结树里都在（「check-constants 查了 38 项，缺 0 项」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-check.out:40`））。

**表 B：容器树（C3rL 的发号记录；三个形态的谱系记录都在这里）。** 根恒为码 2、孩子是码 3 容器，根满长高、末尾分裂、左边的节点一个字节不动，所以每次追加重写：最右那片容器（32 KiB）加最右路径上的码 2 节点（每层 16 KiB）。

| 记录 | 每容器装 | 一层码 2（根）装得下 | 每次追加重写 | 多的写 | 每次追加写的字节 |
|---|---|---|---|---|---|
| 发号记录 16 字节 / 每个新 inode 一条 | 2 039 | 275 265 条 | 2 个单元（容器 + 根） | +4 | 98 304 |
| 同上，超过 275 265 条之后 | 2 039 | 37 160 775 条（两层码 2） | 3 个单元 | +6 | 131 072 |
| 区间记录 24 字节 / 每次建 inode 的发布一条（C3rL 的另一种写法）；谱系记录 24 字节 | 1 359 | 183 465 条 | 2 个单元 | +4 | 98 304 |
| 同上，超过 183 465 条之后 | 1 359 | 24 767 775 条 | 3 个单元 | +6 | 131 072 |

出处：「container-tree-thresholds containers_for_height_1=135 records_for_height_1_16B=275265 records_for_height_2_16B=37160775 records_for_height_1_24B=183465 records_for_height_2_24B=24767775」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:52`）；形状对着真的 `after_a_container_is_opened_at_the_right_end` 逐个新开容器核（扇出 2、3、5、135，929 行）「container-shape-vs-real-probe lines=929 different=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:26`）；真函数 「pub fn after_a_container_is_opened_at_the_right_end(&self, entries_per_node: usize) -> Self {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/container_tree.rs:286`）。每次追加重写的码 2 节点 = 最右路径上每层一个（「最右那一条路径上的节点（每一层最右那一个），按 bump 次序（层级 1 先、根最末）：最右那片容器被重写时 COW 的祖先就是它们」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/container_tree.rs:265`）；有容器被重写时交回的就是它：「self.shape.rightmost_path()」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/container_tree.rs:380`）），再加最右那片容器。每条记录在盘上占 「container-tree-storage-per-record bytes_one_copy_16B=16.07 bytes_two_copies_16B=32.14 bytes_two_copies_24B=48.22」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:53`）（容器装满才切，左边不再碰，所以不像记账树那样浪费一半）。

**表 C：三个形态每个建 inode 的发布多写（按「已建过的 inode 数 / 已留台阶行数」读）**，一次发布建一个 inode 的最坏情形；一次发布建 n 个 inode 时 C2aL 与区间写法的 C3rL 只留一行，每个 inode 一条的 C3rL 留 n 条：

| 已建 | C2aL（记账载体）多的单元 / 写 | C3rL（每 inode 一条）多的单元 / 写 | C4g |
|---|---|---|---|
| 100 | +0 / +0（记账 1 层） | +2 / +4（码 2 1 层） | 0 / 0 |
| 462 | +0 / +0（记账 1 层） | +2 / +4（码 2 1 层） | 0 / 0 |
| 1000 | +2 / +4（记账 2 层） | +2 / +4（码 2 1 层） | 0 / 0 |
| 10000 | +2 / +4（记账 2 层） | +2 / +4（码 2 1 层） | 0 / 0 |
| 100000 | +4 / +8（记账 3 层） | +2 / +4（码 2 1 层） | 0 / 0 |
| 1000000 | +4 / +8（记账 3 层） | +3 / +6（码 2 2 层） | 0 / 0 |

出处：「per-create-publish-extra creates_so_far=1000 C2aL_extra_units=2」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:56`）；「per-create-publish-extra creates_so_far=100000 C2aL_extra_units=4」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:58`）；「per-create-publish-extra creates_so_far=1000000 C2aL_extra_units=4 C2aL_accounting_height=3 C2aL_extra_writes=8 C3rL_extra_units=3」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:59`）。
回退那次发布：三个形态都不多写单元（谱系容器本来每次回退都追加一条、都重写）；记录 16 → 24 字节，一个容器装 2 039 → 1 359 条，10 万次回退 74 个容器对今天的 50 个：「growth-after-n-rollbacks n=100000 lineage_records=100000 containers_24B=74」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:72`）。C2aL 在取法一下回退跳升那次多留一行台阶行（记账树照表 A）。

**崩溃状态数**（E77 闭式 1 + Σ(2^|段| − 1)，段按写数算）。基数取登记过的第一个文件事务流：「segments=24+2+1+2 closed_form=16777223」（`.claude/kb/experiments/142-新池新建文件的干跑.md:97`）（一般建 inode 的发布的段序列没有登记，所以只写倍数）。每多一个单元多 2 次写，状态数乘 4：

| 多的单元 | 段序列 | 闭式 | 倍数 | 谁 |
|---|---|---|---|---|
| +0 | 24+2+1+2 | 16777223 | ×1.0 | C4g；C2aL（≤ 462 行） |
| +2 | 28+2+1+2 | 268435463 | ×16.0 | C3rL（≤ 275 265 条）；C2aL（463 – 36 073 行） |
| +3 | 30+2+1+2 | 1073741831 | ×64.0 | C3rL（超过 275 265 条） |
| +4 | 32+2+1+2 | 4294967303 | ×256.0 | C2aL（36 074 – 2 742 509 行） |
| +6 | 36+2+1+2 | 68719476743 | ×4096.0 | C2aL（≥ 2 742 510 行） |

出处：「crash-states-with-extra-units extra_units=2 extra_writes=4 segments=28+2+1+2 closed_form=268435463 ratio_to_base=16.0」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:64`）；闭式的实现 「/// E77（发布的持久顺序） 的闭式：崩溃状态数 = 1 + Σ(2^|段| − 1)，|段| 按写数算。」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-harness/src/segments.rs:201`）。

**盘上增长（两份副本，一次建一个 inode 一次发布）**：10⁶ 次：C2aL 137 134 080 字节（4 185 片叶），C3rL 32 178 176 字节（491 个容器），C4g 0：「growth-after-n-create-events n=1000000 C2aL_step_rows_if_one_publish_per_create=1000000 C2aL_leaf_units=4185」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:69`）。台阶行不能压：「step-rows-cannot-be-merged example=两行 (10,5),(20,9)；合法对象 (号 6, 出生代 15)；并成 (10,9) 后 ② 判红 rows_kept_per_watermark_change=1 rows_covered_by_an_earlier_row=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:74`）。
C2i 按实例数长：每天崩 10 次、十年 36 500 个实例（「每天崩 10 次、十年 = 36 500 次」（`.claude/kb/decisions/23-journal的角色与格式.md:251`）），第 463 个实例（约 46 天）起记账树就是 3 个节点，36 074 个（约 10 年）起 5 个；每天 1 次则第 463 天起。
一次检查要读的量（推的）：C2aL 读全部台阶行（34 字节 × 行数）；C3rL 读全部发号记录并拿最新树里每条记录去查（16 字节 × 创建过的 inode 数，10⁶ 个 inode 约 16 MB 一份副本）；C4g 读全部谱系记录（24 字节 × 回退次数）。

### 3.8 要开例外的已定项与不变量（逐条对着 kb 现查）

| 要动的条款 | C2aL（记账载体） | C3rL | C4g | C2i（记账载体） |
|---|---|---|---|---|
| D5 已定项 2「每条根的记账树只装这条根那一代的行，不留多代」 | 开例外（台阶行是更早代的行） | 不动 | 不动 | 开例外 |
| I-3.13「每一行的代段等于这一版的 txg」 | 开例外 | 不动 | 不动 | 开例外 |
| 谱系记录 16 字节（D18 已定项 24 第 5 条；组件 ①、权威态、不退出） | +8 字节（H_cur） | +8 字节（count） | +8 字节（W） | 不动 |
| 树表 11 条（I-9.16 ②④、格式常量） | 不动 | 11 → 12，多一种树 | 不动 | 不动 |
| 新不变量 I-9.30 与池级 checker（已实现 84 条 → 85，GPU 判器补一个内核） | 要 | 要 | 要（或并进 I-9.29 / I-9.18） | 要 |
| 水位带树维：谱系那一份按头存 | 单头；n 个可写头要 n 份 H_cur | 单头；n 份 count | 单头；n 份 W | 单头 |

出处：「每条根的记账树只装这条根那一代的行，不留多代」（`.claude/kb/decisions/05-快照-空间记账机制.md:47`）；「每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 checkpoint_txg」（`.claude/kb/invariants.md:155`）；「记录 (T_rb 8, T_old 8) = 16 字节」（`.claude/kb/decisions/18-块里携带什么信息.md:576`）；「打包记录类型 5（谱系记录：T_rb 8 + T_old 8，宽 16 字节，字段表见 D18（块里携带什么信息） 已定项 24） | 组件 ① | 权威态 | 不退出」（`.claude/kb/freeze-layer-membership.md:51`）；权威态参与冻结：「丢了就丢数据。参与格式冻结，是永久契约」（`.claude/kb/decisions/21-权威态与派生态的分界.md:188`）；「树表非空时进树表的十一种（② 那一串）各恰好一条」（`.claude/kb/invariants.md:304`），常量 「pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 11;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:188`）；「池级 checker（walk::check_pool_image）判 84 条」（`.claude/kb/invariants.md:12`），「pub const IMPLEMENTED_INVARIANTS: [&str; 84] = [」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/image.rs:98`）；水位带树维：「带（只为可写头维护）」（`.claude/kb/decisions/05-快照-空间记账机制.md:127`）。
谱系记录加宽是改权威态的字段表：格式冻结之前改字段表就行，冻结之后只能另立新的打包记录类型。三个形态都碰它，所以这一条不分高下；分高下的是 D5 已定项 2 / I-3.13（只有 C2aL、C2i 要）与树表 11 → 12（只有 C3rL 要）。C2aL 的台阶行若放进容器树而不是记账树，D5 / I-3.13 的例外免了，换成树表 11 → 12 与表 B 的代价（推的，没量检查）。
单头前提：三个形态的数都按一个可写头算。克隆头与多头归攻方腿的 K3，我没算；有 n 个可写头时谱系记录要带 n 个水位，记录宽不再固定。

### 3.9 第一轮正推留的 Q1–Q7

| 题 | 这一轮给的 | 出处 |
|---|---|---|
| 「Q1｜C1b 每次可写挂载为取 M_b 多读几条根」（`research/prompts/m3-w3-r1-sonnet-output.md:213`） | 作废：C1 在第二轮判决出局，没算 | 「C1 出局：站得住」（`research/prompts/m3-w3-r2-main-verification.md:21`） |
| 「Q2｜C2 的每次发布脏节点数与树高」（`research/prompts/m3-w3-r1-sonnet-output.md:214`） | 有量：放宽 I-3.13 的那一路，台阶行 ≤ 462 条 1 个节点，463 起 3 个，36 074 起 5 个（表 A，真规划器核过）；保留 I-3.13 的那一路要把整段台阶行每发布换成这一代的代，5 000 行时下一发布要重写 21 个节点（全树）、而放宽的那一路是 3 个 | 「accounting-keep-I-3.13-route history_rows=5000 tree_nodes_before=21 height_before=2 rewritten_nodes_in_next_publish=21」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:25`）；台阶行的代若随发布换，原来那一代存不下去（这一路在 34 字节条目里放不下 g），所以只剩放宽一路（推的） |
| 「Q3｜C3 的追加记录数、叶数与树高随创建发布数的增长」（`research/prompts/m3-w3-r1-sonnet-output.md:215`） | 有量：C3 的载体取谱系树同形的容器树（产品里最近的一种），不是第一轮假设的码 2 叶 677 条；容器 2 039 条、一层根 135 个孩子，275 265 条以内每个建 inode 的发布多 2 个单元，到 37 160 775 条多 3 个（表 B） | 「container-tree-thresholds containers_for_height_1=135 records_for_height_1_16B=275265 records_for_height_2_16B=37160775」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:52`） |
| 「Q4｜台阶压不压得下去」（`research/prompts/m3-w3-r1-sonnet-output.md:216`） | 压不下去：水位沿时间线严格上升，没有一行被更早的行盖住；两行并成一行会让夹在中间的合法对象判红。行数 = 水位变了的发布数，无界 | 「step-rows-cannot-be-merged example=两行 (10,5),(20,9)；合法对象 (号 6, 出生代 15)；并成 (10,9) 后 ② 判红 rows_kept_per_watermark_change=1 rows_covered_by_an_earlier_row=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:74`） |
| 「Q5｜各候选对第 0 层脚本流的闭式状态数增量」（`research/prompts/m3-w3-r1-sonnet-output.md:217`） | 有算：每多一个单元乘 4；C3rL ×16，C2aL 1 / ×16 / ×256 / ×4096，C4g ×1（表 C 下的表） | 「crash-states-with-extra-units extra_units=2 extra_writes=4 segments=28+2+1+2 closed_form=268435463」（`research/prompts/m3-w3-r3-sonnet-model/expected/cost-model.out:64`） |
| 「Q6｜K2 的合法重发一形」（`research/prompts/m3-w3-r1-sonnet-output.md:218`） | 模型上：P 臂 3.8 千万张合法镜像里含 220 478 次合法的跨被抛弃时间线重发，C2aL、C3rL、C4g、C2i、C2ig 误报都是 0（3.4）。真代码上第二轮攻方腿的原型做过（层 0 全域 65 548 个状态，262 192 张镜像，0 误报）；我复核不了（层 0 是 checker 档，不跑） | 「w3a_cross_abandoned_reissue_images=131072」（`research/prompts/m3-w3-r2-opus-model/expected/ab-merge-of-20.out:7`） |
| 「Q7｜C1 行回收之后丢的覆盖」（`research/prompts/m3-w3-r1-sonnet-output.md:219`） | 作废：同 Q1 | 同 Q1 |

### 3.10 可以交用户的代价表（每个数的出处在 3.2–3.9；没标「推的」的都是量或算出来的）

口径：环 4 × 长 7 × 12 个字母的模型空间为准（环 6、8、12 与深一层同向，见 3.2）；单头；「多的单元」对着今天每发布 1 个记账节点；C4g + C2i 里 C2i 的行放记账（放实例表行预留会被清扫删行，第一轮已判那一类出局）。

| 列 | C2aL | C3rL | C4g | C4g + C2i |
|---|---|---|---|---|
| 现有 I-9.29、I-9.6 都不红的真重发事件（138 372）漏 | 0 | 0 | 0 | 0 |
| 同上的持久镜像（352 888）漏 | 0 | 0 | 0 | 0（C2i 单独 13 486） |
| V3 臂（恢复时少算 1）洞事件（76 552）漏 | 0 | 0 | 12 056 | 0 |
| 合法镜像误报（38 429 616 张，含 220 478 次合法跨抛弃重发） | 0 | 0 | 0 | 0 |
| 历史写错 / 不写，V1 臂 9 534 个真重发事件漏（A B C H Z R D Mh） | 9 534、9 534、0、0、0、0、0、0 | 9 534、9 534、0、0、0、0、0、0 | 9 534、9 534、9 534、9 446、9 534、0、0、0 | 9 534、9 534、9 534、0、0、0、0、0 |
| 每个建 inode 的发布多写的单元 / 写 | 0（≤ 462 行）；+2 / +4（463 起）；+4 / +8（36 074 起）；+6 / +12（2 742 510 起） | +2 / +4（≤ 275 265 个 inode）；+3 / +6（到 37 160 775 个） | 0 | C2i 按实例数：0（≤ 462）；+2 / +4（463 – 36 073）；+4 / +8（36 074 起） |
| 每次回退多写的单元；谱系记录 | 0；16 → 24 字节 | 0；16 → 24 字节 | 0；16 → 24 字节 | 0；16 → 24 字节 |
| 崩溃状态（基数 16 777 223，E142 的第一个文件事务流） | ×1 → ×16 → ×256 → ×4 096 | ×16 → ×64 | ×1 | ×1 → ×16 → ×256（按实例数） |
| 盘上增长，10⁶ 次建 inode，两份副本 | 137 134 080 字节（最坏，每次一个发布） | 32 178 176 字节（每个 inode 一条） | 0（每次回退 +24 字节） | C2i 每个实例 34 字节 |
| 要开例外的已定项与不变量 | D5 已定项 2、I-3.13、谱系记录宽、I-9.30 | 谱系记录宽、树表 11 → 12（I-9.16）、I-9.30 | 谱系记录宽、I-9.30 | 加 D5 已定项 2、I-3.13 |
| 依赖 Y1（第四节） | 取法一要写跳升行、谱系带 cur 的值；取法二都不要 | 取法一不能用「记录盖住水位」那一句；取法二能用 | 都不依赖 | 都不依赖 |
| 被攻过几轮 | C2aL：第二轮攻方腿 K2 一次抽样没打中；K3 的三处收严已并进定义 | 同左 | 第二轮辩方腿提、判决标「零轮」 | 零轮 |

读这张表：没有一个形态在所有列上占优。C4g 的代价最小（0 个单元、0 个崩溃状态倍数），洞上与另两个一样，但它是单存储：谱系那一份自己写错或不写时没有第二处可比（V1 + C、Z 全漏），也不管恢复时少算那一形（V3 漏 12 056）；C4g + C2i 补上 V3 与 H、Z，代价是 C2i 的实例行进记账树，每天崩 10 次的话约 46 天起每发布多 2 个单元。C3rL 与 C2aL 有互核，C3rL 每个建 inode 的发布恒多 2 个单元（崩溃状态 ×16），C2aL 前 462 条台阶行免费、之后同样 +2 并越长越多。

## 四、K2：Y1 两种取法

### 4.1 今天的条款、代码和模型里的跳升

- 条款：「| 水位 | inode 号水位与树 ID 水位 = max(cur 那一版内存里的水位, 环里读得出的根的水位)，不跟着 R_old 退回去 |」（`.claude/kb/decisions/23-journal的角色与格式.md:385`）；候选集那一格按 cur 的实例表滤被抛弃的根：「根环里按 cur 的实例表判仍然有效」（`.claude/kb/decisions/23-journal的角色与格式.md:382`）。
- 射程：「含管理员回退抛弃的那一段（回退那次发布的水位取 max，那一段里发过的号不重发）」（`.claude/kb/decisions/08-核心索引结构.md:207`）、「不含崩溃恢复抛弃的那一段（崩溃恢复落到较旧的根之后，被抛弃那一段里发过的号可以重发）」（`.claude/kb/decisions/08-核心索引结构.md:207`）。
- C143 定案里「回退那次取 max、不重发」那一格：「inode 号水位取 max(现行那一版内存里的, 环里读得出的根的)，不退回去、不重发」（`.claude/kb/checks-owed.md:125`）；当初为什么收严成取 max：「水位只取环里读得出的根时，现行那条根在回退那一刻读不出，号 2、3、4 被重发，收严成取 max」（`.claude/kb/checks-owed.md:125`）。代码里这一格的两道守卫：用例 「fn the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_its_root(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs:1540`），变异 「回退那次的 inode 号水位只取环里读得出的根，不与内存里现行那一版的取 max」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/mutations.tsv:717`）。
- 同一处代码里另两个取「环里全部根含被抛弃的」的量：树 ID 水位（「被抛弃时间线上的根也算，它们仍承载那一代发过的号」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1341`））与回退下界 F（「被抛弃时间线上的根带的 F 也算进来」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1570`））。inode 号是三个量里唯一一个条款允许被抛弃那一段重发的。

模型里的跳升（P 臂，环 4 × 7）：回退那次水位高过 cur、而高出来的部分只有被抛弃的根才有 = 55040 次；高水位来自时间线上的根 = 0 次；跳升之后的镜像 244560 张，占 10836269 张的 2.3%：「rollback_jumps_from_abandoned_roots_only=55040 rollback_jumps_from_a_root_on_the_timeline=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-0.out:56`）。

### 4.2 取法一：维持 D23 已定项 14（跳升留着，历史核对要吸收它）

**哪些读法会在合法镜像上误报**（P 臂；括号里是误报里「跳升之后」的张数，两个数相等 = 误报全在跳升之后）：

| 候选 | 环 4 × 7 从不清扫 | 环 4 × 7 阈值 0 | 环 8 × 7 阈值 0 | 环 8 × 11 字母表 4 个 |
|---|---|---|---|---|
| I-9.30 草案字面，台阶行只在建 inode 时留（C2a-create） | 163700（163700） | 152330（152330） | 268634（268634） | 249061（249061） |
| C3r 配草案 ③「记录盖住水位」 | 163700（163700） | 152330（152330） | 268634（268634） | 249061（249061） |
| 谱系带回退之后的水位（T_cur, H_new） | 231204（231204） | 218538（218538） | 356962（356962） | 521585（521585） |
| 谱系那一对去比只在建 inode 时留的台阶 | 56624（56624） | 57302（57302） | 96622（96622） | 192250（192250） |
| C2a-create 加「容许跳升」（③ 放过环里有被抛弃的根带着这个水位） | 81686（81686） | 99848（99848） | 0 | 7347（7347） |
| 台阶行每次水位变了都留（含回退跳升） | 0 | 0 | 0 | 0 |
| C2aL 最强形 | 0 | 0 | 0 | 0 |
| C3rL 最强形 | 0 | 0 | 0 | 0 |
| C4g | 0 | 0 | 0 | 0 |

出处：「candidate=I930-C2a-create false_positive=163700 … false_positive_after_a_jump_from_abandoned_roots=163700」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-0.out:39`）；「candidate=C3r-I930-3 false_positive=163700」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-0.out:23`）；「candidate=C2aL-pair-TcurHnew false_positive=231204」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-0.out:9`）；「candidate=I930-C2a-create-tol false_positive=81686」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-0.out:40`）；「candidate=I930-C2a-create false_positive=249061」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-tolring-0.out:39`）、「candidate=I930-C2a-create-tol false_positive=7347」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-tolring-0.out:40`）；最强形 「candidate=C2aL false_positive=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-0.out:7`）。前两行与第二轮攻方腿的数相同（163 700、231 204、56 624）：见 3.6 的逐格对账。

**「容许这一跳」那一句判据会不会顺带放过真重发。** 把「容许」写成判据 = ③（历史盖住最新根）在「环里有一条按实例表判被抛弃的根带着不小于这个水位的水位」时放过（候选名带 `-tol`）。量到三件事：
1. 单个故障不放过：V1 臂 340 776 个重发事件与 443 952 张持久镜像，三个 `-tol` 候选一个都不漏（「candidate=I930-C2a-create-tol false_positive=0 event_missed=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-2.out:40`））。
2. 叠上第二个故障（历史写错）就放过：现有检查都不红的 9 534 个真重发事件里，历史不写（C）时严格的 ③ 漏 0、容许的 ③ 漏 422；台阶行记成发布之前的水位（D）时严格 0、容许 386。「candidate=I930-C2a-create-tol … event_missed_where_existing_checks_silent=422」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-mutation-5.out:40`）；「candidate=I930-C2a-create-tol … event_missed_where_existing_checks_silent=386」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-mutation2-8.out:40`）；严格的对照 「candidate=I930-C2a-create false_positive=0 event_missed=0 … event_missed_where_existing_checks_silent=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-mutation-5.out:39`）。C2aL 加上谱系互核后只剩 C 那一列的 422（「candidate=C2aL-tol … event_missed_where_existing_checks_silent=422」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-mutation-5.out:12`）），D 那一列互核补回了。P 臂上被改过的镜像里不红的张数，C 一列从 203 941 涨到 305 376：「candidate=I930-C2a-all-tol … mutated_history_undetected=305376」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-mutation-2.out:38`）。
3. 而且它不够用：create 台阶那一形加了容许仍误报（81 686 张；环 8 长 11、历史长过环时 7 347 张），推的：环里那条被抛弃的根被后来的发布挤出环之后，判据就看不到它了——与环长、历史长度对得上（环 4 长 7、环 8 长 11 非 0，环 8 长 7 是 0），但我没逐个镜像核「被挤出」这一步；环 8 长 7 上是 0，只因为历史没长过环，不能当作它修好了。
**结论（取法一）**：吸收这一跳要靠把跳升写进历史（C2aL 每次水位变了都留行、谱系带 cur 的值），不能靠判据里写一句容许。写进历史的做法（I930-C2a-all、C2aL、C3rL、C4g）在 P 臂上误报 0，在 V1 臂上漏 0，在叠第二故障的格里也不比严格版差（C2aL 的 C 列 0，对比 tol 的 422）。
取法一要改的：`crates/` 0 个文件；kb 里 I-9.30 草案要写成「台阶行每次水位变了都留（含回退跳升）」「C3r 不用记录盖住水位」「谱系带 cur 的值、判等于」（这三条第二轮判决已并进形态定义）；D23 已定项 14 可以补一句「回退可以把水位抬过这条时间线没发过的号（跳号，不重发）」（推的，这一句现在没有）。

### 4.3 取法二：回退的水位只取没被抛弃的根（改成 max(cur 内存里的, 环里读得出且按 cur 的实例表判仍有效的根的)）

**今天按实例表判抛弃，表读不出、根读不出怎么办（读代码，不是猜）。**
- 表读不出：回退在取水位之前已经读了 cur 的实例表，读不出就在任何写之前拒（「.map_err(|_unreadable_or_malformed| RollbackError::CurrentInstanceTableMalformed)?」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7634`）），取法二不增加新状态；候选集那一格今天就是用这张表滤的。只有 `effective_rollback_floor` 那一类「留给单外的实验装置」的读法才有「表读不出就不按表滤」（`crates/singlefs-core/src/recovery.rs` 的文档注释），产品的回退不走它。
- 根读不出：本来就不算进水位（「Ok(RebuiltVersion::WithoutFile) | Err(_) => None,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7456`））。取法一下这让回退的水位取决于「那一刻读得出几条被抛弃的高水位根」（读不出的不抬）；取法二下被抛弃的根整条不进，水位不取决于它们读不读得出。cur 自己的根读不出时靠内存里的 cur：「ring.max(current.inode_number_watermark())」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7755`），取法二保留这一道 max。
- 实例表的行什么时候被清扫删：只在环里没有该实例的根时（「判根环每个槽都自证过、环里没有该实例的根」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:5`）），所以不会有「根还在环里、它的实例的行已经没了」而被误判成没被抛弃的根。

**改法（推的，没写进 `crates/`）**：`mount.rs` 里 `ring_roots` 读出来之后、算 `inode_number_watermark` 之前加一道滤——`ring_roots.iter().filter(|root| !root_is_abandoned_by_the_instance_table(root, &current_table))`，`current_table` 就是前面已经读好、候选集在用的那一张；树 ID 水位那一行（「.fold(current.root.tree_identifier_watermark, u64::max);」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7747`））仍取全部根。三行内的改动。判抛弃的函数是现成的：「pub fn root_is_abandoned_by_the_instance_table(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1542`）。
**更大的一种改法**：整段环扫描从 inode 号水位里删掉，只留 cur 内存里的。模型里时间线上的根从来没比 cur 水位高过（见下），环里那一半恒为空；删掉的好处是回退不再从盘上重建环里最多 24 条根（「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:279`），三个区域各一环）的版本，坏处是少了一道「内存里的 cur 水位被写低」的纵深防御，而且 `crates/mutations.tsv` 那一行变异的原文（取 `ring.max(current…)`）要换靶。不推荐，只记在这里。

**模型里的数**（P 臂历史空间与取法一同一份；取法二的臂 = `ProductWithoutAbandonedRoots`）：

| 量 | 取法一 环 4 × 7 从不清扫 | 取法二 同 | 取法一 环 4 阈值 0 | 取法二 同 | 取法一 环 8 阈值 0 | 取法二 同 |
|---|---|---|---|---|---|---|
| 字面 I-9.30（C2a-create）误报 | 163700 | 0 | 152330 | 0 | 268634 | 0 |
| C3r 配草案 ③ 误报 | 163700 | 0 | 152330 | 0 | 268634 | 0 |
| 谱系带回退之后的水位误报 | 231204 | 0 | 218538 | 0 | 356962 | 0 |
| C2aL、C3rL、C4g、C2i、C2ig 误报 | 0 | 0 | 0 | 0 | 0 | 0 |
| I-9.29t、I-9.6t（现有检查）误报 | 0 | 0 | 0 | 0 | 0 | 0 |
| 水位退回的镜像 / 时间线上重发的镜像 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| 合法的跨被抛弃时间线重发 | 48356 | 53960 | 35042 | 36326 | 71380 | 110172 |

出处：「candidate=I930-C2a-create false_positive=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-k2-0.out:39`）；「candidate=C3r-I930-3 false_positive=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-k2-0.out:23`）；「candidate=C2aL-pair-TcurHnew false_positive=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-k2-0.out:9`）；「timeline_reissue_images=0 watermark_regressed_images=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-k2-0.out:41`）；跨抛弃重发 「reissues_across_abandoned_roots=53960」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-k2-0.out:42`）（取法一 「reissues_across_abandoned_roots=48356」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-0.out:57`））、「reissues_across_abandoned_roots=110172」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-k2-2.out:42`）（取法一 「reissues_across_abandoned_roots=71380」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-defense-5.out:54`））。取法二把跳升那一段的号放回「可以重发」，合法的跨抛弃重发多 5 604 次（+11.6%）、环 8 阈值 0 上多 38 792 次（+54%）；它们都是 D8 已定项 6 射程外的重发，现有的 I-9.29（按实例表滤）与 I-9.6 在这些镜像上不红（上表第 5 行 0）。
时间线上的根比 cur 水位高的跳升：取法一与取法二两个臂都是 0（「rollback_jumps_from_a_root_on_the_timeline=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-main-0.out:56`）；「rollback_jumps_from_a_root_on_the_timeline=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-k2-0.out:41`）），所以取法二的滤把环里那一半滤成恒空——这是模型里穷举 12 个字母、环 4 / 6 / 8 的观测，产品里等价于「水位沿时间线不降」（发布路径每次只加，回退取 max，推的）。

**C143 那一格退没退回去。** 用含 `rollback-blind`（回退 1，算水位那一遍读不出 cur 自己的根）的字母表（8 个字母、环 4、长 7，471 952 张镜像），三个臂：取法一（今天）、取法二、控制臂（只取环里读得出的根、不与 cur 内存里的取 max——`mutations.tsv` 那一行变异的模型版）：

| 臂 | 水位退回的镜像 | 时间线上重发的镜像 | 重发事件 | 其中现有检查都不红 |
|---|---|---|---|---|
| 取法一（今天） | 0 | 0 | 0 | 0 |
| 取法二 | 0 | 0 | 0 | 0 |
| 控制臂：只取环 | 15596 | 11932 | 2086 | 656 |

出处：「timeline_reissue_images=0 watermark_regressed_images=0」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-blind-1.out:41`）；控制臂 「timeline_reissue_images=11932 watermark_regressed_images=15596」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-blind-2.out:249`），「events=2086 events_where_I-9.29_holds=656 events_where_existing_checks_silent=656」（`research/prompts/m3-w3-r3-sonnet-model/expected/model-blind-2.out:250`）。结论：取法二保留 `max(cur 内存里的, …)` 这一道，「回退那次取 max、不重发」那一格没退回去；控制臂证明这套穷举看得见那一格坏掉（15 596 张水位退回、11 932 张时间线上重发）。
现有守卫在取法二下仍有效：用例 `the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_…` 钉的是 `max` 里内存那一支，取法二不碰它；变异那一行改的是 `.map_or(current.inode_number_watermark(), |ring| {` 下一行的 `ring.max(current.inode_number_watermark())`（「ring.max(current.inode_number_watermark())」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7755`）），三行改法不动这一句。
取法二会不会让现有用例变红：`rollback_by_a_forward_publish.rs` 里涉及水位的断言只有下面四处（原样），它们都不是「被抛弃的更高水位根在环里读得出时回退」那一形——528–529 行的 cur 是第三版自己，1022、1068 行是 cur 的根槽自证不过被拒、改回后回退：
```
$ grep -n 'inode_number_watermark()' crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs   （冻结树里）
528:        rollback_version.inode_number_watermark(),
529:        third.inode_number_watermark(),
1022:    assert_eq!(third.inode_number_watermark(), 5);
1068:        pool.output.inode_number_watermark(),
```
再扫一遍：冻结树里 `singlefs-harness`、`singlefs-checker-tier`、`singlefs-core`、`singlefs-reference-filesystem` 下含回退调用（`roll_back(`、`roll_back_by_a_forward_publish` 等）的 .rs 文件里，同时含 `abandon` 与 `inode_number_watermark` 的只有三份：`mount.rs`（源码）、`checker_known_bad_images.rs`（1 处，是 3019 行一个用例名字符串）、`rollback_by_a_forward_publish.rs`（5 处，就是上面四处断言加 989 行一句文档注释）。所以没有现成用例钉着取法一的跳升；取法二要新加一条变异（回退水位不按实例表滤）与一条必红用例（被抛弃的高水位根在环里读得出时，回退之后的水位等于 cur 的）。以上是读代码与 grep，没跑 harness 用例。

### 4.4 两种取法并排

| 项 | 取法一：维持 D23 已定项 14 | 取法二：只取没被抛弃的根 |
|---|---|---|
| 要改的 `crates/` | 0 个文件 | `mount.rs` 里加一道滤，三行内（推的）；整段环扫描删掉是更大的改法，不推荐 |
| 要改的 kb | D23 已定项 14 补一句「回退可以把水位抬过这条时间线没发过的号（跳号，不重发）」；I-9.30 草案按第二轮判决的三条收严写（台阶行每次水位变了都留、C3r 不用「记录盖住水位」、谱系带 cur 的值判等于） | D23 已定项 14 的「水位」一格拆成两句（inode 号水位滤、树 ID 水位仍取全部根）；`crates/mutations.tsv` 加一条变异、harness 加一条必红用例 |
| 字面读法在合法镜像上的误报（环 4 × 7 从不清扫） | I-9.30 字面 163700、C3r 配 ③ 163700、谱系带 H_new 231204，全在跳升之后；C2aL、C3rL、C4g 的最强形 0 | 三个字面读法都 0（0、0、0），最强形 0 |
| 「容许跳升」写成判据 | 单故障不放过；叠上历史不写放过 422 / 9 534、台阶记错放过 386 / 9 534；create 台阶那一形仍误报 81 686（环 4）/ 7 347（环 8 长 11） | 不需要这句 |
| C143「回退那次取 max、不重发」那一格 | 保持（跳升只把水位抬高） | 保持：含 rollback-blind 的字母表上水位退回的镜像 0（控制臂 15 596） |
| 合法的跨被抛弃时间线重发 | 48356（环 4 × 7 从不）；71380（环 8 × 7 阈值 0） | 53960（+5 604）；110172（+38 792） |
| 回退后的水位取决于什么 | 取决于那一刻读得出几条被抛弃的高水位根（读不出的不抬） | 只取决于时间线（cur 内存里的值） |
| 与树 ID 水位、回退下界 F 的取法 | 三个量一致：都取环里全部根 | 不一致：inode 号水位滤，另两个不滤；条款依据是 D8 已定项 6 射程只允许 inode 号在被抛弃的那一段重发 |
| 对三个形态的影响 | C4g、C2i 不依赖 Y1；C3rL 不能用草案 ③；C2aL 要在跳升那次留行 | 三个形态的字面读法都能用；C2aL 只在建 inode 的发布留行即可（跳升行省掉，行数基本不变）；谱系带 cur 的值那条纪律仍要（第二轮攻方腿 K3 的结构论证，防的是 A + H 不是跳升，推的） |
| 被攻过几轮 | 条款本身是前两轮的现状；这一轮的「容许判据放过 422」是我的模型量的，零轮 | 零轮（攻方腿第三轮要攻） |

取法二的前提是 D8 已定项 6 的射程：被崩溃恢复抛弃的那一段里发过的号可以重发。被抛弃的根是系统配置没见证到的发布（推的：「每次发布轮换系统配置槽、返回前等它持久」（`.claude/kb/decisions/23-journal的角色与格式.md:136`），所以没被见证 = fsync 没有返回，它们的号没有承诺给过任何人），放掉它们的号不违反「已发布的 inode 号不复用」；条款一侧是 「不含崩溃恢复抛弃的那一段（崩溃恢复落到较旧的根之后，被抛弃那一段里发过的号可以重发）」（`.claude/kb/decisions/08-核心索引结构.md:207`）。这是读条款与推理，不是我量的；要不要把它也当 Y1 的依据交用户定。

## 五、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 C2aL 洞 / 误报 | 一致 | 环 4 × 7 的 138 372 个洞事件与 352 888 张持久洞镜像漏 0，P 臂 38 429 616 张合法镜像误报 0；V1 臂 FP 列不读 |
| K1 C3rL 洞 / 误报 | 一致 | 同上两格 0；V2 臂是唯一抓得到 V2 事件的（不在 W3 射程里） |
| K1 C4g 洞 / 误报 | 一致 | 同上两格 0 |
| K1 C4g V3 臂 | 冲突 | 恢复时少算 1 那一形漏 12 056 / 76 552 个洞事件、149 416 / 190 236 张持久洞镜像；C2aL、C3rL、C4g + C2i 漏 0 |
| K1 历史写错（八种变异） | 冲突（C4g）/ 一致（C2aL、C3rL 的六种） | 单存储的 C4g 在 C、Z 上漏全部 9 534、H 上漏 9 446；两处互核的 C2aL、C3rL 在 C、H、Z、R、D、Mh 上漏 0；A、B 三者都漏全部 |
| K1 每次发布多写 | 一致（有数） | C4g 0 单元；C3rL 恒 +2（275 265 条后 +3）；C2aL 前 462 条 0、463 起 +2、36 074 起 +4、2 742 510 起 +6；规划器那一半对着真规划器核过 |
| K1 崩溃状态数 | 一致（有数） | 基数 16 777 223；+2 单元 ×16，+4 ×256，+6 ×4 096 |
| K1 要开例外的已定项 | 一致（有表） | D5 已定项 2、I-3.13（C2aL、C2i）；谱系记录宽 16 → 24（三个都要）；树表 11 → 12（C3rL）；I-9.30 与 checker 84 → 85（三个都要） |
| K1 前两轮四套模型能不能并排 | 一致（三套抽象模型）/ 冲突（一处报数）/ 规则没说（装置） | 抽象模型 22 + 18 个文件逐数相等；攻方腿 A + H 的 P 臂数与自己的产物对不上（C2aL 169 763，不是 336 805）；装置数与模型不在一个空间 |
| K1 Q1–Q7 | 一致 | Q1、Q7 作废（C1 出局）；Q2–Q5 有量有算；Q6 模型上 0 误报、真代码上的原型我复核不了 |
| K2 取法一：容许跳升的判据 | 冲突 | 单故障不放过，叠上历史写错放过 422 / 9 534 与 386 / 9 534，而且 create 台阶那一形仍误报；吸收跳升要靠写进历史 |
| K2 取法二：表读不出 / 根读不出 | 一致 | 表读不出回退本来就在写之前拒；根读不出本来就不算；被抛弃的根整条不进，水位不取决于读不读得出它们 |
| K2 取法二：C143 那一格 | 一致 | 保留 max(cur 内存里的, …)；含 rollback-blind 的字母表上水位退回 0 张，控制臂 15 596 张 |

## 六、什么现象会推翻上面的结论

- 「三个形态在洞上没有差别」：在 V1 臂找到一条历史，C2aL、C3rL、C4g 任一在现有检查都不红的事件上不红（模型 138 372 + 121 336 个洞事件、352 888 + 440 672 张洞镜像里现在是 0）；或把 C2aL / C3rL 并进 E175 装置的影子之后，装置上的 2 个洞事件、258 个洞步里有一个不红（我没做）。
- 「C4g 管不到 V3、单存储互核不了」：C4g 在 V3 臂洞事件上漏数不是 12 056；或 C4g 在 V1 + C、Z 上红。
- 「合法镜像误报 0」：P 臂 38 429 616 张里找到一张三者任一红的；或第二轮攻方腿的原型（层 0 全域）在同一形上红。
- 「C2aL 463 起 +2 单元」：真规划器在产品容量下第 464 个发布的重写数不是 3（探针现在是 3），或第 36 075 个不是 6 / 5（现在是），或第 2 742 511 个不是 9 / 7（现在是）。
- 「C3rL 恒 +2 单元」：实现把发号记录和它的码 2 根放进同一个单元，或把发号记录挂进记账树（那就退成 C2aL 的一行）。
- 「容许跳升的判据放过真重发」：严格的 ③ 在同样的 9 534 个洞事件里 V1 + C 漏数不是 0；或容许版漏 0。
- 「取法二没让 C143 那一格退回去」：含 rollback-blind 的字母表上取法二臂水位退回的镜像不是 0；或控制臂是 0（那就是这套穷举看不见那一格，取法二的 0 不算数）。
- 「取法二的滤是三行」：`current_table` 在算水位那一处已经被挪走 / 出了作用域（读代码，冻结树里它在候选集之后仍可用）。
- 「四套模型的数能并排」：`rerun.sh compare-r2` 或 `compare-r2opus` 有一格不同；「攻方腿 A + H 数报错」：重跑攻方腿 `model-rest` 得 C2aL-strongest 的 mutated_history_undetected 是 336 805。

## 七、没做什么

- K3（克隆头、副本坏一份、新实例第一个 txg 与被抛弃发布撞出生代）是攻方腿的，我没碰；不替主 agent 采纳或出判决——3.10 只列数，「没有一个形态在所有列上占优」是读表，不是推荐。
- 没跑重型测试：层 0 枚举、checker 档包的测试、全量 `cargo test`、门禁都没碰；第二轮攻方腿的层 0 原型（65 548 个状态）我复核不了，只引它的产物。探针编的是冻结树**副本**里 `singlefs-core` 的 lib 测试（不是 checker 档），经内存包装跑；没跑 harness 的用例，4.3 里「现有用例不钉取法一的跳升」是读代码与 grep。
- 没写 `crates/`：C2aL、C3rL、C4g、C2i 的写者与读者、取法二的那三行滤都是「以后加在哪一处」，没编、没测；数全是抽象模型、冻结树副本上的探针和算术，进 kb 之前要在入库装置上重做。
- 没把 C2aL、C3rL 并进 E175 装置的影子：3.6 里装置那一行用的是第二轮辩方腿的装置数，其中 c2a、c3 是构造出来的上界，只有 c4g、c2i 是真规则。装置上的 2 个洞事件、258 个洞步没有 C2aL / C3rL 的数。
- 联动没展开（推的）：每多写一个码 2 / 码 3 单元，中央映射树多一条映射条目、分配记录多一次落点、journal 点名项多一个（一条记录装 64 个）；C2aL（≥ 463 行）与 C3rL 的 +2 单元各带这三样，C4g 没有。我只数了单元、写和字节。
- 没量一次检查的读量（3.7 末的三句是推的）、没量克隆头 / 多头（水位带树维，谱系那一份要按头存）。
- 环：模型环 4、6、8、12，历史长 ≤ 8（环 8 的容许跳升那一格取长 11）；产品环（每区 8 槽、三个区域）没枚举。
- 规划器：真规划器核到第 2 742 512 个发布（先装 H 个台阶行再接着走）与容量压小的三档各 3 000 个发布；第 5 层（208 431 646 行起）是复刻加右边缘模型，没用真规划器核。
- 「容许跳升」的 `-tol` 是我写的一种读法（③ 放过「环里有被抛弃的根带着这个水位」）；条款里没有这句判据，别的写法（例如 ② 也放宽）没量。
- 抽样：模型是确定性穷举，不是抽样；但我并进来的候选与新臂（`-tol`、`ProductWithoutAbandonedRoots`、`RollbackRingOnly`、`rollback-blind`）只被我自己的模型量过，攻方腿没攻过，按零轮算。
- 没复核第二轮报告里别的数：已核的是辩方腿 22 个文件 441 格与攻方腿 18 个文件 1 299 格（3.6），只发现 A + H 那一处对不上；攻方腿的装置数与原型数我没复现。

## 八、环境与草稿产物

- 开跑前 `ps -o pid,args -u "$(id -u)"` 看到别的会话的 `cargo test --offline -p singlefs-harness --test opus_r3_leaf_death_images opus_r3_right_half -- --nocapture --test-threads 1`（pid 521937），没有 qemu-system、vm-bench、e152-file-system-benchmark、fio；中途另见一条别的腿的 `rerun.sh` 在跑（只看了命令行，没读它的任何东西）。我的编译与跑都加了 `nice -n 19`、经内存包装；负载 loadavg 20–40，有几段在包装里排队（`model-tolring` 墙钟 3 分 15 秒、CPU 35 秒；`model-k2` 墙钟 1–2.5 分）。
- 删掉的编译目录与仓副本（`du -sh` 先记的大小，原因：交回闸要求删；要复核跑 `rerun.sh`）：
  - `/home/fy5090/.local/share/singlefs/work/m3-w3-r3-sonnet/probe/tree` 21M（冻结树副本，接了探针）、`/home/fy5090/.local/share/singlefs/work/m3-w3-r3-sonnet/probe/target` 153M（它的编译目录）；
  - `/home/fy5090/.local/share/singlefs/work/m3-w3-r3-sonnet/target-model` 704K（开发时模型的编译目录）；
  - `/home/fy5090/.local/share/singlefs/work/m3-w3-r3-sonnet/rerun1/probe` 173M、`/home/fy5090/.local/share/singlefs/work/m3-w3-r3-sonnet/rerun1/target-model` 704K（第一遍空目录复跑）；
  - `/home/fy5090/.local/share/singlefs/work/m3-w3-r3-sonnet/rerun2/probe` 174M、`/home/fy5090/.local/share/singlefs/work/m3-w3-r3-sonnet/rerun2/target-model` 704K（最终空目录复跑）；
  - `/home/fy5090/.local/share/singlefs/work/m3-w3-r3-sonnet/rerun3/target-model` 704K（交回前对 README 与 `PYTHONDONTWRITEBYTECODE` 两处小改的快速复核）。
- 留着的：草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-w3-r3-sonnet/`（`out/` 11M：开发时各组模型的原始输出含 W3TRACE；`rerun1/out`、`rerun2/out`、`rerun3/out`：三遍复跑的输出；`seg/`、`seg*.py`、`mkrep.py`：生成本报告各段的脚本，每处引文由它现查行号；`model/`：入库模型目录的开发时旧副本，以仓里为准）。没入库的原因：原始输出里的逐步打印行太大，入库的 `expected/` 已去掉 W3TRACE 行；这些脚本只是写报告用的，不进仓。

## 九、交回前核对

模型目录 `research/prompts/m3-w3-r3-sonnet-model/` 每个文件的 sha256（`find . -type f | LC_ALL=C sort | grep -v '^./expected/' | xargs sha256sum`；`expected/` 47 个文件逐个 sha256 再连起来的 sha256 是 5b7bb9ed14900ab9d6dbb6ce8ca512f5f8d0456bf3f30698bd9a4566b243a91b）：

```
ad1b33bdae84d2ca1f1b09d5b1aebcae1e8c7747f4ffe34b0614d6e1ab8314d9  ./README.md
43e4fe3403027f2f7e9ee7ad08e98a19b1c9a93c943198c038eb778a155183ef  ./compare.sh
0c711706b9753c6b57f32d112c4f8dc017788f2b6c1eff2688783522fd4c3f9d  ./cost/check_constants.py
205eac9f7de1ce9b4b9f4bdf09f7833eb2926827e2589e8288f54e0ca1ae7c94  ./cost/cost_model.py
0968e2836874d58fbb406bb687fb05fcf885991ac141ca5c9bb6ad0ebb4def0c  ./cost/planner.py
7a95eb033e23edd1d780538b4f409fade702a9a3a9b31c7656e12c01233a13d3  ./cost/probe_compare.py
ebce408699f5ef765c4d2eed466e8eab1e05b307e8fb3aa1b551bbd47c9e6743  ./probe/code_two_tree_probe.rs
7eec56b2bf3cabdd68880522e400df392cc6f34cf519be69ba4abf78bc5814d8  ./probe/container_tree_probe.rs
5c533f6d73cbe97778901cafb3558af34296f9711738c45039e7470e61c75746  ./rerun.sh
6e443248d2142b608a9da47006d865053985fc4c9d7e8d2b1b0755105897707a  ./timeline-model/Cargo.lock
45f5f9e0d78123de821b39e79861f580e762cb6c6b9f6f0b9cc7d220b48b9b86  ./timeline-model/Cargo.toml
c7aad8ce2ddd1e6618d82632b9c76d2a72ec859c28a05cdd1df9365c92e6c027  ./timeline-model/src/main.rs
f5dd7aa73d204807a02786b31bac39b56c3d8d7f8fbb58ac03f5a501a2026aa3  ./tools/compare_r2.py
43fb1e565f868e99d5a5148d57813488feb9e344bbdff1d321d28a683cc2be4d  ./tools/compare_r2opus.py
3e03e1869cf407dc30c36b911d0f07289df96d25d28ad98ce3bb2d6555469df5  ./tools/mut_table3.py
3f6a0d9df5280a8289c830a290b33e8ce2367f817fefbc3a6621804370248e0e  ./tools/report_tables.py
972922008fe2d8adbb9367acfa520cb4a6f7dae9955b35b87a6693c2f2fb568f  ./tools/table2.py
```

空目录从头跑通：`W=~/.local/share/singlefs/work/m3-w3-r3-sonnet/rerun2`（交回前删了它的编译目录，输出留在 `rerun2/out`），按第一节的 19 段顺序各一条前台命令（`model-main`、`model-defense`、`model-k2`、`model-tolring` 各 1.5–3 分钟，别的在 1 分钟内），最后一段 `compare` 原样：
```
$ bash $M/rerun.sh $W compare 2>&1 | tail -3
same probe-compare-red.out
compared=47 status=0
段 compare 跑完
```
之后只改了三处：`rerun.sh` 开头加 `export PYTHONDONTWRITEBYTECODE=1`、README 里两句（`mut_table3.py` 的说明与依赖的外部东西）；在另一个空目录 `rerun3` 里重跑 build-model、model-blind、model-mutation、model-mutation2-p、model-mutation2-v1、cost-check 六段再 `compare`：22 个文件 `same`、0 个 `DIFFERENT`（其余文件在那个目录里没跑，`compare` 报缺，不算比过）。`expected/` 里的模型输出是开发时第二遍（alphabet 标签与 D 变异口径改过之后）整理的，不是复跑那一遍，所以复跑是对着它比；`probe-compare.out`、`probe-small-real.out`、`probe-shape.out`、`probe-bulk.out` 与 `report-tables.out` 是确定性输出。

交回前跑 `python3 scripts/agents/cite-check.py research/prompts/m3-w3-r3-sonnet-output.md` 的末行：  ✓ 核了 167 处引文，对上 167 处，没判 0 处（逐处列在上面）
