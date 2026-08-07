#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppRoute {
    Tournament,
    Developer,
}

impl AppRoute {
    pub const fn development_tools_enabled(self) -> bool {
        matches!(self, Self::Developer)
    }

    fn from_path(path: &str) -> Self {
        match path.trim_end_matches('/') {
            "/dev" => Self::Developer,
            _ => Self::Tournament,
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn current_route() -> AppRoute {
    let path = web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .unwrap_or_default();
    AppRoute::from_path(&path)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn current_route() -> AppRoute {
    AppRoute::from_path("/")
}

#[cfg(test)]
mod tests {
    use super::AppRoute;

    #[test]
    fn developer_tools_are_only_enabled_on_the_dev_path() {
        assert_eq!(AppRoute::from_path("/dev"), AppRoute::Developer);
        assert_eq!(AppRoute::from_path("/dev/"), AppRoute::Developer);
        assert_eq!(AppRoute::from_path("/"), AppRoute::Tournament);
        assert_eq!(AppRoute::from_path("/development"), AppRoute::Tournament);
        assert_eq!(AppRoute::from_path("/tournament/dev"), AppRoute::Tournament);
    }
}
