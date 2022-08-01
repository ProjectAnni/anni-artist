# anni-artist

Parser for [Anni Artists Format][artist-format].

[artist-format]: https://book.anni.rs/01.audio-convention/04.zzzz.02.artist.html

## Definition

Anni artists format has defined a string format for describing multiple/nested artists.

Multiple artists can be separated by ideographic comma(`、`). For example:

```text
雨宮天、麻倉もも、夏川椎菜
```

Artists structure may be nested to describe the `included` relation between group and artists. For example:

```text
TrySail（雨宮天、麻倉もも、夏川椎菜）
```

If commas are included in artist name, there're two ways to escape them:

- double ideographic commas: `25時、、ナイトコードで。`
- escape with character `\`: `25時\、ナイトコードで。`
