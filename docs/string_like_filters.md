# String-List-Like Filters

String-list-like filters are filters operating over allocation properties that are sequences of
strings (potentially with some other data). This includes **labels** and **allocation callstacks**.
For the former, the labels themselves are the string elements; for the latter, it's the file name.

## String Filters

A string filter can have two shapes: an actual *string value* or a *regex*. A string value is simply
given as a value. To match precisely the string `"my label"`, one only needs to write `my label`.
So, a filter that matches the list of label `[ "label 1", "label 2" ]` will be written

|     |     |     |
|:---:|:---:|:---:|
| labels | contain | `[` `label 1` `label 2` `]` |

A *regex* on the other hand has to be written between `#"` and `"#`. If we want the same filter as
above, but want to relax the first label to be `label *` where `*` is a single digit number, we
write the filter as

|     |     |     |
|:---:|:---:|:---:|
| labels | contain | `[` `#"label [0-9]"#` `label 2` `]` |

## Callstack (Location) Filters

Allocation callstack information is a list of tuples containing:

- the name of the file,
- the line in the file,
- a column range.

Currently, the range information is ignored. The line in the file is not, and one can specify a line
constraint while writing a callstack filter. The *normal* syntax is

```
<string-filter>:<line-filter>
```

Now, a line filter has two basic shapes

- `_`: anything,
- `<number>`: an actual value.

It can also be a range:

- `[<basic-line-filter>, <basic-line-filter>]`: a potentially open range.

### Line Filter Examples

|     |    |
|:---:|:---|
| `_` | matches any line at all |
| `7` | matches line 7 |
| `[50, 102]` | matches any line between `50` and `102` |
| `[50, _]` | matches any line greater than `50` |
| `[_, 102]` | matches any line less than `102` |
| `[_, _]` | same as `_` (matches any line) |

### Callstack Filter Examples

Whitespaces are inserted for readability but are not needed:

|     |    |
|:---:|:---|
| `src/main.ml : _` | matches any line of `src/main.ml` |
| `#".*/main.ml"# : 107` | matches line 107 of any `main.ml` file regardless of its path |
