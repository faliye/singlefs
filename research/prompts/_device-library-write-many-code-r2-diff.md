# device-library-write-many-code-r2：r1 判决之后的修补（2026-11-10）

## 一、`crates/singlefs-device-library/src/lib.rs`：r1 开工快照那一版 → 此刻

来源：`diff -u <r1 快照那一版（实现员补丁的 baseline）> <此刻主工作区>`。

```diff
--- crates/singlefs-device-library/src/lib.rs
+++ crates/singlefs-device-library/src/lib.rs
@@ -2,13 +2,16 @@
 // 转给 prune-server 的那一份判定库（.claude/rules/verification.md「判定库只有一份」；转交的一侧写在 device-code-key.py 文件头「判定库的转交」）。
 // device-code-key.py 起装置时开两根管道，把装置这一头的描述符号放进环境变量 DEVICE_LIBRARY_REQUEST_FD（装置写请求）与
 // DEVICE_LIBRARY_REPLY_FD（装置读回应）；键是相对这个装置那一种的路径（例 items/<项键>.out、codes/<代码键>.txt、index.tsv），族与种类由那一侧定。
-// 协议一问一答，几个线程共用一条，整问整答在一把锁里：
+// 协议一问一答，几个线程共用一条，整问整答在一把锁里（从写请求到读完回应一直握着，别的线程只等这一问）：
 //   请求 read <相对键>\n                    回 found <字节数>\n<内容>，或 absent\n
 //   请求 write <相对键> <字节数>\n<内容>    回 written\n（服务确认落盘之后才回）
-//   请求 write-many <条数>\n 后接每条 <相对键> <字节数>\n<内容>    整批回一个 written\n（服务确认整批落盘之后才回）：
-//     一条请求里的写在服务那一头合成一次提交，整问整答在锁里，别的线程只等这一条；条数至少 1。
+//   请求 write-many <条数>\n 后接每条 <相对键> <字节数>\n<内容>    整批回一个 written\n；条数至少 1。
+//     整批经 prune-server 客户端写服务：一帧（8192 条或 16 MiB）以内是一次提交，超过切成几帧各自提交（原子性只在一帧之内）；
+//     整批的 written 在全部帧都确认落盘之后才回。
 //   三种都可能回 failed <退出码> <一句>\n：这里打 ✗ 与那一句、进程照那个退出码退（75 是 prune-server 不可用，3 是别的错），不当「键不在」现判。
-//   相对键只用字母、数字与 . _ - /，不带空白（那一侧照这一条核，不合的回 failed 3，整批一条都不写）。管道断了、回应认不出退 3。
+//   相对键只用 ASCII 字母、数字与 . _ - /，不空、不带空白（与 device-code-key.py 的 RELATIVE_KEY_PATTERN 同一条）：write 与 write-many 在拿锁、
+//   拼请求之前先在这一侧核（转交与内存两种都核），不合的照「别的错」退 3，管道上一个字节都不发；那一侧照同一条再核，不合的回 failed 3。
+//   管道断了、回应认不出退 3。
 // 撤掉的项（那一侧核出编译与跑之间源码变了时改写成撤回标记的）由那一侧回 absent，这里看不到标记。
 // 每个用到它的装置在自己的 Cargo.toml 里按 path 依赖引本包（crates/singlefs-device-library），再 `use singlefs_device_library::DeviceLibrary;`
 // （从仓根算单元键时 unit-graph 看不到 `#[path]` 引进来的文件，键拒算；条目级 include! 同样不收）。
@@ -22,8 +25,12 @@
 const REQUEST_DESCRIPTOR_VARIABLE: &str = "DEVICE_LIBRARY_REQUEST_FD";
 /// 装置读回应的那根管道。
 const REPLY_DESCRIPTOR_VARIABLE: &str = "DEVICE_LIBRARY_REPLY_FD";
