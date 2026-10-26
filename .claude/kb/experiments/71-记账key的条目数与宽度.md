## E71 记账 key 的条目数与宽度 —— 已跑（2026-09-01，纯算术，12 单测 / 8 条变异全抓；2026-10-20 第 1 次重跑判据 2 按每条根只装一代重算）：**先撑爆的是 `树数 × 设备数` 这个乘积，且只在「维度用满」那个读法下撑爆**

**中文** · [English](71-entry-count-and-width-of-accounting-keys.en.md) · [日本語](71-記帳キーのエントリ数と幅.ja.md)

<!-- doc-lint:not-numbers Q3 FC1 FC2 FC3 FC4 FC5 V1 V2 V3 V4 S1 S2 S3 EV1 EV2 EV3 EV4 EV5 EV6 EV7 EV8 EV9 EV10 -->

⚠️ **判据 2 已按「每条根只装一代、只写不删」重跑一次（2026-10-20）**，结果在「第 1 次重跑（2026-10-20）：判据 2 按每条根只装一代、只写不删」一节；本页「判据 2：一次 checkpoint 的记账写字节 vs `T_dirty` 的 1%」那一小节是旧口径（一写一删 × 2）的数。

D5（快照 / 空间记账机制） 已定项 1 把 key 定成 `(统计量, 树 ID, 设备, 代)`。
E54（记账的「代」要保留几代） 当时给的公式 `key 数 = 统计量数 × K` **少了维度组合那两段**。
绝对值在之前算不出来，因为统计量个数还没定；D5（快照 / 空间记账机制） 已定项 4（用户定案）
定了**九个统计量**之后就算得出来了。

⚠️ **射程注（2026-09-03）：E71（记账 key 的条目数与宽度） 跑的是 s = 9，而 D5（快照 / 空间记账机制） 已定项 4
在同月 3 日第一次重开，清单变成十一个。** 二进制与产物一个字不改（`replay.sh` 要逐字节比对），
按 s = 11 重算是算术，不是重跑：上界臂 `s × t × d × K` 在 t=64 / d=8 / K=25 一格从 **115 200 变 140 800**；
窄读臂从 **4075 变 4300**（新加的两项都不按树分——碎片度 runs 无维 ⇒ +K=25，
全空聚簇段数带设备维 ⇒ +d×K=200）。**两条结论都不动**：先撑爆的仍是 `树数 × 设备数` 的乘积，
上界相对窄读的高估倍数从 28.3 变 32.7。
### 判据（跑前写死，跑完没改）

1. **绝对值断言**：条目数恰好等于 `s × t × d × K`，不许只做臂间互比。
2. 一次 checkpoint 的记账写字节数 ≤ D16（发布语义） 已定项 5 的 `T_dirty` 的 1%，
   否则判「记账自己就吃掉了脏数据预算」。
3. 四个自变量各自单调：任一个加倍，条目数恰好加倍。

### 失败条款（跑前写死）

| 条款 | 判定 |
|---|---|
| **阳性对照**：去掉设备维，条目数必须**下降 d 倍**；没下降说明维度根本没进 key，整轮作废 | **通过**（上界臂 115200 → 14400，恰好 8 倍）|
| 若 s 取不到真值，结果带「按 s = N 算」的口径，不许写成绝对结论 | **不再适用**：s = 9 已定 |

### ⚠️ 判据 1 与 3 说的那个公式是上界，不是条目数

`s × t × d × K` 把九个统计量**一律**乘上树维与设备维，
而 D5（快照 / 空间记账机制） 已定项 4 逐字写着「**key 的维度元组不是每个统计量都用满**，
不按树分的那几项用一个保留树 ID（取 0）当『全池 / 无归属』行」。

判据是在 D5（快照 / 空间记账机制） 已定项 4 定案**之前**写的
（E71（记账 key 的条目数与宽度） 正文当时逐字写着「绝对值今天算不出来」）
⇒ **上界臂照原样跑，判据 1 / 3 与阳性对照全部落在它身上，一个字没改**；
另加两条按分项计维的臂，把差额量出来。

