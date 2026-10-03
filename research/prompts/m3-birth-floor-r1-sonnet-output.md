# m3-birth-floor-r1 正推腿（Sonnet）报告：F1–F3（2026-10-02）

<!-- doc-lint:not-numbers V1 F1 F2 F3 -->

批次：records/2026-09-30-里程碑三开工第一轮.md#4。立场：正推。读冻结副本 `/tmp/claude-1000/m3-batch4-freeze/tree/`；引 kb 写 kb 文件自己的行号（在冻结副本里现查）。

这是接续腿。前一条同立场腿撞限额，报告没落盘，只留下 `m3-birth-floor-r1-sonnet-model/birth_floor_model.py`（262 行，能跑）。我核过它：回退目标只从最旧端数、没有实例表单元、没有挂载 / 抛弃最新根 / 丢根槽、没有随机长历史、没有坏镜像层。它的核心判法我沿用，另写 `birth_floor_v2.py` 补上这些；旧文件不改、不引它的数。

## 先说结论（都是模型上推的，真实镜像上没造没跑）

1. **F1**：上一轮两份红的镜像，今天 harness 能走到的全是同一类——回退根把回退目标的 inode 树与 extent 树两棵根指针整个照抄（我称 A 类）。正推腿的 379 条历史 393 个红单元，逐条核：R_first 全是回退根、全是整份照抄。攻方的 173 张只有汇总数、拆不开，按「harness 历史没有建 inode 这一步、回退照抄两棵树」推也是 A 类（推的）。不是 A 类的两种是**今天 harness 走不到、`crates/` 的 API 能走到**的：B 类（建 inode 之后回退：只有一棵树从候选集外继承）与 D 类（回退根的根槽读不出之后，接在它后面的非回退根成了 R_first）。
2. **V1 的三种写法在这两类上分开**：攻方原型的写法（只看 inode 树根指针、整根豁免）在 D 类上红；「两棵树都继承才豁免」在 B、D 两类上红；**按树分摊的写法（豁免只给继承来的那棵树的子树里的单元，指针出生代号严格小于 R_prev 的 txg）在我跑的全部合法镜像上零红**（`run_all.out` 里穷举加随机共 2 407 721 张，含建 inode、里程碑三的多文件、丢根槽、抛弃最新根、走过根环；故障档拆开另跑在 `run_fault_split.out`）。
3. **只红它的坏镜像造得出（设计）**：从现成的「覆盖写之后」镜像出发把新数据单元的诞生代号盖到 R_prev 的 txg，按 `checker_known_bad_images.rs` 里 I-4.2 那份的四处同改。攻方写法漏、按树分摊写法不漏的坏镜像也有：回退根自己的固定点单元（树表单元）盖小——模型里 78 575 次盖小，攻方写法 0 次抓到、按树分摊 78 575 次全抓到。
4. **按树分摊写法的代价**：整条引用链（树根指针和树里的单元一起）盖小 ≥ 2 而固定点单元不盖，它抓不到（模型里 8 418 次中 0 次）；盖小 1 抓得到，靠的是严格小于。这条漏洞只有回退根留一个盘上标记才能收：D18 已定项 24 已定「回退那次发布追加一条谱系记录」，`crates/` 里今天没有（grep 零命中，下面 F2），里程碑三三·3 才实现。
5. **F3**：「引用集合连续无洞」与「回退是一次向前发布」是真冲突，而且比下界早红：两步历史（覆盖写一次、回退到第一版）上，第一版的五个单元在候选集里「引用—不引用—引用」。改法：把 ② 的下界与 ① 的连续合成**一条出现规则**——候选集按 txg 升序，相邻两条根 (R⁻, R)，R 引用而 R⁻ 不引用的每个单元 b，要 birth(b) > R⁻.txg 或 b 在 R 的某棵「继承来的」用户可见树里。模型里它与「② 的按树分摊写法 ∪ ① 的放宽写法」逐镜像相等（50 834 张，0 张不同），合法镜像零红。

判据对照（正文第五节，采不采归主 agent）：V1（按树分摊写法）在任一合法镜像上红——没找到；V1 写不出只红它的坏镜像——写得出；① 改法让合法镜像红——不红。

## 复跑与文件

```text
cd research/prompts/m3-birth-floor-r1-sonnet-model && nice -n 19 python3 run_all.py > run_all.out   # 单核约 2 分半，纯 Python，不读 crates
python3 classify_prior_sonnet_379.py                                                              # F1：上一轮 379 条的归类
```

```text
2bae0b6eb443300ab3e3cc4cef46a3dddd11dcdf93447dd2a5d400bc7c587282  birth_floor_v2.py
7a3e3a65ce33f763d28de69b2be94fd708b225090308159ba185cf0d81a8a386  run_all.py
01508af203dee9dc9b9f130fafc55400f73a4bc31df93e4efdb428db0f6cde3a  run_all.out
d4747212b24d450a97091041b06c53eb82f214234646121d9ad8a85b4c7474a9  classify_prior_sonnet_379.py
b16933155fcd4829fe444e8a421debc56c99f118f703b1cb99977afffc636cbf  classify_prior_sonnet_379.out
32e34f0f40a3a02654b5fde4393e08c05061ecfb0c038bca40edfdfdafdc516d  birth_floor_model.py（前任落盘，已被 v2 取代，不引它的数）
183fa7b777585a5e96c01b31241ea4dfa7ac7e354e921d45e155aa4cb3efb8e0  run_fault_split.py
b6b883335a80be9a1d1853fe03381e5cf40ccd8a98773576cd5598023f932090  run_fault_split.out
```

