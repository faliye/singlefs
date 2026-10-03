//! 用法：with_fallocate_errno <errno 名> <程序> [参数…]：装上「fallocate 一律交回这个 errno」的 seccomp 过滤器，再 exec 那个程序。
use std::os::unix::process::CommandExt;

fn main() {
    let mut arguments = std::env::args().skip(1);
    let errno = m3r1_opus_proto::errno_named(&arguments.next().expect("errno 名"));
    let program = arguments.next().expect("程序");
    m3r1_opus_proto::make_fallocate_fail_with(errno);
    let error = std::process::Command::new(program).args(arguments).exec();
    panic!("exec 失败：{error}");
}
