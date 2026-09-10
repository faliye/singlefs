# 反推腿答复：假设「E95 支持 D26 未定项 4 要真定格式形态」是错的

**腿**：Opus（云端 B），立场 = 反推。日期 2026-09-09。
**现查过的东西**：`research/e7-index-bench/src/bin/e95_node_layout_arms.rs`（1257 行，全读）、
`research/results/e95-node-layout-arms-2026-09-09.out`（86 行，全读）、
`research/mutations/e95_node_layout_arms.tsv`（18 行）、
`.claude/kb/experiments/95-老化的节点布局臂.md`（工作区版 125 行与 `git show HEAD` 版 57 行，两版都读）。

**跑过的东西**：把仓里那份源码原样拷进 scratchpad
（`/tmp/claude-1000/-home-fy5090-code-singlefs/9ea13542-e50a-4c0c-b861-c3ea93fc74fe/scratchpad/`），
先确认 **baseline 与入库产物逐字节一致**（`diff base.out research/results/e95-node-layout-arms-2026-09-09.out` 空），
再在副本上做了六个探针。仓里一个文件都没改。
⚠️ 六个探针的产物**未入库、不进判据、不进复跑**，与 kb 工作区版第 118 行那句自陈同一性质。

---

## 判决先说

**「E95 支持要真定格式形态」这个读法，我判它错。** 但要说清是哪一半错了：

| 轴 | 我攻下了没有 |
|---|---|
| **量级轴**（格式臂在 runs8 三格把 runs 压到政策臂一半以下） | **没攻下。** 我加了三条新政策臂，三格照旧打对折，而且把格式臂那边的 knob 补全之后差距**更大**，不是更小 |
| **分类轴**（赢的那条臂凭什么算「格式」） | **攻下了，而且是决定性的。** 赢的那条臂在本模型里读的唯一状态是 `key / N`，key 不是盘上字段。把**同一段函数**换个 tag 注册成政策臂，答案行当场从 `true` 翻成 `false` |

⇒ E95 这份结果支持的命题是：**「固定 key 对齐边界的整组重落，在 1.5×–5× 预算带上、在有局部性的负载上，打赢滑动窗口与轮转整理」**。
这是一条**纯放置政策**的结论。要让它支持格式那一侧，还差一个带分裂合并的模型——
而这句话不是我发明的，kb 工作区版 `.claude/kb/experiments/95-老化的节点布局臂.md:113` 自己就写着
「要分辨这两者，得有一个带分裂合并的模型」。我做的是把它变成一个数。

---

## 第一问：读源码找建模缺陷

### 1.1 四条臂的预算不是真的相等，而且闸只封上界

**预算不等，实测**（逐行抄自 `research/results/e95-node-layout-arms-2026-09-09.out`）：

```
E7RESULT name=grid budget=32 arm=fmt_group_g32 family=format load=runs8 runs_median=4370 write_amp=1.387 budget_spent_pct=77.3 fallback_pct=46.8
E7RESULT name=grid budget=32 arm=fmt_group_g8 family=format load=uniform runs_median=7558 write_amp=1.436 budget_spent_pct=87.2 fallback_pct=0.0
```

政策臂在**全部六格**都是 `budget_spent_pct=100.0`（产物第 31–34、47–50、63–66 行），
格式臂里 `fmt_group` 那三条在九格上花不满，最低 77.3%。

**闸只封上界**——源码 928–933 行：

```rust
let cap = (D + extra) as f64 / D as f64;
assert!(
    amp <= cap + 1e-9,
    "{} 在预算 {extra} 上花超了：write_amp={amp:.6} > {cap:.6}",
    arm.tag()
);
```

判据 4 的原文是「**等预算**比较」，代码实现的是「**不超预算**」。少一条下界断言，
一条只花 10% 预算的臂会一路绿到底。这条落在
`.claude/singlefs-ai-sop/rules/test-discipline.md`「只让多条臂互相比，测不出『所有臂一起错』」
那一节的反面：这里连互比都没有，`budget_spent_pct` 是**报出来的，不是断言出来的**。

**但它不解释赢面**：三个「打对折」的格里，赢的那条臂各自花了 99.9 / 100.0 / 98.5
（产物第 38、54、72 行）。⇒ 预算不等这件事**成立，但推不动结论**。

