# device-library-write-many-code-r2：攻方腿（Opus）报告（2026-11-10）

立场：找「哪一格会错」。被判文件 `crates/singlefs-device-library/src/lib.rs`（sha256 `d9e7b711802fb255d5f10045d4f60d10075f48f13969b8ededf5d9ad1f8c3cbf`，与开工快照 `crates-sha256.txt` 那一行相同，量的：`sha256sum` 现跑）。转交一侧读的是主工作区此刻的 `scripts/experiment-apparatus/device-code-key.py`（sha256 `85a6425fa9b3b82b89e4c34cc23440698e3ede059355278be86c481663b5f545`，不在快照里）。本腿没往任务服务交任何任务，下面凡是「跑出来」的数都标「推的，待主 agent 跑」。

## 复跑命令与模型文件

模型目录 `research/prompts/device-library-write-many-code-r2-opus-model/`，复跑一条（整条交主 agent，写法见文末「要主 agent 跑的命令」第 1 项）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/device-library-write-many-code-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-opus/copy --work /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-opus/rerun-1
```

每个文件的 sha256（量的：在模型目录里 `find . -type f | sort | xargs sha256sum` 现跑）：

```
623add5b6ccc100663289c122b5b61e98ac12f63c71240872763d19f19caa767  ./attack_tests.rs
b7458f5d87cbee8291cff89a8ab1cc3b49a49b3064aba0784a482d9e12c64f68  ./key_rules_cross_check.py
8f49debd029f48d3e65b42ff0142b33e29ef016b44cd9c21e9c5caba3a79169f  ./mutants/read-lock-released-before-found-contents.anchor
5f78f3a6d2638abef44ddf05e3b69cc21d64fcc25e8f6d42d6cd9adf6ecadaa6  ./mutants/read-lock-released-before-found-contents.replacement
7ddcee91bed5a875e6187f3a4278837422e4c94710066f98b723b19c7f708796  ./mutants/write-lock-released-between-request-and-reply.anchor
ef9060dc0f977abf4f54263902ad188162bdd00b4c8885b178552d5cb2ab1a4b  ./mutants/write-lock-released-between-request-and-reply.replacement
58d3c613c7559f8042a4d10b466d9eb3c87ef58a177ce1025bafbe11b881ed83  ./mutants/write-many-lock-released-between-request-and-reply.anchor
44c015ac45078ce98d1f41ce9927f0055abe84a3cd64502182b5ea13a6db66e6  ./mutants/write-many-lock-released-between-request-and-reply.replacement
c89727bb38f338c780e528ec25a55c8c496d880e2e74aaf37f14db948da03218  ./relay_read_smuggle_model.py
2c43a0c0d9efe3518c4e1a5b5adc5e17eac685bfed5b31a853be778bc317b919  ./tools/apply_once.py
b993dd52e308ddffe8b488d4184ddf065c563325f61e9afaeb54bc2af207d46b  ./tools/rerun.sh
```

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`。它做六步：①转交模型（原样载入仓根的 `LibraryRelay`，接假库）喂键里带换行的 read；②拷出来的树里插攻方测试（`attack_tests.rs`），实现不改，整个 lib 目标跑五趟；③两侧相对键规则各跑各的再比；④没改坏时探锁测试一份一份 40 次、同时几份 10 批；⑤照变异表「判定库口子 读的锁在请求与回应之间放过」那一行原样施加（从副本里的 `crates/mutations.tsv` 取，不手抄），探锁测试一份一份 100 次、同时几份 20 批，再照变异门禁的跑法（整个 lib 目标）连跑 30 趟；⑥攻方三个放锁变异各跑整个 lib 目标五趟。同时几份 = 服务给的 `SINGLEFS_THREAD_CAP` 除以 3（登记值 31 时是 10）。挂钟推的：十分钟上下（一个只用 std 的包编五次，探锁测试没改坏时一次约 0.5 秒）。

本轮没有崩溃枚举：被判的是装置读写判定库的口子，不碰盘与崩溃状态，定义第 3b 步的状态数、内存稀疏盘与崩溃枚举函数都用不上。

## 各格判定一览

