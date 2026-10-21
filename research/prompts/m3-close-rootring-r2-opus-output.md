# m3-close-rootring-r2 云端攻方腿（Opus）报告（2026-10-20）

第二轮，前几轮判决：`research/prompts/m3-close-rootring-r1-main-verification.md`。攻的是**系统配置落地补丁打上之前**的 `crates/`（主工作区开工时的样子，拷进草稿目录的仓副本）；补丁只读过（它没动 F2 的入参，见 Y3.3）。副本上量出的数只算副本上的数，不是入库装置上的数。

## 复跑

```
cd /home/fy5090/code/singlefs
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-rootring-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen <开工时 crates/ 的一份拷贝（带 Cargo.toml、Cargo.lock、.claude/heavy-thread-default）> --work <新的空草稿目录>
```

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改了 `rerun_steps`：把 `tests/` 下两份测试与一个公共模块拷进冻结副本的 `crates/singlefs-harness/tests/`，各跑一次 `cargo test -p singlefs-harness --test <目标> -- --nocapture`（线程 31），抽出 `R2SAME` / `R2UNIFY` 行，与 `out/` 里两份逐字节比。我在草稿目录里用新树跑过一次，原样输出：

```
name=rerun_compare file=same.out same=yes lines=27
name=rerun_compare file=unify.out same=yes lines=183
```

| 文件 | sha256 |
|---|---|
| `tools/rerun.sh` | 5a46f88b70597b41147671441ce0149a3bfab0e3c4c925e6e822d69faac12f3a |
| `tests/rootring_r2_common/mod.rs` | 23a76b2697d286c2510a59b7d9eefef7eda7953cb46cf4ead4c56b0324920e2a |
| `tests/rootring_r2_attack_same_instance_rule.rs` | c3450a885f5f1baa5a3ad2f7397fa8ad5ab9aa3ed96051e759b4c0ec731ba6cf |
| `tests/rootring_r2_attack_unified_landing.rs` | 71aaa254a913d75114ccc3e1d68b64f087f51178ed4d2ce953c1bf923f8a64ce |
| `out/same.out` | d5d85e7f2fa3fbd770d5a745fb6b8676b2c6383fed9d97e997ed0aa7eedd9fa9 |
| `out/unify.out` | 46b297a97724f68352f01ceba673a9c5059432d6d705533e6e77b14916ccbd60 |

模型目录：`research/prompts/m3-close-rootring-r2-opus-model/`。造盘面用入库 harness 的内存稀疏盘与 `tests/common_unwitnessed_publish`（只读导入）。core 不改：C751 这一轮的形态与 C755 的几种「同一种失败」都在测试里现算——F2 判法照抄一份（`fall_back_to`，多一个「同实例当作不被抛弃」的开关），关着开关时与 core 的函数逐格比，26 格 `replica_mismatches=0`；「今天」也在模型里按每处自己那一样读者现算一遍，与真跑的恢复、只读挂载、可写挂载逐格比，150 格 `model_mismatches=0`、`writable_matches=false` 0 格。

## 各格判定一览

| 格 | 判定 | 打中的东西（一句） | 量过 / 推的 |
|---|---|---|---|
| Y1 例外条件与别的合法根 | 没打中 | 例外照 I-7.17 写成「txg 0 ⇒ 只核槽 0」；根环洞、跨区域回绕、同 txg 两根逐类核过，都住在自己的公式槽 | 推的（读码） |
| Y1 修补本身（例外重开槽 0 的错位洞） | 没打中后果，改正第一轮的一条理由 | 例外放行「槽 0 读回别区域槽 0 的第 0 代根」；第一轮说「那时区域 0 槽 0 本来就住着第 0 代根」不对，正确的理由是被错位读回的那份第 0 代根自己就在环里 | 推的 |
| Y1 十处入口 | 没打中 | 补一处第一轮表里没单列的入口（取号读根环的实例代号），结局不变 | 推的 |
| Y2 同实例当作不被抛弃 | **打中（三样故障，与 C751 登记那一形同一预算）** | 见证到的那一版属于更旧的实例、区间里有所选根同实例的根时，新形态退到同实例那条——它与所选根共用那一张读不出的实例表，恢复从今天的「退到见证到的那一版、读得出」变成照报失败；四组都是 | 量过（副本，`out/same.out`） |
| Y2 `:656` 与 C751 登记那一形 | 没打中 | `:656` 三处结局一个字不变；登记那一形只读挂载从 Pm 改落 P（修到了），恢复与可写在 P 上照旧失败（P 也是那张读不出的表）——第一轮说的「三处都能落 P」只对落点成立 | 量过 |
| Y3 同一种失败取什么 | **打中（取打开或建上一版时，单故障加一次断电）** | 取打开：恢复 30 / 50 格、可写 22 / 50 格从「退到 P、做得成」变成照报失败；取建上一版：恢复 8 / 50 格同样变坏；取走读或并集：没有一格少挂上，但只读 34 格、可写 8 格从所选那一版落回上一版（见证丢失那一形里所选那一版已确认） | 量过（副本，`out/unify.out`） |
| Y3 走读判退的代价 | 打中（条文层，推的） | 只读挂载与可写挂载要先在所选那一版上跑一遍走读才判得了退不退 | 推的（读码） |
| Y3 与 C754 重扫 | 打中（条文层） | 三处交 F2 的记录来路不同（恢复重扫，两种挂载用第一遍），补丁只在 F1 再判之前重扫 | 读码 |
| Y4 字段与延续 | 没打中 | 钉的值只由槽号定（r + 3s），位图够 checker 现算；跨挂载、回退、取号都接得上 | 推的 |
| Y4 现算用哪个 F | **打中（零读故障、一次断电，推的）** | 照 F_生效 现算，在「新 F 已进系统配置、带新 F 的根没落」的合法状态上与盘上记的不等；I-3.1 对这一格有明文的「不判」例外，新判没有 | 推的 |
| Y4 坏实现判不判得出 | **打中（推的）** | 这一轮的形态比第一轮补法少一句「钉着的槽不住着这个实例写过的根」，「写掉钉着的槽不解钉」判不出；「该重读不重读」「多钉没坏的槽」两类求等永远判不出（判别子观测不到） | 推的 |
| Y4 I-3.1 用哪个式子、C684 | **打中（条文层，推的）** | 形态没写 I-3.1 拿盘上门槛怎么用：沿用按引用并的式子，C773 自己登记的那一形仍红；不看引用并，C684 翻（第一轮已攻，不重复）；另有一式两头都对（零轮） | 推的 |
| Y5 四个形态一起 | 部分打中 | `:656` 与 50 格在「同实例 × 统一」下结局不变；Y2 那一形在统一之后三处同落同实例那条，恢复与可写失败；Y1 × Y4：按自证不过处置的错位读进位图，求等自洽 | 前两句量过；末句推的 |

