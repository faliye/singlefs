// 实验装置读写判定库的口子：装置不开本地库目录、不连服务，读写都经起它的 scripts/experiment-apparatus/device-code-key.py run
// 转给 prune-server 的那一份判定库（.claude/rules/verification.md「判定库只有一份」；转交的一侧写在 device-code-key.py 文件头「判定库的转交」）。
// device-code-key.py 起装置时开两根管道，把装置这一头的描述符号放进环境变量 DEVICE_LIBRARY_REQUEST_FD（装置写请求）与
// DEVICE_LIBRARY_REPLY_FD（装置读回应）；键是相对这个装置那一种的路径（例 items/<项键>.out、codes/<代码键>.txt、index.tsv），族与种类由那一侧定。
// 协议一问一答，几个线程共用一条，整问整答在一把锁里（从写请求到读完回应、连同 found 后面的内容一直握着，别的线程只等这一问）：
//   请求 read <相对键>\n                    回 found <字节数>\n<内容>，或 absent\n
//   请求 write <相对键> <字节数>\n<内容>    回 written\n（服务确认落盘之后才回）
//   请求 write-many <条数>\n 后接每条 <相对键> <字节数>\n<内容>    整批回一个 written\n；条数至少 1。
//     整批经 prune-server 客户端的 write_many 写服务，客户端可能把一批切成几帧、各帧各自提交：条数、字节数、值的形态（长值与以分段清单魔数起头的值分段存）、
//     键留着的登记旧版本（多一次按前缀删）都会让它多切，切法以 scripts/prune-server/prune_server_client.py 文件头为准。原子性只在一帧之内：
//     一批写到半截出了事，服务里可能只有其中几帧。整批的 written 在全部帧都确认落盘之后才回。
//   三种都可能回 failed <退出码> <一句>\n：这里打 ✗ 与那一句、进程照那个退出码退（75 是 prune-server 不可用，3 是别的错），不当「键不在」现判。
//   相对键只用 ASCII 字母、数字与 . _ - /，不空、不带空白（与 device-code-key.py 的 RELATIVE_KEY_PATTERN 同一条）：read、write 与 write-many 在拿锁、
//   拼请求之前先在这一侧核（转交与内存两种都核），不合的照「别的错」退 3，管道上一个字节都不发；那一侧照同一条再核，不合的回 failed 3。
//   管道断了、回应认不出、found 报的字节数在这个进程里分配不出，都退 3。
//   found 报的字节数先可失败地预留、不写零，内容读到多少占多少常驻内存：内容来之前不按报的字节数整段占上。
//   found 后面的内容不是 UTF-8 时 read 交 None（与键不在一样，调用方去现判），不退。
// 撤掉的项（那一侧核出编译与跑之间源码变了时改写成撤回标记的）由那一侧回 absent，这里看不到标记。
// 每个用到它的装置在自己的 Cargo.toml 里按 path 依赖引本包（crates/singlefs-device-library），再 `use singlefs_device_library::DeviceLibrary;`
// （从仓根算单元键时 unit-graph 看不到 `#[path]` 引进来的文件，键拒算；条目级 include! 同样不收）。
// 装置的单测换成 `DeviceLibrary::in_memory()`（内存那一种，测完即丢，不落盘）：它不挂 cfg、也不挂特性，一律编（本包的 Cargo.toml 写了为什么不设特性），
// 装置只在单测里调它；转交的整条路由 device-code-key.py --selftest 在临时服务底下核。
#![forbid(unsafe_code)]

use std::io::{BufRead, Read, Write};

/// 装置写请求的那根管道（device-code-key.py run 放进环境的描述符号）。
const REQUEST_DESCRIPTOR_VARIABLE: &str = "DEVICE_LIBRARY_REQUEST_FD";
/// 装置读回应的那根管道。
const REPLY_DESCRIPTOR_VARIABLE: &str = "DEVICE_LIBRARY_REPLY_FD";
/// 「别的错」的退出码（与 prune-server 客户端「别的错」同一个）：管道断了、回应认不出、相对键不合都照它退。
const OTHER_ERROR_EXIT_CODE: i32 = 3;
/// 转交那一种的锁为什么不会中毒：持锁时做的几样都不 panic。
const RELAY_LOCK_IS_NEVER_POISONED: &str = "转交判定库的锁不会中毒：持锁时只做拼请求、管道读写、按回应报的字节数可失败地预留（报的数超过 isize::MAX 先当认不出退掉，预留不成照别的错退掉）、限着报的字节数读进预留好的那一块（read_to_end 要再长时可失败地分配，分配不出交错误、不 panic）与 give_up（写标准错误不看成败、退进程不展开），这几样都不 panic";
/// 内存那一种的锁为什么不会中毒：持锁时做的几样都不 panic。
const MEMORY_LOCK_IS_NEVER_POISONED: &str = "单测内存库的锁不会中毒：持锁时只做 BTreeMap 的插入、查找、数键与成批写次数加一（usize 数不满），这几样都不 panic";

struct RelayEnds {
    requests: std::fs::File,
    replies: std::io::BufReader<std::fs::File>,
}

/// 单测的内存那一种：存的条目，与成批写发过几次（单测数「一批一次」用，真管道那一头是请求条数）。
#[derive(Default)]
struct MemoryLibrary {
    entries: std::collections::BTreeMap<String, String>,
    write_many_calls: usize,
}

enum Backend {
    Relay(std::sync::Mutex<RelayEnds>),
    Memory(std::sync::Mutex<MemoryLibrary>),
}

/// 一个装置这一趟的判定库口子：克隆的几份共用同一条管道（判等按是不是同一条）。
#[derive(Clone)]
pub struct DeviceLibrary {
    backend: std::sync::Arc<Backend>,
}

impl std::fmt::Debug for DeviceLibrary {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match *self.backend {
            Backend::Relay(_) => "DeviceLibrary(经 device-code-key.py 转给 prune-server)",
            Backend::Memory(_) => "DeviceLibrary(单测的内存那一种)",
        })
    }
}

impl PartialEq for DeviceLibrary {
    fn eq(&self, other: &DeviceLibrary) -> bool {
        std::sync::Arc::ptr_eq(&self.backend, &other.backend)
    }
}

impl Eq for DeviceLibrary {}

/// 读写没成：打 ✗ 与出路，照那个退出码退（文件头「协议」）。持锁时也调它，所以不许 panic：
/// 标准错误写不进（读它的那一头没了）也照退，不用 `eprintln!`（它写不进就 panic）。
fn give_up(exit_code: i32, sentence: &str) -> ! {
    {
        let mut standard_error = std::io::stderr().lock();
        let _ = writeln!(standard_error, "  ✗ 判定库读写没成：{sentence}");
        let _ = writeln!(
            standard_error,
            "     → 怎么办：退 75 是 prune-server 不可用，python3 prune-server/control check 看服务，修好之后由交它的会话重交同一条命令；别的码照这一句修"
        );
    }
    std::process::exit(exit_code)
}

