# m2-wave3-code-r1 云端攻方腿（Opus）：Y1、Y3、Y4

写于 2026-09-24（UTC 00:47 开工、01:45 前后落盘；JST 09:47 / 10:45）。副本在 `/tmp/claude-1000/m2-wave3-opus/`，开工时对 `research/prompts/m2-wave3-code-r1-snapshot/sha256sums.txt` 逐文件核过（不 OK 0 条）。**这里所有数都是副本上的数**，不是入库装置上的数。

## 复跑

```
bash research/prompts/m2-wave3-code-r1-opus-model/run.sh "$(pwd)" /tmp/claude-1000/<新草稿目录>
```

它把仓拷成副本、把两份驱动（`opus_attack_y1.rs`、`opus_attack_y3.rs`）放进 `crates/singlefs-harness/tests/`，再建三个改过的副本（分配器变异 `mutY4-*.diff`、改法 D `fixD-walk.diff`、改法 E `fixE-walk.diff`）各跑各的；日志写进草稿目录，与模型目录 `logs/` 下的同名文件比。驱动只打印、不断言。`logs/y1-run1.log` 与 `logs/y3-run1.log` 是在驱动两处无关改动之前跑的（`type Dev` 改成 `pub`；`opus_attack_y3.rs` 后来追加了两个测试函数），被测的函数体没变。

模型目录每个文件的 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里跑）：

```
6d24a6f48e1aa988d0745db5696d09c374c77a102254ccad215929bc5b0f0566  ./fixD-walk.diff
dffc1774121195c7464ad818951af085b88011deb012052b702659d61a1091bb  ./fixE-walk.diff
ea18bb3586f8f71604076a06cf0e31df0f13b18ca475cd2e4416d5fe7bc28713  ./logs/build.log
00e87f5761421494e67d24f777782d44e4ece10a66414040f0de052f951fc5a2  ./logs/fixD-m356.log
77c4ed2ac46c3767238eb38abe4413cc04c3a2a687b1d4bf81ea9288317eebb1  ./logs/fixD-tests.log
b5769c4877e6012731cfdf8b603cf7cd28fdd04c20c56fba60b6bce437e4180e  ./logs/fixD-y1.log
f6e8fd4a9071ec9da2ebad24839aea2c0b8b808a22cd0edb35997f22cffc6364  ./logs/fixE-mutY4.log
71b5ee7c8a7ac5bd677ffceb7c1c264ea4230ad4dd2c9a0a3b3fb1751e143889  ./logs/fixE-y3.log
4fb647ac5eeeb5e0a8c19806f6ecce1c65d08456c092a0cead1ce2d124807dd4  ./logs/mutY4-run2.log
1c57b6d724cd534e886f73e470bf7ce781b94bdbc9baae117856da688a6a8474  ./logs/mutY4-run.log
8a428ad7ed0a7a93c9d755d7a3d7661a8ec19776e6c82a8959802ab89c676152  ./logs/y1-run1.log
6dbc8f5da5e28d8ee1641c31403b12d05e49e329c4ace2db5282717635463b28  ./logs/y3-formatted.log
c7fcb6f8d62c8d42f80017cbdef60f12a497dca7bf9aecc2fba04629880e27c2  ./logs/y3-run1.log
bf435a016b1751da96fd971bd11a59e74cd973347824f5a9607a01290d7c8f31  ./mutY4-allocator.diff
5cfeae8bb772d19cf0c5546b1c09a4f4ce6fc25f9295c7cb4ab1c2dcf4f31256  ./mutY4-driver.diff
f574990622f719ba631b3d244270d3f5f11d0fba9a60d82a82e76e41a4e65902  ./opus_attack_y1.rs
f42b24d163afda56802a4a0b0f80272f9d8cde30192e61b0a6511cc236a7c242  ./opus_attack_y3.rs
1685da02613ee74bb5a0e4e9275808737c72af0a0e6e7e6e1eae84ede31a5440  ./run.sh
```

## 各格判定一览

| 格 | 子问 | 判定 | 结论种类 | 依据 |
|---|---|---|---|---|
| Y1 | I-7.8 在合法状态上红 | **打中**（Y1-a）：第一个文件版本那次发布崩在记录落盘之前 → 可写挂载 → I-7.8 红；两条流共 130 格（mkfs 流 11 前缀 × 5 动作 + 回退流 15 前缀 × 5 动作） | 和条款说反话（checker 代码 vs I-7.8 的 checker 读法注「孤儿节点不算出现过」） | 副本上跑出；**不分辨** watermark 补丁的三样选择 |
| Y1 | 号被重发 | 没打中 | — | 48 格重发的都是没发布过的号 |
| Y1 | 瞬时读错拉低水位 | 推测，没跑 | — | — |
| Y1 | 树表条数 vs 树建没建 | 没打中 | — | — |
| Y1 | 两个新错误成员 | 没验 | — | 合法历史走不到 u64 那一个 |
| Y1 | `publish_version` 的 `assert_eq!` | 产品路径走不到 | — | 调用点全带 `Some` |
| Y3 | 四条里哪条是字面后果 | ② ③ 兑现，① ④ 是选择 | 替没写的条款做了选择（① ④） | ① 的另一读法在 Y4-a 那三个状态上分得开 |
| Y3 | ④ 不成立而单元没回收 → 误红 | 没打中 | — | ④ 翻面只在根环转圈之后，与已知红 ② 重合 |
| Y3 | C533 两格 | 没打中 | — | 119 格全绿 |
| Y4 | 收窄是不是条文后果 | 不是：与「任一」相反，与射程 ④ 一致 | 替没写的条款做了选择 | 射程 ④ 是 2026-09-23 补的、被攻过零轮 |
| Y4 | 候选集之外一条写错的记录今天判不出、之后被当现行用 | **打中**（Y4-a）：B 的记录已落、根槽没落的 3 个状态，I-3.10 在崩溃镜像上成立，挂载之后红 | 替没写的条款做了选择（两读法可达历史上分得开） | 副本上跑出（配一条分配器变异） |

自提改法（都只在我的模型上量过、被攻过零轮）：D（I-7.8 扫描方向按实例表行排除崩溃孤儿）把 Y1-a 的 130 格全变绿、变异 356 点名的用例仍红；E（I-3.10 另读下一次挂载要施加的那一版）把 Y4-a 的 3 格变红、不带变异的 Y3 两条扫描 347 行判定逐行不变。

## Y1　树 ID 水位的取法与发号次序

### Y1-a　打中：第一个文件版本那次发布崩在记录落盘之前，重开可写挂载之后 I-7.8 在合法状态上判红

**历史**（写序列 + 故障点 + 期望读数），两条流各一份，驱动 `opus_attack_y1.rs`：