## Y1　C743 补了第 0 代例外之后

### Y1.1 例外写成什么条件；别的合法根会不会被判错位（没打中，推的）

例外照 I-7.17 那一句写：「时只判 s == 0，不判 r」（`crates/singlefs-checker/invariants/02-根环.md:24`）。条件是「读出的根 `checkpoint_txg` == 0 ∧ 读的是某个区域的槽 0」，不看区域号；core 里 mkfs 种根的槽就是这几个：「(0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })」（`crates/singlefs-core/src/make_filesystem.rs:677`）。核用的公式与写者是同一个函数：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`crates/singlefs-core/src/root_ring.rs:114`），根槽写在 `crates/singlefs-core/src/transaction.rs` 的 `CommitStep::WriteRootRecordForceUnitAccess` 那一支也经 `target_for_publish` 取槽。

正文点名的三类逐类核（读码，推的）：

| 合法根 | 住在哪 | 带例外的公式核判不判错位 |
|---|---|---|
| 根环洞（根槽写失败推进一格、新实例跳号、崩在记录之后根槽之前） | 那一槽留着上一圈或更早那一圈的根，那条根当年就是按公式写进这一槽的 | 不判：公式只看 txg mod 3S，上一圈的根在自己的槽 |
| 跨区域回绕 | `(txg div R) mod S`，回绕只是 mod | 不判 |
| 同 txg 两条合法根（D22 已定项 7「设备失而复得会造出两条同 txg 的合法根」） | 两条同 txg 的根只能落同一个公式槽；一块盘上一个槽一时只装一条，两条根不会同时出现在一份盘面的两个槽里 | 不判：每条都在自己 txg 的槽 |

已定项 16 那一句「无状态、只由 txg 与 (R, S) 决定」（`.claude/kb/decisions/22-单元原子性怎么合成.md:335`）与公式核同一个意思，不冲突。第一轮正推说的「冲突」落不到盘面上，与第一轮判决记的「没找到走得到的盘面」一致。

推翻条件：找到一个合法写者不经 `target_for_publish` 写根槽（mkfs 之外），或 S 在池的一生里变过。

### Y1.2 修补本身：例外把 C743 的洞在槽 0 上重新打开（打中不了后果，推的）

带例外之后，读区域 r 的槽 0 读回区域 r' 槽 0 那一份第 0 代根，核放行。第一轮攻方说这一形无害，理由是「那时区域 0 槽 0 本来就住着一份第 0 代根」——这一句不对。第一轮那段历史（`out/genesis.out`，txg 1 崩在根槽之前，新实例从 txg 2 起）里，S = 8 时 txg 24 写进区域 0 槽 0、txg 25 才写区域 1 槽 0；txg 24 落盘到 txg 25 之前这一段，区域 0 槽 0 住的是 txg 24，区域 1 槽 0 仍是第 0 代根。这时读区域 0 槽 0 错位读回区域 1 槽 0 的第 0 代根，例外放行。

后果逐处核（推的，按今天的代码）：
- 择根：txg 24 被藏；若它是最新根，它的记录照 D23 已定项 15 施加回来，与「读坏」同一处置。
- 重建分配器两遍：错位那一遍记第 0 代根（空根，txg 0），另一遍读坏时按「两遍里只有一遍自证过」记它（`crates/singlefs-core/src/mount.rs:3058` 那一类）。环里最旧有效根本来就是 0（区域 1 槽 0 那份第 0 代根自己就在环里，读得出），回收门槛不变；准入抬 F 的上限少数一条非空根（txg 24），上限只会更低。
- F_生效：盘 0 上最新的有效根从 24 换成 23（区域 2，盘 0），F 只会取低；系统配置里另记一份 F。

结论：这一洞放行的只有第 0 代根，它是空根、只活在第一圈或洞里，而那一刻环里一定另有一份读得出的第 0 代根（被错位读回的那一份本身），所以回收门槛不动、准入只更保守。第一轮的结论（无害）对，理由要换成这一条。推翻条件：一份盘面上区域 r' 槽 0 的第 0 代根在环里读不出（另一样故障），同时区域 r 槽 0 错位读回它——要两样读故障。

### Y1.3 十处入口按自证不过处置之后（没打中，推的；补一处第一轮表里没单列的）

第一轮那张十处表（攻方 X1.3）照收。逐个 grep `read_root_ring_slot(` 与 `RootRecord::parse_slot(` 的调用点，另有一处读根环的入口第一轮表里没单列：取号算新实例代号的 `highest_root_instance`（`crates/singlefs-core/src/recovery.rs:4198`，经 `visit_valid_roots`），取号式子「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」（`crates/singlefs-core/src/transaction.rs:942`）。错位那一份当没有：被错位读回的那条根在它自己的槽里另读一次，照样进 max；被藏住的那一槽的根的实例代号，取号写过每一份系统配置（D23 已定项 14 实例切换与取号那几句），系统配置那一路照样带着它。只有两样故障同时（被藏那一槽的根是最高实例的唯一一条，且每块盘带它的系统配置槽都读不出）才少数一个号，那是 C555 那一族（撞号），不是错位读新开的。