### 模型和 `crates/` 今天的实现怎么对（先读实现，改法写成对实现的改动）

模型是抽象的：单元 = (类, 诞生代号, 子单元)，根 = (txg, 操作, inode 树根, extent 树根, 实例表单元)，引用 = 固定点单元（每次发布新写）+ 两棵树的子树 + 实例表单元。模型的操作对着实现的哪一处：

| 模型操作 | 实现 | 对不对得上 |
|---|---|---|
| W 覆盖写 | `publish_overwrite`（`crates/singlefs-core/src/transaction.rs` 4746 行）：「改动计数取这次的 txg」（`crates/singlefs-core/src/transaction.rs:4739`），inode 树、extent 树、数据单元都换新 | 对得上 |
| N 建 inode | `publish_new_inodes`（同文件 4859 行）：「inode 叶容器与 inode 根重写，数据单元与 extent 树照抄」（`crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs:48`）；`crates/singlefs-core/src/mounted_session.rs` 的入口里有（33、57 行），但 harness 的历史生成器没有这一步（`crates/singlefs-harness/src/history.rs:367` 的 `HistoryOperation` 八种里没有） | 对得上；harness 走不到 |
| R 回退 | `roll_back_by_a_forward_publish`（`crates/singlefs-core/src/mount.rs` 5237 行）；新根树表只把 extent、inode 两条的根指针换成目标的：「if entry.kind == TREE_KIND_EXTENT」（`crates/singlefs-core/src/mount.rs:5162`），实例表照现行：「instance_table: InstanceTablePlan::Carry(current.root.instance_table),」（`crates/singlefs-core/src/mount.rs:5433`）；`deleted_inodes` 今天零根、这一步没有 | 对得上；kb 说的 `deleted_inodes` 今天没有 |
| F 抬 F | `mount.rs` 的 `newest_first_distinct_states`（1966 行）、`ceiling_from_newest_and_non_empty_roots`（1984 行）：第 4 新的去重非空根 | 对得上 |
| 候选集 | `crates/singlefs-checker/src/walk.rs`：「let walked = *index == newest_index」（`crates/singlefs-checker/src/walk.rs:5977`）起的那几行：被抛弃的、低于 F 的不进，最新根无条件进 | 对得上 |
| 根环容量 24 | `crates/singlefs-format/src/lib.rs` 的 `ROOT_RING_REGIONS`（245 行，3）乘 `ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`（263 行，8） | 对得上 |
| K / X / L | 可写挂载写行加暖机；抛弃最新根（harness 的 `CrashRecoveryAbandoningTheNewestRoot`）；根槽读不出（故障）。条数是抽象，不是实现 | 抽象，只验「不误红」 |

## F1　下界在哪类合法镜像上红

### 分类口径

下界红 = 单元 b 在候选集里第一次出现于根 R_first（它前一条候选根是 R_prev），而 birth(b) ≤ R_prev.txg。按 R_first 分四类：

- **A**：R_first 是回退根，且两棵用户可见树（extent、inode）的根指针都「从候选集之外继承来」——指针非空、指针里的出生 txg 小于 R_prev.txg、落点 R_prev 不引用。
- **B**：回退根，只有一棵树继承来，另一棵与现行版本是同一个落点。要回退目标与现行版本共享一棵树，今天只有建 inode 的发布做得到（它只换 inode 树）。
- **C**：回退根，两棵都不继承。模型里造不出（回退根的两棵根指针恒取目标的）。
- **D**：R_first 不是回退根。要回退根自己不在候选集里、它后面的普通根成了 R_first——根槽读不出（故障）时有。

### 上一轮两份红的镜像

| 来源 | 数 | 归类 | 怎么知道的 |
|---|---|---|---|
| 上一轮正推腿模型（`m3-inv-b-r1-sonnet-model/b3_bounds_model.py`，操作串长 ≤ 6） | 379 条历史，393 个红单元 | A 393，B 0，D 0 | 复跑：`classify_prior_sonnet_379.out` 逐条看 R_first 是不是回退根、引用集是不是整份等于某条更早的根 |
| 攻方两批随机历史 | 2 976 张里 173 张红（150 + 23） | 推为 A | 日志只有汇总，**拆不开**；harness 的历史生成器没有建 inode 这一步（`crates/singlefs-harness/src/history.rs:367` 的 `HistoryOperation`），回退又恒取两棵树，B、D 走不到；佐证：只看 inode 树根指针的 V1 把 173 张全部清零 |

攻方汇总行（日志里的原样）：「PROTO name=B3-lower evaluated=325940 violations=1464 images_red=150」（`research/prompts/m3-inv-b-r1-opus-model/logs/s2-rollback48.log:16`）、「PROTO name=B3-lower evaluated=310068 violations=184 images_red=23」（`research/prompts/m3-inv-b-r1-opus-model/logs/s2-broad48.log:17`）；V1 之后「PROTO name=B3-lower-v1 evaluated=304628 violations=0 images_red=0」（`research/prompts/m3-inv-b-r1-opus-model/logs/s2-rollback48.log:17`）、「PROTO name=B3-lower-v1 evaluated=301192 violations=0 images_red=0」（`research/prompts/m3-inv-b-r1-opus-model/logs/s2-broad48.log:18`）。

我的模型对同一个问题的归类（`run_all.out` 原样摘，`plain` 红的镜像数与按 R_first 归类的单元数）：

