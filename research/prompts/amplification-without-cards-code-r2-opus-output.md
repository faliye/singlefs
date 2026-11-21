# 崩溃放量写待判行之前先问 GPU 服务：代码轮第二轮攻方腿报告

日期：2026-11-18。

打中 7 条（其中 2 条的读数是推的、待主 agent 跑）；八格检查单里不成立 3 格、推不出 0 格。九问里第 4、5 问与第 9 问的 (a)(c) 构造不出，第 9 问 (b) 推不出。

被判文件与开工快照逐个相同：`sha256sum -c research/prompts/amplification-without-cards-code-r2-snapshot/judged-sha256.txt` 与 `crates-sha256.txt` 没有一行不是 OK（`grep -v ': OK$'` 无输出、退 1）。
这一轮没写模型（打中的都是协议与控制流上的走法，靠读代码追到行；要量的只有一组变异探针，放在副本里交主 agent 跑），所以没有模型目录与 `rerun.sh`。
每一句代码引文写进来之前，在被引文件里按子串现找过一次（草稿目录 `quotes.tsv` 列了 65 处，「bad 0」）。下面凡没跑出来的读数都标「推的」。

## 各格判定一览

| 问 / 格 | 判定 | 一句话 |
|---|---|---|
| 1 问与写之间 | 打中（H1、H2） | 写了行之后服务换一把键重起：本趟退 78、末行说「服务不在跑」，行留在库里永远没人判、也没人删；之后每次服务重起的 4 小时里，所有配 GPU 的一趟（连一块都不用判的）开头就退 78。服务答收而一直不判的那一支照旧是空等（H2，推的） |
| 2 只在有行要写时才问 | 构造不出「排出空而等」；另并入 H1 | 排出的行只看结论，上一趟留的同键行照排、照问、`AlreadyPresent` 接着等；排出空时不问也不等。但「问在不在跑」那一下（`check`）不看有没有行，开头无条件问 |
| 3 不收都归 78 | 打中（H3、H4） | 控制命令自己表里写「退 2 是用法错」，判段把它与 Python 异常退 1 一起当「不收」退 78，族名两边各写一份字面量、没有测试绑；`said=` 带进重复的 `family=`、`expected=` 与第二台地址，没有任何读者认这一行头：崩溃放量打出的是显卡那一套出路（量的，任务 19423），门禁把崩溃枚举用例判红、出路写「改代码还是改断言」 |
| 4 测试二进制里 exit(78) | 构造不出「没跑完记成过」 | 登记用例一条一趟、`--exact`，同一二进制里别的用例不在这一趟；门禁要恰好一行 `test result`，exit(78) 之后没有这一行，照红判 |
| 5 计划行后两栏 | 构造不出 | 只排计划在录入之后、读回清单之后；事实表不在库里读回就报错，到不了数栏；两对栏之间不该有大小关系，代码也不假定 |
| 6 切段纯函数 | 打中（H5，轻） | 段长向上取整：18 张盘拿 17 条只起 9 条，另 8 条在单测构建里占着份额直到判完；按盘的次序交回成立 |
| 7 单测那一层恒答收 | 打中（H6，推的） | 「有行才问」那道守卫没有一条测试会红：恒答收那一层与核对那一侧的测试都传「全收」，专门的三条测试都有行可写 |
| 8 删借草稿区之后 | 打中（H7） | `GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort` 在产品路径上造不出来：唯一造它的函数只有一个调用方，当场改成 `Err`；两处产品 `match` 的那一臂、`gpu_memory_was_short` 与一句注释指着被删的语义 |
| 9 (a) 惰性取卡退出留半截行 | 构造不出 | 退出点在头一张要判的盘那一刻，此前写库都是一帧一帧同步写，此后才写观察者结论 |
| 9 (b) 等的那一圈读库开销 | 推不出 | 卡住判法 600 秒窗口里阈值 6 CPU 秒；问服务 20 次约 2.4 秒（第一轮主 agent 量的），每圈成批点读要在 12 毫秒 CPU 以内才判得出卡住；块多时这一项没人量过 |
| 9 (c) 第二台起子进程 | 构造不出（量的，一次） | 任务 19423 正是派到第二台跑的：控制命令在第二台上两台都问，`check` 过、第 ① 段照服务判完 |
| 格 走不到的理由二次查实 | 成立 | HEAD 里借草稿区那一支是私有函数，唯一调用方把来路写死成 `DevicesStartedForThisRun` |
| 格 类型与名字 | 不成立 | 第一轮第 4 条照旧；另：族名在 Rust 与控制命令两边各是一个字面量，没有类型或测试绑（H3） |
| 格 不兼容旧代码与旧格式 | 不成立 | 「控制命令不认 serves 的也当不收」只为没有 `serves` 的旧控制命令留，同一棵树里没有这种控制命令；单测那一层是为旧签名留的 |
| 格 不新建判定库 | 成立 | 问服务只读退出码；没有新存放处 |
| 格 所有任务多机可用 | 成立（一次样本） | 见 9 (c)；另记「serves 至少一台、check 每一台」两种合成的错位（H1 末段） |
| 格 测试优先多线程 | 不成立 | 「照拿到的条数起线程」在不整除时不成立（H5） |
| 格 带 --item 的命令与项 | 成立 | 任务 19409 末段「点名的 17 条变异都抓到了」；另记 H6 那道守卫没有变异行 |
| 格 新崩溃流多罩了什么 | 成立 | 新用例断言 `(Some(0), Some(0), Some(state_count))`，挂 `gpu` 特性 |

