# m3-batch10z-vw-code-r1 攻方腿（opus）报告（2026-10-09）

轮名 m3-batch10z-vw-code-r1（代码轮第一轮）；攻正文第三节 K1–K5 全部，重点 K1-1、K1-2、K1-3、K3、K4。冻结副本 `~/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree/`（只读）；下文代码行号都是冻结副本的行号。量出的数都在草稿目录 `~/.local/share/singlefs/work/m3-batch10z-vw-code-r1-opus/` 的副本上，不是入库装置上的数。

## 复跑

```
bash research/prompts/m3-batch10z-vw-code-r1-opus-model/rerun.sh [<工作目录>]
```

它从冻结副本拷仓：`assemble.sh` 把那份 harness 测试文件与 `atk_vw_part*.rs` 拼成测试目标 `atk_vw_opus`（只借原文件的搭建函数，攻方用例都以 `atk_` 开头、`#[ignore]`）；跑 K1 九形在新接口上逐格重扫（连同被拒之后、要出路之前放开的用户动作）、实现员三形、两槽 C、被抛弃根 A 与 C、窄四档单元区、报里记的「还剩什么」四种改动、只比实例代号与 txg 的别的池、有树无文件那一版、要出路那一次读盘出错、造 F/H/walkSkipped 的搜；再跑十五个攻方变异（`mutants.py`，原文恰好命中一次才改，否则退 3；`run-mutant.sh` 每个从冻结副本另拷一份）、变异表第 3575/3577/3583/3584 行与 M10 对 core 库单测；K4 用 `build-scan-proto.py` 把两份 checker 档源文件测试模块里的扫源码函数与几条扫源码用例原样抄进一个独立小 crate、`gen-scan-inputs.py` 造攻法输入。每件经 `scripts/hosts/run-with-memory-cap.sh`（每件 3G、编译 12G、扫源码 crate 4G、扫一次 2G），线程经 `scripts/hosts/capped.sh 15`。不起 checker 档测试、不枚举崩溃状态。整份约 30 分钟（推的：各件这一轮实测相加）。这一轮的原始输出在模型目录 `runs/`。

模型目录文件的 sha256（`sha256sum` 原样）：

```
ea51c321966f42df1ac22224256e20aeed76225802e1e20669d69f4eda265bfb  apply-row.py
c8ff3b653703a576f8c78b65a84213e6c2c6025e1837d9f9cb672a8c441a3214  assemble.sh
663e2d934f452be4d65d928e692193be1c494acfc05f187308f2af8b35e681fe  atk_vw_part1.rs
f4550ec724d8b7cdb56e5a07a4fe3c021a18ef344b8cb58838ed3be26f870de7  atk_vw_part2.rs
fdab52015d92417dad597e774fd46325a15f61c97ca03dc87a8525b135e16dc7  atk_vw_part3.rs
d4dac9c68bde0a0909b1d110689b0f8d39487c26e07f19af151ecadee5e8eeb2  atk_vw_part4.rs
3e7541742178d1f07b00d0d0c39f791c179f0227b4a8c387919b18656a0ba69f  atk_vw_part5.rs
2c4b1900c8034fde09ee29a2c56a7d66cbe590c7ac6e3e971837c7449aa01b0a  atk_vw_part6.rs
ca4a447d15d43efdcc1a1c33b77a63964f6c7bfcbe181884c208029488ca058b  atk_vw_part7.rs
38d9390151c383605c47ec6a47b1024e70c240a1a686bbcc4c3b270d479cbfb0  atk_vw_part8.rs
6af8ecb8edaac441f22f1d708e83810c1b08f3b277e0b5359e8cac940e0d16e6  batch.sh
9839a5fe949a9396d9cc01fd36b2a3d5db91c8b2235ea94a512b84a5f402f42c  build-scan-proto.py
82b64e9fc5e5cdf620b0aa548bacc146150dfa475cdcb1825364a282caf150c4  gen-scan-inputs.py
253cad837e1fe58e6444040f7e64b90622ca50b796a90648b25b3ffdeb6280ed  mutants.py
7c9d057cb4156c5b39250c942089fac588c0fbe402e22ca9805785de8132d529  rerun.sh
8dc79262b3fce2d1f87cb8a16f15ab6c8850856a48484ab324747e390e897fc0  run-mutant.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| K1-1 延后的预演结局 | 没打中 | 新接口 `ways_out_of_the_wide_protection` 的预演结局与照出路二真推环再发同一个改动逐格相同：K1 九形 283 格、实现员三形 33、两槽 C 245、被抛弃根 A 与 C 209、窄四档 566，加被拒之后放开的 21 种用户动作，`agree` 全中、`mismatch=0`（`moved`、`table-reversed` 另见 K1-3） | 第一节 |
| K1-2 报里记的「还剩什么」 | 没打中 | 覆盖写四种长度、顺序写五种、建 inode 六种、unlink 各在四种卡住的池上发：报里记的 `remainder_of_the_change` 逐字是被拒那一刻真剩下的那一份（内容、写入时刻、inode、个数都对），`remainder_ok` 全中；M3/M5/M6/M7 改坏记的内容被 kinds 用例抓到 | 第二节 |
| K1-3 只比实例代号与 txg | **打中（窄）** | `ways_out_of_the_wide_protection` 第一道只比 `(instance, checkpoint_txg)`（`mounted_session.rs:1255`），这一对不是会话 / 池的唯一身份：别的池同一个实例、推到同一个 txg，拿这个池的「被宽护卡住」报去要出路，不拒成 `SessionMoved`（反而照本池状态预演、出路文字混着外来报的槽号）；文档 `mounted_session.rs:267` 写它也拦「这一份报不是这个会话的」，代码没拦 | 第三节 |
| K1-3b 盘表只按推环那一套核 | **打中（窄）** | 要出路交一份身份对调（次序不同）的盘表：`ways_out` 照推环入口那一套核（次序无关）预演，交回 `adm`／`still`、列推环；真发同一个改动经 `publish_user_change`（`mounted_session.rs:1014` 的 `is_the_device_table_of_the_mount`，次序敏感）却拒成 `DeviceTableOtherThanTheOneOfTheMount`。283 格全 `mismatch` | 第三节 |
| K1-4 报错那一步不碰盘 | 没打中 | 报「被宽护卡住」那一次 `(探物理块大小, 读, 改盘调用) == (0, 0, 0)`、盘面分配器现行版不动；要出路那一次只读盘（`ask_io` 读数 > 0、改盘调用 0、`untouched=true`）；要出路那一次每次读各注入一次读错（4034 种）结局与不带故障逐字相同，从此读不出那一段照出路二真走列不列推环 0 处分叉 | 第一节、第四节 |
| K2 / K5 对齐判共用 | 没打中 | 只进内存的盘 `check_request` 调 `block_device::check_aligned_and_in_range`、设备大小取底下那块盘的；把设备大小改成 `u64::MAX`（M10）那份 core 库单测红在越界与差分对拍两条 | 第五节 |
| K3 判别力 | **打中（窄）** | 新用例集拦住 B、D、F、H、K 与 M 系列（15 个变异 13 个红）；但 `walkSkipped`（预演里 `WalkTheQueue` 那一支恒放得下）照绿，这一轮造出了把它分开的形：unlink 卡在走空里、再把没挡的槽压到只剩 C 那一圈隔离的 4 个（推环做得成、推完之后的走空仍被空间拒），`walkSkipped` 下 `ways_out` 从 `still` 翻成 `adm`、多列一条推环，无用例覆盖 | 第六节 |
| K4 扫源码 | 没打中 | 上一轮 K4-2 那个「块注释里写测试模块头两行」的口子已堵（新切法按词 / 按行、花括号第 0 层、注释与字面量之外找头两行）：块注释、行注释、字符串里写头两行再插一层或 `mod std`，扫都红；`use wgpu;`、块里无钉住签名的 `use`、`impl … + use<'a>`、属性 / 常量初值里的表达式宏、`r#mod` 都放；上一轮两个误判（签名行尾注释带 `fn`、`use wgpu;`／块作用域 `use`）都改对 | 第七节 |
| K5 E171 走的路径 | 没打中（E171 本身没跑） | `mount.rs` 没动、`refusal_is_short_of_space` 函数体逐字未改（`diff` 无输出）；新增的只是结构、方法与测试，报「被宽护卡住」那一报的内容变了（延后预演），不报宽护的形（`NotWhatStopsTheAdmission`）发布路径不变。E171 是 checker 档 / 形式化线，不归这条腿跑 | 第五节、第八节 |