```text
[穷举] HARNESS 深度 7 无故障
  plain        红的镜像 5541
   plain 红单元按 R_first 归类 {'A_whole': 31385, 'B_partial': 0, 'C_rollback_none': 0, 'D_nonrollback': 0}
[穷举] API 深度 6 无故障
  plain        红的镜像 1864
   plain 红单元按 R_first 归类 {'A_whole': 5656, 'B_partial': 1336, 'C_rollback_none': 0, 'D_nonrollback': 0}
[穷举] FUT 深度 6 无故障
  plain        红的镜像 16429
   plain 红单元按 R_first 归类 {'A_whole': 66814, 'B_partial': 4324, 'C_rollback_none': 0, 'D_nonrollback': 0}
[穷举] FAULT 深度 6 含故障
  plain        红的镜像 19810
   plain 红单元按 R_first 归类 {'A_whole': 50798, 'B_partial': 18464, 'C_rollback_none': 0, 'D_nonrollback': 18}
  plain        红的镜像 103278
  plain        红的镜像 105985
  plain        红的镜像 235394
  plain        红的镜像 228382
```

（输出里 A_whole = A、B_partial = B、C_rollback_none = C、D_nonrollback = D；后四行是随机档，没有按类拆。）

### 收窄之后还红不红

对同一批镜像比五种写法（`birth_floor_v2.py` 的 `analyse`）：

- `plain`：原措辞，下界不收窄。
- `opusI`：攻方原型的写法（`research/prompts/m3-inv-b-r1-opus-model/proto.diff:435` 起的 `let inherits`：只看 R_first 的 inode 树根指针，出生 txg ≤ R_prev.txg 就整条 R_first 引用的单元全豁免；「pointer.birth_txg <= *previous_txg」（`research/prompts/m3-inv-b-r1-opus-model/proto.diff:436`））。
- `both`：inode、extent 两棵根指针都满足才整条豁免（正文题面「继承整棵用户可见树」的字面读法）。
- `sc_le` / `sc_lt`：按树分摊——只豁免「落在某棵继承来的树的子树里」的单元，继承的判据是该树根指针出生 txg ≤（`sc_le`）或 <（`sc_lt`）R_prev.txg；别的单元（固定点单元、实例表）照判。

`run_all.out` 按块解析出的表（红的镜像数；`holes_*` 与 `app_lt` 归 F3）：

| 档 | 字母表 | 镜像数 | plain | opusI | both | sc_le | sc_lt | holes_plain | holes_lt | app_lt |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 穷举 | HARNESS 深度 7 无故障 | 293211 | 5541 | 0 | 0 | 0 | 0 | 221713 | 0 | 0 |
| 穷举 | API 深度 6 无故障 | 141580 | 1864 | 0 | 645 | 0 | 0 | 102842 | 0 | 0 |
| 穷举 | FUT 深度 6 无故障 | 354943 | 16429 | 0 | 2123 | 0 | 0 | 307420 | 0 | 0 |
| 穷举 | FAULT 深度 6 含故障 | 537987 | 19810 | 6 | 9227 | 0 | 0 | 301504 | 0 | 0 |
| 随机 | HARNESS 3000 段，种子 24 | 270000 | 103278 | 0 | 0 | 0 | 0 | 231836 | 0 | 0 |
| 随机 | API 3000 段，种子 21 | 270000 | 105985 | 0 | 13895 | 0 | 0 | 232288 | 0 | 0 |
| 随机 | FUT 3000 段，种子 22 | 270000 | 235394 | 0 | 21818 | 0 | 0 | 259704 | 0 | 0 |
| 随机 | FAULT 3000 段，种子 23 | 270000 | 228382 | 1100 | 28917 | 0 | 0 | 254466 | 0 | 0 |

字母表：HARNESS = 覆盖写 / 空发布 / 卸载 / 挂载 / 回退（从最旧端数 3 个、从最新端数 2 个）/ 抬 F；API 加建 inode；FUT 再加里程碑三才有的「建带数据的新文件」「只改第 k 个文件的数据」；FAULT 加抛弃最新根与丢根槽。随机档 90 步、全部 3 000 段走过根环（`run_all.out` 里「走过根环」那一行）。

读法：

1. **今天 harness 能走到的历史上，三种写法都不红**（HARNESS 两行），与攻方在副本上量的「173 → 0」一致。
2. **`both` 在建 inode 之后红**（API、FUT 行）：回退到「只差一次建 inode」的目标时，extent 树的根指针与现行是同一个落点、不算继承，整条豁免条件不成立，A 类之外的 B 类单元就红了。最短例（`run_all.out` 原样）：`((('W',), ('N',), ('N',), ('N',), ('R', ('o', 1)), ('F', 5)), [(('Ir', 4, (('Il', 4, ()),)), 3)])`。
3. **`opusI` 在丢根槽之后红**（FAULT 行，丢根槽单独跑时 6 张，`run_fault_split.out`；抛弃最新根单独跑 0 张）：历史 覆盖写、覆盖写、回退、建 inode、抬 F、丢回退根的根槽——回退根不在了，它后面那条「建 inode」根（inode 树新、extent 树继承）成了 extent 树子树的 R_first，只看 inode 树根指针不够。首例：`((('W',), ('W',), ('R', ('o', 0)), ('N',), ('F', 4), ('L', 2)), [(('Xr', 3, (('Xl', 3, (('d', 3, ()),)),)), 2)])`。
4. **按树分摊的两种写法 `sc_le`、`sc_lt` 在所有合法镜像上零红**，包括丢根槽与抛弃最新根。

### 为什么严格小于而不是小于等于

