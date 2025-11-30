use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedName(pub usize);

#[derive(Debug, Default)]
pub struct Interner(
    HashMap<Box<[u8]>, InternedName>,
    HashMap<InternedName, String>,
);

impl Interner {
    pub fn intern(&mut self, name: &[u8]) -> InternedName {
        if let Some(existing) = self.0.get(name) {
            return *existing;
        }

        let next = InternedName(self.0.len());
        self.0.insert(name.into(), next);
        self.1
            .insert(next, std::str::from_utf8(name).unwrap().to_string());
        next
    }
}

impl IntoIterator for Interner {
    type Item = (InternedName, String);
    type IntoIter = std::collections::hash_map::IntoIter<InternedName, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.1.into_iter()
    }
}
