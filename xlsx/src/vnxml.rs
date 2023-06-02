
use xml;
use std::io::Read;
use std::collections::VecDeque;
use std::str::FromStr;

pub enum XmlNodeRef<'a> {
    Text(&'a XmlText),
    Element(&'a XmlElement),
}

impl<'a> XmlNodeRef<'a> {
    pub fn to_text(self) -> Option<&'a XmlText> {
        match self {
            XmlNodeRef::Text(t) => Some(t),
            _ => None,
        }
    }
    pub fn to_element(self) -> Option<&'a XmlElement> {
        match self {
            XmlNodeRef::Element(t) => Some(t),
            _ => None,
        }
    }
}

impl<'a> XmlNodeMut<'a> {
    pub fn to_text(self) -> Option<&'a mut XmlText> {
        match self {
            XmlNodeMut::Text(t) => Some(t),
            _ => None,
        }
    }
    pub fn to_element(self) -> Option<&'a mut XmlElement> {
        match self {
            XmlNodeMut::Element(t) => Some(t),
            _ => None,
        }
    }
}

pub enum XmlNodeMut<'a> {
    Text(&'a mut XmlText),
    Element(&'a mut XmlElement),
}

pub trait XmlNode {
    fn downcast_ref(&self) -> XmlNodeRef;
    fn downcast_mut(&mut self) -> XmlNodeMut;
}


pub struct XmlText {
    pub content: String,
}

impl XmlNode for XmlText {
    fn downcast_ref(&self) -> XmlNodeRef {
        XmlNodeRef::Text(self)
    }

    fn downcast_mut(&mut self) -> XmlNodeMut {
        XmlNodeMut::Text(self)
    }
}

pub struct XmlElement {
    pub name: xml::name::OwnedName,
    pub namespace: xml::namespace::Namespace,
    pub attributes: Vec<xml::attribute::OwnedAttribute>,
    pub children: Vec<Box<dyn XmlNode>>,
}

impl XmlElement {
    pub fn parse_attribute<F: Fn(&xml::name::OwnedName) -> bool, T: FromStr>(&self, f: F) -> Option<T> {
        if let Some(t) = self.attributes.iter()
            .find(|&t| f(&t.name)) {

            t.value.parse().ok()
        } else {
            None
        }
    }

    pub fn get_attribute<F: Fn(&xml::name::OwnedName) -> bool>(&self, f: F) -> Option<&String> {
        self.attributes.iter()
            .find(|&t| f(&t.name))
            .map(|t| &t.value)
    }
}

impl XmlNode for XmlElement {
    fn downcast_ref(&self) -> XmlNodeRef {
        XmlNodeRef::Element(self)
    }

    fn downcast_mut(&mut self) -> XmlNodeMut {
        XmlNodeMut::Element(self)
    }
}

pub fn read_from<R: Read>(s: R) -> xml::reader::Result<Option<Box<XmlElement>>> {
    let reader = xml::reader::ParserConfig::new()
        .whitespace_to_characters(true)
        .cdata_to_characters(true)
        .create_reader(s);

    let mut stack = VecDeque::new();

    use xml::reader::XmlEvent;
    for ev in reader {
        match ev? {
            XmlEvent::StartElement {
                name, attributes, namespace
            } => {
                stack.push_back(Box::new(XmlElement {
                    name, attributes, namespace,
                    children: Vec::new(),
                }));
            }
            XmlEvent::EndElement { .. } => {
                let e = stack.pop_back().unwrap();
                if let Some(parent) = stack.back_mut() {
                    parent.children.push(e);
                } else {
                    return Ok(Some(e));
                }
            }
            XmlEvent::Characters(content) => {
                if let Some(parent) = stack.back_mut() {
                    parent.children.push(Box::new(XmlText { content }));
                }
            }
            _ => {}
        }
    }

    Ok(None)
}