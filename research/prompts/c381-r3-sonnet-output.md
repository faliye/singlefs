# C381（根已落盘之后发布失败，分配器仍退回） 第三轮：Sonnet 正推腿（K1、K2）

正推候选甲：不立条款，明写「fsync 报错之后数据可以出现也可以不出现」。分到 K1、K2。不碰 K3、K4；不替攻方找反例。

## 一、K1：候选甲的历史 + 「让调用方自己去读」在今天做不做得到

### 1.1 六步持久顺序（前提，逐字抄自权威条款）

D16（发布语义） 已定项 7（`.claude/kb/decisions/16-发布语义.md:99-100`）：

> **定案**：一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽；fsync 等根槽持久之后才返回。恢复重放在施加任何记录之前，必须逐项验证点名单元的校验和。**系统配置槽在根槽持久之后再更新，每个 checkpoint 一次**，它是发布序列的最后一步、进层 0 枚举，一次发布的段序列因此唯一。**fsync 返回条件多一句**：新实例在本实例写成的根覆盖两块盘之前，fsync 不返回、不向管理员确认回退（做法与次数在已定项 8）。

现查 `crates/singlefs-core/src/transaction.rs` 的 `perform`（`:126-191`）：六步对应 `CommitStep` 的五个变体（`WriteUnitToEveryDevice`、`Barrier`、`WriteJournalRecordToEveryDevice`、`Barrier`、`WriteRootRecordForceUnitAccess`、`RotateSystemConfigurationSlots`），逐步调用，任何一步 `write_at`/`barrier` 返回 `Err` 就经 `?` 立刻向上传播（`:137`、`:145`、`:162-169`、`:184`、`:231`）。`publish_admitted` 的调用与它的结果直接原样交给 `publish_version`（`:1275`），后者见到 `Err` 就把分配器整个换回准入之前的样子（`:1276-1277`，注释原文「准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立」，`:1240`）。⚠️ 这几行都比开工快照晚：另一个会话正在做的 `SystemConfiguration` 拆分已经落到 `transaction.rs`（`write_system_configuration_slot` 里的字段现在分成 `immutable`/`mutable`/`runtime`/`quantities` 四段），行号与开工快照对不上，以现查的这一份为准。**今天的实现里，`publish_version` 一次调用要跑完全部六步才返回**：不存在「fsync 在根槽落盘那一刻先返回，系统配置槽异步补」这种分离——已定项 7 那句「fsync 等根槽持久之后才返回」描述的是将来对外的 fsync 语义，今天的函数边界是「整段跑完才返回」，比它更严。

### 1.2 三段可达历史，逐段写调用方按候选甲、按「报错即没成立」各会怎么决定

**H1：第六步（系统配置槽轮换）失败，前五步全部成功。** 这正是 checks-owed.md C381 自己登记的机制（`.claude/kb/checks-owed.md:341`，逐字引原文）：

> 发布的持久顺序是根记录 FUA 之后再转系统配置槽；系统配置槽那一步失败时根已落盘，恢复会选中它，而发布路径把分配器整个退回到这次发布之前（`crates/singlefs-core/src/transaction.rs` 第 1221、1258 行）；同一个写入口接着发下一次发布会把这次占过的槽再分出去，写到一半断电就把已落盘的根引用的单元盖掉

