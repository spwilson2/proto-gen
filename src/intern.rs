use std::borrow::Cow;

use indexmap::IndexSet;

pub type StringId = usize;
#[derive(Default, Debug)]
pub struct StringIntern {
    map: IndexSet<String>,
}
impl StringIntern {
    pub fn get_id(&mut self, s: &String) -> StringId {
        match self.map.get_index_of(s) {
            Some(i) => i,
            None => {
                let place = self.map.len();
                self.map.insert(s.clone());
                place
            }
        }
    }
    pub fn get_str(&mut self, id: StringId) -> Option<Cow<String>> {
        Some(Cow::Borrowed(self.map.get_index(id)?))
    }
}
