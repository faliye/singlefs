# m3-close-rootring-r2 云端正推腿报告（Sonnet）

日期 2026-10-20。立场：正文第四节分工表「云端正推」——把这一轮四个形态（C743 补第 0 代例外、C751 同实例的根当作不被抛弃、C755 只统一落到哪一版、C773 另记钉槽且 checker 现算求等）写成条文，逐处列改动点、结局与代价；第一轮已列的引第一轮报告（`research/prompts/m3-close-rootring-r1-sonnet-output.md`、`research/prompts/m3-close-rootring-r1-opus-output.md`），不重列。

## 各格判定一览（开头版；末尾另有一张）

| 格 | 判定 | 依据 |
|---|---|---|
| Y1 C743 补第 0 代例外 | 一致；冲突 1 处（D22 实例代号行的平局括号与已定项 16 的公式句）；规则没说 1 处（例外放行的读错位盲区写不写进条文） | 例外 = txg 为 0 且槽号为 0，与 I-7.17 和 mkfs 种根的槽同一个集合；副本上量过：122 张合法镜像、2094 个根，例外拒 0，裸公式拒 19（全是年轻环的第 0 代根）；三处结局两者都与今天相同；三条对照（错位会被拒、例外盲区、例外只认槽 0）都量过 |
| Y2 C751 同实例 | 一致 | 同实例的根按表恒不被抛弃（行只写给更旧的实例）；找不到单故障加一次断电内的反例；副本上量过：C751 登记那一形只读从 Pm 变 P，恢复与可写不变，`:656` 三处一个字不变 |
| Y3 C755 只统一落哪一版 | 一致；规则没说 2 处（退不成时保底写不写；GPU 判官不搬并集） | 失败取三样读者的并集、只判退不判成败、退不成留在所选版；副本上量过：并集 0 处挂不上，三处都成且同版 16→46 格，换版 42 处（R 没见证）；单取打开 / 交集挂不上 52 处，单取建上一版 8 处；与 C754 一致的写法是三处触发后各重扫 |
| Y4 C773 另记钉槽 | 冲突 1 处（只加字段与求等修不了 C773 那一形；要修，I-3.1 得多并一类，写法决定 C684 红绿）；规则没说 2 处（折中写法没被攻过；被抛弃根独占又被钉扣着没回收的槽） | 字段 t 8 字节加 p 1 字节，根记录 457→466，journal 记录头 487→496，写盘次数不变；p 取最小钉值，一个字节同时认槽和钉值；I-7.18 求等、I-7.19 钉槽不住本实例的根；读码加算术，没有模型 |
| Y5 四个形态一起 | 一致；耦合 1 处（Y1 单上会让 C773 红的发生面变宽，要和 Y4 一起） | Y2 与 Y3 改 F2 的两个位置，100 格与 `:656` 结局不变，只有 C751 登记那一形的只读变；Y1 让分配器的最旧有效根只低不高，checker 一侧不变，靠 Y4 的 p 接上 |

## 约定

- 判定词：一致（条文写得出、与已定条款和实现对得上）、冲突（两句并排整行抄在该格里）、规则没说（条款没写、给候选读法）。
- 补丁前后：系统配置落地补丁（`~/.local/share/singlefs/work/m3-close-w8-sysconf-landing/patch/crates.patch`）改 `mount.rs`、`recovery.rs`、`transaction.rs`、`system_configuration*.rs`；这一轮的改动点（四处解析、F2 一行、F2 拆分、采样点）都不在补丁改的范围里，补丁前后改法一样，补丁之后只是 `mount.rs` 里调用点的行号后移，按函数名认。行号都是主工作区今天现查的行号。
- 「副本上量过」指：我在草稿目录的仓副本里给核心层接了开关（`tools/core-switch.patch`）、跑了自己写的三份 harness 测试；不是入库装置上的数。「推的」指读码或算术，没跑。kb 条款写 kb 文件名加那份文件自己的行号，行号在 kb 文件里现查。
- 实现今天的样子：`crates/` 没变，补丁没打进主工作区；读根环槽的字节只经四处解析（`crates/singlefs-core/src/recovery.rs:1568`、`crates/singlefs-core/src/mount.rs:2737`、`crates/singlefs-core/src/instance_table_sweep.rs:481`、`crates/singlefs-core/src/recovery.rs:4172`），F2 判法在 `crates/singlefs-core/src/recovery.rs:6893`，三处触发在 `crates/singlefs-core/src/recovery.rs:7115`、`crates/singlefs-core/src/mounted_read.rs:655`、`crates/singlefs-core/src/mount.rs:8000`，钉住的槽与回收门槛在 `crates/singlefs-core/src/mount.rs:2857`、`crates/singlefs-core/src/mount.rs:3178`、`crates/singlefs-core/src/allocator.rs:2591`。

## 模型与复跑

模型目录 `research/prompts/m3-close-rootring-r2-sonnet-model/`（核心层开关补丁 + 三份 harness 测试 + 一份推算脚本 + 我的原始输出）。复跑命令（整条经内存包装；`--frozen` 是开工时 `crates/` 的未打补丁拷贝）：

```
cd /home/fy5090/code/singlefs && bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m3-close-rootring-r2-sonnet-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-rootring-r2-sonnet/frozen --work <新的空草稿目录>
```

我在副本里跑的是 `cargo test --offline -p singlefs-harness --test <三个目标> -- --nocapture`（Y3 那份单线程，两个测试共用一个全局开关），三份都 `ok`，退 0；`rerun.sh` 本身没试跑（`--selftest` 与整条脚本都被 heavy-test-guard 拦在包装之外，脚本只改了模板的 `rerun_steps` 一段，`bash -n` 过）。

| 文件 | sha256 |
|---|---|
| `tools/core-switch.patch` | e2c5b9f354949e3527d2bc451e28a6bfcc50508d9962ae4443b39f16e235309e |
| `tools/derive_y3.py` | 7506722fda0130f34582375c06a721935b34df3a548f3f5992d8191df05e67b5 |
| `tools/rerun.sh` | 589ca6505be7fd9a007717f87d9d25e051410565eaadbb463fad1e110bf892da |
| `tests/rootring_r2_forward_y1_legal_roots_under_the_formula_check.rs` | ebf0dca3819fd861f3406f2169b31be3a7cce169aedafd038b04bd5f8d93ee6d |
| `tests/rootring_r2_forward_y2_same_instance_f2.rs` | 4c7d56909c8699675464766201b3533be1577e7ad165ec85b9e44030944b66bc |
| `tests/rootring_r2_forward_y3_y5_trigger_sweep.rs` | 7919ba8436390a1a8e4dd96acea2dd980557fcc0c46b7a9ca369948a1b6c35e9 |
| `out/y1.out` | a777bebc8daf386484c40a358ab3434b25c00738727684ae90905006a02c64e5 |
| `out/y2.out` | 23e30f3675059a69a497dea1bf6048bfd7cf863921460af58537ca83954aaf79 |
| `out/y3.out` | f5c70d9df8ded1454f569ffa6dad66b9e2c6321857cd7e33bbdbba3a4e21307d |
| `out/derive_y3.out` | 3f5c1f26e72ae2c3fe5eb9a33c34d3a8d4e38f358ac6677bf426267a98ce20ae |

读的东西：主工作区 `crates/` 现状；第一轮两腿报告与判决；`research/prompts/m3-close-rootring-r1-opus-model/out/union.out`；kb 条款；补丁只读过（C754 的重扫在 `crates.patch` 第 652 行起）。

## Y1　C743 补了第 0 代例外之后

### Y1.1 条文（要写进 kb 的字）

例外的条件写成：`checkpoint_txg == 0` 且 `槽号 == 0`，区域不判。三个区域各一个槽 0，正是 mkfs 种根的那三个槽（`crates/singlefs-core/src/make_filesystem.rs:676` 的 `genesis_root_ring_slots`），池级 checker 的 I-7.17 已经按这个写（`crates/singlefs-checker/src/image.rs:1272`）。三处是同一个集合，不是三个近似。

