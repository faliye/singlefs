# m3-batch10z-tu-code-r1 攻方腿（opus）报告（2026-10-09）

轮名 m3-batch10z-tu-code-r1（代码轮第一轮）；攻正文第三节 K1–K5，重点 K1-1、K1-2、K1-3、K3、K5。冻结副本 `~/.local/share/singlefs/work/m3-batch10z-tu-code-r1/tree/`（只读）；下文代码行号都是冻结副本的行号。量出的数都在草稿目录的副本上（`~/.local/share/singlefs/work/m3-batch10z-tu-code-r1-opus/`），不是入库装置上的数。

## 复跑

```
bash research/prompts/m3-batch10z-tu-code-r1-opus-model/rerun.sh [<工作目录>]
```

它从冻结副本拷仓：一份不改，用 `assemble.sh` 把那份 harness 测试文件与 `atk_tu_part*.rs` 拼成测试目标 `atk_tu_opus`（只借原文件的搭建函数，攻方用例都以 `atk_` 开头、`#[ignore]`）；一份是量法副本，`cost-patch.py` 在副本 core 里加量预演挂钟与内存的开关和读数、加一个只供量法的预演入口、把 `block_device.rs` 的对齐判放成 `pub(crate)`、接上 `atk_overlay_tests.rs`（只进内存的盘与抄的对齐判的对拍单测），`assemble-cost.sh` 拼测试目标 `atk_tu_cost_opus`；再跑十一个攻方变异（`mutants.py`，原文恰好命中一次才改，否则退 3；`run-mutant.sh` 每个从冻结副本另拷一份）；K4 用 `build-scan-proto.sh` 把两份 checker 档源文件测试模块里的扫源码函数原样抄进一个独立小 crate，`gen-scan-inputs.py` 造输入。每件经 `scripts/run-with-memory-cap.sh`（每件 3G、并行时一批合计不超过 12G；编译 12G；大盘代价那两件 10G、12G 单独跑），线程经 `scripts/capped.sh`。不起 checker 档测试、不枚举崩溃状态。整份约 35 分钟（推的：各件这一轮实测相加）。这一轮的原始输出在模型目录 `runs/`。

模型目录文件的 sha256（`sha256sum` 原样）：

```
17b246fbc14db3abec66b92d6c8cca2b73792957ca842223818d2fcd4a64a5e9  assemble-cost.sh
8ff633b352df6339743f40467c3eca587809f02abeba5971eda1146c751dfc4e  assemble.sh
1fa6401cafaa57b6d91d7bda7ed7f0c9a4afe202dee6cc341257b63b205b75f1  build-scan-proto.sh
7e742ba73a1326ab75a29a05f2ca1d3634637e743f9bf2c8873475d8fc106583  rerun.sh
13979390665f2c74fd140a1502305e03c31696a6a48151041d37676392911ddf  run-mutant.sh
4fabc35e82e93ace668c39c2d2037dfb58ddb3f424e494ca1dd8df0311a5bb15  atk_cost_part.rs
6f498f2c2eb4514192a273de3e45be413dea1d908e1a5e8070473f131dff1b9c  atk_macro_shadowing.rs
5d7f9591c8777719e7c2b63e7008fed76308000cf708562dfa639878b7a79e4a  atk_overlay_tests.rs
024baadf0a13b15cb9d636fe5b103b78db23fa6cd98f43adb8e2c77072d61fe5  atk_tu_part1.rs
04bd51071cabc93355ae06a50287af1a946f239bc341663d76325eaec76b9fdd  atk_tu_part2.rs
bd3aaefc9284dec52d23cea469c35a4a2df807fa7ac958d0619cdd612b6d4d3b  atk_tu_part3.rs
401f8bee6d2b25f6addcc47de67075e581aaa0448574dcb7bb4788adee22f702  atk_tu_part4.rs
53bc0d4eea9bae702c79e4df7add32925f1c173276fd95f7cee8724066a63134  atk_tu_part5.rs
9a1b71f059273ef3f7384a939d8f8d65a8cbbed48de4903b78181c487c036bff  atk_tu_part6.rs
85d33c240a8c37fd729f22368c1890a5f93e6949f1d3c45768af8586d823f4f7  atk_tu_part7.rs
0372dfba3bbab2d5cce84c86e85d9694be769d9b1fd2d62f881308c65f713a2c  atk_tu_part8.rs
2a036f001dbc4e421a194df878dc488f7f41eb8a182d039f006ea4222227a6ef  cost-patch.py
928f859d4db05ba67f4ec9422d2af2fe16abab45687907d96e50f1a5d0a80a60  gen-scan-inputs.py
8190cd4dee94f3f752fbee9ed26ec208aef7a61d4364bc181772136e219e58c2  mutants.py
```

`runs/` 里 83 份日志；`k4-v2-check.log` 是 `cargo check -p singlefs-checker-tier` 在副本上编不过 `librocksdb-sys` 的那一次（与改动无关，见第六节），不当结果。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| K1-1 预演说的与真结局 | 没打中 | 3393 份「被宽护卡住」的报（K1 九形 283 格、被拒之后放开小动作 2057 次、实现员三形 33、两槽 C 245、被抛弃根 A 与 C 同在 209、更窄四档单元区 566）照出路二真走一遍（推环、再发逐字节同一个改动一次）：列不列推环 0 处对不上，预演放出来的槽数与真推前后之差 0 处对不上，推完照旧被拒那几格预演交回的拒绝与真再发的拒绝 `Debug` 逐字相同 | 第一节 |
| K1-1 不推环再发一次 | 没打中 | 283 格：第二报 0 个改盘调用、盘面与分配器与现行那一版逐字不变、预演结局与第一报相同 | 第一节 |
| K1-2 预演不落盘、读回拼接 | 没打中 | 只进内存的盘只借底下那块盘的 `&dyn`；对拍 360000 次随机读写（含不对齐、越界、贴 u64 顶）0 差，底下那块盘一字不变 | 第二节 |
| K1-2 预演里读盘出错 | 没打中 | 第二报里第 n 次读报错一次（4034 种）：结局与不带故障逐字相同；第 n 次读那一段从此读不出（4034 种）：照出路二在同一组故障上真走，列不列推环 0 处对不上；预演里推环被拒那一支在自然历史里没造出来 | 第二节 |
| K1-3 预演的代价 | **打中（窄，代价数）** | 每报一次都整串推一遍环、再预演一次这次改动：挂钟与常驻内存随盘大小线性涨，两块 2 TiB 的盘上一报 25.0 秒、常驻峰值多 2461 MiB（同一池一次覆盖写 0.687 秒）；384 槽小盘一报 32–42 ms，不预演时 6–8 ms | 第三节 |
| K2-1 出路措辞、N | 没打中 | N 在 3393 报上都等于真推前后每块盘隔离数之差；被抛弃根 A 与 C 同在那一形 N 含 A 独占的那几槽，与文档注释说的相同 | 第一节、第四节 |
| K2-2 冻结之外预演过、真发第一次被拒 | 没打中 | 读错那几形与全部报「推完放得下」的格，真推环都做成 | 第二节、第四节 |
| K2-3 C 有落点无隔离吞掉宽护 | 没攻造 | 读码没找到该报而被吞的形；没造历史 | 第四节 |
| K3 判别力 | **打中（窄）** | 十一个攻方变异，那份 harness 测试文件 55 条拦住 3 个，照绿 8 个；其中只进内存的盘读不回自己写过的块（B）、写进内存时块号错一位（D）让预演交回的拒绝换成错的原因，攻方原型逐字比拒绝拦得住；预演里推环被拒当成放得下（K）没有用例走到 | 第五节 |
| K4-1 卡池扫源码判绿的写法 | 没打中 | 新放行的 `as`、两层花括号、`self` 组、`extern "C"` 块都换不掉钉住那两段用的名字；另有两种合法写法判红（`use wgpu;`、别的函数体里块作用域的 `use core as std;`） | 第六节 |
| K4-2 崩溃放量扫源码骗得过的写法 | **打中（窄）** | 测试模块之前写一段块注释，里面是测试模块的头两行，紧跟一行写完的新层（收叫停、不问）：「测试模块之外」被截掉一段，新层不进名单，扫出 0 处；同一层不加注释扫出 1 处。另一处误判：签名里参数行尾注释带 `fn` 这个词，层名取成注释里的字 | 第六节 |
| K5 抄的那一份对齐判 | 没打中（判法逐句相同）；没有会红的用例 | 与 `block_device.rs` 那一份对拍 100225 个请求 0 差；把它改成恒放行（变异 A）那份测试文件 55 条照绿，`memory_pool.rs` 那一份抄的有对拍用例钉着、这一份没有 | 第二节、第五节 |

