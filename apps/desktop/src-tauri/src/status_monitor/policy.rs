use tokio::time::Duration;

const FAILURE_LIMIT: u32 = 3;
const DEGRADED_POLL_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailureMode {
    Active,
    Degraded,
}

#[derive(Debug, Clone, Copy)]
pub struct FailureState {
    consecutive_failures: u32,
    mode: FailureMode,
}

impl Default for FailureState {
    fn default() -> Self {
        Self {
            consecutive_failures: 0,
            mode: FailureMode::Active,
        }
    }
}

impl FailureState {
    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.mode = FailureMode::Active;
    }

    pub fn current_interval(&self, normal: Duration) -> Duration {
        match self.mode {
            FailureMode::Active => normal,
            FailureMode::Degraded => DEGRADED_POLL_INTERVAL,
        }
    }
}

pub struct FailureAssessment {
    pub message: String,
    pub degraded: bool,
    pub unsupported: bool,
    pub stop_sampling: bool,
}

pub fn assess_failure(error: &str, state: &mut FailureState, backoff_enabled: bool) -> FailureAssessment {
    state.consecutive_failures += 1;
    let lowered = error.to_ascii_lowercase();
    let unsupported = [
        "unsupported collector",
        "unsupported os",
        "invalid input",
        "unknown command",
        "syntax error",
        "bad command or file name",
        "is not recognized as the name of a cmdlet",
        "not supported on this host",
    ]
    .iter()
    .any(|pattern| lowered.contains(pattern));

    if unsupported {
        return FailureAssessment {
            message: format!(
                "高级状态采集已暂停：当前主机可能为网络设备或不支持监控采样。原始错误：{}",
                sanitize_error_message(error)
            ),
            degraded: false,
            unsupported: true,
            stop_sampling: true,
        };
    }

    let degraded = backoff_enabled && state.consecutive_failures >= FAILURE_LIMIT;
    if degraded {
        state.mode = FailureMode::Degraded;
    }

    FailureAssessment {
        message: if degraded {
            format!(
                "状态采集连续失败 {} 次，已降级为慢轮询：{}",
                state.consecutive_failures,
                sanitize_error_message(error)
            )
        } else {
            sanitize_error_message(error)
        },
        degraded,
        unsupported: false,
        stop_sampling: false,
    }
}

fn sanitize_error_message(error: &str) -> String {
    error.split_whitespace().collect::<Vec<_>>().join(" ")
}
