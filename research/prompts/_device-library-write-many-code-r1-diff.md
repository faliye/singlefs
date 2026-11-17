# device-library-write-many-code-r1：判定库口子成批写的改动（2026-11-10）

来源：`git diff HEAD -- crates/singlefs-device-library/src/lib.rs`（此刻主工作区对 HEAD）。

```diff
diff --git a/crates/singlefs-device-library/src/lib.rs b/crates/singlefs-device-library/src/lib.rs
index beab5e1ae..9db983f14 100644
--- a/crates/singlefs-device-library/src/lib.rs
+++ b/crates/singlefs-device-library/src/lib.rs
@@ -5,13 +5,16 @@
 // 协议一问一答，几个线程共用一条，整问整答在一把锁里：
 //   请求 read <相对键>\n                    回 found <字节数>\n<内容>，或 absent\n
 //   请求 write <相对键> <字节数>\n<内容>    回 written\n（服务确认落盘之后才回）
-//   两种都可能回 failed <退出码> <一句>\n：这里打 ✗ 与那一句、进程照那个退出码退（75 是 prune-server 不可用，3 是别的错），不当「键不在」现判。
-//   相对键只用字母、数字与 . _ - /，不带空白（那一侧照这一条核，不合的回 failed 3）。管道断了、回应认不出退 3。
+//   请求 write-many <条数>\n 后接每条 <相对键> <字节数>\n<内容>    整批回一个 written\n（服务确认整批落盘之后才回）：
+//     一条请求里的写在服务那一头合成一次提交，整问整答在锁里，别的线程只等这一条；条数至少 1。
+//   三种都可能回 failed <退出码> <一句>\n：这里打 ✗ 与那一句、进程照那个退出码退（75 是 prune-server 不可用，3 是别的错），不当「键不在」现判。
+//   相对键只用字母、数字与 . _ - /，不带空白（那一侧照这一条核，不合的回 failed 3，整批一条都不写）。管道断了、回应认不出退 3。
 // 撤掉的项（那一侧核出编译与跑之间源码变了时改写成撤回标记的）由那一侧回 absent，这里看不到标记。
 // 每个用到它的装置在自己的 Cargo.toml 里按 path 依赖引本包（crates/singlefs-device-library），再 `use singlefs_device_library::DeviceLibrary;`
 // （从仓根算单元键时 unit-graph 看不到 `#[path]` 引进来的文件，键拒算；条目级 include! 同样不收）。
 // 装置的单测换成 `DeviceLibrary::in_memory()`（内存那一种，测完即丢，不落盘）：它不挂 cfg、也不挂特性，一律编（本包的 Cargo.toml 写了为什么不设特性），
 // 装置只在单测里调它；转交的整条路由 device-code-key.py --selftest 在临时服务底下核。
+#![forbid(unsafe_code)]
 
 use std::io::{BufRead, Read, Write};
 
@@ -27,9 +30,16 @@ struct RelayEnds {
     replies: std::io::BufReader<std::fs::File>,
 }
 
+/// 单测的内存那一种：存的条目，与成批写发过几次（单测数「一批一次」用，真管道那一头是请求条数）。
+#[derive(Default)]
+struct MemoryLibrary {
+    entries: std::collections::BTreeMap<String, String>,
+    write_many_calls: usize,
+}
+
 enum Backend {
     Relay(std::sync::Mutex<RelayEnds>),
-    Memory(std::sync::Mutex<std::collections::BTreeMap<String, String>>),
+    Memory(std::sync::Mutex<MemoryLibrary>),
 }
 
 /// 一个装置这一趟的判定库口子：克隆的几份共用同一条管道（判等按是不是同一条）。
@@ -93,6 +103,20 @@ fn give_up_on_reply(reply: &str, asked: &str) -> ! {
     )
 }
 