| 步 | mkfs 那条流 | 回退之后那条流 | 许可它的那一句 |
|---|---|---|---|
| 1 | mkfs → 取号 1 → 暖机 (1,1)(1,2) | 同左，再发第一个文件版本 (1,3)（树 11..18、水位 19），关掉，`mount_rollback` 到暖机根 (1,2)（回退行那次发布带环里 max 19） | 回退到树表 0 条的一版照常做（C511 第 3 步） |
| 2 | 在 (1,2) 上 `publish_first_file`（txg 3，八棵树从水位 11 发 11..18） | 在回退之后那一版上 `publish_first_file`（八棵树从 19 发 19..26） | `crates/singlefs-core/src/transaction.rs` 的 `publish_first_file` |
| 3（故障） | 录制流取前缀 k=44..54：至少一个码 2 单元（extent 11、inode 12、分配 13、记账 14、映射 15 其中几个）已落盘，记录没落盘 | 前缀 k=84..98，同形（码 2 单元树 ID 19..23） | 断电在发布中间 |
| 4 | `mount_writable`：新实例第一次发布的 txg = max(环里根的 txg, 环里记录的 txg) + 1 = 3，与孤儿节点的诞生代号相等；写行那次发布只写实例表与分配记录那两片，孤儿节点有留在盘上的 | 同形：新根 (3,7)，水位 19 | D23 已定项 14 第 3 条（见下引文） |
| 5（放开扫） | 挂载之后什么都不做 / 连推 1、2、3 次零单元发布 / 再关掉重开一次 / 发第一个文件版本，六种 | 同左 | 用户决定的动作，按规则放开 |
| 读数 | 前五种动作全部只红 I-7.8：`根环水位最大 11，盘上出现过的最大树 ID 12`（随 k 升到 15）；第六种（发第一个文件版本）不红 | 前五种全部只红 I-7.8：`根环水位最大 19，盘上出现过的最大树 ID 19`（随 k 升到 23）；第六种不红 | — |

计数（副本上量的，`y1-run1.log` 按流、动作、红没红数出来；命令与原样输出见附录 A）：mkfs 那条流 24 个前缀 × 前五种动作，每种动作红 11 个前缀；回退之后那条流 24 个前缀 × 前五种动作，每种动作红 15 个前缀；第六种动作两条流 24 个前缀全绿。红的只有 I-7.8 一条。

**为什么是合法状态**：孤儿节点从没被任何根或记录发布过；它们的树 ID 被新实例在第六种动作里重发（mkfs 流发回 11..18、回退流发回 19..26），而条款禁止的是「已发布的树」的号被重发：

> `.claude/kb/decisions/08-核心索引结构.md:240`（第 ② 项；这里只标出承重的那一句，整行原样见附录 D）：「⚠️ **「累计」这两个字是承重的**：读成「本 checkpoint 发出的最高号」（不累计）时，树 ID 会随根轮出环被重发，而**那时被重发的号属于已发布的树**，D6（快照实现模型） 判据 9 正面命中 ⇒ 整条定案不成立。」

checker 这一格的读法本来就要把这种孤儿排除在外（用户 2026-09-14 定甲）：

> `.claude/kb/invariants.md:58` I-7.8 状态列的读法注（标出承重的一句，整行原样见附录 D）：「扫描方向只数诞生代号不超过根环里最大 checkpoint_txg 的码 2 节点——崩在发布之前的那个事务写出的孤儿节点不算「出现过」」

而新实例会重用孤儿节点的 txg：

> `.claude/kb/decisions/23-journal的角色与格式.md:362`（标出承重的一句，整行原样见附录 D）：「新实例（普通挂载、切换、回退）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」

实现取的是等号（`crates/singlefs-core/src/mount.rs:317` `CheckpointTxg(highest_ring_txg.max(highest_record_txg).0 + 1)`）。记录没落盘时 max 里没有孤儿那一版的 txg，新实例就发出同一个 txg。

**差异在哪个字节上**：checker 的过滤只看诞生代号（`crates/singlefs-checker/src/walk.rs:877` `&& read_u64(&bytes, 52 + 2 * usize::from(bytes[51])) <= newest_published_txg`），不看头里写序的实例代号。mkfs 流上孤儿节点头里写序实例代号是 1、诞生代号 3；挂载之后最新根是 (2,4)，实例表里有行 (1, 2, …)——实例 1 被施加到 txg 2 为止，诞生代号 3 > 2 的实例 1 单元就是没发布过的。判定它的东西全在盘上，checker 看得到；同一个判据 D8 那一节也写过：

> `.claude/kb/decisions/08-核心索引结构.md:203`（标出承重的一句，整行原样见附录 D；同一句在这份文件里命中 2 次，取第 203 行那一次）：「孤儿与活版的实例代号必不相等（D18（块里携带什么信息） 已定项 11「作废一个 checkpoint 必须换实例代号」）」

**结论：和条款说反话**——checker 的代码（`scanned_tree_identifiers`）与它自己那条读法注（孤儿不算「出现过」）在上面这段可达历史上说反话；差异在孤儿码 2 节点头的写序实例代号那 4 字节上看得出来。不是 watermark 补丁引入的：mkfs 那条流上水位恒 11、补丁的三样选择都不起作用，照样红。

**打中之后先答四句**：

1. **分不分辨臂**：Y1 这一格被判的是 watermark 补丁的三样选择（读不出的根拿记录顶、被抛弃时间线也算、按常量次序连号发）。这一打中在 mkfs 流上也中，三样选择都碰不到它 ⇒ **不分辨这三条臂**，病根在它们共用的 checker 读法上，应另立一笔账，不拿它判 Y1 的三样选择。
2. **被判系统当时看不看得到判别它的东西**：看得到。checker 已经解出每个码 2 头的写序实例代号（同文件 `unit_write_order_instance`，I-7.7 那一格在用），也已经解出最新根指着的实例表行（`walk.instance_table_rows`）。
3. **满足判据字面的哪一个分句**：跑前判据第 2 条「指出了具体是哪个字节或哪条可达历史上看得出差异」——可达历史是上表，字节是孤儿节点头的写序实例代号。归的是 I-7.8 这一条，不是 I-7.7 或 I-1.2。
4. **跑前条款给的改法在打中的格上还中不中**：正文第五节没有为 Y1 给改法。仓里已有的两条相关变异（`crates/mutations.tsv` 第 356、357 行，回退行水位不取环 max）钉的是另一格，与这一打中无关。

**我提的改法 D**（只在我的模型上量过、被攻过零轮）：I-7.8 的扫描方向再排除「写序实例 i 在最新根的实例表里有一条非回退行 (i, Ti, ·)，Ti > 0，且诞生代号 > Ti」的码 2 节点。回退行不排除（被抛弃时间线发布过的号仍要算，否则 C511 的判别力那一格就没了）。补丁 `fixD-walk.diff`。量到的见附录 B：

