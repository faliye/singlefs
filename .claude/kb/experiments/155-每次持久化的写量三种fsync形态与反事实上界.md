## E155 每次持久化的写量：三种 fsync 形态与反事实上界 —— 部分已跑（2026-09-17，确定性模型，42 单测 / 15 条变异全抓；覆盖跑前登记第 5.7 节全部四段：第一、二段——K1–K9 模型核心、5.3 不动点、5.4 甲 / wal_full / 乙三臂、5.5 阳性对照、Q1.1–Q1.8；第三段——Q2.1–Q2.6、Q3.1、Q4.1 全网格；第四段——Q5.1–Q5.4、P 门槛搜索、φ=0.5 重搜。Q1.4 全量标签矩阵、8.2 全量反向几何扫描（只做了代表格）、K9′ 建模仍未做，见「它答不了的」）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 K7 K8 K9 A1 A5 A8 B1 B2 B3 B4 B5 B6 B7 B9 M1 M2 M3 M4 M5 M6 M7 M9 M12 M15 M19 Q1 Q1.1 Q1.4 Q1.8 Q2 Q2.1 Q2.5 Q3 Q3.1 Q4 Q4.1 Q5 Q6 -->
本文这些编号都是 `research/prompts/e155-preregistration.md` 跑前登记自己的锚点 / 位置策略 / 变异编号，登记位在那份文件里，不是 kb 的 D / E / C / I 编号。

**备料**：给里程碑「第二个事务」增补 1 第 4 件的候选改法配代价数——要重新验证的是 D23（journal 的角色与格式） 已定项 1（每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录）、D16（发布语义） 已定项 9（空发布也重写记账行）、D5（快照 / 空间记账机制） 已定项 2（每行每发布重写）、D19（块指针的结构与宽度预算） 已定项 8（全部指针进映射）、D3（空间分配） 已定项 7（每个新落点每盘一条记录）、D4（校验和位置） 已定项 5（单元恒 32768 含头）几条；候选改法定案之前，这些数不推翻任何一条决策。

**问题**：D23（journal 的角色与格式） 已定项 1 取甲（每次 fsync 是一次全量发布）之后，甲、wal_full（发布 B 那样每次 fsync 写脏叶 + 全部祖先、不发根）、乙（E16（journal 的角色：WAL vs 意图日志） 的 `wal_leaf`，每次 fsync 只写脏叶）三个候选，在树高 1–4、族 F1 / F8A、落点 seq / rand、checkpoint 间隔 N、三种位置策略（K8：旧版落点近 / 远 / 均衡）上，每次持久化的写字节 / 写请求 / 屏障数之比怎么随规模变化；以及 G23.1（祖先链占比达标）的写字节替身（祖先链 + 固定点 + 根槽占甲每次持久化写字节的份额）轨迹。跑前登记 `research/prompts/e155-preregistration.md`；岔路单第 1 行 `research/prompts/m2-s1-forks.md`。

**做了什么（本轮实际覆盖）**：

