//! 第 5 次跑的判定库口子：判定组、代码键、几种判定库与按键复用。

use super::*;

/// 判定单位按哪一组判法算键：键 = 这一组判法的代码键 ‖ 单位的输入文本（被测代码与参数在代码键与输入文本里）。
/// 分组是为了「改闭包里一个单元只现判受影响的」：只在抛弃步里用到的函数变了，只动 `Abandonment` 一组的键。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JudgementGroup {
    /// 抛弃步：节点镜像 → 构造成立与否、被抛弃根集合。
    Abandonment,
    /// 孪生：pool_1 上不注入的挂载加续写。
    Twin,
    /// 账读不出的变体（树表、分配记录树 × 两份、单份 × 两种形态）。
    AccountUnreadable,
    /// 根槽读不出的变体。
    RootSlotUnreadable,
}

impl JudgementGroup {
    pub(super) fn name(self) -> &'static str {
        match self {
            JudgementGroup::Abandonment => "abandonment",
            JudgementGroup::Twin => "twin",
            JudgementGroup::AccountUnreadable => "account_unreadable",
            JudgementGroup::RootSlotUnreadable => "root_slot_unreadable",
        }
    }
}

/// 每一组判法的代码键（起这个装置的 `device-code-key.py` 给的组成清单里各入口的代码键；没连判定库时是 `unconnected`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JudgementCodeKeys {
    pub(super) abandonment: String,
    pub(super) twin: String,
    pub(super) account_unreadable: String,
    pub(super) root_slot_unreadable: String,
}

impl JudgementCodeKeys {
    pub(super) fn unconnected() -> Self {
        JudgementCodeKeys {
            abandonment: "unconnected".to_string(),
            twin: "unconnected".to_string(),
            account_unreadable: "unconnected".to_string(),
            root_slot_unreadable: "unconnected".to_string(),
        }
    }

    fn for_group(&self, group: JudgementGroup) -> &str {
        match group {
            JudgementGroup::Abandonment => &self.abandonment,
            JudgementGroup::Twin => &self.twin,
            JudgementGroup::AccountUnreadable => &self.account_unreadable,
            JudgementGroup::RootSlotUnreadable => &self.root_slot_unreadable,
        }
    }
}

/// 判定库的口子：经 prune-server 的那一份库按键读写文本值。读写没成交 `Err`（不当「库里没有」）。
/// 实际接线由起装置的一侧给（报告「判定入库」一节）；单测用 [`MemoryJudgementLibrary`]。
pub(super) trait JudgementLibrary: Sync {
    fn read_judgement(&self, key: &str) -> Result<Option<String>, String>;
    /// 一批判定一次写进库：一个工作单位判完调一次（[`process_unit`]），空的一批不算写。
    fn write_judgements(&self, judgements: &JudgementsAwaitingWrite) -> Result<(), String>;
    /// 这个库口子在读写上花的线程时间（只给标准错误的一行用，不进产物）；不计时的口子交 None。
    fn timing_summary(&self) -> Option<String> {
        None
    }
}

/// 一个工作单位现判出、还没写进库的判定：库里的键 → 值文本。按键去重，同一单位里同一把键只写一条。
/// 判定只在这个单位整个判完才写：被打断重跑只损失正在做的单位（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）。
pub(super) type JudgementsAwaitingWrite = BTreeMap<String, String>;

/// 没连判定库：什么都读不到、写进去的丢掉；每个单位都现判。
pub(super) struct DisconnectedJudgementLibrary;

impl JudgementLibrary for DisconnectedJudgementLibrary {
    fn read_judgement(&self, _key: &str) -> Result<Option<String>, String> {
        Ok(None)
    }

    fn write_judgements(&self, _judgements: &JudgementsAwaitingWrite) -> Result<(), String> {
        Ok(())
    }
}

/// 合并那一趟许现判的个数：转交那一侧的复用哨兵每一趟让头 8 条命中里的一条回「不在」
/// （scripts/experiment-apparatus/device-code-key.py 文件头「判定库的转交」复用哨兵），装置现判它、写回，转交核写回的与库里的相同。
pub(super) const REUSE_ONLY_FRESH_JUDGEMENTS_ALLOWED: usize = 1;

/// 合并用的判定库口子：读不到就报错，不当「库里没有」交回去现判；只有 `fresh_allowed` 个例外（转交哨兵让回「不在」的那一条）。
/// 片没跑全、或代码键与片那一趟对不上，合并在超出例外的那一条上当场停，不会悄悄现判出一个与分片无关的结果。
pub(super) struct ReuseOnlyJudgementLibrary<'inner> {
    pub(super) inner: &'inner dyn JudgementLibrary,
    fresh_allowed: std::sync::atomic::AtomicUsize,
}

