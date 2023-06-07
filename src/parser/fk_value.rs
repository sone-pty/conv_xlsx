use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::cell::RefCell;

type FKMap<'a> = HashMap<&'a str, HashMap<&'a str, &'a str>>;
// <col, (fk_pattern, vals)>
pub type RawValData<'a> = (usize, (&'a str, Vec<&'a str>));

pub struct FKValue<'a> {
    rawdata: HashMap<usize, ColRawData<'a>>, // <col, data>
    fk_map: RefCell<FKMap<'a>>
}

struct ColRawData<'a> {
    fk_pattern: &'a str,
    vals: Vec<&'a str>
}

impl<'a> FKValue<'a> {
    pub fn new(vals: Vec<RawValData<'a>>) -> Self {
        let mut rawdata: HashMap<usize, ColRawData<'a>> = HashMap::default();
        let fk_map: RefCell<FKMap<'a>> = RefCell::from(HashMap::default());

        for v in vals {
            match rawdata.entry(v.0) {
                Entry::Occupied(mut e) => {
                    for vv in v.1.1 {
                        e.get_mut().vals.push(vv);
                    }
                }
                Entry::Vacant(e) => {
                    e.insert(ColRawData { fk_pattern: v.1.0, vals: Vec::default() });
                }
            }
        }

        Self { rawdata, fk_map }
    }

    pub fn parse(&self) {
        for (_, v) in self.rawdata.iter() {
            for vv in v.vals.iter() {
                self.parse_internal(*vv, v.fk_pattern);
            }
        }
    }

    //----------------------------private-------------------------------
    fn parse_internal(&self, val: &'a str, pattern: &'a str) {
        let mut fks = self.fk_map.borrow_mut();

        
    }
}