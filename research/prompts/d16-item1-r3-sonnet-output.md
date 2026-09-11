# D16 未定项 1 第三轮三方论证——正推 / 校验腿输出（Sonnet）

立场：正推 + 校验。逐条核材料第一节 P1–P10，核 E138 源码与产物，逐判据（3-1~3-7）判臂 A / G′，并在副本里给 G′ 加了一条独立探针。

## 方法说明

- 把 `research/` 整份拷到 `/tmp/claude-1000/.../scratchpad/sonnet-copy/research/`，未改仓里任何文件。
- 在副本里 `cargo test --release --bin e138_per_disk_floor` 跑通全部 **15 个原有单测**（全绿，逐条命令与输出见下）。
- 在副本里加了 3 个新测试（`g_prime_forced_disposal_probe`、`g_original_failure_is_also_one_time_transient_s4_c5`，以及在 `Simulation` 上加了一个 `force_full_disposal_for_g_prime_probe: bool` 字段 + 在 `run_user_window` 的准入分支把 `push_after_commit` 的 `stop_when_writable` 改成 `!self.force_full_disposal_for_g_prime_probe`），用来独立构造材料里的臂 **G′（处置做满）**——E138 本身没建这条臂，只有推测。这是本报告最主要的新证据，见下「三、判据 3-7 与 G′ 独立验证」。
- 没有改动 `research/prompts/e138-preregistration.md` 定义的任何式子；对 A、C、D、G 六条原臂的行为一个字节都没动，只加了新测试与一个默认关闭的开关字段。

## 一、P1–P10 逐条核对

| # | 材料原文 | 核对结果 | 出处 |
|---|---|---|---|
| P1 | D3 已定项 9 两条：`df` ≥ s 时写 s 必须成功；删 s 后同样大小的写在有界步数内成功 | **核对无误**，整行与 `03-空间分配.md:276-305` 一致（附录已整段抄） | `.claude/kb/decisions/03-空间分配.md` |
| P2 | 盘紧时最少保留 4 个持久根，且每块盘上最新持久有效根都在内 | **核对无误**；代码里 `MINIMUM_RETAINED_ROOTS: usize = 4`，`per_disk_retention_upper_limit()` 逐字实现「fourth_newest.min(newest_on_every_disk)」 | `e138_per_disk_floor.rs:18, 421-430` |
| P3 | D23 已定项 14：候选集 = 根环里按实例表判仍然有效的根 | **核对无误**；`is_instance_valid` / `valid_roots_with_disk` 实现与 `23-journal的角色与格式.md:1177-1202` 定义一致（`(i,T)` 有效 ⟺ 实例表无 i 的行，或有行且 `T ≤ Ti`） | `e138_per_disk_floor.rs:380-406` |
| P4 | 判决：两条候选臂零假候选零不可恢复（除 rollback_first_publish）；`df_guaranteed` 下假性 ENOSPC 都是 0；G 在第 2 条出局（S=4/16 各 24/24）；A 两条都达标，界 N 次发布 | **核对无误**，与 `results/e138-per-disk-floor-2026-09-11.out` 第 314 行 `crit3_a_fakes=0 crit3_a_unrecoverable=0 crit3_g_fakes=0 crit3_g_unrecoverable=0 crit4_a_false_guaranteed=0 crit4_g_false_guaranteed=0` 及 `crit5_g_write_after_delete_failed=104`（含近满盘表与保留池扫描的合计，见下方独立复算）逐字一致 | `results/e138-...out:314` |
| P5 | 跑前推的式子：A 保留池 2×5+(N−1)c、残留 5+(N−2)c；G 保留池 2×5+6c、残留 5+5c | **核对无误**，与源码 `reserve_blocks` / `push_residue_blocks` 函数体逐字一致，且判据 1（`crit1_push`）全部 `holds=1` | `e138_per_disk_floor.rs:104-122` |
| P6 | S=16 代价：A 约 36.2/39.3 次强制发布、`df` 少报 108/480 块（1.7/7.5 MiB）、非空发布根平均 2.3/2.2；G 约 4.9 次、少报 26/70 块、非空根约 4.3/4.0（最少 5） | **核对无误**，独立复算：`near_full arm=A... s=16 empty_cost=1` 行 `forced_per_window_x100=3622`→36.22，`hidden_blocks=108`（108×16384=1769472 B≈1.6875 MiB，材料写 1.7 MiB 合理取整），`avg_nonempty_x100=230`→2.30；`empty_cost=5` 行 `forced_per_window_x100=3928`→39.28，`hidden_blocks=480`（480×16384=7864320 B=7.5 MiB 精确），`avg_nonempty_x100=221`→2.21。G 侧 `empty_cost=1`：`forced_per_window_x100=486`→4.86，`hidden_blocks=26`；`empty_cost=5`：`485`→4.85，`hidden_blocks=70`，`avg_nonempty_x100=395`→3.95（材料写 4.0，四舍五入合理） | `results/e138-...out:722-725` |
| P7 | 它答不了的十条：不建实例切换；空发布开销只取两档；没有「活化区间内崩溃、不丢盘」；「按盘取上限」没测到；G 输的机制是推的 | **核对无误**，代码逐条印证：`run_activation_fault_case` 结尾**无条件**执行 `simulation.disk_lost[lost_disk]=true` 才 `crash_and_recover()`，代码里没有跳过丢盘的分支——确认「没有活化区间内崩溃、不丢盘」这格是格式/代码层面的空白，不是文档漏写；单测 `txg_upper_limit_control_is_not_reached_because_floor_is_raised_before_the_jump` 的注释与判据 2 的 `crit2_gtxgcap_retention=0` 互证「按盘取上限」确实未测到 | `e138_per_disk_floor.rs:1066-1082, 1543-1547` |
| P8 | C281：六条臂各 72 个不可恢复 | **核对无误**，`crit8_c281 a=72 c=72 d=72 g=72 g_count=72 g_txg_cap=72` 逐字一致 | `results/e138-...out:316` |
| P9 | C283、I-5.3 现状引用 | **核对无误**，`invariants.md:158-178` 附录整段核对，I-5.3 checker 状态列确为「未实现」 | `.claude/kb/invariants.md` |
| P10 | 臂 A 不改字节；动态回退下界要加 8 字节 F | **核对无误**，与 `16-发布语义.md` 未定项 1 分项 2 的正文一致（F 字段、194→202 那句） | `.claude/kb/decisions/16-发布语义.md:397-401` |

