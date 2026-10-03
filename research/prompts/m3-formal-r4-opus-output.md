# m3-formal-r4 攻方腿（Opus）报告（第一轮，2026-10-02）

攻击面：正文第四节分工表「云端攻方（Opus）」一行，T1–T6，重点 T2、T5。被判对象读冻结副本 `/tmp/claude-1000/m3-formal-r4/tree/`；量都在它的拷贝 `/tmp/claude-1000/m3-formal-r4-opus/tree/` 上（副本上的数，不是入库装置上的数）。下文「量过」都指这份拷贝上的原型。

## 复跑

原型是在拷贝里加的一个 crate `r4probe`（依赖 singlefs-core、singlefs-harness、singlefs-checker-tier 三个库，不跑 checker 档的任何测试目标）。复跑：

```
bash research/prompts/m3-formal-r4-opus-model/rerun.sh /tmp/claude-1000/m3-formal-r4/tree /tmp/claude-1000/<草稿目录>
```

它把冻结副本 rsync 到草稿目录、放进 `Cargo.toml` 与 `main.rs`、在工作区成员里加一行、经内存包装编 release，再逐段跑 `r4probe t5 <历史> 10`，输出 `t5b-<历史>.log`。只量状态数：`r4probe sizes <历史>`。

模型目录 `research/prompts/m3-formal-r4-opus-model/` 每个文件的 sha256（`sha256sum *` 原样）：

```
490844a5bdcdb37453f11bf0ca3cb057974a4f67ec435d647233f3aa0eafc50f  Cargo.toml
06905c7cc8ad6ff162c41983a59067d4bcc2219c6f56c0c4749050bd570d9e06  main.rs
cbc56f1365fbf7596a8105da9dae8e94ff67dab579919c94fb9faf5c36978279  rerun.sh
ee18ceeee062bdf85e833c647e6d4780964ba9b5347400d0c1b6d4182a49bedf  sizes-small-h0.log
2300d88def97d53b56005899202f66a1710142d514cb6829585f727a3b719f70  sizes-small-h1.log
fad172a327272bad7b1b98012003ea46b0a2b37fab3ff3daaf86122b0d3c8d8c  sizes-small-h6.log
15783c00742b9a8dabab2fc6372f3f052153d05b8391fa8e2c09ee628d48801b  sizes-small-s10.log
8f25a1ff9e96291e30151a0031d5a355e19d2d1d266b4109db59bb693e0ba874  sizes-small-s1.log
09d495db3ae9ba1f613604ba727418c7f2d211094fad01df23526153581a6fb8  sizes-small-s2.log
b0262b1cc298f6262a6f1402ce0fa32dd44a708b367ab1c6b4558cb694b13e0d  sizes-small-s3.log
628bf0e9bedf0799fb9c7881176d1c7a8bd501e3a320b6693b82de4bdcb4c438  sizes-small-s5.log
5fb24cbe01c4b4f3fe605b43e821128105221cc3a80d66c07010abe238927e30  sizes-small-s6.log
78330838992e43277499f1935a4c20fed56307bbf21f456c611312effa0ef2af  t5b-s10.log
8ea6c189216af3717d28c496d37ca43c6936ff045d313887bb83d33f3fd14e8a  t5b-s1.log
9145c130bcb56b2f161502e6ec542994dfd6893279a34df1c6cb2954e81021dd  t5b-s2.log
2f66f8ea9673fa8bcfd8f203b9598ac478de51a5b91fcadd76f378b58096a9ba  t5b-s3.log
6c36a016a109ad0014c0a3ad92555787efbd77a5ddd8b3a980a381821fd9283d  t5b-s5.log
cfbc4301fe2fc5451ce13accdc23c0b98e7ae4a149d46577a9372b79a6f79f22  t5b-s6.log
```

原型做什么：每段手写历史（不取自任何层 0 用例）跑一遍，固定前缀当基线不枚举，前缀之后那几步的录制流用 `crates/singlefs-checker-tier/src/crash.rs` 的 `judge_layer0_state_range` 加 `full_expansion` 逐状态取（层 0 同一枚举域：每段都展开、原地覆写取第三态）。每个崩溃状态：
- R1 = 崩溃镜像上的只读恢复，逐字段取（择系统配置、择根、扫环、重放、走读、实例表、分配记录、F 生效值、根环、回退候选集）；
- 用户动作放开扫四种（照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」最后一条）：`mount`（取号、写行、暖机）、`mount+write`（再经会话发一次覆盖写，版本没文件时发第一个文件）、`mount+unmount`、`mount+mount`（再挂一次）；每种在崩溃镜像的副本上做完，再只读恢复一次得 R2，与 R1 逐字段比。

历史（`main.rs` 的 `history` 与 `fixed_prefix`）：

| 名 | 起点 | 步骤 | 固定前缀 | 枚举的是什么 | 状态数（含第三态） |
|---|---|---|---|---|---|
| s1 | mkfs 之后 | 可写挂载 | 只有 mkfs | 第一次可写挂载（版本无文件） | 33 |
| s5 | mkfs 之后 | 可写挂载、零单元发布 | 只有 mkfs | 挂载与一次零单元发布 | 45 |
| s3 | 第一个文件之后 | 覆盖写、回退到环里次新那条 | 起点与覆盖写 | 管理员回退那次发布 | 4 108 |
| s2 | 第一个文件之后 | 可写挂载 | 起点 | 带文件的池上第二个实例的挂载 | 20 511 |
| s6 | 第一个文件之后 | 可写挂载、再可写挂载 | 起点与第一次挂载 | 第三个实例的挂载 | 20 511 |
| s10 | 第一个文件之后 | 24 次覆盖写、可写挂载 | 起点与 24 次覆盖写 | 根环转满之后的挂载 | 20 511 |

