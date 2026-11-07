# 停 systemd 服务之后，回读输出判停没停

**这是 singlefs 的项目本地规则**，不在共享 SOP 里。共享规则在 `.claude/singlefs-ai-sop/rules/`；「改状态的命令回读实际状态」那一条在共享的 `command-safety.md`「别用 echo 假装命令成功」。

## 怎么做

- `systemctl stop 服务名` 之后回读 `systemctl is-active 服务名` 的**输出**，是 `inactive` 或 `failed` 才算停了。
- 再读一遍它的 `cgroup.procs`（目录是 `/sys/fs/cgroup` 接上 `systemctl show -p ControlGroup --value 服务名`），读出来是空的才算进程清干净。
- 不只看 stop 的退出码，也不只看 `is-active` 的退出码：它只在 `active` 时返回 0，`activating`、`deactivating` 也返回非 0。
- 停之前判服务在不在，同样看输出：`active`、`activating`、`reloading`、`deactivating` 都算在，cgroup 里有进程也算在。