D17 已定项 1 的定案句（`.claude/kb/decisions/17-实现分层与第三方管道.md:24`，现文「核心层规定一个失败模型——扇区 512 原子、写可撕裂到扇区、屏障之间可重排」）里，「屏障之间可重排」括号之后加一句（第一轮正推报告 X1.1 的拟稿，只改了例外那一括号）：

> 读根环槽读回的可以是本池另一个根环槽里一条自证得过的根；核心层用落点公式合成：读根环槽读出的根，它的 checkpoint_txg 照 `root_ring::target_for_publish` 不该住这一槽的按自证不过处置（`checkpoint_txg` 为 0 的第 0 代根只认槽 0、不核区域，mkfs 把它种在三个区域的槽 0）；core 里读根环槽的每一处同一处置。读回别池的字节（fsid 已挡）、字节自证不过（校验和已挡）、读回同一槽较旧一圈写下的内容（位置对、时间错）、写错位（I-7.17 判）不在这一项里；读回的恰是另一区域槽 0 里的第 0 代根，公式看不出，也不在这一项里。

D16 已定项 1「根槽这一次读坏」那一行（`.claude/kb/decisions/16-发布语义.md:39`，现文开头「挂载那一刻就读不出或自证不过」）：「读不出或自证不过」三处各加「、或读出的根按落点公式不住这一槽」。D22 已定项 7 字段表实例代号行（`.claude/kb/decisions/22-单元原子性怎么合成.md:140`）括号里的「设备失而复得会造出两条同 txg 的合法根」与已定项 16 的「区域内槽位 = `(txg div R) mod S`，无状态、只由 txg 与 (R, S) 决定」（`.claude/kb/decisions/22-单元原子性怎么合成.md:335`）并排是冲突，见 Y1.4。

### Y1.2 改动点（补丁前后）

读根环槽的字节只经四处解析（逐处现查，与第一轮报告同）：`crates/singlefs-core/src/recovery.rs:1568`、`crates/singlefs-core/src/mount.rs:2737`、`crates/singlefs-core/src/instance_table_sweep.rs:481`、`crates/singlefs-core/src/recovery.rs:4172`。补丁（`~/.local/share/singlefs/work/m3-close-w8-sysconf-landing/patch/crates.patch`）不碰这四处和 `root_ring.rs`，所以补丁前后改法一样，只有 `mount.rs` 里补丁之后的行号后移。

| 位置 | 改法 | 要点 |
|---|---|---|
| `crates/singlefs-core/src/root_ring.rs`（挨着 `target_for_publish`，`crates/singlefs-core/src/root_ring.rs:109`） | 加 `root_lives_in_the_slot_its_txg_names(txg, ring_slot, slots_per_region) -> bool`：txg 为 0 时 `ring_slot.slot == 0`，否则 `target_for_publish(txg, S) == ring_slot`；把 `genesis_root_ring_slots` 从 `crates/singlefs-core/src/make_filesystem.rs:676` 搬到这里并改成 pub，mkfs 写根（`crates/singlefs-core/src/make_filesystem.rs:621`）与这一核读同一个定义 | C743 登记行写的第 0 代根照 `genesis_root_ring_slots` 另认（转述）就是这一条 |
| `crates/singlefs-core/src/recovery.rs:1568` 之后、`crates/singlefs-core/src/recovery.rs:1570` 的 `note_that_the_bytes_read_self_verify` 之前 | 解出的根不住它的槽，交回 `Bad(NotSelfVerified)` | 必须在 `note…` 之前：当场重读那一遍的读者拿 `note…` 换掉这次挂载记下的那一份，错位的字节不能被报成自证过 |
| `crates/singlefs-core/src/mount.rs:2737` | 同上，交回 `BadAndNotAllZero` | 重建分配器两遍、进程内重读走这一个 |
| `crates/singlefs-core/src/instance_table_sweep.rs:481` | 同上，按「这次不删」处置 | 第一轮报告 X1.3 表最后一行的更保守一格 |
| `crates/singlefs-core/src/recovery.rs:4172` | 不用改 | 它拿别处已经核过的根去比字节 |

调用方一律不改签名：两个读函数都拿得到 `immutable_sizes.root_ring_slots_per_region`。补丁新增的两处读根环槽的调用（`crates.patch` 第 302 行起 `mount.rs` 里的函数 `rings_of_the_device_against_the_selected_version`，第 1280 行起 `system_configuration_repair.rs` 里的 `root_of_the_asserted_version_is_readable`）都走 `read_root_ring_slot`，核放在函数里就一并罩住，补丁之后落地时不用另改。GPU 判官不用改：崩溃状态生成器不造读错位。

### Y1.3 还有没有别的合法根会被判成错位

逐类，先推、再看模型（模型的做法和输出在 Y1.5）：

- 根环洞（记录落了、根槽没落，下一次挂载把这个 txg 跳过）：洞那一槽里留着的是上一圈同一公式写下的根，txg 与它所在的槽照样对得上。判的是位置，不是新旧。
- 跨区域回绕：区域号 `txg mod 3`、区域内槽号 `(txg div 3) mod S`，读槽取盘用的区域归属表和写根用的是同一张（`crates/singlefs-core/src/recovery.rs:1563`），回绕只让同一个槽隔 3S 个 txg 再被写一次。
- S 与 R：S 在系统不可变配置里、R 是格式常量，两头是同一个字段。
- 同 txg 两条合法根：见 Y1.4。结论是第 0 代三份之外，公式之下不存在。
- 第 0 代根：只有 mkfs 写它（`crates/singlefs-core/src/make_filesystem.rs:621`），三份同字节。一次断电之后区域 1、2 的槽 0 可以留着它一整圈（第一轮攻方 `genesis.out` 量过），例外放行。

### Y1.4 D22 的两句话与公式核

两句并排（整行抄，行号现查）：

