// 实验装置读写判定库的口子：装置不开本地库目录、不连服务，读写都经起它的 scripts/experiment-apparatus/device-code-key.py run
// 转给 prune-server 的那一份判定库（.claude/rules/verification.md「判定库只有一份」；转交的一侧写在 device-code-key.py 文件头「判定库的转交」）。
// device-code-key.py 起装置时开两根管道，把装置这一头的描述符号放进环境变量 DEVICE_LIBRARY_REQUEST_FD（装置写请求）与
// DEVICE_LIBRARY_REPLY_FD（装置读回应）；键是相对这个装置那一种的路径（例 items/<项键>.out、codes/<代码键>.txt、index.tsv），族与种类由那一侧定。
// 协议一问一答，几个线程共用一条，整问整答在一把锁里：
//   请求 read <相对键>\n                    回 found <字节数>\n<内容>，或 absent\n
//   请求 write <相对键> <字节数>\n<内容>    回 written\n（服务确认落盘之后才回）
//   两种都可能回 failed <退出码> <一句>\n：这里打 ✗ 与那一句、进程照那个退出码退（75 是 prune-server 不可用，3 是别的错），不当「键不在」现判。
//   相对键只用字母、数字与 . _ - /，不带空白（那一侧照这一条核，不合的回 failed 3）。管道断了、回应认不出退 3。
// 撤掉的项（那一侧核出编译与跑之间源码变了时改写成撤回标记的）由那一侧回 absent，这里看不到标记。
// 每个用到它的装置写 `#[path = "device_library/relay.rs"] mod device_library;` 再 `use device_library::DeviceLibrary;`（单元闭包的键不收条目级 include!）。
// 单测换成 `DeviceLibrary::in_memory()`（只在单测里编进来的内存那一种，测完即丢，不落盘）；转交的整条路由 device-code-key.py --selftest 在临时服务底下核。

use std::io::{BufRead, Read, Write};

/// 装置写请求的那根管道（device-code-key.py run 放进环境的描述符号）。
const REQUEST_DESCRIPTOR_VARIABLE: &str = "DEVICE_LIBRARY_REQUEST_FD";
/// 装置读回应的那根管道。
const REPLY_DESCRIPTOR_VARIABLE: &str = "DEVICE_LIBRARY_REPLY_FD";
/// 管道断了、回应认不出时的退出码（与 prune-server 客户端「别的错」同一个）。
const RELAY_BROKEN_EXIT_CODE: i32 = 3;

struct RelayEnds {
    requests: std::fs::File,
    replies: std::io::BufReader<std::fs::File>,
}

enum Backend {
    Relay(std::sync::Mutex<RelayEnds>),
    #[cfg(test)]
    Memory(std::sync::Mutex<std::collections::BTreeMap<String, String>>),
}

/// 一个装置这一趟的判定库口子：克隆的几份共用同一条管道（判等按是不是同一条）。
#[derive(Clone)]
pub(crate) struct DeviceLibrary {
    backend: std::sync::Arc<Backend>,
}

impl std::fmt::Debug for DeviceLibrary {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match *self.backend {
            Backend::Relay(_) => "DeviceLibrary(经 device-code-key.py 转给 prune-server)",
            #[cfg(test)]
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

/// 读写没成：打 ✗ 与出路，照那个退出码退（文件头「协议」）。
fn give_up(exit_code: i32, sentence: &str) -> ! {
    eprintln!("  ✗ 判定库读写没成：{sentence}");
    eprintln!(
        "     → 怎么办：退 75 是 prune-server 不可用，python3 prune-server/control check 看服务，修好之后由交它的会话重交同一条命令；别的码照这一句修"
    );
    std::process::exit(exit_code)
}

/// 读一行回应（去掉换行）；管道断了退 3。
fn reply_line(replies: &mut std::io::BufReader<std::fs::File>) -> String {
    let mut line = String::new();
    match replies.read_line(&mut line) {
        Ok(0) => give_up(RELAY_BROKEN_EXIT_CODE, "转交判定库的管道断了（起这个装置的 device-code-key.py 没了）"),
        Ok(_) => line.trim_end_matches('\n').to_string(),
        Err(error) => give_up(RELAY_BROKEN_EXIT_CODE, &format!("读不了转交判定库的管道：{error}")),
    }
}

/// `failed <退出码> <一句>` 那一种回应：照它退；别的认不出的回应退 3。
fn give_up_on_reply(reply: &str, asked: &str) -> ! {
    if let Some(rest) = reply.strip_prefix("failed ") {
        let (code_text, sentence) = rest.split_once(' ').unwrap_or((rest, ""));
        let exit_code = code_text.parse::<i32>().unwrap_or(RELAY_BROKEN_EXIT_CODE);
        give_up(exit_code, &format!("{asked}：{sentence}"));
    }
    give_up(RELAY_BROKEN_EXIT_CODE, &format!("{asked}：转交判定库的回应认不出「{reply}」"))
}

impl DeviceLibrary {
    /// 照 device-code-key.py run 放进环境的两个描述符号接上转交的管道；没有、写错或打不开交一句为什么（调用方照用法错办）。
    pub(crate) fn from_environment() -> Result<DeviceLibrary, String> {
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
        let requests = std::fs::OpenOptions::new().write(true).open(&request_path).map_err(|error| format!("打不开转交判定库的请求管道 {request_path}：{error}"))?;
        let replies = std::fs::File::open(&reply_path).map_err(|error| format!("打不开转交判定库的回应管道 {reply_path}：{error}"))?;
        Ok(DeviceLibrary {
            backend: std::sync::Arc::new(Backend::Relay(std::sync::Mutex::new(RelayEnds { requests, replies: std::io::BufReader::new(replies) }))),
        })
    }

