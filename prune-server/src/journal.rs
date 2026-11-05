//! 客户端的本地追加日志：没收到确认的写帧记在这里，断线重连、服务重起、客户端进程重起之后按请求号次序重放；
//! 另记用过的最大请求号（高水位），重开之后接着往上分，一个请求号不分第二次。
//!
//! 文件格式：开头 8 字节 `PRUNEJN2`，之后一条接一条的记录 `[u8 种类][u32 内容长][内容][u32 校验]`（小端，校验是种类、
//! 内容长、内容三段的 FNV-1a 32 位）。种类：1 客户端身份（只在开头一条）；2 写帧（`u64 请求号` 加整帧字节，帧里的序号不作数，
//! 重放时换新序号）；3 已确认（`u64 请求号`）；4 请求号高水位（`u64 分出去过的最大请求号`，整理时写在身份之后）。
//! 没确认的写帧 = 写帧记录减去已确认记录。重开时下一个请求号 = 高水位记录、写帧记录、已确认记录里最大的那个号加一：
//! 整理只留没确认的写帧，已确认的号只靠高水位记录留下来；不留的话重开之后从小号分起，服务按（身份，请求号）把新写当成
//! 早已生效的重发，回了确认却一条都没写（弄坏开关 journal-compaction-drops-high-water 退回整理不写高水位那一种）。
//! 开头不是 `PRUNEJN2` 的一律拒开：认不得的日志凭它算不出分出去过哪些请求号；出路是删掉它（里面没确认的写帧随之作废，
//! 重交那一趟任务重新算）或换一个日志路径。
//! 写帧在发出去之前追加；确认到了追加一条已确认。只 `write` 进文件、不 `fsync`：日志防的是服务丢帧与网络断，
//! 客户端这台机器整机掉电时它手里还没算完的活本来就要重做；客户端进程崩了，写进文件的内容内核照样留着。
//! 文件超过整理门槛时整理：只留身份、高水位与没确认的写帧，写进 `<日志>.compacting`、同步、改名换上。整理门槛取
//! `compaction_bytes` 与上次整理完长度的两倍里大的那个：没确认的帧本身就超过 `compaction_bytes` 时（服务慢了、断了，
//! 在途窗口里积了几百帧大帧），不会每来一个确认就把它们整份重写一遍再同步；两次整理之间至少追加上次整理完的长度那么多，
//! 每次重写至多是门槛那么长，所以整理重写的字节合计至多是追加字节的两倍再加一个 `compaction_bytes`。
//! 只按 `compaction_bytes` 判的写法在没确认的帧超过它之后每个确认都整理一次，客户端整个时间花在重写与同步上、
//! 不收回应（按压测 2138 带故障那一条的参数复跑时量到的就是这样：客户端线程停在 `<日志>.compacting` 的 fsync 上、
//! 代理转回应的线程堵在写给客户端上；弄坏开关 journal-compacts-on-every-acknowledgement 退回这一种）。
//! 末尾一条写到一半（客户端在追加时崩了）的，打开时按校验认出来截掉：那一帧还没发出去，丢掉它不丢已发的帧。
//! 同一份日志同一时刻只许一个客户端开着：排他锁（`flock`）加在日志旁边的 `<日志>.lock` 上，持锁之后才开、读、整理日志本身。
//! 锁文件从不改名、不删，整理时日志改名换了 inode 也还是同一把锁，第二个打开者不论卡在哪一步都拿不到锁、报错。
//! 锁加在日志本身上挡不住「已经开了旧日志、还没加锁」的第二个打开者：整理改名之后它锁上的是旧 inode，往没人再读的旧文件里追加，
//! 写帧就丢了（弄坏开关 journal-locks-renamed-file 退回这一种）。
//! `flock` 记在打开的那份文件上：同一进程里别的线程这时 fork 出来、还没 exec 的子进程复制着锁文件的描述符（带 close-on-exec 的也在），
//! 到 exec 才放；这一刻关掉日志再开同一份，会被那份复制挡住、报「被别的客户端开着」。这是偏保守的拒，不会放两个客户端同时写。
//! 仓里在一个进程里关掉再开同一份日志、同时别的线程在起子进程的只有自证，自证每格各在自己的进程里（`selftest.rs` 文件头）。

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::client::random_client_identity;

