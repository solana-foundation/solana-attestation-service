# Contributing

Thanks for contributing to the Solana Attestation Service, the Solana program and clients for issuing and verifying on-chain attestations.

## Before you start

- Search existing issues and pull requests before opening a new one.
- Open an issue describing the change before writing code for anything beyond a typo, a broken link, or a comment-only fix. Agreeing on the approach first is what keeps a finished change from being rejected on scope. In general, small PRs are preferred.
- Do not include secrets, private keys, seed phrases, or production credentials in issues, pull requests, commits, logs, or screenshots.
- All commits into a Solana Foundation repository require [commit signature verification](https://docs.github.com/en/authentication/managing-commit-signature-verification/about-commit-signature-verification) to be enabled. CI requests changes on a PR carrying an unverified commit and dismisses that review once every commit verifies.

## Security vulnerabilities

Do not report security vulnerabilities in public issues. Follow the [security policy](./SECURITY.md) and use the [Report a Vulnerability](https://github.com/solana-foundation/solana-attestation-service/security/advisories/new) link. Expect a response in the advisory, typically within 72 hours.

## Development setup

Toolchain versions are checked into the repository: Rust in `rust-toolchain.toml` (installed automatically by rustup), Node.js in `.nvmrc` (`nvm use` or `fnm use`), and pnpm in the `packageManager` field of `package.json`. Generating the IDL also needs the Shank CLI (`cargo install shank-cli`). Do not update language runtimes, the Solana CLI, or package-manager versions as an incidental part of another change.

```sh
just setup          # install dependencies and configure git hooks
just build          # program .so, IDL, generated clients, TypeScript client
just check          # format check + lint check (also run by the pre-push hook)
just test           # Rust unit tests, integration tests, TypeScript client tests
```

`just --list` shows every recipe.

## Making a change

Keep changes focused. A pull request should solve one problem and include the tests, documentation, generated artifacts, or migration notes needed to keep the repository usable.

Before opening a pull request:

- Run `just check` and `just test` for the affected code.
- Add or update tests when behavior changes. Rust integration tests live in `integration_tests/tests/` and TypeScript tests in `clients/typescript/test/`.
- Run `just generate-clients` whenever program instructions, accounts, events, or types change, and commit the resulting `idl/` diff. The generated client sources under `clients/*/src/generated/` are not committed; they are produced from the IDL at build and publish time. CI runs `just check-generated` and fails on drift.
- Update `README.md` and the relevant `CHANGELOG.md` when the change is part of the user-facing contract. The program, the Rust client, and the TypeScript client each keep their own changelog.
- Explain any new dependency and why the existing dependency set is insufficient. The program is built on Pinocchio; do not introduce `anchor-lang`.

For on-chain changes, document relevant account validation, authority, state-transition, or value-movement considerations, and include a threat-model note when the change creates or modifies a trust boundary. Account layouts and instruction interfaces are expensive to change once anyone depends on them: say explicitly whether a change is compatible with accounts already on-chain.

## Pull requests

Fill in every section of the pull request template: the problem, the approach, how you tested it, and the [AI disclosure](#disclosure). Link related issues and call out behavior changes, compatibility concerns, or follow-up work. CI fails the PR until the disclosure is declared. Use [Conventional Commits](https://www.conventionalcommits.org/) for your commit naming, and name branches `<type>/<short-description>` (for example `fix/attestation-expiry-overflow`).

By default, [Greptile](https://www.greptile.com) is enabled on all Solana Foundation repositories. Before maintainers review, all Greptile comments must be resolved with either a code fix or an explanation of why no change is needed.

Once CI is approved to run by maintainers, all CI errors must be addressed before the PR will be merged.

Maintainers may ask you to rebase, split a broad change, add tests, or revise documentation before merging.

Reviewers are assigned from [CODEOWNERS](.github/CODEOWNERS). Changes to the program, the IDL, or the generated clients need a review from a program maintainer.

## AI use

You may use AI-assisted tools, but you should review the generated code, understand its behavior, and run the same checks expected of any other contribution.

If you are building with AI on Solana, check out the [Solana Dev Skill](https://github.com/solana-foundation/solana-dev-skill) or the [Solana MCP](https://mcp.solana.com/) to aid in your work. This repository ships a [CLAUDE.md](./CLAUDE.md) and [AGENTS.md](./AGENTS.md) with the repo-specific gotchas an agent needs: the non-Anchor wire format, the Shank-to-Codama generation pipeline, the event CPI discriminator. Read it before letting an agent loose here.

Ensure that the generated code adheres to the project's coding standards and best practices. Maintainers can close PRs if they appear to be low-effort AI slop. In particular, audit your changes for the following AI code smells that increase maintenance burden:

- Comments that explain why the _previous_ behavior was wrong and the new behavior is correct. This can be helpful context for reviewers as a Github comment in the review, but we do not need a history of every code change living in the codebase
- Large blocks of comments with high density of technical jargon; comments should be distilled to clearly explain _why_ this code is doing something (if it's not obvious), not _what_ (the code should speak for itself).
- Drive-by refactoring of code that is not relevant to the actual change being made.

Two more that matter here: never hand-edit files under `idl/` or `clients/*/src/generated/`, regenerate them instead, and do not let an agent add defensive checks or allocations to instruction handlers, which run under a compute budget.

You must be able to explain every line of your diff without an LLM. Reviewers may ask you a pointed question about any part of the change; if the answer is pasted from a model or does not come, the PR is closed.

Tool attribution left in a PR (a `Generated with Claude Code` footer, a `Co-Authored-By: Claude` trailer, a `cursor/` or `codex/` branch, and the like) tells us the submission was opened without being read. CI labels these `ai-unreviewed`, fails the check, and explains what to fix. PRs left in that state are closed.

### Disclosure

Disclosure is required. The pull request template has two boxes; check exactly one. If AI tooling was used, name the tool and the extent, for example:

> I wrote all of the code for this feature, and had Claude update the documentation and create tests accordingly

or

> I architected the change and handed all implementation over to Codex

Editor autocomplete of single keywords or short phrases does not count as AI tooling.

### Communication

If maintainers have suggested changes, feedback, or questions about your code, you should not be copy/pasting the questions to an LLM and copy/pasting the response. You being able to distill the information that AI produces it what makes your contribution valuable.

## License

By contributing, you agree that your contributions are licensed under the project's [LICENSE](./LICENSE).