| 格 | 今天 | 改法 D | 标 |
|---|---|---|---|
| Y1-a mkfs 流 11 个前缀 × 前五种动作 | 55 格红 I-7.8 | 0 格红（两条流 2 × 24 前缀 × 6 动作 = 288 格全绿） | 量过（副本 `repo-fixD`，`fixD-y1.log`） |
| Y1-a 回退流 15 个前缀 × 前五种动作 | 75 格红 I-7.8 | 0 格红（同上） | 量过（副本） |
| 变异 356（回退行水位不取环 max）点名的用例 | 红 | 仍红：`回退之后推零单元发布到 (1, 3) 离开根环：池级 checker 一条违例都没有：[("I-7.8", Violated("根环水位最大 11，盘上出现过的最大树 ID 15"))]` | 量过（副本，`fixD-m356.log`） |
| `checker_known_bad_images` 全部 23 条、`second_transaction_step_four_rollback` 全部 11 条 | 绿（今天的仓） | 23 passed、11 passed | 量过（副本，`fixD-tests.log`） |
| 层 0 全量、随机历史三档 | 绿 | 没跑 | 推的：改法只多排除，不会多红；少红的只有「非回退行之上的本实例码 2 节点」，这类节点照定义没被发布过 |

**什么会推翻 Y1-a**：一段合法历史里，崩溃孤儿节点在新实例挂载之前就被别的写盖掉（那样盘上看不到孤儿）——我的扫描里 k=44..54 与 84..98 各前缀都留得下至少一个；或者新实例第一次发布的 txg 取的不是等号而是更大的数（今天代码取等号，见上）。

### Y1 其余几问

| 问 | 试了什么 | 结论 |
|---|---|---|
| 号被重发 | 两条流全部 48 个前缀上接着发第一个文件版本（动作 5）：mkfs 流发回 11..18、回退流发回 19..26，全部是孤儿节点用过、从没发布过的号；checker 全绿 | 没打中：重发的都是没发布过的号，条款（08 第 240 行）禁的是已发布的 |
| I-7.8 在合法状态上红 | 见 Y1-a | **打中**（Y1-a），但不分辨 watermark 补丁的三样选择 |
| 水位被一次瞬时读错拉低 | 只推，没跑：`tree_identifier_watermark_of_the_ring`（`mount.rs:324`）再与要接的那一版取 max；拉低要求「带最高水位的根读不出 ∧ 它那条记录已被绕环盖掉 ∧ 之后没有任何一条读得出的根带过这个水位」，而每次发布都照抄或推高水位，最高水位一旦写进一条根，后面每一条根都带它 | 推测，不算打中也不算没打中；只剩 C342 那一格（根与记录都读不出），已登记 |
| `publish_first_file` 改看树表条数之后，树表条数与「树建没建」对不上 | Y1-a 的全部状态：挂载交回的现行版都是树表 0 条，而盘上有孤儿树节点；动作 5 照常建树 | 没打中：孤儿树没发布过，不算「建了」 |
| 两个新错误成员走不走得到 | 没跑。`TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees` 要盘上水位离 `u64::MAX` 不到八个号：合法历史上水位每次最多涨 8，走不到，只有坏盘输入走得到（变异第 361 行钉着）；`TreeTableOfTheVersionToBuildOnUnreadable` 要那一版的树表单元读不出（介质错），合法历史加一个故障走得到（变异第 362 行钉着） | 没验「走到时盘上逐字节不变」，归没做 |
| `publish_version` 没有上一版时那条 `assert_eq!` 走不走得到 | `grep` 全仓 `publish_version(` 的调用点（附录 A）：`crates/*/src` 里三处（`mount.rs` 852、913、981 行）全部带 `Some(...)`，`transaction.rs` 2156、2271 两处是 `publish_overwrite` / `publish_new_inodes` 里的调用，也带 `Some(previous)`；传 `None` 的只有测试 `second_transaction_supplement_two_accounting_node_full.rs`，它给的新水位就是 `TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH` | 产品路径走不到；只有新写的调用方传 `None` 配别的水位时才会触发 |

## Y3　候选 b：由记录施加出来的那一版怎么认

四条认法在 `crates/singlefs-checker/src/walk.rs:2614`（`fn versions_applied_only_by_records(`）。

### 四条里哪条是条款的字面后果

| 条 | 条款里有没有这一句 | 判 |
|---|---|---|
| ① 带提交标记、最新根的实例表里有这个实例的行且 txg ≤ Ti | 判决 Z3 只说「由记录施加出来、根槽从没落盘」；「施加过」要有行，是实现读出来的。另一个说得通的读法是「下一次挂载会施加的那一版」（按恢复的前缀规则），两者在「记录已持久、根槽没落盘、还没挂载过」那几个状态上分得开（见 Y4-a，那三个状态上 ① 不成立） | 替没写的条款做了选择 |
| ② 根环里没有 (i, txg) 的根 | 「根槽从没落盘」的字面 | 兑现（判决原句） |
| ③ txg ≥ 最新根带的 F | 与候选根同一条，I-3.1 读法注「txg ≥ 最新根自己带的回退下界 F」 | 兑现（沿用已定读法） |
| ④ 根环里有一条没被实例表判抛弃的根，txg 比它小 | 条款没有；是实现从回收门槛推出来的充分条件 | 替没写的条款做了选择 |

### ④ 不成立而单元还没回收：扫了，没打中

历史（`opus_attack_y3.rs` 的 `y3_crash_inside_overwrite_then_mount_then_overwrites`）：mkfs → 取号 → 暖机 → A（txg 3）→ B（覆盖写，txg 4），B 那次发布的录制流每个前缀（k=60..83，24 个）崩；可写挂载；同一个会话里连续覆盖写 1..26 次，每一步判；最后关掉重开再判。

读数（附录 C）：

- 前缀 k=78、79、80（B 的记录已持久、根槽没落盘）：挂载时恢复施加了那条记录（`prefix_applied=1`），挂载之后与覆盖写 1..18 次每一步 **一条都不红**；第 19 次（最新根 txg 26）起只红 I-3.1，第 19 次那一张机理标识「并进遍历的由记录施加出来的版本 1 个、最老的自证过的根 txg 3」、记账比遍历多 16384。
- 对照（k=60，B 一个字节都没落盘，没有由记录施加出来的那一版）：第 21 次（也是最新根 txg 26）起红 I-3.1，差值同是 16384。
- ④ 由成立翻成不成立（最老的有效根 ≥ 4）只在根环转过一整圈之后发生：B 那一版之下的根只会被轮转挤出环，实例表不会判它们抛弃（它们都 ≤ Ti）。那时 I-3.1 已经落进收口表第 ② 行那一族已知红（`KNOWN_RED_FORMS` 第一条：根环转过一圈之后记账多于遍历），分不开是 ④ 造成的还是已知红。

**没打中**，取样范围：一条流 24 个前缀 × 会话内 0..26 次覆盖写 + 一次重开；没扫回退、抬 F、多实例交替。

### C533 那两格（记录新根段里没有实例表指针与「树表 0 条那一版的分配记录树根」）：扫了，没打中

