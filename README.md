# git-assets

A minimalistic tool for managing large (binary) assets in a Git repository.
It uses smudge and clean filters for transparently switching out the file contents
for checkouts and commits.

It solves a similar problem as Git LFS and Git Annex, but in a different way:
The store for the large files that are not stored in git themselves is a simple directory on the local machine.
This allows `git-assets` to stay simple while leaving syncing the files to tools that are made for syncing.

## Quick start

Use it in a git repository by adding this entry to `.git/config` (assuming that `git-assets`) is on the path:

```
[filter "assets"]
	clean = git-assets clean
	smudge = git-assets smudge
	required
```

and then configure `assets` as filter for the files that should be managed by `git-assets` in `.gitattributes`, e.g.

```
"*.xcf" filter=assets
```

Any `.xcf` files that are staged or committed are stored in `.git/x-assets/`, and the file stored in the repo is replaced by reference to the store, using the sha256 hash of the contents.

## TODO

- **Garbage collection**

  Since files are already put into the store when staging them, the store may end up containing files that were never committed. There should be a way to remove them easily.

  Also, after syncing the store with some other location, one might want to remove files that are not referenced from any git heads anymore.

- **Easy setup**

  A command like `git assets install` that would automatically add the entry to `.git/config` would be great, to make installation as easy as with Git LFS.

  Also, automating adding rules to `.gitattributes` would be nice. Git LFS has `git lfs track ...`, something like that should work for `git assets` as well.

- **Ensure correctness**

  It is probably possible to corrupt the repository when the wrong actions are taken, e.g. adding the filter retroactively for files that were already committed, or removing the filter for files stored in the asset store.

  There should also be integration tests that invoke `git-assets` indirectly via git only.

- **Improve error reporting**

  If referenced files are not present in the store (because they are in some other location and have been deleted locally),
  `git-assets` prints a somewhat incomprehensible error message. Those should be more user-friendly.