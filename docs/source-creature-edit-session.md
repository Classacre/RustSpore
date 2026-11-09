# Experimental source CreatureData editing

This command opens an extracted, hash-checked version 9 or 10 CreatureData
payload and edits its existing part transforms. It retains the source bytes
outside the selected 48-byte transform, supports session-local undo and redo,
and can write a new payload under this checkout's `target` directory. It does
not regenerate the baked GMDL, validate the original editor's attachment rules,
create a complete shareable creation, or integrate the result into gameplay.

After building with `tools\build.bat`, a uniquely indexed resource can be
opened directly from a package. For the previously inspected installation's
`40616200A1C46DF2` resource, the following example uses its observed decoded
payload hash; replace the path with your installation's path:

```powershell
target\debug\rustspore.exe --experimental --source-creature-edit-package `
  'C:\path\to\Spore_Content.package' 40616200A1C46DF2 `
  6852040fa9a9376ed9025f4823e05024a22685a147e00e64d1b0f536b33e8cd8 `
  'target\edited.creaturedata'
```

The 16-digit key is scope/group followed by instance. A unique key and the
SHA-256 of the decoded CreatureData are required; index and payload sizes are
bounded. The package path and file must stay stable across the reader's several
opens. Endpoint hashes are checked, but cannot detect transient replacement.
Keep installed game assets private.

An extracted CreatureData file can also be opened with the SHA-256 of its exact
input bytes:

```powershell
$source = 'target\original.creaturedata'
$sha = (Get-FileHash -Algorithm SHA256 $source).Hash.ToLowerInvariant()
target\debug\rustspore.exe --experimental --source-creature-edit $source $sha 'target\edited.creaturedata'
```

Enter one command per line. `list` shows up to 32 source rows, including each
row ordinal, stored index, part prop instance, recorded parent and symmetric
partner indices, and current translation. `list 32` starts at row 32; the
command prints the next page when there are more rows. These links are source
fields, not validation that an edited placement is allowed. Commands select the
serialized row ordinal, which can differ from the stored index.

`show 0` displays the first serialized row's
absolute rotation matrix and translation. `set 0 1 2 3` replaces that row's
absolute translation while retaining its rotation; `move 0 1 2 3` adds a delta
to the current translation. `transform 0` followed by
nine row-major rotation values and three translation values replaces both.
These inputs must be finite; the command does not assert that the rotation or
placement is valid in the original game. A move whose resulting values are not
finite is rejected without changing the session. `undo`, `redo` and `status` operate
on the current in-memory session. A byte-identical edit keeps the redo branch;
invalid edits do not change the document or history. History holds the most
recent 128 edits and is not written to the payload.

`save` exclusively creates the output selected at startup, flushes and syncs it,
then reads the bytes back through the same open handle and reports their
SHA-256. A failed save leaves the session available; `save-as <new path>` can
try a different destination. Its path can contain spaces and its parent must
already exist under `target`. A failed write or readback can leave a partial or
unverified file, which is not overwritten by another attempt. `quit` or end of
input leaves the current session unsaved. The output directory must remain
stable while saving; an external writer can still modify a file after readback.

The untextured [assembly preview](experimental-source-part-assembly.md) can
inspect the resulting transform when all required source parts resolve. The
preview and editable bytes are engineering tools, not an original editor or a
finished Spore creation.