## 一、K1-1：新接口上的预演结局与照出路走的真结局

### 被判的那几句

- 延后：`crates/singlefs-core/src/mounted_session.rs:752` 那一段「报的时候不预演（用户 2026-10-09 弹窗定「改：预演延到要出路时才做」）：只记下被拒在哪一版上（`current`）与这次改动还剩什么」。
- 要出路才预演：`crates/singlefs-core/src/mounted_session.rs:1249` 的 `ways_out_of_the_wide_protection`，第 1280 行调 `rehearse_the_push_then_the_remainder_of_the_change`，交回 `WaysOutOfTheWideProtection { pushing_the_root_ring, ways_out }`（第 256 行）。
- 「与照出路二真推同一个结局」：`crates/singlefs-core/src/mounted_session.rs:1243`「交进来的盘表照推环入口那一套核……与照出路二真推时同一个结局」。

### 造法（`atk_vw_part1.rs`–`atk_vw_part4.rs`，原型里自己造的流）

- 每一格先发那次写，报「被宽护卡住」的，拷下那一刻的盘面与会话，调 `ways_out_of_the_wide_protection`（在一份数调用的拷贝上，`atk_ask`），再另起一份照出路二走：调推环入口、再发逐字节同一个改动一次（`atk_real_follow`），逐格比四样（`atk_compare`）：预演 `adm` ⇔ 推环做成且再发放得下；`still` ⇔ 推环做成且再发被拒（另比两份拒绝的 `Debug`）；`pushref` ⇔ 推环被拒（另比 `Debug`）；预演放出来的隔离槽数 ⇔ 真推前后每块盘隔离数之差；出路条数与结局对得上。
- 形：K1 九形（前缀覆盖写 2、4、8 × 文件长到 6、11、16、21 个单元，崩在 R 第一条记录之后），g 从 C 臂第一个被拒的长度扫到对照臂第一个被拒的长度加 8；实现员三形；两槽 C（挂上之后先覆盖写 0、1、3、6、10 次）；被抛弃根 A 与 C 同在（覆盖写 0、1、2、4、6 次再录 R）；单元区放窄到 352、320、288、256 槽再扫九形。
- 被拒之后、要出路之前放开扫的用户动作（`ATK_ACTIONS`，21 种，只固定前缀与故障）：不做（`none`）、原样再发一次（`same`）、再长 5 个单元（`bigger`）、建 inode、覆盖写、换盘表推环（`push-swapped`／`push-reversed`）、卸载重挂、一次改盘调用报块设备错冻结（`frozen-overwrite`）、推环真发第 1／150 个改盘调用报错（`push-fail-1`／`push-fail-150`）、先要一次出路再原样发再要一次（`ask-then-same`）、要出路时交对调／倒序／只一块／旧盘面的盘表（`table-swapped`／`table-reversed`／`table-one`／`table-stale`）、先建 inode／覆盖写 1 或 3 或 6 次再原样发（`inode-then-same-k`／`overwrite-then-same-k`）。