| 格 | 判定 | 依据（细节在各节） |
|---|---|---|
| 攻击面一 两份键规则对同一个键判得一样不一样 | 成立（推的） | 逐字读两份：空串、`..`、开头 `/`、`a//b`、非 ASCII 字母数字、空白，两边判法相同；交叉核对（③）待主 agent 跑 |
| 攻击面一 `read` 不核键留下了什么 | **打中 A**（条件可达） | 键里带换行的 `read`，换行后面那一段在转交那一侧是一条各自合规的 `write`，照写进库；写一侧的核键挡不住：现有三家调用方都是先 `read` 后 `write` 同一把键 |
| 攻击面一 同上（内存那一种） | **打中 B**（与第一轮打中 4 同形，落在 `read`） | 内存那一种 `read` 对转交会拒的键交 None、不退；真跑转交回 failed 3、装置退 3 |
| 攻击面二 `found` 长度过大 | 成立（持锁不 panic）；附一处推不出 | 超过 `isize::MAX` 退 3；2^62 这类不过 `isize::MAX` 又分配不出的长度是 SIGABRT、不是退 3（转交走不到，推的，测试草稿待跑） |
| 攻击面二 `give_up` 写标准错误写不进 | 成立 | `let _ = writeln!` 不 panic，照码退；变异 7597 的测试判得出（子进程退 101 对 75） |
| 攻击面三 起子进程的七条测试 | 成立 | 每条要么断言退出码加请求字节，要么只有一条路给得出那个码；七条都拒算、每趟照跑 |
| 攻击面三 多线程测试「一次抓到」 | **推不出（打中 C）** | 抓不抓到靠时序，单次观测；判定库只复用「抓到」，任务 12083 那一次抓到存进库后照键复用，不再取样；抓到的机会依赖「2 ms 内装置线程睡到锁上」与探锁线程在处理器上 |
| 攻击面三 锁覆盖的范围 | **打中 D**（推的，待主 agent 跑） | 探锁测试只罩 `read` 的请求与回应行之间；`found` 之后读内容、`write`、`write_many` 在请求与回应之间放锁，没有测试、也没有变异行 |
| 攻击面四 `give_up` 与 libtest 并行 | 成立（不静默），附一处代价 | 父进程里没有测试走到 `give_up`；将来走到时整个测试进程退 3，变异门禁把那一行归「无效」不归「抓到」（推的） |
| 攻击面四 `read` 回应解析有没有变 | 没变（只加一行 `filter`）；附一处没看过的 | 内容不是 UTF-8 时交 None（读成不在、现判），与 `read` 的文档句「读失败照退出码退，不交 None」不符；转交走不到（推的） |
| 检查单 走不到的理由二次查实 | 成立 | 两处 `panic!` 的调用点全在测试模块里（`count_entries` 第一轮没追，这次追了） |
| 检查单 类型与名字 | 成立 | 两条理由常量写的不变量与持锁时做的事对得上；没有 `_ =>` |
| 检查单 不兼容旧代码与旧格式 | 成立 | 现跑 grep 两处命中都不是兼容：`holds` 里的 old、`old mode 100755`（转交脚本丢了执行位，调用处都经 `python3` 起） |
| 检查单 不新建判定库 | 成立 | 内存那一种不是这一批加的，测完即丢 |
| 检查单 所有任务多机可用 | 成立（第二台没核） | 子进程用 `current_exe` 加环境变量与管道；多线程测试靠「请求 256 KiB 大于管道缓冲」，本机页大小 4096、x86_64，第二台没核 |
| 检查单 测试优先多线程 | **推不出（打中，同 C）** | 这一格交给攻击面三判：抓法够不够没量过 |
| 检查单 带 --item 的命令与项 | 成立 | 任务 12083 逐行点名九行都跑了、都抓到；`--list-items` 本轮没跑，命令交主 agent |
| 检查单 新崩溃流多罩了什么 | 成立 | 没有新崩溃流；现跑 grep 两张表无命中 |

## 攻击面一：修补的键核对与两份键规则

### 两份规则对同一个键：成立（推的）

装置一侧 `is_relative_key_the_relay_accepts`（crates/singlefs-device-library/src/lib.rs:90-95）：不空，每个字节是 ASCII 字母数字或 `. _ - /`。转交一侧「RELATIVE_KEY_PATTERN = re.compile(r"[A-Za-z0-9._/-]+")」（`scripts/experiment-apparatus/device-code-key.py:212`），`fullmatch`。逐个边界：空串两边都拒（`+` 要至少一个）；`..`、`.`、`/`、`/abs`、`a//b`、`../x` 两边都收；非 ASCII（`é`、阿拉伯数字 U+0663、不换行空格）两边都拒（Python 字符类的 `A-Z`、`0-9` 是码位区间，不随 Unicode 放宽；Rust 按字节判）；`\r`、`\n`、空格、制表符两边都拒；`fullmatch` 不像 `$` 那样放过末尾换行。两边收的集合我推是同一个。`..` 收下不越界：客户端 `LibraryStore.key` 是前缀加相对键字节直接拼（`scripts/prune-server/prune_server_client.py:1283-1284`），服务那一头是键值库、不规整路径，`../x` 只是一把字面的键。

交叉核对（rerun 第③步，推的，待主 agent 跑）：攻方测试 `listing_of_what_the_key_rule_accepts` 让 Rust 那一份对全部 Unicode 标量与十六条边界键各判一次写进文件，`key_rules_cross_check.py` 拿转交模块里原样的正则逐个比。我推的样子：`name=single_characters rust_accepted=66 python_accepted=66 differ=0`、`name=summary differ=0`。推翻它的现象：`differ` 不为 0。

### 打中 A：键里带换行的 `read`，后半段在转交那一侧被当成一条合规的请求执行（条件可达）

被判的话：「//   相对键只用 ASCII 字母、数字与 . _ - /，不空、不带空白（与 device-code-key.py 的 RELATIVE_KEY_PATTERN 同一条）：write 与 write-many 在拿锁、」（`crates/singlefs-device-library/src/lib.rs:12`）——核键只放在两种写上；`read` 拿锁之后直接拼「                    .write_all(format!("read {relative}\n").as_bytes())」（`crates/singlefs-device-library/src/lib.rs:195`）。

