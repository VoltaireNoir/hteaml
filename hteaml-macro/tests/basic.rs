use hteaml::Render;
use hteaml_macro::hteaml;

#[test]
fn basic_tag() {
    let tag = hteaml!((mytag = ""));
    assert_eq!(tag.render(), Ok("<mytag></mytag>".into()));
}

#[test]
fn tag_attrs() {
    let tag = hteaml!((mytag hello:world = ""));
    assert_eq!(tag.render(), Ok(r#"<mytag hello="world"></mytag>"#.into()));
}

#[test]
fn tag_attr_multi() {
    let tag = hteaml!((mytag hello:world key:value = ""));
    assert_eq!(
        tag.render(),
        Ok(r#"<mytag hello="world" key="value"></mytag>"#.into())
    );
}

#[test]
fn tag_content() {
    let tag = hteaml!((mytag hello:world = "content"));
    assert_eq!(
        tag.render(),
        Ok(r#"<mytag hello="world">content</mytag>"#.into())
    );
}

#[test]
fn tag_dyn_content() {
    let x = String::from("dynamic");
    let tag = hteaml! {
        (mytag hello:world = { x + " content" })
    };
    assert_eq!(
        tag.render(),
        Ok(r#"<mytag hello="world">dynamic content</mytag>"#.into())
    );
}

#[test]
fn tag_nested() {
    let tag = hteaml!((mytag hello:world (tag2 = "content")));
    assert_eq!(
        tag.render(),
        Ok(r#"<mytag hello="world"><tag2>content</tag2></mytag>"#.into())
    );
}

#[test]
fn self_closing() {
    let tag = hteaml!((mytag));
    assert_eq!(tag.render(), Ok("<mytag>".into()));
}

#[test]
fn self_closing_with_attrs() {
    let tag = hteaml!((mytag hello:world));
    assert_eq!(tag.render(), Ok(r#"<mytag hello="world">"#.into()));
}

#[test]
fn multi_tag() {
    let html = hteaml! {
        ("!DOCTYPE" html)
        (p = "hello")
    };
    assert_eq!(html.render(), Ok("<!DOCTYPE html><p>hello</p>".into()))
}

#[test]
fn html_doc() {
    let doc = hteaml! {
      ("!DOCTYPE" html)
      (head (title = "Html Doc"))
      (body (p = "hello world") (p = "this is hteaml"))
    };
    assert_eq!(
        doc.render(),
        Ok(
            r#"<!DOCTYPE html><head><title>Html Doc</title></head><body><p>hello world</p><p>this is hteaml</p></body>"#
                .into()
        )
    );
}

#[test]
fn hteaml_inside_hteaml() {
    let x = String::from("string");
    let html = hteaml! {
          (tag = { hteaml!((tag2 = {x})) })
    };
    assert_eq!(html.render(), Ok("<tag><tag2>string</tag2></tag>".into()));
}

#[test]
fn top_level_expr() {
    let tag = hteaml!((tag));
    let html = hteaml! {
        {tag}
    };
    assert_eq!(html.render(), Ok("<tag>".into()));
}

#[test]
fn top_level_expr_mixed() {
    let tag = hteaml!((tag));
    let html = hteaml! {
        (regular = "content")
        {tag}
    };
    assert_eq!(html.render(), Ok("<regular>content</regular><tag>".into()));
}

#[test]
fn top_level_expr_multi() {
    let tag = hteaml!((tag));
    let tag2 = hteaml!((tag2));
    let html = hteaml! {
        {tag} {tag2}
    };
    assert_eq!(html.render(), Ok("<tag><tag2>".into()));
}

#[test]
fn tag_content_expr_multi() {
    let html = hteaml!(
        (tag = {"one"} {"two"})
    );
    assert_eq!(html.render(), Ok("<tag>onetwo</tag>".into()));
}
