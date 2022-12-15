#[derive(PartialEq, Clone)]
pub struct SemanticVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

#[derive(PartialEq)]
pub enum Compatibility {
    /// Different Major
    Breaking, // 2.1.0 != 1.2.0

    /// Same Major,Minor,Patch
    Exact, // 1.0.1 == 1.0.1
    /// Same Major+Minor, diffent patch
    Partial, // 1.1.0 ~= 1.1.4

    /// Same Major, different minor or patch
    Compatible, // 1.1.3 compat 1.2.0
}

impl SemanticVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
    pub fn from_string(data: String) -> SemanticVersion {
        let parts: Vec<&str> = data.split(".").collect();
        let major = parts[0]
            .parse::<u16>()
            .expect(format!("Need unsigned integer as major version: {}", data).as_str());
        let minor = parts[1]
            .parse::<u16>()
            .expect(format!("Need unsigned integer as minor version: {}", data).as_str());
        let patch = parts[2]
            .parse::<u16>()
            .expect(format!("Need unsigned integer as patch version: {}", data).as_str());
        return SemanticVersion::new(major, minor, patch);
    }

    pub fn match_to(&self, other: &SemanticVersion) -> Compatibility {
        if self.major == other.major {
            if self.minor == other.minor {
                if self.patch == other.patch {
                    return Compatibility::Exact;
                }
                return Compatibility::Partial;
            }
            return Compatibility::Compatible;
        }
        return Compatibility::Breaking;
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}.{}.{}", self.major, self.minor, self.patch).as_str())
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.major > other.major {
            return Some(std::cmp::Ordering::Greater);
        } else if self.major < other.major {
            return Some(std::cmp::Ordering::Less);
        } else {
            // Same Major version
            if self.minor > other.minor {
                return Some(std::cmp::Ordering::Greater);
            } else if self.minor < other.minor {
                return Some(std::cmp::Ordering::Less);
            } else {
                // same Major & same Minor
                if self.patch > other.patch {
                    return Some(std::cmp::Ordering::Greater);
                } else if self.patch < other.patch {
                    return Some(std::cmp::Ordering::Less);
                } else {
                    return Some(std::cmp::Ordering::Equal);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Compatibility, SemanticVersion};

    #[test]
    pub fn same_version_equals() {
        let v1 = SemanticVersion::new(1, 0, 4);
        let v2 = SemanticVersion::new(1, 0, 4);

        assert!(v1.match_to(&v2) == Compatibility::Exact)
    }

    #[test]
    pub fn partial_compat() {
        let v1 = SemanticVersion::new(1, 0, 4);
        let v2 = SemanticVersion::new(1, 0, 0);

        assert!(v1.match_to(&v2) == Compatibility::Partial)
    }

    #[test]
    pub fn breaking_change() {
        let v1 = SemanticVersion::new(2, 0, 4);
        let v2 = SemanticVersion::new(1, 0, 0);

        assert!(v1.match_to(&v2) == Compatibility::Breaking)
    }

    #[test]
    pub fn minor_compat() {
        let v1 = SemanticVersion::new(2, 2, 4);
        let v2 = SemanticVersion::new(2, 1, 0);

        assert!(v1.match_to(&v2) == Compatibility::Compatible)
    }

    #[test]
    pub fn same_version_are_eq() {
        let v1 = SemanticVersion::new(2, 2, 4);
        let v2 = SemanticVersion::new(2, 2, 4);
        assert!(v1 == v2);
    }

    #[test]
    pub fn larger_version_is_ge() {
        let v1 = SemanticVersion::new(2, 4, 4);
        let v2 = SemanticVersion::new(2, 2, 4);
        assert!(v1 > v2);
    }

    #[test]
    pub fn smaller_version_is_le() {
        let v1 = SemanticVersion::new(1, 4, 4);
        let v2 = SemanticVersion::new(2, 2, 4);
        assert!(v1 < v2);
    }
}
