# Deploying the site

The site is a Next.js app that embeds the CFDL engine compiled to WebAssembly.
That one fact drives everything below: **the wasm bundle needs a Rust
toolchain, and Vercel's build image does not have one.**

## How it works

GitHub Actions builds — it is the only place with Rust — and hands Vercel a
finished artifact. Vercel does not build.

```
push ──> GitHub Actions ──> install Rust + wasm-pack
                       ──> ./scripts/build-wasm.sh      (engine -> public/wasm/)
                       ──> vercel build                 (next build -> .vercel/output)
                       ──> vercel deploy --prebuilt     (upload only)
```

- **A branch or PR** deploys to a preview URL.
- **`main`** deploys to production.
- **Locally**: `npm run build:wasm` once, then `npm run dev`. The bundle is
  gitignored and rebuilt on demand.

Deployment is still automatic on every push. What changed is which machine
compiles, not whether a human has to press anything.

## The token is project-scoped, and that shapes the workflow

Vercel requires choosing a project when issuing a token, so a **project-scoped**
token is what is available. It can deploy to its own project. It cannot answer
"who am I" — user and team lookups are outside its scope by design.

That rules out `vercel pull`, which resolves the token to a user, then to a
team, before it ever reaches the project. It fails at the first step:

```
Error: Not able to load user because of unexpected error: User not found. (404)
```

The error names a *user*, not a permission, which is the clue: widening the
scope was never the fix, because there is no wider scope to widen to.

**`pull` is only needed for the lookup, not for the settings.** So the workflow
writes `.vercel/project.json` itself, from the two secrets plus the project's
own root directory, and never calls `pull`. Three things had to be right, each
found by a separate failure:

| | |
|---|---|
| `projectId` + `orgId` alone | not enough — `vercel build` wants a `settings` block too |
| steps running in `site/` | the project already sets `rootDirectory: site`, so paths doubled to `site/site` |
| `rootDirectory: null` in the written link | sent `build` to the repo root, where there is no `next` dependency |

Build and deploy therefore run from the **repository root**, with the written
link naming `site` as the root directory. The wasm build and its smoke test stay
in `site/` — those are npm scripts, not Vercel calls.

### What skipping `pull` costs

`vercel pull` also fetches the project's **environment variables**. The workflow
does not, so CI never sees them.

That is fine today: nothing this site builds needs one. `NEXT_PUBLIC_WASM_BUILD`
is computed in `next.config.ts` from the build stamp. But if an environment
variable is ever added in the Vercel dashboard, **CI will not pick it up** — it
has to be added as a GitHub secret and passed in the workflow as well.

### Why the bundle is not committed

It used to be, because Vercel could not build it. Twenty-seven bundles
accumulated that way — 45 MB of uncompressed blobs, though git deltas
successive wasm builds well enough that they cost about 3.7 MB of the packed
repository, roughly a third of it. The growth is slow but monotonic, and it is
growth for an artifact nobody can review.

It also required machinery to keep honest: a source-hash stamp, a freshness
check against the base ref, and a version check, all of which existed *only*
because a generated artifact was in git. Building in CI deletes the problem
rather than policing it — a bundle built from the current sources is fresh by
construction.

### There is no OIDC alternative to the token

Worth stating because it looks like there should be, and `vercel env pull`
leaves a `VERCEL_OIDC_TOKEN` in `site/.env.local` that makes it look likelier.

Vercel's OIDC federation runs the other way: Vercel issues short-lived tokens so
a *deployed app or build* can authenticate to AWS, GCP or Azure without stored
cloud credentials. It is not a means for an external CI system to authenticate
**to** Vercel. A `VERCEL_TOKEN` is the supported path for deploying, and
project-scoping is what makes it tight.

`check-wasm-smoke.mjs` survives all of that and should stay. It is a functional
test of the built bundle, not a freshness check, and it is the only thing that
would catch a bundle that builds cleanly and does not run.

---

## One-time migration

Do these in order. The site keeps working throughout — nothing here is
irreversible.

The point of the migration is to stop Vercel needing a pre-built artifact, not
to reclaim space. Measured on this repo, the whole committed history of the
bundle costs about 3.7 MB of a 10.6 MB packed repository; purging it is
optional cleanup and is kept in an appendix rather than the main path, because
rewriting published history invalidates every clone in exchange for very
little.

### 0. Back up

Cheap insurance, and a prerequisite if you later run the appendix.

```bash
cd ~/Documents
git clone --mirror https://github.com/bizarc/cfdl.git cfdl-backup.git
tar czf cfdl-backup-$(date +%Y%m%d).tar.gz cfdl-backup.git
```

