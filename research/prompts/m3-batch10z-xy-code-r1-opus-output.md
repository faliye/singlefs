# m3-batch10z-xy-code-r1 攻方腿（opus）报告（2026-10-10）

轮名 m3-batch10z-xy-code-r1（代码轮第一轮）；攻正文第三节 K1–K4，重点 K1-1、K1-2、K2、K3。冻结副本 `~/.local/share/singlefs/work/m3-batch10z-xy-code-r1/tree/`（只读，开工与收工各 `sha256sum -c` 对 `research/prompts/m3-batch10z-xy-code-r1-snapshot/crates-sha256.txt` 一次，都过）；下文代码行号都是冻结副本的行号。量出的数都在草稿目录 `~/.local/share/singlefs/work/m3-batch10z-xy-code-r1-opus/` 的副本上，不是入库装置上的数。

## 复跑

```
bash research/prompts/m3-batch10z-xy-code-r1-opus-model/rerun.sh [<工作目录>]
```

- 从冻结副本拷仓，`assemble.sh` 把那份 harness 测试文件与 `atk_xy_*.rs` 拼成测试目标 `atk_xy_opus`（只借原文件的搭建函数，攻方用例都以 `atk_` 开头、`#[ignore]`；`a*`、`b*` 是上一轮 `m3-batch10z-vw-code-r1-opus-model/atk_vw_part*.rs` 拷来改的，`c1`–`c4` 是这一轮新写的）。
- 依次跑：K1-1 上一轮那几族形（K1 九形 12 格 × 21 种动作、实现员三形、两槽 C、被抛弃根 A 与 C、窄四档 × 11 种动作）；H 形推完放得下（`c1`）；推环入口照收次序对调的盘表（`c2`，连同另一条臂 `ARM-push-checks-the-table-order`）；别的会话同一对数（`c3`）；两道判的先后（`c4`）；攻方变异与变异表第 3603–3623 行；扫源码小程序（`build-scan-proto.py` 从两份 checker 档源文件的 `mod tests` 机械抽出读自己源文件的用例）喂 `gen-scan-inputs.py` 造的写法；第 3624–3640 行与 B1–B4 施加在抄出来的源文件上；K4 两棵冻结副本比对。
- 每件经 `research/scripts/run-with-memory-cap.sh`（并行件 3G，一批至多 4 件；编译 12G；扫源码 4G / 2G），线程经 `research/scripts/capped.sh`（合计不超过 7）。不起 checker 档测试、门禁阶段、崩溃枚举。不用 `set -e`，一步失败照记、接着跑；读参数用 `mapfile`。
- 限度：`rerun.sh` 没有一口气整份跑过（前台一次至多 4 分钟、子 agent 不起后台），是把它里面的每一段命令分批照原样跑过、日志落进 `runs/`；`bash -n` 过。整份估约 40 分钟（推的：这一轮各段实测相加，机器上另有 cargo 在跑时更长）。
- 原型里的盘面是从录下的写流里截一个前缀（崩在 R 第一条记录之后）得来的一个崩溃状态，不是层 0 的整写子集枚举；派发不许起崩溃枚举，没调 `enumerate_layer0`，也没有别的崩溃状态。

模型目录文件的 sha256（源文件逐个列出；`runs/` 下 196 份原始输出连同源文件的 sha256 在模型目录的 `SHA256SUMS` 里，221 行，`cd` 进模型目录后 `sha256sum -c SHA256SUMS` 核）：