在单故障加一次断电之内：每一处的结局与第一轮表相同，没有一处丢已确认的写或挂不上本来挂得上的池。

## Y2　C751 同实例的根当作不被抛弃

### Y2.1 打中：见证到的那一版属于更旧的实例时，新形态退到一条注定走不通的根（量过，副本）

先看结构（读码）：一个实例里只有写行那次发布重写实例表，之后每一版照抄同一张（`crates/singlefs-core/src/transaction.rs:8314` 起几处「instance_table: InstanceTablePlan::Carry(previous.root.instance_table),」）；D23 已定项 14 的行只写给 [这个根的实例, 新实例)（`.claude/kb/decisions/23-journal的角色与格式.md:406`）。所以「与所选根同实例」的根几乎总是与所选根**共用那一张读不出的实例表**（例外只有清扫发布）。新形态在表读不出时多收的正是这些根：它们在只读挂载上打得开（打开不读实例表），在恢复走读与可写建上一版上必定读不通。今天表读不出时只认见证到的那一版（「None => order == witnessed_publish,」，`crates/singlefs-core/src/recovery.rs:6996`），而见证到的那一版可以属于更旧的实例，用的是另一张、读得出的表。

历史（`tests/rootring_r2_attack_same_instance_rule.rs` 的 `cross` 那一族）：R 整次落盘（已确认）→ 一次干净的可写挂载（取号 2，写行 Row、暖机两次）→ 崩在这次挂载第 2 条根（第一次暖机，wu1）的根槽之后、它那次轮换之前（一次断电）。读的那一刻：wu1 指着的实例表（实例 2 写行时新写的那一张）两份读回全零，每块盘较新的系统配置槽（Row 那次轮换写的）读回全零——与 C751 登记那一形同一个故障预算（一个单元两份、每块盘较新的系统配置槽、一次断电）。较旧的系统配置槽是取号那一写，它带的 c_见证 是 R 的计数器，见证到的那一版 = R（实例 1）。

`out/same.out` 原样一行（覆盖写 k = 5）：

```
R2SAME shape=cross kind=ow k=5 crash_after_mount_root#2 fault=table+cfg chosen=(2,11) table_readable=false today rec=Ok(1,9) ro=Ok(2,11) rw=Fail | f2_core=(1,9) f2_replica_off=(1,9) f2_same_instance=(2,10) replica_matches_core=true target_shares_the_chosen_table(off,on)=(false,true) | own/exact: rec=Ok(1,9) ro=Ok(2,11) rw=Ok(1,9) | own/same: rec=Fail@(2,10) ro=Ok(2,11) rw=Fail@(2,10) | walk/exact: rec=Ok(1,9) ro=Ok(1,9) rw=Ok(1,9) | walk/same: rec=Fail@(2,10) ro=Ok(2,10) rw=Fail@(2,10) | union/exact: rec=Ok(1,9) ro=Ok(1,9) rw=Ok(1,9) | union/same: rec=Fail@(2,10) ro=Ok(2,10) rw=Fail@(2,10)
```

读法：所选根 (2,11)，表读不出。今天 F2 只认恰好等于见证到的那一版 R = (1,9)，它用实例 1 的表，走读做成（真跑 `today rec=Ok(1,9)`）；新形态把同实例的 Row = (2,10) 收进来、取最新，Row 与所选根共用那张读不出的表（`target_shares_the_chosen_table(off,on)=(false,true)`），恢复照报失败（`own/same: rec=Fail@(2,10)`）。ow、sw3 两族 k = 3、5 四组同形：

```
today rec=Ok(1,7)	own/same: rec=Fail@(2,8)
today rec=Ok(1,9)	own/same: rec=Fail@(2,10)
today rec=Ok(1,7)	own/same: rec=Fail@(2,8)
today rec=Ok(1,9)	own/same: rec=Fail@(2,10)
```

只读挂载今天不退（所选那一版打得开），新形态下也不退，不受影响；可写挂载真跑今天就拒（重建分配器判抛弃用的最新那条根的实例表读不出，D23 已定项 14 那一拒），模型里那一列 `rw=Ok(1,9)` 只量了建上一版、不含这一拒，所以可写这一处两种形态都照报失败。去掉系统配置那一样故障（`fault=table`），见证到的那一版就是 Row，两种形态都退到 Row、都失败——这一形要的正是 C751 登记那一形的三样故障。

结论：在 C751 自己的故障预算之内，这个改法修了登记那一形的只读挂载，同时让另一形的恢复从「读得出、内容对」（R 已确认，Row 与暖机都不改文件）变成照报失败。推翻条件：一份实现里同实例的根不共用所选根的实例表（同一实例里重写实例表不止写行那一次）。

### Y2.2 `:656` 与 C751 登记那一形（量过，副本）

```
R2SAME shape=registered kind=ow k=5 chosen=(1,9) table_readable=false today rec=Fail ro=Ok(1,7) rw=Fail | f2_core=(1,7) f2_replica_off=(1,7) f2_same_instance=(1,8) replica_matches_core=true target_shares_the_chosen_table(off,on)=(true,true) | own/exact: rec=Fail@(1,7) ro=Ok(1,7) rw=Fail@(1,7) | own/same: rec=Fail@(1,8) ro=Ok(1,8) rw=Fail@(1,8) | walk/exact: rec=Fail@(1,7) ro=Ok(1,7) rw=Fail@(1,7) | walk/same: rec=Fail@(1,8) ro=Ok(1,8) rw=Fail@(1,8) | union/exact: rec=Fail@(1,7) ro=Ok(1,7) rw=Fail@(1,7) | union/same: rec=Fail@(1,8) ro=Ok(1,8) rw=Fail@(1,8)
R2SAME shape=pinned656 kind=ow k=5 chosen=(2,10) table_readable=false today rec=Ok(1,8) ro=Ok(1,8) rw=Fail | f2_core=(1,8) f2_replica_off=(1,8) f2_same_instance=(1,8) replica_matches_core=true target_shares_the_chosen_table(off,on)=(false,false) | own/exact: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8) | own/same: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8) | walk/exact: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8) | walk/same: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8) | union/exact: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8) | union/same: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8)
```