**什么观测会推翻这条**：产物里出现一格，赢的那条格式臂 `budget_spent_pct` 明显低于同格政策臂而仍然赢。现在没有。

### 1.2 连续块粒度确实不一样，可以指到行

- `pol_compact` 的脏 key 走 **`alloc_bump()` 逐个分配**（源码 616 行），
  而 `alloc_bump`（254–278 行）只有两条路：开放段里 bump，或者掉进 `alloc_first_fit`（273–274 行）拿一个散槽。
  **它整条路径上没有任何「找一段长 len 的连续空洞」的搜索。**
- `fmt_group_gN` 的每一组走 **`place_run` → `place_chunk`**（698、716 行 → 311–356 行），
  而 `place_chunk` 有四条分支，第三条正是 `find_free_run(len)`（343 行）——**逐段搜连续空洞**。

⇒ 两条臂**不共用同一张放置面**，这是源码级事实，不是「可能」。

**但它同样推不动结论。** 我建了 `pol_compact_pr`：与 `pol_compact` 唯一的差别是脏 key 按极大连续段走 `place_run`
（也就是把 `find_free_run` 这条分支也给政策臂），整理批次原样不动。实测：

| 格 | pol_compact | pol_compact_pr | 变化 |
|---|---|---|---|
| b32 / runs8 | 6073 | **3987** | 好 1.52× |
| b128 / runs8 | 3922 | 3919 | 几乎不动 |
| b256 / runs8 | 1339 | 1566 | **反而变差** |
| b32 / uniform | 7502 | 7558 | 略差 |

b32/runs8 那格的 `fallback_pct` 从 83.7 掉到 49.8，而 runs 只从 6073 到 3987——
离格式臂的 1595 还差得远（1595 × 2 = 3190 < 3987，**照旧打对折**）。

⇒ **任务书里那个假设（「格式臂赢的是小块更容易找到空位」）我判它不成立。** 如实记。
真正在起作用的是**边界固定**：`fmt_group_g1`（每个脏 key 自己一组，等于没有分组）在 b32/runs8 上是 6481，
和政策臂一个量级；一分组就跳到 1595。

**什么观测会推翻这条**：把 `place_chunk` 的 `find_free_run` 分支（343 行）从格式臂那边也摘掉，
如果格式臂的赢面塌了，那就是我判错了。我没跑这一个方向——**这是本答复里我知道自己没测的一格**。

### 1.3 还有一处不对称，方向偏向格式臂，但幅度未测

`pol_compact` 的整理批次是**无条件**填满 `extra` 个 key 的（源码 621–627 行的 `while sweep_keys.len() < extra`），
而 `step_node_group` 可以**主动少花**：703 行 `while remaining >= g && scanned < groups` 在 `remaining < g` 时就停，
708 行还会跳过本轮已经写过的组。也就是格式臂有一个「这轮不划算就不花」的自由度，政策臂没有。

产物里格式臂最低花到 77.3%，说明这个自由度**真的被用到了**。但三个决胜格上它花了 98.5–100%，
所以我判它**不解释赢面**。⚠️ 这一格我没做对照臂（没建「允许少花」的政策臂），**属于未测**。

---

## 第二问：材料第五节第 1 条成不成立？能不能构造不动格式的政策臂打平？

### 2.1 成立，而且比材料写的更硬——它可以被一条命令证明

材料第五节第 1 条说「格式臂赢的那个机制，在这个模型里不需要任何格式位」。**成立。**
源码级依据：`step_node_group`（671–718 行）读到的全部输入是 `dirty`、`g`、`extra`、`group_cursor`、`sim.l`，
组身份出自 683 行 `dirty.iter().map(|&k| k / g)`——**一次除法，没有任何 `sim` 上代表盘上字段的状态**。

对照着看 `step_gen_sep`（721–751 行）：它读 `sim.last_write[k]`（726 行）与 `sim.obj_gen[key]`（743 行），
**这两个才是真的每对象持久状态**。⇒ **本实验里唯一真的消费盘上字段的那条臂，输了每一格。**

而更硬的一步在判决代码里。源码 951–963 行：