两者在合法镜像上都零红，分开它们的是坏镜像（F2 的「盖小」层）：整条链一起被盖小 1 时（诞生代号与树根指针都等于 R_prev.txg），`sc_le` 当它是继承而豁免，漏；`sc_lt` 不豁免，抓得到。合法一侧为什么严格也够：回退根的树根指针出生 txg ≤ 回退目标的 txg；回退那一刻的现行根 C 比目标新，回退根紧接在 C 后面，所以回退根的 R_prev 就是 C（最新根无条件进候选集，`crates/singlefs-checker/src/walk.rs:5977` 起），严格大于目标的 txg 也就严格大于树根指针的出生 txg；目标因抬 F 离开时候选集里每条根的 txg 都 ≥ F > 目标的 txg。这是推的；C 与回退根的根槽同时读不出（两处丢根槽叠加）这一格我没有专门造；另跑丢根槽权重拉高的随机 9 000 段（`run_fault_double_loss.py`，输出 `run_fault_double_loss.out`，sha256 `90ce91690561351edcefac051e2be5b39908849e662c2afa04ffb9e5a6f6df6b`），`sc_le`、`sc_lt` 仍 0 红（`opusI` 每批 4 千多张红），其中有没有恰好「C 与回退根同时丢」的我没单独计数。

### F1 的推翻条件

- 找到一张合法镜像（可达的历史产出）而 `sc_lt` 红：V1 的按树分摊写法出局。我找的范围：上表八档；没覆盖的有两块盘各自的根环、journal 记录施加出来的未落盘根、实例切换重发（`crates/` 里切换重发今天没实现，`.claude/kb/decisions/23-journal的角色与格式.md:411` 写明留给后面的里程碑）。
- 攻方 173 张里有一张不是 A 类：A 类推断翻。

## F2　V1 的措辞与判法

先说名字：F1 的结果是 V1 要写成**按树分摊**的形态，且 ② 的下界与 ① 的连续是同一个谓词的两半（F3），所以下面写成一条「出现规则」，V1 就是它在「第一次出现」那一半上的写法。编号归主 agent。

### 措辞（建议）

> 回退候选集（D23 已定项 14 的候选集，`crates/singlefs-checker/src/walk.rs:5970` 起）里的根按 txg 升序。对其中任一不是最老的根 R，令 R⁻ 为候选集里 txg 严格小于 R.txg 的最大一条（同 txg 的几份根记录并成一条看），令 K(R) 为 R 的树表里用户可见树各条的根指针（今天是种类 extent、inode 两条；里程碑三起再加 `deleted_inodes`，见下）。R 引用而 R⁻ 不引用的每个单元 b，要满足二者之一：
> (a) birth(b) > R⁻.txg；
> (b) 存在 P ∈ K(R)：P 非全零，P 指针里的出生 txg < R⁻.txg，b 的落点在 P 引用的子树里（含 P 自己）。
> 「birth(b)」取 b 起点槽上单元头里的诞生代号，头读不出或头校验和不过的这一格没有对象（与 I-3.10 的射程 ③ 同）；R 或 R⁻ 的引用集合数不全时这一对报不适用。

**「继承整棵用户可见树」在镜像上怎么认**：不拿回退目标比——目标已离开候选集，盘上没有它，今天的回退也不留标记（`crates/` 里没有谱系记录的写入，grep 在下面）。能读的只有 R 自己树表里的两样：

| 字段 | 在哪 | 今天的读法 |
|---|---|---|
| 树表条目的种类 | 条目偏移 10，u16；1 = extent，2 = inode | `crates/singlefs-checker/src/walk.rs:2893` 的 `read_u16(entry, 10),`；常量 `walk.rs:43`、`walk.rs:44` |
| 树根指针 | 条目偏移 14..100 | 「let tree_root = parse_node_pointer(&entry[14..100]);」（`crates/singlefs-checker/src/walk.rs:2874`） |
| 指针里的出生 txg | 指针偏移 42，u64 | 「birth_txg: read_u64(bytes, 42),」（`crates/singlefs-checker/src/image.rs:672`） |
| R⁻ 的 txg | 根记录 | `RootView.checkpoint_txg`（`crates/singlefs-checker/src/lib.rs:309`） |

「R⁻ 不引用 P 的落点」不用另写：b 在 P 的子树里而 b 不在 R⁻ 的引用集里，就推出 P 不在（引用集含子树）。

**用户可见树取哪几棵**：D23 已定项 14 写「新根的用户可见树（inode 树、extent 树）与 deleted_inodes 的根指针取 R_old 那一版的；墓碑树、谱系树随现行那一版，不取 R_old，回退那次发布写一条谱系记录 (T_rb, T_old)」（冻结副本 D23 文件第 376 行）。今天 `deleted_inodes` 是零根、回退里没有这一步（`crates/singlefs-core/src/mount.rs:5162` 起只换 extent、inode 两条），所以 K 今天是两条；**里程碑三删除落地、`deleted_inodes` 有了根之后，K 必须加种类 9**，否则删后回退的镜像上 `deleted_inodes` 子树里的老单元会红。墓碑树、谱系树随现行版本，它们的单元要么在 R⁻ 里、要么是这次新写的，不进 K。

### 为什么这样写（每个选择对着 F1 的哪一格）

- **按树分摊、不整根豁免**：整根豁免（攻方原型）会把回退根自己的固定点单元（树表单元、记账树、分配记录树、中央映射树节点）一起放过去。F2 坏镜像层 `M1单元 rollback根_fx`：78 575 次把回退根的固定点单元盖到 ≤ R⁻.txg，`opusI` 抓到 0 次，`both` 8 879 次，`sc_le`、`sc_lt` 全抓（原样表见下）。
- **每棵树单独判继承、不要求两棵都继承**：B 类（F1）。
- **严格小于**：F1 末节。
- **(a) 里是「>」不是「=」**：新写单元的诞生代号就是 R.txg，恒 > R⁻.txg；写成 ">" 把「盖小到 R⁻.txg 或更小」都挡住，「盖到 R⁻.txg 与 R.txg 之间」只在两根的 txg 之间有洞（抛弃最新根、挂载写行）时才有，任何下界写法都抓不到（`天花板外` 一列，下面的表里这一列是 0——我采的历史里没有哪次盖小落进洞里，这是采样的限度，不是证明）。

