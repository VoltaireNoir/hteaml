If the regular HTML syntax is not your cup of tea, maybe hteaml is (maybe not).

### What is hteaml?
It's a Rust procedural macro for writing HTML in a more concise way which also allows you to use Rust expressions wherever possible. The macro will expand to Rust code at compile time, which you can call `.render()` on and receive a`String` containg the old-school HTML.

> If you're looking for a crate to use for a serious project then consider using [maud](https://crates.io/crates/maud), which is a proper HTML templating engine.

### What does hteaml look like?
Good question... Well, uh... for better or worse, the syntax will remind you of Lisp dialects. HTML uses `<angled brackets>`, maud, dioxus and some others use `{curly braces}` but hteaml uses `(parenthesis)`.

![](https://imgs.xkcd.com/comics/lisp_cycles.png)

*drumroll* ...
```rust
hteaml! {
  ("!DOCTYPE" html)
  (html
    (head (title = "Hello World"))
    (body = "Yep, lots of parenthesis")
  )
}
```

### Project Status
- Functional but not to be used in production
- Still under development
- Lacks any kind of HTML escaping

### Pending Features/Tasks
- [ ] Docs
- [ ] Guide for using the `hteaml` macro
- [ ] HTML escaping
- [ ] Better error messages
- [ ] Benchmarks
