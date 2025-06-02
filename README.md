# Commenter

Small tool for bulk commenting on GitHub pull requests, useful for triggering multiple github actions.

## Usage

1. Installation

```bash
cargo install --path ./apps/commentor
```

2. Setup

```bash
commentor init <github-token> [--editor_command=<editor>]
```

3. Open config file

```bash
commentor open
```

Specify comments below `comments:` in the config file, one per line. Lines beginning with # are ignored.


4. Post comments

```bash
commentor run
```

5. Help

```bash
commentor help
```