    /// 单测用的内存那一种：一份测试一份，测完即丢。
    #[cfg(test)]
    pub(crate) fn in_memory() -> DeviceLibrary {
        DeviceLibrary { backend: std::sync::Arc::new(Backend::Memory(std::sync::Mutex::new(std::collections::BTreeMap::new()))) }
    }

    /// 读一条：不在（或已撤掉）交 None；读失败照退出码退，不交 None。
    pub(crate) fn read(&self, relative: &str) -> Option<String> {
        match &*self.backend {
            Backend::Relay(ends) => {
                let mut ends = ends.lock().expect("转交判定库的锁没有中毒");
                let asked = format!("读 {relative}");
                if let Err(error) = ends.requests.write_all(format!("read {relative}\n").as_bytes()) {
                    give_up(RELAY_BROKEN_EXIT_CODE, &format!("{asked}：写不进转交判定库的管道：{error}"));
                }
                let reply = reply_line(&mut ends.replies);
                if reply == "absent" {
                    return None;
                }
                let Some(length) = reply.strip_prefix("found ").and_then(|text| text.parse::<usize>().ok()) else { give_up_on_reply(&reply, &asked) };
                let mut contents = vec![0_u8; length];
                if let Err(error) = ends.replies.read_exact(&mut contents) {
                    give_up(RELAY_BROKEN_EXIT_CODE, &format!("{asked}：回应里的 {length} 字节没读全：{error}"));
                }
                String::from_utf8(contents).ok()
            }
            #[cfg(test)]
            Backend::Memory(entries) => entries.lock().expect("单测内存库的锁没有中毒").get(relative).cloned(),
        }
    }

    /// 写一条（服务确认落盘之后才回来）；写失败照退出码退。
    pub(crate) fn write(&self, relative: &str, text: &str) {
        match &*self.backend {
            Backend::Relay(ends) => {
                let mut ends = ends.lock().expect("转交判定库的锁没有中毒");
                let asked = format!("写 {relative}");
                let mut request = format!("write {relative} {}\n", text.len()).into_bytes();
                request.extend_from_slice(text.as_bytes());
                if let Err(error) = ends.requests.write_all(&request) {
                    give_up(RELAY_BROKEN_EXIT_CODE, &format!("{asked}：写不进转交判定库的管道：{error}"));
                }
                let reply = reply_line(&mut ends.replies);
                if reply != "written" {
                    give_up_on_reply(&reply, &asked);
                }
            }
            #[cfg(test)]
            Backend::Memory(entries) => {
                entries.lock().expect("单测内存库的锁没有中毒").insert(relative.to_string(), text.to_string());
            }
        }
    }

    /// 单测数条数：以 prefix 起头、以 suffix 收尾的键有几条。
    #[cfg(test)]
    pub(crate) fn count_entries(&self, prefix: &str, suffix: &str) -> usize {
        match &*self.backend {
            Backend::Relay(_) => panic!("单测只数内存那一种"),
            Backend::Memory(entries) => {
                entries.lock().expect("单测内存库的锁没有中毒").keys().filter(|key| key.starts_with(prefix) && key.ends_with(suffix)).count()
            }
        }
    }
}
