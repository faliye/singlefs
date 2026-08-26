# 工具与环境事实

**本文件放「本工程用到的工具现在是什么状态」**，含实测口径。
规则不进本文件，进 [CLAUDE.md](../../CLAUDE.md) 或 `.claude/rules/`。

## 本地 LLM（三方论证的第三条腿）

| 项 | 值 | 口径 |
|---|---|---|
| 入口 | `~/code/ai-center` 的 OpenAI 兼容网关，`http://127.0.0.1:8200/v1/chat/completions` | 实测 2026-08-26 |
| 鉴权 | 请求头 `X-Api-Key`，取值来自 `~/code/ai-center/.env.tenants` 的 `AI_CENTER_KEY_VSCODE_CHAT` | 实测 |
| 模型 | `local` → `Qwen3-Next-80B-A3B-Thinking-AWQ-4bit`，vLLM 承载，`max_model_len` 262144 | 网关 `/v1/models` 实测返回 |
| 调用脚本 | `bash research/scripts/ask-local.sh <提示文件>` | —— |
| 单轮耗时 | 一句话问题 41 秒；一个含 54 行背景 + 两张清单的论证任务约 60 秒 | 实测 2026-08-26，两次采样，**不满足 N≥5，只作量级参考** |
| 预算 | **不许传 `max_tokens` 或 `thinking_token_budget`**——网关按两本账补值并按形状学习，传偏小的值等于把自己按死在那个值上 | 依据 `~/code/ai-center/.claude/kb/token-budget.md` |

### 已知毛病：4-bit 量化会串字与复读

实测输出里成片出现字词损坏：「空间空间」「效率效率」「需要需要」「复杂复杂复杂」
「无法无法」「未刷刷数据」「增加增加时间」。
另一类毛病是**模式匹配代替推理**：对一张清单里每一条都给出同一个答案
（实测中对「这条收益是不是必须在格式层才拿得到」连答四个「是」，其中至少两条并不显然）。

**处置**：`.claude/rules/three-way-inference.md` 规定这类输出**整轮作废重跑**，
不许记成「三方不一致」——否则每轮都会不一致，三方论证退化成摆设。

## Rust 工具链

| 项 | 值 | 口径 |
|---|---|---|
| 版本 | rustc 1.98.0 / cargo 1.98.0 | 实测 2026-08-26 |
| 安装方式 | rustup，`--profile minimal --default-toolchain stable`，装在宿主 `~/.cargo` | —— |
| 额外目标 | `x86_64-unknown-linux-musl`，产出静态二进制在虚机的 busybox initramfs 里跑 | **已验证**：`e7-index-bench` 编出 static-pie，虚机里跑通 |
| 组件 | `rustfmt` + `clippy`（门禁两个阶段都要用，`--profile minimal` 不含它们） | 已装 |
| **PATH** | rustup 的 shim 已软链到 `~/.local/bin/`（cargo / rustc / rustup / rustdoc / rustfmt / cargo-fmt / cargo-clippy） | 见下方说明 |

### 坑：装了 rust 而门禁看不见

Ubuntu 的 `~/.bashrc` 对非交互 shell **在第 6 行就 `return`**，
rustup 追加在文件末尾的 PATH 那句因此从不执行；
而 `bash -c`（门禁与各脚本的调用形态）**根本不读 `.bashrc`**。
结果是登录 shell 里 `cargo` 好用，`bash .claude/scripts/gate.sh` 却报「cargo 缺失」。

**处置**：把 rustup 的 shim 软链进 `~/.local/bin/`（该目录已在 PATH 上）。
不改 dotfile、不要 sudo。改 `~/.bashrc` 顶部**没有用**——`bash -c` 不读它。

门禁在这件事上是对的：它明写「有 Cargo.toml 但 cargo 缺失 —— 无法验证，按失败处理（不降级、不跳过）」，
并给出了下一步。这条不需要新增检查。

## 虚机 benchmark harness（E7 用）

| 项 | 值 | 口径 |
|---|---|---|
| 脚本 | `bash research/scripts/vm-bench.sh <静态二进制>` / `--selftest` | 在 `research/`，**不进仓库**（研究性脚本留本地） |
| 它补了什么 | 共享 harness 不挂盘也不能塞文件；本脚本给虚机挂一块 virtio 盘（稀疏文件落 `TMPDIR`）并把二进制塞进 initramfs | —— |
| 自检 | **通过**：成功用例回 0、失败用例回 7、盘用例在虚机里看到 `/dev/vda` 4294967296 字节 | 实测 2026-08-26 |
| 完整性闸 | 被测程序收尾行报 `name=done emitted=N`，宿主抓到的条数必须等于 N，否则整轮作废 | **已证明会红**：喂一个声称 4 条只发 3 条的假二进制，harness 判红 |
| 默认规格 | 2048M 内存 / 4 vCPU / 4096M 盘 / 900s 超时，均可用环境变量覆盖 | 读源码 |

### 坑：结果行会被 BIOS 的控制台转义序列顶掉行首

实测控制台首行形如 `Booting from ROM..^[c^[[?7l^[[2JE7RESULT name=device_size ...`，
用 `grep '^E7RESULT'` 锚定行首会**静默漏掉第一条结果**。
处置：不锚定行首，并用上面那道「条数完整性闸」把漏行变成会红的事实。

### 第一组实测数字（虚机内、O_DIRECT、2 GiB virtio 盘）

**这是管道验证，不是 E7 的实验数据**——被测对象是裸设备，不是任何索引结构。

| 项 | 值 |
|---|---|
| 顺序写 1 MiB × 256 | 3509 MiB/s |
| 随机写 4 KiB × 4096 | 16720 IOPS / 65.3 MiB/s |
| 随机读 4 KiB × 4096 | 9881 IOPS / 38.6 MiB/s |

口径：`e7-index-bench` v0.1.0，musl 静态，固定种子 xorshift64\*，宿主 32 核 / 60 GiB 内存，
QEMU 8.2.2 + KVM，`cache=none,aio=native`，背后是宿主文件系统上的稀疏文件。
**单次采样，不满足 N≥5，只作管道通畅的证据，不作性能结论。**

## QEMU harness

| 项 | 值 | 口径 |
|---|---|---|
| 自检 | `bash .claude/scripts/qemu.sh --selftest` 通过：成功用例回 0、失败用例回 7，harness 认得出失败 | 实测 2026-08-26 |
| 内核 | `/boot/vmlinuz-6.17.0-lockdep`（可读） | 实测 |
| 虚机规格 | 512 MB 内存、2 vCPU、`-no-reboot`、超时 180 秒 | 读 `singlefs-ai-sop/scripts/qemu/run.sh` |
| **缺口** | **不挂任何块设备，也没有把额外文件塞进 initramfs 的钩子**。仅有的环境变量是 `SINGLEFS_KERNEL` 与 `TMPDIR` | 读源码确认 |

E7 要在虚机里跑索引 benchmark，因此需要在这个 harness 之上补「挂一块盘 + 塞一个静态二进制」，
见 [experiments.md](experiments.md) E7。

---

## 历史版本

### 2026-08-26（其二）
- 补 vm-bench.sh 一节：它补上了共享 harness 的挂盘/塞文件缺口，自检通过，
  完整性闸已证明会红。记下两个坑：非交互 shell 看不见 cargo、控制台转义序列顶掉结果行首。
- Rust 一行从「已安装，未验证产物」改为「已验证」，并补 rustfmt/clippy 与 PATH 处置。

### 2026-08-26
- 建档。记本地 LLM 入口与已知毛病、Rust 工具链、QEMU harness 的自检结果与缺口。