四句（分不分辨臂、系统看不看得到判别它的东西、满足判据字面哪一句、跑前改法）逐条答在各条末尾：这是代码轮，没有臂，第一句一律「不涉臂」；正文没给改法，第四句一律「跑前没给改法」；我自己提的改法都标「只在我的推演上、被攻过零轮」。

## 撤回我第一轮的说法

- 第一轮打中 A 里「每次 check 摊到 0.3 CPU 秒以上就过阈值、一直挂」：主 agent 量的一次约 0.12 CPU 秒，这一句撤回。
- 第一轮打中 A 的改法「写核对待判行之前退 78、打服务还不收」没分「不收」与「问错了」：现在照这个样子做成了，退 2、退 1 也一起归 78（H3）。那一句改法按原样撤回，改成只把「答了不收」归 78。
- 第一轮打中 E 的改法「31460 行用守卫的 `threads()`」不够：照拿到的条数切段时段长向上取整，段数会少于条数（H5）。撤回「换成守卫的条数就够」。

## 第 1、2 问：问与写之间

### H1：写了行之后服务换一把键重起，行成了孤行；之后每次服务重起的 4 小时里配 GPU 的每一趟开头退 78

前缀（固定）：判段配了 GPU、这台机器有块要判；在跑的服务 S1 编进的 GPU 判器组键是 K1。故障（固定）：本趟写完待判行、服务还没判完时，用户照出路重编换上服务 S2，键 K2 ≠ K1。用户动作放开扫过三种：S2 起来时本趟还在等 / 本趟已被停或已退 / 换上之后接着重跑同一分项（新树、键 K2）或别的分项；三种都落到下面同一个结局。

逐跳（文件:行）：