历史（`y3_crash_inside_first_writable_mount_of_a_formatted_pool`）：只做过 mkfs 的池，第一次可写挂载（写行那次发布在树表 0 条的一版上写实例表与一片分配记录节点、之后暖机）的录制流每个前缀（k=21..37，17 个）崩；再可写挂载；发第一个文件版本；覆盖写 3 次；重开。k=32、33、34 三个前缀是「写行那次发布的记录已持久、根槽没落盘」（`prefix_applied=1`），正是由记录施加出来的是树表 0 条的一版那一格。

读数：17 个前缀 × 7 步 = 119 格，**0 格红**（附录 C 的命令与原样输出）。崩溃镜像上 I-3.10 报不适用（树表 0 条的一版上没有候选的分配记录树），挂载之后起判成立。恢复施加那条写行记录时照抄被施加的那条根的分配记录树根（`recovery.rs` 里那一段的注释写明「施加一条写行记录而它的根槽没落盘时，重建出来的这一版仍指着上一版的分配记录树」），那片孤儿分配记录节点与实例表单元的槽在分配器里是空闲的，第一个文件版本之后 I-3.1 与 I-3.10 都成立，checker 与分配器在这几格上没有分歧。

「那一版不在候选集里」那一格没有专门构造，归「没打中的形状」。

## Y4　I-3.10 只读回退候选集里的分配记录

读集在 `crates/singlefs-checker/src/walk.rs:1581`（`fn allocation_record_node_pointers_of_the_candidate_versions(`）：候选根的分配记录树、候选根记录直接持有的那一片、`versions_applied_only_by_records` 那几版。

### 收窄是不是条文的后果

`.claude/kb/invariants.md:136` 的 I-3.10 行（整行见附录 D）判据列写「任一**未带已释放标志**的分配记录」，射程 ④ 是 2026-09-23 主 agent 补的「只读回退候选集里那几版的分配记录……被攻过零轮」。代码与射程 ④ 字面一致；与判据列的「任一」不一致。④ 本身不是从别的已定分项推出来的，是一次选择 ⇒ **替没写的条款做了选择**（两个都说得通的读法见下，Y4-a 给出分得开的历史）。

### Y4-a　打中：一条会被下一次挂载当成现行记录用的分配记录，崩溃镜像上判不出

**历史**（`y4_residual_record_version_then_mount`，副本 `repo-mutY4`）：与 Y3 同一条流（mkfs → 取号 → 暖机 → A txg 3 → B txg 4）。**被检的缺陷**是分配器副本上的一条变异：只在发布 B 那一次，新记的未释放分配记录的分配代写成 txg + 1 = 5（`mutY4-allocator.diff`，开关由驱动在 B 前后开关，`mutY4-driver.diff`）——这正是 I-3.10 那一行「它拦的是什么」说的那一类「记录里的代与单元头对不上」。故障：B 的录制流取前缀。

| 前缀 | 盘上是什么 | 恢复会做什么 | 崩溃镜像上 I-3.10 | 挂载之后 |
|---|---|---|---|---|
| k=60..77 | B 的单元有的落了，记录没落 | 不施加（`prefix_applied=0`） | 成立 | 成立 |
| **k=78、79、80** | B 的记录已落（k=78 只落盘 0 那一份、k=79 两盘都落、k=80 过了屏障），根槽没落 | **施加 B 的记录**（`prefix_applied=1`），B 那一版成为现行版 | **成立**（判了别的记录，B 那棵分配记录树没读） | **红**：`盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4` |
| k=81..83 | B 的根槽 FUA 已落 | 择 B 的根 | 红（同一条记录） | 红 |

原样输出见附录 E。k=78..80 三个状态上，被判的记录属于下一次挂载一定会施加的那一版，恢复之后它就是现行分配器的账；checker 在崩溃镜像上看不到它，要等挂载把它写进一条候选根之后才判得出。

**两个都说得通的选择**：(a) 今天：只读「已被某次恢复施加过」的那几版（候选 b 的 ①）；(b) 另读「下一次挂载会施加」的那一版（同实例、接在最新根后面、带提交标记、根槽不在环里）。二者在 k=78..80 上分得开：(a) 成立、(b) 红。

**打中之后先答四句**：

1. **分不分辨臂**：分辨。「任一」那一臂读全盘能读到的分配记录树，k=78..80 也红；(b) 红；只有今天的 ④ 在这三格上成立。
2. **被判系统当时看不看得到判别它的东西**：看得到。checker 已经扫出全部 journal 记录（`scanned_journal_records_by_device`），带实例、txg、提交标记与新根段；最新根的 (实例, txg) 也在手里。
3. **满足判据字面的哪一个分句**：跑前判据第 3 条「说得出两个都说得通的不同选择，而且它们在……可达历史上分得开」。不是第 2 条：代码与射程 ④ 字面一致，与「任一」相反，而「任一」已被 ④ 收窄。
4. **改法在打中的格上还中不中**：正文没给 Y4 的改法。我提的改法 E 见下。

**判别力的代价有多大**：同一个变异在 k≥81 的状态上照样红，所以「这条变异能不能被层 0 / 崩溃注入抓到」不受影响；受影响的是**单个状态**的判定——正好落在「记录已持久、根槽没落」那几个状态上的缺陷，要等下一次挂载之后才露出来。k=78..80 在这条流 24 个前缀里占 3 个。

**我提的改法 E**（只在我的模型上量过、被攻过零轮）：I-3.10 的读集再加「同实例、txg = 最新根 txg + 1、带提交标记、根环里没有它的根」的那条记录新根段里的树表，从那里找种类 3 的分配记录树（`fixE-walk.diff`）。只接一步，不复刻恢复的整条前缀规则（断链、回退行的 W、校验点名单元都没有照做），这是它的已知缺口。

量到的（附录 E）：

| 格 | 今天 | 改法 E | 标 |
|---|---|---|---|
| Y4-a k=78、79、80 崩溃镜像，带 B 那一次的分配代变异 | I-3.10 成立（漏） | I-3.10 红：`盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4` | 量过（副本 `repo-fixE-mut`） |
| 同一流 k=60..77、81..83，带变异 | 同左表 | 与今天逐行相同（k=60..77 成立、k≥81 红） | 量过（副本） |
| 不带变异的 Y3 两条扫描（覆盖写那条 24 前缀 × 挂载 / 覆盖写 / 重开，mkfs 池第一次挂载那条 17 前缀 × 7 步） | 347 行判定 | 347 行逐行相同（`diff` 空，附录 E） | 量过（副本 `repo-fixE`） |
| 层 0 全量（含「链接得到残留记录」那 19 个状态）、随机历史、崩溃注入、`checker_known_bad_images` | — | 没跑 | 推的：残留记录那 19 个状态若接在最新根后面，会被读进来；它们的分配记录与单元是同一次发布写的，推它不红，没量 |
| 链上第二条及以后（恢复会连施加几条） | 漏 | 仍漏 | 推的：改法 E 只接一步 |

**什么会推翻 Y4-a**：恢复在 k=78..80 上不施加 B 的记录（今天 `prefix_applied=1`，见附录 E）；或者 B 那一版的分配记录在恢复之后不进现行分配器（今天挂载之后那条记录照样在，判红的就是它）。