**结论：P1–P10 没有发现摘句、算错或口径不一致的问题。** 这与三方论证纪律「材料里的错会被三条腿一起继承」的检查方向相反——我没找到可继承的错误，这是一次「没打中」的核实，按 `three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」的精神，我在这里只报告「一次核对没找到问题」，不代表材料整体无误，仅代表这十条前提本身经得起核对。

## 二、E138 源码与判据的独立复核

### 装置一致性（判据 1）

副本编译、跑测试：

```
$ cargo test --release --bin e138_per_disk_floor
running 15 tests
... (全部 15 个测试 ok)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

单测数 15、变异表 `research/mutations/e138_per_disk_floor.tsv` 9 行（无表头），与经验文档「这是计数模型，不是实现」一节「15 个单测 + 9 条变异」逐字一致（**我核过**，未跑变异 harness 本身，只核对了行数与测试通过）。

### `df_guaranteed` 的口径（供判据 3-4 用）

```rust
fn df_guaranteed(&self) -> u64 {
    self.df_raw().saturating_sub(self.reserve_blocks + self.push_residue_blocks)
}
```

`reserve_blocks` 与 `push_residue_blocks` 都是**跑前按 N（或 B_G=7）、S、`empty_publication_cost` 算出的常量**，与当前池填充率、对象数量、盘上内容都无关；只在构造 `Simulation` 时算一次，之后不变。⇒ **少报的量是一个只随几何参数（S 从而 N）与空发布开销 c 变的常量，不随盘上内容涨。** 这是我自己从源码推的（**我推的**，不是抄材料），与材料判据 3-4「少报的量是不是有界，随什么涨」直接相关：

- 对臂 A：少报 = 2×5 + (N−1)c + 5 + (N−2)c = 15 + (2N−3)c，**随 N（即 S）线性涨**，不随盘大小、不随已用空间涨。S=16、c=5 时 = 480 块 = 7.5 MiB。
- 对臂 G：少报 = 15 + 11c（S 无关），S 任意、c=5 时恒为 70 块 = 1.09 MiB。

