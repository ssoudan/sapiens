# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.4.0 (2023-05-02)

### New Features

 - <csr-id-4626deb7308ec642d0e937fc3b96af494538a027/> store the current state in Trace Events
 - <csr-id-0f106f4ee488d2622ded6ff77115608dd8f2b9eb/> scenario with generic tool
 - <csr-id-a35ed6028cdc335a3f2fa0159d71d334d24427c7/> refactoring of the observer for Step
   BREAKING CHANGES: API changed.
 - <csr-id-d8c51f7bb24e7890baaa234e449c862f11e0d604/> ignore messages from bots

### New Features (BREAKING)

 - <csr-id-6c30344483671b542e73e13f51228407f37da63e/> Collect information in a serializable struct with all that matters
 - <csr-id-f93652f7c0886b47ce438a512bf2c13d978b3a6b/> collect execution traces

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 5 calendar days.
 - 5 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #13 from ssoudan/getting_methodical ([`e0d97aa`](https://github.com/ssoudan/sapiens/commit/e0d97aae47b30bd97b37520a345c84b59523de9d))
    - Store the current state in Trace Events ([`4626deb`](https://github.com/ssoudan/sapiens/commit/4626deb7308ec642d0e937fc3b96af494538a027))
    - Scenario with generic tool ([`0f106f4`](https://github.com/ssoudan/sapiens/commit/0f106f4ee488d2622ded6ff77115608dd8f2b9eb))
    - Merge remote-tracking branch 'origin/main' into getting_methodical ([`69ed1e8`](https://github.com/ssoudan/sapiens/commit/69ed1e8c1919d40c5b2362eb4fd8376ae2786e05))
    - Collect information in a serializable struct with all that matters ([`6c30344`](https://github.com/ssoudan/sapiens/commit/6c30344483671b542e73e13f51228407f37da63e))
    - Collect execution traces ([`f93652f`](https://github.com/ssoudan/sapiens/commit/f93652f7c0886b47ce438a512bf2c13d978b3a6b))
    - Refactoring of the observer for Step ([`a35ed60`](https://github.com/ssoudan/sapiens/commit/a35ed6028cdc335a3f2fa0159d71d334d24427c7))
    - Ignore messages from bots ([`d8c51f7`](https://github.com/ssoudan/sapiens/commit/d8c51f7bb24e7890baaa234e449c862f11e0d604))
</details>

## v0.3.0 (2023-04-27)

<csr-id-8f039921a3bed1d976fd04c3d2ca6b99b1142411/>

### Chore

 - <csr-id-8f039921a3bed1d976fd04c3d2ca6b99b1142411/> cleanup

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.5.0, sapiens_tools v0.5.0, sapiens_bot v0.3.0, sapiens_cli v0.4.0, safety bump 3 crates ([`2fc037f`](https://github.com/ssoudan/sapiens/commit/2fc037fdc051a3cc68261fa89c5fd3b08c683441))
    - Merge pull request #10 from ssoudan/parsing ([`568d536`](https://github.com/ssoudan/sapiens/commit/568d5368896f758ae16d84ba35d0a382eec6ef11))
    - Cleanup ([`8f03992`](https://github.com/ssoudan/sapiens/commit/8f039921a3bed1d976fd04c3d2ca6b99b1142411))
</details>

## v0.2.3 (2023-04-26)

### New Features

 - <csr-id-3c9e702c2e0e1624882c83b66c42113bc99b3b5d/> search and logging/tracing
 - <csr-id-87ae158cc01d299f60a05577f9a20516cf65e6c2/> search tool"
 - <csr-id-88fc63572fb1c8ca68ac9f39cd3585c061801e0b/> better looking bot
 - <csr-id-efb82c5b3019b4e855d2e348272c9b57d629cf36/> not as verbose bot output

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens v0.4.1, sapiens_tools v0.4.1, sapiens_bot v0.2.3, sapiens_cli v0.3.1 ([`16160fc`](https://github.com/ssoudan/sapiens/commit/16160fc0d369b76a08ffcc1cd5085d7387178656))
    - Merge pull request #9 from ssoudan/search ([`8976be2`](https://github.com/ssoudan/sapiens/commit/8976be2e7e056ed5a976a37a5723abc9f531d238))
    - Search and logging/tracing ([`3c9e702`](https://github.com/ssoudan/sapiens/commit/3c9e702c2e0e1624882c83b66c42113bc99b3b5d))
    - Search tool" ([`87ae158`](https://github.com/ssoudan/sapiens/commit/87ae158cc01d299f60a05577f9a20516cf65e6c2))
    - Better looking bot ([`88fc635`](https://github.com/ssoudan/sapiens/commit/88fc63572fb1c8ca68ac9f39cd3585c061801e0b))
    - Not as verbose bot output ([`efb82c5`](https://github.com/ssoudan/sapiens/commit/efb82c5b3019b4e855d2e348272c9b57d629cf36))
</details>

## v0.2.2 (2023-04-25)

<csr-id-7ea6a11630303aefa30680b17f67d7f45ef08c15/>
<csr-id-1a6b8972bab61215ebd95f74ebc46be4e63b98e7/>
<csr-id-6e90dcd3947a192c62da6fddf4dcde0342365081/>
<csr-id-35a6a992c57951ebe2a325d81d60c540053bcb94/>
<csr-id-88681e1896275b6bc49c7882eea0c5a05ee8e07b/>

### Chore

 - <csr-id-7ea6a11630303aefa30680b17f67d7f45ef08c15/> deps updated

### Chore

 - <csr-id-1a6b8972bab61215ebd95f74ebc46be4e63b98e7/> CHANGELOGs
 - <csr-id-6e90dcd3947a192c62da6fddf4dcde0342365081/> CHANGELOG
 - <csr-id-35a6a992c57951ebe2a325d81d60c540053bcb94/> update
 - <csr-id-88681e1896275b6bc49c7882eea0c5a05ee8e07b/> CHANGELOG

### New Features

 - <csr-id-8be8d0f3044a1bbfb5fdbc6fb6c7c8ec682f7114/> Discord bot
 - <csr-id-0840086e2f5da2ebcdddec960c5308ca0eacb8c5/> arXiv query api
 - <csr-id-774d5a6c2dbadf934166e2d7e5665b6cf2ac4280/> async TaskProgressUpdateHandler
 - <csr-id-dee80b442c8035b4d2bf17a2683ff2c3c2a9a585/> basic integration with discord
 - <csr-id-c4981fef8e0fa65a866ddff1582d6b4b39eae8c7/> prototype of a Discord bot
 - <csr-id-23383eb67f19e8fdcff185709ca3a6d12b3000fa/> summarize tool

### New Features (BREAKING)

 - <csr-id-772e8eb4184efd0b305e460a31d719c237790098/> async api for Tools and in most places actually

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 15 commits contributed to the release over the course of 4 calendar days.
 - 12 commits were understood as [conventional](https://www.conventionalcommits.org).
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
    - Summarize tool ([`23383eb`](https://github.com/ssoudan/sapiens/commit/23383eb67f19e8fdcff185709ca3a6d12b3000fa))
    - CHANGELOG ([`88681e1`](https://github.com/ssoudan/sapiens/commit/88681e1896275b6bc49c7882eea0c5a05ee8e07b))
    - Release sapiens v0.3.0, sapiens_derive v0.3.0, sapiens_tools v0.3.0, sapiens_bot v0.2.2, sapiens_cli v0.3.0, safety bump 3 crates ([`12e417c`](https://github.com/ssoudan/sapiens/commit/12e417cc394cbd9b7e18366d427cdd89132ee299))
    - Discord bot ([`8be8d0f`](https://github.com/ssoudan/sapiens/commit/8be8d0f3044a1bbfb5fdbc6fb6c7c8ec682f7114))
    - ArXiv query api ([`0840086`](https://github.com/ssoudan/sapiens/commit/0840086e2f5da2ebcdddec960c5308ca0eacb8c5))
    - Deps updated ([`7ea6a11`](https://github.com/ssoudan/sapiens/commit/7ea6a11630303aefa30680b17f67d7f45ef08c15))
    - Async TaskProgressUpdateHandler ([`774d5a6`](https://github.com/ssoudan/sapiens/commit/774d5a6c2dbadf934166e2d7e5665b6cf2ac4280))
    - Basic integration with discord ([`dee80b4`](https://github.com/ssoudan/sapiens/commit/dee80b442c8035b4d2bf17a2683ff2c3c2a9a585))
    - Prototype of a Discord bot ([`c4981fe`](https://github.com/ssoudan/sapiens/commit/c4981fef8e0fa65a866ddff1582d6b4b39eae8c7))
    - Async api for Tools and in most places actually ([`772e8eb`](https://github.com/ssoudan/sapiens/commit/772e8eb4184efd0b305e460a31d719c237790098))
</details>

