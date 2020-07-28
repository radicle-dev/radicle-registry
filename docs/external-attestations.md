# External Attestations

## Feature goals

The Radicle Registry state contains data about users, organizations and software projects.
However, the same entities will also have an identity off chain, in other systems.
For instance, a Registry user could also have an identity on Radicle Link (hereinafter "Link"), or on Github, Gitlab, Twitter, or they will have an email address...
In the same way, a registered project could also exist on Link, Github, or elsewhere.

When relevant it is desirable to be able to link these identities together so that a registry use can be confident that they do in fact represent the same identity.

It is important to distinguish two different kinds of attestation:

1. Simple linking of identities ("_identity attestation_")
2. Detailed certification of a specific identity revision ("_identity revision attestation_" for platforms that support it, like Link)

The second builds on the first but is an optional extension to it.

## General principle of operation

The feature builds on the ability to refer to one identity from another one.

For instance, it should be possible to insert a reference to a Registry user identity inside a Link user identity (by writing the Registry user ID inside the Link user metadata).
It should also be possible to do the reverse: insert the Link user ID (their URN) inside the Registry user state.

The key point is that only the legitimate user can do each of these operations.
Therefore, if both links are established, we can deduce that the same entity (in this case the same user) is controlling both identities, and therefore they both refer to the same person).

The same concept can be applied to any kind of identity (projects, or even organizations if desired.

## Further details

There are still a few open points.

### Security model

One of them is the "security model" of this attestation: what it proves when everything works as intended is clear, but what to deduce is anything goes wrong is not (like, how to handle user key theft or compromise, or how to show partial attestations where only one of the two links is in place).

Related to the security model is the problem of handling key rotation on both sides of the link (both on the Registry and Link).

### Registry storage cost

Then there is the "registry storage cost" problem: in principle the registry state should contain links to every supported external identity (Link, Github, Twitter...) but this list can be quite long.

An easy solution to this problem is to only store a single link inside the registry, unambiguously referring to an external piece of metadata that in turn will contain all other external links.
To guarantee absence of ambiguity this link should be a cryptographic hash of the external metadata.

A practical approach to this would be storing the Link identity on the Registry, and putting every other identity references inside the Link identity metadata, reducing the registry state requirements to a single link, which is acceptable.
Since Link identities are versioned with cryptographic hashes the system would be unambiguous, as required.

### Data format and verification

Let's say we decide to store the Link user identity in the Registry user state; we should then decide how to store it.
In Link identities are represented by Radicle URNs, as described in the (identity resolution)[https://github.com/radicle-dev/radicle-link/blob/master/docs/rfc/identity_resolution.md] and (identities)[https://github.com/radicle-dev/radicle-link/pull/248] specifications.

However we have the following issues:

- Registry does not depend on Link.
- The URN format is not compact, while in the registry state every byte counts.
- The URN format could evolve, or this feature (identity attestation) could evolve, allowing extensions to store something more than Link URNs, or something different; if we wanted Registry to be able to verify the format of this piece of data the core Registry code would need to be updated before each data format change.
- Even if the Registry could parse the URNs, it could no verify them.

The logical consequence of this is to store identity attestation references as opaque blobs for the Registry, and let the application layer deal with the contents (Upstream, or any other application accessing these data).

The data format should be a compact binary encoding of link identity URNs, which should be specified and implemented in Link.

Upstream, given an identity with a registry attestation, should do the following:

- Load the registry identity, note its ID.
- Parse the attested Link URN.
- Load and verify the corresponding Link user metadata.
- Check that the Registry ID stored on the Link side is the same as the starting Registry ID.

If everything is successful the attestation is verified and the identity can be shown as linked, otherwise some error condition should be reported.

### MVP requirements

An MVP for this feature should:

* Specify and implement the compact (and extensible) URN binary encoding format (in Link).
* Provide the two way link between Link and Registry users:
  - Let users store their Link URN in their registry state.
  - Let users store their Registry ID in their Link profile.
* Implement attestation verification in the application layer (Upstream)

After doing this for users we could (should?) also implement it for projects.

We should decide if we need the MVP for Beta1 or Mainnet.