⇒ **判据 3-4 的判断**（我的判断，不代表用户已确认）：少报量是**有界常数**，符合 D3 已定项 9「界要写成算得出的数，不许是『最终会』」这条硬要求本身；但它的**绝对值**随用户后续选定的 S（根环槽宽，属于 D22 单元原子性怎么合成、D9/mkfs 几何参数）而定，S 越大越省心理容量（越少假性 ENOSPC 概率、越多历史深度）但 `df` 越保守。S=16、c=5 时臂 A 少报 7.5 MiB——这个绝对值在 TB 级盘上可忽略，在追求「每一块都要精确可见」的极小容量设备上可能不可忽略。**这件事本身不违反用户「不许假性 ENOSPC」的原意**（少报只会让文件系统更早报 ENOSPC，不会让它在有空间时报 ENOSPC），只是「`df` 显示的空闲 < 物理空闲」这件事需要用户知情——материал 已经把这一点写进失败条款「若 3-4 成立⇒写在最前面」，我的核对结果是：**3-4 不构成判红（不存在『随盘上内容涨』的情形），但绝对值随 S 涨这件事值得在代价表里显式标出**（已在下面第五节标出）。

## 三、判据 3-7 与 G′ 独立验证（本报告的主要新证据）

E138 文档「它答不了的」第 10 条明写：「臂 G 输的机制是推的，没有单独量」「改法方向（推的，要新立实验）：臂 G 的处置不管落点一律做满 7 次，残留就恒定」。材料把这个推测直接做成候选臂 G′，但**从未真正实现过**。我在副本里把它实现了出来。

### 实现方式（我改了什么）

在 `Simulation` 上加一个默认 `false` 的字段 `force_full_disposal_for_g_prime_probe`；`run_user_window` 里准入分支原本硬编码 `push_after_commit(committed_txg, true)`（`stop_when_writable=true`，一有足够空间就停），改成：

```rust
let stop_when_writable = !self.force_full_disposal_for_g_prime_probe;
let publications = 1 + self.push_after_commit(committed_txg, stop_when_writable);
```

`stop_when_writable=false` 正是判据 1 的探针（`push_probe`）已经在用的那条路径——`push_after_commit(goal_txg, false)` 的停止条件是 `is_goal_released(goal_txg)`，即 `reuse_bound() >= goal_txg`，这恰好等价于「F 已经在每块幸存盘上都持久生效到 goal_txg」，与材料 G′ 定义「不管可分配够没够，一律做到带新 F 的根在每块盘上都持久」**逐字对应**。这不是我另造的机制，是把已有的探针路径接到准入分支上。

对 A、C、D、原 G、G_count、G_txgcap 六条臂**行为零改动**（默认 `false` 时与原代码路径完全相同）——15 个原测试全部照旧通过，证明这条改动没有副作用。

### 结果：G′ 与原臂 G 在判据 5 上**完全相同**，「做满」没有解决问题

新增两个测试，S=4 与 S=16、c∈{1,5}，24 个种子：

```
G′_forced s=4  c=1 write_after_delete_failed=0  ... seeds_with_failure=0  seeds_with_gt1_failure=0
G′_forced s=4  c=5 write_after_delete_failed=24 ... seeds_with_failure=24 seeds_with_gt1_failure=0
G′_forced s=16 c=1 write_after_delete_failed=0  ... seeds_with_failure=0  seeds_with_gt1_failure=0
G′_forced s=16 c=5 write_after_delete_failed=24 ... seeds_with_failure=24 seeds_with_gt1_failure=0
```

对照原臂 G（未改）:

```
G_original c=5 s=4: total_failures=24 seeds_with_failure=24 seeds_with_gt1_failure=0 seed0_failure_windows=[2]
```

**四个数字（G′ 在 s=4/16、c=5 下各 24 次失败）与原臂 G 在 E138 正式产物里的数字（`near_full arm=G_floor_per_disk s=4 empty_cost=5 ... write_after_delete_failed=24`、`s=16 empty_cost=5 ... write_after_delete_failed=24`）逐字相同。**

进一步追踪种子 0 的窗口序列（`g_original_failure_is_also_one_time_transient_s4_c5` 测试，逐窗口打印 `pinned_before/after`、`floor_carried`、`floor_active`）：失败**只发生在窗口 2**（`seed0_failure_windows=[2]`），恰好是「填到第一次 ENOSPC」阶段刚结束、转入「删一个建一个」阶段的**冷启动窗口**；此后进入稳态，`pinned` 在 30/35 之间规律振荡（30 = 5+5×5 的残留式子，35 = 30+5，即上一轮元数据尚未放掉时的瞬时值），**全部 Created，没有再失败**。24 个种子里每个恰好失败 1 次（`seeds_with_gt1_failure=0`），说明这不是「稳态里周期性复发」，而是**每颗种子在冷启动那一刻各撞一次的瞬态**。