- **调用方按「报错即没成立」决定**：相信这次发布在盘上什么都没留下，于是继续拿旧的 `previous`（分配器已换回、`current` 没推进）发下一次。分配器把这次发布占用过的槽当空槽再分出去。下一次发布写到一半断电——盘上留着的是**已经 FUA 落盘、会被 `choose_root` 选中的 txg = N 那个根**，它引用的单元槽正是刚被下一次发布覆写到一半的那些。恢复选中 txg = N 的根，读它引用的单元时读到别的发布写了一半的字节。**这是一处会犯的错，而且是 checks-owed.md 已经登记、有具体判别力的错**（原文续：「造这段历史，恢复之后 checker 全绿、重开成功——今天 checker 五条红、重开报 `Recovery(UnitUnreadable)`」）。
- **调用方按候选甲决定**：候选甲的字面只说「可以出现也可以不出现」，不单独构成「不许拿旧 `previous` 重发」这条禁令。但候选甲逼着调用方承认「我不知道」，一个把这句话当真的调用方**不能**心安理得地假设旧 `previous` 仍然对着盘上现状——它至少要去核实一次（1.3 节给出这一步在今天怎么做）。核实之后，`choose_root` 会告诉它 txg 已经推进到 N，调用方拿新的根重建 `previous`，不会再把 N 占用过的槽当空槽分出去，H1 这处错不会发生。
- **两者对照**：**H1 上「报错即没成立」让调用方犯错，候选甲（若真去核实）不会。** 这是 K1 要找的历史的反方向——我按分工找的是「候选甲让调用方犯错而『报错即没成立』不会」的历史，在 H1 上没找到，找到的是相反的一例。

**H2：第五步（根槽 FUA）本身失败，前四步成功。** 根槽这次没有落盘；`choose_root`（`crates/singlefs-core/src/recovery.rs:289-305`）择根看的是 `(checkpoint_txg, instance)` 全部自证校验和过的根槽取最大——txg = N 的根槽这次没写出合法字节（要么维持旧内容 txg = N−1、要么是撕裂的半截而自证不过），两种情形 `choose_root` 都不会选中 txg = N。**这次发布在「谁是现行根」这件事上确定没有成立**，两种语义在这一段上给出同一个判断（都是「没成立」），调用方按哪种决定都不会错，H2 分不出臂。

**H3：第一步或第三步（单元写 / 记录写）只在部分设备上成功就报错**（`for (_, device) in self.devices.iter_mut() { device.write_at(...)?; }`，单元写的 `write_at` 在 `:137`、journal 记录写的 `write_at` 在 `:145`：一块盘写成功、下一块盘 `write_at` 返回 `Err`，循环靠 `?` 当场中止，后续设备与后续步骤都不会跑）。根槽从未写（第五步没跑到），`choose_root` 择出的仍是旧根 txg = N−1；只有一块盘上多出一份不被任何有效根引用的孤立字节。这段历史里发布在「谁是现行根」上同样确定没有成立，两种语义给出同一个判断，也分不出臂。

**小结（这是这一格唯一能给的结论，不是绕开判据）**：三段覆盖了六步失败的三种代表形态（末步失败 / 关键步失败 / 早步部分失败），H2、H3 两种语义判断一致、分不出臂；H1 是唯一分得出臂的一段，而且方向与 K1 问的相反——「报错即没成立」在这段历史上会让调用方犯错，候选甲不会。**我没能找出候选甲让调用方犯错、而「报错即没成立」不会的历史；找到的是相反的一例。** 这是一次尝试的结果，不是穷举证明；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「一条腿只抽一次样不算一次观测」管的是模型的随机采样，这里是我自己的逐段分析，不是采样，但同样只代表这一次尝试，请核查员与另外两条腿的产出对照。

### 1.3 「让调用方自己去读」，今天的读路径做不做得到

**判据现查**：`grep -rn -i 'probe_write\|探针写\|read_only\|ReadOnly\|转只读\|instance_switch\|实例切换' crates/singlefs-core/src/*.rs` 零命中（与正文第四节给的观测一致，我重新跑过一次，仍是 0 处）。失败表三样（探针写、实例切换、转只读）今天在 `crates/` 里一行代码都没有，这一点与候选甲要不要立没有关系，是失败表本身（D23 已定项 14）的实现缺口，K4 与 4 号题的射程。

**候选甲要求调用方能读到什么，才判得出「这次发布成没成立」**：按 D16 已定项 4（`.claude/kb/decisions/16-发布语义.md:99-100`，逐字）「一次发布整体施加或整体不施加」，判据只需要一个：**这次发布要产生的 `checkpoint_txg`，现在是不是磁盘上自证通过的现行根**。

**这条判据今天在库里有没有现成的读法**：

