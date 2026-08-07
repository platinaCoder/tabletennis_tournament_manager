use std::time::{Duration, SystemTime};

#[cfg(any(test, target_arch = "wasm32"))]
use std::time::UNIX_EPOCH;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug)]
pub(crate) struct DiagnosticInstant(std::time::Instant);

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug)]
pub(crate) struct DiagnosticInstant(f64);

impl DiagnosticInstant {
    pub(crate) fn now() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(std::time::Instant::now())
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self(monotonic_milliseconds())
        }
    }

    pub(crate) fn elapsed(self) -> Duration {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0.elapsed()
        }
        #[cfg(target_arch = "wasm32")]
        {
            let elapsed_milliseconds = (monotonic_milliseconds() - self.0).max(0.0);
            Duration::from_secs_f64(elapsed_milliseconds / 1_000.0)
        }
    }
}

pub(crate) fn system_time_now() -> SystemTime {
    #[cfg(not(target_arch = "wasm32"))]
    {
        SystemTime::now()
    }
    #[cfg(target_arch = "wasm32")]
    {
        let elapsed = Duration::from_secs_f64(js_sys::Date::now().max(0.0) / 1_000.0);
        UNIX_EPOCH.checked_add(elapsed).unwrap_or(UNIX_EPOCH)
    }
}

#[cfg(target_arch = "wasm32")]
fn monotonic_milliseconds() -> f64 {
    web_sys::window()
        .and_then(|window| window.performance())
        .map_or_else(js_sys::Date::now, |performance| performance.now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_diagnostic_and_wall_clocks_are_available() {
        assert!(DiagnosticInstant::now().elapsed() >= Duration::ZERO);
        assert!(system_time_now().duration_since(UNIX_EPOCH).is_ok());
    }
}
