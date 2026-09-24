# 代码轮：池级 checker 补三条、事务号按实例计数、checker 与恢复的挂载口径对齐（2026-09-21）

被判的是同一批里三处互相牵动的改动。三处都已落地、测试全绿（`cargo test --all` 230 passed / 0 failed、`clippy -D warnings` 退出码 0），要攻的是**改法本身对不对**，不是它编不编得过。

## 实现今天的样子（主 agent 的观测，2026-09-21 现查）

读过的路径与看到的事实：

- `crates/singlefs-checker/src/walk.rs`：`check_pool_image` 现判 32 条（`crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 长度 32）。这一轮之前是 29 条。
- `crates/singlefs-checker/src/walk.rs` 的 `TotalOrderKey`：码 3 的全序键是 (诞生代号, 实例代号, 事务号)，**码 1 的键显式停在 `TotalOrderKey::UndecidedForDataUnit`**。
- `crates/singlefs-core/src/transaction.rs` 的 `publish_overwrite`：事务号取 `previous.highest_transaction_number_in_this_instance + 1`。这一轮之前取 `previous.record.transaction + 1`。
- `crates/singlefs-core/src/mount.rs`：抬 F 与暖机的空发布仍写 `transaction: 0`，另传 `highest_transaction_number_before_this_publish`。
- `crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`：一块盘两个系统配置槽都无效时 `continue`，全部盘都无效才 `return Err`。
- `crates/singlefs-core/src/recovery.rs` 的 `rebuild_version`：`highest_transaction_number_in_this_instance` 取「这条记录上的事务号」，注释写明射程。

## 三处改动

### 一、池级 checker 补三条：I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法）

29 → 32 条。层 0 全量（release）262165 状态零违例，三条各自 262165/0（I-8.6 有 4 个环还空着的状态报不适用）。

⚠️ **I-1.8 只落了一半**：① 同一组的成员载荷相同已判；② 同 key 的各组两两全序键不等**只判码 3**，码 1 停着，欠账 C464（I-1.8 对码 1 的全序键写窄了）。停的理由：`I-1.8（归并后版本全序）` 给码 1 的键只有写序，而它自己的括注逐字写着「同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同」——码 3 用「诞生代号打头」解决了同一个问题，码 1 没有。

### 二、事务号按实例计数，不取上一条记录的值

`D23（journal 的角色与格式）` 已定项 7 逐字：「事务号按实例计数、从 1 起」；「记录按事务号顺序追加……**这是实例表行的 W 能当精确前缀的依据**」；「事务号 0 留给不承载事务的空发布记录」。

改前 `publish_overwrite` 取 `previous.record.transaction + 1`，而空发布在记录上写 0 ⇒ 空发布之后下一次发布拿到 `0 + 1 = 1`，与这个实例早先用过的 1 重号。改后取「这个实例见过的最大事务号 + 1」，靠 `PublishPlan` / `TransactionOutput` 两个新字段链式传递，不新增格式字段。

红证：改回旧写法，用例 `the_transaction_number_keeps_counting_per_instance_across_the_empty_publishes_that_raise_the_floor` 报 `left: 1 / right: 5`。

### 三、checker 的早退条件与恢复对齐

`walk.rs` 原本「任一块盘择不出系统配置 ⇒ 全部不变量报不适用并 return」；同一天 `recovery.rs` 改成「全部盘都择不出才失败」。两者不一致时，一块盘系统配置全废的镜像恢复挂得上、checker 一条都不判。改成「全部盘都择不出」才早退。

⚠️ 仍欠：「哪块盘的系统配置全废了」今天没有编号报得出来（C461（checker 判「挂不上的镜像」整片报不适用））。

## 要攻的四个面

| 面 | 问什么 |
|---|---|
| **K1 事务号改法的射程** | 链式传递在哪些路径上会断？`rebuild_version` 取「这条记录上的事务号」，注释说「今天每条恢复路径之后都要取新实例代号」——去 `crates/singlefs-core/src/mount.rs` 逐条核这句话，举出一条反例（恢复之后没取新号就接着发布的路径），或者证明它今天成立 |
| **K2 事务号改法买到了什么、没买到什么** | 重号修掉之后，`D23（journal 的角色与格式）` 已定项 7 说的「实例表行的 W 能当精确前缀」今天成不成立？I-1.2（块头写序已发布） 的谓词里码 1 那半截 `n ≤ W` 呢？改法只动了发布路径，判定那一侧有没有跟着对 |
| **K3 checker 补的三条判得对不对** | 逐条核判据原文与实现：I-7.3 的例外（自证过的根全部是第 0 代）照抄了没有、I-8.6 的两条射程（本实例第一条恒 0、跨实例边界不判）实现成什么样、I-1.8 ① 的归并分组键是不是「类身份段全部字段含写序」。找一个合法镜像让它们误判，或者找一个坏镜像让它们判不出 |
| **K4 早退条件对齐之后的新缺口** | 一块盘系统配置全废的镜像上，32 条里哪几条实际判不了？实测那一格只多 1 条不适用（用例里钉着 `newly_not_applicable.len() <= 1`）——去核那一条是哪个不变量、它判不了是不是对的；另外核「全部盘都择不出」这个新条件在什么镜像上会与恢复再次分叉 |

## 两条攻方腿的分工（攻击面不重叠）

| 腿 | 攻哪几面 | 不碰 |
|---|---|---|
| **云端攻方（Opus）** | **K1 事务号改法的射程**（链式传递在哪条路径上断；`rebuild_version` 那句注释成不成立）、**K2 它买到了什么没买到什么**（W 的前缀性、I-1.2（块头写序已发布） 谓词里码 1 那半截） | 不碰 checker 判据本身（K3）、不碰早退条件（K4） |
| **本地攻方** | **K4 早退条件对齐之后的新缺口**（一块盘系统配置全废的镜像上哪几条实际判不了、「全部盘都择不出」在什么镜像上与恢复再次分叉），以及 **K3 里只涉及「实现与判据原文逐字对不对得上」的那几格** | 不碰事务号（K1、K2）、不碰 K3 里要构造镜像的那几格 |
| **云端正推（Sonnet）** | **K3 补的三条 checker 判得对不对**（逐条核判据原文与实现，构造让它们误判的合法镜像或判不出的坏镜像） | — |

⚠️ **给攻方腿的一条现成线索**（材料员按「已经如何」的动词全仓 grep 挖出来的，不是主 agent 给的答案）：`.claude/kb/checks-owed.md` 的 C286（只读之后 remount 成可写算不算一次可写挂载） 逐字写着「只读之后 remount 成可写算不算一次可写挂载、许不许，全仓没有一句」。K1 要找的正是「恢复之后没取新实例代号就接着发布」的路径，这条欠账是今天唯一现成的候选入口——**去核它成不成立，别直接当结论**。
