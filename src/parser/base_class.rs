use crate::defs::{DEFAULT_LINES, ItemStr};

use super::cell_value::{CellValue, self};
use super::{CodeGenerator, DefaultData, VarData};
use std::rc::Weak;
use std::cell::RefCell;

pub struct BaseClass {
    pub name: String,
    pub defaults: Option<Weak<RefCell<DefaultData>>>,
    pub vals: Option<Weak<RefCell<VarData>>>,
    pub lines: usize,
    pub required_fields: Option<Weak<RefCell<Vec<ItemStr>>>>
}

impl Default for BaseClass {
    fn default() -> Self {
        BaseClass {
            name: String::default(),
            defaults: None,
            vals: None,
            lines: 0,
            required_fields: None
        }
    }
}

impl CodeGenerator for BaseClass {
    type Output = String;

    fn gen_code(&self, end: &'static str, tab_nums: i32) -> Self::Output {
        let mut code = String::with_capacity(512);
        
        let format = |n: i32, code: &mut String| {
            for _ in 0..n {
                code.push('\t');
            }
        };

        if let (Some(weak_defaults), Some(weak_vars)) = (&self.defaults, &self.vals) {
            if let (Some(up_defaults), Some(up_vars)) = (weak_defaults.upgrade(), weak_vars.upgrade()) {
                let map_defaults = &up_defaults.as_ref().borrow().0;
                let map_vars = &up_vars.as_ref().borrow().0;

                if let Some(rfds) = self.required_fields.as_ref().unwrap().upgrade() {
                    let requires = &rfds.as_ref().borrow();

                    //--------------fixed code----------------------------
                    format(tab_nums, &mut code);
                    code.push_str("[Serializable]");
                    code.push_str(end);
                    format(tab_nums, &mut code);
                    code.push_str("public class ");
                    code.push_str(&self.name);
                    code.push_str(" : IEnumerable<");
                    code.push_str(&self.name);
                    code.push_str("Item>, IConfigData");
                    code.push_str(end);
                    format(tab_nums, &mut code);
                    code.push('{');
                    code.push_str(end);
                    format(tab_nums + 1, &mut code);
                    code.push_str("public static ");
                    code.push_str(&self.name);
                    code.push_str(" Instance = new ");
                    code.push_str(&self.name);
                    code.push_str("();");
                    code.push_str(end);
                    format(tab_nums + 1, &mut code);
                    code.push_str("private readonly Dictionary<string, int> _refNameMap = new Dictionary<string, int>();");
                    code.push_str(end);
                    format(tab_nums + 1, &mut code);
                    code.push_str("private List<");
                    code.push_str(&self.name);
                    code.push_str("Item> _dataArray = null;");
                    code.push_str(end);
                    //--------------fixed code----------------------------
                
                    //TODO: DefKey static class

                    for term in 0..(self.lines / DEFAULT_LINES)+1 {
                        code.push_str(end);
                        format(tab_nums + 1, &mut code);
                        code.push_str("private void CreateItems");
                        code.push_str(&term.to_string());
                        code.push_str("()");
                        code.push_str(end);
                        format(tab_nums + 1, &mut code);
                        code.push('{');
                        code.push_str(end);

                        let idx = term * DEFAULT_LINES;
                        let end_idx = if self.lines - idx < DEFAULT_LINES { self.lines } else { idx + DEFAULT_LINES };
                        for row in idx..end_idx {
                            format(tab_nums + 2, &mut code);
                            code.push_str("_dataArray.Add(new ");
                            code.push_str(&self.name);
                            code.push_str("Item(");
                            code.push_str(&row.to_string());
                            code.push(',');

                            for i in 1..requires.len() {
                                if let Some(Some(d)) = requires.get(i) {
                                    if let Some(vv) = map_vars.get(d) {
                                        if vv[row].is_none() {
                                            if let Some(defv) = map_defaults.get(d) {
                                                code.push_str(&defv.gen_code());
                                            } else {
                                                code.push_str(&CellValue::DNone(cell_value::NoneValue).gen_code());
                                            }
                                        } else {
                                            code.push_str(&vv[row].gen_code());
                                        }
                                        code.push(',');
                                    }
                                }
                            }

                            code.remove(code.len() - 1);
                            code.push_str("));");
                            code.push_str(end);
                        }

                        format(tab_nums + 1, &mut code);
                        code.push('}');
                        code.push_str(end);
                    }

                    //--------------------------Init----------------------------------
                    code.push_str(end);
                    format(tab_nums + 1, &mut code);
                    code.push_str("public void Init()");
                    code.push_str(end);
                    format(tab_nums + 1, &mut code);
                    code.push('{');
                    code.push_str(end);
                    format(tab_nums + 2, &mut code);
                    code.push_str("_refNameMap.Clear();");
                    code.push_str(end);
                    format(tab_nums + 2, &mut code);
                    code.push_str("_refNameMap.Load(\"");
                    code.push_str(&self.name);
                    code.push_str("\");");
                    code.push_str(end);
                    format(tab_nums + 2, &mut code);
                    code.push_str("_extraDataMap.Clear();");
                    code.push_str(end);
                    format(tab_nums + 2, &mut code);
                    code.push_str("_dataArray = new List<");
                    code.push_str(&self.name);
                    code.push_str("Item>( ");
                    code.push_str(&self.lines.to_string());
                    code.push_str(" ) {};");
                    for term in 0..(self.lines / DEFAULT_LINES)+1 {
                        code.push_str(end);
                        format(tab_nums + 2, &mut code);
                        code.push_str("CreateItems");
                        code.push_str(&term.to_string());
                        code.push_str("();");
                    }
                    code.push_str(end);
                    format(tab_nums + 1, &mut code);
                    code.push('}');
                    code.push_str(end);
                    //--------------------------Init----------------------------------

                    //--------------------------GetItemId----------------------------------
                    code.push_str(end);
                    format(tab_nums + 1, &mut code);
                    code.push_str("public int GetItemId(string refName)");
                    code.push_str(end);
                    format(tab_nums + 1, &mut code);
                    code.push('{');
                    code.push_str(end);
                    format(tab_nums + 2, &mut code);
                    code.push_str("if (_refNameMap.TryGetValue(refName, out var id))");
                    code.push_str(end);
                    format(tab_nums + 3, &mut code);
                    code.push_str("return id;");
                    code.push_str(end);
                    format(tab_nums + 2, &mut code);
                    code.push_str("throw new Exception($\"{refName} not found.\");");
                    code.push_str(end);
                    format(tab_nums + 1, &mut code);
                    code.push('}');
                    code.push_str(end);
                    //--------------------------GetItemId----------------------------------

                    code.push_str(end);
                    format(tab_nums + 1, &mut code);
                    code.push_str("private readonly Dictionary<int, ");
                    code.push_str(&self.name);
                    code.push_str("Item> _extraDataMap = new Dictionary<int, ");
                    code.push_str(&self.name);
                    code.push_str("Item>();");
                    code.push_str(end);
                    // empty line
                    code.push_str(end);

                    //--------------------------AddExtraItem----------------------------------
                    
                    //--------------------------AddExtraItem----------------------------------


                    format(tab_nums, &mut code);
                    code.push('}');
                }
            }
        }

        code
    }
}