//! E184（常规层与大负载层的代价计数）写流转储 bin（跑前登记 `research/prompts/e184-preregistration.md` 5.3 第一份 bin）。
//!
// admission: always 每跑一次都在今天这份 crates 上重录随机历史的写流、重核基镜像往返，转储里的挂钟每次都是新观测；已有写完的转储由装置自己整份跳过
// run-condition: command python3
//!
//! 只录写流、写转储：只引 `singlefs-verification` 的录制器与内存池、`singlefs-core`，不引崩溃那一侧的模块，好让它在第二段的旧提交副本里也编得过。
//! 转储是二进制文件，装一条流从 mkfs 做完那一刻起的全部东西：基镜像 = 空盘叠上 mkfs 的全部写（连同清零写）、mkfs 之后的写表与段、
//! 版本表、历史怎么收的尾。块键不在这里算（算块键要引崩溃那一侧的模块），由 `e184_regular_layer_cost_counts` 的 `keys` 读转储去算。
//! 转储的写法与读法住 `e184_regular_layer_cost_counts_dump/mod.rs`，这个 bin 与主装置各经 `#[path]` 把它当模块引进去，只有这一份；这个文件只放命令行入口。
//!
//!   e184_regular_layer_cost_counts_write_dump dump-random-histories --steps <步数> --weights <BROAD|BROAD_WITH_COMPACTION_WHILE_MOUNTED>
//!       --width <FourGibibytes|UnitAreaOf240Slots> --seeds <条数> --out <目录> [--check-determinism]
//!
//! 种子 = 种子基 + 下标（下标从 0 起，共 `--seeds` 条）。每条历史一份转储，文件名 `<流名>.dump`，先写临时名再改名：重起同一条命令时已有的文件
//! （末尾有 `name=dump_done` 收尾）整条跳过。`--check-determinism` 另把种子下标 0 与 1 各再录一遍，两遍的写表摘要不同就退 4（登记 X2）。
//! 线程数取环境变量 `E184_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值。
//! 每条历史的 panic 原样记进 `name=dump_failed` 行（登记停机 T7），不换种子补数。
//! 每份新录的转储另核一遍（登记 5.2「转储往返」）：由转储里的写表重建的基镜像，与直接把录制流 mkfs 那几步叠到空盘上的基镜像逐扇区相同，
//! 结果打在 `name=dump` 行的 `base_matches_the_stream=` 上，收尾行报不同的条数 `base_mismatches=`。
//! 退出码：0 正常；2 命令行不对；3 有历史没录成（panic 或 mkfs 之后没有操作）；4 阳性对照没过（确定性核或基镜像往返核，登记 X1、X2）；
//! 78 准入与运行条件没满足（`preflight` 拒绝，带 `--force` 才照跑；强制跑的那一次，标准输出第一行多一行 `name=preflight forced=…`，`emitted=` 不数它）。

#[path = "e184_regular_layer_cost_counts_dump/mod.rs"]
#[allow(
    dead_code,
    reason = "转储模块是两份 bin 共用的：主装置用的读回、录一条历史与转储往返对照这个 bin 不调"
)]
mod write_dump;

use write_dump::{arguments_without_the_force_flag, preflight, run_dump_command, EXIT_CODE_USAGE};

/// 这个 bin 在 `src/bin/` 下的文件名（`preflight` 判它头上的准入与运行条件）。
const OWN_SOURCE_FILE_NAME: &str = "e184_regular_layer_cost_counts_write_dump.rs";

fn main() {
    let forced_summary = preflight(OWN_SOURCE_FILE_NAME);
    if let Some(summary) = forced_summary {
        println!("E7RESULT name=preflight forced={summary}");
    }
    let arguments = arguments_without_the_force_flag(std::env::args().skip(1).collect());
    match arguments.split_first() {
        Some((subcommand, rest)) if subcommand == "dump-random-histories" => {
            std::process::exit(run_dump_command(rest));
        }
        Some(_) | None => {
            eprintln!(
                "用法：e184_regular_layer_cost_counts_write_dump dump-random-histories --steps <步数> --weights <BROAD|BROAD_WITH_COMPACTION_WHILE_MOUNTED> --width <盘宽> --seeds <条数> --out <目录> [--check-determinism]"
            );
            std::process::exit(EXIT_CODE_USAGE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_own_source_file_name_points_at_a_file_in_the_source_directory_of_the_binaries() {
        // 路径写成一串字面量，变异判法的单元键才认得出读的是哪份文件；文件名常量另由下一条断言钉住。
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/bin/e184_regular_layer_cost_counts_write_dump.rs");
        assert!(
            path.ends_with(OWN_SOURCE_FILE_NAME),
            "交给 preflight 的文件名 {OWN_SOURCE_FILE_NAME} 不是 src/bin 下的这一份 {}",
            path.display()
        );
        let text = std::fs::read_to_string(&path).expect("preflight 要判的源文件读得到");
        assert!(
            text.contains("// admission:") && text.contains("// run-condition:"),
            "{} 的文件头要有准入与运行条件",
            path.display()
        );
    }
}