## 没打中的形状

| 格 | 形状 | 取样范围 | 读数 |
|---|---|---|---|
| Y1 | 崩在第一个文件版本那次发布里、重开之后再发第一个文件版本，看号重不重发已发布的 | 2 条流 × 24 前缀 × 动作 5 | 48 格全绿；重发的只有孤儿用过的号 |
| Y1 | 崩在第一个文件版本那次发布里、记录已持久或根已落 | mkfs 流 k=55..60、回退流 k=99..104 × 6 种动作 | 全绿（恢复从记录重建那一版，水位 19 / 27 跟着回来） |
| Y1 | 瞬时读错把水位拉低 | 只推 | 推测（见 Y1 其余几问） |
| Y3 | ④ 不成立而单元没回收 | 覆盖写流 24 前缀 × 会话内 0..26 次覆盖写 + 重开 | ④ 翻面只在环转一圈之后，与已知红 ② 同一时刻，分不开 |
| Y3 | ④ 成立而多并一版把真泄漏藏住 | 没专门造泄漏；覆盖写流上 k=78..80 挂载之后到环转之前一格都不红，干净 | 没打中；泄漏那一半只看了实现员已有的用例名，没复跑 |
| Y3 | 由记录施加出来的是树表 0 条的一版（C533） | mkfs 池第一次挂载 17 前缀 × 7 步 | 119 格全绿 |
| Y3 | 「那一版不在候选集里」（被回退抛弃、低于 F） | 没构造 | — |
| Y4 | 候选集之外（被实例表判抛弃的根、低于 F 的根）有一条分配代写错、之后被回退当现行用 | 只推：回退只能落在候选集里的根上（`mount_rollback` 拒绝被抛弃时间线与不在环里的根，用例 `rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused`），低于 F 的根不在回退候选集 | 推测没有这条路；没跑 |

「没打中」按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」那一节只算一次观测，不能拿去支撑「没有」。我的扫描是确定性的（写死的流、按前缀全取），重跑同一份代码读数不变，但只覆盖上表写的那几段历史。

## 这条腿自己的限度

- 全部数都是在副本上量的（`/tmp/claude-1000/m2-wave3-opus/repo*`，开工时与 `research/prompts/m2-wave3-code-r1-snapshot/sha256sums.txt` 逐文件核过：`sha256sum -c` 不 OK 的 0 条），不是入库装置上的数；要引，主 agent 在入库装置上重做。
- 崩溃点按录制流的**前缀**取（前 k 步整段持久），没取「段内任意子集」那一类（后发的写先持久）；层 0 与崩溃注入那种枚举我没接。前缀是子集的特例，打中的那几格用前缀就够，没打中的那几行只覆盖前缀。
- 挂载之后的用户动作放开扫了六种（Y1）与「0..26 次覆盖写 + 重开」（Y3），没扫回退、抬 F、建 inode、第二个文件。
- 改法 D、E 只在我的模型上量过、被攻过零轮；层 0 全量、随机历史三档、崩溃注入三档、`crates/mutations.tsv` 全表都没在改法副本上跑。
- Y4-a 用了一条我自己写的分配器变异来当「被检的缺陷」；它只在发布 B 那一次开（驱动里开关），不在 `crates/mutations.tsv` 里。
- 没跑门禁（`stage-owners.tsv` 里没有登记给 three-way-attack 的阶段，命令与空输出见附录 A）。
- 机器上同时有别的实现员在编：开工 `ps` 看到 9 条 `cargo test --all`、1 条 `cargo build`、1 条 `cargo run`（e156）与 1 个别人副本里的 `checker_known_bad_images` 测试进程，没有 `qemu-system`、`vm-bench.sh`、`e152`、`fio`；全部命令 `nice -n 19`、`CARGO_BUILD_JOBS=4`、自己的 target。

## 没做什么

- 不判 Y2、Y5、Y6 与逐文件测试覆盖（按分工表不归我）。
- 没把日志放进 `research/results/`（不在这条腿的写范围里），原样日志放在模型目录 `logs/` 下，主 agent 定要不要搬。
- 没改主工作区任何文件；副本与 target 留在 `/tmp/claude-1000/m2-wave3-opus/`，用完没删。
- Y1 两个新错误成员「走到时盘上逐字节不变」没验。

## 附录 A　命令与原样输出

### A.1　Y1 按流、动作、红没红计数（今天的副本 `logs/y1-run1.log`，改法 D 副本 `logs/fixD-y1.log`）

```
$ cd research/prompts/m2-wave3-code-r1-opus-model/logs && for f in y1-run1.log fixD-y1.log; do echo "== $f"; perl -ne '...计数...' $f; done
== y1-run1.log
mkfs a0 clean 13
mkfs a0 red 11
mkfs a1 clean 13
mkfs a1 red 11
mkfs a2 clean 13
mkfs a2 red 11
mkfs a3 clean 13
mkfs a3 red 11
mkfs a4 clean 13
mkfs a4 red 11
mkfs a5 clean 24
rollback a0 clean 9
rollback a0 red 15
rollback a1 clean 9
rollback a1 red 15
rollback a2 clean 9
rollback a2 red 15
rollback a3 clean 9
rollback a3 red 15
rollback a4 clean 9
rollback a4 red 15
rollback a5 clean 24
== fixD-y1.log
mkfs a0 clean 24
mkfs a1 clean 24
mkfs a2 clean 24
mkfs a3 clean 24
mkfs a4 clean 24
mkfs a5 clean 24
rollback a0 clean 24
rollback a1 clean 24
rollback a2 clean 24
rollback a3 clean 24
rollback a4 clean 24
rollback a5 clean 24
```

计数脚本全文：`perl -ne 'if(/^k=(\d+) .*action=(\d)\((.*?)\) mounted .*violations=(\d+)/){$s=($1<81)?"mkfs":"rollback"; $c{"$s a$2 ".($4>0?"red":"clean")}++} END{print "$_ $c{$_}\n" for sort keys %c}'`（k < 81 是 mkfs 流，流内第一个文件版本那次发布占 k=37..60；回退流那次占 k=81..104）。

红的那几格判红的不变量（今天的副本）：

