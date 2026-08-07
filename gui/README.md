# Local tournament GUI

Run the GUI from anywhere with:

```sh
./gui/run-local.sh
```

The launcher enters the project's Nix development shell when `trunk` is not
already available, serves the application at `http://127.0.0.1:8080`, and opens
that address in a new browser tab. Stop the local server with `Ctrl-C`.

Tournament data is currently kept in browser memory. Reloading or closing the
tab starts a new tournament.