### ⚠️ 一处 D5（快照 / 空间记账机制） 已定项 4 没定的东西，E71（记账 key 的条目数与宽度） 不替它定

「哪几个统计量**带树维**」那张表**没有列**——它只列了带不带设备维。
⇒ 按两种读法各跑一臂，**不挑一个当答案**：

| 读法 | 按树分的项 | 带设备维的项（D5（快照 / 空间记账机制） 已定项 4 逐字给）|
|---|---|---|
| **窄读** | 只有第 8（每树独占）、9（每树共享）两项 | 第 1、2、3、5 项，共 4 项 |
| **宽读** | 九项全部 | 第 1、2、3、5 项，共 4 项 |

### 实测

#### 三条臂的条目数（t=64 树、d=8 盘、K=25）

| 臂 | 一代的条目数 | 总条目数（× K）| 相对上界 |
|---|---|---|---|
| **上界（维度用满）** | 4608 | **115 200** | 1.00× |
| 宽读（九项按树） | 2368 | 59 200 | 0.51× |
| **窄读（两项按树）** | **163** | **4 075** | **0.035×** |

⇒ **上界比窄读大 28 倍。** 差额全部来自「维度用满」这个读法，
而 D5（快照 / 空间记账机制） 已定项 4 逐字否掉了那个读法
⇒ **拿 `s × t × d × K` 去估记账树大小会高估一到两个数量级。**

#### 判据 3：加倍试验

| 臂 | 变量 | 基准 | 加倍后 | 恰好 2 倍 |
|---|---|---|---|---|
| 上界 | t / d / K | 115 200 | 230 400 | **是（三个变量都是）** |
| **窄读** | t | 4075 | 7275 | **否，1.785 倍** |

⇒ **判据 3 在上界臂上成立、在真实 key 上不成立**，而后者才是要落地的那个。
**这是结果不是 bug**：只有 2/9 项按树分，树数加倍只放大那 2 项。

#### 阳性对照：去掉设备维

| 臂 | 带设备维 | 去掉后 | 倍数 |
|---|---|---|---|
| **上界** | 115 200 | 14 400 | **恰好 8 倍（= d）✅** |
| 宽读 | 59 200 | 14 400 | 4.11 倍 |
| 窄读 | 4075 | 3375 | 1.21 倍 |

⇒ 阳性对照在它被写来验的那条臂上**恰好成立**，证明设备维确实进了 key；
另两条臂降得少，正因为只有 4/9 项带设备维。

#### 判据 2：一次 checkpoint 的记账写字节 vs `T_dirty` 的 1%

⚠️ 这一小节是 2026-09-01 的旧口径（一写一删，写次数系数 2）。D5（快照 / 空间记账机制） 已定项 2 改成每条根只装一代之后的新口径（系数 1）见「第 1 次重跑（2026-10-20）：判据 2 按每条根只装一代、只写不删」一节，两种口径逐格并列在那一节的产物里。

`T_dirty` = 2 GiB（D16（发布语义） 已定项 5）⇒ 预算线 **21 474 836 字节**。
一次 checkpoint 触及**一代**的全部条目，各写一次各删一次
（D5（快照 / 空间记账机制） 已定项 2 逐字：写次数从 `统计量数` 变成 `统计量数 × 2`）⇒ K 不进这个式子。

**90 个格里 9 格过界，全部在上界臂上；窄读臂一格都没过。**

| 格 | 上界臂 | 窄读臂 | 预算 |
|---|---|---|---|
| t=1024, d=64, key+val=30 B | **35 389 440（超 1.65 倍）** | 138 420 | 21 474 836 |
| t=8192, d=8, key+val=30 B | **35 389 440（超 1.65 倍）** | — | 21 474 836 |
| t=8192, d=64, key+val=50 B | **471 859 200（超 22 倍）** | — | 21 474 836 |
| t=1024, d=64, key+val=22 B | **25 952 256（超 1.21 倍）** | — | 21 474 836 |

