use alloc::string::{String, ToString};
use serde::{Deserialize, Serialize};

/// Light Player path - paths from project root
///
/// Supports both absolute (starting with `/`) and relative paths.
/// Paths are automatically normalized on construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LpPath(String);

impl LpPath {
    /// Create a new LpPath, normalizing the path
    pub fn new(path: String) -> Self {
        Self(normalize(&path))
    }

    /// Get the path as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if the path is absolute (starts with `/`)
    pub fn is_absolute(&self) -> bool {
        self.0.starts_with('/')
    }

    /// Check if the path is relative (!starts with `/`)
    pub fn is_relative(&self) -> bool {
        !self.0.starts_with('/')
    }

    /// Get the parent directory path
    ///
    /// Returns `None` if the path is root (`/`) or empty, or if there's no parent component.
    pub fn parent(&self) -> Option<LpPath> {
        if self.0 == "/" || self.0.is_empty() {
            return None;
        }

        // Find the last `/`
        if let Some(last_slash) = self.0.rfind('/') {
            if last_slash == 0 {
                // Only root `/` remains
                Some(LpPath("/".to_string()))
            } else {
                // Return path up to (but not including) last `/`
                Some(LpPath(self.0[..last_slash].to_string()))
            }
        } else {
            // No `/` found, check if there's a parent component
            // For relative paths like "src/test", parent is "src"
            // For relative paths like "test", there's no parent
            if self.0.contains('/') {
                // This shouldn't happen after normalization, but handle it
                if let Some(last_slash) = self.0.rfind('/') {
                    if last_slash == 0 {
                        Some(LpPath("/".to_string()))
                    } else {
                        Some(LpPath(self.0[..last_slash].to_string()))
                    }
                } else {
                    None
                }
            } else {
                // Single component relative path, no parent
                None
            }
        }
    }

    /// Get the last component (file name)
    ///
    /// Returns `None` if the path is root (`/`) or empty.
    pub fn file_name(&self) -> Option<&str> {
        if self.0 == "/" || self.0.is_empty() {
            return None;
        }

        // Find the last `/`
        if let Some(last_slash) = self.0.rfind('/') {
            if last_slash == self.0.len() - 1 {
                // Trailing `/`, this shouldn't happen after normalization, but handle it
                // Find the component before the trailing slash
                let trimmed = &self.0[..last_slash];
                if trimmed.is_empty() || trimmed == "/" {
                    None
                } else if let Some(prev_slash) = trimmed.rfind('/') {
                    Some(&trimmed[prev_slash + 1..])
                } else {
                    Some(trimmed)
                }
            } else {
                Some(&self.0[last_slash + 1..])
            }
        } else {
            // No `/` found, entire path is the file name
            Some(&self.0)
        }
    }

    /// Get the file stem (file name without extension)
    ///
    /// Returns `None` if there is no file name or no extension.
    pub fn file_stem(&self) -> Option<&str> {
        self.file_name().and_then(|name| {
            if let Some(dot_pos) = name.rfind('.') {
                if dot_pos > 0 {
                    Some(&name[..dot_pos])
                } else {
                    // File name starts with `.`, no stem
                    None
                }
            } else {
                // No extension, entire name is the stem
                Some(name)
            }
        })
    }

    /// Get the file extension (without leading dot)
    ///
    /// Returns `None` if there is no file name or no extension.
    pub fn extension(&self) -> Option<&str> {
        self.file_name().and_then(|name| {
            if let Some(dot_pos) = name.rfind('.') {
                if dot_pos < name.len() - 1 {
                    Some(&name[dot_pos + 1..])
                } else {
                    // Trailing `.`, no extension
                    None
                }
            } else {
                None
            }
        })
    }
}

impl From<String> for LpPath {
    fn from(s: String) -> Self {
        Self(normalize(&s))
    }
}

impl From<&str> for LpPath {
    fn from(s: &str) -> Self {
        Self(normalize(s))
    }
}

