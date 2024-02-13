## How to publish a new version of the docker image

1. Update the version in `studio/package.json` and `Cargo.toml`, you need to keep the version in sync.
2. Commit the changes with the message `Release vX.Y.Z` and tag the commit with the same version (e.g. `git tag vX.Y.Z`).
3. Push the commit and the tag to the repository.
4. You can get the docker image from the [GitHub Container Registry](https://github.com/orgs/open-prophetdb/packages?repo_name=biomedgps) or from the command line with `docker pull ghcr.io/open-prophetdb/biomedgps:vX.Y.Z`.
