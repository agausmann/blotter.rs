# Blotter.rs

Blotter file format parser and Logic World sandbox editor in Rust.

## Current version 

This is the version that is supported for loading/saving and high-level editing:

- **Blotter v6** (Logic World 0.91 beta)
    - Not formalized yet, mostly same as v5, but component positions are
      integers and custom data formats changed for some components

## Legacy versions

There are no high-level APIs for these versions, but converting the savefile to
newer versions is supported:

- [**Blotter v5**][v5] (Logic World 0.90)

[v5]: https://gist.github.com/JimmyCushnie/bebea37a21acbb6e669589967f9512a2
