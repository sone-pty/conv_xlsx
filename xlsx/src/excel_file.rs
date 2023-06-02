
use crate::excel_table::ExcelTable;
use crate::vnxml;

use zip::ZipArchive;

use std::path::Path;
use std::io::{Read, Seek};
use std::fs::File;
use std::rc::Rc;

pub struct ExcelFile<R: Read + Seek> {
    zip: ZipArchive<R>,
    strings: Option<Vec<Rc<String>>>,
}

impl<R: Read + Seek> ExcelFile<R> {
    pub fn load_from(s: R) -> Result<ExcelFile<R>, String> {
        Ok(ExcelFile {
            zip: ZipArchive::new(s).map_err(|e| e.to_string())?,
            strings: None,
        })
    }

    pub fn parse_workbook(&mut self) -> Result<Box<[(String, u32)]>, String> {
        let root =
        vnxml::read_from(
            self.zip.by_name("xl/workbook.xml").map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?.ok_or_else(|| "parse workbook error".to_owned())?;

        let sheets = root.children.iter()
            .filter_map(|t| t.downcast_ref().to_element())
            .find(|t| t.name.local_name == "sheets").ok_or_else(|| "parse workbook error".to_owned())?;

        let mut ret = Vec::new();
        for sheet in sheets.children.iter()
            .filter_map(|t| t.downcast_ref().to_element())
            .filter(|t| t.name.local_name == "sheet") {
            if let Some(attr_name) = sheet.get_attribute(
                |t| t.prefix_ref() == None && t.local_name == "name") {
                if let Some(attr_id) = sheet.get_attribute(
                    |t| t.prefix_ref()
                        .map_or(false, |t| t == "r") && t.local_name == "id") {
                    if attr_id.starts_with("rId") {
                        if let Ok(sheet_id) = unsafe { attr_id.get_unchecked(3..).parse::<u32>() } {
                            ret.push((attr_name.to_owned(), sheet_id));
                        }
                    }
                }
            }
        }

        Ok(ret.into_boxed_slice())
    }

    pub fn parse_sheet(&mut self, sheet_id: u32) -> Result<ExcelTable, String> {
        let strings;
        if let Some(t) = self.strings.take() {
            strings = t;
        } else {
            strings = self.parse_shared_strings()?;
        }

        let ret = self._parse_sheet(&strings, sheet_id);

        self.strings = Some(strings);

        ret
    }

    fn parse_shared_strings(&mut self) -> Result<Vec<Rc<String>>, String> {
        let root =
        vnxml::read_from(
            self.zip.by_name("xl/sharedStrings.xml").map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?.ok_or_else(|| "parse shared strings error".to_owned())?;

        let mut strings =
            if let Some(t) = root
                .parse_attribute(|t| t.local_name == "uniqueCount") {
                Vec::with_capacity(t)
            } else {
                Vec::new()
            };

        for si in root.children.iter()
            .filter_map(|t| t.downcast_ref().to_element())
            .filter(|t|t.name.local_name == "si") {
            if let Some(t) = si.children.iter()
                .filter_map(|t|t.downcast_ref().to_element())
                .find(|&t|t.name.local_name == "t") {
                if let Some(text) = t.children.iter()
                    .find_map(|t|t.downcast_ref().to_text()) {
                    strings.push(Rc::new(text.content.clone()));
                } else {
                    strings.push(Rc::default());
                }
            } else {
                let mut content = String::new();
                si.children.iter()
                    .filter_map(|t| t.downcast_ref().to_element())
                    .filter(|&t| t.name.local_name == "r")
                    .flat_map(|t| t.children.iter())
                    .filter_map(|t| t.downcast_ref().to_element())
                    .filter(|&t| t.name.local_name == "t")
                    .for_each(|t| {
                        if let Some(text) = t.children.iter()
                            .find_map(|t| t.downcast_ref().to_text()) {
                            content.push_str(text.content.as_str());
                        }
                    });
                strings.push(Rc::new(content));
            }
        }

        Ok(strings)
    }

    fn _parse_sheet(&mut self, strings: &Vec<Rc<String>>, sheet_id: u32) -> Result<ExcelTable, String> {

        let root =
            vnxml::read_from(
                self.zip.by_name(format!("xl/worksheets/sheet{}.xml", sheet_id).as_str())
                    .map_err(|_| format!("sheet{} does not exist", sheet_id))?).map_err(|e| e.to_string())?;


        let parse = || {
            let root = root?;
            let (origin, size) = parse_dimension(
                root.children.iter().filter_map(|t| t.downcast_ref().to_element())
                    .find(|&t| t.name.local_name == "dimension")?.get_attribute(|t| t.local_name == "ref")?)?;

            let count = size.0 * size.1;
            let mut cells = Vec::with_capacity(count);
            cells.extend(std::iter::repeat(None).take(count));

            let sheet_data = root.children.iter().filter_map(|t| t.downcast_ref().to_element())
                .find(|&t| t.name.local_name == "sheetData")?;

            for row in sheet_data.children.iter().filter_map(|t| t.downcast_ref().to_element())
                .filter(|&t| t.name.local_name == "row") {
                for cell in row.children.iter().filter_map(|t| t.downcast_ref().to_element())
                    .filter(|&t| t.name.local_name == "c") {
                    let (mut x, mut y) = parse_position(cell.get_attribute(|t| t.local_name == "r")?)?;
                    x = x.wrapping_sub(origin.0);
                    y = y.wrapping_sub(origin.1);
                    if x > size.0 || y > size.1 {
                        continue;
                    }

                    let val = cell.children.iter().filter_map(|t| t.downcast_ref().to_element())
                        .find(|&t| t.name.local_name == "v");

                    let val = if let Some(t) = val {
                        t.children.iter().filter_map(|t| t.downcast_ref().to_text()).next()
                    } else {
                        continue;
                    };

                    if let Some(text) = val {
                        unsafe {
                            *cells.get_unchecked_mut(y * size.0 + x) =
                                if let Some(idx) = cell.get_attribute(|t| t.local_name == "t")
                                    .and_then(|t| if t == "s" { Some(()) } else { None })
                                    .and_then(|_| text.content.parse::<usize>().ok())
                                    .filter(|&t| t < strings.len()) {
                                    Some(strings.get_unchecked(idx).clone())
                                } else {
                                    Some(Rc::new(text.content.clone()))
                                };
                        }
                    }
                }
            }

            let mut merged_cells;

            if let Some(element) = root.children.iter().filter_map(|t| t.downcast_ref().to_element())
            .find(|&t| t.name.local_name == "mergeCells") {
                if let Some(count) = element.parse_attribute(|t| t.local_name == "count") {
                    merged_cells = Vec::with_capacity(count);
                } else {
                    merged_cells = Vec::new();
                }

                for child in element.children.iter().filter_map(|t| t.downcast_ref().to_element())
                .filter(|&t| t.name.local_name == "mergeCell") {
                    if let Some(text) = child.get_attribute(|t| t.local_name == "ref") {
                        if let Some(t) = parse_dimension(text) {
                            merged_cells.push((((t.0).0 - origin.0, (t.0).1 - origin.1), t.1));
                        }
                    }
                }

                merged_cells.sort_unstable_by(|a, b| a.0.cmp(&b.0));

            } else {
                merged_cells = Vec::new();
            }

            Some(ExcelTable::new(origin, size, cells, merged_cells))
        };

        parse().ok_or_else(|| "parse sheet error".to_owned())
    }
}

impl ExcelFile<File> {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<ExcelFile<File>, String> {
        Self::load_from(File::open(path).map_err(|e| e.to_string())?)
    }
}

fn parse_dimension(dimension: &str) -> Option<((usize, usize), (usize, usize))> {
    if let Some(idx) =
        dimension.bytes().enumerate().find(|t| t.1 == b':')
            .map(|t| t.0) {

        let d1 = unsafe { dimension.get_unchecked(..idx) };
        let d2 = unsafe { dimension.get_unchecked(idx+1..) };

        parse_position(d1)
            .and_then(|origin| parse_position(d2)
                .map(|(last_x, last_y)|
                    (origin, (last_x - origin.0 + 1, last_y - origin.1 + 1))
                ))
    } else {
        parse_position(dimension).map(|t| (t, (1, 1)))
    }
}

fn parse_position(position: &str) -> Option<(usize, usize)> {
    let mut x = 0usize;
    let mut y = 0usize;
    let mut iter = position.bytes().peekable();
    while let Some(&b) = iter.peek() {
        match b {
            b'A'..=b'Z' => x = x * 26 + 1 + (b - b'A') as usize,
            b'1'..=b'9' => {
                y = (b - b'0') as usize;
                iter.next();
                break;
            }
            _ => return None,
        }
        iter.next();
    }

    while let Some(&b) = iter.peek() {
        match b {
            b'0'..=b'9' => y = y * 10 + (b - b'0') as usize,
            _ => return None,
        }
        iter.next();
    }

    Some((x, y))
}