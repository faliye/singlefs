# m3-close-g4-rewrite-r2 攻方腿（Opus）报告（2026-10-22）

立场：攻第一轮的修补本身与产生结论的方法（正文分工表「云端攻方」那一行）。按 `crates/` 开工时的实现谈：开工时主工作区的四份代码与 `research/prompts/m3-close-g4-rewrite-r2-snapshot/code-sha256.txt` 逐份 OK，我从主工作区拷了一份冻结副本（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`）在草稿目录里编。量过的数都出自这份副本上的原型，没有在入库装置上跑过。没重复第一轮攻过的角度（V1 的 mkfs 末段、V4 的 C670 与 Q3 拒因位置）。

## 复跑

模型目录 `research/prompts/m3-close-g4-rewrite-r2-opus-model/`。原型是一个工作区成员 crate，只 `cargo build`，编出来的二进制直接执行。崩溃状态用 `singlefs_checker_tier::crash_state_enumeration::enumerate_crash_states_with_expansion` 加 `crash::full_expansion` 取（段内整写子集与原地覆写的第三态都展开），每个状态上崩溃点重放照常跑自己的恢复与池级 checker，另挂一个观察者。观察者把每个状态分成两个域：崩溃注入摆得出的「整写」（`whole`）与只有崩溃点重放才摆的「撕裂」（`torn`，持久集合里有撕裂镜像那一截）。`toggles.patch` 给核心层加三个只在副本里认的运行时开关：`SINGLEFS_PROTO_MUT_B1`（盘 0 写完新槽，同一段把另一槽清成全 0，与第一轮同一份）、`SINGLEFS_PROTO_MUT_B2`（取号两轮之间少一道屏障：每块盘两槽在同一段里各写一次，内容都自证得过）、`SINGLEFS_PROTO_GATE_NEVER_PASSES`（补写闸恒不过）。

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-g4-rewrite-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen <开工时那棵树（crates/、Cargo.toml、Cargo.lock、.cargo、.claude/heavy-thread-default）> --work <新的草稿目录>
```
每条流的计数写进 `<草稿目录>/out/<标签>.counts`，去掉 `history=` 一行与 `o2_reading` 一行之后与 `out/` 里同名文件比，打 `name=rerun_compare tag=... same=yes|no`。整趟约 20–30 分钟（本机负载 400 以上时单条流 40–200 秒），子 agent 的前台时限跑不完整趟，没有用 rerun.sh 整趟复跑过；`out/` 里每份是我逐条跑出来的原样（跑法见「这条腿自己的限度」）。

| 文件 | sha256 |
|---|---|
| Cargo.toml | f50aa534420cb9781c679b7ad8898570dc0fe2bf91c10b4f331764f1def3cbe3 |
| src/main.rs | 70f4c2b8acf6f5c69da66e962cb6f691442c7eb75c205cc8be35bad5b6a7105d |
| toggles.patch | 8c145a835bbc4688937ad9e5302a50d17be51c1432deacb7f4b5269cffab5e82 |
| tools/rerun.sh | 3db75ecc11966faff40bf49ba6c845484b84b0f09d9b4cab2a0fd1c4aa7e9f63 |
| out/remount.counts | b30953241c951f000a0fa86f848d417553ffb6ede3c25b45ce1d3e00ddb9118e |
| out/zeros_mount.counts | d45eb66e9df700b0a432dd0a313396f0537dcd8c2be76dd1eefa32819d18bf48 |
| out/cold_start_then_remount.counts | 05feb8ce498b1d40daf4da6e120502f7a64946fbbbe0f6d8642f0755600365ab |
| out/second_crash_base_0.counts | d3b15a9804c7c3a566489a4f1e2dbe376c0ab4c75974575f8bf54ffd1a69cf2a |
| out/second_crash_base_1.counts | 42aebe163ba7b624f16d3f726a50f5dbca74e86a3314c6790c2b89ae6d71f0c0 |
| out/second_crash_base_2.counts | 2a2dcd528501ef3344b9d980bb99e9126fc9d47b31f378a55228f11a2635e49d |
| out/second_crash_base_3.counts | 889be0d020069bbe7816e4ac670ac191f5639fb186ca29f35ed152933c6dc74f |
| out/second_crash_base_4.counts | 34f6059796eb91b31189bdb76f079ee3935c31c2eee59d7e4b7deeacaea2a1fc |
| out/second_crash_base_5.counts | a18fe8a01ca05dac2ff1633ae6ec4d4bd425359e9024321ccfaf8e8efe5660c3 |
| out/second_crash_base_6.counts | dc4da7fdef3441f534e569f1dbe95337dcec96848c669c8369c98497a3c97746 |
| out/second_crash_base_7.counts | b6a998f4a8a4f190615891d9f025d7b782efbc3bb4843cbae276bcee3d024959 |
| out/remount-SINGLEFS_PROTO_MUT_B1.counts | 061d8cbdc29e7293747d5d7bc1711db3367f47fad3fd3ef50b8dce0663fc83a7 |
| out/remount-SINGLEFS_PROTO_MUT_B1-tally.counts | a555e861af91ad679f53e3e02162a73f8cdf4b907159a53259c08defb1b74baa |
| out/remount-SINGLEFS_PROTO_MUT_B1-o2.counts | a555e861af91ad679f53e3e02162a73f8cdf4b907159a53259c08defb1b74baa |
| out/remount-SINGLEFS_PROTO_MUT_B1+SINGLEFS_PROTO_GATE_NEVER_PASSES.counts | 35b3073e59d795d9aa52ce7f94875f93aae119139ec4f34771336fd8ed5aa3de |
| out/remount-SINGLEFS_PROTO_MUT_B2.counts | beee4c868a83d4fc79b0c3891a0a93554cd8f46f71fc18389c862e4833961169 |
| out/remount-SINGLEFS_PROTO_MUT_B2-o2.counts | 924b8e9d3da0240529420995b20b374775ecca72c51a6ac1743aa3a0a7ca4689 |
| out/hand_made_image_both_slots-none.counts | 78ed454c582e08c62cb6fd15eaef4b3eb5cd5432ef16c0e89a58469afc6ddfdf |
| out/hand_made_image_both_slots-SINGLEFS_PROTO_GATE_NEVER_PASSES.counts | 91c3010a0a094f463bdff7a7eb94129a6355ba4f226bb6d22998cd849abc2bde |
| out/hand_made_image_both_slots_and_a_unit-none.counts | 66e5a07c6bb3e4cd1947c3031767a59dde3850200a3336f4fec36d2948f0bd15 |
| out/hand_made_image_both_slots_and_a_unit-SINGLEFS_PROTO_GATE_NEVER_PASSES.counts | 6f42335c726646c321fabf700aa32b4293a3a62a9e8ed7f4551a7433cf53f27f |

