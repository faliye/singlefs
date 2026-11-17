# device-library-write-many-code-r1 正推腿（Sonnet）报告

立场：正推。从 `crates/` 今天的实现与已定条款、文件头协议推「应该是什么」，再逐格与被判对象比。方案按 `crates/` 今天的实现来谈。
被判文件：`crates/singlefs-device-library/src/lib.rs`（共 435 行，量的：`wc -l`）。开工快照 sha256 与此刻文件逐字节相同（量的）：

```
60dd1dfda44ff8c55876058a11ea4a6c28a3af5000406d2ebca9d759252dbd29  crates/singlefs-device-library/src/lib.rs
```

没写模型、没有模型目录、没有 `rerun.sh`（这一轮是读代码对条款，不是建模）。批次：`records/2026-11-10-门禁与实验欠账收尾调度.md#5`。

## 先说结论

- 格 1（文件头协议三行对服务侧约定）：逐句对下来，只有一句对不上：lib.rs 文件头第 9 行「一条请求里的写在服务那一头合成一次提交」与客户端「每帧一个写批，原子性只在一帧之内」冲突（问题 1 第 4 句，整行并排抄在那里）。服务侧文件头第 55 行「一条请求只等一次 WAL 同步」是同一个毛病。别的句子一致。
- 格 2（五条攻击面）：1、2、3、5 从正面推下去成立（装置这一头不会把没写的当写了、字节计数对、重构与改前逐字相同、整问整答在一把锁里）；攻击面 1 里的「整批原子」只在一帧之内成立（同上）；攻击面 4 成立。
- 格 3（八格）：七格成立；「带 --item 的命令与项」一格里正文写的「本轮改动没有这一类」不成立：变异表新加的 crates-mutation-replay 行就是 `--item` 的项，列不列得出这一步推不出，要主 agent 跑 `--list-items`（问题 3 第 7 格）。
- 背景材料有两处不准（不改变任何格的判定）：新加的 `expect` 在非测试代码里是三处不是两处；`#![forbid(unsafe_code)]` 是这份改动里新加的一行，背景材料没提（问题 3 第 1 格）。

## 要主 agent 跑的命令（汇总；每条的用意在对应小节）

1. 本包的单测（核问题 1、2、3 里「读走一个 written」「请求字节」「空批」「内存计数」这几处推导，和变异行 7393-7399 点名的五条用例）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-device-library 的 lib 测试目标; 范围=1 个测试目标（本包 lib，共 5 条测试）; 为什么=核正推腿从代码推出的请求字节、读走一个 written、空批不发请求、内存计数一批一次' task-server/task submit -- cargo test -p singlefs-device-library --lib`
   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：`TASK_SERVER_RESULT` 那一行 `type=` 与 `exit=0`；`task-server/task log <号>` 里 `test result: ok. 5 passed` 一行；有 `FAILED` 或 `✗` 就是红。
2. 变异表新加的行能不能被 `--list-items` 列出（问题 3 第 7 格）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 的 --list-items; 范围=只列不跑，不起 cargo; 为什么=核变异表第 7393 至 7399 行是否作为可点名的项列出' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items /home/fy5090/code/singlefs`
   - 在哪个目录起：仓根。
   - 看哪几行：逐行列表里有 `判定库口子 write-many 头一行多报一条` 等七个变异名（行号 7393 至 7399）；末了「锚点失效」一栏不点名这七行；`exit=0`。
3. 这七行真的改坏、点名的测试真的红（我只推了每行被哪条测试抓到，没跑）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 的变异表第 7393 至 7399 行; 范围=7 行; 为什么=核判定库口子成批写的七条变异都被点名的测试抓到' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --item 7393 --item 7394 --item 7395 --item 7396 --item 7397 --item 7398 --item 7399 /home/fy5090/code/singlefs`
   - 在哪个目录起：仓根。
   - 看哪几行：成功行写明只判了这七行、全抓到；退 0 算过，退 3 或 1 看日志里点名的行。
4. 服务侧一半（假装置发 write-many 的几格，证 `device-code-key.py` 文件头第 54-57 行那一侧）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=device-code-key.py 的 --selftest（临时服务底下）; 范围=整份自证，含 write-many 六格; 为什么=核转交侧 write-many 的整批写、撤回、哨兵、服务不可用、键不合、条数 0 各格' task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/device-code-key.py --selftest`
   - 在哪个目录起：仓根。
   - 看哪几行：末尾「device-code-key 自证：N 格，红 M 格」，M 为 0；`write-many` 开头的各格没有 `✗`。

## 问题 1（格 1）：lib.rs 的 write_many、send_write_request_and_wait_for_written、文件头协议，对服务侧约定

先摆代码事实，再逐句判。引文都在被引文件里 `grep -nF` 过，行号是那一份文件自己的行号。

代码事实：

