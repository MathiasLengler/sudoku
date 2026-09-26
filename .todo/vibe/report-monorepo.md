# Structuring the JS/TS Side of a Rust-WASM-First Monorepo: What the Community Actually Does

## TL;DR

- There is **no single canonical layout** for "Rust workspace + wasm-pack crate + Vite app." Official rustwasm docs still assume Webpack and a _published_ npm package; the community has quietly converged on two healthy patterns — (1) build the wasm crate to a **named npm package consumed as a versioned dependency** (Automerge, Rerun, Bitwarden), and (2) treat pkg/ as a **workspace member** (Ruffle). `link:`/`file:` on pkg/ (the user's current setup) is real but is mostly a local-dev / small-project pattern.
- For this specific repo the best fit is: make `sudoku-wasm/pkg/` a real **pnpm workspace package** (add `sudoku-web` and `sudoku-wasm/pkg` to a root `pnpm-workspace.yaml`, depend via `"sudoku-wasm": "workspace:*"`), build with `wasm-pack build --target web --no-gitignore`, and **migrate to tsify so the generated `pkg/*.d.ts` becomes the single source of TS types** — deleting the separate `sudoku-rs/bindings/` directory, the cross-directory `include` globs, and the dedicated composite typecheck project.
- The workaround the user wants gone (tsconfig `include` globs reaching into `../sudoku-wasm/pkg` and `../sudoku-rs/bindings`) disappears once the wasm output is a node-resolvable workspace package and tsify folds all types into that package's `.d.ts`; the remaining choices are pnpm mechanics (`workspace:*` vs `link:`) and how to guarantee pkg/ exists before install.

## Key Findings

### 1. There is no official canonical layout — and the official docs are stale for Vite

- The rustwasm project templates (`wasm-pack-template`, `create-wasm-app`, `rust-webpack-template`) all assume **Webpack** and a package **published to npm**. `create-wasm-app` is explicitly "designed for depending on NPM packages that contain Rust-generated WebAssembly." None of them address Vite, pnpm workspaces, or a cargo-first monorepo root.
- The official Rust+Wasm book's "Hello World" puts the app in a `www/` subdirectory of the crate and consumes the published `hello-wasm-pack` npm package — not a local pkg/.
- wasm-pack docs take a position on only one relevant thing: **pkg/ is a build artifact.** By default wasm-pack writes a `.gitignore` containing `*` into the output dir "since it contains build artifacts which are not intended to be checked into version control." The docs explicitly note you can pass `--no-gitignore` "if you want to commit the pkg directory to your repository (e.g. for GitHub Pages, Deno packages, or **monorepo setups**)." So committing pkg/ is officially acknowledged but framed as the exception.
- On the Vite side, the ecosystem standard is `vite-plugin-wasm` (Menci fork), usually paired with `vite-plugin-top-level-await`. Its README states verbatim: **"This plugin supports Vite 2.x to 8.x"** (latest v3.6.0 added Vite 8 support), and it explicitly supports wasm-pack-generated modules. Notably, the Vite team has proposed adopting it into core (Menci/vite-plugin-wasm issue #76, opened July 10 2025: the Vite maintainers said "we'd like to implement it as a built-in feature in Vite… we'd love to adopt it into the Vite core"), signaling this is the converging standard for wasm-in-Vite. Plugins that actually _invoke_ wasm-pack (vite-plugin-wasm-pack by nshen, vite-plugin-rsw by lencx/rwasm) exist but are lightly maintained: **vite-plugin-rsw's latest release is v2.0.11, last published ~2 years ago, with 0 dependents**, and vite-plugin-wasm-pack requires `--target web`.

### 2. How real, actively maintained projects consume an unpublished wasm-pack package

Classification across surveyed real projects:

- **Publish as a named npm package, consume as a versioned dependency (dominant for libraries):** Automerge (`@automerge/automerge-wasm` consumed by the `@automerge/automerge` JS wrapper), Rerun (`@rerun-io/web-viewer`), Bitwarden SDK (`@bitwarden/sdk-internal`), ywasm. pkg/ is gitignored and built in CI. Not directly applicable to a never-published project, but the internal structure — a wrapper package re-exporting the generated bindings — is instructive.
- **(B) Workspace member:** Ruffle's `web/` is an npm-workspaces monorepo with `packages/core` (wasm + JS bindings) consumed by sibling packages. This is the closest analog to what the user should do.
- **(E) Bundler plugin invoking the build:** the vite-plugin-wasm-pack / vite-plugin-rsw ecosystem; also the _consumer-side_ norm (Automerge/Rerun docs tell Vite users to add vite-plugin-wasm + top-level-await).
- **(A) link:/file: on pkg/ (user's current approach):** real but mostly local-dev or small projects. Bitwarden documents `npm link` for local dev; `vite-plugin-wasm-pack-watcher`'s README literally documents `"<wasm package>": "link:./pkg"`; MDN's tutorial uses `"hello-wasm": "file:../pkg"`.
- **(C) --out-dir into app src** and **(D) hand-written wrapper package.json:** least common; (C) appears in tutorials (e.g. the Game-of-Life Vite port writes wasm-bindgen output to a top-level `pkg/`), (D) is essentially what the "publish as package" projects do internally.
- **Committing pkg/:** uniformly gitignored across all surveyed flagship projects; none commit pkg/. The user's current choice to commit pkg/ is against the prevailing norm, though officially supported.

### 3. The workspace-package variant: regeneration and fresh-clone problems

- **wasm-pack regenerates pkg/package.json on every build.** This is fine for `workspace:*` because pnpm re-reads it, but the churn is real; wasm-pack also rewrites the `.gitignore` each build — a long-standing annoyance tracked in drager/wasm-pack issues #728 and #691 and PR #1408 (which proposes removing the auto-`.gitignore` generation).
- **Fresh-clone / gitignored-pkg problem:** if pkg/ is gitignored, `pnpm install` fails because the workspace member doesn't exist yet. Community solutions: (i) build wasm before install (a `wasm-pack build` step in a justfile/CI before `pnpm install`), (ii) commit pkg/ with `--no-gitignore` so the member always exists, or (iii) commit only a minimal stub `package.json` and gitignore the artifacts. There is **no single standardized flag** for this; it is handled per-project with build scripts.
- pnpm's `pnpm deploy`/pack rewrites `workspace:*` to real versions on publish (pnpm issue #6269 notes deploy doesn't rewrite in some cases), but since the user never publishes, this is irrelevant.

### 4. TypeScript structure for generated sources living outside the app

- Accepted modern approaches, in order of community endorsement: (1) **consume the generated code as a workspace/node_modules package** (symlinked) — Nx's monorepo guide calls "workspaces + project references" the recommended combination; (2) **TypeScript project references**; (3) cross-directory `include` globs — the user's current workaround — which is least-liked because it breaks module-boundary assumptions and forces separate strictness projects.
- The "internal package exporting raw .ts source" pattern (package.json `exports`/`types` pointing at `.ts`) works with `moduleResolution: "bundler"` (endorsed in the manzt gist, discussed by moonrepo), but moonrepo warns it loses incremental caching and forces tsc to re-parse source every time. For **generated `.d.ts`** (what wasm-pack + tsify emit), this downside does not apply — `.d.ts` loads fast — so a package that exports generated `.d.ts` is the clean path.
- `moduleResolution: "bundler"` (the Vite default) respects package.json `exports`, so once pkg/ is a resolvable package, the app imports `sudoku-wasm` by name and TS finds the bundled `.d.ts` with no cross-directory globs.

### 5. tsify folds Rust types into the generated pkg/ .d.ts — and the fork situation has reversed

- tsify generates TypeScript definitions from Rust via serde, and crucially "using this with wasm-bindgen will automatically output the types to .d.ts" — the types land **inside the wasm-pack-generated pkg/** via wasm-bindgen's `typescript_custom_section` mechanism, not in a separate directory. This is the key advantage over ts-rs: **ts-rs writes standalone `.ts` files to an arbitrary directory (the `sudoku-rs/bindings/` the user imports by relative path), whereas tsify makes pkg/\*.d.ts the single source of truth.** Migrating to tsify removes the need for `sudoku-rs/bindings/` entirely.
- tsify handles complex serde types well: structs → interfaces, enums (including data-carrying/ADT enums) → discriminated unions, generics, and `Option`/`#[tsify(optional)]` handling. Bitwarden's engineering docs describe the split: "Use tsify unless the web-side needs to call functions or interact with Rust objects directly, in which case, use wasm-bindgen" — because wasm-bindgen exposes _live_ Rust objects (classes with methods) while tsify exposes _serialized data_ (plain typed objects). The user's app runs sudoku-wasm inside a web worker passing data across a boundary — exactly tsify's sweet spot for data types, with wasm-bindgen classes for the stateful handle.
- **Maintenance status / fork situation (important, and it has flipped):** the original `madonoharu/tsify` went dormant, spawning `tsify-next` (siefkenj; also republished by AmbientRun), whose own README conceded "The original Tsify appears to be in hibernation mode. This repository will maintain updates until main Tsify project comes back to life." That situation has now **reversed**: `madonoharu/tsify` is active again — confirmed via docs.rs source, tsify **0.5.6** lists authors "Madono Haru" and "Jason Siefken" and pins `wasm-bindgen = { version = "0.2.104", optional = true }` — while **tsify-next is formally deprecated by RustSec advisory RUSTSEC-2025-0048** ("tsify-next is unmaintained, use tsify instead"), which states verbatim: "The tsify-next crate is not maintained any more; use tsify instead." **New code should target `tsify` (madonoharu), not tsify-next.**
- API note: the newest tsify **deprecates the `into_wasm_abi`/`from_wasm_abi` attributes in favor of a `Ts<T>` wrapper type** used on function signatures. The README marks each of `into_wasm_abi`/`from_wasm_abi` "(deprecated) … Deprecated in favour of using `Ts<T>` as on function parameters and return type," with new usage like `pub fn from_js(point: Ts<Point>) -> Result<(), JsError>` and `point.to_rust()?`. Pin a version, since the surface is shifting.
- Known limitations vs ts-rs: tsify is wasm/serde-coupled (built for the wasm-bindgen boundary, whereas ts-rs is a general Rust→TS exporter usable without wasm); tsify types are serde-shaped, so custom serialization needs `#[tsify(type = ...)]` or the manual `impl_custom_tsify!` escape hatch; and there have been intermittent wasm-bindgen/tsify version-coupling issues (observed in the ezno project's CI). For a project already all-in on wasm-bindgen, these are minor.

### 6. pnpm specifics: link: vs file: vs workspace:*

Per pnpm's own docs:

- **`link:`** — symlinks the target directory; pnpm does **not** install the linked package's dependencies (you manage them manually). This is the user's current setup. It works but pnpm treats it as an external link, not a first-class workspace member, so it does not participate in workspace-wide operations, filtering, or peer resolution the same way.
- **`file:`** — hard-links the package into node_modules and **does** install its dependencies, overriding the target's node_modules. pnpm docs recommend `file:` over `link:` "when dealing with peer dependencies."
- **`workspace:*`** — first-class workspace member: "pnpm will refuse to resolve to anything other than a local workspace package." Gets a proper lockfile entry, participates in `pnpm --filter`, recursive scripts, and strict peer resolution, and (if ever published) is rewritten to a real version. This is the pnpm-endorsed way to depend on local packages in a monorepo.
- pnpm does not forbid `link:` but its workspace documentation clearly centers `workspace:*` as the monorepo mechanism. For the user's case `workspace:*` is strictly better than the current `link:`: it makes pkg/ a real member, gives a lockfile entry, and lets a single root `pnpm-workspace.yaml` replace the current empty one inside `sudoku-web/`.

## Details

### The user's current setup, assessed

The repo (`MathiasLengler/sudoku`, ~2,217 commits, active, MIT-licensed, Rust 77.5% / TypeScript 20.8%) is a cargo workspace at root with `sudoku-rs` (core lib), `sudoku-wasm` (wasm-bindgen wrapper), and `sudoku-web` (React + Vite + TS via pnpm), plus `sudoku-bubblewrap`. The pain points break into three independent problems:

1. **Dependency wiring:** `"sudoku-wasm": "link:../sudoku-wasm/pkg"` + an empty `pnpm-workspace.yaml` inside `sudoku-web/` that declares no packages. A non-workspace using a raw symlink.
2. **Two type-generation paths:** wasm-pack emits `pkg/*.d.ts` (consumed via the link) AND ts-rs emits standalone `.ts` into `sudoku-rs/bindings/` (consumed via relative-path imports + cross-directory tsconfig `include` globs + a dedicated composite tsconfig for different strictness).
3. **Committed pkg/:** against the community norm but not wrong.

Migrating to tsify collapses problem 2 into problem 1: once types are generated _into_ pkg/, there is only one artifact to consume, and if that artifact is a proper package, the tsconfig globs vanish.

### Why workspace:* over the alternatives for this repo

- vs `link:` (current): `workspace:*` gives a real member, lockfile entry, filtering, and lets you delete the misplaced `sudoku-web/pnpm-workspace.yaml` in favor of a root one listing `sudoku-web` and `sudoku-wasm/pkg`.
- vs `file:`: `file:` copies/hard-links and reinstalls deps on each change — worse for a frequently-regenerated artifact; `link:`/`workspace:*` symlinks reflect rebuilds instantly. Since pkg/ has essentially no runtime deps of its own, file:'s peer-dep advantage is moot.
- vs bundler plugin (vite-plugin-wasm-pack / rsw): adds a lightly-maintained dependency and couples cargo build to Vite; the user explicitly prefers not to let a plugin link the package (echoing sentiment in vitejs/vite discussion #2584). Keep the build in the justfile.
- vs `--out-dir` into `sudoku-web/src`: pollutes app source with generated artifacts and re-creates the cross-directory problem inside the app.

### Handling the fresh-clone / regeneration problem

Because `workspace:*` requires the member to exist at install time, and wasm-pack regenerates pkg/package.json (and .gitignore) every build:

- Build wasm as a `prepare`/`postinstall`-adjacent step, or document `just build-wasm` before `pnpm install` in CI and contributor docs.
- Run `wasm-pack build --target web --no-gitignore` so wasm-pack stops rewriting the ignore file; control ignoring from the root `.gitignore`.
- Decide committed vs gitignored: committing pkg/ guarantees fresh-clone installs succeed with zero extra steps (the pragmatic choice for a solo/small project, which is why the user does it today); gitignoring matches flagship-project norms but requires the pre-install build step. Since CI already has Rust, gitignoring pkg/ + a build step is the cleaner end state, but committing is defensible.

## Recommendations

**Stage 1 — Make pkg/ a real workspace package (do first, independent of tsify).**

1. Create a root `pnpm-workspace.yaml` listing `sudoku-web` and `sudoku-wasm/pkg`; delete the empty one inside `sudoku-web/`.
2. Change `sudoku-web`'s dependency from `"sudoku-wasm": "link:../sudoku-wasm/pkg"` to `"sudoku-wasm": "workspace:*"`.
3. Change the wasm build to `wasm-pack build --target web --no-gitignore` (in the justfile) so the ignore file stops churning.
4. Ensure pkg/package.json's `name` is `sudoku-wasm` (wasm-pack derives it from the crate name; set the crate name or use `--out-name`).
5. Success benchmark: `pnpm install` at root wires the symlink; `import ... from "sudoku-wasm"` resolves in the app with no relative paths.

**Stage 2 — Migrate ts-rs → tsify and delete the bindings workaround.** 6. Add `tsify` (madonoharu, **NOT tsify-next** — it is RustSec-deprecated) + serde derives to the shared types; annotate with `#[derive(Tsify)]`. Pin the tsify version and use the current `Ts<T>` API rather than the deprecated `into_wasm_abi`/`from_wasm_abi` attributes. 7. Verify types now appear in `sudoku-wasm/pkg/*.d.ts`. Delete `sudoku-rs/bindings/`, the ts-rs export code, the cross-directory tsconfig `include` globs (`../sudoku-wasm/pkg/*.ts`, `../sudoku-rs/bindings`), and the dedicated composite typecheck project. 8. Rely on `moduleResolution: "bundler"` to resolve `sudoku-wasm`'s bundled `.d.ts` via package.json `exports`/`types`. 9. Success benchmark: `tsc` typechecks the app with a single tsconfig, no paths escaping the project dir, and Rust types flow through automatically.

**Stage 3 — Decide committed vs gitignored pkg/.** 10. Preferred: gitignore pkg/ and add a `just`/CI step that runs the wasm build before `pnpm install`. If fresh-clone friction is unacceptable, keep committing pkg/ (with `--no-gitignore`) — officially supported for monorepos.

**Thresholds that would change the recommendation:**

- If the user ever wants to **publish** sudoku-wasm to npm → keep `workspace:*` (pnpm rewrites it on publish) and stop committing pkg/.
- If Rust rebuild latency in dev becomes painful → add `vite-plugin-rsw`/watcher for auto-rebuild, accepting the maintenance risk (recall rsw is ~2 years stale).
- If tsify's serde-shaped types can't express a needed live-object API → keep that specific type on plain `#[wasm_bindgen]` classes (the Bitwarden split), using tsify only for data DTOs.

## Caveats

- **No settled standard exists.** The strongest evidence-backed claim is: "workspace member (B) or publish-as-package are what healthy monorepo/library projects do; link:/file: is a local-dev / small-project pattern." The user's project is small, so the current `link:` setup is not _wrong_ — it just isn't what scales, and it blocks the single-source-of-types cleanup.
- Star counts and some package.json specifics for the surveyed projects (Automerge, Ruffle) were inferred from release docs and repo listings, not read verbatim, because GitHub raw/tree fetches were blocked; the _mechanism_ classifications are reliable but exact dependency strings were not always directly viewable.
- tsify's newest release deprecates `into_wasm_abi`/`from_wasm_abi` in favor of `Ts<T>`; the API is shifting, so pin a version.
- The tsify vs tsify-next maintenance status has already flipped once; re-verify current activity (and any newer RustSec advisories) before committing, though as of now the direction is unambiguous: tsify active, tsify-next deprecated.
