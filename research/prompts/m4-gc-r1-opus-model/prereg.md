# m4-gc-r1 攻方模型的跑前登记（2026-10-13，写于第一次正式跑之前）

模型 `gc_model.py`：一块盘 NSEG 段 × SEGSZ 槽，一槽一个单元（数据单元不分两槽，节点不分 16 KiB / 32 KiB）；每次发布原子
（根环换根），每次发布 COW 一个元数据块（代表那次发布重写的树节点）并释放上一版那一块；元数据块从开放聚簇段 bump，
开放段 = 最低的全空段，本次挂载开过的段不给用户数据（D3 已定项 8 第 2 条、已定项 10 ①②）；用户数据落点 = 最低空槽、
不在本次挂载开过的段里、不在活整理意图的 R 里（合取 5、6）；释放进 defer，释放代 ≤ 环里最旧根时才可再分配
（D16 已定项 1 的谓词取「环里最旧有效根」那一支）。可选：每次可写挂载写一片实例表（豁免映射、只在挂载时重写）。
崩溃 = 挂载（发布原子，崩溃只丢内存态：开放段、游标、本次挂载开过的段、缓存的搬迁计划）。

操作（每一步一次原子发布，广度优先全枚举，用户动作全放开）：W 新建一个单元的对象、O 覆盖写、D 删除（单批）、
Q 覆盖写且释放前读盘核对不上（隔离：记录留在已分配、映射条目去掉，D19 已定项 5 硬规则 1，一段历史至多一次）、
GI 签发整理意图（R = 一个段，c0 = 签发前那一版的 txg，segs0 = 签发前的全空段数，续做点 = R 起点）、
GB 一批整理（搬至多 B 个分配代 ≤ c0 的数据单元出 R；目标态达成就在同一次发布删意图）、M 挂载（或崩溃后重挂）、
RB k 管理员回退到环里倒数第 k+1 条根（D23 已定项 14：用户可见树取 R_old、复活、按槽释放）。

## 臂（每条臂的定义跑前写死）

| 臂 | 定义 |
|---|---|
| move | rewrite：搬迁同时改引用方指针（位置提示 = 映射）；maponly_regen：只改映射，而非整理的发布照今天的代码按引用方指针重算映射条目（`transaction.rs` 装映射条目取 `pointer.locations`）；maponly_carry：只改映射，非整理的发布照抄上一版映射条目（今天的代码没有这一条） |
| count | occ：段内没有已分配也没有 defer 的槽（代码 `empty_segments`）；rec：段内没有未释放记录（D3 已定项 10 ①，defer 算空） |
| dest | user：整理的目的落点照用户数据政策；holes：只落进 R 外、非全空、不在本次挂载开过的段里的空槽 |
| precond | 签发前判「R 外非全空段的空槽 ≥ R 内要搬的单元数」，不够不签 |
| select | naive：空槽最多的非全空段（含本次挂载开过的段、含装着实例表或隔离槽的段）；safe：排除这三种 |
| cont | skip：续做点之前的不再核；recheck：删意图前对整个 R 全量核 |
| rbint | 回退时 logged_ops 随现行（cur）或取 R_old（old） |
| plan | batch：每批在发布那一刻现取目的落点；cached：签发时算好整张（key, 源, 目的）清单，之后各批照清单 |

## 判据（每个新状态判；转移上判的另注）

live_released（活单元的映射落点已释放或无记录）、unreadable（按提示读、再按映射读都读不到那一版的内容）、dup_map（两个活单元
映射到同一槽，或映射到元数据 / 实例表槽）、leak（未释放的记录不被任何活单元映射、不是当前元数据 / 实例表 / 隔离槽）、
hint_bad（活单元的提示 ≠ 映射且提示那一槽已释放或内容已不是它：今天的 checker 把提示当引用，这一格在 I-3.9 / I-2.1 上红）、
unmet_del（转移：删意图那一刻 R 内还有分配代 ≤ c0 的未释放落点）、false_enospc（转移：W 落不了点，而 R 里有空槽）、
gc_stuck（意图活着、R 内只剩搬不动的违例槽）、
livelock（从这个状态起只跑整理：签发 → 各批 → 删意图时判停机谓词「全空段数 − segs0 ≥ 1」，假就再签一条，
2·NSEG 条意图之内停不下来）。