### 原样输出（主形 pre=4、fill=16）

`runs/k1-pre4-fill16.log` 三格（act=none）：

```
ATK k1 pre=4 fill=16 g=13 n_pub=19 act=none: ask=adm ask_io=(2,666,0) ask_untouched=true ways=2 push=pushed(19) retry=admitted agree=yes n_reh=[51, 51] n_real=[51, 51] n_same=true dbg_same=- ways_ok=true
ATK k1 pre=4 fill=16 g=20 n_pub=19 act=none: ask=still[NoSpaceAfterRaisingTheFloor] ask_io=(2,868,0) ask_untouched=true ways=1 push=pushed(19) retry=notwhat(budget) agree=yes n_reh=[51, 51] n_real=[51, 51] n_same=true dbg_same=true ways_ok=true
```

各族各动作的汇总（`runs/` 下 `grep -h 'ATK k1 totals\|k1impl totals\|k1two totals\|k1ab totals'` 之后按动作合计，`agree` = 逐格四样全对、`mismatch` = 有对不上、`moved` = 要出路被拒成 `SessionMoved`／`moved`）：

```
主形九形 283 格（act=none/same/bigger/ask-then-same/push-swapped/push-fail-1/frozen-overwrite）：agree=283 mismatch=0 moved=0
主形九形 283 格 act=inode / overwrite / push-reversed / unmount-remount / push-fail-150：moved=283（要出路时会话已不在被拒那一版上，拒成 SessionMoved，io=(0,0,0)）
主形九形 283 格 act=table-reversed：mismatch=283（见第三节 K1-3b）
主形九形 283 格 act=table-swapped / table-one / table-stale：agree=283（推环入口那一套核拒掉、出路只列卸载，与真推同结局）
实现员三形 33、两槽 C 245、被抛弃根 A 与 C 209、窄四档 352/320/288/256（223/164/123/56）：act=none/same/… agree 全中 mismatch=0；act=table-reversed mismatch 全中
```

### 读法

- 合计 283+33+245+209+(223+164+123+56)=1336 格的自然动作（none/same/…）加被拒之后不改盘表的放开动作，照出路二真走 0 处 `mismatch`：延后之后要出路交回的预演结局与照出路二真推环再发同一个改动逐格相同（列不列推环、放出来的槽数、`ChangeStillRefused` 里的拒绝 `Debug` 都对）。机理（读码）：`ways_out` 里的 `rehearse_the_push_then_the_remainder_of_the_change` 走的就是真调推环入口（`push_the_root_ring_on`）与会话发布那一段本身，只把盘换成只进内存的那一层、分配器换成拷贝；会话单线程，报与要出路之间没有别的东西动分配器与盘。
- `moved`（inode/overwrite/… 283 格）不是打中：用户在被拒之后做了别的发布（txg 加一）或推过环或卸载重挂，会话已不在被拒那一版上，要出路照定义拒成 `SessionMoved`、`(探, 读, 改盘)=(0,0,0)`、盘不动——这是实现员自定、用户已定「报出来不算打中」的那一条。`push-fail-150` 推环真发到半路报错、`current` 落在后面的一版，同样 `moved`。
- `ask-then-same`：先要一次出路、原样再发、拿第二份报再要出路，两份报的预演结局 `asks_same=true`（第二报输入与第一报逐字相同，省重算只是代价、不是正确性）。

### 四句（没打中，照写限度）

1. 分不分辨臂：这一格比的是同一个实现的预演与它自己的真路（延后到要出路时做），没有对面的臂；它分辨得出的是「预演照推环入口与会话发布本体走」与「另写一份预演」——后者实现员没选、没量。
2. 系统当时看不看得到：看得到，预演读的就是要出路那一刻会话的盘与分配器拷贝。
3. 满足的分句：K1-1「交回的预演结局……和照出路二真推环再发同一个改动的真结局是否相同」两个方向都 0 次对不上（不改盘表、同一会话）。
4. 改法：不需要（延后这一改没在这几格上改变结局，上一轮 tu 同形同数）。

推翻条件：在入库装置上照上面造法另找一形，不改盘表、同一会话，而 `ways_out` 的预演结局与照出路二真推环再发同一个改动有一格对不上。

## 二、K1-2：报里记的「还剩什么」是不是被拒那一刻真剩下的那一份

### 被判的那几句

- `crates/singlefs-core/src/mounted_session.rs:230`「被拒的那一次之后这次用户改动还剩什么要发（内容拷一份）：要出路时推完环照它再预演一次。」
- `crates/singlefs-core/src/mounted_session.rs:788`「remainder_of_the_change: RemainderOfTheUserChangeKeptByTheRefusal::kept_from(remainder),」——报这一形那一刻把 `remainder` 拷一份进报里。

### 造法与原样输出（`atk_vw_part5.rs`，`runs/kinds.log`）

- 四种卡住的池（K1 主形、实现员三形）各发：覆盖写 1／2999／半单元／整单元字节，顺序写 +1／+5／+13／+20／+40 单元，建 1／2／8／32／128／512 个 inode，unlink 第一个文件；报卡住的比报里记的 `remainder_of_the_change`（`{:?}`）与期望的逐字，再要出路、照出路二真走。

