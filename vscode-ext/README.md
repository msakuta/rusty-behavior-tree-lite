# rusty-behavior-tree-lite README

A Visual Studio Code extension for [rusty-behavior-tree-lite](https://github.com/msakuta/rusty-behavior-tree-lite) language syntax highlighting.

The syntax is not finalized yet, so it is not in the markertplace yet.
You can build and install it manually with the procedure in [Build and Install](#build-and-install) section.


## Features

Syntax highlighting for:

* Keywords
* Literals
* Line comments

![screenshot](https://raw.githubusercontent.com/msakuta/rusty-behavior-tree-lite/master/vscode-ext/images/screenshot00.png)

It is very simple, but helps visibility a lot.

## Build and Install

First, install vsce, the VSCode extension manager.

    npm install -g @vscode/vsce

Use it to create a package. A file named `rusty-behavior-tree-lite-0.0.1.vsix` should appear.

    vsce package

Install to your local environment with

    code --install-extension rusty-behavior-tree-lite-0.0.1.vsix

See [the official guide](https://code.visualstudio.com/api/working-with-extensions/publishing-extension) for more information.