/// 转交那一侧收的相对键：不空，只用 ASCII 字母、数字与 . _ - /（device-code-key.py 的 RELATIVE_KEY_PATTERN `[A-Za-z0-9._/-]+` 整串匹配）。
fn is_relative_key_the_relay_accepts(relative: &str) -> bool {
    !relative.is_empty()
        && relative
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/'))
}

/// 相对键不合就照「别的错」退；调用方在拿锁、拼请求之前调，管道上一个字节都不发。
fn give_up_unless_the_relay_accepts_the_key(relative: &str, asked: &str) {
    if !is_relative_key_the_relay_accepts(relative) {
        give_up(
            OTHER_ERROR_EXIT_CODE,
            &format!("{asked}：相对键「{relative}」不合（只许 ASCII 字母、数字与 . _ - /，不空、不带空白）"),
        );
    }
}

/// 读一行回应（去掉换行）；管道断了退 3。
fn reply_line(replies: &mut std::io::BufReader<std::fs::File>) -> String {
    let mut line = String::new();
    match replies.read_line(&mut line) {
        Ok(0) => give_up(
            OTHER_ERROR_EXIT_CODE,
            "转交判定库的管道断了（起这个装置的 device-code-key.py 没了）",
        ),
        Ok(_) => line.trim_end_matches('\n').to_string(),
        Err(error) => give_up(
            OTHER_ERROR_EXIT_CODE,
            &format!("读不了转交判定库的管道：{error}"),
        ),
    }
}

/// `failed <退出码> <一句>` 那一种回应：照它退；别的认不出的回应退 3。
fn give_up_on_reply(reply: &str, asked: &str) -> ! {
    if let Some(rest) = reply.strip_prefix("failed ") {
        let (code_text, sentence) = rest.split_once(' ').unwrap_or((rest, ""));
        let exit_code = code_text.parse::<i32>().unwrap_or(OTHER_ERROR_EXIT_CODE);
        give_up(exit_code, &format!("{asked}：{sentence}"));
    }
    give_up(
        OTHER_ERROR_EXIT_CODE,
        &format!("{asked}：转交判定库的回应认不出「{reply}」"),
    )
}

/// 整条写请求发出去，等回一个 `written`；写不进管道、回的不是 `written` 都照退出码退（文件头「协议」）。
fn send_write_request_and_wait_for_written(ends: &mut RelayEnds, request: &[u8], asked: &str) {
    if let Err(error) = ends.requests.write_all(request) {
        give_up(
            OTHER_ERROR_EXIT_CODE,
            &format!("{asked}：写不进转交判定库的管道：{error}"),
        );
    }
    let reply = reply_line(&mut ends.replies);
    if reply != "written" {
        give_up_on_reply(&reply, asked);
    }
}

impl DeviceLibrary {
    /// 照 device-code-key.py run 放进环境的两个描述符号接上转交的管道；没有、写错或打不开交一句为什么（调用方照用法错办）。
    pub fn from_environment() -> Result<DeviceLibrary, String> {
        let descriptor_path = |name: &str| -> Result<String, String> {
            let text = std::env::var(name).map_err(|_| {
                format!("环境里没有 {name}：读写判定库要经 scripts/experiment-apparatus/device-code-key.py run 起（它开管道转给 prune-server）")
            })?;
            if text.is_empty() || !text.chars().all(|character| character.is_ascii_digit()) {
                return Err(format!("{name}={text} 不是描述符号"));
            }
            Ok(format!("/proc/self/fd/{text}"))
        };
        let request_path = descriptor_path(REQUEST_DESCRIPTOR_VARIABLE)?;
        let reply_path = descriptor_path(REPLY_DESCRIPTOR_VARIABLE)?;
        let requests = std::fs::OpenOptions::new()
            .write(true)
            .open(&request_path)
            .map_err(|error| format!("打不开转交判定库的请求管道 {request_path}：{error}"))?;
        let replies = std::fs::File::open(&reply_path)
            .map_err(|error| format!("打不开转交判定库的回应管道 {reply_path}：{error}"))?;
        Ok(DeviceLibrary {
            backend: std::sync::Arc::new(Backend::Relay(std::sync::Mutex::new(RelayEnds {
                requests,
                replies: std::io::BufReader::new(replies),
            }))),
        })
    }

    /// 单测用的内存那一种：一份测试一份，测完即丢。
    pub fn in_memory() -> DeviceLibrary {
        DeviceLibrary {
            backend: std::sync::Arc::new(Backend::Memory(std::sync::Mutex::new(
                MemoryLibrary::default(),
            ))),
        }
    }

    /// 读一条：不在（或已撤掉）交 None，回的内容不是 UTF-8 也交 None；相对键不合、读失败照退出码退，不交 None。
    pub fn read(&self, relative: &str) -> Option<String> {
        let asked = format!("读 {relative}");
        give_up_unless_the_relay_accepts_the_key(relative, &asked);
        match &*self.backend {
            Backend::Relay(ends) => {
                let mut ends = ends.lock().expect(RELAY_LOCK_IS_NEVER_POISONED);
                if let Err(error) = ends
                    .requests
                    .write_all(format!("read {relative}\n").as_bytes())
                {
                    give_up(
                        OTHER_ERROR_EXIT_CODE,
                        &format!("{asked}：写不进转交判定库的管道：{error}"),
                    );
                }
                let reply = reply_line(&mut ends.replies);
                if reply == "absent" {
                    return None;
                }
                // 报的字节数超过 isize::MAX 的不是任何一个值的长度：当认不出的回应退掉。
                let Some(length) = reply
                    .strip_prefix("found ")
                    .and_then(|text| text.parse::<usize>().ok())
                    .filter(|length| isize::try_from(*length).is_ok())
                else {
                    give_up_on_reply(&reply, &asked)
                };
                // 不可失败的分配（vec!、with_capacity）分配不出时中止进程、退出码不是 3：先可失败地预留，预留不成照别的错退。
                let mut contents = Vec::new();
                if let Err(error) = contents.try_reserve_exact(length) {
                    give_up(
                        OTHER_ERROR_EXIT_CODE,
                        &format!("{asked}：回应报的 {length} 字节在这个进程里分配不出：{error}"),
                    );
                }
                // 预留好的那一块不先写零，读到多少占多少：回应报了大长度而内容迟迟不来时，常驻内存不按报的字节数整段涨上去。
                // usize 在本仓支持的目标上装得进 u64；装不进时取 u64::MAX，照样读到文件尾为止，读到的字节数对不上照「没读全」退。
                let announced_length = u64::try_from(length).unwrap_or(u64::MAX);
                match (&mut ends.replies)
                    .take(announced_length)
                    .read_to_end(&mut contents)
                {
                    Ok(taken) if taken == length => {}
                    Ok(taken) => give_up(
                        OTHER_ERROR_EXIT_CODE,
                        &format!("{asked}：回应里的 {length} 字节没读全：只读到 {taken} 字节"),
                    ),
                    Err(error) => give_up(
                        OTHER_ERROR_EXIT_CODE,
                        &format!("{asked}：回应里的 {length} 字节没读全：{error}"),
                    ),
                }
                String::from_utf8(contents).ok()
            }
            Backend::Memory(library) => library
                .lock()
                .expect(MEMORY_LOCK_IS_NEVER_POISONED)
                .entries
                .get(relative)
                .cloned(),
        }
    }

