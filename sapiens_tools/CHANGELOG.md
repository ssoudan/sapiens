# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Chore

 - <csr-id-35a6a992c57951ebe2a325d81d60c540053bcb94/> update

### New Features

 - <csr-id-3744b79d5b49d205842d041031a0a69ccef50330/> improved format description
 - <csr-id-a08cd2093594b57d54ab5a85b0cd9a1ba83aac2c/> reduce initial prompt size
 - <csr-id-23383eb67f19e8fdcff185709ca3a6d12b3000fa/> summarize tool

### Bug Fixes

 - <csr-id-f62eb17dff74090e0f1def6119895b571c7f4835/> eliminate `import tools` and `from tools import...`
 - <csr-id-b1521356547d673c5695fd69558901f4dba2f8ae/> deps
 - <csr-id-d79778b8b215e2ae5fdc9a1e40913fa3d1d711cf/> filtering of import tools and from tools import
 - <csr-id-ee0c2df176086df1287249abc60f2a5003fc583e/> eliminate `import tools` from the code

### Bug Fixes (BREAKING)

 - <csr-id-11de03cd0c11100c654eb171c1f0ac03e561661a/> renamed Arxiv tool

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release.
 - 1 day passed between releases.
 - 9 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update ([`35a6a99`](https://github.com/ssoudan/sapiens/commit/35a6a992c57951ebe2a325d81d60c540053bcb94))
    - Eliminate `import tools` and `from tools import...` ([`f62eb17`](https://github.com/ssoudan/sapiens/commit/f62eb17dff74090e0f1def6119895b571c7f4835))
    - Improved format description ([`3744b79`](https://github.com/ssoudan/sapiens/commit/3744b79d5b49d205842d041031a0a69ccef50330))
    - Reduce initial prompt size ([`a08cd20`](https://github.com/ssoudan/sapiens/commit/a08cd2093594b57d54ab5a85b0cd9a1ba83aac2c))
    - Deps ([`b152135`](https://github.com/ssoudan/sapiens/commit/b1521356547d673c5695fd69558901f4dba2f8ae))
    - Filtering of import tools and from tools import ([`d79778b`](https://github.com/ssoudan/sapiens/commit/d79778b8b215e2ae5fdc9a1e40913fa3d1d711cf))
    - Renamed Arxiv tool ([`11de03c`](https://github.com/ssoudan/sapiens/commit/11de03cd0c11100c654eb171c1f0ac03e561661a))
    - Eliminate `import tools` from the code ([`ee0c2df`](https://github.com/ssoudan/sapiens/commit/ee0c2df176086df1287249abc60f2a5003fc583e))
    - Summarize tool ([`23383eb`](https://github.com/ssoudan/sapiens/commit/23383eb67f19e8fdcff185709ca3a6d12b3000fa))
</details>

## 0.3.0 (2023-04-24)

<csr-id-14529922572878248a9f6681dfa716e87326d8ff/>
<csr-id-7ea6a11630303aefa30680b17f67d7f45ef08c15/>
<csr-id-385aab3fb6880373a7970cd5e243e68d127dc72c/>
<csr-id-15c52d843721fa8426573d9f6bee2c019d2bd9bb/>
<csr-id-b11b947d3b3699807c03c4500a8dc7a0e53d41d0/>

### Chore

 - <csr-id-14529922572878248a9f6681dfa716e87326d8ff/> release prep
 - <csr-id-7ea6a11630303aefa30680b17f67d7f45ef08c15/> deps updated
 - <csr-id-385aab3fb6880373a7970cd5e243e68d127dc72c/> cleanup

### New Features

 - <csr-id-8be8d0f3044a1bbfb5fdbc6fb6c7c8ec682f7114/> Discord bot
 - <csr-id-0840086e2f5da2ebcdddec960c5308ca0eacb8c5/> arXiv query api
 - <csr-id-f416adf7af52b0a907a9db8419bcdaa5f2a77dc5/> basic integration with discord

### Bug Fixes

 - <csr-id-b926c83e28c3e73484ce304f217a367b565b7f93/> wikipedia input example
 - <csr-id-04d17622f7afea618b5f7693d3b6e7754f29a9c0/> tests snapshot
 - <csr-id-97d7c704a78aa23661fd9b24267667e7dc5f40bf/> linefeed in tool descriptions
 - <csr-id-56ea40e1cc483bf22ca2eaea7f333fad2b001e48/> Better definition of the format for RoomToolInput
 - <csr-id-e1953300aac0607b8c3eabf956f06b31317a338b/> udeps config

### Refactor

 - <csr-id-15c52d843721fa8426573d9f6bee2c019d2bd9bb/> main loop to process model and tools outputs

### New Features (BREAKING)

 - <csr-id-772e8eb4184efd0b305e460a31d719c237790098/> async api for Tools and in most places actually

### Refactor (BREAKING)

 - <csr-id-b11b947d3b3699807c03c4500a8dc7a0e53d41d0/> main loop to process model and tools outputs (part 2)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release over the course of 4 calendar days.
 - 4 days passed between releases.
 - 14 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.3.0, sapiens_derive v0.3.0, sapiens_tools v0.3.0, sapiens_bot v0.2.2, sapiens_cli v0.3.0, safety bump 3 crates ([`12e417c`](https://github.com/ssoudan/sapiens/commit/12e417cc394cbd9b7e18366d427cdd89132ee299))
    - Discord bot ([`8be8d0f`](https://github.com/ssoudan/sapiens/commit/8be8d0f3044a1bbfb5fdbc6fb6c7c8ec682f7114))
    - Release prep ([`1452992`](https://github.com/ssoudan/sapiens/commit/14529922572878248a9f6681dfa716e87326d8ff))
    - ArXiv query api ([`0840086`](https://github.com/ssoudan/sapiens/commit/0840086e2f5da2ebcdddec960c5308ca0eacb8c5))
    - Wikipedia input example ([`b926c83`](https://github.com/ssoudan/sapiens/commit/b926c83e28c3e73484ce304f217a367b565b7f93))
    - Tests snapshot ([`04d1762`](https://github.com/ssoudan/sapiens/commit/04d17622f7afea618b5f7693d3b6e7754f29a9c0))
    - Linefeed in tool descriptions ([`97d7c70`](https://github.com/ssoudan/sapiens/commit/97d7c704a78aa23661fd9b24267667e7dc5f40bf))
    - Merge remote-tracking branch 'origin/main' into bot ([`4f17f43`](https://github.com/ssoudan/sapiens/commit/4f17f438ab2eea6a7f2f6b8cff5fdbec9fbac32e))
    - Better definition of the format for RoomToolInput ([`56ea40e`](https://github.com/ssoudan/sapiens/commit/56ea40e1cc483bf22ca2eaea7f333fad2b001e48))
    - Deps updated ([`7ea6a11`](https://github.com/ssoudan/sapiens/commit/7ea6a11630303aefa30680b17f67d7f45ef08c15))
    - Basic integration with discord ([`f416adf`](https://github.com/ssoudan/sapiens/commit/f416adf7af52b0a907a9db8419bcdaa5f2a77dc5))
    - Udeps config ([`e195330`](https://github.com/ssoudan/sapiens/commit/e1953300aac0607b8c3eabf956f06b31317a338b))
    - Async api for Tools and in most places actually ([`772e8eb`](https://github.com/ssoudan/sapiens/commit/772e8eb4184efd0b305e460a31d719c237790098))
    - Main loop to process model and tools outputs (part 2) ([`b11b947`](https://github.com/ssoudan/sapiens/commit/b11b947d3b3699807c03c4500a8dc7a0e53d41d0))
    - Main loop to process model and tools outputs ([`15c52d8`](https://github.com/ssoudan/sapiens/commit/15c52d843721fa8426573d9f6bee2c019d2bd9bb))
    - Cleanup ([`385aab3`](https://github.com/ssoudan/sapiens/commit/385aab3fb6880373a7970cd5e243e68d127dc72c))
</details>

## 0.2.2 (2023-04-20)

<csr-id-41bca7d7b24a7a7c27e8358d2a1ea3c5b5f786ed/>

- More tools!

### Documentation

 - <csr-id-1844702fb1a2ffd5bb1ce4717e19c6675527738a/> Changelog

### Chore

 - <csr-id-41bca7d7b24a7a7c27e8358d2a1ea3c5b5f786ed/> cleanup

### New Features

 - <csr-id-3e15ff7b615faaab87addf4aff26ae841d94b4dc/> build container without Hue support by default - EXTRA_FEATURE="hue" to enable.
 - <csr-id-49e0a6ede8fb84382e9e32ccfe21a05a62818187/> format of tools.list() output.
 - <csr-id-ed70724e4133083e44c590cf2f74d27bdef65982/> ToolName with Action result.
 - <csr-id-90c5cbd9d8ee6d52e87522c420c1bfde849e28b9/> Added WikipediaTool
 - <csr-id-b0400529f9bacd56466a9104549d4f6eea7f3ccf/> Added WikipediaTool
 - <csr-id-b2902bba5640ed0802af77eff2e628d56992760b/> Added WikidataTool
 - <csr-id-9f251be6efefca6e9219321d4cd56802b6a5ec69/> toolbox assembly conditioned on Cargo features

### Bug Fixes

 - <csr-id-d3ffde8c5a9fc3b3714239c645f078c53af8224e/> Support multivalued parameters

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release.
 - 10 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.2.2, sapiens_derive v0.2.2, sapiens_tools v0.2.2, sapiens_cli v0.2.2 ([`1d9981b`](https://github.com/ssoudan/sapiens/commit/1d9981bef2fe1b4f4441c28e11713470ff1b1f5d))
    - Release sapiens v0.2.2, sapiens_derive v0.2.2, sapiens_tools v0.2.2, sapiens_cli v0.2.2 ([`b72b47f`](https://github.com/ssoudan/sapiens/commit/b72b47f99c52d2d88dc3e2108917103707dc13ba))
    - Changelog ([`1844702`](https://github.com/ssoudan/sapiens/commit/1844702fb1a2ffd5bb1ce4717e19c6675527738a))
    - Build container without Hue support by default - EXTRA_FEATURE="hue" to enable. ([`3e15ff7`](https://github.com/ssoudan/sapiens/commit/3e15ff7b615faaab87addf4aff26ae841d94b4dc))
    - Format of tools.list() output. ([`49e0a6e`](https://github.com/ssoudan/sapiens/commit/49e0a6ede8fb84382e9e32ccfe21a05a62818187))
    - ToolName with Action result. ([`ed70724`](https://github.com/ssoudan/sapiens/commit/ed70724e4133083e44c590cf2f74d27bdef65982))
    - Support multivalued parameters ([`d3ffde8`](https://github.com/ssoudan/sapiens/commit/d3ffde8c5a9fc3b3714239c645f078c53af8224e))
    - Added WikipediaTool ([`90c5cbd`](https://github.com/ssoudan/sapiens/commit/90c5cbd9d8ee6d52e87522c420c1bfde849e28b9))
    - Cleanup ([`41bca7d`](https://github.com/ssoudan/sapiens/commit/41bca7d7b24a7a7c27e8358d2a1ea3c5b5f786ed))
    - Added WikipediaTool ([`b040052`](https://github.com/ssoudan/sapiens/commit/b0400529f9bacd56466a9104549d4f6eea7f3ccf))
    - Added WikidataTool ([`b2902bb`](https://github.com/ssoudan/sapiens/commit/b2902bba5640ed0802af77eff2e628d56992760b))
    - Toolbox assembly conditioned on Cargo features ([`9f251be`](https://github.com/ssoudan/sapiens/commit/9f251be6efefca6e9219321d4cd56802b6a5ec69))
</details>

## v0.2.1 (2023-04-19)

### Documentation

 - <csr-id-2dc34812fa3afc6147fcd3f3b0bc5311b841ab9f/> Changelog

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release.
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
    - [+] SetStatusTool to control Hue lights. ([`e464518`](https://github.com/ssoudan/sapiens/commit/e4645184d43e99c7d90d7bc5ca91b43e3a034c8f))
    - [+] Derive(ProtoToolInvoke) ([`e383e20`](https://github.com/ssoudan/sapiens/commit/e383e2017123d2c2eee8a39ff34f50024001367a))
    - [+] rely on a published version of huelib-rs (named huelib2-rs). ([`cb91e27`](https://github.com/ssoudan/sapiens/commit/cb91e2795e5803f4c8ce8c41ed5605a006e83b15))
    - Release 0.2.0 ([`ab53b6c`](https://github.com/ssoudan/sapiens/commit/ab53b6c999892d82fbd9aed827a3a3bc1aee24a4))
    - [version] ([`43dd12d`](https://github.com/ssoudan/sapiens/commit/43dd12da54faaa1d580ff1e9c793b828592572b1))
    - [+] renaming ([`f664941`](https://github.com/ssoudan/sapiens/commit/f664941f2aba36cd9bce7493a19d030d2945bd50))
</details>

