# Stage

This is the Stage audio framework.

There are many like it, but this one is mine.


## License

All code is licensed under the AGPL, v3 or later.

Please contact [sales@flowdsp.io](mailto:sales@flowdsp.io?subject=Stage%20commercial%20license%20purchase) to purchase a commercial license.

### Third Party Licenses

TinyOSC is published under the [ISC](https://opensource.org/licenses/ISC) license.

```
Copyright (c) 2015, Martin Roth <mhroth@gmail.com> (tinyosc)

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
```


## Rebuild Bindings:

    cd esp_idf

    # edit generate.sh and set XTENSA_ROOT and IDF_PATH

    sudo port -f deactivate libiconv
    cargo install bindgen
    sudo port activate libiconv

    ./generate.sh
