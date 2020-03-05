# Abbreviations
You may use common abbreviations in the codebase as long as they:
- don't obscure their meaning for the sole sake of saving a few characters
- are common programming abbreviations (e.g. `num`, `idx` or `iter`)

For consistency we always use the following domain-specific abbreviation. You
may not use any other domain-specific abbreviations.
- Tx - Transaction
- PoW - Proof of Work
- Org - Organization

# Git Flow

TL;DR: This repository follows the Git rebase flow for the most part. Each feature branch must
be reviewed and approved in a pull request. Once ready to merge, consider squashing the branch's
commits when the separate commits don't add value, rebase it, force push with lease, and merge it
via the GitHub UI.

## Branches

1. Create a separate branch for each issue your are working on
2. Do your magic
3. Keep your branch up to date by rebasing it from its base branch
4. Delete the branch after its been both approved and merged. GitHub does this automatically.

## Commits

1. Make sure you author your commits with the right username and email
2. Follow the git commit convention:
  - Use the imperative mood in the subject line
  - Limit the subject line to 50 chars
  - Capitalise the subject line
  - Wrap the description at 72 characters
  - Have the description preferably explaining what and why instead of how
  - Separate the subject from the body with an empty line

# Documentation

- Document all the public items of a module especially well given that they constitute its API.
- Document private items when the reader's understanding to fully grasp its usage or implementation
  is insufficient, despite the limited context of such items.
- The code must be documented using Rustdoc comments.
- Strive for self-documenting code.
- The reader of the documentation is expected to have general knowledge in the fields of blockchains
  and cryptocurrencies.
- Leave additional comments explaining 'why' rather than 'how', strive to have the code clean
  and elegant to make this aim viable.

# Changelog

We use `./CHANGELOG.md` to record all changes visible to users of the client.
Changes are added to the “Upcoming” section of the change log as part of commit
that makes the change. That is they are included in every pull request. For
breaking changes a migration path must be provided.
