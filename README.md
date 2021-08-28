# Gallery

[![Build Status](https://api.travis-ci.com/Christoph-D/gallery.svg?branch=main)](https://app.travis-ci.com/github/Christoph-D/gallery)

A static site generator for photo galleries.

## Example

https://christoph-d.github.io/gallery/

## Usage

```shell
$ cargo run -- --page_title='My title' \
  --input=some/path \
  --output=some/path/build \
  --footer='All rights reserved. Contact: <a href="mailto:photos@example.com">photos@example.com</a>'
```

Add `--dry_run` to see which files it would write.

## Input directory structure

You can see the example's input directory structure on
https://github.com/Christoph-D/gallery/tree/site/source. The basic structure is:

* All images need to be in dated directories, no nested directories.
* Directory names and image names can be arbitrary and will be used as titles.