具体输入：`read("items/a.out\nwrite items/injected.out 0")`。逐步：
1. 装置写出 `read items/a.out\nwrite items/injected.out 0\n`（攻方测试 `observed_read_with_a_newline_in_the_key_sends_a_second_request_after_the_read` 断言的就是这串字节，推的，待主 agent 跑：现状下绿）。
2. 转交「                    line = requests.readline()」（`scripts/experiment-apparatus/device-code-key.py:613`）先读到 `read items/a.out`，合「        if len(words) == 2 and words[0] == "read" and RELATIVE_KEY_PATTERN.fullmatch(words[1]):」（`scripts/experiment-apparatus/device-code-key.py:650`），回 `absent`。
3. 下一行 `write items/injected.out 0` 合「        if len(words) == 3 and words[0] == "write" and RELATIVE_KEY_PATTERN.fullmatch(words[1]) and words[2].isdigit():」（`scripts/experiment-apparatus/device-code-key.py:659`），读 0 字节、记进这一趟写过的键、写服务、回 `written`。这两条都合规，转交这一轮加的「认不出就停」碰不到它。
4. 装置只读一行回应（`absent`），读成不在。库里多了一把装置从没交给 `write` 的项键 `items/injected.out`（空值）；回应管道里多一行 `written`，下一问读到的是上一问的回应。

模型（rerun 第①步，推的，待主 agent 跑）三格 `read_key_with_newline_then_empty_write`、`…_then_eight_byte_write`、`…_then_write_many` 应各打 `store_writes=items/injected.out injected=True`；对照格 `control_read_key_with_space` 打 `replies=failed/3 store_writes=-`，`control_read_valid_key` 打 `replies=absent store_writes=-`；末行 `name=summary hits=3`。

