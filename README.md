# RDX

> Rhai markDown egui eXtensions (RDX)

> Rust Developer eXperience (RDX)

> Real gooD eXperiment

🦀 Pure Rust (no JavaScript)
🥇 All Platforms (Web, Desktop)
🦕 Extensible - Bring your own eXtensions
🦺 Safe - Run other peoples' FULL STACK code safely in a WebAssembly 

## Why?

Because we need a way to encapsulate the front end Rust into WebAssembly, so the full stack can be run in a trust minimized manner.

## What is RDX?

RDX is a combination of 1) Rhai (for control flow logic) and 2) egui markdown (for User Interface). 

For [example](./examples/counter/src/lib.rs), It looks something like this:

```rhai 
// call the system function `render` on the template with the ctx from scope

// rhai script controls the flow of logic on what to show

if !is_def_var("count") || count == "0" {

    // the render function returns a string of RDX
    // render is provided by the rhai scope by default
    render(ctx, `
        <Vertical>
            <Button on_click=increment()>Increment</Button>
            <Button on_click=decrement()>Decrement</Button>
            <Label>Click to Start counting!</Label>
        </Vertical>
    `)

} else {

    // alternate RDX if count is not 0 
    // the {{count}} is a variable stored in rhai scope
    render(ctx, `
        <Vertical>
            <Button on_click=increment()>Increment</Button>
            <Button on_click=decrement()>Decrement</Button>
            <Label>Count is: {{count}}</Label>
        </Vertical>
    `)

}
```

The `increment()` and `decrement()` functions are provided by WebAssembly exported functions. These functions emit a `count` variable that is stored in the Rhai scope, then displayed back in the gui.

Bundle RDX scripts into WebAssembly then run them as eframe components, natively or in the browser.

eframe template experiment to see if I can parse an RDX format into eframe.

The goal is for this to be the simplest way to get started writing a eGUI app in Rust.

You can compile your app natively or for the web, and share it using Github Pages.

## Getting started

Build a component in either pure Rhai or Rhai + Rust compiled to WASM.

### Learning about egui

```ignore
todo!()
```

### Testing locally

Make sure you have [`just`](https://just.systems/man/en/) installed and are using the latest version of stable rust by running `rustup update`.

`just run`

#### Dependencies

On Linux you may need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

### Web Locally

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page.

We use [Trunk](https://trunkrs.dev/) to build for web target.
1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

> `assets/sw.js` script will try to cache our app, and loads the cached version when it cannot connect to server allowing your app to work offline (like PWA).
> appending `#dev` to `index.html` will skip this caching, allowing us to load the latest builds during development.

### Web Deploy
1. Just run `trunk build --release`.
2. It will generate a `dist` directory as a "static html" website
3. Upload the `dist` directory to any of the numerous free hosting websites including [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).
4. we already provide a workflow that auto-deploys our app to GitHub pages if you enable it.
> To enable Github Pages, you need to go to Repository -> Settings -> Pages -> Source -> set to `gh-pages` branch and `/` (root).
>
> If `gh-pages` is not available in `Source`, just create and push a branch called `gh-pages` and it should be available.
>
> If you renamed the `main` branch to something else (say you re-initialized the repository with `master` as the initial branch), be sure to edit the github workflows `.github/workflows/pages.yml` file to reflect the change
> ```yml
> on:
>   push:
>     branches:
>       - <branch name>
> ```

You can test the template app at <https://emilk.github.io/eframe_template/>.

## Updating egui

As of 2023, egui is in active development with frequent releases with breaking changes. [eframe_template](https://github.com/emilk/eframe_template/) will be updated in lock-step to always use the latest version of egui.

When updating `egui` and `eframe` it is recommended you do so one version at the time, and read about the changes in [the egui changelog](https://github.com/emilk/egui/blob/master/CHANGELOG.md) and [eframe changelog](https://github.com/emilk/egui/blob/master/crates/eframe/CHANGELOG.md).