⇒ **先撑爆的不是四个自变量中的任何一个，是 `t × d` 这个乘积**：
两格过界点（1024×64 与 8192×8）**乘积都是 65 536**，字节数逐字相同。
⇒ 撑爆条件是 `t × d ≳ 65536`（30 字节宽度、上界读法下）。

⚠️ **K 不在这个条件里**——它只放大树的总大小，不放大单次 checkpoint 的写。
⇒ **E54（记账的「代」要保留几代） 关心的 K 与判据 2 关心的量是两回事，不许并成一句。**

#### K 的取值范围（D5（快照 / 空间记账机制） 已定项 2 + D22（单元原子性怎么合成） 已定项 2）

`K = 根环槽总数 + 1 = R × S + 1`，R = 3，S 在 1..16 之间取值：

| 每区槽数 S | 槽总数 | 保守 K |
|---|---|---|
| 1 | 3 | 4 |
| 8 | 24 | 25 |
| 16 | 48 | 49 |

E53（丢一整块盘之后根环还挂不挂得上） 真设备实测的最坏回退是 1 ⇒ 另一个口径 **K = 2**。
两个口径差 24.5 倍，而 **K 只线性放大总条目数**，不影响判据 2。

### 第 1 次重跑（2026-10-20）：判据 2 按每条根只装一代、只写不删

**状态：已跑（够判）**，纯算术，5 轮逐字节一致。跑前登记 `research/prompts/e71-r1-prereg.md`（问题单 `research/prompts/c693-forks.md` 的 Q3，E71（记账 key 的条目数与宽度） 那一半；欠账 C693（K 代撤回之后要重推的论证））。

**只改一处口径**：一次发布的记账写字节 = 写次数系数 × 一代条目数 × 条目宽，系数从 2（一写一删）改成 1——D5（快照 / 空间记账机制） 已定项 2 定案「每条根的记账树只装这条根那一代的行，不留多代」、射程「每次发布把这一代的行全写一遍」。照原装置不动的：2026-09-01 那张九项统计量表（带设备维 4 项，窄读只有 2 项按树分，宽读九项都按树分）、三档宽度 22 / 30 / 50 字节、t ∈ {1, 8, 64, 1024, 8192}、d ∈ {1, 8, 64}、门槛 21 474 836 字节（D16（发布语义） 已定项 5 的 `T_dirty` = 2 GiB 的 1%，严格大于判过界）。**这一节的数一律是「按 2026-09-01 那张九项表、新行数口径」的数**：今天 D5（快照 / 空间记账机制） 已定项 4 在用十二项，这一次没换表。

产物 `research/results/e71-accounting-keys-single-generation-2026-10-20.out`（302 行，收尾行 `E7RESULT name=done emitted=302`）。每格一行 `name=publish_budget`，四档宽度（原三档加 34 字节）× 15 个 (t, d) × 三条臂 = 180 行，旧口径字节、新口径字节、两种判定与新口径占门槛的千分比并列。

**判决格（上界臂、30 字节、t=1024、d=64）新口径不过界**（产物第 288 行）：

```
E7RESULT name=judgement_cell arm=upper_bound width=middle trees=1024 devices=64 old_bytes=35389440 old_over=1 new_bytes=17694720 new_over=0 budget=21474836
```

新口径 17 694 720 字节，占门槛 823‰。登记的失败条款 FC1（D5（快照 / 空间记账机制） 已定项 4 依据里「`1024 × 64` 时……过 `T_dirty` 的 1%」在 E71（记账 key 的条目数与宽度） 这一侧不成立）**触发**。

**每条臂的过界格数**（产物第 289–294 行；`registered_widths` 是原三档 45 格，`current_implementation_width` 是 34 字节那 15 格）：

