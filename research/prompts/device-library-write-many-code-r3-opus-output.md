# device-library-write-many-code-r3 攻方腿（Opus）报告（2026-11-10）

立场：假设前两轮站住的形态是错的，换前两轮没用过的输入、变体与读数去打它。被判文件 `crates/singlefs-device-library/src/lib.rs`，sha256 `773f90a766cb5826a81ffe87d14345c56f49d1876b7a99100c20ea33d3b8bff8`，与开工快照 `research/prompts/device-library-write-many-code-r3-snapshot/crates-sha256.txt` 那一行相同（量的：`sha256sum` 现跑，主工作区与我建的冻结副本各一次）。转交一侧 `scripts/experiment-apparatus/device-code-key.py` 只当上下文读（主工作区此刻那一份，不在快照里）。

本腿没往任务服务交任何任务。下面凡是模型「跑出来」的数都标「推的，待主 agent 跑」；标「量的」的只有三类：`sha256sum`、`grep` 这类只读命令现跑的输出，主 agent 给号的七个任务的 `task-server/task log <号> --all` 与 `task-server/task status <号>`（原样拷在草稿目录 `/home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r3-opus/task-<号>.log`、`status-<号>.txt`）。

## 复跑命令与模型文件

模型目录 `research/prompts/device-library-write-many-code-r3-opus-model/`。复跑（外面一层先删运行目录再调 `tools/rerun.sh`；照判决规则在模型目录的拷贝里起时，把两个模型路径换成拷贝的）：

`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/device-library-write-many-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r3-opus/copy /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r3-opus/model-run /home/fy5090/code/singlefs/research/prompts/device-library-write-many-code-r3-opus-model`

模型里每个文件的 sha256（量的：模型目录里 `find . -type f | sort | xargs sha256sum`）：

