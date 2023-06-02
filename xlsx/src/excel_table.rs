
use std::rc::Rc;
use std::fmt;

pub struct ExcelTable {
    origin: (usize, usize),
    size: (usize, usize),
    cells: Vec<Option<Rc<String>>>,
    merged_cells: Vec<((usize, usize), (usize, usize))>,
}

pub struct RangeName<'a> (&'a ExcelTable, usize, usize, usize, usize);

impl<'a> fmt::Display for RangeName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.3, self.4) {
            (1, 1) => {
                self.0.fmt_column_name(self.1, f)?;
                self.0.fmt_row_name(self.2, f)
            },
            (0, 0) => Err(fmt::Error),
            (w, 0) => {
                self.0.fmt_column_name(self.1, f)?;
                f.write_str(":")?;
                self.0.fmt_column_name(self.1 + w - 1, f)
            },
            (0, h) => {
                self.0.fmt_row_name(self.2, f)?;
                f.write_str(":")?;
                self.0.fmt_row_name(self.2 + h - 1, f)
            }
            (w, h) => {
                self.0.fmt_column_name(self.1, f)?;
                self.0.fmt_row_name(self.2, f)?;
                f.write_str(":")?;
                self.0.fmt_column_name(self.1 + w - 1, f)?;
                self.0.fmt_row_name(self.2 + h - 1, f)
            }
        }
    }
}

impl ExcelTable {

    pub(crate) fn new(origin: (usize, usize), size: (usize, usize), cells: Vec<Option<Rc<String>>>, merged_cells: Vec<((usize, usize), (usize, usize))>) -> ExcelTable {
        ExcelTable { origin, size, cells, merged_cells }
    }
    

    pub fn size(&self) -> &(usize, usize) {
        &self.size
    }

    pub fn width(&self) -> usize {
        self.size.0
    }

    pub fn height(&self) -> usize {
        self.size.1
    }

    pub fn cell(&self, x: usize, y: usize) -> Option<&Rc<String>> {
        self.cells.get(y * self.size.0 + x)
            .and_then(|t| t.as_ref())
    }

    pub unsafe fn cell_unchecked(&self, x: usize, y: usize) -> Option<&Rc<String>> {
        self.cells.get_unchecked(y * self.size.0 + x).as_ref()
    }

    pub fn cell_content(&self, x: usize, y: usize) -> Option<&str> {
        self.cells.get(y * self.size.0 + x)
            .and_then(|t| t.as_ref().map(|t| t.as_str()))
    }

    pub unsafe fn cell_content_unchecked(&self, x: usize, y: usize) -> Option<&str> {
        self.cells.get_unchecked(y * self.size.0 + x).as_ref().map(|t| t.as_str())
    }

    pub fn get_column_name(&self, x: usize) -> String {
        let mut n = x + self.origin.0 - 1;
        let mut name = Vec::new();
        name.push(b'A' + (n % 26) as u8);
        n /= 26;
        while n != 0 {
            name.push(b'A' + (n % 26 - 1) as u8);
            n /= 26;
        }
        name.reverse();
        unsafe { String::from_utf8_unchecked(name) }
    }

    pub fn get_row_name(&self, y: usize) -> String {
        (y + self.origin.1).to_string()
    }

    pub fn get_cell_name(&self, x: usize, y: usize) -> String {
        self.get_column_name(x) + self.get_row_name(y).as_str()
    }

    pub fn merged_cells(&self) -> &[((usize, usize), (usize, usize))] {
        &self.merged_cells
    }

    pub fn get_merged_cell_size(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        let pos = (x, y);
        if let Ok(i) = self.merged_cells.binary_search_by(|t| t.0.cmp(&pos)) {
            Some(unsafe { self.merged_cells.get_unchecked(i).1 })
        } else {
            None
        }
    }

    pub fn fmt_row_name<W: fmt::Write>(&self, y: usize, f: &mut W) -> fmt::Result {
         f.write_fmt(format_args!("{}", y + self.origin.1))
    }

    pub fn fmt_column_name<W: fmt::Write>(&self, x: usize, f: &mut W) -> fmt::Result {
        fn func<W: fmt::Write>(n: usize, f: &mut W) -> fmt::Result {
            if n != 0 {
                func(n / 26, f)?;
                unsafe { f.write_char(std::char::from_u32_unchecked(b'A' as u32 + (n % 26 - 1) as u32)) }
            } else {
                Ok(())
            }
        }
        let n = x + self.origin.0 - 1;
        func(n / 26, f)?;
        unsafe { f.write_char(std::char::from_u32_unchecked(b'A' as u32 + (n % 26) as u32)) }
    }

    pub fn get_range_name(&self, x: usize, y: usize, width: usize, height: usize) -> RangeName {
        RangeName(self, x, y, width, height)
    }
}