## Creating a Release

**1. Update version in `Cargo.toml`:**
```toml
[package]
version = "0.1.0"
```

**2. Commit and push to `main`:**
```bash
git commit -am "chore: prepare release v0.1.0"
git push origin main
```

**3. Create and push an annotated tag:**
```bash
git tag -a v0.1.0 -m "Release v0.1.0

- Summary of changes"
git push origin v0.1.0
```

| Environment | Trigger | Version format | Example |
|---|---|---|---|
| Dev | PR to `main` | `x.y.z-beta-<run>` | `0.1.0-beta-42` |
| QA | Push to `main` | `x.y.z-alpha-<run>` | `0.1.0-alpha-42` |
| Release | Git tag `v*.*.*` | `x.y.z` | `0.1.0` |