```
E7RESULT name=over_budget_count arm=upper_bound widths=registered_widths cells=45 old_over=9 new_over=5 new_over_cells=narrow:8192:64,middle:8192:64,wide:1024:64,wide:8192:8,wide:8192:64
E7RESULT name=over_budget_count arm=upper_bound widths=current_implementation_width cells=15 old_over=3 new_over=1 new_over_cells=current_implementation:8192:64
E7RESULT name=over_budget_count arm=per_statistic_narrow widths=registered_widths cells=45 old_over=0 new_over=0 new_over_cells=none
E7RESULT name=over_budget_count arm=per_statistic_narrow widths=current_implementation_width cells=15 old_over=0 new_over=0 new_over_cells=none
E7RESULT name=over_budget_count arm=per_statistic_wide widths=registered_widths cells=45 old_over=5 new_over=3 new_over_cells=narrow:8192:64,middle:8192:64,wide:8192:64
E7RESULT name=over_budget_count arm=per_statistic_wide widths=current_implementation_width cells=15 old_over=1 new_over=1 new_over_cells=current_implementation:8192:64
```

| 臂 | 原三档 45 格里旧口径过界 | 新口径过界 | 读法 |
|---|---|---|---|
| 上界（维度用满） | 9 | 5 | 新口径过界的格都在 t × d 大的一端：30 字节那档只剩 8192 × 64，1024 × 64 与 8192 × 8 两格都是 17 694 720、不过界；50 字节那档 1024 × 64、8192 × 8、8192 × 64 三格过界。FC2（过界格数为 0）不触发。原页「撑爆条件 `t × d ≳ 65536`（30 字节）」在新口径下不成立：30 字节那档的取样点里过界的只有 t × d = 524 288 那一格（65 536 与 524 288 之间没有取样点，翻面点没量到） |
| 窄读 | 0 | 0 | 两种口径一格都不过。新口径最紧一格是 50 字节、8192 × 64：`new_bytes=832150`、`new_permille_of_budget=38`（产物第 144 行）。FC3 不触发 |
| 宽读（附带，原判据 2 没有这条臂） | 5 | 3 | 三档宽度各在 8192 × 64 一格过界；30 字节 1024 × 64 那一格新口径 8 017 920（373‰）、不过界。D5（快照 / 空间记账机制） 已定项 4 依据字面「宽读那条臂」在 1024 × 64 那一格没有支撑；FC5（宽读臂过界格数为 0）不触发 |

**几何敏感性**（产物第 295–300 行，上界臂判决格换条目宽与 t × d 邻格）：

```
E7RESULT name=geometry_sensitivity point=narrower_entry_width registered_label=S1 arm=upper_bound width=narrow entry_bytes=22 trees=1024 devices=64 new_bytes=12976128 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=main_entry_width registered_label=main arm=upper_bound width=middle entry_bytes=30 trees=1024 devices=64 new_bytes=17694720 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=current_implementation_entry_width registered_label=S2 arm=upper_bound width=current_implementation entry_bytes=34 trees=1024 devices=64 new_bytes=20054016 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=wider_entry_width registered_label=S3 arm=upper_bound width=wide entry_bytes=50 trees=1024 devices=64 new_bytes=29491200 new_over=1 same_new_verdict_as_main_point=0
E7RESULT name=geometry_sensitivity point=neighbor_fewer_devices registered_label=trees_times_devices_neighbor arm=upper_bound width=middle entry_bytes=30 trees=1024 devices=8 new_bytes=2211840 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=neighbor_more_trees registered_label=trees_times_devices_neighbor arm=upper_bound width=middle entry_bytes=30 trees=8192 devices=64 new_bytes=141557760 new_over=1 same_new_verdict_as_main_point=0
```

判决格的判定**依赖条目宽**（FC4 触发）：22、30、34 字节不过界（34 字节 20 054 016，占 933‰），50 字节过界，翻面落在 34 与 50 字节之间。判别力自证在单测 `moving_the_budget_across_the_judgement_cell_flips_its_verdict`：门槛取 17 694 720 时判决格不过界、取 17 694 719 时过界。

**阳性对照与作废条款**（产物第 281–287、301 行）：