```
a9c012db7702e6af1fe387cc54776ccd0da9237ebed6d2272f2131b5137d466f  SHA256SUMS
ea51c321966f42df1ac22224256e20aeed76225802e1e20669d69f4eda265bfb  ./apply-row.py
09e6b7161ac309f5e4b050b8d0845e5b1db11d5efaf02afb0195aa5ec68b5cb9  ./assemble.sh
03ce1547b78050f491302c09f53f9475808509037998ffbcb3a033626a52d3d2  ./atk_xy_a1.rs
239e0cdc6770cde8b3b43144edb1cb0ce1f8f9e0e0cee5e1184b872f1671fe14  ./atk_xy_a2.rs
f0e5f602a06970194aa1900ee77e24338a2d7d347ac1b061a4bb4b8273c3aced  ./atk_xy_a3.rs
5cd1cfe35be941db41e6a82c03c46556030c9325892bb2e7c88e70d6a648de4e  ./atk_xy_a4.rs
9d3b52cbf8ae1fe8197b2ed43482373512099dab11b7e00f76d7767c4ab3b5e5  ./atk_xy_a5.rs
1b3c824f6cf3aa1fdba7be6a59828e6a6a2a3e10e382692bc70fce9f2ac77c31  ./atk_xy_a6.rs
bac60f2194d2caadc5e3bd776f1577bd2e7acaf44c37b9772e30a04ab515e1f5  ./atk_xy_a7.rs
4ec92b5b4c4f7aa83e3ff3f5d368105802f63c48747f660f509ebbce822c76d5  ./atk_xy_a8.rs
53d782ef421561134caeb8fc3ba48b44800bbdbcf8d92cde954be64811917adf  ./atk_xy_a9.rs
e32418bb1f3870332053c2fb8829e498275ab77313b56558e1d482dc03394e41  ./atk_xy_b1.rs
435558d98690b0c218def675f4efce98e6e8f9ccdb4b1d7f2ffd0e039d52d05c  ./atk_xy_b2.rs
7ba3514cbdc8b1ea30b81cd97979f79fa8f8d62086372dc05b4e42324660ba3d  ./atk_xy_c1.rs
680a7ddfbdd94856a3b3f75f5801270d2736bbedd50e7f49e3dd1a336e95a0e1  ./atk_xy_c2.rs
e81e0102374cdc6ceafbc3d331f31cec510aa8e545e523bd90932b34d32a31c4  ./atk_xy_c3.rs
c95a2d8eea37e3c9e2573089ba94c11745650c1ffafa195500550110c16a611b  ./atk_xy_c4.rs
adfb41236a0aeadb07427dc7855c76e0ffa17a3b1e95a1138093faf3eb8dc2c6  ./batch.sh
7ce41761672914a079777271a67671a93c85c8ea2b3e2ed042b920b78e8db29e  ./build-scan-proto.py
65c84c15113c5b52305dfefd2d1b5bb508d615a1f68fa5df57305c9afb2cc01e  ./gen-scan-inputs.py
b27c7475248bd559014bf89fa661ed4b65a9f2b04db82a75f037f81b8eb68faf  ./mutants.py
16dd5add32e964b362babb66a718e836be4e7fe23446d5f078f5e85ac4c9fc36  ./rerun.sh
f70f4b34d89d1f163146ffbf6f33a01a96070c33306e8dad0cc767298139502c  ./run-mutant.sh
76444409a7ff812d0ff19fbb219eb60df9dca4aab653531ad0aa4edc61135cfa  ./scan-mutant.sh
f08697e55d045f3f1806dcb8ab24fdc73b36178976fa8a5453ba12fa50531545  ./scan-mutants.py
```

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| K1-1 上一轮那几族形逐格重扫 | 没打中 | 1336 个卡住的格 × 动作共 17526 格：比得了的 15912 格 0 处对不上；`table-reversed`、`table-swapped`、`table-one` 各 1336 格全拒成新成员、`ask_io=(0,0,0)`、会话与盘不动，照同一份盘表真走也走不通；`table-stale` 次序对、照常预演、推环被拒，与真走同结局 | 一 |
| K1-1 新形：H 形推完放得下 | **打中（窄，注入级）** | unlink 卡在意图之后的走空里、推环做得成、推完走空放得下：要出路列推环、文字写「预演过：推完之后这次改动放得下」，照出路二推完再发同一个 unlink 却拒成 `NoFileVersionToChange`（3 格）；文档第 1254、1259 行说的「再发同一个改动」在这一形上不成立 | 一 |
| K1-1 有没有该预演而被拒、该拒而照样预演 | 没打中 | 新成员只在次序、块数不符时出；次序对而盘不对（`table-stale`）照常预演、推环那一套核拒，与真走同结局 | 一 |
| K1-2 推环入口照收次序对调的盘表 | 没打中（两臂都给了） | 今天：177 格推环结局、按段的写流、推完盘面与会话、再照挂载时盘表发同一个改动、崩了再挂，逐样与照挂载时盘表推的相同（整条流的先后不同）；「推环入口也先比次序」那一臂：177 格在任何写之前拒、写流 0 条，换回挂载时盘表再推就与今天相同 | 二 |
| K1-3 照实写的那一段文档 | 没打中（两处措辞，窄） | 逐句对得上；两处：文档列的「同一个池卸载或崩了之后再挂」那一种撞不上（再挂取新实例，283 格全是 `moved[2.t->3.t]`）；预演发的是报里那一份改动，文档没说，别的会话自己发它可能根本不被拒 | 三 |
| K2 第二十四块 21 行 | 没打中 | 21 行各自点名的用例全红；上一轮活下来的 `walkSkipped` 红 | 四 |
| K2 第二十四块还有照绿的改坏 | **打中（窄）** | N1：盘表那一道挪到版本那一道之前，那份测试文件 66 条与 core 库单测 226 条全绿，而会话已动又交次序对调的盘表时交回从 `moved` 变成盘表那一种（64 格）；N8 照绿但等价 | 四 |
| K2 第二十五块 17 行 | 没打中 | 17 行在扫源码小程序上各自点名的用例全红（W8、W9、X6、X7 是第 3624、3625、3634、3635 行） | 四 |
| K2 第二十五块还有照绿的改坏 | **打中（窄）** | 「方括号也在第 0 层」那一半没有用例：两份切口只数圆括号（B2、B3）照绿，此时包进 `keep_hidden![…]` 方括号里、前面先放一条以分号收尾的条目的真测试模块（rustc 编得过，非测试构建里可调用）切口照认 | 四 |
| K3 点名的写法与对照 | 没打中 | S14、S14b、S19、G14、G16、N01、N02 都红；G15、G17 照旧红；原样绿；N03、N15、三处「取第一处」都在名单里 | 五 |
| K3 新加两道条件误伤合法写法 | **打中（窄，文档）** | 括号那一道：字符字面量、生命周期、各种字符串、注释、属性、`macro_rules!` 在头两行前，0 误判 0 panic；卡池那一道：卡池模块挂 `#[cfg(feature = …)]`、别处另有一个叫 `card_pool` 的模块、卡池外另有一个 `fn lease_card`、借据丢掉时那一步挂 `#[cfg_attr(…, allow(…))]` 四种合法写法判红，故意拒的名单里都没写 | 五 |
| K4 E171 走的路径 | 没打中 | `mount.rs` 逐字节相同、`refusal_is_short_of_space` 函数体相同；`mounted_session.rs` 测试之外的代码改动只有新成员与要出路里那一道核；两份 checker 档文件测试模块之外逐字相同 | 六 |

## 一、K1-1：新加的次序核在上一轮那几族形上；H 形推完放得下（打中，窄）

### 被判的那几句

- `crates/singlefs-core/src/mounted_session.rs:1261`：「第一道看会话还在不在被拒的那一版上（实例代号与 txg 都与报里记的相同），再比盘表：哪一道不过都不预演，一个盘都不读。」
- `crates/singlefs-core/src/mounted_session.rs:1259`：「核不过就是推环被拒、出路只列卸载再挂），与照出路二真推、再发这次改动时同一个结局。」
- `crates/singlefs-core/src/mounted_session.rs:1254`：「与真调推环入口、再发同一个改动走同一段），照结局列出路（推完放得下才列推环）。」

### 上一轮那几族形逐格重扫（`atk_xy_a*.rs`，`runs/k1-pre*-g*.log`、`runs/fam-*.log`、`runs/area*-[AB].log`）

- 形与上一轮同（K1 九形 283、实现员三形 33、两槽 C 245、被抛弃根 A 与 C 209、窄四档 223 + 164 + 123 + 56），合计 1336 个报卡住的格。被拒之后、要出路之前放开的动作：K1 九形 21 种，别的族 11 种（`none,same,table-reversed,table-swapped,table-one,table-stale,frozen-overwrite,inode-then-same-1,overwrite-then-same-1,ask-then-same,push-fail-1`）。
- 比法照上一轮（`atk_compare`：预演 `adm` ⇔ 照出路二推环做成且再发放得下；`still` ⇔ 推环做成且再发被拒、两份拒绝 `Debug` 逐字相同；`pushref` ⇔ 推环被拒；放出来的槽数逐盘相同；出路条数与结局对得上；会话与盘不动）。新加一判：要出路拒成盘表那一种（`tableref`）⇔ 拿同一份盘表照出路二走不通（推环被拒，或推得过、再发拒成 `DeviceTableOtherThanTheOneOfTheMount`），且这一次探物理块大小、读、改盘调用都是 0。
- 合计（`grep ' act='` 数出来的）：17526 格，`agree=yes` 15912、对不上 0；不比的 1614 格 = 要出路拒成 `moved` 的 1391（inode、overwrite、push-reversed、unmount-remount、push-fail-150 那几种动作让会话动了）+ 原样再发直接放得下、没有新报的 220 + 实现员三形没有旧盘面、`table-stale` 造不出的 3。
- 盘表四种（每种 1336 格）：

