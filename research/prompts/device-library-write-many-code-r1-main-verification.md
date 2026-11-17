# device-library-write-many-code-r1 判决：判定库口子成批写的代码轮第一轮（2026-11-10）

## 一、核查

- 腿：正推 `research/prompts/device-library-write-many-code-r1-sonnet-output.md`；攻方 `research/prompts/device-library-write-many-code-r1-opus-output.md`。各抽一次，「没打中」不支撑结论。
- 开工快照：`snapshot-tree.py device-library-write-many-code-r1`（任务 12061）`✓ 快照 device-library-write-many-code-r1：清单点名 2 份，倒推出 2 份，sha256 对上 2 份`。
- `python3 scripts/agents/cite-check.py` 两份腿报告，`--root` 给倒推出的树：`✓ 核了 74 处引文、115 处裸行号，对上 34 处，没判 155 处`；没判的引文落在转交脚本 `scripts/experiment-apparatus/device-code-key.py` 与客户端 `scripts/prune-server/prune_server_client.py`，不在快照里。同一条命令 `--root` 给此刻主工作区：`✓ 核了 74 处引文、115 处裸行号，对上 74 处，没判 115 处`。判决引的行（`prune_server_client.py` 第 346–348 行、`device-code-key.py` 第 590–606、655–670、742–750 行、`crates/singlefs-device-library/src/lib.rs` 第 236–242 行）主 agent 现读过，与腿引的一致。
- 复跑（主 agent 的读数，量的）：
  - 正推腿四条：单测任务 12057 `test result: ok. 3 passed`（点名 `write_many`）；转交自证任务 12055 `device-code-key 自证：33 格，红 0 格`；列项任务 12056 列出本包九行、`锚点失效 0 行`；点名九行任务 12058 `计数：抓到 9 条、没红 0 条、无效 0 条、锚点失效 0 行（共 9 条）`。
  - 攻方腿模型拷到 `~/.local/share/singlefs/work/device-library-write-many-code-r1-verify/model`，在副本里跑（任务 12060）：
    - `name=bad_key_first_then_request_shaped_content ... first_reply=failed/3 ... store_writes=items/injected.out`
    - `name=bad_key_first_then_large_multiline_batch device_blocked_in_write_all=True first_reply=- writer_error=BrokenPipeError`
    - `name=frames_small_values entries=8193 frames=2`、`name=frames_two_nine_mebibyte_values entries=2 frames=2`
    - `name=tests_unmutated rc=0`（攻方加的四条测试与原有五条，`9 passed`）
    - `name=tests_mutant mutant=reply-check-dropped rc=101`（`write_answered_failed_exits_with_the_code_in_the_reply`、`write_many_answered_failed_exits_with_the_code_in_the_reply` 两条 FAILED）
    - `name=tests_mutant mutant=lock-released-between-read-request-and-reply rc=0`（多线程那条测试抓不到这个变异）

## 二、逐格判

