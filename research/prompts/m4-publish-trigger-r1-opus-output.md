# m4-publish-trigger-r1 攻方腿（Opus）报告（2026-10-26）

立场：正文第四节「云端攻方」那一行。第一轮角度：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。方案按冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/` 的 `crates/` 谈；引代码一律写冻结副本里的绝对路径与它自己的行号，引 kb 写仓里 kb 文件自己的行号。没读禁读清单里的正推腿产出。

## 复跑

```
bash scripts/hosts/run-with-memory-cap.sh 3G bash research/prompts/m4-publish-trigger-r1-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的空目录>
```

它把冻结副本拷进 `--work`、把原型 crate 加进拷贝那份工作区、`cargo build --offline --release` 原型（只依赖 RefFS 与 singlefs-core，不依赖 checker 档包），跑两个二进制与一份 Python 计数模型，三份输出与 `out/` 逐字节比，最后删掉拷贝。交回前在新 shell 里整份跑过一次（`env -i` 只留 HOME、PATH、XDG_RUNTIME_DIR、DBUS_SESSION_BUS_ADDRESS），末尾原样：

```
name=rerun_compared file=reference_filesystem_publish_points.out equal=yes
name=rerun_compared file=test_only_switch_in_a_release_binary.out equal=yes
name=rerun_compared file=idle_trigger_model.out equal=yes
name=rerun_summary files=3 differing=0
```

退出码 0（内存包装排队 69 秒后起跑）。模板自带的 `--selftest` 判的是模板那一段 `rerun_steps`，换成本腿的步骤之后它红（拷进去的假模型目录里没有 `proto/`），这是预期的，不代表复跑坏。

模型目录 `research/prompts/m4-publish-trigger-r1-opus-model/` 每个文件的 sha256：

```
415a763ddcbec4ff71b1a9f032d44236187bd5178c486d5f7a79ad9259eda46b  ./models/idle_trigger_model.py
10ee9a55d63e197aa76a14359c9f869d67dd5c5961dcc42abb12ee0aeab40ba3  ./out/idle_trigger_model.out
c91625f1e1f13e93c30f639772509b460ca117597a5f6c5f5d670a5803796e33  ./out/reference_filesystem_publish_points.out
9601eacb1bd27b00acc0b6dd257dd6725afeeeb5421b3686d3197618a9648699  ./out/test_only_switch_in_a_release_binary.out
459ead29ec5af33db4b429ef68aa52399b7e02016ca5c3371fdad5111fbacd4e  ./proto/Cargo.toml
b8bcc9f79e8a319c523a3029a523f35cddd8a4135df0d2025e03438c5d4b7462  ./proto/src/bin/reference_filesystem_publish_points.rs
a25bbfae10a2e9c688121898bbc67468710e779f5b7b91262d78380b27b806e0  ./proto/src/bin/test_only_switch_in_a_release_binary.rs
be55a2148a79b33419e300d9b507552ebc0884760d976d5a5277980c5251969f  ./tools/rerun.sh
```

数都是副本与模型上的数，不是入库装置上的数。这一轮没有崩溃状态枚举：攻的是触发判据、允许集合与开关的形状，没跑 `enumerate_crash_states`，所以崩溃状态的 10⁶ / 10⁷ 两道量都是 0。

## 各格判定一览

| 格 | 判定 | 打中的形状（一句） | 证据 |
|---|---|---|---|
| P1 | 打中 | ① 只在下一次请求进门时判，空闲时丢窗口等于空闲时长、无界；② 调用方定时调的入口也只把界推给调用方：窗口 = T_time + tick 周期 + 调用方线程上排在前面的那次长调用，钟回拨还要再加回拨量，D16 已定项 5 的丢窗口式子里没有这几项；③ 已定项 2 的式子照字面在脏字节为 0 时也触发，空闲时每 T_time 一次空发布，S = 8 时 120 秒把回退候选集挤成 1 个状态 | 量过（计数模型，72 行 + 135 行） |
| P2 | 打中（推的） | 「已收下、还没发布的数据单元字节」不数只改元数据的改动，已定项 2 依据里交给脏量那一支管的「journal 占用撑过环几何」那一形它管不到；它和 min(字段, 环长 ÷ F) 那一夹量纲不合（夹的是 journal 字节）；批内同一单元覆盖写 K 次，按已定项 15 占 K 个落点，按折叠后的字节只数 1 个 | 推的（按代码与常量算） |
| P3 | 打中 | ① D8 已定项 16 第 2 条要求删除类每一步各占一个事务号，并进别的改动就违这一句；② 同一项第 6 条的队列从现行那一版读、写入口的「删除中」判也读上一版，攒批里没发布的 unlink 两处都看不见，同批里接着写同一个 inode 就把它写活；③ 整理的 G8 第三样保证与 `B_gc` 重置都锚在「一次调用发成」上；④ 一批的条数上限只在发布时判，超了就是返回成功之后才拒 | 推的（逐句指到条款与代码行） |
| P4 | 打中 | 34 条崩溃枚举流里 33 条直接调事务层发布函数、只有 1 条经会话；攒批放在会话层的话，这 33 条一条也碰不到攒批，「现有崩溃流仍绿」不是证据；RefFS 翻译层把每一步都写成「改动 + 显式发布点」 | 量过（grep 计数） |
| P5 | 打中 | RefFS 今天的允许集合在合法的攒批历史上判红：照今天的翻译交 14 格红 10 格，只交内容改动也红 10 格；放宽成 [lo, hi] 里任意 S_j 之后这 14 格全绿，但一次请求被触发切开那 4 格仍红 2 格，要再加按单元的键序前缀才 0 红；自动发布写了之后报错、攒批照收下一次改动，RefFS 判不了 | 量过（真 RefFS 代码） |
| P6 | 打中 | 今天「只供测试的开关」的做法（pub 成员 + pub 入口，没有 cfg 门）在发布构建的普通 bin 里够得着：会话持有者一句话就能关掉空间准入。阈值开关照抄这个做法，就是一条产品可达的第二条配置通道；C733 定的「两端值挂载时拒绝」那条分支在里程碑四没有入口 | 量过（release 编译跑通）+ 推的 |
| P7 | 部分打中 | 串行下 C425 原形（上一次没发完下一次就触发）确实走不到；但照字面式子、T_time 小于发布耗时时空闲一小时推 360–7200 次空发布；发布写失败冻结之后，今天的代码拒后面所有发布，攒批之后「收下」不再等于「发布」，没有收下那一判就会在冻结期间一直攒，窗口由调用方什么时候重发定，串行下也会出现 | 量过（计数模型）+ 推的 |

打中的每一格先答四句，写在各节末尾。下面「被攻过零轮」的改法都只在我的模型上量过。

## P1　时间那一支由谁判、空闲时怎么触发

### P1-甲　只在请求进门时判：空闲丢窗口无界（打中）

历史：起点任意一版带文件；t = 0 收下一次覆盖写（攒批，不发布）；之后没有请求；t = idle 时崩溃。

- 许可「收下不发布」的是已定项 10：「**定案**：攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次。」（`.claude/kb/decisions/16-发布语义.md:267`）
- 判据：「**定案**：`触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty)`」（`.claude/kb/decisions/16-发布语义.md:106`）——式子没说 now 在什么时候被判；只在进门时判，空闲时这个式子一次都不求值。
- 打中的是已定项 2 自己的依据：「只按脏数据量，空闲系统上最后一批写可以无限期不落、崩溃丢窗口无界；只按时间，突发写会把一个 checkpoint 的 journal 占用撑过环几何」（`.claude/kb/decisions/16-发布语义.md:112`）。进门才判的时间那一支在空闲时与「只按脏数据量」同形。

模型 `models/idle_trigger_model.py` 第一节，扫空闲时长 {10, 60, 3600} 秒、tick 周期 {0.1, 1, 5} 秒、长调用 {0, 30} 秒、钟回拨 {0, 120} 秒，共 72 行（`out/idle_trigger_model.out`），原样摘：

```
name=loss_window arm=entry idle=3600 tick_period=1 blocking_call=0 backwards=0 published_at=None
name=loss_window_summary arm=entry worst_window_seconds=3600 formula_t_time=5
```

`None` 是空闲期间一次都没发布，窗口等于空闲时长；空闲时长是扫描上沿，窗口随它线性涨，没有上界。

### P1-乙　调用方定时调「到点了没」：窗口里多出调用方那几项（打中）

今天的先例是整理的定时兜底：「一次次调，核心层不起线程、不读钟（钟与前台的 fsync 由 `caller` 答）」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:2146`），钟往回走的处置是「调用方的钟往回走时按没过间隔算」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/compaction_plan.rs:731`），写法「Some(last_check) => now.0.saturating_sub(last_check.0) >= interval,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/compaction_plan.rs:745`）。发布的定时入口照这个形状做，它和整理后台入口、用户改动入口一样拿 `&mut self`，调用方线程上排在前面的那一次调用（整理一趟至多 20 次发布、一次 unlink 走完全部批）跑完之前轮不到它。

