# 背景材料：增补 3 第 4 件（故障注入）的实现，代码轮第一轮

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 -->

判的是里程碑「第二个事务」增补 3 第 4 件写出来的代码，还没提交。开工快照 `research/prompts/m2-supp3-item4-code-r1-start-snapshot.sha256`。

## 一、这一件要做成什么（里程碑原文）

> 4. 故障注入：一个通用的设备包装，按种子让任意一次写、读、刷盘返回 `BlockDeviceError`，或让一次读返回改坏的字节；实现要返回错误而不是 panic，出错之后重开要恢复到模型允许的版本。各用例里手写的包装并进来。

## 二、实现今天的样子（主 agent 的观测，读的是工作区那一版）

- 新模块 `crates/singlefs-harness/src/fault_injection.rs`（2053 行）：`FaultInjectingBlockDevice<Inner>` 加共用的 `SharedFaultPlan`。注入什么写成一个枚举，成员自带作用在哪一种调用上（`WriteFails` / `WriteIsSwallowed` / `ReadFails` / `ReadReturnsCorruptedBytes{flipped_bit}` / `BarrierFails` / `BarrierIsSwallowed`），「让一次屏障返回改坏的字节」这种组合写不出来。选择子四样：哪块盘、哪个落点（任意 / 偏移小于 / 偏移恰好）、整池还是逐盘数、第 n 次 / 第 n 次起每一次。
- **包装摆在录制器外面**：报错的写、被吞的写、被吞的屏障都不进录制流 ⇒ 录制流恒等于真正落到盘上的那一串，注入之后的镜像从录制流重建。
- 一段历史跑两遍：测量跑（不注入、不跑 checker）记每一步的调用数与模型提交过的版本；按种子**先抽一种注入、再抽一步、再抽那一步里的一次调用**；每个注入点整段重跑，判三样——一个 panic 都没有、注入那一步返回 `Err` 或冷启动读回报错、重开跑 `recovery::recover` 走到的那一版在模型允许的集合里且内容逐字节相同（`model::crash_recovery_disagreement`，与第 3 件崩溃注入同一份实现）。
- 四处手写的设备包装并进通用包装：`tests/instance_acquisition.rs`、`tests/second_transaction_supplement_one_write_accounting.rs`、`tests/second_transaction_step_three_formatted_pool.rs`、`src/bin/first_transaction_on_device.rs`。主 agent 现查：`FaultInjectingDevice` 今天全仓只剩类型别名与注释，旧实现没了。
- 用例 `crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs`（482 行）：快档、一段写死的历史、线程数对拍、C381 判别力用例、大档（`#[ignore]`）。
- 快档一轮（24 段 × 20 步 × 每段 4 次 = 95 个注入点，16.2 秒）：**panic 0 次、新发现 0 条、重开走到模型都不允许的版本 0 次**。注入种类 write_fails 26 / read_fails 28 / read_returns_corrupted_bytes 18 / barrier_fails 23；收尾「返回了错误」48、「被容错跑完」45、「停在别的东西上」1、「没走到」1；重开 95 次里走到模型认下的某一版 77、走到失败那一步正在写的那一版 18（收口表第 40 行那一格，**只记不判**）。注入点 42 格里**命中 20 格、没命中 22 格**（报告逐个列名）。
- `crates/mutations.tsv` 新增第 197–204 行 8 条：197–201 钉注入装置自己注得进去，202 钉被测的性质（`transaction.rs` 落盘阶段的块设备错改成 `.expect(...)` ⇒ 快档 panic 0 → 37），203–204 钉 C381 那一格的判别力。门禁 59 号 199 条全红、`check.sh` 退出码 0。

## 三、要判的问题

**K1（注入点抽得够不够）**：抽样是「先抽一种注入、再抽一步、再抽那一步里的一次调用」，不是按注入点分层。快档 42 格里 22 格没命中。举出一类**结构性抽不到**的注入点（不是规模不够，是这个抽法永远抽不到），能写成可跑的构造更好。

**K2（注入之后历史不往下跑，罩不到什么）**：模型里没有「设备错」这条拒绝理由（`model_comparison` 明写 I/O 错一律 `Unexplained`），执行器在注入那一步就停。⇒「这次失败把池弄坏了、下一次发布才现形」这一整类损伤，随机注入这一路**看不见**，只有写死的那条 C381 用例罩着。逐条核：这一类里还有哪几种形态今天零覆盖；要让 campaign 也罩得住，最小的改动是什么。

**K3（设备说谎那一类该不该进）**：`WriteIsSwallowed` / `BarrierIsSwallowed` 实现了但**没进随机抽样**——里程碑逐字写的是「返回 `BlockDeviceError`，或让一次读返回改坏的字节」，这两种是设备说谎。实现员实测：把 `WriteIsSwallowed` 抽进来时 24 段里 2 段的 checker 判出 I-2.1 / I-4.8 / I-7.4 红（根指着一份从没落盘的单元），而写入口拿到的是 `Ok`、实现发现不了。逐格判：这是不是一个真缺陷、该不该立「说谎的设备」这一类故障模型、不立的话这一格今天谁在罩。

**K4（三条判定够不够）**：今天判三样（不 panic、注入那步返回 Err、重开落在模型允许的集合里）。逐条核：这三样合起来能不能推出「故障注入验到了它该验的东西」；有没有一类失败**三样全过而实际是错的**。

**K5（起点那一段不注入）**：`HistoryPool::start` 那几步（mkfs、取号、暖机、第一个文件）是 `expect`，注入在那里撞的是执行器自己的 panic、不是 core 的缺口。逐格判：起点段今天有多少次设备调用、这些调用上的故障今天由谁罩、不罩的话缺口有多大。

**K6（收口表第 40 行那一格「只记不判」）**：重开走到「失败那一步正在写的那一版」快档 95 次里占 18 次，今天放行只记数。这一格是 C381（三轮判完、改法待用户定）。逐格判：放行这个读法会让哪几类真错误也被放行；C381 的改法一旦定下来，这一格要改成什么、快档会不会立刻红。

## 四、分工

| 腿 | 攻击面 |
|---|---|
| 云端攻方（Opus） | K1、K2：抽法结构性抽不到的注入点、注入之后不往下跑罩不到的那一整类损伤。要举得出具体的注入点或历史形态，能写成可跑的构造更好；副本上量的数写明是副本 |
| 云端正推（Sonnet） | K4、K5：逐条核那三条判定合起来够不够、起点段的缺口有多大。引代码行号与产物读数，不引结论句 |
| 本地攻方（英文，逐格填表） | K3、K6：设备说谎那一类该不该进、收口表第 40 行那一格放行的代价。按表格逐格填，不许只答 yes / no；一份提示不超过 16 格 |

两条攻方腿的攻击面不重叠：Opus 攻**抽样与执行的射程**（抽不到、跑不到的那些），本地攻**判定口径**（哪一类故障不算故障、哪一版算允许）。

判决写 `research/prompts/m2-supp3-item4-code-r1-main-verification.md`，按路径点名这一轮改过的每个 `crates/*/src/*.rs`（门禁 56 号判形式）。
