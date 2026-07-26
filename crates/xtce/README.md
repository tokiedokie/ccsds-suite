# XTCE

This crate decodes XTCE 1.3 XML into Rust types and encodes the Rust types back into XML.

## Generating the Rust bindings

The Rust type definitions are generated from the following files:

- Input schema: `SpaceSystem.xsd`
- Generator: `examples/generate.rs`
- Generated output: `src/generated.rs`

`src/generated.rs` is generated code. Do not edit it directly. Update the schema or generator and regenerate the bindings instead. Regeneration overwrites the existing file.

Run the following command from the workspace root:

```sh
cargo run -p xtce --example generate
```

From the `crates/xtce` directory, run:

```sh
cargo run --example generate
```

`SpaceSystem.xsd` references the W3C `xml.xsd` schema, so generation requires network access.

The generator disables `NILLABLE_TYPE_SUPPORT`. As a result, the generated `SpaceSystem` type directly represents `SpaceSystemType` instead of `Nillable<SpaceSystemType>`. A `SpaceSystem` with `xsi:nil="true"` is not supported.

## Verifying the generated code

After generation, run the formatter, tests, and Clippy:

```sh
cargo fmt --all
cargo test -p xtce
cargo clippy -p xtce --all-targets -- -D warnings
```