```
ATK k1kinds totals {"first=admitted": 16, "stuck walk_only=false remainder_ok=true follow_ok=true": 48}
```

16 格那一发没卡住（admitted，不进比对）；48 格卡住的 `remainder_ok=true follow_ok=true` 全中。unlink 在这四种池上都没卡（意图发成、admitted，`walk_only` 那一支的报没造在这里，另见第六节 H 那一形）。

### 读法与四句

- 覆盖写记成 `Overwrite { content, write_time_seconds }`、顺序写记成 `SequentialWrite{…}`、建 inode 记成 `NewInodes { count, write_time_seconds }`，逐字都对。M3（覆盖写记的时刻加一）、M5（inode 个数加一）、M6（顺序写时刻加一）、M7（覆盖写内容拷成空）四个变异下 kinds 用例红（`runs/mut-M3/5/6/7-*-3-*.log` 的 `remainder_ok=false`），原样下绿——报里记的确是被拒那一刻的那一份。
- 四句（没打中）：①不分辨臂（同一实现的报与它的真路）；②看得到（内容拷进报里）；③满足 K1-2「记进报里的是不是被拒那一刻真剩下的那一份」——是；④不需要改法。推翻条件：造一形报里记的 `remainder` 与被拒那一刻真剩下的对不上。

## 三、K1-3：只比实例代号与 txg（打中，窄）与盘表只按推环那一套核（打中，窄）

### 被判的那几句

- `crates/singlefs-core/src/mounted_session.rs:1255`「version_of_trees.root.instance == stuck.instance_of_the_version_refused_on」、第 1257 行「== stuck.checkpoint_txg_of_the_version_refused_on」：要出路第一道只比这一对。
- 文档 `crates/singlefs-core/src/mounted_session.rs:267`「会话的现行那一版已不是报「被宽护卡住」时被拒的那一次要接在后面的那一版（之后又发过：推过环、别的改动发成、走空走了几步；或这一份报不是这个会话的）……第一版不支持。」——括注里把「这一份报不是这个会话的」也归进 `SessionMoved`。
- 实现员自定第 3 条：要出路不先核 `is_the_device_table_of_the_mount`，盘表照推环入口那一套核。

### 打中一：别的池同一对数（`atk_vw_part5.rs`，`runs/other.log`）

```
ATK k1other pre=4 fill=16 g=13 own=adm other=same-prefix-without-r reached=true same_pair=true pair=2.16 ask=pushref[NothingHeldUntilThisProcessWritesARingSlot] ask_io=(2,16,0) ask_untouched=true ways=1 same_text_as_own=false real_push_on_other=push_refused[NothingHeldUntilThisProcessWritesARingSlot]
ATK k1other   way_one_on_other=出路一：正常卸载（mount::unmount）之后再可写挂载一次。R 那一槽（根环槽 2.3）没有读得出的有效根：再挂时择的是卸载推出来的最后一版，R 那几条记录不在它的水位之上，C 那一圈照它重算、不再有这一槽，宽护不再出现。
```

- 第一个池（K1 主形）被拒在 `(instance 2, txg 16)`，拿它的 `StuckBehindTheWideProtection` 报去另一个池 `same-prefix-without-r`（同一份 mkfs、同实例、覆盖写推到同一个 txg 16，但没录 R，没有 C 那一圈）要出路：`same_pair=true`——第二个池的现行版 `(2,16)` 与报里记的那一对相同，`ways_out_of_the_wide_protection` 没拒成 `SessionMoved`，照第二个池的状态预演。这里第二个池没有 C 那一圈，预演里推环拒成 `NothingHeld`、出路只列卸载再挂，恰好与在第二个池上真推一致（`real_push_on_other` 同）——但出路一的文字（`way_one_on_other`）用的是**外来报**里 R 那一槽（根环槽 2.3）与 txg，说它「判成被抛弃」，而第二个池根本没有这一条被抛弃的 R，这句话对第二个池不成立（`same_text_as_own=false` 另记它与本池的出路文字也不同）。
- 机理（读码）：`(instance, checkpoint_txg)` 是 `TransactionOutput::root` 的两个数，同一个会话里 txg 每发一次加一、不会重；但跨池 / 跨会话这一对可以撞上（实测撞上了）。代码第一道只比这一对，不比会话 / 池身份；报里的 `ring_slot_of_r`、`empty_publishes_until…`、`isolated_slots_on_each_device` 全是外来报的，而预演用本池的分配器，两边拼在一起的出路可能内部不一致。

### 四句（打中一）

1. 分不分辨臂：分辨「用 `(instance, txg)` 判报 identity」与「另记一个会话 / 池的 nonce 再比」。
2. 系统当时看不看得到：**看不到**——现在的报与会话里没有会话 / 池 nonce，`(instance, txg)` 撞上时，代码在要出路那一刻分不出外来报与本池报。这是「判别子观测不到」那一类（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）：要拦得加一个字段，改判据、改完是新一轮。
3. 满足的分句：K1-3「有没有一种状态变化不动这两个数而让预演用的已不是被拒那一刻的状态」——跨池同 `(instance, txg)` 不同分配器状态，是。
4. 改法（推的、零轮）：报里与会话里各带一个这次挂载起的 nonce（或池 fsid + 实例起的随机数），要出路第一道连 nonce 一起比，不是本会话的拒成 `SessionMoved`。只在攻方模型上推过、没实现、被攻过零轮。

### 打中二：盘表身份对调（`atk_vw_part3.rs`，`runs/k1-pre4-fill16.log` act=table-reversed）