const JOURNAL_MAGIC: &[u8; 8] = b"PRUNEJN2";
const IDENTITY_RECORD: u8 = 1;
const WRITE_RECORD: u8 = 2;
const ACKNOWLEDGED_RECORD: u8 = 3;
const HIGH_WATER_RECORD: u8 = 4;
/// 锁文件：日志路径后面接这一段（文件头）。
const LOCK_FILE_SUFFIX: &str = ".lock";
/// 记录头：种类 1 字节加内容长 4 字节；记录尾：校验 4 字节。
const RECORD_HEADER_BYTES: usize = 5;
const RECORD_CHECKSUM_BYTES: usize = 4;

fn fnv1a(parts: &[&[u8]]) -> u32 {
    let mut hash: u32 = 0x811C_9DC5;
    for part in parts {
        for byte in *part {
            hash ^= u32::from(*byte);
            hash = hash.wrapping_mul(0x0100_0193);
        }
    }
    hash
}

fn encode_record(kind: u8, payload: &[u8]) -> Vec<u8> {
    let length = u32::try_from(payload.len()).expect("一帧不超过帧长上限 64 MiB，放得进 u32");
    let length_bytes = length.to_le_bytes();
    let checksum = fnv1a(&[&[kind], &length_bytes, payload]);
    let mut record =
        Vec::with_capacity(RECORD_HEADER_BYTES + payload.len() + RECORD_CHECKSUM_BYTES);
    record.push(kind);
    record.extend_from_slice(&length_bytes);
    record.extend_from_slice(payload);
    record.extend_from_slice(&checksum.to_le_bytes());
    record
}

/// 一条没确认的写帧。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournaledWrite {
    pub request_number: u64,
    pub frame: Vec<u8>,
}

/// 打开日志的选项：整理门槛与几处弄坏开关（只给自证用，表在 `src/bin/prune_server.rs` 文件头）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalOptions {
    pub compaction_bytes: u64,
    /// 弄坏开关 unacknowledged-frames-not-journaled：什么都不记。
    pub is_disabled: bool,
    /// 弄坏开关 journal-compacts-on-every-acknowledgement：门槛一直是 `compaction_bytes`。
    pub compacts_on_every_acknowledgement: bool,
    /// 弄坏开关 journal-compaction-drops-high-water：整理不写请求号高水位。
    pub drops_high_water_on_compaction: bool,
    /// 弄坏开关 journal-locks-renamed-file：锁加在日志本身上（整理改名会换 inode），不用旁边的锁文件。
    pub locks_renamed_file: bool,
}

impl JournalOptions {
    /// 整理门槛照给的，弄坏开关照 `break_switches` 里打开了的。
    pub fn from_break_switches(
        compaction_bytes: u64,
        break_switches: &BreakSwitches,
    ) -> JournalOptions {
        JournalOptions {
            compaction_bytes,
            is_disabled: break_switches.is_enabled(BreakSwitch::UnacknowledgedFramesNotJournaled),
            compacts_on_every_acknowledgement: break_switches
                .is_enabled(BreakSwitch::JournalCompactsOnEveryAcknowledgement),
            drops_high_water_on_compaction: break_switches
                .is_enabled(BreakSwitch::JournalCompactionDropsHighWater),
            locks_renamed_file: break_switches.is_enabled(BreakSwitch::JournalLocksRenamedFile),
        }
    }
}