```
table-reversed total=1336 tableref_io0_untouched=1336 agree=1336
table-swapped total=1336 tableref_io0_untouched=1336 agree=1336
table-one total=1336 tableref_io0_untouched=1336 agree=1336
table-stale total=1336 tableref_io0_untouched=0 agree=1333
```

`runs/k1-pre4-fill16-g2.log` 与 `runs/k1-pre4-fill16-g3.log` 各一行原样：

```
ATK k1 pre=4 fill=16 g=13 n_pub=19 act=table-reversed: restored_ask=adm ask=tableref[[0, 1]->[1, 0]] ask_io=(0,0,0) ask_untouched=true push=pushed(19) retry=other(DeviceTableOtherThanTheOneOfTheMount { device_table_of_the_mount: [DeviceIdentity(0), Devi) agree=yes
ATK k1 pre=4 fill=16 g=13 n_pub=19 act=table-stale: restored_ask=adm ask=pushref[DevicesHandedIn:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot] ask_io=(2,45,0) ask_untouched=true ways=1 push=push_refused[DevicesHandedIn] retry=- agree=yes n_reh=[] n_real=[] n_same=true dbg_same=true ways_ok=true
```

读法：上一轮 283 格全对不上的 `table-reversed` 现在在预演之前拒、一个盘都不读，换回挂载时那一份盘表（`restored_ask`）照常预演；照同一份次序对调的盘表真走，推环做得成（推环入口不看次序，见第二节），再发被拒成盘表那一种——「这份盘表走不通」两边一致。`table-swapped`（两块盘身份对调、盘不动）身份次序也变了，同样在预演之前拒。`table-stale`（次序对、第二块是旧盘面）过了新判、照常预演（读了 33–50 次），预演里推环那一套核拒成 `DevicesHandedIn`，与真推同结局；这一形不在新成员文档说的范围里，文档第 1259 行说核不过就是推环被拒，对得上。没找到「该能预演而被拒成新成员」或「该拒成新成员而照样预演」的形。

### 新形：H 形推完放得下（`atk_xy_c1.rs`，`runs/k1h.log`）

