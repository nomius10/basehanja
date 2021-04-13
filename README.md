# Overview

This is a toy-project through which to experiment rust+WASM.

Basically, this is a base64-like encoding, but instead of using the usual latin letters and numbers, it uses various charsets such as chinese characters or hangul syllables. A similar project is [BaseHangul](https://github.com/basehangul/basehangul-javascript) (though this implementation is incompatible with that spec).

One might assume that this implementation is fast, since rust is a low-level compiled language. My intent was not to make it fast, but rather modular & expandable. Performance might be atrocious.

A webpage exposing this module should be available [at this link](https://detether.net/toolbox/basehanja).

# Implementation notes

For encodings that result in chars containing more bits than the input I had to implement an alternative padding scheme which I've named `DropPad`. Consider the example of the kanji encoding:

```
"aaa" text    : 011000010110000101100001
bin boundry   : ________--------________
kanji boundry : ________________----------------
```

The encoding-decoding process would produce an erroneous extra null byte:

```
original text  : 01100001 01100001 01100001             "aaa"

u8->u16 repack : 0110000101100001 0110000100000000      "𠖫𠕊"

u16->u8 repack : 01100001 01100001 01100001 00000000    "aaa\0"
```

The conventional padding approach (which I've named `BlockPad`) is practically useless. To workaround that problem, for encodings longer than 8 bits, the padding character will signify how many characters to drop in the decoding process. This also ensures that the encoded strings are concatenable:

```
"aaa" --[kanji_enc]--> "𠖫𠕊" --[compute_pad]--> "𠖫𠕊々" --[kanji_dec]--> "aaa\0" --[drop_pad]--> "aaa"
```

# Efficiency

Here's a table providing ***approximate*** efficiencies of the encodings (sample size 1), both at the \[**b**\]inary and \[**c**\]haracter level. ***None*** of the encodings beat `base64` at the byte-level efficiency. Some do however beat `base64` at the character-level efficiency. Therefore, unless your purpose is to encode as much information as possible in a tweet (currently 140 char limit), this whole project is useless.

- `%incr` values are from the original to the encoded text.
- `50%` and `-50%` means it increased by half, and halved, respectively
- `100%`, `200%` means it doubled, tripled, etc.
- `Δ` values are relative to `base64` `%incr` values

|encoding|chars_kind|%c_incr|%b_incr|Δc|Δb|example_text|char_change|byte_change|
|--------|----------|------:|------:|-:|-:|:-----------|:---------:|:---------:|
|binary|ASCII|700.00|700.00|666.67|666.67|hello_world_012345|18 -> 144|18 -> 144|
|binary|2 byte|2300.00|700.00|2000.00|666.67|俺の日本語は下手|8 -> 192|24 -> 192|
|binary|3 byte|3100.00|700.00|2628.57|657.14|𠜎𠜱𠝹𠱓𠱸𠲖𠳏|7 -> 224|28 -> 224|
|binary|emoji|3100.00|700.00|2600.00|650.00|🐵🙈🙉🙊|4 -> 128|16 -> 128|
|hex|ASCII|100.00|100.00|66.67|66.67|hello_world_012345|18 -> 36|18 -> 36|
|hex|2 byte|500.00|100.00|200.00|66.67|俺の日本語は下手|8 -> 48|24 -> 48|
|hex|3 byte|700.00|100.00|228.57|57.14|𠜎𠜱𠝹𠱓𠱸𠲖𠳏|7 -> 56|28 -> 56|
|hex|emoji|700.00|100.00|200.00|50.00|🐵🙈🙉🙊|4 -> 32|16 -> 32|
|base64|ASCII|33.33|33.33|0.00|0.00|hello_world_012345|18 -> 24|18 -> 24|
|base64|2 byte|300.00|33.33|0.00|0.00|俺の日本語は下手|8 -> 32|24 -> 32|
|base64|3 byte|471.43|42.86|0.00|0.00|𠜎𠜱𠝹𠱓𠱸𠲖𠳏|7 -> 40|28 -> 40|
|base64|emoji|500.00|50.00|0.00|0.00|🐵🙈🙉🙊|4 -> 24|16 -> 24|
|hiragana|ASCII|33.33|300.00|0.00|266.67|hello_world_012345|18 -> 24|18 -> 72|
|hiragana|2 byte|300.00|300.00|0.00|266.67|俺の日本語は下手|8 -> 32|24 -> 96|
|hiragana|3 byte|471.43|328.57|0.00|285.71|𠜎𠜱𠝹𠱓𠱸𠲖𠳏|7 -> 40|28 -> 120|
|hiragana|emoji|500.00|350.00|0.00|300.00|🐵🙈🙉🙊|4 -> 24|16 -> 72|
|katakana|ASCII|33.33|300.00|0.00|266.67|hello_world_012345|18 -> 24|18 -> 72|
|katakana|2 byte|300.00|300.00|0.00|266.67|俺の日本語は下手|8 -> 32|24 -> 96|
|katakana|3 byte|471.43|328.57|0.00|285.71|𠜎𠜱𠝹𠱓𠱸𠲖𠳏|7 -> 40|28 -> 120|
|katakana|emoji|500.00|350.00|0.00|300.00|🐵🙈🙉🙊|4 -> 24|16 -> 72|
|hangul|ASCII|-22.22|133.33|-55.56|100.00|hello_world_012345|18 -> 14|18 -> 42|
|hangul|2 byte|100.00|100.00|-200.00|66.67|俺の日本語は下手|8 -> 16|24 -> 48|
|hangul|3 byte|185.71|114.29|-285.71|71.43|𠜎𠜱𠝹𠱓𠱸𠲖𠳏|7 -> 20|28 -> 60|
|hangul|emoji|175.00|106.25|-325.00|56.25|🐵🙈🙉🙊|4 -> 11|16 -> 33|
|kanji|ASCII|-50.00|83.33|-83.33|50.00|hello_world_012345|18 -> 9|18 -> 33|
|kanji|2 byte|50.00|100.00|-250.00|66.67|俺の日本語は下手|8 -> 12|24 -> 48|
|kanji|3 byte|100.00|100.00|-371.43|57.14|𠜎𠜱𠝹𠱓𠱸𠲖𠳏|7 -> 14|28 -> 56|
|kanji|emoji|100.00|100.00|-400.00|50.00|🐵🙈🙉🙊|4 -> 8|16 -> 32|

# Wasm

Compiles to wasm via wasm-pack. I just run:

```bash
wasm-pack build --target web
cp pkg/{basehanja.js,basehanja_bg.wasm} ~/etc/etc/etc/
```