```rust
            let best_policy = grid
                .iter()
                .filter(|g| g.0 == extra && g.2 == load && g.1.starts_with("pol_"))
                .map(|g| g.3)
                .min()
                .expect("每格必须有政策臂");
            let (best_fmt_tag, best_fmt) = grid
                .iter()
                .filter(|g| g.0 == extra && g.2 == load && g.1.starts_with("fmt_"))
```

「格式」与「政策」的分界**是 tag 字符串的前缀**，上游是 `Arm::family()`（108–113 行）按 enum variant 硬写的
`"policy"` / `"format"`。**模型里没有任何东西表示「这条臂消费了一个盘上字段」。**

### 2.2 那条不动格式的政策臂：`pol_align_gN`（对齐窗口重写）

**定义**（跑前写死的形态，我没有跑完再改）：

> 每个 checkpoint，对每个脏 key `k`，取 key 空间上的**固定对齐窗口** `W(k) = [⌊k/N⌋·N, ⌊k/N⌋·N + N)`；
> 窗口去重；若剩余预算 ≥ 该窗口里干净对象的个数，就把**整窗**当作一段连续落盘，
> 否则只落窗口里脏的那几个；预算有剩就按窗口轮转，主动整窗重落。
> `N` 是**运行时旋钮**，盘上不记录，换挂载可以改，读路径完全不受影响。

**它跟 E93 已有的四条臂差在哪**：

| E93 的臂 | 差别 |
|---|---|
| `first_fit` | 那条根本不做聚簇 |
| `bump_seg` | 只把脏的推进开放段，不带任何干净邻居 |
| `bump_neighbor(R)` | **滑动窗口**：窗口以脏 key 为中心、跟着脏 key 走，每轮边界都不一样 |
| `bump_compact(B)` | **轮转整理**：整理批次由 sweep 游标定，**与这一轮脏了谁无关** |

⇒ 唯一的新东西是「**边界固定在 key 空间上，而且与脏集有关**」——
它既不是滑动窗口（边界不跟着脏 key 漂），也不是轮转整理（选谁由脏集决定）。

**为什么它不动格式**：放置是**写侧决定**，读路径按指针走，盘上没有任何字节需要记住窗口边界。
写路径要算 `⌊k/N⌋` 只需要那个节点的 key，而 key 本来就在父节点的分隔键里（D8 的树形态给的），
不需要节点头里多一个字段。

### 2.3 实测：不是「打平」，是**逐格逐字节相同**，而且答案行翻面

我把 `pol_align_gN` 注册进 arms（family 报 `"policy"`），dispatch 到**同一个** `step_node_group`，
其余一个字不改。18 个单测 + 13 个 lib 单测全绿，**14 个 xfixture median 一个不差**（说明共享底座没被我动过）。

结果（scratchpad `probe.out`，N ∈ {2,8,32}，全部 24 格）：

```
budget=128  arm=fmt_group_g8   load=runs8  runs_median=904  write_amp=3.000  budget_spent_pct=100.0  fallback_pct=0.0
budget=128  arm=pol_align_g8   load=runs8  runs_median=904  write_amp=3.000  budget_spent_pct=100.0  fallback_pct=0.0
```

**24 格全部逐字相同**（同一段代码，这是必然的，不是发现）。**发现在判决行**：

```
E7RESULT name=verdict budget=32 load=runs8 best_policy=1595 best_format=1595 best_format_arm=fmt_group_g8 format_halves_policy=false
E7RESULT name=verdict budget=128 load=runs8 best_policy=904 best_format=904 best_format_arm=fmt_group_g8 format_halves_policy=false
E7RESULT name=verdict budget=256 load=runs8 best_policy=276 best_format=276 best_format_arm=fmt_group_g32 format_halves_policy=false
E7RESULT name=answer any_cell_format_halves_policy=false criterion=E95_5
```

**六格全 false，答案行翻成 false。** 注册一条不动一个字节格式的政策臂，判据 5 就不再成立。

⇒ **判据 5 量的不是「格式买到了什么」，是「注册表里有没有人用政策的 tag 提交过这个机制」。**

