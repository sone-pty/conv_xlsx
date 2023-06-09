use crate::defs::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Error, ErrorKind, Result},
    rc::Rc, path::Path,
};
use xlsx_read::{excel_file::ExcelFile, excel_table::ExcelTable};

use item_class::ItemClass;
mod item_class;

use base_class::BaseClass;
mod base_class;

use cell_value::CellValue;

use self::fk_value::{FKValue, RawValData};
mod cell_value;

//use fk_value::FKValue;
mod fk_value;

mod stack;

type LSMap = Rc<RefCell<HashMap<Rc<String>, usize>>>;

trait CodeGenerator {
    type Output;
    fn gen_code(&self, end: &'static str, tab_nums: i32) -> Self::Output;
}

pub enum KeyType {
    None,
    DefKey(Vec<(ItemStr, usize)>),
    OriginalTemplateId,
}

pub struct DefaultData(HashMap<Rc<String>, Box<CellValue>>);
impl Default for DefaultData {
    fn default() -> Self {
        DefaultData(HashMap::with_capacity(20))
    }
}

pub struct VarData(HashMap<Rc<String>, Vec<Box<CellValue>>>);
impl Default for VarData {
    fn default() -> Self {
        VarData(HashMap::with_capacity(20))
    }
}

pub struct Parser {
    item_class: ItemClass,
    base_class: BaseClass,
    defaults: Rc<RefCell<DefaultData>>,
    vals: Rc<RefCell<VarData>>,
    required_fields: Rc<RefCell<Vec<ItemStr>>>,
    key_type: Rc<RefCell<KeyType>>,
    skip_cols: Vec<usize>
}

impl CodeGenerator for Parser {
    type Output = String;

