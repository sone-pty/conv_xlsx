use crate::defs::DEFAULT_LINES;

use super::{CodeGenerator, DefaultData, VarData};
use std::rc::Weak;
use std::cell::RefCell;

pub struct BaseClass {
    pub name: String,
    pub defaults: Option<Weak<RefCell<DefaultData>>>,
    pub vals: Option<Weak<RefCell<VarData>>>,
    pub lines: usize
}

impl Default for BaseClass {
    fn default() -> Self {
        BaseClass {
            name: String::default(),
            defaults: None,
            vals: None,
            lines: 0
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
                let map_defaults = &up_defaults.borrow().0;
                let map_vars = &up_vars.borrow().0;
                
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
                }

                format(tab_nums, &mut code);
                code.push('}');
            }
        }

        code
    }
}