**什么观测会推翻这条**：拿出一处论证，说明在 D8 定的树形态下，写路径在冲刷一个索引节点时**拿不到**它的 key 区间、
因此 `⌊k/N⌋` 算不出来，非得节点头里存一个组号不可。那样 `pol_align_gN` 就不是政策臂，
这一整条攻击当场作废。我在 `.claude/kb/decisions/08-核心索引结构.md` 里**没有**去找这一条——**这是我留下的缺口**。

⚠️ **我不主张「所以永远不需要格式位」**。我主张的是：**这份实验的算术里没有它**。
两者的差别正是 kb 工作区版 113 行那句话。

---

## 第三问：材料第五节第 2 条怎么验？给一个能分辨的取样点

### 3.1 取样点：把负载的重写段长 RL 与组大小 g 各自扫开

材料第 2 条说「runs8 的赢面可能被 g=8 对齐放大」。**能验，取样点就是把两个都参数化**：
`Load::Runs(RL)`，`RL ∈ {1(uniform), 2, 4, 8, 16, 32}` × `g ∈ {1, 2, 4, 8, 16, 32, 64}`。
E95 只取了 RL ∈ {1, 8} × g ∈ {2, 8, 32}——**两条轴各只有 2 个和 3 个点，转折点落在格子外面**。

### 3.2 实测（scratchpad `gsweep.out`，每格 5 种子中位）

```
预算 负载    | g=1    g=2    g=4    g=8    g=16   g=32   g=64   | 最好的 g | 最好政策臂
  32 uniform | 7945   6128   7022   7558   7726   7819   8153   | g=2   (6128) | 6544
  32 runs2   | 7499   4187   4583   4917   5439   5690   5761   | g=2   (4187) | 5351
  32 runs4   | 7095   3863   2715   3312   4763   5054   5310   | g=4   (2715) | 4691
  32 runs8   | 6481   3548   1924   1595   3121   4370   4989   | g=8   (1595) | 3987
  32 runs16  | 5591   3225   1766   939    853    2487   3984   | g=16  (853) | 3421
  32 runs32  | 4135   2743   1575   862    461    544    1943   | g=16  (461) | 2683
 128 uniform | 6078   3966   3912   5797   6804   7249   7519   | g=4   (3912) | 5426
 128 runs2   | 5437   3569   1978   2753   4029   5042   5477   | g=4   (1978) | 5279
 128 runs4   | 4990   3092   1840   1005   2145   4136   4477   | g=8   (1005) | 4582
 128 runs8   | 3798   2575   1576   904    522    1969   3679   | g=16  (522) | 3919
 128 runs16  | 2492   1940   1253   781    440    271    1916   | g=32  (271) | 2675
 128 runs32  | 1203   1032   807    579    386    171    139    | g=64  (139) | 1380
 256 uniform | 3818   3414   2016   3669   5529   6567   7064   | g=4   (2016) | 4791
 256 runs2   | 2676   2571   1841   999    2368   4224   5029   | g=8   (999) | 3534
 256 runs4   | 1697   1946   1485   933    520    2048   3519   | g=16  (520) | 2475
 256 runs8   | 830    1139   1079   742    458    276    1948   | g=32  (276) | 1339
 256 runs16  | 222    333    584    554    341    215    143    | g=64  (143) | 394
 256 runs32  | 158    170    181    254    259    146    126    | g=64  (126) | 158
```

### 3.3 判决：材料第 2 条**按它写的形态不成立**，但底下的那件事成立

- **不成立的部分**：最优 g **不等于**负载段长。b32 那一行上最优 g 确实跟着 RL 走（2/2/4/8/16/16），
  但 b128 上是 4/4/8/16/32/64、b256 上是 4/8/16/32/64/64——**最优 g 同时跟着预算走，大约每档预算翻一倍**。
  所以「g=8 恰好对上 runs8」是**三点栅格里的最好点**，不是对齐巧合。
- **成立的部分**：g 挑错，收益确实没了。b256/uniform 上 g=32 是 6567，比政策臂的 4791 还差。
  这正是 kb 工作区版 116 行那句「组大小要按负载的局部性挑，挑错就没有收益」。
- ⚠️ **一个反方向的发现**：三点栅格**低估**了格式臂。b128/runs8 真正的最优是 g=16 → **522**，
  而 E95 报的最好格式臂是 g=8 → 904。**低估了 1.73×。**
  ⇒ 把 knob 补全，格式那一侧比 E95 报的更强，不是更弱。**如实记，这一条对我的立场不利。**

