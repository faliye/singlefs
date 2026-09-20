# 设备侧比对把 FUA 与 FLUSH 归成了同一个事件（2026-09-20，主 agent 现查）

读的是工作区那一版 `crates/singlefs-harness/src/device_log.rs`。

## 一、四处原文

`DeviceEvent` 没有 FUA 这一形（第 24-36 行）：

```rust
pub enum DeviceEvent {
    Write { offset: DeviceOffsetInBytes, length: u64, content_hash: u64 },
    Flush,
    Discard { offset: DeviceOffsetInBytes, length: u64 },
    Mark,
}
```

解析设备侧日志时，**写上带 FUA 标志**就在它后面补一个 `Flush`（第 132-134 行）：

```rust
if flags & LOG_FUA_FLAG != 0 {
    events.push(DeviceEvent::Flush);
}
```

**独立的 FLUSH 条目**也是一个 `Flush`（第 136-138 行）：

```rust
if flags & LOG_FLUSH_FLAG != 0 {
    events.push(DeviceEvent::Flush);
}
```

期望那一侧，**每个 FUA 写后面无条件补一个 `Flush`**（第 167-169 行），文档注释第 147 行写着「FUA 写之后跟一个 FLUSH」：

```rust
if operation.kind == RecordedOperationKind::WriteForceUnitAccess {
    events.push(DeviceEvent::Flush);
}
```

## 二、推论

两种设备型号跑出来的事件流**归一化之后一模一样**：

| 设备 | 盘上实际收到 | 解析成 |
|---|---|---|
| virtio-blk（`fua=0`，块层模拟） | 一次普通写，随后一个独立的 FLUSH 条目 | `Write`, `Flush` |
| NVMe（`fua=1`，原生透传） | 一次带 FUA 标志的写，**没有**独立 FLUSH | `Write`, `Flush` |

所以：

1. 门禁 55 号换成 `-device nvme` **不会红**——先前担心的那件事不成立，这里更正。
2. 但设备侧这条比对路**分不出这两种盘**。而两者的持久承诺不同：FUA 只保证**这一次写的数据**在介质上，FLUSH 保证**此前已完成的那些写**在介质上（`Documentation/block/writeback_cache_control.rst`，Linux 6.17）。

## 三、这是一个名字指着两个量

`.claude/singlefs-ai-sop/rules/show-me-test.md` 里「跨装置的检查要钉住『这个名字指的是哪个量』」说的就是这一形：`DeviceEvent::Flush` 今天同时指「盘被要求把整个缓存刷出去」与「这一次写自己到了介质」。只比值不比量的检查，在一个名字被当两个量用的时候一声不吭。

对崩溃点重放的枚举域来说，分得出这两个量才要紧：前者让**前面那些写**也持久，后者不让。D13（验证路线） 已定项 4 争的正是这一格。

## 四、射程

这一份只说 `device_log.rs` 今天这么写。**它没说这么写是错的**——对「程序发出去的写与屏障有没有原样到达盘上、次序变没变」这个问题，归一化是对的，门禁 55 号问的也正是这个。归一化抹掉的是另一个问题的答案：「这块盘上，那个危险的崩溃状态造不造得出来」。

要哪一个，得先说清这条比对路要回答哪个问题。这一格交三方判（这一轮正文的 Q5）。
