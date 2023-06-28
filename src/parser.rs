use crate::{defs::*, reference::RefData};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    io::{Error, ErrorKind, Result, Write},
    rc::Rc, path::Path, fs::File, sync::Arc
};
use std::path::PathBuf;
use std::fs;
use lazy_static::lazy_static;
use xlsx_read::{excel_file::ExcelFile, excel_table::ExcelTable};

use item_class::ItemClass;
mod item_class;

use base_class::BaseClass;
mod base_class;

use cell_value::CellValue;
mod cell_value;

use self::fk_value::{FKValue, RawValData};
mod fk_value;

mod stack;
mod bm_search;
mod fsm;

type LSMap = Rc<RefCell<HashMap<Rc<String>, usize>>>;
type ENMap = Rc<RefCell<HashMap<ItemStr, ItemStr>>>;

trait CodeGenerator {
    fn gen_code<W: Write + ?Sized>(&self, end: &'static str, tab_nums: i32, stream: &mut W) -> Result<()>;
}

pub enum KeyType {
    None,
    DefKey(Vec<(ItemStr, usize, ItemStr)>),
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

lazy_static! (
    static ref ENUM_FLAGS_FILTER: HashSet<&'static str> = {
        let mut ret = HashSet::<&'static str>::default();
        ret.insert("Inherit");
        ret.insert("Archive, Inherit");
        ret.insert("Archive, Readonly");
        ret.insert("Readonly, Inherit");
        ret.insert("Archive, Readonly, Inherit");
        ret.insert("Archive");
        ret.insert("Readonly");
        ret.insert("0");
        ret.insert("1");
        ret
    };
);

pub struct Parser {
    item_class: ItemClass,
    base_class: BaseClass,
    defaults: Rc<RefCell<DefaultData>>,
    vals: Rc<RefCell<VarData>>,
    required_fields: Rc<RefCell<Vec<ItemStr>>>,
    key_type: Rc<RefCell<KeyType>>,
    skip_cols: Vec<usize>,
    enmap: Rc<RefCell<HashMap<String, ENMap>>>,
    nodefs: Rc<RefCell<HashSet<Rc<String>>>>,
    enumflags: Rc<RefCell<HashMap<String, Vec<Rc<String>>>>>
}

impl CodeGenerator for Parser {
    fn gen_code<W: Write + ?Sized>(&self, end: &'static str, tab_nums: i32, stream: &mut W) -> Result<()> {
        // comment
        stream.write("////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write("// This File is generated by the program, DO NOT EDIT MANUALLY!".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write("// 此文件由程序生成, 切勿手动编辑!".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write("////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////".as_bytes())?;
        stream.write(end.as_bytes())?;

        // using
        stream.write("using System;".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write("using System.Linq;".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write("using System.Collections;".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write("using System.Collections.Generic;".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write("using Config.Common;".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;

        // #pragma
        stream.write("#pragma warning disable 1591".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write(end.as_bytes())?;

        // namespace-start
        stream.write("namespace Config".as_bytes())?;
        stream.write(end.as_bytes())?;
        stream.write("{".as_bytes())?;
        stream.write(end.as_bytes())?;

        // ItemClass
        self.item_class.gen_code(end, tab_nums + 1, stream)?;
        stream.write(end.as_bytes())?;
        // empty line
        stream.write(end.as_bytes())?;
        // BaseClass
        self.base_class.gen_code(end, tab_nums + 1, stream)?;
        stream.write(end.as_bytes())?;

        // namespace-end
        stream.write("}".as_bytes())?;

        Ok(())
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
            required_fields: Rc::from(RefCell::from(Vec::default())),
            enmap: Rc::from(RefCell::from(HashMap::<String, ENMap>::default())),
            nodefs: Rc::default(),
            enumflags: Rc::default()
        }
    }

    pub fn read_file<P: AsRef<Path>>(&mut self, base_name: &str, path: P, refdata: Option<Arc<RefData>>) -> Result<()> {
        self.item_class.name = String::from(base_name);
        self.base_class.name = String::from(base_name);
        if refdata.is_some() {
            self.base_class.refdata = Some(refdata.unwrap().clone());
        }
        
        let file = ExcelFile::load_from_path(path);
        if let Ok(mut ff) = file {
            match ff.parse_workbook() {
                Ok(ret) => {
                    let mut template_table = Option::<ExcelTable>::None;
                    for (name, id) in ret.into_iter() {
                        if let Ok(table) = ff.parse_sheet(*id) {
                            match name.as_str() {
                                "Template" => { template_table = Some(table); },
                                v if v.starts_with("t_") => { self.parse_enum(table, &name[2..], base_name)?; }
                                _ => {}
                            }
                        }
                    }
                    template_table.map(|table| self.parse_template(table, base_name));
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

        Ok(())
    }

    pub fn generate<W: Write + ?Sized>(&self, end: &'static str, stream: &mut W) -> Result<()> {
        self.gen_code(end, 0, stream)
    }

    pub(crate) fn get_table_with_id<P: AsRef<Path>>(path: P, sheet: &str) -> Result<ExcelTable> {
        let file = ExcelFile::load_from_path(path);
        if let Ok(mut ff) = file {
            match ff.parse_workbook() {
                Ok(ret) => {
                    for (name, id) in ret.into_iter() {
                        if name == sheet || sheet.is_empty() {
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

    fn parse_enum(&mut self, table: ExcelTable, enum_name: &str, base_name: &str) -> Result<()> {
        let height = table.height();
        let en_map = ENMap::default();
        
        let dest = format!("{}/E{}{}.cs", OUTPUT_ENUM_CODE_DIR, base_name, enum_name);
        if let Ok(mut file) = File::create(dest) {
            file.write("#pragma warning disable 1591".as_bytes())?;
            file.write(LINE_END_FLAG.as_bytes())?;
            file.write(LINE_END_FLAG.as_bytes())?;
            file.write("/// <summary>".as_bytes())?;
            file.write(LINE_END_FLAG.as_bytes())?;
            file.write_fmt(format_args!("/// {} -> {}{}", base_name, enum_name, LINE_END_FLAG))?;
            file.write("/// </summary>".as_bytes())?;
            file.write(LINE_END_FLAG.as_bytes())?;
            file.write_fmt(format_args!("public enum E{}{}{}", base_name, enum_name, LINE_END_FLAG))?;
            file.write("{".as_bytes())?;
            file.write(LINE_END_FLAG.as_bytes())?;

            for row in 0..height {
                if let (Some(ident), Some(val), Some(desc)) = 
                    (table.cell(ENUM_COL_IDENT, row), table.cell(ENUM_COL_VAL, row), table.cell(ENUM_COL_DESC, row)) {
                    file.write_fmt(format_args!("{}/// <summary>{}", '\t', LINE_END_FLAG))?;
                    file.write_fmt(format_args!("{}/// {}{}", '\t', desc, LINE_END_FLAG))?;
                    file.write_fmt(format_args!("{}/// </summary>{}", '\t', LINE_END_FLAG))?;
                    file.write_fmt(format_args!("{}{} = {},{}", '\t', ident, val, LINE_END_FLAG))?;
                    en_map.as_ref().borrow_mut().insert(Some(desc.clone()), Some(ident.clone()));
                }
            }

            file.write_fmt(format_args!("{}Count{}", '\t', LINE_END_FLAG))?;
            file.write("}".as_bytes())?;
            file.flush()?;
            self.enmap.as_ref().borrow_mut().insert(String::from(enum_name), en_map);
        }

        Ok(())
    }

    fn parse_template(&mut self, table: ExcelTable, base_name: &str) {
        let width = table.width();
        let mut height = table.height();
        let ls_map: LSMap = Rc::from(RefCell::from(HashMap::with_capacity(64)));
        let mut ls_seed = 0;

        // get height
        for row in 0..height {
            if let Some(v) = table.cell(0, row) {
                if v.contains("EOF") {
                    height = row + 1;
                    break;
                }
            }
        }

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

                    let mut mty = ty.clone();
                    convert_type(Rc::make_mut(&mut mty));
                    fk_data.push((col, (&v[1..], vals, CellValue::get_type(&mty))));
                }
            }
        }
        let fk_value = FKValue::new(fk_data);
        fk_value.parse();

        let mut defkey_col = DATA_TEMPLATE_ID_POS.1;
        // collect skip_cols and required fields and defkeys and enum flags
        for col in 0..width {
            if let Some(v) = table.cell(col, DATA_IDENTIFY_ROW) {
                if v.starts_with('#') {
                    self.skip_cols.push(col);
                    if v.contains("DefKey") {
                        self.key_type = Rc::from(RefCell::from(KeyType::DefKey(Vec::default())));
                        defkey_col = col;
                    }
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
                if v.contains("LString") || v.contains("Lstring") {
                    if v.as_ref() == "LString" || v.as_ref() == "Lstring" {
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
            let mut ty = table.cell(col, DATA_TYPE_ROW).unwrap().clone();
            convert_type(Rc::make_mut(&mut ty));

            if let Some(v) = table.cell(col, DATA_ENUM_FLAG_ROW) {
                if !ENUM_FLAGS_FILTER.contains(v.as_str()) && v.chars().all(|c| c.is_alphabetic()) {
                    use std::collections::hash_map::Entry;
                    match self.enumflags.as_ref().borrow_mut().entry(String::from(v.as_str())) {
                        Entry::Occupied(mut e) => {
                            e.get_mut().push(ident.clone());
                        }
                        Entry::Vacant(e) => {
                            let mut vec = Vec::<Rc<String>>::with_capacity(10);
                            vec.push(ident.clone());
                            e.insert(vec);
                        }
                    }
                }
            }

            // collect (comment, identify, type) in row (1, 3, 4)
            if let Some(c1) = table.cell(col, DATA_COMMENT_ROW) {
                self.item_class.items.push((
                    Some(c1.clone()),
                    Some(ident.clone()),
                    Some(ty.clone()),
                ));
            } else {
                self.item_class.items.push((
                    None,
                    Some(ident.clone()),
                    Some(ty.clone()),
                ));
            }

            // collect defaults
            if let Some(default) = table.cell(col, DATA_DEFAULT_ROW) {
                if default.as_str() == "None" || default.is_empty() {
                    self.nodefs.as_ref().borrow_mut().insert(ident.clone());
                } else {
                    use std::collections::hash_map::Entry;
                    match self.defaults.as_ref().borrow_mut().0.entry(ident.clone()) {
                        Entry::Occupied(_) => {}
                        Entry::Vacant(e) => {
                            let fk_default = fk_value.get_value(col, DATA_DEFAULT_ROW);
                            if !fk_default.is_empty() {
                                e.insert(Box::new(CellValue::new(&Rc::from(String::from(fk_default)), &ty, &ls_map, &ident, &self.enmap, base_name)));
                            } else {
                                e.insert(Box::new(CellValue::new(default, &ty, &ls_map, &ident, &self.enmap, base_name)));
                            }
                        }
                    }
                }
            } else {
                self.nodefs.as_ref().borrow_mut().insert(ident.clone());
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
                                    e.get_mut().push(Box::new(CellValue::new(&Rc::from(String::from(fk_v)), &ty, &ls_map, &ident, &self.enmap, base_name)));
                                } else {
                                    e.get_mut().push(Box::new(CellValue::new(v, &ty, &ls_map, &ident, &self.enmap, base_name)));
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
                                        e.get_mut().push(Box::new(CellValue::new(&Rc::from(String::from(fk_default)), &ty, &ls_map, &ident, &self.enmap, base_name)));
                                    } else {
                                        e.get_mut().push(Box::new(CellValue::new(default, &ty, &ls_map, &ident, &self.enmap, base_name)));
                                    }
                                } else {
                                    e.get_mut().push(Box::new(CellValue::new(&Rc::default(), &ty, &ls_map, &ident, &self.enmap, base_name)));
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
        self.item_class.enmaps = Some(Rc::downgrade(&self.enmap));
        self.item_class.enumflags = Some(Rc::downgrade(&self.enumflags));
        // base_class
        self.base_class.lines = height - DATA_START_ROW - 1;
        self.base_class.defaults = Some(Rc::downgrade(&self.defaults));
        self.base_class.vals = Some(Rc::downgrade(&self.vals));
        self.base_class.required_fields = Some(Rc::downgrade(&self.required_fields));
        self.base_class.keytypes = Some(Rc::downgrade(&self.key_type));
        self.base_class.nodefs = Rc::downgrade(&self.nodefs);
        self.base_class.enumflags = Some(Rc::downgrade(&self.enumflags));
        if let Some(v) = table.cell(0, 4) {
            self.base_class.id_type = v.clone();
        }

        // collect DefKey in col 1, data start frow row 8
        if let KeyType::DefKey(ref mut vec) = *self.key_type.as_ref().borrow_mut() {
            for row in DATA_START_ROW..height-1 {
                if let (Some(v0), Some(v1)) = (table.cell(0, row), table.cell(defkey_col, row)) {
                    vec.push((Some(v1.clone()), row - DATA_START_ROW, Some(v0.clone())));
                }
            }
        }
    }

    fn pre_process_lstring<'a>(ls_map: &LSMap, val: &str, is_trivial: bool, ls_seed: &'a mut usize) {
        if val.is_empty() { return; }
        let mut data = ls_map.as_ref().borrow_mut();
        use std::collections::hash_map::Entry;

        if !is_trivial {
            let pre_str = val[1..val.len()-1].chars().filter(|c| *c != ' ').collect::<String>();
            let elements: Vec<&str> = pre_str.split(',').collect();
            for v in elements {
                if v.is_empty() { continue; }
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

fn convert_type(v: &mut String) {
    if let Some(idx) = v.find('[') {
        let mut n = idx;
        while let Some(c) = v.chars().nth(n) {
            if c == ']' {
                break;
            } else {
                n = n + 1;
            }
        }
        v.replace_range(idx + 1..n, "");
    }
}

pub fn find_file<P: AsRef<Path>>(dir: P, filename: &str) -> PathBuf {
    let dir = dir.as_ref();

    if let Ok(rdir) = fs::read_dir(dir) {
        for entry in rdir {
            if let Ok(e) = entry {
                let path = e.path();
                if path.is_dir() {
                    let ret = find_file(&path, filename);
                    if ret.is_file() {
                        return ret;
                    }
                } else if path.file_name().and_then(|name| name.to_str()) == Some(filename) {
                    return path.to_path_buf()
                }
            }
        }
        PathBuf::default()
    } else {
        PathBuf::default()
    }
}