```
E7RESULT name=positive_control_write_factor arm=upper_bound cells=60 old_equals_twice_new=60
E7RESULT name=positive_control_write_factor arm=per_statistic_narrow cells=60 old_equals_twice_new=60
E7RESULT name=positive_control_write_factor arm=per_statistic_wide cells=60 old_equals_twice_new=60
E7RESULT name=positive_control_drop_device arm=upper_bound trees=64 devices=8 width=middle with_device_new_bytes=138240 without_device_new_bytes=17280 ratio_x1000=8000 exactly_by_device_count=1 holds=1
E7RESULT name=positive_control_drop_device arm=per_statistic_narrow trees=64 devices=8 width=middle with_device_new_bytes=4890 without_device_new_bytes=4050 ratio_x1000=1207 exactly_by_device_count=0 holds=1
E7RESULT name=positive_control_drop_device arm=per_statistic_wide trees=64 devices=8 width=middle with_device_new_bytes=71040 without_device_new_bytes=17280 ratio_x1000=4111 exactly_by_device_count=0 holds=1
E7RESULT name=arm_ordering groups=60 upper_at_least_wide_at_least_narrow=60
E7RESULT name=verdict cells=180 positive_control_write_factor_holds=true positive_control_drop_device_holds=true arm_ordering_holds=true
```

| 作废条款 | 怎么核 | 结果 |
|---|---|---|
| V1 旧口径复出原产物 | 新装置照原格式重打的 90 行 `name=ckpt_budget` 与原产物 `e71-accounting-keys-2026-09-01.out`（已归档，`git show 3cff909b0^:research/results/e71-accounting-keys-2026-09-01.out` 取出，sha256 `9826e85e…`）同名行 `diff` | 两边各 90 行，`diff` 0 行不同 |
| V2 独立锚点 | 跑前登记作者的 `anchors.py` 输出（sha256 `a3d0bd98…`）里 48 行 `e71` 锚点逐行对 `name=publish_budget` | `e71_anchor_rows matched=48 differing=0 missing=0` |
| V3 阳性对照 | 产物第 281–286 行 | 每臂 60 格旧 = 2 × 新；上界臂去设备维恰好 ÷ 8，窄读、宽读严格下降且不到 8 倍 |
| V4 确定性 | 同一个二进制跑 5 轮 `cmp` | 5 轮逐字节相同（sha256 `885c238a…`）。确定性模型，轮数只说明没有隐藏状态 |

**单测与变异**：15 个单测全绿；变异表 `scripts/mutation/tables/e71_accounting_keys_single_generation.tsv` 10 条（登记 EV1–EV9，加登记修订补的 EV10「宽读臂丢掉设备维」），抓到 10、无效 0、没红 0；`mutate.sh` 收尾行原样（输出整份在 `research/results/e71-accounting-keys-single-generation-mutations-2026-10-20.log`）：

```
计数：内存撞顶 0 条（上限 2G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条、锚点失效 0 条、现判 7 条、复用 3 条、拒算 0 条
已还原，基线仍全绿
```

**登记修订**（产物之前写进跑前登记「十二、修订」）：宽读臂由「够判后不跑」改为进产物；补变异 EV10；阳性对照甲扩到四档宽度。**够判后未跑**：登记的附带量「撑爆点」（每档宽度下上界臂新口径过界的最小 t × d，对应 D5（快照 / 空间记账机制） 已定项 1，不在 Q3 里）与「今天实现的行集 3 + 6d 行 × 34 字节」（只给量级参照）。

**它答不了的**：纯算术，没有记账树、btree 与 write buffer；统计量是 2026-09-01 那张九项表，不是今天的十二项，也不是今天实现每次发布写的 3 + 6d 行；不建模 inode 号水位的台阶行（随根留着、一次发布至多多写一行）；34 字节只当几何敏感性点；数的是条目字节，不数脏节点（那是 E97（记账与分配记录的条目编码） 判据 10）。

### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 新装置两口径算术 | 每格按臂数一代条目数，乘写次数系数（旧 2、新 1）与条目宽，与门槛比；扫遍登记的 180 格（四档宽度 × 15 个 (t, d) × 三臂），不是抽样 | `scripts/experiment-apparatus/e71_accounting_keys_single_generation.rs:219 publish_write_bytes` | 2026-09-01 那张九项表的维度标记、三档宽度与 34 字节、门槛 21 474 836、两个写次数系数（都抄自跑前登记读法第 1–3 条） |
| 2 | 独立锚点脚本 | 只按跑前登记读法第 2、3 条的式子现算 48 格（四档宽度 × 四组 (t, d) × 三臂），不读装置与 `crates/` 的代码；48 格是挑的判决格与邻格，不是扫遍 | 无实现：仓里没有这份脚本，它在跑前登记作者的草稿目录 `~/.local/share/singlefs/work/m3-close-w8-c693-reruns/anchors.py`（sha256 `1cef0e2b…`），输出被新装置的单测 `independent_anchor_rows_match_on_every_width_arm_and_cell` 钉成 48 行常量 | 同第 1 行的九项表、宽度、门槛与系数（同一份登记读法） |
| 3 | 原装置旧口径 | 原装置 `checkpoint_bytes`（2 × 一代条目数 × 条目宽），2026-09-01 那一次产出的 90 格 `name=ckpt_budget` | `scripts/experiment-apparatus/e71_accounting_keys.rs:128 checkpoint_bytes` | 九项表、三档宽度、门槛；旧系数 2 |

三条路径共用同一张九项表与同一条门槛：表本身若不是要判的那张（今天是十二项），三条会一起错，这一节的数因此只写成「按 2026-09-01 那张九项表」。路径 1 对路径 2 的比对钉在单测里（48 行逐格 `assert_eq!`）；路径 1 对路径 3 只有 3 行钉在单测 `original_format_lines_match_the_original_product_text` 里，90 行全量比对是一条 `diff` 命令、源码里没有断言连起来，还欠这一条。

### 它答不了的那一半

1. 纯算术模型：没有记账树实现、没有 btree、没有 write buffer，文件操作 0 处。
2. **key 各段的字节宽度仓里一处都没定过**（D5（快照 / 空间记账机制） 已定项 3 只定了形态，没定宽度）
   ⇒ 三档宽度（22 / 30 / 50 字节含 value）是**假设**，不许把某一档的字节数当结论。
   ⚠️ **代段宽度已有一条硬下界（2026-09-02）**：D16（发布语义）已定项 6 定了代按发布走
   （本机 2785 发布/秒下 32 位 17.8 天回绕）⇒ 骑手条款「代段 ≥ 48 位」，
   **E71（记账 key 的条目数与宽度）的字节算术要按此重跑**——重跑前这三档数字只当形状参考。
   过界条件 `t × d ≳ 65536` 是按 30 字节那档给的，宽度变了要重算。
3. **树数 t 的现实取值范围没有依据**：D6（快照实现模型） 定了多可写头，但一个池里会有多少棵树
   一处也没写过。t=8192 那几档是扫出来的，不是负载。
4. **「哪几个统计量带树维」没定**（D5（快照 / 空间记账机制） 已定项 4 那张表只列了带不带设备维），
   所以窄读与宽读之间差 14.5 倍这件事
   是 E71（记账 key 的条目数与宽度） 量出来的**分歧幅度**，不是答案。

### 前置

无。纯算术。**原登记的前置「D5（快照 / 空间记账机制） 已定项 4（统计量有哪几个）」已于 2026-09-01 定案。**

### 口径与复跑

- 代码 `scripts/experiment-apparatus/e71_accounting_keys.rs`
  （`cargo run --release --bin e71-accounting-keys`），
  原始输出 `e71-accounting-keys-2026-09-01.out`（209 行，收尾行 `emitted=209`）。
- **N=5 轮逐字节一致**（同一个 md5）。**12 个单测，8 条变异逐条命中、0 个盲区**，
  变异表 `scripts/mutation/tables/e71_accounting_keys.tsv`
  （`bash scripts/mutation/mutate.sh e71-accounting-keys scripts/experiment-apparatus/e71_accounting_keys.rs scripts/mutation/tables/e71_accounting_keys.tsv`）。
