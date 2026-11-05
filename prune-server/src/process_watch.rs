//! 读 `/proc/<进程号>` 取服务进程的内存峰值（`VmHWM`）与用掉的 CPU 秒数（用户态加内核态）。只在本机、只读自己起的服务进程。

/// 内存峰值，单位 KiB；读不到回 `None`。
pub fn peak_resident_memory_kibibytes(process_identifier: u32) -> Option<u64> {
    let status = std::fs::read_to_string(format!("/proc/{process_identifier}/status")).ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

/// 用户态加内核态的 CPU 时间，单位是时钟滴答；读不到回 `None`。
pub fn processor_ticks(process_identifier: u32) -> Option<u64> {
    let status_text = std::fs::read_to_string(format!("/proc/{process_identifier}/stat")).ok()?;
    // 第二个字段是带括号的命令名，里面可能有空格；从最后一个右括号之后数字段。
    let after_command = &status_text[status_text.rfind(')')? + 1..];
    let fields: Vec<&str> = after_command.split_whitespace().collect();
    // 括号之后第一个字段是状态（总第 3 个），utime 与 stime 是总第 14、15 个。
    let user_ticks: u64 = fields.get(11)?.parse().ok()?;
    let system_ticks: u64 = fields.get(12)?.parse().ok()?;
    Some(user_ticks + system_ticks)
}

/// 每秒的时钟滴答数。
pub fn ticks_per_second() -> u64 {
    // SAFETY: sysconf 只读一个配置值，没有指针参数。
    let configured_ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    u64::try_from(configured_ticks)
        .ok()
        .filter(|positive_ticks| *positive_ticks > 0)
        .expect("sysconf(_SC_CLK_TCK) 在 Linux 上回正数；回别的说明这台的 libc 不对")
}
