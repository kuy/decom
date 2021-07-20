# decom

Decompose [`docker-compose`](https://docs.docker.com/compose/reference/) logs and organize them.

## Usage

### WIP: Install

```bash
$ cargo install decom
```

## Development

### Architecture

Use multi-process model.

```
[main (thread)] <-> [tui]
[log collector (thread)] <-> [docker logs (process)]
```