1. `PoolWriter::devices` 字段是 `pub devices: &'pool mut [(DeviceIdentity, Device)]`（`transaction.rs:98`），发布失败之后调用方手里这个引用还在。
2. `crates/singlefs-core/src/recovery.rs:67` 有 `impl<Device: BlockDevice> PoolReader for [(DeviceIdentity, Device)]`——**这个切片类型本来就实现了 `PoolReader`**，不需要另写适配器就能把 `pool.devices` 当 `&dyn PoolReader` 传给恢复那几个函数。
3. `choose_system_configuration(reader: &dyn PoolReader)`（`recovery.rs:220-259`）：每块盘只读两个固定槽（`:224-225` 读第一槽、`:232-233` 读第二槽，`SYSTEM_CONFIGURATION_SLOT_BYTES` 大小），不是全盘扫描。
4. `choose_root(reader, &system_configuration)`（`recovery.rs:289-305`，靠 `visit_valid_roots`：`:262-285`）：只读 `ROOT_RING_REGIONS × ROOT_RING_SLOTS_PER_REGION` 个固定槽位，同样是有界读，不是全盘扫描——r2 判决第 17 行讲的是探针写要写的那个格式落点在地址空间表没有登记位（另一件事，指探针写本身要写的那个格式位置），不是「择根要不要扫全盘」，两者不是一回事，这里不拿 r2 的结论回答这一问。
5. `grep -n "choose_root(" crates/singlefs-core/src/*.rs` 命中五处：`recovery.rs:289`（定义）、`recovery.rs:1355`（`recover()` 内部）、`mount.rs:407`、`mount.rs:1113`、`mount.rs:1192`——**全部在 `recovery::recover` 与 `mount.rs` 的挂载 / 回退流程里，`transaction.rs` 的发布失败路径一次都没有调用它**。

**结论**：候选甲要求的那条判据（「这次的 txg 是不是现行根」）今天在 `crates/singlefs-core::recovery` 里有现成的、有界的读法——`choose_system_configuration` + `choose_root` 都能在同一进程里对着仍然打开的设备句柄调用，不需要卸载重挂、不需要跑 `singlefs-checker` 那种镜像级扫描。**但今天没有任何调用点把它接到发布失败路径上**：`publish_version` 返回 `Err` 之后，`crates/` 里没有代码替调用方做这次核实、也没有一个公开函数叫「查一下上次发布成没成立」。⇒ **候选甲说「自己去读」，字面上「读什么、怎么判」在库里已经有零件，但把零件接起来的那段胶水代码今天不存在，调用方要自己写**。这与「读路径今天完全做不到」不是一回事——是「零件在，没人接」，写报告时不许把这两者混成一句话。

**推翻条件**：找到 `choose_system_configuration` 或 `choose_root` 在某种可达输入上算出与实际磁盘状态不符的根（判据本身错），或找到一段历史里 `pool.devices` 在发布失败之后已经不可用（比如某个步骤把 `devices` 消费掉、调用方拿不到那个引用）——我 grep 过 `publish_version`/`publish_admitted` 的签名（`transaction.rs:1246`、`:1581`），两者都按 `&mut` 借用 `pool: &mut PoolWriter`，返回 `Err` 之后 `pool` 本身还在调用方手里，没有被消费，没找到这条反例。

### 1.4 一个连带发现（不进 K1 判定，标出来给 K3/K4 用）

分析 H1 时发现：D23 已定项 14 的实例切换（`.claude/kb/decisions/23-journal的角色与格式.md:371`，整段整括号抄）「所选根取被重发的那个在飞 checkpoint 所基于的根（旧实例发布过根时是它最后发布的根，没有时是这次挂载恢复重放之后的根，回退那次发布里是 R_old）」，字面上取的是**调用方自己内存里记的「最后发布的根」**，不是重新读盘的根。H1 那种「根已经落盘，但内存里的 `previous`/`current` 没推进」的分歧，按今天写死的字面，连已经定案的实例切换本身都可能读到旧根、重蹈 H1 的覆辙——这条不是我分工的格（它落在 K3「候选乙的安全性与定义域」或 K4「两条定案落地」的射程），我不在这里下判定，只记下来：**这一发现的推翻条件是找到已定项 14 或它的实现里有一步会话先重新核对磁盘根再动手**，我在 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 全文（`:340-391`）与 `crates/` 的三处 `choose_root` 调用点里没找到这一步。

