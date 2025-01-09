use crate::{Error, Result};
use std::path::{Path, PathBuf};

/// A module resolver that handles both relative and absolute module specifiers.
#[derive(Debug, Clone)]
pub struct ModuleResolver {
    /// Base paths to search for modules
    base_paths: Vec<PathBuf>,
}

impl ModuleResolver {
    /// Create a new module resolver
    pub fn new() -> Self {
        Self {
            base_paths: Vec::new(),
        }
    }

    /// Add a base path for module resolution
    pub fn add_base_path<P: AsRef<Path>>(&mut self, path: P) {
        self.base_paths.push(path.as_ref().to_path_buf());
    }

    /// Resolve a module specifier to a canonical path
    pub fn resolve(&self, specifier: &str, parent_specifier: Option<&str>) -> Result<String> {
        // Normalize path separators
        let specifier = specifier.replace('\\', "/");

        // If it's a relative path and we have a parent module
        if specifier.starts_with("./") || specifier.starts_with("../") {
            if let Some(parent) = parent_specifier {
                let parent_path = Path::new(&parent.replace('\\', "/"));
                if let Some(parent_dir) = parent_path.parent() {
                    let resolved = parent_dir.join(&specifier);
                    match resolved.canonicalize() {
                        Ok(canonical) => return Ok(canonical.to_string_lossy()
                            .replace('\\', "/")
                            .into_owned()),
                        Err(e) => return Err(Error::PathResolution {
                            path: specifier,
                            reason: e.to_string(),
                        }),
                    }
                }
            }
            return Err(Error::ModuleNotFound {
                name: specifier.to_string(),
                available_modules: Vec::new(),
                reason: Some("No parent module for relative import".to_string()),
            });
        }

        // Try each base path
        let mut tried_paths = Vec::new();
        for base in &self.base_paths {
            let path = base.join(&specifier);
            match path.canonicalize() {
                Ok(canonical) => return Ok(canonical.to_string_lossy()
                    .replace('\\', "/")
                    .into_owned()),
                Err(_) => tried_paths.push(path),
            }
        }

        // If no base paths worked, return error with attempted paths
        if !self.base_paths.is_empty() {
            return Err(Error::ModuleNotFound {
                name: specifier.to_string(),
                available_modules: Vec::new(),
                reason: Some(format!("Tried paths: {:?}", tried_paths)),
            });
        }

        // If it's not a path, treat it as a module name
        Ok(specifier.to_string())
    }

    /// Check if a module specifier is relative
    pub fn is_relative(specifier: &str) -> bool {
        specifier.starts_with("./") || specifier.starts_with("../")
    }

    /// Get the parent directory of a module specifier
    pub fn get_parent_dir(specifier: &str) -> Option<String> {
        let path = Path::new(specifier);
        path.parent().map(|p| p.to_string_lossy().into_owned())
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn setup_test_dir() -> (tempfile::TempDir, ModuleResolver) {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut resolver = ModuleResolver::new();
        resolver.add_base_path(&temp_dir);
        
        // Create test files
        fs::create_dir_all(temp_dir.path().join("utils")).unwrap();
        fs::write(
            temp_dir.path().join("math.js"),
            "export const add = (a, b) => a + b;",
        ).unwrap();
        fs::write(
            temp_dir.path().join("utils/helpers.js"),
            "export const format = (num) => `Result: ${num}`;",
        ).unwrap();
        
        (temp_dir, resolver)
    }

    #[test]
    fn test_relative_paths() {
        let mut resolver = ModuleResolver::new();
        resolver.add_base_path("/modules");

        assert!(ModuleResolver::is_relative("./foo"));
        assert!(ModuleResolver::is_relative("../foo"));
        assert!(!ModuleResolver::is_relative("foo"));
    }

    #[test]
    fn test_absolute_imports() {
        let (_temp_dir, resolver) = setup_test_dir();
        
        // Test absolute module name
        assert!(resolver.resolve("math", None).is_ok());
        
        // Test non-existent module
        assert!(resolver.resolve("nonexistent", None).is_ok());
    }

    #[test]
    fn test_relative_imports() {
        let (temp_dir, resolver) = setup_test_dir();
        let parent = temp_dir.path().join("math.js").to_string_lossy().into_owned();
        
        // Test relative import from parent
        let result = resolver.resolve("./utils/helpers", Some(&parent));
        assert!(result.is_ok());
        
        // Test parent directory traversal
        fs::create_dir_all(temp_dir.path().join("deep/nested")).unwrap();
        fs::write(
            temp_dir.path().join("deep/nested/child.js"),
            "export const child = () => {};",
        ).unwrap();
        
        let deep_parent = temp_dir.path().join("deep/nested/child.js").to_string_lossy().into_owned();
        let result = resolver.resolve("../../math", Some(&deep_parent));
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_base_paths() {
        let temp_dir1 = tempfile::tempdir().unwrap();
        let temp_dir2 = tempfile::tempdir().unwrap();
        let mut resolver = ModuleResolver::new();
        
        resolver.add_base_path(&temp_dir1);
        resolver.add_base_path(&temp_dir2);
        
        // Create test files in different base paths
        fs::write(
            temp_dir1.path().join("module1.js"),
            "export const one = 1;",
        ).unwrap();
        fs::write(
            temp_dir2.path().join("module2.js"),
            "export const two = 2;",
        ).unwrap();
        
        // Both modules should be resolvable
        assert!(resolver.resolve("module1", None).is_ok());
        assert!(resolver.resolve("module2", None).is_ok());
    }

    #[test]
    fn test_path_canonicalization() {
        let (_temp_dir, resolver) = setup_test_dir();
        
        // Test that different path formats resolve to the same canonical path
        let path1 = resolver.resolve("./utils/../math", None);
        let path2 = resolver.resolve("math", None);
        
        assert!(path1.is_ok() && path2.is_ok());
        assert_eq!(path1.unwrap(), path2.unwrap());
    }
}