    /// 写一条（服务确认落盘之后才回来）；相对键不合、写失败照退出码退。
    pub fn write(&self, relative: &str, text: &str) {
        let asked = format!("写 {relative}");
        give_up_unless_the_relay_accepts_the_key(relative, &asked);
        match &*self.backend {
            Backend::Relay(ends) => {
                let mut ends = ends.lock().expect(RELAY_LOCK_IS_NEVER_POISONED);
                let mut request = format!("write {relative} {}\n", text.len()).into_bytes();
                request.extend_from_slice(text.as_bytes());
                send_write_request_and_wait_for_written(&mut ends, &request, &asked);
            }
            Backend::Memory(library) => {
                library
                    .lock()
                    .expect(MEMORY_LOCK_IS_NEVER_POISONED)
                    .entries
                    .insert(relative.to_string(), text.to_string());
            }
        }
    }

    /// 一次写一批（相对键, 内容），服务确认整批落盘之后才回来；有一条相对键不合整批一条都不发、照退出码退，写失败照退出码退。空的一批不发请求。
    pub fn write_many(&self, entries: &[(String, String)]) {
        if entries.is_empty() {
            return;
        }
        let asked = format!("成批写 {} 条", entries.len());
        for (relative, _text) in entries {
            give_up_unless_the_relay_accepts_the_key(relative, &asked);
        }
        match &*self.backend {
            Backend::Relay(ends) => {
                let mut ends = ends.lock().expect(RELAY_LOCK_IS_NEVER_POISONED);
                let mut request = format!("write-many {}\n", entries.len()).into_bytes();
                for (relative, text) in entries {
                    request.extend_from_slice(format!("{relative} {}\n", text.len()).as_bytes());
                    request.extend_from_slice(text.as_bytes());
                }
                send_write_request_and_wait_for_written(&mut ends, &request, &asked);
            }
            Backend::Memory(library) => {
                let mut library = library.lock().expect(MEMORY_LOCK_IS_NEVER_POISONED);
                library.write_many_calls += 1;
                for (relative, text) in entries {
                    library.entries.insert(relative.clone(), text.clone());
                }
            }
        }
    }

    /// 单测数条数：以 prefix 起头、以 suffix 收尾的键有几条。
    pub fn count_entries(&self, prefix: &str, suffix: &str) -> usize {
        match &*self.backend {
            Backend::Relay(_) => panic!("单测只数内存那一种"),
            Backend::Memory(library) => library
                .lock()
                .expect(MEMORY_LOCK_IS_NEVER_POISONED)
                .entries
                .keys()
                .filter(|key| key.starts_with(prefix) && key.ends_with(suffix))
                .count(),
        }
    }