```
ATK k1 pre=4 fill=16 g=13 n_pub=19 act=table-reversed: ask=adm ask_io=(…) ask_untouched=true ways=2 push=pushed(19) retry=other(DeviceTableOtherThanTheOneOfTheMount { device_table_of_the_mount: [DeviceIdentity(0), Devi) agree=NO n_reh=[51, 51] n_real=[51, 51] n_same=true ways_ok=true !MISMATCH
```

- 要出路交一份身份次序对调的盘表（同样两块盘，次序倒过来）：`ways_out` 照推环入口那一套核（`parameters_of_the_pool_agreeing_with_the_caller_inputs`，`mounted_session.rs:1325` 那一串，身份次序无关），预演推环做成、再预演这次改动（`publish_the_change_raising_the_floor_for_space` 不再核盘表），交回 `adm`、列两条出路（含推环）；但真发同一个改动走公开入口 `publish_user_change`，第 1014 行 `is_the_device_table_of_the_mount`（`mount.rs:954` 的 `.eq(...)`，次序敏感）先拒成 `DeviceTableOtherThanTheOneOfTheMount`。主形九形 283 格、实现员三形、两槽 C、被抛弃根、窄四档全 `mismatch`。
- 对照：`table-swapped`（把两块盘的身份号互换、盘不动）、`table-one`（只交一块）、`table-stale`（交旧盘面的第二块）这三种 `ways_out` 预演里推环就拒了（`DevicesHandedIn`、次序无关的那几判打到了），出路只列卸载再挂，与真推同结局、`agree`。所以差的只是**次序**这一条：推环那一套核不看次序，`publish_user_change` 看次序。

### 四句（打中二）

1. 分不分辨臂：分辨实现员自定第 3 条「要出路不核 `is_the_device_table_of_the_mount`、只按推环那一套核」与「要出路照 `publish_user_change` 那样核盘表次序」。
2. 系统当时看不看得到：看得到——`ways_out` 手里有盘表，能比次序。
3. 满足的分句：K1-1「预演结局……和照出路二真推环再发同一个改动的真结局是否相同」——次序对调的盘表上预演 `adm`、真再发拒成 `DeviceTableOtherThanTheOneOfTheMount`，不同。
4. 改法（推的、零轮）：`ways_out` 预演之前先核 `is_the_device_table_of_the_mount`，次序不符就拒成 `SessionMoved` 一类、不预演。在打中的那 283 格上这个改法把 `adm` 翻成拒、与真发同结局（改法碰得到这些格）。只在攻方模型上量过、没实现、被攻过零轮。

- 限度：两条都要用户在要出路时交一份与被拒那一刻（报 / 盘表）不一致的东西——外来报，或身份次序不同的盘表。同一会话、同一份盘表全程一致时（act=none/same 的 1336 格）两条都不出现。是不是现实的误用，交主 agent 判；文档第 267 行与实现员自定第 3 条各自碰到这两处，所以不是纯装置造出来的。

## 四、K1-4：报错那一步不碰盘、要出路那一次读盘出错

### 被判的那几句

- `crates/singlefs-core/src/mounted_session.rs:212`「报这一形只读分配器（内存）：不拷分配器、不预演、一个盘都不读。」
- `crates/singlefs-core/src/mounted_session.rs:1241`「收 `&self`：预演只读交进来的盘、在分配器的拷贝上走，会话的分配器、现行那一版一样都不动——类型上就动不了（`PoolAllocator` 里没有内部可变），」

### 原样输出（`atk_vw_part3.rs`、`atk_vw_part8.rs`）

- 报那一次 `(探物理块大小, 改盘调用) == (0, 0)`、要出路那一次 `ask_io=(2, 读数, 0)`（两块只进内存的盘各探一次、只读、0 个改盘调用）、`ask_untouched=true`（分配器、现行版、盘面 `{:?}` 全不变）——见第一节每一行的 `ask_io`、`ask_untouched`。
- 要出路那一次每次读各注入一次读错（`runs/askrd-4-16-13.log` 等，4034 种）与从此读不出那一段：

```
ATK k1askrd pre=4 fill=16 g=13 transient 666 × adm same_as_without_fault=true untouched=true (first n=1)
ATK k1askrd pre=4 fill=16 g=13 pinned 666 × adm untouched=true | follow: pushed(19) retry=admitted ok=Some(true) (first n=1)
ATK k1askrd pre=8 fill=11 g=40 pinned 205 × still[FloorRaiseFailed:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread] untouched=true | follow: pushed(19) retry=other(FloorRaiseFailedWhilePushingForSpace(FloorRaise ok=Some(true) (first n=33)
ATK k1askrd pre=8 fill=11 g=40 pinned 42 × still[NoSpaceAfterRaisingTheFloor] untouched=true | follow: pushed(19) retry=notwhat(budget) ok=Some(false) (first n=660)
```

### 读法与四句（没打中）

- 一次性读错（`transient`）每一种都 `same_as_without_fault=true untouched=true`：要出路里哪一次读报错一次都被重读或另一槽接住，结局与不带故障逐字相同，会话不动。
- 从此读不出（`pinned`）：`ok=Some(false)` 的格全是 `still[…]` ⇔ 真走也 `retry=other(FloorRaiseFailed…)` 或 `notwhat(budget)`——两边都不列推环（`still` 一条出路、真走也被拒），差的只是 `refusal` 那一格的原因（空间 vs 读坏），列不列推环 0 处分叉，与上一轮 tu 第二节同形同数。`untouched=true` 全中：要出路读坏也没动会话。
- 四句：①不分辨臂（同实现预演 vs 真走）；②看得到；③满足 K1-4「报错 / 要出路这一步碰不碰盘」——报那一步不碰、要出路只读不写不动会话；④不需要改法。推翻条件：造一形要出路读坏之后会话被动了（`ask_untouched=false`），或列不列推环分叉。