+/// 整条写请求发出去，等回一个 `written`；写不进管道、回的不是 `written` 都照退出码退（文件头「协议」）。
+fn send_write_request_and_wait_for_written(ends: &mut RelayEnds, request: &[u8], asked: &str) {
+    if let Err(error) = ends.requests.write_all(request) {
+        give_up(
+            RELAY_BROKEN_EXIT_CODE,
+            &format!("{asked}：写不进转交判定库的管道：{error}"),
+        );
+    }
+    let reply = reply_line(&mut ends.replies);
+    if reply != "written" {
+        give_up_on_reply(&reply, asked);
+    }
+}
+
 impl DeviceLibrary {
     /// 照 device-code-key.py run 放进环境的两个描述符号接上转交的管道；没有、写错或打不开交一句为什么（调用方照用法错办）。
     pub fn from_environment() -> Result<DeviceLibrary, String> {
@@ -125,7 +149,7 @@ impl DeviceLibrary {
     pub fn in_memory() -> DeviceLibrary {
         DeviceLibrary {
             backend: std::sync::Arc::new(Backend::Memory(std::sync::Mutex::new(
-                std::collections::BTreeMap::new(),
+                MemoryLibrary::default(),
             ))),
         }
     }
@@ -164,9 +188,10 @@ impl DeviceLibrary {
                 }
                 String::from_utf8(contents).ok()
             }
-            Backend::Memory(entries) => entries
+            Backend::Memory(library) => library
                 .lock()
                 .expect("单测内存库的锁没有中毒")
+                .entries
                 .get(relative)
                 .cloned(),
         }
@@ -180,38 +205,70 @@ impl DeviceLibrary {
                 let asked = format!("写 {relative}");
                 let mut request = format!("write {relative} {}\n", text.len()).into_bytes();
                 request.extend_from_slice(text.as_bytes());
-                if let Err(error) = ends.requests.write_all(&request) {
-                    give_up(
-                        RELAY_BROKEN_EXIT_CODE,
-                        &format!("{asked}：写不进转交判定库的管道：{error}"),
-                    );
-                }
-                let reply = reply_line(&mut ends.replies);
-                if reply != "written" {
-                    give_up_on_reply(&reply, &asked);
-                }
+                send_write_request_and_wait_for_written(&mut ends, &request, &asked);
             }
-            Backend::Memory(entries) => {
-                entries
+            Backend::Memory(library) => {
+                library
                     .lock()
                     .expect("单测内存库的锁没有中毒")
+                    .entries
                     .insert(relative.to_string(), text.to_string());
             }
         }
     }
 
+    /// 一次写一批（相对键, 内容），服务确认整批落盘之后才回来；写失败照退出码退。空的一批不发请求。
+    pub fn write_many(&self, entries: &[(String, String)]) {
+        if entries.is_empty() {
+            return;
+        }
+        match &*self.backend {
+            Backend::Relay(ends) => {
+                let mut ends = ends.lock().expect("转交判定库的锁没有中毒");
+                let asked = format!("成批写 {} 条", entries.len());
+                let mut request = format!("write-many {}\n", entries.len()).into_bytes();
+                for (relative, text) in entries {
+                    request.extend_from_slice(format!("{relative} {}\n", text.len()).as_bytes());
+                    request.extend_from_slice(text.as_bytes());
+                }
+                send_write_request_and_wait_for_written(&mut ends, &request, &asked);
+            }
+            Backend::Memory(library) => {
+                let mut library = library.lock().expect("单测内存库的锁没有中毒");
+                library.write_many_calls += 1;
+                for (relative, text) in entries {
+                    library.entries.insert(relative.clone(), text.clone());
+                }
+            }
+        }
+    }
+
     /// 单测数条数：以 prefix 起头、以 suffix 收尾的键有几条。
     pub fn count_entries(&self, prefix: &str, suffix: &str) -> usize {
         match &*self.backend {
             Backend::Relay(_) => panic!("单测只数内存那一种"),
-            Backend::Memory(entries) => entries
+            Backend::Memory(library) => library
                 .lock()
                 .expect("单测内存库的锁没有中毒")
+                .entries
                 .keys()
                 .filter(|key| key.starts_with(prefix) && key.ends_with(suffix))
                 .count(),
         }
     }
