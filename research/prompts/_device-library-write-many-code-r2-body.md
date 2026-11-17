# device-library-write-many-code-r2：判定库口子成批写，代码轮第二轮正文（2026-11-10）

## 一、题面

第一轮判决 `research/prompts/device-library-write-many-code-r1-main-verification.md` 打中五处（文件头「一次提交」过度外推；坏键之后转交把剩下的字节当新请求读、大批两头互等；内存那一种不核键；`expect` 没写凭什么不中毒；`reply != "written"` 没有测试喂过 `failed`），第三节交了五条改法。这一轮攻修补本身，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮分工：第一轮的修补本身、产生结论的方法、从未被看过的地方。不许重复第一轮已攻过的角度（第一轮报告：`research/prompts/device-library-write-many-code-r1-sonnet-output.md`、`research/prompts/device-library-write-many-code-r1-opus-output.md`）。

改动全文：`research/prompts/_device-library-write-many-code-r2-diff.md`，第一节是 `lib.rs` 从第一轮开工快照那一版到此刻，第二节是转交脚本对 HEAD（上下文）。开工快照：`research/prompts/device-library-write-many-code-r2-snapshot/crates-sha256.txt`。

## 二、实现今天的样子（主 agent 的观测，2026-11-10）

- 被判的文件：`crates/singlefs-device-library/src/lib.rs`（快照 sha256 `d9e7b711…`）。实现员报告：`~/.local/share/singlefs/work/device-library-write-many-r1-fix/result.md`。
- 修补做了什么（照实现员报告，主 agent 读过 diff）：
  - `write`、`write_many` 在拿锁、按后端分支之前先核相对键（`give_up_unless_the_relay_accepts_the_key`，`lib.rs` 第 98 行起），转交与内存两种都核，一批里有一条不合整批不发；退出码常量 `RELAY_BROKEN_EXIT_CODE` 改名 `OTHER_ERROR_EXIT_CODE`（第 29 行）。`read` 不核键。
  - 文件头写明一帧（8192 条或 16 MiB）以内一次提交、超过切几帧各自提交、整批 `written` 在全部帧确认之后才回。
  - 两把锁的 `expect` 各用一条常量写理由。
  - 为让「持锁期间不会 panic」成真，另改两处：`found` 报的字节数超过 `isize::MAX` 当认不出退 3；`give_up` 写标准错误写不进也照退（原来 `eprintln!` 会 panic）。
  - 收进攻方的两条 `*_answered_failed_*` 测试（再起一次本测试二进制当子进程），新多线程测试 `a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply`。实现员自己写明：这条多线程测试在「锁在请求与回应之间放过」变异下不是严格必红（std `Mutex` 放锁不交给等的一方），抓到的机会是推的、没量过。
- 转交一侧（上下文）：`scripts/experiment-apparatus/device-code-key.py` 认不出一条请求后抛 `RefusedRequest`，`serve` 回完 `failed` 就停读这根管道；文件头同样改成一帧一次提交。
- 主 agent 的读数（量的）：
  - 本包测试任务 12078 退 0；clippy 任务 12079 退 0；变异行静态核任务 12080 退 0。
  - 证红任务 12083：`✓ 点名 9 条：现判 2 条、复用 0 条、拒算 7 条；跑了 9 条，抓到 9 条`（「读的锁在请求与回应之间放过」那一行这一次抓到，只跑了一次）。
  - 下游：E158 装置单测任务 12081 `158 passed`；E174、E178 任务 12085 `30 passed`、`54 passed`。
  - 转交自证任务 12073 `device-code-key 自证：36 格，红 0 格`；弄坏开关 `relay-reads-on-after-refusal` 任务 12074 只红新加三格。
- 压着的条款与第一轮相同：`.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」；`.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」里「一块判完才落盘，写入是原子的」。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. 修补本身：核键放在拿锁之前，键规则与转交 `RELATIVE_KEY_PATTERN` 是两份实现，两边会不会对同一个键判得不一样（空串、`..`、开头的 `/`、非 ASCII 字母数字）；`read` 不核键留下了什么。
2. 修补本身：`found` 长度过大、`give_up` 标准错误写不进这两处新改法，是不是引进了别的 panic 或吞掉了本该报的错。
3. 产生结论的方法：起子进程的七条测试（拒算）判的是不是子进程真走到的路；多线程测试在锁变异下抓到的机会，「一次抓到」能不能当证据（`.claude/rules/mutation-sampling.md` 第三类「取样点不敏感」）。
4. 从未被看过的地方：`give_up` 一族在测试进程里退整个进程，与 libtest 的多线程并行跑测试怎么相处；`read` 的回应解析（`found <字节数>` 之后的内容读法）在修补后有没有变。

## 四、检查单

### 检查单：走不到的理由二次查实

实现员报告「走不到的理由」写无：产品代码没有新加 `todo!`、`unreachable!`、`assert!` 与条款没定的错误成员。`count_write_many_calls` 在 `Backend::Relay` 上的 `panic!` 第一轮已判走不到（只在本包与 E158 的测试模块里调）。腿逐处追调用点判。

### 检查单：类型与名字

新加：`give_up_unless_the_relay_accepts_the_key`，常量 `OTHER_ERROR_EXIT_CODE`（由 `RELAY_BROKEN_EXIT_CODE` 改名），两把锁 `expect` 理由的常量，新测试九条（名单在 `crates/mutations.tsv` 末尾九行的测试列）。腿判名字、`expect` 理由写的不变量对不对、有没有 `_ =>`。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支；腿现跑 `grep -nE 'legacy|old|migrat|兼容' research/prompts/_device-library-write-many-code-r2-diff.md` 核。

### 检查单：不新建判定库

本轮改动没有这一类：只加键核对、错误处理与测试；内存那一种照旧只在单测、测完即丢。

### 检查单：所有任务多机可用

起子进程的测试用 `std::env::current_exe()` 再起本测试二进制，不读主机名与地址。腿核子进程读不读本机独有状态。

### 检查单：测试优先多线程

新多线程测试本身就是并发测试；其余新测试毫秒级、不切片。腿判那条多线程测试的抓法够不够（攻击面 3）。

### 检查单：带 --item 的命令与项

`crates/mutations.tsv` 末尾新加九行（名字以「判定库口子」起头），点名证红任务 12083 九条全抓到；门禁 crates-mutation-replay 的 `--item` 命令写法与第一轮任务 12058 相同（`--item '<变异名>'` 逐个点名，末尾带仓根）。腿核新项能被 `--list-items` 列出（第一轮任务 12056 的列法）。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新的崩溃点重放流或崩溃枚举用例。

## 五、格

1. 第一轮判决第三节五条，每条的改法是不是修到了它点名的那一格（逐条判「修到」「没修到」「修过头」）。
2. 第三节四条攻击面。
3. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