## 一、K1-1：预演说的与照出路走的真结局

### 被判的那几句

- 出路二只在预演说放得下时列，话里写着：`crates/singlefs-core/src/mounted_session.rs:270` 那一句以「预演过：推完之后这次改动放得下。」收尾；不列的两支在 `crates/singlefs-core/src/mounted_session.rs:282`「RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }」。
- 推完照旧被拒那一支带着预演交回的拒绝：`crates/singlefs-core/src/mounted_session.rs:235`「/// 推环做成，推完之后这次改动照旧被拒（`refusal` 是预演里交回的那一种，例如推完上限退到环里最旧的根、推满仍不够）：出路不列推环。」
- 预演怎么走：`crates/singlefs-core/src/mounted_session.rs:497`「let mut rehearsed_allocator = allocator.clone();」之后调推环入口本体，再照会话发布那一段发这次改动剩下的（`remainder`，三处调用点在第 996、1041、1067 行）。

### 造法（`atk_tu_part1.rs`–`atk_tu_part5.rs`，原型里自己造的流）

- 每一格先发那次写，报「被宽护卡住」的，拷下那一刻的盘面与会话（`copy_of_the_session`），另起一份照出路二走：调推环入口、再发**逐字节同一个**改动一次（内容与写入时刻都定死，`AtkWrite`）。逐格比四样：预演 `adm` ⇔ 推环做成且再发放得下、`still` ⇔ 推环做成且再发被拒、`pushref` ⇔ 推环被拒；预演里每块盘放出来的隔离槽数 ⇔ 真推前后每块盘隔离数之差；`still` 那一支预演交回的拒绝与真再发的拒绝 `Debug` 逐字比；出路条数与预演结局对得上（`adm` 两条、别的一条）。
- 形：上一轮 K1 那九形（前缀覆盖写 2、4、8 × 文件长到 6、11、16、21 个单元，崩在 R 第一条记录之后，两形录不成 R），g 从 C 臂第一个被拒的长度扫到对照臂第一个被拒的长度加 8；实现员三形卡住那次覆盖写本身，再加写长到 2–80 个单元（10 个长度）；两槽 C（`record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites`，前缀 1、3、8 × 挂上之后先覆盖写 0、1、3、6、10 次）与被抛弃根 A 与 C 同在（A 那份盘面，挂上之后覆盖写 0、1、2、4、6 次再录 R）各扫写长 1–90 个单元；单元区放窄到 352、320、288、256 槽再扫 K1 九形。
- 用户在被拒之后做什么放开扫：①不推环原样再发一次（录写流数改盘调用，比盘面、分配器 `Debug`、现行那一版 `Debug`）；②先建 inode 或覆盖写 2999 字节 k 次（k = 1、2、3、4、6），再发同一长度的写，那一次若报卡住就照它的出路走、逐格比。

### 原样输出

`runs/k1-main.log`（主形 pre=4、fill=16 的三格）：

```
ATK k1 pre=4 fill=16 g=19 ctrl=admitted first=stuck reh=adm ways=2 push=pushed(19) retry=admitted agree=yes n_reh=[51, 51] n_real=[51, 51] n_same=true dbg_same=- ways_ok=true again=stuck again_reh=adm again_raises=0 again_ops=0 again_unchanged=true
ATK k1 pre=4 fill=16 g=20 ctrl=admitted first=stuck reh=still ways=1 push=pushed(19) retry=notwhat(budget) agree=yes n_reh=[51, 51] n_real=[51, 51] n_same=true dbg_same=true ways_ok=true again=stuck again_reh=still again_raises=0 again_ops=0 again_unchanged=true
ATK k1 pre=4 fill=16 g=36 ctrl=refused first=stuck reh=still ways=1 push=pushed(19) retry=notwhat(budget) agree=yes n_reh=[51, 51] n_real=[51, 51] n_same=true dbg_same=true ways_ok=true again=stuck again_reh=still again_raises=0 again_ops=0 again_unchanged=true
```

数（命令与输出原样，在 `runs/` 下）：

```
$ L=k1-main.log; echo "cells=$(grep -c ' first=stuck ' $L) adm=$(grep ' first=stuck ' $L | grep -c 'reh=adm') still=$(grep ' first=stuck ' $L | grep -c 'reh=still') pushref=$(grep ' first=stuck ' $L | grep -c 'reh=pushref') agree_no=$(grep -c 'agree=NO' $L) n_false=$(grep -c 'n_same=false' $L) dbg_false=$(grep -c 'dbg_same=false' $L) again_ops_nonzero=$(grep ' first=stuck ' $L | grep -vc 'again_ops=0 ') again_changed=$(grep -c 'again_unchanged=false' $L)"
cells=283 adm=20 still=263 pushref=0 agree_no=0 n_false=0 dbg_false=0 again_ops_nonzero=0 again_changed=0
```

各件的汇总行（`grep -h totals`，原样）：

```
ATK k1 totals area=384 cells=283 stuck=283 disagree=0 small_stuck=0 small_disagree=0 rehearsal_kinds={"adm": 20, "still[NoSpaceAfterRaisingTheFloor]": 263}
ATK k1 totals area=384 cells=53 stuck=53 disagree=0 small_stuck=439 small_disagree=0 rehearsal_kinds={"still[NoSpaceAfterRaisingTheFloor]": 53}
ATK k1 totals area=384 cells=55 stuck=55 disagree=0 small_stuck=389 small_disagree=0 rehearsal_kinds={"adm": 12, "still[NoSpaceAfterRaisingTheFloor]": 43}
ATK k1 totals area=384 cells=88 stuck=88 disagree=0 small_stuck=621 small_disagree=0 rehearsal_kinds={"adm": 7, "still[NoSpaceAfterRaisingTheFloor]": 81}
ATK k1 totals area=384 cells=87 stuck=87 disagree=0 small_stuck=608 small_disagree=0 rehearsal_kinds={"adm": 1, "still[NoSpaceAfterRaisingTheFloor]": 86}
ATK k1impl totals stuck=33 disagree=0
ATK k1two totals stuck=126 disagree=0
ATK k1two totals stuck=119 disagree=0
ATK k1ab totals stuck=209 disagree=0
ATK k1 totals area=352 cells=223 stuck=223 disagree=0 small_stuck=0 small_disagree=0 rehearsal_kinds={"adm": 6, "still[NoSpaceAfterRaisingTheFloor]": 217}
ATK k1 totals area=320 cells=164 stuck=164 disagree=0 small_stuck=0 small_disagree=0 rehearsal_kinds={"still[NoSpaceAfterRaisingTheFloor]": 164}
ATK k1 totals area=288 cells=123 stuck=123 disagree=0 small_stuck=0 small_disagree=0 rehearsal_kinds={"still[NoSpaceAfterRaisingTheFloor]": 123}
ATK k1 totals area=256 cells=56 stuck=56 disagree=0 small_stuck=0 small_disagree=0 rehearsal_kinds={"still[NoSpaceAfterRaisingTheFloor]": 56}
```