+
+    /// 单测数次数：非空的成批写发过几次。
+    pub fn count_write_many_calls(&self) -> usize {
+        match &*self.backend {
+            Backend::Relay(_) => panic!("单测只数内存那一种"),
+            Backend::Memory(library) => {
+                library
+                    .lock()
+                    .expect("单测内存库的锁没有中毒")
+                    .write_many_calls
+            }
+        }
+    }
 }
 
 #[cfg(test)]
@@ -300,4 +357,79 @@ mod tests {
             "write 请求报的是内容的字节数（两个汉字六个字节），后面紧跟内容"
         );
     }
+
+    /// 回应管道里这一刻没读走的全部字节（预先写好的回应写端已关，读得到文件尾）。
+    fn replies_left_unread(device_library: &DeviceLibrary) -> Vec<u8> {
+        match &*device_library.backend {
+            Backend::Relay(ends) => {
+                let mut left = Vec::new();
+                ends.lock()
+                    .expect("单测里没有别的线程握这把锁")
+                    .replies
+                    .read_to_end(&mut left)
+                    .expect("回应管道读得到文件尾");
+                left
+            }
+            Backend::Memory(_) => panic!("只数转交那一种的回应"),
+        }
+    }
+
+    #[test]
+    fn write_many_announces_the_count_and_each_length_in_bytes_and_takes_one_written_for_the_whole_batch(
+    ) {
+        let counterpart = relay_with_prepared_replies(b"written\nnot-for-this-batch\n");
+        counterpart.device_library.write_many(&[
+            ("items/multibyte.out".to_string(), "判定".to_string()),
+            ("items/ascii.out".to_string(), "ok".to_string()),
+        ]);
+        assert_eq!(
+            replies_left_unread(&counterpart.device_library),
+            b"not-for-this-batch\n",
+            "整批恰好读走一个 written，不多读也不少读"
+        );
+        assert_eq!(
+            requests_after_the_device_is_dropped(counterpart),
+            "write-many 2\nitems/multibyte.out 6\n判定items/ascii.out 2\nok",
+            "头一行报条数；每条报内容的字节数（两个汉字六个字节），后面紧跟内容"
+        );
+    }
+
+    #[test]
+    fn write_many_of_nothing_sends_no_request_and_takes_no_reply() {
+        let counterpart = relay_with_prepared_replies(b"written\n");
+        counterpart.device_library.write_many(&[]);
+        assert_eq!(
+            replies_left_unread(&counterpart.device_library),
+            b"written\n",
+            "空的一批不等回应"
+        );
+        assert_eq!(
+            requests_after_the_device_is_dropped(counterpart),
+            "",
+            "空的一批不发请求"
+        );
+    }
+
+    #[test]
+    fn write_many_in_memory_stores_every_entry_and_counts_one_call_per_non_empty_batch() {
+        let device_library = DeviceLibrary::in_memory();
+        device_library.write_many(&[
+            ("items/one.out".to_string(), "甲".to_string()),
+            ("items/two.out".to_string(), "乙".to_string()),
+        ]);
+        device_library.write_many(&[]);
+        device_library.write("codes/key.txt", "清单");
+        assert_eq!(device_library.read("items/one.out").as_deref(), Some("甲"));
+        assert_eq!(
+            device_library.read("items/two.out").as_deref(),
+            Some("乙"),
+            "批里每一条都存下，不止头一条"
+        );
+        assert_eq!(device_library.count_entries("items/", ".out"), 2);
+        assert_eq!(
+            device_library.count_write_many_calls(),
+            1,
+            "非空的一批记一次，空的一批与单条写不记"
+        );
+    }
 }
```