`-tally` 与 `-o2` 两份是同一条 MUT-B1 流用后两版原型重跑的（多打了崩溃点重放自己的判与池级 checker 那一路的对照），两份逐字节相同。

## 各格判定一览

| 格 | 判定 | 量的 / 推的 |
|---|---|---|
| W1 | **没打中（只抽了一次）**：正确实现在 mkfs 之后、没注入读故障的崩溃态上，G4-丙 一条都没判出。取样：关闭再可写挂载（取号两轮、写行、暖机）20519 态；读故障那一步（每块盘最新一槽读回全 0 的可写挂载）20519 态；冷启动再可写挂载 20519 态；第三截那一形（八个整写域的第一次崩溃镜像上各起一次可写挂载，再枚举这次挂载的全部二次崩溃）172366 态。判官读法与核心层读法两路在这些态上处处一致（池级 checker 那一路只在 MUT-B1、MUT-B2 两条流上对照过） | 量的（副本原型）；覆盖写、清扫两条流超过约 10⁶ 态没跑，回退流 86066 态超过前台时限交主 agent，抬 F 在这个起点上被拒、没写盘 |
| W2 | **打中三处**：① 挂在崩溃注入第二、三截的 G4-丙 只看得到整写子集（崩溃注入「撕裂并进没持久」），「同一块盘两槽同段」那一类坏实现（MUT-B2）的两槽全废态全在撕裂域：整写域 0 态，撕裂域 16 态单盘全废、1 态两盘全废；② 采 G4-丙 而留着 G4 拒绝那一臂，同一张盘面记两条：MUT-B1 加补写闸恒不过的 42 张拒绝盘面上两条都报，42 张都是；③ G4-丙「复用判官里已有的谓词」，判自证用的是核心层的 `SystemConfiguration::parse_slot` 与 `choose_system_configuration`，与被判的实现共用同一段算法 | ①② 量的；③ 读码，另用池级 checker 的独立读法在同一批态上对照过（量的，三路一致） |
| W3 | **打中两处**：① 甲-二 那条测试杀不了它自己要加的那一行变异「补写闸恒不过」：开关与变异是同一条路径，开着开关的测试在变异下照样绿；这一行变异按代码推今天已被 harness 档那条「两槽清零的盘过闸补写、挂上」的测试杀（同向的「挂得上」断言在变异下失败是量的，盘面是我手造的两槽各翻一字节；那条测试本身没跑）；② 判官在甲与甲-二 两张盘面上判出的完全相同，甲-二 在今天的判官与 G4-丙 下都不多罩任何东西。**没打中**：开关会不会变成第二条发布路径——补写闸恒不过只把补写变成取号之前的拒、一个字节不写，不是发布路径 | ① 杀与不杀是量的（盘面与断言），「同一条路径」是读码；② 量的；没打中的那条推的 |
| W4 | **打中**：D13 第 187 行改与不改都在一格说假话（不改：今天 G4 只在拒时判；改成「由 G4-丙 判（崩溃注入第二、三截）」：撕裂态那一半照样没人判，「与 C670 那条同接法」也不再成立）；D23 第 396 行不改，说的是补写例外之前的实现；改了而判官复算不跟，D13 第 72 行「照那一条的判据在崩溃镜像上自己复算成立」对这一拒因不再成立。另：正文写「新 C 号从 C873 起」，工作区此刻的 `checks-owed.md` 第 578 行已有 C873 | 条文与代码的对照是读码；「带读故障的档里没有两槽全废的可达盘面」那一句是量的（读故障那一步 20519 态，0 态） |

## 自己提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 丙-置：G4-丙 不挂在崩溃注入第二、三截，改挂成崩溃点重放的一个观察者（全量域，撕裂态在内），只判 mkfs 那几段之后的状态；判自证读池级 checker 的 `system_configuration_redundancy_report`（独立解析器） | W2 ①③ | 它在 MUT-B1、MUT-B2 上判得出几态是量的（MUT-B1 整写 8 + 撕裂 34；MUT-B2 撕裂 16，另 1 态两盘全废），池级 checker 那一路与判官读法逐态一致是量的；挂法与 mkfs 段的排除没实现，推的 |
| 丙-择：G4-丙 在没注入读故障、mkfs 之后的崩溃态上择不到系统配置也记一条 | W2 ①（MUT-B2 那 1 态两盘全废） | 正确实现的四条流 + 八个基线上一态都没有「择不到」是量的；记法推的 |
| 丙-撤：采 G4-丙 时撤掉 G4 拒绝那一臂，`crates/mutations.tsv` 第 2406–2408 行改指 G4-丙 | W2 ② | 两条同报 42/42 是量的；撤掉之后这些拒由 U6 判合法、靠 G4-丙 记一条是读码推的 |
| 不做甲-二；要判官那一半的反向测试，就拿手造盘面加一个照字段造出来的拒因直接调判官，不在核心层留开关 | W3 ①② | 推的 |
| 补写闸那五件分支进拒因（或进挂载的交回），运行时看得出走了哪一件 | W3（分支纪律第 4 条） | 推的 |
| D23 第 396 行带上补写例外（指到 D22 第 181 行），D13 第 72 行同时写明判官对这一拒因的复算不带补写闸、欠账一行 | W4 | 推的 |

## W1：G4-丙 在正确实现、mkfs 之后、没注入读故障的崩溃态上误不误报