- `:656` 那一形（`pinned656`）：见证到的那一版 P = (1,8) 属于实例 1，所选根 Row 属于实例 2，区间里没有同实例的根，新形态与今天都退到 P，六种组合（今天、走读、并集各配两种取法）下三处都是 `Ok(1,8)`（模型的可写只量建上一版）。入库用例 `:656` 的三处结局不变，与第一轮量的一致。
- 登记那一形（`registered`）：只读挂载从 Pm = (1,7) 改落 P = (1,8)，修到了 C751 要修的那一处；恢复与可写在 P 上照旧失败，因为 R、P、Pm 共用那张读不出的表（`target_shares_the_chosen_table(off,on)=(true,true)`）。第一轮攻方说这个改法让「三处都能落 P 而不是照报失败」——落点对，「不是照报失败」只对只读挂载成立：第一轮那份模型只量了 F2 交回哪一条，没在交回的那一条上跑三样读者。

### Y2.3 「同实例其实已被抛弃」：没造出，归到共有的前提（推的）

同一实例里的根不会被同一实例的表抛弃（行只写给更旧的实例），回退与切换都不在同一实例内抛弃根。唯一的形是实例代号被重用：取号式子是 max(系统配置, 根环) + 1，看不到的号会被再取一次——这就是 C555「两盘最新系统配置槽都坏时取号撞号」（`.claude/kb/checks-owed.md:361`）。撞号之后被撂下的那段时间线若留下了读得出的根，新形态会把它当同实例收；但表读得出时今天的 F2 同样把它当不被抛弃（新实例的表里没有同号的行）。这一形打中的是两种取法共有的前提（实例代号唯一），不分辨这一轮的改法，归 C555。

### Y2.4 一条两头都不坏的取法（我提的，只在我的模型输出上逐格推过、被攻过零轮）

「表读不出时，见证到的那一版与所选根**同实例**才照收区间里同实例最新一条；不同实例时照今天只认恰好相等」。逐格按 `out/same.out` 推（推的，没改模型重跑）：登记那一形 W = Pm 与 R 同实例 ⇒ 落 P，只读修到；`:656` W = P 与 Row 不同实例 ⇒ 落 P，不变；Y2.1 那一形 W = R 与 wu1 不同实例 ⇒ 落 R，恢复照今天做成；`cross` 第 3 条根那几格 W = Row 与所选根同实例 ⇒ 落 wu1，恢复今天也失败。判得出：两条根的实例代号都在根记录里，不读表。

## Y3　C755 只统一落到哪一版

### Y3.1 「同一种失败」取哪一样：四个候选逐个量（量过，副本）

造法（`tests/rootring_r2_attack_unified_landing.rs`）：第一轮攻方那一族（覆盖写或顺序写三个数据单元的 R，k = 3、5，R 写出的每个单元两份读回全零、或 R 的根指着的那张实例表），三种盘面各 50 格：`witnessed`（R 整次落盘，见证过）、`witness_lost`（R 整次落盘、已确认，每块盘较新的系统配置槽读回全零：两样故障）、`unwitnessed`（崩在 R 的根槽之后、轮换之前：一样故障加一次断电）。「同一种失败」取四种候选：走读（恢复那一样）、打开（只读那一样）、建上一版（可写那一样）、三样的并集；按它判退不退，退就交 F2（照抄的判法，关开关就是今天的 F2），落到那一版之后各处跑自己那一样读者判成不成（「成败各照今天」）。与「今天」（每处看自己那一样）逐处比，`out/unify.out` 里机器数出的变化（只列不是 `same` 的；`rule=same` 那一半与 `rule=exact` 逐行相同，下面只贴 `exact`）：

```
R2UNIFY_TALLY img=unwitnessed trigger=open rule=exact place=rec change=ok_to_fail cells=30
R2UNIFY_TALLY img=unwitnessed trigger=open rule=exact place=rw change=ok_to_fail cells=22
R2UNIFY_TALLY img=unwitnessed trigger=rebuild rule=exact place=rec change=ok_to_fail cells=8
R2UNIFY_TALLY img=unwitnessed trigger=rebuild rule=exact place=ro change=older_ok cells=26
R2UNIFY_TALLY img=unwitnessed trigger=union rule=exact place=ro change=older_ok cells=34
R2UNIFY_TALLY img=unwitnessed trigger=union rule=exact place=rw change=older_ok cells=8
R2UNIFY_TALLY img=unwitnessed trigger=walk rule=exact place=ro change=older_ok cells=34
R2UNIFY_TALLY img=unwitnessed trigger=walk rule=exact place=rw change=older_ok cells=8
R2UNIFY_TALLY img=witness_lost trigger=open rule=exact place=rec change=ok_to_fail cells=30
R2UNIFY_TALLY img=witness_lost trigger=open rule=exact place=rw change=ok_to_fail cells=22
R2UNIFY_TALLY img=witness_lost trigger=rebuild rule=exact place=rec change=ok_to_fail cells=8
R2UNIFY_TALLY img=witness_lost trigger=rebuild rule=exact place=ro change=older_ok cells=26
R2UNIFY_TALLY img=witness_lost trigger=union rule=exact place=ro change=older_ok cells=34
R2UNIFY_TALLY img=witness_lost trigger=union rule=exact place=rw change=older_ok cells=8
R2UNIFY_TALLY img=witness_lost trigger=walk rule=exact place=ro change=older_ok cells=34
R2UNIFY_TALLY img=witness_lost trigger=walk rule=exact place=rw change=older_ok cells=8
```

整理成表（格数 / 50；`witnessed` 那一盘四种候选都 0 变化，不列）：

