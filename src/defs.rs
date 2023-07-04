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
// 外键标识所在行
pub const DATA_FOREIGN_KEY_ROW: usize = 5;
// 枚举标识所在行
pub const DATA_ENUM_FLAG_ROW: usize = 6;
// 模版ID字段所在单元格
pub const DATA_TEMPLATE_ID_POS: (usize, usize) = (1, 3);

pub type ItemStr = Option<Rc<String>>;

pub static mut OUTPUT_SCRIPT_CODE_DIR: &'static str = "ExportScripts/";
pub static mut OUTPUT_ENUM_CODE_DIR: &'static str = "ConfigExportEnum/";
pub static mut SOURCE_XLSXS_DIR: &'static str = "D:/Config-beta/";
pub static mut REF_TEXT_DIR: &'static str = "ConfigRefNameMapping/";

// 默认多少行数据切换构造方法
pub const DEFAULT_LINES: usize = 101;

// 默认的文件后缀
pub const DEFAULT_SOURCE_SUFFIX: &'static str = "xlsx";
pub const DEFAULT_DEST_SUFFIX: &'static str = "cs";
pub const DEFAULT_DEF_SUFFIX: &'static str = "ref.txt";

// enum列属性
pub const ENUM_COL_IDENT: usize = 0;
pub const ENUM_COL_VAL: usize = 1;
pub const ENUM_COL_DESC: usize = 2;

// 行结束符
pub const LINE_END_FLAG: &'static str = "\r\n";

// 请求最大延时(s)
pub const MAX_REQ_DELAY: u64 = 5;