（`disagree` 数的是四样里任一样对不上的格。）放开小动作那 2830 次（283 格 × 两种动作 × 五个 k）里：

```
$ for f in k1small-1.log k1small-2.log k1small-3.log k1small-4.log; do grep -h 'k1small' $f; done | awk '{print $6}' | sed 's/[0-9]*:/:/; s/(.*//' | sort | uniq -c
    393 inode:admitted
   1022 inode:stuck
    380 overwrite:admitted
   1035 overwrite:stuck
```

几形各一行（`runs/k1impl.log`、`runs/k1two-a.log`、`runs/k1ab.log`、`runs/k1small-3.log`）：

```
ATK k1impl form=RollbackSixVersionsBack { overwrites_before: 20 } own_overwrite reh=adm ways=2 push=pushed(20) retry=admitted agree=yes n_reh=[220, 220] n_real=[220, 220] n_same=true dbg_same=- ways_ok=true
ATK k1impl form=RollbackSixVersionsBack { overwrites_before: 20 } g=60 first=stuck n_pub=20 reh=still ways=1 push=pushed(20) retry=notwhat(budget) agree=yes n_reh=[220, 220] n_real=[220, 220] n_same=true dbg_same=true ways_ok=true
ATK k1two o=1 pre=1 c=(2, 52) g=88 first=stuck n_pub=21 reh=still ways=1 push=pushed(21) retry=notwhat(budget) agree=yes n_reh=[26, 26] n_real=[26, 26] n_same=true dbg_same=true ways_ok=true
ATK k1ab k=4 abandoned=1 c=(1, 18) iso_all=42 g=40 first=stuck n_pub=20 reh=adm ways=2 push=pushed(20) retry=admitted agree=yes n_reh=[102, 102] n_real=[102, 102] n_same=true dbg_same=- ways_ok=true
ATK k1small pre=4 fill=16 g=13 inode1:stuck(small_refused=0) reh=adm ways=2 push=pushed(16) retry=admitted agree=yes n_reh=[51, 51] n_real=[51, 51] n_same=true dbg_same=- ways_ok=true
```

### 读法

- 合计 283 + 2057 + 33 + 245 + 209 + 566 = 3393 份报，照出路二真走 0 处对不上；推完照旧被拒的那几格，预演交回的拒绝与真再发的拒绝逐字相同（包括推了几串抬 F、每串的根、最后一次被拒时每盘的可用与需求）。机理（读码）：预演走的就是真调推环入口与会话发布那一段本身，只把盘换成只进内存的那一层、分配器换成拷贝；会话单线程，报与用户照出路走之间没有别的东西动分配器与盘。照出路走的真路比预演多的只有 `publish_user_change` 开头那几道收盘表的核，都只读盘。
- 不推环原样再发一次：283 格都是 0 次抬 F、0 个改盘调用，盘面与会话逐字不变、第二报的预演结局与第一报相同。预演自己不改会话、也不往底下的盘写。
- 放开小动作：先建 inode 或覆盖写 k 次之后有 773 次不推环就放得下了（上一轮已量过同形），其余 2057 次报卡住、照那一报走都对得上。用户先做别的再推环、或推完先做别的，那一报没有说过什么，不算它的话。
- 上一轮打中的「推完照旧被拒」那 182 格（这一轮同形 263 格）现在不列推环，只列卸载重挂；推环在这几形上只在 20/283 格列出。
- 被抛弃根 A 与 C 同在那一形，N 里含 A 独占的那几槽（k=4 那一行 C 罩着的隔离 18 槽，N 每盘 102，真推之差同），与 `crates/singlefs-core/src/mounted_session.rs:235` 前一条成员的文档注释说的「与 `isolated_slots_on_each_device` 不是同一个数」相符。

### 四句（没打中，照写限度）

1. 分不分辨臂：这一格比的是同一个实现的预演与它自己的真路，没有对面的臂；它能分辨的是「预演照推环入口与会话发布本体走」与「另写一份预演」——后者是实现员没选的那一臂，没量。
2. 系统当时看不看得到：看得到，预演读的就是那一刻的盘与分配器拷贝。
3. 满足的分句：K1-1「预演说推完放得下、照推环走完同一个写却被拒，或预演说放不下而推完其实放得下」两个方向都 0 次。
4. 改法：不需要。

推翻条件：在入库装置上照上面造法另找一形（尤其报与照出路走之间会话外有别的东西动盘或分配器的，比如两进程同读一份空闲——那是用户已定、不算），列不列推环或 N 有一格对不上。

## 二、K1-2、K5：只进内存的盘、预演里读盘出错、抄的对齐判

### 被判的那几句

- `crates/singlefs-core/src/mounted_session.rs:289`「/// 预演用的盘：读穿到底下那块盘（这一刻盘上的字节），写与写零只记进内存、盖在之后的读上，屏障什么都不做——一个写都不发到底下那块盘。」
- 抄的对齐判：`crates/singlefs-core/src/mounted_session.rs:314`「    /// 写不交给底下那块盘，这里不判就收得下一个真盘会拒的写。」；原件 `crates/singlefs-core/src/block_device.rs:313`「/// 请求要按物理块对齐、不越过设备末尾；两个后端共用这一份判据。」
- 对照：`crates/singlefs-harness/src/memory_pool.rs:115` 那一行写着内存稀疏盘那一份抄的「由用例 `tests/sparse_devices_refuse_requests_outside_the_device_like_the_file_backend.rs` 拿文件后端逐个请求对拍钉住。」

### 写不写得穿、读回拼得对不对（量法副本里的库单测，`atk_overlay_tests.rs`）

- 读码：这一层只持 `&'disk dyn BlockDevice`（共享借用），写、写零只进 `written_blocks`，屏障交回 `Ok`；`BlockDevice` 那六个动作里能改盘的三个都要 `&mut self`，拿不到底下那块盘的 `&mut`，类型上写不穿。分配器 `PoolAllocator` 里没有 `Rc`、`RefCell`、`Cell` 一类共享的内部可变（`grep` 过 `allocator.rs`），预演改的是拷贝。
- 造的反例尝试：一块参照盘（整块字节，读写先过 `block_device.rs` 那一份判据）与「只进内存的盘盖在同样初值的另一块参照盘上」各收同一串随机请求（物理块 512、4096 × 盘长 64 块、64 块加 100 字节、7 块 × 300 个种子 × 200 步；写、写零、屏障、读，多半对齐、少数不对齐、越界、贴着 u64 顶），逐次比交回值与读出的字节（读缓冲先填 0xAA，没读到的那一截露得出来）；跑完比底下那块盘与初值。另把两份对齐判在 5 种物理块（1、512、520、4096、65536）× 5 种盘长（含 0 与 u64::MAX − 7）上各比约 4000 个请求（含边角）。原样（`runs/overlay-unit.log`）：

