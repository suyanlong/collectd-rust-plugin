# A Collectd Plugin Written in Rust

Collectd gathers system and application metrics and stores the values in any
manner. Since Collectd provides a plugin API, this `collectd_plugin` overlays a
ergonomic, yet extremely low cost abstractions to interface with Collectd.

## Usage

Put this in your `Cargo.toml`:

```toml
[dependencies]
collectd_plugin = "0.3"
```

Or, if you want [Serde](https://github.com/serde-rs/serde) support, include
features like this:

```toml
[dependencies]
collectd_plugin = { version = "0.3", features = ["serde"] }
```

Then put this in your crate root:

```rust
extern crate collectd_plugin;
```

Rust 1.20 or later is needed to build.

This repo is tested on the following:

- Collectd 5.4 (Ubuntu 14.04)
- Collectd 5.5 (Ubuntu 16.04)
- Collectd 5.7 (Ubuntu 17.04)

## Quickstart

Below is a complete plugin that dummy reports [load](https://en.wikipedia.org/wiki/Load_(computing)) values to collectd, as it registers a `READ` hook. For an implementation has the same behavior as Collectd's own load plugin, see [plugins/load](https://github.com/nickbabcock/collectd-rust-plugin/tree/master/plugins/load)

```rust
#[macro_use]
extern crate collectd_plugin;
extern crate failure;

use collectd_plugin::{ConfigItem, Plugin, PluginCapabilities, PluginManager, PluginRegistration,
                      Value, ValueListBuilder};
use failure::Error;

#[derive(Default)]
struct MyPlugin;

// A manager decides the name of the family of plugins and also registers one or more plugins based
// on collectd's configuration files
impl PluginManager for MyPlugin {
    // A plugin needs a unique name to be referenced by collectd
    fn name() -> &'static str {
        "myplugin"
    }

    // Our plugin might have configuration section in collectd.conf, which will be passed here if
    // present. Our contrived plugin doesn't care about configuration so it returns only a single
    // plugin (itself).
    fn plugins(_config: Option<&[ConfigItem]>) -> Result<PluginRegistration, Error> {
        Ok(PluginRegistration::Single(Box::new(MyPlugin)))
    }
}

impl Plugin for MyPlugin {
    // We define that our plugin will only be reporting / submitting values to writers
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::READ
    }

    fn read_values(&mut self) -> Result<(), Error> {
        // Create a list of values to submit to collectd. We'll be sending in a vector representing the
        // "load" type. Short-term load is first (15.0) followed by mid-term and long-term. The number
        // of values that you submit at a time depends on types.db in collectd configurations
        let values = vec![Value::Gauge(15.0), Value::Gauge(10.0), Value::Gauge(12.0)];

        // Submit our values to collectd. A plugin can submit any number of times.
        ValueListBuilder::new(Self::name(), "load")
            .values(&values)
            .submit()
    }
}

// We pass in our plugin manager type
collectd_plugin!(MyPlugin);
```

## Motivation

There are four main ways to extend collectd:

- Write plugin against the C api: `<collectd/core/daemon/plugin.h>`
- Write plugin for [collectd-python](https://collectd.org/documentation/manpages/collectd-python.5.shtml)
- Write plugin for [collectd-java](https://collectd.org/wiki/index.php/Plugin:Java)
- Write a cli for the [exec plugin](https://collectd.org/documentation/manpages/collectd-exec.5.shtml)
- Write a service that [writes to a unix socket](https://collectd.org/wiki/index.php/Plugin:UnixSock)

And my thoughts:

- I'm not confident enough to write C without leaks and there isn't a great package manager for C.
- Python and Java aren't self contained, aren't necessarily deployed on the server, are more heavy weight, and I suspect that maintenance plays second fiddle to the C api.
- The exec plugin is costly as it creates a new process for every collection
- Depending on the circumstances, writing to a unix socket could be good fit, but I enjoy the ease of deployment, and the collectd integration -- there's no need to re-invent logging scheme, configuration, and system init files.

Rust's combination of ecosystem, package manager, C ffi, single file dynamic library, and optimized code made it seem like a natural choice.

## To Build

To ensure a successful build, the following steps are needed:

- When building, you must supply the collectd version you'll be deploying:
    - `cargo build --features collectd-54`
    - `cargo build --features collectd-55`
    - `cargo build --features collectd-57`
- Your project crate type must be `cdylib`
- If you want to use `bindgen` to generate the ffi functions, use the `bindgen` feature (still alongside the desired collectd version). Make sure you have an appropriate version of clang installed and `collectd-dev`
- Collectd expects plugins to not be prefixed with `lib`, so `cp target/debug/libmyplugin.so /usr/lib/collectd/myplugin.so`
- Add `LoadPlugin myplugin` to collectd.conf

## Plugin Configuration

The load plugin in
[plugins/load](https://github.com/nickbabcock/collectd-rust-plugin/tree/master/plugins/load)
demonstrates how to expose configuration values to Collectd.

```xml
# In this example configuration we provide short and long term load and leave
# Mid to the default value. Yes, this is very much contrived
<Plugin loadrust>
    ReportRelative true
</Plugin>
```