写一侧的核键为什么挡不住：三家调用方都是先读后写同一把键，坏键先经 `read` 出门。E158「        let Some(stored) = self.store.read(&Self::item_path(key)) else {」（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/fifth_run_judgement_library.rs:105`）之后才「        self.store.write_many(&items);」（`crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/fifth_run_judgement_library.rs:132`）；E178「        if let Some(item_lines) = read_back_the_item(library.store.read(&path), &segment_four_item_header(&item_key, &library.code_key)) {」（`scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:6418`）之后才「        library.store.write(&path, &text);」（`scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:6423`）。带换行的键在 `read` 那一步已经把后半段送进去执行了，到 `write` 才退 3，库里那一条不撤（只在代码键变了时撤）。

可达性：三家现有的键是十六进制或 index.tsv 按行拆出来的字段，带不进换行，走不到。按第一轮判决给打中 2 用的同一把尺子（「E158 的键是十六进制，走不到；装置这一侧不核键，别的调用方走得到」），这一格同样算条件可达。

四句：1. 没有臂。2. 装置看得到判别它的东西（键在手里，`write` 已经在核），转交看不到（两条都合规）。3. 满足的是第 12 行「不合的照「别的错」退 3，管道上一个字节都不发」只对两种写成立；第一轮判决第三节第 1 条「1. 装置这一侧在发请求之前核相对键（字母、数字与 `. _ - /`，不带空白），`write`、`write_many` 两种、转交与内存两个后端都核；不合的照「别的错」退 3、不发请求。修打中 2、3、4。」（`research/prompts/device-library-write-many-code-r1-main-verification.md:42`）字面只点名两种写，实现照字面修到；打中 2 那一类（装置交出的字节被转交当成装置没交过的请求执行）经 `read` 这条门还开着，算「没修到」。4. 跑前没有改法条款。

推翻它的现象：转交对 `read` 之后紧跟的同一次写入另有边界判断（我读的 `serve` 第 606–629 行、`answer` 第 645–671 行没有）；或 `read` 在拼请求之前核键。

### 打中 B：内存那一种 `read` 对转交会拒的键交 None（与第一轮打中 4 同形）

内存那一种 `read` 直接 `entries.get(relative)`（crates/singlefs-device-library/src/lib.rs:223-228），不核键，`read("items/bad key.out")`、`read("")` 交 None、不退（攻方测试 `observed_in_memory_read_with_a_key_the_relay_refuses_reads_as_missing`，推的，待主 agent 跑：现状下绿）。真跑转交第 650 行不合，回 failed 3、停，装置退 3。单测里读成「不在、去现判」的，真跑退 3。

走得到的样子（推的）：E178 拿库里读回的内容拼下一把读的键（第 6376 行读上一趟的项、从项头取 `code_key=` 再第 6390 行读 `codes/<它>.txt`），读回的内容坏了时内存单测走 `None => unreadable += 1`，真跑退 3。内容是装置自己写的十六进制，走不到。

推翻它的现象：内存那一种 `read` 也核键。

## 攻击面二：`found` 长度与 `give_up` 两处新改法

### `found` 长度：成立（持锁不 panic）；「回应认不出退 3」只罩到 `isize::MAX` 以上（推不出它算不算打中，走不到）

新加的「                    .filter(|length| isize::try_from(*length).is_ok())」（`crates/singlefs-device-library/src/lib.rs:210`）把超过 `isize::MAX` 的长度交 `give_up_on_reply`，退 3；`vec!` 的容量溢出 panic 因此走不到，锁不中毒。新的 panic 没有引进：`filter`、`try_from` 不 panic，`give_up_on_reply` 只做 `strip_prefix`、`split_once`、`parse`。

没罩到的一段：长度不过 `isize::MAX`、又分配不出（例 2^62 字节，大过 x86_64 的用户地址空间），「                let mut contents = vec![0_u8; length];」（`crates/singlefs-device-library/src/lib.rs:214`）分配失败，进程 SIGABRT（不展开，锁不中毒，理由常量第 31 行写了这一句），退出码不是 3。文件头「//   管道断了、回应认不出退 3。」（`crates/singlefs-device-library/src/lib.rs:14`）对一个转交不会发的长度来说，2^62 与 2^63 同样「认不出」，两者结局不同。攻方测试 `read_answered_found_with_a_length_that_cannot_be_allocated_exits_with_the_other_error_code`（子进程那一种，照第 698 行那条的写法）断言退 3；我推现状下红，`left: None`（被信号杀掉）、标准错误里有 `memory allocation of 4611686018427387904 bytes failed`（推的，待主 agent 跑；分配器在过大的请求上是中止进程，这一条是我印象里的 std 行为，本机没装 rust-src，没在本仓核）。转交只发 `len(data)`，走不到，所以我只记「推不出」，不记打中。改法（零轮）：长度上限改成转交那一侧的值上限，或 `Vec::try_reserve_exact` 失败也走 `give_up_on_reply`。

### `give_up` 写标准错误写不进：成立

`let _ = writeln!` 写不进不 panic，`process::exit` 不展开，锁不中毒；退出码照传。吞掉的只有那两行话，码还在。变异 7597（第一行换回 `eprintln!`）下子进程在 `give_up` 里 panic、libtest 退 101，`give_up_with_the_standard_error_closed_still_exits_with_the_code_in_the_reply` 断言 75，判得出（任务 12083 那一行抓到，拒算、每趟照跑）。

附带的一处差别（不是打中）：`std::io::stderr()` 的句柄直接写文件描述符 2，不经 libtest 的输出捕获（`eprintln!` 经捕获）。父测试进程里将来有测试走到 `give_up` 时，那两行话直接出现在 cargo 输出里、不归到那条测试的捕获段里（推的，我印象里的 std 行为）。

试过没打中的：别的线程握着标准错误的锁、卡在写满的管道上时 `give_up` 跟着卡住、手里还握着转交锁——改前 `eprintln!` 一样要这把锁，不是这一批引进的；`device-code-key.py run` 起装置时标准错误照继承，任务服务底下落进日志文件，不会写满。

## 攻击面三：产生结论的方法

### 起子进程的七条测试：成立

七条都是「子进程只调一次口子，口子没退就 `exit(0)`」，父进程断言退出码，三条另断言请求管道上的字节。逐条问「这个码还能从别的路来吗」：
- 写与成批写的坏键两条：退 3 且请求管道空。别的退 3 都在发请求之后（写不进、回应认不出），请求管道不会空。
- 内存坏键一条：内存那一种只有核键一处退进程。
- 两条 `*_answered_failed_*`：75 只有「回应是 failed 75」一条路。
- 长度过 `isize::MAX` 一条：没 `filter` 时 `vec!` 容量溢出 panic、libtest 退 101，与 3 分得开。
- 标准错误关掉一条：见攻击面二。
子进程照测试名 `--exact` 只跑那一条；名字写错时子进程跑 0 条、退 0，父进程断言的码都不是 0，会红，不会静默过。七条在任务 12083 里都「拒算（current-exe …）」、每趟照跑，不复用。

### 打中 C：多线程测试在锁变异下「一次抓到」不能当证据，而判定库把这一次存成常绿（推不出）

1. 单次观测。「跑至少 5 轮。要说「通过」就得每轮都通过，要说「失败」就得每轮都失败；」（`.claude/singlefs-ai-sop/rules/test-discipline.md:6`），前提「前提是观测本身有变化：真实 I/O、并发、时序、随机源，至少占一样。」（`.claude/singlefs-ai-sop/rules/test-discipline.md:9`）——这条测试四样里占了并发与时序。证红任务 12083 只跑了一次（背景材料与 `task-server/task log 12083 --all` 里那一行都是一次）；实现员自己写明「不是严格必红」。
2. 一次抓到之后不再取样。`task-server/task log 12083 --all` 里这一行标「（现判：行新加）」，另七行标「（拒算）」；门禁文件头「#   --list-items 末行报键对不上的条数。只有「抓到」与基线 ok 复用。」（`.claude/gate.d/crates-mutation-replay.sh:120`）。所以这一行的「抓到」写进判定库，被改坏的单元、点名测试的闭包、工具链不变，下一趟照键复用、不再跑；确定性的七条子进程测试反而每趟照跑。会重新取样的只有复用哨兵随机抽中它的那一趟。我在 `scripts/mutation/crates-mutation-rows.py` 里 grep「不复用|照跑」，只见到锚点失效、拒算、撞键三种照跑的原因，没见到「这一行靠时序、每趟照跑」的标记（范围只是这一次 grep）。
3. 抓到的机会是几个没扫过的量的函数（推的）：探锁一方只能在装置线程放锁、再拿锁之间那一刻抢到。测试靠「        const QUEUEING_TIME: std::time::Duration = std::time::Duration::from_millis(2);」（`crates/singlefs-device-library/src/lib.rs:771`）让装置线程先睡到锁上，第 770 行的注释写了为什么：锁记成「有人等过」，放锁要进内核叫醒，空当才够宽。装置线程 2 ms 内没被调度到、没睡到锁上，放锁就不进内核，空当只剩几条指令；探锁线程那一刻不在处理器上也抢不到。变异门禁跑的是整个 lib 目标（变异表那一行的参数列是 `-p singlefs-device-library --lib`），十几条测试与七个子进程同时起。另一个前提是「        // 请求比管道缓冲（Linux 默认 64 KiB）大得多：装置线程写请求时握着锁卡在管道上，等替身读。」（`crates/singlefs-device-library/src/lib.rs:766`）：请求整条装得进管道缓冲时，装置线程在替身读到请求开头之前就写完、放锁、再拿锁，探锁一方还没开始抢，一次都抓不到。本机页大小 4096（量的，`getconf PAGESIZE`），默认缓冲 16 页 64 KiB；第二台没核。
4. 这一格按 `.claude/rules/mutation-sampling.md` 的第三类问：「| 在**我选的取样点上**恰好同值，换一个取样点就不同 | **取样点不敏感** | **补一个敏感的取样点**，不是留档 |」（`.claude/rules/mutation-sampling.md:17`）。这里的「取样点」是一次调度的结果：同一棵树、同一条命令，换一次调度就可能不红，要量的是比例，不是一次。

量法（rerun 第④⑤步，推的，待主 agent 跑）：没改坏时 `probe mutant=unmutated mode=sequential … probe_samples=40 probe_red=0`、`mode=concurrent … probe_red=0`；施加变异后 `mode=sequential … probe_samples=100 probe_red=<N>`、`mode=concurrent … probe_samples=<10×20> probe_red=<M>`，再 `name=replay_like round=1..30 named=ok|FAILED caught_at=left: Some(<轮>)`、末行 `name=replay_like_summary runs=30 named_failed=<K>`。我推本机空闲时 N 接近 100（每一轮放锁都要进内核，空当约一次系统调用），同时几份时 M/(10×20) 低一些，K 接近 30；这三个数都没量过。判法：K 小于 30 就说明「变异门禁那一趟抓到」靠运气，任务 12083 那一次存进库的「抓到」不能代表这一行；三个比例都是 100% 也只说明在这台机器、这个负载下稳定（第二台的页大小与负载另算）。

四句：1. 没有臂。2. 测试看得到判别它的东西（探锁那一刻 `replied` 有没有记上），前提是它抢得到锁。3. 满足的是 test-discipline「单次观测不算数」那一句，不是变异没被抓的三类里任何一类（这一次抓到了）。4. 跑前没有改法条款。

推翻它的现象：rerun 第⑤步 30 趟全红、100 次与同时几份那两格都是 100%，第二台上再量一次也一样；或判定库对这一行不复用（每趟照跑）。

### 打中 D：锁「从写请求到读完回应一直握着」只有一段有测试罩（推的，待主 agent 跑）

文件头第 5 行说三种请求都整问整答在一把锁里。探锁测试的替身每一轮只回「                        .write_all(b"absent\n")」（`crates/singlefs-device-library/src/lib.rs:817`），装置线程只调 `read`：它罩的是 `read` 写请求到读回应行之间这一段（变异 7598 改的正是这一段）。另外三段没有测试、也没有变异行：
- `found` 之后读内容：变异 `read-lock-released-before-found-contents` 在第 214 行之前放锁再拿锁。
- 单条 `write`：变异 `write-lock-released-between-request-and-reply` 在调用方把 `send_write_request_and_wait_for_written` 拆开，写完请求放锁、再拿锁读回应。
- `write_many`：变异 `write-many-lock-released-between-request-and-reply`，同上。

rerun 第⑥步每个变异跑整个 lib 目标五趟，我推每趟的 `failed=` 与没改坏时相同，都是 1（只有那条 2^62 的攻方测试红；原有十六条与别的攻方测试没有一条在两个线程上调写或读到 `found`）。这三处真放了锁时的后果（推的）：别的线程在空当里发请求、先读到不属于它的回应——读到上一问的 `written` 当认不出退 3、读到别人的 `failed 75` 退 75、读到别人 `found` 后面的内容当一行回应；多半退得出来，少数读成 `absent` 去现判。

四句：1. 没有臂。2. 测试看得到（替身知道自己回没回），只是没造这几种回应与调用。3. 满足的是第一轮判决第三节第 5 条「多线程测试换成抓得到「锁放在读请求与读回应之间」的造法」——实现照字面只修了读的请求与回应行之间；文件头那句承诺比测试宽。4. 跑前没有改法条款。

改法（只在我的模型上设计过、被攻过零轮）：把三种请求的「写请求加读完回应」各收进一个只拿 `&mut RelayEnds` 的函数（写那两种已经是 `send_write_request_and_wait_for_written`，`read` 那一段没有），函数里拿不到守卫、放不了锁，放锁只能改调用方的结构；再把探锁测试的替身改成 `found` 的回应行与内容分两次写，中间留探锁一方抢的空当，加一个写的装置线程。

推翻它的现象：rerun 第⑥步任何一个变异下，除那条 2^62 的攻方测试之外还有测试红。

## 攻击面四：从未被看过的地方

### `give_up` 退整个进程与 libtest 并行：成立（不静默），附一处代价

父测试进程里调口子的只有五条原有测试与探锁测试，喂的回应只有 `found`、`absent`、`written`，没有一条走到 `give_up`；会退进程的七条都在子进程里，子进程照名字只跑那一条，`exit` 只带走它自己。逐行看任务 12083 那九行与第一轮七行变异：没有一行让父进程里的测试走到 `give_up`（推的）。

代价（推的）：内存那一种这一批起核键、坏键 `give_up`，E158、E174、E178 的单测都在父进程里用内存那一种。将来哪个调用方的变异把键拼坏（例 `format!("items/{key} .out")`），整个测试进程退 3，点名的那条测试那一行打不出来，cargo 报 `process didn't exit successfully … exit status: 3`；变异门禁照 `.claude/rules/mutation-sampling.md` 第 79 行的判法把这一行归「无效」，不归「抓到」，出路写成「测试进程被杀先查是谁杀的」，查到的是 `give_up` 的一句「相对键不合」。不静默，整道照样红，只是归错栏。另一个极端：父进程里有测试喂到 `failed 0`，整个测试进程退 0、后面的测试没跑而 cargo 报绿；现在没有测试喂它，转交也不发（第一轮已核），只记下。

### `read` 的回应解析：修补后没变（只加一行 `filter`），没被看过的一处

改动全文第一节 `@@ -173,16 +203,18 @@` 那一块只加了注释与 `filter` 一行；读内容仍是 `vec!` 加 `read_exact`，再「                String::from_utf8(contents).ok()」（`crates/singlefs-device-library/src/lib.rs:221`）。内容不是 UTF-8 时交 None，与文档句「    /// 读一条：不在（或已撤掉）交 None；读失败照退出码退，不交 None。」（`crates/singlefs-device-library/src/lib.rs:187`）不符：调用方读成「不在」、去现判，不退。攻方测试 `observed_read_answered_found_with_bytes_that_are_not_utf8_reads_as_missing` 喂 `found 2\n\xff\xfe`，断言交 None（推的，待主 agent 跑：现状下绿）。可达性：库里的值是装置自己用 `&str` 写的，撤回标记由转交回 `absent`，我没找到别的写这一族这一种的写者（只看了 `device-code-key.py` 与三家调用方），走不到，所以只记「没被看过、与文档不符」，不记打中。这一处改动之前就在。

另一处同类：回应的失败那一句由转交用 `surrogateescape` 编码，句子里要是带了非 UTF-8 字节，`reply_line` 的 `read_line` 交 `InvalidData`，装置退 3 而不是回应里的 75（推的；`PruneServerFailure` 的话里会不会带这种字节，我没追）。

## 第四节检查单八格

| 格 | 回 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 产品段（第 1–309 行）现跑 grep `panic!\|unreachable!\|todo!\|assert\|.unwrap(\|.expect(\|_ =>`：两处 `panic!`（第 285、299 行）与锁的 `expect`。`count_entries` 第一轮没追：调用点 E158 第 29955、29959、30035 行（`#[cfg(test)]` 第 29232 行起的模块，到第 30195 行文件尾）、E178 第 7693 行（第 7075 行 `#[cfg(test)]` 起、第 7823 行收尾的 `mod tests`）、E174 第 4187 行（第 3870 行起、第 4328 行收尾），都在测试模块里，传的都是内存那一种 |
| 类型与名字 | 成立 | 理由常量第 31、33 行写的「持锁时只做…」与三处持锁段逐句对得上（拼请求、管道读写、按长度分配、`give_up`；`BTreeMap` 插入查找数键、次数加一）；`_ =>` 现跑 grep 0 处；名字没有缩写 |
| 不兼容旧代码与旧格式 | 成立 | 现跑 `grep -nE 'legacy\|old\|migrat\|兼容' research/prompts/_device-library-write-many-code-r2-diff.md` 两行：第 591 行 `holds` 里的 old、第 732 行 `old mode 100755`。后者是转交脚本的执行位去掉了（暂存区 `mode change 100755 => 100644`，量的）；`replay.sh` 第 4167、4178 行与 `scripts/gate/admission.py` 第 8138 行都经 `python3` 起它，不受影响 |
| 不新建判定库 | 成立 | 内存那一种改动前就有，测完即丢 |
| 所有任务多机可用 | 成立（第二台没核） | 子进程只读 `current_exe`（第 536 行）与一个环境变量；探锁测试依赖管道缓冲小于 256 KiB（第 766–767 行），本机 4096 字节页、x86_64（量的），第二台没核 |
| 测试优先多线程 | **推不出（打中，同 C）** | 抓法够不够没量过，量法见打中 C |
| 带 --item 的命令与项 | 成立 | `task-server/task log 12083 --all` 九行逐行「抓到」、末行「✓ 点名 9 条：现判 2 条、复用 0 条、拒算 7 条；跑了 9 条，抓到 9 条…」；九行在变异表第 7590–7598 行、表共 7598 行（量的，`wc -l`）；`--list-items` 本轮没跑，命令交主 agent |
| 新崩溃流多罩了什么 | 成立 | 现跑 `grep -n 'device-library\|device_library' .claude/gate.d/stage-inputs.tsv .claude/gate.d/crash-point-replay-items.tsv` 无输出、退 1 |

## 改法与各修哪一格（全部只在我的模型上设计过、被攻过零轮）

| 改法 | 修哪一格 | 标 |
|---|---|---|
| `read` 在拿锁、拼请求之前照 `give_up_unless_the_relay_accepts_the_key` 核键，两种后端都核；配一条子进程测试（键带换行与空格各一）与变异行「read 不核相对键」 | 打中 A、打中 B | 推的 |
| `read` 的写请求到读完内容收进一个只拿 `&mut RelayEnds` 的函数（与两种写共用的 `send_write_request_and_wait_for_written` 同形），放锁只能改调用方的结构 | 打中 D（读），也让 7598 那一类变异不再靠时序 | 推的 |
| 探锁测试扩一种替身：`found` 的回应行与内容分两次写、中间留空当；再加一个调 `write` 的装置线程 | 打中 D | 推的 |
| 变异门禁对靠时序抓的行不复用「抓到」（或连抓满 5 次才存），像拒算那样每趟照跑；不在被判文件里，归工具员 | 打中 C | 推的 |
| 7598 那一行这一轮先照 rerun 第⑤步量出比例再定留不留；比例不是 100% 的，那一行的「抓到」不当作数 | 打中 C | 推的，待主 agent 跑 |
| `found` 的长度先与转交的值上限比、或用 `try_reserve_exact`，分配不出也走「回应认不出退 3」 | 攻击面二推不出那一处 | 推的 |
| 内容不是 UTF-8 时 `give_up_on_reply` 退 3，或把文档句改成「内容不是 UTF-8 读成不在」 | 攻击面四没被看过的一处 | 推的 |

## 要主 agent 跑的命令

1. 模型整份复跑（`rerun.sh` 里每条 `cargo test` 各自带了范围声明，种类 target；外面这一条是驱动脚本，照第一轮任务 12060 的写法不另带）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/device-library-write-many-code-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-opus/copy --work /home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-opus/rerun-1
```

   - 在哪起：仓根（路径全是绝对路径）。`--frozen` 是我开工时用 `minimal-copy.py` 建的仓副本，lib.rs 的 sha256 与快照相同（量的），草稿目录 `.deliverables` 写成「交接」。照判决规则在模型目录的拷贝里起时，把第一个路径换成拷贝里的 `tools/rerun.sh`，`--work` 给一个新目录。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`（脚本照常退 0；非 0 是脚本某一步没跑成，多半是攻方测试草稿编不过，那是草稿的错）。`task-server/task log <号> --all` 里：
     - `name=relay_model rc=0`，三格 `injected=True`、两格对照 `injected=False`，`name=summary hits=3`（打中 A 转交那一半）；
     - `name=tests_unmutated round=1..5`：每趟 `failed=1`，红的只有 `read_answered_found_with_a_length_that_cannot_be_allocated_exits_with_the_other_error_code`（下面一段 `left: None`、`memory allocation of … failed`）；三条 `observed_*` 都 `ok`（打中 A 装置那一半、打中 B、UTF-8 那一处）；
     - `name=key_rules_cross_check rc=0`、`name=summary differ=0`；
     - `name=probe mutant=unmutated` 两行 `probe_red=0`；`mutant=read-lock-released-between-request-and-reply` 两行的 `probe_red` 与 `probe_samples`，`name=replay_like_summary runs=30 named_failed=<K>`（打中 C 要的三个比例）；
     - `name=tests_mutant mutant=<攻方三个变异> round=1..5` 每趟的 `failed=` 与红的是哪几条（打中 D：我推与没改坏时相同，都是 1、只有那条 2^62 的攻方测试）。
   - 跑完要删：`--work` 下的 `tree/`（仓副本，含 `tree/target/` 编译目录，或服务给的编译目录照 `task-server/task clean <号>`）。
   - 挂钟：推的，十分钟上下。要比两台，同一条命令换一个 `--work` 再交一次，看 `probe_red` 的比例变不变。

2. 列项（检查单「带 --item 的命令与项」那一格的 `--list-items`，照第一轮任务 12056 的写法与范围声明交）：`task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items /home/fy5090/code/singlefs`，在仓根起；看输出里九个「判定库口子」新名（第 7590–7598 行）都列出、末行键对不上的条数。

3. 引文核对（直接跑的文本工具，我交回前已跑，见文末）：`python3 scripts/agents/cite-check.py research/prompts/device-library-write-many-code-r2-opus-output.md`，在仓根起，看末行。

## 没打中的形状（试了哪些、取样多大）

| 维度 | 取的值 |
|---|---|
| 相对键（两份规则） | 空串、`.`、`..`、`/`、`/abs`、`a//b`、`../x`、`items/..`、`-`、空格、制表符、`\r`、`\n`、不换行空格、`é`、U+0663、4096 字节长键；rerun 第③步另扫全部 Unicode 标量 |
| `read` 的键 | 带换行后接空写、八字节写、`write-many`；带空格；合规 |
| `found` 长度 | `isize::MAX + 1`（已有测试）、2^62、合规小值、非 UTF-8 内容 |
| 子进程测试 | 七条逐条问「同一个退出码还能从哪条路来」 |
| 探锁测试 | 没改坏 40 次加同时几份 10 批；变异 100 次、同时几份 20 批、整个 lib 目标 30 趟（都待跑） |

没打中的：
- 两份键规则判得不一样：逐个边界相同（上面「两份规则」一节）。`..` 越出这一种：键是字面拼接，越不出去。
- `found` 的长度写成 `+5`：Rust 的 `usize` 解析收前导 `+`，转交只发 `len(data)` 的十进制，走不到。
- 坏键在 `write_many` 批里第几条：拿锁之前整批核完，第几条都一样一个字节不发（第 259–261 行）。
- 子进程测试名写错：子进程跑 0 条退 0，父进程的码断言红，不静默。
- 标准错误关掉那条测试：别的测试同时起子进程时，那一刻复制出去的读端会让写进得去（变异下不 panic、不红）；窗口只在复制到 exec 之间，我推很小，没量。
- 探锁测试没改坏也红：替身先记 `replied` 再写 `absent`，装置读完才放锁，探锁一方抢到锁时 `replied` 一定已记上（第 815–818 行），我推不会误红；rerun 第④步量。
- 转交脚本丢了执行位：调用处都经 `python3` 起（检查单那一格）。

## 这条腿自己的限度

- 一个数都没跑：模型、攻方测试草稿、三个变异全是写出来交主 agent 跑的。Rust 草稿我没编过，编不过时 rerun 第②步 `rc` 非 0、`ok=` 偏少，那是草稿的错、不是被判代码的错。`apply_once.py` 照名字从副本里的变异表取 7598 那一行原样施加，名字或原文对不上它退 1、rerun 停在那一步。
- 几处 std 的行为是我印象里的、没在本仓核（本机三套工具链都没装 rust-src）：futex 锁「有人等过」时放锁进内核叫醒、分配过大中止进程、`std::io::stderr()` 句柄不经 libtest 捕获、`usize` 解析收前导 `+`。打中 C 的推理用到第一条；测试第 770 行的注释写的是同一件事。
- 打中 C 要的是比例：rerun 只在服务派到的那一台上量，负载随那一刻的机器；比两台要交两次。
- 打中 A、B 在现有三家调用方上走不到，可达性用的是第一轮判决给打中 2 的那把尺子；主 agent 要是换尺子，这两格跟着降成「没被看过、走不到」。
- 模型载入的是仓根此刻的 `device-code-key.py`（输出头一行打 sha256）；判决之前它改了，模型的结论跟着变。
- 我读过的范围：被判文件整份；`device-code-key.py` 第 575–790 行与改动全文第二节；`prune_server_client.py` 第 1255–1300 行；`crates-mutation-replay.sh` 第 104–125 行、`crates-mutation-rows.py` 的 grep 命中；E158、E174、E178 的读写调用点与测试模块边界；任务 12083 的整份日志。别处没读。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；第五节第 1 格（第一轮五条改法逐条修到没有）不在我的分工里，只在打中 A、D 里顺带写了第 1、5 条的字面与范围。
- 没往统一任务服务交任务，判决要引的数一个都没复跑（命令在「要主 agent 跑的命令」）；副本上量出的数（等主 agent 跑出来）不算入库装置上的数。
- 没读禁读清单里的两份。
- 没删 `/home/fy5090/.local/share/singlefs/work/device-library-write-many-code-r2-opus/copy`（仓副本，53M，`du -sh` 量的）：交接给主 agent 当 rerun 的 `--frozen`，草稿目录 `.deliverables` 已写。没有建编译目录。草稿目录里另有 `task-12083.txt`（任务 12083 日志的拷贝）、`minimal-copy.txt`、`rerun-steps.part`（拼 rerun.sh 用的那一段），都是普通文件。

## 引文核对（交回前现跑）

`python3 scripts/agents/cite-check.py research/prompts/device-library-write-many-code-r2-opus-output.md`（仓根起）末行：

```
  ✓ 核了 20 处引文、6 处裸行号，对上 20 处，没判 6 处（其中裸行号没有引文可比 6 处；逐处列在上面）
```
