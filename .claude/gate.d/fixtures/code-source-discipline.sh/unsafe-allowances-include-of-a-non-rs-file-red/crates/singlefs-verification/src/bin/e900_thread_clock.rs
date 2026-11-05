//! 样本：bin 根用 include! 引一份不是 .rs 的文件，那份文件里的 unsafe 照扫（没登记的 bin 根照 forbid，应当判红）。
include!("e900_thread_clock_body.inc");
