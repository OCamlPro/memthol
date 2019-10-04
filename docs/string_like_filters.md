# String-List-Like Filters

String-list-like filters are filters operating over allocation properties that are sequences of
strings (potentially with some other data). This includes **labels** and **allocation callstacks**.
For the former, the labels themselves are the string elements; for the latter, it's the file name.

## String Filters

A string filter can have three shapes: an actual *string value*, a *regex*, or a *match anything* / *wildcard* filter represented by the string `"..."`. This wildcard filter is discussed in [its own section] below. A string value is simply
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

## The Wildcard Filter

The wildcard filter, written `...`, **lazily** matches a repetition of any string-like element of
the list. To break this definition down, let us separate two cases: the first one is when `...` is
not followed by another string-like filter, and second one is when it is followed by another filter.

In the first case, `...` simply matches everything. Consider for instance the filter

|     |     |     |
|:---:|:---:|:---:|
| labels | contain | `[` `#"label [0-9]"#` `...` `]` |

This filter matches any list of label that starts with a label accepted by the first regex filter.
The following lists of labels are all accepted by the filter above.

- `[` `label 0` `]`
- `[` `label 7` `anything` `at` `all` `]`
- `[` `label 3` `label 7` `]`

Now when `...` is followed by another string-like filter `f`, then the wildcard matches *anything up
to and **excluding** a label matched by `f`*. This is important because wildcards are *lazy*, which
can lead to some counterintuitive results. Consider the following filter.

|     |     |     |
|:---:|:---:|:---:|
| labels | contain | `[` `...` `#"label [0-9]"#` `]` |

It is tempting to read it as *matching any list ending with `label <d>` where `<d>` is a digit*, but
it is wrong. A counterexample is this list of labels:

```
[ "some label" "label 7" "another label" "label 0" ]
```

Now, by definition `...` matches anything up to and excluding a label recognized by `#"label
[0-9]"#`. So here, `...` matches `some label`, but that's it since `label 7` is a match for `#"label
[0-9]"#`. Hence the filter rejects this list of labels, because there should be nothing left after
the match for `#"label [0-9]"#`. But there are still `another label` and `label 0` left.

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