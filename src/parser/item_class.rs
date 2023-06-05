use super::{CodeGenerator, DefaultData, VarData};
use crate::defs::ItemStr;
use std::cell::RefCell;
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
    type Output = String;

    fn gen_code(&self, end: &'static str, tab_nums: i32) -> Self::Output {
        let mut code: String = String::with_capacity(2048);

        let format = |n: i32, code: &mut String| {
            for _ in 0..n {
                code.push('\t');
            }
        };

        let comment = |content: &str, code: &mut String| {
            format(tab_nums + 1, code);
            code.push_str("/// <summary>");
            code.push_str(end);
            format(tab_nums + 1, code);
            code.push_str("/// ");
            code.push_str(content);
            code.push_str(end);
            format(tab_nums + 1, code);
            code.push_str("/// </summary>");
            code.push_str(end);
        };

        if let (Some(weak_defaults), Some(weak_vars)) = (&self.defaults, &self.vals) {
            if let (Some(up_defaults), Some(up_vars)) = (weak_defaults.upgrade(), weak_vars.upgrade()) {
                let map_defaults = &up_defaults.borrow().0;
                let map_vars = &up_vars.borrow().0;

                let mut base_name = String::from(&self.name);
                base_name.push_str("Item");

                format(tab_nums, &mut code);
                code.push_str("[Serializable]");
                code.push_str(end);
                format(tab_nums, &mut code);
                code.push_str("public class ");
                code.push_str(&base_name);
                code.push_str(end);
                format(tab_nums, &mut code);
                code.push('{');
                code.push_str(end);

                // with args
                let mut construct_0 = String::with_capacity(64);
                construct_0.push_str("public ");
                construct_0.push_str(&base_name);
                construct_0.push('(');

                // default
                let mut construct_1 = String::with_capacity(64);
                construct_1.push_str("public ");
                construct_1.push_str(&base_name);
                construct_1.push_str("()");
                construct_1.push_str(end);

                let mut count: i32 = 0;
                for item in self.items.iter() {
                    if let Some(item_comment) = &item.0 {
                        comment(item_comment, &mut code);
                    } else {
                        println!("ItemClass gen_code failed in comment");
                    }

                    if let (Some(item_type), Some(item_identify)) = (&item.2, &item.1) {
                        format(tab_nums + 1, &mut code);
                        code.push_str("public readonly ");
                        let s = item_type.clone().as_ref().clone();
                        code.push_str(&s.replace("LString", "string"));
                        code.push(' ');

                        let cell_ident = map_vars.get(item_identify).unwrap();
                        if !cell_ident.is_empty() {
                            if cell_ident[0].is_lstring() {
                                construct_0.push_str("int");
                            } else if cell_ident[0].is_lstring_arr() {
                                construct_0.push_str("int[]");
                            } else {
                                construct_0.push_str(item_type);
                            }
                        }

                        construct_0.push_str(" arg");
                        construct_0.push_str(&count.to_string());
                        construct_0.push(',');
                    } else {
                        println!("ItemClass gen_code failed in type");
                    }

                    if let Some(item_identify) = &item.1 {
                        code.push_str(item_identify);
                        code.push(';');
                        code.push_str(end);
                    }

                    code.push_str(end);
                    count += 1;
                }

                construct_0.remove(construct_0.len() - 1);
                construct_0.push(')');
                construct_0.push_str(end);
                format(tab_nums + 1, &mut construct_0);
                construct_0.push('{');
                construct_0.push_str(end);
                format(tab_nums + 1, &mut construct_1);
                construct_1.push('{');
                construct_1.push_str(end);

                count = 0;
                for item in self.items.iter() {
                    if let Some(item_identify) = &item.1 {
                        // with args
                        format(tab_nums + 2, &mut construct_0);
                        construct_0.push_str(item_identify);
                        
                        // process LString
                        let cell_ident = map_vars.get(item_identify).unwrap();
                        if !cell_ident.is_empty() {
                            if cell_ident[0].is_lstring() {
                                construct_0.push_str(" = LocalStringManager.GetConfig(\"");
                                construct_0.push_str(&base_name);
                                construct_0.push_str("_language\", arg");
                                construct_0.push_str(&count.to_string());
                                construct_0.push(')');
                            } else if cell_ident[0].is_lstring_arr() {
                                construct_0.push_str(" = LocalStringManager.ConvertConfigList(\"");
                                construct_0.push_str(&base_name);
                                construct_0.push_str("_language\", arg");
                                construct_0.push_str(&count.to_string());
                                construct_0.push(')');
                            } else {
                                construct_0.push_str(" = arg");
                                construct_0.push_str(&count.to_string());
                            }
                        }

                        construct_0.push(';');
                        construct_0.push_str(end);

                        // default
                        format(tab_nums + 2, &mut construct_1);
                        construct_1.push_str(item_identify);

                        if map_defaults.contains_key(item_identify) {
                            let val = map_defaults.get(item_identify).unwrap();
                            construct_1.push_str(" = ");
                            construct_1.push_str(&val.gen_code());
                            construct_1.push(';');
                        } else {
                            construct_1.push_str(" = default;");
                        }
                        construct_1.push_str(end);
                    }
                    count += 1;
                }

                format(tab_nums + 1, &mut construct_0);
                construct_0.push('}');
                construct_0.push_str(end);
                format(tab_nums + 1, &mut construct_1);
                construct_1.push('}');
                construct_1.push_str(end);

                // concat
                format(tab_nums + 1, &mut code);
                code.push_str(&construct_0);
                code.push_str(end);
                format(tab_nums + 1, &mut code);
                code.push_str(&construct_1);
                format(tab_nums, &mut code);
                code.push('}');
            }
        }

        code
    }
}
