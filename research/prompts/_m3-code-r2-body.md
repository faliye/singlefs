# 里程碑三第一批实现：代码轮第二轮正文（2026-10-01）

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 H6 H7 H8 Y1 Y2 Y3 -->

## 一、题面

第一轮判决 `research/prompts/m3-code-r1-main-verification.md` 的改法 H1–H8 已由实现员改完（报告 `/tmp/claude-1000/m3-r1/code-r1-fixes/report.md`），打进 worktree `/tmp/claude-1000/m3-r1/merge-wt`。

合入后验证：harness 档 98 个测试二进制全绿、clippy 过；fmt 只红在原有 5 份没动过的文件（日志 `/tmp/claude-1000/merge-and-verify/2026-10-01-12862/`）。门禁 code-source-discipline 的 unsafe-allowances 格在 merge-wt 上仍绿。

这一轮只攻 H1–H8 本身与第一轮没攻到的面；辩方复核第一轮判决。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

腿读冻结副本 `/tmp/claude-1000/m3-code-r2/tree/`（被判文件 17 份，名单与 sha256 在 `research/prompts/m3-code-r2-snapshot/`）。这一轮的改动 = `research/prompts/m3-code-r2-diff.patch`：第一轮冻结副本 `/tmp/claude-1000/m3-code-r1/tree/crates` 对这一轮冻结副本的 `crates` 逐文件 diff，1 268 行，11 个文件。

逐条落点（实现员报告「各条要点」）：
- **H1**：entries 文件加两条，会话发布 n = 1、可写挂载 n = 8，用 harness 自带的故障注入盘只让 P 的根槽第 n 次读坏一次，两条都照常做成；变异行替换文 `let reread = first_reading;`。
- **H2**：
  - P 在 txg 7、根在盘 1 一条；
  - `AnotherRoot`（实例 1、txg 3）一条，根环转过一圈的 txg 27；
  - 都只走会话发布这个入口。
- **H3**：可写挂载在逐盘核那里连着两次读坏，拒成 `BehindTheSelectedVersionAndItsRootNotInItsRingSlot`。断言整池写调用为 0、两块盘逐扇区不变、真读坏的恰好 2 次。
- **H4**：成员改名为 `DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`，`crates/` 下 30 处，旧名零命中；`DeviceBehindTheCurrentVersion` 没改（理由：缺什么由 `lacking` 字段的类型说）。kb 里 D18（块里携带什么信息） 已定项 11 还写着旧名，归主 agent 写回。
- **H5**：文件后端过了 `check_request` 之后，长度 0 直接交回 `Ok`。单测比文件后端与 `O_DIRECT` 后端三样：交回什么、本线程交给 write 的字节数、镜像变没变。稀疏盘那一臂没比。
- **H6**：守卫 `ImageFilesRemovedOnDrop`；panic 路径实现员在副本里插一句 `panic!` 证过。
- **H7**：SAFETY 注释改写，描述符、偏移、长度三类词都在。
- **H8**：`EINTR` 时原样重打洞，分路表加一行；多加了只供测试的开关 `ActAsIfTheFirstPunchWereInterruptedBySignal` 与计数 `punch_attempts_interrupted_by_signal`。
- **变异表**：追加 8 行；整行换 2 行，是 H8 改循环后锚点对不上的「退路一个 0 都不写」「报别的错却交回成功」，这两行没重证。

`crates/` 之外：用户 2026-10-01 定可写挂载上的缺根一半「留着，条款补句、补用例」。条款补句没写，等这一轮判完。

## 三、格

### Y1　复核第一轮判决（云端辩方）
逐条复核第一轮判决第二节 14 行：
- 打中的够不够得着：长度 0 今天没有调用方；`EINTR` 在普通文件打洞上会不会真出现。
- H1–H8 有没有把合法行为也改坏：长度 0 交 `Ok` 与「请求要对齐、在范围内」的契约合不合；`EINTR` 无界重试会不会卡死。
- H4 的新名字是不是比旧名说得清。
- 第二节第 10 条「打洞成功那一支不另加开关」站不站得住。

替被改的一方辩护；同样的打法打不打得中所有替代方案。

### Y2　H1–H8 哪一格会错（云端攻方）
不重复第一轮 Z2–Z4 攻过的角度（见第一轮攻方报告）。至少攻：
① H1–H3 新用例自己：造法是不是真崩溃或真故障留得下的盘面；断言是不是钉住了判决要它钉的那一句。写「看起来对、其实错」的实现，看放不放得过：重读次数写死成 2；只在会话发布入口重读；`AnotherRoot` 只比实例不比 txg。
② H4 改名之后，`history.rs`、`model_comparison.rs` 的匹配臂与 checker 档两个 bin 的行为没变吗。
③ H5：长度 0 交 `Ok` 之后，打洞与退路两个计数会不会被误加；`O_DIRECT` 后端对长度 0 的行为与文件后端是不是逐项相同。
④ H6：守卫在 `std::process::exit`、abort、测试超时被杀时删不删得掉；两条用例并行时守卫会不会删错别人的镜像。
⑤ H8：`EINTR` 重试有没有上界；测试开关与计数会不会在非测试路径上生效。
⑥ 整行换掉的那 2 行变异，在副本上证一次红。

只跑 harness 档与 core 的 lib 单测。

### Y3　第一轮没攻到的面（云端攻方，同一条腿、另成一节）
第一轮攻方只跑了会话发布入口的 H2 那一类；正常卸载、抬 F、管理员回退三个入口上的「根在别的盘」「根环转过一圈」有没有不同结局。只跑 harness 档。

## 四、分工

| 腿 | 格 | 立场 | 理由 |
|---|---|---|---|
| 云端辩方（Sonnet） | Y1 | 复核第一轮判决、替被改的一方辩护 | 有前一轮判决可复核 |
| 云端攻方（Opus） | Y2、Y3 | 在副本上真跑 | 两节分开报 |

## 五、跑前写死的判据

- Y2 造出 H1–H8 引进的新错（合法历史被拒、该拒的被放、测试开关在生产路径生效、守卫删错文件、重试卡死），或写出一种错实现放得过新用例 ⇒ 打中，改法并进下一批。
- Y3 造出别的入口上结局不同而用例不钉 ⇒ 补用例，并进下一批。
- Y1 判出第一轮某条打中够不着、或改法改坏了合法行为 ⇒ 那一条交用户定。
- 一条腿报「没打中」的格只抽了一次样，不支撑否定结论。

交付：腿的报告写 `research/prompts/m3-code-r2-<模型>-output.md`（`-sonnet-` / `-opus-`），分段落盘，每次写进文件不超过 150 行，第一段排他新建，回复只写短句；攻方的副本改法与用例放 `research/prompts/m3-code-r2-opus-model/`。草稿放腿自己的目录 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查。方案按 `crates/` 今天的实现来谈（冻结副本 `/tmp/claude-1000/m3-code-r2/tree/crates/`）。