- `write_many` 空批直接返回：「if entries.is_empty() {」（`crates/singlefs-device-library/src/lib.rs:222`）。
- 头一行报条数：「let mut request = format!("write-many {}\n", entries.len()).into_bytes();」（`crates/singlefs-device-library/src/lib.rs:229`）。
- 每条报内容的字节数：「request.extend_from_slice(format!("{relative} {}\n", text.len()).as_bytes());」（`crates/singlefs-device-library/src/lib.rs:231`）。
- 整批只走一次发请求、等回应：「send_write_request_and_wait_for_written(&mut ends, &request, &asked);」（`crates/singlefs-device-library/src/lib.rs:234`），函数里只读一行回应、不是 `written` 就退：「if reply != "written" {」（`crates/singlefs-device-library/src/lib.rs:115`）。
- 锁拿在整批之前，到分支结束才放：「let mut ends = ends.lock().expect("转交判定库的锁没有中毒");」（`crates/singlefs-device-library/src/lib.rs:227`），守卫 `ends` 的作用域是 227 至 235 行那个分支块（读文件所见）。
- 服务侧读整批：「def answer_write_many(self, count, requests):」（`scripts/experiment-apparatus/device-code-key.py:655`），先逐条读头与内容并核键，核完才记账（670-677 行），最后写服务：「done, _ = self.through_service(lambda: self.store.write_many(sent))」（`scripts/experiment-apparatus/device-code-key.py:679`）。

### 逐句对

| # | 被判的句子 | 另一头 | 判定 |
|---|---|---|---|
| 1 | 「//   请求 write-many <条数>\n 后接每条 <相对键> <字节数>\n<内容>    整批回一个 written\n（服务确认整批落盘之后才回）：」（`crates/singlefs-device-library/src/lib.rs:8`）的请求格式 | 服务侧「write-many <条数>，后接每条 <相对键> <字节数> 加内容 → 整批一个 written：整批读全、核全之后才动手，每条照 write 记账（写过的键、复用哨兵），」（`scripts/experiment-apparatus/device-code-key.py:54`）；解析在 `device-code-key.py:645`（条数）与 663-669（每条头与内容） | 一致。Rust 发的字节与服务按字节读的次序逐段对上；单测断言的字节串 `write-many 2\nitems/multibyte.out 6\n判定items/ascii.out 2\nok`（`crates/singlefs-device-library/src/lib.rs:392`）与服务的读法吻合（推的：没跑，见命令 1） |
| 2 | 「整批回一个 written\n」 | `device-code-key.py:680`：「return b"written\n" if done or break_is_set("write-many-failure-reported-as-written") else self.failed()」；Rust 只读一行：`crates/singlefs-device-library/src/lib.rs:114` | 一致。弄坏开关只在自证里认（`device-code-key.py:95`） |
| 3 | 「服务确认整批落盘之后才回」 | 客户端 `write_batches` 逐帧等确认并核条数：「if written != len(batch):」（`scripts/prune-server/prune_server_client.py:853`）；服务侧 `device-code-key.py:679-680` 写完才回 | 一致（回来时每一帧都已确认；是否「一次」见第 4 句） |
| 4 | 「//     一条请求里的写在服务那一头合成一次提交，整问整答在锁里，别的线程只等这一条；条数至少 1。」（`crates/singlefs-device-library/src/lib.rs:9`）前半句「合成一次提交」 | 客户端：「# 一帧装的上限：远低于服务的帧长上限，键多、值大时切成几帧（每帧一个写批，原子性只在一帧之内）」（`scripts/prune-server/prune_server_client.py:346`），上限「FRAME_BATCH_BYTES = 16 * 1024 * 1024」（`scripts/prune-server/prune_server_client.py:347`）、「FRAME_BATCH_KEYS = 8192」（`scripts/prune-server/prune_server_client.py:348`）；`write_many` 的文档串「"""[(键, 值)] 按帧上限切成几帧写；同一帧里的原子（文件头「键空间」：一份判定与它的 items 放同一帧）。」（`scripts/prune-server/prune_server_client.py:857`）| **冲突**（见下）。服务侧第 55 行「一条请求只等一次 WAL 同步」同毛病 |
| 5 | 「整问整答在锁里，别的线程只等这一条」 | `crates/singlefs-device-library/src/lib.rs:227` 拿锁、守卫活到分支块末；发请求与读回应都在 `send_write_request_and_wait_for_written` 里，锁没松 | 一致 |
| 6 | 「条数至少 1」 | Rust 空批不发（`crates/singlefs-device-library/src/lib.rs:222`），服务 0 条拒：`device-code-key.py:645` 的 `int(words[1]) > 0` | 一致，两头对得上：Rust 永远不发 0 条，服务拒 0 条 |
| 7 | 「//   三种都可能回 failed <退出码> <一句>\n：这里打 ✗ 与那一句、进程照那个退出码退（75 是 prune-server 不可用，3 是别的错），不当…现判。」（`crates/singlefs-device-library/src/lib.rs:10`；「…」处是「键不在」三个字加引号）| 服务侧失败后每个请求都照它回：write-many 在 `device-code-key.py:680`（`self.failed()`），读在 629，write 在 644；Rust 一侧 `crates/singlefs-device-library/src/lib.rs:115-117` 对不是 `written` 的回应一律进 `give_up_on_reply`（`crates/singlefs-device-library/src/lib.rs:94-104`，`failed ` 前缀照码退，别的退 3） | 一致 |
| 8 | 「相对键只用字母、数字与 . _ - /，不带空白（那一侧照这一条核，不合的回 failed 3，整批一条都不写）。」（`crates/singlefs-device-library/src/lib.rs:11`）| 键的正则「RELATIVE_KEY_PATTERN = re.compile(r"[A-Za-z0-9._/-]+")」（`device-code-key.py:205`）；核键发生在写服务之前：「if not key_is_valid:」（`device-code-key.py:664`）直接返回，写服务在 679 行 | 一致。注意 Rust 一侧 `write_many` 不预核键（与单条 `write` 同），不合的键由服务回 failed 3，装置照退 3 |
| 9 | 服务侧第 53 行：「write <相对键> <字节数> 加内容  → written（服务确认落盘之后才回）；先记进这一趟写过的键再写服务（没确认的写也可能落了盘，撤的时候一并撤）」| 单条 write 的服务侧在 `device-code-key.py:635-644`（先记键 639-641、note_write 642、再写服务 643）；Rust 一侧经重构后的 `send_write_request_and_wait_for_written` 发（`crates/singlefs-device-library/src/lib.rs:208`） | 一致 |
| 10 | 服务侧第 54 行「整批读全、核全之后才动手，每条照 write 记账」 | 读全核全在 `device-code-key.py:657-669` 的循环；之后才 670-677 记账（`written` 与 `note_write`） | 一致 |
| 11 | 服务侧第 56 行「读写服务没成（PruneServerFailure）→ failed <退出码> <一句>，此后每个请求都照它回、不再连服务；装置照那个码退」 | `through_service`：失败后不再连：`device-code-key.py:609-610`；`failed()` 在 617-619 | 一致 |
| 12 | 服务侧第 57 行「认不出的请求、相对键不只是字母数字与 . _ - / → failed 3（write-many 里有一条不合的，整批一条都不写；装置收到就退，那条请求后面没读的字节不再对得上请求的边界）」 | 「整批一条都不写」：不合就在循环里返回（664-665），没到 679；「装置收到就退」：`give_up_on_reply` 对 `failed 3 …` 退 3（`crates/singlefs-device-library/src/lib.rs:94-99`）| 一致。补一条观察见问题 2 攻击面 1 |

### 第 4 句的冲突，两边整行并排

```
crates/singlefs-device-library/src/lib.rs:9
//     一条请求里的写在服务那一头合成一次提交，整问整答在锁里，别的线程只等这一条；条数至少 1。
scripts/prune-server/prune_server_client.py:346
# 一帧装的上限：远低于服务的帧长上限，键多、值大时切成几帧（每帧一个写批，原子性只在一帧之内）
```

推导（正推）：

1. 服务侧经客户端 `LibraryStore.write_many` 一次交整批：「process_client().write_many([(self.key(relative), …」（`scripts/prune-server/prune_server_client.py:1299`，方法从 1298 行起）。
2. 客户端 `write_many` 先把大于一段的值分段（`scripts/prune-server/prune_server_client.py:857` 起的函数体：登记键、各段、清单分几步写），最后「self.write_batches(final_entries)」（`scripts/prune-server/prune_server_client.py:915`）；`write_batches` 按帧切（`scripts/prune-server/prune_server_client.py:849-853`）。
3. 所以一条请求要装进「一次提交」，条件是：这一批的键不超过 8192 条、键加值加每条 9 字节开销合计不超过 16 MiB（切分条件在 `batches_of`：「if batch and (size + item_size > FRAME_BATCH_BYTES or len(batch) >= FRAME_BATCH_KEYS):」（`scripts/prune-server/prune_server_client.py:1110`），每条的开销在「lambda entry: len(entry[0]) + len(entry[1]) + 9」（`scripts/prune-server/prune_server_client.py:851`）），并且没有一个值大过一段（16 MiB）或以分段清单的魔数起头。条件不满足时，一条请求变成几次提交，服务在两次提交之间挂掉时，库里留下这一批的一部分。
4. 回到文件头第 9 行：这句话在批小的时候是真话，批大时不是。`write_many` 对 Rust 一侧的调用方没有大小上限（`crates/singlefs-device-library/src/lib.rs:221` 的签名接任意长的切片）。

这一冲突影响什么：影响「整批原子」这个说法，不影响「回来时整批都已确认」（第 3 句）。唯一的现役调用方是 E158 按单位成批写（`crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:24484`）；一个单位的判定数有没有可能超过 8192 条，我没量（推不出）。续跑只补没判的这一半仍成立，因为每条判定按自己的键独立存、读回时按键核头（「if header != Self::item_header(key) {」，`crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:24468`）；原子这一半在超过一帧时不成立。

什么现象会推翻这一条冲突：(a) 客户端在 `write_many` 之外还有一层把整个调用并成一帧（我读到的是没有，`scripts/prune-server/prune_server_client.py:849-915`）；(b) E158 一个单位的判定数、字节数有硬上限小于 8192 与 16 MiB，则文件头第 9 行对现役调用方成立，仍不是对接口成立。

## 问题 2（格 2）：第三节五条攻击面，从正面推

### 攻击面 1：原子性与确认

推导 1（回来时整批是否都已落盘）：Rust 一侧只有一条路返回：回应恰好是 `written`。函数里「if reply != "written" {」（`crates/singlefs-device-library/src/lib.rs:115`）之后任何别的回应都进 `give_up_on_reply` 退出（`crates/singlefs-device-library/src/lib.rs:94-104`）。服务侧只在写服务返回之后才回 `written`：「done, _ = self.through_service(lambda: self.store.write_many(sent))」（`scripts/experiment-apparatus/device-code-key.py:679`）与「return b"written\n" if done or break_is_set("write-many-failure-reported-as-written") else self.failed()」（`scripts/experiment-apparatus/device-code-key.py:680`）；客户端每帧核条数（`scripts/prune-server/prune_server_client.py:853`）。吞掉失败的唯一一条路是弄坏开关，它只在自证里认（`scripts/experiment-apparatus/device-code-key.py:95`）。判：成立（确认）。整批「一次提交」的原子性只在一帧之内，见问题 1 第 4 句。

推导 2（服务侧整批拒时装置是否退、会不会把没写的当写）：键不合在循环里就返回认不出的请求（`scripts/experiment-apparatus/device-code-key.py:664-665`），此时记账与写服务（670-680 行）都还没做，所以服务里一条没写；Rust 收到 `failed 3 …` 走 `give_up_on_reply` 退 3。Rust 没有任何路径把「不是 written」的回应当成功。判：成立。

观察（服务侧文件头第 57 行已承认「那条请求后面没读的字节不再对得上请求的边界」，这里补它的后果；推的，没跑）：转交线程在回完 `failed 3` 之后回到循环、继续读请求：「answer = self.answer(line, requests)」（`scripts/experiment-apparatus/device-code-key.py:599`）读的下一行就是被拒那一批剩下的内容行。内容行认不出就多回一条 `failed 3`（写进没人读的管道），只有内容里恰好含一整条合法 `write <键> <字节数>` 请求时才会真写服务。装置在收到第一条 `failed` 时已经退出，管道随之关闭。单条 `write` 键不合时服务侧也是同一个行为，不是这份改动新带来的。什么现象推翻：用一个内容里含合法请求行的批试一次，库里没有那条键。

推导 3（管道半途断开，锁与后续请求的边界）：
- 请求写到一半：`write_all` 报错 → 退 3（`crates/singlefs-device-library/src/lib.rs:108-113`）。Rust 运行时对 SIGPIPE 的处理不在本包里，我没验；如果进程被信号杀掉，也是同样不再有后续请求。
- 回应读到一半：EOF 退 3（`crates/singlefs-device-library/src/lib.rs:81-85`）；读到半行 `writt` 之类，不等于 `written`，走 `give_up_on_reply` 的「认不出」退 3（`crates/singlefs-device-library/src/lib.rs:100-103`）。
- 锁：这些退出都是 `process::exit`，不返回；持锁的守卫不会被别的线程拿到（进程已没了）；`expect("转交判定库的锁没有中毒")` 触发要有线程在持锁时 panic，本包里持锁的代码里没有 panic 点（`give_up` 是 exit，不是 panic；读 `crates/singlefs-device-library/src/lib.rs:157-198` 与 `:201-244` 所见）。
- 服务侧：头或内容读不全（装置中途退了）就返回 None，转交循环结束（`scripts/experiment-apparatus/device-code-key.py:660-661` 与 `667-668`）。
判：成立。什么现象推翻：任何一个线程在持锁时 panic（本包里没找到这样的点）；或回应管道里留着没读的回应字节，使下一问读到上一问的回应（Rust 每问必读一个完整回应，退出之外没有「问了不读」的路径）。

### 攻击面 2：字节计数

推导：Rust 报的是 `str::len`，即字节数：「request.extend_from_slice(format!("{relative} {}\n", text.len()).as_bytes());」（`crates/singlefs-device-library/src/lib.rs:231`）；单测断言两个汉字报 6（`crates/singlefs-device-library/src/lib.rs:392-393`）；变异行 7393-7399 里「每条报字符数而不是字节数」那一行（`crates/mutations.tsv:7394`）把 `text.len()` 换成 `text.chars().count()`，测试里「判定」会报 2 而不是 6，请求字节串对不上，会红（推的，命令 3 核）。服务侧按字节读：「data = requests.read(int(fields[1]))」（`scripts/experiment-apparatus/device-code-key.py:666`），读不满长度就当请求断了（667-668 行）。
- 内容里带换行：服务按计数读，不按行读，内容里的换行不会被当成下一条的头。成立。
- 内容为空串：Rust 报 `key 0\n` 后不接字节；服务 `"0".isdigit()` 为真、`read(0)` 得空、长度相等，进入下一条。读回时 `data` 是空字节不是 `None`，服务回 `found 0`，Rust `read_exact` 读 0 字节得 `Some("")`。成立。服务端存储是否接受空值，我没查（不在这份改动里，单条 `write` 同样走）。
- 服务侧 `isdigit` 对非 ASCII 数字也为真而 `int()` 会抛：量的（`python3 -I` 现跑：`'²'.isdigit()` 为 True、`int('²')` 抛 ValueError）。Rust 发的永远是 ASCII 十进制，这条只对别的发送方有意义，不在被判范围，不算打中。
判：成立。什么现象推翻：命令 1 里 `write_many_announces_the_count_and_each_length_in_bytes_and_takes_one_written_for_the_whole_batch` 红，或服务侧自证（命令 4）的 write-many 各格红。

### 攻击面 3：抽出的 send_write_request_and_wait_for_written，单条 write 行为与改前逐字相同吗

对照改前（`git show HEAD:crates/singlefs-device-library/src/lib.rs` 第 176-193 行，量的）：

```
183:                 if let Err(error) = ends.requests.write_all(&request) {
184:                     give_up(
185:                         RELAY_BROKEN_EXIT_CODE,
186:                         &format!("{asked}：写不进转交判定库的管道：{error}"),
187:                     );
188:                 }
189:                 let reply = reply_line(&mut ends.replies);
190:                 if reply != "written" {
191:                     give_up_on_reply(&reply, &asked);
192:                 }
```

现在的函数体是同样七条语句：「if let Err(error) = ends.requests.write_all(request) {」（`crates/singlefs-device-library/src/lib.rs:108`）、退出码与话「&format!("{asked}：写不进转交判定库的管道：{error}"),」（`crates/singlefs-device-library/src/lib.rs:111`）、「let reply = reply_line(&mut ends.replies);」（`crates/singlefs-device-library/src/lib.rs:114`）、「give_up_on_reply(&reply, asked);」（`crates/singlefs-device-library/src/lib.rs:116`）。单条 `write` 的请求字节构造两行没动：「let mut request = format!("write {relative} {}\n", text.len()).into_bytes();」（`crates/singlefs-device-library/src/lib.rs:206`），之后只剩调用：「send_write_request_and_wait_for_written(&mut ends, &request, &asked);」（`crates/singlefs-device-library/src/lib.rs:208`）。差别只有类型：`&request`（`&Vec<u8>`）变成 `&[u8]`，`&asked`（`&String`）变成 `&str`，字节与文字不变；锁守卫仍在调用方手里，到分支末才放。判：成立（逐字相同）。
什么现象推翻：命令 1 里已有的 `write_announces_the_length_in_bytes_not_characters_and_returns_after_written` 红。这条测试只钉了「请求字节」与「读走 written」，失败的退出码与话（`give_up` 一路）没有测试钉住，因为 `give_up` 是 `process::exit`，单测里起不了；这一半靠逐语句对照，不靠测试。

### 攻击面 4：内存那一种与 count_write_many_calls 会不会让真管道那一侧 panic；空批两头对不对

- `count_write_many_calls` 在真管道上 panic：「Backend::Relay(_) => panic!("单测只数内存那一种"),」（`crates/singlefs-device-library/src/lib.rs:263`；同样的一句在改前就有的 `count_entries` 里：`crates/singlefs-device-library/src/lib.rs:249`）。调用点现查（量的，`grep -rn count_write_many_calls` 跑在仓根，限 `.rs .py .sh .toml`，不含 research 与 target）四处命中，一处是定义（`crates/singlefs-device-library/src/lib.rs:261`），三处是调用点：`crates/singlefs-device-library/src/lib.rs:430`（本包 `mod tests` 里）、`crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:30030` 与 `:30043`。后两处在「#[cfg(test)]」（`crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:29232`）紧跟的「mod fifth_run_tests {」（`crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:29233`）里，该模块到文件末尾 30195 行才收（量的：29233 之后第一个行首 `}` 在 30195 行）。所以非测试编译里没有调用点：这条 panic 走不到。判：成立。
- `replies_left_unread` 在内存那一种上 panic（`crates/singlefs-device-library/src/lib.rs:373`）：定义在 `mod tests` 里（`crates/singlefs-device-library/src/lib.rs:362`），只有 386、402 两处调用，两处的对象都来自 `relay_with_prepared_replies`（`crates/singlefs-device-library/src/lib.rs:380` 与 `:399`）。走不到。判：成立。
- 空批两头：Rust 空批返回（`crates/singlefs-device-library/src/lib.rs:222`），不发、不等；服务拒 0 条（`scripts/experiment-apparatus/device-code-key.py:645`）。两头一致；服务侧 0 条的拒绝由自证格「write-many 0：回 failed 3、装置照它退 3」钉住（`scripts/experiment-apparatus/device-code-key.py:1441`）。
- 内存那一种与真管道的语义差：内存里同一批的重复键后写覆盖先写（`BTreeMap::insert`，`crates/singlefs-device-library/src/lib.rs:240`）；真管道那一头重复键在同一帧里怎么落，取决于服务写批按次序应用，我没读服务的存储层，推的。调用方 E158 的待写批是按键的映射：「type JudgementsAwaitingWrite = BTreeMap<String, String>;」（`crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:24313`），键不会重复，所以现役调用方碰不到重复键。
判：攻击面 4 成立。什么现象推翻：非测试代码里出现 `count_write_many_calls` 的调用点（grep 命令可重放）。

### 攻击面 5：多线程，几个线程共用一条管道，整问整答在一把锁里

推导：克隆的几份共用同一个后端：「backend: std::sync::Arc<Backend>,」（`crates/singlefs-device-library/src/lib.rs:48`），后端里只有一个互斥量包着管道两头（`RelayEnds`，`lib.rs:28-31`）。三种请求都是「拿锁、发整条请求、读完整条回应、分支块末放锁」：`read` 在 `lib.rs:161` 拿锁并在 `read_exact` 读完内容之后才放，`write` 在 `lib.rs:204`，`write_many` 在「let mut ends = ends.lock().expect("转交判定库的锁没有中毒");」（`crates/singlefs-device-library/src/lib.rs:227`）。所以别的线程读不到这个线程的回应；大批时别的线程等锁，等的是服务把整批写完确认的这段时间，不会插队。服务侧转交本身只有一个线程串行读请求（`scripts/experiment-apparatus/device-code-key.py:582`），即使 Rust 一侧不加锁也不会并行处理，加锁是为了回应对得上请求。判：成立。

代价（推的）：E158 的工人线程共用这一条管道时，一个大批让别的工人的读写都等；这是吞吐，不是正确性。

缺口（量的）：本包里没有任何测试起线程（`grep -nE 'thread|spawn' crates/singlefs-device-library/src/lib.rs` 零命中，退出码 1），变异表里这份文件只有 9 行（`grep -c 'singlefs-device-library/src/lib.rs' crates/mutations.tsv` 得 9，行号 6983、6984 与 7393 至 7399；`git show HEAD:crates/mutations.tsv` 里同一命令得 0，所以这九行对 HEAD 都是新的，背景材料只点名了后七行），没有一行改锁的范围。所以「整问整答在锁里」只由读代码保证，没有测试或变异钉它。可以补的用例（推的，没写没跑）：两个线程各发一条 `write` 与一条 `write_many`，预先准备够 4 个 `written`，等两线程结束后读请求管道，断言请求字节串是两条请求的任一整段拼接而不是交错。
什么现象推翻成立：把 `write_many` 里的守卫在发请求之后、读回应之前放掉（现有测试不会红，这正是缺口）。

## 问题 3（格 3）：第四节八格

| 格 | 判定 | 证据与推导 |
|---|---|---|
| 1 走不到的理由二次查实 | 成立（附两处背景材料不准） | 见下 |
| 2 类型与名字 | 成立 | 见下 |
| 3 不兼容旧代码与旧格式 | 成立（没有） | 见下 |
| 4 不新建判定库 | 成立 | 见下 |
| 5 所有任务多机可用 | 成立 | 见下 |
| 6 测试优先多线程 | 成立（按条文）；另有缺口 | 见下 |
| 7 带 --item 的命令与项 | 推不出（算打中） | 见下 |
| 8 新崩溃流多罩了什么 | 成立（没有这一类） | 见下 |

### 格 1：走不到的理由二次查实

- 条文要追的两处 panic：`count_write_many_calls` 在 Relay 上 panic（`crates/singlefs-device-library/src/lib.rs:263`）与 `replies_left_unread` 在 Memory 上 panic（`crates/singlefs-device-library/src/lib.rs:373`）。追调用点的过程与结论在问题 2 攻击面 4：前者只有三个调用点（本包 `lib.rs:430`，E158 的 `:30030` 与 `:30043`，后两个在 `mod fifth_run_tests` 里，该模块受 29232 行的 `#[cfg(test)]` 管、到 30195 行收），后者只有两个调用点且对象都是 Relay。走不到成立。
- 条文还问「条款没定的错误成员判是不是在任何落盘动作之前返回」：这份改动没有新加错误成员（`write_many` 返回 `()`，失败都 `process::exit`），不适用。
- 背景材料第四节说「新加的 `expect` 两处是锁没中毒」，现查（量的，`git diff HEAD` 对这份文件的新增行）不止两处：非测试代码三处，`crates/singlefs-device-library/src/lib.rs:227`、`:237`、`:267`（用到两句话：`转交判定库的锁没有中毒`、`单测内存库的锁没有中毒`）；测试辅助里另两处，`:367`「单测里没有别的线程握这把锁」与 `:370`「回应管道读得到文件尾」。三处非测试的 `expect` 依赖同一条不变量：持锁时不 panic（攻击面 1 推导 3），测试里两处依赖「单测没有别的线程」与「回应写端已关」，即 relay_with_prepared_replies 里「drop(reply_end_of_the_counterpart);」（`crates/singlefs-device-library/src/lib.rs:294`）。不改变判定。
- 背景材料没提这份改动加了一行 `#![forbid(unsafe_code)]`（`crates/singlefs-device-library/src/lib.rs:17`；`git show HEAD:crates/singlefs-device-library/src/lib.rs | grep -c unsafe` 得 0，量的：改前没有）。文件里没有 `unsafe`（`grep -n unsafe` 只命中这一行，量的）。这是编译期约束，不改运行行为；它是不是这一轮该带的改动，由主 agent 判。
- 背景材料「没有新加 todo!、unreachable!、assert!」：现查成立（新增行里这三种零命中；新测试里用的是 `assert_eq!`，在 `mod tests` 里）。

### 格 2：类型与名字

条文：同一个量不同语义不共用裸整数、不缩写、不写 `_ =>`、`expect` 写清依赖哪条不变量。
- 裸整数：新字段 `write_many_calls: usize`（`crates/singlefs-device-library/src/lib.rs:37`）只有「发过几次」一种语义；`entries.len()` 与 `text.len()` 只是就地格式化进请求，不存进共用变量。成立。
- 键与内容共用裸 `String`：`write_many(&self, entries: &[(String, String)])`（`crates/singlefs-device-library/src/lib.rs:221`）的两个量都是字符串，条文说的是裸整数，不是它；调换顺序也编得过，这一点与改前就有的 `write(&self, relative: &str, text: &str)`（`crates/singlefs-device-library/src/lib.rs:201`）同形，没有加新的风险面。按条文成立；如果主 agent 要把条文读宽到「同一裸类型的两种语义」，那么 `write` 与 `write_many` 一起不合，不是这份改动单独引入的，我把它记作条文射程问题，不记作这一格的打中。
- 缩写：新名字 `MemoryLibrary`、`entries`、`write_many_calls`、`send_write_request_and_wait_for_written`、`write_many`、`count_write_many_calls`、`replies_left_unread`、`asked`、`library`，没有缩写（读名字所见）。
- `_ =>`：`grep -nE '_ =>' crates/singlefs-device-library/src/lib.rs` 零命中（量的，退出码 1）；`Backend::Memory(_) =>` 是点名变体加忽略载荷，不是兜底分支。
- `expect`：上一格列的五处，话里都写了依赖什么（锁没中毒、没有别的线程握锁、回应写端已关）。成立。
- 名字里的位置词：测试断言话里有「头一行报条数」（`crates/singlefs-device-library/src/lib.rs:393`）与「不止头一条」（`crates/singlefs-device-library/src/lib.rs:426`），共用约束的"不用序数或位置代称"归命名检查四格判，那四格由主 agent 跑，我不判这条（推不出；这一条不归我判，请主 agent 跑命名检查时留意）。

### 格 3：不兼容旧代码与旧格式

`grep -nE 'legacy|old|migrat|兼容'` 对改动全文 `research/prompts/_device-library-write-many-code-r1-diff.md` 零命中，对 `crates/singlefs-device-library/src/lib.rs` 全文也零命中（量的，退出码都是 1）。`read`、`write` 的请求与回应没改（逐语句对照见攻击面 3），`write-many` 是新加的一种请求，没有认旧版本的分支。成立（没有）。

### 格 4：不新建判定库

内存那一种在改前就有，是 `Mutex<BTreeMap<String, String>>`（`git diff` 的删除行），这一轮只把它包进 `MemoryLibrary` 加了一个计数字段（`crates/singlefs-device-library/src/lib.rs:33-38`）。内存那一种的分支里没有任何文件、目录、sqlite 的调用，只有 `lock()` 加 `BTreeMap` 的读写（`crates/singlefs-device-library/src/lib.rs:191-196`、`:210-216`、`:236-242`、`:250-257`）。管道那一种仍然只是把请求写进起装置那一侧开的管道，由那一侧经 prune-server 的客户端写同一份库（`crates/singlefs-device-library/src/lib.rs:1-2` 与服务侧 `scripts/experiment-apparatus/device-code-key.py:679`）。内存那一种不是新的判定存放处：进程内、测完即丢、不落盘、不经任何库路径。成立。
什么现象推翻：内存那一种的分支里出现落盘调用，或真管道之外出现另一条通往库的路。

### 格 5：所有任务多机可用

本包读的环境只有两个描述符号变量（`crates/singlefs-device-library/src/lib.rs:22` 与 `:24`），打开的是 `/proc/self/fd/<号>`（`crates/singlefs-device-library/src/lib.rs:130`）。主机名、地址、家目录：`grep -nE 'HOME|hostname|/home|127\.0|localhost'` 零命中（量的，退出码 1）。`/proc/self/fd` 是 Linux 的，两台都是 Linux 这一点我没核（推的，仓里别处也在用这个路径）。新测试用 `std::io::pipe` 建管道，改前的两条测试已经在用（`crates/singlefs-device-library/src/lib.rs:287`），不是这一轮新带的前提。成立。

### 格 6：测试优先多线程

条文：「新测试与崩溃检测切片并行没有、线程数取登记值；不能并行的写明为什么」，判据「该并行而没并行、线程数写死或覆盖，打中」。三条新测试都是毫秒级、每条一两次请求，没有可切片的工作量；本包里没有写死的线程数（`thread|spawn` 零命中）；`cargo test` 默认就在测试函数之间并行。按条文：成立。
缺口另记：攻击面 5 的「整问整答在锁里」没有任何测试起线程，也没有变异行钉锁的范围（见攻击面 5）。条文这一格不要求它，所以不记打中；它是被判代码里最脆弱的一处，写进下面「判定一览」。

### 格 7：带 --item 的命令与项

背景材料写「本轮改动没有这一类：没有新的崩溃点重放项、门禁项或实验复跑项」。这句不成立：变异表里 `crates/mutations.tsv` 第 7393 至 7399 行是这一轮新加的七行，改前一行都没有（量的：`git show HEAD:crates/mutations.tsv | grep -cF '判定库口子 write-many'` 得 0；此刻文件里同一前缀得 6，第七行「判定库口子 写请求不等 written 就回来」前缀不同）；门禁 crates-mutation-replay 把变异表的每一行当一个可点名的项：「#   bash .claude/gate.d/crates-mutation-replay.sh --item <行号或变异名> [--item …] [项目根]   只判点名的几行（crates-mutation-rows.py --item 交到这里）」（`.claude/gate.d/crates-mutation-replay.sh:22`）。所以这一轮有新的 `--item` 项：背景材料点名的七个（6983、6984 两行对 HEAD 也是新的，见攻击面 5，共九个；调用侧 E158 的 7400 至 7405 行还有六行，改前也没有，量的：`git show HEAD:crates/mutations.tsv | grep -cE '^E158 r5'` 得 0；那六行在 E158 调用侧，不在这一份被判文件的射程里，只记一笔）。
这一格判据是「命令缺项或新项列不出」。命令（范围声明、`<根>`、每个新项）背景材料里没有写，新项列不列得出要跑 `--list-items` 才知道：推不出，按条文算打中。命令放在文首「要主 agent 跑的命令」的第 2、3 条。
我推出的每一行被哪条测试抓到（推的，命令 3 核）：7393 头一行多报一条 → 请求字节串；7394 字符数不是字节数 → 同一条；7395 空批也发 → 空批那条；7396 不等 written → 读走一个 written 的断言（`written\nnot-for-this-batch\n` 里剩下的会变成整串）；7397 管道那一种只发头一条 → 请求字节串；7398 内存只存头一条 → 内存那条；7399 一批记两次 → 同一条。点名的测试名与 `lib.rs` 里的函数名逐个对得上（`crates/singlefs-device-library/src/lib.rs:378`、`:398`、`:414`，量的）。
什么现象推翻「推不出」：命令 2 的列表里有这七行、末了锚点失效一栏不点名它们，命令 3 全抓到。

### 格 8：新崩溃流多罩了什么

这份改动是装置读写库的口子，不涉及盘上格式、挂载、恢复，没有新的崩溃点重放流或崩溃枚举用例。现查（量的）：`grep -nE 'device-library|device_library' .claude/gate.d/stage-inputs.tsv .claude/gate.d/crash-point-replay-items.tsv` 零命中（退出码 1）。成立（没有这一类）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 格 1 逐句（12 句） | 11 句一致，1 句冲突 | 冲突是文件头第 9 行「合成一次提交」对客户端「原子性只在一帧之内」；服务侧文件头第 55 行「只等一次 WAL 同步」同毛病；其余协议三行与服务侧第 53-57 行逐句对得上 |
| 格 2 攻击面 1 原子性与确认 | 确认成立；整批原子仅在一帧内 | 回来时整批已确认，服务整批拒时装置退、不会当写了；超过一帧（8192 条或 16 MiB）时不是一次提交 |
| 格 2 攻击面 2 字节计数 | 成立 | `str::len` 报字节、服务按计数读，换行与空串都对 |
| 格 2 攻击面 3 抽出函数 | 成立 | 与改前七条语句逐字相同，失败那一半没有测试钉，靠对照 |
| 格 2 攻击面 4 内存那一种与空批 | 成立 | 两处 panic 只在测试里有调用点，空批与服务拒 0 条两头对得上 |
| 格 2 攻击面 5 多线程 | 成立；有缺口 | 整问整答在一把锁里；但无测试起线程、无变异行钉锁的范围 |
| 格 3 走不到的理由 | 成立 | 另：背景材料 `expect` 数不准（三处非测试加两处测试），且漏提 `#![forbid(unsafe_code)]` 一行 |
| 格 3 类型与名字 | 成立 | 裸 `String` 键值对按条文（裸整数）不算，与改前 `write` 同形 |
| 格 3 不兼容旧代码 | 成立（没有） | 零命中，退出码 1 |
| 格 3 不新建判定库 | 成立 | 内存那一种是改前就有的 BTreeMap 加一个计数字段，无落盘 |
| 格 3 所有任务多机可用 | 成立 | 只读两个描述符号环境变量，无主机名与家目录 |
| 格 3 测试优先多线程 | 成立（按条文） | 毫秒级小测试无切片可并行；锁范围缺口见攻击面 5 |
| 格 3 带 --item 的命令与项 | 推不出（算打中） | 「没有这一类」不成立：变异表新行就是 crates-mutation-replay 的 `--item` 项；列不列得出待命令 2、3 |
| 格 3 新崩溃流 | 成立（没有） | 阶段输入登记表（stage-inputs.tsv）与崩溃点重放项表（crash-point-replay-items.tsv）对本包零命中 |

## 什么现象会推翻各条结论（汇总）

- 格 1 的冲突：客户端在 `write_many` 之外还有一层把整个调用并成一帧；或 E158 一个单位的判定数与字节数有硬上限小于 8192 与 16 MiB（后者只让现役调用方不受影响，接口仍不是一次提交）。
- 攻击面 1 至 4 的成立：命令 1 里点名的五条用例有一条红；命令 4 的 write-many 各格有 `✗`；非测试代码里出现 `count_write_many_calls` 的调用点。
- 攻击面 5 的成立：把守卫提前放掉的改法在现有测试下不红，说明它只靠读代码保证（这正是缺口，不是推翻）；任何一个持锁时 panic 的点会推翻「锁没中毒」。
- 格 3 第 7 格的「推不出」：命令 2 列出七行（外加 6983、6984）且命令 3 全抓到，则降为成立。

## 没做什么

- 没跑任何测试、门禁、自证、`--list-items`；这些命令都写在文首「要主 agent 跑的命令」一节，每一项那一格写的是「待主 agent 跑」。我自己跑过的只有 `cargo fmt -p singlefs-device-library -- --check`（退出码 0，无输出，量的）、`python3 -m py_compile` 对 `device-code-key.py`（退出码 0）与 `python3 -I` 的 `isdigit` 现跑。
- 没读服务（prune-server 的 Rust 存储层）里写批对重复键的应用次序与空值是否收，所以内存那一种与真管道在重复键上的一致、服务端接不接受空值，都是推的。
- 没量 E158 一个单位的判定数与字节数，所以「超过一帧会不会真发生」推不出。
- 没改任何文件（包括被判的 `lib.rs` 与 `device-code-key.py`）；没替主 agent采纳或出判决；判决要引的数没复跑。
- 没判「头一行」「头一条」这类位置词算不算位置代称（归命名检查，主 agent 跑）；没判 `#![forbid(unsafe_code)]` 这一行是不是这一轮该带的改动。
- 不判别的腿的格：没有别的腿的格要判。没读禁读清单里的两份文件。
