# decom

Decompose [`docker-compose`](https://docs.docker.com/compose/reference/) logs and organize them.

## Usage

### WIP: Install

```bash
$ cargo install decom
```

## Development

### TODOs

- Command-line options
- Storage to handle massive logs

### Architecture

- decom_core
- decom_ui_cli
  - tui-rs
  - start `flaterm` project using `crossterm`
- decom_ui_tauri
  - tauri
- decom_store_mem
- decom_store

Use multi-process model.

```
[main (thread)] <-> [tui]
[log collector (thread)] <-> [docker logs (process)]
```
