# device-library-write-many-code-r2 判决：判定库口子成批写的代码轮第二轮（2026-11-10）

## 一、核查

- 腿：辩方 `research/prompts/device-library-write-many-code-r2-sonnet-output.md`（复核 `research/prompts/device-library-write-many-code-r1-main-verification.md`）；攻方 `research/prompts/device-library-write-many-code-r2-opus-output.md`。各抽一次，「没打中」不支撑结论。
- 开工快照：`snapshot-tree.py device-library-write-many-code-r2`（任务 12151）`✓ 快照 device-library-write-many-code-r2：清单点名 4 份，倒推出 4 份，sha256 对上 4 份`。
- `python3 scripts/agents/cite-check.py` 两份腿报告：`--root` 给倒推出的树 `✓ 核了 86 处引文、23 处裸行号，对上 38 处，没判 71 处`（没判的落在转交脚本、客户端与 E158、E178 装置，不在快照里）；`--root` 给此刻主工作区 `✓ 核了 86 处引文、23 处裸行号，对上 86 处，没判 23 处`。
- 复跑（主 agent 的读数，量的；两份模型都拷到 `~/.local/share/singlefs/work/device-library-write-many-code-r2-verify/` 下的副本里跑）：
  - 辩方探针任务 12133：`name=p1_read_key_with_newline_then_write_shaped_bytes store_writes=['items/injected.out']`；`name=p2_client_write_many case=one_small_value_that_starts_with_the_manifest_magic entries=1 frames=3`；`name=p2_client_write_many case=one_small_value_for_a_key_with_a_leftover_registered_version entries=1 frames=2 frame_entry_counts=[1, 1] prefix_deletes=1`；`name=p2_client_write_many case=8193_small_values entries=8193 frames=2 frame_entry_counts=[8192, 1]`。
  - 辩方多轮脚本任务 12134：`name=rounds_mutant_summary caught=5 of=5`、`name=rounds_unmutated_summary red=5 of=5`。未变异那五轮红不是测试误报：脚本让两份副本共用一个 `CARGO_TARGET_DIR`，未变异那一份跑的是变异编出的二进制（两边都 `finished in 0.00s`、同一句断言）。主 agent 另交任务 12135，未变异代码在独立副本加新编译目录里与工作区里各 5 轮：`name=alone round=1 rc=0 test result: ok. 1 passed`……`name=workspace round=5 rc=0 test result: ok. 1 passed`，十轮全绿。
  - 攻方模型任务 12136：
    - `name=read_key_with_newline_then_empty_write control=False relay_finished=True replies=absent,written store_writes=items/injected.out relay_written=items/injected.out injected=True`（另两格同样 `injected=True`；对照 `name=control_read_key_with_space ... injected=False`、`name=control_read_valid_key ... injected=False`）
    - `name=single_characters rust_accepted=66 python_accepted=66 differ=0`、`name=summary differ=0`
    - `name=probe mutant=unmutated mode=sequential rc=0 probe_samples=40 probe_red=0`、`name=probe mutant=unmutated mode=concurrent rc=0 probe_samples=100 probe_red=0`
    - `name=probe mutant=read-lock-released-between-request-and-reply mode=sequential rc=0 probe_samples=100 probe_red=100`、`... mode=concurrent rc=0 probe_samples=200 probe_red=200`
    - `name=replay_like_summary runs=30 named_failed=30`
    - 另三个放锁变异（`read-lock-released-before-found-contents`、`write-lock-released-between-request-and-reply`、`write-many-lock-released-between-request-and-reply`）各 5 轮 `ok=19 failed=1`，红的那一条在每一轮都是攻方草稿测试 `read_answered_found_with_a_length_that_cannot_be_allocated_exits_with_the_other_error_code`，与未变异五轮（`name=tests_unmutated round=1 rc=101 ok=19 failed=1`）红的同一条：三个放锁变异没有一条测试抓得到。

## 二、逐格判

