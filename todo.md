# TODO

- investigate whether the `color` module in `charts` could be made to use what `plotters` expects
  right away (instead of using custom types)
- wait for plotters to merge https://github.com/38/plotters/pull/156 and update `chart` and `client`
  deps

## Server-client File Exchanges

Server currently loads init/diffs as text and sends the text, would be better to send the actual
diff: server-side parsing.

## UI

### Charts

- collapse/expand
- move up/down
- filtering by list of constraints (last chart line catches everything else)
    - location, label: includes/excludes
    - size, lifetime: >=/<=

### List of Allocations

- where
    - bottom of view or
    - new tab
- allow unfollowing 
    - remove alloc completely from everything
    - add to ignore list (to avoid problem if/when it dies)