历史（只固定前缀「收下一次改动」，后面三样放开扫）：t = 0 收下改动；调用方线程上 t ∈ [0, 30) 正跑着一趟整理；调用方每 1 秒调一次定时入口；另一格里调用方交进来的钟在收下之后回拨了 120 秒。原样摘：

```
name=loss_window arm=tick idle=3600 tick_period=1 blocking_call=30 backwards=0 published_at=30.0
name=loss_window arm=tick_monotonic_backwards idle=3600 tick_period=1 blocking_call=0 backwards=120 published_at=125.0
name=loss_window_summary arm=tick worst_window_seconds=30 formula_t_time=5
name=loss_window_summary arm=tick_monotonic_backwards worst_window_seconds=125 formula_t_time=5
```

D16 已定项 5 写的界是「**由此崩溃最多丢的是 T_time + N_switch × (T_retry + T_redo)，不是恰好 5 秒**」（`.claude/kb/decisions/16-发布语义.md:164`）。调用方给钟、调用方驱动之后，这个式子少了三项：tick 周期、调用方线程上排在前面那一次调用的时长、钟回拨量。三项都由调用方定，核心层给不出界；契约要写成「核心层保证：每次定时入口被调时判一次，到点就在这次调用里发完才返回；丢窗口 ≤ T_time + 调用方两次调定时入口之间的最长间隔」，后一项归调用方。tick 周期、长调用都在扫描上沿，量到的是「随它们涨」，不是它们的最坏值。

与第 39 批的关系：后台入口与定时入口共用 `&mut self`、共用调用方的同一条线程，整理一趟的发布次数（定时兜底一个周期至多 `COMPACTION_PUBLISHES_PER_TIMER_FALLBACK_PERIOD_AT_MOST_BY_DEFAULT` = 20 次）直接进发布的丢窗口。

### P1-丙　照字面式子，空闲时每 T_time 一次空发布，回退候选集挤成 1 个状态（打中）