1. 纯算术计数模型：容量公式（K1）与树高公式（K2）用同一个 `node_capacity(key_bytes, entry_bytes)` / `tree_layers` 覆盖 extent、inode、分配记录、记账、映射、树表六棵树；5.3 的连续组 / 散组 / 组合并三步公式（`continuous_groups_touch` / `scattered_group_touch` / `combine_touches`）逐字对应登记原文；K8 三种位置策略（Lnear 恒近、Lfar 多节点类恒远、Lbalanced 按 `min(1, insert/cov)`（seq）或按上一轮迭代的 `U_l/n_l`（rand）取远近比例）在一个通用函数 `class_dirty_nodes_this_layer` 里实现，映射树六个类与分配记录树两块盘复用同一套 `multiclass_tree_dirty_nodes_per_layer`。5.3「不动点」按登记的迭代顺序实现：分配记录 / 映射两棵树的条目数、层数、脏节点数与 W 反复重算到变化 < 1e-9 或撞 1000 次上限（全部已跑格都在 1000 次以内收敛，`fixed_point_converges_well_below_the_iteration_ceiling` 单测覆盖 P 到 1 亿）。
2. 甲的一次 fsync（`solve_jia_publish`）与 WAL 两臂的一次 fsync（`solve_write_ahead_log_fsync`，不经不动点，wal_full 写全部祖先记两棵树的根、乙只写叶记脏叶数）、WAL 两臂的一次 checkpoint（`solve_write_ahead_log_checkpoint`，复用 5.3 不动点核心，extent / inode 的组换成「N 次 fsync 并集」——seq 连续组 k=Nd、rand 散组，乙的 checkpoint 额外补第 1 层起的祖先追赶，wal_full 不用补）。
3. 第七节 B1（容量）、B2（甲 P=1、F1、seq 逐字节对 `crates` 发布 B）、B3（G23.1（祖先链占比达标）份额字面 47.70% 与含超级块 50.07%）、B4（wal_full）、B5（乙）、B6（三臂在锚点上的差 139776 / 32768）、B7（P=145）、B9（环容量）与 A1、A5、A8，以及第七节 7.1 第 7 条断言（甲的持久顺序每次 fsync 恰好两道屏障、每道每盘一次共 4 次刷写、根槽一次 FUA）全部钉进单测，逐字节 / 逐条与登记第七节的手算值相符；5.5 阳性对照（甲 − wal_full = 139776、wal_full − 乙 = 32768、根槽次数 16 对 1、P=145 一格乙 fsync 行不含两个根）全部通过。V2（乙 ≤ wal_full ≤ 甲 的包含关系）在 P∈{1,145,10⁴,10⁶}×族×落点×位置策略的网格上逐格核过，没有一格违反。
4. 7.3 要求的 5 个手算格全部补齐并钉进单测（算式见各测试的文档注释）：分配记录树第 2 层刚出现在 P=205（F1、seq、主几何）、映射树第 2 层刚出现在 P=285；rand 下 Lfar 与 Lbalanced 在 P=1000（F1）给出不同的字节数（P=145 这一格两种策略仍相同，因为除 extent 外别的树都还是单节点，位置策略无从体现远近之别）；点名项超过 67 用主网格自带的 P=10⁴、F8A、rand、Lfar 一格（`named_items=220`，无需在网格外另造）；Q3.1 在 P=1 的反事实上界逐个单元手算，七个非数据单元取整到 512 字节后单盘省 125440、两盘省 250880，`U_3 = 250880/344576 ≈ 72.81%`。**这 5 个手算格的单测在跑产物之前就已经写好并通过；把它们转录进登记第十二节「修订」这一步，操作顺序上排在了产物之后——内容与产物无关（纯手算），但流程上没有照「先登记、后产物」的顺序做，如实报给主 agent，登记文件本身这一轮没有再改。**
5. 变异表 `research/mutations/e155_fsync_write_volume.tsv` 15 条，覆盖 D=1、甲不写映射树 / 不写超级块槽、根槽宽恒 4096、inode 树允许 1 层、wal_full 的 fsync 多写一个根槽宽度、乙在 fsync 写全部祖先、摊销分母取 N+1、散组不饱和、G23.1（祖先链占比达标）分子加超级块槽、环安全系数改 2、一条记录容量改成 71、Q3.1 不计头 H、Q4.1 不计单元头 134、Q5.3 用 32768 当一层的宽，`bash scripts/mutate.sh e155-fsync-write-volume e7-index-bench/src/bin/e155_fsync_write_volume.rs mutations/e155_fsync_write_volume.tsv` 报「基线：全绿」→ 15 条全部「红」→「已还原，基线仍全绿」，0 条无效、0 条未抓到。M8（不动点只迭代一次）、M11（连续组丢空隙项）、M13（Lbalanced 的 q 不迭代）、M16（反事实不重算不动点）——这四条依赖 7.3 手算格但我没找到能真正命中而不误配对的钉死断言，未写；M14（K9 换成 K9′，K9′ 未建模，无代码可换）、M20（施加记录数不计 checkpoint 那条，只在测试里现算、不是共享函数）、M21（Lfar 下数据单元新落点按散组打散槽号序，本模型的分配记录树没有按对象类型分流的这一层）也未写，见「它答不了的」。
6. 停机条款 S1：`crates/singlefs-core/src/write_accounting.rs` 与 `crates/singlefs-harness/tests/second_transaction_supplement_one_write_accounting.rs` 现跑，`first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table` 与 `writable_remount_row_publish_and_each_warm_up_publish_add_up_to_the_recorded_writes` 两个测试全绿，逐种（数据、extent 根、inode 叶容器、inode 根、分配记录、记账、映射、树表、记录、根槽、超级块）与 P=1、F1、seq 一格甲的锚点逐项相符，不停机。S2：开工时与产物前两次取 `transaction.rs` 等五个文件的 sha256，`transaction.rs` 在工作区被改动（暖机记录号从 txg 改接续 jsn 计数器、出生序号作用域文档化、记账树也加了装不下报错），未涉及发布 B 写哪些单元、宽度或次序；实测重跑 S1 两个测试与 B2 甲锚点全部仍然相符，判定「不停机、继续」。
7. 产物第一、二段留存为 `research/results/e155-fsync-write-volume-2026-09-17-stage2.out`（305 行，历史快照，不再是 `replay.sh` 的目标）；第三、四段做完之后同一个二进制重跑，产物扩写为 `research/results/e155-fsync-write-volume-2026-09-17-stage4.out`（1334 行，`E7RESULT name=done emitted=1334` 与实收行数一致，`grep -oE "name=[a-z0-9_]+" ... | sort | uniq -c` 核过每类行数：`row1_grid` 288、`row2_fixed_point_upper_bounds` 72、`row2_tree_table_count_axis` 5、`row3_whole_unit_rewrite_upper_bound` 144、`row4_data_unit_rounding_upper_bound` 720、`row5_growth` 54、`row5_height_threshold` 25、`row5_record_count_trend` 9，其余（`config`/`scope`/`b1_capacities`/`positive_control`/`q1_8_record_count_alternate`/`geometry_sensitivity_sample`/`done`）与第二段相同）。代码 `research/e7-index-bench/src/bin/e155_fsync_write_volume.rs`（`cargo run --release --bin e155-fsync-write-volume`），原始输出 `research/results/e155-fsync-write-volume-2026-09-17-stage4.out`。复跑命令：

