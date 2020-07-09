# Current and Future Features


## Labels

Memthol's input dump format allows allocations to be annotated with *labels* or *tags*. These are
lists of *strings* that could typically be used to add a list of file/line information representing
(parts of) the callstack at allocation-site. Information about the type of the allocations could
also be encoded as such a label.

Memthol currently supports filtering on labels using lists of regular expression. However, we think
this feature is not mature enough yet to be featured in this document.

## Lifetimes

Perhaps the most important feature we want to finish implementing is lifetime filters. That is, the
ability to filter base on how long an allocation stayed alive. This is already mostly implemented
and should be released soon.

Also on the topic of lifetime, memthol will eventually allow to plot lifetime information over time,
such as highest lifetime and average lifetime over time. Currently, this feature is only partially
implemented and is not ready for user interaction.

## Saving and Loading Settings, Filters and Layout

If you use memthol you will probably get frustrated very quickly by the obligation to manually
re-specify all you filters and graph layout every time you re-run it. Or even if you just refresh
your browser. As it turns out, memthol handles everything server-side so all your changes from
the default configuration can be fairly easily named and saved on the disk.

You could then simply select a previously saved configuration at any time, and memthol would load it
for you. While not strictly mandatory, this feature is clearly crucial in terms of usability.