## 跑前预测

1. 对照（rewrite / rec / user / naive / recheck / old / batch，不开实例表与隔离）：live_released、unreadable、dup_map、leak、hint_bad、unmet_del 全 0；livelock > 0（目的落点吃掉全空段）。
2. maponly_regen：live_released、unreadable、leak 都 > 0。maponly_carry：这三格 0，hint_bad > 0。
3. count=occ（其余同对照）：livelock 比 rec 多。
4. dest=holes + precond：livelock = 0。只开 dest=holes 不开 precond：livelock 仍 > 0 或 gc_stuck 类停下 > 0（不确定）。
5. plan=cached：dup_map > 0、unreadable > 0。
6. rbint=cur + cont=skip：unmet_del > 0；rbint=cur + recheck：0；rbint=old + skip：0。
7. 开实例表 + select=naive：gc_stuck > 0；select=safe：gc_stuck = 0。
8. 开隔离（Q）+ select=naive：gc_stuck > 0、false_enospc > 0；select=safe：gc_stuck = 0。

## 修订一（同日，正式跑之前；改之前在草稿目录跑过 3×3 深度 5 与 4×4 深度 6 两轮试跑，试跑的数不入正式产物）

试跑里看到的、促成这次修订的：naive 选段在 mkfs 之后第一步就活锁（它挑只装着当前元数据块的段，元数据块每次发布换段，整理在两段之间追着元数据跑），把别的现象全盖住；`holes` 按「段内有 defer 槽就算非全空」取，搬进去的单元正好压住一个快要排空的段。

1. 对照臂的选段改成 safe 加「R 里至少有一个能搬的单元」（needmov）；naive 另立一条臂（select=naive、不要求能搬）。
2. `holes` 与 precond 里的「非全空段」改按段内有未释放记录（rec 口径）判。
3. 删意图那一次发布自己的落点照样排除 R（意图在上一版里还活着）。
4. 加臂 bounded（一条意图完成而停机谓词为假就停、不签下一条）与 holes_pre_bounded；加转移判据 del_pred_true / del_pred_false（删意图那一刻谓词的值）；livelock 分成 cycle（只跑整理时回到见过的状态，确定停不下）与 cap（2·NSEG 条意图之内没停）；加 wasted（bounded 臂上意图完成、谓词为假而停）。

试跑已经看到、因此不算预测的：4×4 深度 6 上 occ 臂 livelock 为 0 而对照臂不为 0（与预测 3 相反）；fault_safe 的 gc_stuck 不为 0（与预测 8 后半相反：隔离发生在意图签发之后，签发时的排除管不到）；4×4 上 false_enospc 为 0。这三条照原预测判输，正式产物里照实报。

新加两臂的预测（写在正式跑之前）：bounded：livelock 0、wasted > 0；holes_pre_bounded：livelock 0、wasted 小于 bounded。

## 修订二（同日，正式跑之前）

1. 规范化：单元 key 的写序换成同一对象在环里各版之间的名次（新写的 key 写序取 10⁶ + txg，保证与已有各版不等）。修订一之前的规范化让只跑整理时的状态永不重复，`cycle` 判不出来、只能落到 `cap`；改了之后同一段只跑整理的轨迹回到见过的状态就判 `cycle`。`cap` 的上限改成 8·NSEG 条意图（环境变量 `GC_MODEL_CAP_FACTOR` 可改，正式跑用默认 8）。
2. 加合取 7（`全空聚簇段数 ≥ N₀`，按这次发布动账之前的那一版判，口径随 count 臂）：臂 n0_all（N₀ = 1，管全部发布：写入、删除、整理、挂载）与 n0_fg（N₀ = 1，只管写入类发布）；加状态判据 frozen（除回退之外的每一种操作都发不出去）与转移判据 n0_refused（写入类发布因合取 7 被拒）。

新两臂的预测（写在正式跑之前，3×3 深度 6 试跑过一次之后写的，试跑的数照样不入正式产物）：n0_all：frozen > 0；n0_fg：frozen = 0、n0_refused > 0。