impl<'inner> ReuseOnlyJudgementLibrary<'inner> {
    pub(super) fn over(inner: &'inner dyn JudgementLibrary, fresh_allowed: usize) -> Self {
        ReuseOnlyJudgementLibrary {
            inner,
            fresh_allowed: std::sync::atomic::AtomicUsize::new(fresh_allowed),
        }
    }
}

impl JudgementLibrary for ReuseOnlyJudgementLibrary<'_> {
    fn read_judgement(&self, key: &str) -> Result<Option<String>, String> {
        match self.inner.read_judgement(key)? {
            Some(stored) => Ok(Some(stored)),
            None => match self.fresh_allowed.fetch_update(
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
                |left| left.checked_sub(1),
            ) {
                Ok(_) => Ok(None),
                Err(_) => Err(format!(
                    "库里没有判定 {key}：合并只取库，不现判；片没跑全或代码键对不上"
                )),
            },
        }
    }

    fn write_judgements(&self, judgements: &JudgementsAwaitingWrite) -> Result<(), String> {
        self.inner.write_judgements(judgements)
    }

    fn timing_summary(&self) -> Option<String> {
        self.inner.timing_summary()
    }
}

/// 内存里的判定库，只给单测核「复用三问」用，进程一退就没了。
#[cfg(test)]
#[derive(Default)]
pub(super) struct MemoryJudgementLibrary {
    pub(super) rows: std::sync::Mutex<BTreeMap<String, String>>,
}

#[cfg(test)]
impl JudgementLibrary for MemoryJudgementLibrary {
    fn read_judgement(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.rows.lock().expect("内存判定库的锁").get(key).cloned())
    }

    fn write_judgements(&self, judgements: &JudgementsAwaitingWrite) -> Result<(), String> {
        self.rows
            .lock()
            .expect("内存判定库的锁")
            .extend(judgements.clone());
        Ok(())
    }
}

#[cfg(test)]
impl MemoryJudgementLibrary {
    /// 删掉键序最小的一条判定，交回它的键（单测造「缺一个判定」用）。
    pub(super) fn remove_first_judgement(&self) -> String {
        let mut rows = self.rows.lock().expect("内存判定库的锁");
        let key = rows.keys().next().expect("库里有判定").clone();
        rows.remove(&key);
        key
    }
}

/// 组成清单第一行（device-code-key.py 文件头「组成清单」）。
pub(super) const COMPOSITION_FORM_LINE: &str = "form\tdevice-code-key-v1";

/// 四组判法的入口，次序与 [`JudgementGroup`] 的声明次序一致；起装置那一条 device-code-key.py 的 `--entry` 按这四个原样给。
pub(super) const JUDGEMENT_ENTRIES: [(&str, JudgementGroup); 4] = [
    (
        "singlefs_implementation_experiments::abandonment_judgement_entry",
        JudgementGroup::Abandonment,
    ),
    (
        "singlefs_implementation_experiments::twin_judgement_entry",
        JudgementGroup::Twin,
    ),
    (
        "singlefs_implementation_experiments::account_unreadable_judgement_entry",
        JudgementGroup::AccountUnreadable,
    ),
    (
        "singlefs_implementation_experiments::root_slot_unreadable_judgement_entry",
        JudgementGroup::RootSlotUnreadable,
    ),
];

impl JudgementCodeKeys {
    /// 读 device-code-key.py 给的组成清单：第一行是 form，四个入口各恰好一行 code_key（键是十六进制），多一行、少一行、认不得的入口都报错。
    pub(super) fn from_composition(composition: &str) -> Result<Self, String> {
        if composition.lines().next() != Some(COMPOSITION_FORM_LINE) {
            return Err("--code-key-file 不是 device-code-key.py 打的组成清单（第一行不是 form\tdevice-code-key-v1）".to_string());
        }
        let mut found: BTreeMap<&str, &str> = BTreeMap::new();
        for line in composition.lines() {
            let Some(rest) = line.strip_prefix("code_key\t") else {
                continue;
            };
            let Some((entry, code_key)) = rest.split_once('\t') else {
                return Err(format!("组成清单里的 code_key 行缺字段：{line}"));
            };
            if !JUDGEMENT_ENTRIES.iter().any(|(known, _)| *known == entry) {
                return Err(format!("组成清单里有认不得的入口 {entry}"));
            }
            if code_key.is_empty() || !code_key.chars().all(|digit| digit.is_ascii_hexdigit()) {
                return Err(format!(
                    "组成清单里 {entry} 的代码键不是十六进制：{code_key}"
                ));
            }
            if found.insert(entry, code_key).is_some() {
                return Err(format!("组成清单里入口 {entry} 不止一行 code_key"));
            }
        }
        let key_of = |group: JudgementGroup| -> Result<String, String> {
            let (entry, _) = JUDGEMENT_ENTRIES
                .iter()
                .find(|(_, candidate)| *candidate == group)
                .expect("四组各有入口");
            found
                .get(entry)
                .map(|code_key| (*code_key).to_string())
                .ok_or_else(|| format!("组成清单里缺入口 {entry} 的 code_key 行"))
        };
        Ok(JudgementCodeKeys {
            abandonment: key_of(JudgementGroup::Abandonment)?,
            twin: key_of(JudgementGroup::Twin)?,
            account_unreadable: key_of(JudgementGroup::AccountUnreadable)?,
            root_slot_unreadable: key_of(JudgementGroup::RootSlotUnreadable)?,
        })
    }