```
ATK k5 compared=100225 differ=0
ATK overlay compared=360000
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 216 filtered out; finished in 22.17s
```

### 预演里读盘出错（`atk_tu_part6.rs`、`atk_tu_part7.rs`）

- 一次性读错：在第一报之后的盘面上原样再发一次（第二报），整池第 n 次读报错一次，n 扫过这一次调用的每一次读（五格 736 + 736 + 938 + 742 + 882 = 4034 种）：每一种的交回值 `Debug` 与不带故障的逐字相同，会话分配器不动（`runs/k1rd-1.log`、`runs/k1rd-2.log`）：

```
ATK k1rd pre=4 fill=16 g=13     736 × stuck:adm ways=2 fired=1 same_as_without_fault=true (first n=1)
ATK k1rd pre=4 fill=16 g=20     938 × stuck:still[NoSpaceAfterRaisingTheFloor] ways=1 fired=1 same_as_without_fault=true (first n=1)
```

  每一次读错都被重读或另一槽接住，预演与真路一样。
- 那一段从此读不出（写也不撤，与 `DeviceWithReadFaultsOnChosenSlots` 同一种故障模型）：第 n 次读读的那一段从这一次起一直读不出，照出路二在同一组故障上真走一遍（4034 种）：`agree=false` 0 次（`grep -h 'agree=false' k1pin-1.log k1pin-2.log k1pin-3.log | wc -l` 输出 `0`）。g=20 那一格的分布（`runs/k1pin-2.log`）：

```
ATK k1pin pre=4 fill=16 g=20      16 × other(FloorRaiseFailedWhilePushingForSpace(FloorRaiseFailedWhilePushin fired_in_report=2 (first n=30)
ATK k1pin pre=4 fill=16 g=20       8 × stuck:still[FloorRaiseFailed:NewerStateStillUnreadableAfterOneReread] | follow:pushed(19) retry=other(FloorRaiseFailedWhilePushingForSpace( agree=true fired_in_report=2 (first n=721)
ATK k1pin pre=4 fill=16 g=20     232 × stuck:still[FloorRaiseFailed:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread] | follow:pushed(19) retry=other(FloorRaiseFailedWhilePushingForSpace( agree=true fired_in_report=2 (first n=102)
ATK k1pin pre=4 fill=16 g=20     369 × stuck:still[NoSpaceAfterRaisingTheFloor] | follow:pushed(19) retry=other(FloorRaiseFailedWhilePushingForSpace( agree=true fired_in_report=2 (first n=36)
```

  预演里抬 F 那一串读坏、重读仍坏时，报的是 `ChangeStillRefusedAfterThePush`、拒绝里点名那一种读错，出路不列推环；真推环做成、再发同样因读错被拒。这一格有 370 种（369 + 1）预演交回 `NoSpaceAfterRaisingTheFloor`、真再发交回 `FloorRaiseFailedWhilePushingForSpace`：两边都不是放得下、出路都不列推环；为什么一边先被空间拒、一边先读到坏段，没逐条查（推的：预演里坏段已被内存里写过的块盖住，真盘上写也不撤）。不算打中（列不列推环对得上；拒的原因不同只在 `refusal` 那一格）。
- C 那一圈罩着的隔离槽从挂上之后就读不出（`runs/k1pf.log`，五格，每格 86–110 槽）：真推环、再发都没读到它们一次（`fired_total=0`），结局与不带故障相同：

```
ATK k1pf pre=4 fill=16 g=13 faulted_slots=110 report=stuck:adm ways=2 fired_in_report=0 follow=pushed(19) retry=admitted fired_total=0 control_report=adm
```

- 预演里推环自己被拒（`PushRefused`，例如推到半路取不到落点）：上面 3393 份报里 0 次；被拒之后一直建 inode 或覆盖写（至多 40 次）也只让那次写放得下（`runs/k1fill-probe.log` 汇总 `kinds={"adm": 2, "admitted": 80}`）。自然历史里没造出这一支，它报什么只读码：出路不列推环（第 282 行那一臂），会话不动。

### K5 读法

两份判据逐句同形：先判偏移与长度是不是物理块的整数倍、再 `checked_add` 判越界，错误成员与字段相同；对拍 100225 个请求 0 差。分叉只可能来自「写不交给底下那块盘」本身：底下的盘若对写另有拒法（块设备错、屏障失败），预演看不见——这是只进内存这一做法共有的，属于「块设备错这一类」。不分辨臂、不算打中。差的一处是钉不住：把这一份改成恒放行（第五节变异 A），那份测试文件 55 条照绿；`memory_pool.rs` 那一份抄的有对拍用例，这一份没有（下一批改成共用就不需要了）。

## 三、K1-3：预演的代价

### 被判的那几句