- 第 1 次重跑（2026-10-20）：代码 `scripts/experiment-apparatus/e71_accounting_keys_single_generation.rs`（bin `e71-accounting-keys-single-generation`，不 `use` 原装置与 `crates/`），
  原始输出 `research/results/e71-accounting-keys-single-generation-2026-10-20.out`（302 行，收尾行 `emitted=302`），
  复跑登记是 `scripts/experiment-apparatus/replay.sh` 里 E71（记账 key 的条目数与宽度） 名下的第二行：驱动 `e71-accounting-keys-single-generation`、产物 `e71-accounting-keys-single-generation-2026-10-20.out`、判法 exact。
  变异：`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e71-accounting-keys-single-generation e71_accounting_keys_single_generation.rs ../mutation/tables/e71_accounting_keys_single_generation.tsv`（上限 `MUTATE_MEMORY_MAX=2G`）。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D5（快照 / 空间记账机制） 已定项 4 | 支撑 | 2026-10-21 不受影响：标题与索引行补上 2026-10-20 第 1 次重跑一项，正文、数、结论与产物不变；此前 2026-10-21 不受影响：正文只把复跑登记那一句改写成带简称的写法，数、结论与产物不变；2026-10-20 改了：依据句按第 1 次重跑的数改写——条目数比随 `树数 × 设备数` 涨，`1024 × 64` 宽读臂三种宽度新口径都不过界、上界臂只有 50 字节那档过，窄读臂 45 格 0 格过界（`research/prompts/m3-close-c693-r3-main-verification.md` 第二节 T3 一格） |
| D5（快照 / 空间记账机制） 已定项 1 | 支撑 | 2026-10-21 不受影响：标题与索引行补上 2026-10-20 第 1 次重跑一项，正文、数、结论与产物不变；此前 2026-10-21 不受影响：正文只把复跑登记那一句改写成带简称的写法，数、结论与产物不变；2026-10-20 不受影响：依据里「先撑爆预算的是 `树数 × 设备数` 这个乘积」在新口径下照旧成立——上界臂 5 格过界都在 t × d 大的一端，窄读臂 0 格；变的只是过界门槛点（30 字节那档 1024 × 64 不再过界） |
| D5（快照 / 空间记账机制） 已定项 2 | 不影响 | 2026-10-21 不受影响：标题与索引行补上 2026-10-20 第 1 次重跑一项，正文、数、结论与产物不变；此前 2026-10-21 不受影响：正文只把复跑登记那一句改写成带简称的写法，数、结论与产物不变；2026-10-20 不受影响：第 1 次重跑把这一项的现行定案（每条根只装一代、每次发布全写一遍）当输入口径用，不量它、不判它 |
| D16（发布语义） | 不影响 | 2026-10-21 不受影响：标题与索引行补上 2026-10-20 第 1 次重跑一项，正文、数、结论与产物不变；此前 2026-10-21 不受影响：正文只把复跑登记那一句改写成带简称的写法，数、结论与产物不变；2026-10-20 不受影响：门槛仍取已定项 5 的 `T_dirty` = 2 GiB 的 1%，第 1 次重跑没动它 |
| D6（快照实现模型） | 不影响 | 2026-10-21 不受影响：标题与索引行补上 2026-10-20 第 1 次重跑一项，正文、数、结论与产物不变；此前 2026-10-21 不受影响：正文只把复跑登记那一句改写成带简称的写法，数、结论与产物不变；2026-10-20 不受影响：树数 t 仍是扫出来的取值，第 1 次重跑没加依据 |
| D22（单元原子性怎么合成） | 不影响 | 2026-10-21 不受影响：标题与索引行补上 2026-10-20 第 1 次重跑一项，正文、数、结论与产物不变；此前 2026-10-21 不受影响：正文只把复跑登记那一句改写成带简称的写法，数、结论与产物不变；2026-10-20 不受影响：K 的取值只进总条目数，第 1 次重跑的新口径 K ≡ 1、不用它 |

## 历史版本

E71（记账 key 的条目数与宽度）的历史条目集中在 [experiments-history.md](../experiments-history.md)。
