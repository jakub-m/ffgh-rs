# ffgh = fzf + gh

Quick access to GitHub PRs from terminal.

`ffgh-rs` was rewritten from the original [`ffgh` in Go](https://github.com/jakub-m/ffgh) to Rust using Claude.


## Installation

```bash
cargo install --path .
```

## How to use

Run sync in terminal:

```bash
ffgh-bin -v sync

# or

while [ 1 ] ; do  ./bin/ffgh-bin -v sync ; echo "RESTART"; sleep 1; done
```

I run such session as "buried session" in iTerm (hidden in the very background). I couldn't make `crontab` work with
`gh` client.

To run the UI run `./ffgh`.

ffgh **requires** [`fzf`][ref_fzf] and [`gh` CLI][ref_gh].

[ref_fzf]:https://github.com/junegunn/fzf
[ref_gh]:https://cli.github.com/


## Key bindings

* enter - Open all the selected PRs in the browser.
* ctrl-r - Mark as read without opening (does not work with multi-select), mute and unmute.
* ctrl-n - Add a custom note.
* ctrl-a - Annotate with a standard annotation (configurable).
* ctrl-f - Cycle view mode (show all, mute to the top, hide muted).
* ctrl-o - Open without exiting (does not work with multi-select).
* tab - Multi-select.


## xbar

You can use [`ffgh_xbar_plugin.10s.sh`](ffgh_xbar_plugin.10s.sh) as [xbar][ref_xbar] plugin. The muted PRs are ignored
by xbar. If the xbar shows `GH err!` it means that the state is out of sync. Check if synchronization is running.

[ref_xbar]:https://github.com/matryer/xbar


## config

You can define a config with GitHub queries. Run ffzf -h to see the default config.

`attribution_order` - it is used to assign query name to a PR if the same PR appears in the same query. This is useful
if you want to show certain PR as "team PR" if you are part of the team, since the same PR will show up in the query
for the team and the query for you as an assignee.

`display_order` - it is used to define which queries are displayed first.

`mute` - allows marking results of some queries as muted by default (unless explicityly unmuted).

# Troubleshooting

Q: My PRs are not visible

Check if you have `gh` tool configured. See if this shows your PRs:

```
gh search prs --author=@me
```
