# m2-newq-r1 开工快照的倒推（2026-09-24）

腿跑着的时候主 agent 往主工作区打了八份补丁，快照里 4 个 `crates/` 文件因此对不上。倒推做法：把主工作区的 `crates/` 整份拷出，按打进去的倒序 `git apply -R` 这八份补丁，4 个文件的 sha256 与 `sha256sums.txt` 逐个相同。倒推出来的整棵 `crates/` 放在 `/tmp/claude-1000/m2-newq-r1-reconstructed/crates/`，交核查员核腿引的代码行。

| 次序 | 撤回的补丁 | sha256 |
|---|---|---|
| 1 | `/tmp/claude-1000/impl-m2-checker2/impl-m2-checker2.patch` | c392400becfc173804a32f4d2d48d3883b41cef99e352ba15cb620d30938af43 |
| 2 | `/tmp/claude-1000/impl-m2-line1/impl-m2-line1.patch` | d6a6633b53eb92e0f84ef2aa60cee154a6508b127635cb580cadac1372bc12d3 |
| 3 | `/tmp/claude-1000/impl-m2-release-check/impl-m2-release-check.patch` | c50793fa9c2a2aa716748f061c477fa955d85cb1df3a62d97bfe4fd1d6dc8629 |
| 4 | `/tmp/claude-1000/impl-m2-instpage/impl-m2-instpage.patch` | bcb3cec03f81a2fbf0a7dc1aad14f97defade2be4122258c3828e9922713ee8a |
| 5 | `/tmp/claude-1000/impl-m2-admission/impl-m2-admission.patch` | fe9496beb2f86eeb4ec492360a1f82a566cb616ffacf00aebf610df61ebc76dd |
| 6 | `/tmp/claude-1000/impl-m2-blocks/impl-m2-blocks.patch` | fcd2bfec325d98c232c535bca0d86025369cac0cafda423b3a0f1d596b03eca5 |
| 7 | `/tmp/claude-1000/impl-m2-repro/impl-m2-repro.patch` | 5e2237882c1bf0da562ddf4b2c99ed6c42f250626b5a90788ee10273ea13f1c2 |
| 8 | `/tmp/claude-1000/impl-m2-qemu/impl-m2-qemu.patch` | 159bbdbcadeafcf42915ebf59978ba700759e5455c0ff96e0d03983b642b5264 |

倒推之后 `sha256sum -c`（只取这 4 行）：

```
crates/singlefs-core/src/transaction.rs: OK
crates/singlefs-core/src/recovery.rs: OK
crates/singlefs-core/src/allocator.rs: OK
crates/singlefs-core/src/mount.rs: OK
```

另两份对不上的是 `.claude/kb/checks-owed.md` 与 `.claude/kb/milestone/02-second-txn.md`：主 agent 同日按用户定案写回 C495、C483 与收口表几行改的，不倒推；腿对它们按行号引的地方由核查员逐个核。