| 格 | 辩方 | 攻方 | 判 | 依据 |
|---|---|---|---|---|
| r1 第二节五处打中复核 | 都站得住；「`expect`」那一条取严读法 | — | 维持 r1 判决。`expect` 那一条判的是严读法：理由要写出凭什么不中毒（全仓 `expect(".*中毒` 38 处里 7 处带理由，含 `crates/singlefs-core/src/allocator.rs:1982`，辩方现数） | |
| r1 第三节第 1–4 条改法 | 修到 | — | 修到 | 任务 12078、12083 |
| r1 第三节第 5 条（多线程测试） | 只修到一半（一次样） | 推不出（打中 C） | **撤掉打中 C，算修到**：锁变异下探针 300/300 红、点名测试 30/30 红，未变异 140/140 不红、十轮点名全绿 | 任务 12135、12136 |
| 攻击面 1：两份键规则 | — | 成立 | 成立 | 任务 12136 `differ=0` |
| 攻击面 1：`read` 不核键 | 推的 | 打中 A | **打中**：`read` 的键带换行，转交把换行后面那一段当一条 `write` 办，库里写进 `items/injected.out` | 任务 12133、12136 三格 `injected=True`；现有调用方的键带不进换行，走不到；与 r1 打中 2 同一把尺子「别的调用方走得到」 |
| 攻击面 1：内存那一种 `read` | — | 打中 B | **打中**：内存那一种对坏键交 `None`，真跑转交回 `failed 3`、装置退 3 | 与 r1 打中 4 同形，落在 `read` |
| 攻击面 2：`found` 长度 | — | 成立，另有一处推不出 | **打中（小）**：长度没过 `isize::MAX` 而分配不出时进程 SIGABRT、不退 3 | 攻方草稿测试在未变异代码上五轮都红（任务 12136）；转交不会回这种长度，走不到 |
| 攻击面 2：`give_up` | — | 成立 | 成立 | |
| 攻击面 3：七条子进程测试 | 成立 | 成立 | 成立 | |
| 攻击面 3、4：放锁的另三处 | 推的（多线程测试不罩 `found` 之后） | 打中 D | **打中**：`read` 读 `found` 内容、`write`、`write_many` 三处放锁，没有测试也没有变异行 | 任务 12136 三个变异各 5 轮都没被抓到 |
| 攻击面 4：`give_up` 与 libtest 并行 | — | 成立 | 成立 | 代价记下：父进程里测试走到 `give_up` 时，变异门禁把那一行判「无效」 |
| 攻击面 4：`read` 回应解析 | 量的：非 UTF-8 交 `None`，与文档句不符，HEAD 里就有 | 走不到 | **打中（文档）**：文档句与行为不一致，改文档或改行为二选一 | |
| 文件头「一帧以内是一次提交」 | 第二轮新形态又过度外推（组提交、魔数、旧版本） | — | **打中**：一个以分段清单魔数起头的小值一帧拆三帧；键留有登记旧版本时多一次前缀删除 | 任务 12133 两格；E158 走不到 |
| 检查单：走不到的理由二次查实 | 成立 | 成立 | 成立 | |
| 检查单：类型与名字 | 成立 | 成立 | 成立 | |
| 检查单：不兼容旧代码与旧格式 | 成立 | 成立 | 成立 | |
| 检查单：不新建判定库 | 成立 | 成立 | 成立 | |
| 检查单：所有任务多机可用 | 成立 | 成立 | 成立 | 多线程测试在第二台量过（任务 12134、12135、12136 都派在第二台） |
| 检查单：测试优先多线程 | 成立 | 推不出 | 成立（同打中 C 撤掉的依据） | |
| 检查单：带 --item 的命令与项 | 推不出 | 成立 | 成立：九行锚点各命中一次（辩方现数），点名证红任务 12083 九条全抓到 | |
| 检查单：新崩溃流多罩了什么 | 成立 | 成立 | 成立 | |

辩方另报一处（量的、不在本轮被判文件里）：`scripts/experiment-apparatus/device-code-key.py` 工作区文件模式丢了执行位。主 agent 已改回与 HEAD 相同的 100755（`git diff HEAD --summary` 对它为空）；暂存区里那一份由暂存它的会话重暂存。

## 三、交实现员（下一批，写回之后开 r3）

1. `read` 发请求之前照 `write` 那一条核相对键，转交与内存两个后端都核；不合的退 3、一个字节都不发。修打中 A、B。配测试（键带换行、带空格、内存坏键）与变异行。
2. `found` 报的长度分配不出时退 3：用可失败的分配（`try_reserve` 一类），不 abort。修攻击面 2 的小打中。配测试与变异行。
3. 放锁另三处（`read` 读 `found` 内容、`write`、`write_many`）各配一条抓得到的测试与变异行，造法照 `a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply`。修打中 D。
4. 文件头与转交脚本文件头里「一帧以内一次提交」改成不外推的说法：整批经客户端 `write_many` 写服务，客户端可能切成几帧各自提交（条数、字节数、值的形态都会让它多切），原子性只在一帧之内；整批 `written` 在全部帧确认之后才回。读回非 UTF-8 的值交 `None` 这一条，把文档改成实话。修两处文档打中。

## 四、交用户

无。

## 五、延后（去向）

无：第三节四条全进下一批，改完开 `device-library-write-many-code-r3`（第三轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第三轮之后不再开）。

## 这一轮被判过的文件

- crates/singlefs-device-library/src/lib.rs：r1 判决之后的修补（键核对、`found` 长度、`give_up`、`expect` 理由、子进程测试与多线程锁测试、文件头协议）；判了 r1 五条改法与四条攻击面、检查单八格，打中五处，交第三节。

## 回看决策

不涉及决策：改动是实验装置读写判定库的口子，不碰 `.claude/kb/decisions/` 里任何一条的条款与依据。