**什么观测会推翻这条**：换一套种子或换 `L`/`S`/`G` 之后，最优 g 不再随预算漂移。我只在一组配置上扫过。

---

## 第四问：判据 3 的阳性对照够不够？举一个「机制坏掉而断言全绿」的坏法

### 4.1 那道闸根本没碰过臂

源码 843–846 行：

```rust
        let mut sim = Sim::new(L, S, G);
        let (si, sa, ci, ca) = measurement_control(&mut sim);
        assert_eq!((si, sa), (L as u64, L as u64), "{:?} 全隔离布局必须报 runs=L", arm);
        assert_eq!((ci, ca), (1, 1), "{:?} 连续布局必须报 runs=1", arm);
```

`arm` 只出现在**断言的消息串里**。`measurement_control`（754–766 行）自己也只调 `force_to`，
不碰 `run_arm`、不碰 `step_node_group`、不碰 `step_gen_sep`。
⇒ 15 条 `name=control` 行（产物第 2–16 行）是**同一次计算重复了 15 遍**，
`.claude/singlefs-ai-sop/rules/test-discipline.md`「阳性对照必须对每一条被测的臂都跑」在**字面上**满足，
**在射程上**一条臂都没过闸。任务书这一问的判断是对的，我给它补了行号。

### 4.2 具体的坏法（跑过，不是设想）：**把组大小 knob 弄成死的**

改源码 646 行一处：

```rust
            Arm::NodeGroup { g, extra } => {
                budget_total += extra as u64;
                step_node_group(&mut sim, &dirty, g, extra, &mut group_cursor);   // ← 把 g 换成常量 8
            }
```

**全绿清单（实跑，scratchpad `mutC`）**：

- `cargo test --release`：**31 passed / 0 failed**（本文件 18 条 + lib 13 条）
- 8 条 `name=control` 行照旧 `scatter_runs=8192 compact_runs=1`
- 14 个 xfixture median **一个不差**
- write_amp 上界断言（929 行）过、守恒断言（656–658 行）过
- 18 条变异表里**没有一条**打这一处

**而产物变了**：

```
E7RESULT name=grid budget=128 arm=fmt_group_g2 family=format load=uniform runs_median=5797 write_amp=2.960 budget_spent_pct=98.0 fallback_pct=0.0
E7RESULT name=grid budget=128 arm=fmt_group_g8 family=format load=uniform runs_median=5797 write_amp=2.960 budget_spent_pct=98.0 fallback_pct=0.0
E7RESULT name=grid budget=128 arm=fmt_group_g32 family=format load=uniform runs_median=5797 write_amp=2.960 budget_spent_pct=98.0 fallback_pct=0.0
```

g2 / g8 / g32 三行**逐字相同**，而 b256/runs8 那一格的判决从 `true` 翻成 `false`。
⇒ 源码 15 行自陈的「两条格式臂的knob都扫一遍取每格最好的那个值」这句话，**没有任何东西在守**。

**根因**：18 条单测**全部直接调 `step_node_group`**（1079、1099、1112 行），
**没有一条走 `run_arm` 再看 knob 有没有传下去**。函数本身测得很扎实，**函数与 knob 之间那根线一点都没测**。

### 4.3 另外两个也全绿（一并跑过）

| 坏法 | 结果 |
|---|---|
| 去掉 708 行 `if (lo..hi).any(\|k\| written.contains(&k)) { continue; }`（本轮已写过的组还能再被轮转重落一次） | **31/31 全绿**，产物动了（b32/runs8/g2 3548 → 3556，b256/uniform 最好格式臂 3414 → 3479） |
| 把 655 行 `if t % sample_every == 0` 改成 `if false`（关掉 `run_arm` 里**全部**守恒断言） | **31/31 全绿**，产物**逐字节不变** |

对照组：把 706 行的轮转游标冻住，`node_group_step_spends_leftover_budget_round_robin`（1095 行）**当场红**。
⇒ 这套断言**不是**普遍失效，它是**只守函数、不守接线**。

