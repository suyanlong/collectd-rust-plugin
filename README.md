# A Collectd Plugin Written in Rust

Collectd gathers system and application metrics and stores the values in any manner. Since Collectd provides a plugin API, this repo demonstrates how to create a Collectd plugin written in Rust that uses [bindgen](https://github.com/rust-lang-nursery/rust-bindgen) to generate the ffi functions. If you want to write a collectd plugin start with this repo as it defines common functions and provides an ergonomic Rust structure on top of `value_list_t`.

Rust 1.20 or later is needed to build.

This repo is tested on the following:

- Collectd 5.4 (Ubuntu 14.04)
- Collectd 5.5 (Ubuntu 16.04)
- Collectd 5.7 (Ubuntu 17.04)

## Quickstart

Below is a complete plugin that dummy reports [load](https://en.wikipedia.org/wiki/Load_(computing)) values to collectd, as it registers a `READ` hook.

```rust
#[macro_use]
extern crate collectd_plugin;
extern crate failure;
#[macro_use]
extern crate lazy_static;

use collectd_plugin::{Plugin, Value, ValueListBuilder, PluginCapabilities};
use std::sync::Mutex;
use failure::Error;

struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "myplugin"
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::READ
    }

    fn report_values(&mut self) -> Result<(), Error> {
        // Create a list of values to submit to collectd. We'll be sending in a vector representing the
        // "load" type. Short-term load is first (15.0) followed by mid-term and long-term. The number
        // of values that you submit at a time depends on types.db in collectd configurations
        let values = vec![Value::Gauge(15.0), Value::Gauge(10.0), Value::Gauge(12.0)];

        // Submit our values to collectd. A plugin can submit any number of times.
        ValueListBuilder::new(self.name(), "load")
            .values(values)
            .submit()
    }
}

lazy_static! {
    static ref PLUGIN: Mutex<MyPlugin> = Mutex::new(MyPlugin);
}

collectd_plugin!(PLUGIN);
```

Currently a global mutex for our plugin is necessary, as there are collectd configuration hooks where our plugin is provided with configuration values relevant to our plugin. Thus our plugin has to be globally accessible and mutable.

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

Rust's combination of ecosystem, package manager, C ffi, single file, and optimized library made it seem like a natural choice.

## To Build

After cloning this repo, you'll need to ensure that a few dependencies are satisfied. Don't worry these aren't needed on the deployed server.

```bash
# Install collectd library so that rust bindgen works.
apt install collectd-dev

# If you are not on ubuntu 16.10 or later, a recent clang version is required
# wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key|sudo apt-key add -
# apt-get install llvm-3.9-dev libclang-3.9-dev clang-3.9

# Must supply the version of collectd you're building against (see the list
# above for supported versions)
cargo build --features collectd-54

# Copy plugin (and rename it) to plugin directory as Collectd assumes a
# standard naming convention
cp target/debug/libmyplugin.so /usr/lib/collectd/myplugin.so

# Add "LoadPlugin myplugin" to collectd.conf
```

## Plugin Configuration

This plugin demonstrates how to expose configuration values to Collectd (in
this case, it's [load](https://en.wikipedia.org/wiki/Load_(computing))) using
contrived numbers that can be overridden using the standard Collectd config:

```xml
# In this example configuration we provide short and long term load and leave
# Mid to the default value. Yes, this is very much contrived
<Plugin myplugin>
    Short "2"
    Long "5.5"
</Plugin>
```