Keep the tarball off the machine you are working on. It contains every commit,
branch and tag as they are today, and it is the only way back.

### 1. Vercel: get the project IDs and a token

The Vercel UI moves; these are the current paths and the labels may differ
slightly.

1. **Link the project locally** (once), to learn its IDs:
   ```bash
   cd site
   npx vercel@latest login
   npx vercel@latest link
   ```

   Where the CLI writes the result depends on which link it performs, and
   recent versions default to the repo-level one:

   | link type | file | shape |
   |---|---|---|
   | repo-level (current default) | `<repo root>/.vercel/repo.json` | `projects[0].id` and `projects[0].orgId`, plus the project's `directory` |
   | project-level (older) | `site/.vercel/project.json` | `projectId` and `orgId` at the top level |

   So read whichever exists:
   ```bash
   cat ../.vercel/repo.json 2>/dev/null || cat .vercel/project.json
   ```

   On a repo-level link, check that `projects[0].directory` is `site`. That is
   the CLI recording where the app lives, and it is what makes a root-level link
   correct rather than a mistake.

   Both locations are gitignored — the root `.vercel/` by the bare `.vercel`
   rule, `site/.vercel/` by its own. Worth confirming with
   `git check-ignore -v` rather than assuming, because the `site/.vercel/` rule
   on its own does not cover a root-level directory.