**什么观测会推翻这一问的结论**：给 `run_arm` 补一条「g2/g8/g32 三条臂在同一格上不许报同一个 median」的断言，
如果它在正常源码上就红（比如某格三个 g 本来就同值），那我举的这个坏法就不是个好例子。
我查过：产物里 18 组 `fmt_group` 三元组**没有一组三值全同**，所以这条断言在正常源码上是绿的。

---

## 第五问：跨装置闸证明了什么、没证明什么

### 5.1 证明了：共享底座与 E93 是同一个东西

`run_e93_arm`（528–589 行）复现 14 个 median，走到的代码是
`Sim::new` / `move_key` / `alloc_first_fit` / `alloc_bump` / `open_empty_seg` /
`place_run` / `place_chunk` / `find_free_run` / `take_slot` / `end_checkpoint` /
`runs_inc` / `audit_runs` / `occupied` / `dirty_set` / `extend_neighbors` / `Rng` / `median5`。
这一批**确实**被钉死了，`.claude/singlefs-ai-sop/rules/show-me-test.md`「检查要立在装置之间」那条要求被满足。

### 5.2 没证明：本二进制新加的两条格式臂**一个字都没验**

`run_e93_arm` 的函数体里，下面这些各出现 **0 次**（我 grep 过函数体）：
`step_node_group`、`step_gen_sep`、`place_gen_run`、`open_empty_seg_for`、
`alloc_first_fit_gen`、`age_gen`、`extend_neighbors_budgeted`。

**实证**：4.2 与 4.3 里三个坏法，**14 个 median 全都照旧对上**。
⇒ 跨装置闸对「新加的两条格式臂是不是对的」这一问，**回答是空的**。

### 5.3 还有一条**政策臂**也在闸外，而它是三个决胜格里两格的对手

`extend_neighbors_budgeted`（491–518 行）是本轮新写的，源码 490 行自陈
「**与 E93 的 bump_neighbor 差在封顶**——不封顶就落不到等预算格上」。
它**不在** E93_MEDIANS 那张表里，跨装置闸碰不到它。
而 `pol_neighbor` 是六格里三格的最好政策臂（b32/uniform 7031、b32/runs8 5146、b128/uniform 5463），
其中 **b32/runs8 正是一个「打对折」的格**。

守它的只有一条单测 `budgeted_neighbors_respect_budget`（1184–1192 行），
取样点是 3 个脏 key、预算 0/1/6——**离 64 个脏 key、预算 256 的实际取样点很远**。

⚠️ **但我攻这一处失败了。** 我怀疑它有偏：`runs` 按 key 序遍历（504 行），
预算不够时永远先喂 key 最小的那几个 run，于是只养 key 空间低端。
建了 `pol_nb_rr`（起点在 run 列表上轮转）实测：

| 格 | pol_neighbor | pol_nb_rr |
|---|---|---|
| b32 / uniform | 7031 | **6544**（好 7%）|
| b32 / runs8 | 5146 | 5133（好 0.3%）|
| b128 / uniform | 5463 | 5426 |
| b256 / uniform | 5378 | **5461**（反而差）|
| b256 / runs8 | 3392 | **3494**（反而差）|

⇒ 偏是真的（b32/uniform 好 7%），但**幅度不足以改任何一格的判决**。如实记。

### 5.4 把两条新政策臂都算进去之后，三个决胜格还成不成立

```
b32 / runs8 ：best_policy 5146 → 3987（pol_compact_pr）；best_format 1595；1595×2=3190 < 3987 ⇒ 照旧打对折
b128 / runs8：best_policy 3922 → 3919（pol_compact_pr）；best_format 904 ；⇒ 照旧
b256 / runs8：best_policy 1339 → 1339（没人比它好）；best_format 276 ；⇒ 照旧
```

⇒ **量级轴我没攻下来。如实记。**

---

## 第六问：本材料有没有摘句

### 6.1 第二节那四个整抄块：**对 `git show HEAD` 逐字一致，没有摘句**