## 各格判定一览

| 格 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| T5 | 打中（量过） | 卸载记号、F（根记录里的与生效值）在 `mount`、`mount+mount`、`mount+write` 三种动作下，六段历史挂载做成的 65 634 个状态里一个都不变，只在 `mount+unmount` 下变（带文件的 65 591 个全变）；树 ID 水位只在发第一个文件时变（43 个）：把它们列进「允许不同」就是该相等却被模掉 | 第一节 T5-1 |
| T5 | 打中（量过） | 「已确认的发布必须相同」两种读法结局相反：读成「R1 那一版在 R2 里仍是回退候选」，由记录重建的那 3 个状态（每段历史）在任何动作下都不成立、加卸载时全部不成立；读成「R2 里挂载后发布之前那一版的内容等于 R1」，全部成立 | 第一节 T5-2 |
| T5 | 打中（量过） | 「用户可见状态必须相同」对 Y7 写的「发布一次之后再恢复」字面必红：要比的是挂载之后那次发布之前的那一版，不是 R2 的现行版 | 第一节 T5-3 |
| T5 | 打中（量过） | 实例表的行不是「模掉」而是有一条确定关系；「R2 = R1 加一行 (R1 实例, R1 txg, W)」在无文件的池上 7/18、7/25 个状态不成立（实例 0 不写行、被烧掉的号另得一行） | 第一节 T5-4 |
| T6 | 打中（量过） | 不带故障的域（D13 已定项 4 含第三态）里有一批崩溃状态可写挂载被拒，「挂载后再恢复」没有第二次结局；崩溃注入的枚举域不含第三态，Y7 选「崩溃注入里量」就一个都看不到 | 第二节 T6-1 |
| T6 | 打中（量过 + 条款） | 挂载读法不是 M3 的第三读：R2 的根在 8 个字段上 100% 与 R1 不同；与它同形的是 I-8.4 的等价关系（读出的内容相同，不比实例代号与行） | 第二节 T6-2 |
| T2 | 打中（量过） | 语义级允许集合（只看 POSIX 内容）放过盘级下界：回退那段 4 108 个状态里 9 个状态最后那条根槽已持久，语义级允许读回回退之前的内容，崩溃注入判据 ③ / 层 0 RecoveredToAnOlderRoot 不允许；三种丢法在这里一起放过（不分辨丢法，分辨的是「语义级单独判」与「语义级加盘级下界」） | 第三节 T2-1 |
| T2 | 打中（条款，推的） | 「按 POSIX 操作前缀丢」与 D23 已定项 14「崩溃原子性的单位是一个单元、不是一次写请求」字面冲突：攒批下一次写请求的事务可被 checkpoint 切开 | 第三节 T2-2 |
| T2 | 打中（量过） | 今天的 `crates/` 上三种丢法在每段可达历史上给出同一个允许集合（六段历史 65 719 个状态里 R1 读回的内容全部是「这几步之前」或「之后」那一份，没有第三种）：正文判据第 2 条要的「短历史上给出不同允许集合」在今天的实现上造不出 | 第三节 T2-3 |
| T1 | 打中（推的） | 今天没有不持久就返回的入口，fsync 三种读法在任何可达历史上同一允许集合；条款面上 D16 已定项 17 的「报错之后可以出现也可以不出现」要求允许集合里有第三种结局 | 第四节 |
| T3 | 打中（代码，推的） | 「RefFS 由 O1 扩成」与「语义级、没有盘上格式」互相冲突：O1 今天就带根环落点与盘级下界 | 第五节 |
| T4 | 没打中 / 推的 | 今天的可达历史上 RefFS 语义级判不出 O1 加判据 ③ 判不出的东西；C191 第一对要 O3 先交出 POSIX 内容才比得起来 | 第六节 |

（65 634 = s1 18 + s5 25 + s3 4 103 + s2、s6、s10 各 20 496，是六段历史里可写挂载做成的状态数；65 591 是其中带文件的四段之和。数由下文第一节那条 grep 命令现加。）

## 第一节　T5：挂载后再恢复，模掉什么

### 量法与总表

每个崩溃状态先算 R1，再对四种动作各算 R2（见「复跑」）。下表是六段历史合起来、可写挂载做成的状态上各字段「R2 与 R1 相同」的状态数，由这条命令现加（模型目录下跑）：

```
for f in eff.unmount_marker eff.rollback_floor effective_floor eff.tree_identifier_watermark; do for a in 'mount' 'mount+mount' 'mount+write' 'mount+unmount'; do s=$(grep -h "^FIELD action=$a field=$f verdict=same" t5b-*.log | sed 's/.*count=//' | paste -sd+ | bc); d=$(grep -h "^FIELD action=$a field=$f verdict=diff" t5b-*.log | sed 's/.*count=//' | paste -sd+ | bc); echo "$f $a same=${s:-0} diff=${d:-0}"; done; done
```

原样输出：