```
$ grep "^k=" y1-run1.log | grep -v "violations=0" | grep -o "violations=[0-9]*: \[\"I-[0-9.]*" | sort | uniq -c
    130 violations=1: ["I-7.8
$ grep -E "^k=(44|54|84|98) .*action=[05]" y1-run1.log | cut -c1-260
k=84 last=write@dev0+824229888 action=0(none) mounted root=(3,7) wm=19 tree_table_empty=true violations=1: ["I-7.8: \"根环水位最大 19，盘上出现过的最大树 ID 19\""]
k=84 last=write@dev0+824229888 action=5(first file ok trees [19, 20, 21, 22, 23, 24, 25, 26] wm 27) mounted root=(3,7) wm=19 tree_table_empty=true violations=0: []
k=44 last=write@dev0+823197696 action=0(none) mounted root=(2,4) wm=11 tree_table_empty=true violations=1: ["I-7.8: \"根环水位最大 11，盘上出现过的最大树 ID 12\""]
k=44 last=write@dev0+823197696 action=5(first file ok trees [11, 12, 13, 14, 15, 16, 17, 18] wm 19) mounted root=(2,4) wm=11 tree_table_empty=true violations=0: []
k=54 last=barrier@dev0+0 action=0(none) mounted root=(2,4) wm=11 tree_table_empty=true violations=1: ["I-7.8: \"根环水位最大 11，盘上出现过的最大树 ID 15\""]
k=54 last=barrier@dev0+0 action=5(first file ok trees [11, 12, 13, 14, 15, 16, 17, 18] wm 19) mounted root=(2,4) wm=11 tree_table_empty=true violations=0: []
k=98 last=barrier@dev0+0 action=0(none) mounted root=(3,7) wm=19 tree_table_empty=true violations=1: ["I-7.8: \"根环水位最大 19，盘上出现过的最大树 ID 23\""]
k=98 last=barrier@dev0+0 action=5(first file ok trees [19, 20, 21, 22, 23, 24, 25, 26] wm 27) mounted root=(3,7) wm=19 tree_table_empty=true violations=0: []
```

### A.2　`publish_version(` 的调用点、登记给这条腿的门禁阶段、快照核对（主工作区上现跑）

```
$ grep -rn "publish_version(" --include=*.rs crates | grep -v "fn publish_version"
crates/singlefs-core/src/mount.rs:852:        let next = publish_version(
crates/singlefs-core/src/mount.rs:913:    publish_version(
crates/singlefs-core/src/mount.rs:981:            publish_version(pool, allocator, plan, Some(current_file_version))
crates/singlefs-harness/tests/second_transaction_supplement_two_row_publish_admission.rs:76:    let output = publish_version(
crates/singlefs-core/src/transaction.rs:2156:    publish_version(
crates/singlefs-core/src/transaction.rs:2271:    publish_version(
crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs:457:    publish_version(
crates/singlefs-harness/tests/second_transaction_supplement_two_accounting_node_full.rs:69:    publish_version(
crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:421:    publish_version(
crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:1387:            publish_version(
$ awk ... me=three-way-attack ... .claude/gate.d/stage-owners.tsv   # 共用约束「门禁」一节那条命令
（空：没有登记给 three-way-attack 的阶段）
$ sha256sum -c research/prompts/m2-wave3-code-r1-snapshot/sha256sums.txt | grep -vc ": OK$"
2
```

快照核对不 OK 的两条是 `.claude/kb/checks-owed.md` 与 `.claude/kb/milestone/02-second-txn.md`（开工之后别的会话改的；`crates/` 下 0 条变，这份报告引的代码行号都在副本与主工作区上一致）。开工时（00:47 UTC）同一条命令是 0。

## 附录 B　改法 D 副本上的测试

```
$ grep -E '^test result|^exit|FAILED' research/prompts/m2-wave3-code-r1-opus-model/logs/fixD-tests.log research/prompts/m2-wave3-code-r1-opus-model/logs/fixD-m356.log
research/prompts/m2-wave3-code-r1-opus-model/logs/fixD-m356.log:test after_rolling_back_to_a_warm_up_root_the_ring_watermark_outlives_the_file_version_roots_leaving_the_ring ... FAILED
research/prompts/m2-wave3-code-r1-opus-model/logs/fixD-m356.log:test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 201.30s
research/prompts/m2-wave3-code-r1-opus-model/logs/fixD-m356.log:exit=101
research/prompts/m2-wave3-code-r1-opus-model/logs/fixD-tests.log:test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1039.53s
research/prompts/m2-wave3-code-r1-opus-model/logs/fixD-tests.log:test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 668.32s
research/prompts/m2-wave3-code-r1-opus-model/logs/fixD-tests.log:exit=0
$ grep -F 'panicked' -A1 research/prompts/m2-wave3-code-r1-opus-model/logs/fixD-m356.log | cut -c1-300
thread 'after_rolling_back_to_a_warm_up_root_the_ring_watermark_outlives_the_file_version_roots_leaving_the_ring' (2074397) panicked at crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs:128:5:
回退之后推零单元发布到 (1, 3) 离开根环：池级 checker 一条违例都没有：[("I-7.8", Violated("根环水位最大 11，盘上出现过的最大树 ID 15"))]
```

`fixD-m356.log` 是「改法 D 之上再打变异 356」的副本：它红在变异点名的那条用例上，红的是 I-7.8，说明改法 D 没把 C511 那一格的判别力拿掉。

## 附录 C　Y3 两条扫描

```
$ for f in y3-run1.log y3-formatted.log; do echo "== $f"; grep -E '^k=' $f | perl -ne '@inv=(/\\"(I-[\d.]+):|"(I-[\d.]+):/g); @inv=grep {defined} @inv; %u=map {$_=>1} @inv; print join(",", sort keys %u) || "none"; print "\n"' | sort | uniq -c; done
== y3-run1.log
    180 I-3.1
     48 none
== y3-formatted.log
    119 none
$ grep -E '^k=(60|78) step=(mount|overwrite(1|18|19|20|21)|remount) ' research/prompts/m2-wave3-code-r1-opus-model/logs/y3-run1.log | perl -pe 's/(red=\[.{0,40}).*?(并进遍历的由记录施加出来的版本 \d+ 个).*/$1…$2/' | cut -c1-230
k=60 step=mount root=(2,5) red=[] I-3.10=Holds
k=60 step=overwrite21 txg=26 red=["I-3.1: \"盘 0：记账的已分配 Som…并进遍历的由记录施加出来的版本 0 个
k=60 step=remount ok=true red=["I-3.1: \"盘 0：记账的已分配 Som…并进遍历的由记录施加出来的版本 0 个
k=78 step=mount root=(2,7) red=[] I-3.10=Holds
k=78 step=overwrite19 txg=26 red=["I-3.1: \"盘 0：记账的已分配 Som…并进遍历的由记录施加出来的版本 1 个
k=78 step=overwrite20 txg=27 red=["I-3.1: \"盘 0：记账的已分配 Som…并进遍历的由记录施加出来的版本 0 个
k=78 step=overwrite21 txg=28 red=["I-3.1: \"盘 0：记账的已分配 Som…并进遍历的由记录施加出来的版本 0 个
k=78 step=remount ok=true red=["I-3.1: \"盘 0：记账的已分配 Som…并进遍历的由记录施加出来的版本 0 个
$ grep -o 'k=[0-9]* last=[^ ]* crash-image: prefix_applied=[0-9]*' research/prompts/m2-wave3-code-r1-opus-model/logs/y3-formatted.log
k=21 last=- crash-image: prefix_applied=0
k=22 last=write@dev0+0 crash-image: prefix_applied=0
k=23 last=write@dev1+0 crash-image: prefix_applied=0
k=24 last=barrier@dev0+0 crash-image: prefix_applied=0
k=25 last=write@dev0+16777216 crash-image: prefix_applied=0
k=26 last=write@dev1+16777216 crash-image: prefix_applied=0
k=27 last=barrier@dev0+0 crash-image: prefix_applied=0
k=28 last=fua@dev1+4194304 crash-image: prefix_applied=0
k=29 last=write@dev0+4096 crash-image: prefix_applied=0
k=30 last=write@dev1+4096 crash-image: prefix_applied=0
k=31 last=barrier@dev0+0 crash-image: prefix_applied=0
k=32 last=write@dev0+16781312 crash-image: prefix_applied=1
k=33 last=write@dev1+16781312 crash-image: prefix_applied=1
k=34 last=barrier@dev0+0 crash-image: prefix_applied=1
k=35 last=fua@dev0+7340032 crash-image: prefix_applied=0
k=36 last=write@dev0+0 crash-image: prefix_applied=0
k=37 last=write@dev1+0 crash-image: prefix_applied=0
```