- `.claude/kb/decisions/22-单元原子性怎么合成.md:140`：「择新在 checkpoint_txg 平局时按它高者赢（设备失而复得会造出两条同 txg 的合法根」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:335`：「区域内槽位 = `(txg div R) mod S`，无状态、只由 txg 与 (R, S) 决定」

推的：一个 txg 只对应一个 (区域, 槽)，区域落在哪块盘由 mkfs 写下的区域归属表定死（第一版 `crates/singlefs-core/src/make_filesystem.rs:63`），所以两条同 txg 的根只能争同一块盘上的同一个槽，后写的盖掉先写的；丢盘期间新时间线写不进那块盘的区域，按根槽写失败重发时 checkpoint_txg 推进一格再发的条款换 txg（转述）。C88 登记行（`.claude/kb/checks-owed.md:80`）那句「设备失而复得会让根环里并存两条同 checkpoint_txg 的自证合法根」我没有找到走得到的盘面。这一条是规则没说的一格：第 140 行的括号要改成「平局只出现在 mkfs 种的三份第 0 代根之间，三份同字节」，或者加一个走得到的盘面。择新键仍写 `(checkpoint_txg, 实例代号)` 不用动。入库里两个手摆平局的用例（第一轮报告 X1.5 点了名：`root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot.rs` 与 `common_images_for_the_checker_tier/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs`）把两条同 txg 的根放进公式之外的槽，加核之后要改成放公式槽的不同 txg 用例。

### Y1.5 模型：合法历史上公式核会不会误判（副本上量过）

装置：`research/prompts/m3-close-rootring-r2-sonnet-model/tests/rootring_r2_forward_y1_legal_roots_under_the_formula_check.rs`，核心层带一个开关（`tools/core-switch.patch`：新文件 `r2_switch.rs`，在 `crates/singlefs-core/src/recovery.rs:1568` 与 `crates/singlefs-core/src/mount.rs:2737` 两处解析点后接公式核）。开关 0 是今天，1 是裸公式（第一轮用户那一支字面），2 是带第 0 代例外（这一轮 C743 的形）。三个开关各把每张镜像过一遍：逐槽读根环（统计被核拒掉的自证过的根），再跑恢复、只读挂载、可写挂载，结局与开关 0 逐镜像比。**副本上的数，不是入库装置上的数。**

合法历史三组，共 122 张镜像（2094 个自证过的根）：

| 历史 | 镜像数 | 裸公式拒掉的根 | 带例外拒掉的根 | 三处结局与今天不同的镜像 |
|---|---|---|---|---|
| h1：mkfs；mkfs 后第一次可写挂载的每个写前缀各断电一次（13 张）；断电后再挂两次 | 16 | 19（全在第 0 代根还没被盖掉的几张：区域 1、2 的槽 0） | 0 | 0（裸与例外都是 0） |
| h2：一个实例里覆盖写 k 次（k = 3、30、60，一圈 24 个 txg，30、60 转过一圈与近三圈），再录一次发布，它的每个写前缀（含根槽之前、记录之后的「洞」）断电一次 | 78 | 0 | 0 | 0 |
| h3：覆盖写与断电再挂载交替 14 轮（每轮新实例、写行、暖机，txg 到 85，转过三圈多），每步一张 | 28 | 0 | 0 | 0 |

读法：

- 例外之后 122 张镜像、2094 个根一个都没被拒；裸公式拒的 19 个全是第 0 代根（h1 的年轻环），h2、h3 里转过一圈以上的环上洞、回绕、跨区域、新实例跳号都没有被拒的根。这印证 Y1.3 对「别的合法根」的逐类推演，不是新的推演。
- 三处结局与今天不同的镜像：开关 1、2 都是 0。裸公式在年轻环上也没改变恢复、只读挂载、可写挂载的结局；它改的是分配器眼里的根环表（第 0 代根被当成读坏的槽，钉住回收门槛、并进抬 F 的上限）。这一层我没量，第一轮攻方说的抬 F 上限钉在 1（转述）仍是推的。例外要不要，不靠这三处结局，靠分配器那一层的同一个后果：裸公式把区域 1、2 的第 0 代根当读坏后，重建分配器判每块盘读出了有效根没有（`a_valid_root_was_read_on_every_device`）时盘 1 一条都没有，会整环重读一次（rr2）——`crates/singlefs-core/src/mount.rs:3010` 的注释写的是公式归盘把三份第 0 代根都算到盘 0 的同一个后果。这一层我没量。
- 对照一（会红）：把区域 0 槽 3 的字节抄到区域 1 槽 3（一条非第 0 代根住到别的区域）：开关 1、2 都拒这一条（`rejected_by_exception=[(1, 3, 14, 81)]`），开关 0 放行。
- 对照二（例外的盲区）：把第 0 代根的字节抄到区域 1 槽 0：开关 1 拒，开关 2 放行（0 条被拒）。例外放行的这一类读错位公式看不出，要写进条文末句（Y1.1 已写）。
- 对照三：把第 0 代根的字节抄到区域 1 槽 3：开关 2 拒（`rejected_by_exception=[(1, 3, 0, 0)]`）。例外只认槽 0，不是只认 txg 0。

推翻它的现象：找到一条合法写者写出的根，开关 2 拒它（Y1.3 的任一类里出现一张被拒镜像）。

### Y1.6 十处入口按自证不过处置之后，单故障加一次断电内结局变没变

不变。理由两层：（一）合法写者写出的根都过这一核（Y1.5 的 122 张镜像里 0 个被拒），所以改后每一处读到的根集合与今天逐字相同，没有一处分支被换；（二）核只在读出错位根时才起作用，那是读故障，不在一次断电造得出的状态里。读错位本身出现时（读故障加断电，或读故障加读故障）结局的逐处比对，第一轮正推 X1.3、X1.4 与攻方 X1.3 已列，不重列；一条补充：根归「读坏」之后，F2 里 `readable_roots`（`crates/singlefs-core/src/recovery.rs:6977`）读不到它，区间里少一条根，只会更不退（Y2 的条件也随之更难成立）。

## Y2　C751 同实例的根当作不被抛弃

### Y2.1 条文与改动点

条件只改 F2 判法在「所选根的实例表读不出」那一支认哪些根。今天那一支只认恰好等于见证到的那一版（`crates/singlefs-core/src/recovery.rs:6996`，现文「None => order == witnessed_publish,」）；改成「恰好等于见证到的那一版，或与所选根同一个实例代号」，区间 [见证到的那一版, 所选根) 与取最新一条的写法不动。

| 位置 | 改法 |
|---|---|
| `crates/singlefs-core/src/recovery.rs:6996`（补丁前后都在，补丁不碰这个函数） | 那一臂改成 `None => order == witnessed_publish \|\| root.instance == chosen_root.instance`，`chosen_root` 已在函数参数里 |
| `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:219` | `var is_admissible = is_the_witnessed_version;` 后加 `|| root.instance == chosen.instance`；对拍用例 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 不跑就不知道有没有别的分叉（没跑） |
| `crates/mutations.tsv:2931` | 锚在 `None => order == witnessed_publish,` 上的行锚点要重核；加一行表读不出时只认恰好相等的一条，由新用例抓 |
| D23 已定项 14 F2 段（`.claude/kb/decisions/23-journal的角色与格式.md:398`） | 「所选根的实例表读不出时只认与见证到的那一版恰好相等、读得出的那条根」改成「所选根的实例表读不出时认两类读得出的根：与见证到的那一版恰好相等的，和与所选根同一个实例代号的（实例表只有比表的主人旧的实例的行，同实例的根按表判也不被抛弃），取择根序最大的一条」 |
| C751 登记行（`.claude/kb/checks-owed.md:503`） | 随实现销账；登记那一形只读挂载落到已确认的 P |

### Y2.2 同实例的根按表判从不被抛弃：条文和实现对得上

实例表的行 (i, Ti, Wi) 只写给比表的主人旧的实例：新实例写行时只写所选根那个实例到新实例之前的各行（`.claude/kb/decisions/23-journal的角色与格式.md:406`，现文「行只落在 [这个根的实例, 新实例)」）。所以任何根的表里没有它自己那个实例的行。抛弃的判法（`crates/singlefs-core/src/recovery.rs:1877`，现文「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」）是表里有这个根的实例的行且根的 txg 比行里的 Ti 大，对同实例的根恒假。结论：这个改法在「所选根的表读不出」时对同实例的根给的结局，恰好等于「表读得出」时给的结局，没有造出新的取法。

### Y2.3 有没有一形，同实例的根其实已被抛弃

- 同一实例内被回退抛弃：没有这个机制。管理员回退是挂着时的一次向前发布，不抛弃任何根（`crates/singlefs-checker/invariants/02-根环.md:11`，I-7.4 那一行，现文「管理员回退是挂着时的一次向前发布，不抛弃任何根」）。回退发布的根是同实例里一条 txg 更大的根，内容取 R_old，是发布过、确认过的一版，退到它读出的是一个完整的版本。
- 被取号之后的新实例抛弃而实例代号没变：抛弃 R 的实例 j' 一定大于 R 的实例 j，它的根 (txg, j') 在择根序上不比 R 小——新实例第一次发布的 txg 是环里与记录里见过的最大 txg 加一（`crates/singlefs-core/src/mount.rs:1337`）。R 被选中，说明 j' 的根这一刻全读不出。要这样，需要：M2 那次挂载读不出 P 与 R（落回更旧的 T、取号 j'），j' 的根现在读不出，R 的实例表读不出（三样故障，两次读故障之间还隔一次挂载）。超出单故障加一次断电。
- 即使走到：R 自己已是被抛弃的时间线上的根，今天的择根就选了它（择根不看别的实例的表，看的是被选中那条根的表）；F2 退到同实例的 P 不比「选 R」多出一类害处，也没有多丢已确认的写：j' 的写本来就读不出。今天的取法（退到恰好等于见证到的 Pm）在这一形里同样是被抛弃时间线上的一版。

推翻它的现象：找到一个同实例、txg 在区间里、而被某张读得出的表按行抛弃的根——即某张表里有它自己实例的行（违反 Y2.2 的前提）。

### Y2.4 结局变没变

- `:656`（`crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs:656`）：所选根是写行那次发布 Row（实例 2），区间里读得出的只有实例 1 的 P（第一轮攻方 `c751.out` 的 `readable_roots_strictly_between=[(1, 9)]`、`newest_same_instance_root_between=None`）。同实例的一条都没有，三处结局一个字不变。
- C751 登记那一形（R、P、Pm 同实例 1，共用一片读回全零的实例表）：F2 交回的从 Pm 改成 P。恢复和可写挂载在 P 上照样读实例表，仍照报失败（和今天一样）；只读挂载打开 P（打开不读实例表）。所以「三处都能落 P」不成立，变的只有只读挂载那一处，从 Pm 变成已确认的 P，正是 C751 要修的越过 P。实测见 Y2.5。
- F2 其余格：表读得出的各格走另一臂，不动。表读不出、区间里没有同实例的根的格：不动。表读不出、见证到的那一条根读不出而区间里有同实例的根：今天 `newest_root_in_the_range` 取不到、判法交回 `None`、不退（照报失败），改后退到同实例的根，多挂上；这是改法带来的新结局，登记行没有，用例要钉。
- `skips_a_publish_whose_last_record_reads_while_its_root_does_not`（`crates/singlefs-core/src/recovery.rs:7003` 起的那一判）：区间下沿从 Pm 抬到同实例的最新根，要核的记录集合变小，不会比今天多拒。


### Y2.5 模型：同实例的一臂放进去，三处结局（副本上量过）

装置：`tests/rootring_r2_forward_y2_same_instance_f2.rs`（改自第一轮攻方的 `c751` 装置，判法部分重写）；核心层开关「同实例」在 `crates/singlefs-core/src/recovery.rs:6996` 那一臂（`tools/core-switch.patch`）。同一张盘面开关关、开各算一遍：F2 交回哪一版，恢复、只读挂载、可写挂载各落哪。**副本上的数，不是入库装置上的数。** 原样输出（k = 5 两行，k = 3 与 sw3 同形，全文在 `out/y2.out`）：

```
Y2 registered_c751 kind=ow k=5 | same_instance=false chosen=(1, 9) table_readable=false f2_target=Some((1, 7)) … rec=Fail ro=Ok(1,7) rw=Fail | same_instance=true chosen=(1, 9) table_readable=false f2_target=Some((1, 8)) … rec=Fail ro=Ok(1,8) rw=Fail
Y2 pinned_656 kind=ow k=5 | same_instance=false chosen=(2, 10) table_readable=false f2_target=Some((1, 8)) … rec=Ok(1,8) ro=Ok(1,8) rw=Fail | same_instance=true chosen=(2, 10) table_readable=false f2_target=Some((1, 8)) … rec=Ok(1,8) ro=Ok(1,8) rw=Fail
```

六行（ow / sw3 × k = 3、5 的登记那一形四行，`:656` 那一形 ow × k = 3、5 两行）：

- 登记那一形四行：F2 目标从 Pm 变成 P（k = 3：`(1,5)`→`(1,6)`，k = 5：`(1,7)`→`(1,8)`）；恢复 `Fail`→`Fail`；可写 `Fail`→`Fail`；只读 `Ok(Pm)`→`Ok(P)`。
- `:656` 那一形两行：目标、恢复、只读、可写开关开与关逐字相同。

三处的结局只有登记那一形的只读挂载变了，变成已确认的 P。第一轮攻方报告写的「三处都能落 P」不对：恢复与可写挂载在 P 上也读那片读回全零的实例表，仍报失败。

推翻它的现象：登记那一形或 `:656` 里任一处结局与上面不同；或另找到一张盘面，开关开比关多挂不上。

## Y3　C755 只统一落到哪一版

### Y3.1 「同一种失败」取什么

三样读者今天各自的失败：恢复看走读（`walk_to_file`，`crates/singlefs-core/src/recovery.rs:6806`；调用点 `crates/singlefs-core/src/recovery.rs:7115` 起），只读挂载看打开（`open_pool_for_read`，`crates/singlefs-core/src/mounted_read.rs:359`；调用点 `crates/singlefs-core/src/mounted_read.rs:655`），可写挂载看建上一版（`rebuild_version`，`crates/singlefs-core/src/recovery.rs:6828`，加沿链读实例表；调用点 `crates/singlefs-core/src/mount.rs:8000`）。三样读的单元不同、对数据单元的口径不同（建上一版容下读不出的数据单元，D19 已定项 5 的 N2，`crates/singlefs-core/src/recovery.rs:2957`）。

取法：**并集只用来判退**。对所选根，三样读者各跑一遍；有一样不通，就算「这一版建不出」，交给 F2 判法去找退到哪一版（判法原文不变）。**不用来判成败**：落到哪一版之后，恢复看走读、只读挂载看打开、可写挂载看建上一版，成不成各自照今天。再加一条保底：退到的那一版自己的读者读不通、而所选那一版自己的读者读得通，留在所选那一版。

为什么只能是并集：设每处挂载 m 今天成不成取决于「自己的读者在所选根上通不通、不通就看退到的那一版」。要一处挂载「今天能挂的，改后还能挂」（保底在），触发就必须在每一处挂载自己的读者不通时都触发，即触发是三样不通的并集。单取走读、打开、建上一版中的一样，或三样都不通才算（交集），在「另两样自己不通、而这一样通」的盘面上，今天靠退回挂得上的那一处会挂不上（下面表里量了五种取法；走读在那 100 格里处处不通，所以走读与并集数相同，条文取并集的理由在表后的读法里）。一句话的证明（保底在时）：今天 m 成功，要么 `own_m(R)` 为真，要么 `own_m(R)` 为假且 F2 有目标 P 且 `own_m(P)` 为真。并集触发下：`own_m(R)` 为真时，不触发则落 R 成功；触发则 `own_m(P)` 真落 P 成功，假则保底留 R 成功。`own_m(R)` 为假时必触发（并集），F2 有目标且 `own_m(P)` 真落 P 成功，否则同今天失败。每一格改后成功 ⊇ 今天成功。

### Y3.2 条文（D23 已定项 14 F2 段，`.claude/kb/decisions/23-journal的角色与格式.md:398`）

现文「恢复、可写挂载读阶段与只读挂载择根三处用同一套逻辑，退到 [见证到的那一版, 所选根) 里……」改成：

> 恢复、可写挂载读阶段与只读挂载择根三处用同一种失败判退不退、退到哪一版：所选那一版的走读、打开、建上一版三样里有一样读不出、解不开、判不过，且系统配置没见证它，就退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（判法原文不变）。这个并集只用来判退，不用来判成败：落到哪一版之后，恢复看走读、只读挂载看打开、可写挂载看建上一版，成不成各自照今天；退到的那一版自己的读者读不通而所选那一版自己的读者读得通时，留在所选那一版。三处交给判法的记录都在触发之后重扫一遍 journal 环。

### Y3.3 改动点（补丁前后）

| 位置 | 改法 |
|---|---|
| F2 判法函数（`crates/singlefs-core/src/recovery.rs:6893`） | 拆成前后两半：前半（所选版本是不是施加记录重建的、c_见证、见证过没有，`recovery.rs` 函数体开头到 `is_witnessed` 判完）不读单元，只读每块盘两个系统配置槽；后半（`witnessed_publish` 起，含 Y2 的一行）不动 |
| 新函数（放在能同时引用 `recovery` 与 `mounted_read` 的地方，不放 `recovery.rs`，`mounted_read.rs` 本来就引 `recovery.rs`） | `readers_agree_the_version_builds(reader, root) -> (走读, 打开, 建上一版)`，三样各跑一遍，`TreeTableHasNoEntries` 不算失败（与 `crates/singlefs-core/src/mounted_read.rs:648` 同） |
| 恢复（`crates/singlefs-core/src/recovery.rs:7100` 起的 `match`） | 先算前半；已见证则照今天；没见证则走读成功后再跑另两样，任一样不通进后半 |
| 只读挂载（`crates/singlefs-core/src/mounted_read.rs:640` 起） | 同上：打开成功后若没见证，再跑走读与建上一版 |
| 可写挂载（`crates/singlefs-core/src/mount.rs:8000`–`8072` 一带，补丁后后移，按函数 `mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread` 认） | 同上：建上一版成功后若没见证，再跑走读与打开 |
| 三处给后半的记录 | 触发之后各自 `scan_journal` 一遍：恢复今天就这么做（`crates/singlefs-core/src/recovery.rs:7124`）；只读挂载今天用挂载开头那一份（`crates/singlefs-core/src/mounted_read.rs:628`、`:662`）；可写挂载今天用读阶段那一份（`mount.rs` 的 F2 调用点，`&records`）。补丁（`crates.patch` 第 652 行起）已在退回之后、F1 再判之前加了一次重扫，这一次重扫可以和 F2 前的那一次合并成一次 |
| GPU 判官（`crash_judge_recovery.wgsl:11` 的 `recover`） | 无读故障的崩溃态上并集等于走读，不用动；将来崩溃枚举注读故障时，打开与建上一版多读的几类要搬过去（第一轮正推 X3.6，200 行以上 WGSL，推的） |

常态路径成本：见证过的（几乎所有挂载）只多算前半，4 个系统配置槽的读；恢复与只读挂载本来就要读系统配置，可写挂载读阶段本来就读（`newer_publish_witness`）；没见证的才跑另外两样读者（量级等于一次可写挂载的建上一版，推的，没量）。

### Y3.4 第一轮攻方那 100 格（50 格已确认、50 格没见证）在这个形态下

装置：`research/prompts/m3-close-rootring-r2-sonnet-model/tests/rootring_r2_forward_y3_y5_trigger_sweep.rs`，盘面照第一轮攻方 `union.out` 那 100 格（覆盖写 / 顺序写三个数据单元 × k = 3、5 × 已确认 / 没见证 × R 写出的每个单元与共用的实例表）、记录那 4 格、C751 登记那 4 格。每格先真挂载跑今天的恢复、只读挂载、可写挂载；再在测试里按五种触发取法算统一之后每处落哪一版、成不成：退到的那一版取同一份 F2 判法交回的，成败用各处自己的读者（走读 / 打开 / 建上一版）在那一版上真读。校准：「各处取自己的读者」的模型结局与真挂载逐格相等，100 格、记录 4 格、登记 4 格共 108 格，0 格对不上。**副本上的数，不是入库装置上的数**，单线程跑（两个测试共用一个全局开关，并行跑第一次有串扰，已改单线程重跑）。另一条独立的路是 `tools/derive_y3.py`：只用 `union.out` 里「所选根上三样读者通不通」和一条假设（退到的那一版的读者在 R 自己写的单元出故障的格里全通，共用实例表那 4 格里走读与建上一版不通）重算，数与真读者的模型相同（并集 0 格挂不上、42 处换版；打开与交集 52 处 30 格；建上一版 8 处 8 格）。

已确认 50 格：F2 判法前半判见证过、不退，五种取法结局都等于今天，没有一格比今天少挂上，也没有一格换版。

没见证 50 格（每个格 3 处挂载，共 150 处）：

| 触发取法 | 三处都成且落同一版 | 今天成、改后不成的挂载 | 改后换了版的挂载（成→成） | 成的几处落同一版（含有处报失败） |
|---|---|---|---|---|
| 今天（各取自己的读者） | 16 格 | — | — | 20 格（16 格三处同版成功，4 格共用实例表：恢复与可写报失败，只读落 R） |
| 走读 | 46 格 | 0 | 42（只读 34、可写 8，R→P） | 50 格 |
| 并集（三样有一样不通） | 46 格 | 0 | 42 | 50 格 |
| 打开 | 16 格 | 52 处（30 格） | 0 | 50 格 |
| 建上一版 | 38 格 | 8 处（8 格，全是恢复） | 26 | 50 格 |
| 交集（三样都不通） | 16 格 | 52 处（30 格） | 0 | 50 格 |

「成的几处落同一版」这一列五种取法都是 50 格，分不出取法，分得出的是前三列。数的来源：「今天成、改后不成」两条路相同（真读者的模型：并集 0、走读 0、打开 52 处 30 格、建上一版 8 处 8 格、交集 52 处 30 格）；「三处都成且落同一版」与「换版（成→成）」两列只有 `derive_y3.py` 算过（模型里的 `mounts_on_another_version` 把成→败也算换版，口径不同：并集 42、走读 42、建上一版 34 = 26 + 8、打开与交集 52 = 52 + 0，两边对得上）。宽严两种退法（退到的那一版读不通时留在所选那一版 / 照报失败）在这 100 格里每一格都一样。

读法：

- 没有一格比今天少挂上，只有走读、并集这两种取法做到；打开、交集让恢复与可写挂载在 30 格里挂不上（它们今天靠退回挂上，改后触发不了），建上一版在 8 格里让恢复挂不上。
- 这 100 格里走读与并集给出相同的数，因为每个故障都落在 R 的走读读得到的单元上（`readers` 的走读一列 100 格全是 false）。并集比走读多出的那部分要靠别的故障集才量得到，比如只落在墓碑、谱系、退役树容器或别的文件数据单元上的故障（那里建上一版读、走读不读）；这是为什么条文取并集而不取走读：走读 ⊂ 并集，取走读时可写挂载在这类故障上不触发，今天靠退回挂上的可写挂载挂不上。这类故障我没造盘面，**推的**。
- 单故障下「退到的那一版也读不通」只发生在共用单元的那几格（实例表）。保底（留在所选那一版）与不保底（照报所选那一版的失败）在这 100 格里结局相同；它们只在双故障下分叉，所以保底是对「今天能挂的改后还能挂」这条性质的保险，不是单故障下买到的东西。
- 三处都成且落同一版从 16 格涨到 46 格（另 4 格共用实例表，三处落点同为 P，恢复与可写照旧报失败）；换版的 42 处挂载全是只读（34）与可写（8）从未见证的 R 退到已确认的 P：F2 判法在 R 没见证时本来就许可退回（D23 已定项 14 的「所选根走读失败、系统配置没见证它时退回」），恢复今天就这么做；只读挂载与可写挂载在这些格里读不到 R 里那个出了故障的单元所属的内容，改后读到的是 P，内容完整、版本旧一格。
- D19 已定项 5（读不出的数据单元「挂载照常」，`.claude/kb/decisions/19-块指针的结构与宽度预算.md:115`）与 D23 已定项 14 的两处「只读挂载照常」（`.claude/kb/decisions/23-journal的角色与格式.md:396`）：形态只改版本，不改成败。已确认的格（第一轮攻方说的 34 / 50 格）一个字不变，所以这两句没有被撞。

### Y3.5 与 C754「重扫」：三处给判法的记录是不是同一份

今天不是同一份：恢复在走读失败那一刻重扫（`crates/singlefs-core/src/recovery.rs:7124`），只读挂载传挂载开头扫的（`crates/singlefs-core/src/mounted_read.rs:628` 扫、`:662` 传），可写挂载传读阶段的（`mount.rs` F2 调用点）；补丁（`crates.patch` 第 652 行起）在 F2 退回之后、F1 再判之前加了一次重扫，F2 自己的输入没动。形态把三处统一成「触发之后重扫一遍交给判法」（Y3.2 末句），于是三处给 F2 判法的是同一种来源的记录；补丁里那次 F1 再判之前的重扫保留，两次可以合并成一次。同一份盘面、无读故障时三处扫出的一样；有瞬时读故障时各扫一遍结果可以不同（不同挂载各自扫），这一点今天也有，形态不改。

## Y4　C773 另记钉槽、checker 现算求等

### Y4.1 钉住的槽在实现里是什么（先读码，条文才写得准）

挂载时读坏或读回全零的槽钉在「它可能住过的最旧 txg」：`txgs_ring_slots_bad_at_mount_can_hold`（`crates/singlefs-core/src/mount.rs:2857`）里 `for txg in 0..first_txg.0.min(ring_length_in_txgs)` 取第一个落在这个槽上的 txg（`crates/singlefs-core/src/mount.rs:2880`）。0 到 3S 之内 `target_for_publish` 是 `(txg mod 3, txg div 3)`，所以一个槽 (区域 r, 槽 s) 的钉值恒为 `3·s + r`，与槽一一对应，S 最大 16（`crates/singlefs-format/src/lib.rs:300`）、最小 4（`crates/singlefs-format/src/lib.rs:295`），钉值小于 48。钉着的槽在两件事上解开：这个进程写掉那一槽（`record_root_written_by_this_process` 里 `occupants.insert(ring_slot, ValidRoot{..})`，`crates/singlefs-core/src/allocator.rs:2591`），或进程内记根时它成了环里最小的那个、重读读出有效根或被抛弃的根（`crates/singlefs-core/src/allocator.rs:1685` 的 `oldest_txg_in_the_ring_rereading_slots_bad_at_mount`）。回收门槛 = `reclaim_floor(F, 环里最旧有效根并上钉值)`（挂载时 `crates/singlefs-core/src/mount.rs:3178`，进程内 `crates/singlefs-core/src/allocator.rs:2610`）。

所以「钉着的槽」的集合是一个进程内的、由读故障决定的动态集合：挂载时一部分，之后只会减（写掉、重读解开），不会增。读故障不留在盘上，镜像里看不到它。

### Y4.2 字段（要写进 kb 的字）

根记录新加两项，放在分配记录树根指针之后、算法类型之前（偏移 428 起；偏移 428 之前的字段一个不动，校验和偏移 138 与 `record[342..428]` 这类切片不变；`crates/singlefs-core/src/root_record.rs:13`、`crates/singlefs-core/src/root_record.rs:89`）：

| 字段 | 宽 | 偏移 | 值 |
|---|---|---|---|
| 回收门槛 t | 8 字节，小端 | 428 | 这条根指着的那一版记账建成时最后一次回收用的门槛，即 `max(这条根自己带的 F, 环里最旧有效根并上钉值)`；零单元发布抄上一条根的；mkfs 第 0 代为 0；由记录施加出来的版本取记录新根段里的 |
| 钉槽 p | 1 字节 | 436 | 建账时还钉着的槽里钉值最小的那个（`3·槽 + 区域`，小于 48）；没有钉着的槽写 255；零单元发布抄上一条根的；mkfs 第 0 代写 255 |

算法类型移到 437、nonce 438、MAC 450、合计 466（`crates/singlefs-format/src/lib.rs:234` 现 457，加 9）；512 字节槽内余量 46。journal 记录头新根段末尾（卸载记号之后）同样加这 9 字节，新根段 364 到 373（`crates/singlefs-format/src/lib.rs:244`），记录头 487 到 496（`crates/singlefs-format/src/lib.rs:247`），512 扇区内余 16，点名项个数 ⌊(4096 − 496) ÷ 56⌋ = 64 不变。每次发布的写盘次数、字节数不变（两处写调用的长度都是整槽、整记录），崩溃状态数不变。

p 为什么一个字节够：钉值与槽一一对应，p 同时告诉了哪个槽和钉在哪个 txg。为什么只记最小的那一个：回收门槛只取 `min`，钉着的别的槽不比 p 更小，它们在门槛里没有作用；它们钉得对不对，要到它们成了最小的那一刻才有后果，到那时它就是 p。这一条是我的推，Y4.5 第一行是它的会红检查。

采样点：带单元的发布，在 `record_root_written_by_this_process`（`crates/singlefs-core/src/allocator.rs:2591`）取完 `reclaim_floor` 之后（`crates/singlefs-core/src/allocator.rs:2610`），同一处把 `min(钉值)` 一起交回给装根的人；这一个调用本来就在装记账行与根之前（带单元的发布）。F 取这条根自己带的那一个，不取 `occupancy.effective_floor`：后者是挂载时的 F，抬 F 生效之后不跟着抬（`allocator.rs` 里 `RootRingOccupancy` 的 `effective_floor` 注释），抬 F 自己已经回收到新 F，写进字段的门槛必须是 `max(新 F, …)`，否则 checker 现算的（取 F_生效）会比字段高。

### Y4.3 checker 现算与求等（三条，编号待主 agent 认；`I-7.18`、`I-7.19` 在 `.claude/kb` 与 `crates` 里零命中）

> I-7.18 | 盘上回收门槛等于现算 | 最新根（最新一版由记录施加出来的取那条记录新根段）带 t 与 p。p 不是 255 时先要 p < 3S 并能还原成一个槽（`target_for_publish(p)`），否则判红。现算值 `t* = max(F_生效, min(环里按最新根的实例表仍有效的根的最小 txg, p))`（p 为 255 时第二项只取前者；环里一条有效根都没有时取 F_生效）。这一版的账是这条根自己那次发布建的（树表指针与 txg 小一的那条根的不同；那条根在环里找不到时当不是）时 t 必须等于 t*；否则（零单元发布抄来的，或由记录施加出来的版本）只要求 t 不高于 t*。不等时报「盘上门槛 t、现算 t*、p、环里最旧有效根、F_生效」五个数。

> I-7.19 | 钉着的槽不住着本实例写的根 | 这一版的账是这条根自己那次发布建的、p 不是 255 时，槽 `target_for_publish(p)` 在镜像里若是一条自证过的根，它的实例代号不得等于最新根的实例代号。同一个实例的根都是同一个进程写的，写掉那一槽的进程在写的那一刻已经解开了它的钉。零单元发布抄来的 p 属于上一个进程，不判（Y4.6）。

> I-3.1 的改动见 Y4.4。

不等时该报哪一条：t 与 t* 不等报 I-7.18；p 还原出的槽住着本实例的根报 I-7.19；记账与遍历不等报 I-3.1（沿用）。三条各自的红，坏镜像分别造（第一轮攻方的做法）。

### Y4.4 I-3.1 要改什么，C773 与 C684 各是什么结局

只加字段、让 checker 现算门槛，修不了 C773 登记行那一形。那一形的 44 槽释放代全在 (21, 25]（`.claude/kb/checks-owed.md:523`），而 checker 看到的环里最旧有效根是 25：释放代 g 的槽被 txg 小于 g 的版本引用，txg ≤ 24 的根已不在环里，所以这 44 槽没有任何有效根引用（推的，复现盘面 `i31-abandon-investigation` 已不在，复核不了）。现行 I-3.1 的丁-defer 只并「F_生效 之下、没被抛弃的根引用着（转述）」的 defer 槽（`crates/singlefs-checker/src/walk.rs:8338` 一带），没有引用就无从并起，门槛换成 21 也一样红。要转绿，I-3.1 要多并一类：释放代大于盘上门槛 t、环里没有任何根（有效或被抛弃）引用的已释放槽。

这就把 C684 摆到了岔口上。C684 那一形（`.claude/kb/checks-owed.md:447`，现文「读法甲：照现行原式，这一形归读不出的根那一类已知形态去判，测试钉成例外」）里那一槽只剩被抛弃根引用，用户 2026-10-20 定「不并，记成已认的代价」。两种写法：

| 写法 | C773 那一形 | C684 那一形 | 与用户「不并」 |
|---|---|---|---|
| 并：释放代大于 t 的已释放槽一律并，不看引用（第一轮攻方 X4、正推 X4 的改式） | 绿 | 绿（那一槽释放代大于 t，被并进来） | 相反：替 C684 选了「并」 |
| 折中：只并环里没有任何根引用的；被抛弃根引用着的不并 | 绿 | 红（照旧，测试钉成例外） | 一致 |

这一轮的形态（C684 不变）只有折中成立。折中的代价：checker 要走被抛弃根（现在 checker 不走它们），取它们的引用集合当排除项；这和「被抛弃的根也并」那一式的工作量相当，只是并集变成了排除集。折中是我从用户这一支推出来的，没有被攻过：被抛弃根独占引用、又被钉扣着没回收的槽（单次读故障加早先一次崩溃恢复落旧根，两次故障横跨两次挂载，超出单故障加一次断电）在折中下仍红，这一类我没有造盘面验；折中里「被抛弃根引用的不并」对它是误红还是该红，条文没说，规则没说，要主 agent 或用户定。

C773 转绿还靠一条前提：写者写下的 t 与 p 如实。下面是不如实的情形。

### Y4.5 坏实现判不判得出

| 坏实现 | I-7.18（t 等于现算） | I-7.19（钉槽不住本实例的根） | I-3.1 | 判出 |
|---|---|---|---|---|
| 钉低不解开：写掉了钉着的槽，p 仍写原值 | t 与 p 一致，现算等于 t，不红 | 那一槽现在住着本实例写的根，红 | 折中后门槛偏低，多并，不红 | 是，靠 I-7.19 |
| t 写成固定的 F（忘了环里最旧有效根） | t 不等于 t*，红 | — | 可能红 | 是，靠 I-7.18 |
| t 写得比建账用的高（采样点选在回收之后） | 红 | — | 少并，红 | 是 |
| p 写 255 但其实还钉着槽（漏记钉） | t 低于 t*（t* 没有钉值，更高），红 | — | 红 | 是 |
| 钉着的槽重读读出了有效根，写者没解钉，p 仍是它 | 不红（t 与 p 一致） | 不红（那一槽住着前一个实例或更早写的根） | 不红 | **否**：读故障不留在盘上，「钉着」与「重读解开了」在镜像上逐字节相同 |
| 凭空钉一个本来读得出的槽 | 不红 | 不红（那一槽没被本实例写掉） | 不红 | **否**：同上 |

最后两行是这个形态判不出的，且只有一个方向的害处：门槛偏低、空间少回收一截。害处有上界：钉在这个进程写掉那一槽时必解（Y4.2），一个进程 3S 次发布之内每个槽都被写掉，p 在那之后只能是 255，否则 I-7.19 红。所以「钉低不解开」最多漏回收 3S 个 txg 的一截，到被写掉那一槽就被抓。

模型：没有（Y4 全是读码加算术）。推翻它的现象：找到合法历史上 t 不等于 t*（零单元抄来的除外），I-7.18 误红；或找到本实例写下的根住在 p 还原出的槽上而写者的钉确实还在（那说明「写掉就解」不成立）。

### Y4.6 跨挂载、回退、取号之后怎么延续

- 新挂载：钉着的槽集合从读根环重新算（`mount.rs` 的重建分配器），旧根上的 t、p 描述的是旧进程，core 不读它们去建状态。core 只在一处读它们：施加记录拼根时，把记录新根段里的 t、p 原样带进拼出来的根（`crates/singlefs-core/src/recovery.rs:4624`、`:4648` 两处，第一轮报告 X4.1）。
- 回退（F2 退、管理员回退）：F2 退到的是一条已有的根，t、p 是它当年的；管理员回退是向前发布，带单元，按采样点现算。
- 取号之后：写行那次发布与暖机是零单元发布，t、p 抄所选那一版的，那是上一个进程的钉；第一次带单元的发布起现算。所以 I-7.19 只判建账的发布（与 I-7.18 求等同一个条件）：零单元抄来的 p 可能指着一个槽，而这个进程的写行或暖机恰好写掉了它，照判会在合法盘面上红。
- 建账的发布才求等的来由：账是这次发布建的，t 的含义是建账时最后一次回收用的门槛。零单元发布的账是抄来的，所以 t 也抄，checker 对它只判 `t ≤ t*`（环只往上走，F 只增，钉只减）。最新一条根常是零单元发布（暖机、卸载那一串），那时求等落空，只剩单边；要在卸载过的池上也求等，得往回找最近一条建账的根，而它盖过的槽已不在环里，现算不出当时的环。
- I-7.19 自己的一格误红：挂载时读坏钉着的槽，这个进程对它的一次根槽写报失败、实际落了盘（分配器整个换回发布之前，那一槽照旧钉着，`crates/singlefs-core/src/allocator.rs:2570` 一带的调用时机说明），之后建账的根的 p 还指着它，而镜像上它住着本实例的根。读故障加写失败两样故障，超出单故障。

## Y5　四个形态一起

### Y5.1 Y2 的「同实例」与 Y3 的统一判法放在一起：`:656` 与第一轮攻方那 100 格

两个形态改的是 F2 的两个不同位置：Y3 改什么时候去找退到哪一版（触发，在三处调用点与 F2 前半），Y2 改表读不出时认哪些根（F2 后半的一臂，`crates/singlefs-core/src/recovery.rs:6996`）。次序写死：见证过没有（Y3 的前半）→ 三样读者有没有不通（Y3 的并集）→ 找目标（Y2 的一臂）→ 目标自己的读者通不通（保底）→ 跳过一次根读不出的发布的核（`crates/singlefs-core/src/recovery.rs:7003`）。

副本上量过（开关：同实例开 / 关；触发：五种取法）：

- 第一轮攻方那 100 格与记录那 4 格：同实例开、关两种下恢复、只读挂载、可写挂载的真挂载结局逐格相同（模型对每格都两种开关各真跑一遍，计数 `same_instance_switch_changes_a_real_outcome` 已确认 0、没见证 0、记录 0）。原因在盘面：没见证那 50 格里见证到的就是 P，区间 [P, R) 里只有 P，P 与 R 同实例，本来就是「恰好等于见证到的那一版」。所以 Y3 的表（Y3.4）就是两个形态一起的结局：并集取法 0 格少挂上，三处都成且同版 46 格，换版 42 处。
- `:656` 那一形（`c751.out` 里同名两格，k = 3、5）：Y2 开、关的恢复、只读挂载、可写挂载结局一个字不变（恢复 `Ok(1,6)` / `Ok(1,8)`，只读同，可写报失败，k = 3、5 各一）。Y3 在这一形上不改结局：恢复与只读今天就因为走读、打开不通而退，退到的目标不变；可写挂载今天在读阶段（最新根的实例表读不出，取号之前拒）就拒了，早于 F2（读码推的，副本上的真挂载结局印证是 `Fail`）。
- C751 登记那一形（k = 3、5 各两种写法，4 格）：恢复 `Fail`→`Fail`，可写 `Fail`→`Fail`，只读从 `Ok(Pm)` 变 `Ok(P)`（k = 3：`Ok(1,5)`→`Ok(1,6)`；k = 5：`Ok(1,7)`→`Ok(1,8)`），4 格里开关开、关的真挂载结局都不同（计数 4），不同的都只在只读那一处。五种触发取法在这 4 格上结局相同（走读、打开、建上一版在 R 上全不通，哪种取法都触发）。

### Y5.2 Y1 按自证不过处置的根是否改变 Y4 的「环里最旧有效根」

分配器一侧会变，checker 一侧不变，字段把两边接上：

- 分配器：错位读出的根归读坏，挂载时钉在这一槽的钉值 `3·槽 + 区域`（Y4.1）；今天它被记成别处一条根的 txg，真根掉出表，门槛可以越过真根（C743 的形）。改后门槛只会更低或相等。
- checker：读镜像，镜像上没有读错位（读故障不留在盘上），它的「环里最旧有效根」不变。
- 接口：钉住的槽进 Y4 的 p，所以 I-7.18 的现算 `t* = max(F_生效, min(环里最旧有效根, p))` 在读错位出现的历史上仍等于分配器写下的 t。没有 p 时（只有 t）这一格会红：分配器的 t 低于 checker 能算出来的值，只能靠单边「t 不高于」放行。
- 相反的方向：Y1 把读错位收进失败模型，等于给「钉」多了一个来源（读故障之外再加读错位）。Y4 没做时，I-3.1 在钉住期间红的那一类合法盘面（C773）随之变多；Y4 做了，由 p 承接。所以 Y1 与 Y4 不是可以单独上其一的两笔：只上 Y1，C773 登记行那一类红的发生面变宽。

### Y5.3 其余两两关系

- Y2 与 Y4：F2 退到 P 之后取号，写行那次发布抄 P 的 t、p（Y4.6），不会抄 R 的。
- Y3 与 Y4：Y3 多读的单元只在没见证时发生，不改回收门槛，不碰 t、p。
- Y1 与 Y2、Y3：Y1 让 F2 的 `readable_roots` 少读到被拒的根，Y2 的区间更难成立，退回更难发生；Y3 的触发不依赖 `readable_roots`。
- 一格需要主 agent 注意的：Y3 并集触发让恢复与只读挂载在没见证时多读，D23 已定项 14 的代价句（`.claude/kb/decisions/23-journal的角色与格式.md:423`，「已确认的版本在较新的系统配置槽与它的单元同时读回全 0 时，恢复、可写挂载与只读挂载悄悄读回上一版」）的适用面不变，仍是已确认的版本被读回上一版；Y2 把「上一版」从 Pm 改成同实例里更新的 P，这一句里「上一版」的所指要跟着改。

## 要主 agent 跑的命令

- 复跑模型（我在副本里跑过，下面是入库后的复跑；整条经内存包装，线程取 31）：

```
cd /home/fy5090/code/singlefs && bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m3-close-rootring-r2-sonnet-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-rootring-r2-sonnet/frozen --work <新的空草稿目录>
```

  `--frozen` 是开工时 `crates/` 的拷贝（未打开关补丁，与主工作区 `diff -rq` 无差别）。输出看 `name=rerun_compare` 四行；`out/` 里的 `y1.out`、`y2.out`、`y3.out` 是我的原始输出，复跑时逐行比。
- 我没跑（要虚机、要重型或不在这一轮）：E165 读槽对拍加核之后重跑（重型，第一轮报告 X1.7）；GPU 判官与 CPU 判官对拍（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`，Y2 的 WGSL 一行与 Y3、Y4 都动它）；整份 harness 测试在开关 1、2 下哪些用例变红（第一轮点名的三个手摆根用例我只读码判了，没跑）。

