//! 显卡配置怎么读：`GPU_CARDS_BY_PRIORITY` 点名的卡（在哪台机器、`nvidia-smi` 序号、显存额度）、这个进程在哪台机器、`键=值` 配置文件里一个键的值、
//! 点名的卡里哪几张用不上。崩溃放量取卡（`crate::crash_amplification`，随 `verdict-store` 编）与 GPU 用例共用的卡池
//! （`crate::gpu_unit_checks::gpu_test_start`，随 `gpu` 编）都照它挑卡，所以单放一份、只开其中一个特性也编得进；
//! `crate::crash_amplification` 再 `pub use` 回去，它的调用方与单测照旧经它引。

use std::path::Path;

/// 配置里的一张卡在哪台机器上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardHost {
    Local,
    Peer,
}

impl CardHost {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Peer => "peer",
        }
    }

    /// 这个进程在哪台机器上：环境变量 `SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`，没设是本机。
    ///
    /// # Errors
    /// 写的不是 `local` 也不是 `peer`。
    pub fn of_this_process() -> Result<Self, String> {
        match std::env::var("SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE") {
            Err(_) => Ok(Self::Local),
            Ok(text) => Self::parse(&text),
        }
    }

    fn parse(text: &str) -> Result<Self, String> {
        match text.trim() {
            "local" => Ok(Self::Local),
            "peer" => Ok(Self::Peer),
            other => Err(format!("机器要写 local 或 peer，实际「{other}」")),
        }
    }
}

/// 配置点名的一张卡（`GPU_CARDS_BY_PRIORITY` 里的一项）：在哪台机器、`nvidia-smi` 的序号、给崩溃放量的显存额度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardByPriority {
    pub host: CardHost,
    pub index: u32,
    pub memory_quota_mebibytes: u64,
}

/// 读 `GPU_CARDS_BY_PRIORITY` 的值：空白隔开的若干项，每项 `<local|peer>:<nvidia-smi 序号>:<显存额度>`，
/// 额度是正整数加 `G` 或 `M`（1024 进制）；写在前面的优先级高，任务先装它、装满了才给后面的。
///
/// # Errors
/// 一项不是三段、机器或序号或额度写法不对、同一张卡点名两次、一项都没有。
pub fn parse_cards_by_priority(text: &str) -> Result<Vec<CardByPriority>, String> {
    let mut cards: Vec<CardByPriority> = Vec::new();
    for entry in text.split_whitespace() {
        let fields: Vec<&str> = entry.split(':').collect();
        let [host, index, quota] = fields.as_slice() else {
            return Err(format!(
                "显卡优先级的一项要写 <local|peer>:<序号>:<显存额度>，实际「{entry}」"
            ));
        };
        let host = CardHost::parse(host)?;
        let index: u32 = index
            .parse()
            .map_err(|_| format!("显卡序号不是数：「{entry}」"))?;
        let (number, unit_mebibytes) = match quota.as_bytes().last() {
            Some(b'G') => (&quota[..quota.len() - 1], 1024u64),
            Some(b'M') => (&quota[..quota.len() - 1], 1u64),
            _ => {
                return Err(format!(
                    "显存额度要带单位 G 或 M（1024 进制），实际「{entry}」"
                ))
            }
        };
        let number: u64 = number
            .parse()
            .ok()
            .filter(|number| *number > 0)
            .ok_or_else(|| format!("显存额度不是正整数：「{entry}」"))?;
        if cards
            .iter()
            .any(|card| card.host == host && card.index == index)
        {
            return Err(format!("同一张卡点名了两次：「{entry}」"));
        }
        cards.push(CardByPriority {
            host,
            index,
            memory_quota_mebibytes: number * unit_mebibytes,
        });
    }
    if cards.is_empty() {
        return Err("显卡优先级一项都没写".to_string());
    }
    Ok(cards)
}

/// `键=值` 配置文件里一个键的值（去掉两头的空白与一层引号）；文件不在、键没写是 `None`。
#[must_use]
pub fn configuration_value(path: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    text.lines().find_map(|line| {
        let value = line.trim().strip_prefix(key)?.strip_prefix('=')?.trim();
        let value = value
            .strip_prefix('\'')
            .and_then(|rest| rest.strip_suffix('\''))
            .or_else(|| {
                value
                    .strip_prefix('"')
                    .and_then(|rest| rest.strip_suffix('"'))
            })
            .unwrap_or(value);
        Some(value.to_string())
    })
}

/// 配置的显卡优先级：环境变量 `GPU_CARDS_BY_PRIORITY` 设了就照它（驱动脚本按清没清场从显卡配置里挑一份送过来，第二台的树副本也靠它），
/// 没设读显卡配置文件（`SINGLEFS_GPU_CARDS_CONFIG` 指的那一份，模板是仓根的 `gpu-cards.env.example`）里的同名键；
/// 都没有是 `None`（本机能用的卡按空闲显存从大到小用，不设额度）。
///
/// # Errors
/// 写了而写法不对（[`parse_cards_by_priority`]）。
pub fn cards_by_priority_from_configuration_or_environment(
    configuration_file: Option<&Path>,
) -> Result<Option<Vec<CardByPriority>>, String> {
    let text = match std::env::var("GPU_CARDS_BY_PRIORITY") {
        Ok(text) => Some(text),
        Err(_) => {
            configuration_file.and_then(|path| configuration_value(path, "GPU_CARDS_BY_PRIORITY"))
        }
    };
    match text {
        Some(text) if !text.trim().is_empty() => parse_cards_by_priority(&text).map(Some),
        _ => Ok(None),
    }
}

/// 配置点名的卡里哪几张（序号）不在用得上的卡里，按配置的次序。
#[must_use]
pub fn configured_cards_not_usable(configured: &[(u32, u64)], usable: &[u32]) -> Vec<u32> {
    configured
        .iter()
        .map(|(index, _)| *index)
        .filter(|index| !usable.contains(index))
        .collect()
}
