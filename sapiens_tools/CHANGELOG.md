# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.9.0 (2023-05-17)

### Chore

 - <csr-id-099c611a319519c44b62274627e35b01746ce37e/> bump serde from 1.0.162 to 1.0.163
   Bumps [serde](https://github.com/serde-rs/serde) from 1.0.162 to 1.0.163.
   - [Release notes](https://github.com/serde-rs/serde/releases)
   - [Commits](https://github.com/serde-rs/serde/compare/v1.0.162...v1.0.163)
   
   ---
   updated-dependencies:
   - dependency-name: serde
     dependency-type: direct:production
     update-type: version-update:semver-patch
   ...

### New Features

 - <csr-id-34bf58ba79af2aaa84ab3e79ba565d73ae2f8266/> allow dependabot-triggered CI to complete successfully - take 2
 - <csr-id-2bff729d9dc84e9d04619ea082fdbc071e2d45d9/> allow dependabot-triggered CI to complete successfully

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 6 calendar days.
 - 6 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #19 from ssoudan/dependabot/cargo/serde-1.0.163 ([`d6497e5`](https://github.com/ssoudan/sapiens/commit/d6497e564aa66cfa80500b8f91bc5e0b4447f689))
    - Bump serde from 1.0.162 to 1.0.163 ([`099c611`](https://github.com/ssoudan/sapiens/commit/099c611a319519c44b62274627e35b01746ce37e))
    - Allow dependabot-triggered CI to complete successfully - take 2 ([`34bf58b`](https://github.com/ssoudan/sapiens/commit/34bf58ba79af2aaa84ab3e79ba565d73ae2f8266))
    - Merge pull request #20 from ssoudan/for_dependabot ([`accf76c`](https://github.com/ssoudan/sapiens/commit/accf76c3e9c154ca75fe8ac3b550387a3e11b991))
    - Allow dependabot-triggered CI to complete successfully ([`2bff729`](https://github.com/ssoudan/sapiens/commit/2bff729d9dc84e9d04619ea082fdbc071e2d45d9))
</details>

## 0.8.0 (2023-05-11)

<csr-id-37fe5c57cb17dabf56fa61e8dcad8c5585846750/>
<csr-id-c46990fe7b630ecc1e2698da5c168508da47fe34/>
<csr-id-25661955e8aba7f9dee4a16e046c621c5ffd3fca/>

### Chore

 - <csr-id-37fe5c57cb17dabf56fa61e8dcad8c5585846750/> update
 - <csr-id-c46990fe7b630ecc1e2698da5c168508da47fe34/> renaming of fields to match the new visible names (`tool_name`, `parameters`)
 - <csr-id-25661955e8aba7f9dee4a16e046c621c5ffd3fca/> badges, docs.rs links, ...

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 5 calendar days.
 - 5 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.8.0, sapiens_derive v0.4.2, sapiens_tools v0.8.0, sapiens_bot v0.6.0, sapiens_cli v0.7.0, sapiens_exp v0.6.0, safety bump 4 crates ([`a18acd9`](https://github.com/ssoudan/sapiens/commit/a18acd9218045421957aff1c73c9f0a1597eb8d5))
    - Update ([`37fe5c5`](https://github.com/ssoudan/sapiens/commit/37fe5c57cb17dabf56fa61e8dcad8c5585846750))
    - Merge pull request #16 from ssoudan/decoupling ([`981d4d4`](https://github.com/ssoudan/sapiens/commit/981d4d4e1e6b625e17d7dcc2fc94e0709e11267c))
    - Renaming of fields to match the new visible names (`tool_name`, `parameters`) ([`c46990f`](https://github.com/ssoudan/sapiens/commit/c46990fe7b630ecc1e2698da5c168508da47fe34))
    - Badges, docs.rs links, ... ([`2566195`](https://github.com/ssoudan/sapiens/commit/25661955e8aba7f9dee4a16e046c621c5ffd3fca))
</details>

## 0.7.0 (2023-05-05)

<csr-id-98826b19cd97872a032e955478ff2d3b9af8262c/>
<csr-id-acb2a6a46192a116b17cacb9301cba22a7b3e719/>
<csr-id-8efe0a225520f14d2c3e0abc7ea8c99578146ca0/>

### Chore

 - <csr-id-98826b19cd97872a032e955478ff2d3b9af8262c/> updated
 - <csr-id-acb2a6a46192a116b17cacb9301cba22a7b3e719/> cleanup

### Chore

 - <csr-id-8efe0a225520f14d2c3e0abc7ea8c99578146ca0/> CHANGELOG

### New Features

 - <csr-id-45f5286228d2a8d42ebf89b6ea1f14a6eeb53f52/> cover the case with `exit` in Python code
 - <csr-id-ff4d730e62c2cd4c370a83fe1ab0a74325389a84/> release GIL when doing something else

### New Features (BREAKING)

 - <csr-id-2912f4ff80a8b87c9727d3e05eaae469f7a4fd94/> change in format to improve task completion rate

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.7.0, sapiens_derive v0.4.1, sapiens_tools v0.7.0, sapiens_bot v0.5.0, sapiens_cli v0.6.0, sapiens_exp v0.5.0, safety bump 4 crates ([`3b2c461`](https://github.com/ssoudan/sapiens/commit/3b2c461f7273b55660f37ed73b0a10b88e0f788b))
    - CHANGELOG ([`8efe0a2`](https://github.com/ssoudan/sapiens/commit/8efe0a225520f14d2c3e0abc7ea8c99578146ca0))
    - Merge pull request #15 from ssoudan/getting_better ([`936b298`](https://github.com/ssoudan/sapiens/commit/936b2986bf8cf96b1d731b6e1144b3f3fb271fbe))
    - Updated ([`98826b1`](https://github.com/ssoudan/sapiens/commit/98826b19cd97872a032e955478ff2d3b9af8262c))
    - Cleanup ([`acb2a6a`](https://github.com/ssoudan/sapiens/commit/acb2a6a46192a116b17cacb9301cba22a7b3e719))
    - Change in format to improve task completion rate ([`2912f4f`](https://github.com/ssoudan/sapiens/commit/2912f4ff80a8b87c9727d3e05eaae469f7a4fd94))
    - Cover the case with `exit` in Python code ([`45f5286`](https://github.com/ssoudan/sapiens/commit/45f5286228d2a8d42ebf89b6ea1f14a6eeb53f52))
    - Release GIL when doing something else ([`ff4d730`](https://github.com/ssoudan/sapiens/commit/ff4d730e62c2cd4c370a83fe1ab0a74325389a84))
</details>

## 0.6.0 (2023-05-02)

### New Features

 - <csr-id-4626deb7308ec642d0e937fc3b96af494538a027/> store the current state in Trace Events
 - <csr-id-c6d00560865da9fff220eb0ae506a30672053a27/> added scenario_0
 - <csr-id-404afb184a3fe0daedc3103ad9be0cefd4c4a890/> generate Python docstring so `help(tools.Something)` works.
 - <csr-id-a35ed6028cdc335a3f2fa0159d71d334d24427c7/> refactoring of the observer for Step
   BREAKING CHANGES: API changed.
 - <csr-id-7c98fcb78fe6b76ce8a65a60b0f481d3d942fe52/> sapiens_exp

### New Features (BREAKING)

 - <csr-id-04e83c2a214212d045ef5a890a72c3dc5ab61076/> Richer errors while invoking tools
 - <csr-id-6c30344483671b542e73e13f51228407f37da63e/> Collect information in a serializable struct with all that matters
 - <csr-id-f93652f7c0886b47ce438a512bf2c13d978b3a6b/> collect execution traces

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 4 calendar days.
 - 5 days passed between releases.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.6.0, sapiens_derive v0.4.0, sapiens_tools v0.6.0, sapiens_bot v0.4.0, sapiens_cli v0.5.0, sapiens_exp v0.4.1, safety bump 4 crates ([`1b9dd43`](https://github.com/ssoudan/sapiens/commit/1b9dd43e9291f0aef2a83c1610cede57c897a56c))
    - Merge pull request #13 from ssoudan/getting_methodical ([`e0d97aa`](https://github.com/ssoudan/sapiens/commit/e0d97aae47b30bd97b37520a345c84b59523de9d))
    - Store the current state in Trace Events ([`4626deb`](https://github.com/ssoudan/sapiens/commit/4626deb7308ec642d0e937fc3b96af494538a027))
    - Richer errors while invoking tools ([`04e83c2`](https://github.com/ssoudan/sapiens/commit/04e83c2a214212d045ef5a890a72c3dc5ab61076))
    - Added scenario_0 ([`c6d0056`](https://github.com/ssoudan/sapiens/commit/c6d00560865da9fff220eb0ae506a30672053a27))
    - Merge remote-tracking branch 'origin/main' into getting_methodical ([`69ed1e8`](https://github.com/ssoudan/sapiens/commit/69ed1e8c1919d40c5b2362eb4fd8376ae2786e05))
    - Collect information in a serializable struct with all that matters ([`6c30344`](https://github.com/ssoudan/sapiens/commit/6c30344483671b542e73e13f51228407f37da63e))
    - Collect execution traces ([`f93652f`](https://github.com/ssoudan/sapiens/commit/f93652f7c0886b47ce438a512bf2c13d978b3a6b))
    - Generate Python docstring so `help(tools.Something)` works. ([`404afb1`](https://github.com/ssoudan/sapiens/commit/404afb184a3fe0daedc3103ad9be0cefd4c4a890))
    - Refactoring of the observer for Step ([`a35ed60`](https://github.com/ssoudan/sapiens/commit/a35ed6028cdc335a3f2fa0159d71d334d24427c7))
    - Sapiens_exp ([`7c98fcb`](https://github.com/ssoudan/sapiens/commit/7c98fcb78fe6b76ce8a65a60b0f481d3d942fe52))
</details>

## 0.5.0 (2023-04-27)

<csr-id-8f039921a3bed1d976fd04c3d2ca6b99b1142411/>

### Chore

 - <csr-id-8f039921a3bed1d976fd04c3d2ca6b99b1142411/> cleanup

### New Features (BREAKING)

 - <csr-id-5d785d779955f5a4c2f54b1ff60e2262e85bfa05/> more flexible parsing

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.5.0, sapiens_tools v0.5.0, sapiens_bot v0.3.0, sapiens_cli v0.4.0, safety bump 3 crates ([`2fc037f`](https://github.com/ssoudan/sapiens/commit/2fc037fdc051a3cc68261fa89c5fd3b08c683441))
    - Merge pull request #10 from ssoudan/parsing ([`568d536`](https://github.com/ssoudan/sapiens/commit/568d5368896f758ae16d84ba35d0a382eec6ef11))
    - Cleanup ([`8f03992`](https://github.com/ssoudan/sapiens/commit/8f039921a3bed1d976fd04c3d2ca6b99b1142411))
    - More flexible parsing ([`5d785d7`](https://github.com/ssoudan/sapiens/commit/5d785d779955f5a4c2f54b1ff60e2262e85bfa05))
</details>

## 0.4.1 (2023-04-26)

### New Features

 - <csr-id-acb725e1d421844815e2f2a8f8156ed4aa523849/> dotenvy in tests
 - <csr-id-cd14da74329603501a13ca89ca2700b6eca92af7/> custom Debug
 - <csr-id-3c9e702c2e0e1624882c83b66c42113bc99b3b5d/> search and logging/tracing
 - <csr-id-87ae158cc01d299f60a05577f9a20516cf65e6c2/> search tool"
 - <csr-id-f67aea1870f102c146005c7fd381b0299b02a84f/> update tool calling convention description

### Bug Fixes

 - <csr-id-27059b5520a417af8eda46880c9c9bf5e27757c9/> more dotenvy

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.4.1, sapiens_tools v0.4.1, sapiens_bot v0.2.3, sapiens_cli v0.3.1 ([`16160fc`](https://github.com/ssoudan/sapiens/commit/16160fc0d369b76a08ffcc1cd5085d7387178656))
    - Merge pull request #9 from ssoudan/search ([`8976be2`](https://github.com/ssoudan/sapiens/commit/8976be2e7e056ed5a976a37a5723abc9f531d238))
    - More dotenvy ([`27059b5`](https://github.com/ssoudan/sapiens/commit/27059b5520a417af8eda46880c9c9bf5e27757c9))
    - Dotenvy in tests ([`acb725e`](https://github.com/ssoudan/sapiens/commit/acb725e1d421844815e2f2a8f8156ed4aa523849))
    - Custom Debug ([`cd14da7`](https://github.com/ssoudan/sapiens/commit/cd14da74329603501a13ca89ca2700b6eca92af7))
    - Search and logging/tracing ([`3c9e702`](https://github.com/ssoudan/sapiens/commit/3c9e702c2e0e1624882c83b66c42113bc99b3b5d))
    - Search tool" ([`87ae158`](https://github.com/ssoudan/sapiens/commit/87ae158cc01d299f60a05577f9a20516cf65e6c2))
    - Update tool calling convention description ([`f67aea1`](https://github.com/ssoudan/sapiens/commit/f67aea1870f102c146005c7fd381b0299b02a84f))
</details>

## 0.4.0 (2023-04-25)

<csr-id-1a6b8972bab61215ebd95f74ebc46be4e63b98e7/>
<csr-id-6e90dcd3947a192c62da6fddf4dcde0342365081/>
<csr-id-35a6a992c57951ebe2a325d81d60c540053bcb94/>

### Chore

 - <csr-id-1a6b8972bab61215ebd95f74ebc46be4e63b98e7/> CHANGELOGs
 - <csr-id-6e90dcd3947a192c62da6fddf4dcde0342365081/> CHANGELOG
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

 - 13 commits contributed to the release over the course of 1 calendar day.
 - 1 day passed between releases.
 - 11 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.4.0, sapiens_derive v0.3.1, sapiens_tools v0.4.0, sapiens_bot v0.2.2, sapiens_cli v0.3.0, safety bump 3 crates ([`0da51f4`](https://github.com/ssoudan/sapiens/commit/0da51f431c56f43088c651b0428c3f2fc6be11be))
    - CHANGELOGs ([`1a6b897`](https://github.com/ssoudan/sapiens/commit/1a6b8972bab61215ebd95f74ebc46be4e63b98e7))
    - Merge pull request #8 from ssoudan/tools ([`3b7f2b2`](https://github.com/ssoudan/sapiens/commit/3b7f2b25afb545217e9d0c078b372e1a24981d78))
    - CHANGELOG ([`6e90dcd`](https://github.com/ssoudan/sapiens/commit/6e90dcd3947a192c62da6fddf4dcde0342365081))
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