pub struct Journal {
    path: PathBuf,
    file: File,
    #[allow(
        dead_code,
        reason = "只为持锁：放手时关掉描述符、放掉 flock；弄坏开关 journal-locks-renamed-file 下锁在 file 上，这里是 None"
    )]
    lock_file: Option<File>,
    length_in_bytes: u64,
    /// 长度超过它就整理：`compaction_bytes` 与上次整理完长度的两倍里大的那个。
    compaction_threshold: u64,
    client_identity: Vec<u8>,
    /// 分出去过的最大请求号（高水位，文件头）；整理时写进新文件。
    highest_request_number: u64,
    options: JournalOptions,
    statistics: JournalStatistics,
}

/// 追加与整理的计数（压测器每一路的末行报，自证核整理的重写量）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JournalStatistics {
    pub appended_bytes: u64,
    pub compactions: u64,
    pub rewritten_bytes: u64,
}

/// 这一版的整理门槛。
fn threshold_after_compaction(compaction_bytes: u64, compacted_length: u64, fixed: bool) -> u64 {
    if fixed {
        compaction_bytes
    } else {
        compaction_bytes.max(compacted_length.saturating_mul(2))
    }
}

/// 打开日志时读出来的东西。
pub struct RecoveredJournal {
    pub journal: Journal,
    pub client_identity: Vec<u8>,
    pub unacknowledged: Vec<JournaledWrite>,
    pub next_request_number: u64,
}