2. **Create a PROJECT-SCOPED token.** Vercel supports three scopes — Full
   Account (your account *and every team you belong to*), Team (one team, all
   its projects), and Project (one project). Use Project: it "denies any request
   to another project, to a user-level resource, or to a team-level resource",
   so a leak costs this project's deployments and nothing else.

   From the [Account Tokens page](https://vercel.com/account/tokens) — the
   scope selector at top-left must show your **personal account**, not a team;
   team settings have no Tokens entry, which is the usual reason people cannot
   find it. Then: **Scope** dropdown → select the team that owns `cfdl` → it
   drills into that team's projects → select **cfdl**.

   Selecting **All Projects** silently gives you a *team*-scoped token instead.
   Project tokens are prefixed `vcp_`.

   On expiry: because the scope is one project the blast radius is small, which
   justifies a longer life than a full-account token would. Expiry also fails
   safe — deploys stop rather than misbehave. Pick the longest you will actually
   put in a calendar reminder, and not "no expiration": a standing credential
   with no review date is one nobody ever revisits.

   Some teams require 2FA before they will issue tokens scoped to them; the
   dashboard says so when you select the team.

### 2. GitHub: add the secrets

Repository → **Settings** → **Secrets and variables** → **Actions** →
**Repository secrets** → *New repository secret*, three times:

| name | value |
|---|---|
| `VERCEL_TOKEN` | the token from step 1 |
| `VERCEL_ORG_ID` | `projects[0].orgId` (or top-level `orgId` on an older link) |
| `VERCEL_PROJECT_ID` | `projects[0].id` (or top-level `projectId`) |

**Repository secrets, not Environment secrets.** GitHub will already be showing
`Preview` and `Production` environments — the Vercel GitHub integration creates
those to post deployment status, and they are not where these belong. They also
go vestigial once step 4 disconnects that integration.

There is no production/preview split to make here in any case: the two IDs
identify the *project* rather than an environment, and one token authenticates
you rather than a target. The distinction is made inside the workflow, from the
git ref —

```
--environment=${{ github.ref == 'refs/heads/main' && 'production' || 'preview' }}
```

— which is what `vercel pull` uses to fetch the right *application* environment
variables from Vercel. Those are the ones Vercel scopes by Production/Preview.
These three are CI credentials, one layer up.

If you later want a human gate on production deploys, that is the one case where
Environment secrets earn their keep: put `VERCEL_TOKEN` in the `Production`
environment, add a required reviewer, and declare `environment: Production` on
the deploy job.

### 3. Add the deploy workflow, and prove it works BEFORE turning Vercel off

Add the job below to `.github/workflows/site.yml` (or a new `deploy.yml`).
Push it on a branch and confirm the preview URL renders the playground and runs
a model. Only when that passes do you disable Vercel's own build.

```yaml
  deploy:
    runs-on: ubuntu-latest
    env:
      VERCEL_ORG_ID: ${{ secrets.VERCEL_ORG_ID }}
      VERCEL_PROJECT_ID: ${{ secrets.VERCEL_PROJECT_ID }}
    steps:
      - uses: actions/checkout@v4

      # The whole reason this job exists: Vercel's image has no Rust.
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - name: Install wasm-pack
        run: cargo install wasm-pack --version 0.13.1 --locked

      - uses: actions/setup-node@v4
        with:
          # Match the Vercel build image so CI cannot pass where deploys fail.
          node-version: "24"
          cache: npm
          cache-dependency-path: site/package-lock.json

      - name: Install
        working-directory: site
        run: npm ci

      - name: Pull Vercel environment
        working-directory: site
        run: npx vercel@latest pull --yes
             --environment=${{ github.ref == 'refs/heads/main' && 'production' || 'preview' }}
             --token=${{ secrets.VERCEL_TOKEN }}

      # BEFORE `vercel build`, so the bundle is in public/ when Next copies it.
      - name: Build the wasm bundle
        working-directory: site
        run: npm run build:wasm

      - name: Functional smoke test of the built bundle
        working-directory: site
        run: node scripts/check-wasm-smoke.mjs

      - name: Build
        working-directory: site
        run: npx vercel@latest build ${{ github.ref == 'refs/heads/main' && '--prod' || '' }}
             --token=${{ secrets.VERCEL_TOKEN }}

      - name: Deploy
        working-directory: site
        run: npx vercel@latest deploy --prebuilt
             ${{ github.ref == 'refs/heads/main' && '--prod' || '' }}
             --token=${{ secrets.VERCEL_TOKEN }}
```

**Ordering matters in one place**: `build:wasm` must run before `vercel build`,
because `vercel build` runs `next build`, and Next copies `public/` as it finds
it.

### 4. Vercel: stop it building

Only after step 3's preview deploy is verified.

Vercel dashboard → project → **Settings** → **Git**. Either:

- **Disconnect the Git repository** — cleanest. Vercel stops watching entirely
  and only ever receives prebuilt uploads from CI; or
- leave it connected and set **Ignored Build Step** to `exit 0`, which makes
  Vercel skip the build. Prefer disconnecting: with the integration live, a push
  still creates a deployment record, and having two paths that can both produce
  a deployment is the thing worth removing.

### 5. Remove the bundle from the working tree

Land this as an ordinary commit.

```bash
printf 'site/public/wasm/\n' >> .gitignore
git rm -r --cached site/public/wasm/
```

Then delete what is now dead weight:

- `site/scripts/wasm-stamp.mjs` and `site/public/wasm/.build-stamp`
- `site/scripts/check-wasm-fresh.mjs`
- `site/scripts/check-wasm-version.mjs`
- the `check:wasm` composite script in `package.json` — replace with a direct
  call to `check-wasm-smoke.mjs`
- the `wasm-check` target's stamp/version steps in the root `makefile`
- the `crates/**` and `Cargo.toml` path filters in `site.yml`, which existed
  only so the freshness gate would run

Keep `BUDGET_KB` in `build-wasm.sh`. It now runs in CI on every deploy, which is
a better place for it than a developer's laptop.


---

# Appendix: purging the bundle from history (optional)

**Irreversible, and probably not worth it.** Measured on this repo: 10.56 MiB
packed before, 6.87 MiB after — 3.69 MB, about a third. The 45 MB of blob bytes
that number is sometimes quoted as is uncompressed; git deltas successive wasm
builds against each other well enough that twenty-seven of them cost roughly
140 KB each in the pack.

Once the migration above has landed, nothing new accumulates, so this only ever
recovers what is already there. Do not start without step 0's tarball.

```bash
pip install git-filter-repo          # or: brew install git-filter-repo

cd ~/Documents/cfdl
git filter-repo --path site/public/wasm/ --invert-paths
```

`git filter-repo` refuses to run on a repo with a remote configured, and
deliberately drops the remote after rewriting — both are guard rails, not bugs.
Re-add it and force-push everything:

```bash
git remote add origin https://github.com/bizarc/cfdl.git
git push --force --all origin
git push --force --tags origin
```

All 12 tags are rewritten, so they must be force-pushed too. Verify:

```bash
git count-objects -vH | grep size-pack     # expect roughly 16 MB, from 63 MB
git log --all --oneline -- site/public/wasm/ | wc -l   # expect 0
```

## Afterwards (only if you ran the appendix)

Every existing clone is now incompatible and must be re-cloned. On this repo
that is a short list — at the time of writing it had **0 forks, 0 stars and 0
open pull requests**, so the only affected clones are your own. Delete them and
clone fresh rather than trying to rebase across the rewrite.

Any open branch you care about should be merged or rebased *before* step 6, not
after.