| 候选 | 盘面 | 恢复 | 只读挂载 | 可写挂载 |
|---|---|---|---|---|
| 走读 | witness_lost | 0 | 34 格落回上一版 | 8 格落回上一版 |
| 走读 | unwitnessed | 0 | 34 格落回上一版 | 8 格落回上一版 |
| 并集 | 两盘 | 同走读 | 同走读 | 同走读 |
| 打开 | 两盘 | **30 格做得成变失败** | 0 | **22 格做得成变失败** |
| 建上一版 | 两盘 | **8 格做得成变失败** | 26 格落回上一版 | 0 |

两行原样（覆盖写 k = 5、R 的第一个数据单元）：

```
R2UNIFY kind=ow k=5 img=witness_lost fault=unit#0:len32768 chosen=(1,9) today rec=Some((1, 8)) ro=Some((1, 9)) rw=Some((1, 9)) model_matches_today=true writable_matches=true | own: rec=Ok(1,8) ro=Ok(1,9) rw=Ok(1,9) | walk/exact: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8) | open/exact: rec=Fail@(1,9) ro=Ok(1,9) rw=Ok(1,9) | rebuild/exact: rec=Fail@(1,9) ro=Ok(1,9) rw=Ok(1,9) | union/exact: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8) | walk/same: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8) | open/same: rec=Fail@(1,9) ro=Ok(1,9) rw=Ok(1,9) | rebuild/same: rec=Fail@(1,9) ro=Ok(1,9) rw=Ok(1,9) | union/same: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8)
R2UNIFY kind=ow k=5 img=unwitnessed fault=unit#0:len32768 chosen=(1,9) today rec=Some((1, 8)) ro=Some((1, 9)) rw=Some((1, 9)) model_matches_today=true writable_matches=true | own: rec=Ok(1,8) ro=Ok(1,9) rw=Ok(1,9) | walk/exact: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8) | open/exact: rec=Fail@(1,9) ro=Ok(1,9) rw=Ok(1,9) | rebuild/exact: rec=Fail@(1,9) ro=Ok(1,9) rw=Ok(1,9) | union/exact: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8) | walk/same: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8) | open/same: rec=Fail@(1,9) ro=Ok(1,9) rw=Ok(1,9) | rebuild/same: rec=Fail@(1,9) ro=Ok(1,9) rw=Ok(1,9) | union/same: rec=Ok(1,8) ro=Ok(1,8) rw=Ok(1,8)
```

读法：
- 取打开或建上一版：`unwitnessed` 那一盘是一样读故障加一次断电，恢复今天「走读失败 ⇒ 退到 P、读得出」，统一之后所选那一版打得开（或建得出）就不退，恢复留在所选那一版上照报失败——单故障加一次断电之内「少挂上」：恢复 30 格（打开）或 8 格（建上一版），可写 22 格（打开）。
- 取走读或并集：三种盘面 150 格里没有一格从做得成变失败，统一的代价全是「落回上一版」：只读挂载 34 格、可写挂载 8 格今天落在所选那一版（打开、建上一版不读那个坏单元），统一之后跟着恢复退到 P。`unwitnessed` 那一盘 R 没确认，退到 P 在 F2 许可之内；`witness_lost` 那一盘 R 已确认，这是 D23 已定项 14 记的那一条代价（「已确认的版本在较新的系统配置槽与它的单元同时读回全 0 时，恢复、可写挂载与只读挂载悄悄读回上一版」（`.claude/kb/decisions/23-journal的角色与格式.md:423`））——今天只有恢复真付这一笔（只读、可写在这 34 / 8 格读到的是已确认的 R），统一之后三处都付。
- `witnessed` 那一盘（单故障、零断电）：F2 见证过就不退，四种候选都与今天逐格相同。第一轮打中的「统一成败让池挂不上」在「只统一落到哪一版」这一读法下消失，与第一轮正推的读法一致。

所以「同一种失败」能写成当下判得出的条件，但只有走读（或并集）不让任何一处少挂上，它的代价是把已认的那一笔代价从恢复一处扩到三处，并要两种挂载先跑一遍走读（Y3.2）。
### Y3.2 判退要的那一样读者谁来跑（推的，读码）

「同一种失败」取走读或并集时，只读挂载与可写挂载判退之前要先在所选那一版上跑一遍走读：今天只读挂载只做打开（不读实例表、分配记录树、记账树、数据单元，第一轮攻方 X3.1 那张表），可写挂载建上一版不读数据单元内容。走读 `walk_to_file` 读整棵 extent 树与第一个文件的数据单元；第一版池里只有一个文件，走一遍就是整版，挂载时间从「读打开要的那几样」变成「读整版元数据加数据单元」。取打开或建上一版判退则不加读，但恢复会在 Y3.1 量出的那几格从「退到上一版、读得出」变成「照报失败」。这一格是读码推的，没量挂载耗时。

### Y3.3 与 C754「重扫」：三处交给 F2 的记录不是同一份（读码，补丁前后相同）

今天三处调同一个判法，入参 `records` 来路不同：恢复走读失败之后重扫一遍（`crates/singlefs-core/src/recovery.rs:7124`），只读挂载用开头那一遍扫出的（`crates/singlefs-core/src/mounted_read.rs:657` 那次调用传的 `records`），可写挂载用读阶段那一份（`crates/singlefs-core/src/mount.rs:8018` 那次调用）。系统配置落地补丁只在可写挂载退了之后、F1 再判之前加了一次重扫（补丁里「判之前先重扫一遍 journal 环（C754，用户 2026-10-20 定「重扫」）」那几行），F2 的入参没动。第一轮攻方 X3.5 已指出这一点；这一轮的形态把「退不退、退到哪」统一了，F2 的三个判断（见证过没有、见证到的那一版是哪条、会不会跳过一次末条读得出而根读不出的发布）全吃 `records`，同一份盘面上一条记录瞬时读不出时，三处可以落到不同版本，「统一落到哪一版」在条文上成立、在实现上不成立。条文要连「交 F2 之前三处都重扫一遍」（或都用第一遍）一起写；只重扫不够的地方是两次独立的挂载各读各的，瞬时读故障下两次本来就可以不同——统一只能管到同一种函数、同一种取法的入参，管不到两次读到的字节。