覆盖写那条扫描只在「红了」或「最后一次」时打印一行，所以 k=78 没有 overwrite1..18 的行就是那 18 步一条都没红（驱动 `main_sweep` 的打印条件）。

## 附录 D　正文引到的 kb 行，整行原样（`awk "NR==行号"` 现取）

**`.claude/kb/decisions/08-核心索引结构.md:203`**

```markdown
**射程**：**唯一性的射程是一条已发布历史内**——每个容器号都是某条记录首次插入时的 inode 号；号在回退之后可以重发，所以同一棵树里两个容器可以同号，分开它们的是容器出生代（右半的出生代 = 触发分裂那次发布的 txg，而回退之后每一个新 txg 都严格大于每个已发布的 txg，D23（journal 的角色与格式） 已定项 14），同一次发布内「分裂 → 合并 → 再分裂」也造不出同身份两版。跨崩溃不成立：崩溃后 checkpoint 号会被重发，被抛弃那版与重试那版四段逐字相同、内容不同、都在盘上——这与数据单元的五元组、码 2 节点的 (树, 层级, 诞生代号) 是同一种暴露（根记录靠 D16（发布语义） 骑手条款 1 的实例代号分辨，别的单元类靠类身份段里的写序 (实例代号, 事务号) 分辨——容器不单独加实例代号，写序里那 4 字节就是它），常规路径由 D3（空间分配） 已定项 1 的谓词（分配记录）判，扫描重建路径按 C113（扫描重建时多版单元的现行版本判定无输入） 定案的 P2 / P3 规则判。⚠️ **判它安全的依据与树 ID 那条同源**：孤儿与活版的实例代号必不相等（D18（块里携带什么信息） 已定项 11「作废一个 checkpoint 必须换实例代号」），扫描重建的两道闸按写序择新。⚠️ **残留的疑点**：讲统计量合并规则那一句「『未发布的等于没发生』只对常规挂载路径成立，**扫描重建看的是盘**」读起来像是反方向的——它管的是「水位在一个代段里取最大值而不是后者胜」，与「未发布过的号可不可以重发」不是同一个命题，但两者挨得很近。⚠️ **重建的射程是单头镜像**：有克隆头时共享叶头里的出生树是 origin（I-1.3（块头树 ID 一致） 的 C110（跨头共享与加密后元数据类的块头一致性读法） 那一格），按出生树认领会把克隆头的树认残——多头镜像的归属划分要克隆祖先表（E90（树 ID 进 AAD 与跨头共享） 第二笔），落地前多头镜像的 inode 树重建没有输入。⚠️ 填充率没有下界：`⌈n/233⌉` 是新建树的值，1e6 顺序创建再删 90% 不合并是 9.98 倍空间放大；加宽记录能同时缩小爆炸半径（160 ⇒ 204 条），这笔取舍归 C100（inode 容器丢失是永久损失而没有任何冗余条款覆盖它）。⚠️ **挡着「第一个事务写」的九样，① ② 已定案，其余照录**：① 树表条目已由已定项 8 定；② 记账条目的 seq 住 value、宽 4 字节（已定项 7）；③ 统计量标签值（C71（不认识的统计量标签静默忽略））；④ 不带设备维的统计量的设备段取值、新建树的记账行初始化、读现行值的读法（C117（不带设备维的统计量的设备段没有取值））；⑤ 码 3 容器带不带扩展点（C108（扩展点归属对打包记录单元无判定））；⑥ 码 2 节点头的基础字节数（已由已定项 11 收口）；⑦ `deleted_inodes` 的形态（C118（`deleted_inodes` 树的形态无落点））；⑧ ⑨ 记录格式与统计量登记表两个冻结组件的 spec 文件（C119（冻结组件没有 spec 文件））。⚠️ 跨头首次 COW 时容器身份要不要重生的另一半归 E90（树 ID 进 AAD 与跨头共享） 第二笔。⚠️ **`blocks` 不承担对外的 `st_blocks`**：stat 报的 `st_blocks` 怎么给（stat 时从 extent 树现算分到的单元）、SEEK_HOLE / SEEK_DATA，挪到做 POSIX 面的里程碑。`blocks` 按文件长度向上取整改第一个事务的字节：那条记录偏移 48 的 8 字节从 64 变 6。
```

**`.claude/kb/decisions/08-核心索引结构.md:240`**

```markdown
- **② 树 ID 水位住根记录，取根环里全部根记录该字段的 max**。逐字形态：`新水位 = max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)`，**每次发布随根记录重写**。「根环里全部根记录」读作根环里全部自证过的根（被抛弃时间线上的也算），并上环里全部自证通过的 journal 记录新根段带的该字段（D23（journal 的角色与格式） 已定项 15）：根所在的槽读不出而它那条记录还在时，恢复本来就从记录把那条根重建出来，记录带的水位就是那条根的水位；journal 记录要等它对应的树写好、根落盘之后才会被绕环盖掉。根与它那条记录都读不出（双重介质故障）的那一格盖不住，记在 C342（树 ID 水位在根读不出时退回去重发）。从 mkfs 之外的水位发第一个文件版本时，八棵树的号从水位起连号发，次序照格式常量 11..18 那一组（extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist）。⚠️ **「累计」这两个字是承重的**：读成「本 checkpoint 发出的最高号」（不累计）时，树 ID 会随根轮出环被重发，而**那时被重发的号属于已发布的树**，D6（快照实现模型） 判据 9 正面命中 ⇒ 整条定案不成立。
```

**`.claude/kb/decisions/23-journal的角色与格式.md:362`**

