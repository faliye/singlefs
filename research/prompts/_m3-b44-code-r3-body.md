# 代码欠账批 4-4 修红批的代码轮：第三轮正文（2026-10-18）

<!-- doc-lint:not-numbers L1 L2 L3 L4 L5 L6 J1 J2 J3 J5 W13 W14 W15 W16 -->

## 一、这一轮要判什么

代码欠账批 4-4 第二轮代码轮（`research/prompts/m3-b44-code-r2-main-verification.md`）判下的修红批与用户在岔路单 `research/prompts/m3-b44-code-r2-forks.md` 上定的岔路一「读一次两边共用」，由实现员 W13–W16 写完、打进主工作区。合入后验证第二趟测试全绿（`merge-and-verify.sh --skip-apply`：223 个测试二进制 2310 条，fmt、clippy、build 过），轻型三条：形式化 3 格绿，崩溃 17 项派出 16 绿 0 红 1 本次未跑（钉表那一项），变异这一批新增与整行替换的 37 行里抓到 35 行、2 行基线在内存上限下撞顶没跑。这一轮攻 W13–W16 对七份产品代码的改动：`crates/singlefs-core/src/` 下 `block_device.rs`、`deleted_inodes.rs`、`mounted_session.rs`、`mount.rs`、`recovery.rs`、`transaction.rs`、`unit.rs`，判「代码做的是不是条款说的」「哪一格会错」。

被判的改动全文：`research/prompts/_m3-b44-code-r3-diff.md`（这七份在 W13 之前与四份补丁都打上之后的 diff，7 个文件 +1178/−183）。实现员报告：`/home/fy5090/.local/share/singlefs/work/m3-b44-w13/report.md`、`m3-b44-w14/report.md`、`m3-b44-w15/report.md`、`m3-b44-w16/report.md`。调度记录：`records/2026-10-13-代码欠账批.md` 里「W13 交回」到「崩溃轻型（单机，W13–W16 合入后」那几条，含主 agent 对 W13 四样读法、W15 收窄与 W16 停下五条的判定。

出口：每格判「代码对得上条款 / 对不上（给反例）」；对不上的写成改法（并进验证段的下一批修红）或交用户。

## 二、实现今天的样子（主 agent 的观测，2026-10-18 现查）

冻结副本 `~/.local/share/singlefs/work/m3-b44-code-r3/repo/`：主工作区此刻的 `crates/`（含没跟踪的新测试文件）、`Cargo.toml`、`Cargo.lock`、`.cargo`、`litmus/`、`.claude/heavy-thread-default`。`crates/*/src/` 下 `.rs` 与 `.wgsl` 的 sha256 在 `research/prompts/m3-b44-code-r3-snapshot/crates-sha256.txt`（118 份）。行号按冻结副本。方案按 `crates/` 今天的实现来谈。

几个落点（冻结副本里现查的名字与行号）：

- `mounted_session.rs`：`DeviceReadOnceInThisMount`（第 571 行，W13：这次挂载里每一块第一次读才穿到底下那块盘、读到的字节记下，之后再读取记下的那一份；写过的块读写下的字节）；`ReadRouteOfThisMount`（第 583 行，W15：`SharedBetweenTheRehearsalAndTheRealRun` 与 `StraightToTheDisk` 两路）；它对 `BlockDevice` 新方法 `reread_at_after_a_bad_read`、`keep_the_self_verified_reread` 的实现（第 497、510、829、841 行，W16：读坏当场重读穿到盘上，读的一方核过自证之后用重读的字节换掉记下的那一份）。
- `block_device.rs`：`BlockDevice` 上两个带默认实现的新方法（第 162、171 行，W16）。
- `recovery.rs`：`PoolReaderOfOneReadPass`（第 171 行，W16）与各读者的 `keep_the_self_verified_reread`；J1：所选那一版 journal 记录新根段里的指针在重新编码（`record.to_bytes()`）之前按 I-2.5 判（W13）；`deleted_inodes` 根条目宽 `!= 8` 就拒（W13，外部审核 ②）。
- `mount.rs`：`establish_instance` 里建分路、逐盘核与实例表清扫包在 `with_reads_straight_to_the_disk`（第 6824 行）里（W15）；取号拆成「写之前那几次读」与「写」两半（W15，连同 `transaction.rs`）；预演读错时取号之前交 `MountError::Publish`，cause 是 `PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition`（W13）；`instance_in_the_self_verified_unit_header_at`（第 2214 行，W14：比单元头里的 fsid 与本池系统配置的 fsid，不同就不算被更新单元盖掉）。
- `unit.rs`：`data_unit_payload` 自己判声明长度越界、交类型化错误（W13，外部审核 ⑥）。