-/// 管道断了、回应认不出时的退出码（与 prune-server 客户端「别的错」同一个）。
-const RELAY_BROKEN_EXIT_CODE: i32 = 3;
+/// 「别的错」的退出码（与 prune-server 客户端「别的错」同一个）：管道断了、回应认不出、相对键不合都照它退。
+const OTHER_ERROR_EXIT_CODE: i32 = 3;
+/// 转交那一种的锁为什么不会中毒：持锁时做的几样都不 panic。
+const RELAY_LOCK_IS_NEVER_POISONED: &str = "转交判定库的锁不会中毒：持锁时只做拼请求、管道读写、按回应报的字节数分配（报的数超过 isize::MAX 先当认不出退掉，分配失败是中止进程、不展开）与 give_up（写标准错误不看成败、退进程不展开），这几样都不 panic";
+/// 内存那一种的锁为什么不会中毒：持锁时做的几样都不 panic。
+const MEMORY_LOCK_IS_NEVER_POISONED: &str = "单测内存库的锁不会中毒：持锁时只做 BTreeMap 的插入、查找、数键与成批写次数加一（usize 数不满），这几样都不 panic";
 
 struct RelayEnds {
     requests: std::fs::File,
@@ -65,26 +72,49 @@
 
 impl Eq for DeviceLibrary {}
 
-/// 读写没成：打 ✗ 与出路，照那个退出码退（文件头「协议」）。
+/// 读写没成：打 ✗ 与出路，照那个退出码退（文件头「协议」）。持锁时也调它，所以不许 panic：
+/// 标准错误写不进（读它的那一头没了）也照退，不用 `eprintln!`（它写不进就 panic）。
 fn give_up(exit_code: i32, sentence: &str) -> ! {
-    eprintln!("  ✗ 判定库读写没成：{sentence}");
-    eprintln!(
-        "     → 怎么办：退 75 是 prune-server 不可用，python3 prune-server/control check 看服务，修好之后由交它的会话重交同一条命令；别的码照这一句修"
-    );
+    {
+        let mut standard_error = std::io::stderr().lock();
+        let _ = writeln!(standard_error, "  ✗ 判定库读写没成：{sentence}");
+        let _ = writeln!(
+            standard_error,
+            "     → 怎么办：退 75 是 prune-server 不可用，python3 prune-server/control check 看服务，修好之后由交它的会话重交同一条命令；别的码照这一句修"
+        );
+    }
     std::process::exit(exit_code)
 }
 
+/// 转交那一侧收的相对键：不空，只用 ASCII 字母、数字与 . _ - /（device-code-key.py 的 RELATIVE_KEY_PATTERN `[A-Za-z0-9._/-]+` 整串匹配）。
+fn is_relative_key_the_relay_accepts(relative: &str) -> bool {
+    !relative.is_empty()
+        && relative
+            .bytes()
+            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/'))
+}
+
+/// 相对键不合就照「别的错」退；调用方在拿锁、拼请求之前调，管道上一个字节都不发。
+fn give_up_unless_the_relay_accepts_the_key(relative: &str, asked: &str) {
+    if !is_relative_key_the_relay_accepts(relative) {
+        give_up(
+            OTHER_ERROR_EXIT_CODE,
+            &format!("{asked}：相对键「{relative}」不合（只许 ASCII 字母、数字与 . _ - /，不空、不带空白）"),
+        );
+    }
+}
+
 /// 读一行回应（去掉换行）；管道断了退 3。
 fn reply_line(replies: &mut std::io::BufReader<std::fs::File>) -> String {
     let mut line = String::new();
     match replies.read_line(&mut line) {
         Ok(0) => give_up(
-            RELAY_BROKEN_EXIT_CODE,
+            OTHER_ERROR_EXIT_CODE,
             "转交判定库的管道断了（起这个装置的 device-code-key.py 没了）",
         ),
         Ok(_) => line.trim_end_matches('\n').to_string(),
         Err(error) => give_up(
-            RELAY_BROKEN_EXIT_CODE,
+            OTHER_ERROR_EXIT_CODE,
             &format!("读不了转交判定库的管道：{error}"),
         ),
     }
@@ -94,11 +124,11 @@
 fn give_up_on_reply(reply: &str, asked: &str) -> ! {
     if let Some(rest) = reply.strip_prefix("failed ") {
         let (code_text, sentence) = rest.split_once(' ').unwrap_or((rest, ""));
-        let exit_code = code_text.parse::<i32>().unwrap_or(RELAY_BROKEN_EXIT_CODE);
+        let exit_code = code_text.parse::<i32>().unwrap_or(OTHER_ERROR_EXIT_CODE);
         give_up(exit_code, &format!("{asked}：{sentence}"));
     }
     give_up(
-        RELAY_BROKEN_EXIT_CODE,
+        OTHER_ERROR_EXIT_CODE,
         &format!("{asked}：转交判定库的回应认不出「{reply}」"),
     )
 }
@@ -107,7 +137,7 @@
 fn send_write_request_and_wait_for_written(ends: &mut RelayEnds, request: &[u8], asked: &str) {
     if let Err(error) = ends.requests.write_all(request) {
         give_up(
-            RELAY_BROKEN_EXIT_CODE,
+            OTHER_ERROR_EXIT_CODE,
             &format!("{asked}：写不进转交判定库的管道：{error}"),
         );
     }
@@ -158,14 +188,14 @@
     pub fn read(&self, relative: &str) -> Option<String> {
         match &*self.backend {
             Backend::Relay(ends) => {
-                let mut ends = ends.lock().expect("转交判定库的锁没有中毒");
+                let mut ends = ends.lock().expect(RELAY_LOCK_IS_NEVER_POISONED);
                 let asked = format!("读 {relative}");
                 if let Err(error) = ends
                     .requests
                     .write_all(format!("read {relative}\n").as_bytes())
                 {
                     give_up(
-                        RELAY_BROKEN_EXIT_CODE,
+                        OTHER_ERROR_EXIT_CODE,
                         &format!("{asked}：写不进转交判定库的管道：{error}"),
                     );
                 }
@@ -173,16 +203,18 @@
                 if reply == "absent" {
                     return None;
                 }
+                // 报的字节数超过 isize::MAX 时 vec! 容量溢出会 panic、在锁里毒掉锁：当认不出的回应退掉。
                 let Some(length) = reply
                     .strip_prefix("found ")
                     .and_then(|text| text.parse::<usize>().ok())
+                    .filter(|length| isize::try_from(*length).is_ok())
                 else {
                     give_up_on_reply(&reply, &asked)
                 };
                 let mut contents = vec![0_u8; length];
                 if let Err(error) = ends.replies.read_exact(&mut contents) {
                     give_up(
-                        RELAY_BROKEN_EXIT_CODE,
+                        OTHER_ERROR_EXIT_CODE,
                         &format!("{asked}：回应里的 {length} 字节没读全：{error}"),
                     );
                 }
@@ -190,19 +222,20 @@
             }
             Backend::Memory(library) => library
                 .lock()
-                .expect("单测内存库的锁没有中毒")
+                .expect(MEMORY_LOCK_IS_NEVER_POISONED)
                 .entries
                 .get(relative)
                 .cloned(),
         }
     }
 
-    /// 写一条（服务确认落盘之后才回来）；写失败照退出码退。
+    /// 写一条（服务确认落盘之后才回来）；相对键不合、写失败照退出码退。
     pub fn write(&self, relative: &str, text: &str) {
+        let asked = format!("写 {relative}");
+        give_up_unless_the_relay_accepts_the_key(relative, &asked);
         match &*self.backend {
             Backend::Relay(ends) => {
-                let mut ends = ends.lock().expect("转交判定库的锁没有中毒");
-                let asked = format!("写 {relative}");
+                let mut ends = ends.lock().expect(RELAY_LOCK_IS_NEVER_POISONED);
                 let mut request = format!("write {relative} {}\n", text.len()).into_bytes();
                 request.extend_from_slice(text.as_bytes());
                 send_write_request_and_wait_for_written(&mut ends, &request, &asked);
@@ -210,22 +243,25 @@
             Backend::Memory(library) => {
                 library
                     .lock()
-                    .expect("单测内存库的锁没有中毒")
+                    .expect(MEMORY_LOCK_IS_NEVER_POISONED)
                     .entries
                     .insert(relative.to_string(), text.to_string());
             }
         }
     }
 
-    /// 一次写一批（相对键, 内容），服务确认整批落盘之后才回来；写失败照退出码退。空的一批不发请求。
+    /// 一次写一批（相对键, 内容），服务确认整批落盘之后才回来；有一条相对键不合整批一条都不发、照退出码退，写失败照退出码退。空的一批不发请求。
     pub fn write_many(&self, entries: &[(String, String)]) {
         if entries.is_empty() {
             return;
         }
+        let asked = format!("成批写 {} 条", entries.len());
+        for (relative, _text) in entries {
+            give_up_unless_the_relay_accepts_the_key(relative, &asked);
+        }
         match &*self.backend {
             Backend::Relay(ends) => {
-                let mut ends = ends.lock().expect("转交判定库的锁没有中毒");
-                let asked = format!("成批写 {} 条", entries.len());
+                let mut ends = ends.lock().expect(RELAY_LOCK_IS_NEVER_POISONED);
                 let mut request = format!("write-many {}\n", entries.len()).into_bytes();
                 for (relative, text) in entries {
                     request.extend_from_slice(format!("{relative} {}\n", text.len()).as_bytes());
@@ -234,7 +270,7 @@
                 send_write_request_and_wait_for_written(&mut ends, &request, &asked);
             }
             Backend::Memory(library) => {
-                let mut library = library.lock().expect("单测内存库的锁没有中毒");
+                let mut library = library.lock().expect(MEMORY_LOCK_IS_NEVER_POISONED);
                 library.write_many_calls += 1;
                 for (relative, text) in entries {
                     library.entries.insert(relative.clone(), text.clone());
@@ -249,7 +285,7 @@
             Backend::Relay(_) => panic!("单测只数内存那一种"),
             Backend::Memory(library) => library
                 .lock()
-                .expect("单测内存库的锁没有中毒")
+                .expect(MEMORY_LOCK_IS_NEVER_POISONED)
                 .entries
                 .keys()
                 .filter(|key| key.starts_with(prefix) && key.ends_with(suffix))
@@ -264,7 +300,7 @@
             Backend::Memory(library) => {
                 library
                     .lock()
-                    .expect("单测内存库的锁没有中毒")
+                    .expect(MEMORY_LOCK_IS_NEVER_POISONED)
                     .write_many_calls
             }
         }
@@ -273,8 +309,9 @@
 
 #[cfg(test)]
 mod tests {
-    use super::{Backend, DeviceLibrary, RelayEnds};
-    use std::io::{Read, Write};
+    use super::{is_relative_key_the_relay_accepts, Backend, DeviceLibrary, RelayEnds};
+    use std::io::{BufRead, Read, Write};
+    use std::sync::atomic::{AtomicUsize, Ordering};
 
     /// 转交那一侧在测试里的替身：装置写进去的请求从 `requests_written_by_the_device` 读出来，
     /// 预先写好的回应放在装置读的那一根管道里（写端随即关上）。
@@ -283,26 +320,34 @@
         requests_written_by_the_device: std::io::PipeReader,
     }
 
-    fn relay_with_prepared_replies(prepared_replies: &[u8]) -> RelayCounterpart {
-        let (requests_written_by_the_device, request_end_of_the_device) =
-            std::io::pipe().expect("建请求管道");
+    /// 转交那一种的口子：请求写进 `requests`，回应是预先写好的那几行（回应管道的写端随即关上，读完就是文件尾）。
+    fn relay_writing_requests_to(
+        requests: std::fs::File,
+        prepared_replies: &[u8],
+    ) -> DeviceLibrary {
         let (reply_end_of_the_device, mut reply_end_of_the_counterpart) =
             std::io::pipe().expect("建回应管道");
         reply_end_of_the_counterpart
             .write_all(prepared_replies)
             .expect("预先写好的回应比管道缓冲小，写得进");
         drop(reply_end_of_the_counterpart);
-        let requests = std::fs::File::from(std::os::fd::OwnedFd::from(request_end_of_the_device));
         let replies = std::io::BufReader::new(std::fs::File::from(std::os::fd::OwnedFd::from(
             reply_end_of_the_device,
         )));
+        DeviceLibrary {
+            backend: std::sync::Arc::new(Backend::Relay(std::sync::Mutex::new(RelayEnds {
+                requests,
+                replies,
+            }))),
+        }
+    }
+
+    fn relay_with_prepared_replies(prepared_replies: &[u8]) -> RelayCounterpart {
+        let (requests_written_by_the_device, request_end_of_the_device) =
+            std::io::pipe().expect("建请求管道");
+        let requests = std::fs::File::from(std::os::fd::OwnedFd::from(request_end_of_the_device));
         RelayCounterpart {
-            device_library: DeviceLibrary {
-                backend: std::sync::Arc::new(Backend::Relay(std::sync::Mutex::new(RelayEnds {
-                    requests,
-                    replies,
-                }))),
-            },
+            device_library: relay_writing_requests_to(requests, prepared_replies),
             requests_written_by_the_device,
         }
     }
@@ -432,4 +477,423 @@
             "非空的一批记一次，空的一批与单条写不记"
         );
     }
+
+    #[test]
+    fn relative_keys_the_relay_accepts_are_non_empty_ascii_letters_digits_and_dot_underscore_dash_slash(
+    ) {
+        for accepted in ["items/a-b_c.out", "codes/0.txt", "index.tsv", "A/Z/9"] {
+            assert!(
+                is_relative_key_the_relay_accepts(accepted),
+                "「{accepted}」只用字母、数字与 . _ - /，收"
+            );
+        }
+        for refused in [
+            "",
+            "items/bad key.out",
+            "items/tab\tkey.out",
+            "items/new\nline.out",
+            "items/é.out",
+            "items/semicolon;.out",
+        ] {
+            assert!(
+                !is_relative_key_the_relay_accepts(refused),
+                "「{refused}」是空串、带空白或带别的字符，不收"
+            );
+        }
+    }
+
+    // ── 退进程的几条：口子照退出码退进程（give_up 调 std::process::exit），同一个进程里测不了，
+    //    照测试名再起一次本测试二进制、只跑那一条，看它的退出码与它往请求管道写过的字节。
+
+    /// 子进程那一半的标记：设了它，点名的测试只做它那一半（调一次口子），口子没退进程就退 0。
+    const CHILD_HALF_VARIABLE: &str = "SINGLEFS_DEVICE_LIBRARY_TEST_CHILD_HALF";
+
+    fn is_the_child_half() -> bool {
+        std::env::var_os(CHILD_HALF_VARIABLE).is_some()
+    }
+
+    /// 子进程的标准错误接到哪里。
+    enum ChildStandardError {
+        /// 接一根管道，读出来写进断言的消息里。
+        Captured,
+        /// 接一根读的那一头起子进程之前就关掉的管道：子进程往标准错误写都写不进。
+        ClosedBeforeTheChildWrites,
+    }
+
+    /// 子进程那一半跑完的样子：退出码（被信号杀掉是 None）、它往请求管道写过的全部字节、它打到标准错误的字。
+    struct ChildHalfOutcome {
+        exit_code: Option<i32>,
+        requests_sent: Vec<u8>,
+        standard_error: String,
+    }
+
+    /// 照测试名（`tests::<函数名>`）再起一次本测试二进制、只跑那一条的子进程那一半。子进程的标准输入接一根管道的写端，
+    /// 子进程把它当请求那一头（`relay_sending_requests_to_the_standard_input`），这里读出它写过的字节。
+    fn run_the_child_half(test_name: &str, standard_error: ChildStandardError) -> ChildHalfOutcome {
+        let (mut requests_sent_by_the_child, request_end_for_the_child) =
+            std::io::pipe().expect("建请求管道");
+        let mut command =
+            std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"));
+        command
+            .args([test_name, "--exact", "--nocapture"])
+            .env(CHILD_HALF_VARIABLE, "1")
+            .stdin(request_end_for_the_child)
+            .stdout(std::process::Stdio::null());
+        match standard_error {
+            ChildStandardError::Captured => {
+                command.stderr(std::process::Stdio::piped());
+            }
+            ChildStandardError::ClosedBeforeTheChildWrites => {
+                let (reader_closed_at_once, standard_error_of_the_child) =
+                    std::io::pipe().expect("建标准错误管道");
+                drop(reader_closed_at_once);
+                command.stderr(standard_error_of_the_child);
+            }
+        }
+        let output = command.output().expect("起得了子进程");
+        // 请求管道写端的这一份在 command 里，丢掉它之后读得到文件尾。
+        drop(command);
+        let mut requests_sent = Vec::new();
+        requests_sent_by_the_child
+            .read_to_end(&mut requests_sent)
+            .expect("请求管道读得到文件尾");
+        ChildHalfOutcome {
+            exit_code: output.status.code(),
+            requests_sent,
+            standard_error: String::from_utf8_lossy(&output.stderr).into_owned(),
+        }
+    }
+
+    /// 子进程那一半的转交口子：请求写进标准输入（父进程接的是一根管道的写端），回应是预先写好的那几行。
+    fn relay_sending_requests_to_the_standard_input(prepared_replies: &[u8]) -> DeviceLibrary {
+        use std::os::fd::AsFd;
+        let requests = std::fs::File::from(
+            std::io::stdin()
+                .as_fd()
+                .try_clone_to_owned()
+                .expect("标准输入的描述符复制得了"),
+        );
+        relay_writing_requests_to(requests, prepared_replies)
+    }
+
+    #[test]
+    fn write_with_a_key_the_relay_refuses_exits_with_the_other_error_code_before_sending_a_byte() {
+        if is_the_child_half() {
+            relay_sending_requests_to_the_standard_input(b"written\n")
+                .write("items/bad key.out", "x");
+            std::process::exit(0);
+        }
+        let outcome = run_the_child_half(
+            "tests::write_with_a_key_the_relay_refuses_exits_with_the_other_error_code_before_sending_a_byte",
+            ChildStandardError::Captured,
+        );
+        assert_eq!(
+            outcome.exit_code,
+            Some(3),
+            "带空格的相对键照「别的错」退 3，不当写成了；标准错误：{}",
+            outcome.standard_error
+        );
+        assert_eq!(
+            String::from_utf8_lossy(&outcome.requests_sent),
+            "",
+            "相对键不合，管道上一个字节都不发"
+        );
+    }
+
+    #[test]
+    fn write_many_with_a_key_the_relay_refuses_exits_with_the_other_error_code_before_sending_a_byte(
+    ) {
+        if is_the_child_half() {
+            relay_sending_requests_to_the_standard_input(b"written\n").write_many(&[
+                ("items/good.out".to_string(), "x".to_string()),
+                ("items/bad key.out".to_string(), "y".to_string()),
+            ]);
+            std::process::exit(0);
+        }
+        let outcome = run_the_child_half(
+            "tests::write_many_with_a_key_the_relay_refuses_exits_with_the_other_error_code_before_sending_a_byte",
+            ChildStandardError::Captured,
+        );
+        assert_eq!(
+            outcome.exit_code,
+            Some(3),
+            "一批里有一条相对键带空格，整批照「别的错」退 3；标准错误：{}",
+            outcome.standard_error
+        );
+        assert_eq!(
+            String::from_utf8_lossy(&outcome.requests_sent),
+            "",
+            "一批里有一条相对键不合，前面合的那一条也不发，管道上一个字节都不发"
+        );
+    }
+
+    #[test]
+    fn in_memory_write_many_with_a_key_the_relay_refuses_exits_with_the_other_error_code() {
+        if is_the_child_half() {
+            DeviceLibrary::in_memory()
+                .write_many(&[("items/bad key.out".to_string(), "x".to_string())]);
+            std::process::exit(0);
+        }
+        let outcome = run_the_child_half(
+            "tests::in_memory_write_many_with_a_key_the_relay_refuses_exits_with_the_other_error_code",
+            ChildStandardError::Captured,
+        );
+        assert_eq!(
+            outcome.exit_code,
+            Some(3),
+            "内存那一种与转交那一种核同一条相对键规则：单测里收下的键真跑也收；标准错误：{}",
+            outcome.standard_error
+        );
+    }
+
+    #[test]
+    fn write_answered_failed_exits_with_the_code_in_the_reply() {
+        if is_the_child_half() {
+            relay_sending_requests_to_the_standard_input(b"failed 75 prune-server stopped\n")
+                .write("items/a.out", "x");
+            std::process::exit(0);
+        }
+        let outcome = run_the_child_half(
+            "tests::write_answered_failed_exits_with_the_code_in_the_reply",
+            ChildStandardError::Captured,
+        );
+        assert_eq!(
+            outcome.exit_code,
+            Some(75),
+            "回 failed 75 的单条写照 75 退，不当写成了；标准错误：{}",
+            outcome.standard_error
+        );
+        assert_eq!(
+            String::from_utf8_lossy(&outcome.requests_sent),
+            "write items/a.out 1\nx",
+            "请求照常发出，回应是 failed 才退"
+        );
+    }
+
+    #[test]
+    fn write_many_answered_failed_exits_with_the_code_in_the_reply() {
+        if is_the_child_half() {
+            relay_sending_requests_to_the_standard_input(b"failed 75 prune-server stopped\n")
+                .write_many(&[("items/a.out".to_string(), "x".to_string())]);
+            std::process::exit(0);
+        }
+        let outcome = run_the_child_half(
+            "tests::write_many_answered_failed_exits_with_the_code_in_the_reply",
+            ChildStandardError::Captured,
+        );
+        assert_eq!(
+            outcome.exit_code,
+            Some(75),
+            "回 failed 75 的一批照 75 退，不当写成了；标准错误：{}",
+            outcome.standard_error
+        );
+        assert_eq!(
+            String::from_utf8_lossy(&outcome.requests_sent),
+            "write-many 1\nitems/a.out 1\nx",
+            "请求照常发出，回应是 failed 才退"
+        );
+    }
+
+    #[test]
+    fn read_answered_found_with_a_length_past_isize_max_exits_with_the_other_error_code() {
+        if is_the_child_half() {
+            let length_past_isize_max =
+                usize::try_from(isize::MAX).expect("isize::MAX 是正数，装得进 usize") + 1;
+            relay_sending_requests_to_the_standard_input(
+                format!("found {length_past_isize_max}\n").as_bytes(),
+            )
+            .read("items/a.out");
+            std::process::exit(0);
+        }
+        let outcome = run_the_child_half(
+            "tests::read_answered_found_with_a_length_past_isize_max_exits_with_the_other_error_code",
+            ChildStandardError::Captured,
+        );
+        assert_eq!(
+            outcome.exit_code,
+            Some(3),
+            "报的字节数超过 isize::MAX 当认不出的回应退 3，不在锁里 panic；标准错误：{}",
+            outcome.standard_error
+        );
+    }
+
+    #[test]
+    fn give_up_with_the_standard_error_closed_still_exits_with_the_code_in_the_reply() {
+        if is_the_child_half() {
+            relay_sending_requests_to_the_standard_input(b"failed 75 prune-server stopped\n")
+                .write("items/a.out", "x");
+            std::process::exit(0);
+        }
+        let outcome = run_the_child_half(
+            "tests::give_up_with_the_standard_error_closed_still_exits_with_the_code_in_the_reply",
+            ChildStandardError::ClosedBeforeTheChildWrites,
+        );
+        assert_eq!(
+            outcome.exit_code,
+            Some(75),
+            "标准错误写不进也照回应里的 75 退，不 panic（panic 的测试退 101）"
+        );
+    }
+
+    // ── 锁从写请求一直握到读完回应：探锁的一方在装置线程放锁的那一刻空转着抢。
+
+    /// 一轮里各方走到哪一步；每个量记的是走到这一步的最大轮号（轮号从 1 数）。
+    #[derive(Default)]
+    struct ProbeRoundProgress {
+        /// 替身读到了这一轮请求的开头（装置线程这时握着锁、请求写到一半：请求比管道缓冲大）。
+        request_begun: AtomicUsize,
+        /// 探锁的一方空转起来之后，许替身读完这一轮的请求。
+        draining_allowed: AtomicUsize,
+        /// 替身读全了这一轮的请求。
+        request_complete: AtomicUsize,
+        /// 探锁的一方许替身回这一轮。
+        reply_allowed: AtomicUsize,
+        /// 替身要回这一轮了（先记再写回应）。
+        replied: AtomicUsize,
+    }
+
+    /// 空转等到条件成立（让出处理器，不睡）。
+    fn spin_until(condition: impl Fn() -> bool) {
+        while !condition() {
+            std::thread::yield_now();
+        }
+    }
+
+    // 线程数是场景的一部分，不是切片并行：装置线程、替身线程与探锁的本线程各一个，每轮管道上只有一问，
+    // 探锁抢到锁的那一刻才判得出它落没落在这一问的请求与回应之间。
+    #[test]
+    fn a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
+        // 请求比管道缓冲（Linux 默认 64 KiB）大得多：装置线程写请求时握着锁卡在管道上，等替身读。
+        const KEY_LENGTH_IN_BYTES: usize = 256 * 1024;
+        // 轮数只关系到锁在请求与回应之间放过时多快判出来；锁一直握着时每一轮都绿。
+        const ROUNDS: usize = 200;
+        // 本线程先握锁、让装置线程在锁上睡下的时长：它醒来拿到锁时锁记成「有人等过」，放锁要进内核叫醒，留给空转的探锁一方抢的空当。
+        const QUEUEING_TIME: std::time::Duration = std::time::Duration::from_millis(2);
+        // 替身读全请求之后探锁的一方再空转多久才许它回。
+        const GRACE_AFTER_THE_REQUEST_IS_COMPLETE: std::time::Duration =
+            std::time::Duration::from_micros(200);
+        let (requests_for_the_counterpart, request_end_of_the_device) =
+            std::io::pipe().expect("建请求管道");
+        let (reply_end_of_the_device, mut reply_end_of_the_counterpart) =
+            std::io::pipe().expect("建回应管道");
+        let device_library = DeviceLibrary {
+            backend: std::sync::Arc::new(Backend::Relay(std::sync::Mutex::new(RelayEnds {
+                requests: std::fs::File::from(std::os::fd::OwnedFd::from(
+                    request_end_of_the_device,
+                )),
+                replies: std::io::BufReader::new(std::fs::File::from(std::os::fd::OwnedFd::from(
+                    reply_end_of_the_device,
+                ))),
+            }))),
+        };
+        let progress = std::sync::Arc::new(ProbeRoundProgress::default());
+        let counterpart = {
+            let progress = std::sync::Arc::clone(&progress);
+            std::thread::spawn(move || {
+                let mut requests = std::io::BufReader::new(requests_for_the_counterpart);
+                for round_number in 1.. {
+                    let mut beginning = [0_u8; 5];
+                    if requests.read_exact(&mut beginning).is_err() {
+                        return;
+                    }
+                    assert_eq!(&beginning, b"read ", "第 {round_number} 问是一条 read");
+                    progress.request_begun.store(round_number, Ordering::SeqCst);
+                    spin_until(|| progress.draining_allowed.load(Ordering::SeqCst) >= round_number);
+                    let mut rest = Vec::new();
+                    requests
+                        .read_until(b'\n', &mut rest)
+                        .expect("请求管道读得了");
+                    assert_eq!(
+                        rest.len(),
+                        "items/".len() + KEY_LENGTH_IN_BYTES + 1,
+                        "第 {round_number} 问整条读全：相对键加换行"
+                    );
+                    progress
+                        .request_complete
+                        .store(round_number, Ordering::SeqCst);
+                    spin_until(|| progress.reply_allowed.load(Ordering::SeqCst) >= round_number);
+                    progress.replied.store(round_number, Ordering::SeqCst);
+                    reply_end_of_the_counterpart
+                        .write_all(b"absent\n")
+                        .expect("回应管道写得进");
+                }
+            })
+        };
+        let (orders, orders_received) = std::sync::mpsc::channel::<()>();
+        let (answers, answers_received) = std::sync::mpsc::channel::<Option<String>>();
+        let device_thread = {
+            let device_library = device_library.clone();
+            let key = format!("items/{}", "k".repeat(KEY_LENGTH_IN_BYTES));
+            std::thread::spawn(move || {
+                for () in orders_received {
+                    answers
+                        .send(device_library.read(&key))
+                        .expect("本线程等着收读的结果");
+                }
+            })
+        };
+        let Backend::Relay(relay_lock) = &*device_library.backend else {
+            panic!("这条测试只造转交那一种");
+        };
+        let mut round_where_the_lock_was_free_between_request_and_reply = None;
+        for round_number in 1..=ROUNDS {
+            let held_while_the_device_thread_queues = relay_lock
+                .lock()
+                .expect("这把锁只在本测试的三个线程之间传，没有线程握着它 panic");
+            orders.send(()).expect("装置线程等着收令");
+            std::thread::sleep(QUEUEING_TIME);
+            drop(held_while_the_device_thread_queues);
+            spin_until(|| progress.request_begun.load(Ordering::SeqCst) >= round_number);
+            progress
+                .draining_allowed
+                .store(round_number, Ordering::SeqCst);
+            let mut request_seen_complete_at = None;
+            let lock_was_free_between_request_and_reply = loop {
+                match relay_lock.try_lock() {
+                    Ok(probe) => {
+                        spin_until(|| {
+                            progress.request_complete.load(Ordering::SeqCst) >= round_number
+                                || progress.replied.load(Ordering::SeqCst) >= round_number
+                        });
+                        let not_yet_replied =
+                            progress.replied.load(Ordering::SeqCst) < round_number;
+                        drop(probe);
+                        break not_yet_replied;
+                    }
+                    Err(std::sync::TryLockError::WouldBlock) => {
+                        if progress.request_complete.load(Ordering::SeqCst) >= round_number {
+                            let seen_at = *request_seen_complete_at
+                                .get_or_insert_with(std::time::Instant::now);
+                            if seen_at.elapsed() >= GRACE_AFTER_THE_REQUEST_IS_COMPLETE {
+                                progress.reply_allowed.store(round_number, Ordering::SeqCst);
+                            }
+                        }
+                    }
+                    Err(std::sync::TryLockError::Poisoned(_)) => {
+                        panic!("这把锁只在本测试的三个线程之间传，没有线程握着它 panic")
+                    }
+                }
+            };
+            progress.reply_allowed.store(round_number, Ordering::SeqCst);
+            assert_eq!(
+                answers_received.recv().expect("装置线程交回读的结果"),
+                None,
+                "第 {round_number} 轮替身回 absent，读成键不在"
+            );
+            if lock_was_free_between_request_and_reply {
+                round_where_the_lock_was_free_between_request_and_reply = Some(round_number);
+                break;
+            }
+        }
+        drop(orders);
+        device_thread.join().expect("装置线程照常收尾");
+        drop(device_library);
+        counterpart
+            .join()
+            .expect("替身线程核过每一问是整条 read、读到请求管道的文件尾收尾");
+        assert_eq!(
+            round_where_the_lock_was_free_between_request_and_reply, None,
+            "探锁的一方抢到锁时替身已读全这一问、还没回：锁在请求与回应之间放过了"
+        );
+    }
 }
```

## 二、`scripts/experiment-apparatus/device-code-key.py`：对 HEAD（上下文，转交一侧，不是被判的产品代码）

```diff
diff --git a/scripts/experiment-apparatus/device-code-key.py b/scripts/experiment-apparatus/device-code-key.py
old mode 100755
new mode 100644
index 1f7407038..b0d9370c6
--- a/scripts/experiment-apparatus/device-code-key.py
+++ b/scripts/experiment-apparatus/device-code-key.py
@@ -30,9 +30,9 @@ research/prompts/e174-preregistration.md 修订 2 之五与修订 3）；装置
   crates/<名>                   仓根工作区的成员：构建根是仓根（仓根的 Cargo.toml、Cargo.lock、[profile.release]），unit-graph 的根是 <仓根>
   构建根定三样：Cargo.lock 与 [profile.release] 读哪一份、cargo build 在哪起、装置编到哪（<构建根>/<CARGO_TARGET_DIR 或 target>/release/<bin 名>）。
   入口的写法两种包一样，照 unit-graph：<包的 crate 名>::<模块…>::<函数名>，头一段是 crate 名（[lib] name，没写就是包名把 - 换成 _），
-    不是 bin 名；bin 由 --binary 给、交 unit-graph 的 --target bin:<bin 名>。例 E205（多处读坏倒挂三条改法的代价）的装置在
-    crates/singlefs-verification 的 bin e205_c971_inverted_first_txg_arm_costs 里：--package-directory crates/singlefs-verification
-    --binary e205_c971_inverted_first_txg_arm_costs --entry singlefs_verification::fault_cell_verdict。头一段对不上包的 crate 名，取键之前退 2、点名该写什么。
+    不是 bin 名；bin 由 --binary 给、交 unit-graph 的 --target bin:<bin 名>。例 装置在
+    crates/singlefs-verification 的 bin <bin 名> 里：--package-directory crates/singlefs-verification
+    --binary <bin 名> --entry singlefs_verification::<函数名>。头一段对不上包的 crate 名，取键之前退 2、点名该写什么。
 
 键的组成（每个入口一把代码键；代码键 = unit-graph key 打的 key 那一个 hex，它把单元、工具链行、依赖行一起算进去）：
   单元     unit-graph closure 与 key：<unit-graph 的根> <入口> --target bin:<bin 名> [--cfg feature="<特性>"]… --threads <N>
@@ -51,8 +51,13 @@ research/prompts/e174-preregistration.md 修订 2 之五与修订 3）；装置
   这里一个线程照请求经 scripts/prune-server/prune_server_client.py 读写服务；协议（一问一答）照装置一侧 crates/singlefs-device-library/src/lib.rs 文件头：
     read <相对键>                  → found <字节数> 加内容，或 absent（不在，或是撤回标记）
     write <相对键> <字节数> 加内容  → written（服务确认落盘之后才回）；先记进这一趟写过的键再写服务（没确认的写也可能落了盘，撤的时候一并撤）
+    write-many <条数>，后接每条 <相对键> <字节数> 加内容 → 整批一个 written：整批读全、核全之后才动手，每条照 write 记账（写过的键、复用哨兵），
+      再经客户端 write_many 写服务：一帧（prune_server_client.py 的 FRAME_BATCH_KEYS 条或 FRAME_BATCH_BYTES 字节，8192 条、16 MiB）以内一次提交、一次 WAL 同步，
+      超过切成几帧各自提交、各自同步（原子性只在一帧之内）；整批的 written 在全部帧都确认落盘之后才回。条数至少 1
     读写服务没成（PruneServerFailure）→ failed <退出码> <一句>，此后每个请求都照它回、不再连服务；装置照那个码退（75 是 prune-server 不可用）
-    认不出的请求、相对键不只是字母数字与 . _ - / → failed 3
+    认不出的请求、相对键不只是字母数字与 . _ - /（write 的键不合、write-many 里有一条不合都算：整批一条都不写）→ failed 3，回完就停止服务这两根管道：
+      不再读请求管道里剩下的字节（那条请求后面没读的字节不再对得上请求的边界，内容里像请求的那一段不能当请求办），也不再往回应管道写；
+      装置的写卡在请求管道上时得到写端断开，读回应读到那一句 failed 3，装置照它退
   键空间：族 apparatus-items（prune_server_client.py 文件头「键空间」），种类是 --binary 给的 bin 名（例 e174-lease-drop-wait-memory-return、
     e178-leaf-and-record-death-counts），相对键由装置给（E174（借据丢掉时等待与显存回还）是 <轮键>.out、codes/<代码键>.txt、index.tsv；E178（叶容器与记录死亡的计数）是 items/<项键>.out、codes/<代码键>.txt、index.tsv），
     命名空间按 <仓根> 定（落在临时目录底下的另占一份，自证一格一份；主仓不带：第二台在同一棵共享的树里、同一个绝对路径上跑，两台读写同一份）。
@@ -136,6 +141,22 @@ research/prompts/e174-preregistration.md 修订 2 之五与修订 3）；装置
       「crates 包：闭包点到别的 crate 的单元，那个 crate 依赖的外部包抬一版键变」
   DEVICE_CODE_KEY_BREAK=entry-crate-unchecked  不核入口头一段是不是包的 crate 名（原样交 unit-graph，它报 crate 找不到）
       「入口头一段写成 bin 名：退 2、点名要写的 crate 名」
+  DEVICE_CODE_KEY_BREAK=write-many-first-entry-only  write-many 整批只把头一条写进服务，照回 written
+      「write-many：两条一次写进临时服务、整批只回一个 written」
+  DEVICE_CODE_KEY_BREAK=write-many-keys-unrecorded  write-many 的键不记进这一趟写过的键（该撤的时候撤不到）
+      「write-many：编译与跑之间改了入口调到的函数，这一批每一条都撤」
+  DEVICE_CODE_KEY_BREAK=write-many-note-write-skipped  write-many 的每条不经 note_write（复用哨兵看不见这一批的写）
+      「write-many：复用哨兵照样记这一批的写」
+  DEVICE_CODE_KEY_BREAK=write-many-failure-reported-as-written  write-many 写服务没成时照回 written（装置当写成了、失败被吃掉）
+      「write-many：prune-server 连不上，装置收到 failed 75、这里退 75」
+  DEVICE_CODE_KEY_BREAK=write-many-keys-unchecked  write-many 不核每条的相对键（不合的键也写进服务）
+      「write-many：有一条相对键不合，回 failed 3、整批一条都不写」
+  DEVICE_CODE_KEY_BREAK=write-many-empty-accepted  write-many 0 照回 written（条数至少 1 不核）
+      「write-many 0：回 failed 3、装置照它退 3」
+  DEVICE_CODE_KEY_BREAK=relay-reads-on-after-refusal  一条请求判成认不出、回完 failed 3 之后接着读请求管道（把后面的字节当新请求，每条回一次应答）
+      「write-many：坏键在前、内容像 write items/injected.out 的一批：回 failed 3、转交不再读后面的字节、库里没有 items/injected.out」
+      「write：键不合、内容像 write items/injected.out 的请求：回 failed 3、转交不再读后面的字节、库里没有 items/injected.out」
+      「write-many：坏键在前、又大又多行的一批：装置不卡在写请求上、转交不卡在写回应上，两头都退」
 """
 import datetime
 import hashlib
@@ -176,7 +197,9 @@ SELFTEST_RUN_VARIABLE = "DEVICE_CODE_KEY_SELFTEST_RUN"
 BREAK_SWITCHES = ("raw-source-digest", "toolchain-lines-dropped", "host-line-dropped", "dependency-lines-dropped", "dependency-lines-not-narrowed", "bracket-skipped",
                   "device-binary-unset", "after-key-crash-propagates", "after-key-refusal-as-changed", "withdrawal-skipped", "withdrawn-read-as-present",
                   "relay-failure-reported-as-absent", "unknown-request-answered-absent", "sentinel-skipped", "forced-read-answered-found",
-                  "unit-graph-root-fixed", "build-in-apparatus-directory", "device-binary-under-package", "dependency-lines-one-crate", "entry-crate-unchecked")
+                  "unit-graph-root-fixed", "build-in-apparatus-directory", "device-binary-under-package", "dependency-lines-one-crate", "entry-crate-unchecked",
+                  "write-many-first-entry-only", "write-many-keys-unrecorded", "write-many-note-write-skipped", "write-many-failure-reported-as-written",
+                  "write-many-keys-unchecked", "write-many-empty-accepted", "relay-reads-on-after-refusal")
 HOST_LABELS = ("本机", "第二台")
 REFUSAL_SHOWN = 3
 TOKYO = datetime.timezone(datetime.timedelta(hours=9))
@@ -205,6 +228,14 @@ class UnitGraphMissing(Exception):
     """unit-graph 程序找不到：退 78。"""
 
 
+class RefusedRequest(Exception):
+    """转交里一条请求判成认不出：reply 是回给装置的那一句 failed；回完转交停止服务这两根管道（文件头「判定库的转交」）。"""
+
+    def __init__(self, reply):
+        super().__init__(reply)
+        self.reply = reply
+
+
 def break_is_set(switch):
     if os.environ.get(SELFTEST_RUN_VARIABLE) != "1":
         return False
@@ -492,13 +523,13 @@ class LibraryRelay:
 
     def __init__(self, store, label):
         self.store = store
-        self.written = []
+        self.written = {}   # 当有序集合用：一趟几万条，列表查找是平方级
         self.failure = None
         self.lock = threading.Lock()
         self.label = label
         self.seed = lib_reuse_checks.sentinel_seed()
         self.forced_hit = forced_hit_number(self.seed)
-        self.hits, self.rewritten = [], set()
+        self.hits, self.rewritten = {}, set()
         self.forced, self.forced_stored, self.forced_written = None, None, None
 
     def sentinel_announcement(self):
@@ -512,7 +543,7 @@ class LibraryRelay:
         if break_is_set("sentinel-skipped") or not key.endswith(ITEM_SUFFIX):
             return False
         if key not in self.hits:
-            self.hits.append(key)
+            self.hits[key] = None
         if self.forced is None and len(self.hits) == self.forced_hit:
             self.forced, self.forced_stored = key, verdict_digest(data)
             return not break_is_set("forced-read-answered-found")
@@ -573,14 +604,23 @@ class LibraryRelay:
         return code
 
     def serve(self, request_descriptor, reply_descriptor):
-        """读请求、回应答，到请求管道的尽头（装置退了）或装置不再读回应为止。"""
+        """读请求、回应答，到请求管道的尽头（装置退了）、装置不再读回应、或一条请求判成认不出为止；
+        认不出的请求回完 failed 3 就停：后面没读的字节不再对得上请求的边界，不当请求读，回应管道也不再写（装置的写卡在请求管道上时得到写端断开）。
+        弄坏开关 relay-reads-on-after-refusal 下回完接着读。"""
         with os.fdopen(request_descriptor, "rb") as requests, os.fdopen(reply_descriptor, "wb") as replies:
             try:
                 while True:
                     line = requests.readline()
                     if not line:
                         return
-                    answer = self.answer(line, requests)
+                    try:
+                        answer = self.answer(line, requests)
+                    except RefusedRequest as refused:
+                        replies.write(refused.reply)
+                        replies.flush()
+                        if break_is_set("relay-reads-on-after-refusal"):
+                            continue
+                        return
                     if answer is None:
                         return
                     replies.write(answer)
@@ -603,7 +643,7 @@ class LibraryRelay:
         return f"failed {code} {sentence}\n".encode("utf-8", "surrogateescape")
 
     def answer(self, line, requests):
-        """一个请求的应答（bytes）；请求读不全（装置中途退了）交 None。"""
+        """一个请求的应答（bytes）；请求读不全（装置中途退了）交 None；认不出的请求抛 RefusedRequest（带回给装置的那一句 failed）。"""
         if not line.endswith(b"\n"):
             return None
         words = line[:-1].decode("utf-8", "surrogateescape").split(" ")
@@ -622,14 +662,47 @@ class LibraryRelay:
                 return None
             with self.lock:
                 if words[1] not in self.written:
-                    self.written.append(words[1])
+                    self.written[words[1]] = None
             self.note_write(words[1], data)
             done, _ = self.through_service(lambda: self.store.write(words[1], data))
             return b"written\n" if done else self.failed()
+        if len(words) == 2 and words[0] == "write-many" and words[1].isdigit() and (int(words[1]) > 0 or break_is_set("write-many-empty-accepted")):
+            return self.answer_write_many(int(words[1]), requests)
+        return self.unrecognized_request(line)
+
+    def unrecognized_request(self, line):
+        """认不出的请求：抛 RefusedRequest，serve 回完那一句就停。"""
         if break_is_set("unknown-request-answered-absent"):
-            return b"absent\n"
+            raise RefusedRequest(b"absent\n")
         shown = line[:-1].decode("utf-8", "replace")[:120]
-        return f"failed {EXIT_REFUSED} 认不出的请求「{shown}」（只认 read <相对键>、write <相对键> <字节数>；相对键只用字母、数字与 . _ - /）\n".encode()
+        raise RefusedRequest(f"failed {EXIT_REFUSED} 认不出的请求「{shown}」（只认 read <相对键>、write <相对键> <字节数>、write-many <条数>；相对键只用字母、数字与 . _ - /）\n".encode())
+
+    def answer_write_many(self, count, requests):
+        """一批写的应答（文件头「判定库的转交」write-many）：整批读全、核全之后才记账、才写服务；请求读不全交 None，有一条不合抛认不出的请求（整批不写）。"""
+        entries = []
+        for _ in range(count):
+            header = requests.readline()
+            if not header.endswith(b"\n"):
+                return None
+            fields = header[:-1].decode("utf-8", "surrogateescape").split(" ")
+            key_is_valid = len(fields) == 2 and fields[1].isdigit() and (RELATIVE_KEY_PATTERN.fullmatch(fields[0]) or break_is_set("write-many-keys-unchecked"))
+            if not key_is_valid:
+                return self.unrecognized_request(header)
+            data = requests.read(int(fields[1]))
+            if len(data) != int(fields[1]):
+                return None
+            entries.append((fields[0], data))
+        if not break_is_set("write-many-keys-unrecorded"):
+            with self.lock:
+                for key, _data in entries:
+                    if key not in self.written:
+                        self.written[key] = None
+        if not break_is_set("write-many-note-write-skipped"):
+            for key, data in entries:
+                self.note_write(key, data)
+        sent = entries[:1] if break_is_set("write-many-first-entry-only") else entries
+        done, _ = self.through_service(lambda: self.store.write_many(sent))
+        return b"written\n" if done or break_is_set("write-many-failure-reported-as-written") else self.failed()
 
 
 def withdraw_written(relay):
@@ -886,7 +959,7 @@ sys.exit(int(os.environ.get("FAKE_CARGO_EXIT", "0")))
 '''
 
 SELFTEST_DEVICE = r'''
-import os, sys
+import os, sys, threading
 arguments = sys.argv[1:]
 with open(os.environ["FAKE_DEVICE_LOG"], "a", encoding="utf-8") as handle:
     handle.write("\t".join(arguments) + "\n")
@@ -914,22 +987,64 @@ if "--code-key-file" in arguments:
         if line.startswith("failed "):
             sys.exit(int(line.split(" ")[1]))
         return replies.read(int(line.split(" ")[1])) if line.startswith("found ") else None
-    if os.environ.get("FAKE_DEVICE_UNKNOWN_REQUEST"):
-        ask("erase items/x.out")
+    unknown_request = os.environ.get("FAKE_DEVICE_UNKNOWN_REQUEST")
+    if unknown_request:
+        ask("erase items/x.out" if unknown_request == "1" else unknown_request)
+    # 键不合的请求，后面紧跟的字节像请求（injected）或又大又多行：先把整条请求写完再读回应（写在线程里，卡住 15 秒记进 blocked.txt 退出；转交关了请求管道时写端得到断开）
+    refused_then_bytes = os.environ.get("FAKE_DEVICE_REFUSED_THEN_BYTES")
+    if refused_then_bytes:
+        injected = b"write items/injected.out 8\ninjected"
+        if refused_then_bytes == "write-many":
+            refused_request, refused_body = "write-many 1", f"items/bad key.out {len(injected)}\n".encode() + injected
+        elif refused_then_bytes == "write":
+            refused_request, refused_body = f"write items/bad key.out {len(injected)}", injected
+        else:
+            long_value = b"\n".join(b"z" * 100 for _ in range(20))
+            refused_request = "write-many 601"
+            refused_body = b"items/bad key.out 1\nx" + b"".join(f"items/{index:064x}.out {len(long_value)}\n".encode() + long_value for index in range(600))
+        def send_refused_request():
+            try:
+                requests.write(refused_request.encode() + b"\n" + refused_body)
+                requests.flush()
+            except BrokenPipeError:
+                pass
+        sender = threading.Thread(target=send_refused_request, daemon=True)
+        sender.start()
+        sender.join(15)
+        if sender.is_alive():
+            with open(os.path.join(work, "blocked.txt"), "w", encoding="utf-8") as handle:
+                handle.write("blocked in write\n")
+            os._exit(1)
+        line = replies.readline().decode().rstrip("\n")
+        seen.write(line + "\n")
+        seen.flush()
+        os._exit(int(line.split(" ")[1]) if line.startswith("failed ") else 1)   # 请求管道那一头关了时，退出时的冲刷不再报写端断开
+    # write-many 那几格：判出来的项攒成一批、读完之后一条请求写（WITHOUT_READS 另不读，头一个请求就是这一批）
+    write_many, without_reads = os.environ.get("FAKE_DEVICE_WRITE_MANY"), os.environ.get("FAKE_DEVICE_WRITE_MANY_WITHOUT_READS")
+    if os.environ.get("FAKE_DEVICE_WRITE_MANY_INVALID_KEY"):
+        ask("write-many 2", b"items/valid.out 1\nxitems/not*valid.out 1\ny")
+    batch = []
     outcomes = []
     # 复用哨兵那一格的三种坏装置：读到了还写（哨兵被现判）、现判写回的判定变了、读成不在却不写回（强制现判没起作用）
     rejudge_found, suffix, absent_unwritten = (os.environ.get(name, "") for name in ("FAKE_DEVICE_REJUDGE_FOUND", "FAKE_DEVICE_JUDGED_SUFFIX", "FAKE_DEVICE_ABSENT_UNWRITTEN"))
     for entry, code_key in entries:
         key = f"items/{code_key}.out"
-        found = ask(f"read {key}") is not None
+        found = not without_reads and ask(f"read {key}") is not None
         if found and not rejudge_found:
             outcomes.append(f"{entry}\treused")
         elif not found and absent_unwritten:
             outcomes.append(f"{entry}\treused")
         else:
             data = f"judged {code_key}{suffix}\nmeta process={os.getpid()}\n".encode()   # meta 行每趟不同：转交比强制现判时要去掉它
-            ask(f"write {key} {len(data)}", data)
+            if write_many or without_reads:
+                batch.append((key, data))
+            else:
+                ask(f"write {key} {len(data)}", data)
             outcomes.append(f"{entry}\tjudged")
+    if batch:
+        written.write("".join(f"{key}\n" for key, _data in batch))
+        written.flush()
+        ask(f"write-many {len(batch)}", b"".join(f"{key} {len(data)}\n".encode() + data for key, data in batch))
     data = composition.encode()
     ask(f"write codes/{entries[0][1]}.txt {len(data)}", data)
     with open(os.path.join(work, "outcomes.txt"), "w", encoding="utf-8") as handle:
@@ -1278,6 +1393,60 @@ def check_unknown_request(code, _stderr, _device_log, work, _store, _directory,
     return passed, f"退 {code}（要 {EXIT_REFUSED}），假装置收到的回应 {replies[:2]}"
 
 
+TWO_ENTRY_RUN_ARGUMENTS = ["run", "--binary", "fake-device", "--threads", "1", "--entry", SCRIPTS_ENTRY, "--entry", SCRIPTS_OTHER_ENTRY, "--", "segment-four"]
+
+
+def two_entry_run(extra_environment, check):
+    """两入口的小树上跑一次 run（假装置判出两条项）；check(退出码, 标准错误, 假装置收到的回应（逐行）, 假装置写的项的相对键, 小树那一份库) → (过不过, 说明)。"""
+    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
+        environment = selftest_tree(directory)
+        with open(os.path.join(directory, "tree", SELFTEST_SCRIPTS_SOURCE), "w", encoding="utf-8") as handle:
+            handle.write(SELFTEST_TWO_ENTRY_SOURCE)
+        environment.update({name: value.replace("<tree>", os.path.join(directory, "tree")) for name, value in extra_environment.items()})
+        code, _stdout, stderr = selftest_call(directory, environment, TWO_ENTRY_RUN_ARGUMENTS)
+        work = environment["FAKE_DEVICE_WORK"]
+        replies = [line for line in work_text(work, "replies.txt").split("\n") if line]
+        keys = [line for line in work_text(work, "written-keys.txt").split("\n") if line.startswith("items/")]
+        return check(code, stderr, replies, keys, selftest_store(directory))
+
+
+def check_write_many_stored(code, stderr, replies, keys, store):
+    """两条一次写：退 0、两条都在临时服务里；回应是两个读的 absent、这一批一个 written、组成清单那一个 written。"""
+    stored = [store.read(key) for key in keys]
+    passed = code == 0 and len(keys) == 2 and all(item is not None and item.startswith(b"judged ") for item in stored) and replies == ["absent", "absent", "written", "written"]
+    return passed, f"退 {code}，假装置写的 {keys}，临时服务里 {stored!r}，回应 {replies}，{stderr.strip()[-200:]}"
+
+
+def check_write_many_withdrawn(code, stderr, _replies, keys, store):
+    """编译与跑之间源码变了：这一批每一条都改写成撤回标记、退 1。"""
+    stored = [store.read(key) for key in keys]
+    passed = code == EXIT_FAILED and len(keys) == 2 and all(item == WITHDRAWN_VALUE for item in stored) and "源码在编译与跑之间变了" in stderr
+    return passed, f"退 {code}（要 {EXIT_FAILED}），临时服务里 {stored!r}，{stderr.strip()[-200:]}"
+
+
+def check_write_many_unreachable(code, stderr, replies, _keys, _store):
+    """头一个请求就是一批写、服务连不上：回 failed 75、装置退 75，这里退 75，失败不吃掉。"""
+    passed = code == prune_server_client.EXIT_UNAVAILABLE and bool(replies) and replies[0].startswith(f"failed {prune_server_client.EXIT_UNAVAILABLE} ")
+    return passed, f"退 {code}（要 {prune_server_client.EXIT_UNAVAILABLE}），回应 {replies[:3]}，{stderr.strip()[-200:]}"
+
+
+def check_write_many_invalid_key(code, stderr, replies, _keys, store):
+    """一批里有一条相对键不合：回 failed 3、装置退 3，批里合规的那一条也没写。"""
+    valid_entry = store.read("items/valid.out")
+    passed = code == EXIT_REFUSED and bool(replies) and replies[0].startswith(f"failed {EXIT_REFUSED} 认不出的请求") and valid_entry is None
+    return passed, f"退 {code}（要 {EXIT_REFUSED}），回应 {replies[:2]}，临时服务里 items/valid.out 是 {valid_entry!r}，{stderr.strip()[-200:]}"
+
+
+def check_refused_then_stops(code, stderr, _device_log, work, store, _directory, _environment):
+    """键不合的请求：回 failed 3、装置退 3、转交不再读后面的字节（库里没有 items/injected.out）、装置没卡在写请求上（没有 blocked.txt）。"""
+    replies = [line for line in work_text(work, "replies.txt").split("\n") if line]
+    injected = store.read("items/injected.out")
+    blocked = work_text(work, "blocked.txt")
+    passed = code == EXIT_REFUSED and bool(replies) and replies[0].startswith(f"failed {EXIT_REFUSED} 认不出的请求") and injected is None and not blocked
+    return passed, (f"退 {code}（要 {EXIT_REFUSED}），回应 {replies[:2]}，临时服务里 items/injected.out 是 {injected!r}，"
+                    f"装置{'卡在写请求上' if blocked else '没卡'}，{stderr.strip()[-200:]}")
+
+
 def selftest():
     """每种判定至少一格；弄坏开关打开时（DEVICE_CODE_KEY_BREAK，文件头逐个列了红在哪一格）照样跑，红在点名的那一格。只在临时 prune-server 上跑。"""
     problem = prune_server_client.selftest_service_problem()
@@ -1325,6 +1494,21 @@ def selftest():
          lambda: run_cell(run_arguments, {prune_server_client.SOCKET_VARIABLE: "<tree>/absent.sock", prune_server_client.TCP_ADDRESS_VARIABLE: "127.0.0.1:1",
                                           prune_server_client.RECONNECT_LIMIT_VARIABLE: "1"}, check_unreachable)),
         ("认不出的请求：回 failed 3、装置照它退 3", lambda: run_cell(run_arguments, {"FAKE_DEVICE_UNKNOWN_REQUEST": "1"}, check_unknown_request)),
+        ("write-many：两条一次写进临时服务、整批只回一个 written", lambda: two_entry_run({"FAKE_DEVICE_WRITE_MANY": "1"}, check_write_many_stored)),
+        ("write-many：编译与跑之间改了入口调到的函数，这一批每一条都撤",
+         lambda: two_entry_run({"FAKE_DEVICE_WRITE_MANY": "1", "FAKE_DEVICE_EDIT_SOURCE": f"<tree>/{SELFTEST_SCRIPTS_SOURCE}"}, check_write_many_withdrawn)),
+        ("write-many：复用哨兵照样记这一批的写", lambda: reuse_sentinel_cell({"FAKE_DEVICE_WRITE_MANY": "1"})),
+        ("write-many：prune-server 连不上，装置收到 failed 75、这里退 75",
+         lambda: two_entry_run({"FAKE_DEVICE_WRITE_MANY_WITHOUT_READS": "1", prune_server_client.SOCKET_VARIABLE: "<tree>/absent.sock",
+                                prune_server_client.TCP_ADDRESS_VARIABLE: "127.0.0.1:1", prune_server_client.RECONNECT_LIMIT_VARIABLE: "1"}, check_write_many_unreachable)),
+        ("write-many：有一条相对键不合，回 failed 3、整批一条都不写", lambda: two_entry_run({"FAKE_DEVICE_WRITE_MANY_INVALID_KEY": "1"}, check_write_many_invalid_key)),
+        ("write-many 0：回 failed 3、装置照它退 3", lambda: run_cell(run_arguments, {"FAKE_DEVICE_UNKNOWN_REQUEST": "write-many 0"}, check_unknown_request)),
+        ("write-many：坏键在前、内容像 write items/injected.out 的一批：回 failed 3、转交不再读后面的字节、库里没有 items/injected.out",
+         lambda: run_cell(run_arguments, {"FAKE_DEVICE_REFUSED_THEN_BYTES": "write-many"}, check_refused_then_stops)),
+        ("write：键不合、内容像 write items/injected.out 的请求：回 failed 3、转交不再读后面的字节、库里没有 items/injected.out",
+         lambda: run_cell(run_arguments, {"FAKE_DEVICE_REFUSED_THEN_BYTES": "write"}, check_refused_then_stops)),
+        ("write-many：坏键在前、又大又多行的一批：装置不卡在写请求上、转交不卡在写回应上，两头都退",
+         lambda: run_cell(run_arguments, {"FAKE_DEVICE_REFUSED_THEN_BYTES": "large"}, check_refused_then_stops)),
         ("工具链行带一行 host: <三元组>，closure 与 key 两趟都给（unit-graph 缺它退 2）", host_line_cell),
 
         ("reuse-three-questions", three_questions_cell),
@@ -1425,16 +1609,17 @@ SENTINEL_CASES = (
 )
 
 
-def reuse_sentinel_cell():
+def reuse_sentinel_cell(writer_variables=None):
     """复用哨兵在转交里（文件头「判定库的转交」复用哨兵那一段）：两入口的小树跑两趟，第二趟的种子取强制现判落在第 1 条命中的那一个
-    （entry_function 那一条强制现判、other_entry 那一条当哨兵）；SENTINEL_CASES 四种各一棵树，绿的那一种另核 DEVICE_CODE_KEY_SENTINEL 那一行点名了两条。"""
+    （entry_function 那一条强制现判、other_entry 那一条当哨兵）；SENTINEL_CASES 四种各一棵树，绿的那一种另核 DEVICE_CODE_KEY_SENTINEL 那一行点名了两条。
+    writer_variables 是两趟都另设的环境（假装置换一种写法写，例如 write-many）。"""
     seed = next(seed for seed in range(1, 1 << 16) if forced_hit_number(seed) == 1)
     entries = [SCRIPTS_ENTRY, SCRIPTS_OTHER_ENTRY]
     arguments = ["run", "--binary", "fake-device", "--threads", "1"] + [word for entry in entries for word in ("--entry", entry)] + ["--", "segment-four"]
     wrong = []
     for label, variables, expected_code, sentence in SENTINEL_CASES:
         with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
-            environment = selftest_tree(directory)
+            environment = dict(selftest_tree(directory), **(writer_variables or {}))
             with open(os.path.join(directory, "tree", SELFTEST_SCRIPTS_SOURCE), "w", encoding="utf-8") as handle:
                 handle.write(SELFTEST_TWO_ENTRY_SOURCE)
             first_code, _stdout, first_stderr = selftest_call(directory, environment, arguments)
```