```
cd research && bash scripts/replay.sh E155
```

报「字节一致」（`replay.sh` 里 E155（每次持久化的写量：三种 fsync 形态与反事实上界） 那一行已指到 stage4）。

8. **第三段（Q2/Q3/Q4 反事实上界）**：`SuppressedFixedPoints` 结构体让 `solve_fixed_point_core` 可以按需把记账 / 分配记录 / 映射 / 树表四样固定点中任意一样的这次持久化脏节点数强制为 0（树仍在、条目数与层数照常算，只是不重写、不进别的树的映射类、不点名），`solve_jia_publish_with_suppressed_fixed_points` 在此基础上重算 `B(甲|X不写)`；树表条目数也从写死的常量改成参数，`tree_layers` 复用同一套公式给树表自己算高度（Q2.5）。Q3.1 把「每层脏节点数 → 这一层的条件数 c → 取整字节」串成 `q3_1_tree_savings`，六棵树各自的 H / w / 全宽度按第七节 K1/K3 的常量喂进去，在 P=1 一格与 7.3 手算格逐字节相同（`q3_1_general_implementation_matches_the_hand_calculated_cell_at_file_count_1`）。Q4.1 是纯闭式公式，用 B8 的八个锚点值钉住。
9. **第四段（Q5 门槛与三种 ρ 读法）**：`bisect_first_threshold` / `height_growth_thresholds` 在树高单调不减的假设下二分查找每棵会长高的树（extent、inode、分配记录、映射）的门槛 P*，extent 门槛在单测里钉死等于 B7 锚点的 145、分配记录门槛钉死等于 7.3 手算格的 205。`rho_readings` 按登记 Q5.2a/b/c 三种读法分别算：a 用六棵树各自当前高度对应的真实单元宽求「一条路径」、b/c 用最高的那棵树乘 6 乘统一单元宽（16384 或 32768）。
10. 命名纪律：`naming-lint.sh` 对 e155 全部通过（0 处违规）；门禁 27 号（格式常量同步）一度因为我给「数据单元头 + 尾开销 134」取名 `DATA_UNIT_HEADER_BYTES`、恰好撞上 `.claude/kb/decisions/18-块里携带什么信息.md:592` 已登记的 `DATA_UNIT_HEADER_BYTES = 105`（那个是单纯的头，105 + 29 才是我这个 134）而判红，改名 `DATA_UNIT_HEADER_AND_TRAILER_BYTES` 后绿。门禁 27/33/34/40/52/65/80/85/86/88 十个阶段（experiment-runner 在 `stage-owners.tsv` 里登记的那些）全部跑过，本轮改动涉及的都是绿的。

