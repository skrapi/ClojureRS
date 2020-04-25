//! For very small Persistent maps;  this is a persistent map implemented upon
//! persistent lists, which already can share structure.  In this case, each
//! element of the list is a key-value pair (so this is very similar to implementing
//! a persistent associative structure in Elisp with alists)
//!
//!   (MapEntry :a 1)
//!       /           \
//!      /            \
//!(MapEntry :b 2)    (MapEntry :b 3)
//!   /               \
//! a                 b
//! -------------------
//! a => {:a 1 :b 2}
//! b => {:a 1 :b 3}

use crate::maps::MapEntry;
use crate::symbol::Symbol;
use crate::value::Value;

use std::collections::HashMap;
use std::convert::From;
use std::fmt;
use std::iter::FromIterator;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum PersistentListMap {
    Map(Rc<PersistentListMap>, MapEntry),
    Empty,
}

// Again, only using strange IBlah convention to reflect the Clojure base
// @TODO really though .. just rethink this
/// A PersistentListMap.

pub trait IPersistentListMap {
    fn get(&self, key: &Rc<Value>) -> Rc<Value>;
    fn assoc(&self, key: Rc<Value>, value: Rc<Value>) -> Self;
}

impl IPersistentListMap for PersistentListMap {
    // @TODO make fn of ILookup
    fn get(&self, key: &Rc<Value>) -> Rc<Value> {
        match self {
            PersistentListMap::Map(parent, entry) => {
                if entry.key == *key {
                    return Rc::clone(&entry.val);
                }
                parent.get(key)
            }
            PersistentListMap::Empty => Rc::new(Value::Nil),
        }
    }
    fn assoc(&self, key: Rc<Value>, val: Rc<Value>) -> PersistentListMap {
        PersistentListMap::Map(Rc::new(self.clone()), MapEntry { key, val })
    }
}

impl IPersistentListMap for Rc<PersistentListMap> {
    // @TODO make fn of ILookup
    fn get(&self, key: &Rc<Value>) -> Rc<Value> {
        match &**self {
            PersistentListMap::Map(parent, entry) => {
                if entry.key == *key {
                    return Rc::clone(&entry.val);
                }
                parent.get(key)
            }
            PersistentListMap::Empty => Rc::new(Value::Nil),
        }
    }
    fn assoc(&self, key: Rc<Value>, val: Rc<Value>) -> Rc<PersistentListMap> {
        Rc::new(PersistentListMap::Map(
            Rc::clone(self),
            MapEntry { key, val },
        ))
    }
}

// The purpose of these functions are no longer to implement conversion,
// but to give us a cleaner way to invoke it

pub trait ToPersistentListMap {
    fn into_list_map(self) -> PersistentListMap;
}

impl<T> ToPersistentListMap for T
where
    T: Into<PersistentListMap>,
{
    fn into_list_map(self) -> PersistentListMap {
        Into::<PersistentListMap>::into(self)
    }
}

impl From<Vec<MapEntry>> for PersistentListMap {
    fn from(item: Vec<MapEntry>) -> Self {
        item.into_iter().collect::<PersistentListMap>()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Iterating
//
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct PersistentListMapIter {
    node: Rc<PersistentListMap>,
    seen: HashMap<Rc<Value>, bool>,
}

pub trait ToPersistentListMapIter {
    fn iter(&self) -> PersistentListMapIter;
}

impl Iterator for PersistentListMapIter {
    type Item = MapEntry;
    fn next(&mut self) -> Option<Self::Item> {
        match &*(Rc::clone(&self.node)) {
            PersistentListMap::Map(parent, mapentry) => {
                self.node = Rc::clone(parent);
                if self.seen.contains_key(&mapentry.key) {
                    return self.next();
                }
                self.seen.insert(mapentry.key.clone(), true);
                Some(mapentry.clone())
            }
            PersistentListMap::Empty => None,
        }
    }
}

impl ToPersistentListMapIter for Rc<PersistentListMap> {
    fn iter(&self) -> PersistentListMapIter {
        PersistentListMapIter {
            node: Rc::clone(self),
            seen: HashMap::new(),
        }
    }
}

impl ToPersistentListMapIter for PersistentListMap {
    fn iter(&self) -> PersistentListMapIter {
        Rc::new(self.clone()).iter()
    }
}

impl FromIterator<MapEntry> for PersistentListMap {
    fn from_iter<I: IntoIterator<Item = MapEntry>>(iter: I) -> Self {
        let mut map_so_far = PersistentListMap::Empty;

        for i in iter {
            map_so_far = PersistentListMap::Map(Rc::new(map_so_far), i.clone());
        }
        map_so_far
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// End Iteration
////////////////////////////////////////////////////////////////////////////////////////////////////

impl fmt::Display for PersistentListMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut as_str = String::from("{");
        let mut first_loop = true;

        for mapentry in self.iter() {
            if !first_loop {
                as_str.push_str(", ");
            }
            first_loop = false;
            as_str.push_str(&format!(
                "{} {}",
                mapentry.key.to_string_explicit(),
                mapentry.val.to_string_explicit()
            ));
        }
        as_str.push_str("}");

        write!(f, "{}", as_str)
    }
}

#[cfg(test)]
mod tests {
    use crate::persistent_list_map::*;
    use crate::value::ToValue;
    #[test]
    fn test_persistent_list_map() {
        let empty = PersistentListMap::Empty;
        let map1 = vec![
            MapEntry {
                key: Symbol::intern("a").to_value(),
                val: 15_i32.to_rc_value(),
            },
            MapEntry {
                key: Symbol::intern("b").to_value(),
                val: "stuff".to_rc_value(),
            },
        ]
        .into_iter()
        .collect::<PersistentListMap>();
        println!("{}", map1);
        let map2 = map1.assoc(Symbol::intern("c").to_value(), 100_i32.to_rc_value());
        println!("{}", map1);
        println!("{}", map2);
        let map3 = map1.assoc(Symbol::intern("a").to_value(), 100_i32.to_rc_value());
        println!("{}", map1);
        println!("{}", map2);
        println!("{}", map3);
        let map4 = map2.assoc(Symbol::intern("a").to_value(), 100_i32.to_rc_value());
        println!("{}", map1);
        println!("{}", map2);
        println!("{}", map3);
        println!("{}", map4);
    }
}
