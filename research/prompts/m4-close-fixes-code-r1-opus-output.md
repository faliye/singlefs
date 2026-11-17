# m4-close-fixes-code-r1 攻方腿（Opus，第一轮：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱）

日期：2026-11-09。被判树：主工作区 `crates/singlefs-core/src/` 六份，与 `research/prompts/m4-close-fixes-code-r1-snapshot/crates-sha256.txt` 比 6 个 OK（量的，`sha256sum -c`）。下面代码行号都是这棵树的。

模型目录 `research/prompts/m4-close-fixes-code-r1-opus-model/`，**编不编得过、绿不绿都没跑**（子 agent 不交任务，只过了 `rustfmt` 解析）。

| 文件 | sha256 |
|---|---|
| `tests/attack_fixes_r1_one_copy.rs` | `53a3d0c5ad29d137af561ad6367a5f48246d8c351744df60220c35b6d95ba110` |
| `tools/rerun.sh`（照 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps`） | `c05b5b5e4b46e3da92e3c4f4c92997b9defe880378d096955b33d609fbf7423e` |

复跑命令在末尾「要主 agent 跑的命令」。用例断言今天的行为：目标绿 = 下面标「模型」的几形在这棵树上成立；两条 sweep 断言「没有别的打中」，红了就是新的打中。标「推的，待主 agent 跑」的都还不是量的，代码行是我现查的。

## 各格判定一览

| 格 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| 攻击面 1 T3 | 打中（名字与报告） | 被拒推抬 F 的路径本身没打穿；结局报在只说快照的成员里，实现员说补了的文档没补 | 代码 |
| 攻击面 1 T4 | 没打中 | `mounted_session.rs` 182 行变异锚点逐个恰好 1 次（量的）；新 `assert_eq!` 走不到红 | 量的、代码 |
| 攻击面 2 第 11909 行 | 打中（轻）、不伤数据 | 两种查不到走不到；「没有这把 key」只有造坏镜像走得到，走到时少计一次、接着发布被拒；提示过期那一支产品路径走不到，是越出判决现量的外推 | 代码、模型 |
| 攻击面 3 「可写挂载期间别的读」 | **打中** | 三处真读不计不搬；用户读根本不经会话，系统在读的那一刻看不到自己在可写挂载期间 | 模型（推的，待主 agent 跑） |
| 攻击面 4 只试一次 | 没打中 | 只试一次、被拒不取、下次挂载重记，都对得上 D4 | 模型（同上） |
| 攻击面 4 结局成员 | 打中（名字） | 同 T3 | 代码 |
| 攻击面 4 三类豁免 | **打中** | 「本来就换落点」对映射树叶是假的；「旧落点走释放之前读盘核」对三类都是假的 | 模型（同上）、代码 |
| 攻击面 5 两种计数口径 | 数上不打架；理由句打架 | 一次调用内去重这一读法下数自洽；「同一种口径」两句注释说不通 | 代码 |
| 条款格 D4 已定项 4 | 不成立三处 | 见攻击面 3、4 | |
| 条款格 C946 | 不成立（kb 过期，自相矛盾） | 正文与验收还是旧说法，验收要冷恢复与只读挂载搬，与 D4 反 | 现查 |
| 检查单：走不到的理由 | 不成立一处 | 见下 | |
| 检查单：类型与名字 | 不成立 | 三个名字只说一半 | |
| 检查单：不兼容旧代码 | 成立 | `grep` 0 条 | 量的 |
| 检查单：不新建判定库 | 成立（只核 diff） | | |
| 检查单：多机可用 | 成立 | `/proc/self/status` 只进进度行 | |
| 检查单：测试优先多线程 | 成立 | | |
| 检查单：带 --item | 不成立 | 正文行号与今天的表差开；一条入口脚本已不在 | 量的 |
| 检查单：新崩溃流 | 推不出（算打中） | 「其余三条流」对不上数；只读挂载打开新加的下界判法没有流也没有变异行 | 现查 |
| 分工：最脆弱 | D4 第 73 行三类豁免那一句 | 是实现员「推的」写回 kb 的，保留的那半句押错了地方 | |

## 攻击面 1：T3、T4

### T3 另发一批搬迁与被拒推抬 F

路径本身我试的形状都没打穿（见「没打中的形状」）。打中的是这一批把挂载那一半并进来之后，报告口子没跟上：

- 字段文档只说读回与快照：`crates/singlefs-core/src/mounted_session.rs:205`：「这次改动里读—改—写读回时只坏一份、改动之后仍被活快照钉着的旧数据单元，另发的那一批搬迁做成了什么」。
- 成员 `NotNeeded` 的文档同样：`crates/singlefs-core/src/mounted_session.rs:219`：「不另发：这次改动读回旧单元时没有只坏一份的」。挂载记下的 key 全被改动重写时也是它，文档答不上。
- 第一批报告说补了（`~/.local/share/singlefs/work/m4-close-code-r3-fix-writer/result.md` 第 192 行，「文档注释补了挂载时那一半」）；类型上那两处没补，只有函数 `relocate_the_old_data_units_kept_for_the_snapshots` 的文档写了（`crates/singlefs-core/src/mounted_session.rs:2074`）。
- `Relocated { mapping_keys }` 说「搬了这几把映射 key」（`crates/singlefs-core/src/mounted_session.rs:221`）。计划里一把 key 的单元若是这一批发布本来就重写的节点（`crates/singlefs-core/src/transaction.rs:10747` 那一支），不算搬也报在里面。快照下那批全是数据单元走不到；挂载这批含码 2、码 3 节点才走得到。推的，要多层分配记录树才造得出，sweep 只罩小树。

四句：不分辨臂（只改名字与文档）；系统看得到；满足 D4 第 71 行「结局随那次改动报出」的字面，报是报了，名字说错；改法在改法表第 5 行。

**T3 的一条线索，今天产品路径走不到**：挂载之后、头一次改动之前跑过整理把那个单元搬走，key 还在映射里，头一次改动之后再搬一次好单元，空间紧时为它推抬 F。产品路径上整理批大小没定、一个单元都不搬（`crates/singlefs-core/src/mounted_session.rs:2644`：「批大小条款没写，产品路径在签任何意图之前停下」），所以只记线索。批大小定下来的那一轮要回来看：计划前按「映射里的落点还是不是记下时那一组」滤一道就没这事，今天记的是 key、丢了位置。

### T4 `publish_the_change_once`

没打中。这一批只在它后面接了 `add_the_reads_of_the_writer`。新 `assert_eq!` 依赖写入口两份账一次一把，唯一记账处同一分支记两样（`crates/singlefs-core/src/transaction.rs:11675`、`crates/singlefs-core/src/transaction.rs:11676`），写入口每次发布新开（`crates/singlefs-core/src/mounted_session.rs:2317`）。预演另带一份账（`crates/singlefs-core/src/mounted_session.rs:1783`），不进累计。

变异锚点（量的，一段内联 python 读 `crates/mutations.tsv`，第三列 `\n` 换成换行、在主工作区数子串）：

```
rows_per_file {'recovery.rs': 271, 'transaction.rs': 449, 'mount.rs': 494, 'mounted_read.rs': 38, 'mounted_session.rs': 182, 'compaction_plan.rs': 54}
rows_not_exactly_one 0
```

## 攻击面 2：`transaction.rs` 第 11909 行

```
if !hints_are_the_mapping_position {
    copies_read.set(CopiesOfAUnitRead::WithoutAFailingCopy);
```

记 false 有三种来路，逐个查：

| 来路 | 走得到吗 | 依据 |
|---|---|---|
| `EntryNarrowerThanItsFieldTable` | 走不到 | 重建时映射叶每条都解，切不动就拒挂载（`crates/singlefs-core/src/recovery.rs:4556`）；写者写的条目定宽 |
| `NodeMalformedOrNoEntryWithThisKey` 的「叶解不开」 | 走不到 | 重建读映射树时头判过、叶字节自证过；写者写的解得开（推的，没逐行追写者） |
| 同一成员的「叶里没有这把 key」 | **只有造坏镜像走得到** | 重建只数条数（`crates/singlefs-core/src/recovery.rs:6825`），不核每个进映射单元的 key 在不在；一把 key 换成别的、条数不变就过，`take_the_mapping_positions_over_the_location_hints` 对查不到的不动提示（`crates/singlefs-core/src/recovery.rs:3714`） |
| `Found` 而不等于提示（提示过期，这一判本来要防的） | 产品路径走不到 | 交进来的上一版在内存里记的已是映射位置，两处调用（`crates/singlefs-core/src/transaction.rs:3863`：「两处调它」）。第二批报告已提（第 180 行），我逐处追了一遍同意：发布的一版指针来自上一版或新落点，整理那批换成目的地，重建盖映射位置 |

**走到时计数少一次对不对**：模型 `the_read_back_with_no_entry_in_the_mapping_drops_the_count_and_the_publish_is_refused_anyway`（手改内存里那一版的叶字节）。期望观测（推的，待主 agent 跑）：对照那一读记只坏一份；查不到 key 那一读记没有坏的一份；接着的部分写被拒（照映射释放查不到，`crates/singlefs-core/src/transaction.rs:4714`），写入口计 0。所以少这一次不丢修复：这个单元本来就发不出去。按 D4 字面仍是少一次：`.claude/kb/decisions/04-校验和位置.md:71`：「读到只坏一份时计数」；这张盘池级 checker 该判红（推的）。与另三个读者的套子同一判法（`crates/singlefs-core/src/recovery.rs:415`），一处改要四处改。

**越位这一层才是这一格要报的**：判决第二节 T2 ② 现量的只有只读 `read_at` 与可写挂载（`research/prompts/m4-close-code-r3-main-verification.md:27`：「只读 `read_at` 与可写挂载照样各计 1 次」），部分写读回那一处是实现员外推进去的。于是：产品路径上这一判恒真；证它的用例要手造一版提示旧落点的上一版，直接调读回（第二批报告第 1 条）；变异行抓到的是手造路径。什么现象推翻：造出一条产品路径（经会话、不手改指针）让上一版的指针与映射落点不等。

四句：不分辨臂；系统看得到；满足的是 D4 第 71 行「读到只坏一份时计数」，只在造坏镜像上；改法在改法表第 6 行。

## 攻击面 3：D4 已定项 4「可写挂载期间别的读」

条款：`.claude/kb/decisions/04-校验和位置.md:73`：「可写挂载里的读都计数并搬，包括部分写与截断的读回、可写挂载时建上一版那一读、可写挂载期间别的读」。

第一批报告答「没有别的读要接」（`~/.local/share/singlefs/work/m4-close-code-r3-fix-writer/result.md` 第 193 行）。我核到的可写挂载期间四处真读，都不计：

| 读 | 代码 | 读到只坏一份时 | 模型用例 |
|---|---|---|---|
| 用户读文件 | 会话没有读入口（`mounted_session.rs` 的 `pub fn` 我列了一遍，量的，33 行里没有读文件的）；今天只有 `open_pool_for_read` + `read_at` | 读者自己的观测计 1；会话两个数都是 0、没有 key、下一次改动 `NotNeeded`、坏的一份还在映射落点上 | `a_file_read_during_the_writable_mount_is_counted_nowhere_in_the_session_and_not_relocated` |
| 为空间推抬 F 时沿链读实例表 | `crates/singlefs-core/src/mount.rs:5380`：「let table = instance_table_chain_of_root(&*devices, &current.root)」；读者是盘表，默认实现什么都不做（`crates/singlefs-core/src/recovery.rs:193`） | 不计 | `the_instance_table_read_while_raising_the_floor_in_the_session_counts_nothing`（带两份都坏的对照，证那一读真发了） |
| 释放之前读盘核 | `crates/singlefs-core/src/transaction.rs:5352` 起 | 两份一起隔离，不计 | `the_release_check_meeting_one_damaged_copy_quarantines_both_and_counts_nothing` |
| 搬留存单元读源 | `crates/singlefs-core/src/transaction.rs:11182` 那一段 `find_map` | 不计（这个单元本来就在搬） | 没写，读代码 |

还有 `discard` 之前读实例表（`crates/singlefs-core/src/mounted_session.rs:3476`），同第二行的读者。

用户读那一行是这一格的分量所在。四句：

1. 分不分辨臂：分辨。两种读法——乙「只算会话自己的读」下用户读不归这句管，甲「可写挂载期间任何读」下它归；今天的实现只对乙成立。
2. 被判的系统当时看得到吗：**看不到**。`open_pool_for_read(reader, root)` 的入参里没有会话，也认不出同一池上开着可写挂载。这是「判别子观测不到」那一形，不改接口判据做不到。
3. 满足的是哪个分句：上面第 73 行「可写挂载期间别的读」，与第 71 行「可写挂载里读到的，再经已有的搬迁路径把这个单元整个搬到新槽」。按乙读法这两句落空（今天没有那样的读），按甲读法这两句不成立。
4. 改法在这几格上还中不中：见改法表第 1、2 行。

什么现象推翻：用户定案原话里「可写挂载期间别的读」说的是某一条已有路径。我没读变更史里的原话，这一格的读法要主 agent 对原话（`.claude/kb/decisions-history.md` 一类），我这里是「按字面」。

## 攻击面 4：第 7 条

### 只试一次：没打中

模型两条（推的，待主 agent 跑）：`the_keys_kept_at_the_mount_survive_a_refused_change_and_are_taken_by_the_next_published_one`（被拒的改动不取走，第 2596 行的 `?` 在取走第 2602 行之前）；`a_mount_key_refused_not_for_space_is_tried_once_and_comes_back_at_the_next_mount`（拒过一次清空、下一次改动 `NotNeeded`、下一次挂载重记）。都对得上 D4 第 73 行「那一批搬没搬成都只试这一次」。

有一条是条款认下的代价，交主 agent 知道：一批是全有全无（`crates/singlefs-core/src/transaction.rs:10861` 逐项核，一项不合整批拒），一把留存单元另一份此刻也读不出（`UnitBytesNotAvailable`）就连带挂载那几把一起丢，要等下一次挂载。要连着两次盘故障，推的，没造。

### 结局报在只说快照的成员里：打中名字

见 T3。与它连着的：计划可以含码 2、码 3 节点（`crates/singlefs-core/src/transaction.rs:11794` 起按类标签取），名字与成员都叫 `old_data_units`。

### 三类豁免：打中两句

条款：`.claude/kb/decisions/04-校验和位置.md:73`：「豁免映射的三类（映射树节点、树表、实例表片）认不出映射 key，不进计划：它们在挂载这一串的发布里本来就换落点，旧落点走释放之前读盘核那一路（推的，多片实例表链里不被改写的那几片没核）」。

**① 映射树叶不换落点。** 映射树按 key 重写走到的那几片（`plan_the_tree_after_this_publish` 只重写动到的节点，`crates/singlefs-core/src/code_two_tree.rs:301`：「上一版里这次被换下的节点（没被照抄进这一版的）」），老数据单元那片叶挂载与头一次改动都不碰。

具体历史（模型 `a_mapping_tree_leaf_with_one_copy_damaged_stays_where_it_is_after_the_mount_and_the_first_change`，推的，待主 agent 跑）：第一个文件 700 个单元（映射树多层）→ 在一份没弄坏的盘上走一遍找出落点不变的叶 → 另一份上弄坏那片叶一份 → 可写挂载（每一步许可：`crates/singlefs-core/src/recovery.rs:1177` 经数的那一层读映射节点，计 1；`crates/singlefs-core/src/transaction.rs:11739` 认不出 key）→ 建一个空 inode（`NotNeeded`）→ 期望：叶落点不变、坏的那一份还在、下一次挂载又计 1。没有任何路径修它：搬迁入口拒（`crates/singlefs-core/src/transaction.rs:11126`，`PinnedUnitInThePlan`），整理判钉住。一份坏掉之后，另一份就是这片叶几百个单元的唯一一份映射，每次挂载计一次、不修，直到哪次改动碰巧重写了这片叶里的 key 把它换下（用户动作这里只放开到「不碰这片叶」的改动：建空 inode；重写那几个单元的改动会换下它，换下时走 ②，也不隔离）。

**② 旧落点不走释放之前读盘核。** 树表与实例表确实换落点（写行重写整条链，`crates/singlefs-core/src/mount.rs:1316`），但释放之前读盘核只判经映射的角色：`crates/singlefs-core/src/transaction.rs:5217-5221` 三类记 `false`；同一个函数的文档 `crates/singlefs-core/src/transaction.rs:5172`：「映射树、树表、实例表豁免映射」「硬规则 1 的读盘核挂在「经映射核到那条映射条目之后」，它们不在其内」。模型 `the_exempt_roles_replaced_by_the_mount_are_released_without_the_release_check`：树表、实例表第 0 片各坏一份，挂载的写行、暖机、续做与现行那一版里隔离清单都没有它们（推的，待主 agent 跑）。旧槽就此进释放，坏的那块槽不隔离。

代码自己也写反了：`crates/singlefs-core/src/transaction.rs:11739`：「也认不出——它们的旧落点照释放之前读盘核那一路走」，上一行举的例子里有树表。

**③ 条款保留的那半句押错了地方。** 第 73 行只给「多片实例表链里不被改写的那几片」打了「推的」，而代码说链是整条重写的（`crates/singlefs-core/src/mount.rs:1343`：「每次写行整条链 COW 重写」）：打了保留的那一处大概没事，没打保留的两处是假的。这句的来路是第一批报告第 193 行的「推的」，写回 kb 时大半句成了射程正文。

四句（①②合）：不分辨「搬 / 不搬」，分辨的是这句理由站不站得住（树表、实例表第 0 片落点确实换，① 只打映射树节点，② 打三类）；系统看得到（经数的那一层交得出位置条目，认得出是映射节点）；满足的是上面那句的「本来就换落点」「旧落点走释放之前读盘核那一路」两个分句；改法在改法表第 3、4 行，第 3 行碰不到 ②、第 4 行碰不到 ①。

## 攻击面 5：`read_at` 并进累计与同 key 只计一次

**数上不打架**，三处各数各的，每处都能写成「一次入口调用内同一个单元至多一次」：

| 数 | 去重的范围 | 依据 |
|---|---|---|
| 挂载建上一版 | 一次挂载、同一组位置条目 | `crates/singlefs-core/src/recovery.rs:305` |
| 写入口累计 | 一次用户改动、同一把 key；跨改动相加 | `crates/singlefs-core/src/mounted_session.rs:311` |
| `read_at` 累计 | 不去重；一次调用里一个单元只读一遍，跨调用相加，报错的那次也并 | `crates/singlefs-core/src/mounted_read.rs:999` |

**打架的是两句理由**：

- `crates/singlefs-core/src/mounted_session.rs:261`：「被拒的那几次发布读回的也算：读是真发到盘上的。同一个旧单元一次改动里只算一次」。前半句的理由（真发到盘上）放到后半句就不成立：重发那一次也是真发到盘上的，不算。
- `crates/singlefs-core/src/mounted_read.rs:997`：「与写入口「改动被拒时那几次读也算」同一种口径」，而写入口已改成去重，`read_at` 没有。
- `crates/singlefs-core/src/mounted_session.rs:263` 说与挂载「同一份坏副本一次挂载只计一次」「同一种口径」。一次挂载里同一份坏副本写入口能计多次：改动被拒、用户再发一次，读回同一份、再计一次（推的，没造：要真空间拒到底）；挂载计一次之后头一次改动读回又在另一个数上计一次（模型 `the_same_key_at_the_mount_and_in_the_read_back_under_a_snapshot_is_planned_once` 钉 `(1, 1)`）。

报错那次的 `observation` 随错丢掉（`crates/singlefs-core/src/mounted_read.rs:998`），累计与逐次返回值之和不再相等：条款没写要等，注释写明了，不算打中。

四句：不分辨臂（只改注释）；看得到；D4 只写「计数」，没定口径，这一格是条款空着；要定口径该交用户，不该由注释说「同一种」。什么现象推翻：有一条定义「只坏一份次数」的条款，三处都合。

## 第一轮分工

**过度外推与越位**

1. 第 7 条「豁免三类」从实现员「推的」进了 kb 射程（上一节 ③）。
2. 第 4 条从判决现量的两个读者外推到部分写读回（攻击面 2）。
3. 「没有别的读要接」（第一批第 193 行）是只看「经数的那一层的读」得出的，把「数了的」当成了「读了的」（攻击面 3）。

**内部矛盾与循环依赖**

1. `crates/singlefs-core/src/transaction.rs:11739` 与 `crates/singlefs-core/src/transaction.rs:5172` 互相否定（树表走不走读盘核）。
2. D4 第 71 行写「两份以最新那一版里指着这个单元的指针为准」，第 4 条的实现按映射落点判（`crates/singlefs-core/src/recovery.rs:1016`：「只坏一份按这个单元此刻的权威落点判」）。第 71 行说的是读哪一份、代码说的是计不计，严格说不矛盾；但提示过期不计这一条 kb 里没有，只在判决里。写回时要补一句。
3. C946 验收要冷恢复读与只读挂载之后搬：`.claude/kb/checks-owed.md:589`：「部分写读回、截断尾单元读回、挂载态 `read_at`、冷恢复读各跑一次，读出的都是原内容、计数各加一，之后这个单元落在新槽」；D4 第 73 行「冷恢复读与只读挂载只计数、不搬」。C946 正文「不计数」「读到一份坏之后没有代码把它修掉」、D4 第 77 行「这两样未实现」也过期（`research/prompts/m4-close-code-r3-main-verification.md:55` 说随收尾写回，到今天没写）。
4. 循环依赖：第 1 条判分隔 key 的文档靠「条目按分隔 key 严格递增由这两句合出来（叶都不空）」，三个读者叶不空各自判过（`crates/singlefs-core/src/mounted_read.rs:575` 那一段我读了逐条判下界），我推了一遍成立，不算打中。

**哪一条最脆弱**：D4 第 73 行三类豁免那一句。它是唯一一句把「不搬」的理由写成事实的，两个分句都能用一张小盘证伪，而且映射树叶那一形的后果是一处没有路径主动去修的单点。

## 检查单八格

**走不到的理由二次查实：不成立一处。** 两份报告里写成走不到的我逐处追了：

| 分支 | 判 |
|---|---|
| 第 11909 行提示过期那一支（第二批第 180 行） | 成立：产品路径走不到 |
| 第 11909 行查不到 key 那一支（报告没提） | **不成立**：造坏镜像走得到（攻击面 2） |
| `relocate_the_old_data_units_kept_for_the_snapshots` 的 `expect`「现行那一版是有树的一版」（`crates/singlefs-core/src/mounted_session.rs:2101`） | 成立：只在 `published?` 过了之后调，发成的都是有树的一版 |
| `add_the_reads_of_the_writer` 的 `assert_eq!` 与 `checked_sub`（`crates/singlefs-core/src/mounted_session.rs:298` 一带） | 成立：减数不超过这次发布的 key 数；`expect` 的消息只说了加法那一半，`checked_sub` 走不到不要紧 |
| `mapping_keys_of_the_units_at_these_locations` 里映射查不到两臂（`crates/singlefs-core/src/transaction.rs:11759` 一带） | 成立：`mapped_units` 的 key 查不到只有造坏镜像，走到照「认不出」办 |

**类型与名字：不成立。**

- `RelocationOfTheOldDataUnitsKeptForTheSnapshots`、它的 `NotNeeded` 文档、`UserChangePublished` 那个字段、`relocate_the_old_data_units_kept_for_the_snapshots`：数据单元、快照两样都不再全对（T3）。
- `inode_records_of_the_leaf_ascend_after_the_leaves_before`（`crates/singlefs-core/src/recovery.rs:6203`）多判一句分隔 key，名字只说记录升序，参数名写着才看得出。
- `PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition`、`keep_the_mapping_keys_not_read_back_before_in_this_change`、`read_the_data_units_of_the_range` 说的是做什么，成立。

**不兼容旧代码与旧格式：成立。** `grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-fixes-code-r1-diff.md` 量的 `rc=1`、没有输出。diff 里也没有认旧版本的分支（读的）。

**不新建判定库：成立（只核 diff）。** diff 全在 `crates/singlefs-core/src/`，没有存判定的东西。

**所有任务多机可用：成立。** `data_unit_read_with_one_copy_failing_is_counted.rs` 里没有环境变量、家目录、主机名（量的，`grep`）。`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 3833 行读 `/proc/self/status` 只进进度行，不进判定；卡配置是文件头写的已有那一套。

**测试优先多线程：成立。** GPU 对拍 CPU 那一侧按 `cpu_threads_from_the_value_of_the_variable` 取线程（第 3649 行），进度线程在 `thread::scope` 里，判定域没动（读的，没跑）。数据单元那一份各条单池单场景，libtest 并行。

**带 --item 的命令与项：不成立。**

- 正文写第 6822–6824、6921–6924 行；今天的表（量的，`grep -n 'm4-close-code-r3' crates/mutations.tsv`）这七行在 6821–6823 与 6910–6913，第 6921–6924 行是 E205 那一族。表在别的会话手里漂，点名要按名字（`--item` 收的就是名字），正文的号今天对不上。
- 第一批报告「要主 agent 跑的命令」第 7 条点的 `.claude/gate.d/checker-tier-crates-mutation-replay.sh` 已不在（量的，`ls .claude/gate.d/ | grep -i mutation` 只有 `broad-mutation.sh`、`crates-mutation-replay.sh`、`mutation-duplicates-lag.tsv`）；同一条的 `--item` 是占位串。
- 「两份报告都带 `SINGLEFS_RUN_SCOPE`」：两份的 `naming-lint.sh` 与 `static-check` 那几条没带（第一批第 163、171 行，第二批第 155、163 行）。这几条不算起验证、范围声明闸不要（推的，没喂闸），句子字面不成立、不伤事。

**新崩溃流多罩了什么：推不出，算打中。**

- 「其余三条流」我对不上数。两批新加的只有三条：第一批 `two-inode-leaves-separator-key-of-the-right-leaf-lowered`，第二批两条（`crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:21155`、`crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:21181`）。正文说的第二批两条之外再有三条，推不出是哪三条。那条分隔 key 流多罩 I-9.12 第三句的 GPU 判法，内容上我核了（第 12603 行起）。
- 第一批给 GPU 只读挂载的打开加了记录号下界（`crates/singlefs-verification/src/crash_judge_recovery.wgsl:286`：「if (lt64(inode, separator) || lt64(inode, identity_container)) {」），对应 CPU 那边的 `separator_235`。全部 GPU 流只有「分隔 key 改低」（`grep separator` 量的），没有改高的；`crates/mutations.tsv` 里没有锚这一句的行（量的，`grep -n 'lt64(inode, separator)'` 0 条）。这一判改坏，GPU 对拍照绿（推的，没上卡）。第一批报告「每处改法一条变异行」在这里欠一行、欠一条流，第二批只补了叶序那两样。

## 改法表（都只在我的模型上量过、被攻过零轮；今天连模型都没跑，全是「推的」）

| 行 | 改法 | 修哪一格 | 还中不中 | 状态 |
|---|---|---|---|---|
| 1 | D4 第 73 行「可写挂载期间别的读」写明指什么：今天没有经会话的读文件入口，这句改成「今天会话里除读回之外的读不计数不搬」 | 用户读那一格按乙读法落定 | 不动代码，三处内部读照样不计 | 推的，改条款要走用户 |
| 2 | 给 `MountedSession` 加读文件入口，经数的那一层读、记 key、下一次改动搬 | 用户读那一格按甲读法成立 | `open_pool_for_read` 这条老路照样不归会话管；要的话那条路归只读语义写明 | 推的 |
| 3 | 映射树节点：第 73 行改写成「只计数不搬，直到某次改动换下」，或者记下读到只坏一份的叶位置、下一次发布强制重写它 | 第 73 行 ① | 碰不到 ②；后一种要改 `plan_the_tree_after_this_publish` 的调用方与分段，算要新的提交步骤 | 推的 |
| 4 | `mapped_roles_released_by_this_publish` 把三类豁免也核一次（按指针里的位置项读盘），或第 73 行与 `transaction.rs:11739` 那句删「走释放之前读盘核」 | 第 73 行 ② | 碰不到 ①；前一种动硬规则 1 的范围（D19 已定项 5），交用户 | 推的 |
| 5 | 成员与字段改名（说清含挂载那一半），`crates/mutations.tsv` 那一行一起改；`Relocated` 文档写明含本来就重写的节点 | T3、攻击面 4 名字 | 不改行为 | 推的 |
| 6 | 第 11909 行那一判改成 `debug_assert` 或删掉、用例删掉手造那一条 | 攻击面 2 的死分支 | 不动造坏镜像那一形的计数；与判决第三节第 4 条的字面要对一下 | 推的 |
| 7 | `locations_of_the_units_counted`、`mapping_keys` 这几处 `contains` 换 `BTreeSet`、`HashSet` | 大面积坏盘时挂载的代价（下一节） | 不改结论 | 推的 |
| 8 | 三句「同一种口径」注释改写成各自的范围；要统一口径交用户 | 攻击面 5 | 不改数 | 推的 |
| 9 | GPU 分隔 key 改高一条流、一行变异 | 检查单新崩溃流 | | 推的，待主 agent 跑 |

## 代价线索：去重与认 key 是坏单元数的函数

坏一段盘时 L（坏的单元数）能到 M（进映射的单元数）那个量级。`crates/singlefs-core/src/recovery.rs:305` 每报一次在 Vec 上 `contains`，L²；`crates/singlefs-core/src/transaction.rs:11756` 每个进映射单元在 `locations` 上 `contains`，M×L；`crates/singlefs-core/src/transaction.rs:11752` 与计划那边（`crates/singlefs-core/src/transaction.rs:11786` 一带）再一层。迭代上界的注释（`crates/singlefs-core/src/transaction.rs:11750`：「迭代上界是这一版进映射的单元数加留存条目数；跨轮携带的只有已认出的那几把 key。」）没把这一层算进去。没有条款定上界，不判违例；挂载时长是 M×L 的函数这件事没人量过。模型 `the_counting_reader_and_the_key_recognition_grow_with_the_units_read_with_one_copy_failing` 打量程（L 5 000 到 20 000、M 约 700 与 L 1 000 到 16 000），待主 agent 跑。今天 M 受单元字节全住内存所限（推的）。

## 没打中的形状

| 形状 | 试法与范围 | 结论 |
|---|---|---|
| 被拒的改动提前取走 key | 读 `publish_user_change` 次序；模型一条（不存在的 inode 上 unlink） | 不提前取 |
| 同一把 key 同时在挂载账与读回账，计划出两次 | 读 `planned_before`；模型一条（快照下） | 只一次 |
| 挂载记下的 key 被头一次改动重写 | 读 `relocations_of_the_units_read_with_one_copy_failing_while_mounting`：查不到就不进计划 | 不进计划 |
| `assert_eq!` 在产品路径红 | 两份账只在 `read_back_the_old_payload_counting_a_failing_copy` 一处记（`crates/singlefs-core/src/transaction.rs:11675` 起），调它的两处都把同一对账交进去（`crates/singlefs-core/src/transaction.rs:11510`、`crates/singlefs-core/src/transaction.rs:12066`） | 走不到 |
| 套子转发漏一个方法 | `PoolReader` 九个方法，`PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition` 逐个转 | 没漏 |
| 映射查询在 `note` 里重入（`OnceCell`、`RefCell`） | 查询读映射节点用的是套子里面那个读者，不回到套子；`RefCell` 全文件只有计数层一处，借用不跨调用 | 没有重入 |
| 查询在 `note` 里失败改变判退结局 | 失败被吞、`tree` 没记下，后面照常再读、照常报 | 结局不变（读的，没上流） |
| 退版两趟去重串台（P 的单元认到 R 上） | 位置条目连校验和一起比，相等就是同一个单元 | 没串 |
| 一次发布大到撞 journal 条数上限整批被拒 | 默认环 768 MiB ÷ 4096 ÷ 3 = 65536 条（`crates/singlefs-format/src/lib.rs`，算的），每条 64 项：约 419 万个单元 | 够不着 |
| 跨度与类不符毒死整批 | `span_slots` 按落点规则，`user_visible_role_standing_for_a_unit_class` 按类；sweep 逐角色跑（待主 agent 跑） | 读的没见不符 |
| 每个进映射角色坏一份之后：挂载、计数、搬、文件与快照内容、池级 checker | sweep 两个起点（三单元；三单元加快照、留存、空 inode），每个 `mapped_units` 角色一次；用户动作固定为建空 inode | 待主 agent 跑，红了就是打中 |
| T3：挂载 key 走被空间拒 | 模型一条（开关拒一次），只断言不是 `NotNeeded`、打观测 | 待主 agent 跑 |

用户动作放开到哪：头一次改动试了建空 inode、部分写、整份顺序写、对不存在 inode 的 unlink、建快照。没试真空间拒到底（开关之外）、截断做头一次改动。崩溃状态：这条腿的模型一个都不枚举（盘上字节是发布之后的，故障是手改一份），另发那一批的崩溃点没动我这边的判断，见限度。

## 这条腿自己的限度

- 模型没编过、没跑过。上面每个「模型」都是读代码写的期望。编不过就是我写错了 API，不说明别的；断言红了要先分是我推错还是新打中。sweep 里「挂载计数是 1」对个别角色（挂载不读它的）可能不成立，那要分开看。
- 全程只抽这一次，「没打中」的格不支撑结论（规则要再抽一次）。
- 用户定案原话（变更史）没读；攻击面 3 的甲乙两种读法等主 agent 对原话。
- 崩溃点：挂载 key 走的是整理的搬迁路径，活单元那一批发布路径有崩溃枚举（推的，没逐个数用例）；「挂载记 key → 头一次改动 → 另发那批」这条会话级序列有没有自己的崩溃流我没查全，没写原型。
- 第二批 GPU 流没上卡、`states_staying_on_r=9` 没复核；WGSL 只读了 `separator` 那几处。
- 只读挂载读树节点时只坏一份不计（默认实现、没有数的那一层），这不是这一批的块，没算打中；D4 第 73 行「只计数」要是连树节点也算，这是下一轮的线索。
- 改法全部被攻过零轮。

## 要主 agent 跑的命令

1. 复跑模型（整份）
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-fixes-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r1-opus/copy --work /home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r1-opus/rerun-1`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。`/home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r1-opus/copy` 是 `minimal-copy.py --with-test-inputs` 建的副本（51M，量的），`.deliverables` 写了交接；六份被判文件与开工快照逐个相同（量的）。要拷到别处复跑，模型目录整份拷、`--frozen` 仍指这份。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`；日志里 `name=attack_fixes_r1_one_copy_target rc=0 expected=0`、`name=attack_fixes_r1_summary target_as_expected=yes`；`attack-fixes-r1:` 观测行逐条对上面的期望；`/home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r1-opus/rerun-1/one-copy-test.log` 看 `error`、`panicked at`。
   - 编译错、断言红都交回：新派同立场的腿，「承接：research/prompts/m4-close-fixes-code-r1-opus-output.md」带任务号。挂钟按编验证包一次加一条目标算（推的，几分钟到十几分钟）；sweep 两个起点合计几十个角色（推的），各一次挂载加一次 checker，不枚举崩溃状态。超过 40 分钟的量不到这里（推的）。

等这条结果才能往下做的：把「推的，待主 agent 跑」换成量的，sweep 红了的逐条判四句。

## 没做什么

- 没往统一任务服务交任务；判决要引的数一个都没复跑。模型只过了 `rustfmt` 解析（`/home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r1-opus/fmt.txt` 里 0 个 `error` 行，退 1 是排版差异）。
- 不判正推、辩方那几格；不替主 agent 采纳；没改 `crates/`、kb、规则。
- 没写崩溃枚举原型；没上卡。
- 没删 /home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r1-opus/copy：交主 agent 作 `--frozen` 跑上面那一条，跑完由主 agent 删（51M）。`/home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r1-opus` 里另有 `copy.txt`、`fmt.txt`、`rerun_steps.txt`、`rerun.sh.new`、`session-pubfns.txt` 几个普通文件。

## 结论什么会推翻

- 攻击面 3：用户原话把「可写挂载期间别的读」说成某一条已有路径；或模型里那几读其实经过了数的那一层。
- 三类豁免 ①：模型的 `leaves_untouched` 是 0（映射树每次全重写），或坏的叶头一次改动之后换了落点。
- 三类豁免 ②：隔离清单里出现 `TreeTable`、`InstanceTable`。
- 攻击面 2：造出一条经会话、不手改指针的路，让上一版指针与映射落点不等。