我把材料第 46–57、63–71、76–90、95–101 行分别与
`git show "HEAD:.claude/kb/experiments/95-老化的节点布局臂.md"` 的 24–35、36–43、10–23、44–49 行做了 `diff`，
**四块全部逐字一致**（唯一的差是围栏 ``` 那一行，属于材料自己的 markdown）。
⇒ 第二节没有摘句，材料写的「整段抄，未转述」属实。

### 6.2 但材料的行号今天已经指不到了，而且**漏了三节**

`.claude/kb/experiments/95-老化的节点布局臂.md` 在工作区里**已被改写**（`git status` 报 `M`）：
HEAD 版 57 行、标题写「**未跑**（判据已注册 2026-09-03）」；工作区版 125 行、标题写「**已跑**」。

| 材料第一节的小节清单说 | 工作区里的实情 |
|---|---|
| 「（四节，全抄）」 | 工作区有 **8 个内容小节** |
| 列了「前置」标「不抄」 | 工作区**没有「前置」这一节**（HEAD 有，在第 50 行） |
| 引 `95:44-49` 当「它答不了的」 | 工作区 44–49 行是 **`### 口径`**；「它答不了的（跑前写死 2026-09-03）」搬到了 **96–100 行** |
| 抄的标题是 `### 它答不了的` | 工作区的标题是 `### 它答不了的（跑前写死 2026-09-03）` |
| —— | **`### 口径`（44 行）没进清单** |
| —— | **`### 结果`（72 行）没进清单** |
| —— | **`### 它答不了的（2026-09-09 跑完之后才看出来的三处）`（102 行）没进清单** |

`.claude/rules/three-way-inference.md` 要求「漏掉的那一节在材料里要留下一行『不抄，因为……』，
而不是无声消失」。**这三节是无声消失的。**
⚠️ 我判这**不是材料作者的摘句**——材料是照 HEAD 写的，写的时候那三节还不存在。
它是**材料与仓在同一天里岔开了**。但后果一样：任何腿今天照材料的行号去核，都会落到别的小节上。

### 6.3 最要紧的一处：材料第五节的三条，与 kb 工作区版的三条**不是同一批**

kb 工作区版 102–121 行那一节写了 **4 / 5 / 6** 三条。
材料第五节的第 1、2 条对应 kb 的第 4、5 条（内容一致）。
**材料第 3 条（「判据 5 的措辞是任一等预算格」）在 kb 里没有；kb 的第 6 条在材料里没有。**

kb 第 6 条整抄（`.claude/kb/experiments/95-老化的节点布局臂.md:117-121`）：

```markdown
6. **结论在预算轴上不单调，而登记的三档没覆盖到转折点。**
   跑完之后在 scratchpad 副本上加了一档 extra = 512（write_amp 9×）探针（**产物未留存**，
   不进判据、不进复跑）：uniform 上 `pol_compact=1037` 反超 `fmt_group_g2=2545`，
   runs8 上 `pol_compact=195` 反超 `fmt_group_g8=401` 与 `fmt_group_g32=230`。
   ⇒ **格式臂的优势落在 1.5×–5× 这一段，预算再大就翻过去了**；转折点在 5× 与 9× 之间，没测。
```

**这一条是三条里唯一能改变读法的**，而它没进材料。

### 6.4 我独立复现了它，而且往外推了一档

我在副本上把 `BUDGETS`（源码 46 行）扩到 512 与 1024，其余一字不改：

```
budget=512   arm=pol_compact          load=uniform  runs_median=1037  budget_spent_pct=100.0  fallback_pct=0.0
budget=512   arm=pol_compact          load=runs8    runs_median=195   budget_spent_pct=100.0  fallback_pct=0.0
budget=512   arm=fmt_group_g2         load=uniform  runs_median=2545  budget_spent_pct=100.0  fallback_pct=0.0
budget=512   arm=fmt_group_g8         load=runs8    runs_median=401   budget_spent_pct=100.0  fallback_pct=0.0
budget=512   arm=fmt_group_g32        load=runs8    runs_median=230   budget_spent_pct=100.0  fallback_pct=0.0
```

**四个数与 kb 第 6 条逐字相同。** 再往外一档（extra = 1024，17×）：