**判定：没打中（只抽了一次）。**（量的，副本原型）下面每条流的每个崩溃状态上，用判官今天那一份读法（择系统配置、每块盘两槽各读一次、`parse_slot` 过且 fsid 等于择到的那一份才算本池自证过）数「一份都没有」的盘，一条都没数到；同一个状态上另用核心层 `verified_system_configuration_slots` 数一遍，两路有不同就记一行 `readings_differ`，这几条流里一行都没有。

| 历史类（正文 W1 那一格） | 流 | 崩溃状态（整写 / 撕裂） | G4-丙 判出 |
|---|---|---|---|
| 取号写两轮中途崩溃（第二截崩溃镜像那一形） | `remount`：第一个文件之后关闭、可写挂载（取号两轮、写行、暖机） | 20499 / 20 | 0 |
| 读故障那一步：每块盘自证过、世代号最大那一槽读回全 0 的可写挂载 | `zeros_mount` | 20499 / 20 | 0 |
| 用户动作放开：先冷启动恢复、再可写挂载 | `cold_start_then_remount` | 20499 / 20 | 0 |
| 二次崩溃（第三截那一形） | `second_crash_base_0`–`7`：`remount` 那段历史里四个系统配置槽写段（取号两轮、两次轮换）各摆「只落了盘 0 那一写」「只落了盘 1 那一写」两个整写域的第一次崩溃镜像，各起一次可写挂载，枚举这次挂载的全部崩溃状态 | 六个基线各 20499 / 20，两个基线各 24601 / 25，合计 172366 | 0 |
| 补写本身中途崩溃 | 走不到：上面全部第一次崩溃态与八个基线里没有一块盘两槽全废，补写闸一次都没被调到（基线标签 `slots_per_device=[2, 2]`）；第一轮量过的补写中途崩溃起点是 mkfs 末段，不在射程 | — | — |
| 清扫与覆盖写 | `overwrite` 1048588 态、`overwrite_losing_power_then_mount`（掉电在第一条记录、再可写挂载，挂载末尾清扫）1069094 态 | 超过约 10⁶，没跑 | — |
| 回退 | `remount_then_roll_back` 86066 态 | 子 agent 前台时限跑不完，命令交主 agent | — |
| 抬 F | `raise_floor_steps 1`–`4`（第一个文件之后抬 F） | 四种目标都被拒（`RollbackFloorAboveCeiling`），一个写都没发；先覆盖写再抬 F 的流超过约 10⁶ | — |

原样输出（`out/` 里的计数行，计数行之外的 `name=` 行只有历史与段形两行）：
    name=remount domain=torn states count=20
    name=remount domain=whole states count=20499
    name=zeros_mount domain=torn states count=20
    name=zeros_mount domain=whole states count=20499
    name=cold_start_then_remount domain=torn states count=20
    name=cold_start_then_remount domain=whole states count=20499
    name=second_crash_base_0 base=[segment=0 landed_device=0 slots_per_device=[2, 2]] first_mount=mounted
    name=second_crash_base_0 domain=torn states count=20
    name=second_crash_base_0 domain=whole states count=20499
    name=second_crash_base_1 base=[segment=0 landed_device=1 slots_per_device=[2, 2]] first_mount=mounted
    name=second_crash_base_1 domain=torn states count=20
    name=second_crash_base_1 domain=whole states count=20499
    name=second_crash_base_2 base=[segment=1 landed_device=0 slots_per_device=[2, 2]] first_mount=mounted
    name=second_crash_base_2 domain=torn states count=20
    name=second_crash_base_2 domain=whole states count=20499
    name=second_crash_base_3 base=[segment=1 landed_device=1 slots_per_device=[2, 2]] first_mount=mounted
    name=second_crash_base_3 domain=torn states count=20
    name=second_crash_base_3 domain=whole states count=20499
    name=second_crash_base_4 base=[segment=5 landed_device=0 slots_per_device=[2, 2]] first_mount=mounted
    name=second_crash_base_4 domain=torn states count=25
    name=second_crash_base_4 domain=whole states count=24601
    name=second_crash_base_5 base=[segment=5 landed_device=1 slots_per_device=[2, 2]] first_mount=mounted
    name=second_crash_base_5 domain=torn states count=25
    name=second_crash_base_5 domain=whole states count=24601
    name=second_crash_base_6 base=[segment=9 landed_device=0 slots_per_device=[2, 2]] first_mount=mounted
    name=second_crash_base_6 domain=torn states count=20
    name=second_crash_base_6 domain=whole states count=20499
    name=second_crash_base_7 base=[segment=9 landed_device=1 slots_per_device=[2, 2]] first_mount=mounted
    name=second_crash_base_7 domain=torn states count=20
    name=second_crash_base_7 domain=whole states count=20499

为什么一条都没有（推的，与上面量的相符）：核心层每一次系统配置槽写只写这块盘较旧的那一槽，世代号取这块盘自证过的最大值加一、槽号取世代号模 2；取号两轮之间、抬 F 那一写前后、回卷写之前、每次发布轮换之后都有一道池屏障（`crates/singlefs-core/src/transaction.rs` 里每一处都写了理由）。所以 mkfs 之后同一段里同一块盘至多一个系统配置写，整写子集里那一槽要么旧要么新、都自证得过，撕裂态里也只坏这一槽，另一槽是上一段已持久的。mkfs 把两槽都写成世代号 0（槽 1 的世代号与槽号奇偶不合），下一写落槽 1、覆写的是同为 0 的那一份，不丢唯一自证的那一槽。

要不要读补写闸：在正确实现的这些历史里 G4-丙 一次都没碰上两槽全废的盘，判据里带不带闸结论一样；带闸就得在判官里另写一份五件（`.claude/rules/fs-design.md`「运行时与 checker 必须用不同的算法」），这一份在这些取样里没有一态用得上（推的）。

什么会推翻「没打中」：同一套读法在下面任一处数出 G4-丙 判出的态——没跑的三条流（覆盖写、清扫、回退）、三块盘的池、别的几何与种子、抬 F 真写了盘的历史；或第二条同立场腿（`-s2`）在别的历史上造出来。

## W2：G4-丙 的接法，与 G4 拒绝那一臂、C670、Q3 同一张盘面上记几条

