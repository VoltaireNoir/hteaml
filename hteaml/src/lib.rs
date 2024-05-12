#![doc = include_str!("../../README.md")]
use std::borrow::Cow;
use std::fmt::{self, Write};

pub use hteaml_macro::hteaml;

/// The trait through which the provided types (i.e. [`Html`], [`Tag`]) render themselves to HTML
///
/// This trait is implemented on every type that represents an HTML element, which means that each type can also be rendered to a String separately.
///
/// If you wish to make your custom type be directly usable within the [`hteaml`] macro or other types, see [`IntoStr`]
pub trait Render {
    /// Render self to HTML
    fn render(&self) -> Result<String, fmt::Error> {
        let mut buf = String::new();
        self.render_to_buf(&mut buf)?;
        Ok(buf)
    }

    /// Render self to HTML by writing to the given `String` buffer
    fn render_to_buf(&self, buf: &mut String) -> fmt::Result;
}

impl Render for Str<'_> {
    fn render_to_buf(&self, buf: &mut String) -> fmt::Result {
        buf.write_str(self)
    }
}

/// The primary trait used in the generic parameters in the types exposed by this crate
///
/// It is used for accepting wide range of String types as well as types that implement `AsRef<str>`
/// This trait is very similar to using the trait bound `T: Into<Cow<str>>` but with an extra blanket implementation.
///
/// Users of this library can implement this trait on their custom types to make them work seamlessly with the [`hteaml`] macro.
/// ## Implementing IntoStr
/// ```
/// use hteaml::{IntoStr, Str};
///
/// struct CustomOwned(String);
///
/// impl<'a> IntoStr<'a> for CustomOwned {
///    fn into_str(self) -> Str<'a> {
///        Str::Owned(self.0)
///    }
/// }
///
/// struct CustomBorrow<'a>(&'a str);
///
/// impl<'a> IntoStr<'a> for CustomBorrow<'a> {
///    fn into_str(self) -> Str<'a> {
///        Str::Borrowed(self.0)
///    }
/// }
/// ```
pub trait IntoStr<'a> {
    /// Convert self to `Str` which is an alias for `Cow<str>`
    fn into_str(self) -> Str<'a>;
}

impl<'a> IntoStr<'a> for String {
    fn into_str(self) -> Str<'a> {
        Cow::Owned(self)
    }
}

impl<'a> IntoStr<'a> for &'a str {
    fn into_str(self) -> Str<'a> {
        Cow::Borrowed(self)
    }
}

impl<'a, T: AsRef<str>> IntoStr<'a> for &'a T {
    fn into_str(self) -> Cow<'a, str> {
        Cow::Borrowed(self.as_ref())
    }
}

/// Type alias for `Cow<str>`
///
/// This type is widely used in the types exposed by the crate to accept both owned
/// and borrowed string types.
///
/// If you wish to make your custom type be directly usable within the [`hteaml`] macro or other types, see [`IntoStr`]
pub type Str<'a> = Cow<'a, str>;

/// Top level representation of HTML markup which can contain a single tag or a comment or a sequence of both
///
/// The [`hteaml`] macro returns this type on every invocation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Html<'a> {
    /// An HTML tag
    Tag(Tag<'a>),
    /// An HTML comment
    Comment(Comment<'a>),
    /// A sequence containing tags and comments or more nested sequences
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

/// Type that represents an HTML comment
///
/// Note: Comments are still not supported in the [`hteaml`] macro
///
/// ## Example
/// ```
/// use hteaml::Render;
/// assert_eq!(hteaml::Comment::new("comment").render(), Ok("<!-- comment -->".into()))
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment<'a>(Str<'a>);

impl<'a> Comment<'a> {
    /// Construct a new comment
    pub fn new<T: IntoStr<'a>>(comment: T) -> Self {
        Self(comment.into_str())
    }
}

impl Render for Comment<'_> {
    fn render_to_buf(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "<!-- {} -->", self.0)
    }
}

/// Represents an HTML tag
///
/// This is the building block for HTML. A tag can be created either directly through the provided builder
/// pattern or using the [`hteaml`] macro
///
/// > Note: calling `.self_closing()` on the tag type will ignore any content (if it was provided).
///
/// ## Example
/// ```
/// use hteaml::{Html, Tag, hteaml};   
/// let tag = Tag::new("div").attr("key","val").content("content");
/// assert_eq!(Html::Tag(tag), hteaml!((div key:val = "content")));
///
/// let tag = Tag::new("br").self_closing();
/// assert_eq!(Html::Tag(tag), hteaml!((br)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Create a new Tag type
    ///
    /// It is preferable to use the [`hteaml`] macro to write HTML tags.
    /// However, the user is free to either use the macro of the [`Tag`] and [`Html`] types directly.
    pub fn new<T: IntoStr<'a>>(name: T) -> Self {
        Self {
            name: name.into_str(),
            attributes: vec![],
            content: vec![],
            self_closing: false,
        }
    }

    /// Append a tag attribute to the tag
    ///
    /// The generic parameters accept any type that implements the trait [`IntoStr`].
    /// The [`IntoStr`] trait is implemented for `&str`, `String and any type that implements `AsRef<str>`
    pub fn attr<A, B>(mut self, key: A, val: B) -> Self
    where
        A: IntoStr<'a>,
        B: IntoStr<'a>,
    {
        self.attributes.push(Attr {
            key: key.into_str(),
            val: val.into_str(),
        });
        self
    }

    /// Append content to the tag
    ///
    /// The `content` parameter accepts any type that implements `Into<Content>`.
    /// All types that implement [`IntoStr`] also implement [`Into<Content>`], which means all string types and others that implement `AsRef<str>`.
    /// Other types that implement `Into<Content>` are the types in the [`Html`] enum variants, and the enum itself.
    pub fn content<C: Into<Content<'a>>>(mut self, content: C) -> Self {
        self.content.push(content.into());
        self
    }

    /// Make the tag self-enclosing (i.e. single tag without an additional closing tag)
    ///
    /// Note: self closing tags do not contain any content, therefore any content appended to the tag will be ignored.
    pub fn self_closing(mut self) -> Self {
        self.self_closing = true;
        self
    }
}

/// Represents an HTML tag attribute
#[derive(Debug, Clone, PartialEq, Eq)]
struct Attr<'a> {
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

/// Represents the content of an HTML tag
///
/// This is a superset of the [`Html`] enum with the addition of the [`Str`] type (which is not a member of the top level Html enum).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content<'a> {
    /// Html content
    Html(Html<'a>),
    /// Plain string
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
