use std::path::PathBuf;

pub struct DevArgs {
    pub host: String,
    pub dir: Option<PathBuf>,
    pub push: bool,
}
