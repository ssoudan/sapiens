# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.5.0 (2023-05-05)

<csr-id-98826b19cd97872a032e955478ff2d3b9af8262c/>

### Chore

 - <csr-id-98826b19cd97872a032e955478ff2d3b9af8262c/> updated

### Chore

 - <csr-id-8efe0a225520f14d2c3e0abc7ea8c99578146ca0/> CHANGELOG

### New Features

 - <csr-id-e8ebd2ff31d6179b4a1fe6abaf5bf3d12dce97b7/> allow to point to another endpoint providing an 'OpenAI' API - lm-sys/FastChat for example.

### Bug Fixes

 - <csr-id-d7adc36996f6ff935c4c34d462fc2fdd27474c01/> typo
 - <csr-id-657104d2ea2f0ec1e42468b2793471f2a45bdcf3/> Better phrasing of the task
 - <csr-id-c5997dd9cf24cf3499c5b32f04413be8f603d497/> Scenario 0's state machine!

### New Features (BREAKING)

 - <csr-id-2912f4ff80a8b87c9727d3e05eaae469f7a4fd94/> change in format to improve task completion rate

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 2 calendar days.
 - 2 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - CHANGELOG ([`8efe0a2`](https://github.com/ssoudan/sapiens/commit/8efe0a225520f14d2c3e0abc7ea8c99578146ca0))
    - Merge pull request #15 from ssoudan/getting_better ([`936b298`](https://github.com/ssoudan/sapiens/commit/936b2986bf8cf96b1d731b6e1144b3f3fb271fbe))
    - Updated ([`98826b1`](https://github.com/ssoudan/sapiens/commit/98826b19cd97872a032e955478ff2d3b9af8262c))
    - Change in format to improve task completion rate ([`2912f4f`](https://github.com/ssoudan/sapiens/commit/2912f4ff80a8b87c9727d3e05eaae469f7a4fd94))
    - Allow to point to another endpoint providing an 'OpenAI' API - lm-sys/FastChat for example. ([`e8ebd2f`](https://github.com/ssoudan/sapiens/commit/e8ebd2ff31d6179b4a1fe6abaf5bf3d12dce97b7))
    - Typo ([`d7adc36`](https://github.com/ssoudan/sapiens/commit/d7adc36996f6ff935c4c34d462fc2fdd27474c01))
    - Better phrasing of the task ([`657104d`](https://github.com/ssoudan/sapiens/commit/657104d2ea2f0ec1e42468b2793471f2a45bdcf3))
    - Scenario 0's state machine! ([`c5997dd`](https://github.com/ssoudan/sapiens/commit/c5997dd9cf24cf3499c5b32f04413be8f603d497))
</details>

## v0.4.1 (2023-05-02)

<csr-id-cd0bd17051045dfaa1b821a2c83acad33634721a/>
<csr-id-8d569ea760d79f70a6b99aa096db9185edc25fe8/>
<csr-id-15d27552da86d0d95c5e1a81d3c26dec90a80e90/>
<csr-id-e7eb8309c32d4bc174e4cb51b0f344f336fa8e37/>
<csr-id-5e2e3e7cbd04b6473518790f48e0d1ce80656f72/>
<csr-id-500eeba315a540ff769d6e0278dfdac62ef74761/>
<csr-id-cb65403daee612757dfa64c843a9b85a726f721d/>

### Chore

 - <csr-id-cd0bd17051045dfaa1b821a2c83acad33634721a/> updated
 - <csr-id-8d569ea760d79f70a6b99aa096db9185edc25fe8/> ...
 - <csr-id-15d27552da86d0d95c5e1a81d3c26dec90a80e90/> ...
 - <csr-id-e7eb8309c32d4bc174e4cb51b0f344f336fa8e37/> versions
 - <csr-id-5e2e3e7cbd04b6473518790f48e0d1ce80656f72/> versions
 - <csr-id-500eeba315a540ff769d6e0278dfdac62ef74761/> updated

### Chore

 - <csr-id-cb65403daee612757dfa64c843a9b85a726f721d/> CHANGELOG

### New Features

 - <csr-id-4626deb7308ec642d0e937fc3b96af494538a027/> store the current state in Trace Events
 - <csr-id-6b9c35cfd73343ee79d10a39ebbcb057f0fe1057/> cleaner output format for Trials
 - <csr-id-41f16790e7e7e602f091ff003a7e4086e248c883/> jupter notebook to explore the records from sapiens_exp
 - <csr-id-c6d00560865da9fff220eb0ae506a30672053a27/> added scenario_0
 - <csr-id-0f106f4ee488d2622ded6ff77115608dd8f2b9eb/> scenario with generic tool
 - <csr-id-e883e880f8e41ddadab5cb4b6f546827f02591e1/> GenericTool for building the scenarios
 - <csr-id-a35ed6028cdc335a3f2fa0159d71d334d24427c7/> refactoring of the observer for Step
   BREAKING CHANGES: API changed.
 - <csr-id-7c98fcb78fe6b76ce8a65a60b0f481d3d942fe52/> sapiens_exp

### New Features (BREAKING)

 - <csr-id-04e83c2a214212d045ef5a890a72c3dc5ab61076/> Richer errors while invoking tools
 - <csr-id-6c30344483671b542e73e13f51228407f37da63e/> Collect information in a serializable struct with all that matters
 - <csr-id-f93652f7c0886b47ce438a512bf2c13d978b3a6b/> collect execution traces

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 22 commits contributed to the release over the course of 4 calendar days.
 - 18 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release sapiens_exp v0.4.1 ([`5766e50`](https://github.com/ssoudan/sapiens/commit/5766e50296998fa79d75f1d2af79b74abd7a6c72))
    - CHANGELOG ([`cb65403`](https://github.com/ssoudan/sapiens/commit/cb65403daee612757dfa64c843a9b85a726f721d))
    - Release sapiens v0.6.0, sapiens_derive v0.4.0, sapiens_tools v0.6.0, sapiens_bot v0.4.0, sapiens_cli v0.5.0, sapiens_exp v0.4.1, safety bump 4 crates ([`1b9dd43`](https://github.com/ssoudan/sapiens/commit/1b9dd43e9291f0aef2a83c1610cede57c897a56c))
    - Merge pull request #13 from ssoudan/getting_methodical ([`e0d97aa`](https://github.com/ssoudan/sapiens/commit/e0d97aae47b30bd97b37520a345c84b59523de9d))
    - Store the current state in Trace Events ([`4626deb`](https://github.com/ssoudan/sapiens/commit/4626deb7308ec642d0e937fc3b96af494538a027))
    - Updated ([`cd0bd17`](https://github.com/ssoudan/sapiens/commit/cd0bd17051045dfaa1b821a2c83acad33634721a))
    - Cleaner output format for Trials ([`6b9c35c`](https://github.com/ssoudan/sapiens/commit/6b9c35cfd73343ee79d10a39ebbcb057f0fe1057))
    - Richer errors while invoking tools ([`04e83c2`](https://github.com/ssoudan/sapiens/commit/04e83c2a214212d045ef5a890a72c3dc5ab61076))
    - Wip ([`b6d2dd7`](https://github.com/ssoudan/sapiens/commit/b6d2dd71fa5ed2ff009bea6b1f3113e40f5ae4b3))
    - Jupter notebook to explore the records from sapiens_exp ([`41f1679`](https://github.com/ssoudan/sapiens/commit/41f16790e7e7e602f091ff003a7e4086e248c883))
    - Added scenario_0 ([`c6d0056`](https://github.com/ssoudan/sapiens/commit/c6d00560865da9fff220eb0ae506a30672053a27))
    - Scenario with generic tool ([`0f106f4`](https://github.com/ssoudan/sapiens/commit/0f106f4ee488d2622ded6ff77115608dd8f2b9eb))
    - GenericTool for building the scenarios ([`e883e88`](https://github.com/ssoudan/sapiens/commit/e883e880f8e41ddadab5cb4b6f546827f02591e1))
    - ... ([`8d569ea`](https://github.com/ssoudan/sapiens/commit/8d569ea760d79f70a6b99aa096db9185edc25fe8))
    - ... ([`15d2755`](https://github.com/ssoudan/sapiens/commit/15d27552da86d0d95c5e1a81d3c26dec90a80e90))
    - Collect information in a serializable struct with all that matters ([`6c30344`](https://github.com/ssoudan/sapiens/commit/6c30344483671b542e73e13f51228407f37da63e))
    - Versions ([`e7eb830`](https://github.com/ssoudan/sapiens/commit/e7eb8309c32d4bc174e4cb51b0f344f336fa8e37))
    - Versions ([`5e2e3e7`](https://github.com/ssoudan/sapiens/commit/5e2e3e7cbd04b6473518790f48e0d1ce80656f72))
    - Updated ([`500eeba`](https://github.com/ssoudan/sapiens/commit/500eeba315a540ff769d6e0278dfdac62ef74761))
    - Collect execution traces ([`f93652f`](https://github.com/ssoudan/sapiens/commit/f93652f7c0886b47ce438a512bf2c13d978b3a6b))
    - Refactoring of the observer for Step ([`a35ed60`](https://github.com/ssoudan/sapiens/commit/a35ed6028cdc335a3f2fa0159d71d334d24427c7))
    - Sapiens_exp ([`7c98fcb`](https://github.com/ssoudan/sapiens/commit/7c98fcb78fe6b76ce8a65a60b0f481d3d942fe52))
</details>

