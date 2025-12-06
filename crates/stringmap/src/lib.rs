#![no_std]

use core::fmt::{Debug, DebugMap};

use alloc::{collections::btree_map::BTreeMap, format, string::String};

extern crate alloc;

pub struct StringMap<T> {
    value: Option<T>,
    next: BTreeMap<char, Self>,
}

impl<T> StringMap<T> {
    pub const fn new() -> Self {
        Self {
            value: None,
            next: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, name: &str, value: T) {
        let mut current = self;
        for ch in name.chars() {
            let entry = current.next.entry(ch);
            let next = entry.or_insert(Self::new());
            current = next;
        }

        current.value = Some(value);
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        let mut current = self;
        for ch in name.chars() {
            if let Some(node) = current.next.get(&ch) {
                current = node;
            } else {
                return None;
            }
        }
        current.value.as_ref()
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut T> {
        let mut current = self;
        for ch in name.chars() {
            if let Some(node) = current.next.get_mut(&ch) {
                current = node;
            } else {
                return None;
            }
        }
        current.value.as_mut()
    }

    pub fn for_each(&self, mut f: impl FnMut(String, &T)) {
        recur(self, String::new(), &mut f);
    }
}

fn recur<T>(node: &StringMap<T>, current: String, f: &mut impl FnMut(String, &T)) {
    for (ch, child) in &node.next {
        recur(child, format!("{}{}", current, ch), f);
    }

    if let Some(value) = &node.value {
        f(current, value);
    }
}

impl<T: Debug> Debug for StringMap<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut map = f.debug_map();
        let current_name = String::new();

        debug_map(&mut map, self, current_name);

        map.finish()
    }
}

fn debug_map<T: Debug>(map: &mut DebugMap, node: &StringMap<T>, current: String) {
    for (ch, child) in &node.next {
        debug_map(map, child, format!("{}{}", current, ch));
    }

    if let Some(value) = &node.value {
        map.entry(&current, value);
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;

    #[test]
    fn test_get() {
        let mut map = StringMap::new();
        map.insert("hello", "world");
        map.insert("hella", "worla");
        map.insert("<zodiac::mem::vm_space::VmSpace>::new_user", "good");
        map.insert("<zodiac::mem::vm_space::Cursor>::map_iomem", "test");
        assert_eq!(
            map.get("<zodiac::mem::vm_space::VmSpace>::new_user"),
            Some(&"good")
        );
    }
}
