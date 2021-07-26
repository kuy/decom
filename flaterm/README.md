# flaterm

Terminal UI toolkit with Minimal design.

## Usage

```rust
let layout = layout! {
    <Block title="flaterm-example">
        <Block title="tabs" height=3 />
        <Block title="content" />
    </Block>
};
let area = f.size();
flaterm::render(&layout, f, area);
```

## Features

- [JSX](https://reactjs.org/docs/introducing-jsx.html)-like declarative UI construction
- Layout by [flexbox](https://developer.mozilla.org/en-US/docs/Learn/CSS/CSS_layout/Flexbox)
- Widgets
- Themes
- [crossterm](https://github.com/crossterm-rs/crossterm) backend

## Themes

- `tui` (default)
- `flat` (under development)

## License

MIT

## Author

Yuki Kodama / [@kuy](https://twitter.com/kuy)

## Acknowledgements

- [yew-macro](https://github.com/yewstack/yew/tree/master/packages/yew-macro)
