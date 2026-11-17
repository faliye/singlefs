# device-library-write-many-code-r3：r2 判决之后的修补（2026-11-10）

来源：`diff -u <r2 开工快照那一版（实现员补丁的 baseline）> <此刻主工作区>`。主 agent 在实现员补丁之外另改一处：测试辅助的类型抽成别名 `AskTheLibrary`（clippy type_complexity）。

```diff
--- crates/singlefs-device-library/src/lib.rs
+++ crates/singlefs-device-library/src/lib.rs
@@ -2,16 +2,18 @@
 // 转给 prune-server 的那一份判定库（.claude/rules/verification.md「判定库只有一份」；转交的一侧写在 device-code-key.py 文件头「判定库的转交」）。
 // device-code-key.py 起装置时开两根管道，把装置这一头的描述符号放进环境变量 DEVICE_LIBRARY_REQUEST_FD（装置写请求）与
 // DEVICE_LIBRARY_REPLY_FD（装置读回应）；键是相对这个装置那一种的路径（例 items/<项键>.out、codes/<代码键>.txt、index.tsv），族与种类由那一侧定。
-// 协议一问一答，几个线程共用一条，整问整答在一把锁里（从写请求到读完回应一直握着，别的线程只等这一问）：
+// 协议一问一答，几个线程共用一条，整问整答在一把锁里（从写请求到读完回应、连同 found 后面的内容一直握着，别的线程只等这一问）：
 //   请求 read <相对键>\n                    回 found <字节数>\n<内容>，或 absent\n
 //   请求 write <相对键> <字节数>\n<内容>    回 written\n（服务确认落盘之后才回）
 //   请求 write-many <条数>\n 后接每条 <相对键> <字节数>\n<内容>    整批回一个 written\n；条数至少 1。
-//     整批经 prune-server 客户端写服务：一帧（8192 条或 16 MiB）以内是一次提交，超过切成几帧各自提交（原子性只在一帧之内）；
-//     整批的 written 在全部帧都确认落盘之后才回。
+//     整批经 prune-server 客户端的 write_many 写服务，客户端可能把一批切成几帧、各帧各自提交：条数、字节数、值的形态（长值与以分段清单魔数起头的值分段存）、
+//     键留着的登记旧版本（多一次按前缀删）都会让它多切，切法以 scripts/prune-server/prune_server_client.py 文件头为准。原子性只在一帧之内：
+//     一批写到半截出了事，服务里可能只有其中几帧。整批的 written 在全部帧都确认落盘之后才回。
 //   三种都可能回 failed <退出码> <一句>\n：这里打 ✗ 与那一句、进程照那个退出码退（75 是 prune-server 不可用，3 是别的错），不当「键不在」现判。
-//   相对键只用 ASCII 字母、数字与 . _ - /，不空、不带空白（与 device-code-key.py 的 RELATIVE_KEY_PATTERN 同一条）：write 与 write-many 在拿锁、
+//   相对键只用 ASCII 字母、数字与 . _ - /，不空、不带空白（与 device-code-key.py 的 RELATIVE_KEY_PATTERN 同一条）：read、write 与 write-many 在拿锁、
 //   拼请求之前先在这一侧核（转交与内存两种都核），不合的照「别的错」退 3，管道上一个字节都不发；那一侧照同一条再核，不合的回 failed 3。
-//   管道断了、回应认不出退 3。
+//   管道断了、回应认不出、found 报的字节数在这个进程里分配不出，都退 3。
+//   found 后面的内容不是 UTF-8 时 read 交 None（与键不在一样，调用方去现判），不退。
 // 撤掉的项（那一侧核出编译与跑之间源码变了时改写成撤回标记的）由那一侧回 absent，这里看不到标记。
 // 每个用到它的装置在自己的 Cargo.toml 里按 path 依赖引本包（crates/singlefs-device-library），再 `use singlefs_device_library::DeviceLibrary;`
 // （从仓根算单元键时 unit-graph 看不到 `#[path]` 引进来的文件，键拒算；条目级 include! 同样不收）。