| 文件 | sha256 |
|---|---|
| `drafts/appended-tests.rs` | `b6bf50aaa404e619b2f847e1f762f9dedd68af78b4f0022a26faa4abf6bc6b09` |
| `tools/apply_variant.py` | `fe7cd91a5793833b26f90db5633d8c037cdb74660a069d4a37a26d39f1aaad6b` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps` 那一段） | `68b93589014d1af12e956dec674c3f4f54158515a260904e013d82506582757c` |
| `variants/contents-not-utf8-decoded-lossily/anchor` | `175d392fdb1f62e90aaa0bdc29e44a025d6dee329ee81431c5879b71d491cac9` |
| `variants/contents-not-utf8-decoded-lossily/replacement` | `0b9ce16fdec7354f040ce564e01e167de12d21f6519c806c3f381c1c450b2917` |
| `variants/read-answered-failed-taken-as-absent/anchor` | `e4249ca4957adf6562fc36a07d173fe5d47d22695d97cb833644784b4e67e3b9` |
| `variants/read-answered-failed-taken-as-absent/replacement` | `61961df84cbb5b0076cd53a1e54f31389397a4f0f62c9b85a4eeec6a0a8745a1` |
| `variants/read-contents-without-filling-zeros-first/anchor` | `c2dbc6401bd29c0286165c2370e256251c32c96553b3dce861df25c7c9891520` |
| `variants/read-contents-without-filling-zeros-first/replacement` | `fd483e8178e977a181f99b5a56e7cb63232b68a081aa11d8342ab53d62fde320` |
| `variants/write-key-checked-in-the-relay-arm-only/anchor` | `324b0cd41b3b488f037568925599309bf4040452ba530f2ba76e0d7ba929c820` |
| `variants/write-key-checked-in-the-relay-arm-only/replacement` | `32e768ad9d14819120c38c6174e1b2c77f8646322c2aa5ad80f0714d01167f8a` |
| `variants/write-lock-released-between-header-and-contents/anchor` | `43cc4793312d9faca84bd8b437cf1856c7b87e0410a7b5fe5ce8e42e3f5459a4` |
| `variants/write-lock-released-between-header-and-contents/replacement` | `cf971561b57d0fa41329ede5f3d931701e924d3257c7e0a79efada98e7c527fb` |

模型做的：每个变体从冻结副本拷一棵树、一个编译目录；`tools/apply_variant.py` 在 `lib.rs` 的 `mod tests` 末尾接上 `drafts/appended-tests.rs` 的四条草稿测试，变体另换上一处原文（原文在 `lib.rs` 里恰好命中一次，开工时量过五份都是 1、在草稿测试里都是 0）；每趟跑 `cargo test -p singlefs-device-library --lib -- --show-output`（内带 target 的范围声明），打 `name=round`、`name=test`、`name=resident`、`name=variant_summary` 几种行。六个变体：`unmodified`（只接草稿测试）3 趟，四个弄坏的变体各 3 趟（放锁那一个 5 趟），我提的改法 `read-contents-without-filling-zeros-first` 3 趟。不涉及崩溃状态（这一题没有崩溃点重放流与崩溃枚举），第 3b 步的状态数不适用。挂钟推的：六次拷 53M 的树、六次只编本包、二十趟每趟一秒到四秒，合计两分钟上下，没先跑小段估（我不交任务）。

跑前写下的期望（推的）与「反过来我怎么改」：

| 变体 | 期望 | 反过来时 |
|---|---|---|
| `unmodified` | 原有 21 条全绿；三条草稿测试（内存 `write` 坏键、`read` 回 `failed`、非 UTF-8）绿；常驻内存那一条红，`growth=` 接近 33554432 | 常驻内存那一条绿，就撤回打中乙「内容来之前整段写零」那一半 |
| `write-key-checked-in-the-relay-arm-only` | 原有 21 条全绿（`original_failed_rounds=0`）；草稿 `in_memory_write_…` 3/3 红 | 原有有一条红，撤回打中甲 |
| `read-answered-failed-taken-as-absent` | 原有全绿；草稿 `read_answered_failed_…` 3/3 红 | 原有有一条红，撤回打中丁 |
| `contents-not-utf8-decoded-lossily` | 原有全绿；草稿非 UTF-8 那条 3/3 红 | 原有有一条红，撤回打中戊 |
| `write-lock-released-between-header-and-contents` | `a_write_holds_…` 5 趟里多数绿（探锁罩不到请求写到一半那一段） | 5 趟里有 3 趟以上红，撤回丙 |
| `read-contents-without-filling-zeros-first` | 原有 21 条与四条草稿全绿，`growth=` 远小于 16777216 | 原有有一条红或常驻内存仍涨，撤回这个改法 |

## 各格判定一览

| 格 | 判 | 一句话 |
|---|---|---|
| 攻击面一：键规则在 `read`、`write`、`write_many` 三处是不是同一个函数、同一个次序 | 实现成立；**打中甲（小，零轮）** | 三处都调同一个 `give_up_unless_the_relay_accepts_the_key`，都在 `match &*self.backend` 之前、拿锁之前。测试不对称：`read` 与 `write_many` 的内存那一种各有一条坏键测试，单条 `write` 没有；把 `write` 的核键挪进转交那一支（r1 打中 4 的形状），推的：现有 21 条全绿，待主 agent 跑 |
| 攻击面一：`try_reserve_exact` 失败与 `isize::MAX` 过滤两条路分不分得开 | 两条路成立；**打中乙（小，零轮）** | 两句话分得开，两条测试与两行变异各钉一条。另有第三条路前两轮没看：预留成了而物理内存没有，`resize` 在内容来之前整段写零，服务给的内存上限里被 OOM 杀掉、不退 3，文件头第 15 行「分配不出都退 3」外推；转交走不到 |
| 攻击面一：四条探锁测试 | 成立；**丙（零轮，推的待跑）** 罩不到的一段 | 读数核得上（任务 12234）。四条只罩「请求写完到回应读完」；「请求写到一半放锁」那一段（`write` 先写条头、放锁、再写内容）探锁一方多半抢不到，模型变体量 |
| 攻击面二：`pub` 入口与五种回应 | **打中丁（小，零轮）**；**戊（小，零轮）** | 丁：`read` 收到 `failed` 没有测试；把它当 `absent` 的变异（转交那一侧同形的弄坏开关是 `relay-failure-reported-as-absent`）推的现有 21 条全绿。戊：非 UTF-8 交 None，r2 只把文档改成实话，行为没钉测试也没有变异行 |
| 攻击面二：其余入口（`from_environment`、`in_memory`、`count_*`、`Clone`/`PartialEq`/`Debug`） | 没打中 | 见「没打中的形状」 |
| 攻击面三：第二节每个读数逐字找得到 | 成立（量的） | 七个任务日志里都找得到；12233 那一句是整行的前缀（整行后面还有「（其中复用 0 条），本次未跑 0 条，跑了的都抓到了」），意思没走样 |
| 攻击面三：「第二节的任务都派在第二台跑过」 | **不成立（量的）** | 七个任务里只有 12232（变异行静态核）、12235（E158）在第二台；12233、12234、12236、12237、12238 都在本机。这一批的 21 条测试（含三条新探锁测试）一次都没在第二台跑过 |
| 攻击面三：`rounds.sh`「每趟一个编译目录」 | 成立（源码与用时，推的那一半见正文） | 源码每趟 `CARGO_TARGET_DIR` 给 `target-round-<趟>`、跑完删；任务 12234「编译目录 无」，服务没给编译目录、不会盖掉它；每趟的 cargo 日志随 `rounds-run-1` 删了，`Compiling` 行没法逐趟核，复核命令交主 agent |
| 检查单：走不到的理由二次查实 | 成立 | 产品段新加的只有可失败预留与核键；预留成了之后 `resize` 不再分配 |
| 检查单：类型与名字 | 成立 | 无 `_ =>`（量的 grep 0 处）；新名字不带序数；`expect` 理由写了凭什么 |
| 检查单：不兼容旧代码与旧格式 | 成立 | 背景给的 grep 命中 5 行，全是测试名里的 `holds`（量的） |
| 检查单：不新建判定库 | 成立 | 本轮改动只有核键、可失败分配与测试 |
| 检查单：所有任务多机可用 | **推不出（打中）** | 新测试只读 `current_exe`、一个环境变量与管道，不读本机独有状态（成立那一半）；可第二节说派在第二台跑过的那一半是错的（攻击面三），三条新探锁测试在第二台的抓到率与不误报一次都没量 |
| 检查单：测试优先多线程 | 成立 | 探锁测试本身三线程；libtest 在测试函数之间并行；多轮量法五个变体各五趟。「够不够」只在本机量过，归上一格 |
| 检查单：带 --item 的命令与项 | 成立（推的一半：`--list-items` 没跑） | 本包 25 行静态核：每行六段、原文在 `lib.rs` 里恰好命中一次、点名测试的函数在（量的，见正文）；12233 点名十条都找到；`--list-items` 交主 agent |
| 检查单：新崩溃流多罩了什么 | 成立 | 改动全文里 `checker`、`crash` 0 处（量的） |

打中五处（甲乙丁戊与多机那一格）都是「小」：现有调用方与转交都走不到，或只影响测试罩不罩得住；没有一处让判定库里落下错的判定。丙是推的，等变体读数。

## 攻击面一：「已定」项撑不撑得住

### 键规则三处：同一个函数、同一个次序（成立）；打中甲：单条 `write` 的内存那一种没钉

三处调的是同一个函数、在同一个位置：`read` 是「        give_up_unless_the_relay_accepts_the_key(relative, &asked);」（`crates/singlefs-device-library/src/lib.rs:192`），`write` 是「        give_up_unless_the_relay_accepts_the_key(relative, &asked);」（`crates/singlefs-device-library/src/lib.rs:246`），`write_many` 是逐条「            give_up_unless_the_relay_accepts_the_key(relative, &asked);」（`crates/singlefs-device-library/src/lib.rs:271`）。三处都在 `match &*self.backend` 之前，转交与内存两支一起核，拿锁之前退。唯一的次序差别是 `write_many` 先判空批再核键，空批里没有键，不影响结局。实现这一半成立。

钉住它的测试不对称（量的：`grep -nE '\.write\("' crates/singlefs-device-library/src/lib.rs` 六处，内存那一种上的单条 `write` 只有第 477 行一处、键是合的）：

| 入口 | 转交那一种的坏键测试 | 内存那一种的坏键测试 | 「只在转交那一支核」的变异行 |
|---|---|---|---|
| `read` | 第 761、785 行两条 | 「    fn in_memory_read_with_a_key_the_relay_refuses_exits_with_the_other_error_code() {」（`crates/singlefs-device-library/src/lib.rs:808`） | 「判定库口子 read 只在转交那一种核相对键」（`crates/mutations.tsv:7607`） |
| `write_many` | 第 615 行 | 「    fn in_memory_write_many_with_a_key_the_relay_refuses_exits_with_the_other_error_code() {」（`crates/singlefs-device-library/src/lib.rs:642`） | 没有，但内存那条测试会抓 |
| `write` | 「    fn write_with_a_key_the_relay_refuses_exits_with_the_other_error_code_before_sending_a_byte() {」（`crates/singlefs-device-library/src/lib.rs:591`） | **没有** | **没有**；只有整条删掉的「判定库口子 write 不核相对键」（`crates/mutations.tsv:7590`），它由转交那条测试抓 |

具体输入：把 `write` 的核键挪进 `Backend::Relay` 那一支（模型变体 `write-key-checked-in-the-relay-arm-only`），内存那一种的 `write("items/bad key.out", "x")` 走「                    .insert(relative.to_string(), text.to_string());」（`crates/singlefs-device-library/src/lib.rs:259`）收下、不退。这正是 r1 判的「**打中**：内存那一种不核相对键，单测绿、真跑退 3」（`research/prompts/device-library-write-many-code-r1-main-verification.md:26`）在单条 `write` 上的形状。推的：现有 21 条全绿（转交那条测试走转交那一支，照样退 3），草稿测试 `in_memory_write_with_a_key_the_relay_refuses_exits_with_the_other_error_code` 红；待主 agent 跑模型。

四句：不涉及臂（代码轮，判的是测试罩不罩得住一处已修的打中）；被判的系统（变异门禁）看得到判别它的东西，只要有一条喂内存那一种单条 `write` 坏键的测试；满足的是 r2 交实现员第 1 条承接下来的 r1 第三节第 1 条「`write`、`write_many` 两种、转交与内存两个后端都核」里「内存后端的 `write`」那一个分句——实现做到了，测试与变异行没跟上；r1、r2 给的改法（加测试与变异行）照形状补一条就修到这一格。推翻它的现象：模型变体 `original_failed_rounds` 不是 0。

### `try_reserve_exact` 与 `isize::MAX` 两条路分得开（成立）；打中乙：第三条路「预留成了而内存没有」

两条路分得开：过 `isize::MAX` 的在「                // 报的字节数超过 isize::MAX 的不是任何一个值的长度：当认不出的回应退掉。」（`crates/singlefs-device-library/src/lib.rs:209`）那一段走 `give_up_on_reply`，打「回应认不出」，第 730 行「            outcome.standard_error.contains("回应认不出"),」（`crates/singlefs-device-library/src/lib.rs:730`）钉住，变异 7596 由它抓；没过 `isize::MAX` 而预留不成的打「                        &format!("{asked}：回应报的 {length} 字节在这个进程里分配不出：{error}"),」（`crates/singlefs-device-library/src/lib.rs:222`），2^62 那条测试只钉退出码 3、不钉这一句（把两句合成一句的改法在现有测试下不红，记一笔，不算打中）。

第三条路前两轮没看：`try_reserve_exact` 成了只说明虚地址与内核的提交估算放行，不说明物理内存在。随后「                contents.resize(length, 0_u8);」（`crates/singlefs-device-library/src/lib.rs:225`）在内容一个字节都没来之前把 `length` 个字节整段写零，页一页一页落进常驻内存。具体输入：回应 `found 1073741824\n`，后面不跟内容（或跟得很慢）。本机 `vm.overcommit_memory` 是 0、`MemTotal: 63033412 kB`、`SwapTotal: 8388604 kB`（量的：读 `/proc/sys/vm/overcommit_memory` 与 `/proc/meminfo`），启发式下 1 GiB 的单次预留会放行（推的，按内核启发式「单次不超过内存加交换」）；本包测试任务 12238 的状态是「本机、上限 256M」（量的：`task-server/task status 12238`），写零写到 256 MiB 时服务给的内存范围里 OOM 杀掉测试进程：被信号杀掉、退出码不是 3。这与文件头「//   管道断了、回应认不出、found 报的字节数在这个进程里分配不出，都退 3。」（`crates/singlefs-device-library/src/lib.rs:15`）不符：在这个进程的内存范围里确实分配不出，退的却不是 3。

可达性：转交一律把内容与长度一起写，「            return f"found {len(data)}\n".encode() + data」（`scripts/experiment-apparatus/device-code-key.py:658`），长度就是库里那一份的真实字节数；真值大到撞上限时，内容照样要读进来，常驻内存的峰值与写不写零无关。所以走不到，代价只在两处：每次 `found` 多一遍写零（推的，一遍 memset，读大值时可见），文件头那一句外推。与 r2 判的「长度没过 `isize::MAX` 而分配不出时进程 SIGABRT、不退 3」同一类、同一把尺子（转交不会回这种长度），是那一处修补没盖到的另一半。

量法（推的，待主 agent 跑）：草稿测试 `read_answered_found_touches_no_more_memory_than_the_contents_that_arrived` 在本进程里报 `found 33554432`、内容不写，等常驻内存涨过一半或满三秒，记下涨了多少，再写全内容收尾；断言内容来之前涨不过报的一半。期望：`unmodified` 那一行 `name=resident … growth=` 接近 33554432、测试红；我提的改法（下一段）那一行远小于 16777216、测试绿。量的是常驻内存，不去撞上限（不让服务看到一次 OOM）。

我提的改法（只在我的模型上量过——还没量，推的、被攻过零轮）：预留照旧，不写零，用 `Read::take(length).read_to_end` 读进预留好的那一块，读到的字节数不等于 `length` 照「没读全」退 3（模型变体 `read-contents-without-filling-zeros-first`）。它修乙这一格；变异表 6983 行的锚点 `contents.resize(length, 0_u8);` 随之没了，要换一行「少读一个字节」（例：`take(announced)` 换成 `take(announced.saturating_sub(1))`，点名同一条测试）。

四句：不涉及臂；被判的系统在做决定的那一刻看不到 cgroup 还剩多少，`try_reserve_exact` 的成败分不出「有物理内存」与「没有」——这是「判别子观测不到」那一形，改法不去判它，改成只在内容到了才占内存；满足的是文件头第 15 行「found 报的字节数在这个进程里分配不出，…都退 3」那一个分句；r2 第三节第 2 条的改法（可失败预留）在这一格上照样中（预留成了），我提的改法推的不中。推翻它的现象：`unmodified` 那几行 `growth=` 小于 16777216。

### 四条探锁测试（成立）；丙：罩不到「请求写到一半」那一段（零轮，推的待跑）

读数核得上：任务 12234 四个放锁变异各 `named_failed=5 any_failed=5 of=5`、没改坏的 `any_failed=0 of=5`（量的，见攻击面三）。没改坏时不红的推理（锁一直握着，探锁一方抢到锁时最后一段回应一定已记上）我逐步读过，成立。

罩不到的一段：探锁一方只在「            spin_until(|| progress.request_begun.load(Ordering::SeqCst) >= round_number);」（`crates/singlefs-device-library/src/lib.rs:997`）之后才空转抢锁，而 `request_begun` 要替身先读到请求开头 5 个字节才记（「                    progress.request_begun.store(round_number, Ordering::SeqCst);」（`crates/singlefs-device-library/src/lib.rs:944`））。装置线程若在请求写到一半放锁、再拿锁（例：`write` 先写条头 `write items/a.out 262144\n`、放锁、再拿锁写 256 KiB 内容），放锁那一刻替身多半还没被调度起来读开头，探锁一方还在等 `request_begun`、不在抢锁，空当过去了。今天的实现没有这一段：整条请求先拼进一个 `Vec`，「                let mut request = format!("write {relative} {}\n", text.len()).into_bytes();」（`crates/singlefs-device-library/src/lib.rs:250`）之后一次 `write_all`。所以这不是实现的错，是文件头「整问整答在一把锁里（从写请求到读完回应、连同 found 后面的内容一直握着，别的线程只等这一问）」（`crates/singlefs-device-library/src/lib.rs:5`）这一句比四条测试罩的宽：将来把 `write_many` 改成边拼边写（大批省一份内存，是个顺手的改法），锁要是跟着按条放，现有测试推的抓不到。

模型变体 `write-lock-released-between-header-and-contents` 量 `a_write_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply` 五趟（推的：多数绿）。这是零轮形态，照规则不为它开轮，交判决「零轮」一栏。

## 攻击面二：未定项清单的完整性

前两轮列过的：`write`、`write_many` 的 `written` 与 `failed`、`read` 的 `found` 长度与内容、`absent`、相对键、`give_up`、四处放锁、`count_*` 的 `panic!`、两次 `from_environment`。逐个入口、逐种回应对一遍，下面两处前两轮没列成攻击面（或列了、修补之后没钉）。

### 打中丁：`read` 收到 `failed` 没有测试（零轮）

文件头写三种请求都可能回 failed：「//   三种都可能回 failed <退出码> <一句>\n：这里打 ✗ 与那一句、进程照那个退出码退（75 是 prune-server 不可用，3 是别的错），不当「键不在」现判。」（`crates/singlefs-device-library/src/lib.rs:12`）。`write` 与 `write_many` 各有一条喂 `failed 75` 的测试（第 661、685 行），`read` 没有（量的：`grep -nE 'failed 75'` 三处喂 `failed 75` 的都是写）。`read` 收到 `failed` 走的是 `let … else` 里的「                    give_up_on_reply(&reply, &asked)」（`crates/singlefs-device-library/src/lib.rs:215`），前面先判「                if reply == "absent" {」（`crates/singlefs-device-library/src/lib.rs:206`）。

具体输入：变异把第 206 行写成 `if reply == "absent" || reply.starts_with("failed ") {`（模型变体 `read-answered-failed-taken-as-absent`）。推的：现有 21 条全绿——喂 `read` 的回应只有 `found 5`、`absent`、过大与分配不出的 `found`，过大那条走 `let … else`、不经这一判；草稿 `read_answered_failed_exits_with_the_code_in_the_reply` 红。转交那一侧有同形的弄坏开关，「  DEVICE_CODE_KEY_BREAK=relay-failure-reported-as-absent  读服务没成时回 absent（装置当不在、现判）」（`scripts/experiment-apparatus/device-code-key.py:125`），装置这一侧没有对应的钉子。

后果（推的）：服务在一串读的中途停了，装置把每条都当不在、去现判；转交「此后每个请求都照它回」，下一次写收到 `failed 75` 照退。装置最后几步若只读不写、退 0，转交那头「        if device_code != 0 or relay.failure is not None:」（`scripts/experiment-apparatus/device-code-key.py:772`）之后交的是装置的 0。判定照样对（都现判了），损失是现判的时间与「服务不可用」被吃掉。所以是小打中。

四句：不涉及臂；变异门禁看得到它，只要有一条喂 `read` 的 `failed`；满足的是文件头第 12 行「三种都可能回 failed …不当「键不在」现判」里「三种」那一个分句（读那一种）；r1 第三节第 5 条（收进 `*_answered_failed_*` 两条）只修了写的两种，在读这一格上不中。推翻它的现象：模型变体 `original_failed_rounds` 不是 0。

### 戊：非 UTF-8 的内容交 None，文档改成了实话，行为没钉（零轮）

r2 判「**打中（文档）**：文档句与行为不一致，改文档或改行为二选一」（`research/prompts/device-library-write-many-code-r2-main-verification.md:34`），这一批选了改文档：「    /// 读一条：不在（或已撤掉）交 None，回的内容不是 UTF-8 也交 None；相对键不合、读失败照退出码退，不交 None。」（`crates/singlefs-device-library/src/lib.rs:189`）。可这句现在是对外的约定，却没有一条测试喂非 UTF-8 的内容，也没有变异行（量的：`grep -nE '\\xff|from_utf8'` 只命中产品段第 232 行与测试里读子进程标准错误的 `from_utf8_lossy`）。

具体输入：把「                String::from_utf8(contents).ok()」（`crates/singlefs-device-library/src/lib.rs:232`）换成 `Some(String::from_utf8_lossy(&contents).into_owned())`（模型变体 `contents-not-utf8-decoded-lossily`）。推的：现有 21 条全绿；草稿 `read_answered_found_with_contents_that_are_not_utf8_gives_none_and_takes_exactly_those_bytes` 红。后果（推的）：库里一份坏了字节的值被换成 U+FFFD 交给装置，装置当命中复用，不再现判——这正是文档那一句要防的。可达性低：本包只写 `&str`，同族别的写者才可能写进非 UTF-8；分段值拼不回由客户端退 3（不交字节）。

四句：不涉及臂；看得到；满足的是第 189 行「回的内容不是 UTF-8 也交 None」；r2 改法（改文档）在「钉住」这一格上不中，加一条测试与一行变异就中。与 r2 攻方「内容不是 UTF-8 时交 None」那一条的差别：那一条判的是文档与行为不符，这一条判的是修补之后行为没人看着。

## 接续说明与判定一览补一行

前一条攻方腿写到攻击面二「戊」为止（上面 129 行一字没改）。下面是同一立场的接续腿写的：攻击面三的正文、检查单八格的正文、没打中的形状、限度、要主 agent 跑的命令与没做什么。接续时核过现场（量的）：模型目录 14 份文件的 sha256 与上表逐个相同；`tools/rerun.sh` 与草稿目录里的 `rerun.sh.composed` 逐字节相同（`cmp`）；`bash -n` 两份 shell、`py_compile` 一份 python 都过；五个变体的原文在冻结副本 `lib.rs` 里各命中 1 次、在草稿测试里 0 次；冻结副本 `lib.rs` 的 sha256 是 `773f90a766cb…`，与开工快照同。七个任务的 `task-server/task status` 现跑一遍，与前一条腿存的 `status-<号>.txt` 逐字节相同（`cmp`）。

接续时新冒出一格，补进判定一览（零轮）：

| 格 | 判 | 一句话 |
|---|---|---|
| 攻击面三：第二节的读数跑在哪一版 `lib.rs` 上 | **打中己（小，零轮）** | 任务 12233、12234 开跑在主 agent 把别名 `AskTheLibrary` 改进主工作区之前，跑的是补丁打上、别名还没改的那一版（推的 sha256 `14b72704…`），不是开工快照那一版 `773f90a7…`；两版只差别名那几行（量的 diff），日志里不留树的哈希，单看日志核不到 |

## 攻击面三：那一轮报出的数字能不能核

### 第二节每个读数逐字找得到（成立，量的）

在草稿目录里的日志拷贝（前一条腿用 `task-server/task log <号> --all` 拷出）上现跑（量的）：

```
$ grep -nF 'test result: ok. 21 passed' task-12238.log
29:test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
$ grep -nF '点名 10 条：现判 4 条、复用 0 条、拒算 6 条；跑了 10 条，抓到 10 条' task-12233.log
21:✓ 点名 10 条：现判 4 条、复用 0 条、拒算 6 条；跑了 10 条，抓到 10 条（其中复用 0 条），本次未跑 0 条，跑了的都抓到了
$ grep -nF 'name=variant_summary' task-12234.log
7:name=variant_summary variant=unmutated named_test=- named_failed=0 any_failed=0 of=5
13:name=variant_summary variant=read-lock-between-request-and-reply named_test=a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply named_failed=5 any_failed=5 of=5
19:name=variant_summary variant=read-lock-between-found-and-contents named_test=a_read_answered_found_holds_the_relay_lock_until_taking_the_contents named_failed=5 any_failed=5 of=5
25:name=variant_summary variant=write-lock-between-request-and-reply named_test=a_write_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply named_failed=5 any_failed=5 of=5
31:name=variant_summary variant=write-many-lock-between-request-and-reply named_test=a_write_many_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply named_failed=5 any_failed=5 of=5
$ grep -nF '158 passed' task-12235.log
585:test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 345.33s
$ grep -nE '(30|54) passed' task-12236.log
120:test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
180:test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.30s
```

12237（clippy）与 12232（变异行静态核）的「退 0」在 `task-server/task status` 里：两条都是「结局 succeeded、退 0」（量的）。七个读数都找得到，字面没走样。另记一处（量的，12234 每趟一行）：`read-lock-between-request-and-reply` 五趟每趟红两条（点名的那条与 `a_read_answered_found_holds_…`），汇总行只报 `named_failed`、`any_failed`，看不出一处变异撞了两条测试；不影响「抓到」，不算打中。

### 打中己：12233、12234 跑的不是开工快照那一版（零轮）

两版的差（量的）：拿 r2 修补实现员留下的补丁底（`device-library-write-many-r2-fix/patch/baseline/…/lib.rs`，sha256 `d9e7b711…`）在草稿目录 `patch-check/` 里打上同一目录的 `crates.patch`，得到的 `lib.rs` sha256 是 `14b72704259ec571f3bcd36b71016d905a8f6f4b3c1edabe08073c65853114c`；它与此刻主工作区（也就是开工快照，sha256 `773f90a7…`）的 `diff` 原样是：

```
851a852,854
>     /// 装置线程每一轮调一次口子的那一步：读的交回读到的，写的交 None。
>     type AskTheLibrary = Box<dyn Fn(&DeviceLibrary) -> Option<String> + Send>;
> 
855c858
<         ask: Box<dyn Fn(&DeviceLibrary) -> Option<String> + Send>,
---
>         ask: AskTheLibrary,
```

也就是背景材料第二节说的「主 agent 另把测试辅助的类型抽成别名」那一处：「    type AskTheLibrary = Box<dyn Fn(&DeviceLibrary) -> Option<String> + Send>;」（`crates/singlefs-device-library/src/lib.rs:853`）、「        ask: AskTheLibrary,」（`crates/singlefs-device-library/src/lib.rs:858`）。

先后（mtime 是量的，开跑时刻是推的：拿任务日志的最后修改时刻减去 `task-server/task status` 报的用时）：主工作区 `lib.rs` 的最后修改时刻与各任务开跑时刻之差，单位秒，负数是任务先开跑。

| 任务 | 用时（status） | 开跑减改别名 | 跑的 `lib.rs` 里有没有别名 |
|---|---|---|---|
| 12232 变异行静态核（第二台） | 210.827 | −30.73 | 没有（推的）；只核原文命中，别名那几行不是任何一行的原文 |
| 12233 证红 | 39.234 | −30.11 | 没有（推的）；用的是 `--copy …/prove-copy`，那棵树已删，核不到 |
| 12234 多轮量法 | 30.164 | −29.59 | 没有（推的）；`rounds.sh` 第 40 行开跑时从 `--repo` 拷树，五棵树都在改别名之前拷完 |
| 12235 E158（第二台） | 402.988 | −28.38 | 无关：别名在 `mod tests` 里，本包当依赖编时不编它 |
| 12236 E174、E178 | 46.256 | +0.65 | 无关，理由同上 |
| 12237 clippy | 1.015 | +14.32 | 有 |
| 12238 本包测试 | 2.014 | +14.76 | 有 |

「本包当依赖编时不编它」的依据：测试模块挂「#[cfg(test)]」（`crates/singlefs-device-library/src/lib.rs:321`）。

为什么单看日志核不到：`rounds.sh` 不打它拷的那棵树的 `lib.rs` 哈希，跑完删树（它的第 72 行 `rm -rf "$tree"`）；12233 的 `prove-copy` 也已不在（量的：`ls` 草稿目录 `device-library-write-many-r2-fix/` 没有它）。

后果（推的）：别名不改类型，编出来的测试二进制行为相同，五个汇总读数大概率照样成立；伤的是「读数对应被判的那一份文件」这句话本身——背景材料把 12233、12234 列在「被判的文件（快照 sha256 `773f90a7…`）」名下。

四句：不涉及臂；被判的系统（主 agent 的读数链）看得到判别它的东西——拷树之后打一行 `sha256sum` 就有；满足的是攻击面三「第二节每个读数，从它的任务日志里逐字找得到」背后那一层「读数是被判那一版的读数」（字面那一层找得到，见上一小节）；改法：在开工快照那一版上原样重交 12234 的命令（命令见末节，交之前与跑完各核一次主工作区 `lib.rs` 的 sha256），12233 那十行可经 crates-mutation-replay 的 `--item` 在主工作区重证。推翻它的现象：重交的 12234 五个汇总行与原先不同（那就不止是「核不到」，而是读数不对），或主 agent 有别处记录表明别名在 12234 开跑之前已在主工作区。

### 「第二节的任务都派在第二台跑过」不成立（量的）

现跑 `task-server/task status <号>`，每个任务第 1 次尝试派到哪台（原样摘「第 1 次：」后面那一段，别的字段里有第二台的私有路径，不贴）：

```
12232 第 1 次：第二台
12233 第 1 次：本机
12234 第 1 次：本机
12235 第 1 次：第二台
12236 第 1 次：本机
12237 第 1 次：本机
12238 第 1 次：本机
```

在第二台跑的只有变异行静态核（12232，只核原文命中、不编不跑本包测试：它的日志里 `test result` 与 `device-library` 都是 0 处，量的 `grep -c`）与 E158（12235，本包当依赖、不跑本包测试）。跑本包测试的三个任务（12233、12234、12238）都在本机。

前两轮的「第二台量过」管不到这一版：r2 判决这一格写的是「| 检查单：所有任务多机可用 | 成立 | 成立 | 成立 | 多线程测试在第二台量过（任务 12134、12135、12136 都派在第二台） |」（`research/prompts/device-library-write-many-code-r2-main-verification.md:40`），那时只有一条探锁测试，函数体在这一批整个换掉了：改动全文删了「-    fn a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {」（`research/prompts/_device-library-write-many-code-r3-diff.md:252`）、另加了「+    fn a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {」（`research/prompts/_device-library-write-many-code-r3-diff.md:458`），新函数体改调共用的 `round_where_the_lock_was_free_in_the_middle_of_the_exchange`，另三条探锁测试是新加的。所以四条探锁测试此刻这一版在第二台上的抓到率与不误报，一次都没量（量的：上表）。管道缓冲与页大小那一层的担心（请求 256 KiB 比管道缓冲大，第二台的页大小没核）r2 攻方已提过，这里不重复，只指出这一批没补上。

改法：本包测试目标与 12234 的多轮量法各在第二台交一次（服务定派到哪台，调用方不写挑台参数；派到本机的就再交一次，直到第二台出一份——这一步由主 agent 照服务的派法办，我写不了挑台）。推翻它的现象：主 agent 有别的任务号，跑的是此刻这一版、派在第二台。

### `rounds.sh`「每趟一个编译目录」（成立，量的加推的）

源码（`device-library-write-many-r2-fix/rounds/rounds.sh`，草稿目录里的脚本，不在仓里，行号是那份文件的，量的 `cat -n`）：第 39 行每个变体一棵 `tree="$WORK/$slug/tree"`；第 53 行每趟 `CARGO_TARGET_DIR="$WORK/$slug/target-round-$round"`；第 69 行每趟跑完 `rm -rf "$WORK/$slug/target-round-$round"`；第 72 行每个变体跑完删树。12234 的 `task-server/task status` 报「编译目录 无」（量的），服务没另给编译目录，不会盖掉脚本自己设的那一个。

第二轮辩方脚本栽的那一形是「两份副本共用一个 `CARGO_TARGET_DIR`，未变异那一份跑的是变异编出的二进制（两边都 `finished in 0.00s`、同一句断言）」（r2 判决第 10 行）。12234 每一趟的行（量的，日志第 2–30 行）里没有这一形的痕迹：二十五趟的 `finished in` 都在 0.47 到 0.54 秒；红的测试随变体而变（`read-lock-between-request-and-reply` 每趟红两条，另三个变体每趟各红自己点名的那一条，没改坏的五趟 21 条全绿）——要是几份共用一个二进制，红的那一组会在变体之间相同。每趟有没有真的重编，日志里没有 `Compiling` 行可数（`rounds-run-1` 已删，量的：工作根下没有这个目录），这一半是推的：本包只用 std（「只用 std，不依赖工作区任何 crate」写在本包 `Cargo.toml` 的 `description` 里），二十五趟连拷五棵树共 30.164 秒，每趟一秒多，与「每趟从零编一个只用 std 的小包再跑 21 条测试」相容。

推翻它的现象：主 agent 重交 12234 的命令时，在 `--work` 下留着某一趟的 cargo 日志而里面没有 `Compiling singlefs-device-library`。

## 检查单八格

### 走不到的理由二次查实（成立）

产品段里的 `panic!` 只有 `count_entries`、`count_write_many_calls` 在转交那一种上的两处。调用点现跑（量的）：

```
$ grep -rn --include=*.rs --include=*.py -E 'count_write_many_calls|count_entries' crates scripts | grep -v '^crates/singlefs-device-library/src/lib.rs'
crates/singlefs-verification/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:891:fn count_entries_and_value_bytes<Key>(map: &RefCell<BTreeMap<Key, Vec<u8>>>) -> (u64, u64) {
crates/singlefs-verification/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:900:        count_entries_and_value_bytes(&self.bytes_read_by_placement)
crates/singlefs-verification/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:906:        count_entries_and_value_bytes(&self.kept_bytes_by_placement)
crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:29955:            u64::try_from(store.count_entries("items/", ".out")).expect("条数装得进"),
crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:29959:        assert_eq!(store.count_entries("codes/", ".txt"), 4);
crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:30030:            store.count_write_many_calls(),
crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:30035:            u64::try_from(store.count_entries("items/", ".out")).expect("条数装得进"),
crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:30043:            store.count_write_many_calls(),
scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:7693:        library.store.count_entries("items/", ".out")
scripts/experiment-apparatus/e174_lease_drop_wait_memory_return.rs:4187:        library.store.count_entries("", ".out")
```

E197 那三处是同名前缀的别的函数。E158 那五处在第 29232 行的 `#[cfg(test)]` 之后（`mod fifth_run_tests`）；E178 第 7693 行之前最近的是第 7075 行 `#[cfg(test)]` 与第 7076 行 `mod tests {`；E174 第 4187 行之前最近的是第 3870、3871 行同样两行（量的 `awk`）。这一批产品段新加的两样（可失败预留、`read` 核键）都没有「走不到」的说法要查；`try_reserve_exact` 成了之后 `resize` 不再分配，这一句前一条腿已核。

### 类型与名字（成立）

`grep -cE '_ =>' crates/singlefs-device-library/src/lib.rs` 是 0（量的）。产品段 `.expect(` 8 处，全是两个锁的常量理由（第 195、236、249、257、275、284、299、314 行，量的），理由写的是「持锁时只做哪几样、这几样凭什么不 panic」，`RELAY_LOCK_IS_NEVER_POISONED` 随这一批改成了「可失败地预留…预留不成照别的错退掉」，与第 219–224 行的实现对得上。新名字（`AskTheLibrary`、`ProbedExchange`、`ProbeRoundProgress`、`PROBED_REQUEST_PAYLOAD_IN_BYTES`、`PROBED_REQUEST_BEGINNING_IN_BYTES`、`reply_parts`、`round_where_the_lock_was_free_in_the_middle_of_the_exchange`、三条新探锁测试、`read` 的三条键测试与分配不出那一条）没有序数与位置代称；`part_number`、`round_number` 是计数的值，不是拿位置当名字。

### 不兼容旧代码与旧格式（成立）

照背景给的命令现跑（量的）：

```
$ grep -nE 'legacy|old|migrat|兼容' research/prompts/_device-library-write-many-code-r3-diff.md
252:-    fn a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
458:+    fn a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
475:+    fn a_read_answered_found_holds_the_relay_lock_until_taking_the_contents() {
491:+    fn a_write_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
511:+    fn a_write_many_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
```

五处全是测试名里的 `holds`。

### 不新建判定库（成立）

产品段（第 1–320 行）里开文件的只有 `from_environment` 打开两个描述符号（第 166 行 `OpenOptions`），写的只有往请求管道 `write_all`（第 140、198 行）；`File::create`、`create_dir`、`rocksdb`、`sqlite` 0 处（量的 `grep -nE`）。改动全文里 `checker`、`crash` 也是 0 处（量的 `grep -cE`）。

### 所有任务多机可用（推不出，算打中）

新测试读的本机状态只有 `current_exe`（第 547 行）、子进程那一半的标记变量（第 523 行）与管道（量的 `grep -nE 'std::env::|/proc/|current_exe|hostname|/home/'`，产品段另有第 156、162 行读描述符号），不读主机名、家目录与本机独有的文件——字面判据那一半成立。可这一格背景材料给的依据是「第二节的任务都派在第二台跑过」，攻击面三一节量的是不成立：跑本包测试的三个任务都在本机，四条探锁测试此刻这一版在第二台一次没跑过。依据不成立、结论又要靠第二台上的读数撑（探锁测试的抓到率依赖管道缓冲与调度），所以推不出。

### 测试优先多线程（成立）

探锁那几条本身三线程（装置线程、替身线程、探锁的本线程）；libtest 在测试函数之间并行；多轮量法五份各五趟。「够不够」只在本机量过，归上一格。

### 带 --item 的命令与项（成立，推的一半：`--list-items` 没跑）

本包变异行现跑静态核（量的：python 读 `crates/mutations.tsv` 与 `lib.rs`，第二段是本包 `lib.rs` 的行）：25 行（第 6983、6984、7393–7399、7590–7598、7605–7611 行），每行六段、原文在 `lib.rs` 里恰好命中 1 次、第五段都是 `-p singlefs-device-library --lib`、第六段点名的测试函数都在 `lib.rs` 里，不合的 0 行；表共 7611 行（量的 `wc -l`）。12233 点名的十行是 7605–7611 七行与 6983、7590、7596 三行（量的：日志第 10–19 行逐行「抓到」）。

另记一处（推的，不算打中）：第 7598 行「判定库口子 读的锁在请求与回应之间放过」不在 12233 的十行里，可它点名的 `a_read_holds_…` 这一批整个换了函数体（攻击面三一节引的改动全文第 252、458 行）；这一行在新函数体上的「抓到」只有 12234 的 5/5，而 12234 跑的是别名之前那一版（打中己）。`--list-items` 与这一行的 `--item` 重证都交主 agent（末节）。`--list-items` 列不出新名的，这一格翻成不成立。

### 新崩溃流多罩了什么（成立）

改动全文里 `checker`、`crash` 0 处（量的）；本包不碰盘、不在崩溃枚举的流里。第 3b 步的崩溃状态数不适用（模型也不涉崩溃状态，见开头）。

## 没打中的形状

| 形状 | 取样 | 为什么不算打中 |
|---|---|---|
| `failed` 那一行里带非 UTF-8 字节 | 读源码：`reply_line` 用 `read_line`，非 UTF-8 时走 `Err`，打「读不了转交判定库的管道」退 3，不照回应里的码 | 转交那头拼 failed 行是「        return f"failed {code} {sentence}\n".encode("utf-8", "surrogateescape")」（`scripts/experiment-apparatus/device-code-key.py:643`），句子来自客户端，客户端解服务的说明用「            return cursor.length_prefixed("说明").decode("utf-8", "replace")」（`scripts/prune-server/prune_server_client.py:780`），替换过的串编回去仍是 UTF-8；我没找到会带进孤立代理码的来源（推的，只读了这两处）。走不到 |
| `found 0` | 读源码：预留 0、`resize` 0、`read_exact` 空切片，交 `Some("")` | 与库里空值一致，r1 正推已核空内容 |
| `found +5`、`failed 0`、`failed 256` | — | 前两轮已攻（r2 攻方「`found` 的长度写成 `+5`」、r1 攻方「回应 `failed 0 …`、`failed 256 …`」），不重复 |
| 一批里同一个键两条 | 读源码：内存那一种后写的盖前写的 | 转交那一侧与服务一批里重复键的次序不在被判文件里，没追（见限度） |
| `write_many` 先判空批再核键 | 空批里没有键 | 结局相同，前一条腿已记 |
| `Clone` 之后两份在两个线程同时 `read` | 读源码：同一把锁 | r1、r2 的探锁测试已罩 |
| 12233「拒算 6 条」是不是没跑 | 日志第 4–9 行：「行键算不出（照跑、不复用、不写库）」，第 10–19 行十行都「抓到」 | 跑了，只是不进判定库复用；不算读数错 |

没打中的只有一次取样（这一条腿），照规则不当「没打中」的结论用。

## 这条腿自己的限度

- 模型一次没跑（我不交任务），甲、乙、丁、戊、丙与我提的改法全是推的；乙的常驻内存量法依赖 libtest 同进程里别的测试不在那三秒里涨过 16 MiB（推的：探锁测试每份 256 KiB 级），同进程并行的测试要是涨得多，`unmodified` 会红得对、改法那一臂可能误红——读 `name=resident` 那几行时一起看 `growth=` 的大小，不只看红绿。
- 草稿测试 `read_answered_found_touches_…` 用 `std::io::pipe()` 与 `/proc/self/status`，只在 Linux 上成立；本包别的测试也用 `std::io::pipe()`（`lib.rs` 里 6 处，量的），不额外缩窄。
- 打中己的先后是从 mtime 与任务用时推的；主会话的记录我只用来核「改别名那一步在 12233、12234 交出之后」，没有逐条读。
- 转交一侧（`device-code-key.py`）与客户端只读了引到的几处，没有逐段攻；一批里重复键、服务的分帧次序不在被判文件里，没追。
- 第二台的页大小与管道缓冲我核不到（不登第二台），所以「多机」那一格只能是推不出，不能是不成立。

## 要主 agent 跑的命令

一行一条，自己不交。前四条读数回来之前，甲乙丙丁戊、我提的改法与打中己的「行为相同」那一半都是推的。

1. 模型整份复跑（`rerun.sh` 外面那一层先删运行目录；照判决规则在模型目录的拷贝里起时，把两个模型路径换成拷贝的）：
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/device-library-write-many-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r3-opus/copy /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r3-opus/model-run /home/fy5090/code/singlefs/research/prompts/device-library-write-many-code-r3-opus-model`
   - 在哪起：仓根（路径都是绝对的）。`--frozen` 是草稿目录里的 `copy/`（minimal-copy 建，`lib.rs` sha256 与开工快照同，量的），`.deliverables` 写成「交接」。里面每条 `cargo test` 自带 target 的范围声明（模型目录 `tools/rerun.sh` 第 98 行那一条）。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`（脚本把 cargo 的退出码收进输出、自己照常退 0）；`task-server/task log <号> --all` 里 `name=frozen_library sha256=773f90a766cb…` 一行，六行 `name=variant_summary` 对开头「跑前写下的期望」那张表逐格比，`name=resident` 几行的 `growth=`，每趟 `compiled=` 只在每个变体的第一趟是 1。
   - 跑完要删：运行目录 `…/device-library-write-many-code-r3-opus/model-run`（`rerun.sh` 每个变体跑完已删树与编译目录，剩 `logs/`）；服务给的编译目录照 `task-server/task clean <号>`。
2. 在开工快照那一版上原样重交 12234 的多轮量法（核打中己）：
   - 命令：`task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/device-library-write-many-r2-fix/rounds/rounds.sh --repo /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r3-opus/rounds-on-snapshot-run`
   - 在哪起：仓根。交之前与 `task wait` 回来之后各跑一次 `sha256sum crates/singlefs-device-library/src/lib.rs`，两次都要是 `773f90a766cb5826a81ffe87d14345c56f49d1876b7a99100c20ea33d3b8bff8`，不是的这一趟作废。`rounds.sh` 里的 `cargo test` 自带范围声明（它的第 53 行）。
   - 看哪几行：五行 `name=variant_summary` 与原 12234 的五行逐字比（没改坏的 `any_failed=0 of=5`、四个放锁变异 `named_failed=5 any_failed=5 of=5`）；派到第二台的，这一条同时补上「多机」那一格的一份读数。
   - 跑完要删：`--work` 下只留 `logs/`（脚本自己删树与编译目录），用完删整个 `rounds-on-snapshot-run`。
3. 列项（检查单「带 --item 的命令与项」那一格）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 的 --list-items; 范围=只列不跑，不起 cargo; 为什么=核变异表第 7605 至 7611 行与改过的第 6983、7590 行能被点名' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items /home/fy5090/code/singlefs`
   - 在哪起：仓根。
   - 看哪几行：输出里七个新名（第 7605–7611 行）与第 6983、7590 两行的名字都列出；末行键对不上的条数。
4. 第 7598 行在新函数体上重证（检查单「带 --item」那一格的另记一处）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 的一行变异; 范围=1 项; 为什么=第 7598 行点名的探锁测试这一批换了函数体，证红任务没点它' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --item '判定库口子 读的锁在请求与回应之间放过' /home/fy5090/code/singlefs`
   - 在哪起：仓根。
   - 看哪几行：那一行「抓到」与成功句里「点名 1 条…抓到 1 条」；退 77 是本次未跑，不算过。
5. 本包测试目标在第二台出一份读数（「多机」那一格）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标; 范围=1 个测试目标; 为什么=四条探锁测试此刻这一版在第二台一次没跑过' task-server/task submit -- cargo test -p singlefs-device-library --lib`
   - 在哪起：仓根。派到哪台由服务定，我写不了挑台；`task-server/task status <号>` 的「第 1 次：」是本机的，这一份不补「多机」那一格，怎么拿到第二台的一份由主 agent 定。
   - 看哪几行：`test result: ok. 21 passed`；`task-server/task status <号>` 的「第 1 次：」那一段。

## 没做什么

- 没往任务服务交任何任务；模型、重交 12234、`--list-items`、第 7598 行重证、第二台那一份都在上一节，判决要引的数由主 agent 复跑。
- 不判正推、辩方那几格；不替主 agent 采纳；第五节格 1（r2 第三节四条修没修到）不在派给我的攻击面里，没逐条判。
- 副本上量出的数（模型那一份）不算入库装置上的数。
- 我提的改法（`read-contents-without-filling-zeros-first`）只在我的模型上，还没量，被攻过零轮；它同时要把变异表第 6983 行的锚点换掉（见乙那一节）。
- 打中甲、乙、丁、戊、丙、己与「多机」那一格都是第三轮新冒出的零轮形态，照规则不为它们开第四轮，交判决「零轮」一栏。
- 清理：没删 `/home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r3-opus/copy`（仓副本，53M，量的 `du -sh`）：交接给主 agent 当第 1 条命令的 `--frozen`，`.deliverables` 写成「交接」，主 agent 用完删。草稿目录里 `patch-check/`（只有一份打过补丁的 `lib.rs`，核打中己的两版差）在 `.deliverables` 写成「报告」。我没建编译目录。

引文核对（量的，交回前现跑 `python3 scripts/agents/cite-check.py research/prompts/device-library-write-many-code-r3-opus-output.md`，退 0，末行原样）：

```
  ✓ 核了 35 处引文、11 处裸行号，对上 35 处，没判 11 处（其中裸行号没有引文可比 11 处；逐处列在上面）
```