## 判定一览（末尾版）

| 格 | 判定 | 一句话 |
|---|---|---|
| Y1 | 一致；冲突 1 处（D22 实例代号行的平局括号与已定项 16 的公式句）；规则没说 1 处（例外放行的读错位盲区写不写进条文） | 例外 = txg 为 0 且槽号为 0，与 I-7.17、mkfs 种根的槽同一个集合；122 张合法镜像 2094 个根，例外拒 0、裸公式拒 19（全是年轻环的第 0 代根）；三处结局两者都与今天相同 |
| Y2 | 一致 | 同实例的根按表恒不被抛弃（行只写给更旧的实例）；C751 登记那一形只读从 Pm 变 P，`:656` 一个字不变；找不到单故障加一次断电内同实例的根已被抛弃的一形 |
| Y3 | 一致；规则没说 2 处（保底写不写；GPU 判官不搬并集） | 取并集只判退、成败各看各、退不成留在所选版：100 格 0 处挂不上，三处都成且同版 16→46 格，换版 42 处（只读 34、可写 8，R→P，R 没见证）；单取打开 / 交集挂不上 52 处，单取建上一版 8 处；三处给判法的记录改成触发之后各重扫 |
| Y4 | 冲突 1 处（字段加求等修不了 C773 那一形；要修，I-3.1 得多并一类，写法决定 C684 红绿）；规则没说 2 处（折中写法没被攻过；被抛弃根独占又被钉扣着的槽） | 字段 t 8 字节 + p 1 字节，根记录 457→466，journal 记录头 487→496，写盘 0 次增加；p 为最小钉值，既认槽又认钉值；I-7.18 求等（建账的发布）、I-7.19 钉槽不住本实例的根；坏实现里「钉低不解开」判得出，重读解开了而没解钉、凭空钉判不出 |
| Y5 | 一致；冲突 0 处；耦合 1 处（Y1 单上会让 C773 红的发生面变宽，需 Y4） | Y2 与 Y3 改 F2 的两个位置，100 格与 `:656` 结局不变，只有 C751 登记那一形的只读挂载变（Pm→P）；Y1 让分配器的最旧有效根只低不高，checker 一侧不变，靠 p 接上 |

