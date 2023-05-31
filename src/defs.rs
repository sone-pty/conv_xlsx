use std::rc::Rc;

// 数据开始所在行
pub const DATA_START_ROW: usize = 8;
// 默认数据所在行
pub const DATA_DEFAULT_ROW: usize = 7;
// 注释所在行
pub const DATA_COMMENT_ROW: usize = 1;
// 标识符所在行
pub const DATA_IDENTIFY_ROW: usize = 3;
// 数据类型所在行
pub const DATA_TYPE_ROW: usize = 4;
// 模版ID字段所在单元格
pub const DATA_TEMPLATE_ID_POS: (usize, usize) = (1, 3);

pub type ItemStr = Option<Rc<String>>;

pub const OUTPUT_PATH: &'static str = "output";