```
E7RESULT name=verdict budget=512  load=uniform best_policy=1037 best_format=1005 best_format_arm=fmt_group_g8        format_halves_policy=false
E7RESULT name=verdict budget=512  load=runs8   best_policy=195  best_format=205  best_format_arm=fmt_gensep_256_1024 format_halves_policy=false
E7RESULT name=verdict budget=1024 load=uniform best_policy=576  best_format=616  best_format_arm=fmt_gensep_256_1024 format_halves_policy=false
E7RESULT name=verdict budget=1024 load=runs8   best_policy=102  best_format=133  best_format_arm=fmt_group_g2        format_halves_policy=false
```

⇒ 到 17× 时 `pol_compact` 在**两个负载上都赢过全部六条格式臂**。
**翻面不是单点毛刺，它随预算单调加深。** kb 第 6 条成立，而且比它写的更强。

### 6.5 一处顺带查出来的事实：跑完之后动过一个「跑完不许改」的判据

```
HEAD    :28  3. **阳性对照每臂都过闸**：强制放置开关下全隔离布局报 runs == L、连续布局报 runs == 1。
工作区   :28  3. **阳性对照每臂都过闸**：强制放置开关下全隔离布局报 runs == L，连续布局报 runs == 1。
```

一个顿号改成逗号，语义没变。但那一节的标题逐字是
`### 判据（跑前写死 2026-09-03，跑完不许改）`。**我只报事实，不判它。**

---

## 第七：一处材料第四节的说法，我判它不成立

材料第四节把回落率当赢面的佐证：「节点组臂多数格是 0.0，政策臂在 31.4–88.1 之间」，
kb 工作区版 92–94 行同样写了这一句。

**`fallback_pct` 与 `runs_median` 在这个模型里是脱钩的。** 干预实测：
我给 `place_gen_run`（372–403 行）补上 `place_chunk` 本来就有的那条连续 run 搜索分支，
代际分离臂的 `fallback_pct` 在**全部 18 格**从 55.7–89.8 掉到 **0.0**，
而 `runs_median` **18 格逐字不变**（8121 / 7103 / 7827 / 6593 / 8022 / 6503 / 7956 / 6672 / 7093 /
4924 / 6869 / 4130 / 7777 / 6219 / 6039 / 2834 / 4347 / 1343，两边完全相同）。

**原因是可以指到行的**：`len == 1` 时，`find_free_run(1)`（280–303 行）与 `alloc_first_fit`（238–247 行）
返回**同一个槽**——都是 `self.free` 迭代序里第一个不在开放段的槽。
差别只在前者**不给 `fallback_allocs` 加一**。
⇒ `fallback_pct` 量的是**走了哪条分支**，不是**落得好不好**。
格式臂 `fallback_pct=0.0`，有一部分只是因为它走 `place_chunk`（有第三条分支）而政策臂走 `alloc_bump`（没有）。

**推论**：材料第四节与 kb 92–94 行那个对比**不能当赢面的机制解释**。它是真的数，但它解释不了 runs。

**什么观测会推翻这条**：找到一格，`fallback_pct` 变化而 `runs_median` 跟着同向变化，且能排除别的变量。
我在代际分离那 18 格上没找到。⚠️ 我**没有**在节点组臂上做同样的干预——那一格未测。

---

## 附：本答复里我知道自己没测的几格

按 `.claude/singlefs-ai-sop/rules/kb-discipline.md`「显式记录不知道」列出来，别让它变成空白：

1. **摘掉格式臂的 `find_free_run` 分支之后赢面还剩多少**——没跑（第 1.2 节末）。
2. **建一条「允许少花预算」的政策臂**，看 1.3 节那个自由度值多少——没跑。
3. **`⌊k/N⌋` 在 D8 定的树形态下写路径拿不拿得到**——我没去读 `.claude/kb/decisions/08-核心索引结构.md` 核这一条。
   这是第二问那整条攻击唯一的支点，**它现在是推的，不是查的**。
4. **换配置（L / S / G / 种子集）之后最优 g 还随不随预算漂**——只在一组配置上扫过。
5. **节点组臂上做 `fallback_pct` 干预**——只在代际分离臂上做过。
6. 六个探针的产物**未入库**，与 kb 第 6 条那个 512 探针同一性质。要拿去改决策，得先按
   `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 入库并注册进 replay。

## 历史版本

（本文件是 2026-09-09 那一轮三方论证的反推腿原样答复，按
`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」，
要补说明请另开文件。）
