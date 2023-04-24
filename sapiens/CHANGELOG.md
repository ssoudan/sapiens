# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.0 (2023-04-24)

### Chore

 - <csr-id-cc8115aed3e0723dbc3158317b7e62c94c6021de/> deps updated
 - <csr-id-7ea6a11630303aefa30680b17f67d7f45ef08c15/> deps updated
 - <csr-id-1a0bdf6c3525dfee29211feab7a9700c632a3195/> API changes in deps

### New Features

 - <csr-id-8be8d0f3044a1bbfb5fdbc6fb6c7c8ec682f7114/> Discord bot
 - <csr-id-774d5a6c2dbadf934166e2d7e5665b6cf2ac4280/> async TaskProgressUpdateHandler
 - <csr-id-f416adf7af52b0a907a9db8419bcdaa5f2a77dc5/> basic integration with discord
 - <csr-id-dee80b442c8035b4d2bf17a2683ff2c3c2a9a585/> basic integration with discord
 - <csr-id-c4981fef8e0fa65a866ddff1582d6b4b39eae8c7/> prototype of a Discord bot
 - <csr-id-d497429d749d9f62a36133e29d1063e8842bd7cf/> more detailed updates while working on a task

### Bug Fixes

 - <csr-id-c37e7ca813eaff1fe87b11736670024b81c5088d/> sorted tool descriptions (by name)
 - <csr-id-c0604dc55545bb092bab88baa3138e7d4401f72d/> duplicate entry
 - <csr-id-0ac757b5621dd659115c6f1650fbc7915162ce5c/> deps

### Refactor

 - <csr-id-15c52d843721fa8426573d9f6bee2c019d2bd9bb/> main loop to process model and tools outputs

### New Features (BREAKING)

 - <csr-id-772e8eb4184efd0b305e460a31d719c237790098/> async api for Tools and in most places actually