1. 判段入口先问一次在不在跑：「let control_command = exit_unless_the_gpu_verification_server_runs()?;」（`crates/singlefs-verification/src/crash_amplification.rs:34360`）。
2. 排出了行才问收不收：「if !rows_to_write.is_empty() {」（`crates/singlefs-verification/src/crash_amplification.rs:34078`），S1 答收，逐块「.put_the_gpu_pending_row_unless_present_or_raise_it_to_forced(」（`crates/singlefs-verification/src/crash_amplification.rs:34090`），行里期待的键是 K1。
3. S2 起来：服务只认领键相同的行，「row.expected_gpu_judge_units.0 == self.gpu_judge」（`gpu-verification-server/src/library_pending_pool.rs:144`）；对不上的一行打一次「let line = version_mismatch_line(&pending, &self.version_keys);」（`gpu-verification-server/src/library_pending_pool.rs:601`）、「self.name_once(pending.name.pending_key(), line);」（`gpu-verification-server/src/library_pending_pool.rs:602`），随即 `continue`，行不删、不改。「打一次」是每个进程一次：「if self.named.insert(key) {」（`gpu-verification-server/src/library_pending_pool.rs:559`）。
4. 本趟还在等的：满 15 圈问一次在不在跑，「if reckonings_since_the_last_server_check >= waiting.reckonings_between_two_server_checks {」（`crates/singlefs-verification/src/crash_amplification.rs:34156`）。控制命令 `check` 判过在跑之后读日志：「refused = version_mismatch_refusal(paths)」（`gpu-verification-server/control:934`），只认此刻主进程打的，「if str(entry.get("_PID", "")) != main」（`gpu-verification-server/control:844`），往回看「VERSION_MISMATCH_LOOKBACK_SECONDS = 4 * 3600」（`gpu-verification-server/control:239`），有就退 1。
5. 判段把退 1 当不在跑：「return Ok(WaitingForTheGpuVerificationServerEnded::ServerNotRunning { line });」（`crates/singlefs-verification/src/crash_amplification.rs:34158`），最外层「exit_because_the_gpu_verification_server_is_not_running(&line)」（`crates/singlefs-verification/src/crash_amplification.rs:34393`），退 78。末行是 `CRASH_AMPLIFICATION_GPU_VERIFICATION_SERVER_NOT_RUNNING … exit_code=1 said=✗ gpu-verification-server 在跑，但… / … / → 怎么办：用此刻的主工作区重编…`，后半句是固定的「服务不在跑就退 {GPU_VERIFICATION_SERVER_NOT_RUNNING_EXIT_CODE}，不退回进程内判。出路：python3 gpu-verification-server/control start，」（`crates/singlefs-verification/src/crash_amplification.rs:34271`）。同一行里 `said=` 说在跑、本体说不在跑。
6. 库里那批 K1 行：产品代码里没有删它们的路。服务只在写结论那一帧里删、或判完而可复用时删（都在键相同之后）；验证包里删待判行的调用只在测试模块里，例「.delete_the_gpu_pending_row(&stream, block_number)」（`crates/singlefs-verification/src/crash_amplification.rs:13888`，测试模块从第 7198 行起）。按新树重跑同一分项，流键里带期待的键，是另一条流，K1 那批照旧不动。
7. 之后每一趟配 GPU 的判段（崩溃放量两段、崩溃枚举用例）都先过第 1 跳的 `check`：S2 的日志里有它对 K1 行打的那几行，`check` 退 1，开头就退 78。这一跳不看有没有块要判：调用处「let gpu_judging = judge_my_blocks_on_gpu(」（`crates/singlefs-verification/src/crash_amplification.rs:24963`）在第 ① 段 GPU 那一臂里无条件调，整条流都复用的一趟同样退 78。
8. 出路自己绕圈：末行的出路是 `control start`，`check` 的出路是重编、install、stop、start：「VERSION_MISMATCH_HOWTO = ("用此刻的主工作区重编」（`gpu-verification-server/control:271`）；分项驱动认出这一类打的是「先起服务 python3 gpu-verification-server/control start」（`scripts/crash/crash-amplification-lib.sh:968`）。照做换上 S3（还是 K2），S3 是新进程，`named` 是空的，对 K1 行再打一遍，`check` 再退 1 四小时。出得去的只有三条：等满 4 小时不再重起服务；有人手工删那批行（没有工具）；重编回 K1。

会看到什么（推的，没量）：本趟退 78、末行如第 5 跳；库里每块一行 K1 待判行永久留着；此后 S2（以及之后每次换上的进程）起来的 4 小时内，所有配 GPU 的一趟都在开头打 `NOT_RUNNING … exit_code=1`、退 78，崩溃枚举用例在门禁里照红判（`.claude/gate.d/crash-point-replay.sh:3449` 说服务不在跑那一行照红判）。

同形的第二条路（推的）：两台都点名了卡，A 收 K、B 收 K'（只在一台换上了新二进制）。`serves` 至少一台收就退 0：「served = bool(serving) and (len(serving) == len(outcomes) or not control_broken("serves-every-host-required"))」（`gpu-verification-server/control:1192`）；`check` 要每一台都 0：「return RUN_CONDITION_EXIT if RUN_CONDITION_EXIT in codes else 0」（`gpu-verification-server/control:1073`，退 1 的另照「头一个不是 0 也不是 78 的」交回）。本趟写了 K 行，B 扫到打对不上行，下一次 `check` 退 1，本趟退 78，而 A 其实在判这批行。

四句：不涉臂；系统看得到——`check` 的标准错误里写着期待的键与这一份二进制的键，判段只要按「这一趟期待的键在不在对不上那几行里」分，就能把「别人的孤行」与「我的行没人收」分开，此刻没分；满足第 1 问「库里留不留没人判的行、退出码与末行说的是什么」与第 2 问「空的那一支不问就等，等的是谁」的反面（空的那一支不等，但开头那一问照问）；跑前没给改法。

改法（只在我的推演上、被攻过零轮）：等的那一圈与开头那一问都换成按族、按键问（`serves <族> <这一趟的键>`），不再用全局的 `check` 判「在不在跑」；开头那一问挪到排出了行之后；另给服务或判定库一条删「没有一台在跑的服务收这把键」的待判行的路，删之前打一行点名。

### H2：服务答收这一把键、却一直不判时，照旧是第一轮第 1 条那种空等（推的）

- 服务认领到一行之后配不出它的流（读回判表、流程、排计划任一步报错），这个进程就再也不取它：「self.streams_not_configured.insert(pending.name);」（`gpu-verification-server/src/library_pending_pool.rs:747`），扫库时「|| self.streams_not_configured.contains(&pending.name)」（`gpu-verification-server/src/library_pending_pool.rs:585`）就跳过。它打的是 `GPU_VERIFICATION_SERVER_PENDING_ROW_SKIPPED`，`check` 只认版本对不上那一类行，照退 0；`serves` 照答收。
- 判段这一边：块数不动不打进度行，注释自己写了这一种结局，「块数不动就不打：服务答收这一族这把版本键却一直不判时日志不涨，交统一任务服务照卡住停」（`crates/singlefs-verification/src/crash_amplification.rs:33839`）。
- 能走到这里的一种（推的）：服务与任务从两棵树编、GPU 判器组键相同、而读回流程或判表的编码不同（编码不在判器那一组判法单元里），服务解不开。结局：日志不涨，交卡住判法；卡住判法会不会触发见第 9 问 (b)。库里留行，与第一轮第 1 条同形，只是窄了。
- 四句：不涉臂；系统看不到——判段手里只有「行还在」这一样观测，分不出慢与不判；满足第 1 问末句「会不会回到第一轮第 1 条的空等」；跑前没给改法。改法（零轮）：服务把配不出流的行连同原因写回一个判段读得到的地方，或判段在块数不动时照进度间隔照打一行、满一段时间退 78。

## 第 3 问：不收的几种都归 78

### H3：控制命令表里的「用法错」与异常退出被当成「不收」退 78，这一侧的缺陷不红

- 判段的约定：「退 78（不收）、别的退出码（不认 `serves` 的控制命令退 2 这类）、」（`crates/singlefs-verification/src/crash_amplification.rs:33702`）都当不收；那一行的正文也写「（控制命令不认 serves 的也当不收）」（`crates/singlefs-verification/src/crash_amplification.rs:33736`）。单测把它钉成了期望：「exit_code=2 said=usage: control {start,stop,check}: invalid choice: serves：」（`crates/singlefs-verification/src/crash_amplification.rs:14604`）。
- 同一棵树里的控制命令有 `serves`，它的退出码表把 2 定义成调用方的错：「2 用法错（含 serves 族名不认得、缺参数）」（`gpu-verification-server/control:44`）。族名两边各写一份字面量：控制命令「SERVED_FAMILY_NAMES = ("image-judgement", "facts-verifier")」（`gpu-verification-server/control:253`），判段「Self::FactsVerifier => "facts-verifier",」（`crates/singlefs-verification/src/crash_amplification.rs:33683`）；替身控制命令里的族名取的就是判段自己的那一份（`a_stand_in_control_command`，第 14309 行起），没有一条测试拿判段的族名去问真的控制命令或读它的源码比。
- 走法：任一边改了族名（或判段拼错了版本键的写法、少传一个参数），真控制命令退 2；Python 写错抛异常退 1；两种都进「不收」那一支，打 `DOES_NOT_SERVE … exit_code=2 said=…`、退 78。崩溃放量入口与分项驱动把 78 记运行条件，不判红；出路叫用户「让服务登记这一族这把键」，与病根无关。控制命令路径取错、`python3` 不在 PATH 两种在 `check` 那一下就先退了（同样 78，记「服务不在跑」）；控制命令路径由多机配置所在的目录推（「fn gpu_verification_server_control_command(configuration_file: &Path) -> PathBuf {」，`crates/singlefs-verification/src/crash_amplification.rs:34279`），此刻服务单元的配置写的是仓根那一份（`systemctl --user show -p Environment` 现读），所以这一条此刻走不到。
- 四句：不涉臂；系统看得到——退出码 2 在控制命令的表里只有「用法错」一个意思，判段拿到就分得出；满足第 3 问「有没有一种其实是这一侧的缺陷被归成了环境问题而不红」；跑前没给改法。
- 改法（零轮）：只把「退 78 且标准输出有 `GPU_VERIFICATION_SERVER_SERVES … served=no`」当不收；别的退出码、被信号杀、起不来交回 `Err`（照这一侧的错判红）；删掉「不认 serves 也当不收」那一句与它的单测；加一条单测从 `gpu-verification-server/control` 源码读 `SERVED_FAMILY_NAMES` 与判段的两个族名比。

### H4：`said=` 那一行认得出行头，读的人却没有一个认它（第 3 问后半）

- 拼法：标准输出里 `GPU_VERIFICATION_SERVER_SERVES` 起头的那一行原样放进 `said=`，没有那一行时取标准错误、换行换成「 / 」：「|| standard_error.trim().replace('\n', " / "),」（`crates/singlefs-verification/src/crash_amplification.rs:33728`）。所以换行不会断行，行头始终在行首、认得出。
- 键值层面认不准：真环境的一行（任务 19423 的 `run.log` 第 45 行，量的；第二台上看到的本机地址换成了〔本机地址〕）是 `CRASH_AMPLIFICATION_GPU_VERIFICATION_SERVER_DOES_NOT_SERVE family=facts-verifier expected=0f6a…4db5 control=/home/fy5090/code/singlefs/gpu-verification-server/control exit_code=78 said=GPU_VERIFICATION_SERVER_SERVES family=facts-verifier expected=0f6a…4db5 served=no host.this-host=running/unregistered host.〔本机地址〕=running/unregistered：配了 GPU，…`。`family=`、`expected=` 各出现两次（这一次两份值相同）；最后一项的值后面直接粘着中文正文；`said=` 把两台的地址带进日志（日志不进仓，但判决若整行抄要照共用约束换掉）。单测钉的样子没有 `host.` 那几项：「said=GPU_VERIFICATION_SERVER_SERVES family=facts-verifier served=no：」（`crates/singlefs-verification/src/crash_amplification.rs:14363`），与真控制命令打的不一样。
- 没有一个读者认这一行头：`grep -rnE 'DOES_NOT_SERVE|does-not-serve|does_not_serve' scripts .claude/gate.d task-server` 无输出（退 1，现跑）。后果两处：
  - 崩溃放量：分项驱动认不出类，原样交 78（「*) echo "$run_exit" ;;」，`scripts/crash/crash-amplification-lib.sh:954`），出路打 78 的通用那一句「78) echo "  → 怎么办：修好设备，或改配置：日志里 GPU_CARD_NOT_USABLE 行点名的卡用不了」（`scripts/crash/crash-amplification-lib.sh:977`）。量的：`task-server/task log 19423 --all` 里正是这一句「→ 怎么办：修好设备，或改配置：日志里 GPU_CARD_NOT_USABLE 行点名的卡用不了…」与入口那一行「✗ 分项 fixed-script-through-rollback 的驱动退 78（显卡用不上，或多机配置不满足）」。人看到的出路是查显卡。
  - 崩溃枚举用例（门禁 crash-point-replay）：第 ① 段问到不收，用例那一趟 cargo test 退非 0，门禁照红判，出路是「判红：cargo test 退非 0（上面是它的尾部）」（`.claude/gate.d/crash-point-replay.sh:3542`）与「直接改断言等于把测试关掉」（`.claude/gate.d/crash-point-replay.sh:3544`）。同是「GPU 服务还不收这一族」，观察者盘那一路记本次未跑（78），这一路记红，两处不一致；不过是往严里错，不会记成过。
- 四句：不涉臂；系统看得到（行头是固定常量）；满足第 3 问「那一行还认不认得出」——行认得出，认的人没有；跑前没给改法。改法（零轮）：`crash_amplification_failure_kind` 加一类认 `CRASH_AMPLIFICATION_GPU_VERIFICATION_SERVER_DOES_NOT_SERVE`，出路照那一行自己写的；门禁崩溃枚举用例那一路同样认、定成本次未跑还是红由主 agent 定；`said=` 改成单独一项、值做转义或放到行尾的括号里。

## 第 4 问：测试二进制里 exit(78)：构造不出「没跑完记成过」

- 崩溃枚举用例一条一趟：门禁交的命令是「"--", "--include-ignored", "--exact", case.function, "--nocapture", *test_threads_words]」（`scripts/gate/admission.py:3475`），同一个二进制里别的用例这一趟不跑，exit(78) 杀不到它们。
- 用例里走到退出的那一跳：「match judge_my_blocks_on_gpu(」（`crates/singlefs-verification/src/crash_state_enumeration.rs:2735`）→ 第 34360 行开头的 `check`、或第 34395 行问到不收 → `std::process::exit(78)`。测试二进制退 78，libtest 不打 `test result:`，cargo test 退 101。
- 门禁读到的：退非 0 先过「观察者盘取不到卡」那一类（认不出，不是），再照红判（第 3540–3546 行）；判绿要走 `admission.py crash-case-judge` 看 `test result` 恰好一条通过，这一趟到不了。被停那一类只认信号：「(signal: (2, SIGINT|9, SIGKILL|15, SIGTERM)」（`.claude/gate.d/crash-point-replay.sh:2875`），退 78 不算被停。
- 按条复用那两趟整包跑：登记用例经 `--skip` 排掉；别的用例开一次性库，判段在开头「判段的库是一次性库」那一句交 `Err`，在 exit 之前（第 34357–34358 行）。我没逐个核验证包里每一个不登记、又走共用库的用例（见「限度」）；有的话，exit(78) 会杀掉同一二进制里正在跑的别的用例，已打出的 `ok` 行是真跑完的，cargo 退非 0，整格红，同样不会记成过（推的）。

## 第 5 问：只排计划那一行的后两栏：构造不出

- 次序：只排计划那一支「if placement_only {」（`crates/singlefs-verification/src/crash_amplification.rs:24846`）在读回清单、录入复用数、观察者录入段（只排计划那一趟不录观察者）之后；事实表摘要取的是读回的那一份「let facts_digest_read_back = read_back.facts.digest();」（`crates/singlefs-verification/src/crash_amplification.rs:24819`），核对键照它拼「&facts_verifier_row_key(&gpu_facts_verifier_units, &facts_digest_read_back),」（`crates/singlefs-verification/src/crash_amplification.rs:24858`）。事实表不在库里时读回那一步就报错，到不了数栏，不会数成全部或零；分项驱动「跑不跑只看排计划那一趟的日志里有没有这一行」（`crates/singlefs-verification/src/crash_amplification.rs:34684`），没有这一行照日志分类退。
- 两对栏的关系：第 ① 段按结论版本数（复用口径），第 ② 段按「GPU 核对器组键 + 事实表摘要」那一把核对键数，两把键各自变：判器单元改了而核对器没改，`pending_blocks` 可以大于 `verifier_pending_blocks`；新用例那种库况反过来。两对之间本来就不该有大小关系，代码也没假定；对内的关系（状态数是这几块的状态和）两栏都由同一份块表加出来。没构造出对不上的。

## 第 6 问：切段纯函数

### H5：段长向上取整，拿到的线程在单测构建里有一半闲着、占着份额（轻）

- 拿：「crate::crash_library::InnerThreadsTaken::take(checking_threads().min(image_count).max(1));」（`crates/singlefs-verification/src/crash_amplification.rs:31746`）；单测构建里给的条数不超过要的（`inner_threads_granted` 是 `wanted.min(份额).max(1)`，`crates/singlefs-verification/src/crash_library.rs:580`），所以「拿到的条数大于盘数」从这个调用方走不到，只有纯函数的定义域里有。
- 切：「let piece_length = image_count.div_ceil(threads_taken.max(1)).max(1);」（`crates/singlefs-verification/src/crash_amplification.rs:31758`），一段一条线程。按这个式子算（python 照式子算的，不是跑 Rust）：5 张 4 条起 3 条；7 张 6 条起 4 条；18 张 17 条起 9 条、8 条闲；31 张 17 条起 16 条（单测钉的就是它：「assert_eq!(pieces.len(), 16, "每段 2 张、31 张切 16 段：{pieces:?}");」，`crates/singlefs-verification/src/crash_amplification.rs:14629`）。
- 闲着的那几条在单测构建里记在里层份额上，直到这批盘判完才还；同一进程里别的用例拿里层线程时照它等。产品构建里 `take` 照给、不记账，只是没起。0 张盘：拿 1 条、切 0 段；产品路径上调它之前先判了 `!to_judge.is_empty()`（第 32998 行），走不到。
- 按盘的次序交回：几段连着、升序，`handles` 按段的次序 `join` 再 `flat_map`，不整除时末段短一些，次序照样对。成立。
- 四句：不涉臂；系统看得到（盘数与条数都在手里）；满足第 6 问「拿了而没用的线程占不占着别人的份额」——占；检查单「切段那一处改成照拿到的条数起线程」那一句在不整除时不成立；跑前没给改法。改法（零轮）：段数取 `min(盘数, 拿到的条数)`，前 `盘数 % 段数` 段各多一张（均分），段数恒等于拿到的条数；单测改钉 18/17 起 17 段。

## 第 7 问：只在单测构建里编的那一层恒答收

### H6：「有行才问」那道守卫没有一条测试会红（推的，探针已备）

- 那一层恒答收：「&|_family, _expected_units| None,」（`crates/singlefs-verification/src/crash_amplification.rs:33960`）；核对那一侧的测试同样传「&the_gpu_verification_server_serves_every_family,」（`crates/singlefs-verification/src/crash_amplification.rs:14947`）。
- 专门问「收不收」的五条测试（第 14377、14478、14554、15238、15329 行起）都在有行要写的库况下问。没有一条是「一块都不用判 / 一块都不用核、替身控制命令一问就答不收」。
- 所以守卫「if !rows_to_write.is_empty() {」（`crates/singlefs-verification/src/crash_amplification.rs:34078`）与第 34984 行那一道改成恒真（没行也问），推的结局是每一条测试照绿；变异表里这两处只有「跳过那一问」的方向：「if false {\n        if let Some(line) = server_does_not_serve_line(」（`crates/mutations.tsv:8157`）与「if !blocks.not_yet_verified.is_empty() {」那一行（`crates/mutations.tsv:8159`，替换文也是 `if false {`）。
- 恒真的后果（推的）：整条流都复用的一趟也去问 `serves`；服务不收时整趟退 78，而本来一行都不用写。崩溃枚举用例那一侧「问到不收」：判段那一层的接线由源码钉着（`judge_my_blocks_on_gpu` 整个函数体在第 9877–9895 行逐句钉住），结局退 78 之后门禁怎么判，没有样本、没有测试（H4）。
- 那一层会不会让本该红的单测照过：用它的五条测试测的是等、停、不在跑，不测问；问的那几条直接调 `ask_the_server_then_judge_my_blocks_through_the_pending_rows`。我没找到一条因它照过的；照过的是上面那道守卫的反方向，归 H6。
- 探针：副本里两行变异（下面「要主 agent 跑的命令」第 1 条），守卫改成 `if true {`，跑两段各自的全部相关测试。四句：不涉臂；系统看得到；满足第 7 问；跑前没给改法。改法（零轮）：两段各加一条「全部复用、替身控制命令一问就退 78 且留记号」的测试，断言交回判完、记号文件不在；变异表加「恒问」方向两行。

## 第 8 问：删借草稿区之后

### H7：`NotRunTheGpuMemoryWasShort` 在产品路径上造不出来，两处 match 那一臂与一个布尔指着被删的语义

- 唯一造它的是 `judge_my_blocks_on_the_cards_started_for_this_run`（第 5039、5071、5219 行），唯一调用方当场改成错：「GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { reason, .. } => Err(reason),」（`crates/singlefs-verification/src/crash_amplification.rs:4745`）。产品入口 `judge_my_blocks_on_gpu` 带 gpu 特性只交「Ok(GpuJudgingOfMyBlocks::Judged { tally, outcome })」（`crates/singlefs-verification/src/crash_amplification.rs:34410`），不带特性只交 `Err`（第 34424 行）。
- 所以这几处走不到：「GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason: _ } => {」（`crates/singlefs-verification/src/crash_amplification.rs:24984`）；「GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason } => Err(format!(」（`crates/singlefs-verification/src/crash_state_enumeration.rs:2757`）；「let stop_was_requested = gpu_memory_was_short || stop_is_requested();」（`crates/singlefs-verification/src/crash_amplification.rs:25002`）里前一项恒为假；注释「被叫停（第 ① 段判器显存不够、或等服务核完的时候被叫停的也算）」（`crates/singlefs-verification/src/crash_amplification.rs:25071`）说的那一种此刻没有。两种编法都编得过（不带特性时这个枚举整体允许没用到），不报。
- 第 25002 行那一句另有一条变异行钉着（`crates/mutations.tsv:2968`，改的是后一项），删前一项时要跟着改它。
- 四句：不涉臂；系统看得到；满足第 8 问「造不出来的成员是不是该删」——该删；跑前没给改法。改法（零轮）：`judge_my_blocks_on_the_cards_started_for_this_run` 直接交 `Err`，枚举只剩 `Judged`（或去掉枚举、交元组），删两处那一臂、`gpu_memory_was_short` 与那半句注释。

## 第 9 问：第一轮只抽查过的三处

### (a) 惰性取卡退出时库里留不留半截行：构造不出

- 退出点：建观察者盘的池级 checker 时交进去的退出函数「crate::crash_amplification::exit_because_no_card_is_taken_for_the_images_after_the_observers,」（`crates/singlefs-verification/src/crash_state_enumeration.rs:2705`），在头一张要判的盘那一刻才调；取到一次之后不再取。此前这一趟写库的都是一行一帧、同步写回（`put_one_row_through_the_batch` 每次一批一行，`crates/singlefs-verification/src/verdict_store.rs:4892` 起），观察者结论在判完一批盘之后才写，头一批之前退出时一行都还没写。
- 留下的只有：第 ① 段若被叫停而提早收尾，那几行待判行（完整的行，服务照判）；复用哨兵的强制现判表只在进程里（`also_force_these_conclusion_blocks_to_be_judged_again` 只往内存的表里加），不进库。
- 崩溃放量那一路：配了 GPU 时观察者判段排在第 ② 段之后，此刻第 ② 段问到不收就先退 78，这一支走不到；崩溃枚举用例没有第 ② 段，走得到。这一条现在有两次样本（我第一轮一次、这一轮重核一次），两次都没构造出。

### (b) 等的那一圈自己读库的开销会不会让卡住判法不触发：推不出

- 阈值：「STALL_SECONDS = 600.0」（`task-server/task_server.py:766`）、「STALL_CPU_CORES = 0.01」（`task-server/task_server.py:836`），一个窗口 6 CPU 秒。每圈睡「std::time::Duration::from_secs(2);」（`crates/singlefs-verification/src/crash_amplification.rs:33614`），600 秒 300 圈；每 30 秒一次 `check`，20 次，第一轮主 agent 量的一次约 0.12 CPU 秒，合 2.4 秒（量的）。剩 3.6 秒摊到 300 圈，每圈成批点读剩下的块（一次往返，解每一行）要在 12 毫秒 CPU 以内才判得出卡住（推的）。在第二台上 `check` 还要起 ssh 问另一台，那一份 CPU 没量过。
- 几十块的流肯定在阈值下面；几万块的流每圈解几万行，会不会过 12 毫秒没人量过。H1 之后，「版本对不上」那一种不再走到空等；还走得到空等的只剩 H2 那一种，所以这一项只在 H2 上有用。注释那一句「（对不上版本）时，这个进程的日志与 CPU 都不涨，统一任务服务才判得出它卡住」（`crates/singlefs-verification/src/crash_amplification.rs:33617`）说的情形现在由 `check` 退 1 接住了，注释也该改成 H2 那一种。

### (c) 问服务的子进程在第二台上起：构造不出（量的，一次样本）

- 任务 19423 就是派到第二台跑的：`task-server/task log 19423 --all` 有「MULTI_HOST_RUN_STEP step=peer-command event=begin elapsed_seconds=2.5」与「MULTI_HOST_RUN_STEP step=peer-command event=end elapsed_seconds=169.1 seconds=166.6 exit=78」；`run.log` 第 45 行 `said=` 里另一台写的是本机的地址（我在本机 `hostname -I` 里比对过，地址本身不抄）。
- 第二台上起的控制命令两台都问：`host.this-host=running/unregistered host.〔本机地址〕=running/unregistered`。同一趟第 ① 段 `check` 过了（`check` 要每一台都在跑、都没有对不上行）、`serves image-judgement` 收了，`run.log` 第 43 行「CRASH_AMPLIFICATION_GPU_TOTAL cards_used=0 blocks=32 states=141」。所以第二台上走的就是两台都问那一支，ssh 从第二台连得回本机。
- 只一次样本；`systemctl --user` 在第二台任务进程里能不能用、ssh 失败时会怎样，这一次都没碰到。

## 八格检查单

| 格 | 判定 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 在 `git show HEAD:crates/singlefs-verification/src/crash_amplification.rs` 的输出里按名字 `judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from` 找，命中 6 行：第 4931 行是唯一的调用（在 `judge_my_blocks_on_gpu_cards` 里），第 5263 行是定义（`fn`，不是 `pub`），其余 4 行是注释与源码钉住的字符串；第 4939 行传的是 `GpuCardsSource::DevicesStartedForThisRun,`。`git grep` 别的文件零命中。卡池那一臂没有第二个调用方 |
| 类型与名字 | 不成立 | 第一轮第 4 条照旧（正文已认）。新加的几处另有一处：族名是两边各写一份的字面量（H3），没有类型、没有测试把它们绑在一起；`PlacementOfTheRun` 两对栏都是裸 `usize`/`u64`，构造处用具名字段，没找到串用 |
| 不兼容旧代码与旧格式 | 不成立 | 「（控制命令不认 serves 的也当不收）」（`crates/singlefs-verification/src/crash_amplification.rs:33736`）与第 14554 行那条单测只为没有 `serves` 的控制命令留；同一棵树里控制命令有 `serves`，这一支是给旧版本留的放宽，按「里程碑 17 收尾之前不兼容任何旧东西」（`.claude/agent-common.md:41`）该删（H3）。「pub(crate) fn judge_my_blocks_through_the_pending_rows(」（`crates/singlefs-verification/src/crash_amplification.rs:33941`）是为旧签名留的，我判也算，删它已登账（正文） |
| 不新建判定库 | 成立 | 问服务只读控制命令的退出码与标准输出；计划行只打印；没看到新的存放处 |
| 所有任务多机可用 | 成立（一次样本） | 见 9 (c)。另记 H1 末段：`serves` 至少一台收就收、`check` 每一台都要过，两台二进制不同时判段会在服务替它判着的时候退 78 |
| 测试优先多线程 | 不成立 | H5：18 张盘拿 17 条只起 9 条，「照拿到的条数起线程」在不整除时不成立 |
| 带 --item 的命令与项 | 成立 | `task-server/task log 19409` 末段「✓ 点名的 17 条变异都抓到了（只判了这几行，不是整道结论；现判、复用、拒算各几条见上面「复用：」那一行）」。另记 H6：两道「有行才问」的守卫没有「恒问」方向的变异行 |
| 新崩溃流多罩了什么 | 成立 | 新用例断言「(Some(0), Some(0), Some(state_count)),」（`crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1540`），另断言 `verifier_pending_blocks` 大于 0；挂 `gpu` 特性，只在带特性那一趟跑 |

## 改法各修哪一格（都只在我的推演上、被攻过零轮）

| 改法 | H1 孤行与 4 小时 | H2 答收不判 | H3 退 2 当不收 | H4 没人认行头 | H5 切段 | H6 守卫无测试 | H7 死臂 |
|---|---|---|---|---|---|---|---|
| 等的那一圈与开头都改问 `serves <族> <键>`，开头那一问挪到排出了行之后 | 修（推的） | 不修 | 不修 | 不修 | — | 要随之加测试 | — |
| 给孤行一条有点名的删除路 | 修库里那一半（推的） | 不修 | — | — | — | — | — |
| 块数不动也照间隔打进度行，满一段退 78 | 不修 | 修（推的） | — | — | — | — | — |
| 只把「退 78 且 served=no」当不收，别的交 `Err`；族名从控制命令源码比 | — | — | 修（推的） | — | — | — | — |
| 分项驱动与门禁认 `DOES_NOT_SERVE` 行头 | — | — | 部分（出路对了，退 2 照旧 78） | 修（推的） | — | — | — |
| 均分切段，段数等于拿到的条数 | — | — | — | — | 修（推的） | — | — |
| 两段各加「全部复用、一问就不收」的测试与「恒问」变异行 | — | — | — | — | — | 修（推的，待主 agent 跑第 1 条） | — |
| 删 `NotRunTheGpuMemoryWasShort` 与两处那一臂 | — | — | — | — | — | — | 修（推的） |

## 没打中的形状

- 第 1 问：服务停（不换键）——行完整、照键留着，服务起来照判，下一趟复用；写到一半停——写行不经 GPU 服务，停了照写完；`serves` 答收之后 `serving` 那一行被换掉——控制命令照没在跑答，本趟下一次 `check` 退 78。三种都不留孤行。
- 第 2 问：上一趟留下同键、强制现判的行，这一趟不强制——`AlreadyPresent` 交回库里那一份，这一趟照强制现判核凭据；服务照行判、留凭据，核得过。排出空——不问、不写、不等。
- 第 4 问：登记用例 `--exact` 一条一趟、门禁只认信号算被停、判绿要恰好一行 `test result`；整包两趟用一次性库的用例在 exit 之前交 `Err`。
- 第 5 问：录入前后、事实表不在库里、两对栏之间的大小关系，见第 5 问一节。
- 第 6 问：0 张盘、拿到的多于盘数——前者产品路径先判了非空，后者 `take` 不给多于要的。
- 第 9 问 (a)：退出点之前的写都是整帧；强制现判表只在内存里。
- 取样范围：`crash_amplification.rs` 第 33600–35360 行（两段等服务与只排计划）、第 31700–31810 行（切段）、第 24560–25110 行（一趟的主干），`crash_state_enumeration.rs` 第 2190–2780 行，`gpu-verification-server/control` 第 1–90、380–470、820–1240 行，`gpu-verification-server/src/library_pending_pool.rs` 第 130–170、540–760 行，`crash-point-replay.sh` 第 318–357、2860–2880、3490–3555 行，`crash-amplification-lib.sh` 第 40–60、940–980 行。

## 这条腿自己的限度

- 没跑任何东西：H1、H2、H3 的退出码与末行是读代码推的；H4 的错出路与 9 (c) 是读主 agent 跑出的任务 19423 的日志（量的，一次）。H6 的「没有一条会红」只核了两段各自调到那两个函数的测试（第 1 条命令点名的 16 条），没跑整个验证包库测试。
- 第 4 问末段「整包两趟里没有不登记、又走共用库的用例」没逐个核：调 `enumerate_crash_states` 一族的文件有 57 份，登记的崩溃枚举用例 108 行，我没把两张单子逐条对上。
- H1 没在真服务上复现：要在服务上换一把键，会打断主 agent 正在用的服务，没写成命令；第 2 条只读此刻的 `check`，看库里是不是已经有孤行。
- 9 (b) 只算了阈值，没量块多的流上一圈读库的 CPU。
- 9 (c) 只一次样本，`systemctl --user` 在第二台任务进程里出错、ssh 回本机失败两种没碰到。

## 要主 agent 跑的命令

1. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=证红脚本在攻方腿副本里判两行探针变异; 范围=2 行变异; 为什么=量有行才问那两道守卫改成恒问时相关库测试里有没有一条会红，攻方腿第七问' task-server/task submit -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r2-opus/copy singlefs-verification '攻方腿探针 写待判行之前那一问不看有没有行要写：第 ① 段一律问' '攻方腿探针 写核对待判行之前那一问不看有没有块要核：第 ② 段一律问'`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`（`--copy` 指副本；副本 `crates/mutations.tsv` 末尾追加了这两行，sha256 `d0921aca446b40362fb233438c7215e1939cc99371c02be748d534338acfffc7`，副本在草稿目录 `.deliverables` 写成「交接」）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=`；`task-server/task log <号> --all` 里两行探针各自的结局。推的：两行都是「没红」（点名的测试照绿），计数行写没红 2 条、`exit` 非 0。哪一行是「抓到」，H6 在那一段就不成立。
2. `task-server/task submit -- python3 gpu-verification-server/control check`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：`exit=0` 是此刻没有对不上的行；`exit=1` 且日志里有「期待的键：GPU 判器组 …；这一份二进制的键：…」那几行，说明库里此刻已经有别的键留下的待判行、H1 的「开头就退 78」正在发生，记下那几组键与行数。
3. 交回前我直接跑了 `python3 scripts/agents/cite-check.py research/prompts/amplification-without-cards-code-r2-opus-output.md`（文本工具），末行贴在交回里。

## 没做什么

- 不判辩方腿的格，没读禁读清单里的报告与草稿目录；不替主 agent 采纳。
- 没往统一任务服务交任务；H1、H2、H3、H6 的读数是推的，判决要引的数由主 agent 跑上面第 1、2 条。没写模型，没有模型目录与复跑脚本。
- 副本 `/home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r2-opus/copy`（57M）留着交主 agent 跑第 1 条，在 `.deliverables` 写成「交接」；用完由主 agent 删。读 HEAD 那一份 `crash_amplification.rs` 拷进草稿目录的单个文件（1.7M）与一份调用方清单已删。
- 自己提的改法都只在我的推演上，被攻过零轮。
