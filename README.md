# decom

Decompose [`docker-compose`](https://docs.docker.com/compose/reference/) logs and organize them.

## Usage

### WIP: Installation

```bash
$ cargo install decom
```

## Development

### Architecture

Use multi-process model.

```
[main] <-> [tui]
[log collector] <-> [process: docker logs]
```