## 没做什么

- 没替主 agent 采纳或出判决；Y4 折中写法、Y3 保底、Y1 盲区要不要写进条文，都是候选。
- 没改 kb、没改 `crates/` 主工作区；条文是拟稿，落地归写回批与实现员。模型里的核心层补丁只在草稿目录的仓副本和 `tools/core-switch.patch` 里。
- 没打系统配置落地补丁（补丁只读，改动点写明补丁前后）；Y1 的四处解析点、Y2 的一行、Y3 的 F2 拆分都不在补丁改的范围，Y3 的可写挂载调用点补丁后行号后移，按函数名认。
- 没为 Y4 写模型：宽度、采样点、三条求等规则、折中写法都是读码加算术；`i31-abandon-investigation` 复现盘面不在，C773 那 44 槽「无任何根引用」是推的；折中写法与 C684 的结局没造盘面。
- 没量分配器一侧的内部数（裸公式把第 0 代根当读坏之后准入抬 F 的上限、回收门槛）；Y1 的模型量的是三处挂载结局与逐槽的拒收。
- Y3 的「退到的那一版读不通」在单故障下只出现在共用单元的 4 格，保底与不保底的差别只在双故障，没造双故障盘面。
- Y3 的并集比走读多出的那部分（只落在墓碑、谱系、退役树容器或别的文件数据单元上的故障）没造盘面；那是推的。
- 没跑整份 harness 测试，没判哪些旧用例变红（第一轮点了三个手摆根用例）；没跑 E165、GPU 对拍、门禁、变异。
- 草稿目录里排进内存包装队列的三条编译或运行命令，两条已跑完或自行结束；交回前还剩一条（`--no-run` 编译，`logs/build1.log`）在队里，我用 proc.py 停了它的一条子进程（3446786），它随即结束，没有留下在跑的进程；结论都不靠它，三份测试的运行结果来自另两条命令（`logs/run1.log`、`logs/run2.log`）。
- 没读禁读清单里的文件（`research/prompts/m3-close-rootring-r2-opus-output.md`、`research/prompts/m3-close-rootring-r2-opus-model/`、`~/.local/share/singlefs/work/m3-close-rootring-r2-opus/`）。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-close-rootring-r2-sonnet-output.md --root . --background research/prompts/_m3-close-rootring-r2-background.md` 的末行原样：

```
  ✓ 核了 4 处引文、91 处裸行号，对上 4 处，没判 91 处（其中裸行号没有引文可比 91 处；逐处列在上面）
```