### Refactor (BREAKING)

 - <csr-id-b11b947d3b3699807c03c4500a8dc7a0e53d41d0/> main loop to process model and tools outputs (part 2)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release over the course of 3 calendar days.
 - 4 days passed between releases.
 - 15 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Discord bot ([`8be8d0f`](https://github.com/ssoudan/sapiens/commit/8be8d0f3044a1bbfb5fdbc6fb6c7c8ec682f7114))
    - Sorted tool descriptions (by name) ([`c37e7ca`](https://github.com/ssoudan/sapiens/commit/c37e7ca813eaff1fe87b11736670024b81c5088d))
    - Duplicate entry ([`c0604dc`](https://github.com/ssoudan/sapiens/commit/c0604dc55545bb092bab88baa3138e7d4401f72d))
    - Merge remote-tracking branch 'origin/main' into bot ([`4f17f43`](https://github.com/ssoudan/sapiens/commit/4f17f438ab2eea6a7f2f6b8cff5fdbec9fbac32e))
    - Deps updated ([`cc8115a`](https://github.com/ssoudan/sapiens/commit/cc8115aed3e0723dbc3158317b7e62c94c6021de))
    - Deps updated ([`7ea6a11`](https://github.com/ssoudan/sapiens/commit/7ea6a11630303aefa30680b17f67d7f45ef08c15))
    - Async TaskProgressUpdateHandler ([`774d5a6`](https://github.com/ssoudan/sapiens/commit/774d5a6c2dbadf934166e2d7e5665b6cf2ac4280))
    - Basic integration with discord ([`f416adf`](https://github.com/ssoudan/sapiens/commit/f416adf7af52b0a907a9db8419bcdaa5f2a77dc5))
    - Basic integration with discord ([`dee80b4`](https://github.com/ssoudan/sapiens/commit/dee80b442c8035b4d2bf17a2683ff2c3c2a9a585))
    - Deps ([`0ac757b`](https://github.com/ssoudan/sapiens/commit/0ac757b5621dd659115c6f1650fbc7915162ce5c))
    - Prototype of a Discord bot ([`c4981fe`](https://github.com/ssoudan/sapiens/commit/c4981fef8e0fa65a866ddff1582d6b4b39eae8c7))
    - Async api for Tools and in most places actually ([`772e8eb`](https://github.com/ssoudan/sapiens/commit/772e8eb4184efd0b305e460a31d719c237790098))
    - More detailed updates while working on a task ([`d497429`](https://github.com/ssoudan/sapiens/commit/d497429d749d9f62a36133e29d1063e8842bd7cf))
    - API changes in deps ([`1a0bdf6`](https://github.com/ssoudan/sapiens/commit/1a0bdf6c3525dfee29211feab7a9700c632a3195))
    - Main loop to process model and tools outputs (part 2) ([`b11b947`](https://github.com/ssoudan/sapiens/commit/b11b947d3b3699807c03c4500a8dc7a0e53d41d0))
    - Main loop to process model and tools outputs ([`15c52d8`](https://github.com/ssoudan/sapiens/commit/15c52d843721fa8426573d9f6bee2c019d2bd9bb))
</details>

## 0.2.2 (2023-04-20)

Limit response size. Include tool name in response.

### Documentation

 - <csr-id-1844702fb1a2ffd5bb1ce4717e19c6675527738a/> Changelog

### New Features

 - <csr-id-ed70724e4133083e44c590cf2f74d27bdef65982/> ToolName with Action result.
 - <csr-id-a1eefedbc23011994f9d7b06d9f5054db489a759/> error on too long response
 - <csr-id-9f251be6efefca6e9219321d4cd56802b6a5ec69/> toolbox assembly conditioned on Cargo features

### Bug Fixes

 - <csr-id-55a0255535ef3ad10e1129de5a10d5ac377f8b30/> Increased max response size

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.2.2, sapiens_derive v0.2.2, sapiens_tools v0.2.2, sapiens_cli v0.2.2 ([`1d9981b`](https://github.com/ssoudan/sapiens/commit/1d9981bef2fe1b4f4441c28e11713470ff1b1f5d))
    - Release sapiens v0.2.2, sapiens_derive v0.2.2, sapiens_tools v0.2.2, sapiens_cli v0.2.2 ([`b72b47f`](https://github.com/ssoudan/sapiens/commit/b72b47f99c52d2d88dc3e2108917103707dc13ba))
    - Changelog ([`1844702`](https://github.com/ssoudan/sapiens/commit/1844702fb1a2ffd5bb1ce4717e19c6675527738a))
    - Increased max response size ([`55a0255`](https://github.com/ssoudan/sapiens/commit/55a0255535ef3ad10e1129de5a10d5ac377f8b30))
    - ToolName with Action result. ([`ed70724`](https://github.com/ssoudan/sapiens/commit/ed70724e4133083e44c590cf2f74d27bdef65982))
    - Error on too long response ([`a1eefed`](https://github.com/ssoudan/sapiens/commit/a1eefedbc23011994f9d7b06d9f5054db489a759))
    - Toolbox assembly conditioned on Cargo features ([`9f251be`](https://github.com/ssoudan/sapiens/commit/9f251be6efefca6e9219321d4cd56802b6a5ec69))
</details>

## 0.2.1 (2023-04-19)

- Added ProtoToolInvoke derive macro
- Hue lights SetStatusTool
- Updated deps to published crates

### Documentation

 - <csr-id-ba8709fa7f21a6d77739437a9f65409016c6b364/> Changelog

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.2.1 ([`6d011b1`](https://github.com/ssoudan/sapiens/commit/6d011b16157847923433b870a6d57d5ad1b73438))
    - Changelog ([`ba8709f`](https://github.com/ssoudan/sapiens/commit/ba8709fa7f21a6d77739437a9f65409016c6b364))
    - [changelog] ([`a36c535`](https://github.com/ssoudan/sapiens/commit/a36c5354033a5d4d59fa5765b3be85d92c0d5556))
    - [deps] updated ([`e9d7a0e`](https://github.com/ssoudan/sapiens/commit/e9d7a0e59e122aa070b5ecb96e16bf53e73144af))
    - [deps] updated ([`eaf3f18`](https://github.com/ssoudan/sapiens/commit/eaf3f18ac20ca49c1146a84e131e90b82a294d4c))
    - [version] ([`43dd12d`](https://github.com/ssoudan/sapiens/commit/43dd12da54faaa1d580ff1e9c793b828592572b1))
    - [deps] ([`83b587d`](https://github.com/ssoudan/sapiens/commit/83b587dccea6e5ef2d9340f1a3ac125369945ae3))
    - [+] cleanup ([`ff4d208`](https://github.com/ssoudan/sapiens/commit/ff4d208b951253b18c3faf5f76f99b891ff41c15))
    - [+] cleanup ([`a53a608`](https://github.com/ssoudan/sapiens/commit/a53a6088db84fe99ad20e74905a59d0505d87feb))
    - [+] renaming ([`f664941`](https://github.com/ssoudan/sapiens/commit/f664941f2aba36cd9bce7493a19d030d2945bd50))
</details>