@@ -28,7 +30,7 @@
 /// 「别的错」的退出码（与 prune-server 客户端「别的错」同一个）：管道断了、回应认不出、相对键不合都照它退。
 const OTHER_ERROR_EXIT_CODE: i32 = 3;
 /// 转交那一种的锁为什么不会中毒：持锁时做的几样都不 panic。
-const RELAY_LOCK_IS_NEVER_POISONED: &str = "转交判定库的锁不会中毒：持锁时只做拼请求、管道读写、按回应报的字节数分配（报的数超过 isize::MAX 先当认不出退掉，分配失败是中止进程、不展开）与 give_up（写标准错误不看成败、退进程不展开），这几样都不 panic";
+const RELAY_LOCK_IS_NEVER_POISONED: &str = "转交判定库的锁不会中毒：持锁时只做拼请求、管道读写、按回应报的字节数可失败地预留（报的数超过 isize::MAX 先当认不出退掉，预留不成照别的错退掉）与 give_up（写标准错误不看成败、退进程不展开），这几样都不 panic";
 /// 内存那一种的锁为什么不会中毒：持锁时做的几样都不 panic。
 const MEMORY_LOCK_IS_NEVER_POISONED: &str = "单测内存库的锁不会中毒：持锁时只做 BTreeMap 的插入、查找、数键与成批写次数加一（usize 数不满），这几样都不 panic";
 
@@ -184,12 +186,13 @@
         }
     }
 