```
eff.unmount_marker mount same=65634 diff=0
eff.unmount_marker mount+mount same=65634 diff=0
eff.unmount_marker mount+write same=65634 diff=0
eff.unmount_marker mount+unmount same=43 diff=65591
eff.rollback_floor mount same=65634 diff=0
eff.rollback_floor mount+mount same=65634 diff=0
eff.rollback_floor mount+write same=65634 diff=0
eff.rollback_floor mount+unmount same=43 diff=65591
effective_floor mount same=65634 diff=0
effective_floor mount+mount same=65634 diff=0
effective_floor mount+write same=65634 diff=0
effective_floor mount+unmount same=43 diff=65591
eff.tree_identifier_watermark mount same=65634 diff=0
eff.tree_identifier_watermark mount+mount same=65634 diff=0
eff.tree_identifier_watermark mount+write same=65591 diff=43
eff.tree_identifier_watermark mount+unmount same=65634 diff=0
```

（`mount+unmount` 下 same=43 是 s1、s5：版本没文件时卸载一个字节都不写。树 ID 水位 diff=43 是 s1、s5 的 `mount+write` 发了第一个文件、建了新树。）

逐字段一张表（「量过」= 上面与六份 `t5b-*.log` 的 FIELD 行；字段名照产物）：

| 字段 | mount / mount+mount | mount+write | mount+unmount | 量过 / 推的 |
|---|---|---|---|---|
| 读回内容（`posix.content`） | 全同 | 等于那次发布写的内容，全同 | 全同 | 量过 |
| 挂载后那次发布之前一版的内容（`posix.content_of_the_version_before_the_post_mount_publish`） | — | 全同 | — | 量过 |
| 实例代号、checkpoint_txg、所选根键 | 全不同 | 全不同 | 全不同 | 量过 |
| 树表指针、映射树根、实例表指针、分配记录、系统配置运行量 | 带文件的全不同 | 全不同 | 全不同 | 量过 |
| 卸载记号、根记录的 F、F 生效值 | 全同 | 全同 | 带文件的全不同 | 量过 |
| 树 ID 水位 | 全同 | 只在发第一个文件时不同 | 全同 | 量过 |
| journal 施加条数 | 只在 R1 由记录重建的状态上不同 | 同左 | 同左 | 量过 |
| 实例表的行 | 见 T5-4，有确定关系 | 同左 | 同左 | 量过 |
| 回退候选集 R1 ⊆ R2 | 全成立 | 全成立 | 全不成立 | 量过；根环转满而 F 低于被挤出的根时不成立是推的（T5-2） |

R1 由记录重建（根槽没落、记录落了）的状态每段 3 个（s5 是 6 个），产物行例：`t5b-s2.log` 的「CLASS torn=false r1_outcome=file r1_applied_records=true count=3」。

### T5-1　卸载记号、F、树 ID 水位：该相等却被模掉

正文 T5 的候选清单把「卸载记号」放在允许不同那一边。量到的是：只要动作里没有卸载，它在 65 634 个状态里一个不变；F 两处（根记录里的、生效值）同。卸载才把 F 抬到现行 txg（「let rollback_floor = current_file_version.root.checkpoint_txg;」（`crates/singlefs-core/src/mount.rs:2139`）），记号也只在那时写。

打中之后四句：
- 分不分辨臂：分辨「无条件模掉卸载记号 / F」与「按动作条件模掉」两种写法。前一种放过「可写挂载错写卸载记号或把 F 抬了」这类坏实现，后一种抓得到。
- 判别子看不看得到：比对方手里有动作序列（自己发的），看得到。
- 满足的分句：正文第四节分工表攻方那一行「一个模掉之后仍不该相等、或该相等却被模掉的字段」（`research/prompts/_m3-formal-r4-body.md:47`）里「该相等却被模掉」；被打的是 T5 格里「哪些字段允许不同（实例代号、checkpoint_txg、根槽位置、卸载记号、实例表的行、分配记录……）」（`research/prompts/_m3-formal-r4-body.md:37`）列进去的卸载记号。
- 跑前条款的改法：正文判据「两条腿给的『模 / 不模』相反 ⇒ 在实现拷贝上量那个字段挂载前后的值」——已量，结果如上；无条件模掉这一改法在这几格上照样放过，写明「对这几格不起作用」。

改法候选（只在我的模型上量过、被攻过零轮）：卸载记号与 F 写成「动作不含卸载、抬 F（含准入推的抬 F）时必须相等」；树 ID 水位写成「动作不发第一个文件、不建树时必须相等」。

### T5-2　「已确认的发布必须相同」两种读法

- 读法 A（回退候选）：R1 走到的那一版在 R2 里仍是回退候选。量到：`mount`、`mount+mount`、`mount+write` 下每段 3 个由记录重建的状态不成立（那一版在根环里没有根，一开始就不是候选；产物行「FIELD action=mount field=r1_effective_is_a_candidate_in_r2 verdict=diff count=3」在 `t5b-s2.log`、`t5b-s6.log`、`t5b-s10.log` 都有）；`mount+unmount` 下带文件的全部不成立（F 抬到现行 txg）。根环转满：s10 上「FIELD action=mount field=ring.r1_subset_of_r2 verdict=diff count=20496」（`research/prompts/m3-formal-r4-opus-model/t5b-s10.log:170`），挂载写的三条根挤出了环里最旧的根；候选集仍是子集（同一份的第 69 行「FIELD action=mount field=candidates.r1_subset_of_r2 verdict=same count=20496」），因为这段历史在小盘上推过抬 F，被挤出的根早在 F 之下。F 不抬时被挤出的是候选——这一句是推的，条款面上 D16 已定项 8 自己写着「空发布占根环槽位，把旧的非空根更早挤出环」（`.claude/kb/decisions/16-发布语义.md:202`）。
- 读法 B（内容）：R2 里挂载之后那次发布之前的那一版（与 R2 同实例、txg 小一的根）读回的内容等于 R1 读回的内容。量到全部成立（`mount+write` 下 65 634 个状态全同，六份 FIELD 行 `posix.content_of_the_version_before_the_post_mount_publish verdict=same`）。

