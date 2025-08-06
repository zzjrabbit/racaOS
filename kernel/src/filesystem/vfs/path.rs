use core::{
    fmt::Display,
    ops::{Add, AddAssign, Deref, DerefMut},
};

use alloc::{format, string::String, vec::Vec};

#[derive(Clone, Debug)]
pub struct Path {
    inner: String,
}

impl Path {
    pub fn new<S>(path: S) -> Path
    where
        String: From<S>,
    {
        Path {
            inner: String::from(path),
        }
    }

    pub fn parent(&self) -> Option<Path> {
        let mut path = self.clone();

        path.delete_end_spliters();

        if path.is_empty() {
            return None;
        }

        while !path.ends_with("/") && !path.is_empty() {
            path.pop();
        }
        Some(path)
    }

    pub fn name(&self) -> String {
        let mut path = self.clone();
        path.delete_end_spliters();

        let mut name = String::new();

        while !path.ends_with("/") && !path.is_empty() {
            name.insert(0, path.pop().unwrap());
        }

        name
    }

    /// Gets the ancestors of the given path
    pub fn ancestors(&self) -> Vec<Path> {
        let mut ancestors = Vec::new();
        let mut current_path = self.clone();

        while let Some(parent) = current_path.parent() {
            ancestors.push(parent.clone());
            current_path = parent.clone();
        }

        ancestors.reverse();
        ancestors
    }

    /// Gets the parts of the given path
    pub fn parts(&self) -> Vec<Path> {
        self.inner
            .split("/")
            .filter(|s| !s.is_empty())
            .map(|s| Path::new(s))
            .collect::<Vec<_>>()
    }

    /// Deltes all the "/" from the end of the path.
    pub fn delete_end_spliters(&mut self) {
        while self.inner.ends_with("/") {
            self.inner.pop();
        }
    }

    pub fn join(&self, mut second: Path) -> Path {
        let mut first = self.clone();

        first.delete_end_spliters();

        while second.starts_with("/") {
            second.remove(0);
        }

        Path::new(format!("{}/{}", first, second))
    }

    pub fn dir_format(&self) -> Path {
        let mut path = self.clone();
        path.delete_end_spliters();
        path.push('/');
        path
    }
}

impl From<Path> for String {
    fn from(value: Path) -> Self {
        value.inner
    }
}

impl<A> Add<A> for Path
where
    String: From<A>,
{
    type Output = Path;
    fn add(self, rhs: A) -> Self::Output {
        Path::new::<String>(format!("{}{}", self.inner, String::from(rhs)))
    }
}

impl<A> AddAssign<A> for Path
where
    String: From<A>,
{
    fn add_assign(&mut self, rhs: A) {
        self.inner = format!("{}{}", self.inner, String::from(rhs));
    }
}

impl Deref for Path {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
