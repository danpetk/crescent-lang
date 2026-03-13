use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternID(usize);

#[derive(Default)]
pub struct Interner {
    map: HashMap<String, InternID>,
    strings: Vec<String>,
}

// NOTE: There might be a way to do this with one string allocation instead of two
// but its not trivial
impl Interner {
    pub fn intern(&mut self, string: &str) -> InternID {
        if let Some(&id) = self.map.get(string) {
            return id;
        }

        let id = InternID(self.strings.len());
        self.strings.push(string.to_string());
        self.map.insert(string.to_string(), id);
        id
    }

    pub fn resolve(&self, id: InternID) -> &str {
        &self.strings[id.0]
    }
}