## 五、K2 / K5：对齐判共用、E171 路径

- 对齐判共用：`crates/singlefs-core/src/mounted_session.rs:346` 的 `check_request` 调 `crates/singlefs-core/src/block_device.rs:315` 开成 `pub(crate)` 的 `check_aligned_and_in_range`，设备大小取底下那块盘的 `size_in_bytes()`。变异 M10（把设备大小传成 `u64::MAX`）在 core 库单测红在 `a_request_past_the_end_of_the_disk_below_is_refused_and_one_ending_at_the_end_is_kept` 与差分对拍两条（`runs/mut-M10-core-lib.log`：`FAILED. 5 passed; 2 failed`）——越界判确实用底下那块盘的大小。变异表第 3575/3577/3583/3584 行（B 拼接、keepOffByOne、K5 只进内存的盘不对齐／越过盘尾）在副本上各自点名的用例红（`runs/row-3575/3577/3583/3584.log` 都 `FAILED`，点名的那一条在失败里）。
- K5 E171：`crates/singlefs-core/src/mount.rs` 没动（`diff` 两棵树 crates 只差这两块题面里的文件加 `mutations.tsv`）；`refusal_is_short_of_space`（`mounted_session.rs:863`）函数体逐字未改（`diff <(sed -n '/^pub fn refusal_is_short_of_space/,/^}/p' 旧) <(新)` 无输出）。这两块的新代码只在结构、方法与 `#[cfg(test)]` 里，报「被宽护卡住」那一报的字段变了（延后预演：删 `pushing_the_root_ring`、加三样私有字段），不报宽护的形（`NotWhatStopsTheAdmission`）发布路径不变。E171（三 oracle 合谋测试）是 checker 档 / 形式化线，按派发这条腿不跑；没发现测试之外、第二十二块题面之外的改动。

## 六、K3：判别力（打中，窄：walkSkipped 照绿，这一轮造出了分开它的形）

### 十五个攻方变异（`mutants.py`，`run-mutant.sh` 每个从冻结副本另拷一份）