    fn gen_code(&self, end: &'static str, tab_nums: i32) -> Self::Output {
        let mut code = String::with_capacity(8192);

        // comment
        code.push_str("////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////");
        code.push_str(end);
        code.push_str("// This File is generated by the program, DO NOT EDIT MANUALLY!");
        code.push_str(end);
        code.push_str("// 此文件由程序生成, 切勿手动编辑!");
        code.push_str(end);
        code.push_str("////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////");
        code.push_str(end);

        // using
        code.push_str("using System;");
        code.push_str(end);
        code.push_str("using System.Linq;");
        code.push_str(end);
        code.push_str("using System.Collections;");
        code.push_str(end);
        code.push_str("using System.Collections.Generic;");
        code.push_str(end);
        code.push_str("using Config.Common;");
        code.push_str(end);
        code.push_str(end);

        // #pragma
        code.push_str("#pragma warning disable 1591");
        code.push_str(end);
        code.push_str(end);

        // namespace-start
        code.push_str("namespace Config");
        code.push_str(end);
        code.push('{');
        code.push_str(end);

        // ItemClass
        code.push_str(self.item_class.gen_code(end, tab_nums + 1).as_str());
        code.push_str(end);
        // empty line
        code.push_str(end);
        // BaseClass
        code.push_str(self.base_class.gen_code(end, tab_nums + 1).as_str());
        code.push_str(end);

        // namespace-end
        code.push('}');

        code
    }
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            item_class: ItemClass::default(),
            base_class: BaseClass::default(),
            defaults: Rc::from(RefCell::from(DefaultData::default())),
            vals: Rc::from(RefCell::from(VarData::default())),
            key_type: Rc::from(RefCell::from(KeyType::None)),
            skip_cols: Vec::default(),
            required_fields: Rc::from(RefCell::from(Vec::default()))
        }
    }

    pub fn read_file<P: AsRef<Path>>(&mut self, base_name: &str, path: P) -> Result<()> {
        self.item_class.name = String::from(base_name);
        self.base_class.name = String::from(base_name);
        let table = Self::get_table_with_id(path, "Template")?;
        self.parse_template(table);
        Ok(())
    }

    pub fn generate(&self, end: &'static str) -> String {
        self.gen_code(end, 0)
    }

    pub(crate) fn get_table_with_id<P: AsRef<Path>>(path: P ,sheet: &str) -> Result<ExcelTable> {
        let file = ExcelFile::load_from_path(path);
        if let Ok(mut ff) = file {
            match ff.parse_workbook() {
                Ok(ret) => {
                    for (name, id) in ret.into_iter() {
                        if name == sheet {
                            if let Ok(table) = ff.parse_sheet(*id) {
                                return Ok(table);
                            }
                        }
                    }
                    return Err(Error::new(ErrorKind::Other, "sheet not found"));
                },
                Err(e) => {
                    return Err(Error::new(ErrorKind::Other, e));
                }
            }
        } else if let Err(e) = file {
            return Err(Error::new(ErrorKind::Other, e));
        } else {
            return Err(Error::new(ErrorKind::Other, "load from xlsx file failed"));
        }
    }

    //------------------------private---------------------------------

    fn parse_template(&mut self, table: ExcelTable) {
        let width = table.width();
        let height = table.height();
        let ls_map: LSMap = Rc::from(RefCell::from(HashMap::with_capacity(64)));
        let mut ls_seed = 0;

        // parse FK
        let mut fk_data = Vec::<RawValData>::default();
        for col in 0..width {
            if let (Some(v), Some(ty)) = (table.cell(col, DATA_FOREIGN_KEY_ROW), table.cell(col, DATA_TYPE_ROW)) {
                if v.starts_with('*') {
                    let mut vals: Vec<&str> = Vec::default();
                    for idx in DATA_DEFAULT_ROW..height-1 {
                        if let Some(d) = table.cell(col, idx) {
                            vals.push(d);
                        } else if let Some(default) = table.cell(col, DATA_DEFAULT_ROW) {
                            vals.push(default);
                        } else {
                            vals.push("");
                        }
                    }
                    fk_data.push((col, (&v[1..], vals, CellValue::get_type(ty))));
                }
            }
        }
        let fk_value = FKValue::new(fk_data);
        fk_value.parse();

        // check flag for (1, 3)
        if let Some(v) = table.cell(DATA_TEMPLATE_ID_POS.0, DATA_TEMPLATE_ID_POS.1) {
            if v.starts_with('#') {
                if v.contains("DefKey") {
                    self.key_type = Rc::from(RefCell::from(KeyType::DefKey(Vec::default())));
                } else {
                    self.key_type = Rc::from(RefCell::from(KeyType::OriginalTemplateId));
                }
            }
        }

        // collect skip_cols and required fields
        for col in 0..width {
            if let Some(v) = table.cell(col, DATA_IDENTIFY_ROW) {
                if v.starts_with('#') {
                    self.skip_cols.push(col);
                } else {
                    self.required_fields.as_ref().borrow_mut().push(Some(v.clone()));
                }
            } else {
                self.skip_cols.push(col);
            }
        }

        // pre-process LString
        let mut ls_cols: Vec<(usize, bool)> = Vec::default();
        for col in 0..width {
            if let Some(v) = table.cell(col, DATA_TYPE_ROW) {
                if v.contains("LString") {
                    if v.as_ref() == "LString" {
                        ls_cols.push((col, true))
                    } else {
                        ls_cols.push((col, false))
                    }
                }
            }
        }
        for row in DATA_START_ROW..height-1 {
            for td in ls_cols.iter() {
                if let Some(data) = table.cell(td.0, row) {
                    Self::pre_process_lstring(&ls_map, data, td.1, &mut ls_seed);
                } else {
                    // empty cell
                    if let Some(default) = table.cell(td.0, DATA_DEFAULT_ROW) {
                        Self::pre_process_lstring(&ls_map, default, td.1, &mut ls_seed);
                    } else {
                        Self::pre_process_lstring(&ls_map, "", true, &mut ls_seed);
                    }
                }
            }
        }

        for col in (0..width).filter(|x| !self.skip_cols.contains(x)) {
            let ident = table.cell(col, DATA_IDENTIFY_ROW).unwrap();
            let ty = convert_type(table.cell(col, DATA_TYPE_ROW).unwrap().clone());

            // collect (comment, identify, type) in row (1, 3, 4)
            if let Some(c1) = table.cell(col, DATA_COMMENT_ROW) {
                self.item_class.items.push((
                    Some(c1.clone()),
                    Some(ident.clone()),
                    Some(ty.clone()),
                ));
            }

            // collect defaults
            if let Some(default) = table.cell(col, DATA_DEFAULT_ROW) {
                use std::collections::hash_map::Entry;
                match self.defaults.as_ref().borrow_mut().0.entry(ident.clone()) {
                    Entry::Occupied(_) => {}
                    Entry::Vacant(e) => {
                        let fk_default = fk_value.get_value(col, DATA_DEFAULT_ROW);
                        if !fk_default.is_empty() {
                            e.insert(Box::new(CellValue::new(&Rc::from(String::from(fk_default)), &ty, &ls_map)));
                        } else {
                            e.insert(Box::new(CellValue::new(default, &ty, &ls_map)));
                        }
                    }
                }
            }

            // collect vars
            if !self.vals.as_ref().borrow_mut().0.contains_key(ident) {
                self.vals.as_ref().borrow_mut().0.insert(ident.clone(), Vec::default());
                
                for row in DATA_START_ROW..height-1 {
                    use std::collections::hash_map::Entry;
                    if let Some(v) = table.cell(col, row) {
                        match self.vals.as_ref().borrow_mut().0.entry(ident.clone()) {
                            Entry::Occupied(mut e) => {
                                let fk_v = fk_value.get_value(col, row);
                                if !fk_v.is_empty() {
                                    e.get_mut().push(Box::new(CellValue::new(&Rc::from(String::from(fk_v)), &ty, &ls_map)));
                                } else {
                                    e.get_mut().push(Box::new(CellValue::new(v, &ty, &ls_map)));
                                }
                            }
                            Entry::Vacant(_) => {}
                        }
                    } else {
                        // empty cell
                        match self.vals.as_ref().borrow_mut().0.entry(ident.clone()) {
                            Entry::Occupied(mut e) => {
                                if let Some(default) = table.cell(col, DATA_DEFAULT_ROW) {
                                    let fk_default = fk_value.get_value(col, DATA_DEFAULT_ROW);
                                    if !fk_default.is_empty() {
                                        e.get_mut().push(Box::new(CellValue::new(&Rc::from(String::from(fk_default)), &ty, &ls_map)));
                                    } else {
                                        e.get_mut().push(Box::new(CellValue::new(default, &ty, &ls_map)));
                                    }
                                } else {
                                    e.get_mut().push(Box::new(CellValue::new(&Rc::from(String::default()), &ty, &ls_map)));
                                }
                            }
                            Entry::Vacant(_) => {}
                        }
                    }
                }
            }
        }

        // item_class
        self.item_class.defaults = Some(Rc::downgrade(&self.defaults));
        self.item_class.vals = Some(Rc::downgrade(&self.vals));
        // base_class
        self.base_class.lines = height - DATA_START_ROW - 1;
        self.base_class.defaults = Some(Rc::downgrade(&self.defaults));
        self.base_class.vals = Some(Rc::downgrade(&self.vals));
        self.base_class.required_fields = Some(Rc::downgrade(&self.required_fields));
        self.base_class.keytypes = Some(Rc::downgrade(&self.key_type));

        // collect DefKey in col 1, data start frow row 8
        if let KeyType::DefKey(ref mut vec) = *self.key_type.as_ref().borrow_mut() {
            for row in DATA_START_ROW..height-1 {
                if let Some(v) = table.cell(DATA_TEMPLATE_ID_POS.0, row) {
                    vec.push((Some(v.clone()), row - DATA_START_ROW));
                }
            }
        }
    }

    fn pre_process_lstring<'a>(ls_map: &LSMap, val: &str, is_trivial: bool, ls_seed: &'a mut usize) {
        let mut data = ls_map.as_ref().borrow_mut();
        use std::collections::hash_map::Entry;

        if !is_trivial {
            let elements: Vec<&str> = val[1..val.len()-1].split(',').collect();
            for v in elements {
                match data.entry(Rc::from(String::from(v))) {
                    Entry::Occupied(_) => {}
                    Entry::Vacant(e) => {
                        e.insert(*ls_seed);
                        *ls_seed += 1;
                    }
                }
            }
        } else {
            match data.entry(Rc::from(String::from(val))) {
                Entry::Occupied(_) => {}
                Entry::Vacant(e) => {
                    e.insert(*ls_seed);
                    *ls_seed += 1;
                }
            }
        }
    }
}

fn convert_type(mut v: Rc<String>) -> Rc<String> {
    if let Some(s) = Rc::get_mut(&mut v) {
        // convert array
        if let Some(idx) = s.find('[') {
            let mut n = idx;
            while let Some(c) = s.chars().nth(n) {
                if c == ']' {
                    break;
                } else {
                    n = n + 1;
                }
            }
            s.replace_range(idx + 1..n, "");
        }
    }
    v
}
