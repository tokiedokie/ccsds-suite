# XTCE Editor

XTCE Editor is a native desktop application for creating and editing
[CCSDS XML Telemetric and Command Exchange (XTCE)](https://www.omg.org/xtce/)
documents. It uses the XTCE 1.3 data model provided by the workspace's `xtce`
crate and is built with Rust, GPUI, and GPUI Component.

## Features

- Create, open, edit, and save XTCE XML documents.
- Browse nested `SpaceSystem` elements in a searchable tree.
- Edit telemetry metadata, command metadata, and services with structured
  forms.
- Add and remove elements from XTCE sets.
- Prevent deletion when another editable element still references the target.
- Open an XTCE document directly from the command line.
- Import documents using the XTCE 1.2 namespace through a compatibility
  conversion.
- Export an application log from the Help menu.

## Running

Run the editor from the workspace root:

```sh
cargo run --release --package xtce-editor
```

Pass an XTCE XML file to open it at startup:

```sh
cargo run --release --package xtce-editor -- path/to/document.xml
```

You can also open a document from **File > Open…** after the application
starts.

## Building

Install the latest stable Rust toolchain and the native development packages
required by GPUI.

On Ubuntu, install the Linux dependencies with:

```sh
sudo apt-get update
sudo apt-get install --yes --no-install-recommends \
  libfontconfig-dev \
  libssl-dev \
  libwayland-dev \
  libx11-xcb-dev \
  libxkbcommon-x11-dev
```

Build the release executable from the workspace root:

```sh
cargo build --locked --release --package xtce-editor
```

The executable is written to `target/release/xtce-editor` on Linux and macOS,
or `target/release/xtce-editor.exe` on Windows.

## Document compatibility

The editor reads and writes XTCE 1.3 XML. When opening a document that uses the
XTCE 1.2 namespace, it retries parsing after replacing that namespace with the
XTCE 1.3 namespace. This is a compatibility conversion, not a complete
schema-to-schema migration.

Saving serializes the in-memory XTCE model into new XML. Formatting, comments,
namespace prefixes, and other source-level details from the original XML are
not preserved.

## Development

Run the editor tests and lint checks from the workspace root:

```sh
cargo test --package xtce-editor
cargo clippy --package xtce-editor --all-targets -- -D warnings
```

## License

XTCE Editor is distributed under the
[GNU General Public License version 3 or later](../../LICENSE).
