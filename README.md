# Commenter

Small tool for bulk commenting on GitHub pull requests, useful for triggering multiple github actions.

## Usage

1. Installation

```bash
cargo install commentor
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

Example config file:
```md
editor: subl
github_token: abc
pr_url: www.github.com
comments:
/run-some-build
# /but-not-this-one-yet
```

4. Post comments

```bash
commentor run
```

5. Help

```bash
commentor help
```