-    /// 读一条：不在（或已撤掉）交 None；读失败照退出码退，不交 None。
+    /// 读一条：不在（或已撤掉）交 None，回的内容不是 UTF-8 也交 None；相对键不合、读失败照退出码退，不交 None。
     pub fn read(&self, relative: &str) -> Option<String> {
+        let asked = format!("读 {relative}");
+        give_up_unless_the_relay_accepts_the_key(relative, &asked);
         match &*self.backend {
             Backend::Relay(ends) => {
                 let mut ends = ends.lock().expect(RELAY_LOCK_IS_NEVER_POISONED);
-                let asked = format!("读 {relative}");
                 if let Err(error) = ends
                     .requests
                     .write_all(format!("read {relative}\n").as_bytes())
@@ -203,7 +206,7 @@
                 if reply == "absent" {
                     return None;
                 }
-                // 报的字节数超过 isize::MAX 时 vec! 容量溢出会 panic、在锁里毒掉锁：当认不出的回应退掉。
+                // 报的字节数超过 isize::MAX 的不是任何一个值的长度：当认不出的回应退掉。
                 let Some(length) = reply
                     .strip_prefix("found ")
                     .and_then(|text| text.parse::<usize>().ok())
@@ -211,7 +214,15 @@
                 else {
                     give_up_on_reply(&reply, &asked)
                 };
-                let mut contents = vec![0_u8; length];
+                // 不可失败的分配（vec!、with_capacity）分配不出时中止进程、退出码不是 3：先可失败地预留，预留不成照别的错退。
+                let mut contents = Vec::new();
+                if let Err(error) = contents.try_reserve_exact(length) {
+                    give_up(
+                        OTHER_ERROR_EXIT_CODE,
+                        &format!("{asked}：回应报的 {length} 字节在这个进程里分配不出：{error}"),
+                    );
+                }
+                contents.resize(length, 0_u8);
                 if let Err(error) = ends.replies.read_exact(&mut contents) {
                     give_up(
                         OTHER_ERROR_EXIT_CODE,
@@ -310,7 +321,7 @@
 #[cfg(test)]
 mod tests {
     use super::{is_relative_key_the_relay_accepts, Backend, DeviceLibrary, RelayEnds};
-    use std::io::{BufRead, Read, Write};
+    use std::io::{Read, Write};
     use std::sync::atomic::{AtomicUsize, Ordering};
 
     /// 转交那一侧在测试里的替身：装置写进去的请求从 `requests_written_by_the_device` 读出来，
@@ -715,6 +726,100 @@
             "报的字节数超过 isize::MAX 当认不出的回应退 3，不在锁里 panic；标准错误：{}",
             outcome.standard_error
         );
+        assert!(
+            outcome.standard_error.contains("回应认不出"),
+            "超过 isize::MAX 的字节数在预留之前就当认不出的回应退掉，不走到「分配不出」；标准错误：{}",
+            outcome.standard_error
+        );
+    }
+
+    #[test]
+    fn read_answered_found_with_a_length_that_cannot_be_allocated_exits_with_the_other_error_code()
+    {
+        if is_the_child_half() {
+            // 2^62 字节不过 isize::MAX，可大过 x86_64 与 aarch64 的用户地址空间，哪台机器上都分配不出。
+            let length_that_cannot_be_allocated = 1_usize << 62;
+            relay_sending_requests_to_the_standard_input(
+                format!("found {length_that_cannot_be_allocated}\n").as_bytes(),
+            )
+            .read("items/a.out");
+            std::process::exit(0);
+        }
+        let outcome = run_the_child_half(
+            "tests::read_answered_found_with_a_length_that_cannot_be_allocated_exits_with_the_other_error_code",
+            ChildStandardError::Captured,
+        );
+        assert_eq!(
+            outcome.exit_code,
+            Some(3),
+            "found 报的字节数分配不出照「别的错」退 3，不中止进程（被信号杀掉时退出码是 None）；标准错误：{}",
+            outcome.standard_error
+        );
+    }
+
+    #[test]
+    fn read_with_a_newline_in_the_key_exits_with_the_other_error_code_before_sending_a_byte() {
+        if is_the_child_half() {
+            relay_sending_requests_to_the_standard_input(b"absent\nwritten\n")
+                .read("items/a.out\nwrite items/injected.out 0");
+            std::process::exit(0);
+        }
+        let outcome = run_the_child_half(
+            "tests::read_with_a_newline_in_the_key_exits_with_the_other_error_code_before_sending_a_byte",
+            ChildStandardError::Captured,
+        );
+        assert_eq!(
+            outcome.exit_code,
+            Some(3),
+            "带换行的相对键照「别的错」退 3，不读成键不在；标准错误：{}",
+            outcome.standard_error
+        );
+        assert_eq!(
+            String::from_utf8_lossy(&outcome.requests_sent),
+            "",
+            "换行后面那一段不上管道：转交那一侧读不出一条装置没交给 write 的写请求"
+        );
+    }
+
+    #[test]
+    fn read_with_a_space_in_the_key_exits_with_the_other_error_code_before_sending_a_byte() {
+        if is_the_child_half() {
+            relay_sending_requests_to_the_standard_input(b"absent\n").read("items/bad key.out");
+            std::process::exit(0);
+        }
+        let outcome = run_the_child_half(
+            "tests::read_with_a_space_in_the_key_exits_with_the_other_error_code_before_sending_a_byte",
+            ChildStandardError::Captured,
+        );
+        assert_eq!(
+            outcome.exit_code,
+            Some(3),
+            "带空格的相对键照「别的错」退 3，不读成键不在；标准错误：{}",
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
+    fn in_memory_read_with_a_key_the_relay_refuses_exits_with_the_other_error_code() {
+        if is_the_child_half() {
+            DeviceLibrary::in_memory().read("items/bad key.out");
+            std::process::exit(0);
+        }
+        let outcome = run_the_child_half(
+            "tests::in_memory_read_with_a_key_the_relay_refuses_exits_with_the_other_error_code",
+            ChildStandardError::Captured,
+        );
+        assert_eq!(
+            outcome.exit_code,
+            Some(3),
+            "内存那一种的 read 与转交那一种核同一条相对键规则：单测里读成不在的键真跑不会退 3；标准错误：{}",
+            outcome.standard_error
+        );
     }
 
     #[test]
@@ -735,21 +840,44 @@
         );
     }
 
-    // ── 锁从写请求一直握到读完回应：探锁的一方在装置线程放锁的那一刻空转着抢。
+    // ── 锁从写请求一直握到读完回应（连同 found 后面的内容）：探锁的一方在装置线程放锁的那一刻空转着抢。
 
-    /// 一轮里各方走到哪一步；每个量记的是走到这一步的最大轮号（轮号从 1 数）。
+    /// 每一问的请求里那一段大数据的字节数（相对键或内容）：比管道缓冲（Linux 默认 64 KiB）大得多，
+    /// 装置线程写请求时握着锁卡在管道上、等替身读，探锁的一方在请求写完之前就空转起来。
+    const PROBED_REQUEST_PAYLOAD_IN_BYTES: usize = 256 * 1024;
+
+    /// 替身先读这么多字节的请求开头、记下「请求开了头」，再等探锁的一方许它读完。
+    const PROBED_REQUEST_BEGINNING_IN_BYTES: usize = 5;
+
+    /// 装置线程每一轮调一次口子的那一步：读的交回读到的，写的交 None。
+    type AskTheLibrary = Box<dyn Fn(&DeviceLibrary) -> Option<String> + Send>;
+
+    /// 探锁的几条测试里装置线程每一轮问的那一问。
+    struct ProbedExchange {
+        /// 装置线程每一轮调一次口子；读的交回读到的，写的交 None。
+        ask: AskTheLibrary,
+        /// 这一问的整条请求，逐字节（替身照它核每一轮读到的请求）。
+        expected_request: Vec<u8>,
+        /// 替身按次序一段一段写的回应：每一段写之前等探锁的一方许可，段与段之间是留给探锁的一方抢锁的空当。
+        reply_parts: Vec<&'static [u8]>,
+        /// 口子每一轮该交回的。
+        expected_answer: Option<String>,
+    }
+
+    /// 一轮里各方走到哪一步。请求那三个量记的是走到这一步的最大轮号（从 1 数）；
+    /// 回应那两个量记的是从头数起的段数（每轮的段数相同，第 n 轮写完时是 n 乘每轮段数）。
     #[derive(Default)]
     struct ProbeRoundProgress {
-        /// 替身读到了这一轮请求的开头（装置线程这时握着锁、请求写到一半：请求比管道缓冲大）。
+        /// 替身读到了这一轮请求的开头（装置线程这时握着锁、请求写到一半）。
         request_begun: AtomicUsize,
         /// 探锁的一方空转起来之后，许替身读完这一轮的请求。
         draining_allowed: AtomicUsize,
         /// 替身读全了这一轮的请求。
         request_complete: AtomicUsize,
-        /// 探锁的一方许替身回这一轮。
-        reply_allowed: AtomicUsize,
-        /// 替身要回这一轮了（先记再写回应）。
-        replied: AtomicUsize,
+        /// 探锁的一方许替身写到第几段回应。
+        reply_parts_allowed: AtomicUsize,
+        /// 替身要写第几段回应了（先记再写）。
+        reply_parts_written: AtomicUsize,
     }
 
     /// 空转等到条件成立（让出处理器，不睡）。
@@ -759,20 +887,28 @@
         }
     }
 
-    // 线程数是场景的一部分，不是切片并行：装置线程、替身线程与探锁的本线程各一个，每轮管道上只有一问，
-    // 探锁抢到锁的那一刻才判得出它落没落在这一问的请求与回应之间。
-    #[test]
-    fn a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
-        // 请求比管道缓冲（Linux 默认 64 KiB）大得多：装置线程写请求时握着锁卡在管道上，等替身读。
-        const KEY_LENGTH_IN_BYTES: usize = 256 * 1024;
-        // 轮数只关系到锁在请求与回应之间放过时多快判出来；锁一直握着时每一轮都绿。
+    // 线程数是场景的一部分，不是切片并行：装置线程、替身线程与探锁的本线程各一个，每一轮管道上只有一问，
+    // 探锁抢到锁的那一刻才判得出它落没落在这一问的中间。
+    /// 装置线程一轮一轮问 `exchange` 那一问，探锁的一方每一轮空转着抢锁；抢到锁时替身还没写完这一轮的回应，
+    /// 交那一轮的轮号（锁在一问的中间放过了），轮数跑满都没有交 None。锁一直握着时每一轮都是 None：
+    /// 探锁的一方只在替身读到请求开头（装置线程已握着锁）之后才抢，装置线程读完最后一段回应才放锁，那一段在写之前就记上了。
+    fn round_where_the_lock_was_free_in_the_middle_of_the_exchange(
+        exchange: ProbedExchange,
+    ) -> Option<usize> {
+        // 轮数只关系到锁在一问的中间放过时多快判出来；锁一直握着时每一轮都绿。
         const ROUNDS: usize = 200;
         // 本线程先握锁、让装置线程在锁上睡下的时长：它醒来拿到锁时锁记成「有人等过」，放锁要进内核叫醒，留给空转的探锁一方抢的空当。
         const QUEUEING_TIME: std::time::Duration = std::time::Duration::from_millis(2);
-        // 替身读全请求之后探锁的一方再空转多久才许它回。
-        const GRACE_AFTER_THE_REQUEST_IS_COMPLETE: std::time::Duration =
-            std::time::Duration::from_micros(200);
-        let (requests_for_the_counterpart, request_end_of_the_device) =
+        // 替身走到下一步（读全请求、写完一段回应）之后，探锁的一方再空转多久才许它写下一段回应。
+        const GRACE_AFTER_EACH_STEP: std::time::Duration = std::time::Duration::from_micros(200);
+        let ProbedExchange {
+            ask,
+            expected_request,
+            reply_parts,
+            expected_answer,
+        } = exchange;
+        let reply_parts_per_round = reply_parts.len();
+        let (mut requests_for_the_counterpart, request_end_of_the_device) =
             std::io::pipe().expect("建请求管道");
         let (reply_end_of_the_device, mut reply_end_of_the_counterpart) =
             std::io::pipe().expect("建回应管道");
@@ -790,32 +926,46 @@
         let counterpart = {
             let progress = std::sync::Arc::clone(&progress);
             std::thread::spawn(move || {
-                let mut requests = std::io::BufReader::new(requests_for_the_counterpart);
+                let (expected_beginning, expected_rest) =
+                    expected_request.split_at(PROBED_REQUEST_BEGINNING_IN_BYTES);
                 for round_number in 1.. {
-                    let mut beginning = [0_u8; 5];
-                    if requests.read_exact(&mut beginning).is_err() {
+                    let mut beginning = [0_u8; PROBED_REQUEST_BEGINNING_IN_BYTES];
+                    if requests_for_the_counterpart
+                        .read_exact(&mut beginning)
+                        .is_err()
+                    {
                         return;
                     }
-                    assert_eq!(&beginning, b"read ", "第 {round_number} 问是一条 read");
+                    assert_eq!(
+                        beginning.as_slice(),
+                        expected_beginning,
+                        "第 {round_number} 问的开头"
+                    );
                     progress.request_begun.store(round_number, Ordering::SeqCst);
                     spin_until(|| progress.draining_allowed.load(Ordering::SeqCst) >= round_number);
-                    let mut rest = Vec::new();
-                    requests
-                        .read_until(b'\n', &mut rest)
+                    let mut rest = vec![0_u8; expected_rest.len()];
+                    requests_for_the_counterpart
+                        .read_exact(&mut rest)
                         .expect("请求管道读得了");
-                    assert_eq!(
-                        rest.len(),
-                        "items/".len() + KEY_LENGTH_IN_BYTES + 1,
-                        "第 {round_number} 问整条读全：相对键加换行"
+                    assert!(
+                        rest == expected_rest,
+                        "第 {round_number} 问整条读全，与预期逐字节相同"
                     );
                     progress
                         .request_complete
                         .store(round_number, Ordering::SeqCst);
-                    spin_until(|| progress.reply_allowed.load(Ordering::SeqCst) >= round_number);
-                    progress.replied.store(round_number, Ordering::SeqCst);
-                    reply_end_of_the_counterpart
-                        .write_all(b"absent\n")
-                        .expect("回应管道写得进");
+                    for reply_part in &reply_parts {
+                        let part_number = progress.reply_parts_written.load(Ordering::SeqCst) + 1;
+                        spin_until(|| {
+                            progress.reply_parts_allowed.load(Ordering::SeqCst) >= part_number
+                        });
+                        progress
+                            .reply_parts_written
+                            .store(part_number, Ordering::SeqCst);
+                        reply_end_of_the_counterpart
+                            .write_all(reply_part)
+                            .expect("回应管道写得进");
+                    }
                 }
             })
         };
@@ -823,20 +973,21 @@
         let (answers, answers_received) = std::sync::mpsc::channel::<Option<String>>();
         let device_thread = {
             let device_library = device_library.clone();
-            let key = format!("items/{}", "k".repeat(KEY_LENGTH_IN_BYTES));
             std::thread::spawn(move || {
                 for () in orders_received {
                     answers
-                        .send(device_library.read(&key))
-                        .expect("本线程等着收读的结果");
+                        .send(ask(&device_library))
+                        .expect("本线程等着收口子交回的");
                 }
             })
         };
         let Backend::Relay(relay_lock) = &*device_library.backend else {
-            panic!("这条测试只造转交那一种");
+            panic!("探锁只造转交那一种");
         };
-        let mut round_where_the_lock_was_free_between_request_and_reply = None;
+        let mut round_where_the_lock_was_free = None;
         for round_number in 1..=ROUNDS {
+            let reply_parts_before_this_round = (round_number - 1) * reply_parts_per_round;
+            let reply_parts_through_this_round = round_number * reply_parts_per_round;
             let held_while_the_device_thread_queues = relay_lock
                 .lock()
                 .expect("这把锁只在本测试的三个线程之间传，没有线程握着它 panic");
@@ -847,25 +998,35 @@
             progress
                 .draining_allowed
                 .store(round_number, Ordering::SeqCst);
-            let mut request_seen_complete_at = None;
-            let lock_was_free_between_request_and_reply = loop {
+            let mut step_seen_at = None;
+            let lock_was_free_in_the_middle = loop {
                 match relay_lock.try_lock() {
                     Ok(probe) => {
-                        spin_until(|| {
-                            progress.request_complete.load(Ordering::SeqCst) >= round_number
-                                || progress.replied.load(Ordering::SeqCst) >= round_number
-                        });
-                        let not_yet_replied =
-                            progress.replied.load(Ordering::SeqCst) < round_number;
+                        let not_yet_fully_replied =
+                            progress.reply_parts_written.load(Ordering::SeqCst)
+                                < reply_parts_through_this_round;
                         drop(probe);
-                        break not_yet_replied;
+                        break not_yet_fully_replied;
                     }
                     Err(std::sync::TryLockError::WouldBlock) => {
-                        if progress.request_complete.load(Ordering::SeqCst) >= round_number {
-                            let seen_at = *request_seen_complete_at
-                                .get_or_insert_with(std::time::Instant::now);
-                            if seen_at.elapsed() >= GRACE_AFTER_THE_REQUEST_IS_COMPLETE {
-                                progress.reply_allowed.store(round_number, Ordering::SeqCst);
+                        let reply_parts_allowed =
+                            progress.reply_parts_allowed.load(Ordering::SeqCst);
+                        let is_the_counterpart_at_the_next_gate =
+                            if reply_parts_allowed == reply_parts_before_this_round {
+                                progress.request_complete.load(Ordering::SeqCst) >= round_number
+                            } else {
+                                progress.reply_parts_written.load(Ordering::SeqCst)
+                                    >= reply_parts_allowed
+                            };
+                        if reply_parts_allowed < reply_parts_through_this_round
+                            && is_the_counterpart_at_the_next_gate
+                        {
+                            let seen_at = *step_seen_at.get_or_insert_with(std::time::Instant::now);
+                            if seen_at.elapsed() >= GRACE_AFTER_EACH_STEP {
+                                progress
+                                    .reply_parts_allowed
+                                    .store(reply_parts_allowed + 1, Ordering::SeqCst);
+                                step_seen_at = None;
                             }
                         }
                     }
@@ -874,14 +1035,16 @@
                     }
                 }
             };
-            progress.reply_allowed.store(round_number, Ordering::SeqCst);
+            progress
+                .reply_parts_allowed
+                .store(reply_parts_through_this_round, Ordering::SeqCst);
             assert_eq!(
-                answers_received.recv().expect("装置线程交回读的结果"),
-                None,
-                "第 {round_number} 轮替身回 absent，读成键不在"
+                answers_received.recv().expect("装置线程交回口子交回的"),
+                expected_answer,
+                "第 {round_number} 轮口子照替身的回应交回"
             );
-            if lock_was_free_between_request_and_reply {
-                round_where_the_lock_was_free_between_request_and_reply = Some(round_number);
+            if lock_was_free_in_the_middle {
+                round_where_the_lock_was_free = Some(round_number);
                 break;
             }
         }
@@ -890,10 +1053,80 @@
         drop(device_library);
         counterpart
             .join()
-            .expect("替身线程核过每一问是整条 read、读到请求管道的文件尾收尾");
+            .expect("替身线程核过每一问整条与预期相同、读到请求管道的文件尾收尾");
+        round_where_the_lock_was_free
+    }
+
+    #[test]
+    fn a_read_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
+        let key = format!("items/{}", "k".repeat(PROBED_REQUEST_PAYLOAD_IN_BYTES));
+        let expected_request = format!("read {key}\n").into_bytes();
         assert_eq!(
-            round_where_the_lock_was_free_between_request_and_reply, None,
+            round_where_the_lock_was_free_in_the_middle_of_the_exchange(ProbedExchange {
+                ask: Box::new(move |device_library| device_library.read(&key)),
+                expected_request,
+                reply_parts: vec![b"absent\n".as_slice()],
+                expected_answer: None,
+            }),
+            None,
             "探锁的一方抢到锁时替身已读全这一问、还没回：锁在请求与回应之间放过了"
         );
     }
+
+    #[test]
+    fn a_read_answered_found_holds_the_relay_lock_until_taking_the_contents() {
+        let key = format!("items/{}", "k".repeat(PROBED_REQUEST_PAYLOAD_IN_BYTES));
+        let expected_request = format!("read {key}\n").into_bytes();
+        assert_eq!(
+            round_where_the_lock_was_free_in_the_middle_of_the_exchange(ProbedExchange {
+                ask: Box::new(move |device_library| device_library.read(&key)),
+                expected_request,
+                reply_parts: vec![b"found 5\n".as_slice(), b"hello".as_slice()],
+                expected_answer: Some("hello".to_string()),
+            }),
+            None,
+            "探锁的一方抢到锁时替身回了 found 那一行、还没写内容：锁在 found 那一行与内容之间放过了"
+        );
+    }
+
+    #[test]
+    fn a_write_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
+        let text = "v".repeat(PROBED_REQUEST_PAYLOAD_IN_BYTES);
+        let expected_request =
+            format!("write items/a.out {PROBED_REQUEST_PAYLOAD_IN_BYTES}\n{text}").into_bytes();
+        assert_eq!(
+            round_where_the_lock_was_free_in_the_middle_of_the_exchange(ProbedExchange {
+                ask: Box::new(move |device_library| {
+                    device_library.write("items/a.out", &text);
+                    None
+                }),
+                expected_request,
+                reply_parts: vec![b"written\n".as_slice()],
+                expected_answer: None,
+            }),
+            None,
+            "探锁的一方抢到锁时替身已读全这一问、还没回：锁在写请求与 written 之间放过了"
+        );
+    }
+
+    #[test]
+    fn a_write_many_holds_the_relay_lock_from_sending_the_request_until_taking_the_reply() {
+        let text = "v".repeat(PROBED_REQUEST_PAYLOAD_IN_BYTES);
+        let expected_request =
+            format!("write-many 1\nitems/a.out {PROBED_REQUEST_PAYLOAD_IN_BYTES}\n{text}")
+                .into_bytes();
+        assert_eq!(
+            round_where_the_lock_was_free_in_the_middle_of_the_exchange(ProbedExchange {
+                ask: Box::new(move |device_library| {
+                    device_library.write_many(&[("items/a.out".to_string(), text.clone())]);
+                    None
+                }),
+                expected_request,
+                reply_parts: vec![b"written\n".as_slice()],
+                expected_answer: None,
+            }),
+            None,
+            "探锁的一方抢到锁时替身已读全这一问、还没回：锁在成批写请求与 written 之间放过了"
+        );
+    }
 }
```