式子 `(now − 上次 checkpoint 的时刻 ≥ T_time) ∨ …` 在脏字节为 0 时左边照样成立。已定项 1 数状态时「只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算」，且「**根环容量是这句的边界**」（`.claude/kb/decisions/16-发布语义.md:31`）：空发布不算状态，却占根环槽。整理那一边已经为同一件事设了预算：「一次整理运行的发布预算（判决 G8 `B_gc`，正推 O11：整理的发布不改用户可见状态，连发会把回退候选集挤成 1 个状态）：」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/compaction_plan.rs:1078`）。时间那一支照字面就是一条没有预算的同形发布源。

模型第二节：根环 3S 槽，空闲前改过 w 次状态（w 由用户定，扫 {1, 4, 6}），S ∈ {4, 8, 16}，T_time ∈ {0.2, 1, 5}，空闲 {10, 60, 120, 600, 3600} 秒，135 行。原样摘：

```
name=ring_churn S=8 T_time=5 idle=120 writes_before_idle=4 empty_publishes_literal=24 candidates_literal=1 candidates_dirty_guarded=4
name=ring_churn S=16 T_time=5 idle=600 writes_before_idle=6 empty_publishes_literal=120 candidates_literal=1 candidates_dirty_guarded=4
name=ring_churn_summary rows=135 rows_collapsed_to_one_state_only_under_the_literal_formula=72 idle_seconds_to_collapse_at_T_time_5={4: 60.0, 8: 120.0, 16: 240.0}
```

135 行里 72 行只在照字面那一臂挤到 1 个状态；加「脏字节为 0 不触发」那一臂这 72 行都留着空闲前的状态数（上限 4）。w = 1 的行两臂都是 1，是空闲前本来就只有一个状态，不分辨臂。

### P1 的四句

| 问 | P1-甲 | P1-乙 | P1-丙 |
|---|---|---|---|
| 分不分辨臂 | 分：进门判这一臂中，定时入口那一臂不中 | 分：只中「界写成 T_time + …」的写法，不中把调用方那几项写进契约的写法 | 分：照字面那一臂 72 行中，加脏字节 > 0 的那一臂 0 行中 |
| 系统当时看不看得到判别它的东西 | 看不到：空闲时没有调用进门，核心层不读钟 | 看不到：tick 周期与回拨在调用方手里 | 看得到：脏字节为 0 是会话自己知道的 |
| 满足的是判据字面的哪一句 | 已定项 2 依据第一个分句「只按脏数据量……无界」的同形 | 已定项 5「崩溃最多丢的是 T_time + …」那一句 | 已定项 2 定案式子的左边分句，碰已定项 1 的「根环容量是这句的边界」 |
| 正文给的改法在这几格上还中不中 | 正文 P1 提的「调用方定时调入口」不中 P1-甲，但中 P1-乙 | 时间由调用方给本身就是 P1-乙的来源，改不掉，只能改契约 | 正文没给改法；加「脏字节 > 0」这一合取（只在我的模型上量过、被攻过零轮） |

测试里怎么让两支分别先到：可控时钟时间先到，用只供测试的脏量阈值压到一单元脏量先到；两支要分开断言「触发的是哪一支」，按 `.claude/rules/fs-design.md` 五条硬要求第 4 条报出来（推的，没写用例）。

## P2　「脏字节」按什么算（C348）

被攻的读法：没有前端时数「已收下、还没发布的数据单元字节」（正文 P2 提的第三种读法）。

### P2-甲　只数数据单元字节，脏量那一支管不到它该管的那一形（打中，推的）

已定项 2 把两支分工写死：时间那一支管空闲，脏量那一支管「突发写会把一个 checkpoint 的 journal 占用撑过环几何」（`.claude/kb/decisions/16-发布语义.md:112`）。今天一次发布的 journal 条数在发布时才判：「调用方要做的：把改动切成几次发布（顺序写分几次写）。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:5459`），条数上限是在飞上限「    journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/system_configuration.rs:118`）。条数跟着点名项走（数据单元与元数据节点都点名），不跟着数据字节走。

历史：攒批窗口里只收 `UserChange::NewInodes { count, .. }`（数据字节 0），或只收「按 inode 号覆盖写」而内容很短、落在许多不同的 inode 上。照第三种读法脏字节几乎不涨，脏量那一支不触发；这一批的点名项只由时间那一支截断。已定项 5 写 `T_time` 用户可设、「超出 [200 ms, 5 s] 不拒绝，只给警告」（`.claude/kb/decisions/16-发布语义.md:162`），字段 u32 毫秒的上沿约 49.7 天：时间那一支可以被设得很长，这时只剩脏量那一支，而它不数这些改动。结局是返回成功之后在发布时被条数上限拒（P3-丁）。

算术（按冻结副本的常量，推的，没跑负载）：默认环 `JOURNAL_RING_DEFAULT_BYTES` = 805_306_368，记录 4096 字节，F = 3 ⇒ 在飞上限 65536 条，一条 64 项 ⇒ 一次发布至多 4,194,304 个点名项。要真撞上它，只改元数据的改动得在一个窗口里点名几百万个节点：默认 5 秒的窗口里撞不上，`T_time` 设长之后撞得上。这一格打中的是条款口径（脏量那一支不管它该管的东西），默认参数下的实害没量到。

### P2-乙　和 min(字段, 环长 ÷ F) 那一夹量纲不合（打中，推的）

夹法原文：「有效 `T_dirty` = min(系统配置里那个字段, 环长 ÷ F)」（`.claude/kb/decisions/16-发布语义.md:162`）。环长 ÷ F = 268,435,456 字节 = 在飞上限 65536 条 × 4096 字节，夹的是 journal 字节。按数据单元字节读：数据单元 `DATA_UNIT_BYTES` = 32768，256 MiB 数据是 8192 个单元、约 128 条记录（64 项一条），离 65536 条差 512 倍；按 C310 那份「一事务一条记录」的旧账也差 8 倍。两种账下这一夹都不咬数据负载，咬不咬元数据负载取决于 P2-甲那几样数不数。这一夹只有在「脏字节」是 journal 字节（或与它成比例的量）时才自洽。

### P2-丙　批内同一单元覆盖写 K 次，占 K 个落点、只数 1 个（打中，推的）

- 已定项 5 的取号口径「触发之后不再分配事务进它，已分配的事务把各自的单元写完」（`.claude/kb/decisions/16-发布语义.md:164`）：单元在开放窗口里就写下、各占落点。
- 已定项 15「**定案**：checkpoint C 中产生的一切释放，在 C 被发布之前不得进入可分配集合。」（`.claude/kb/decisions/16-发布语义.md:335`）：同一批里被后来者盖掉的那 K − 1 个落点在这一批发布之前不回可分配集合。
- D8 已定项 13 折叠：「同一次发布里同一个 key 改了多次时落叶只做一次」（`.claude/kb/decisions/08-核心索引结构.md:25`）：落叶只见最后一个。

所以「数据单元字节」按折叠之后数是 1 个单元，落点占了 K 个；按收下的字节数是 K 个单元，与落点一致，而与「它就是脏页内存预算本身」（`.claude/kb/decisions/16-发布语义.md:162`）那句（内存里只留折叠后的一份时）又不一致。今天每次改动各发一次，同一个文件热写时落点只多扣 defer 窗口那几个；攒批之后这一批里多扣 K − 1 个，K 最大到 T_dirty ÷ 单元大小（默认有效值下 8192）。读法要连同「单元什么时候取落点」一起定，单写「数据单元字节」是两种量。

### 测试里跨不跨得了阈值

按任何一种字节读法，只供测试的开关把有效 T_dirty 压到一个单元就跨得了（推的）；要分辨 P2-甲，用例要有一段只改元数据的批：按数据字节读，这段永远不触发。

### P2 的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分：P2-甲、乙只中「数据单元字节」读法，不中「这一批的点名项或 journal 字节」读法；P2-丙只中按折叠后数、单元在收下时取落点的组合 |
| 系统当时看不看得到判别它的东西 | 看得到：会话在收下时知道这次改动点名几项、取了几个落点 |
| 满足的是判据字面的哪一句 | 已定项 2 依据「只按时间……撑过环几何」那一分句；已定项 5 夹法那一句；已定项 15 定案句 |
| 正文给的改法在这几格上还中不中 | 正文只给了第三种读法，没给别的改法。我的改法（只在我的推导上、被攻过零轮）：脏量按「这一批收下时已点名的 journal 记录条数 × 4096」数，与夹法同一量纲，收下时超过就先发 |

## P3　一次发布里攒什么

### P3-甲　删除类的步并进别的改动：违 D8 已定项 16 第 2 条，判器也跟着看触发落在哪（打中，推的）

- 条款原文：「意图那次、每一批与续做的每一批各占一个事务号，等于本实例的下一个（与普通发布同一套编号）」（`.claude/kb/decisions/08-核心索引结构.md:505`）。一个事务号就是一次发布（D16 已定项 6「一个 txg 一个号」）。把 unlink 的意图、某一批或最后一批与别的改动并进同一次发布，字面上就违这一句，不是「可以并、要证」的问题，要并就先改这条已定项。
- 同事务约束：「不可能出现「意图没了但目标状态没达成」——**删除意图必须与最后一批工作在同一个事务里提交**。」（`.claude/kb/decisions/08-核心索引结构.md:122`）。今天每一步各一次发布，崩溃枚举流在每两步之间都有一个发布边界，坏实现「删 `deleted_inodes` 那条 key 放在最后一批之前那次发布里」必有一个崩溃态落在两者之间、判红。并批之后边界由触发定：同一个坏实现只在触发恰好落在这两步之间时才红，固定一个触发时刻的流（例如只在末尾到点）就把两步装进同一次发布，判器看不到。所以「判器还分不分得出」答「只在流把触发位置也枚举一遍时分得出」——这一条要写进验收，否则改坏之后流照绿（推的：没有攒批实现，没跑改坏）。

### P3-乙　攒批里没发布的 unlink，队列与写入口都看不见（打中，推的）

- 队列读法：「会话里的队列每次从现行那一版读、不缓存」（`.claude/kb/decisions/08-核心索引结构.md:509`）；实现「先把现行那一版的 `deleted_inodes` 走空（每一次用户改动都先走空」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:1902`）。
- 写入口判「删除中」读的是上一版的 inode 记录：「let is_being_deleted = record.nlink == NLINK_OF_AN_INODE_BEING_DELETED;」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:9384`）。

历史：现行那一版有 inode 1、inode 3；同一个攒批窗口里收下 `Unlink { inode: 3 }`（只收不发），接着收下 `OverwriteOfAnInode { inode: 3 }`。第二次改动进门「先走空」读现行那一版，队列空；写入口读上一版，inode 3 的 nlink 不是 0，放行。两样一起发布，就是条款第 4 条禁的「写入口拒 nlink = 0 的 inode」被绕开、对象被写回活的。再加「任何一版里 `deleted_inodes` 至多一条 key」（`.claude/kb/decisions/08-核心索引结构.md:509`）：两次 unlink 进不了同一次发布。结论：unlink、截短、删快照这三种承诺类改动只能当攒批的屏障（进门先把攒着的那一批发掉，自己照今天各步各发），建快照照今天「再发一次独立发布，不与别的改动合在一起。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:122`）；能并的只剩覆盖写、顺序写、建 inode、变长与等长的 size 改动。

