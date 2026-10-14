#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTab {
    DecodeBuild,
    Sign,
    Merge,
    Install,
}
impl MainTab {
    pub const ALL: [Self; 4] = [Self::DecodeBuild, Self::Sign, Self::Merge, Self::Install];
    pub fn title(self) -> &'static str {
        match self {
            Self::DecodeBuild => "Decode + Build",
            Self::Sign => "Sign",
            Self::Merge => "Merge",
            Self::Install => "Install",
        }
    }
}
