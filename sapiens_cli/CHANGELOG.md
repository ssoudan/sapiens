# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.0 (2023-04-24)

<csr-id-14529922572878248a9f6681dfa716e87326d8ff/>
<csr-id-15c52d843721fa8426573d9f6bee2c019d2bd9bb/>
<csr-id-b11b947d3b3699807c03c4500a8dc7a0e53d41d0/>

### Chore

 - <csr-id-14529922572878248a9f6681dfa716e87326d8ff/> release prep

### New Features

 - <csr-id-8be8d0f3044a1bbfb5fdbc6fb6c7c8ec682f7114/> Discord bot
 - <csr-id-0840086e2f5da2ebcdddec960c5308ca0eacb8c5/> arXiv query api
 - <csr-id-774d5a6c2dbadf934166e2d7e5665b6cf2ac4280/> async TaskProgressUpdateHandler
 - <csr-id-dee80b442c8035b4d2bf17a2683ff2c3c2a9a585/> basic integration with discord
 - <csr-id-d497429d749d9f62a36133e29d1063e8842bd7cf/> more detailed updates while working on a task

### Bug Fixes

 - <csr-id-c37e7ca813eaff1fe87b11736670024b81c5088d/> sorted tool descriptions (by name)

### Refactor

 - <csr-id-15c52d843721fa8426573d9f6bee2c019d2bd9bb/> main loop to process model and tools outputs

### New Features (BREAKING)

 - <csr-id-772e8eb4184efd0b305e460a31d719c237790098/> async api for Tools and in most places actually

### Refactor (BREAKING)

 - <csr-id-b11b947d3b3699807c03c4500a8dc7a0e53d41d0/> main loop to process model and tools outputs (part 2)

## 0.2.2 (2023-04-20)

<csr-id-a346ece7e9f72c907986f3daa924a3a51ab69f1f/>

More Tools!

### Documentation

 - <csr-id-1844702fb1a2ffd5bb1ce4717e19c6675527738a/> Changelog

### Chore

 - <csr-id-a346ece7e9f72c907986f3daa924a3a51ab69f1f/> deps

### New Features

 - <csr-id-3e15ff7b615faaab87addf4aff26ae841d94b4dc/> build container without Hue support by default - EXTRA_FEATURE="hue" to enable.
 - <csr-id-ed70724e4133083e44c590cf2f74d27bdef65982/> ToolName with Action result.
 - <csr-id-b0400529f9bacd56466a9104549d4f6eea7f3ccf/> Added WikipediaTool
 - <csr-id-b2902bba5640ed0802af77eff2e628d56992760b/> Added WikidataTool
 - <csr-id-9f251be6efefca6e9219321d4cd56802b6a5ec69/> toolbox assembly conditioned on Cargo features

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.2.2, sapiens_derive v0.2.2, sapiens_tools v0.2.2, sapiens_cli v0.2.2 ([`1d9981b`](https://github.com/ssoudan/sapiens/commit/1d9981bef2fe1b4f4441c28e11713470ff1b1f5d))
    - Release sapiens v0.2.2, sapiens_derive v0.2.2, sapiens_tools v0.2.2, sapiens_cli v0.2.2 ([`b72b47f`](https://github.com/ssoudan/sapiens/commit/b72b47f99c52d2d88dc3e2108917103707dc13ba))
    - Changelog ([`1844702`](https://github.com/ssoudan/sapiens/commit/1844702fb1a2ffd5bb1ce4717e19c6675527738a))
    - Deps ([`a346ece`](https://github.com/ssoudan/sapiens/commit/a346ece7e9f72c907986f3daa924a3a51ab69f1f))
    - Build container without Hue support by default - EXTRA_FEATURE="hue" to enable. ([`3e15ff7`](https://github.com/ssoudan/sapiens/commit/3e15ff7b615faaab87addf4aff26ae841d94b4dc))
    - ToolName with Action result. ([`ed70724`](https://github.com/ssoudan/sapiens/commit/ed70724e4133083e44c590cf2f74d27bdef65982))
    - Added WikipediaTool ([`b040052`](https://github.com/ssoudan/sapiens/commit/b0400529f9bacd56466a9104549d4f6eea7f3ccf))
    - Added WikidataTool ([`b2902bb`](https://github.com/ssoudan/sapiens/commit/b2902bba5640ed0802af77eff2e628d56992760b))
    - Toolbox assembly conditioned on Cargo features ([`9f251be`](https://github.com/ssoudan/sapiens/commit/9f251be6efefca6e9219321d4cd56802b6a5ec69))
</details>

## v0.2.1 (2023-04-19)

### Documentation

 - <csr-id-2dc34812fa3afc6147fcd3f3b0bc5311b841ab9f/> Changelog

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens_derive v0.2.1, sapiens_tools v0.2.1, sapiens_cli v0.2.1 ([`e56d735`](https://github.com/ssoudan/sapiens/commit/e56d735f014fed54461dfcc64b96e5d801f995e6))
    - Changelog ([`2dc3481`](https://github.com/ssoudan/sapiens/commit/2dc34812fa3afc6147fcd3f3b0bc5311b841ab9f))
    - Adjusting changelogs prior to release of sapiens_derive v0.2.1, sapiens_tools v0.2.1, sapiens_cli v0.2.1 ([`3ea2039`](https://github.com/ssoudan/sapiens/commit/3ea2039b68192d4de6b9d370db54abcae054e3cc))
    - Release sapiens v0.2.1 ([`6d011b1`](https://github.com/ssoudan/sapiens/commit/6d011b16157847923433b870a6d57d5ad1b73438))
    - [deps] updated ([`e9d7a0e`](https://github.com/ssoudan/sapiens/commit/e9d7a0e59e122aa070b5ecb96e16bf53e73144af))
    - [+] SetStatusTool to control Hue lights. ([`e464518`](https://github.com/ssoudan/sapiens/commit/e4645184d43e99c7d90d7bc5ca91b43e3a034c8f))
    - [deps] updated ([`eaf3f18`](https://github.com/ssoudan/sapiens/commit/eaf3f18ac20ca49c1146a84e131e90b82a294d4c))
    - [+] rely on a published version of huelib-rs (named huelib2-rs). ([`cb91e27`](https://github.com/ssoudan/sapiens/commit/cb91e2795e5803f4c8ce8c41ed5605a006e83b15))
    - Release 0.2.0 ([`ab53b6c`](https://github.com/ssoudan/sapiens/commit/ab53b6c999892d82fbd9aed827a3a3bc1aee24a4))
    - [version] ([`43dd12d`](https://github.com/ssoudan/sapiens/commit/43dd12da54faaa1d580ff1e9c793b828592572b1))
    - [cleanup] ([`0a69fed`](https://github.com/ssoudan/sapiens/commit/0a69fedd83f039317a7fcd26819d01408f2d6c97))
    - [+] renaming ([`f664941`](https://github.com/ssoudan/sapiens/commit/f664941f2aba36cd9bce7493a19d030d2945bd50))
</details>

