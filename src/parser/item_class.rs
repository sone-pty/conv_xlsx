use super::{CodeGenerator, DefaultData, VarData};
use crate::defs::ItemStr;
use std::cell::RefCell;
use std::io::{Write, Result};
use std::rc::Weak;

pub struct ItemClass {
    pub name: String,
    pub items: Vec<(ItemStr, ItemStr, ItemStr)>, // (comment, identify, type)
    pub defaults: Option<Weak<RefCell<DefaultData>>>,
    pub vals: Option<Weak<RefCell<VarData>>>
}

impl Default for ItemClass {
    fn default() -> Self {
        ItemClass {
            name: String::default(),
            items: Vec::default(),
            defaults: None,
            vals: None
        }
    }
}

impl CodeGenerator for ItemClass {
    fn gen_code<W: Write + ?Sized>(&self, end: &'static str, tab_nums: i32, stream: &mut W) -> Result<()> {
        let format = |n: i32, stream: &mut W| -> Result<()> {
            for _ in 0..n {
                stream.write("\t".as_bytes())?;
            }
            Ok(())
        };

        let comment = |content: &str, stream: &mut W| -> Result<()> {
            format(tab_nums + 1, stream)?;
            stream.write("/// <summary>".as_bytes())?;
            stream.write(end.as_bytes())?;
            format(tab_nums + 1, stream)?;
            stream.write("/// ".as_bytes())?;
            stream.write(content.as_bytes())?;
            stream.write(end.as_bytes())?;
            format(tab_nums + 1, stream)?;
            stream.write("/// </summary>".as_bytes())?;
            stream.write(end.as_bytes())?;
            Ok(())
        };

        if let (Some(weak_defaults), Some(weak_vars)) = (&self.defaults, &self.vals) {
            if let (Some(up_defaults), Some(up_vars)) = (weak_defaults.upgrade(), weak_vars.upgrade()) {
                let map_defaults = &up_defaults.borrow().0;
                let map_vars = &up_vars.borrow().0;
                #[allow(unused_assignments)]
                let mut count = 0;
                let mut base_name = String::from(&self.name);
                base_name.push_str("Item");

                format(tab_nums, stream)?;
                stream.write("[Serializable]".as_bytes())?;
                stream.write(end.as_bytes())?;
                format(tab_nums, stream)?;
                stream.write("public class ".as_bytes())?;
                stream.write(base_name.as_bytes())?;
                stream.write(end.as_bytes())?;
                format(tab_nums, stream)?;
                stream.write("{".as_bytes())?;
                stream.write(end.as_bytes())?;

                for item in self.items.iter() {
                    if let Some(item_comment) = &item.0 {
                        comment(item_comment, stream)?;
                    } else {
                        println!("ItemClass gen_code failed in comment");
                    }

                    if let Some(item_type) = &item.2 {
                        format(tab_nums + 1, stream)?;
                        stream.write("public readonly ".as_bytes())?;
                        let s = item_type.clone().as_ref().clone();
                        stream.write(replace_lstring(&s).as_bytes())?;
                        stream.write(" ".as_bytes())?;
                    } else {
                        println!("ItemClass gen_code failed in type");
                    }

                    if let Some(item_identify) = &item.1 {
                        stream.write(item_identify.as_bytes())?;
                        stream.write(";".as_bytes())?;
                        stream.write(end.as_bytes())?;
                    }

                    stream.write(end.as_bytes())?;
                }

                // construct_0
                format(tab_nums + 1, stream)?;
                stream.write("public ".as_bytes())?;
                stream.write(base_name.as_bytes())?;
                stream.write("(".as_bytes())?;
                
                count = 0;
                for item in self.items.iter() {
                    if let (Some(item_identify), Some(item_type)) = (&item.1, &item.2) {
                        let cell_ident = map_vars.get(item_identify).unwrap();
                        if !cell_ident.is_empty() {
                            if cell_ident[0].is_lstring() {
                                stream.write("int".as_bytes())?;
                            } else if cell_ident[0].is_lstring_arr() {
                                stream.write("int[]".as_bytes())?;
                            } else {
                                stream.write(item_type.as_bytes())?;
                            }
                        }

                        stream.write(" arg".as_bytes())?;
                        stream.write(count.to_string().as_bytes())?;
                        if count < self.items.len()-1 {
                            stream.write(",".as_bytes())?;
                        }
                    }
                    count += 1;
                }

                stream.write(")".as_bytes())?;
                stream.write(end.as_bytes())?;
                format(tab_nums + 1, stream)?;
                stream.write("{".as_bytes())?;
                stream.write(end.as_bytes())?;

                count = 0;
                for item in self.items.iter() {
                    if let Some(item_identify) = &item.1 {
                        // with args
                        format(tab_nums + 2, stream)?;
                        stream.write(item_identify.as_bytes())?;
                        
                        // process LString
                        let cell_ident = map_vars.get(item_identify).unwrap();
                        if !cell_ident.is_empty() {
                            let countstr = count.to_string();
                            if cell_ident[0].is_lstring() {
                                stream.write(" = LocalStringManager.GetConfig(\"".as_bytes())?;
                                stream.write(base_name.as_bytes())?;
                                stream.write("_language\", arg".as_bytes())?;
                                stream.write(countstr.as_bytes())?;
                                stream.write(")".as_bytes())?;
                            } else if cell_ident[0].is_lstring_arr() {
                                stream.write(" = LocalStringManager.ConvertConfigList(\"".as_bytes())?;
                                stream.write(base_name.as_bytes())?;
                                stream.write("_language\", arg".as_bytes())?;
                                stream.write(countstr.as_bytes())?;
                                stream.write(")".as_bytes())?;
                            } else {
                                stream.write(" = arg".as_bytes())?;
                                stream.write(countstr.as_bytes())?;
                            }
                        }

                        stream.write(";".as_bytes())?;
                        stream.write(end.as_bytes())?;
                    }
                    count += 1;
                }

                format(tab_nums + 1, stream)?;
                stream.write("}".as_bytes())?;
                stream.write(end.as_bytes())?;
                stream.write(end.as_bytes())?;
                format(tab_nums + 1, stream)?;

                // construct_1
                stream.write("public ".as_bytes())?;
                stream.write(base_name.as_bytes())?;
                stream.write("()".as_bytes())?;
                stream.write(end.as_bytes())?;

                format(tab_nums + 1, stream)?;
                stream.write("{".as_bytes())?;
                stream.write(end.as_bytes())?;

                count = 0;
                for item in self.items.iter() {
                    if let Some(item_identify) = &item.1 {
                        // default
                        format(tab_nums + 2, stream)?;
                        stream.write(item_identify.as_bytes())?;

                        if map_defaults.contains_key(item_identify) {
                            let val = map_defaults.get(item_identify).unwrap();
                            stream.write(" = ".as_bytes())?;
                            val.gen_code(stream)?;
                            stream.write(";".as_bytes())?;
                        } else {
                            stream.write(" = default;".as_bytes())?;
                        }
                        stream.write(end.as_bytes())?;
                    }
                    count += 1;
                }

                format(tab_nums + 1, stream)?;
                stream.write("}".as_bytes())?;
                stream.write(end.as_bytes())?;

                format(tab_nums, stream)?;
                stream.write("}".as_bytes())?;
            }
        }

        Ok(())
    }
}

fn replace_lstring(val: &str) -> String {
    let mut ret = String::with_capacity(val.len());
    let indexs_1 = super::bm_search::bm_search(val, "LString");
    let indexs_2 = super::bm_search::bm_search(val, "Lstring");

    if indexs_1.is_empty() && indexs_2.is_empty() {
        return String::from(val);
    } else if indexs_1.is_empty() {
        if indexs_2[0] == 0 {
            ret.push_str("string");
            ret.push_str(&val[7..]);
        } else {
            ret.push_str(&val[..indexs_2[0]]);
            ret.push('s');
            ret.push_str(&val[indexs_2[0]+2..]);
        }
    } else if indexs_2.is_empty() {
        if indexs_1[0] == 0 {
            ret.push_str("string");
            ret.push_str(&val[7..]);
        } else {
            ret.push_str(&val[..indexs_1[0]]);
            ret.push('s');
            ret.push_str(&val[indexs_1[0]+2..]);
        }
    } else {
        unreachable!()
    }
    ret
}