use crate::defs::{DEFAULT_LINES, ItemStr};
use crate::reference::RefData;

use super::{CodeGenerator, DefaultData, VarData, KeyType};
use std::fs::OpenOptions;
use std::io::{Write, Result};
use std::rc::Weak;
use std::cell::RefCell;

pub struct BaseClass {
    pub name: String,
    pub defaults: Option<Weak<RefCell<DefaultData>>>,
    pub vals: Option<Weak<RefCell<VarData>>>,
    pub lines: usize,
    pub required_fields: Option<Weak<RefCell<Vec<ItemStr>>>>,
    pub keytypes: Option<Weak<RefCell<KeyType>>>,
    pub refdata: Option<RefData>,
    pub additionals: RefCell<Vec<ItemStr>>,
}

impl Default for BaseClass {
    fn default() -> Self {
        BaseClass {
            name: String::default(),
            defaults: None,
            vals: None,
            lines: 0,
            required_fields: None,
            keytypes: None,
            refdata: None,
            additionals: RefCell::from(Vec::default()),
        }
    }
}

impl CodeGenerator for BaseClass {
    fn gen_code<W: Write + ?Sized>(&self, end: &'static str, tab_nums: i32, stream: &mut W) -> Result<()> {
        let format = |n: i32, stream: &mut W| -> Result<()> {
            for _ in 0..n {
                stream.write("\t".as_bytes())?;
            }
            Ok(())
        };

        if let (Some(weak_defaults), Some(weak_vars), Some(weak_keys)) = (&self.defaults, &self.vals, &self.keytypes) {
            if let (Some(up_defaults), Some(up_vars), Some(up_keys)) = (weak_defaults.upgrade(), weak_vars.upgrade(), weak_keys.upgrade()) {
                let map_defaults = &up_defaults.as_ref().borrow().0;
                let map_vars = &up_vars.as_ref().borrow().0;
                let keys = up_keys.as_ref().borrow();

                if let Some(rfds) = self.required_fields.as_ref().unwrap().upgrade() {
                    let requires = &rfds.as_ref().borrow();

                    //--------------fixed code----------------------------
                    format(tab_nums, stream)?;
                    stream.write("[Serializable]".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums, stream)?;
                    stream.write("public class ".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write(" : IEnumerable<".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item>, IConfigData".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("public static ".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write(" Instance = new ".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("();".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("private readonly Dictionary<string, int> _refNameMap = new Dictionary<string, int>();".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("private List<".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item> _dataArray = null;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    //--------------fixed code----------------------------
                
                    // DefKey static class
                    if let KeyType::DefKey(ref vals) = *keys {
                        stream.write(end.as_bytes())?;
                        format(tab_nums + 1, stream)?;
                        stream.write("public static class DefKey".as_bytes())?;
                        stream.write(end.as_bytes())?;
                        format(tab_nums + 1, stream)?;
                        stream.write("{".as_bytes())?;
                        stream.write(end.as_bytes())?;

                        if self.refdata.is_none() {
                            for v in vals {
                                if let Some(ref v1) = v.0 {
                                    if !v1.is_empty() {
                                        format(tab_nums + 2, stream)?;
                                        stream.write("public const short ".as_bytes())?;
                                        stream.write(v1.as_bytes())?;
                                        stream.write(" = ".as_bytes())?;
                                        stream.write(v.1.to_string().as_bytes())?;
                                        stream.write(";".as_bytes())?;
                                        stream.write(end.as_bytes())?;
                                    }
                                }
                            }
                        } else {
                            let refdata = self.refdata.as_ref().unwrap();
                            for v in vals {
                                if let (Some(ref v0), Some(ref v2)) = (&v.0, &v.2) {
                                    if !v0.is_empty() {
                                        if refdata.data.contains_key(v2.as_str()) {
                                            format(tab_nums + 2, stream)?;
                                            stream.write("public const short ".as_bytes())?;
                                            stream.write(v0.as_bytes())?;
                                            stream.write(" = ".as_bytes())?;
                                            stream.write(refdata.data[v2.as_str()].to_string().as_bytes())?;
                                            stream.write(";".as_bytes())?;
                                            stream.write(end.as_bytes())?;
                                        } else {
                                            self.additionals.borrow_mut().push(Some(v2.clone()));
                                        }
                                    }
                                }
                            }
                        }

                        format(tab_nums + 1, stream)?;
                        stream.write("}".as_bytes())?;
                        stream.write(end.as_bytes())?;
                    }

                    for term in 0..(self.lines / DEFAULT_LINES)+1 {
                        stream.write(end.as_bytes())?;
                        format(tab_nums + 1, stream)?;
                        stream.write("private void CreateItems".as_bytes())?;
                        stream.write(term.to_string().as_bytes())?;
                        stream.write("()".as_bytes())?;
                        stream.write(end.as_bytes())?;
                        format(tab_nums + 1, stream)?;
                        stream.write("{".as_bytes())?;
                        stream.write(end.as_bytes())?;

                        let idx = term * DEFAULT_LINES;
                        let end_idx = if self.lines - idx < DEFAULT_LINES { self.lines } else { idx + DEFAULT_LINES };
                        for row in idx..end_idx {
                            format(tab_nums + 2, stream)?;
                            stream.write("_dataArray.Add(new ".as_bytes())?;
                            stream.write(self.name.as_bytes())?;
                            stream.write("Item(".as_bytes())?;
                            stream.write(row.to_string().as_bytes())?;
                            stream.write(",".as_bytes())?;

                            for i in 1..requires.len() {
                                if let Some(Some(d)) = requires.get(i) {
                                    if let Some(vv) = map_vars.get(d) {
                                        if vv[row].is_none() {
                                            if let Some(defv) = map_defaults.get(d) {
                                                defv.gen_code(stream)?;
                                            } else {
                                                vv[row].gen_code(stream)?;
                                            }
                                        } else {
                                            vv[row].gen_code(stream)?;
                                        }
                                        if i != requires.len()-1 {
                                            stream.write(",".as_bytes())?;
                                        }
                                    }
                                }
                            }

                            stream.write("));".as_bytes())?;
                            stream.write(end.as_bytes())?;
                        }

                        format(tab_nums + 1, stream)?;
                        stream.write("}".as_bytes())?;
                        stream.write(end.as_bytes())?;
                    }

                    //--------------------------Init-begin----------------------------------
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("public void Init()".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("_refNameMap.Clear();".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("_refNameMap.Load(\"".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("\");".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("_extraDataMap.Clear();".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("_dataArray = new List<".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item>( ".as_bytes())?;
                    stream.write(self.lines.to_string().as_bytes())?;
                    stream.write(" ) {};".as_bytes())?;
                    for term in 0..(self.lines / DEFAULT_LINES)+1 {
                        stream.write(end.as_bytes())?;
                        format(tab_nums + 2, stream)?;
                        stream.write("CreateItems".as_bytes())?;
                        stream.write(term.to_string().as_bytes())?;
                        stream.write("();".as_bytes())?;
                    }
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("}".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    //--------------------------Init-end.as_bytes()----------------------------------

                    //--------------------------GetItemId-begin----------------------------------
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("public int GetItemId(string refName)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("if (_refNameMap.TryGetValue(refName, out var id))".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("return id;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("throw new Exception($\"{refName} not found.\");".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("}".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    //--------------------------GetItemId-end.as_bytes()----------------------------------

                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("private readonly Dictionary<int, ".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item> _extraDataMap = new Dictionary<int, ".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item>();".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    // empty line
                    stream.write(end.as_bytes())?;

                    //--------------------------AddExtraItem-begin----------------------------------
                    format(tab_nums + 1, stream)?;
                    stream.write("public int AddExtraItem(string identifier, string refName, object configItem)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("var item = (".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item)configItem;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("var id = (int) item.TemplateId;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("if (id < _dataArray.Count)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("throw new Exception($\"".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write(" template id {item.TemplateId} created by {identifier} already exist.\");".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("if (_extraDataMap.ContainsKey(id))".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("throw new Exception($\"".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write(" extra template id {item.TemplateId} created by {identifier} already exist.\");".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("if (_refNameMap.TryGetValue(refName, out var refId))".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("throw new Exception($\"".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write(" template reference name {refName}(id = {item.TemplateId}) created by {identifier} already exist with templateId {refId}).\");".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("_refNameMap.Add(refName, id);".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("_extraDataMap.Add(id, item);".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("return id;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("}".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    //--------------------------AddExtraItem-end.as_bytes()----------------------------------

                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("public ".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item this[sbyte id] => GetItem(id);".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("public ".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item this[int id] => GetItem((sbyte)id);".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("public ".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item this[string refName] => this[_refNameMap[refName]];".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    stream.write(end.as_bytes())?;

                    //--------------------------GetItem-begin----------------------------------
                    format(tab_nums + 1, stream)?;
                    stream.write("public ".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item GetItem(sbyte id)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("if (id < 0) return null;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("if (id < _dataArray.Count) return _dataArray[(int)id];".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("if (_extraDataMap.TryGetValue((int) id, out var item)) return item;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("// 预期为有效 Id 但仍然访问不到数据时".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("GameData.Utilities.AdaptableLog.TagWarning(GetType().FullName, $\"index {id} is not in range [0, {_dataArray.Count}) and is not defined in _extraDataMap (count: {_extraDataMap.Count})\");".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("return null;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("}".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    stream.write(end.as_bytes())?;
                    //--------------------------GetItem-end.as_bytes()----------------------------------
                    
                    //--------------------------RequiredFields-begin----------------------------------
                    format(tab_nums + 1, stream)?;
                    stream.write("private readonly HashSet<string> RequiredFields = new HashSet<string>()".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    for v in requires.iter() {
                        if let Some(vv) = v {
                            format(tab_nums + 2, stream)?;
                            stream.write("\"".as_bytes())?;
                            stream.write(vv.as_bytes())?;
                            stream.write("\"".as_bytes())?;
                            stream.write(",".as_bytes())?;
                            stream.write(end.as_bytes())?;
                        }
                    }
                    format(tab_nums + 1, stream)?;
                    stream.write("};".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    //--------------------------RequiredFields-end.as_bytes()----------------------------------

                    //--------------------------GetAllKeys-begin----------------------------------
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("public List<sbyte> GetAllKeys()".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("return (from item in _dataArray where null != item select item.TemplateId).ToList();".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("}".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    //--------------------------GetAllKeys-end.as_bytes()----------------------------------

                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("public int Count => _dataArray.Count;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("public int CountWithExtra => Count + _extraDataMap.Count;".as_bytes())?;
                    stream.write(end.as_bytes())?;

                    //--------------------------Iterate-begin----------------------------------
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("public void Iterate(Func<".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item,bool> iterateFunc)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("if(null == iterateFunc)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("return;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("foreach(".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item item in _dataArray)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("if(null == item)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 4, stream)?;
                    stream.write("continue;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("if(!iterateFunc(item))".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 4, stream)?;
                    stream.write("break;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("}".as_bytes())?;

                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("foreach(".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item item in _extraDataMap.Values)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("if(null == item)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 4, stream)?;
                    stream.write("continue;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("if(!iterateFunc(item))".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 4, stream)?;
                    stream.write("break;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("}".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("}".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    //--------------------------Iterate-end.as_bytes()----------------------------------

                    //--------------------------GetEnumerator-begin----------------------------------
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("IEnumerator<".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item> IEnumerable<".as_bytes())?;
                    stream.write(self.name.as_bytes())?;
                    stream.write("Item>.GetEnumerator()".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("foreach (var item in _dataArray)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("yield return item;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("foreach (var item in _extraDataMap.Values)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("yield return item;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("}".as_bytes())?;
                    stream.write(end.as_bytes())?;

                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("IEnumerator IEnumerable.GetEnumerator()".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("{".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("foreach (var item in _dataArray)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("yield return item;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 2, stream)?;
                    stream.write("foreach (var item in _extraDataMap.Values)".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 3, stream)?;
                    stream.write("yield return item;".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    format(tab_nums + 1, stream)?;
                    stream.write("}".as_bytes())?;
                    stream.write(end.as_bytes())?;
                    //--------------------------GetEnumerator-end.as_bytes()----------------------------------

                    format(tab_nums, stream)?;
                    stream.write("}".as_bytes())?;
                }
            }
        }

        // re-write ref.txt
        if self.refdata.is_some() {
            let refdata = self.refdata.as_ref().unwrap();
            let mut file = OpenOptions::new().append(true).open(refdata.file.clone())?;
            let mut num = refdata.max_num;

            for v in self.additionals.borrow().iter() {
                let vv = v.as_ref().unwrap();
                file.write(vv.as_bytes())?;
                file.write(end.as_bytes())?;
                file.write(num.to_string().as_bytes())?;
                num += 1;
                file.write(end.as_bytes())?;
            }
            file.flush()?;
        }

        Ok(())
    }
}