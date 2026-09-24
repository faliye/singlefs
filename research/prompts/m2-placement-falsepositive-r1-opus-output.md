# 云端攻方（Opus）：Z3、Z4 —— 攻候选改法

轮名 `m2-placement-falsepositive-r1`。立场：攻候选改法，不攻现状。时刻一律 UTC（本机时钟）；跑在 2026-09-23 UTC。

## 〇、各格判定一览

| 格 | 状态合不合法 | 判据错在哪一句（判据第 1 条） | 候选改法 | 第一半（误报消失） | 第二半（改完仍会红的坏镜像 / 历史） | 判据第 2 条 |
|---|---|---|---|---|---|---|
| Z3 | **合法**（D23 已定项 14 要求施加；那条流 oracle `violations=0`） | `walk.rs` 的 `judgements.judge("I-3.1", allocated == Some(walked), …)` 里的 `walked` —— 它只由 `candidate_indexes`（根环里读得出的根）贡献，而「由记录施加出来、根槽从没落盘」的那一版根本不在 `roots` 里 | c：放宽成「记账 ≥ 遍历」 | **量过**：12 → 0 | **量过：交不出** —— 我造的泄漏坏镜像由红转绿（`all_violated=[]`） | **没验过** |
| Z3 | 同上 | 同上 | f：放宽成「遍历 ≤ 记账 ≤ 遍历 + defer 队列字节（统计量 5）」 | **量过**：12 → 0 | **量过：交不出** —— 同一份泄漏坏镜像由红转绿，连没碰 defer 行的那一份也绿（干净镜像上 defer 行本来就有 16384 的松弛） | **没验过** |
| Z3 | 同上 | 同上 | e：有「只由记录施加出来的版本」时报不适用 | 推的：12 → 0（不适用不算红） | **推的：交不出** —— 盲窗跟着残留记录活满一圈环，窗口内任何记账错都不判 | **没验过**（只在纸上） |
| Z3 | 同上 | 同上 | b：把由记录施加出来的版本也并进并集 | 推的 | 推的：交得出（见第三节 b） | **没验过**（没实现、没跑） |
| Z4 | **合法**（合法复用：D23 已定项 14 的重放下界 + D16 已定项 1 的 F；那条流 8 个状态 oracle 全绿） | `crash.rs:670` 的 `copy_is_missing = |copy| !in_place(copy) && !written_over_later(copy)` 的**前一个合取项**：改法之前只有 `!in_place(copy)`，位置让给后来的写之后旧字节必然 `!in_place` | L：**仓里今天落地的那一版**（`written_over_later` 不看更晚那次写持没持久） | **量过**：8 → 0 | **量过：交不出** —— 我造的真洞（Z4-A）在它上面判绿，`record_claimed_state_missing_unit=0`，而 oracle 同一格 `violations=1` | **没验过** |
| Z4 | 同上 | 同上 | P：正文写的那条候选改法（被更晚**已持久**的同位置写盖掉的才不算缺席） | **量过**：8 → 0（整份用例文件 8 passed） | **量过：交得出** —— Z4-A 在它上面判红（`=1`），且真阳性那一格（`②`）照旧红 | **验过了**（这一轮唯一过了第 2 条的改法） |
| Z4 | 同上 | 同上 | P 的剩余盲点 | —— | **量过（可达性存疑）**：Z4-B 在 P 上判绿而 oracle 红；那个持久集合不是段模型枚举得到的 | 见第四节「P 还剩什么」 |

**一句话**：Z4 那条候选改法（P）本身站得住，**但仓里今天落地的不是它**——落地的 L 比 P 宽，宽掉的那一块正好是一整类真洞。Z3 的两个「放宽判据」型候选改法（c、f）都交不出第二半，两个都当场被打穿。

## 一、复跑

