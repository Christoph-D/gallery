# Gallery

[![Build Status](https://api.travis-ci.com/Christoph-D/gallery.svg?branch=main)](https://app.travis-ci.com/github/Christoph-D/gallery)

A static site generator for photo galleries.

## Example

https://christoph-d.github.io/gallery/

## Usage

```shell
$ cargo run -- --page_title='My title' --footer='All rights reserved. Contact: <a href="mailto:photos@example.com">photos@example.com</a>' --input=some/path --output=some/other/path
```

Add `--dry_run` to see which files it would write.