每个在一份新拷的副本上编，整份跑那份 harness 测试文件（`atk_vw_opus` 之外的 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table`，58 条），红了几条见下（`runs/mut-<名>-file.log` 的结果行）：

| 变异 | 改了什么 | 那份测试文件 |
|---|---|---|
| F-walk-before-keeps-only-the-walk | 走空之前被拒时报里只记走空 | FAILED 1（拦住） |
| H-after-the-intent-keeps-the-change | unlink 意图之后走空被拒时报里记成走空再发这次改动 | FAILED 1（拦住） |
| M2-version-check-only-instance | 要出路版本判只比实例代号、不比 txg | FAILED 1（拦住） |
| M9-instance-kept-off-by-one | 报里记的被拒那一版实例代号加一 | FAILED 17（拦住） |
| walkSkipped | 预演里 `WalkTheQueue` 那一支恒放得下 | **ok. 58 passed（照绿）** |
| M1/M3/M4/M5/M6/M7/M8/M10 | 见 `mutants.py`（版本判只比 txg、报里记的内容 / 时刻 / inode / 个数错一位、有树无文件判成 moved、对齐判设备大小不设限） | 各自对应用例红（M1/M8 由 M2/M9 同族用例或 kinds 用例覆盖；M10 见第五节） |

原样（`runs/mut-walkSkipped-*-file.log`、`runs/mut-M9-*-file.log` 等的结果行）：

```
walkSkipped | test result: ok. 58 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
F-walk-before-keeps-only-the-walk | test result: FAILED. 57 passed; 1 failed; 1 ignored
H-after-the-intent-keeps-the-change | test result: FAILED. 57 passed; 1 failed; 1 ignored
M2-version-check-only-instance | test result: FAILED. 57 passed; 1 failed; 1 ignored
M9-instance-kept-off-by-one | test result: FAILED. 41 passed; 17 failed; 1 ignored
```

### walkSkipped 照绿，这一轮造出了分开它的形（`atk_vw_part6.rs`，`runs/k3walk-restick.log`、`runs/mut-walkSkipped-1-…`、`runs/k3walk-H-baseline-keep4.log`）

- 实现员报告「停下交主 agent 的」第 1 条说 `walkSkipped` 没有用例拦住、「推完之后那一段」造不出。这一轮造出来了：unlink 卡在走空里（意图发成、之后走空被宽护卡住，`remainder` 记成 `WalkTheQueue`），再把每块盘没挡的槽压到只剩 C 那一圈隔离的 4 个（`ATK_KEEP_ISO=4`，注入级：其余没挡的槽标成已分配，隔离位不动）——推环做得成（放得出 C 那一圈的隔离），但推完之后那一段走空仍被空间拒。
- 原样对照（同一格 ow=10 leave=27 keep-iso=4）：

```
# 原样（runs/k3walk-H-baseline-keep4.log）
ATK k3walk ow=10 leave=27 unlink=stuck-H n_pub=1 remainder=remainder_of_the_change: WalkTheQueue } ask=still[NoSpaceAfterRaisingTheFloor] ask_io=(2,141,0) ask_untouched=true | real: resend_unlink=pushed(1)->notwhat(ceiling)
# walkSkipped（runs/mut-walkSkipped-1-atk_k3_walk_stuck_with_a_push_that_goes_through.log）
ATK k3walk ow=10 leave=27 unlink=stuck-H n_pub=1 remainder=remainder_of_the_change: WalkTheQueue } ask=adm ask_io=(2,91,0) ask_untouched=true | real: resend_unlink=pushed(1)->notwhat(ceiling)
```

- 原样下 `ways_out` 预演是 `still`（推环做成、推完之后走空照旧被拒，不列推环）、真走 `resend_unlink=pushed(1)->notwhat(ceiling)`（推环 OK、再发被拒）——一致。`walkSkipped` 下预演翻成 `adm`（推完之后走空被当成放得下，列两条出路、含推环）、真走照旧被拒——预演说放得下、真发不放得下。攻方原型 `atk_compare` 在这一格会标 `NO`（预演 `adm`、真 refused）。那份 harness 测试文件照绿是因为它里头拦 H 的用例（`an_unlink_whose_walk_after_the_intent_is_stuck_behind_the_wide_protection_keeps_only_the_walk…`）把没挡的槽压到推环**取不到落点**（`PushRefused`），走不到 `walkSkipped` 改的那一支；没有一条用例让推环做得成、而推完之后的走空被拒。

### 四句（打中，窄）

1. 分不分辨臂：分辨「这一批的用例集」与「再加一条：unlink 卡在走空里、推环做得成、比预演 `ChangeStillRefused` 与推完之后真再发同一个 unlink 的拒绝」——前者 `walkSkipped` 照绿，后者红。
2. 系统当时看不看得到：看得到；预演结局与真走结局在同一进程里都拿得到。
3. 满足的分句：K3「能不能造出一形把 F、H 推完之后那一段与 `walkSkipped` 分开」——造出来了（推环只要推完 C 那一圈、同时走空被拒）。
4. 改法（推的、零轮）：加一条注入级用例：`pool_whose_unlink…is_stuck_in_the_walk_after_the_intent` 之后把没挡的槽压到只剩 C 那一圈隔离的几个（推环做得成），要出路交回 `ChangeStillRefusedAfterThePush`（不是 `ChangeAdmitted`）、只列卸载再挂、预演的拒绝与推完之后真再发同一个 unlink 的拒绝 `Debug` 逐字相同。这个改法在 `walkSkipped` 照绿的那几格上红（量过：原样 `still`、`walkSkipped` `adm`）。只在攻方模型上量过、被攻过零轮。

| 改法 | 修哪一格 | 量过还是推的 |
|---|---|---|
| 注入级「unlink 卡在走空里、推环做得成、比预演 ChangeStillRefused 与推完真再发的拒绝」 | walkSkipped | 量过（`runs/k3walk-H-baseline-keep4.log` 原样 still、`runs/mut-walkSkipped-1-…` adm） |

## 七、K4：第二十三块扫源码（没打中）

不跑 checker 档测试。量法：`build-scan-proto.py` 把两份源文件测试模块里的扫源码函数（`code_before_the_tests_module`、`rust_tokens_*`、`ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call`、`text_outside_the_tests_module`、`lines_read_past_comments_and_literals`、`ways_a_layer_handed_the_stop_request_could_drop_it` 等）与四条卡池用例、一条收叫停用例原样抄进一个独立小 crate，用例 `#[test] fn 名() { let source = include_str!(…);` 改成收 `source` 的函数、别的一字不改；`gen-scan-inputs.py` 造攻法输入，每份对每条用例跑一次（`catch_unwind`），`green` = 那条用例过、`RED` = 红、`panicked` = 当场 panic。副本里对改过的源文件没做 `cargo check`（上一轮在 `librocksdb-sys` 构建脚本上编不过，与改动无关），能不能编过是读 Rust 语法推的。

### 上一轮 K4-2 那个口子已堵（`runs/k4-stop.log`、`runs/k4-gpu.log`）

- 收叫停那一扫（`crash_amplification.rs` 的 `text_outside_the_tests_module`，按行、花括号第 0 层、注释与字面量换空格之后找头两行）：块注释里写测试模块头两行再插一层（`c2`）、行注释里写（`c3`）、跨行字符串里写（`c4`），各插一层收叫停、不往下交的新层——三种扫都 `RED`（抓到新层），上一轮 tu 第六节那个「块注释藏一层、扫出 0 处」的口子堵上了。
- 卡池那一扫（`gpu_unit_checks.rs` 的 `code_before_the_tests_module`，按词、花括号第 0 层、注释与字面量之外找头两行）：卡池里写块注释的头两行再真起一个 `mod std`（`g11`）——`RED`（切到真测试模块、`mod std` 露出来、抓到）。
- 收叫停那种闭包的新形（`stop_request_shapes_outside_the_pinned_type`）：新层收叫停写成 `Send`（`c-shape-send`）、`Box<dyn …>`（`c-shape-boxed`）、`&mut dyn FnMut`（`c-shape-mut`）、`fn() -> bool` 指针（`c-shape-fnptr`）、`impl Fn`（`c-shape-impltrait`）、泛型约束（`c11`）——各自丢掉叫停，都 `RED`（按钉住类型找不到、按形找到）。

### 新放行的几样放对了、上一轮两个误判改对了

- `g2` `use wgpu;`（引的就是外部 crate 自己）、`g3` 块里无钉住签名的 `use core as std;`、`g4` `impl … + use<'a>`、`g5` 属性里 `#[doc = include_str!(…)]`、`g6` 常量初值 `const _: () = assert!(true);`、`g7` `fn r#mod()`——各 `green`（放）。阳性对照：`g12` 模块这一层 `use core as std;`、`g13` 块里真含钉住签名加 `use core as std;`、`g9` 挂 `#[cfg]` 的 `use` 引进按名字调的函数、`g10` 条目层真宏 `thread_local!`——各 `RED`（拒）。
- 上一轮 tu 两个误判：既有钉住层 `check_blocks_where_until` 签名加行尾注释带 `fn`（`c13`）、加块注释里写 `Fn() -> bool`（`c14`）——这一轮都 `green`（注释先换空格，层名不再取成注释里的字）；`use wgpu;`（`g2`）也从上一轮的误判红改成 `green`。