实现员自己定、主 agent 认了的读法（调度记录「W13 交回」「W15 交回」两条）：ⓐ（W15 收窄后）共用的那一份只罩抬 F、续做与它们的预演，逐盘核、取号写之前那几次读、实例表清扫照旧读盘；ⓑ 两趟预演里读错与「没走到结局」同时出现就拒，被另一份兜住的那次读错也算；ⓒ 报成功的写记成写下的字节；ⓓ 写行与暖机那一串预演的读错照旧走原来的错误成员；读坏当场重读绕过记下的那一份、读到自证过的就换掉记下的那一份。这几样都被攻过零轮。

主工作区里别的会话还有没提交的改动（`crates/singlefs-checker-tier/`、`crates/singlefs-unit-graph/`、`crates/singlefs-checker/src/walk.rs`），不在这一轮射程里；冻结副本照原样带着它们，编译要用。

## 三、格

| 格 | 改动 | 压着的条款 | 判什么 |
|---|---|---|---|
| L1 | 读一次两边共用：`DeviceReadOnceInThisMount`、`ReadRouteOfThisMount` 两路与取号读写两半（W13、W15） | 岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一「读一次两边共用」那一行（用户已定）；第二轮判决 J2、J3 两行；逐盘核、取号、实例表清扫三段各自要求现读盘的条款（W15 报告「逐条定因」第 1–8 条点名的条款与文档注释，腿现查原文，不照抄报告里的编号） | 预演与真做在抬 F、续做里读的是不是同一份；照旧读盘的三段是不是每一次都真读到盘上、有没有漏进共用那一份或多读；ⓐ–ⓓ 每一样会不会让预演与真做的结局不同、会不会改掉无故障时任何一次挂载写出的字节；ⓒ 在写报成功而没落盘的设备上这次挂载之后读到的与真盘分不分得开；记下那一份占的内存有没有上界（岔路二交 E179 量，这里只判会不会撑爆一次合法挂载） |
| L2 | 预演读错在取号之前拒（W13 的新拒因、W16 重造的两条用例） | 第二轮判决 J2 那一行与岔路一那一行；`mount.rs` 里 `MountError::Publish` 与新拒因的文档 | 拒了之后两块盘逐字节不变、再挂一次不带故障照常走完；ⓑ 偏拒的读法会不会把只是被另一份兜住、真做本来走得通的挂载拒掉（给出一段可复现的历史与读故障窗口，量拒的比例）；调用方按 `MountError::Publish` 认的东西在这一拒上还成不成立 |
| L3 | 读坏当场重读穿到盘上、换掉记下的那一份（W16：`BlockDevice` 两个新方法、`PoolReaderOfOneReadPass`、改到的六处重读） | 读得出而对不上要重读的条款（D19（块指针的结构与宽度预算） 已定项 5 与 D16（发布语义） 已定项 1 里「知道住着根的槽」那几句，腿现查原文）；调度记录「W15 交回」那一条主 agent 判定 ② | 重读之后换掉的那一份会不会让预演与真做看到不同字节；坏字节第一次被记下、重读又失败时后面那几读看到的是什么；W16 没改的两处（只重读「读不出」的那一处、E2 进程内重读挂载时读坏的槽）是不是真的不用改；带读故障注入的设备上默认实现走不走故障注入那一层 |
| L4 | J1：journal 记录新根段指针在重新编码之前按 I-2.5 判（W13，W15 改了一条用例的造法） | I-2.5（`crates/singlefs-checker/invariants/04-校验和.md` 第 10 行）；第二轮判决 J1 那一行 | 新根段里的指针全没全（还有哪类没进这一判）；坏镜像在任何写之前交类型化错误、两块盘不变、不 panic；会不会误拒合法写路径写出的记录（空根、全零位置条目、单盘池） |
| L5 | 比 fsid（W14） | 用户 2026-10-17 弹窗定「比 fsid」（调度记录「用户弹窗定 C834 那一判加比 fsid」那一条）；C834 用户定案 `research/prompts/c834-r3-main-verification.md` 第四节 | 本池 fsid 取自哪一份系统配置、那一份在这一刻是不是已经择定；合法写路径写出的单元头 fsid 是不是恒等于本池的（格式化、换盘、扩盘、回退之后）；别池的单元头放过、本池的误拒，各给不给得出一段可复现的镜像 |
| L6 | 外部审核 ②⑥（W13）与 W13–W16 新写、改过的注释 | 调度记录「2026-10-16 外部审核报来七条」那一条的 ② 与 ⑥；`.claude/singlefs-ai-sop/rules/code-discipline.md`「注释只写为什么」 | `deleted_inodes` 根条目宽 `!= 8` 拒会不会误拒合法写路径写出的根；`data_unit_payload` 的防护与 `parse_data_unit` 的判是不是同一个界；注释说的是不是今天代码做的事 |

