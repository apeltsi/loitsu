![The loitsu logo](https://github.com/apeltsi/loitsu/assets/49206921/e9ae0cc1-896c-4120-9582-8ef7bae007b3)

# loitsu

A cross-platform game engine written in Rust. Loitsu is designed to be able to support multiple implementation languages. Currently [Rune](https://rune-rs.github.io/) is the main scripting language, but more languages are planned.

> :warning: Loitsu is very experimental, expect many API changes. 

## Platform support

- Linux (both Wayland and X11)
- Windows
- Web (WebGL)

> Loitsu might work on other platforms, but is only tested on the above. Mobile support is not planned currently.

Loitsu generally defaults to the Vulkan backend provided by [wgpu](https://github.com/gfx-rs/wgpu) on desktop platforms, but is capable of running with DirectX as well.

## Tooling

Loitsu projects are built with the loitsu-cli. Example
- To run your project `loitsu run`
- To build your project `loitsu build`
- To edit your project in the loitsu editor `loitsu edit`
- To clean the asset cache `loitsu clean`

To build for a specific platform you can suffix the `run` and `build` commands with `-t [Platform]`.

- `loitsu run -t web`

To force assets to be regenerated (useful when changing loitsu versions or running into unexpected errors) use the `--force` or `-f` argument.

## Name

"loitsu" is Finnish for spell