    /// 单测数次数：非空的成批写发过几次。
    pub fn count_write_many_calls(&self) -> usize {
        match &*self.backend {
            Backend::Relay(_) => panic!("单测只数内存那一种"),
            Backend::Memory(library) => {
                library
                    .lock()
                    .expect(MEMORY_LOCK_IS_NEVER_POISONED)
                    .write_many_calls
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_relative_key_the_relay_accepts, Backend, DeviceLibrary, RelayEnds};
    use std::io::{Read, Write};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// 转交那一侧在测试里的替身：装置写进去的请求从 `requests_written_by_the_device` 读出来，
    /// 预先写好的回应放在装置读的那一根管道里（写端随即关上）。
    struct RelayCounterpart {
        device_library: DeviceLibrary,
        requests_written_by_the_device: std::io::PipeReader,
    }

    /// 转交那一种的口子：请求写进 `requests`，回应是预先写好的那几行（回应管道的写端随即关上，读完就是文件尾）。
    fn relay_writing_requests_to(
        requests: std::fs::File,
        prepared_replies: &[u8],
    ) -> DeviceLibrary {
        let (reply_end_of_the_device, mut reply_end_of_the_counterpart) =
            std::io::pipe().expect("建回应管道");
        reply_end_of_the_counterpart
            .write_all(prepared_replies)
            .expect("预先写好的回应比管道缓冲小，写得进");
        drop(reply_end_of_the_counterpart);
        let replies = std::io::BufReader::new(std::fs::File::from(std::os::fd::OwnedFd::from(
            reply_end_of_the_device,
        )));
        DeviceLibrary {
            backend: std::sync::Arc::new(Backend::Relay(std::sync::Mutex::new(RelayEnds {
                requests,
                replies,
            }))),
        }
    }

    fn relay_with_prepared_replies(prepared_replies: &[u8]) -> RelayCounterpart {
        let (requests_written_by_the_device, request_end_of_the_device) =
            std::io::pipe().expect("建请求管道");
        let requests = std::fs::File::from(std::os::fd::OwnedFd::from(request_end_of_the_device));
        RelayCounterpart {
            device_library: relay_writing_requests_to(requests, prepared_replies),
            requests_written_by_the_device,
        }
    }

    /// 装置这一头全部关上之后（克隆都丢了），把它写过的请求整段读出来。
    fn requests_after_the_device_is_dropped(counterpart: RelayCounterpart) -> String {
        let RelayCounterpart {
            device_library,
            mut requests_written_by_the_device,
        } = counterpart;
        drop(device_library);
        let mut requests = String::new();
        requests_written_by_the_device
            .read_to_string(&mut requests)
            .expect("请求管道读得到文件尾");
        requests
    }

    #[test]
    fn read_takes_exactly_the_announced_bytes_after_found_and_the_next_reply_absent_reads_as_missing(
    ) {
        let counterpart = relay_with_prepared_replies(b"found 5\nhelloabsent\n");
        assert_eq!(
            counterpart
                .device_library
                .read("items/present.out")
                .as_deref(),
            Some("hello"),
            "found 之后恰好取它报的字节数，不多不少"
        );
        assert_eq!(
            counterpart.device_library.read("items/missing.out"),
            None,
            "下一条回应 absent 读成键不在"
        );
        assert_eq!(
            requests_after_the_device_is_dropped(counterpart),
            "read items/present.out\nread items/missing.out\n",
            "每问一条 read 请求、以换行收尾"
        );
    }

    // 在子进程那一半里读：found 之后读的字节数不对时，下一条回应从非 UTF-8 的字节读起，口子照「别的错」退进程，
    // 在同一个进程里会连同并行的别的测试一起退掉；子进程退了，这里断言它的退出码。
    #[test]
    fn read_answered_found_with_contents_that_are_not_utf8_gives_none_and_takes_exactly_those_bytes(
    ) {
        if is_the_child_half() {
            let device_library =
                relay_sending_requests_to_the_standard_input(b"found 2\n\xff\xfeabsent\n");
            assert_eq!(
                device_library.read("items/not-utf8.out"),
                None,
                "found 后面的内容不是 UTF-8 时交 None（与键不在一样，调用方去现判），不换成替换字符交出去"
            );
            assert_eq!(
                device_library.read("items/missing.out"),
                None,
                "下一条回应 absent 照常读成不在：非 UTF-8 的两个字节恰好读走"
            );
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::read_answered_found_with_contents_that_are_not_utf8_gives_none_and_takes_exactly_those_bytes",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(0),
            "子进程那一半两次读都交 None、没退进程（断言红退 101，口子退进程退 3）；标准错误：{}",
            outcome.standard_error
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.requests_sent),
            "read items/not-utf8.out\nread items/missing.out\n",
            "每问一条 read 请求、以换行收尾"
        );
    }

    #[test]
    fn write_announces_the_length_in_bytes_not_characters_and_returns_after_written() {
        let counterpart = relay_with_prepared_replies(b"written\n");
        counterpart
            .device_library
            .write("codes/multibyte.txt", "判定");
        assert_eq!(
            requests_after_the_device_is_dropped(counterpart),
            "write codes/multibyte.txt 6\n判定",
            "write 请求报的是内容的字节数（两个汉字六个字节），后面紧跟内容"
        );
    }

    /// 回应管道里这一刻没读走的全部字节（预先写好的回应写端已关，读得到文件尾）。
    fn replies_left_unread(device_library: &DeviceLibrary) -> Vec<u8> {
        match &*device_library.backend {
            Backend::Relay(ends) => {
                let mut left = Vec::new();
                ends.lock()
                    .expect("单测里没有别的线程握这把锁")
                    .replies
                    .read_to_end(&mut left)
                    .expect("回应管道读得到文件尾");
                left
            }
            Backend::Memory(_) => panic!("只数转交那一种的回应"),
        }
    }

    #[test]
    fn write_many_announces_the_count_and_each_length_in_bytes_and_takes_one_written_for_the_whole_batch(
    ) {
        let counterpart = relay_with_prepared_replies(b"written\nnot-for-this-batch\n");
        counterpart.device_library.write_many(&[
            ("items/multibyte.out".to_string(), "判定".to_string()),
            ("items/ascii.out".to_string(), "ok".to_string()),
        ]);
        assert_eq!(
            replies_left_unread(&counterpart.device_library),
            b"not-for-this-batch\n",
            "整批恰好读走一个 written，不多读也不少读"
        );
        assert_eq!(
            requests_after_the_device_is_dropped(counterpart),
            "write-many 2\nitems/multibyte.out 6\n判定items/ascii.out 2\nok",
            "头一行报条数；每条报内容的字节数（两个汉字六个字节），后面紧跟内容"
        );
    }

    #[test]
    fn write_many_of_nothing_sends_no_request_and_takes_no_reply() {
        let counterpart = relay_with_prepared_replies(b"written\n");
        counterpart.device_library.write_many(&[]);
        assert_eq!(
            replies_left_unread(&counterpart.device_library),
            b"written\n",
            "空的一批不等回应"
        );
        assert_eq!(
            requests_after_the_device_is_dropped(counterpart),
            "",
            "空的一批不发请求"
        );
    }

    #[test]
    fn write_many_in_memory_stores_every_entry_and_counts_one_call_per_non_empty_batch() {
        let device_library = DeviceLibrary::in_memory();
        device_library.write_many(&[
            ("items/one.out".to_string(), "甲".to_string()),
            ("items/two.out".to_string(), "乙".to_string()),
        ]);
        device_library.write_many(&[]);
        device_library.write("codes/key.txt", "清单");
        assert_eq!(device_library.read("items/one.out").as_deref(), Some("甲"));
        assert_eq!(
            device_library.read("items/two.out").as_deref(),
            Some("乙"),
            "批里每一条都存下，不止头一条"
        );
        assert_eq!(device_library.count_entries("items/", ".out"), 2);
        assert_eq!(
            device_library.count_write_many_calls(),
            1,
            "非空的一批记一次，空的一批与单条写不记"
        );
    }

    #[test]
    fn relative_keys_the_relay_accepts_are_non_empty_ascii_letters_digits_and_dot_underscore_dash_slash(
    ) {
        for accepted in ["items/a-b_c.out", "codes/0.txt", "index.tsv", "A/Z/9"] {
            assert!(
                is_relative_key_the_relay_accepts(accepted),
                "「{accepted}」只用字母、数字与 . _ - /，收"
            );
        }
        for refused in [
            "",
            "items/bad key.out",
            "items/tab\tkey.out",
            "items/new\nline.out",
            "items/é.out",
            "items/semicolon;.out",
        ] {
            assert!(
                !is_relative_key_the_relay_accepts(refused),
                "「{refused}」是空串、带空白或带别的字符，不收"
            );
        }
    }

    // ── 退进程的几条：口子照退出码退进程（give_up 调 std::process::exit），同一个进程里测不了，
    //    照测试名再起一次本测试二进制、只跑那一条，看它的退出码与它往请求管道写过的字节。

    /// 子进程那一半的标记：设了它，点名的测试只做它那一半（调一次口子），口子没退进程就退 0。
    const CHILD_HALF_VARIABLE: &str = "SINGLEFS_DEVICE_LIBRARY_TEST_CHILD_HALF";

    fn is_the_child_half() -> bool {
        std::env::var_os(CHILD_HALF_VARIABLE).is_some()
    }

    /// 子进程的标准错误接到哪里。
    enum ChildStandardError {
        /// 接一根管道，读出来写进断言的消息里。
        Captured,
        /// 接一根读的那一头起子进程之前就关掉的管道：子进程往标准错误写都写不进。
        ClosedBeforeTheChildWrites,
    }

    /// 子进程那一半跑完的样子：退出码（被信号杀掉是 None）、它往请求管道写过的全部字节、它打到标准错误的字。
    struct ChildHalfOutcome {
        exit_code: Option<i32>,
        requests_sent: Vec<u8>,
        standard_error: String,
    }

    /// 照测试名（`tests::<函数名>`）再起一次本测试二进制、只跑那一条的子进程那一半。子进程的标准输入接一根管道的写端，
    /// 子进程把它当请求那一头（`relay_sending_requests_to_the_standard_input`），这里读出它写过的字节。
    fn run_the_child_half(test_name: &str, standard_error: ChildStandardError) -> ChildHalfOutcome {
        let (mut requests_sent_by_the_child, request_end_for_the_child) =
            std::io::pipe().expect("建请求管道");
        let mut command =
            std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"));
        command
            .args([test_name, "--exact", "--nocapture"])
            .env(CHILD_HALF_VARIABLE, "1")
            .stdin(request_end_for_the_child)
            .stdout(std::process::Stdio::null());
        match standard_error {
            ChildStandardError::Captured => {
                command.stderr(std::process::Stdio::piped());
            }
            ChildStandardError::ClosedBeforeTheChildWrites => {
                let (reader_closed_at_once, standard_error_of_the_child) =
                    std::io::pipe().expect("建标准错误管道");
                drop(reader_closed_at_once);
                command.stderr(standard_error_of_the_child);
            }
        }
        let output = command.output().expect("起得了子进程");
        // 请求管道写端的这一份在 command 里，丢掉它之后读得到文件尾。
        drop(command);
        let mut requests_sent = Vec::new();
        requests_sent_by_the_child
            .read_to_end(&mut requests_sent)
            .expect("请求管道读得到文件尾");
        ChildHalfOutcome {
            exit_code: output.status.code(),
            requests_sent,
            standard_error: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// 子进程那一半的转交口子：请求写进标准输入（父进程接的是一根管道的写端），回应是预先写好的那几行。
    fn relay_sending_requests_to_the_standard_input(prepared_replies: &[u8]) -> DeviceLibrary {
        use std::os::fd::AsFd;
        let requests = std::fs::File::from(
            std::io::stdin()
                .as_fd()
                .try_clone_to_owned()
                .expect("标准输入的描述符复制得了"),
        );
        relay_writing_requests_to(requests, prepared_replies)
    }

    #[test]
    fn write_with_a_key_the_relay_refuses_exits_with_the_other_error_code_before_sending_a_byte() {
        if is_the_child_half() {
            relay_sending_requests_to_the_standard_input(b"written\n")
                .write("items/bad key.out", "x");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::write_with_a_key_the_relay_refuses_exits_with_the_other_error_code_before_sending_a_byte",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "带空格的相对键照「别的错」退 3，不当写成了；标准错误：{}",
            outcome.standard_error
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.requests_sent),
            "",
            "相对键不合，管道上一个字节都不发"
        );
    }

    #[test]
    fn write_many_with_a_key_the_relay_refuses_exits_with_the_other_error_code_before_sending_a_byte(
    ) {
        if is_the_child_half() {
            relay_sending_requests_to_the_standard_input(b"written\n").write_many(&[
                ("items/good.out".to_string(), "x".to_string()),
                ("items/bad key.out".to_string(), "y".to_string()),
            ]);
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::write_many_with_a_key_the_relay_refuses_exits_with_the_other_error_code_before_sending_a_byte",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "一批里有一条相对键带空格，整批照「别的错」退 3；标准错误：{}",
            outcome.standard_error
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.requests_sent),
            "",
            "一批里有一条相对键不合，前面合的那一条也不发，管道上一个字节都不发"
        );
    }

    #[test]
    fn in_memory_write_many_with_a_key_the_relay_refuses_exits_with_the_other_error_code() {
        if is_the_child_half() {
            DeviceLibrary::in_memory()
                .write_many(&[("items/bad key.out".to_string(), "x".to_string())]);
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::in_memory_write_many_with_a_key_the_relay_refuses_exits_with_the_other_error_code",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "内存那一种与转交那一种核同一条相对键规则：单测里收下的键真跑也收；标准错误：{}",
            outcome.standard_error
        );
    }

    #[test]
    fn in_memory_write_with_a_key_the_relay_refuses_exits_with_the_other_error_code() {
        if is_the_child_half() {
            DeviceLibrary::in_memory().write("items/bad key.out", "x");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::in_memory_write_with_a_key_the_relay_refuses_exits_with_the_other_error_code",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "内存那一种的单条 write 与转交那一种核同一条相对键规则：单测里收下的键真跑也收；标准错误：{}",
            outcome.standard_error
        );
    }

    #[test]
    fn write_answered_failed_exits_with_the_code_in_the_reply() {
        if is_the_child_half() {
            relay_sending_requests_to_the_standard_input(b"failed 75 prune-server stopped\n")
                .write("items/a.out", "x");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::write_answered_failed_exits_with_the_code_in_the_reply",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(75),
            "回 failed 75 的单条写照 75 退，不当写成了；标准错误：{}",
            outcome.standard_error
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.requests_sent),
            "write items/a.out 1\nx",
            "请求照常发出，回应是 failed 才退"
        );
    }

