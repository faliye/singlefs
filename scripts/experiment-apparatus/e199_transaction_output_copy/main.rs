//! E199（发布照抄上一版的代价）第一段：纯算术计数模型。
//!
//! 跑前登记：`research/prompts/e199-preregistration-r4.md`（第四份；它没写到的照第三份 `e199-preregistration-r3.md`，
//! 第三份写「照留」的第二份、第一份各节，按它点名的那几节读）。第四份写死的五样：乙2 与乙1 阳性对照的退化形态、乙1 的照抄字节、
//! 甲丙丁三式常驻峰值（丁加残余式）、10⁶ 次 unlink 累计的次序（第四份第五节）。
//! 臂：甲-B2后（真实基线）、甲-今天（对照）、乙2 持久结构、乙1 每片引用计数（附带，只报数）、丙 按值交进来加撤回日志、丁 只记差量加压实。
//! 每次发布的照抄字节、新装字节、现算重建字节照登记第五节的式子算；判据照第六节；锚点照第七节，写成单测。
//! 独立手写，`crates/` 一行都不引；第三节的文件:行只当照抄点清单。
//!
//! 用法：不带参数，产物就是标准输出。线程数：取登记的重活默认（`.claude/heavy-thread-default`），
//! 只用在「10⁶ 次 unlink 累计」那一段：按发布序号切片并行、按片的次序相加，产物与线程数无关
//! （单测 `cumulative_unlinks_do_not_depend_on_the_thread_count`）。别的取样点每点只做 O(层数) 次算术，单线程。

use std::io::Write;
use std::mem::size_of;

mod arms_and_judged_cells;
mod compaction_and_version_sizes;
mod copied_bytes_per_arm_before_and_after_admission;
mod knobs_trees_and_geometry;
mod persistent_and_reference_counted_arms;
mod registered_constants;
mod report_lines_of_the_checks;
mod resident_peaks_and_the_accumulated_history;
mod sampling_points_and_point_lines;
#[cfg(test)]
mod test_support_for_the_main_runs;

use self::arms_and_judged_cells::*;
use self::compaction_and_version_sizes::*;
use self::copied_bytes_per_arm_before_and_after_admission::*;
use self::knobs_trees_and_geometry::*;
use self::persistent_and_reference_counted_arms::*;
use self::registered_constants::*;
use self::report_lines_of_the_checks::*;
use self::resident_peaks_and_the_accumulated_history::*;
use self::sampling_points_and_point_lines::*;
#[cfg(test)]
use self::test_support_for_the_main_runs::*;

// 写成 `#[path] mod`：单元闭包的键不收条目级 `include!`（同 E178（叶容器与记录死亡的计数））。
#[path = "../heavy_threads_default/threads.rs"]
mod heavy_threads_default;
use heavy_threads_default::default_heavy_threads;

fn main() {
    if std::env::args().len() > 1 {
        eprintln!("用法：e199-transaction-output-copy（不带参数；产物写到标准输出）");
        std::process::exit(2);
    }
    let threads = default_heavy_threads(include_str!("../../../.claude/heavy-thread-default"));
    let lines = counting_model_lines(threads);
    let standard_output = std::io::stdout();
    let mut output = std::io::BufWriter::new(standard_output.lock());
    for line in &lines {
        writeln!(output, "E7RESULT {line}").expect("写得进标准输出");
    }
    writeln!(output, "E7RESULT name=done emitted={}", lines.len() + 1).expect("写得进标准输出");
}

#[cfg(test)]
mod tests;