副本（不入库）：`/tmp/claude-1000/placement-opus/repo`，由 `rsync -a --exclude target --exclude .git` 从 `/home/fy5090/code/singlefs` 拷出。
`export CARGO_TARGET_DIR=/tmp/claude-1000/placement-opus/target`，一律 `nice -n 19 cargo test -j 8`（开跑前 `ps -o pid,args -u "$(id -u)"` 没有 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` / `fio`；没等过锁）。

```
sha256sum /tmp/claude-1000/placement-opus/*.rs*
90419bc87d9f4eb502f8493f6df4131f64f533fdc5743e25c5d8ccafda5b1f7c  crash.rs.landed     # = 开工快照里的 crates/singlefs-harness/src/crash.rs
0462ed0fa62406d221bf01a3a0ba97a13c60b770f9942b66498b11ae75dbac0a  crash.rs.prefix     # 变异表第 191 行的改法（改法落地之前那一版）
c06211301dd652b60c037b838755efabf0d02e5502fbcc47a73b39a36707287e  crash.rs.variantP   # 候选改法 P
6220c45bb8ae061601038e2cb2f980af6acc4cf6ab549e0dcb7e42525a31ae4b  walk.rs.today       # 我拷走那一刻的主树 walk.rs
e20b493b72f6244c92f1c8194cc6840b222027b041af7a49b4b1fb905bd74a4e  walk.rs.fix-c       # 候选改法 c
669646661970f31f415362009730e4980b15e8e53491fa038022c02b83b4a150  walk.rs.fix-d       # 候选改法 f
c4c7f1ea7d1039d85ca6f17e161b324b348f4191f900a03ab274abcaae985136  repo/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs   # 尾部追加了 Z4-A、Z4-B 两条；写完报告前又在 Z4-A 里加了一段打印段结构的 `Z4_SEGMENTS`（第四节第 3 条用的就是它），所以这个 sha 是加完之后的
d74717362b10179c4960fd061a24b603a81bc4ab19c2a670372f1ad9797164ee  repo/crates/singlefs-harness/tests/checker_known_bad_images.rs             # 尾部追加了 Z3-A、Z3-B 两条
```

⚠️ **开工快照比对（`sha256sum -c research/prompts/m2-placement-falsepositive-r1-snapshot/opening.sha256` 这一次真跑）**：
八个文件里七个 `OK`，`crates/singlefs-checker/src/walk.rs` 一个 `FAILED`（`sha256sum: WARNING: 1 computed checksum did NOT match`）。
我拷副本那一刻主树的 walk.rs 是 `6220c45b…`，快照记的是 `136e5704…`，`HEAD`/`HEAD~1`/`HEAD~2` 三个提交里都是 `9f17ae43…` —— 三者互不相同，快照那一份在工作区与最近三个提交里都取不到（实现线在改它，未提交）。**写这份报告时再量一次，主树已经变成 `d686e2c5…`**。
⇒ **Z3 那几条跑的是 `6220c45b…` 这一版 walk.rs，不是快照那一版，也不是此刻主树那一版**；`crash.rs` 与快照逐字节相同，Z4 那几条不受影响。walk.rs 我一律按**函数名与那一行源码原文**引，行号另标是哪一版上量的。

复跑命令（在副本里，把 `crash.rs` / `walk.rs` 换成上表里对应那一份之后）：

```
nice -n 19 cargo test -j 8 -p singlefs-harness --test second_transaction_step_zero_layer0 -- z4_attack --nocapture
nice -n 19 cargo test -j 8 -p singlefs-harness --test second_transaction_step_zero_layer0 -- residual_record --nocapture
nice -n 19 cargo test -j 8 -p singlefs-harness --test second_transaction_step_zero_layer0 -- stale_tail
nice -n 19 cargo test -j 8 -p singlefs-harness --test checker_known_bad_images -- z3_attack --nocapture
nice -n 19 cargo test -j 8 -p singlefs-harness --test checker_known_bad_images
```

## 二、Z3：那 12 个状态合不合法、I-3.1 错在哪一句

### 2.1 合法——依据两条分项的原文

那一版（实例 1、txg 5）是**记录施加出来的**，根槽从没落盘。它被施加不是可选项，是被要求的：

> **定案**：**恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录；陈旧 tail 只是「从哪开始扫环」的优化，不再决定重放集合。**

（`.claude/kb/decisions/23-journal的角色与格式.md` 第 342 行，已定项 14 的定案句；小标题在第 340 行「#### 已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换」）

那四个单元被写行发布（txg 6）放进 defer、而不是立刻可再分配，同样有条款：`.claude/kb/decisions/05-快照-空间记账机制.md` 第 86 行起的已定项 4「式子逐项对照」表里，**「已分配」与「defer 待释放」是式子里两个不同的项**——

| 式子里的项 | 谁维护它 |
|---|---|
| 已分配 | 第 1 项（已分配字节） |

| 式子里的项 | 谁维护它 |
|---|---|
| defer 待释放 | 第 5 项（defer 队列待释放（按代）） |

（两行都取自同一张表，`.claude/kb/decisions/05-快照-空间记账机制.md` 第 102、104 行；表头的机器标记 `<!-- gate:admission-terms -->` 在第 97 行。）

**跑出来的旁证**：那条流 31 个状态上 `violations=0`、`ignored_violations=0`、`failed_states=0`——oracle（实际走的根是哪一代就得读出那一代的内容）在每个状态上都过。
盘面合法而检查判红 ⇒ 按正文第一节的分支，这是**检查错了**。

### 2.2 错在哪一句（判据第 1 条）

判红的那一行，原文（副本 `walk.rs.today`，`6220c45b…` 这一版第 2516 行；函数 `check_pool_image` 里 `// I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。` 之后）：

```
            judgements.judge("I-3.1", allocated == Some(walked), || {
```

`walked` 的来源（同一版第 2510 行起）是 `per_device`，而 `per_device` 只装 `walk.references`——只有 `candidate_indexes` 里那些根走出来的引用。`candidate_indexes` 的过滤条件（同一版第 2346 行）：

```
            let walked = *index == newest_index || (!abandoned && !below_floor);
```

**它的输入 `roots` 是「根环里读得出的根槽」**。「由记录施加出来、根槽从没落盘」的那一版**一个根槽都没写过**，所以它既不是 `abandoned` 也不是 `below_floor`——它压根进不了 `roots`，这两个理由一个都用不上。⇒ **判红的那一句是等式左右两边的定义差**：
- 左边（分配器 / 记账）：「占着、还不能再分配的槽」；
- 右边（checker）：「根环里读得出的、且过了实例表与 F 两道筛的那些根引用的并集」。

kb 里这条读法的原文（`.claude/kb/invariants.md` 第 125 行，I-3.1 那一行，整行太长，这里只指位置不做短版本）：正文是「已分配空间统计 == 实际遍历所有引用得到的和」，⚠️ 注里把「实际遍历所有引用」定成「按根环里全部有效根的引用取并集」，**2026-09-17 起「有效根」= 回退候选集里的根**。
**那条注里没有任何一个分句讲「根槽从没落盘的那一版」**——它只讲了两种落选（实例表判抛弃、低于 F），而这一格是第三种：**从来没进过根环**。这就是条款的洞，也是判红的那一句。

实现侧同一处洞有一行自白，`crates/singlefs-core/src/transaction.rs` 第 2300 行（主树现查，与快照逐字节相同）：

```
        // 第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。
```

括号里那句「**被根环里的有效根引用**」正是分配器为「defer 的仍算已分配」给出的理由。在这 12 个状态上这句话**是假的**：那四个单元在根环里没有任何有效根引用。两边用同一个假前提，一边靠它记账、一边靠它遍历，结果必然差 65 536 字节（4 槽 × 16 384）。

### 2.3 候选改法 c：「记账 ≥ 遍历」——当场打穿

正文没给 Z3 的候选改法清单，我按「把判据放宽到这几个状态不再判红」这条最省事的路子列了四个（c、f、e、b），逐个攻。

**改法 c**：`judge("I-3.1", allocated.is_some_and(|allocated| allocated >= walked), …)`（副本 `walk.rs.fix-c`）。

**第一半（误报消失）——量过**：

```
$ nice -n 19 cargo test -j 8 -p singlefs-harness --test second_transaction_step_zero_layer0 -- residual_record --nocapture
RESIDUAL_RECORD states=31 states_whose_chain_reaches_the_residual_record=19 violations=0 checker_by_invariant(evaluated/violated/not_applicable) … I-3.1=31/0/0 …
assertion `left == right` failed: I-3.1 判违例的状态数：None
  left: 0
 right: 12
```
（用例里钉着现状的 12，所以断言红；`I-3.1=31/0/0` 就是「31 个状态都判了、0 个判红」——误报确实没了。）

**第二半（改完仍会红的坏镜像）——交不出，我造的那一份由红转绿**：

坏镜像 Z3-A（副本 `checker_known_bad_images.rs` 尾部的 `z3_attack_a_leak_that_keeps_the_free_plus_allocated_equation`）：干净镜像上，记账树里盘 0 的**「已分配」+16 384、「空闲」−16 384**，重封记账单元。
它是什么错：**一整槽的可分配空间凭空消失、没有任何单元在用它**——纯泄漏，而且是真实泄漏的形状（分配器为一个没人引用的槽同时减了空闲、加了已分配）。
为什么只有 I-3.1 管得住它：D5 已定项 4 明写「**第 2 项（空闲字节）必须独立维护，不许由 `容量 − 已分配` 现算**」，所以 I-5.2 的「空闲 + 已分配 = 单元区」在这份镜像上照旧成立。

今天的等式（`walk.rs.today`）：

```
Z3_ATTACK I-3.1=Violated("盘 0：记账的已分配 Some(229376)，遍历全部有效根得到 212992；机理：根环槽数 24、最新根 txg 3、环里自证过的根槽 4 个、最老的自证过的根 txg 0、遍历的候选根槽 4 个、被实例表判抛弃的根槽 0 个、回退下界 F 0、低于 F 的根槽 0 个") I-5.2=Holds all_violated=["I-3.1"]
```

换成改法 c（`walk.rs.fix-c`）：

```
Z3_ATTACK I-3.1=Holds I-5.2=Holds all_violated=[]
```

`all_violated=[]` 是逐条不变量扫出来的，不是我挑的：**全仓 38 条已实现的不变量一条都不红**。
同一次跑里仓里原有的那份 I-3.1 坏镜像也跟着哑：

```
thread 'the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target' panicked at crates/singlefs-harness/tests/checker_known_bad_images.rs:1403:9:
I-3.1 那份坏镜像应当判违例，实际 Holds
test result: FAILED. 9 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.97s
```

⇒ **改法 c 按判据第 2 条算「没验过」，而且是最坏的那种没验过：它交不出任何一条改完之后仍会红的记账坏镜像**，因为它放弃的正是 I-3.1 唯一的方向（记账多于遍历）。I-3.1 的另一个方向（记账少于遍历）由 I-5.1 与 I-2.1 部分罩着，多算这一侧**全仓只有 I-3.1 一条**。

### 2.4 候选改法 f：「遍历 ≤ 记账 ≤ 遍历 + defer 队列字节」——也当场打穿

这是 c 的「聪明版」，而且不用新造统计量：`STATISTIC_DEFER_QUEUE_BYTES = 5` 已经逐盘落盘了（`crates/singlefs-core/src/transaction.rs` 第 2301–2304 行，值是 `device_map.deferred_slots() * SLOT_BYTES`）。改法：

```
            let defer_queue_bytes = accounting.get(&(5u16, *device)).copied().unwrap_or(0);
            judgements.judge("I-3.1", allocated.is_some_and(|allocated| allocated >= walked && allocated - walked <= defer_queue_bytes), || {
```
（副本 `walk.rs.fix-d`）

**第一半——量过**：残留记录那条流同样 `I-3.1` 判红 0 个（断言 `left: 0 / right: 12` 同上）。

**第二半——交不出**。两份坏镜像都绿：

```
Z3_ATTACK_TWO allocated=229376 free=3472654336 defer_queue=32768 violated=[] I-3.1=Holds
Z3_ATTACK I-3.1=Holds I-5.2=Holds all_violated=[]
```

- Z3-B（`z3_attack_two_a_leak_hidden_inside_an_inflated_defer_queue_row`）是「泄漏 + 把 defer 行一起抬高 16 384」：不等式的**松弛量由被测那一方自己供给**，分配器只要在同一个错误路径上既漏掉一槽、又把它算进 defer 计数，这条判据永远绿。
- 更糟的是 **Z3-A 也绿**——它一个字节都没碰 defer 行。原因量出来了：干净镜像上 `defer_queue` 本来就是 16 384（第一个事务之后第 0 版树表单元在 defer 里，`.claude/kb/invariants.md` 第 125 行那条注讲的就是这一槽），**松弛量从第一天起就够藏一整槽**。

⇒ 改法 f 同样「没验过」。它把一条等式换成一条不等式，而不等式的上界是没有任何检查管的一个数（`defer_queue_bytes` 全仓没有对应的不变量：`.claude/kb/invariants.md` 第 130 行 I-3.4「可用空间扣待删占用」状态列写着「未实现」，第 175 行 I-5.3「报出的空闲都兑现得了」也写着「未实现」）。

（「全仓 38 条」是数出来的：`crates/singlefs-checker/src/image.rs` 第 36 行 `pub const IMPLEMENTED_INVARIANTS: [&str; 38] = [`。）

### 2.5 候选改法 e：「有只由记录施加出来的版本时报不适用」——推的，交不出第二半

checker 只看一份镜像，但它认得出这个条件：环里有记录点名的 `(实例, txg)` 在根环里找不到对应根槽。改法 e 就是在这种状态上 `judgements.not_applicable("I-3.1", …)`。

**第一半**：12 个状态变成「不适用」，按 `Layer0Tally` 的口径不进 `checker_violated_states`，误报消失（**推的**，我没实现这一版；按 `walk.rs` 里 `not_applicable` 的已有用法推，`check_pool_image` 对每条不变量只有 Holds / Violated / NotApplicable 三态）。

**第二半交不出，理由是盲窗的长度**：残留记录不是一闪而过的东西——它留在环里直到被后来的记录轮转覆写。环的默认大小是 `JOURNAL_RING_DEFAULT_BYTES = 768 * 1024 * 1024`（`crates/singlefs-format/src/lib.rs` 第 162 行）、一条记录 `JOURNAL_RECORD_BYTES = 4096`（同文件第 139 行）⇒ 默认几何下 196 608 条。⇒ **从残留记录产生那一刻起，到它被后来的记录挤出环为止，I-3.1 在这块盘上一次都不判**；这个窗口有多长由环几何定，不由这一格定。把 Z3-A 那份泄漏坏镜像放进这个窗口里，它同样不判——不是判绿，是根本不判，而「不适用」在今天的层 0 汇总里与「判过、没红」一样不会引起任何人注意。
标注：**只在纸上量过、被攻过零轮**；要坐实得实现 e 再跑一次残留记录那条流 + 一份带残留记录的泄漏坏镜像，我这一轮没做。

### 2.6 候选改法 b：「把由记录施加出来的版本也并进并集」——唯一交得出第二半的方向，但代价在别处

改法 b：checker 扫环里的记录，把「记录点名、而且按 D23 已定项 14 的六条前缀判定该被施加」的那些单元也并进 `walked`。

**这是四个里唯一没被我打穿的方向**：它不放宽等式，而是把等式右边补全——补的正好是 2.2 里指出的那个缺口（「从来没进过根环的那一版」）。Z3-A、Z3-B 两份泄漏坏镜像在它上面照旧红（**推的**：那两份镜像的环里没有多余记录，`walked` 一个字节都不变）。

**它的代价（我攻得动的那一面）**：
1. **它把 D13 已定项 7 的分界推翻了一半。** 原文（`.claude/kb/decisions/13-验证路线.md` 第 127–128 行）：
   > **定案：O2（独立解析器 + checker） 的定义域是单个镜像**——一个一元谓词 `P(镜像) → 成立/不成立`，
   > 只判集合成员，永不判 identity。**任何需要第二个输入（另一个镜像、或一段记录流）的检查都不属于它。**

   环里的记录**在**那一份镜像里，所以形式上不违反；但「按六条前缀判定算出该施加哪些记录」就是把恢复算法抄进 checker，**checker 与被测的恢复从此共用同一个前缀判定**——错在前缀判定上的 bug 两边一起错，I-3.1 在这一类上判别力归零。这一条今天有人管：`.claude/kb/invariants.md` 第 125 行那条注里 checker 的候选集**故意只读根环与实例表，不读记录**。
2. **判据第 3 条的提醒**：b 要新写条款（「记录施加出来的版本算不算有效根」），不是一处实现选择。

⇒ b 我**交不出**「改完之后仍该红、而它不红了」的历史，照分工表的要求说明为什么这一族不存在：b 不放宽任何东西，它只加项；加项只可能把「记账少算」这一侧变松，而那一侧本来就由 I-5.1（引用不重叠）与 I-2.1 兜着。**代价不在漏判，在把两条独立路径合成一条**——这是另一类账，不是判据第 2 条的账。

### 2.7 Z3 小结

| 改法 | 误报消失 | 仍会红的坏镜像 | 判据第 2 条 | 我用的是量的还是推的 |
|---|---|---|---|---|
| c（记账 ≥ 遍历） | 是（12 → 0） | **交不出**（Z3-A `all_violated=[]`） | 没验过 | 两半都量过 |
| f（记账 − 遍历 ≤ defer 行） | 是（12 → 0） | **交不出**（Z3-A、Z3-B 都绿） | 没验过 | 两半都量过 |
| e（报不适用） | 是 | **交不出**（盲窗跟着残留记录活满一圈环） | 没验过 | 两半都是推的 |
| b（并集补上记录施加出来的版本） | 是 | 交得出 | 这一轮没打穿 | 推的；代价另算（D13 已定项 7 的分界） |

**给主 agent 的一句**：Z3 这一格真正要定的不是「I-3.1 怎么改」，而是**「已分配」这个词在两处的定义要统一成哪一个**。今天分配器用「占着、不能再分配」，checker 用「有效根引用的并集」，两者只在「每个 defer 的单元都被候选集里某条根引用」这个前提下相等——`transaction.rs` 第 2300 行那行注释把这个前提写下来了，而残留记录那条流正好证伪它。

## 三、Z4：那 8 个状态合不合法、判据错在哪一句、候选改法够不够

### 3.1 合法

陈旧 tail 那条流里，A（txg 3）那条记录点名的单元 50180 被 txg 18 复用，而复用之前已经抬过 F（脚本 `ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish` 接在 `ReuseAfterRaisingFloor` 之后）。合法性的条款出处同 Z3 的第二条：
`.claude/kb/decisions/23-journal的角色与格式.md` 第 342 行已定项 14 的定案句（上面已整段抄过）——**陈旧 tail 只是「从哪开始扫环」的优化，不决定重放集合**，所以 tail 之下那条记录点名的单元对不上不是错。
跑出来的旁证（`stale_tail_with_a_reused_named_unit…` 这条用例自带的断言）：8 个状态上 `violations=0`、`failed_states=0`、`verification_failed_states=0`，而且**改坏 tail 之后恢复的终态与 tail 没改坏时逐项相同**。

### 3.2 判据错在哪一句（判据第 1 条）

改法落地之前那一版是（变异表第 191 行的「改回去」列，`crates/mutations.tsv`）：

```
    let copy_is_missing = |copy: usize| !in_place(copy);
```

`in_place` 的定义（`crates/singlefs-harness/src/crash.rs` 第 653–659 行，这一份与开工快照逐字节相同）是「崩溃后镜像那个位置上的字节与**记录流里写下的**逐字节相同」。位置一旦让给后来的写，旧字节按定义就不在了 ⇒ `!in_place` 恒真 ⇒ 判红。
**就是 `!in_place(copy)` 这一个合取项**：它把「这份副本的字节不在盘上」直接当成「这次发布的这个单元缺席」，而两者只在「这个位置此后没被合法让出去」时等价。这一句在 `RecordCheck` 的字段注释里写得更清楚（第 592 行上一行，第 591 行）：

```
    /// 恢复实际走的根 txg ≥ 某次发布，而那次发布写出的某个单元两份都不在（E77（发布的持久顺序） 的独立审计）。
```

「两份都不在」这个措辞本身没错，错的是把「不在」实现成「旧字节不在这个位置上」。

### 3.3 ⚠️ 先报一件事实：仓里今天落地的**不是**正文里那条候选改法

里程碑收口表第 55 行（`.claude/kb/milestone/02-second-txn.md` 第 356 行）写的候选改法是「被更晚**已持久**的同位置写盖掉的不算缺席」，且「用例按现状钉成 8」。
**主树与开工快照里的 `crash.rs`（sha256 `90419bc8…`，两者逐字节相同）已经把它落地了，而且落的是一个更宽的版本**：

```
660:    let written_over_later = |index: usize| {
670:    let copy_is_missing = |copy: usize| !in_place(copy) && !written_over_later(copy);
```

`written_over_later` 的函数体（第 660–669 行）只判「后面还有一次写，同一块盘、区间相交」——**它不看那次更晚的写有没有落盘**。文档里也是这么写的（第 645–646 行）：

```
/// 一份单元副本在这条记录流后面又被别的写盖过时，它不在盘上算不得缺席（那是合法的复用：位置让给了后来的写，
/// 旧字节本来就不该还在）——一次发布的某个单元要全部副本都「不在而且没被盖过」，才算「两份都不在」。
```

「**被别的写盖过**」与收口表写的「被更晚**已持久**的同位置写盖掉」不是同一句话。下面两小节分别攻这两个版本，记作 **L**（落地的）与 **P**（收口表写的）。

### 3.4 打穿 L：一条改完之后仍该红、而它不红了的历史

**装置**：副本 `second_transaction_step_zero_layer0.rs` 尾部的 `z4_attack_a_later_write_that_never_persisted_excuses_a_genuine_missing_unit`。

**写序列**：固定脚本 `ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish` 的录制流，一个字节都没改。
**故障点**：持久集合 = 「写下标 ≤ 30（A 的根槽 FUA）」**减去**落在 50180 那一对单元写（下标 12、13），下标 > 30 的一律没持久。翻成人话：**A 这次发布的数据单元两份一份都没落盘，而它的 journal 记录与根槽已经落盘；后面 txg 18 在同一对槽上的那次写，一个字节都没落盘。**
**期望读数**：这是 E77 那条判据要抓的标准形——「恢复自称的 txg ≥ 某次发布、那次发布的某个单元两份都不在」。盘上那个位置是 mkfs 之后的旧字节，A 的数据**真的没了**。

**实测（副本，三个版本各跑一次，`--nocapture`，原样贴）**：

```
===== variant=landed
Z4_ATTACK a_root_index=30 first_reuse_unit=12 copies_at_the_reused_offset=[12, 13, 279, 280] effective_root=Some((InstanceGeneration(1), CheckpointTxg(3))) outcome=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(3))), failure: MappingStillUnreadable { slot: SlotNumber(50180) } } violations=1 record_claimed_state_missing_unit=0 record_root_without_record=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 2.45s
===== variant=prefix
Z4_ATTACK a_root_index=30 first_reuse_unit=12 copies_at_the_reused_offset=[12, 13, 279, 280] effective_root=Some((InstanceGeneration(1), CheckpointTxg(3))) outcome=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(3))), failure: MappingStillUnreadable { slot: SlotNumber(50180) } } violations=1 record_claimed_state_missing_unit=1 record_root_without_record=0
assertion `left == right` failed: 记录核对器两条判据在已定的持久顺序下恒 0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 2.27s
===== variant=variantP
Z4_ATTACK a_root_index=30 first_reuse_unit=12 copies_at_the_reused_offset=[12, 13, 279, 280] effective_root=Some((InstanceGeneration(1), CheckpointTxg(3))) outcome=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(3))), failure: MappingStillUnreadable { slot: SlotNumber(50180) } } violations=1 record_claimed_state_missing_unit=1 record_root_without_record=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 3.02s
```

读数：
- 三个版本上盘面完全一样：`effective_root=(1, 3)`（= A 那次发布自己），`outcome=Failed { … MappingStillUnreadable { slot: SlotNumber(50180) } }`，oracle `violations=1`。**这是一个真洞，不是我编的**。
- **L（落地的）`record_claimed_state_missing_unit=0`** —— 改法把它放过去了。
- 改法之前（prefix）`=1`：这一格原来抓得住。
- **P（收口表那条）`=1`**：P 抓得住。

第一段那次跑还顺带贴出这一格的 checker 读数（同一次输出的后半段）：`I-2.1=1/1/0 I-4.8=1/1/0 I-7.2=1/1/0 I-7.4=1/1/0` —— 走树那条路的四条不变量都红。
⇒ **L 丢掉的不是「有没有人报」，是记录核对器这条独立审计路径**：它存在的全部理由就是不解析树、不走恢复（`.claude/kb/decisions/13-验证路线.md` 第 127–128 行的定案，上面整段抄过）。在这一格上，两条路径只剩一条。

**这条历史可达吗**：它的持久集合是「前缀 + 段内子集」之外的形状（下标 12、13 没持久而 14–30 持久），层 0 的段枚举造不出来。**但这不改变判定**——判据第 2 条问的是「这个改法改完之后还有没有仍会红的坏历史」，而 L 的问题是它把一个**原本判得出**的真洞变成判不出，且这个真洞与仓里已有的真阳性用例（`second_transaction_step_zero_layer0.rs` 第 633 行那一格「② B 的根槽已持久、B 的十六个单元写一份都没持久」，断言 `record_claimed_state_missing_unit == 1`）是**同一类错**，唯一的差别是这个槽在流里后面被写过。⇒ **以后固定脚本只要复用流里写过的槽，这一类真洞在那个槽上就永远报不出来**——与收口表第 55 行说的误报方向正好对称的一句。

### 3.5 P 过了判据第 2 条——两半都量过

**第一半（8 个误报消失）**：换上 P 之后整份用例文件全绿，`stale_tail_with_a_reused_named_unit…` 里那句 `assert_checker_and_record_checker_counts(&tally, &[])`（要求记录核对器两条判据都恒 0）过了：

```
running 9 tests
test targeted_controls_on_the_second_publish_go_red_where_they_should ... ok
test z4_attack_a_later_write_that_never_persisted_excuses_a_genuine_missing_unit ... ok
test residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it ... ok
test stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state ... ok
test every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims ... ok
test one_state_slices_on_eight_threads_merge_into_the_same_tally_and_observation_order_as_one_slice_on_one_thread ... ok
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 12.36s
```
（这一次跑的是加了我两条攻击用例之后的文件，所以是 9 tests / 8 passed + 1 ignored；`targeted_controls_on_the_second_publish_go_red_where_they_should` 里就有 3.4 提到的真阳性那一格。）

**第二半（改完之后仍会红的历史）**：就是 3.4 的 Z4-A，P 上 `record_claimed_state_missing_unit=1`。

⇒ **P 是这一轮唯一同时交出两半的候选改法**。P 的一种最小实现（副本 `crash.rs.variantP`，把 `written_over_later` 的闭包体改成）：

```
        writes[index + 1..]
            .iter()
            .enumerate()
            .any(|(offset_in_tail, later)| {
                let later_start = later.offset.0;
                let later_end = later_start + later.length_in_bytes();
                later.device == write.device
                    && later_start < end
                    && start < later_end
                    && in_place(index + 1 + offset_in_tail)
            })
```

**标注**：这个实现是**我自己提的收严，只在我的副本上量过、被攻过零轮**（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节）。它改的那几格：

| 格 | 量过还是推的 | 读数 |
|---|---|---|
| 陈旧 tail 那条流 8 个状态 | **量过**（副本） | 记录核对器两条判据都 0，整份用例文件 8 passed / 1 ignored |
| 真阳性那一格（根槽持久、16 个单元写都没持久） | **量过**（副本，在上面那次全量里） | `targeted_controls_…` 绿 ⇒ 它里面 `record_claimed_state_missing_unit == 1` 的断言过 |
| Z4-A（我造的真洞） | **量过**（副本） | `record_claimed_state_missing_unit=1` |
| Z4-B（见 3.6） | **量过**（副本） | `record_claimed_state_missing_unit=0` ⇒ P 也漏 |
| 崩溃注入那一路（`crash_injection.rs` 第 707 行调 `check_records_against`） | **推的** | 那一路的 `writes` 是到当前段为止的前缀、`reader` 是另一份镜像；`in_place` 的口子本来就按 `reader` 读，P 不改这个口子 ⇒ 按代码推不变，**没跑** |

### 3.6 P 还剩什么：更晚那次同位置的写**已落盘**、而恢复走的根仍是旧的那一版

**装置**：副本同一份文件尾部的 `z4_attack_two_a_persisted_later_write_at_the_same_offset_also_excuses_the_hole`。
持久集合 = 3.4 那一份，**再把下标 279、280（txg 18 落在 50180 的两份）打开**。

```
Z4_ATTACK_TWO later_copies=[279, 280] effective_root=Some((InstanceGeneration(1), CheckpointTxg(3))) outcome=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(3))), failure: MappingStillUnreadable { slot: SlotNumber(50180) } } violations=1 record_claimed_state_missing_unit=0
```
（P 上判绿；prefix 版上同一格 `record_claimed_state_missing_unit=1`。）

**这一格说明 P 的盲点在哪**：「被更晚已持久的同位置写盖掉」只证明**位置让出去了**，没证明**让得合法**。让得合法要另一个前提——那次复用发生在「所有还能被恢复选中的根都不再引用这个单元」之后（D16 已定项 1 的回退下界 F）。P 不查这个前提，于是 **C22（刚释放的块立即重分配）那一类错在这条判据上从此静默**：一次过早的复用，只要它自己落了盘，就替被它毁掉的那个单元开脱。

**可达性我不写死**：Z4-B 的持久集合不是段模型枚举得到的（下标 279、280 持久而中间的段不持久），所以**这一格本身在层 0 那条路上走不到**。我没造出走得到的同形——要造得有一条「不抬 F 就复用」的脚本，仓里今天的五个 `Script` 分支里没有（`ReuseAfterRaisingFloor` 与 `ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish` 两条都先抬 F）。⇒ **这条我报成「判据的入参空间里有这个盲点」，不报成「今天走得到」**；要坐实，得先有那条脚本。

## 四、没打中的形状

逐条写试过什么、取样多大，不写成「没发现问题」：

1. **Z4 判据的另一半 `root_without_record`**：没攻。它不在正文给我的两格里（收口表第 55 行只点第二条判据）。
2. **Z3 改法 b（并集补上记录施加出来的版本）**：我**没能**构造出「b 改完之后仍该红、而它不红了」的坏镜像或历史。试过的形状两族：① 把泄漏藏进「记录点名的单元」里——不成立，b 只会把 `walked` 加大，加大之后等式左边（记账）不变，泄漏仍然是 `allocated > walked`；② 让记录本身撒谎（记录点名一个没人用的槽）——这会让 `walked` 虚高、等式转向「遍历多于记账」那一侧，那一侧红得更早，不是漏判。⇒ 这一族**不存在**的理由在 2.6 写了：b 不放宽任何东西。b 的代价另算（把 checker 与恢复的前缀判定绑成一条路径）。
3. **在层 0 段模型里造 Z4-A 的同形**：没造出来。取样范围 = 固定脚本 `ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish` 的**全部 57 个段**，逐段数「同时含根槽写与单元写」的段有几个，**量出来是 0**（原样输出）：

```
Z4_SEGMENTS a_root_index=30 segment_of_a_root=Some(8) segment_of_copy12=Some(6) segment_of_copy13=Some(6) same_segment_root_and_its_own_units=0 segment_sizes=[2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 10, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 18, 2, 1, 18, 2, 1, 18, 2, 1, 18, 2, 1, 10, 2, 1, 10, 2, 1, 18, 2, 1, 18, 2, 1, 2]
```
根槽写一律自成一段（那些 `1`）。段内子集任意、段间严格前缀 ⇒ 根槽持久时它自己的单元写必然全持久 ⇒ **层 0 那一路根本产生不出 `claimed_state_missing_unit`**。真阳性那一格（第 633 行的 ②）是手搭 `persisted` 向量、不走段枚举的。**这一条顺带推翻了一个可能的反驳**：「L 反正在层 0 上不会漏判」——是的，因为层 0 上它也从来不会判。这条判据的判别力全靠手搭状态与崩溃注入那一路。
4. **崩溃注入那一路（增补 3 第 3 件，`crash_injection.rs` 第 707 行）**：一次都没跑。L / P 在那一路上的差别我只按代码推（见 3.5 的表），**没有实测**。
5. **Z3 改法 e 的实现**：没实现、没跑，只在纸上推。
6. **不等盘上的 Z4**：`check_records_against` 里 `copies_by_offset` 只按 `write.offset.0` 分组、**不带设备维**（`crates/singlefs-harness/src/crash.rs` 第 680–689 行）。两块盘上同一个单元落在不同偏移时，每个偏移各成一组，`.any(组内全缺席)` 于是变成「**任一**副本缺席就红」，而判据字面是「两份都不在」。这一条我**只读了代码、没跑**（今天的固定脚本两盘同槽，造不出差别），而且它落在 Z2 的题面上（盘不等大时各盘落点可以不同），**不归我判**，只登记一笔。

## 五、这条腿自己的限度

- **副本上的数不入库**。全部读数来自 `/tmp/claude-1000/placement-opus/repo`（`CARGO_TARGET_DIR=/tmp/claude-1000/placement-opus/target`），debug profile。主 agent 要引，得在入库装置上重做一次。
- **walk.rs 不是快照那一版**（见第一节的 ⚠️）。Z3 的三条读数（今天的等式、改法 c、改法 f）都跑在 `6220c45b…` 上；我拿到主树这一版时它与快照 `136e5704…`、与 `HEAD` 的 `9f17ae43…` 都不同，写报告时主树又变成 `d686e2c5…`。**如果实现线这几天动了 I-3.1 的候选集或那条 `judge`，Z3 的三条读数要重跑**。`crash.rs` 与快照逐字节相同（`90419bc8…`），Z4 的读数不受这一条影响。
- **我提的 P 的实现被攻过零轮**，而且它自己还有 3.6 那个盲点。**我不建议把 P 当成这一格的终点**：P 只把判据从「旧字节还在不在」挪到「位置有没有被合法让出去」的一半，另一半（让得合不合法）它不查。
- **「没打中」只抽了一次样**（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」）：第四节第 2 条「b 这一族不存在」是一次推理的结论，不是两次独立观测，按那一节的门槛**不足以采信**。
- 我没判 Z1、Z2，也没判本地攻方那张表；没替主 agent 采纳任何改法。

## 六、什么现象会推翻这份报告的结论

| 结论 | 推翻它的观测 |
|---|---|
| Z3 的 12 个状态合法 | 找出一条分项，明写「只由记录施加出来、根槽从没落盘的那一版」不许存在，或它的单元不许算已分配 |
| 改法 c / f 没验过 | 有人交出一条 c 或 f 改完之后仍会红、而且是「记账多于遍历」那一侧的坏镜像 |
| f 的松弛量从第一天起就够藏一整槽 | 干净镜像上 `defer_queue_bytes` 量出来是 0（我量到的是 16384） |
| L 比 P 宽、宽掉的是一整类真洞 | Z4-A 在 L 上量出 `record_claimed_state_missing_unit=1` |
| P 过了判据第 2 条 | Z4-A 在 P 上量出 0，或 8 个陈旧 tail 状态在 P 上有任何一个判红 |
| 层 0 那一路产生不出 `claimed_state_missing_unit` | 找出一个段，里面同时有某次发布的根槽写与它自己的单元写 |

## 七、没做什么

- 没写 kb、没写用例进主树、没改主树任何一个字节；这一轮我只写了 `research/prompts/m2-placement-falsepositive-r1-opus-output.md` 一个文件。
- 没跑 `gate.sh`，没跑 `.claude/gate.d/stage-owners.tsv` 里的任何阶段（按 `.claude/agent-common.md`「门禁」一节查过：`three-way-attack` 没有登记给我的阶段）。
- 没跑层 0 全量（`full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean` 标着 ignored，debug 下半小时以上），没跑崩溃注入那一路，没跑变异表。
- 没开吃满 32 核的跑法：一律 `nice -n 19 cargo test -j 8`。
- **草稿产物为什么不入库**：`/tmp/claude-1000/placement-opus/` 下的仓副本、六份改过的源码（三份 `crash.rs`、三份 `walk.rs`）与两份攻击用例，都是副本上的东西，按派发提示「副本上的数不入库，主 agent 会在入库装置上重做」。要留的话，值钱的是那**四条攻击用例**（Z3-A、Z3-B、Z4-A、Z4-B），它们是判据第 2 条要的「仍会红的坏镜像 / 历史」的可执行形态；路径见第一节的 sha256 表。**主 agent 不拷的话，这台机器重启就没了。**