    /// 四把键去重后的十六进制（存组成清单用：`codes/<代码键>.txt`）。
    fn distinct(&self) -> BTreeSet<&str> {
        [
            self.abandonment.as_str(),
            self.twin.as_str(),
            self.account_unreadable.as_str(),
            self.root_slot_unreadable.as_str(),
        ]
        .into_iter()
        .collect()
    }

    /// 写进 `name=r5_run_parameters` 行的样子。
    fn describe(&self) -> String {
        format!(
            "abandonment:{} twin:{} account_unreadable:{} root_slot_unreadable:{}",
            self.abandonment, self.twin, self.account_unreadable, self.root_slot_unreadable
        )
    }
}

/// 经 `device-code-key.py run` 转给 prune-server 那一份判定库（族 apparatus-items，种类 = bin 名）：
/// 一个判定单位一条 `items/<单位键>.out`，值 = 一行头（带单位键，读回时核）加判定值文本。
/// 读写没成由 [`DeviceLibrary`] 照退出码退（75 = prune-server 不可用），不退回本地、不当「库里没有」。
pub(super) struct DeviceJudgementLibrary {
    pub(super) store: DeviceLibrary,
    pub(super) timing: LibraryTiming,
}

/// 各线程在判定库读写上花的时间（读写共用一条管道、整问整答握一把锁，花的时间含等锁）。
#[derive(Default)]
pub(super) struct LibraryTiming {
    pub(super) read_calls: std::sync::atomic::AtomicU64,
    read_nanoseconds: std::sync::atomic::AtomicU64,
    pub(super) write_calls: std::sync::atomic::AtomicU64,
    write_nanoseconds: std::sync::atomic::AtomicU64,
}

impl LibraryTiming {
    pub(super) fn note(
        calls: &std::sync::atomic::AtomicU64,
        nanoseconds: &std::sync::atomic::AtomicU64,
        elapsed: std::time::Duration,
    ) {
        calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        nanoseconds.fetch_add(
            u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX),
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    pub(super) fn summary(&self) -> String {
        let load = |counter: &std::sync::atomic::AtomicU64| {
            counter.load(std::sync::atomic::Ordering::Relaxed)
        };
        format!(
            "library_read_calls={} library_read_thread_milliseconds={} library_write_calls={} library_write_thread_milliseconds={}",
            load(&self.read_calls),
            load(&self.read_nanoseconds) / 1_000_000,
            load(&self.write_calls),
            load(&self.write_nanoseconds) / 1_000_000
        )
    }
}

impl DeviceJudgementLibrary {
    fn item_path(key: &str) -> String {
        format!("items/{key}.out")
    }

    fn item_header(key: &str) -> String {
        format!("key judgement_key={key}")
    }
}

impl JudgementLibrary for DeviceJudgementLibrary {
    fn read_judgement(&self, key: &str) -> Result<Option<String>, String> {
        let asked_at = std::time::Instant::now();
        let stored = self.store.read(&Self::item_path(key));
        LibraryTiming::note(
            &self.timing.read_calls,
            &self.timing.read_nanoseconds,
            asked_at.elapsed(),
        );
        let Some(stored) = stored else {
            return Ok(None);
        };
        let mismatch = || {
            format!(
                "库里 {} 的头对不上这把键（读到的头：{}）",
                Self::item_path(key),
                stored.lines().next().unwrap_or("")
            )
        };
        let (header, body) = stored.split_once('\n').ok_or_else(mismatch)?;
        if header != Self::item_header(key) {
            return Err(mismatch());
        }
        Ok(Some(body.to_string()))
    }

