use std::borrow::Cow;
use std::fmt::{self, Write};

pub use hteaml_macro::hteaml;

pub trait Render {
    fn render(&self) -> Result<String, fmt::Error> {
        let mut buf = String::new();
        self.render_to_buf(&mut buf)?;
        Ok(buf)
    }

    fn render_to_buf(&self, buf: &mut String) -> fmt::Result;
}

impl Render for Str<'_> {
    fn render_to_buf(&self, buf: &mut String) -> fmt::Result {
        buf.write_str(self)
    }
}

pub trait ToStr<'a> {
    fn to_str(self) -> Str<'a>;
}

impl<'a> ToStr<'a> for String {
    fn to_str(self) -> Str<'a> {
        Cow::Owned(self)
    }
}

impl<'a> ToStr<'a> for &'a str {
    fn to_str(self) -> Str<'a> {
        Cow::Borrowed(self)
    }
}

impl<'a, T: AsRef<str>> ToStr<'a> for &'a T {
    fn to_str(self) -> Cow<'a, str> {
        Cow::Borrowed(self.as_ref())
    }
}

pub type Str<'a> = Cow<'a, str>;

pub enum Html<'a> {
    Tag(Tag<'a>),
    Comment(Comment<'a>),
    Html(Vec<Html<'a>>),
}

impl<'a> From<Tag<'a>> for Html<'a> {
    fn from(value: Tag<'a>) -> Self {
        Self::Tag(value)
    }
}

impl<'a> From<Comment<'a>> for Html<'a> {
    fn from(value: Comment<'a>) -> Self {
        Self::Comment(value)
    }
}

impl<'a> From<Vec<Html<'a>>> for Html<'a> {
    fn from(value: Vec<Html<'a>>) -> Self {
        Self::Html(value)
    }
}

impl Render for Html<'_> {
    fn render_to_buf(&self, buf: &mut String) -> fmt::Result {
        match self {
            Html::Tag(t) => t.render_to_buf(buf),
            Html::Comment(c) => c.render_to_buf(buf),
            Html::Html(h) => h.iter().try_for_each(|e| e.render_to_buf(buf)),
        }
    }
}

pub struct Comment<'a>(Str<'a>);

impl<'a> Comment<'a> {
    pub fn new<T: ToStr<'a>>(comment: T) -> Self {
        Self(comment.to_str())
    }
}

impl Render for Comment<'_> {
    fn render_to_buf(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "<!-- {} -->", self.0)
    }
}

pub struct Tag<'a> {
    name: Str<'a>,
    attributes: Vec<Attr<'a>>,
    content: Vec<Content<'a>>,
    self_closing: bool,
}

impl Render for Tag<'_> {
    fn render_to_buf(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "<{}", self.name)?;
        self.attributes.iter().try_for_each(|attr| -> fmt::Result {
            buf.write_char(' ')?;
            attr.render_to_buf(buf)?;
            Ok(())
        })?;
        if self.self_closing {
            return write!(buf, ">");
        }
        buf.write_char('>')?;
        self.content.iter().try_for_each(|c| c.render_to_buf(buf))?;
        write!(buf, "</{name}>", name = self.name)
    }
}

impl<'a> Tag<'a> {
    pub fn new<T: ToStr<'a>>(name: T) -> Self {
        Self {
            name: name.to_str(),
            attributes: vec![],
            content: vec![],
            self_closing: false,
        }
    }

    pub fn attr<A, B>(mut self, key: A, val: B) -> Self
    where
        A: ToStr<'a>,
        B: ToStr<'a>,
    {
        self.attributes.push(Attr {
            key: key.to_str(),
            val: val.to_str(),
        });
        self
    }

    pub fn content<C: Into<Content<'a>>>(mut self, content: C) -> Self {
        self.content.push(content.into());
        self
    }

    pub fn self_closing(mut self) -> Self {
        self.self_closing = true;
        self
    }
}

pub struct Attr<'a> {
    key: Str<'a>,
    val: Str<'a>,
}

impl Render for Attr<'_> {
    fn render_to_buf(&self, buf: &mut String) -> fmt::Result {
        if self.val.is_empty() {
            return write!(buf, "{key}", key = self.key);
        }
        write!(buf, r#"{key}="{val}""#, key = self.key, val = self.val)
    }
}

pub enum Content<'a> {
    Html(Html<'a>),
    Str(Str<'a>),
}

impl<'a, T> From<T> for Content<'a>
where
    T: Into<Str<'a>>,
{
    fn from(value: T) -> Self {
        Self::Str(value.into())
    }
}

impl<'a> From<Tag<'a>> for Content<'a> {
    fn from(value: Tag<'a>) -> Self {
        Self::Html(Html::Tag(value))
    }
}

impl<'a> From<Comment<'a>> for Content<'a> {
    fn from(value: Comment<'a>) -> Self {
        Self::Html(Html::Comment(value))
    }
}

impl<'a> From<Html<'a>> for Content<'a> {
    fn from(value: Html<'a>) -> Self {
        Self::Html(value)
    }
}

impl Default for Content<'_> {
    fn default() -> Self {
        Self::Str(Str::Borrowed(""))
    }
}

impl Render for Content<'_> {
    fn render_to_buf(&self, buf: &mut String) -> fmt::Result {
        match self {
            Content::Html(h) => h.render_to_buf(buf),
            Content::Str(s) => s.render_to_buf(buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Comment, Html, Render, Tag};

    #[test]
    fn tag() {
        let tag = Tag::new("tag");
        assert_eq!(tag.render(), Ok("<tag></tag>".into()));
    }

    #[test]
    fn self_closing_tag() {
        let tag = Tag::new("close").self_closing();
        assert_eq!(tag.render(), Ok("<close>".into()));
    }

    #[test]
    fn tag_attributes() {
        let tag = Tag::new("tag").attr("key", "val").content("hello");
        assert_eq!(tag.render(), Ok(r#"<tag key="val">hello</tag>"#.into()))
    }

    #[test]
    fn tag_in_tag() {
        let tag = Tag::new("tag");
        let tag2 = Tag::new("tag2").content("hello world");
        let tag = tag.content(tag2);
        assert_eq!(
            tag.render(),
            Ok("<tag><tag2>hello world</tag2></tag>".into())
        );
    }

    #[test]
    fn comment() {
        let c = Comment::new("a comment");
        assert_eq!(c.render(), Ok("<!-- a comment -->".into()));
    }

    #[test]
    fn html_doc() {
        let inner: Html = vec![
            Comment::new("a comment").into(),
            Tag::new("p").content("hello world").into(),
        ]
        .into();
        let doc: Html = vec![
            Tag::new("!DOCTYPE").attr("html", "").self_closing().into(),
            Tag::new("head")
                .content(Tag::new("title").content("Html Doc"))
                .into(),
            Tag::new("body").content(inner).into(),
        ]
        .into();
        assert_eq!(
            doc.render(),
            Ok(r#"<!DOCTYPE html><head><title>Html Doc</title></head><body><!-- a comment --><p>hello world</p></body>"#.into())
        );
    }
}