### P3-丙　整理的两样保证锚在「一次调用发成」上（打中，推的）

- 「从头到尾，为这次改动推的抬 F 与之后的再判之间插不进整理批（第三轮判决第二节 G8 的第三样保证）」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:1992`）。攒批之后改动的收下与发布分在两次调用里，中间可以插整理的后台入口；这一样保证要改写成「收下到这一批发成之间插不进整理批」，或者整理入口进门先把攒着的那一批发掉。
- `B_gc` 的重置「compaction.publishes_since_the_last_user_change = 0;」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:1993`）今天放在改动发成之后。改成「收下就重置」的话，几次只收不发的改动各给整理一份新预算，而环里没有新状态，就是 P1-丙那一形（候选集挤成 1 个）由整理来做；重置要放在「改了用户可见状态的那次发布发成之后」。

### P3-丁　一批合起来的准入与条数上限只在发布时判（打中，推的）

今天两种拒都在一次改动之内、写之前：条数超限「在算定这次发布的样子之后、准入、读盘核与动分配器之前返回，一个写都没发、盘上逐字节不变。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:5460`），空间准入同样按一次发布判。攒批之后，这一批里每次改动都已返回成功，发布时被拒，拒绝没有一个能交给的改动；D16 已定项 17 管的是 fsync 报错，不管「返回成功之后整批在写之前被拒」。合算的口径（推的）：准入需求 = 这一批每次改动按收下时取的落点之和（P2-丙：不按折叠后算），条数 = 这一批全部点名项 ÷ 64 向上取整；两样在收下那一刻按「已攒的 + 这一次」判，过了就先发已攒的那一批、再收这一次，仍不过才把这一次拒在任何写之前。

### P3 的四句

| 问 | 甲 | 乙 | 丙 | 丁 |
|---|---|---|---|---|
| 分不分辨臂 | 分：只中「删除类并批」那一臂 | 分：只中「unlink 不当屏障」那一臂 | 分：只中「收下与发布分开、保证不改写」那一臂 | 分：只中「发布时才判」那一臂 |
| 系统当时看不看得到 | 看得到：会话知道这次改动是删除类 | 看得到：攒着的批在会话手里，只是条款让它读现行那一版 | 看得到 | 看得到：收下时就算得出点名项与落点 |
| 字面哪一句 | D8 已定项 16 第 2 条末句；D8 已定项 5 三态表下那一句 | D8 已定项 16 第 6 条「从现行那一版读」、第 4 条写入口拒 nlink = 0 | 实现注释里的 G8 第三样保证 | 条数超限拒的文档句 |
| 正文给的改法还中不中 | 正文没给改法 | 正文没给改法 | 正文没给改法 | 正文没给改法；上面那段合算口径只在我的推导上、被攻过零轮 |

## P4　攒批之后「返回即落盘」不再成立

### 数出来的（量过：冻结副本上 grep，工作目录 `crates/`）

```
$ ls singlefs-checker-tier/tests/crash_enumeration_*.rs | wc -l
34
$ grep -l 'publish_user_change(' singlefs-checker-tier/tests/crash_enumeration_*.rs | wc -l
0
$ grep -l 'unlink_the_first_file_through_the_session' singlefs-checker-tier/tests/crash_enumeration_*.rs
singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_fixed_script_stream.rs
$ grep -rl 'publish_user_change(' singlefs-harness/tests --include=*.rs | wc -l
18
$ grep -rlE '\bpublish_(overwrite|sequential_write|first_file|new_inodes|unlink_intent|overwrite_of_an_inode|sequential_write_of_an_inode|size_change|create_snapshot)\(' singlefs-harness/tests --include=*.rs | wc -l
120
$ grep -n 'publish_user_change(' singlefs-harness/src/history.rs | wc -l    # 随机历史
8
```

```
$ ls singlefs-harness/tests/*.rs | wc -l
295
```

（`history.rs` 那 8 处在第 4645、5077、5251、5486、5587、5670、5765、7245 行。）

读法：

- 崩溃枚举流 34 条，只有 1 条经会话（`common_crash_points` 的 「pub fn unlink_the_first_file_through_the_session」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:203`）），其余 33 条不调 `publish_user_change`、每一步是一次自足的事务层发布。攒批放在会话层，这 33 条一条也走不到攒批：它们之后照绿，说明不了攒批对不对。要有新的流：攒批窗口里收下几次改动、触发落在各个位置、每个位置崩。
- harness 测试 295 个文件里 18 个经会话、120 个直接调事务层发布函数（两组有交叠，按文件数，不是按断言数）。
- 真把「返回即落盘」写进判法的是三处，都在会话那一侧：随机历史经会话发每一步（`history.rs` 8 处）；模型对拍逐次比每次调用发成的各版：「（之后没再推抬 F 时）。交给调用方逐次比（模型对拍）：会话的 `current` 只是最后那一版。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:289`）；RefFS 翻译层把一步写成「改动 + 显式发布点」：「发布第一个文件 = 建第一个文件 + 发布调用；覆盖写 = 覆盖 + 发布调用」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/reference_filesystem_translation.rs:13`），代码「                operation: Operation::ExplicitPublish,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/reference_filesystem_translation.rs:364`）。这三处在攒批之后的判法变化见 P5。「多少条用例的断言依赖返回即落盘」按断言数我没数（要逐条读），只数到文件与调用点。

### 不加「立刻发布」入口，验收还验不验得了

验收原文（里程碑四待优化表那一行）要「用可控时钟和脏量分别跨阈值，证明先到触发」。不加入口也验得了「触发」本身：可控时钟推过 T_time、或脏量压过只供测试的阈值，再看有没有发布、是哪一支触发的。验不了的是「返回即落盘」那一类既有断言：随机历史与 RefFS 翻译要在每一步之后知道「这一步发没发布」，没有立刻发布的入口，就得把每一步的结局改成「收下」与「发布」两件事分开报（推的）。入口的形态按已定项 11「**定案**：所有结构、所有操作先进同一个 journal、共用同一个 checkpoint 节奏。」（`.claude/kb/decisions/16-发布语义.md:283`）：它只能是「把攒着的这一批现在发」，不能另走一条路径。

### P4 的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分：只中「攒批放在会话层、靠既有流兜底」那一臂；攒批放在事务层的话 33 条流全要改，另一种代价 |
| 系统当时看不看得到 | 不涉及（判的是验证面，不是被判系统的决定） |
| 字面哪一句 | 正文 P4「现有崩溃流、模型对拍与 harness 随机历史里有几条假设「每次改动返回即落盘」」那一问 |
| 改法还中不中 | 正文的「立刻发布」入口修随机历史与 RefFS 翻译，修不了「33 条流碰不到攒批」那一格 |

## P5　RefFS 怎么接自动触发的发布点

RefFS 今天的允许集合：「一次崩溃之后恢复允许读出的状态：{S_j : j = lo 或 j 恰是某次显式发布或回退的末尾，lo ≤ j ≤ hi}，按下标升序。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-reference-filesystem/src/lib.rs:142`），文件头写明「自动触发的攒批发布落地之前只接显式发布点（r6 判决第二节 U2 一行）。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-reference-filesystem/src/lib.rs:15`）。

原型 `proto/src/bin/reference_filesystem_publish_points.rs` 原样调冻结副本的 RefFS（不改它），只固定起点（建第一个文件并发布），由用户决定的两样放开扫：崩溃之前覆盖写几次（1..=4）、自动发布落在第几次之后（0..=k）。四种交法：照今天的翻译交（每步「改动 + 显式发布点」，攒批之后返回成功而那一批没发布）、只交内容改动、放宽成 [lo, hi] 里任意 S_j（原型自己算）、再加按单元键序前缀（原型自己算）。全量原样输出（`out/reference_filesystem_publish_points.out`）：

```
name=auto_publish_between_changes overwrites=1 auto_publish_after=0 translated_today=red(allowed_states=1) content_only=green any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=1 auto_publish_after=1 translated_today=green content_only=red(allowed_states=1) any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=2 auto_publish_after=0 translated_today=red(allowed_states=1) content_only=green any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=2 auto_publish_after=1 translated_today=red(allowed_states=1) content_only=red(allowed_states=1) any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=2 auto_publish_after=2 translated_today=green content_only=red(allowed_states=1) any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=3 auto_publish_after=0 translated_today=red(allowed_states=1) content_only=green any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=3 auto_publish_after=1 translated_today=red(allowed_states=1) content_only=red(allowed_states=1) any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=3 auto_publish_after=2 translated_today=red(allowed_states=1) content_only=red(allowed_states=1) any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=3 auto_publish_after=3 translated_today=green content_only=red(allowed_states=1) any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=4 auto_publish_after=0 translated_today=red(allowed_states=1) content_only=green any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=4 auto_publish_after=1 translated_today=red(allowed_states=1) content_only=red(allowed_states=1) any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=4 auto_publish_after=2 translated_today=red(allowed_states=1) content_only=red(allowed_states=1) any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=4 auto_publish_after=3 translated_today=red(allowed_states=1) content_only=red(allowed_states=1) any_state_between_lo_and_hi=green
name=auto_publish_between_changes overwrites=4 auto_publish_after=4 translated_today=green content_only=red(allowed_states=1) any_state_between_lo_and_hi=green
name=auto_publish_between_changes_summary cases=14 red_translated_today=10 red_content_only=10 red_any_state_between_lo_and_hi=0
name=one_request_split_by_the_trigger units_published=0 translated_today=red(allowed_states=1) content_only=green any_state_between_lo_and_hi=green unit_key_prefix=green
name=one_request_split_by_the_trigger units_published=1 translated_today=red(allowed_states=1) content_only=red(allowed_states=1) any_state_between_lo_and_hi=red unit_key_prefix=green
name=one_request_split_by_the_trigger units_published=2 translated_today=red(allowed_states=1) content_only=red(allowed_states=1) any_state_between_lo_and_hi=red unit_key_prefix=green
name=one_request_split_by_the_trigger units_published=3 translated_today=green content_only=red(allowed_states=1) any_state_between_lo_and_hi=green unit_key_prefix=green
name=one_request_split_by_the_trigger_summary cases=4 red_translated_today=3 red_content_only=3 red_any_state_between_lo_and_hi=2 red_unit_key_prefix=0
name=accept_after_a_failed_auto_publish recovered=first failure_on_a_publish_point=not_accepted(operation_after_a_failure_after_writing) failure_not_submitted=green
name=accept_after_a_failed_auto_publish recovered=failed_publish_landed failure_on_a_publish_point=not_accepted(operation_after_a_failure_after_writing) failure_not_submitted=red(allowed_states=1)
```

读法：

- 照今天的翻译交，红的 10 格全是「自动发布点落在最后一次改动之前」：返回成功抬了 lo，恢复读出 lo 之前那一版就判红。这是合法态被判红（攒批之后「返回成功」不等于「发布」）。
- 只交内容改动，红的 10 格是「自动发布之后崩、读出发布那一版」：lo 没动，RefFS 不知道那个自动发布点。
- 放宽成 [lo, hi] 里任意 S_j，这 14 格全绿；但一次覆盖写被触发切开的 4 格里仍红 2 格（读出前 1 或 2 个单元新、其余旧，不是任何一个 S_j）。D16 已定项 5 允许这一形：「同一次请求切出的若干事务按其单元的 key 升序取号」（`.claude/kb/decisions/16-发布语义.md:164`）、「触发之后不再分配事务进它」（同一行）。RefFS 自己写着它不认一个操作的中间内容：「就是这个操作施加之前与之后的状态，不用另起一类「一个操作的中间内容」。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-reference-filesystem/src/lib.rs:58`）。要么条款写「触发不切开一次请求」（与已定项 5 的取号口径、与条数超限时「顺序写分几次写」都要对齐），要么 RefFS 加按单元键序的前缀态。
- 自动发布写了之后报错、攒批照收下一次改动：报错挂在显式发布点上，RefFS 判不了（`not_accepted(operation_after_a_failure_after_writing)`，来自「没写报错与崩溃之间起的操作算什么，第一版不支持。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-reference-filesystem/src/lib.rs:167`））；不交报错，读出失败那一批（D16 已定项 17「fsync 报错之后，这次发布的数据**可以出现也可以不出现**」（`.claude/kb/decisions/16-发布语义.md:370`））就判红。

### 由谁把自动发布点告诉 RefFS

今天的先例是调用方翻译：「调用方把实现写下的每一条根（发布调用、回退、可写挂载的写行与暖机、抬 F、卸载）映到写它时的现行版本」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-reference-filesystem/src/lib.rs:92`）。把自动发布点也这样交进去，RefFS 判的就是「实现说它在哪发布了」：实现把发布点记错（比如记成在第 2 次之后，实际在第 1 次之后），RefFS 跟着错，这一格只剩别的判器兜。不交、照「[lo, hi] 任意 S_j + 单元键序前缀」判，不要实现给任何信息，代价是判别力：RefFS 再也抓不到「恢复出一个从没发布过的中间版」那一类（单文件整份内容的状态模型里，这一类与合法前缀态逐字节同形，抓不到的那部分本来就在这个状态模型之外，推的）。后一种改法只在我的模型上量过、被攻过零轮。

### P5 的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分：今天的集合与只交内容两臂各红 10 / 14 与 3 / 4；放宽臂 0 / 14、2 / 4；加键序前缀 0 / 14、0 / 4 |
| 系统当时看不看得到 | RefFS 看不到自动发布点（它的输入白名单里没有）；这正是要不要由实现告诉它的那一岔 |
| 字面哪一句 | RefFS `AllowedSet` 文档那一句；D16 已定项 5 取号口径；已定项 17 第一条 |
| 改法还中不中 | 正文没给改法；「放宽」修 14 格、修不了切开请求那 2 格；「加键序前缀」4 + 14 格都修，判不了「报错之后又收改动」那一形（要 RefFS 先支持报错之后的操作） |

## P6　阈值的只供测试开关与「配置生效」

### P6-甲　今天的「只供测试的开关」在产品构建里够得着，先例就是一条绕开准入的路（打中，量过）

先例：「只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：发布与可写挂载判不判空间准入。产品路径恒 `JudgedByTheFormula`。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/admission.rs:1078`），成员「SkippedByTheTestOnlySwitch,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/admission.rs:1086`）；装法「pub fn set_space_admission(&mut self, space_admission: SpaceAdmission) {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/allocator.rs:2003`），会话的分配器是 pub 字段「    pub allocator: PoolAllocator,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:91`），挂载入口「pub fn mount_writable_with_space_admission<Device: BlockDevice>(」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mount.rs:8099`）。core 的 `Cargo.toml` 里这几样都不在特性门后面（唯一的特性是 `lookup-work-counters`）。

原型 `proto/src/bin/test_only_switch_in_a_release_binary.rs` 是普通 bin 目标（不是测试、不经 dev 依赖、不开特性），里面一个函数对 `&mut MountedSession` 做 `session.allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch)`。`cargo build --offline --release` 编过（31 秒，副本），跑出原样：

```
name=test_only_switch_in_a_release_binary cfg_test=false debug_assertions=true session_entry_compiles=true mount_entry_is_public=true switch_branch_name=space_admission=skipped
```

关掉之后被式子拒的那次覆盖写做成，入库用例里已经有：「        .expect("同一次挂载里关掉准入，同一次覆盖写做成：拒的是式子，不是落点")」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs:500`）。所以「测试开关成了绕开准入的第二条路」今天就成立，不是阈值开关才有的事；阈值开关照「与空间准入开关同一个做法」（`unlink::DeletionSwitches` 文档那一句）写，就同样是任何下游调用方在产品构建里可设的第二条配置通道，挂载选项通道（里程碑十）还没来就先有了它。

### P6-乙　开关设在哪一层，决定夹法那条分支还测不测得到（打中，推的）

开关要是直接给「有效值」，min(字段, 环长 ÷ F) 那一夹就被跳过：夹法那条分支测试进不去（违 `.claude/rules/fs-design.md`「每条分支必须能被测试强制进入」那一条），产品构建里还能用它把有效 T_dirty 抬过环长 ÷ F，结局是 P3-丁那种返回成功之后发布被条数上限拒。开关要给的是夹之前的那个值（与盘上字段同一个位置），夹照常做。

### P6-丙　「配置生效、无越界误触发」在里程碑四验到哪一步（推的）

- 能验：开关给的值进了触发判据（可控时钟推到 T_time − 1 毫秒不发、推到 T_time 发）；夹法生效（给大于环长 ÷ F 的 T_dirty，按夹后的值触发）；无越界误触发（脏量差一字节不发）。
- 验不了、留给里程碑十：挂载时传值覆盖盘上字段并回写（C422）、重挂读回生效值（C424）；C733 已定的处置「两端值（低于 1 ms 与超过 u32 毫秒上限）挂载时拒绝」（`.claude/kb/checks-owed.md:459`）住在挂载传值那条路上，里程碑四没有入口，这一条分支在里程碑四进不去，要在条款里点名延到里程碑十，否则就是一条没有测试能进的分支。
- 越界警告（「超出 [200 ms, 5 s] 不拒绝，只给警告」）在里程碑四没有接收方：没有挂载选项、没有日志通道。能做的是挂载结果里带一个类型化的观察项（同 `MountOutput` 里别的观察项），用例断言它在 199 ms 与 5001 ms 时出现、200 ms 与 5000 ms 时不出现。

### P6 的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分：只中「照空间准入开关的做法写」那一臂；放进默认关的 cargo 特性（同 `lookup-work-counters` 的做法）、只门住设置入口而判据代码不分叉的那一臂不中 |
| 系统当时看不看得到 | 不涉及运行时判别；判的是 API 面 |
| 字面哪一句 | 正文 P6「不变成第二条配置通道」；`admission.rs` 自己写的「产品路径恒 `JudgedByTheFormula`」在 API 面上不成立 |
| 改法还中不中 | 正文没给改法；特性门那一臂只在我的编译上推过（没编那一臂）、被攻过零轮；它的代价是产品构建与测试构建的设置入口不同，判据代码同一份 |

## P7　串行下不该在里程碑四立的

### P7-甲　C425 原形串行下走不到；照字面式子的空转串行下照样有（部分打中，量过）

C425：「| C425 | T_time 小于一次发布耗时时触发空转 |」（`.claude/kb/checks-owed.md:300`），病根写的是「上一次没发完下一次就触发」，在飞深度 2。串行、深度 1、发布同步占着调用方线程：发布进行中定时入口进不来，每次触发就是一次发布，「触发次数与实际发布次数」恒等，这一形确实不出现（没打中）。

但照字面式子（P1-丙）、上次 checkpoint 的时刻取触发那一刻、T_time 小于发布耗时 D 时，每次发完下一次 tick 就到点：空闲时连推空发布。模型第三节 12 行，原样摘：

```
name=serial_short_t_time T_time=0.2 D=0.5 tick_period=0.05 idle_hour_publishes_literal=7200 idle_hour_publishes_dirty_guarded=0 busy_worst_accept_to_durable_seconds=1.2
name=serial_short_t_time T_time=1 D=10 tick_period=1 idle_hour_publishes_literal=360 idle_hour_publishes_dirty_guarded=0 busy_worst_accept_to_durable_seconds=21
```

（这一节的次数是闭式近似：一小时 ÷ max(D, T_time, tick 周期) 量级，推的成分大于量的；要紧的是「加脏字节 > 0 那一臂恒 0」。）持续写的负载下一次改动收下到落盘至多约 2D + max(T_time, tick 周期)，有界：串行下「脏数据一直攒」不出现。

### P7-乙　「重做期间开放的 checkpoint」串行下也会一直攒（打中，推的）

今天发布落盘途中写错就冻结，之后的发布在写之前拒：「PublishFrozenAfterAWriteFailureIsNotResentYet {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:5453`），文档「重发成功才建下一次发布）：先 `resend_the_frozen_publish`」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:5451`）。重发由调用方起。今天每次改动都要发布，冻结期间每次改动都被拒，写者被挡在门外，什么也攒不下。

攒批之后改动的收下不等于发布：自动发布写错冻结之后，会话要是照常收下改动，这些改动就攒在 D16 已定项 5 说的那个「重做期间开放的那个 checkpoint 发布不了、只能一直攒」（`.claude/kb/decisions/16-发布语义.md:164`）里，攒多久由调用方什么时候重发定，串行、深度 1 下同样出现。要立的条款：冻结着一次发布时，收下改动也在写之前拒（与今天发布时拒同一个拒因）；或者明写「冻结期间收下的改动不进任何承诺，直到重发成功」。前者在串行下等于今天的行为，推荐它（只在我的推导上、被攻过零轮）。已定项 5 的「**「上次 checkpoint 的时刻」在重发时取重发那一刻**」（`.claude/kb/decisions/16-发布语义.md:164`）在这一形上要跟着落：重发成功那一刻重置计时。

### P7-丙　丢窗口式子里的切换项在里程碑四是 0（没打中）

「的「重试到 T_retry 用尽才算失败」这里没有做」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:1000`）：挂载内实例切换、T_retry 都没实现，N_switch × (T_retry + T_redo) 这一项在里程碑四串行下取 0，可以留到里程碑六；但 P1-乙那几项（调用方 tick、长调用、钟回拨）要替它进式子。

### P7 的四句

| 问 | 甲 | 乙 |
|---|---|---|
| 分不分辨臂 | 分：只中照字面式子那一臂 | 分：只中「冻结期间照常收下」那一臂 |
| 系统当时看不看得到 | 看得到：脏字节为 0、有冻结着的发布，会话都知道 | 看得到 |
| 字面哪一句 | 已定项 2 定案式子左分句 | 已定项 5「重做期间开放的那个 checkpoint……只能一直攒」 |
| 改法还中不中 | 「脏字节 > 0」合取修它 | 「冻结时收下也拒」修它；只改计时那一句修不了 |

## 没打中的形状

- P7 C425 原形（触发与发布脱钩、在飞深度 2 才有）：串行下按构造走不到，没造出反例。
- P7 切换项：T_retry 与挂载内实例切换没实现，取 0。
- P5 放宽臂的判别力损失：想造「放宽之后漏掉一个真坏实现」的历史，在单文件整份内容的状态模型里造不出（中间版与合法前缀态逐字节同形），扫了覆盖写 1..=4 次、自动发布落点 0..=k 共 14 格。
- P2 默认参数下撞在飞上限：默认 5 秒窗口里只改元数据撞不上 4,194,304 项，只在 T_time 设长时撞得上；没跑负载量。
- P3 整理与攒批之间的 discard：想过「攒着的批的单元落在一段被判可整段 discard 的段里」，「pub fn discard_the_segments_reusable_as_a_whole<Device: BlockDevice>(」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:2679`）读的是分配器，收下时取的落点在分配器里算已分配，推不出它被 discard，没写成打中。

## 这条腿自己的限度

- 没有攒批实现可改坏：P1、P3、P7 的打中是在条款与代码行上推的历史，加两份计数模型；只有 P5（真 RefFS）、P6（真 core 的 API 面）、P4（grep 计数）是量过的。
- 计数模型的扫描上沿就是窗口的上沿（空闲 3600 秒、长调用 30 秒、回拨 120 秒），报的是「随它涨」，不是最坏值。
- P7-甲的次数是闭式近似。
- 没跑崩溃状态枚举，没跑任何门禁阶段、checker 档测试与 GPU。
- 只抽了一次样：这几格的「没打中」按规则不算数。

## 没做什么

- 不判正推、辩方那几格，不替主 agent 采纳；副本上的数不是入库装置上的数。
- 没读禁读清单里的正推腿产出。
- 编译与跑的负载：开跑前 `ps` 看到别的会话的 cargo（合入后验证、变异复跑、崩溃放量）在跑，没有性能测量在跑；本机内存 slice 紧，第一次 8G 上限的编译排了 607 秒没起跑，我停了它的排队进程，改 3G 上限重编，31 秒编完；全部在本机加 `nice -n 19` 跑。第一次编译命令我给的前台超时写成了 600000 毫秒，超过共用约束的 240000，被移到后台，等完成通知之后才接着做。
- `crates/`、`scripts/` 下的 `.rs` 一个没改，命名检查不涉及；没有要主 agent 跑的命令。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-publish-trigger-r1-opus/` 里的仓副本与编译目录 `tree/`（删前 `du -sh` 299M）已删；复跑目录 `rerun-check/`（48K，只剩三份输出与编译日志，拷贝已由 rerun.sh 删掉）与几份文本日志留着。