四句：分辨两种读法（A 在可达的无故障状态上为假，B 全真）；判别子看得到（两次恢复都读得到根环）；满足的是正文 T5「哪些必须相同（用户可见的 POSIX 状态、已确认的发布）」（`research/prompts/_m3-formal-r4-body.md:37`）里「已确认的发布」那半句；改法：读法 A 不能当「必须相同」，只能写成「卸载、抬 F、根环转满之外，R1 的候选集包含于 R2 的」（零轮）。

### T5-3　「用户可见状态必须相同」对 Y7 字面必红

Y7 的量法是「可写挂载取号、暖机、发布一次之后再恢复」（正文 T5，`research/prompts/_m3-formal-r4-body.md:37`），今天崩溃注入也正是挂载后发一次布（「let mount_run = mount_and_publish_once_on_the_crash_image(」（`crates/singlefs-checker-tier/src/crash_injection.rs:918`））。R2 的现行版是那次发布写的内容，与 R1 读回的不同是设计如此：`mount+write` 下 `recover.outcome_kind` 在 s1、s5 全不同（无文件变有文件），带文件的几段内容全换成那次发布的。所以「用户可见 POSIX 状态必须相同」按字面在每个状态上都红，除非比的对象换成「R2 根环里挂载之后那次发布之前的那一版」（T5-2 读法 B，全同）或者 Y7 不发那一次布（`mount`，全同）。

四句：不分辨 T5 的候选清单，打的是「必须相同」那一半的比较对象（打中不分辨臂，病根在共用前提「比 R2 的现行版」）；判别子看得到；满足 T5 格「哪些必须相同（用户可见的 POSIX 状态、……）」；改法：比的对象写成「R2 里新实例暖机的最后一版（txg = R2 现行 − 1，同实例）读回的内容 = R1 读回的内容」，或量法里去掉那一次发布——两个改法在这几格上都中（量过前一个）。

### T5-4　实例表的行：不是模掉，是一条确定关系

`mount` 下「R2 的行 = R1 的行再加一行 (R1 走到那一版的实例, 它的 txg, R1 施加的最大事务号 W)」在带文件的四段里全部成立（例：`t5b-s2.log` 的「FIELD action=mount field=rows.r2_is_r1_plus_row_for_r1_effective verdict=same count=20496」），在无文件的两段不成立：「FIELD action=mount field=rows.r2_is_r1_plus_row_for_r1_effective verdict=diff count=7」（`research/prompts/m3-formal-r4-opus-model/t5b-s1.log:133`；s5 同一行在第 134 行）。不成立的例子（`t5b-s5.log` 第 135–137 行，ordinal=10）：R1 走到 mkfs 的第 0 代根 (txg 0, 实例 0)、`rows=Some([])`；R2 走到 (txg 4, 实例 2)、`rows=Some([(1, 0, 0)])`——实例 0 不写行，崩在第一次挂载取号之后的那个号 1 被烧掉、得一行 (1, 0, 0)。

所以「实例表的行允许不同」太宽：它放过「挂载写错行」（W 写错、T 写错）这一类坏实现。条款候选（只在我的模型上量过、被攻过零轮）：R2 的行 = R1 的行 ∪ {[R1 走到那一版的实例, 新实例) 里每个非 0 实例一行；R1 那一版的实例那一行是 (实例, R1 的 txg, R1 的 W)}。我的原型只量了前半（R1 那一行在、R1 的行是前缀），中间被烧掉的号那几行的值没逐个核。

### T5 小结：允许不同 / 必须相同 / 有确定关系

| 类 | 字段 |
|---|---|
| 允许不同（量到每个状态都变） | 实例代号、checkpoint_txg、所选根、树表指针、映射树根、实例表指针、分配记录（逐条）、系统配置运行量（世代号、tail、实例） |
| 必须相同（量到全同，模掉会放过坏实现） | 读回内容（`mount`、`mount+mount`；`mount+write` 比挂载后发布之前那一版）；卸载记号、F（动作不含卸载与抬 F）；树 ID 水位（不发第一个文件） |
| 有确定关系（不是相同也不是任意） | 实例表的行（T5-4）；回退候选集（R1 ⊆ R2，卸载、抬 F、根环转满之外）；journal 施加条数（R1 由记录重建时 R2 为 0） |

「分配记录」我只比了整张表相等（全不同），没比「R1 那一版引用的用户可见单元在 R2 的账里仍是已分配」——后者才是不该模掉的那一半（推的，没量）。

## 第二节　T6：比的对象与射程

### T6-1　不带故障的域里有挂载被拒的状态，「挂载后再恢复」没有第二次结局

D13 已定项 4 把原地覆写（系统配置槽写在内）的第三态放进不带故障的枚举域：「全量与快档都枚举」（`.claude/kb/decisions/13-验证路线.md:78`）。原型在这一域上逐状态起可写挂载，凡第三态落在系统配置槽上的状态，挂载都在取号之前拒，四种动作一样（产物原样，每段一行）：

```
t5b-s1.log:16:ACTION mount | torn=true r1_outcome=no_file r1_applied_records=false -> mount_refused:NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfiguratio | count=15
t5b-s5.log:17:ACTION mount | torn=true r1_outcome=no_file r1_applied_records=false -> mount_refused:NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfiguratio | count=20
t5b-s3.log:19:ACTION mount | torn=true r1_outcome=file r1_applied_records=false -> mount_refused:NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfiguratio | count=5
t5b-s2.log:16:ACTION mount | torn=true r1_outcome=file r1_applied_records=false -> mount_refused:NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfiguratio | count=15
t5b-s6.log:17:ACTION mount | torn=true r1_outcome=file r1_applied_records=false -> mount_refused:NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfiguratio | count=15
t5b-s10.log:39:ACTION mount | torn=true r1_outcome=file r1_applied_records=false -> mount_refused:NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfiguratio | count=15
```

（截断处是成员名 `PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion`，`crates/singlefs-core/src/mount.rs:310`。）只读恢复在这些状态上照常读回文件（`r1_outcome=file`）。六段合计 85 个状态；有第三态的状态全部被拒，没有第三态的全部做成（每段 ACTION 行只有 torn=true → 拒、torn=false → ok 两类）。这是 C331 取甲的已知代价：「可写挂载每次都拒」（`.claude/kb/decisions/23-journal的角色与格式.md:407`），欠账「撕裂的系统配置轮换之后只剩只读挂载」（`.claude/kb/checks-owed.md:465`）。

两件事随之而来：

1. Y7 的候选「崩溃注入里量」看不到这一类：崩溃注入的枚举域「更晚的段一个都没持久，撕裂并进」（`crates/singlefs-checker-tier/src/crash_injection.rs:5`），不取第三态。候选「层 0 上量」取第三态，就撞上这 85 个状态的同类。所以两条候选在「射程」上不是一个罩得多、一个罩得少，而是一个碰得到「没有第二次结局」、一个碰不到。
2. 碰到了怎么判今天没有条款：崩溃注入现在把可写挂载被拒当成「模型说该成、实现拒了」报红（「崩溃后镜像上的可写挂载拒了：{member}」，`crates/singlefs-checker-tier/src/crash_injection.rs:1351`）。若把第三态加进崩溃注入、又不写例外，这 85 个同类状态当场红。

四句：
- 分不分辨臂：分辨 Y7 的「崩溃注入里量」与「层 0 上量」（前者 0 个、后者每段 5–20 个）；也分辨 T6 射程的两种写法（「不带故障的全部状态」要说被拒的怎么记，「挂载做成的状态」不用）。
- 判别子看不看得到：挂载读得到那一槽撕裂（就是因为读到才拒），看得到。
- 满足的分句：T6「射程限在哪些崩溃状态（不带故障的、带介质损坏的）」（`research/prompts/_m3-formal-r4-body.md:40`）里「不带故障的」——第三态按 D13 已定项 4 属于不带故障。
- 改法：(a) 射程写成「可写挂载做成的状态」，被拒的单列计数、不比；(b) 「被拒算相等」——没有 R2 可比，这条改法在这 85 格上不起作用；(c) 等 C596 有改法再比。(a) 在这几格上中（量过：85 个状态全被 torn 一类罩住）。

### T6-2　挂载读法不是 M3 的第三读

M3 两读比的都是根本身：不动点「返回的根不变」，记录级「得到的根相同」，另有一条「第二次因所选根 `checkpoint_txg` 顶格报」算相等（`.claude/kb/decisions/13-验证路线.md:33`）。挂载读法下 R2 的根在实例代号、txg、所选根、树表、映射树根、实例表指针上每个状态都与 R1 不同（第一节总表），按根比恒红；能比的只有投影。条款里与它同形的是 I-8.4 的等价关系「不比扇区、实例代号与行」（`.claude/kb/invariants.md:93`）——但 I-8.4 的对象是同一条记录施加两次，不是挂载。M3 那一格也写明「I-8.4（重放幂等） 是记录级一读，不动点是另一条性质」（`.claude/kb/decisions/13-验证路线.md:33`）。

所以「第三读」的写法会把一条按投影比的性质塞进按根比的 M3：要么 M3 的「=」被放宽成投影（两读也跟着弱），要么第三读永远红。四句：分辨「当 M3 第三读」与「独立一条」；判别子看得到；满足 T6「与 M3 不动点、I-8.4（重放幂等） 的关系（是第三读，还是独立一条）」；改法：独立一条、等价关系照第一节小结的三类写（零轮）。

### T6 里没量的

- 带介质损坏的崩溃状态：原型没造坏盘，射程那一半没量。
- 挂载途中再崩一次（崩溃注入的第三截）：s2、s6、s10 枚举的就是挂载途中的崩溃状态（R1 = 挂载途中崩了之后的只读恢复），这些状态上的结论已在上表；「挂载后再恢复」之上再套一次挂载途中崩溃（R1 → 挂载崩 → 挂载 → R2）没量。

## 第三节　T2：崩溃之后允许看到的状态集合

### T2-1　语义级允许集合放过盘级下界（量过）

原型对每个崩溃状态另记：R1 读回的内容是「枚举那几步之前」那一份、「之后」那一份、还是别的（`main.rs` 的 `semantic`）；再看盘上已持久的最新根槽是不是枚举那几步最后写出的那条根（`newest_persisted_root`）。管理员回退那段（s3，回退前 201 字节、回退后 3 000 字节）：

```
t5b-s3.log:11:CLASS SEM r1=after lower_bound=final_root_slot_persisted_and_contents_differ count=9
t5b-s3.log:12:CLASS SEM r1=after lower_bound=other count=3
t5b-s3.log:13:CLASS SEM r1=before lower_bound=other count=4096
```

那 9 个状态里回退那次发布的根槽已经持久、系统配置轮换还没做完（fsync 没返回）。三种丢法（前缀、任意子集、按发布）在这里给的允许集合都是 {回退前, 回退后}：一个读回回退前内容的恢复，在语义级判据下过，在盘级判据下红——崩溃注入的 ③「③ 盘上已持久的最新根槽是 (T, i) 时，择到的根按 (txg, 实例) 不许比它旧」（`crates/singlefs-harness/src/model.rs:2238`），层 0 的「RecoveredToAnOlderRoot,」（`crates/singlefs-checker-tier/src/crash.rs:416`）。今天的实现在这 9 个上读回的是回退后（`r1=after`），所以这是「语义级放过的一类坏实现」（择根无视已持久的最新根，C331 那一族），不是今天的实现会出的坏状态——我没造这条变异去跑，是从这 9 个状态的分类推的。

四句：
- 分不分辨臂：不分辨 T2 的三种丢法（三种在这里同一个集合，打中不分辨臂）；分辨的是 T4 的「RefFS 语义级单独判」与「RefFS 语义级 + 盘级下界一起判」。
- 判别子：RefFS 语义级只看操作与 fsync 返回，看不到根槽落没落；盘级判据看得到。属于「判别子观测不到」——要语义级判这一格，就得给它盘上的输入。
- 满足的分句：T2「某种丢法放过一个实现真会出的坏状态」（`research/prompts/_m3-formal-r4-body.md:47`）——「真会出」这半句我没证到（今天的实现不出）；按最弱读法记成「放过一类坏实现」。
- 改法：T4 写明「语义级允许集合与判据 ③ 合取，不替换」；这一改法在 9 格上中（推的：③ 的判法照代码读）。

### T2-2　「按 POSIX 操作前缀丢」与条款的崩溃原子性单位冲突（条款，推的）

D23 已定项 14：「崩溃原子性的单位是一个单元、不是一次写请求」（`.claude/kb/decisions/23-journal的角色与格式.md:389`）。D16 已定项 5：「checkpoint 的切分只关闭成员资格」，且「同一次请求切出的若干事务按其单元的 key 升序取号」（`.claude/kb/decisions/16-发布语义.md:133`）。D16 已定项 10 的攒批：「攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次。」（`.claude/kb/decisions/16-发布语义.md:233`）。三句合起来：一次写请求切出的几个一单元事务，前几个可以进 checkpoint C、后几个进 C+1；C 发布之后、C+1 发布之前崩，恢复读到的是「这次 write() 写了一半」，按条款合法。

短历史：一个文件，一次覆盖两个数据单元 u1 < u2 的 write()，之后崩（没有 fsync）。各候选的允许集合（推的，按上面三句）：

| 丢法 | 允许集合 | 个数 |
|---|---|---|
| 按 POSIX 操作前缀丢 | {旧, 新} | 2 |
| 按事务前缀丢（事务按 key 升序取号） | {旧, u1 新 u2 旧, 新} | 3 |
| 按任意子集丢 | 再加 {u1 旧 u2 新} | 4 |
| 按发布为单位丢 | 取决于切在哪：切点不是 POSIX 面上的量 | 2 或 3 |

「按操作前缀」把条款允许的第三种判坏；「任意子集」放过「后取号的先持久」这一类（D16 已定项 4「一次发布整体施加或整体不施加」与串行提交下走不到）；「按发布」的切点 RefFS 语义级看不到（判别子观测不到）。

今天的 `crates/` 走不到这一格：一次用户改动经会话发一次布（「pub fn publish_user_change<Device: BlockDevice>(」，`crates/singlefs-core/src/mounted_session.rs:243`），顺序写整份内容也是一次发布（「pub fn publish_sequential_write<Device: BlockDevice>(」，`crates/singlefs-core/src/transaction.rs:4800`），没有攒批。所以这是一条「条款与今天的实现给出不同允许集合」的岔路：按今天的实现写 RefFS 取 2，按条款写取 3。正文判据第 2 条（`research/prompts/_m3-formal-r4-body.md:54`）在这里成立，交用户时要带代价数——我没算。

四句：分辨「按操作前缀」与「按事务前缀」两条臂；判别子：RefFS 看得到写请求的字节范围，要知道单元边界（32 KiB）与 key 序才能算「按事务前缀」——单元宽度是格式常量，D13 已定项 5 许可共享，但这就不再是「没有盘上格式」；满足 T2「最后一次持久点之后的操作按前缀丢（只允许丢一个后缀）、按任意子集丢、还是按发布为单位丢」（`research/prompts/_m3-formal-r4-body.md:28`）三选一里的前两个；改法：RefFS 的操作粒度取「一单元事务」并把单元宽度当共享常量（零轮）。

### T2-3　今天的实现上三种丢法造不出不同的允许集合（量过）

六段历史 65 719 个状态，R1 读回的内容没有一个落在「之前」「之后」之外（`grep -h '^CLASS SEM' t5b-*.log | grep -c neither` 输出 0）。原因：今天每个用户入口返回之前就持久，一步里 POSIX 可见的改动至多一次发布。所以「在一段短历史上给出不同允许集合」（正文判据第 2 条）在今天的 `crates/` 上取不到样本：凡要分出三种丢法的历史，都要攒批或并行提交，二者今天都没有。正推腿若在今天的实现上量「三种丢法的允许集合大小」，量出来只会相等。

目录操作（rename、unlink）与数据写的相对次序：今天 core 里没有 rename、unlink 的发布入口（`grep -rlnw 'rename\|unlink' crates/singlefs-core/src/` 只命中 `records.rs` 与 `transaction.rs` 里的注释与「删除中」的拒绝判定），这一半没有可比的实现，条款写什么今天都打不中也证不了。

## 第四节　T1：fsync 之后什么算持久（推的）

- 今天没有「不持久就返回」的入口：发布入口返回之前已持久，D16 已定项 7「fsync 等系统配置轮换持久之后才返回」（`.claude/kb/decisions/16-发布语义.md:179`）。T1 的三种读法（只有 f、f 加所在目录项、那一刻之前的全部操作）在任何可达历史上给出同一允许集合（T2-3 的量是同一件事）。判别子在今天的实现上观测不到——这一格在今天的 `crates/` 上没有能打中或证实的量。
- 条款面上取「全部操作」：fsync 是「提前触发一次发布」（`.claude/kb/decisions/16-发布语义.md:249`），一次发布「一次发布整体施加或整体不施加」（`.claude/kb/decisions/16-发布语义.md:111`），提前发布带走开放 checkpoint 里已取号的全部事务。「只有 f」那一读比实现宽：它放过「fsync(f) 返回之后崩、更早的 g 的写丢了而 f 在」这一类坏实现。但「全部操作」也要带射程：取号是事务进 checkpoint 那一刻（D16 已定项 5，第 133 行），串行提交下 fsync 之前的操作都已取号；并行提交（D13 已定项 2 将来那条）下另一线程还没取号的操作不在里面。
- 攻「fsync 返回 ⇒ 之后崩溃必在」这句单调性：D16 已定项 17「这次发布的数据可以出现也可以不出现」（`.claude/kb/decisions/16-发布语义.md:336`）。fsync 报错的那一批，在之后任何一次崩溃里都可以在也可以不在（实例切换会重发它），直到下一次 fsync 成功。RefFS 若只有「返回 / 没返回」两种 fsync 结局，判不了这一格；要第三种结局「报错」，其后允许集合里那一批「在 / 不在」两可。四句：分辨「两种结局」与「三种结局」；判别子：RefFS 看得到 fsync 的返回值，看得到；满足 T1「fsync(f) 返回之后哪些东西崩溃之后必须还在」（正文第 25 行）；改法：允许集合按 fsync 的三种结局分（零轮）。今天的实现里报错那一支（实例切换）还没落地，量不到。

## 第五节　T3：RefFS 与实现、与 O1 的独立性（代码，推的）

- 「RefFS 由 O1 扩成」与「语义级、没有盘上格式」互相冲突。O1 今天不只记 POSIX 内容：它记根环落点（「pub struct ModelRingPosition {」，`crates/singlefs-harness/src/model.rs:235`）、实例表的行、F、候选集；它的崩溃判定用的是盘上已持久的最新根槽（第 2238 行那条 ③）。从 O1 扩出 RefFS 语义级，要么把这些拿掉（T2-1 那 9 个状态那一类就没人判），要么留着（就不再「没有盘上格式」）。条款原句：D17 已定项 4「扩成的 RefFS 定义 fsync 的持久性承诺」（`.claude/kb/decisions/17-实现分层与第三方管道.md:69`）；D13 已定项 3「RefFS 与实现不许共用崩溃点枚举的那段代码」（`.claude/kb/decisions/13-验证路线.md:65`）。
- 「那段代码」的边界：RefFS 语义级的崩溃时刻是「第 k 个操作在飞」。把一个盘级崩溃状态对到「第 k 个操作在飞」要靠录制流里每一步结束时的写数（崩溃注入里 `StreamMarks` 那一份，我的原型用的是 `execute_history_with` 的观察者在每步之后取 `stream.operation_count()`）。这一步只用操作边界，不用切段与子集；若「不许共用」读成连这一步也不许，RefFS 与实现就对不上同一个崩溃状态，T4 无从比。条款没写这一步属不属于「那段代码」。
- 回退候选集、F 是管理员可见的状态（回退被拒不被拒），不是 POSIX 状态。RefFS 只管 POSIX 状态，这一半要么留在 O1，要么 RefFS 也记（又回到盘上概念）。

## 第六节　T4：RefFS 怎么接到验证上（没打中 / 推的）

- 今天的可达历史上，RefFS 语义级能判的（读回内容 ∈ {之前, 之后}）是崩溃注入判据 ② ④ 已判的一个子集（判据 ② 要「模型提交过的某一版」、④ 要内容逐字节同），再弱于 ③（T2-1）。所以今天把它接进崩溃注入，判不出 O1 加 ③ 判不出的东西；它的边际价值要等攒批或并行提交落地。我没在今天的实现上找到反例，记「没打中」。
- C191 第一对「O3 对 RefFS」：O3（Y1 的几种形态）判的是一个盘级崩溃状态的择根与取前缀，交出的是根或版本；RefFS 交出的是 POSIX 内容的允许集合。两者要比，O3 得再走读到 POSIX 内容（走读那一截今天只有实现与 checker 有），或 RefFS 得交出版本号（又回到盘上概念）。正文 T4 格没交代比的对象是哪个。推的，没量。

## 改法一览（各修哪一格）

全部「只在我的模型上量过、被攻过零轮」，或推的。

| 改法 | 修的格 | 在打中的格上还中不中 | 量过 / 推的 |
|---|---|---|---|
| 卸载记号、F 按动作条件「必须相等」 | T5-1 | 中：65 634 个状态全同、带卸载的 65 591 个全变 | 量过 |
| 「已确认的发布」写成读法 B（挂载后发布之前那一版的内容 = R1 内容） | T5-2、T5-3 | 中：`mount+write` 下全同 | 量过 |
| 回退候选集写成「R1 ⊆ R2，卸载、抬 F、根环转满之外」 | T5-2 | 小盘上中；4 GiB 盘根环转满而 F 不抬时不中 | 小盘量过，4 GiB 推的 |
| 实例表的行写成 T5-4 的关系 | T5-4 | 前半中（R1 那一行与前缀），中间烧号那几行的值没核 | 前半量过 |
| T6 射程写成「可写挂载做成的状态」，被拒的单列计数 | T6-1 | 中：85 个被拒状态全是第三态一类 | 量过 |
| 「被拒算相等」 | T6-1 | 不起作用：没有 R2 | 推的 |
| 挂载读法独立一条，不当 M3 第三读 | T6-2 | 中 | 推的（条款） |
| T4 写明语义级与判据 ③ 合取 | T2-1 | 中：9 个状态 | 推的 |
| RefFS 操作粒度取一单元事务、单元宽度当共享常量 | T2-2 | 条款面上中；今天的实现走不到 | 推的 |
| fsync 三种结局 | T1 | 条款面上中；今天的实现走不到 | 推的 |

## 没打中的形状

- T5：在挂载做成的 65 634 个状态上找「R1 读回内容与 R2（`mount`、`mount+mount`、`mount+unmount`）读回内容不同」的状态——0 个；找「R2 中挂载后发布之前那一版的内容 ≠ R1」——0 个；找「F、卸载记号在不含卸载的动作下变了」——0 个。取样：六段手写历史（表在「复跑」一节），层 0 枚举域全量（每段展开、第三态），四种动作。
- T5：R1 由记录重建（根槽没落、记录落了）之后挂载，R2 是否丢掉那一版的内容——这类状态 3–6 个一段，内容全同，没打中。
- T2：在今天的实现上找「R1 读回的内容既不是之前也不是之后」——65 719 个状态里 0 个。
- T6：找「没有第三态而挂载被拒」的状态——0 个（拒的全是第三态一类）。
- T4：没找到 RefFS 语义级判得出而 O1 加 ③ 判不出的状态。
- 一次抽样的没打中不算没打中（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」）：上面几条要支撑结论，需同立场再派一条。

## 这条腿自己的限度

- **盘宽缩了**：原型用 `HistoryDeviceWidth::UnitAreaOf384Slots`（journal 环 6 MiB），不是快档的 4 GiB。缩前缩后：4 GiB 上 s2 的状态数 327 711、先跑的 500 个用了 145.5 秒（10 线程）、按此推全量约 26 小时；小盘上 s2 20 511 个、500 个 2.3 秒，全量实跑 130.9 秒。估时算法：全量秒数 = 小段秒数 × 全量状态数 ÷ 小段状态数。小盘上推过抬 F（s10 的 F = 20），这改变了根环转满时候选集的结局（T5-2 读法 A 那一半只能推）。4 GiB 盘上的估时日志 `est-s2-500.log` 在草稿目录，没入库。
- **历史缩了**：小盘上一次带文件的发布有一个 20 次写的单元段（2^20 ≈ 10^6 个状态，`sizes-small-h1.log`；4 GiB 上是 24 次写、2^24），所以枚举段里含覆盖写或第一个文件的历史（h0、h1、h6，各约 1.05×10^6、2.1×10^6、3.1×10^6）一段都没跑，覆盖写那次发布中途崩溃之后的挂载没量；s6 在 4 GiB 上 4 259 871 个、也没跑。只跑了枚举段里没有带文件发布的六段，每段的崩溃状态一个不落。
- **状态数合计**：第一遍 s1、s5、s3、s2、s6 共 45 208；第二遍六段共 65 719；估时三次（4 GiB 上 500 个、4 GiB 上一次 2 000 个跑到一半被我停掉、小盘上 500 个）不超过 3 000。合计不超过 113 927，在 10^7 之内。
- **动作只扫了四种**：挂载之后的回退、抬 F、连发多次没扫；「挂载后再恢复」之前再插一次挂载途中崩溃没扫。
- 原型没造坏盘，T6「带介质损坏的」那一半没量。
- R1 用的是我在原型里按 `recover` 同一串调用（择系统配置、择根、扫环、`replay_journal`）另取的根记录，不是改 `recover` 的返回值；两者走的是同一批函数。
- 副本上的数不进 kb；主 agent 要引，按 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」在入库装置上重做。

## 没做什么

- 不判正推、辩方那几格；没读禁读清单里的文件。
- 没跑 checker 档的测试与门禁；原型是拷贝里新加的 crate，经 `cargo build -p r4probe` 编、直接执行编出的二进制（都经内存包装）。
- T2-2 的代价数、T1 第三种结局在实现上的样子没量（今天的实现走不到）。
- 草稿目录 `/tmp/claude-1000/m3-formal-r4-opus/` 里的仓副本 `tree/`（12M）与编译目录 `target/`（515M）交回前已删；留下的是几份估时与第一遍日志（`t5-small-*.log`、`est-*.log`、`sizes-*.log`，合计 2.2M），没入库：第一遍日志被第二遍（多两个字段）取代，估时日志只支撑「限度」一节的估时。复跑用模型目录的 `rerun.sh` 从冻结副本重建。
