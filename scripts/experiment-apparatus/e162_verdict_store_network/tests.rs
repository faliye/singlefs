use super::*;

/// 单测里起子进程角色：测试二进制带着这个环境变量、只跑 `child_process_role_entry` 一个测试。
pub(super) const CHILD_ROLE_ENVIRONMENT: &str = "E162_S3_TEST_CHILD_ROLE";

#[test]
fn child_process_role_entry() {
    if let Ok(target) = std::env::var(RELAY_TARGET_ENVIRONMENT) {
        use std::os::fd::FromRawFd;
        let path = target.strip_prefix("unix:").expect("中继目标写成 unix:<路径>").to_string();
        // SAFETY: 单测的对端命令用 `3>&1 1>/dev/null` 把数据通道接在 3 号描述符上（libtest 往 1 号打的字不进数据通道），
        // 这个进程里没有别的东西持有或关掉 3 号描述符。
        let data_channel = unsafe { File::from_raw_fd(3) };
        std::process::exit(sender_role::relay_between(&path, Box::new(data_channel)));
    }
    let Ok(joined) = std::env::var(CHILD_ROLE_ENVIRONMENT) else { return };
    let arguments: Vec<String> = joined.split('\u{1f}').map(str::to_string).collect();
    let remaining = &arguments[1..];
    let exit_code = match arguments[0].as_str() {
        "library" => run_library_role(remaining),
        "forward" => run_forward_role(remaining),
        "verify" => run_verify_role(remaining),
        "send" => sender_role::run_send_role(remaining),
        "relay" => sender_role::run_relay_role(remaining),
        other => panic!("认不出的子进程角色 {other}"),
    };
    std::process::exit(exit_code);
}