- 每报一次都预演：`crates/singlefs-core/src/mounted_session.rs:652`「let pushing_the_root_ring = rehearse_the_push_then_the_remainder_of_the_change(」，在每一次停在上限、C 还隔离着槽的被拒里都走；预演里先拷分配器（第 497 行），推环入口本体自己再拷一份做整串预演（`crates/singlefs-core/src/mounted_session.rs:1173`「let mut copy = allocator.clone();」），再在只进内存的盘上真发整串、再发这次改动剩下的（连同它推的抬 F）。
- 分配器每块盘按槽各存两张位图：`crates/singlefs-core/src/allocator.rs:254`「    allocated: Vec<bool>,」、`crates/singlefs-core/src/allocator.rs:270`「    isolated: Vec<bool>,」。

### 量法（量法副本，`cost-patch.py`、`atk_cost_part.rs`；副本上的数）

- 一：384 槽那一档 K1 主形（pre=4、fill=16）上，同一次写各发两遍：一遍把预演关掉（副本里的开关，关掉时预演那一格直接交 `PushRefused`），一遍照常；记挂钟、这一次调用里常驻峰值比调用前多了多少（`/proc/self/clear_refs` 写 5 重置峰值再读 `VmHWM`）、只进内存的盘记下的块数与字节。
- 二：盘放大：同一个 K1 形（C 一槽、隔离 102 槽），单元区从 384 槽放到 1.34 亿槽（每块盘 2 TiB，内存稀疏盘只存写过的扇区），各量一次普通覆盖写、一次真推环、一次「预演推环加这次覆盖写」（副本里另开的入口 `atk_rehearse_for`，与「被宽护卡住」那一报调同一个预演函数，只是不必先被拒）。大盘上造不出真的卡住（单次写的上限见下），所以这一件量的是预演那一步本身，不是整次被拒的调用。

### 原样输出

`runs/cost-384.log`：

```
ATK cost area=384 units=29 write_mib=0.9 without_rehearsal=stuck:pushref 0.008s peak+0.0MiB | with=stuck:adm 0.038s peak+0.0MiB rehearsal=0.029s overlay_blocks=12660 overlay_mib=6.2 rehearsals_so_far=8
ATK cost area=384 units=35 write_mib=1.1 without_rehearsal=stuck:pushref 0.007s peak+0.0MiB | with=stuck:adm 0.032s peak+0.0MiB rehearsal=0.024s overlay_blocks=13524 overlay_mib=6.6 rehearsals_so_far=10
ATK cost area=384 units=36 write_mib=1.1 without_rehearsal=stuck:pushref 0.006s peak+0.0MiB | with=stuck:still 0.042s peak+0.0MiB rehearsal=0.035s overlay_blocks=9656 overlay_mib=4.7 rehearsals_so_far=11
```

`runs/costsize-probe.log`、`runs/costsize-big.log`、`runs/costsize-huge.log`（按盘大小排）：

```
ATK costsize area=384 device_gib=0.04 setup_s=0.0 allocator_clone_s=0.0000 overwrite=admitted 0.003s peak+1.1MiB | push=Ok(22) 0.030s peak+32.3MiB | rehearsal=adm 0.025s peak+3.3MiB overlay_blocks=9767 overlay_mib=4.8
ATK costsize area=65536 device_gib=1.03 setup_s=0.0 allocator_clone_s=0.0000 overwrite=admitted 0.002s peak+0.0MiB | push=Ok(22) 0.024s peak+13.1MiB | rehearsal=adm 0.025s peak+1.7MiB overlay_blocks=9767 overlay_mib=4.8
ATK costsize area=1048576 device_gib=16.03 setup_s=0.1 allocator_clone_s=0.0005 overwrite=admitted 0.006s peak+14.1MiB | push=Ok(22) 0.125s peak+23.6MiB | rehearsal=adm 0.091s peak+13.7MiB overlay_blocks=12839 overlay_mib=6.3
ATK costsize area=8388608 device_gib=128.03 setup_s=0.5 allocator_clone_s=0.0136 overwrite=admitted 0.042s peak+95.1MiB | push=Ok(22) 1.398s peak+68.8MiB | rehearsal=adm 1.559s peak+110.2MiB overlay_blocks=12839 overlay_mib=6.3
ATK costsize area=33554432 device_gib=512.03 setup_s=2.3 allocator_clone_s=0.0495 overwrite=admitted 0.196s peak+442.2MiB | push=Ok(22) 6.920s peak+446.2MiB | rehearsal=adm 7.131s peak+669.8MiB overlay_blocks=15655 overlay_mib=7.6
ATK costsize area=67108864 device_gib=1024.03 setup_s=5.0 allocator_clone_s=0.1189 overwrite=admitted 0.394s peak+902.1MiB | push=Ok(22) 14.006s peak+845.8MiB | rehearsal=adm 13.829s peak+1229.8MiB overlay_blocks=15655 overlay_mib=7.6
ATK costsize area=134217728 device_gib=2048.03 setup_s=10.0 allocator_clone_s=0.2206 overwrite=admitted 0.687s peak+1789.7MiB | push=Ok(22) 26.750s peak+1613.7MiB | rehearsal=adm 24.965s peak+2461.1MiB overlay_blocks=15655 overlay_mib=7.6
```

单次写的上限（`runs/cost-4096.log`，4096 槽那一档上找第一个被拒的长度，碰到的是记录条数上限，不是空间）：

```
ATK cost area=4096 units=513 write_mib=16.0 without_rehearsal=other(Publish { cause: JournalRecordsOfT 0.004s peak+0.0MiB | with=other(Publish { cause: JournalRecordsOfT 0.004s peak+0.0MiB rehearsal=0.000s overlay_blocks=0 overlay_mib=0.0 rehearsals_so_far=0
```

### 读法

- 随什么涨：预演的挂钟与内存随**每块盘的槽数**线性涨（16 GiB → 2 TiB 每块，预演 0.091 → 24.965 秒、峰值多 13.7 → 2461 MiB），推环次数（≤ 3S = 24）是乘数；只进内存的盘记下的字节几乎不随盘长（4.8–7.6 MiB，推环那 22 次空发布写的固定点与根，加这次改动的单元）。预演与一次真推环同量级（2 TiB 时 24.965 秒对 26.750 秒），是同一池一次覆盖写的 36 倍。时间花在哪一步没拆（推的：每次空发布里分配器拷贝与位图扫描，`allocator_clone_s` 一次 0.22 秒）。
- 这次改动本身那一截有界：一次发布切出来的记录条数有上限（`crates/singlefs-core/src/transaction.rs:6782`「    let record_limit = journal_record_limit_of_one_publish(journal_ring_bytes);」），6 MiB 环那一档顺序写到 513 个单元（16 MiB）就被拒成 `JournalRecordsOfThePublishExceedTheLimit`，所以预演里替这次改动多记的内存是十几 MiB 一档，不是 GiB 一档。
- 报错这一步本身：在 384 槽小盘上一报 32–42 ms，比不预演（6–8 ms）慢 5–6 倍，可用；在两块 2 TiB 的盘上一报多 25 秒、常驻峰值多 2.4 GiB，应用照 ENOSPC 重试一次就再付一次（报的那一刻会话的盘面与分配器不变，第二报的结局与第一报相同，第一节）。4 TiB 一块的盘没量（量法副本上 1 TiB 与 2 TiB 那一件整个进程峰值 4859532 KB（`/usr/bin/time` 的 `maxrss_kb`，约 4.6 GiB），推的线性外推：一报约 50 秒、峰值多约 5 GiB）。

### 打中之后的四句

1. 分不分辨臂：分辨「每报一次都整串预演」与「预演结果按报的那一刻的状态记住、同一状态再被拒时不重算」或「出路二的预演延到调用方要出路时才做」（后两种推的、没实现）；不分辨「另写一份不落盘的预演」——那一份照样要拷分配器、走一遍空发布，代价同量级（推的）。
2. 系统看不看得到：看得到：报的那一刻知道盘的大小与要推几次；第二报的输入（盘面、分配器、现行那一版、这次改动）与第一报逐字相同，第一节量过。
3. 满足的分句：K1-3「内存与时间随什么涨？有没有一形报错这一步本身就失败或慢到不可用？」前一半：随每块盘槽数线性涨、乘推环次数；后一半：没有失败的一形，「慢到不可用」是要用户定的口径，这里只给数。
4. 改法：正文没有跑前改法。攻方提的两个，都只在攻方的模型上量过代价、没实现、被攻过零轮：

| 改法 | 修哪一格 | 量过还是推的 |
|---|---|---|
| 记住上一次预演：会话里存（分配器的版本号或现行那一版的 txg、这次改动的指纹）→ 预演结局，下一次被拒输入相同就照抄 | 应用重试时的第二报以后 | 推的；第二报输入与第一报逐字相同是量过的（283 格 `again_unchanged=true`） |
| 预演延到调用方要出路时（`ways_out()` 或另开一个方法）才做 | 只要错误码、不看出路的调用方不付这一笔 | 推的；`ways_out()` 现在只读字段，改了要连带改它的签名（要盘表） |

## 四、K2：出路措辞、冻结与无隔离

- K2-1：两种 R 的出路一说法、出路二的次数与槽数照文档；N 在第一节全部 3393 报上都等于真推前后每块盘隔离数之差（`n_same=true`），两槽 C 那一形 N 每盘 26、C 罩着的隔离两盘合计 52，被抛弃根 A 与 C 同在那一形 N 含 A 独占的槽。出路二那句写「宽护随之解开，隔离的槽放出来 N 个」，在 A 那一形里 N 有一截是 A 离环放出来的、不是宽护——文档注释写明了两个数不同，话里没写，不分辨臂，不算打中。有盘读坏的槽那一格见第二节（读坏、重读仍坏时预演交回点名读错的拒绝、不列推环，真走也被同一种读错拒）。
- K2-2：冻结之外，预演过、真发第一次就被拒的状态：第一节报「推完放得下」的全部格、第二节两种读故障下的全部格，真推环都做成（`push=pushed(…)`），0 次 `push_refused`。没造块设备写错（预演不经底下的盘写，写错只在真路出现，属「块设备错这一类」，上一轮已量）。
- K2-3：C 记着落点而一槽都没隔离时报 `NotWhatStopsTheAdmission`、推环入口拒成 `NothingHeldUntilThisProcessWritesARingSlot`。读码：C 罩着的槽此刻全在已分配（现行账里或 defer 窗口里），写掉 R 那一槽放不出它们；它们回收时照补隔离（之后的被拒里才算进隔离）。被抛弃根那一圈的隔离不在 C 里，推环写掉它那一槽也能放，但那不是宽护，这一判不报它照字面成立。没造历史，限度里写。

## 五、K3：改回去照不照绿

攻方自己的十一个变异（`mutants.py`，不是入库变异表；原文恰好命中一次才改），每个在一份新拷的副本上编，整份跑那份 harness 测试文件（`--include-ignored`，55 条；未改的副本 `runs/mut-baseline.log`：`test result: ok. 55 passed; 0 failed`），再跑攻方原型的两小段（K1 主形 26 格、实现员三形 33 报；B、D、I 另跑全量几段）：

| 变异 | 改了什么（第几行） | 那份测试文件 | 攻方原型 |
|---|---|---|---|
| A | 只进内存的盘的对齐判恒放行（第 315 行那个函数开头先交 `Ok`） | 55 passed（照绿） | 0 格对不上 |
| B | 读时看不见自己写过的块（第 387 行查表的块号或上最高位） | 55 passed（照绿） | 19/26、9/33；全量 263/283、175/209、245/245，都只是 `dbg_same=false` |
| C | 读的末尾那一段没写过的块不从底下的盘读 | 13 failed（拦住） | 26/26、33/33 |
| D | 写进内存时块号错一位（第 367 行） | 55 passed（照绿） | 19/26、9/33；全量 263/283，都只是 `dbg_same=false` |
| E | 写零不记进内存 | 55 passed（照绿） | 0；发布路径不写零，等价变异 |
| F | 走空之前被拒时，预演只走空、不再发这次改动（第 996 行） | 55 passed（照绿） | 0；两边都没造出走空之前被卡 |
| G | 这次改动被拒时，预演只走空（第 1067 行） | 8 failed（拦住） | 19/26、9/33 |
| H | unlink 意图之后走空被拒时，预演再发一次这次改动（第 1041 行） | 55 passed（照绿） | 0；两边都没造出 unlink 之后被卡 |
| I | 预演里发这次改动时影子账关着 | 55 passed（照绿） | 全量 283、209、245 报 0 格对不上；这几形里等价 |
| K | 预演里推环被拒当成「推完放得下」（第 507 行） | 55 passed（照绿） | 0；自然历史里没有一报走到这一支 |
| N | 「被宽护卡住」那一报预演时只走空（第 652 行那次调用的 `remainder`） | 8 failed（拦住） | 19/26、9/33 |

原样（各份 `runs/mut-*-file.log` 的结果行，与 B 的全量那几段）：

```
A-overlay-check-always-ok | test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; | ATK k1 totals cells=26 stuck=26 disagree=0 | ATK k1impl totals stuck=33 disagree=0
B-overlay-reads-never-see-own-writes | test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; | ATK k1 totals area=384 cells=26 stuck=26 disagree=19 | ATK k1impl totals stuck=33 disagree=9
C-overlay-trailing-unwritten-run-not-read | test result: FAILED. 42 passed; 13 failed; 0 ignored; 0 meas | ATK k1 totals cells=26 stuck=26 disagree=26 | ATK k1impl totals stuck=33 disagree=33
D-overlay-keeps-each-block-one-block-later | test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; | ATK k1 totals area=384 cells=26 stuck=26 disagree=19 | ATK k1impl totals stuck=33 disagree=9
G-remainder-of-the-change-is-only-the-walk | test result: FAILED. 47 passed; 8 failed; 0 ignored; 0 measu | ATK k1 totals area=384 cells=26 stuck=26 disagree=19 | ATK k1impl totals stuck=33 disagree=9
K-push-refused-in-the-rehearsal-counts-as-admitted | test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; | ATK k1 totals cells=26 stuck=26 disagree=0 | ATK k1impl totals stuck=33 disagree=0
N-rehearsal-replays-only-the-walk | test result: FAILED. 47 passed; 8 failed; 0 ignored; 0 measu | ATK k1 totals area=384 cells=26 stuck=26 disagree=19 | ATK k1impl totals stuck=33 disagree=9
```

B 在 K1 九形全量上（`runs/mut-B-overlay-reads-never-see-own-writes-atk-k1full.log` 汇总行，`rehearsal_kinds` 截掉之后）与分布：

```
ATK k1 totals area=384 cells=283 stuck=283 disagree=263 small_stuck=0 small_disagree=0 
     20 reh=adm retry=admitted agree=yes n_same=true dbg_same=- 
    263 reh=still retry=notwhat(budget) agree=yes n_same=true dbg_same=false 
```

（同一次的 `rehearsal_kinds` 是 `still[FloorRaiseFailed:DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot]`，263 格都是；D 那一次是 189 格这一种、74 格 `still[FloorRaiseFailed:Recovery]`，`runs/mut-D-overlay-keeps-each-block-one-block-later-atk-k1full.log`。）

### 读法

- 拦住的三个（C、G、N）是「预演把这次改动丢了」与「读回拼接漏了末尾」这两类，那份测试文件里「推完照旧被拒只列卸载」那几条与「推完放得下列两条」那几条直接判到。
- 照绿而攻方原型拦得住的两个（B、D）：只进内存的盘读不回自己写过的块、或写到错的块号。列不列推环在 737 格（K1 全量 283、A 那一形 209、两槽 C 245）上一格都没翻——「推完放得下」的那些格这次改动剩下的发布不需要抬 F，「推完照旧被拒」那些格本来就被拒——变的是 `ChangeStillRefusedAfterThePush` 里那一份 `refusal`：从真路上的「推满仍不够、预算用完」变成「抬 F 那一串读到落后的盘」。那一格的文档注释说它是「预演里交回的那一种」，公开字段，调用方拿它说为什么推环不够；照绿就是没有用例比这一份与真再发的那一份。
- 照绿、原型也拦不住的：A（对齐判恒放行：核心只发对齐的请求，第二节）、E（发布路径不写零）是等价或近等价；F、H 要「走空之前被卡」「unlink 意图之后走空被卡」，实现员报告与攻方两边都没造出来，那两处 `remainder` 选哪一支没有一条用例走到；I 在这几形里等价（影子账开关只改被抛弃根那一圈的补隔离，推完之后这次改动的发布不碰）；K（预演里推环被拒）那一支自然历史里没走到，用例也没有注入一形。

### 打中之后的四句

1. 分不分辨臂：分辨「这一批的用例集」与「再加一条逐字比预演拒绝的用例」：B、D 在前者照绿、在后者红（攻方原型 263/283 格）。F、H、K 两边都不红，要注入级用例（`isolate_free_slots` 一类先把推环要的落点隔离掉，让预演里的推环取不到落点；或造一个删除队列不空的盘面卡在走空里）才分得开（推的）。
2. 系统看不看得到：看得到；B、D 改坏的是只进内存的盘，预演交回的拒绝与真走交回的拒绝在同一进程里都拿得到。
3. 满足的分句：K3「还有哪几种改坏的写法照绿？」——B、D、F、H、K 五个有行为后果、照绿；A、E、I 是等价或这几形里等价。
4. 改法（推的、零轮）：在「推完照旧被拒只列卸载」那几条里另比一样：预演 `ChangeStillRefusedAfterThePush` 里的 `refusal` 与推环之后真再发同一个改动交回的拒绝 `Debug` 逐字相同（攻方原型 `atk_follow_way_two` 的 `dbg_same` 就是这一判，量过：B、D 下红、原样下绿）；K 那一支加一条注入级用例：卡住之后把推环要的空闲槽隔离掉，钉住出路只列卸载、推环入口真调也拒成 `RefusedByTheRehearsalBeforeAnyWrite`（推的，没写）。

| 改法 | 修哪一格 | 量过还是推的 |
|---|---|---|
| 比预演交回的拒绝与真再发的拒绝 | B、D | 量过（攻方原型上：`runs/mut-B-…-atk-k1full.log` 263 格 `dbg_same=false`，原样 `runs/k1-main.log` 0 格） |
| 注入级「预演里推环取不到落点」 | K | 推的 |
| 注入级「走空之前 / unlink 意图之后被卡」 | F、H | 推的 |

## 六、K4：第二十一块扫源码

不跑 checker 档测试。量法：`build-scan-proto.sh` 把两份源文件测试模块里的扫源码函数按行原样抄进一个独立小 crate（行号写在它造的源文件头一行；脚本生成的源文件与这一轮量的那一份 `cmp` 过、逐字相同），对冻结副本的源文件与 `gen-scan-inputs.py` 造的改法各扫一次。副本里 `cargo check -p singlefs-checker-tier` 在 `librocksdb-sys` 的构建脚本上失败（`runs/k4-v2-check.log`），没能对改过的源文件做一次真编；下面改法能编过是读 Rust 语法推的（块注释里任何文字都不进编译）。

### 原样输出（`runs/k4-stop-scan.log`、`runs/k4-stop-scan-fp.log`、`runs/k4-gpu-scan.log`；`sed` 去掉了输入目录前缀，`cut -c1-200`）

```
ATK scan v0-frozen.rs found=0
ATK scan v1-new-layer-plain.rs found=1
ATK scan   收叫停的是这六层，连同没编进 gpu 特性时那一份 judge_my_blocks_on_gpu（多一层少一层、参数换了名字都要回来看这条用例罩没罩到），读到的是 [
ATK scan v2-new-layer-behind-a-commented-opening.rs found=0
ATK scan v3-multiline-new-layer-behind-a-commented-opening.rs found=1
ATK scan   收叫停的是这六层，连同没编进 gpu 特性时那一份 judge_my_blocks_on_gpu（多一层少一层、参数换了名字都要回来看这条用例罩没罩到），读到的是 [
ATK scan v4-trailing-comment-with-fn-in-a-signature.rs found=1
ATK scan   收叫停的是这六层，连同没编进 gpu 特性时那一份 judge_my_blocks_on_gpu（多一层少一层、参数换了名字都要回来看这条用例罩没罩到），读到的是 [
ATK scan v5-attribute-line-before-a-layer.rs found=0
ATK gpuscan g0-frozen.rs found=0
ATK gpuscan g1-use-wgpu-itself.rs found=1
ATK gpuscan   卡池模块经 use 引进叫 wgpu 的名字（引的是 wgpu），遮住外部 crate
ATK gpuscan g2-block-scoped-rename-in-a-helper.rs found=1
ATK gpuscan   卡池模块经 use 引进叫 std 的名字（引的是 core），遮住外部 crate
ATK gpuscan g3-commented-tests-opening-inside-the-pool.rs panicked
```

v4 那一条读到的名单头一项（`grep -o`）：`读到的是 [("外面定", "stop_is_requested"), ("check_pending_blocks_in_pieces", "stop_is_req`。

### K4-2 打中（窄）：块注释里写测试模块的头两行，藏住一层

- 被判的那几句：「测试模块之外」按文本切，`crates/singlefs-checker-tier/src/crash_amplification.rs:4668` 是「            .find(TESTS_MODULE_OPENING)」，取的是源文件里**第一处** `#[cfg(test)]` 换行 `mod tests {`（`crates/singlefs-checker-tier/src/crash_amplification.rs:3581` 那个常量），收尾是之后第一处顶格的右花括号行（第 4672 行）；字符串与块注释不认。`crates/singlefs-checker-tier/src/crash_amplification.rs:4633` 读环境变量名的那一扫用同一种切法。
- 造法（v2）：在真测试模块头两行之前插四行块注释（`/*`、`#[cfg(test)]`、`mod tests {`、`*/`），紧接一行写完的新层 `pub fn an_extra_layer_never_asking(first_input: u8, stop_is_requested: &(dyn Fn() -> bool + Sync)) -> bool { let _ = stop_is_requested; first_input == 0 }`。切法从注释里那一处起、到真测试模块的收尾止，新层与真测试模块一起被当成「测试模块」切掉：收叫停那一扫 0 处（v2），同一层不加注释 1 处（v1）。新层写成多行、右花括号顶格（v3），切到它的右花括号就停，真测试模块露出来，照样红——要骗过，新层的右花括号不能顶格。
- 四句：①分辨「按第一处文本切」与「先认注释与字面量再切、或要求测试模块头两行在源文件里恰好一处」；后者 v2 判红（`runs/k4-inputs.log` 里 `gen-scan-inputs.py` 打印的次数：v0 1 次、v2 2 次，量过）。②看得到：源文件全在手里。③满足 K4-2「还有哪几种写法骗得过」。④改法（推的、零轮）：`code_lines_outside_the_tests_of` 与第 4633 行那一扫开头各断言 `source.matches(TESTS_MODULE_OPENING).count() == 1`；不在文档注释的「拦不住」里另记一条也行。这一格只在攻方抄出来的那份扫源码函数上量过，没在 checker 档测试里跑。

### 判红的合法写法（窄，误判）

- 崩溃放量那一扫取层名照 `crates/singlefs-checker-tier/src/crash_amplification.rs:4883`「        /// 往前最近的 `fn` 后面那个名字（往前没有 `fn` 的交空串）。」，取法在 `crates/singlefs-checker-tier/src/crash_amplification.rs:4917`「                let layer = whole_word_positions(&code[..ampersand], "fn")」，只去掉整行注释，不去掉行尾注释与字符串：签名里参数行尾写 `// 判法版本，fn 外面定`（v4），层名取成「外面定」，名单对不上、红。属性行与行尾注释不带 `fn` 时（v5）不红。
- 卡池那一扫把 `use` 引进叫 `std`、`wgpu` 的名字一律拒（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2776`「                    if matches!(name.as_str(), "std" | "wgpu") {」）：`use wgpu;`（引的就是外部 crate 自己，g1）与别的函数体里块作用域的 `use core as std;`（只在那个函数里有效、碰不到钉住那两段，g2）都判红。g3（卡池模块里写同样那段块注释）扫的时候当场 panic，卡池这一侧骗不过。

### K4-1 没打中

新放行的四类在卡池模块这一层换不掉钉住那两段用的名字：`as` 改成的名字、几层花括号里的名字、花括号里 `self` 取的名字都照引进来的名字判（g2 那一条就是改名之后被认出来的）；`extern "C" { … }` 里能声明的只在值的命名空间（函数、静态量），路径首段 `std::`、`wgpu::` 按类型命名空间解析，遮不住（推的，Rust 名字解析）。宏那一条另试过：卡池模块外面、文本上在它之前定义的同名 `macro_rules!` 会顶替方法体里调的标准库宏（`atk_macro_shadowing.rs`，独立小程序跑出「ATK 外面定义的 eprintln! 被调到了」那一行，`rustc` 编、经内存包装跑），可钉住的「借之前那一步」整句里没有宏调用，借据丢掉时那一步只钉五段文本、`eprintln!` 所在的那个闭包本来就不在钉住的文本里，直接改闭包也照绿——不是这一批新开的口子，不算打中。

## 没打中的形状

| 形状 | 取样范围 | 结局 |
|---|---|---|
| K1-1 预演说放得下、照推环走完同一个写被拒 | 3393 份报（第一节六件） | 0 次 |
| K1-1 预演说放不下、推完其实放得下 | 同上 | 0 次 |
| K1-1 预演放出来的槽数与真推之差不同 | 同上，逐盘比 | 0 次（含两槽 C、被抛弃根 A 与 C 同在） |
| K1-1 第二报与第一报不同、或第二报动了盘与会话 | 283 格 | 0 次；0 个改盘调用 |
| K1-2 写穿到底下那块盘、读回拼错 | 360000 次随机请求 × 2 种物理块 × 3 种盘长 | 0 差；底下那块盘一字不变 |
| K1-2 一次性读错让预演报错、真路却放得下 | 五格 × 每一次读（4034 种） | 0 次；每一种都与不带故障逐字相同 |
| K1-2 那一段从此读不出时预演与真走分叉 | 五格 × 每一次读（4034 种） | 列不列推环 0 次分叉；370 种拒的原因不同（两边都拒） |
| K1-2 隔离着的 C 槽永久读不出 | 五格 | 真走一次都没读到它们 |
| K1-2 预演里推环自己被拒 | 3393 份报，另被拒之后建 inode / 覆盖写至多 40 次 | 一次都没走到这一支 |
| K2-2 冻结之外预演过、真发第一次被拒 | 第一节全部「放得下」格与第二节两种读故障 | 0 次 |
| K4-1 新放行的四类遮住钉住的名字 | 读码加 g1、g2 两种输入 | 遮不住；倒是两种合法写法被判红 |
| K4-1 外面的同名 `macro_rules!` 顶替钉住段里的宏 | 独立小程序 | 顶替得了，但钉住的整句里没有宏 |
| K5 两份对齐判不同 | 100225 个请求，5 种物理块 × 5 种盘长 | 0 差 |

## 这条腿自己的限度

- 全部数在草稿目录的副本上，不是入库装置上的数；K1-3 的数在量法副本上（core 里加了开关与读数、另开了一个只供量法的预演入口），K4 的数在攻方抄出来的扫源码函数上，都不是 checker 档测试的结局。
- K1-1 比的是「推环、再发逐字节同一个改动一次」这一条走法；推完先做别的、或推完发另一个改动，那一报没有说过，没比。根环每区槽数只有 mkfs 写的 S = 8 一档（3S = 24），没在别的 S 上量。卡住的形只造在单元区 384 槽及更窄的四档上。
- 「走空之前被卡」「unlink 意图之后走空被卡」「预演里推环取不到落点」三支没造出来，K3 的 F、H、K 照绿只说明没有用例走到，没说明它们改坏之后报错的样子。
- K1-3 大盘那一件量的是「预演推环加一次覆盖写」那一步（副本入口），不是整次被拒的调用；4 TiB 一块的盘没量；时间花在哪一步没拆。
- 读故障只造了两种模型（一次性、写也不撤的那一段）；「写会撤读故障」的那一种（`DeviceReadingBackAllZeroOnRanges`）与只进内存的盘同一种语义，没另量；写错、屏障失败不进预演，是这个做法共有的，没量。
- K1 那几形的盘面是照上一轮的造法从录下的写流里截一个前缀得来的（崩在 R 第一条记录之后，一个崩溃状态），不是层 0 的整写子集枚举：派发这一轮不许起崩溃枚举，所以没调 `enumerate_layer0` / `enumerate_layer0_versions`，也没算 `closed_form_state_count`；这一轮原型里没有别的崩溃状态。
- K4 只在抄出来的函数上跑；副本里对改过的源文件 `cargo check -p singlefs-checker-tier` 编不过（失败在 `librocksdb-sys` 的构建脚本，与改动无关），v2 能编过是读语法推的。
- 攻方自己提的改法（第三节两条、第五节三条、第六节一条）都只在攻方的模型上量过或推过，被攻过零轮。

## 取样与估时

正式跑之前各先跑一小段估时，估出的全量都在 40 分钟以内，没有缩历史、候选与几何的取样。开跑前 `ps` 看过：没有性能测量在跑、没有别的 cargo。并行时每件 3G、线程 3，一批至多 4 件（合计 12G、12 线程）；不给批设限时。

| 批 | 件 | 先跑的一小段 | 估时 | 实际 |
|---|---|---|---|---|
| K1 主形扫 | 1 件（10 形 283 格） | 一形 26 格 6.53 秒 | 约 65 秒 | 80.94 秒（定稿后重跑一次同数） |
| 放开小动作 | 4 件（283 格 × 10） | 同上一行，每次跟走约 0.2 秒 | 每件 1–2 分钟 | 56.80–102.71 秒，4 个 `.rc` 都是 0（定稿后重跑一次同数） |
| 实现员三形、两槽 C、A 与 C | 4 件 | 没另估（与上一行同一种量） | 每件 1–3 分钟 | 3.96–21.43 秒，4 个 `.rc` 都是 0 |
| 窄四档单元区 | 4 件 | 同 K1 主形 | 每件 1–2 分钟 | 4 个 `.rc` 都是 0 |
| 一次性读错 | 先 1 件 5 格，定稿后 2 件 | 一格步长 50：0.66 秒 | 每格约 30 秒 | 180.89 秒（5 格）；重跑 2 件，2 个 `.rc` 都是 0 |
| 那一段从此读不出 | 3 件 | 一格 30.92 秒 | 每件 1 分钟上下 | 3 个 `.rc` 都是 0 |
| 代价 | 3 件，单独跑 | 384 槽 1.43 秒 | 大盘几分钟 | 0.54、21.22、98.79 秒 |
| 变异 | 11 个，逐个（编译约 30 秒 + 测试文件 14 秒 + 原型两段 11 秒） | A 一个 60.80 秒 | 每个 1 分钟 | 每个约 1 分钟；B、D、I 另跑全量几段各约 2 分钟 |
| 只进内存的盘对拍、对齐判对拍 | 1 件 | 没另估 | 1 分钟内 | 22.17 秒 |

## 没做什么

- 没跑任何门禁阶段、checker 档测试、崩溃枚举、GPU 测试；编、跑的只有 harness 档（拼进来的 `atk_tu_opus`、`atk_tu_cost_opus` 与那份 harness 测试文件）、`singlefs-core` 量法副本的库单测（`atk_overlay` 两条）、草稿目录里的两个独立小 crate。
- 没判正推、辩方的格；副本上量出的数不进 kb，要引得在入库装置上重做。
- K2-3 没造历史，只读码；K4-1 没在 checker 档里编、跑。
- 草稿目录里的仓副本与编译目录（`~/.local/share/singlefs/work/m3-batch10z-tu-code-r1-opus/` 下的 `repo`、`repo-cost`、`repo-mut`、`target`、`target-cost`、`target-mut`、`scan-proto/target`、`macro-proto/atk-macro`）交回前删掉；原始输出已拷进模型目录 `runs/`，草稿目录只留日志、`model-src/` 与两个小 crate 的源码。

## 引文核对

`python3 scripts/cite-check.py research/prompts/m3-batch10z-tu-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m3-batch10z-tu-code-r1/tree --root /home/fy5090/code/singlefs` 末行原样：

```
  ✓ 核了 14 处引文，对上 14 处，没判 0 处（逐处列在上面）
```