### K4-1 的限度

- 新层一旦收叫停、不往下交，`every_layer…` 那一扫都会把名单变了报出来（`c1`、`c8`、`c9`、`c12` 都 `RED`）——这是设计（任何新层都要回来看），不是误判。文档里列为「拦不住」的那几样（类型别名 `type AtkStop = dyn Fn()…;` 收叫停，`c10`）照旧拦不住（`RED` 是因为别名展开后 shape 文本仍在，这一形恰好被 shape 扫碰到；真要绕得让别名里也不出现 `Fn()->bool` 字样，没造）。没找到一种新的、能遮住钉住路径或绕过叫停而判绿的写法。

四句（没打中）：①分辨「上一轮 tu 的切法 / 分词」与「这一轮的切法 / 分词」；②看得到（源文件全在手里）；③满足 K4「有没有一种写法真能遮住钉住路径或绕过叫停而判绿」——没有；④不需要改法。推翻条件：造一份源文件插一层收叫停、不往下交的真层（或卡池里真遮住 `std`／`wgpu`／按名字调的函数），而扫出 0 处（`green`）。

## 八、没打中的形状

| 形状 | 取样范围 | 结局 |
|---|---|---|
| K1-1 预演结局与照出路二真推再发同一个改动不同（不改盘表、同一会话） | 九形 283 + 实现员三形 33 + 两槽 C 245 + A 与 C 209 + 窄四档 566，× 不改盘表的放开动作 | 0 处 mismatch |
| K1-1 预演放出来的槽数与真推之差不同 | 同上，逐盘比 | 0 处 |
| K1-2 报里记的「还剩什么」不是被拒那一刻真剩下的 | 四种池 × 覆盖写 / 顺序写 / inode / unlink | 0 处（48 格卡住全对） |
| K1-4 报那一步碰盘、要出路动会话 | 283 格报的 (探,读,改盘)、要出路的 untouched；要出路每次读注入读错 4034 种 | 报 (0,0,0)；要出路只读不写不动会话 |
| K1-4 从此读不出时要出路与真走列不列推环分叉 | 四格 × 每次读 | 0 处分叉（refusal 原因不同、两边都拒） |
| K2 两份对齐判不同 | M10 设备大小改成 u64::MAX | 越界判用底下那块盘的大小，红 |
| K4 一种遮住钉住路径 / 绕过叫停而判绿的写法 | 收叫停 13 种输入、卡池 14 种输入 | 没找到；上一轮 K4-2 口子与两个误判都改对 |
| K5 mount.rs / refusal_is_short_of_space 变了 | diff 两棵树 | 没动 |

## 九、这条腿自己的限度

- 全部数在草稿目录的副本上，不是入库装置上的数；K4 的数在攻方抄出来的扫源码函数上（副本里没对改过的源文件 `cargo check`，能编过是读语法推的），不是 checker 档测试的结局。
- K1-1 比的是「推环、再发逐字节同一个改动一次」这一条走法；推完先做别的、或推完发另一个改动，那一报没有说过，没比。根环每区槽数只有 mkfs 写的 S = 8 一档（3S = 24）；卡住的形造在单元区 384 槽及更窄的四档上。
- K1-3 两条打中都要用户在要出路时交一份与被拒那一刻不一致的东西（外来报、或身份次序不同的盘表）；是不是现实的误用交主 agent 判，这条腿只给出「代码分不出 / 核不到次序」这个事实。打中一是「判别子观测不到」（要加字段），改法是新一轮。
- K3 的 walkSkipped 分开形是注入级（`mark_allocated` 占住没挡的槽、`atk_keep_only_isolated_by_c` 只留 C 那一圈隔离的几个），不是自然历史里走到的；F、H 推完之后那一段仍没在自然历史里走到（推环要推完 C 那一圈才到走空，而自然历史里卡在走空时推环多半取不到落点）。
- K1 那几形的盘面是照上一轮的造法从录下的写流里截一个前缀得来的（崩在 R 第一条记录之后，一个崩溃状态），不是层 0 的整写子集枚举：派发这一轮不许起崩溃枚举，所以没调 `enumerate_layer0` / `enumerate_layer0_versions`，也没算 `closed_form_state_count`；原型里没有别的崩溃状态。
- 攻方自己提的改法（第三节两条、第六节一条）都只在攻方模型上量过或推过，被攻过零轮。

## 没做什么

- 没跑任何门禁阶段、checker 档测试、崩溃枚举、GPU 测试、E171；编、跑的只有 harness 档（拼进来的 `atk_vw_opus` 与那份 harness 测试文件、十五个变异）、`singlefs-core` 库单测（M10 与变异表第 3575/3577/3583/3584 行）、草稿目录里的扫源码小 crate。
- 没判正推、辩方的格；副本上量出的数不进 kb，要引得在入库装置上重做。
- 草稿目录里的仓副本与编译目录（`~/.local/share/singlefs/work/m3-batch10z-vw-code-r1-opus/` 下的 `repo`、`repo-mut`、`repo-row`、`chk`、`target`、`target-mut`、`target-row`、`scan-proto/target`）交回前删掉；原始输出已拷进模型目录 `runs/`。
