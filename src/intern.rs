use std::borrow::Cow;

use indexmap::IndexSet;

pub type StringId = usize;
#[derive(Default, Clone, Debug)]
pub struct StringIntern {
    map: IndexSet<String>,
}
impl StringIntern {
    pub fn get_id(&mut self, s: &str) -> StringId {
        match self.map.get_index_of(s) {
            Some(i) => i,
            None => {
                let place = self.map.len();
                self.map.insert(String::from(s));
                place
            }
        }
    }
    pub fn get_str(&self, id: StringId) -> Option<Cow<String>> {
        Some(Cow::Borrowed(self.map.get_index(id)?))
    }
}