### 对 `crates/` 今天的实现改哪里

1. `RootReferences`（`crates/singlefs-checker/src/walk.rs:2731`）只有一份合并的 `placements`（`walk.rs:2733`），分不出哪个落点属于哪棵树。加一个按树的记录：种类、树根指针的出生 txg、这棵树自己的落点集。填的地方是 `references_of_root`（`walk.rs:2804`）里逐条树表条目的循环（`walk.rs:2863`）：对种类 1、2（以后 9）的条目，另起一份空的引用集，`collect_tree_references`（`walk.rs:3197`）走进去，树根指针本身也记进去（`references.note(&tree_root);` 在 `walk.rs:2879`），再并进合并的那一份；走不全就把 `is_complete`（`walk.rs:2743`）置假，同今天。
2. 扫描深度：`judge_release_generation_and_tree_table_birth`（`walk.rs:3406`）里「let depth = if newest」（`crates/singlefs-checker/src/walk.rs:3432`）今天只在最新根的账里有已释放记录时才对每条候选根走到叶，否则只读树表。新规则每条候选根都要走到叶（引用集合只在那个深度上作数），所以这个条件要去掉。**代价没量**：层 0 每个崩溃状态多走一遍每条候选根，同 B3 上一轮对 I-4.9 的估计。
3. 新判定函数，挂在 `scanned.sort_unstable_by_key(|root| root.checkpoint_txg);`（`walk.rs:3453`）之后、与 `judge_release_generations`、`judge_tree_table_birth_txg` 并排：对相邻两条（同 txg 的先并），取 R 的落点集减 R⁻ 的落点集，每个落点读起点槽单元头（读法同 `walk.rs:2522` 的 `reader.read(device, slot * SLOT_BYTES, UNIT_HEADER_SCAN_BYTES)`，诞生代号取 `birth_txg_in_a_usable_unit_header`，`walk.rs:3534`），按 (a)(b) 判；`is_complete` 不全的那一对不判。两块盘各一份的单元是两个落点，各读各的头（两份头不一致归别的条判，这里不重复报）。
4. 固定点单元与实例表单元永远走 (a)：新写的诞生代号 = R.txg；实例表单元照抄时在 R⁻ 的引用集里，不进差集（回退照现行的实例表：「instance_table: InstanceTablePlan::Carry(current.root.instance_table),」，`crates/singlefs-core/src/mount.rs:5433`）。

### 坏镜像：模型层（盖小之后各写法漏不漏）

`birth_floor_v2.py` 的 `mutation_study`：3 000 段随机合法历史（API 字母表、无故障），对每条非最老候选根 R_j 做四种「诞生代号盖小」，数各写法在 R_j 上抓到几次。M1 单元：R_j 新写的单元一个一个盖到 v（v 取 0 到诞生代号减 1 的每个值），树表条目的根指针不动；M2 整条链（用户可见）：新写的 inode / extent / 数据单元的头与两个树根指针一起盖小 k，固定点单元照旧；M3 整条链（含固定点）：M2 再把固定点单元盖小 k。`run_all.out` 原样：

```text
[坏镜像层·盖小] 3000 段随机合法历史（API 字母表，无故障），种子 5；「天花板外」= 盖到的数不比 R_prev.txg 小，任何下界写法都抓不到
  模式               类                  镜像数     天花板外  plain  opusI   both  sc_le  sc_lt
  M1单元             rollback根_fx     78575        0  78575      0   8879  78575  78575
  M1单元             普通根_fx          152737        0 152737 152737 152737 152737 152737
  M1单元             普通根_it           11880        0  11880  11880  11880  11880  11880
  M1单元             普通根_用户可见        339945        0 339945 339945 339945 339945 339945
  M2链(用户可见) k=1    普通根               8418        0   8418      0   1198      0   8418
  M2链(用户可见) k=2    普通根               8418        0   8418      0   1198      0      0
  M2链(用户可见) k=3    普通根               8418        0   8418      0   1198      0      0
  M3链(含固定点) k=1    普通根               8418        0   8418      0   1198   8418   8418
  M3链(含固定点) k=2    普通根               8418        0   8418      0   1198   8418   8418
  M3链(含固定点) k=3    普通根               8418        0   8418      0   1198   8418   8418
```

读法：

- M1：`普通根_*` 四行五种写法全抓（盖小的单元不在继承树里）；`rollback根_fx` 一行 `opusI` 0 / 78 575、`both` 8 879 / 78 575、两种按树分摊全抓——整根豁免把回退根自己的固定点单元放过去了。
- M2：`sc_le` 与 `opusI` 对 k = 1、2、3 都 0 抓到；`sc_lt` 对 k = 1 全抓（8 418 / 8 418），对 k = 2、3 0 抓到。**这是按树分摊写法留下的洞**：树根指针与整棵子树一起被盖小 ≥ 2、而固定点单元还对着，它判成「继承」。
- M3：固定点单元也跟着盖小时，两种按树分摊全抓（它们不在继承树里）；`opusI`、`both` 仍然漏。
- 「天花板外」一列是 0：我采的历史里没有哪次盖小落进 (R⁻.txg, 诞生代号) 的空当（那里任何下界写法都抓不到）；挂载写行 / 抛弃最新根造出的 txg 洞里会有，没单独采。