### 打中 W2-①（量的，分辨挂法）：挂在崩溃注入第二、三截，只看得到整写子集，「同一块盘两槽同段」那一类判不到

- 被判的那一句：`research/prompts/_m3-close-g4-rewrite-r2-body.md:13`：「挂在崩溃注入第二截的崩溃镜像与第三截的二次崩溃镜像上，各判一次」。
- 崩溃注入摆的域：`crates/singlefs-checker-tier/src/crash_injection.rs:5`：「更早的段整段持久、当前段任意真子集、更晚的段一个都没持久，撕裂并进」；第二截与第三截的镜像都只按录制流的写给持久位，没有撕裂镜像：`crates/singlefs-checker-tier/src/crash_injection.rs:1598`：「persisted: crash_point.persisted_within_the_segment.clone(),」，`crates/singlefs-checker-tier/src/crash_injection.rs:4286`：「persisted: second.persisted_within_the_segment.clone(),」。
- 崩溃态的定义含第三态，系统配置槽写正属于那一类：`.claude/kb/decisions/13-验证路线.md:138`：「系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态」。
- 核心层自己写明这几道屏障防的正是撕裂：`crates/singlefs-core/src/transaction.rs:800`：「没有这一道，两次写同在一段里，崩溃时两槽可以一起撕坏。」
- 量的（原型 `remount` 流，两块盘、384 槽小几何、种子 0；整写 / 撕裂按持久集合里有没有撕裂镜像分）：

| 坏实现 | 态数 | 两槽全废的态：整写域 / 撕裂域 | 崩溃点重放自己的判（恢复 + 池级 checker） | G4-丙 挂在崩溃注入（只整写域） | G4-丙 挂在全量域 |
|---|---|---|---|---|---|
| MUT-B1（盘 0 写完新槽，同段清另一槽） | 20582 | 8 / 34，全是盘 0，补写闸正常时 42 张全挂上 | 0 违例 | 8（崩溃注入摆不摆得到这 8 态看用例的摆法，推的） | 42 |
| MUT-B2（取号两轮之间少一道屏障） | 20583 | 0 / 16 单盘全废（盘 0、盘 1 各 8，全挂上）+ 1 态两盘全废（择不到系统配置） | 1 违例（就是两盘全废那一态：恢复择不到根） | 0 | 16，另 1 态择不到 |

原样输出：
    name=remount domain=torn g4_c=fires dead=[0] recovery=file_read remount=mounted dead_after_the_mount=0 records_on_this_image=1 count=8
    name=remount domain=torn g4_c=fires dead=[1] recovery=file_read remount=mounted dead_after_the_mount=0 records_on_this_image=1 count=8
    name=remount domain=torn g4_c=no_system_configuration_chosen count=1
    name=remount domain=torn o2_reading judge=None o2=[0, 1] count=1
    name=remount domain=torn readings_differ judge=None core=[0, 1] count=1
    name=remount domain=torn states count=75
    name=remount domain=whole states count=20508
    name=remount domain=torn g4_c=fires dead=[0] recovery=file_read remount=mounted dead_after_the_mount=0 records_on_this_image=1 count=34
    name=remount domain=torn states count=67
    name=remount domain=whole g4_c=fires dead=[0] recovery=file_read remount=mounted dead_after_the_mount=0 records_on_this_image=1 count=8
    name=remount domain=whole states count=20515