## 四、分工

| 腿 | 立场 | 格 |
|---|---|---|
| 云端正推（Sonnet） | 逐格核「代码做的是不是条款说的」：每一格引条款原文与冻结副本的代码行，对得上与对不上逐句写 | L1–L6 |
| 云端攻方（Opus） | 攻「哪一格会错」：在冻结副本的拷贝上造镜像、历史、读故障窗口或崩溃状态，找让改动判错（写掉还在用的单元、误拒合法态、预演与真做结局不同、panic、盘变了）的那一段 | L1、L2、L3、L5，有余力再 L4、L6 |

这一轮是第三轮：第一、二轮攻过的面（第一轮 K1–K8、第二轮 J1–J5 的原形）不重复攻，只攻这一批改出来的新形态。

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格两条腿都没给出反例 ⇒ 那一格代码对得上条款 | 两份报告那一格都写试过的形、没打中 |
| 一格被造出判错的段 ⇒ 那一处要改，改法并进修红批 | 一段可复现的镜像、历史、读故障窗口或崩溃状态 |
| 一格的反例落在条款没写的地方 ⇒ 交用户定条款 | 反例成立，而条款对那一形没有一句话 |

⚠️ 打中之后，先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、交付要求

- 报告：
  - 先写进 `research/prompts/m3-b44-code-r3-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`）。
  - 每一次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建。
  - 回复只写短句；早点开写。
- 引文：
  - 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
  - 引代码写冻结副本里的路径与行号。
- 草稿与模型：
  - 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-b44-code-r3-<腿名>/`。
  - 量法拷冻结副本，不在冻结副本里改。
  - 模型与驱动随报告拷进 `research/prompts/m3-b44-code-r3-<腿名>-model/`，含 `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换 `rerun_steps` 那一段；复跑经 `scripts/hosts/run-with-memory-cap.sh`。
  - 编 rocksdb 的那一步带 `BINDGEN_EXTRA_CLANG_ARGS`，取 `scripts/crash/crash-amplification-lib.sh` 的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS`。
- 资源：
  - 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 14G 的上限。
  - 同一时刻只起一条 cargo；用自己的编译目录，不从别处拷编译产物。
  - 前台命令预计超过 4 分钟的拆成小段。
  - 一段历史的崩溃状态全量超过约 10⁶ 个的，不跑，写进报告「要主 agent 跑的命令」。
  - 只在本机跑：不连第二台，不起任何会走两台的命令（崩溃放量、双机驱动、`crash-point-replay.sh`）。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
- 新写的文字不用「层 0」「第一段 / 第二段」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。