**这与决策文档「臂 G 为什么输」那段叙述的机制解释不符**：那段推测的机制是「上一次处置落在少的那一档（6 次发布）、这一次落在多的那一档（7 次发布）时，删掉的 8 块里有『开销』那么多被新的残留吃掉」——这描述的是一个**稳态里周期性、随每次删建循环反复发生**的现象；而我的追踪显示失败**只在冷启动那一刻发生一次**，稳态里从不复发，且**在强制「做满」（G′）之后现象完全没变**。如果「落点决定残留」的机制解释是对的，G′ 应该能消除这个失败（因为 G′ 把残留钉死为恒定的 5+5c，不再有 6 次/7 次两档之分）——但实测 G′ 与原 G 在这一格上**逐种子逐次数完全相同**。

⇒ **判据 3-7 的判断**：材料 P7「G 输的机制是推的」这句话本身是准确的（E138 文档自己也承认没有单独量），而我独立验证之后发现**推测的机制很可能是错的**——真正的成因看起来是「从『只建不删』的填满态切换到『先删再建』态时，`floor_carried`/`floor_active` 需要一次性从填满前的水位追上来，这次追赶本身要吃掉比稳态更多的净空间，与落在哪个发布数档位无关」。这不是稳态问题，是一次性的启动开销。**这个判断是我推的，没有进一步做变量控制实验去证实具体机制**（比如把 warm-up 阶段也换成规律的删建循环、看失败是否消失），如果要把这个判断写进 kb，还需要再补一轮实验——我在这里只能确认「G′（做满）在这一格上不能修复判据 5」这个**观测结果**是可靠的（同一现象在 S=4 与 S=16 两组几何下都复现，且与原 G 的失败逐种子对应）。

**对候选 G′ 的直接影响**：如果 G′ 就是材料定义的那种（同 G，只是处置一律做满），那么**它在判据 5 上和原臂 G 一样出局**（S=4、S=16，c=5 时各 24/24 个种子删了再写失败）——**候选臂 G′ 应当判「输」，不是材料倾向段暗示的「留作反过来的选项」**。材料第五节「结果反过来我接不接受」写「若有腿构造出臂 A 违反 3-1/3-2/3-3 ⇒ 取 G′ 并立实验量它」——按我这次的探针，G′ 现在**不需要额外立项去验**，因为它在同一失败条款上已经和原 G 一样输了，**除非「做满」的实现方式与我的理解不同**（例如材料是否想要「做满」还包含把 warm-up/冷启动阶段也纳入强制处置逻辑——如果是,那是另一种构造，我没有测那种）。这一点请主 agent 核实材料对 G′ 的具体定义是否与我的实现一致；我的理解依据是材料原文「不管可分配够没够，一律做到带新 F 的根在每块盘上都持久（最多 7 次发布），让残留恒为 5 + 5 × 空发布开销」——我的实现确实做到了这一句字面要求（残留在稳态下确实恒为 30 = 5+5×5，这一点被判据 1 的 `push_probe` 断言钉住），但**残留恒定本身没有解决冷启动那一次的失败**。

## 四、判据 3-1 / 3-2 / 3-3 逐条

| 判据 | 臂 A | 臂 G（材料候选 G′ 视为同一形态） |
|---|---|---|
| 3-1 正确性 | **过**（`crit3_a_fakes=0 unrecoverable=0`，含 activation_faults 16×8×2 组合）。**但**这个「过」的射程只到 E138 建过的世界——journal 重放、实例切换、活化区间内崩溃不丢盘（P7 第 8 条）都没测，我没能在副本里补出这些世界（工作量超出这一轮，未做，如实说未做） | 同上（`crit3_g_fakes=0 unrecoverable=0`），射程同样受限；另外 G_txgcap 那个「按盘取上限」的判别力本身没被测到（判据 2），这削弱了「按盘」这半机制本身被验证的程度，但不直接影响 G 在 3-1 的判定 |
| 3-2 第 1 条（假性 ENOSPC） | **过**（`crit4_a_false_guaranteed=0`），独立复算 `near_full` 全部 6 个 S×c 格逐格 `false_enospc_guaranteed=0`，且 `raw_df_detector_fires_for_ring_arm_near_full` 证明检测器本身会红（阳性对照） | **过**（`crit4_g_false_guaranteed=0`），同样逐格核对 |
| 3-3 第 2 条（有界步数可用） | **过**，界 N，`crit5_a_bound_exceeded_cells=0 write_after_delete_failed=0` | **出局**，`crit5_g_write_after_delete_failed=104`（近满盘 6 格 + 保留池扫描合计，我独立复算过其中一部分：近满盘 S=4/16、c=5 各 24，共 48；保留池扫描部分我未逐行重算，只核对了 `reserve_sweep` 那 9 行整行抄自产物）。**G′ 在这条上与 G 一样出局**（见上一节） |

