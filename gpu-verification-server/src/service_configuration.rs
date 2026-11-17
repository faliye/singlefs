//! 服务的配置：原文是这个工程根下的 `service-configuration.txt`（编译时读进来，起服务时解），写法在那份文件头。
//! 只有服务自己的参数（凑批的限时、扫库间隔、认领租期）；机器、卡与额度这类环境参数不在这里，照多机配置
//! （`crate::gpu_judges_on_a_card::OpenedGpuCards::open_the_cards_named_for_this_host`）。

use std::time::Duration;

/// 编进服务的那份配置原文。
pub const SERVICE_CONFIGURATION_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/service-configuration.txt"
));

const OLDEST_PENDING_WAIT_LIMIT_NAME: &str = "OLDEST_PENDING_WAIT_LIMIT_MILLISECONDS";
const LIBRARY_SCAN_INTERVAL_NAME: &str = "LIBRARY_SCAN_INTERVAL_MILLISECONDS";
const CLAIM_LEASE_NAME: &str = "CLAIM_LEASE_MILLISECONDS";

/// 解出来的配置。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceConfiguration {
    /// 最老待判的限时（`crate::batch_scheduling::BatchSchedulingConfiguration::oldest_pending_wait_limit`）。
    pub oldest_pending_wait_limit: Duration,
    /// 隔多久扫一次判定库里的待判行（不是 0）。
    pub library_scan_interval: Duration,
    /// 认领的租期（不是 0，毫秒数装得进 u32）。
    pub claim_lease: Duration,
}

/// 一行「<名>=<值>」解出来的：名、值、第几行。
struct WrittenItem<'text> {
    name: &'text str,
    value: &'text str,
    line_number: usize,
}

/// 逐项各取一次：同一个名写两次、认不得的名都在这里拒。
struct ItemsOfTheText<'text> {
    items: Vec<WrittenItem<'text>>,
}

impl<'text> ItemsOfTheText<'text> {
    fn of_the_text(text: &'text str) -> Result<Self, String> {
        const KNOWN: [&str; 3] = [
            OLDEST_PENDING_WAIT_LIMIT_NAME,
            LIBRARY_SCAN_INTERVAL_NAME,
            CLAIM_LEASE_NAME,
        ];
        let mut items: Vec<WrittenItem<'text>> = Vec::new();
        for (line_index, line) in text.lines().enumerate() {
            let line_number = line_index + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let (name, value) = trimmed
                .split_once('=')
                .ok_or_else(|| format!("配置第 {line_number} 行不是「<名>=<值>」：{trimmed}"))?;
            let name = name.trim();
            if !KNOWN.contains(&name) {
                return Err(format!("配置第 {line_number} 行：认不得的名「{name}」"));
            }
            if items.iter().any(|item| item.name == name) {
                return Err(format!("配置第 {line_number} 行：{name} 写了两次"));
            }
            items.push(WrittenItem {
                name,
                value: value.trim(),
                line_number,
            });
        }
        Ok(Self { items })
    }

    fn value_of(&self, name: &str) -> Result<(&'text str, usize), String> {
        self.items
            .iter()
            .find(|item| item.name == name)
            .map(|item| (item.value, item.line_number))
            .ok_or_else(|| format!("配置少写了 {name}"))
    }

    fn milliseconds_of(&self, name: &str) -> Result<Duration, String> {
        let (value, line_number) = self.value_of(name)?;
        let milliseconds: u64 = value
            .parse()
            .map_err(|_| format!("配置第 {line_number} 行：{name} 的值「{value}」不是非负整数"))?;
        Ok(Duration::from_millis(milliseconds))
    }

    fn positive_milliseconds_of(&self, name: &str) -> Result<Duration, String> {
        let duration = self.milliseconds_of(name)?;
        if duration.is_zero() {
            let (_, line_number) = self.value_of(name)?;
            return Err(format!("配置第 {line_number} 行：{name} 不许是 0"));
        }
        Ok(duration)
    }
}

impl ServiceConfiguration {
    /// 解配置原文。
    ///
    /// # Errors
    /// 有一行不是「<名>=<值>」、名认不得、写了两次、值写法不对、租期或扫库间隔是 0、租期装不进 u32 毫秒、少写了一项：
    /// 交回哪一行、哪里不对。
    pub fn from_the_text(text: &str) -> Result<Self, String> {
        let items = ItemsOfTheText::of_the_text(text)?;
        let claim_lease = items.positive_milliseconds_of(CLAIM_LEASE_NAME)?;
        if u32::try_from(claim_lease.as_millis()).is_err() {
            let (_, line_number) = items.value_of(CLAIM_LEASE_NAME)?;
            return Err(format!(
                "配置第 {line_number} 行：{CLAIM_LEASE_NAME} 装不进 u32 毫秒（认领帧里租期是 u32）"
            ));
        }
        Ok(Self {
            oldest_pending_wait_limit: items.milliseconds_of(OLDEST_PENDING_WAIT_LIMIT_NAME)?,
            library_scan_interval: items.positive_milliseconds_of(LIBRARY_SCAN_INTERVAL_NAME)?,
            claim_lease,
        })
    }

    /// 认领帧里的租期（毫秒，[`Self::from_the_text`] 核过装得进 u32）。
    ///
    /// # Panics
    /// 不经 [`Self::from_the_text`] 造出的配置租期装不进 u32。
    #[must_use]
    pub fn claim_lease_milliseconds(&self) -> u32 {
        u32::try_from(self.claim_lease.as_millis()).expect("解配置时核过租期装得进 u32 毫秒")
    }

    /// 编进服务的那一份。
    ///
    /// # Panics
    /// 那份原文解不开（写坏了的配置不许悄悄退回缺省值）。
    #[must_use]
    pub fn of_the_repository() -> Self {
        Self::from_the_text(SERVICE_CONFIGURATION_TEXT).unwrap_or_else(|refusal| {
            panic!("gpu-verification-server/service-configuration.txt：{refusal}")
        })
    }
}
