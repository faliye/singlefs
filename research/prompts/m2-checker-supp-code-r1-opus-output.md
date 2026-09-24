# m2-checker-supp-code-r1 云端攻方腿（Opus）：K1 事务号改法的射程 / K2 它买到了什么没买到什么

攻的是 K1、K2 两面。**不判 K3（checker 判据本身）、K4（早退条件）**，那两面归正推腿与本地攻方腿。

时区：下面写的时刻都是 UTC（本机时钟），人在东京是 UTC+9。

## 复跑命令与 sha256

副本：`rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ /tmp/claude-1000/m2-checker-supp-r1-opus/repo/`，
基准是 HEAD `11a551b` 加这一批未提交的改动。开工与收尾各跑一次
`sha256sum -c research/prompts/m2-checker-supp-code-r1-start-snapshot.sha256`，九个文件全部 OK——主工作区一个字都没动。

```text
e14bd3005ab3460d180b47313e2cc4c1a4bcda436cc303601644a773c4229c35  research/prompts/m2-checker-supp-code-r1-opus-model/opus_probe_txn_chain.rs
b93285f6a5b9f47b6a55b9b2c59b112f6c4dab6ed3e9f31a9b6692f8644f6abc  research/prompts/m2-checker-supp-code-r1-opus-model/step_five_write_order_scan.rs.append
8d7ad9e97d779801935345f7383ceae84cbe6050a198b967bf89aff9e8c585c1  research/prompts/m2-checker-supp-code-r1-opus-model/README.md
```

两条复跑命令（在副本里跑，放法见 `README.md`）：

```text
nice -n 19 cargo test -p singlefs-harness --test opus_probe_txn_chain
nice -n 19 cargo test -p singlefs-harness --test second_transaction_step_five_reuse opus_probe -- --nocapture
nice -n 19 cargo test -p singlefs-harness --test opus_probe_txn_chain sweeping -- --nocapture
```