**产物读出来的几个数**（判断——是否满足 G23.1（祖先链占比达标）、Q1.4 的标签怎么归类——留给后续推论，这里只抄产物）：
- 甲 / 乙（fsync 行字节）在 F1、seq、Lbalanced、N=16 上随 P 从 1/100 的 2.00×（P≤100 两者都是单节点、比值不变）涨到 P=10⁴ 的 4.31×、P=10⁶ 的 7.38×、P=10⁸ 的 10.65×，逐步 δ 都 ≥ +5%，按规则 R 标「随轴扩大」，未见收敛迹象（`grep "family=F1 placement=seq policy=Lbalanced n=16 " research/results/e155-fsync-write-volume-2026-09-17-stage2.out`）。
- G23.1（祖先链占比达标）份额（Q1.6，判据已撤，只报份额）同一批格上从 47.70%（P≤100）涨到 P=10⁴ 的 75.67%、P=10⁶ 的 85.81%、P=10⁸ 的 90.16%。
- 极端格 P=10⁸、F8A、rand、Lfar：甲每次 fsync 3 463 319 次写调用、56 117 424 480 字节，G23.1（祖先链占比达标）份额 99.63%；这一格下 wal_full / 乙的 checkpoint 分别要 61 050 302 956 / 61 856 107 548 字节、在飞记录峰值 31801 / 32166，远超默认 768 MiB 环的 65536 上限（`write_ahead_log_full_default_ring_ok=true` 是因为该行用的是 N=4096 时公式算出的峰值，具体见产物同一行的 `_in_flight_peak` 字段，貼行时把这四个字段一起读）。
- Q1.8（C310（事务切分纪律与记录数口径打架） 口径）：`main_record_count` 与 `alternate_record_count` 在 P=1、10⁴、10⁸ × F1/F8A 六格上全部相同（`differs=false`），本轮跑到的规模上两种记录数口径不敏感；更大规模、rand+Lfar 会不会分歧未测。
- 8.2 反向几何敏感性只在代表格 P=10⁵、F1、seq、Lbalanced、N=16 上跑（不是全量 1800 格扫描）：φ=0.5、g=0、K3′ 三个反向点在这一格上甲 / wal_full 的字节比值都没变（`_ratio_changes=false`），pbs=4096 让甲的比值变了（`jia_ratio_changes=true`，根槽从 512 变 4096、超级块槽照旧 4096×2 不受影响）；K9′ 未建模，产物里那一行明写「未建」。
- **第 2 行（Q2.1–Q2.4，F1、seq、Lbalanced）**：记账与树表的上界随 P 变大反而**缩小**（P=1 都是 9.51%，P=10⁸ 分别降到 7.19% / 1.80%，恒在 10% 门槛之下——这两样是固定成本，池越大占比越薄）；分配记录与映射的上界随 P 变大**长大过 10% 门槛**（P=1 是 9.51%，P=10⁴ 涨到 26.9% / 35.6%，P=10⁸ 到 32.8% / 52.4%——这两样的字节随池规模长，长得比 `B(甲)` 整体快）（`grep "name=row2_fixed_point_upper_bounds" .../stage4.out`）。
- **第 2 行 Q2.5（树表按棵数 T，P=10⁴、F1、seq）**：T=7 与 81 时上界相同（4.45%，树表仍是 1 层）；T=82 起长到 2 层，上界跳到 8.66%；T=6562 起长到 3 层，上界过 10% 门槛（12.38%）——门槛只在树表自己长高一层时才跳，不随 T 连续增长。
- **第 3 行（Q3.1，F1、seq、Lbalanced、pbs=512）**：上界全程在 72.8%–91.9% 之间、随 P 单调上升，从没有低于 10% 门槛——「改一条记录也整单元重写」在这个模型的任何规模下都值得设计具体候选。
- **第 4 行（Q4.1，s=4100、pbs=512、F1、seq、Lbalanced）**：上界不是单调的——P≤100 是 16.3%（过门槛），P=145 是 14.9%，P=10⁴ 起掉到 10% 以下（7.6%、4.4%、3.1%）：这一样的绝对省钱额不随 P 变（每次只碰 d=1 个数据单元），而 `B(甲)` 随 P 变大，所以份额被稀释。
- **第 5 行（用户直接问的那一题：写量随对象数是不是指数级恶化）**：**在主几何 Lbalanced 下，不管 seq 还是 rand，答案是否定的**——ρ_a（六棵树各自按当前高度算一条路径当分母）在 P 从 1 涨到 10⁸ 的过程中只在 1.0 与 4.0 之间摆动，从没有失控增长（例如 rand+Lbalanced 那一串：1 → 1.52 → 3.74 → 1.65 → 2.71 → 1.83 → 2.36，见 `grep "name=row5_growth" ... | grep "placement=rand policy=Lbalanced"`）。**但在最坏的位置策略 Lfar（假设旧版永远落在离插入点最远的地方）下，rand 落点时 ρ_a 从 P=1000 的 1.52 一路长到 P=10⁸ 的 11270**，且 `a_tree` 本身在 P=10⁵ 之后几乎与 P 成正比（P 每涨 10 倍，`a_tree` 也涨 8.7–10 倍）——这是「随规模几乎线性变差」，不是数学意义上的指数（`c^P`），但确实是这个模型里能找到的最坏走向，且只在 Lfar 这个人为构造的最坏假设下出现，不在真实历史模拟里（这份计数模型本来就不模拟真实历史，见跑前登记第五节 5.3）。extent / inode 长高一层的 e 值恰好是 1.0（钉进单测），是「一层一条路径」；分配记录 / 映射长高一层的 e 值恰好是 2.0——长一层要多写两条路径的份额，不是一条，这是它们的上界比 extent/inode 长得快的一个直接原因。