## 二、K2：POSIX 的 fsync 错误语义、别家实现 EIO 之后再 fsync 的行为

按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「别的项目怎么做是线索不是证据」：这一格只当反证与「该测哪条路径」用，不进正推的承重链；1.1–1.4 节的结论不依赖这一节。

**来源与现查方式**：本机 `manpages-dev` 包（Linux man-pages 6.7）自带的 `fsync(2)`、`close(2)` 手册页，用 `man -P cat` 渲染后落盘到 `/tmp/claude-1000/c381-r3-sonnet/fsync-rendered.txt`、`close-rendered.txt`，行号是这两份落盘文件自己的行号（`grep -n` 现查，不是背景材料里的行号）。

**fsync(2) 的 EIO 一条**（`/tmp/claude-1000/c381-r3-sonnet/fsync-rendered.txt:66-72`，整段抄）：

> EIO    An error occurred during synchronization.  This error may relate
>        to data written to some other file descriptor on the same file.
>        Since Linux 4.13, errors from write-back will be reported to all
>        file  descriptors  that  might have written the data which trig‐
>        gered the error.  Some filesystems (e.g., NFS) keep close  track
>        of  which data came through which file descriptor, and give more
>        precise reporting.  Other filesystems (e.g., most local filesys‐
>        tems) will report errors to all file descriptors that were  open

**close(2) 的 NOTES 一段**（`/tmp/claude-1000/c381-r3-sonnet/close-rendered.txt:94-107`，整段抄，含它自己的括注）：

> A careful programmer will check the return value of close(),  since  it
> is  quite possible that errors on a previous write(2) operation are re‐
> ported only on the final close() that releases the open  file  descrip‐
> tion.   Failing  to check the return value when closing a file may lead
> to silent loss of data.  This can especially be observed with  NFS  and
> with disk quota.
>
> Note, however, that a failure return should be used only for diagnostic
> purposes  (i.e.,  a  warning to the application that there may still be
> I/O pending or there may have been failed  I/O)  or  remedial  purposes
> (e.g., writing the file once more or creating a backup).
>
> Retrying  the  close() after a failure return is the wrong thing to do,
> since this may cause a reused file descriptor from another thread to be
> closed.

**这一段与候选甲对不对得上**：POSIX/Linux 手册页里没有一句话说「fsync 或 close 报错 ⇒ 这次写没有生效」；相反，它逐字说「a failure return should be used only for diagnostic purposes ... or remedial purposes」——报错只当**警示**用，不当「数据确定不在」的证明，而且明说「重试是错的」（"the wrong thing to do"）。这与候选甲的字面（「可以出现也可以不出现」，不承诺任何一边）**方向一致**：都不把「报错」读成「没成立」的充分条件。它与候选甲**不一致的地方**是候选甲此刻只写了「可以出现也可以不出现」一句判断，没有像 close(2) 那样紧跟一句「所以报错之后不许做什么」（POSIX 明说「不许重试」）——**这是候选甲字面上比 POSIX 更单薄的一处，值得在写回条款时补一句「报错之后不许假设可以照旧复用同一份内存态重试」**，这正是 1.1 节 H1 分析出的那处真实会犯错的动作。

**与本工程一处已知差异**（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「引用别家实现的时候，必须同时写下它和这个项目的一处已知差异」）：POSIX 这两条手册页管的是**单个文件描述符上的单次写 / 单次 fsync**，错误粒度是「这一个 fd 曾经写过的字节可能没落盘」；singlefs 的一次发布是**六步、多设备、整体施加或整体不施加**的一个事务（D16 已定项 4），错误粒度是「六步里第几步失败」——POSIX 语义完全没有 singlefs 这种「后几步失败但前几步（含根槽）已经落盘、而且落盘的那部分单独就构成一次完整可读状态」的结构；这一层结构上的差异是 singlefs 自己的复杂度，POSIX 的经验教训能提示「报错不等于没发生」这一半，提示不了「怎么知道到底发生到哪一步」——那一半只能靠 1.3 节里 `crates/` 自己的读法。