    #[test]
    fn write_many_answered_failed_exits_with_the_code_in_the_reply() {
        if is_the_child_half() {
            relay_sending_requests_to_the_standard_input(b"failed 75 prune-server stopped\n")
                .write_many(&[("items/a.out".to_string(), "x".to_string())]);
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::write_many_answered_failed_exits_with_the_code_in_the_reply",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(75),
            "回 failed 75 的一批照 75 退，不当写成了；标准错误：{}",
            outcome.standard_error
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.requests_sent),
            "write-many 1\nitems/a.out 1\nx",
            "请求照常发出，回应是 failed 才退"
        );
    }

    #[test]
    fn read_answered_failed_exits_with_the_code_in_the_reply() {
        if is_the_child_half() {
            relay_sending_requests_to_the_standard_input(b"failed 75 prune-server stopped\n")
                .read("items/a.out");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::read_answered_failed_exits_with_the_code_in_the_reply",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(75),
            "回 failed 75 的读照 75 退，不当键不在；标准错误：{}",
            outcome.standard_error
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.requests_sent),
            "read items/a.out\n",
            "请求照常发出，回应是 failed 才退"
        );
    }

    #[test]
    fn read_answered_found_with_a_length_past_isize_maximum_exits_with_the_other_error_code() {
        if is_the_child_half() {
            let length_past_isize_maximum =
                usize::try_from(isize::MAX).expect("isize::MAX 是正数，装得进 usize") + 1;
            relay_sending_requests_to_the_standard_input(
                format!("found {length_past_isize_maximum}\n").as_bytes(),
            )
            .read("items/a.out");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::read_answered_found_with_a_length_past_isize_maximum_exits_with_the_other_error_code",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "报的字节数超过 isize::MAX 当认不出的回应退 3，不在锁里 panic；标准错误：{}",
            outcome.standard_error
        );
        assert!(
            outcome.standard_error.contains("回应认不出"),
            "超过 isize::MAX 的字节数在预留之前就当认不出的回应退掉，不走到「分配不出」；标准错误：{}",
            outcome.standard_error
        );
    }

    #[test]
    fn read_answered_found_with_a_length_that_cannot_be_allocated_exits_with_the_other_error_code()
    {
        if is_the_child_half() {
            // 2^62 字节不过 isize::MAX，可大过 x86_64 与 aarch64 的用户地址空间，哪台机器上都分配不出。
            let length_that_cannot_be_allocated = 1_usize << 62;
            relay_sending_requests_to_the_standard_input(
                format!("found {length_that_cannot_be_allocated}\n").as_bytes(),
            )
            .read("items/a.out");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::read_answered_found_with_a_length_that_cannot_be_allocated_exits_with_the_other_error_code",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "found 报的字节数分配不出照「别的错」退 3，不中止进程（被信号杀掉时退出码是 None）；标准错误：{}",
            outcome.standard_error
        );
    }

    #[test]
    fn read_answered_found_with_fewer_bytes_than_announced_exits_with_the_other_error_code() {
        if is_the_child_half() {
            relay_sending_requests_to_the_standard_input(b"found 5\nhel").read("items/a.out");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::read_answered_found_with_fewer_bytes_than_announced_exits_with_the_other_error_code",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "found 报 5 字节、管道到文件尾只来了 3 字节，照「没读全」退 3，不把读到的那几个字节当整份交出去；标准错误：{}",
            outcome.standard_error
        );
        assert!(
            outcome.standard_error.contains("没读全"),
            "退的理由是内容没读全；标准错误：{}",
            outcome.standard_error
        );
    }

    /// 本进程此刻的常驻内存（/proc/self/status 的 VmRSS 一行），字节。
    fn resident_bytes_of_this_process() -> usize {
        let status =
            std::fs::read_to_string("/proc/self/status").expect("读得了 /proc/self/status");
        let resident_line = status
            .lines()
            .find(|line| line.starts_with("VmRSS:"))
            .expect("/proc/self/status 里有 VmRSS 一行");
        let resident_kibibytes: usize = resident_line
            .trim_start_matches("VmRSS:")
            .trim()
            .trim_end_matches("kB")
            .trim()
            .parse()
            .expect("VmRSS 一行报的是 kB 数");
        resident_kibibytes
            .checked_mul(1024)
            .expect("常驻内存的字节数装得进 usize")
    }

    // 量的是本进程的常驻内存：同一个测试二进制里并行的别的测试也占内存，它们合起来远小于报的字节数的一半，判据留了这个余量。
    #[test]
    fn read_answered_found_touches_no_more_memory_than_the_contents_that_arrived() {
        const ANNOUNCED_BYTES: usize = 32 * 1024 * 1024;
        const CHUNK_BYTES: usize = 64 * 1024;
        // 内容一个字节都还没写时，等常驻内存涨过报的一半、最多等这么久。
        const WAIT_BEFORE_THE_CONTENTS: std::time::Duration = std::time::Duration::from_secs(3);
        const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(20);
        let (_requests_for_the_counterpart, request_end_of_the_device) =
            std::io::pipe().expect("建请求管道");
        let (reply_end_of_the_device, mut reply_end_of_the_counterpart) =
            std::io::pipe().expect("建回应管道");
        let device_library = DeviceLibrary {
            backend: std::sync::Arc::new(Backend::Relay(std::sync::Mutex::new(RelayEnds {
                requests: std::fs::File::from(std::os::fd::OwnedFd::from(
                    request_end_of_the_device,
                )),
                replies: std::io::BufReader::new(std::fs::File::from(std::os::fd::OwnedFd::from(
                    reply_end_of_the_device,
                ))),
            }))),
        };
        reply_end_of_the_counterpart
            .write_all(format!("found {ANNOUNCED_BYTES}\n").as_bytes())
            .expect("回应管道写得进");
        let resident_before = resident_bytes_of_this_process();
        let reader = {
            let device_library = device_library.clone();
            std::thread::spawn(move || device_library.read("items/a.out").map(|text| text.len()))
        };
        let started = std::time::Instant::now();
        let mut growth_while_no_contents_arrived = 0_usize;
        while started.elapsed() < WAIT_BEFORE_THE_CONTENTS {
            growth_while_no_contents_arrived =
                resident_bytes_of_this_process().saturating_sub(resident_before);
            if growth_while_no_contents_arrived >= ANNOUNCED_BYTES / 2 {
                break;
            }
            std::thread::sleep(POLL_INTERVAL);
        }
        let chunk = vec![b'v'; CHUNK_BYTES];
        for _ in 0..ANNOUNCED_BYTES / CHUNK_BYTES {
            reply_end_of_the_counterpart
                .write_all(&chunk)
                .expect("回应管道写得进");
        }
        assert_eq!(
            reader.join().expect("装置线程照常收尾"),
            Some(ANNOUNCED_BYTES),
            "内容到齐之后照常交回报的那么多字节"
        );
        println!(
            "name=resident_growth_before_contents announced={ANNOUNCED_BYTES} growth={growth_while_no_contents_arrived}"
        );
        assert!(
            growth_while_no_contents_arrived < ANNOUNCED_BYTES / 2,
            "found 报了 {ANNOUNCED_BYTES} 字节、内容一个字节都没来，常驻内存已经涨了 {growth_while_no_contents_arrived} 字节：内容来之前就把报的字节数整段写了零"
        );
    }

    #[test]
    fn read_with_a_newline_in_the_key_exits_with_the_other_error_code_before_sending_a_byte() {
        if is_the_child_half() {
            relay_sending_requests_to_the_standard_input(b"absent\nwritten\n")
                .read("items/a.out\nwrite items/injected.out 0");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::read_with_a_newline_in_the_key_exits_with_the_other_error_code_before_sending_a_byte",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "带换行的相对键照「别的错」退 3，不读成键不在；标准错误：{}",
            outcome.standard_error
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.requests_sent),
            "",
            "换行后面那一段不上管道：转交那一侧读不出一条装置没交给 write 的写请求"
        );
    }

    #[test]
    fn read_with_a_space_in_the_key_exits_with_the_other_error_code_before_sending_a_byte() {
        if is_the_child_half() {
            relay_sending_requests_to_the_standard_input(b"absent\n").read("items/bad key.out");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::read_with_a_space_in_the_key_exits_with_the_other_error_code_before_sending_a_byte",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "带空格的相对键照「别的错」退 3，不读成键不在；标准错误：{}",
            outcome.standard_error
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.requests_sent),
            "",
            "相对键不合，管道上一个字节都不发"
        );
    }

    #[test]
    fn in_memory_read_with_a_key_the_relay_refuses_exits_with_the_other_error_code() {
        if is_the_child_half() {
            DeviceLibrary::in_memory().read("items/bad key.out");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::in_memory_read_with_a_key_the_relay_refuses_exits_with_the_other_error_code",
            ChildStandardError::Captured,
        );
        assert_eq!(
            outcome.exit_code,
            Some(3),
            "内存那一种的 read 与转交那一种核同一条相对键规则：单测里读成不在的键真跑不会退 3；标准错误：{}",
            outcome.standard_error
        );
    }

    #[test]
    fn give_up_with_the_standard_error_closed_still_exits_with_the_code_in_the_reply() {
        if is_the_child_half() {
            relay_sending_requests_to_the_standard_input(b"failed 75 prune-server stopped\n")
                .write("items/a.out", "x");
            std::process::exit(0);
        }
        let outcome = run_the_child_half(
            "tests::give_up_with_the_standard_error_closed_still_exits_with_the_code_in_the_reply",
            ChildStandardError::ClosedBeforeTheChildWrites,
        );
        assert_eq!(
            outcome.exit_code,
            Some(75),
            "标准错误写不进也照回应里的 75 退，不 panic（panic 的测试退 101）"
        );
    }

    // ── 锁从写请求一直握到读完回应（连同 found 后面的内容）：探锁的一方在装置线程放锁的那一刻空转着抢。

    /// 每一问的请求里那一段大数据的字节数（相对键或内容）：比管道缓冲（Linux 默认 64 KiB）大得多，
    /// 装置线程写请求时握着锁卡在管道上、等替身读，探锁的一方在请求写完之前就空转起来。
    const PROBED_REQUEST_PAYLOAD_IN_BYTES: usize = 256 * 1024;

    /// 替身先读这么多字节的请求开头、记下「请求开了头」，再等探锁的一方许它读完。
    const PROBED_REQUEST_BEGINNING_IN_BYTES: usize = 5;

    /// 装置线程每一轮调一次口子的那一步：读的交回读到的，写的交 None。
    type AskTheLibrary = Box<dyn Fn(&DeviceLibrary) -> Option<String> + Send>;

    /// 探锁的几条测试里装置线程每一轮问的那一问。
    struct ProbedExchange {
        /// 装置线程每一轮调一次口子；读的交回读到的，写的交 None。
        ask: AskTheLibrary,
        /// 这一问的整条请求，逐字节（替身照它核每一轮读到的请求）。
        expected_request: Vec<u8>,
        /// 替身按次序一段一段写的回应：每一段写之前等探锁的一方许可，段与段之间是留给探锁的一方抢锁的空当。
        reply_parts: Vec<&'static [u8]>,
        /// 口子每一轮该交回的。
        expected_answer: Option<String>,
    }

    /// 一轮里各方走到哪一步。请求那三个量记的是走到这一步的最大轮号（从 1 数）；
    /// 回应那两个量记的是从头数起的段数（每轮的段数相同，第 n 轮写完时是 n 乘每轮段数）。
    #[derive(Default)]
    struct ProbeRoundProgress {
        /// 替身读到了这一轮请求的开头（装置线程这时握着锁、请求写到一半）。
        request_begun: AtomicUsize,
        /// 探锁的一方空转起来之后，许替身读完这一轮的请求。
        draining_allowed: AtomicUsize,
        /// 替身读全了这一轮的请求。
        request_complete: AtomicUsize,
        /// 探锁的一方许替身写到第几段回应。
        reply_parts_allowed: AtomicUsize,
        /// 替身要写第几段回应了（先记再写）。
        reply_parts_written: AtomicUsize,
    }

    /// 空转等到条件成立（让出处理器，不睡）。
    fn spin_until(condition: impl Fn() -> bool) {
        while !condition() {
            std::thread::yield_now();
        }
    }

    // 线程数是场景的一部分，不是切片并行：装置线程、替身线程与探锁的本线程各一个，每一轮管道上只有一问，
    // 探锁抢到锁的那一刻才判得出它落没落在这一问的中间。
    /// 装置线程一轮一轮问 `exchange` 那一问，探锁的一方每一轮空转着抢锁；抢到锁时替身还没写完这一轮的回应，
    /// 交那一轮的轮号（锁在一问的中间放过了），轮数跑满都没有交 None。锁一直握着时每一轮都是 None：
    /// 探锁的一方只在替身读到请求开头（装置线程已握着锁）之后才抢，装置线程读完最后一段回应才放锁，那一段在写之前就记上了。
    fn round_where_the_lock_was_free_in_the_middle_of_the_exchange(
        exchange: ProbedExchange,
    ) -> Option<usize> {
        // 轮数只关系到锁在一问的中间放过时多快判出来；锁一直握着时每一轮都绿。
        const ROUNDS: usize = 200;
        // 本线程先握锁、让装置线程在锁上睡下的时长：它醒来拿到锁时锁记成「有人等过」，放锁要进内核叫醒，留给空转的探锁一方抢的空当。
        const QUEUEING_TIME: std::time::Duration = std::time::Duration::from_millis(2);
        // 替身走到下一步（读全请求、写完一段回应）之后，探锁的一方再空转多久才许它写下一段回应。
        const GRACE_AFTER_EACH_STEP: std::time::Duration = std::time::Duration::from_micros(200);
        let ProbedExchange {
            ask,
            expected_request,
            reply_parts,
            expected_answer,
        } = exchange;
        let reply_parts_per_round = reply_parts.len();
        let (mut requests_for_the_counterpart, request_end_of_the_device) =
            std::io::pipe().expect("建请求管道");
        let (reply_end_of_the_device, mut reply_end_of_the_counterpart) =
            std::io::pipe().expect("建回应管道");
        let device_library = DeviceLibrary {
            backend: std::sync::Arc::new(Backend::Relay(std::sync::Mutex::new(RelayEnds {
                requests: std::fs::File::from(std::os::fd::OwnedFd::from(
                    request_end_of_the_device,
                )),
                replies: std::io::BufReader::new(std::fs::File::from(std::os::fd::OwnedFd::from(
                    reply_end_of_the_device,
                ))),
            }))),
        };
        let progress = std::sync::Arc::new(ProbeRoundProgress::default());
        let counterpart = {
            let progress = std::sync::Arc::clone(&progress);
            std::thread::spawn(move || {
                let (expected_beginning, expected_rest) =
                    expected_request.split_at(PROBED_REQUEST_BEGINNING_IN_BYTES);
                for round_number in 1.. {
                    let mut beginning = [0_u8; PROBED_REQUEST_BEGINNING_IN_BYTES];
                    if requests_for_the_counterpart
                        .read_exact(&mut beginning)
                        .is_err()
                    {
                        return;
                    }
                    assert_eq!(
                        beginning.as_slice(),
                        expected_beginning,
                        "第 {round_number} 问的开头"
                    );
                    progress.request_begun.store(round_number, Ordering::SeqCst);
                    spin_until(|| progress.draining_allowed.load(Ordering::SeqCst) >= round_number);
                    let mut rest = vec![0_u8; expected_rest.len()];
                    requests_for_the_counterpart
                        .read_exact(&mut rest)
                        .expect("请求管道读得了");
                    assert!(
                        rest == expected_rest,
                        "第 {round_number} 问整条读全，与预期逐字节相同"
                    );
                    progress
                        .request_complete
                        .store(round_number, Ordering::SeqCst);
                    for reply_part in &reply_parts {
                        let part_number = progress.reply_parts_written.load(Ordering::SeqCst) + 1;
                        spin_until(|| {
                            progress.reply_parts_allowed.load(Ordering::SeqCst) >= part_number
                        });
                        progress
                            .reply_parts_written
                            .store(part_number, Ordering::SeqCst);
                        reply_end_of_the_counterpart
                            .write_all(reply_part)
                            .expect("回应管道写得进");
                    }
                }
            })
        };
        let (orders, orders_received) = std::sync::mpsc::channel::<()>();
        let (answers, answers_received) = std::sync::mpsc::channel::<Option<String>>();
        let device_thread = {
            let device_library = device_library.clone();
            std::thread::spawn(move || {
                for () in orders_received {
                    answers
                        .send(ask(&device_library))
                        .expect("本线程等着收口子交回的");
                }
            })
        };
        let Backend::Relay(relay_lock) = &*device_library.backend else {
            panic!("探锁只造转交那一种");
        };
        let mut round_where_the_lock_was_free = None;
        for round_number in 1..=ROUNDS {
            let reply_parts_before_this_round = (round_number - 1) * reply_parts_per_round;
            let reply_parts_through_this_round = round_number * reply_parts_per_round;
            let held_while_the_device_thread_queues = relay_lock
                .lock()
                .expect("这把锁只在本测试的三个线程之间传，没有线程握着它 panic");
            orders.send(()).expect("装置线程等着收令");
            std::thread::sleep(QUEUEING_TIME);
            drop(held_while_the_device_thread_queues);
            spin_until(|| progress.request_begun.load(Ordering::SeqCst) >= round_number);
            progress
                .draining_allowed
                .store(round_number, Ordering::SeqCst);
            let mut step_seen_at = None;
            let lock_was_free_in_the_middle = loop {
                match relay_lock.try_lock() {
                    Ok(probe) => {
                        let not_yet_fully_replied =
                            progress.reply_parts_written.load(Ordering::SeqCst)
                                < reply_parts_through_this_round;
                        drop(probe);
                        break not_yet_fully_replied;
                    }
                    Err(std::sync::TryLockError::WouldBlock) => {
                        let reply_parts_allowed =
                            progress.reply_parts_allowed.load(Ordering::SeqCst);
                        let is_the_counterpart_at_the_next_gate =
                            if reply_parts_allowed == reply_parts_before_this_round {
                                progress.request_complete.load(Ordering::SeqCst) >= round_number
                            } else {
                                progress.reply_parts_written.load(Ordering::SeqCst)
                                    >= reply_parts_allowed
                            };
                        if reply_parts_allowed < reply_parts_through_this_round
                            && is_the_counterpart_at_the_next_gate
                        {
                            let seen_at = *step_seen_at.get_or_insert_with(std::time::Instant::now);
                            if seen_at.elapsed() >= GRACE_AFTER_EACH_STEP {
                                progress
                                    .reply_parts_allowed
                                    .store(reply_parts_allowed + 1, Ordering::SeqCst);
                                step_seen_at = None;
                            }
                        }
                    }
                    Err(std::sync::TryLockError::Poisoned(_)) => {
                        panic!("这把锁只在本测试的三个线程之间传，没有线程握着它 panic")
                    }
                }
            };
            progress
                .reply_parts_allowed
                .store(reply_parts_through_this_round, Ordering::SeqCst);
            assert_eq!(
                answers_received.recv().expect("装置线程交回口子交回的"),
                expected_answer,
                "第 {round_number} 轮口子照替身的回应交回"
            );
            if lock_was_free_in_the_middle {
                round_where_the_lock_was_free = Some(round_number);
                break;
            }
        }
        drop(orders);
        device_thread.join().expect("装置线程照常收尾");
        drop(device_library);
        counterpart
            .join()
            .expect("替身线程核过每一问整条与预期相同、读到请求管道的文件尾收尾");
        round_where_the_lock_was_free
    }

    #[test]
    fn read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
        let key = format!("items/{}", "k".repeat(PROBED_REQUEST_PAYLOAD_IN_BYTES));
        let expected_request = format!("read {key}\n").into_bytes();
        assert_eq!(
            round_where_the_lock_was_free_in_the_middle_of_the_exchange(ProbedExchange {
                ask: Box::new(move |device_library| device_library.read(&key)),
                expected_request,
                reply_parts: vec![b"absent\n".as_slice()],
                expected_answer: None,
            }),
            None,
            "探锁的一方抢到锁时替身已读全这一问、还没回：锁在请求与回应之间放过了"
        );
    }

    #[test]
    fn read_answered_found_holds_the_relay_lock_until_taking_the_contents() {
        let key = format!("items/{}", "k".repeat(PROBED_REQUEST_PAYLOAD_IN_BYTES));
        let expected_request = format!("read {key}\n").into_bytes();
        assert_eq!(
            round_where_the_lock_was_free_in_the_middle_of_the_exchange(ProbedExchange {
                ask: Box::new(move |device_library| device_library.read(&key)),
                expected_request,
                reply_parts: vec![b"found 5\n".as_slice(), b"hello".as_slice()],
                expected_answer: Some("hello".to_string()),
            }),
            None,
            "探锁的一方抢到锁时替身回了 found 那一行、还没写内容：锁在 found 那一行与内容之间放过了"
        );
    }

    // 相对键与内容都比管道缓冲大：请求头（write、相对键与字节数那一行）自己就要等替身读，装置线程写完请求头时
    // 探锁的一方已在空转，锁在请求头与内容之间放过也抢得到；内容那一段照样罩住请求与 written 之间。
    #[test]
    fn write_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
        let key = format!("items/{}", "k".repeat(PROBED_REQUEST_PAYLOAD_IN_BYTES));
        let text = "v".repeat(PROBED_REQUEST_PAYLOAD_IN_BYTES);
        let expected_request =
            format!("write {key} {PROBED_REQUEST_PAYLOAD_IN_BYTES}\n{text}").into_bytes();
        assert_eq!(
            round_where_the_lock_was_free_in_the_middle_of_the_exchange(ProbedExchange {
                ask: Box::new(move |device_library| {
                    device_library.write(&key, &text);
                    None
                }),
                expected_request,
                reply_parts: vec![b"written\n".as_slice()],
                expected_answer: None,
            }),
            None,
            "探锁的一方抢到锁时替身还没回 written：锁在请求头与内容之间、或写请求与 written 之间放过了"
        );
    }

    #[test]
    fn write_many_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
        let text = "v".repeat(PROBED_REQUEST_PAYLOAD_IN_BYTES);
        let expected_request =
            format!("write-many 1\nitems/a.out {PROBED_REQUEST_PAYLOAD_IN_BYTES}\n{text}")
                .into_bytes();
        assert_eq!(
            round_where_the_lock_was_free_in_the_middle_of_the_exchange(ProbedExchange {
                ask: Box::new(move |device_library| {
                    device_library.write_many(&[("items/a.out".to_string(), text.clone())]);
                    None
                }),
                expected_request,
                reply_parts: vec![b"written\n".as_slice()],
                expected_answer: None,
            }),
            None,
            "探锁的一方抢到锁时替身已读全这一问、还没回：锁在成批写请求与 written 之间放过了"
        );
    }
}