## Y4　C773 另记钉槽、checker 现算求等

先改正第一轮攻方 X4.2 第 1 条的一个读码错（它推出的「门槛差一截」不成立）：分配器记根在装记账行之前，「带单元的发布在装记账行之前调，记账行因此已经按回收之后的数写」（`crates/singlefs-core/src/allocator.rs:2580`），记的时候先把这条根放进根环表、挤掉那一槽原来的根（`crates/singlefs-core/src/allocator.rs:2606` 起），再按新表算门槛回收。所以发布 T 的记账用的门槛就是「T 写进环之后」那张表算的，与 checker 在 T 之后的镜像上按环现算的是同一张环。另一个读码要点：钉的值不靠窗口——「可能住过的最旧 txg：落到这个槽、小于 `first_txg` 的最小 txg。」（`crates/singlefs-core/src/mount.rs:2845`），循环「for txg in 0..first_txg.0.min(ring_length_in_txgs) {」（`crates/singlefs-core/src/mount.rs:2880`）：一个钉着的槽 (r, s) 的值恒是 r + 3s（年轻的环里还轮不到它的槽不钉）。checker 拿钉槽位图就能逐槽现算出钉的值，不需要盘上另记 N 或钉的值。

### Y4.1 字段宽度、落点与延续（没打中，推的）

门槛 8 字节加钉槽位图 ⌈3 × 16 / 8⌉ = 6 字节（S 上界 16），根记录 457 → 471，仍在一个 512 字节扇区里。跨挂载：钉是这个进程挂载那一刻读出来的，新挂载重算，位图只说这个进程；回退（F2 落到旧根）时 checker 判的是镜像上最新那条根，旧根的字段是它那个进程的，不参与；取号之后新实例第一条根（写行那次）带这次挂载的钉。没找到延续上断的一格。

### Y4.2 现算用哪个 F：用 F_生效 会在合法的一次断电状态上误红（打中，条文层，推的）

形态只写「池级 checker 现算门槛并要求与盘上记的相等」。今天 checker 的门槛函数取 F_生效（`crates/singlefs-checker/src/walk.rs:8338` 的 `deferred_slots_reclaim_threshold`，入参 `effective_rollback_floor`），F_生效 = 各盘最新有效根带的 F 与系统配置里 F 取大。抬 F 的持久顺序是先写系统配置、再回收、再发带新 F 的根；崩在系统配置写完、新根落盘之前，镜像上最新根 T 带旧 F、记的是按旧 F 算的门槛，F_生效 已是新 F。照 F_生效 现算，门槛等于 max(新 F, 环里最旧有效根)，不等于 T 记的 max(旧 F, …)，求等判红——零读故障、一次断电。I-3.1 自己对这一格有明文例外：「最新根带的 F 低于 F_生效（两者不等）时这一格不判」（`crates/singlefs-checker/invariants/05-空间记账.md:8`）。

改法（推的，被攻过零轮）：现算门槛取「被判那条根自己带的 F」，不取 F_生效；或照 I-3.1 那一格在两者不等时报不适用。两条都写不进的话这条新判在合法状态上红。

### Y4.3 「坏实现」判不判得出：缺第一轮补法里的一句，又有一类观测不到（打中，推的）

这一轮的形态比第一轮攻方的补法少了一句：第一轮补法是「checker … 现算出门槛、要求与字段相等，另判「钉着的槽不住着这个实例写过的根」」（第一轮攻方报告 X4.4）；这一轮正文只写「现算门槛并要求与盘上记的相等」。钉的值只由槽号定（r + 3s），checker 现算时钉哪几槽照写者自己写的位图取：

| 坏实现 | 求等判不判得出 | 理由 |
|---|---|---|
| 门槛恒取 F，位图如实 | 判得出 | 现算 max(F, min(…)) 一般不等于 F |
| 写掉钉着的那一槽也不解钉，位图照记 | **判不出**（缺那一句时） | 现算照位图取 r + 3s，与写者用的同一个数，相等；补上「钉着的槽不住着这个实例写过的根」才红 |
| 下一次发布不重读（该解钉时不解），位图照记 | **判不出** | 那一槽在镜像上读得出一条根，但重读那一刻「又读坏一次」是合法的；镜像上两种情形逐字节相同（判别子观测不到） |
| 位图里多钉几槽（挂载时根本没读坏的） | **判不出** | checker 看不到挂载那一刻读坏了哪几槽，位图是写者自报 |

后三类都是「钉低不放」的泄漏（记账多的一边），今天的 I-3.1 在它们上面红（那一红是用户认的代价，见 I-3.1 那一格「钉最旧 txg 的代价（用户认的）」，`crates/singlefs-checker/invariants/05-空间记账.md:8`）；换成求等之后，合法的钉与后两类泄漏一起绿。泄漏有界：这个进程写掉那一槽就自动解（`crates/singlefs-core/src/allocator.rs:2606` 起那一插），最多一圈 3S 次发布。补上第一轮那一句只买回第二行。

### Y4.4 求等之外 I-3.1 用哪个式子：形态没写，两种直接读法各撞一边（打中，条文层，推的）

求等只判两个数一致，没说 I-3.1 拿盘上门槛怎么用。C773 那一形（登记行：记账 302、遍历 258，「差的 44 槽释放代全在 (21, 25]」（`.claude/kb/checks-owed.md:523`））里：写者门槛 21、checker 现算 25（环里最旧有效根）；释放代 22–25 的槽被 txg 21–24 的根引用，那几条根在镜像上已不是环里的有效根（否则 checker 的最旧有效根不会是 25），所以这 44 槽在镜像上没有有效根引用。
- 读法一（今天的丁-defer 选根式不变，只把门槛换成盘上的 21）：并进的只是「F 之下的有效根引用着」的槽（`crates/singlefs-checker/src/walk.rs:8329`），这 44 槽没人引用，仍不并，**C773 那一形仍红**——修补没修到它自己的登记形。
- 读法二（释放代高于盘上门槛的已释放槽不看引用都并）：C773 转绿，C684 那一形也转绿——这是第一轮攻方 X4.5 已攻过的「改式不看引用」，不重复。
- 读法三（我提的，只在推理里，被攻过零轮）：只把「钉带出来的那一截」不看引用并进来——释放代落在 (盘上门槛, 不带钉现算的门槛] 的已释放槽并进，其余照今天按引用。没有钉时那一截是空的，C684 那一形照旧红（与用户定的「不并」一致）；C773 那一形 (21, 25] 正是这一截，转绿。没量：C684 那一形同时带钉时那一截罩不罩得到它那一槽。

### Y4.5 C684 那一形的结局与「不并」一致吗

取决于 Y4.4 选哪一读：读法一、三一致（仍红、测试照旧钉成例外），读法二不一致。形态条文要写明是哪一读；不写的话「C684 不变」只在读法一、三下成立，而读法一修不了 C773。

## Y5　四个形态一起

1. **Y2 × Y3 在 `:656` 与第一轮那 50 格上（量过，副本）**：`:656` 那一形在「今天 / 走读 / 并集」× 「恰好相等 / 同实例」六种组合下三处都落 (1,8)、都做得成（`out/same.out` 的 `pinned656` 两行）；第一轮那 50 格（这里的 `witnessed` 与 `unwitnessed` 两盘，加 `witness_lost`）上同实例那一半的变化表与恰好相等那一半逐行相同（`R2UNIFY_TALLY` 的 `rule=same` 与 `rule=exact` 去掉取法名之后 diff 为空）：这些盘面里表读不出的只有共用实例表那一格，那一格落 P 还是 Pm 只差在只读挂载打开哪一版，两种取法都算「落回上一版」。
2. **Y2 × Y3 在 Y2.1 那一形上（量过）**：统一判退取走读或并集时，恰好相等那一取法三处都落已确认的 R = (1,9)、恢复与只读都做得成；同实例那一取法三处都落 Row = (2,10)，只读做得成、恢复照报失败（`walk/same: rec=Fail@(2,10) ro=Ok(2,10) rw=Fail@(2,10)`）。统一把 Y2.1 的坏结局从恢复一处扩到了「三处同一版」——条文上更整齐，实际上恢复读不到本来读得到的已确认版本。
3. **Y1 × Y4（推的）**：错位读按自证不过处置之后，那一槽在挂载时归「读坏」，钉在 r + 3s（Y4 开头那一段），进位图；checker 现算照位图取同一个数，求等自洽，门槛只会更低（更保守）。错位读本身 checker 看不见（盘面上那一槽是对的），Y4 的求等也判不出「这一槽为什么钉着」——与 Y4.3 第三、四行同一类。持续错位的槽靠「这个进程写掉那一槽」解钉，最多一圈。
4. **Y3 × C754（读码）**：见 Y3.3，统一落点要连「交 F2 的记录取哪一份」一起写。

## 打中之后的四句

| 打中 | 分不分辨臂 | 系统当时看得到判别它的东西吗 | 满足判据字面的哪一个分句 | 跑前条款给的改法在打中的格上还中不中 |
|---|---|---|---|---|
| Y2.1 同实例形态让恢复失败 | 分辨：今天的「只认恰好相等」不中（恢复 `Ok(1,9)`），这一轮的「同实例当作不被抛弃」中（`Fail@(2,10)`） | 看得到：所选根、见证到的那一版、候选根的实例代号与实例表指针都在根记录里，不用读那张表 | Y2 第二问「F2 其余许可格的结局有没有变」：变了，恢复从读得出变成照报失败（挂不上本来挂得上的那一路） | 这一轮的形态：中；第一轮登记行的 `F2_NO_SKIP=2`：没在这一形上量；Y2.4 那条取法：不中（推的） |
| Y3.1 取打开或建上一版判退 | 分辨四个候选：打开、建上一版中（恢复少挂上），走读、并集不中 | 看得到：三样读者在所选那一版上读不读得通，挂载那一刻都能现跑 | Y3 第二问「有没有一格比今天少挂上」：有，单故障加一次断电 | 正文只说「同一种失败」，没定哪一样；取走读或并集不中，代价是只读 34 格、可写 8 格落回上一版 |
| Y3.3 交 F2 的记录不同 | 条文层，不是臂 | — | Y3 第三问「三处给判法的记录是不是同一份」：不是 | 补丁的 C754 重扫不碰 F2 的入参，不中这一格 |
| Y4.2 现算用 F_生效 | 分辨「用根自己的 F」与「用 F_生效」两读：后一读在合法状态上红 | 看得到：根带的 F 与系统配置里的 F 都在盘上 | Y4 第二问「合法历史会不会误红」 | 用根自己的 F，或照 I-3.1 那一格不判：不中（推的） |
| Y4.3 坏实现判不出 | 不分辨这一轮的形态与第一轮补法以外的取法：后两类在任何只看镜像的判据上都判不出 | **看不到**：重读那一刻读坏没读坏、挂载那一刻哪几槽读坏，镜像上不留痕（判别子观测不到，要改判据：只能把「钉的理由」也落盘，或接受） | Y4 第二问「坏实现会不会判出来」 | 补第一轮那一句：只修「写掉不解钉」那一行 |
| Y4.4 I-3.1 用哪式 | 分辨读法一、二、三：一修不到 C773，二翻 C684，三两头都对（推的） | 看得到：盘上门槛与位图都在根记录里 | Y4 第三问「C684 那一形结局与用户定的不并是否一致」 | 读法三（零轮） |

## 几个改法各修哪一格

| 改法 | Y2.1 跨实例见证 | C751 登记形只读 | `:656` | Y3 恢复少挂上 | Y3 只读、可写落回上一版 | Y4.2 F | Y4.3 写掉不解钉 | Y4.4 C773 / C684 |
|---|---|---|---|---|---|---|---|---|
| 这一轮四个形态照字面 | 坏（量过） | 修（量过） | 不变（量过） | 看取哪一样：打开 / 建上一版坏（量过），走读 / 并集不坏（量过） | 走读 / 并集：34 + 8 格（量过） | 坏（推的） | 坏（推的） | 没写（推的） |
| Y2.4「见证到的那一版与所选根同实例才收同实例」（我提的，只在我的模型输出上逐格推过、被攻过零轮） | 修（推的） | 修（推的） | 不变（推的） | — | — | — | — | — |
| C755 同一种失败取走读（或并集） | — | — | 不变（量过） | 修（量过） | 留着（量过） | — | — | — |
| C773 现算取根自己的 F（我提的，零轮） | — | — | — | — | — | 修（推的） | — | — |
| C773 补回「钉着的槽不住着这个实例写过的根」（第一轮攻方补法里的那一句） | — | — | — | — | — | — | 修（推的） | — |
| I-3.1 读法三：只把钉带出的那一截不看引用并进（我提的，零轮） | — | — | — | — | — | — | — | C773 绿、C684 仍红（推的） |

## 没打中的形状

- Y1：根环洞（根槽写失败推进、新实例跳号、崩在记录之后根槽之前）、跨区域回绕、同 txg 两根，按「写者只有 `target_for_publish` 与 mkfs 种根」逐类核（推的）；例外放行的错位读只放行第 0 代根，后果逐处核（推的）；十处入口加取号那一处（推的）。
- Y2：同一实例里被抛弃的根——找不到写这种行的路径；实例代号重用归 C555（推的）。`fault=table`（不带系统配置故障）的 10 格两种取法逐格相同（量过）。
- Y3：`witnessed` 那一盘 50 格四种候选与今天逐格相同（量过）。
- Y4：字段宽度、跨挂载、回退、取号之后的延续（推的）。
- 取样：Y2 四组（ow、sw3 × k = 3、5）× 这次挂载的每条根（k = 3 三条、k = 5 两条）× 两种故障，加登记形 4 组、`:656` 2 组，共 26 格；Y3 2 种 R × 2 种 k × 3 种盘面 × R 写出的每个单元加实例表，共 150 格。两份测试真跑合计约 54 秒（6.71 秒、51.04 秒），没有缩。

## 这条腿自己的限度

1. 统一之后与同实例之后的结局都在测试里现算（`three_places`、`fall_back_to`），core 没改；照抄的判法与 core 逐格比过（26 格全同），可写那一列只量建上一版，不含重建分配器读最新根实例表那一拒（Y2.1 那一形真跑可写今天就拒）。
2. 崩溃状态没照崩溃点重放的枚举域取：每段历史只取「某条根的根槽之后、它那次轮换之前」与「整次落盘」两种崩溃点，没调 `enumerate_crash_states`，经枚举函数跑的状态数是 0。打中的几格只要一个反例；「`witnessed` 那一盘四种候选与今天相同」「不注读故障时不越界」这类否定句只在这几个崩溃点上成立。
3. 「一样故障」照 C751、C755 登记行的数法：一个单元两份同时读回全零记一样，每块盘较新的系统配置槽同时读回全零记一样。
4. Y4 全部是读码推的，没实现字段、没跑 checker；Y2.4、Y4 的几条改法是我提的，被攻过零轮。
5. 只量了 384 槽小盘一种几何、S = 8。
6. 副本上的数，不是入库装置上的数。

## 进程与负载

开跑前 `ps` 看到别的会话的 `cargo test`（checker 档、e71 / e97 / e186 几个实验、变异复跑）、一条双机实验在跑，没有 `qemu-system`、`fio`、`vm-bench.sh`、`e152-file-system-benchmark`；照常 `nice -n 19`。内存上限派发提示没给，先取 `scripts/experiments/replay.sh` 的默认 8G：第一次排了 902 秒、第二次（两份测试并进一次包装）排了 1928 秒都没排上（`--status` 显示 slice 账上长跑的几条合计约 34 GiB 一直占着、总上限 40.1 GiB，8G 一直放不下），我停掉自己那两次排队（各停一个 `proc.py stop`，命令一行都没跑，包装退 251），改用 2G（比默认严）排了 47 秒起跑，两份测试各退 0；复跑脚本那一次用 4G（要在新树里编译）。工具把两次超过 240 秒的前台命令移到了后台，都等到完成通知、没有留着的进程。

## 要主 agent 跑的命令

没有必须的。判决要复跑数时照「复跑」一节：工作目录仓根，`--frozen` 用我留着的开工拷贝 `/home/fy5090/.local/share/singlefs/work/m3-close-rootring-r2-opus/repo/`（`crates/` 全部 `.rs` 的 sha256 清单再取 sha256 是 b523d85891b7327b57d1a760417241421b4749bd7891d00f019189826e375eb2，与开工那一刻拷出来时相同），内存上限 4G 或更大（脚本里要在新树里编译），线程 31；看 `name=rerun_compare` 两行都是 `same=yes`。

## 没做什么

- 没改 core，没实现任何一笔；同实例、统一判退的结局都在测试里现算，C773 那一格全是读码推的。
- 没打系统配置落地补丁，没在补丁后的树上攻；补丁只读过 C754 那一处重扫（它不碰 F2 的入参）。
- 没调崩溃枚举函数，没跑 checker 档、崩溃点重放、GPU 判器、变异。
- 没判正推那几格，没替主 agent 采纳，没读禁读清单里的文件。
- 草稿目录里留着仓副本 `/home/fy5090/.local/share/singlefs/work/m3-close-rootring-r2-opus/repo/`（开工时 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default` 的拷贝，约 35M，编译目录与我的测试已删）：判决复跑时当 `--frozen` 用，主 agent 用完可删。复跑那一次的新树与两个编译目录（各约 1.9G）已删。

## cite-check 末行

```
  ✓ 核了 12 处引文、13 处裸行号，对上 12 处，没判 13 处（其中裸行号没有引文可比 13 处；逐处列在上面）
```