崩溃点重放自己的判（`judged_tally` 一行四到六千字，不整行抄，用命令取字段）：

    $ grep -o 'violations:[0-9]*,\|first_violation:[^,]*,\|failed_states:[0-9]*,' out/remount-SINGLEFS_PROTO_MUT_B2-o2.counts out/remount-SINGLEFS_PROTO_MUT_B1-o2.counts
    out/remount-SINGLEFS_PROTO_MUT_B2-o2.counts:violations:1,
    out/remount-SINGLEFS_PROTO_MUT_B2-o2.counts:failed_states:1,
    out/remount-SINGLEFS_PROTO_MUT_B2-o2.counts:first_violation:Some("journal_consulted_oracle:no_root_chosen"),
    out/remount-SINGLEFS_PROTO_MUT_B2-o2.counts:violations:1,
    out/remount-SINGLEFS_PROTO_MUT_B2-o2.counts:first_violation:{"I-7.16":"pool_checker:I-7.16",
    out/remount-SINGLEFS_PROTO_MUT_B1-o2.counts:violations:0,
    out/remount-SINGLEFS_PROTO_MUT_B1-o2.counts:failed_states:0,
    out/remount-SINGLEFS_PROTO_MUT_B1-o2.counts:first_violation:None,
    out/remount-SINGLEFS_PROTO_MUT_B1-o2.counts:violations:0,
    out/remount-SINGLEFS_PROTO_MUT_B1-o2.counts:first_violation:{},

四句：
1. 分不分辨臂：分辨挂法（崩溃注入第二、三截 / 崩溃点重放的全量域）。不分辨 G4-丙 与「G4 照旧」：MUT-B2 那 16 态全挂上，G4 照旧只在拒时判，同样判不到。
2. 被判的系统看不看得到：看得到。撕裂态就是一张盘面，G4-丙 只读盘面；判不到是因为崩溃注入不摆这些态，不是判据看不见。
3. 满足的分句：`.claude/kb/decisions/13-验证路线.md:187`：「这一性质由 G4 判（崩溃注入第二、三截」——那条性质说的是无读故障的「崩溃态」，崩溃态按 D13 第 138 行含第三态；挂法只罩整写子集。
4. 改法在这几格上还中不中：G4-丙 照正文的挂法在 MUT-B2 那 16 态上照中（判不到）；G4 照旧、收窄、收窄加 U6 都照中；丙-置（挂成崩溃点重放的观察者，只判 mkfs 那几段之后的状态）在这 16 态上判得出——原型的观察者就是这么挂的，判得出是量的，挂法本身没实现。

旁注（推的）：MUT-B2 这一个坏实现不会活下来，崩溃点重放在两盘同时撕坏两槽的那 1 态上报了。活下来的是它的近亲：三块盘以上的池，或只有一块盘两槽同段的写序错，就只剩单盘全废的态，崩溃点重放一态不报（单盘全废只走旁路报告，`crates/singlefs-checker/invariants/03-journal.md:12`：「单盘全废不判违例，走旁路报告」），挂在崩溃注入的 G4-丙 也一态不报。

### 打中 W2-②（量的）：采 G4-丙 而留着 G4 拒绝那一臂，同一张盘面记两条

MUT-B1 加补写闸恒不过（等于撤掉丙'）：42 张两槽全废的盘面全以「读阶段判完之后系统配置又读不出」在取号之前的逐盘核拒，判官 G4 那一臂报不合法，G4-丙 在同一张盘面上也报，每张记两条；C670、Q3 两臂一条都没被调到（`other_conditions=0`，补写闸正常那一趟 42 张全挂上，也没被这两拒拦下）：
    name=remount domain=torn g4_c=fires dead=[0] recovery=file_read remount=refused reason=SystemConfigurationUnreadableAfterTheReadStage/CheckingEachDeviceBeforeTheAcquisition verdict=illegal g4_arm=fires other_conditions=0 records_on_this_image=2 count=34
    name=remount domain=torn states count=67
    name=remount domain=whole g4_c=fires dead=[0] recovery=file_read remount=refused reason=SystemConfigurationUnreadableAfterTheReadStage/CheckingEachDeviceBeforeTheAcquisition verdict=illegal g4_arm=fires other_conditions=0 records_on_this_image=2 count=8
    name=remount domain=whole states count=20515

- 读码：没注入读故障的崩溃镜像上，G4 那一臂报 ⟹ G4-丙 报。那一臂的复算（`system_configuration_unreadable_after_the_read_stage_on_the_crash_image`）为真只有三种：判不出（要有槽设备报读错，内存崩溃镜像不报）、某块盘一份本池自证过的都没有（就是 G4-丙）、择不到系统配置（读阶段先以别的拒因拒，走不到这一拒，推的）。
- 留着：每张盘面两条，第一轮判决改过的那条测试（`crash_injection.rs` 第三截那条，按 G4 那一臂计数，`crates/singlefs-checker-tier/src/crash_injection.rs:4730` 那一臂）要改成数两条。另一处重数：第三截二次崩溃在挂载第一段一个写都没落的那一态，镜像与第一次崩溃那份相同，同一张盘面在第二截、第三截各判一次（推的，按二次崩溃的抽法读）。
- 撤掉：这些拒由 U6 复算为真判合法，发现由 G4-丙 记一条；G4 那一臂今天的理由本来就是这张盘面不该出现（`crates/singlefs-harness/src/mount_transparency.rs:3066`：「正确实现在 mkfs 返回之后没有读故障的历史里留不下它」），撤掉不丢判别力（推的）。`crates/mutations.tsv` 第 2406–2408 行三条都指着那一臂，要改指 G4-丙；G4-丙 的变异至少四条：整条不判、读故障态也判、只判第二截不判第三截、择不到系统配置时不报。正确实现的崩溃注入里摆不出两槽全废的盘面（W1），四条都要靠手造盘面杀：前两条与第四条在 harness 档直接调判官，第三条照今天第 8968 行那条的靶向二次崩溃（推的）。
- 四句：分辨臂（留与撤计数不同，发现集合相同）；系统看得到；满足的是正文 W2 格「同一张盘面上记几条」；改法：丙-撤 修这一格，正文的 G4-丙 照中。

### 打中 W2-③（读码，另有量的对照）：「复用判官里已有的谓词」与被判的实现共用判自证的那一段

- 判官判「本池自证过」用的就是核心层的解析：`crates/singlefs-harness/src/mount_transparency.rs:2374`：「if let Ok(system_configuration) = SystemConfiguration::parse_slot(&bytes) {」，类型来自 `crates/singlefs-harness/src/mount_transparency.rs:44`：「use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};」；槽距与 fsid 取自核心层的择法 `crates/singlefs-harness/src/mount_transparency.rs:36`：「choose_root, choose_system_configuration, chosen_system_configuration_slot_bytes,」。补写闸与取号逐盘读槽用的是同一个解析：`crates/singlefs-core/src/recovery.rs:4365`：「Some(bytes) => match SystemConfiguration::parse_slot(&bytes) {」。正文说「读槽走判官自己的读法」，自己的只是那个两槽的循环。
- 规则：`.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历`：「运行时与 checker 必须用不同的算法。」
- 独立的一份已经有：池级 checker（O2）自己的解析数单盘全废，`crates/singlefs-checker/src/image.rs:1492`：「pub fn system_configuration_redundancy_report(」。原型在 MUT-B1、MUT-B2 两条流的每个状态上拿它与判官读法比，不一致就记一行 `o2_reading`：MUT-B1 一行都没有，MUT-B2 只有两盘全废那 1 态（判官读法择不到系统配置交 `None`，O2 数出两块盘），上面原样输出里那一行。它不分 fsid（本池与别的池的槽一样数），用在 mkfs 清过 [0, 根环基址) 之后的盘面上没差别（推的）。
- 四句：分辨臂只在核心层的 `parse_slot` 本身错了时分得出（这样的变异没量，推的）；系统看得到；满足的是上面那条规则；改法：丙-置 读 O2 的报告修这一格，正文的「复用判官谓词」照中。

## W3：甲-二（补写闸恒不过的开关、反向测试与一行变异）

被判的那一句：`research/prompts/_m3-close-g4-rewrite-r2-body.md:14`：「同一张闸过的盘面，在补写闸恒不过的副本上断言 G4 报；再加一行变异」。

量的（原型，手造盘面：第一个文件之后盘 0 两槽各在偏移 2048 翻一个字节；`_and_a_unit` 另把盘 0 上第一个文件最后一个单元写翻一个字节，即甲；`assertion_mount_succeeds` 是「这张盘面可写挂载挂得上」这一句成不成立）：
    name=hand_made_image_both_slots gate_never_passes=false g4_c_dead=Some([0]) remount=mounted dead_after_the_mount=0 records_on_this_image=1 assertion_mount_succeeds=holds
    name=hand_made_image_both_slots gate_never_passes=true g4_c_dead=Some([0]) remount=refused reason=SystemConfigurationUnreadableAfterTheReadStage/CheckingEachDeviceBeforeTheAcquisition verdict=illegal g4_arm=fires other_conditions=0 records_on_this_image=2 assertion_mount_succeeds=fails
    name=hand_made_image_both_slots_and_a_unit gate_never_passes=false g4_c_dead=Some([0]) remount=refused reason=SystemConfigurationUnreadableAfterTheReadStage/CheckingEachDeviceBeforeTheAcquisition verdict=illegal g4_arm=fires other_conditions=0 records_on_this_image=2 assertion_mount_succeeds=fails
    name=hand_made_image_both_slots_and_a_unit gate_never_passes=true g4_c_dead=Some([0]) remount=refused reason=SystemConfigurationUnreadableAfterTheReadStage/CheckingEachDeviceBeforeTheAcquisition verdict=illegal g4_arm=fires other_conditions=0 records_on_this_image=2 assertion_mount_succeeds=fails

### 打中 W3-①（杀与不杀是量的，「同一条路径」是读码）：甲-二 那条测试杀不了它自己要加的那一行变异

- 甲-二 的测试开着「补写闸恒不过」开关，断言 G4 报。变异「补写闸恒不过」改的是同一处（补写闸那个函数恒交回一件没过），落地之后不开开关也走同一条路：上面第二行（开关或变异，原型里是同一个开关）判官报 `g4_arm=fires`，测试照样绿。这一行变异只有「闸过的盘面挂得上」那一句杀得掉：第一行 `assertion_mount_succeeds=holds`，第二行 `fails`。
- 那一句今天已经有：`crates/singlefs-harness/tests/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs:159`：「fn a_device_holding_the_version_whose_both_slots_were_zeroed_is_rewritten_by_the_acquisition_and_the_checker_stays_green(」；同一类变异今天已登记一行并由它杀：`crates/mutations.tsv:5010`：「丙' 取号那一刻的见证读不认过了补写闸的盘（照旧拒）」。所以甲-二 那一行要点名的是这条现成的测试，甲-二 自己那条测试对它没有判别力（那条 harness 测试本身没跑，杀不杀它是按代码推的；同向断言在我的手造盘面上失败是量的）。
- 四句：分辨臂（分辨「甲-二 的测试」与「现有的正向测试」谁杀这一行）；系统看得到（测试看得到挂载结局）；满足的是变异登记口径「一行要由点名的测试判红」；改法：不做甲-二、这一行若要加就点名现成那条——在这一格上不再中。

### 打中 W3-②（量的）：判官在甲与甲-二 两张盘面上判出的一样，甲-二 多罩的东西是零

- 甲（第三行，闸一不过）与甲-二（第二行，闸过而开关强拒）判官的结论逐字段相同：`verdict=illegal g4_arm=fires other_conditions=0`。判官今天不读补写闸，G4-丙 也不读，两张盘面在判官眼里只差一个它不看的单元字节。甲-二 只在判官改成读闸（第一轮的 G4-乙）时才多罩一格，那一臂没被采。
- 要测判官那一半不必动核心层：拒因的字段是公开的（`crates/singlefs-core/src/mount.rs:357`：「pub unreadable: SystemConfigurationUnreadableWhen,」），手造盘面加一个照字段造出来的拒因直接调判官就是同一个断言（推的，没写）。
- 与 G4-丙 同时采时：G4-丙 在两张盘面上都只看两槽，甲-二 剩下的价值是零（推的，依据是上面两行判官输出相同、G4-丙 判出的 `g4_c_dead` 四行都是 `Some([0])`）。
- 四句：分辨臂（分辨甲与甲-二 有没有差别）；系统看得到；满足的是正文 W3 格「甲-二 比甲多罩什么」；改法：不做甲-二。

### 没打中（推的）：开关不会变成第二条发布路径

- 开关只让补写闸交回「没过」，补写闸没过就在取号之前拒、一个字节不写：`crates/singlefs-core/src/mount.rs:5797`：「Some(_gate_not_met) => return Err(device),」。规则说的是 `.claude/rules/fs-design.md:88`：「绕开状态机的第二条发布路径不允许」——「恒不过」只少写、不多写，够不着。反过来的「补写闸恒过」才会在闸拒的盘上写系统配置（例如带着更新的根的克隆盘），那是另一种开关，正文没提。
- 放在哪：核心层已有同一族只供测试的开关，值从公开入口传、产品路径写死（`crates/singlefs-core/src/mount.rs:7785`：「pub fn mount_writable_with_test_only_switches<Device: BlockDevice>(」），「补写闸恒不过」照这一族放即可。没造。

### 顺带：它是不是分支纪律要的那种开关（读码）

- `.claude/rules/fs-design.md:122`：「2. **每条分支必须能被测试强制进入。**」——补写闸五件各自已有手造盘面强制进入，`crates/mutations.tsv` 第 5011–5015 行各点名一条 harness 测试（例 `crates/mutations.tsv:5011`：「丙' 补写闸一不看本盘带不带所选那一版的单元」）；「恒不过」一个开关只交回固定的一件，强制不了其余四件。
- 没满足的是另两条：`.claude/rules/fs-design.md:127`：「4. **分支必须可观测**：运行时能看出走了哪一条。」——没过的是哪一件被丢掉了（上面第 5797 行那一句，`crates/singlefs-core/src/mount.rs:5728` 文档注释写着只在核里分辨），拒因只带盘号（`crates/singlefs-core/src/mount.rs:364`：「CheckingEachDeviceBeforeTheAcquisition { device: DeviceIdentity },」）；`.claude/rules/fs-design.md:125`：「3. **每条分支单独验，包括崩溃点重放。**」——补写那一支在正确实现的崩溃注入里一次都进不去（W1：mkfs 之后的盘面上没有两槽全废），崩溃点重放不跑可写挂载。「恒不过」对这两条都不起作用。

## W4：条文改与不改各在哪一格说假话

### D13 已定项 6 第 187 行

被判的那一句：`.claude/kb/decisions/13-验证路线.md:187`：「这一性质由 G4 判（崩溃注入第二、三截，与 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那条同接法）」。

| 写法 | 哪一格说假话 | 依据 |
|---|---|---|
| 不改 | 「由 G4 判」：G4 只在拒的时候判，闸过的两槽全废盘面挂上了没人判（第一轮判决 V2 那一格，这里不重攻） | 第一轮判决 |
| 只把 G4 换成 G4-丙，挂法照正文（崩溃注入第二、三截） | 「这一性质」说的是无读故障的崩溃态，崩溃态含第三态（第 138 行），挂法只罩整写子集：MUT-B2 的 16 态单盘全废全在撕裂域，这样写之后仍然一态没人判 | W2-① 量的 |
| 同上 | 「与 C670 那条同接法」：C670 那一臂判的是一次拒绝，G4-丙 判的是盘面、不挂在拒绝上，接法不再相同 | 读码 |
| 同上 | 同一节后半句：`.claude/kb/decisions/13-验证路线.md:187`：「G4 与那条还有一处不同形：择不到系统配置时那条的复算交」——下文说这时读阶段已拒、到不了这一格；G4-丙 不挂在拒绝上，到得了：MUT-B2 有 1 态两盘全废、择不到系统配置，G4-丙 要先定这一态记不记 | W2-① 量的（`g4_c=no_system_configuration_chosen count=1`） |
| 改成挂在崩溃点重放的全量域（丙-置） | `.claude/kb/decisions/13-验证路线.md:187`：「mkfs 返回之前的崩溃态不在 G4 的射程…崩溃注入的候选段与 E171（三oracle合谋测试） 的基镜像都取在 mkfs 之后」——这半句的理由不再罩得住：崩溃点重放的流从 mkfs 起（合表里有 `make-filesystem-over-a-used-pool` 一项），排除 mkfs 那几段要由枚举按段号做，判官自己看不出（第一轮 V1 已量 mkfs 末段，这里只说理由换了） | 读码，推的 |

能写真的一稿（零轮，只供判决参考）：「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由 G4-丙 判：崩溃点重放 mkfs 那几段之后的全部状态（第三态在内）上判一次，崩溃注入第二、三截的整写子集上判一次；择不到系统配置也记一条；判自证读池级 checker 的逐盘数，不用核心层的解析。

### D23 已定项 14 第 396 行与判官判据

- 被判的那一句：`.claude/kb/decisions/23-journal的角色与格式.md:396`：「这一步在某块盘一份自证过的系统配置都没有时也拒（用户 2026-10-03 定留着）」。
- 实现：取号那一刻读见证时，过了补写闸的盘不拒：`crates/singlefs-core/src/transaction.rs:920`：「if devices_rewritten_by_the_acquisition.contains(&slots_of_this_device.device) {」；条文那一头在 `.claude/kb/decisions/22-单元原子性怎么合成.md:181`：「取号那一步补写它的两槽，要这几样都成立，有一样不成立就拒」。
- 判官的判据照的是 D23 那一句，不带闸：`crates/singlefs-harness/src/mount_transparency.rs:2414` 那段文档注释引的就是它；D13 给判官的口径是 `.claude/kb/decisions/13-验证路线.md:72`：「照那一条的判据在崩溃镜像上自己复算成立」。

| 写法 | 哪一格说假话 |
|---|---|
| 不改 | D23 第 396 行：自 2026-10-19 补写例外起，过了补写闸的盘取号那一步不拒，这一句说的是例外之前的实现 |
| 改 D23（带补写例外，指到 D22 第 181 行），判官照旧不带闸 | D13 第 72 行：对「读阶段判完之后系统配置又读不出」这一拒因，判官复算的已不是「那一条的判据」。咬人的地方只在带读故障的档里（没读故障的档有 G4 那一臂或 G4-丙 压着）：闸过的两槽全废盘面上错拒，判官判合法 |
| 两处都改：D23 带例外，D13 第 72 行写明这一拒因的复算不带补写闸、为什么可以不带，欠账一行 | 没找到说假话的格。「可以不带」的依据：带读故障的档里摆不出两槽全废的盘面——读故障那一步（`zeros_mount`）20519 态里 0 态（量的）；「崩溃恢复抛弃根」那一步只藏根与单元、不藏系统配置槽（读码，推的） |

判官要不要跟着带闸：带闸要在判官里另写一份五件（规则同 W2-③），能多判出的只有上表第二行那一格，而那一格在量过的读故障历史里没有可达盘面——不带（推的）。

### 账记在哪

- 正文说新 C 号从 C873 起；工作区此刻的 `.claude/kb/checks-owed.md:578`：「隔离记录两盘不对称没人判」，C873 已被占（这份文件的 sha256 开工时是快照里的 c3b73d…，此刻是 528110…，开工之后被别的会话改过；HEAD 与暂存区里都还没有 C873）。下一个空号要现查。
- 不采 G4-丙 时要记的空档两条（零轮）：闸过的两槽全废盘面挂上之后没人判「每块盘至少一份自证过」（第一轮的空档）；崩溃点重放撕裂域里的单盘全废只走旁路报告、没人判（W2-①，量的：MUT-B2 16 态）。采了 G4-丙 而照正文挂法，后一条照样要记。

## 没打中的形状

- 正确实现、mkfs 之后、没注入读故障：关闭再可写挂载 20519 态、冷启动再可写挂载 20519 态、读故障那一步 20519 态（这一步在崩溃注入里算注入过读故障，G4-丙 本不判；量它是看「没注入读故障」那个条件是不是在挡东西——这一步也一态都没有，条件在这些取样里没挡住任何东西）、八个整写基线上的二次崩溃 172366 态：G4-丙 判出 0 态；判官读法与核心层读法两路 0 处不一致。
- 补写本身中途崩溃：mkfs 之后的这些历史里一次都走不到补写（没有两槽全废的盘），只在第一轮从 mkfs 末段起的那一形里走得到。
- 甲-二 的开关成为第二条发布路径：恒不过只把补写换成取号之前的拒，不写（推的）。
- 取样范围：两块盘、单元区 384 槽的小几何、历史种子 0、起点是第一个文件之后。放开的用户动作：关闭再可写挂载、冷启动再可写挂载、读故障可写挂载、第三截的八个基线；抬 F 在这个起点上四种目标都被拒没写盘；回退、覆盖写、清扫没跑（见下）。三块盘、别的几何与种子没取。

## 这条腿自己的限度

- 只抽了一次：W1 的「没打中」按 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不能单独支撑结论，要再派一条同立场腿（`-s2`）。
- 没跑的流：`overwrite` 1048588 态、`overwrite_losing_power_then_mount` 1069094 态（各自超过约 10⁶，照定义不跑）；`remount_then_roll_back` 86066 态（估约 6 分钟，超过子 agent 前台 240 秒的时限，命令交主 agent）；先覆盖写再抬 F 的流超过约 10⁶。
- 第三截那一形是我直接按写表叠出的八个整写基线（每个系统配置槽写段落一块盘的那一写），不是崩溃注入按种子抽的那几个第一次崩溃；每个基线上那次挂载的全部崩溃状态都枚举了（含撕裂态，比崩溃注入第三截多）。崩溃注入第二、三截本身（checker 档的代码）没跑：它们是重型，按派发不跑。所以「挂在崩溃注入的 G4-丙 判出几态」是按它的域（整写子集）从全量枚举里分出来的，没有经崩溃注入的抽法；MUT-B1 那 8 个整写态崩溃注入抽不抽得到是推的。
- G4-丙 是我照正文的描述在原型里另写的一份（判官的那两个函数是私有的，照它们的读法重写），不是将来的实现。
- MUT-B2 是我造的（取号两轮之间删掉那一道屏障），不在 `crates/mutations.tsv` 里；harness 档那条「取号两轮与写行单元之间有屏障」的测试（`acquisition_rounds_and_the_row_publish_units_of_a_remount_are_separated_by_barriers.rs`）多半直接杀它，没核。它在这里只用来量「两槽同段而内容都自证得过」这一类的两槽全废落在哪个域。
- `out/` 里的计数出自先后几版原型（每版只加打印，不改判法）：`remount` 出自第一版（不打历史结局）；`remount-SINGLEFS_PROTO_MUT_B1`、`+SINGLEFS_PROTO_GATE_NEVER_PASSES`、`zeros_mount` 出自第二版；`remount-SINGLEFS_PROTO_MUT_B2`、八个 `second_crash_base_*`、`-tally`、`cold_start_then_remount`、四份手造盘面出自第五版（加了历史结局与崩溃点重放自己的判）；两份 `-o2` 出自第六版（加了池级 checker 那一路的对照，也就是 `src/main.rs` 现在这一份）。rerun.sh 比对时去掉 `history=` 与 `o2_reading` 两样，其余逐行比。池级 checker 那一路只在 `-o2` 两份上跑过。
- 跑法：每条流经 `scripts/hosts/capped.sh 31` 与 `run-with-memory-cap.sh 8G`（编译 16G）加 `nice -n 19` 在本机跑；先单跑 `remount` 一条估时（77.6 秒、峰值 114 MiB），按它定每批两条并行。批与件：b1（MUT-B1、MUT-B1 加闸恒不过）、b2（MUT-B2、读故障那一步）、b3（MUT-B2 重跑、基线 0）、b4（基线 1、2）、b5（基线 3、4）、b6（基线 5、6）、b7（基线 7）、b8（MUT-B1 重跑）、b9（冷启动）、b10、b11（两份 `-o2`）；每批照 `.rc` 文件数核过件数，退出码全是 0。b6 超过工具的 240 秒前台时限，被工具挪到后台，我没有轮询它，等完成通知之后读的两个 `.rc`（都是 0）。单条流 40–200 秒（本机负载 400–550，别的会话在编崩溃点重放）。这条腿跑过的崩溃状态合计 378000 个（只数逐态枚举过的，只算态数的几趟不算），在约 10⁷ 之内。
- 多机：起重活之前的资源提醒每次都写「本机放不下、第二台放得下」（按默认 8 GiB 预测；实测峰值 114 MiB）。第二台要经 `scripts/hosts/multi-host-run.sh --peer`，它拷的是起它那棵树（要有 `scripts/` 与规范副本）；我的编译副本只有 `crates/` 那几样，没有走第二台，在本机加 `nice -n 19` 跑完。
- 引文行号按主工作区现查；开工时那几份代码与快照 sha256 相同，之后别的会话可能改过（`checks-owed.md` 就改了）。

## 要主 agent 跑的命令

1. 回退那条流（W1 没跑的一格）。工作目录随意，先照「复跑」那一条把原型编出来（或用 rerun.sh 编出的 `<草稿目录>/target/release/g4-attack-prototype-r2`），再：
   `CARGO="$(rustup which cargo)" SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=<新的库目录> bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G nice -n 19 <草稿目录>/target/release/g4-attack-prototype-r2 remount_then_roll_back`
   内存上限 8G、线程 31，约 6 分钟。看 `name=remount_then_roll_back` 开头、带 `g4_c=` 或 `readings_differ` 的行：有就是 W1 打中，没有就是这一格也 0。
2. 整趟复跑（判决要引数时）：「复跑」那一节的命令，`--frozen` 给一棵与 `research/prompts/m3-close-g4-rewrite-r2-snapshot/code-sha256.txt` 逐份相同的树；看末尾 18 行 `name=rerun_compare ... same=`。

## 没做什么

- 没判辩方那几格，没读禁读清单里的三样，没替主 agent 采纳任何改法。
- 没跑 checker 档包的测试、崩溃点重放入口、崩溃注入、门禁；原型只调了 checker 档的枚举库函数，编出来的二进制直接执行。
- 没改 `crates/`、`crates/mutations.tsv` 与 kb；副本上的数不算入库装置上的数，要引得先在入库装置上重做。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-g4-rewrite-r2-opus` 里的编译目录、仓副本与库目录交回前删掉，留下的是各趟原样输出（`out/` 的 `.txt` 与 `.stderr`）、构建日志、跑法脚本与进度记录。