### 坏镜像：镜像层（设计，没造没跑）

沿用 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的改法：`mutate_unit_of`（288 行）改一个单元、重封两道校验和、把新的整单元校验和沿引用链补上去。I-4.2 那份（`"I-4.2",`，`crates/singlefs-harness/tests/checker_known_bad_images.rs:1162`）改的四处就是模板。

**镜像 1（只红出现规则的下界半）**：起点是 `image_after_the_overwrite`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:1260`，A txg 3、B txg 4，单元表 `UNITS_AFTER_OVERWRITE`，`checker_known_bad_images.rs:65`；候选集 txg 0–4 五条根）。改 B 新写的那个数据单元的四处，把 4 改成 3（= R⁻ 的 txg）：

1. 单元头偏移 75 的诞生代号（`DATA_UNIT_BIRTH_TXG_OFFSET`，`checker_known_bad_images.rs:584`），两盘；
2. B 的 extent 根里那条数据指针的诞生代号；
3. 中央映射里这个单元的 key 的出生 txg（`rewrite_the_data_unit_mapping_key_birth_txg`，`checker_known_bad_images.rs:605`）；
4. 分配记录两片叶里这个落点未释放记录的分配代。

推得：R = B，R⁻ = A；b 的诞生代号 3 不大于 R⁻.txg 3，(a) 不成立；B 的 extent、inode 两棵根指针出生 txg 都是 4，不小于 3，(b) 不成立 ⇒ 红。I-4.2（3 ≤ 4）、I-1.2、I-1.11（key 与头同改）、I-3.10（分配代与头同改）各自对得上；I-3.9 只判已释放的记录，I-9.14 比的是树表条目自己的诞生 txg，都没动 ⇒ 推得只红它。**这是推的，没造没跑**；造出来之后要在 harness 里确认其余条都绿。变异（判定恒真）：把 (a)(b) 的合取换成 `true`。

**镜像 2（攻方整根豁免漏、按树分摊抓）**：历史 覆盖写三次（A 3、B 4、C 5、D 6）→ 回退到 A（E 7）→ 抬 F 到 4（正是 F1 的最短例，`HARNESS` 档里 `W W W R F` 那一串）：A 的单元只在 A、E 里出现，合法镜像上规则绿。把 E 自己的**树表单元**（固定点单元，新写、诞生代号 7）的诞生代号盖成 6（= D 的 txg）：头、根记录里树表指针的出生 txg、映射 key、分配代四处同改。推得：树表单元在 R 的引用里、不在 R⁻（D）的引用里，不在任何继承树里，(a) 的 6 > 6 不成立、(b) 无 P 罩得住它 ⇒ 红；攻方写法（E 的 inode 树根指针出生 3 ≤ 6 ⇒ 整条 E 豁免）不红。同样没造没跑。

**不抓的（要照实交）**：整条用户可见链盖小 ≥ 2 而固定点单元对着（M2 k ≥ 2）——造法同镜像 1，但改的是 R 新写的 inode 树根、extent 树根、数据单元三类单元的头与树表里两条根指针的出生 txg 都盖成 R.txg − 2。它在盘上与「回退根继承了更早一版」分不出：指针出生 txg 早于 R⁻、R⁻ 不引用。收它要回退根留标记：D18 已定项 24 写了回退写一条谱系记录——「记录 (T_rb 8, T_old 8) = 16 字节」（`.claude/kb/decisions/18-块里携带什么信息.md:569`），但 `crates/` 里没有（命令与原样输出见下），谱系树的不变量 I-9.17 状态是「未实现（三·3 实现）」（冻结副本 `invariants.md` 第 291 行）。有了它之后 (b) 可以收成：b 的 R 里存在一条 T_rb = R.txg 的谱系记录，且 P 的出生 txg ≤ 它的 T_old。那一版没有任何一轮攻过。

## F3　I-3.5 ① 要不要改

### 是不是真冲突：是，而且比下界早红

条款三句：I-3.5 的内容范围「对任一块 b 与任一根 R（快照根或活头根）」（冻结副本 `invariants.md` 第 137 行）；可判定形式里「按 txg 升序枚举全部快照根与活头得 Reach(b)」（同一行）；子判据 ①「Reach(b) 在 txg 序上连续无洞」（同一行）。可达谓词是一个区间：「块 b 被快照 S 引用 ⟺ birth(b) ≤ S.txg < death(b)」（`.claude/kb/decisions/05-快照-空间记账机制.md:313`）。回退那次发布把用户可见单元「改回已分配，分配代写回 R_old 那一版账里的分配代」（冻结副本 D23 文件第 382 行），新根又指着它们（D23 已定项 14，`.claude/kb/decisions/23-journal的角色与格式.md:376`）。

推：一个单元被回退目标引用、被中间几条根（覆盖写）不引用、又被回退根引用，它的可达集 {目标, …, 回退根} 不是区间。这与读成「候选集」还是读成「快照根与活头」无关——快照 S 取在目标与回退之间时，S 不引用它而活头引用，洞照样在。**今天 `crates/` 里没有快照根**（grep 见下），I-3.5 行的状态是「未实现 ⚠️ 射程注（2026-09-13）」（同一行），所以没有任何在跑的判定因此红；冲突在于：一旦把 ① 实现成候选集上的判定（C395 底稿与上一轮都是这样读的），最简单的回退镜像就红。

模型复跑（`smallest_hole_example.out` 原样，sha256 `2f11baafe038b40c51f6e9a1a95d20543e94acc74d9c4f12449dfa4a248b2fce`）——两步历史，没有抬 F，回退目标还在候选集里：

```text
操作: (('W',), ('R', ('o', 0)))
候选根 (txg, 谁写的): [(0, 'mkfs'), (1, 'warm'), (2, 'warm'), (3, 'first'), (4, 'W'), (5, 'R')]
① 连续无洞，原写法（holes_plain）红的单元数: 5 例: [(('d', 3, ()), 5), (('Xr', 3, (('Xl', 3, (('d', 3, ()),)),)), 5)]
② 下界 plain 红: []  下界 sc_lt 红: []
① 放宽（holes_lt）红: []  出现规则 app_lt 红: []
  单元 Il 诞生代号 3 被候选根（txg）引用: [3, 5]
  单元 Ir 诞生代号 3 被候选根（txg）引用: [3, 5]
  单元 Xl 诞生代号 3 被候选根（txg）引用: [3, 5]
  单元 Xr 诞生代号 3 被候选根（txg）引用: [3, 5]
  单元 d 诞生代号 3 被候选根（txg）引用: [3, 5]