    fn write_judgements(&self, judgements: &JudgementsAwaitingWrite) -> Result<(), String> {
        let items: Vec<(String, String)> = judgements
            .iter()
            .map(|(key, value)| {
                (
                    Self::item_path(key),
                    format!("{}\n{value}", Self::item_header(key)),
                )
            })
            .collect();
        let asked_at = std::time::Instant::now();
        self.store.write_many(&items);
        LibraryTiming::note(
            &self.timing.write_calls,
            &self.timing.write_nanoseconds,
            asked_at.elapsed(),
        );
        Ok(())
    }

    fn timing_summary(&self) -> Option<String> {
        Some(self.timing.summary())
    }
}

/// 一趟用的判定库与它的说明（写进 `name=r5_run_parameters`）。
pub(super) struct FifthRunJudging {
    pub(super) library: Box<dyn JudgementLibrary>,
    pub(super) code_keys: JudgementCodeKeys,
    pub(super) description: String,
}

/// 命令行 `--code-key-file <文件>`：没给交 None；给了没跟文件报错。
fn code_key_file_of(arguments: &[String]) -> Result<Option<std::path::PathBuf>, String> {
    let mut file = None;
    let mut remaining = arguments.iter();
    while let Some(argument) = remaining.next() {
        if argument == "--code-key-file" {
            let value = remaining.next().ok_or("--code-key-file 后面缺文件")?;
            file = Some(std::path::PathBuf::from(value));
        }
    }
    Ok(file)
}

/// 接判定库：组成清单读成四把代码键，`codes/<代码键>.txt` 存清单原文（只给键对不上时查变了哪些单元），库口子来自转交的管道。
/// 没给 `--code-key-file`、清单不对、环境里没有管道（没经 device-code-key.py run 起）都报错，不退回本地。
pub(super) fn connect_judgements(
    arguments: &[String],
    store: Result<DeviceLibrary, String>,
) -> Result<FifthRunJudging, String> {
    let Some(file) = code_key_file_of(arguments)? else {
        return Err("没给 --code-key-file：读写判定库要经 scripts/experiment-apparatus/device-code-key.py run 起".to_string());
    };
    let composition = std::fs::read_to_string(&file)
        .map_err(|error| format!("读不了 --code-key-file {}：{error}", file.display()))?;
    let code_keys = JudgementCodeKeys::from_composition(&composition)?;
    let store = store?;
    for code_key in code_keys.distinct() {
        store.write(&format!("codes/{code_key}.txt"), &composition);
    }
    let description = format!(
        "judgement_library=prune-server-apparatus-items code_keys=[{}]",
        code_keys.describe()
    );
    Ok(FifthRunJudging {
        library: Box::new(DeviceJudgementLibrary {
            store,
            timing: LibraryTiming::default(),
        }),
        code_keys,
        description,
    })
}

/// 判定单位的复用：键相同的读库里的值，不同的现判、记进这个工作单位待写的那一批（[`process_unit`] 判完才一次写进库）。
pub(super) struct ReusableJudgements<'library> {
    pub(super) library: &'library dyn JudgementLibrary,
    pub(super) code_keys: JudgementCodeKeys,
}

impl ReusableJudgements<'_> {
    /// 库里的键：代码键与单位输入文本的 SHA-256。
    fn stored_key(&self, group: JudgementGroup, key_text: &str) -> String {
        let material = format!(
            "{}\n{}\n{key_text}",
            group.name(),
            self.code_keys.for_group(group)
        );
        singlefs_unit_graph::sha256::sha256_of(material.as_bytes()).hexadecimal()
    }

    /// 交回 (值文本, 是不是复用来的)。本单位待写的那一批或库里有就不调 `judge`；
    /// 本单位已判、还没写进库的键算复用，现判与复用的个数不随写的时机变。
    pub(super) fn judge_or_reuse(
        &self,
        awaiting_write: &mut JudgementsAwaitingWrite,
        group: JudgementGroup,
        key_text: &str,
        judge: impl FnOnce() -> Result<String, String>,
    ) -> Result<(String, bool), String> {
        let key = self.stored_key(group, key_text);
        if let Some(judged_in_this_unit) = awaiting_write.get(&key) {
            return Ok((judged_in_this_unit.clone(), true));
        }
        if let Some(stored) = self.library.read_judgement(&key)? {
            return Ok((stored, true));
        }
        let judged = judge()?;
        awaiting_write.insert(key, judged.clone());
        Ok((judged, false))
    }
}