- 造法：实现员新用例那一形（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:6519`：「const ISOLATED_FREE_SLOTS_THE_INJECTED_RING_SLOT_HOLDS_ON_EACH_DEVICE: u64 = 4;」，注入级：C 那一圈整个换成现行那一版之后第 k 次发布写的那一槽、每块盘记着 iso 个隔离的空闲槽，没挡的槽只留 left 个），三个注入量放开：k ∈ {1,2,4,8,16}、iso ∈ {4,8,16,32,64}、left ∈ {0,5,10,20,30}，共 125 格；unlink 第一个文件。报卡住的，要出路，再照出路二真走三种后续：推环之后原样再发同一个 unlink（`resend_unlink`）、推环之后建一个 inode（`then_inode`）、只推环（`push_only`）。
- 原样（k=4、left=30；iso=4 就是实现员那一条用例的形）：

```
ATK k1h k=4 iso=4 left=30 stuck-H published=1 n_pub=1 remainder=remainder_of_the_change: WalkTheQueue } ask=still[NoSpaceAfterRaisingTheFloor] ask_io=(2,141,0) ask_untouched=true ways=1 | real: resend_unlink=pushed(1)->notwhat(ceiling) then_inode=pushed(1)->notwhat(ceiling) push_only=pushed(1):WithFile:queue_empty=false
ATK k1h k=4 iso=16 left=30 stuck-H published=1 n_pub=1 remainder=remainder_of_the_change: WalkTheQueue } ask=adm ask_io=(2,180,0) ask_untouched=true ways=2 | real: resend_unlink=pushed(1)->other(NoFileVersionToChange { floor_raises: [], refusals_that_pushed_the_floor_raises: [], versi) then_inode=pushed(1)->notwhat(raise-refused) push_only=pushed(1):WithFile:queue_empty=false !ADM_BUT_RESEND_REFUSED
ATK k1h k=4 iso=32 left=30 stuck-H published=1 n_pub=1 remainder=remainder_of_the_change: WalkTheQueue } ask=adm ask_io=(2,180,0) ask_untouched=true ways=2 | real: resend_unlink=pushed(1)->other(NoFileVersionToChange { floor_raises: [], refusals_that_pushed_the_floor_raises: [], versi) then_inode=pushed(1)->notwhat(ceiling) push_only=pushed(1):WithFile:queue_empty=false !ADM_BUT_RESEND_REFUSED
ATK k1h k=4 iso=64 left=30 stuck-H published=1 n_pub=1 remainder=remainder_of_the_change: WalkTheQueue } ask=adm ask_io=(2,180,0) ask_untouched=true ways=2 | real: resend_unlink=pushed(1)->other(NoFileVersionToChange { floor_raises: [], refusals_that_pushed_the_floor_raises: [], versi) then_inode=pushed(1)->notwhat(ceiling) push_only=pushed(1):WithFile:queue_empty=false !ADM_BUT_RESEND_REFUSED
ATK k1h   way_two=出路二：调 MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states（推环到 R 那一槽，根环槽 2.6）：从现行那一版起发 1 次 F 不变的空发布，写掉根环里接下来轮到的 1 槽，R 那一槽是最后一槽；宽护随之解开，隔离的槽放出来 32 个（盘 0 上 16 个，盘 1 上 16 个）。这几槽里原来的根换成同一个用户状态的空发布根，回退候选集可以掉到少于 4 个不同状态（D16（发布语义） 已定项 1），调用即确认。预演过：推完之后这次改动放得下。
```

- 合计（`runs/k1h.log` 的 totals）：H 形（意图已发成、卡在之后的走空里）报卡住 15 格，其中 `adm` 3 格，3 格照出路二推完再发同一个 unlink 都拒成 `NoFileVersionToChange`（拒绝里 `versions_published` 不空：再发那一次开头的走空走完了，接着发 unlink 时这个池已没有文件）；意图那一次就卡住的 `adm` 2 格再发都放得下。不注入 C 那一圈、只压没挡的槽（k=0，left 0 到 80 每隔 2，`runs/k1h-natural-c.log`）H 形只有推环被拒的 1 格，没有 `adm`——这一形只在注入级出现，与实现员那条用例同一级。
- 机理（读码）：H 形报里记的这次改动还剩什么只是走空（`WalkTheQueue`），预演在推完之后只走空（`crates/singlefs-core/src/mounted_session.rs:683`：「RemainderOfTheUserChange::WalkTheQueue => {」那一支），走空放得下就列推环、写「推完之后这次改动放得下」；可这次 unlink 的意图已经发成，照文档「再发同一个改动」再发，会话先走空（走完）再发 unlink，这时被 unlink 的那个文件已不在，拒成 `NoFileVersionToChange`（池里另有文件时推的是 `UnlinkOfAnInodeThatIsNotALiveRecord` 一类，没造）。预演说的「走空放得下」本身是对的（再发那一次的走空确实走完了），错的是「再发同一个改动」这个走法与出路二末句的说法。

### 四句（H 形打中）

1. 分不分辨臂：分辨「出路二与文档照『再发这次改动』说」与「H 形（报里只剩走空）按『意图已发成、推完之后剩下的走空放得下、不要再发这次 unlink』说」；预演本身不用改。
2. 系统当时看不看得到：看得到，报里的 `remainder_of_the_change` 是 `WalkTheQueue`，要出路那一刻就分得出这一形。
3. 满足的分句：K1-1「要出路交回的结局与照出路二真推环再发同一个改动的真结局是否相同」——预演 `adm`、真走推完再发同一个 unlink 被拒，不同；也打中 `mounted_session.rs` 第 1254、1259 行两句文档的字面。
4. 改法（推的、被攻过零轮，没实现）：`ways_out_after` 在报里只剩走空时换一句出路二末句（意图已发成，推完之后剩下的走空在下一次改动开头走、放得下；不必再发这次 unlink），第 1254、1259 行的「再发这次改动」限定为报里还剩这次改动时；补一条注入级用例：k=4、iso=16 的这一形要出路列推环，照出路二推完再发同一个 unlink 被拒、推完之后发别的改动时开头的走空做成。这一改法只改说法与文档，碰得到打中的 3 格（推的）。

推翻条件：在入库装置上造出这一形（k=4、iso ≥ 16、left=30），推完再发同一个 unlink 放得下；或证明调用方在这一形上本来就不会再发这次 unlink（例如文档别处写明了意图发成之后不再发）。

## 二、K1-2：推环入口照收次序对调的盘表（没打中；两臂结局都给出）

### 被判的那几句

- 推环入口核盘表走 `crates/singlefs-core/src/mounted_session.rs:1358`：「let parameters_of_the_pool = parameters_of_the_pool_agreeing_with_the_caller_inputs(」——重复身份、系统配置、设备数与每块盘的本盘设备号、「可见」、落后支，不比次序；要出路那一边先比次序（第 1301 行）。
- 读码：写入口按身份找盘（根槽那一写 `crates/singlefs-core/src/transaction.rs:459`：「.find(|(identity, _)| *identity == region_device)」）、位置条目按身份排序（`crates/singlefs-core/src/make_filesystem.rs:273`：「sorted.sort();」），按下标走的只有「每块盘都写一份」与系统配置槽轮换那几处（写的先后跟着盘表次序变）。

### 造法（`atk_xy_c2.rs`，`runs/k12-pre{2,4,8}.log`、`runs/k12-arm-pre{2,4,8}.log`）

K1 九形每个报卡住的格（g 从 1 扫到 48），在被拒那一刻的拷贝上推环两次：照挂载时那一份盘表、照次序对调的那一份；盘外面包一层录写流。逐样比：推环结局 `{:?}`、写流按屏障与 FUA 切段之后每段的写的多重集（层 0 的切法：段段相同，段内整写子集就是同一批崩溃状态；没有枚举）、推完的盘面、推完的分配器与现行那一版、再照挂载时那一份盘表发同一个改动的结局与盘面、崩了再可写挂载一次的结局（实例代号、择到的根）。阳性对照：整条流不切段、按原样先后的散列（`same_order_unsplit`）。另一条臂（推环入口也先照 `publish_user_change` 那样比次序，不符在任何读写之前拒；臂的造法是 `mutants.py` 的 `ARM-push-checks-the-table-order`，借 `DevicesHandedIn` 装拒绝，不是提案的报法）在另拷的副本上跑同一个用例。

```
# runs/k12-pre2.log、k12-pre4.log、k12-pre8.log 各一行
ATK k12 totals {"push_m=Ok push_r=Ok differing=[] same_order_unsplit=false": 87}
ATK k12 totals {"push_m=Ok push_r=Ok differing=[] same_order_unsplit=false": 48}
ATK k12 totals {"push_m=Ok push_r=Ok differing=[] same_order_unsplit=false": 42}
# runs/k12-arm-pre2.log
ATK k12 totals {"push_m=Ok push_r=Err differing=[\"result\", \"segments\", \"image\", \"session\", \"then\", \"image_after_then\", \"remount\"] same_order_unsplit=false": 87}
ATK k12 pre=4 fill=16 g=13 push_mount=Ok(RootRingPushed { versions_published:  push_reversed=Ok(RootRingPushed { versions_published: [TransactionOutput { root: RootRecord { filesystem_identifier: [95, 83, 70, 83,  ops_mount=437 ops_reversed=437 segs=76/76 same_order_unsplit=false then_mount=admitted then_after_reversed=admitted differing=[]
```

（第四行是 `runs/k12-arm-pre2.log`；另两份 ARM 是 48、42 格，同一个 `differing` 列表。）

### 两臂在同一组形（7 个 K1 形、177 个报卡住的格）上的结局

| 臂 | 推环那一次 | 写落在哪 | 之后的发布 | 回退候选集、C 那一圈 | 量过还是推的 |
|---|---|---|---|---|---|
| 今天（推环入口不比次序） | 177 格全做成，结局与照挂载时盘表推的逐字相同 | 写的先后变了（177 格整条流散列都不同），按段的多重集、推完的盘面逐字节相同：写落在对的盘上 | 照挂载时盘表再发同一个改动，结局与盘面相同；照次序对调的盘表再发拒成盘表那一种（第一节 1336 格） | 推完的分配器（含根环表、C 那一圈）与现行那一版 `{:?}` 相同，崩了再挂择到的根相同 | 量过 |
| 推环入口也先比次序 | 177 格在任何写之前拒，写流 0 条、盘不动 | 不写 | 不推环直接再发：照旧卡住（`then_after_reversed=stuck`）；换回挂载时盘表再推，走的是与今天同一段代码 | 换回之后同今天 | 拒的那一半量过；换回之后同今天是推的（同一段代码） |

读法：今天的做法下次序对调的盘表推环只改了写的先后、不改写什么写到哪，落盘、发布、回退候选集与 C 那一圈都与照挂载时盘表推的相同；两臂的差别只在调用方交错次序时推环做不做（今天做、结局对；另一臂拒、要再调一次），以及推环入口与要出路、`publish_user_change` 两个入口收不收这份盘表一致不一致（今天不一致：要出路拒、推环收、再发拒）。没有一格因为推环入口不比次序而写错盘或判错。限度：只扫了 g ≤ 48，K1 九形里 fill=6 那几形卡住的 g 在 56 以上，没进这一节；崩溃状态只比了按段的写集合，没枚举。

## 三、K1-3：`SessionMoved…` 照实写的那一段（没打中；两处措辞）

被判的那几句：`crates/singlefs-core/src/mounted_session.rs:273`：「这一道只比实例代号与 txg，报里不记是哪个会话报的。别的会话（同一个池卸载或崩了之后再挂、同一份 mkfs 起的另一个池、读盘造的另一条会话）」；`crates/singlefs-core/src/mounted_session.rs:274`：「的报若这两个数恰好与这个会话的现行那一版相同，这里不拦：照这个会话的状态（分配器、盘、现行那一版）预演，出路一的文字仍是报里的数」。

造法（`atk_xy_c3.rs`，`runs/k13.log`）：报的池 A 是 K1 那一形（pre=4、fill=16、长 13 个单元被拒，停在 (2,16)）；别的池 B 是 K1 另 9 格崩了的盘面各挂一次（实例代号都是 2），覆盖写推到 txg 16；拿 A 的报去 B 要出路，与 A 自己要出路、B 自己发同一个改动之后要出路、照 B 的状态真推环逐样比。7 个 B 撞上同一对数：

```
ATK k13 B=pre2:fill6 A_j=0 reached=true same_pair=true pair=2.16 ask_on_B=adm ask_io=(2,735,0) ask_untouched=true | A_own=Some(("2.3", "19", "102")) on_B_with_A_report=Some(("2.3", "19", "58")) B_own=None B_real_push=pushed(15) ring_slots_written=["1.2"]
ATK k13 B=pre2:fill21 A_j=0 reached=false same_pair=true pair=2.16 ask_on_B=adm ask_io=(2,666,0) ask_untouched=true | A_own=Some(("2.3", "19", "102")) on_B_with_A_report=Some(("2.3", "19", "122")) B_own=Some(("1.3", "18", "122")) B_real_push=pushed(18) ring_slots_written=["1.3"]
```

（三元组是出路二里的「R 那一槽、要推的次数、放出来的合计」。）

逐句对：不拦——对（7 格都照 B 预演）；出路一的文字是报里的 R 那一槽——对（7 格出路一与 A 自己的逐字相同，而 B 自己报的是根环槽 1.3）；出路二里 R 那一槽与要推的次数是报里的——对（写 2.3、19 次，照 B 真推是 15 到 22 次、写到 B 自己的那一槽）；放出来的槽数是这个会话预演出来的（58、78、122，不是 A 的 102）——文档没逐字说，与「照这个会话的状态预演」相容。两处措辞：

1. 文档列的第一种「同一个池卸载或崩了之后再挂」撞不上：再挂取新实例，现行那一版的实例代号随之变（第一节 `unmount-remount` 283 格全是 `moved[2.t->3.t]`、`remount=ok(instance 3)`；崩了再挂同取盘上最大加一，推的）。写进文档无害（只是多列了一种不会发生的），但读的人会以为同一个池重挂后旧报还可能被认。
2. 预演发的是报里拷下的那一份改动（A 的内容），文档只说「照这个会话的状态预演」；7 格里 6 格 B 自己发这一份改动直接放得下（`B 自己发：admitted`），要出路却照样列出路——出路对 B 不是「被宽护卡住」的出路。文档末句「调用方要拿报去问报它的那个会话」已经把责任交给调用方，这一处只是没写全。

四句：①不分辨臂（照实写的那一段与代码一致，没有对面的臂）；②系统看不到报是谁的（用户已定撤身份）；③满足 K1-3「与代码逐句对不对得上」——对得上，两处是文档列举与省略；④改法（推的、零轮）：第 273 行删去「同一个池卸载或崩了之后再挂」或注明撞不上；第 274 行补「预演发的是报里拷下的那一份改动」。

## 四、K2：判别力

### 第二十四块：变异表 21 行与上一轮活下来的变异（`run-mutant.sh`，每个从冻结副本另拷一份）

- 基线（不改）：那份 harness 测试文件 `ok. 66 passed; 0 failed`，core 库单测 `ok. 226 passed; 0 failed`（`runs/mut-baseline-*.log`）。
- 第 3603–3623 行逐行施加（原文恰好命中一次才改）、跑那一行点名的用例：21 行全 `FAILED. 0 passed; 1 failed`（`runs/mut-row-36xx-row.log`）。其中正推上一轮活下来的 `walkSkipped`（第 3606 行）、记进报里的字段 8 种（第 3607–3613、3614–3621 行两份）、`checkRequestBlock512`（第 3622、3623 行）都红；攻方上一轮活下来的 `walkSkipped` 用攻方自己那份变异（与第 3606 行同一原文）整份跑那份测试文件：`FAILED. 65 passed; 1 failed`，红在 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:6602`：「"推完之后走空照旧被拒：只列卸载再挂：{ways_out:?}"」那一句。
- `walkSkipped` 那条注入级用例钉的是不是条款说的那件事：钉的是「报里只剩走空、推环做得成、推完走空照旧被拒」这一侧——预演 `ChangeStillRefusedAfterThePush`、只列卸载、预演的拒绝与推完真再发同一个 unlink 的拒绝 `Debug` 逐字相同，`walkSkipped` 把走空那一支当成恒放得下就列推环、红，钉到了。没钉的是同一形走空放得下的那一侧（第一节 H 形打中的那 3 格：预演 `adm`、真再发被拒）——那一侧今天没有用例，也说不出「真走」该比什么（见第一节四句的改法）。

### 第二十四块：攻方新造的改坏（`mutants.py`）

| 变异 | 改了什么 | 那份测试文件 / core 库单测 |
|---|---|---|
| N1-table-check-before-the-version-check | 盘表那一道挪到版本那一道之前 | **ok. 66 passed / ok. 226 passed（照绿）** |
| N2-table-check-ignores-the-order | 盘表按集合比（排序后比） | FAILED 1 / ok |
| N3-table-check-only-the-count | 只比块数 | FAILED 1 / ok |
| N4-refusal-records-the-mount-table-as-handed-in | 拒绝里「交进来的那一份」记成挂载时那一份 | FAILED 3 / ok |
| N5-rehearse-once-before-the-table-check | 先预演一遍（读盘）再比盘表 | FAILED 4 / ok |
| N6-as-remainder-drops-the-change | 读回报里的改动时丢掉这次改动、只剩走空 | FAILED 8 / ok |
| N7-kept-from-keeps-only-the-walk | 报里只记走空 | FAILED 13 / ok |
| N8-block-bytes-512 | 只进内存的盘按 512 字节一块记 | ok / ok（照绿，等价） |
| N9-over-the-disk-takes-512 | 只进内存的盘探出来的物理块换成 512 | ok / FAILED 2 |

N1 照绿而行为变了（`atk_xy_c4.rs`，`runs/k2order-baseline.log` 对 `runs/k2order-N1.log`，K1 主形 32 个卡住的格 × 会话又动了两种）：

```
# runs/k2order-baseline.log
ATK k2order totals {"overwrite:admitted: reversed=moved[2.16->2.17] restored=moved[2.16->2.17]": 32, "push:pushed(19): reversed=moved[2.16->2.35] restored=moved[2.16->2.35]": 32}
# runs/k2order-N1.log
ATK k2order totals {"overwrite:admitted: reversed=tableref[[0, restored=moved[2.16->2.17]": 32, "push:pushed(19): reversed=tableref[[0, restored=moved[2.16->2.35]": 32}
```

会话已不在被拒的那一版上、又交次序对调的盘表：原样拒成 `SessionMoved`，N1 下拒成盘表那一种；调用方照它「换回挂载时那一份盘表再要出路」，下一次才拿到 `SessionMoved`。新成员的文档（`crates/singlefs-core/src/mounted_session.rs:283`：「会话还在被拒的那一版上（实例代号与 txg 与报里记的相同），交进来的盘表的设备身份」）说盘表那一种只在会话还在那一版上时报，N1 违背了它而 66 + 226 条全绿：两道都不过的那一形没有用例。四句：①分辨「先版本后盘表」与「先盘表后版本」两个次序；②系统看得到（两道都在同一个函数里、都不读盘）；③满足 K2「还有哪几种改坏的写法照绿」；④改法（推的、零轮）：补一条用例——会话已动、又交次序对调的盘表，要出路拒成 `SessionMoved`；量过：原样 64 格 `moved`、N1 下 64 格 `tableref`，这条用例在 N1 下会红。

N8 照绿是等价改动（推的，量过一半）：请求先过按探出来的物理块（4096）判的对齐，过了的偏移与长度都是 512 的倍数，按 512 记块与按 4096 记块读回同样的字节；core 库单测 226 条（含 4096 差分对拍）全绿。

### 第二十五块：变异表 17 行（施加在抄出来的源文件上，`scan-mutant.sh`）

checker 档测试不归这条腿跑。量法：`build-scan-proto.py` 从冻结副本两份源文件的 `mod tests` 机械抽出「读自己源文件」的扫源码用例与它们引的条目（`include_str!` 换成读 `SRC`，别的一字不改），在冻结原样上卡池 16 条、崩溃放量 11 条全绿（`runs/k3-gpu-frozen.log`、`runs/k3-crash-frozen.log`）。每一行施加在拷出来的那份源文件上、重抽重编、在改过的那份上跑点名的用例：第 3624–3640 行 17 行全 `RED`（`runs/scanmut-row-36xx.log`；W8、W9、X6、X7 是第 3624、3625、3634、3635 行）。

### 第二十五块：攻方新造的改坏（打中，窄）

| 改坏 | 点名的用例（改过的源文件上） | 方括号包住的 S14b（`c09`、`g09`） |
|---|---|---|
| B1 崩溃放量只在 `(` 上加一层（`)`、`]` 照减） | RED（「右圆括号、右方括号不多于左的」，冻结原样就 panic） | — |
| B2 崩溃放量只数圆括号（两边的方括号都去掉） | **GREEN** | **切口照认、判绿**（原样下判红） |
| B3 卡池只数圆括号 | **GREEN** | **切口照认、判绿**（原样下判红） |
| B4 返回类型 `bool` 后面紧跟标识符的字也算 | GREEN（方向是多报、不是放过） | — |

`c09`、`g09` 是 S14b 换成方括号的宏调用：`keep_hidden![` 起、前面先放一条以分号收尾的条目、再接真测试模块的头两行、收尾 `];`；同形的一个小文件 rustc 编得过，非测试构建里能调 `tests::hidden()`（草稿目录 `snip/brackets.rs`，只编不跑）。两份文档都写了「在花括号的第 0 层，也在圆括号、方括号的第 0 层」（`crates/singlefs-checker-tier/src/crash_amplification.rs:4661`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2645`），两条新用例里的宏调用只有 `keep_hidden!(` 与 `keep_hidden! {` 两种，没有方括号，方括号那一半拿掉照绿。四句：①分辨「圆括号、方括号都数」与「只数圆括号」；②扫源码看得到（源文件全在手里）；③满足 K2「还有哪几种改坏的写法照绿」——B2、B3 照绿，且照绿时放过一个真能把测试模块编进非测试构建的写法；④改法（推的、零轮）：两条切口新用例各加一形「方括号宏调用里、前面先放一条以分号收尾的条目」，量过：这一形原样下两边都红（`runs/k3-summary.txt` 的 `c09`、`g09` 行），B2、B3 下都绿。

## 五、K3：第二十五块扫源码（只核补法）

量法同第四节第二十五块那一段（抄出来的小程序，不是 checker 档测试的结局）。`gen-scan-inputs.py` 造 37 份输入：`-red-` 是点名的写法或阳性对照，`-legal-` 是合法写法，`-list-` 是名单里写明拦不住的。崩溃放量一侧跑主用例 `every_layer_handed_the_stop_request_hands_it_on_or_asks_it` 与切口；卡池一侧跑五条钉住的用例（入口、借据与借卡入口、借之前那一步、卡池模块遮蔽、卡交出自己的设备）与切口。`runs/k3-summary.txt` 原样（每个字母一条，`g` 绿、`R` 红或 panic；崩溃放量两列是主用例、切口，卡池六列是五条用例、切口）：

```
c00-legal-frozen gg
c01-red-plain-new-layer Rg
c02-red-S14-real-tests-wrapped-hiding-a-layer RR
c03-red-S14b-wrapped-after-an-item-ending-in-a-semicolon RR
c04-red-S19-wrapped-fake-tests-hiding-a-layer Rg
c05-red-N01-core-primitive-bool Rg
c06-red-N02-bool-in-parentheses Rg
c07-list-N03-alias-of-bool gg
c08-list-N15-type-macro gg
c09-red-S14b-in-square-brackets RR
c10-legal-char-literals gg
c11-legal-lifetimes gg
c12-legal-strings gg
c13-legal-comments gg
c14-legal-attribute-before-the-opening gg
c15-legal-macro-rules-before-the-opening gg
g00-legal-frozen gggggg
g01-red-S14-real-tests-wrapped RRRRRR
g02-red-S14b-wrapped-after-an-item-ending-in-a-semicolon RRRRRR
g03-red-S19-wrapped-fake-tests-hiding-a-bad-drop gRgggg
g04-red-G14-clean-drop-under-cfg-then-bad gRgggg
g05-red-G15-bad-drop-only gRgggg
g06-red-G16-clean-entry-under-cfg-then-bad gRgggg
g07-red-G17-bad-entry-only gRgggg
g08-red-clean-pool-under-cfg-then-real-with-mod-wgpu gggRgg
g09-red-S14b-in-square-brackets RRRRRR
g10-legal-char-literals gggggg
g11-legal-lifetimes gggggg
g12-legal-strings gggggg
g13-legal-comments gggggg
g14-legal-attribute-before-the-opening gggggg
g15-legal-macro-rules-before-the-opening gggggg
g16-legal-pool-module-under-a-feature gggRgg
g17-legal-another-module-named-card-pool gggRgg
g18-legal-pinned-words-in-comments-and-strings gggggg
g19-legal-another-fn-lease-card-outside-the-pool gRgggg
g20-legal-drop-under-a-cfg-attr gRgggg
```

- 点名的写法：S14（`c02`、`g01`）、S14b（`c03`、`g02`）红在切口「恰好一处」；S19（`c04` 藏一层收叫停的层、`g03` 藏第二份借据丢掉时那一步）红在主用例；G14、G16（`g04`、`g06`）红在「按词恰好一处」；N01、N02（`c05`、`c06`）红在按形找到的那一句。对照：G15、G17（`g05`、`g07`）照旧红在钉住的那几句；原样（`c00`、`g00`）绿；`c01` 插一层普通的新层红（名单变了，上一轮起的设计）。名单：N03、N15（`c07`、`c08`）照旧绿，写在 `crates/singlefs-checker-tier/src/crash_amplification.rs:5435`：「叫停闭包的返回类型写成别名（`type 别名 = bool;` 之后 `Fn() -> 别名`，正推 K4-1 N03）或类型宏」；三处「取第一处」各自的名单都在（两个入口那一条在 `the_entries_outside_the_card_pool_start_their_devices_as_outside_the_card_pool` 上面；借之前那一步、卡交出自己的设备两条原名单里「在方法上面加 `#[cfg]`」那一句照旧在）。合法写法判红而写明故意拒的四样（函数体里的 `mod`、挂 `#[cfg(any())]` 的 `extern crate`、头两行换写法、CRLF）在名单与用例里都有，这一轮没重造。
- 「圆括号、方括号也在第 0 层」误伤：头两行之前放字符字面量 `'('`、`')'`、`b']'`、`'\''`、`'\u{29}'`，生命周期、`r#"(("#`、`"\")["`、`b")"`、`c"("`、`br#"]]"#`，注释里的括号，`#[allow(…)]` 紧挨着头两行，`macro_rules!` 在前：两份都绿、0 panic（`c10`–`c15`、`g10`–`g15`；这些片段连同 S14b 的宏用 rustc 编过，只编不跑，草稿目录 `snip/snippets.rs`）。
- 「卡池模块恰好一处」等新判误伤（**打中，窄，文档**）：四种合法写法判红，名单里没写故意拒：
  - `g16` 卡池模块挂 `#[cfg(feature = "gpu")]`：红在「前面挂着 ["cfg"]」；
  - `g17` 测试模块之外另有一个与卡池无关、也叫 `card_pool` 的模块（`mod atk_other { pub mod card_pool {} }`）：红在「按词恰好一处」；
  - `g19` 卡池之外另一个类型上也有 `fn lease_card`：红在「按词恰好一处」；
  - `g20` 借据丢掉时那一步挂 `#[cfg_attr(feature = "trace", allow(dead_code))]`（展开不出 `cfg`）：红在「前面挂着 ["cfg_attr"]」。
  规则本身写了（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2345`：「与四个入口（按词 `fn 名字`）各恰好一处、前面不挂 `#[cfg]`、`#[cfg_attr]`」；`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2708`：「卡池模块按词恰好一处、不挂 `#[cfg]`」），但「故意拒的写法」那一张（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:3114`：「故意拒的写法（换不换得掉那几个名字，扫这一个模块的源码判不出），一种一条，各写为什么」）只列了通配、条目层宏、另起模块与 `extern crate`。`g17`、`g19` 与卡池无关也判红，是「按词全文件数」比「按卡池模块里数」宽出来的；`g16`、`g20` 判红方向是安全的（挂 cfg 可能真把它编掉）。注释与字符串里写这几个词（`g18`）不判红。四句：①分辨「名单写明故意拒」与「没写」；②扫源码看得到；③满足 K3「合法写法判红的是不是都写明了故意拒」「有没有把合法的产品写法判红」；④改法（推的、零轮）：把这四种写进故意拒那一张、各写为什么；或 `g17`、`g19` 那两种改成只在卡池所在的那个父模块里数（这一改法会让「取第一处」类绕法在父模块之外放回来，要另核）。
- 这一节不开新的绕法：方括号那一形放在第四节，是「改坏照绿」的佐证，原样下两份都红。

## 六、K4：E171 走的路径（没打中）

`runs/k4.log`（上一轮冻结副本 `~/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree/` 对这一轮）：`diff -rq` 两棵树的 `crates` 只差 `mutations.tsv` 与两块题面里的四份文件；`mount.rs` `cmp` 相同；`refusal_is_short_of_space` 函数体 `diff` 无输出（`crates/singlefs-core/src/mount.rs:33` 仍是「use crate::mounted_session::refusal_is_short_of_space;」）。`mounted_session.rs` 测试模块之外去掉文档行之后的改动只有新成员 `DeviceTableOtherThanTheOneOfTheMount { … }` 四行与要出路里那一道核（第 1299–1312 行）；两份 checker 档源文件测试模块之外逐字相同。全仓用到 `WaysOutOfTheWideProtectionRefused`、`ways_out_of_the_wide_protection` 的只有 `singlefs-core` 那一份与那份 harness 测试文件，E171 与 checker 档装置不引它们。不报宽护的形（`NotWhatStopsTheAdmission`）与挂载、发布路径一行没动；E171 本身是 checker 档 / 形式化线，这条腿不跑。

## 七、攻方自己提的改法（都只在攻方模型上量过或推的、被攻过零轮，没实现）

| 改法 | 修哪一格 | 量过还是推的 |
|---|---|---|
| H 形（报里只剩走空）出路二末句改成「意图已发成、推完之后剩下的走空放得下、不必再发这次 unlink」，第 1254、1259 行的「再发这次改动」限定为报里还剩这次改动时；补 k=4、iso=16 那一形的用例 | 第一节 H 形 3 格 | 推的（打中那 3 格是量过的：预演 `adm`、推完再发同一个 unlink 拒成 `NoFileVersionToChange`） |
| 补用例：会话已动、又交次序对调的盘表，要出路拒成 `SessionMoved` | 第四节 N1 | 量过一半：原样 64 格 `moved`、N1 下 64 格 `tableref`（`runs/k2order-*.log`）；用例本身没写 |
| 两条切口新用例各加一形「方括号宏调用里、前面先放以分号收尾的条目」 | 第四节 B2、B3 | 量过一半：这一形原样两边红、B2/B3 下两边绿（`runs/k3-summary.txt`、`runs/scanmut-B2-…`、`runs/scanmut-B3-…`）；用例本身没写 |
| `g16`、`g17`、`g19`、`g20` 四种写进「故意拒的写法」、各写为什么 | 第五节 | 推的 |
| 第 273 行删去或注明「同一个池卸载或崩了之后再挂」撞不上；第 274 行补「预演发的是报里拷下的那一份改动」 | 第三节两处措辞 | 推的（撞不上那一条量过：283 格 `moved[2.t->3.t]`） |
| 推环入口也先比次序（K1-2 另一条臂） | 不修哪一格：今天那一臂 177 格结局都对 | 拒的那一半量过（177 格 0 写），换回之后同今天是推的 |

## 没打中的形状

| 形状 | 取样范围 | 结局 |
|---|---|---|
| 新次序核下要出路的结局与照出路二真走不同（上一轮那几族形） | 1336 个卡住的格 × 21 / 11 种动作，17526 格 | 比得了的 15912 格 0 处对不上 |
| 盘表那一种拒的时候读了盘或动了会话 | `table-reversed`、`table-swapped`、`table-one` 各 1336 格 | 全 `ask_io=(0,0,0)`、`ask_untouched=true` |
| 该能预演而被拒成新成员 / 该拒而照样预演 | 同上，加 `table-stale` 1336 格 | 没有；`table-stale` 照常预演、推环那一套核拒，与真走同结局 |
| 推环入口收次序对调的盘表写错盘、改了发布或回退候选集、C 那一圈 | 7 个 K1 形 177 格；结局、按段写集合、盘面、会话、再发、再挂 | 0 处不同（写的先后不同） |
| 别的会话同一对数时代码与文档那一段对不上 | A 一形 × B 9 格（7 格撞上同一对数） | 逐句对得上 |
| 第二十四块 21 行、上一轮活下来的 `walkSkipped` 照绿 | 21 行 + 攻方那份 `walkSkipped` | 全红 |
| 第二十五块 17 行照绿 | 17 行（扫源码小程序上） | 全红 |
| 点名的写法 S14、S14b、S19、G14、G16、N01、N02 判绿 | 各一到两份输入，两份切口 | 全红 |
| 括号那一道误伤字符字面量、生命周期、字符串、注释、属性、`macro_rules!` | 6 种 × 两份 | 0 误判、0 panic |
| E171 走的路径变了 | 两棵冻结副本比对 | `mount.rs`、`refusal_is_short_of_space` 没动，测试之外的改动只在要出路里 |

## 这条腿自己的限度

- 全部数在草稿目录的副本上，不是入库装置上的数；第四、五节第二十五块的数在攻方抄出来的扫源码小程序上（机械抽取、`include_str!` 换成读 `SRC`），不是 checker 档测试的结局。
- H 形打中只在注入级出现（与实现员那条用例同一级）；不注入 C 那一圈时（`runs/k1h-natural-c.log`，left 0–80）没造出 H 形放得下的格，上一轮两腿也说 F、H 推完之后那一段在自然历史里分不开。池里只有一个文件，再发被拒的成员是 `NoFileVersionToChange`；多文件时被拒成别的成员是推的。
- K1-2 只扫了 g ≤ 48（fill=6 那三形不在内）；崩溃状态只比了按屏障与 FUA 切段之后的写集合，没枚举、没调 `enumerate_layer0`、没算 `closed_form_state_count`；派发不许起崩溃枚举。
- K1-3 只造了 A 一形；「崩了之后再挂取新实例」是读码推的，造的是卸载之后再挂。
- `rerun.sh` 没有一口气整份跑过（见「复跑」）；`runs/` 里的日志是这一轮分批照原样跑出来的，少数在后来改过的原型上又重跑过一次（`k1h.log`、`k3-*.log`），改动不碰别的用例。
- 第一、二节的「真走」只比「推环、再发逐字节同一个改动一次」这一条走法与几种放开的动作；推完先做别的、发别的改动，比得不全。
- 攻方自己提的改法都被攻过零轮。

## 没做什么

- 没跑任何门禁阶段、checker 档测试、崩溃枚举、GPU 测试、E171；编、跑的只有 harness 档（拼进来的 `atk_xy_opus` 与那份 harness 测试文件）、`singlefs-core` 库单测、草稿目录里的扫源码小 crate 与 rustc 只编不跑的两个小文件。
- 没判正推、辩方的格；没读禁读清单里的文件；副本上量出的数不进 kb，要引得在入库装置上重做。
- 交回前删掉的：草稿目录 `~/.local/share/singlefs/work/m3-batch10z-xy-code-r1-opus/` 下的仓副本 `repo`、`repo-mut`，编译目录 `target`、`target-mut`，扫源码小 crate `scan/`、`scanmut/`（连同它们的 `target`）；原始输出已拷进模型目录 `runs/`，草稿目录留着日志、件表、上一轮原型的拷贝与造的输入。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-batch10z-xy-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m3-batch10z-xy-code-r1/tree --root /home/fy5090/code/singlefs` 末行：

```
  ✓ 核了 17 处引文，对上 17 处，没判 0 处（逐处列在上面）
```
