# _OpenRA IDE_ - Architecture

The _OpenRA IDE_ project is made up of multiple codebases, each potentially containing multiple packages, that all come together to give you a better modding experience.

This particular codebase contains the "core" pieces of the _OpenRA IDE_ project:

- `oraml`: lexing, parsing, and AST-building of MiniYaml documents
- `oraws`: SDK-based workspace management
- `orals`: an LSP server for use with LSP clients (such as the Visual Studio Code extension `oraide-vscode`)

Each of these builds on top of the previous ones.

## High-Level Overview

Things to cover:
- `Files`, file database (`FileId`, too)
- Scanning the workspace root for `mods/`
- Reading all mod manifests, which yields more files to read
- Reading all the files listed in the manifests
- What "reading a file" encompasses, in practice
    - lexing: producing a stream of tokens
    - parsing: producing a stream of nodes
    - tree-building: organizing the nodes into a data structure that looks like a tree (an `Arena`)
- Symbol tables (this doesn't exist in the code yet)
    - What / Why / How

## oraml

> "OpenRA Markup Language" or "OpenRA MiniYaml Language" or \<whatever you want it to stand for\>

This package converts a text document into an [AST](https://en.wikipedia.org/wiki/Abstract_syntax_tree) in stages.

### The `lexer` Module

The entrypoint into the `lexer` module is the aptly named `Lexer` type which holds mutable state that trackes lexing progress, results, and errors encountered.

TODO

## oraws

TODO

## orals

TODO