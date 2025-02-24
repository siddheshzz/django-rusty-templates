use std::borrow::Cow;

use html_escape::encode_quoted_attribute_to_string;
use pyo3::prelude::*;

use crate::render::{Context, Resolve, ResolveResult};
use crate::types::Argument;
use crate::{render::Content, types::TemplateString};

trait IntoOwnedContent<'t, 'py> {
    fn into_content(self) -> Option<Content<'t, 'py>>;
}

trait AsBorrowedContent<'a, 't, 'py>
where
    'a: 't,
{
    fn as_content(&'a self) -> Option<Content<'t, 'py>>;
}

impl<'a, 't, 'py> AsBorrowedContent<'a, 't, 'py> for str
where
    'a: 't,
{
    fn as_content(&'a self) -> Option<Content<'t, 'py>> {
        Some(Content::String(Cow::Borrowed(self)))
    }
}

impl<'t, 'py> IntoOwnedContent<'t, 'py> for String {
    fn into_content(self) -> Option<Content<'t, 'py>> {
        Some(Content::String(Cow::Owned(self)))
    }
}

#[derive(Debug)]
pub enum FilterType {
    Add(AddFilter),
    AddSlashes(AddSlashesFilter),
    Capfirst(CapfirstFilter),
    Default(DefaultFilter),
    Escape(EscapeFilter),
    External(ExternalFilter),
    Lower(LowerFilter),
    Safe(SafeFilter),
}

pub trait ResolveFilter {
    fn resolve<'t, 'py>(
        &self,
        variable: Option<Content<'t, 'py>>,
        py: Python<'py>,
        template: TemplateString<'t>,
        context: &mut Context,
    ) -> ResolveResult<'t, 'py>;
}

#[derive(Debug)]
pub struct AddSlashesFilter;

impl ResolveFilter for AddSlashesFilter {
    fn resolve<'t, 'py>(
        &self,
        variable: Option<Content<'t, 'py>>,
        _py: Python<'py>,
        _template: TemplateString<'t>,
        context: &mut Context,
    ) -> ResolveResult<'t, 'py> {
        let content = match variable {
            Some(content) => content
                .render(context)?
                .replace(r"\", r"\\")
                .replace("\"", "\\\"")
                .replace("'", r"\'")
                .into_content(),
            None => "".as_content(),
        };
        Ok(content)
    }
}

#[derive(Debug)]
pub struct AddFilter {
    pub argument: Argument,
}

impl AddFilter {
    pub fn new(argument: Argument) -> Self {
        Self { argument }
    }
}

impl ResolveFilter for AddFilter {
    fn resolve<'t, 'py>(
        &self,
        variable: Option<Content<'t, 'py>>,
        py: Python<'py>,
        template: TemplateString<'t>,
        context: &mut Context,
    ) -> ResolveResult<'t, 'py> {
        let variable = match variable {
            Some(left) => left,
            None => return Ok(None),
        };
        let right = self
            .argument
            .resolve(py, template, context)?
            .expect("missing argument in context should already have raised");
        match (variable.to_bigint(), right.to_bigint()) {
            (Some(variable), Some(right)) => Ok(Some(Content::Int(variable + right))),
            _ => {
                let variable = variable.to_py(py)?;
                let right = right.to_py(py)?;
                match variable.add(right) {
                    Ok(sum) => Ok(Some(Content::Py(sum))),
                    Err(_) => Ok(None),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct CapfirstFilter;

impl ResolveFilter for CapfirstFilter {
    fn resolve<'t, 'py>(
        &self,
        variable: Option<Content<'t, 'py>>,
        _py: Python<'py>,
        _template: TemplateString<'t>,
        context: &mut Context,
    ) -> ResolveResult<'t, 'py> {
        let content = match variable {
            Some(content) => {
                let content_string = content.render(context)?.into_owned();
                let mut chars = content_string.chars();
                let first_char = match chars.next() {
                    Some(c) => c.to_uppercase(),
                    None => return Ok("".as_content()),
                };
                let string: String = first_char.chain(chars).collect();
                string.into_content()
            }
            None => "".as_content(),
        };
        Ok(content)
    }
}

#[derive(Debug)]
pub struct DefaultFilter {
    pub argument: Argument,
}

impl DefaultFilter {
    pub fn new(argument: Argument) -> Self {
        Self { argument }
    }
}

impl ResolveFilter for DefaultFilter {
    fn resolve<'t, 'py>(
        &self,
        variable: Option<Content<'t, 'py>>,
        py: Python<'py>,
        template: TemplateString<'t>,
        context: &mut Context,
    ) -> ResolveResult<'t, 'py> {
        let content = match variable {
            Some(left) => Some(left),
            None => self.argument.resolve(py, template, context)?,
        };
        Ok(content)
    }
}

#[derive(Debug)]
pub struct EscapeFilter;

impl ResolveFilter for EscapeFilter {
    fn resolve<'t, 'py>(
        &self,
        variable: Option<Content<'t, 'py>>,
        _py: Python<'py>,
        _template: TemplateString<'t>,
        _context: &mut Context,
    ) -> ResolveResult<'t, 'py> {
        Ok(match variable {
            Some(content) => match content {
                Content::HtmlSafe(content) => Some(Content::HtmlSafe(content)),
                Content::String(content) => {
                    let mut encoded = String::new();
                    encode_quoted_attribute_to_string(&content, &mut encoded);
                    Some(Content::HtmlSafe(Cow::Owned(encoded)))
                }
                Content::Int(n) => Some(Content::HtmlSafe(Cow::Owned(n.to_string()))),
                Content::Float(n) => Some(Content::HtmlSafe(Cow::Owned(n.to_string()))),
                Content::Py(object) => {
                    let content = object.str()?.extract::<String>()?;
                    let mut encoded = String::new();
                    encode_quoted_attribute_to_string(&content, &mut encoded);
                    Some(Content::HtmlSafe(Cow::Owned(encoded)))
                }
            },
            None => Some(Content::HtmlSafe(Cow::Borrowed(""))),
        })
    }
}

#[derive(Debug)]
pub struct ExternalFilter {
    pub filter: Py<PyAny>,
    pub argument: Option<Argument>,
}

impl ExternalFilter {
    pub fn new(filter: Py<PyAny>, argument: Option<Argument>) -> Self {
        Self { filter, argument }
    }
}

impl ResolveFilter for ExternalFilter {
    fn resolve<'t, 'py>(
        &self,
        variable: Option<Content<'t, 'py>>,
        py: Python<'py>,
        template: TemplateString<'t>,
        context: &mut Context,
    ) -> ResolveResult<'t, 'py> {
        let arg = match &self.argument {
            Some(arg) => arg.resolve(py, template, context)?,
            None => None,
        };
        let filter = self.filter.bind(py);
        let value = match arg {
            Some(arg) => filter.call1((variable, arg))?,
            None => filter.call1((variable,))?,
        };
        Ok(Some(Content::Py(value)))
    }
}

#[derive(Debug)]
pub struct LowerFilter;

impl ResolveFilter for LowerFilter {
    fn resolve<'t, 'py>(
        &self,
        variable: Option<Content<'t, 'py>>,
        _py: Python<'py>,
        _template: TemplateString<'t>,
        context: &mut Context,
    ) -> ResolveResult<'t, 'py> {
        let content = match variable {
            Some(content) => Some(
                content
                    .resolve_string(context)?
                    .map_content(|content| Cow::Owned(content.to_lowercase())),
            ),
            None => "".as_content(),
        };
        Ok(content)
    }
}

#[derive(Debug)]
pub struct SafeFilter;

impl ResolveFilter for SafeFilter {
    fn resolve<'t, 'py>(
        &self,
        variable: Option<Content<'t, 'py>>,
        _py: Python<'py>,
        _template: TemplateString<'t>,
        _context: &mut Context,
    ) -> ResolveResult<'t, 'py> {
        let content = match variable {
            Some(content) => match content {
                Content::HtmlSafe(content) => Some(Content::HtmlSafe(content)),
                Content::String(content) => Some(Content::HtmlSafe(content)),
                Content::Int(n) => Some(Content::HtmlSafe(Cow::Owned(n.to_string()))),
                Content::Float(n) => Some(Content::HtmlSafe(Cow::Owned(n.to_string()))),
                Content::Py(object) => {
                    let content = object.str()?.extract::<String>()?;
                    Some(Content::HtmlSafe(Cow::Owned(content)))
                }
            },
            None => Some(Content::HtmlSafe(Cow::Borrowed(""))),
        };
        Ok(content)
    }
}