**它答不了的**：
- 8.2 的几何敏感性只跑了 1 个代表格 × 4 个反向点（φ=0.5、g=0、K3′、pbs=4096），不是登记要求的「每道判定在它自己的判定格上」全量扫描；第 5 行的 φ=0.5 门槛搜索做了（见「做了什么」第 9 条），但 Q2/Q3/Q4 的上界判定没有按 8.2 逐个换到反向点重判；K9′（间隔内被换掉的中间版也写一条已释放的分配记录）完全没有实现，凡是判定依赖它的都答不了。
- 映射树 / 分配记录树的位置策略实现按「每个类 / 设备区独立算 ceil(区域大小/层覆盖) 当作它自己的节点数，总节点数另用合并后的条目数算」，两者在类边界上有 O(类数) 量级的近似误差；M21（Lfar 下数据单元新落点的分配记录按散组打散槽号序）描述的是分配记录树对「新落点是数据单元」这一类的专门处理，本模型的分配记录树只有「每块盘一个区间」的通用插删，不区分落点对象是数据单元还是节点，所以这条变异没有可以对应的生产代码，未写。
- Q1.4 的标签只在若干代表性序列上手动核对（「产物读出来的几个数」那几条），没有对登记要求的每一种（三个比值 × fsync/摊销两行 × 三种位置策略 × N 轴/族轴/落点轴）都单独算一遍标签矩阵。
- M8（不动点只迭代一次）、M11（连续组公式丢掉组间空隙项）、M13（Lbalanced 的 q 不迭代）、M16（反事实不重算不动点）、M20（施加记录数不计 checkpoint 那条）——这五条变异都还没写，见「做了什么」第 5 条的理由；这五条对应的行为目前**零变异覆盖**，不是「测过发现是等价变异」。
- 第 5 行的门槛搜索只对 extent / inode / 分配记录 / 映射四棵会长高的树、只在 seq、只到 P=1 亿搜；rand 落点、accounting/树表（本几何下恒 1 层，不会触发）、更大的 P 没有搜。ρ 的三种读法（Q5.2a/b/c）与 e 值（Q5.3）只在 F1、Lbalanced/Lfar 两种位置策略上重点核对，Lnear 那一括号没有单独写进正文（产物里有，只是没抄进来）。
- 7.3 手算格的单测在产物之前就写好并通过，但转录进登记第十二节「修订」这一步排在了产物之后，见「做了什么」第 4 条；这一条已经在登记第十二节补记了真实时点与理由（不影响那五格的结论），仍留痕在这里以防遗漏。

## 历史版本

### 2026-09-17
- 首次跑，第一、二段（产物 stage2，305 行）。
- 续做，第三、四段（产物扩写为 stage4，1334 行；`replay.sh` 改指向 stage4）。