/// Normalize a path string
///
/// Normalization rules:
/// - Trim whitespace
/// - Remove leading `./` or `.` (if present)
/// - For absolute paths: ensure leading `/`
/// - For relative paths: keep as-is (no leading `/`)
/// - Collapse multiple consecutive slashes (`//` → `/`)
/// - Remove trailing `/` unless it's the root path (`/`)
/// - Handle empty paths: `""` → `"/"` (absolute root)
fn normalize(path: &str) -> String {
    let mut normalized = path.trim();

    // Remove leading "./" or "."
    if normalized.starts_with("./") {
        normalized = &normalized[2..];
    } else if normalized == "." {
        normalized = "";
    }

    // Handle empty path
    let normalized = if normalized.is_empty() {
        "/".to_string()
    } else {
        normalized.to_string()
    };

    // Collapse multiple slashes
    let normalized = normalized.replace("//", "/");

    // Remove trailing "/" unless it's the root
    if normalized.len() > 1 && normalized.ends_with('/') {
        normalized[..normalized.len() - 1].to_string()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lp_path_creation() {
        let path = LpPath::new("/src/test.texture".to_string());
        assert_eq!(path.as_str(), "/src/test.texture");
    }

    #[test]
    fn test_lp_path_from_string() {
        let path = LpPath::from("/src/test.shader".to_string());
        assert_eq!(path.as_str(), "/src/test.shader");
    }

    #[test]
    fn test_lp_path_from_str() {
        let path = LpPath::from("/src/test.output");
        assert_eq!(path.as_str(), "/src/test.output");
    }

    #[test]
    fn test_normalization_absolute() {
        assert_eq!(LpPath::from("//src//test//").as_str(), "/src/test");
        assert_eq!(LpPath::from("/src/test/").as_str(), "/src/test");
        assert_eq!(LpPath::from("  /src/test  ").as_str(), "/src/test");
    }

    #[test]
    fn test_normalization_relative() {
        assert_eq!(LpPath::from("src/test").as_str(), "src/test");
        assert_eq!(LpPath::from("./src/test").as_str(), "src/test");
        assert_eq!(LpPath::from(".").as_str(), "/");
        assert_eq!(LpPath::from("").as_str(), "/");
    }

    #[test]
    fn test_is_absolute() {
        assert!(LpPath::from("/src/test").is_absolute());
        assert!(!LpPath::from("src/test").is_absolute());
        assert!(LpPath::from("/").is_absolute());
    }

    #[test]
    fn test_is_relative() {
        assert!(!LpPath::from("/src/test").is_relative());
        assert!(LpPath::from("src/test").is_relative());
        assert!(!LpPath::from("/").is_relative());
    }

    #[test]
    fn test_parent() {
        assert_eq!(
            LpPath::from("/src/test").parent(),
            Some(LpPath::from("/src"))
        );
        assert_eq!(LpPath::from("/src").parent(), Some(LpPath::from("/")));
        assert_eq!(LpPath::from("/").parent(), None);
        assert_eq!(LpPath::from("src/test").parent(), Some(LpPath::from("src")));
        assert_eq!(LpPath::from("test").parent(), None);
    }

    #[test]
    fn test_file_name() {
        assert_eq!(LpPath::from("/src/test.txt").file_name(), Some("test.txt"));
        assert_eq!(LpPath::from("/src/test").file_name(), Some("test"));
        assert_eq!(LpPath::from("/").file_name(), None);
        assert_eq!(LpPath::from("test.txt").file_name(), Some("test.txt"));
    }

    #[test]
    fn test_file_stem() {
        assert_eq!(LpPath::from("/src/test.txt").file_stem(), Some("test"));
        assert_eq!(
            LpPath::from("/src/test.tar.gz").file_stem(),
            Some("test.tar")
        );
        assert_eq!(LpPath::from("/src/test").file_stem(), Some("test"));
        assert_eq!(LpPath::from("/src/.hidden").file_stem(), None);
        assert_eq!(LpPath::from("/").file_stem(), None);
    }

    #[test]
    fn test_extension() {
        assert_eq!(LpPath::from("/src/test.txt").extension(), Some("txt"));
        assert_eq!(LpPath::from("/src/test.tar.gz").extension(), Some("gz"));
        assert_eq!(LpPath::from("/src/test").extension(), None);
        assert_eq!(LpPath::from("/src/test.").extension(), None);
        assert_eq!(LpPath::from("/").extension(), None);
    }
}