开跑前 `ps -o pid,args -u "$(id -u)"` 看过负载：没有 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` / `fio`
在跑；在跑的是另一个会话的 `doc-lint.sh`（pid 30346）与常驻的 vllm 网关。副本编译与跑测试一律 `nice -n 19`，没等过锁。

## 各格判定一览

| 格 | 问什么 | 判定 | 量过 / 推的 |
|---|---|---|---|
| K1-a | 还有没有第 8 处构造点 | **没有**：`crates/singlefs-core/src/` 里 `PublishPlan` 字面构造恰 5 处（`mount.rs` 691 / 746 / 774、`transaction.rs` 1200 / 1244），加 `transaction.rs:2192` 那一处收尾与 `recovery.rs:710` 的 `TransactionOutput`，正好是主 agent 说的 7 处；测试里 3 处（两处写死 0，逐个核过都对） | 量过（`grep -rnE` 计数） |
| K1-b | 传的值有没有错的 | **有一类没有字段**：`ZeroUnitPublishPlan` 三处（`mount.rs:800`、`mount.rs:1063`、`transaction.rs:488`）整条链上没有这个字段，`PoolVersion::WithoutFile` 这一侧的计数无声归 0；今天够不着，靠 `publish_first_file` 的 `follows_directly` 挡着 | 推的（读代码，没实测） |
| K1-c | 链在哪条路径上断 | **打中**：`publish_version` 的失败路径把链整个丢掉。根槽那一步报错 ⇒ 事务号 4 的记录（提交标记 1）已经持久，而同一个写入口拿同一个 `previous` 接着发布又拿到 4、jsn 也重号，实例没换 | 量过（探针①，副本上跑绿） |
| K1-d | `rebuild_version` 那句射程成不成立 | **今天成立，但靠一个没被任何东西钉住的前提**：`mount.rs` 两个公开入口都走 `establish_instance` 取新号，写行那次显式传 0；而「不会被拿去接着发布」没有类型、断言或用例钉着，`rebuild_version` 与 `publish_overwrite` 都是 `pub`，一次调用就重号 | 量过（探针②，副本上跑绿） |
| K1-e | C286（只读后 remount 取不取新实例代号） 成不成立 | **成立，而且正是这一句射程的下一张门**：`crates/` 里根本没有只读挂载入口（`pub fn mount` 只有 `mount_writable`、`mount_rollback`），26 处 `remount` 全是「再可写挂载一次」 | 量过（grep 计数） |
| K2-a | W 的前缀性今天成不成立 | **发布侧成立、判定侧没动也不用动**：算 W 的 `recovery.rs:958` 不在这一批的 diff 里，它本来就取 max；改法补回的是让 max 成为水位的前提 | 量过（grep 计数） |
| K2-b | 码 1 那半截 `n ≤ W` 分得开了吗 | **分得开了，有一格仍分不开**：步 5 那条脚本上八个码 1 单元的写序改后互不相同、改回旧写法只剩七个（重的是 `(3, 1)`）；仍分不开的那一格是 K1-c 的失败重试——**重的是号**（八格全中），**盘上留下两个同写序的版本这一格八格全不中**（重试原地盖掉） | 量过（写序扫描，带判别力自证） |
| K2-c | 已定项 19 ①「不进实例表 W 的 max」对不对得上 | **对得上**，但是靠「0 抬不高一个非负的 max」这条算术，不是靠实现里有一句把空发布排除 | 量过（读 `recovery.rs:957-958`） |
| K2-d | 这一批给 `n ≤ W` 添了第一个可执行消费者 | **是**：`walk.rs:1414` 是 `crates/` 里第一处真去算 `n ≤ W` 的地方；`recovery.rs` 的五处 I-1.2 前置检查一处都没算过写序与 W | 量过（grep 全部消费者） |
| K2-e | 这一格的覆盖有多薄 | 理想模型把 W 写死 0（`model.rs` 四处，理由逐字「不建崩溃，一条都不施加」）⇒ 模型对拍那一路永远见不到 W > 0；测试里断言非 0 的 W 只有 2 处 | 量过（grep 计数） |

---

## K1 事务号改法的射程

### K1-a / K1-b：每一个构造点

`crates/singlefs-core/src/` 里 `PublishPlan` 的字面构造，现查（`grep -rnE "(^|[^a-zA-Z_])PublishPlan \{" crates/singlefs-core/src/`）：

```text
crates/singlefs-core/src/mount.rs:691:            PublishPlan {
crates/singlefs-core/src/mount.rs:746:        PublishPlan {
crates/singlefs-core/src/mount.rs:774:            let plan = PublishPlan {
crates/singlefs-core/src/transaction.rs:1200:        PublishPlan {
crates/singlefs-core/src/transaction.rs:1244:        PublishPlan {
```

全仓（含测试）8 处，另外三处在 `second_transaction_supplement_two_accounting_node_full.rs:72`、
`second_transaction_supplement_two_row_publish_admission.rs:79`、`second_transaction_step_three_second_instance.rs:450`。
两处写死 0 的都核过：前者是「第一个文件版本形态的一次发布（txg 3、jsn 3、事务号 1）」，后者注释逐字
「新实例（2）自己的第一条记录，事务号按实例各算各的」——都对。

收尾那一处（`crates/singlefs-core/src/transaction.rs:2190-2192`）：

```rust
        highest_transaction_number_in_this_instance: plan
            .highest_transaction_number_before_this_publish
            .max(plan.transaction),
```

**没有第 8 处**。但有一类整条链上根本没有这个字段：`ZeroUnitPublishPlan`（`transaction.rs:512` 定义）三处构造
（`mount.rs:800`、`mount.rs:1063`、`transaction.rs:488`），它产出的 `ZeroUnitPublishOutput` 也没有这个字段，
于是 `PoolVersion::WithoutFile` 这一侧的计数无声地是 0。今天够不着，挡住它的是
`publish_first_file` 里的 `follows_directly`（要求上一条记录是 txg 2、jsn 2），而 txg 全池单调递增、
回退到树表 0 条的根在任何写之前被 `RollbackToVersionWithoutFileUnsupported` 拒掉。
**这一条是推的，没实测**：我没有造出一段能在 `WithoutFile` 之后再发第一个文件版本的历史。

### K1-c 打中：失败路径把链整个丢掉（探针①，副本上量过）

`publish_version` 的持久顺序是「单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换」
（`transaction.rs:2143` 那段注释）。链只活在**成功返回的** `TransactionOutput` 里：
`Err` 里一个字都没有，调用方手上只剩老的 `previous`，而那次失败已经把事务号写进了环。

探针①（`opus_probe_txn_chain.rs`）造的是最短的那段历史：实例 1 发第一个文件版本（事务号 1）、
覆盖写两次（2、3），再把下一次发布要写的那个根槽（`slot_offset(target_for_publish(txg 6), 4096)`）上的写注入块设备错。
副本上跑绿，逐格断言：

- 失败那次返回 `PublishError::BlockDevice`；
- 环里 jsn = `previous.record.counter + 1` 那一条解得开，`transaction == 4`、`is_commit == true`、`instance == 1`；
- 拿同一个 `previous` 接着发布：`record.transaction == 4`（同一个号第二次发出去）、
  `record.counter == previous.record.counter + 1`（jsn 也重号）、`root.instance == InstanceGeneration(1)`（实例没换）。

**这一格打中的是 `D23（journal 的角色与格式）` 已定项 7 自己那半句。** 原文整行（`.claude/kb/decisions/23-journal的角色与格式.md:196`）：

> - 记录按事务号顺序追加，一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加——这是实例表行的 W 能当精确前缀的依据（第一版串行提交下 W 的前缀性本来就成立，D13（验证路线） 已定项 2；这条纪律买的是并行提交打开之后它仍然成立）。

这一批补回了前半句（记录按事务号顺序追加），后半句仍是零代码：`grep -rn 实例切换 crates/ --include=*.rs` 零命中，
`.claude/kb/checks-owed.md:336`（C381（根已落盘之后发布失败，分配器仍退回））逐字写着
「**代码那一半整个仍欠**：`crates/` 里探针写、只读复核、实例切换、转只读四样都没有。」

### K1-c 的射程：今天为什么还没炸

同一段探针里接着量了两样：

- 单元区头 64 槽上解得开的码 1 单元：`(0, 50180, 1, 1, 3) (0, 50182, 1, 2, 4) (0, 50184, 1, 3, 5) (0, 50186, 1, 4, 6)`，
  两块盘各一份；写序 `(1, 4)` 的**只有一个**（槽 50186），就是重试那次自己写的——重试把失败那次的单元原地盖掉了。
- 重试之后 `recover(..., JournalPolicy::Consult)` 交回 `FileRead { root: (InstanceGeneration(1), CheckpointTxg(6)), … }`，读得回来。

所以号重了而盘面没坏，原因是重试连 jsn 计数器和落点一起重用、把上一次的痕迹整个盖掉。
**它不是一条独立的新缺陷线，是 C381 那条线上多出来的一格**：C381 第 336 行只写了分配器退回与槽复用
（「同一个写入口接着发下一次发布会把这次占过的槽再分出去」），**没有一个字写事务号也跟着重**，
而事务号这一格正是已定项 7 点名的「W 能当精确前缀的依据」。
⇒ 建议把这一格记进 C381 那一行（或另记一笔），**不建议**据此改这一批的代码：改法在这一格上不起作用。

### K1-d：`rebuild_version` 那句射程，逐条核 `mount.rs`

被判的那句在 `crates/singlefs-core/src/recovery.rs:726-730`：

```rust
        // 重建出来的这一版属于**旧**实例。今天每条恢复路径（普通挂载、回退、切换）之后都要取新实例代号，
        // 而事务号按实例各算各的、从 1 重新起（D23（journal 的角色与格式） 已定项 7），所以这个值不会被拿去接着发布——
        // 写行那次发布显式传 0。将来真要在同一个实例上续发，得由调用方扫环算出这个实例的最大非 0 事务号传进来，
        // 这里这条记录上的事务号在它是空发布时是 0，单独拿它续号会重号。
```

逐条核的结果，**这句话今天成立**：

| 要核的 | 现查 |
|---|---|
| 重建的那一版被谁拿走 | `rebuild_version` 在 `crates/singlefs-core/` 里只有一个调用点：`mount.rs:229`（`rebuild_previous_version`）；它又只有两个调用点：`mount.rs:1144`（`mount_writable`）与 `mount.rs:1272`（`mount_rollback`） |
| 这两条路取不取新号 | 两条都把重建的那一版交给 `establish_instance`（`mount.rs:1171`、`mount.rs:1305`），而 `establish_instance` 在 `mount.rs:1017` 走 `acquire_expected_instance`，之后 `assert_eq!(instance, instance_to_acquire, …)` |
| 写行那次传什么 | `mount.rs:749-751`：`transaction: 0`，注释「新实例的第一条记录（反向链恒 0），事务号按实例各算各的，从这里重新从 1 起」，`highest_transaction_number_before_this_publish: 0` |
| 「切换」那条路 | `crates/` 里没有实例切换（`grep -rn 实例切换 crates/ --include=*.rs` 零命中，`.claude/kb/checks-owed.md:336` 与 `:405` 都逐字这么写）。这一句对那条路是空转的 |
| 别的公开入口会不会接着发 | `publish_overwrite` 在非测试代码里只有 `first_transaction_on_device.rs` 的三处与 `history.rs:2300` 一处，四处的 `previous` 都来自同一个进程里成功返回的那一版，没有一处喂的是重建出来的 |

**但它靠的前提没有任何东西钉着，而且一次公开 API 调用就能破。** 探针②（副本上跑绿）：
实例 1 发三个事务、重开取实例 2、实例 2 再发三个（事务号 1、2、3）、抬 F 到 5（推空发布，记录上写 0），
此时内存里的链记着 3；`rebuild_version(&devices, &current.root, Some(current.record.clone()))` 交回的那一版
`highest_transaction_number_in_this_instance == 0`；拿它当 `previous`、实例仍填 2 发一次覆盖写，
`record.transaction == 1`——**与实例 2 的第一个事务重号**，正是这一批要修的那个毛病原样回来。

`rebuild_version`、`RebuiltVersion`、`publish_overwrite`、`TransactionOutput::highest_transaction_number_in_this_instance`
四样全是 `pub`，中间没有类型、没有断言、没有一条用例钉住「重建出来的那一版不许接着发布」。
**改法把一个能从盘上重算的量换成了只活在内存里的累加量**：改前 `previous.record.transaction + 1` 从盘上读得出来（只是答案错），
改后正确的答案要扫环，而这一批明写「不扫环」。代价就是多了一条没写下来的前提。

另有一处注释这一批之后**不再成立**：`crates/singlefs-core/src/mount.rs:1135`

```rust
    // 事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。树表 0 条的根用不到记录（只做过 mkfs 的池环里一条都没有）。
```

`own_record` 现在还被 `recovery.rs:709` 取走 `record.transaction`。而 `mount.rs:1140-1143` 那个 `or_else` 在所选根自己那条记录读不出时
拿**环里 jsn 最大的那条**顶上，那一条可以属于别的实例 ⇒ 重建出来那一版的
`highest_transaction_number_in_this_instance` 可以是**另一个实例的**事务号。今天同样没有消费者，
但这句注释是读代码的人判「这个字段能不能信」的唯一依据。**这一条是推的，没实测**（我没造这段历史）。

### K1-e：C286 的线索核过了，成立，而且就是这一句射程的下一张门

`.claude/kb/checks-owed.md:254`（C286（只读后 remount 取不取新实例代号））那一行逐字写着
「**……而只读之后 remount 成可写算不算一次可写挂载、许不许，全仓没有一句**（2026-09-11 `grep -rn remount .claude/kb` 零命中）」。
现查：

- `grep -n "pub fn mount" crates/singlefs-core/src/*.rs` 只有两个：`mount.rs:1118 mount_writable`、`mount.rs:1196 mount_rollback`。**没有只读挂载入口。**
- `grep -rn remount crates/ --include=*.rs` 26 处，逐条看过，全是测试里「再可写挂载一次」的命名（`build_six_overwrites_after_a_writable_remount`、`remounted` 变量等），没有一处是「只读 → 可写」。

⇒ 今天 C286 那一格连对象都没有，所以它**推不翻** `rebuild_version` 那句射程。但两件事扣在一起看：
C286 自己那一行写的「最弱读法（remount 不取号）」一旦成真，remount 之后就是「同一个实例、从盘上重建的那一版接着发布」——
正好是探针②量出来的那条路。**这一句射程的有效期，等于 C286 与实例切换（C458（实例切换取内存里的根，不重新读盘））两笔欠账的有效期。**

---

## K2 它买到了什么、没买到什么

### K2-a：W 的前缀性今天成不成立，判定侧有没有跟着对

算 W 的地方是 `crates/singlefs-core/src/recovery.rs:957-958`：

```rust
        report.maximum_applied_transaction =
            report.maximum_applied_transaction.max(record.transaction);
```

它在 `replay_journal` 的施加循环里，而那个循环的候选已经按 `recovery.rs:895-899` 过滤成「只有所选根自己那个实例的记录」（第 898 行 `record.instance == root.instance && (record.instance, record.checkpoint_txg) > water`）。
**这一批的 diff 没有碰它**：`grep -c maximum_applied_transaction research/prompts/_m2-checker-supp-code-r1-diff.md` 交回 0，退出码 1（一处都没命中）。

判定侧不用跟着改，理由是算 W 的方式本来就是 max，而 max 能当水位靠的是「同一实例内事务号沿 jsn 严格递增」——
**那正是这一批补回来的那条前提**。改前空发布把计数拉回去，同一实例里同一个号出现两次，max 就不再是水位：
一个撕裂事务的单元写序可能 ≤ 某个更早的已施加号，`n ≤ W` 于是把垃圾判成已发布。

⚠️ 两条要写明的射程：

1. **W 不是「这个实例用过的最大事务号」，是「这次重放施加的最大事务号」。** 干净关机（所选根就是最新那条）一条都不施加，W = 0，
   这时 `n ≤ W` 恒假，判定全靠 `b ≤ T_pub` 那一半。W > 0 只在「记录已持久、根还没持久」那一类崩溃上出现。
2. 因此**改法买到的那一格，只有在 W > 0 的镜像上才看得见**。

### K2-b：码 1 那半截 `n ≤ W`，修掉之后分得开了吗

**分得开了。** 量的是步 5 那条固定脚本（回退之后四次覆盖写、抬 F 到 11、发布 E），
扫单元区头 256 槽上解得开的码 1 数据单元，按盘 0 列写序：

改法在（副本，原样输出）：

```text
回退之后四次覆盖写的事务号 [1, 2, 3, 4]，E 的事务号 5
盘 0 上码 1 单元 8 个，写序去重之后 8 个：[(1, 1), (1, 2), (2, 1), (3, 1), (3, 2), (3, 3), (3, 4), (3, 5)]
```

判别力自证——把 `transaction.rs:1247` 改回 `transaction: previous.record.transaction + 1,`，同一份扫描判红（原样输出）：

```text
回退之后四次覆盖写的事务号 [1, 2, 3, 4]，E 的事务号 1
盘 0 上码 1 单元 8 个，写序去重之后 7 个：[(1, 1), (1, 2), (2, 1), (3, 1), (3, 1), (3, 2), (3, 3), (3, 4)]
assertion `left == right` failed: 同一块盘上没有两个码 1 单元写序相同
  left: 7
 right: 8
```

⚠️ **由此带出一条要交给主 agent 现查的事**（我不判 K3，只把测到的数摆出来）：
这一批新写的注释 `crates/singlefs-checker/src/walk.rs:1430-1431` 逐字是

```rust
    /// 写序只在同 txg 跨事务改同 key 时顺带定序」。按前者判，同一实例的两次覆盖写在盘上就能撞出写序相同的两版
    /// （`second_transaction_step_five_reuse.rs` 那条脚本上真有一对），按后者判它们由诞生代号分得开。
```

按上面量到的数，**改法在的时候那条脚本上没有那一对**（八个写序互不相同），有那一对的是改回旧写法的那一臂。
这条注释与事务号那处改动在同一批未提交的改动里。C464（I-1.8 对码 1 的全序键写窄了） 那一行自己的题面是另一个场景
（`.claude/kb/checks-owed.md:411` 逐字「一个事务跨两个 checkpoint 时两次发布的记录事务号相同」，而今天
`publish_overwrite` 每次发布都加一、一个事务跨不了两个 checkpoint），并且那一行末尾已经写明
「与事务号重号那个实现 bug 是两件事，修了 bug 这一条照样成立」。⇒ 我只报**代码注释里那个被点名的具体证据今天量不出来**，
C464 本身成不成立归 K3 与主 agent。

**仍然分不开的那一格**：K1-c 的失败重试。失败那次把事务号 N 写进了环、单元也已经落盘，重试拿到同一个 N。
今天重试连槽一起重用、把上一版盖掉，所以盘上只剩一个写序为 N 的单元（探针①量到的就是这个）；
**重试本身再被打断**，盘上就会同时留下「记录说 N、单元是另一份内容」——那正是 C381 第 336 行已经登记的那一格。

### K2-c：已定项 19 ①「不进实例表 W 的 max」对不对得上

原文整行（`.claude/kb/decisions/23-journal的角色与格式.md:458`）：

> - **① 空发布记录**：事务号 0 保留给不承载事务的记录，提交标记写 1；不进实例表 W 的 max，不进「丢掉提交标记没出现的尾巴」判定。

对得上，两半都对，但两半的成立方式不一样：

- 「不进 W 的 max」：实现里**没有**一句把空发布排除在外，`recovery.rs:958` 照样把它的 `transaction` 喂进 `max`；
  对得上是因为那个值是 0、而 `maximum_applied_transaction` 从 0 起，**0 抬不高一个非负的 max**。
  结论一样，但这一格是算术挡住的，不是条款挡住的：哪天空发布的事务号换一个哨兵值（比如 `u64::MAX`），这里会无声地反过来。
- 「不进丢尾巴判定」：空发布写 `is_commit: true`（`transaction.rs:547`），永远不是那条没出现提交标记的尾巴。

### K2-d：这一批给 `n ≤ W` 添了第一个可执行消费者

实例表行里 W 的全部消费者，现查（`grep -rn applied_transaction_high_water crates/ --include=*.rs`，排掉 `tests/`）：

| 消费者 | 干什么 |
|---|---|
| `crates/singlefs-core/src/recovery.rs:501` | `rollback_high_water_of_root`：只取**回退行**的 W 当重放下界，不是 I-1.2 的谓词 |
| `crates/singlefs-checker/src/walk.rs:1414` | `PublishedPredicate::holds_for` 里码 1 那半截 `transaction <= row.applied_transaction_high_water_mark` |
| `crates/singlefs-core/src/mount.rs:973 / 980 / 1168 / 1302` | 写行时把 W 放进行里 |

`walk.rs:1414` 是这一批新加的（diff 第 204 行）。在它之前，`crates/` 里**一处都没有**真去算码 1 那半截 `n ≤ W`：
`recovery.rs` 挂 `"I-1.2"` 名字的五处（1015、1027、1276、1282、1369）逐个看过，比的是
`birth_txg > root.checkpoint_txg`、`instance`、`birth_sequence`，**一处都没读写序里的事务号、也没读行里的 W**。

⇒ 「重号修掉」与「`n ≤ W` 第一次真的被算」是同一批落地的。`.claude/kb/invariants.md:25` 里 I-1.2 的状态列仍写
「未实现（池级 checker 还没判；恢复路径把它当前置检查……）」——按 `walk.rs:1393-1394` 的注释，池级 checker 确实不**报** I-1.8 之外的 I-1.2 违例，
所以这句状态没写错；但「恢复路径把它当前置检查」这半句现在盖不住全部现状了，谓词自己已经有了第二个求值点。

### K2-e：这一格今天的覆盖有多薄

- 理想模型把 W 写死 0，四处（`crates/singlefs-harness/src/model.rs:1008 / 1055 / 1086 / 2006`），
  `model.rs:1007` 的理由逐字「W = 这次恢复施加的记录里最大的事务号（D23（journal 的角色与格式） 已定项 14 第 4 条）：不建崩溃，一条都不施加。」
  ⇒ **随机历史 + 模型对拍那一路永远见不到 W > 0**，也就永远碰不到 `n ≤ W` 这半截。
- 测试里断言非 0 的 W 只有两处：`second_transaction_step_zero_layer0.rs:782` 与
  `second_transaction_step_three_second_instance.rs:401`（都是 `applied_transaction_high_water: 3`）。
- 这一批新加的那条红证用例（`the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor`）
  钉的是**进程内**那几个事务号互不重复，没有钉盘上的写序、也没有钉行里的 W。

---

## 打中之后先答的四句

按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」办。这一轮被判的是一个实现，不是几条候选臂，所以第一句照「改法碰不到打中的格」那一形态答。

| 问 | K1-c（失败路径丢链） | K1-d（重建那一版的取值） | K2-b（写序分得开） |
|---|---|---|---|
| 分不分辨这一批的改法 | **不分辨**：改回旧写法同样中（旧写法取 `previous.record.transaction + 1`，失败之后 `previous` 一样没动）。它打的是改法之外的一格 | **分辨**：改前这个字段不存在，`rebuild_version` 交回的那一版拿去发布取的是记录上的号加一；改后多了一个只活在内存里的量，射程句就是为它写的 | **分辨**：改回旧写法同一份扫描判红 |
| 被判的系统当时看不看得到判别它的东西 | **看不到**。`PublishError::BlockDevice(cause)` 只带块设备错，**不带是哪一步失败的**；调用方无从知道「事务号 N 的记录到底落没落盘」。要它「失败之后不许再用 N」，等于要它区分一件它观测不到的事 ⇒ 这一格是「判别子观测不到」那一形态，真正的出路在已定项 7 后半句（失败即实例切换），不在这个取号表达式 | 看得到：字段就在手里，是没人规定不许用 | 看得到 |
| 满足的是判据字面的哪一个分句 | `D23（journal 的角色与格式）` 已定项 7 第 2 条的**后半句**「一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加」。不是前半句「记录按事务号顺序追加」——前半句这一批补上了 | `recovery.rs:726-729` 那条注释自己的第二句「所以这个值不会被拿去接着发布」 | 已定项 7 第 1 条「事务号按实例计数、从 1 起」与 `I-1.8（归并后版本全序）` 里「码 1 的键是写序」 |
| 这一批的改法在这一格上还中不中 | **还中**（改法碰不到） | 改法自己造出来的格 | 不中了（这正是改法买到的） |

## 我提的改法：只在我的模型上量过、被攻过零轮

| 改法 | K1-c | K1-d | K2-b 剩下那一格 |
|---|---|---|---|
| 甲：`PublishError::BlockDevice` 带上「失败在哪一步」与「这次用掉的事务号 / jsn 落没落盘」，调用方据此决定换不换实例 | **修**（推的：按代码推，没实现没跑） | 不修（推的） | 修（推的） |
| 乙：`rebuild_version` 不再填这个字段——交回的类型里它是「要扫环才知道」的那一种，让「拿重建的那一版接着发布」编译不过 | 不修（推的） | **修**（推的：按代码推，没实现没跑） | 不修（推的） |
| 丙：把已定项 7 后半句落成代码（失败即实例切换） | **修**（推的） | **修**（推的：新实例从 0 起，重建那一版的值再也不进发布路径） | 修（推的） |
| 丁：只补一条会红的用例，钉住「重建出来的那一版拿去 `publish_overwrite` 必须拒」 | 不修（推的） | **挡住**，不是修（推的） | 不修（推的） |

⚠️ 四条都**只在我副本上的探针里推过，一条都没实现、没跑**，按 `.claude/rules/three-way-inference.md`「攻方腿自己提的收严……算『没被攻过』」算**被攻过零轮**。
丙那一条不是新设计：它是 C381（根已落盘之后发布失败，分配器仍退回） 三轮三方已经判完、用户已定的那条线上的活，不该在这一轮顺手做。
我推荐的最小动作是：**这一批不改代码**，把 K1-c 记进 C381 那一行（事务号也在重用之列），把 K1-d 记成一笔新的小账（重建那一版的取值没有任何东西钉着）。

## 没打中的形状

| 试的是什么 | 取样范围 | 结果 |
|---|---|---|
| 让「失败 + 重试」在盘上真留下两个写序相同的码 1 版本 | 失败前发过 0 / 1 / 2 / 3 次 × 失败点落在根槽 / journal 记录，8 格全跑，每格扫两块盘单元区头 64 槽 | **没打中**：8 格里盘 0 的码 1 单元数与写序去重之后的个数**格格相等**，重试连 jsn 与落点一起重用、把失败那次原地盖掉。原样输出见下 |
| 让 `ZeroUnitPublishPlan` 那条没有字段的链造出重号 | 读了三处构造点与 `publish_first_file` 的 `follows_directly` 守卫 | 没打中，**没实测**：txg 全池单调、回退到树表 0 条的根在写之前被拒，我没造出第二次「`WithoutFile` 之后再发第一个文件版本」的历史 |
| 让 `instance_generation_to_acquire` 再发一个用过的号 | 读了 `transaction.rs:349-365` 与取号失败回卷那段注释 | 没打中，**没实测**：回卷之后那个号名下一条记录都没发过 |
| 让 `mount.rs:1140` 那个 `or_else`（所选根的记录读不出 ⇒ 拿 jsn 最大那条顶上）把别的实例的事务号带进重建的那一版 | 只读代码 | **没造历史**，列为推的；今天没有消费者，但 `mount.rs:1135` 那句注释因此已经不准 |
| 让抬 F / 写行 / 暖机三条空发布路径传出非 0 的事务号 | 读了三处，`transaction: 0` 都是字面量 | 没打中 |

失败点扫描的原样输出（副本，`cargo test -p singlefs-harness --test opus_probe_txn_chain sweeping -- --nocapture`）：

```text
失败前发过 0 次 / 失败点 TheRootSlot：失败那次报 BlockDevice；环里那条记录的事务号 Some(2)；重试拿到 2（该拿 2）；盘 0 码 1 单元 2 个、写序去重 2 个
失败前发过 0 次 / 失败点 TheJournalRecord：失败那次报 BlockDevice；环里那条记录的事务号 None；重试拿到 2（该拿 2）；盘 0 码 1 单元 2 个、写序去重 2 个
失败前发过 1 次 / 失败点 TheRootSlot：失败那次报 BlockDevice；环里那条记录的事务号 Some(3)；重试拿到 3（该拿 3）；盘 0 码 1 单元 3 个、写序去重 3 个
失败前发过 1 次 / 失败点 TheJournalRecord：失败那次报 BlockDevice；环里那条记录的事务号 None；重试拿到 3（该拿 3）；盘 0 码 1 单元 3 个、写序去重 3 个
失败前发过 2 次 / 失败点 TheRootSlot：失败那次报 BlockDevice；环里那条记录的事务号 Some(4)；重试拿到 4（该拿 4）；盘 0 码 1 单元 4 个、写序去重 4 个
失败前发过 2 次 / 失败点 TheJournalRecord：失败那次报 BlockDevice；环里那条记录的事务号 None；重试拿到 4（该拿 4）；盘 0 码 1 单元 4 个、写序去重 4 个
失败前发过 3 次 / 失败点 TheRootSlot：失败那次报 BlockDevice；环里那条记录的事务号 Some(5)；重试拿到 5（该拿 5）；盘 0 码 1 单元 5 个、写序去重 5 个
失败前发过 3 次 / 失败点 TheJournalRecord：失败那次报 BlockDevice；环里那条记录的事务号 None；重试拿到 5（该拿 5）；盘 0 码 1 单元 5 个、写序去重 5 个
```

这张表还顺带把 K1-c 收窄成了它该有的大小：**「重试拿到同一个号」八格全中，而「盘上留下两个同写序的版本」八格全不中**。
落在根槽的那四格里，那个号已经进了一条提交标记为 1 的持久记录（`Some(2/3/4/5)`），落在 journal 记录的那四格里没进（`None`）——
后四格重试拿同一个号是对的。⇒ **真正打中的只有「根槽及其之后那几步失败」这一族。**

## 这条腿自己的限度

1. **副本上量的数不算入库装置上的数。** 三份探针都只在 `/tmp/claude-1000/m2-checker-supp-r1-opus/repo` 上跑过；
   要引用得在入库装置上重做一次（跑前登记写明是核哪一句）。主工作区一个字没动，收尾 `sha256sum -c` 九个文件全 OK。
2. **只攻 K1、K2。** K3（I-1.8 / I-7.3 / I-8.6 三条判据本身）与 K4（早退条件）一个字都没判；
   K2-b 那一段量到的数会碰到 C464（I-1.8 对码 1 的全序键写窄了），我只摆数、不下 C464 成不成立的结论。
3. **没跑 `cargo test --all`、没跑层 0 全量、没跑门禁任何阶段。** `.claude/gate.d/stage-owners.tsv` 里
   `three-way-attack` 一个阶段都没登记（`awk` 过一遍，输出为空）。我在副本上加的那个扫描测试挂在
   `second_transaction_step_five_reuse.rs` 末尾，副本上那个文件因此与主工作区不同——**没有拷回主工作区**。
4. **「没打中」的几格多数是推的，不是实测的。** 上面那张表里标了「没实测」的三行，各自写了取样范围；
   按「一条腿的『没打中』一次不算」，这几行不该拿去支撑任何结论。
5. 探针把用户动作写死的地方，已经按 `.claude/rules/three-way-inference.md`「攻方腿的装置把用户动作写死时……」那一条放开扫过两维
   （失败前发几次、失败点落在哪一步）；**没放开的还有两维**：内容长度（固定 4100 / 3100 字节）与设备数（固定两块盘）。
6. 时刻：这一轮的测量都在 2026-09-21 UTC 当天跑完（东京时间 UTC+9）。
