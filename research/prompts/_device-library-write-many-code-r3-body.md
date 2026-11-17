# device-library-write-many-code-r3：判定库口子成批写，代码轮第三轮正文（2026-11-10）

## 一、题面

第三轮，也是这一题的最后一轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：第三轮之后不再开）。前两轮判决：`research/prompts/device-library-write-many-code-r1-main-verification.md`、`research/prompts/device-library-write-many-code-r2-main-verification.md`。这一轮只攻前两轮站住的形态；这一轮新冒出来的零轮形态不再开轮，写进判决的交用户表、标「零轮」。第三轮分工：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核。不许重复前两轮报告已攻过的角度（`research/prompts/device-library-write-many-code-r1-sonnet-output.md`、`-r1-opus-output.md`、`-r2-sonnet-output.md`、`-r2-opus-output.md`）。

改动全文：`research/prompts/_device-library-write-many-code-r3-diff.md`（`lib.rs` 从 r2 开工快照那一版到此刻）。开工快照：`research/prompts/device-library-write-many-code-r3-snapshot/crates-sha256.txt`。

## 二、实现今天的样子（主 agent 的观测，2026-11-10）

- 被判的文件：`crates/singlefs-device-library/src/lib.rs`（快照 sha256 `773f90a7…`）。r2 修补实现员报告：`~/.local/share/singlefs/work/device-library-write-many-r2-fix/result.md`。
- r2 之后的修补（主 agent 读过 diff）：`read` 在拿锁、拼请求之前照 `write` 核相对键，转交与内存两个后端都核；`found` 报的长度改成 `Vec::new()` 加 `try_reserve_exact`，预留不成退 3；放锁另三处（`read` 读 `found` 内容、`write`、`write_many`）各一条探锁测试，四条共用一个探锁函数；文件头改成「客户端可能切成几帧各自提交」，读回非 UTF-8 的值交 `None` 照实写。变异表本包两行整行替换（`found 之后少读一个字节`、`write 不核相对键` 的锚点随改动换了），另追加 7 行。主 agent 另把测试辅助的类型抽成别名 `AskTheLibrary`（clippy `type_complexity`）。
- 主 agent 的读数（量的）：
  - 本包测试任务 12238 `test result: ok. 21 passed`；clippy 任务 12237 退 0；变异行静态核任务 12232 退 0。
  - 证红任务 12233：`✓ 点名 10 条：现判 4 条、复用 0 条、拒算 6 条；跑了 10 条，抓到 10 条`。
  - 多轮量法任务 12234（每份一棵树、每趟一个编译目录）：`name=variant_summary variant=unmutated named_test=- named_failed=0 any_failed=0 of=5`；四个放锁变异 `read-lock-between-request-and-reply`、`read-lock-between-found-and-contents`、`write-lock-between-request-and-reply`、`write-many-lock-between-request-and-reply` 各 `named_failed=5 any_failed=5 of=5`。
  - 下游：E158 任务 12235 `158 passed`；E174、E178 任务 12236 `30 passed`、`54 passed`。
- 压着的条款与前两轮相同：`.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」；`.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」里「一块判完才落盘，写入是原子的」。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. 「已定」项撑不撑得住：前两轮判「成立」「修到」的每一格，换一个前两轮没用过的输入或交错再核一次（例：键规则在 `read`、`write`、`write_many` 三处是不是同一个函数、同一个次序；`try_reserve_exact` 失败与 `isize::MAX` 过滤两条路的话是不是分得开）。
2. 未定项清单的完整性：这份文件对外的每一个 `pub` 入口与每一种回应（`found`、`absent`、`written`、`failed`、认不出），有没有前两轮都没列进攻击面的。
3. 那一轮报出的数字能不能核：第二节每个读数，从它的任务日志里逐字找得到；多轮量法 `rounds.sh` 的「每趟一个编译目录」是不是真的（第二轮辩方脚本就栽在共用编译目录上）。

## 四、检查单

### 检查单：走不到的理由二次查实

实现员报告「走不到的理由」写无。`count_write_many_calls` 在 `Backend::Relay` 上的 `panic!` 前两轮已判走不到。腿逐处追调用点判。

### 检查单：类型与名字

新加：类型别名 `AskTheLibrary`，探锁共用函数与三条新探锁测试，`read` 的三条键测试，`found` 分配失败那一条测试（名单在 `crates/mutations.tsv` 末尾以「判定库口子」起头的行的测试列）。腿判名字、`expect` 理由、有没有 `_ =>`。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支；腿现跑 `grep -nE 'legacy|old|migrat|兼容' research/prompts/_device-library-write-many-code-r3-diff.md` 核。

### 检查单：不新建判定库

本轮改动没有这一类：只加键核对、可失败分配与测试。

### 检查单：所有任务多机可用

第二节的任务都派在第二台跑过。腿核新测试读不读本机独有状态。

### 检查单：测试优先多线程

四条探锁测试本身是并发测试；多轮量法的读数在第二节。腿判够不够。

### 检查单：带 --item 的命令与项

本包新、改的变异行点名证红任务 12233 十条全抓到；门禁 crates-mutation-replay 的 `--item` 写法与第一轮任务 12058 相同（`--item '<变异名>'` 逐个点名、末尾带仓根）。腿核新项能被 `--list-items` 列出。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新的崩溃点重放流或崩溃枚举用例。

## 五、格

1. 第二轮判决第三节四条，每条的改法是不是修到了它点名的那一格（逐条判「修到」「没修到」「修过头」）。
2. 第三节三条攻击面。
3. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