## 五、判据 3-6：代价表（只报数）

| 臂 | 界（次发布） | `df` 少报（S=16,c=5） | 非空候选根数（S=16,c=5，平均） | 每用户窗口强制发布数（S=16,c=5） | 格式字节 |
|---|---|---|---|---|---|
| A（整环+推空） | N=3S（S=16 时 48） | 480 块 = 7.5 MiB，公式 15+(2N−3)c，**随 S 线性涨** | 2.21 | 39.28 | 0（不改盘上字节，P10） |
| G′（按盘回退下界+做满） | 7（与 S 无关） | 70 块 = 1.09 MiB，公式 15+11c，**与 S 无关** | 3.95 | 4.85（但判据 5 出局，这个代价数字对已出局的臂只做参考） | 根记录 +8 字节 F（194→202，P10） |

## 六、判据 3-5：动没动用户定案

两条候选臂的定义都**没有改动** D23 已定项 14 与 P2 的最少保留口径——A 沿用「候选集=按实例表有效的全部根」原样，G′ 沿用 D16 未定项 1 第二轮已有的「按盘取上限」形态,只改了「盘紧时怎么处置」这一列。**判 3-5：不涉及，不需要用户重判**。

## 七、判据 × 臂 总表

| 判据 | 臂 A（整环+推空） | 臂 G′（按盘回退下界，处置做满） |
|---|---|---|
| 3-1 正确性 | 赢（射程有限，未测的世界见 P7 第 3/4/8 条） | 赢（同样射程有限；「按盘取上限」半条机制的判别力本身未验证） |
| 3-2 第 1 条（不许假性 ENOSPC） | 赢 | 赢 |
| 3-3 第 2 条（有界步数可用） | 赢 | **输**（S=4/16、c=5 各 24/24 个种子；本报告独立验证「做满」不能修复这个失败，与决策文档给出的机制解释不符，见第三节） |
| 3-4（`df` 口径合不合用户原意） | 记下：少报有界、随 S 线性涨（S=16,c=5 时 7.5 MiB），需用户确认是否可接受 | 记下：少报有界、与 S 无关（1.09 MiB） |
| 3-5（动没动用户定案） | 不涉及 | 不涉及 |
| 3-6（代价） | 见第五节 | 见第五节 |
| 3-7（界的算术） | 界 N 成立（判据 1 全中，`crit1_push` 逐格 `holds=1`） | 界 7 成立（同上）；**但「做满」这个改法本身没有解决判据 3-3 的失败，候选 G′ 按此判据应与原 G 同判出局** |

## 八、给主 agent 的结论性建议（我的判断，供参考，不代表定案）

1. **臂 A 在我核过的判据上全部过关**，材料倾向臂 A 的结论在这一轮里没有被我推翻，且 3-1/3-2/3-3 的独立复算与材料一致。
2. **候选 G′（按材料原文定义的『做满』实现）在判据 3-3 上与原臂 G 一样出局**——这与材料自己的失败条款「若有腿构造出臂 A 违反 3-1–3-5 ⇒ 取 G′ 并立实验量它」的前提假设（G′ 能修好判据 5）不符。这是本报告与材料倾向最大的分歧点，**建议在判决前让反推腿或主 agent 复核我在副本里的改法是否忠实于材料对 G′ 的定义**（我的改法见第三节，改动的确切代码差异可以按需要重新生成 diff）。
3. **判据 3-4（`df` 口径）不构成判红**，但少报绝对值随 S 线性涨这件事，如果 D16 未定项 1 最终选 S 偏大的几何，需要在交用户的代价表里显式标出（不能只写「少报常数」，要写清随 S 怎么涨）。
4. 我没有触及、也没有能力在这一轮独立验证的：journal 重放窗口、实例切换、真机压测——这些是 E138 自己承认的空白（P7），我的核对没有改变这个状态。