fn lock_exclusively(file: &File, path: &Path) -> Result<(), String> {
    // SAFETY: 描述符来自活着的 File；flock 只收两个整数。
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
        return Err(format!(
            "追加日志 {} 被别的客户端开着（flock：{}）；同一份日志只许一个客户端用",
            path.display(),
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

fn u64_payload(payload: &[u8]) -> Option<u64> {
    Some(u64::from_le_bytes(payload.get(..8)?.try_into().ok()?))
}

/// 日志旁边的锁文件（文件头）：日志路径后面接 `LOCK_FILE_SUFFIX`。
pub fn lock_path_for(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(LOCK_FILE_SUFFIX);
    PathBuf::from(name)
}

impl Journal {
    pub fn open(path: &Path, options: JournalOptions) -> Result<RecoveredJournal, String> {
        Journal::open_pausing_before_lock(path, options, || {})
    }

    /// 同 `open`；要上锁的那个文件开好之后、加锁之前调一次 `pause_before_lock`
    /// （自证 journal-lock-survives-compaction 在这里卡住第二个打开者）。
    pub fn open_pausing_before_lock(
        path: &Path,
        options: JournalOptions,
        pause_before_lock: impl FnOnce(),
    ) -> Result<RecoveredJournal, String> {
        let mut pause_before_lock = Some(pause_before_lock);
        let lock_file = if options.locks_renamed_file {
            None
        } else {
            let lock_path = lock_path_for(path);
            let lock_file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&lock_path)
                .map_err(|error| {
                    format!("打不开追加日志的锁文件 {}：{error}", lock_path.display())
                })?;
            if let Some(pause) = pause_before_lock.take() {
                pause();
            }
            lock_exclusively(&lock_file, path)?;
            Some(lock_file)
        };
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)
            .map_err(|error| format!("打不开追加日志 {}：{error}", path.display()))?;
        if lock_file.is_none() {
            if let Some(pause) = pause_before_lock.take() {
                pause();
            }
            lock_exclusively(&file, path)?;
        }
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|error| format!("读不了追加日志 {}：{error}", path.display()))?;
        if content.is_empty() {
            let client_identity = random_client_identity()?;
            let mut header = JOURNAL_MAGIC.to_vec();
            header.extend_from_slice(&encode_record(IDENTITY_RECORD, &client_identity));
            file.write_all(&header)
                .map_err(|error| format!("写不了追加日志 {}：{error}", path.display()))?;
            let journal = Journal {
                path: path.to_path_buf(),
                file,
                lock_file,
                length_in_bytes: u64::try_from(header.len()).expect("几十字节"),
                compaction_threshold: options.compaction_bytes,
                client_identity: client_identity.clone(),
                highest_request_number: 0,
                options,
                statistics: JournalStatistics::default(),
            };
            return Ok(RecoveredJournal {
                journal,
                client_identity,
                unacknowledged: Vec::new(),
                next_request_number: 1,
            });
        }
        if content.get(..JOURNAL_MAGIC.len()) != Some(JOURNAL_MAGIC.as_slice()) {
            return Err(format!(
                "{} 开头不是 PRUNEJN2，不是这一版客户端的追加日志（认不得的日志凭它算不出分出去过哪些请求号）；\
                 删掉它（里面没确认的写帧随之作废，重交那一趟任务重新算）或换一个日志路径",
                path.display()
            ));
        }
        let mut position = JOURNAL_MAGIC.len();
        let mut client_identity: Option<Vec<u8>> = None;
        let mut writes: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
        let mut highest_request_number = 0_u64;
        while let Some(header) = content.get(position..position + RECORD_HEADER_BYTES) {
            let kind = header[0];
            let length =
                usize::try_from(u32::from_le_bytes(header[1..5].try_into().expect("4 字节")))
                    .expect("u32 放得进 usize");
            let payload_end = position + RECORD_HEADER_BYTES + length;
            let Some(checksum_bytes) =
                content.get(payload_end..payload_end + RECORD_CHECKSUM_BYTES)
            else {
                break;
            };
            let payload = &content[position + RECORD_HEADER_BYTES..payload_end];
            let stored_checksum = u32::from_le_bytes(checksum_bytes.try_into().expect("4 字节"));
            if stored_checksum != fnv1a(&[&[kind], &header[1..5], payload]) {
                break;
            }
            match kind {
                IDENTITY_RECORD if client_identity.is_none() && !payload.is_empty() => {
                    client_identity = Some(payload.to_vec());
                }
                WRITE_RECORD => {
                    let request_number = u64_payload(payload).ok_or_else(|| format!("{} 里有一条写帧记录短于 8 字节", path.display()))?;
                    highest_request_number = highest_request_number.max(request_number);
                    writes.insert(request_number, payload[8..].to_vec());
                }
                ACKNOWLEDGED_RECORD => {
                    let request_number = u64_payload(payload).ok_or_else(|| format!("{} 里有一条已确认记录短于 8 字节", path.display()))?;
                    highest_request_number = highest_request_number.max(request_number);
                    writes.remove(&request_number);
                }
                HIGH_WATER_RECORD => {
                    let request_number = u64_payload(payload).ok_or_else(|| format!("{} 里有一条高水位记录短于 8 字节", path.display()))?;
                    highest_request_number = highest_request_number.max(request_number);
                }
                other => {
                    return Err(format!(
                        "{} 里有认不得的记录种类 {other}（或身份记录不在开头）；删掉它或换一个 --journal 路径",
                        path.display()
                    ))
                }
            }
            position = payload_end + RECORD_CHECKSUM_BYTES;
        }
        let client_identity =
            client_identity.ok_or_else(|| format!("{} 里没有客户端身份记录", path.display()))?;
        let valid_length = u64::try_from(position).expect("文件长度放得进 u64");
        if position < content.len() {
            file.set_len(valid_length).map_err(|error| {
                format!(
                    "截不掉追加日志 {} 末尾写到一半的记录：{error}",
                    path.display()
                )
            })?;
        }
        let unacknowledged = writes
            .into_iter()
            .map(|(request_number, frame)| JournaledWrite {
                request_number,
                frame,
            })
            .collect();
        Ok(RecoveredJournal {
            journal: Journal {
                path: path.to_path_buf(),
                file,
                lock_file,
                length_in_bytes: valid_length,
                compaction_threshold: options.compaction_bytes,
                client_identity: client_identity.clone(),
                highest_request_number,
                options,
                statistics: JournalStatistics::default(),
            },
            client_identity,
            unacknowledged,
            next_request_number: highest_request_number + 1,
        })
    }

    fn append(&mut self, record: &[u8]) -> Result<(), String> {
        self.file
            .write_all(record)
            .map_err(|error| format!("写不了追加日志 {}：{error}", self.path.display()))?;
        let record_bytes = u64::try_from(record.len()).expect("放得进 u64");
        self.length_in_bytes += record_bytes;
        self.statistics.appended_bytes += record_bytes;
        Ok(())
    }

    pub fn statistics(&self) -> JournalStatistics {
        self.statistics
    }

    /// 写帧发出去之前调。
    pub fn append_write(&mut self, request_number: u64, frame: &[u8]) -> Result<(), String> {
        if self.options.is_disabled {
            return Ok(());
        }
        self.highest_request_number = self.highest_request_number.max(request_number);
        let mut payload = Vec::with_capacity(8 + frame.len());
        payload.extend_from_slice(&request_number.to_le_bytes());
        payload.extend_from_slice(frame);
        self.append(&encode_record(WRITE_RECORD, &payload))
    }

    /// 确认到了调；`still_unacknowledged` 是此刻全部没确认的写帧（整理时只留它们）。
    pub fn append_acknowledgement<'frames>(
        &mut self,
        request_number: u64,
        still_unacknowledged: impl Iterator<Item = (u64, &'frames [u8])>,
    ) -> Result<(), String> {
        if self.options.is_disabled {
            return Ok(());
        }
        self.append(&encode_record(
            ACKNOWLEDGED_RECORD,
            &request_number.to_le_bytes(),
        ))?;
        if self.length_in_bytes > self.compaction_threshold {
            self.compact(still_unacknowledged)?;
        }
        Ok(())
    }

    fn compact<'frames>(
        &mut self,
        still_unacknowledged: impl Iterator<Item = (u64, &'frames [u8])>,
    ) -> Result<(), String> {
        let compacting_path = self.path.with_extension("compacting");
        let mut replacement = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .truncate(false)
            .open(&compacting_path)
            .map_err(|error| format!("建不了整理用的 {}：{error}", compacting_path.display()))?;
        if self.options.locks_renamed_file {
            lock_exclusively(&replacement, &compacting_path)?;
        }
        replacement
            .set_len(0)
            .map_err(|error| format!("清不了 {}：{error}", compacting_path.display()))?;
        let mut content = JOURNAL_MAGIC.to_vec();
        content.extend_from_slice(&encode_record(IDENTITY_RECORD, &self.client_identity));
        if !self.options.drops_high_water_on_compaction {
            content.extend_from_slice(&encode_record(
                HIGH_WATER_RECORD,
                &self.highest_request_number.to_le_bytes(),
            ));
        }
        for (request_number, frame) in still_unacknowledged {
            let mut payload = Vec::with_capacity(8 + frame.len());
            payload.extend_from_slice(&request_number.to_le_bytes());
            payload.extend_from_slice(frame);
            content.extend_from_slice(&encode_record(WRITE_RECORD, &payload));
        }
        replacement
            .write_all(&content)
            .and_then(|()| replacement.sync_all())
            .map_err(|error| format!("写不了 {}：{error}", compacting_path.display()))?;
        std::fs::rename(&compacting_path, &self.path).map_err(|error| {
            format!(
                "换不上整理过的追加日志（{} → {}）：{error}",
                compacting_path.display(),
                self.path.display()
            )
        })?;
        self.file = replacement;
        self.length_in_bytes = u64::try_from(content.len()).expect("放得进 u64");
        self.compaction_threshold = threshold_after_compaction(
            self.options.compaction_bytes,
            self.length_in_bytes,
            self.options.compacts_on_every_acknowledgement,
        );
        self.statistics.compactions += 1;
        self.statistics.rewritten_bytes += self.length_in_bytes;
        Ok(())
    }
}