```markdown
3. **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零。前缀末的计数器取环里读得出的最大那一个：所选根自己那条记录两份都读不出时，新实例的第一条编在读得出的最大号 + 1，落回那条读不出的记录原来的槽（被重写的本来就读不出）；计数器从记录头读，不按 txg 推。定长环的槽位由 jsn 决定（槽位与序号解耦那条出路已被 E36（槽位映射那一维） 判掉），归零会让新实例的记录落到旧前缀还要用的槽上；已定项 9 的 48 位本来就按全卷寿命算。⚠️ **checkpoint_txg 也一样**：新实例（普通挂载、切换、回退）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1；普通挂载与回退本来就全环扫描，切换不为算这个下界重扫 journal 环（内存里的 txg 已不小于这次挂载见过的一切；切换的所选根要读盘择，见实例切换那一句）。它与 C331（择根倒挂压过已确认的写） 的修法候选「记录扫描水位」是同一个量，C331（择根倒挂压过已确认的写） 还清时一起写。代价：跳号比今天大（最多到被抛弃时间线的长度），D5（快照 / 空间记账机制） 已定项 2 的点删随之改成「删掉 ≤ 当前代 − K 的全部代」。
```

**`.claude/kb/invariants.md:58`**

```markdown
| I-7.8 | 根记录树 ID 水位不低于全池最大树 ID | **根环里全部根记录的「树 ID 水位」字段取 max，必须 > 该池中出现过的最大树 ID。**「出现过的」由 checker 全盘扫描算出：全部单元头的索引节点类身份段里那个树 ID（D18（块里携带什么信息） 已定项 7），并上树表条目里的树 ID；**不许与运行时共用同一段代码**（D13（验证路线） 已定项 5）。取 max 的范围是「根环里**全部**根记录」不是「回退候选集」——被抛弃时间线的根仍然承载它那一代发过的号，形态与 D23（journal 的角色与格式） 已定项 14 给 checkpoint_txg 取值那句同源。**判别力**：抓两类——① 某次发布没更新水位；② 水位被实现成「本 checkpoint 发出的最高号」而不是**累计高水位**（后者的镜像是连发 R + 1 个 checkpoint 各建一棵树，第一棵的号会随根轮出环而被重发）。⚠️ **只许写成不等式**：水位是「下一个可用号」的上界，写成等式会把正常镜像判红 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **checker 口径注（2026-09-13）**：「出现过的最大树 ID」要把树表里根指针为零的条目也数进去（livelist 6、稀疏旁表 7 day-1 只有树表条目、没有任何单元，D6（快照实现模型） 已定项 2 / D5（快照 / 空间记账机制） 已定项 6），只数单元头会漏。 ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：扫描方向只数诞生代号不超过根环里最大 checkpoint_txg 的码 2 节点——崩在发布之前的那个事务写出的孤儿节点不算「出现过」；按字面数它们，层 0 全量 262165 个状态里 261891 个判红（水位 11 而盘上已有树 ID 11–15 的孤儿节点），见 records 2026-09-13-总审核 十一·七 |
```

**`.claude/kb/invariants.md:136`**

```markdown
| I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
```

**`.claude/kb/invariants.md:127`**

```markdown
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——第一个事务之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**2026-09-17 起「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ 最新根自己带的回退下界 F（checker 只看一个镜像，读最新根那一份 F；回退候选集用的 F_生效 要跨盘算（各幸存盘所带 F 最大值的最小值，D16（发布语义） 已定项 1），一块盘的载体根坏掉之后两者可以不同：那段时间 checker 按最新根的 F 判，下一次可写挂载的新根把 F_生效 写回盘上——代码三方第二轮云端攻方腿打中 F 回落让三条不变量一起红，口径交用户，见里程碑「第二个事务」步 5 的决策点）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「第二个事务」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`） |
```


## 附录 E　Y4-a 与改法 E

```
$ grep -E '^k=(77|78|79|80|81) (last|step=mount)' research/prompts/m2-wave3-code-r1-opus-model/logs/mutY4-run2.log | sed -E 's/I-3.10=(\w+).*/I-3.10=\1/' | cut -c1-230
k=77 last=barrier@dev0+0 crash-image: recovery prefix_applied=0 red=[] I-3.10=Holds
k=77 step=mount root=(2,5) red=[] I-3.10=Holds
k=78 last=write@dev0+16789504 crash-image: recovery prefix_applied=1 red=[] I-3.10=Holds
k=78 step=mount root=(2,7) red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
k=79 last=write@dev1+16789504 crash-image: recovery prefix_applied=1 red=[] I-3.10=Holds
k=79 step=mount root=(2,7) red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
k=80 last=barrier@dev0+0 crash-image: recovery prefix_applied=1 red=[] I-3.10=Holds
k=80 step=mount root=(2,7) red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
k=81 last=fua@dev1+4198400 crash-image: recovery prefix_applied=0 red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
k=81 step=mount root=(2,7) red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
$ grep -E '^k=(77|78|79|80|81) (last|step=mount)' research/prompts/m2-wave3-code-r1-opus-model/logs/fixE-mutY4.log | sed -E 's/I-3.10=(\w+).*/I-3.10=\1/' | cut -c1-230
k=77 last=barrier@dev0+0 crash-image: recovery prefix_applied=0 red=[] I-3.10=Holds
k=77 step=mount root=(2,5) red=[] I-3.10=Holds
k=78 last=write@dev0+16789504 crash-image: recovery prefix_applied=1 red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
k=78 step=mount root=(2,7) red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
k=79 last=write@dev1+16789504 crash-image: recovery prefix_applied=1 red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
k=79 step=mount root=(2,7) red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
k=80 last=barrier@dev0+0 crash-image: recovery prefix_applied=1 red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
k=80 step=mount root=(2,7) red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
k=81 last=fua@dev1+4198400 crash-image: recovery prefix_applied=0 red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
k=81 step=mount root=(2,7) red=["I-3.10: \"盘 0 槽 50182 的分配记录（未释放、跨 2 槽）分配代 5，它罩住的单元头里的诞生代号是 4\""] I-3.10=Violated
$ diff <(grep -E '^k=' research/prompts/m2-wave3-code-r1-opus-model/logs/y3-run1.log research/prompts/m2-wave3-code-r1-opus-model/logs/y3-formatted.log -h | sort) <(grep -E '^k=' research/prompts/m2-wave3-code-r1-opus-model/logs/fixE-y3.log | sort) && echo IDENTICAL_LINES; grep -cE '^k=' research/prompts/m2-wave3-code-r1-opus-model/logs/fixE-y3.log
IDENTICAL_LINES
347
$ diff <(grep -E '^k=(6[0-9]|7[0-7]|8[1-3]) ' research/prompts/m2-wave3-code-r1-opus-model/logs/mutY4-run2.log) <(grep -E '^k=(6[0-9]|7[0-7]|8[1-3]) ' research/prompts/m2-wave3-code-r1-opus-model/logs/fixE-mutY4.log) && echo SAME_OUTSIDE_78_80
SAME_OUTSIDE_78_80
```

`mutY4-run.log` 是第一次的变异（不分发布、见 txg 4 就改），挂载那次写行发布也落在 txg 4、被一起改坏，挂载之后的红分不开是谁的，不用；正文只用 `mutY4-run2.log`（开关只在发布 B 那一次开）。
