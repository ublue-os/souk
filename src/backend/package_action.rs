#[derive(Debug, Clone, PartialEq, Hash)]
pub enum PackageAction {
    Install,
    Uninstall,
    Update,
}
