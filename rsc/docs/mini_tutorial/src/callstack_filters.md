# Callstack Filters

Callstack filters are filters operating over allocation properties that are sequences of strings
(potentially with some other data). Currently, this means **allocation callstacks**, where the
strings are file names with line/column information.

## String Filters

A string filter can have three shapes: an actual *string value*, a *regex*, or a *match anything* /
*wildcard* filter represented by the string `"..."`. This wildcard filter is discussed in [its own
section] below.


A string value is simply given as a value. To match precisely the string `"file_name"`, one only
needs to write `file_name`. So, a filter that matches precisely the list of strings `[
"file_name_1", "file_name_2" ]` will be written

|     |     |     |
|:---:|:---:|:---:|
| string list | contains | `[` `file_name_1` `file_name_2` `]` |

A *regex* on the other hand has to be written between `#"` and `"#`. If we want the same filter as
above, but want to relax the first string description to be `file_name_<i>` where `<i>` is a single
digit, we write the filter as

|     |     |     |
|:---:|:---:|:---:|
| string list | contains | `[` `#"file_name_[0-9]"#` `file_name_2` `]` |

## The Wildcard Filter

The wildcard filter, written `...`, **lazily** (in general, see below) matches a repetition of any
string-like element of the list. To break this definition down, let us separate two cases: the first
one is when `...` is not followed by another string-like filter, and second one is when it is
followed by another filter.

In the first case, `...` simply matches everything. Consider for instance the filter

|     |     |     |
|:---:|:---:|:---:|
| string list | contain | `[` `#"file_name_[0-9]"#` `...` `]` |

This filter matches any list of strings that starts with a string accepted by the first regex
filter. The following lists of strings are all accepted by the filter above.

- `[` `file_name_0` `]`
- `[` `file_name_7` `anything` `at` `all` `]`
- `[` `file_name_3` `file_name_7` `]`

Now, there is one case when `...` is not actually lazy: when the `n` string-filters *after* it are
not `...`. In this case, all elements of the list but the `n` last ones will be skipped, leaving them for the `n` last string filters.

For this reason

|     |     |     |
|:---:|:---:|:---:|
| string list | contain | `[` `...` `#"file_name_[0-9]"#` `]` |

does work as expected. For example, on the string list

```
[ "some_file_name" "file_name_7" "another_file_name" "file_name_0" ]
```

a lazy behavior would not match. First, `...` would match anything up to and excluding a string
recognized by `#"file_name_[0-9]"#`. So `...` would match `some_file_name`, but that's it since
`file_name_7` is a match for `#"file_name_[0-9]"#`. Hence the filter would reject this list of
strings, because there should be nothing left after the match for `#"file_name_[0-9]"#`. But there
are still `another_file_name` and `file_name_0` left.

Instead, the filter works as expected. `...` discards all elements but the last one `file_name_0`,
which is accepted by `#"file_name_[0-9]"#`.

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

[its own section]: #the-wildcard-filter (The Wildcard Filter)