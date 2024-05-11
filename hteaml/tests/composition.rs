use hteaml::{hteaml, Html, Render};

#[test]
fn composition() {
    let message = "Rust macros are very cool!";
    let page = hteaml! {
        {head()}
        {body(message)}
        {footer()}
    };
    assert_eq!(
        page.render(),
        Ok(format!(
            "<head><title>composition</title></head><body>{message}</body><footer>nothing to see here</footer>"
        ))
    )
}

#[test]
fn composition_advanced() {
    let message = "Rust macros are very cool!";
    let page = hteaml! {
        (html = {
            hteaml! {
                {head()}
                {body(message)}
                {footer()}
            }
        })
    };
    assert_eq!(
        page.render(),
        Ok(format!(
            "<html><head><title>composition</title></head><body>{message}</body><footer>nothing to see here</footer></html>"
        ))
    )
}

fn head() -> Html<'static> {
    hteaml! {
        (head (title = "composition"))
    }
}

fn body(text: &str) -> Html {
    hteaml! {
        (body = {text})
    }
}

fn footer<'a>() -> Html<'a> {
    hteaml!((footer = "nothing to see here"))
}