**Postgres 的 fsyncgate（只当反证信号，不当证据）**：本机装了 `postgresql-16` 二进制，`strings` 扫它的可执行文件能看到 `could not fsync file "%s": %m`、`fsync failed`、`could not fsync file "%s" but retrying: %m` 等字符串共存，说明同一套代码里既有「报错就不再信任、要求重启」的路径，也有「报错但仍按某些场景重试」的路径——这与 POSIX 手册页「重试是错的」不是同一句话简单套用，说明「报错之后能不能重试」本身要按重试的是哪一层（同一次未确认的写，还是另一份独立的检查点）分别定，不能整体照抄。我没有 Postgres 源码可读（只有编译好的二进制），这一条只到「二进制里同时存在两种字符串」为止，**不写成「Postgres 怎么处理 fsync 错误」的正证**，按 evidence-discipline 的表，这属于「事实：某函数散在几处」一类可以引的东西，不属于「因为 Postgres 这么做所以这么做是对的」那种搬运。

## 三、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 历史（问一前半） | 候选甲没有被打中 | 三段可达历史（末步失败 H1、根槽自身失败 H2、早步部分失败 H3）里，H2/H3 两种语义判断一致分不出臂，H1 是「报错即没成立」让调用方犯错、候选甲不会——方向与要找的相反；这是一次尝试，不是穷举 |
| K1 读路径（问一后半） | 零件在、没人接 | `choose_system_configuration` + `choose_root` 能在同一进程对开着的设备句柄现查、判据是 txg 是否推进，但发布失败路径今天一次都不调它，调用方要自己写这段胶水 |
| K1 连带发现（不进判定） | 记一笔给 K3/K4 | D23 已定项 14 的实例切换字面取内存里「最后发布的根」，不是重新读盘，H1 的分歧对已定条款本身可能同样成立 |
| K2 外部语义 | 方向一致、字面比 POSIX 单薄 | POSIX 手册页不把 fsync/close 报错读成「没成立」，且明写「报错之后不许重试」；候选甲目前的字面只有前半句，没有后半句的禁止性条款，只当线索不进正推 |

## 四、没做什么

- 不判 K3（候选乙的安全性与定义域）、K4（两条定案的落地）：按分工不碰，也没有替 Opus 攻方找候选乙的反例。
- 没有跑任何变异或崩溃点重放装置：这一格是设计层面的历史枚举与代码读，不是实验，没有产物落 `research/results/`。
- 1.2 节的「一次尝试没找到反例」不是穷举证明，也不构成「候选甲已被验证」；三段历史是我自己按六步持久顺序逐个构造的，没有覆盖「多个失败点复合」（比如 S1 部分成功又叠加 S6 失败）这类更复杂的组合，是这一格没有做全的地方。
- 没有查阅 PostgreSQL 或其他数据库项目的源码本身（本机只有编译好的二进制），K2 里引它只到「二进制字符串同时存在两类信息」为止，没有做成正证。
- 没有跑 `cargo test`、没有编译，全程只读代码与手册页；开工前 `ps -o pid,args -u "$(id -u)"` 没有看到本会话之外的重负载编译或性能测量进程需要避让（这一步在读代码阶段就已够用，没有跑门禁或编译）。
- kb 条款（D16、D23、checks-owed.md C381）引用前均用 `grep -nF` 现查过命中行号；`crates/` 侧因另一个会话在做 `SystemConfiguration` → `SystemConfiguration` 改名与结构体拆分，行号在写作过程中变过一次，已按现查结果全部改正，写作时点以 2026-09-21 当次 `sha256sum` 现查为准，不回退、不代为修那个改名。