```

第一版的五个单元在候选集里「引用（txg 3）、不引用（txg 4）、引用（txg 5）」：`holes_plain` 红，② 下界这时还不红（回退目标在候选集里，R_first 是它自己）。所以 ① 比下界先红，而且上一轮攻方报的「173 张」只是 ② 半边——① 不需要抬 F 越过目标就红。上表 `holes_plain` 一列：HARNESS 档 293 211 张里 221 713 张。

### 改成什么：把 ① 并进出现规则

原来的 ② 说的是一个单元**第一次**出现在候选集里时的诞生代号；① 说的是它**再次**出现要不要连续。两者都是「b 在 R 里、在 R 的前一条 R⁻ 里不在」这一件事的两种情形，F2 的出现规则一句话管完：

- 第一次出现：新写的（birth > R⁻.txg）或继承来的（(b)）；
- 再次出现（洞的右缘）：birth 必然 ≤ 更早那条根的 txg < R⁻.txg，(a) 不成立，只能靠 (b)——也就是只许出现在继承点上。

模型验证（`run_all.out`，原样）：

```text
[坏镜像层·复活旧单元（只红 ①）] {'普通根': {'n': 21680, 'sc_lt': 0, 'holes_plain': 21680, 'holes_lt': 21680, 'app_lt': 21680}, '回退根': {'n': 6012, 'sc_lt': 0, 'holes_plain': 6012, 'holes_lt': 6012, 'app_lt': 6012}}
[出现规则 = ② 的 sc_lt ∪ ① 的 holes_lt] FAULT 字母表深度 5 共 50834 张镜像，红集不一致的 0 张
```

- 合法镜像上零红：上表 `holes_lt`、`app_lt` 两列在八档上全是 0（`holes_plain` 在同样的镜像上大量红）。
- 出现规则的红集与「② 的 `sc_lt` ∪ ① 的 `holes_lt`」逐镜像相等：FAULT 字母表深度 5 的 50 834 张镜像，不同的 0 张（上面第二行）。**所以不用同时实现两条**：一个按相邻两根的差集判的循环就是两条的和，比 ② 原来的「找每个单元的 R_first」还简单。
- 再次出现的坏镜像（模型层，非回退根 R 多引用一个更早的候选根引用过、R⁻ 已不引用的旧单元）：上面第一行，`普通根` 21 680 次 `holes_plain`、`holes_lt`、`app_lt` 全抓、② 下界的 `sc_lt` 0 次——① 半边有 ② 半边抓不到的东西。`回退根` 一行是把旧单元塞在继承树之外的落点上，同样红；真在继承树里的是合法的回退，分不出，也不该红。

**限度，照实写**：① 半边的「只红它」镜像我没有便宜的造法。造一份「非回退根再次引用旧单元」要同时让映射条目、分配记录（改回已分配）、记账行与引用一致，实质是手工做一次复活；做不平，红的是 I-3.1 / I-3.11 / 映射那几条，不是它。所以镜像层只有 F2 的镜像 1、2（下界半）；① 半边的独立价值是模型层的，镜像层没证。这条要是被主 agent 当作「V1 写不出只红它的坏镜像」那一条来判，判的对象是下界半，下界半写得出。

### 两个选择（我的推导落在第二个，采不采归主 agent）

| 选择 | 合法镜像 | 抓得到什么 | 代价 |
|---|---|---|---|
| α 删掉 ①，只留 ② 的 V1，I-3.5 行补一句「回退之后候选集里的 Reach 允许有洞」 | 零红 | 再次出现（模型层 21 680 次）抓不到 | 少一半判定，少一份实现 |
| β 出现规则：① ② 合成一条（F2 的措辞） | 零红 | 再次出现抓得到（模型层），镜像层没证 | 同一个循环，不多写；I-3.5 行的 ①、② 两句都要改写 |

「改法会让合法镜像红 ⇒ 不改」那条判据，两个选择都过。

### F3 的推翻条件

- 一张合法镜像上 `app_lt`（或 `holes_lt`）红：β 出局；α 不受影响。
- 能造出一份非回退根的合法镜像、旧单元在洞的右缘再次出现：说明 ① 本来就不该判，两个选择都要改。
- 把 I-3.5 的「快照根」读成真有快照：快照取在目标与回退之间时洞照样在，冲突只更严重，不会消失。

## 复核命令（冻结副本里现跑，原样）

冻结副本我引的十个文件的 sha256 与 `research/prompts/m3-birth-floor-r1-snapshot/tree-sha256.txt` 里的逐个比过，十个都一致（`mount.rs`、`walk.rs`、`image.rs`、`transaction.rs`、`invariants.md`、D23、D18、D5、`checker_known_bad_images.rs`、`history.rs`）。

「`crates/` 里没有 X」：

```text
$ grep -rnE 'T_rb|LineageRecord|lineage_record|谱系记录|PACKED_TYPE_LINEAGE' crates --include=*.rs | wc -l
0
$ grep -c lineage crates/singlefs-core/src/mount.rs
0
$ grep -rl 'snapshot_root\|SnapshotRoot' crates --include=*.rs | wc -l
0
```

谱系树在 `crates/` 里只有零根注册：`crates/singlefs-core/src/transaction.rs:2371` 的 `pub lineage: TreeIdentifier,`。回退不写谱系记录、没有快照根。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| F1 上一轮红的镜像归类 | 一致：同属一类（A） | 正推腿 379 条历史 393 单元全是「回退根整份照抄目标」；攻方 173 张拆不开，推为同类 |
| F1 V1 收窄后还红不红 | 分写法：按树分摊（`sc_lt`）零红；攻方原型（`opusI`）在丢根槽后红；「两棵都继承」（`both`）在建 inode 后红 | 八档共 2 407 721 张 + 丢根槽叠加 9 000 段，`sc_lt` 一张不红 |
| F2 措辞与判法 | 写得出：出现规则，按树分摊，严格小于 | 认继承只用 R 树表里 extent、inode（以后加 `deleted_inodes`）的根指针出生 txg 与 R⁻.txg 比；改 `walk.rs` 的引用集结构、扫描深度、加一个相邻对的判定循环 |
| F2 只红它的坏镜像 | 造得出（设计，没造没跑） | 镜像 1（新数据单元盖到 R⁻.txg，四处同改）；镜像 2（回退根的树表单元盖小，攻方写法漏） |
| F2 按树分摊留下的洞 | 有：整条用户可见链盖小 ≥ 2 而固定点单元对着 | 模型 8 418 次中 0 次；收它要回退根留标记，谱系记录里程碑三才有 |
| F3 ① 与回退是不是真冲突 | 一致（真冲突，比下界早红） | 两步历史 覆盖写、回退，第一版五个单元「引用—不引用—引用」；今天 I-3.5 未实现，不是在跑的红 |
| F3 改法 | 并进出现规则（β）；退一步可只删 ①（α） | 合法镜像零红；与 ② 的 V1 ∪ ① 放宽逐镜像相等（50 834 张 0 张不同）；① 半边镜像层的只红它造法没有 |

## 什么现象会推翻这些结论

- 一张合法镜像上 `sc_lt` / `app_lt` 红：V1 的按树分摊写法与出现规则出局。我没覆盖到的：两块盘各自的根环、journal 记录施加出来而根槽没落盘的版本、实例切换重发（`crates/` 里今天没有）、回退根与现行根的根槽同时读不出。
- 把镜像 1 或 2 造出来、跑出来红的不止它自己（或根本不红）：对应的「只红它」不成立；先看是不是映射、分配记录、记账行没改全。
- 攻方 173 张里有一张在回退根之外、或回退根只继承一棵树：F1 的 A 类推断翻。
- 在入库装置上量到扫描深度改动的代价大到不能接受：与判定无关，影响的是「立不立」的取舍。
- 模型是抽象的（单元是元组、两棵树、没有真实多层树和 journal 施加）：模型的零红只对它的操作集说话。

## 没做什么

- 没有造、没有跑 F2 的两份坏镜像（设计）；没有跑 harness、没有编、没有跑 checker 档的任何东西。攻方 173 张的逐张归类、攻方的数在入库装置上重量，都在我的范围之外，也复核不了（日志只有汇总）。
- 没有量扫描深度改动的层 0 代价（没有数）。
- 没有判「下界立不立」与取 V1 还是别的形态——只给了正推出来的措辞、判法与代价，采不采归主 agent。F3 的 α / β 同。
- 谱系记录版的 (b)（里程碑三之后）只是写了一句推演，没有任何一轮攻过，也没有模型。
- `deleted_inodes`（种类 9）进 K 是照 D23 已定项 14 的字面推的，今天零根，没有镜像可验。
- 草稿：`/tmp/claude-1000/m3-birth-floor-r1-sonnet/` 里只有跑模型的小脚本与输出（`run_tiers.py`、`run_random.py`、`tier-*.out`、`rnd*-*.out`），没有编译目录、工作树或仓副本，不需要删；入库的全在 `research/prompts/m3-birth-floor-r1-sonnet-model/`。

## 引文核对

kb 条款的行号都是冻结副本里的。活仓的 kb 在冻结之后又动过，其中七处（`invariants.md` 的 137、291 行，D23 文件的 376、382 行）在活仓里行号已移，我在冻结副本里逐字核过（改写前 22 处引文在冻结副本为根时全部对上），报告里写成「冻结副本……第 N 行」，不写成机器认的形式，免得活仓为根时误报。末行：

```text
$ python3 research/scripts/cite-check.py research/prompts/m3-birth-floor-r1-sonnet-output.md
  ✓ 核了 15 处引文，对上 15 处，没判 0 处（逐处列在上面）
$ python3 research/scripts/cite-check.py research/prompts/m3-birth-floor-r1-sonnet-output.md --root /tmp/claude-1000/m3-batch4-freeze/tree --root /home/fy5090/code/singlefs --background research/prompts/_m3-birth-floor-r1-background.md
  ✓ 核了 15 处引文，对上 15 处，没判 0 处（逐处列在上面）
```
