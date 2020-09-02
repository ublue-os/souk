#[derive(Debug, Clone, PartialEq)]
pub enum PackageAction{
    Install,
    Uninstall,
    Update,
    Remove,
}