| 格 | 正推 | 攻方 | 判 | 依据 |
|---|---|---|---|---|
| 文件头协议与服务侧约定 | 一句冲突 | 打中 1 | **打中**：`lib.rs` 第 9 行「一条请求里的写在服务那一头合成一次提交」与 `device-code-key.py` 第 55 行「一条请求只等一次 WAL 同步」过度外推 | 客户端 `prune_server_client.py` 第 346 行「每帧一个写批，原子性只在一帧之内」，上限第 347–348 行 16 MiB、8192 条；复跑 8193 条切 2 帧。回来时整批已确认这一句不受影响 |
| 攻击面 1 原子性与确认 | 成立（限一帧） | 打中 2、3 | **打中**：一批里有一条键不合，转交回 `failed 3` 之后把这一批剩下的字节当新请求接着读，内容里像请求的那一段会写进库；大批多行时两头互等 | `device-code-key.py` 第 596–603 行 `serve` 循环在 `answer` 交回应答后接着 `readline`；第 663–664 行键不合交 `unrecognized_request` 不读完这一批。复跑 `store_writes=items/injected.out`、`device_blocked_in_write_all=True`。单条 `write` 键不合时同一个病。E158 的键是十六进制，走不到；装置这一侧不核键，别的调用方走得到 |
| 攻击面 2 字节计数 | 成立 | 成立 | 成立 | 变异 7394 抓到（任务 12058） |
| 攻击面 3 抽出的发请求函数 | 成立 | 成立 | 成立 | 单条 `write` 请求字节与回应读法前后相同；变异 6984、7396 抓到 |
| 攻击面 4 内存那一种 | 成立 | 打中 4 | **打中**：内存那一种不核相对键，单测绿、真跑退 3 | `lib.rs` 第 236–242 行直接 `insert`；攻方测试 `in_memory_write_many_takes_a_key_the_relay_refuses` 在未改的实现上 `ok`（就是证据：内存一侧收了转交会拒的键） |
| 攻击面 5 多线程 | 成立、没测试 | 成立、没测试 | 成立，**欠测试**：攻方补的多线程测试抓不到「锁放在读请求与读回应之间」那个变异（`rc=0`） | 任务 12060 |
| 检查单：走不到的理由二次查实 | 成立 | 成立 | 成立 | `count_write_many_calls` 只在本包与 E158 的测试模块里调 |
| 检查单：类型与名字 | 成立 | 不成立 | **打中**：三处新 `expect` 只写「锁没有中毒」，没写凭什么不中毒 | 持锁期间只做管道读写与 `give_up`（退进程）、`BTreeMap::insert`，要把这一条写进 `expect` |
| 检查单：不兼容旧代码与旧格式 | 成立 | 成立 | 成立 | |
| 检查单：不新建判定库 | 成立 | 成立 | 成立 | 内存那一种只在单测、测完即丢 |
| 检查单：所有任务多机可用 | 成立 | 成立 | 成立 | |
| 检查单：测试优先多线程 | 成立 | 成立 | 成立 | 并发归攻击面 5 那一格 |
| 检查单：带 --item 的命令与项 | 推不出 | 不成立 | **正文写错，实测成立**：正文写「本轮改动没有这一类」不对，变异表第 7393–7399 行（与第 6983、6984 行）是新项；实测九项都能列出、点名都抓到（任务 12056、12058） | |
| 检查单：新崩溃流多罩了什么 | 成立 | 成立 | 成立 | |
| 最脆弱的一处 | — | `lib.rs` 第 115 行 `if reply != "written" {` 没有测试喂过 `failed` | **打中**：变异 reply-check-dropped 下原有五条全绿，只有攻方新加的两条红 | 任务 12060 |

背景材料两处不准（正推腿报）：新加 `expect` 非测试代码三处不是两处；改动里另有一行 `#![forbid(unsafe_code)]`。不改变任何格的判定。

## 三、交实现员（下一批，写回之后开 r2）

1. 装置这一侧在发请求之前核相对键（字母、数字与 `. _ - /`，不带空白），`write`、`write_many` 两种、转交与内存两个后端都核；不合的照「别的错」退 3、不发请求。修打中 2、3、4。
2. 转交脚本 `device-code-key.py`：一条请求判成认不出（含成批写里有一条键不合）之后回 `failed` 就停止服务这一根管道，不再把后面的字节当请求读；修打中 2、3 的转交一侧。归工具员（脚本）。
3. `lib.rs` 第 9 行与 `device-code-key.py` 第 55 行改写成实话：整批经客户端写服务，一帧（8192 条或 16 MiB）以内是一次提交，超过切成几帧各自提交；整批 `written` 在全部帧确认之后才回。修打中 1。
4. 三处 `expect` 写明持锁期间不会 panic 的理由。修「类型与名字」。
5. 收进攻方的两条 `*_answered_failed_*` 测试与内存键核对测试，各配变异行（`reply-check-dropped`、内存不核键）；多线程测试换成抓得到「锁放在读请求与读回应之间」的造法，配变异行。修最脆弱的一处与攻击面 5。

## 四、交用户

无。

## 五、延后（去向）

无：上面五条全进下一批，改完开 `device-library-write-many-code-r2`。

## 这一轮被判过的文件

- crates/singlefs-device-library/src/lib.rs：成批写 `write_many`、抽出的 `send_write_request_and_wait_for_written`、内存那一种与计数、文件头协议；判了五条攻击面与检查单八格，打中五处，交第三节。

## 回看决策

不涉及决策：改动是实验装置读写判定库的口子，不碰 `.claude/kb/decisions/` 里任何一条的条款与依据。
