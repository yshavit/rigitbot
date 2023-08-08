# Rigitbot

Description TODO.

# Developing

## Setup

Please run:

```
git config core.hooksPath githooks
```

## REPL

See: https://www.cargo-lambda.info/guide/getting-started.html. tl;dr:

- In one terminal tab, run:
  ```
  cargo lambda watch
  ```
- Invoking it will be something like:
  ```
  cargo lambda invoke basic-lambda --data-file <something>.json
  ```
  ... but I'm still figuring that out.
