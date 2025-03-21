[![animation](https://raw.githubusercontent.com/orhun/git-cliff/main/website/static/img/git-cliff-anim.gif)](https://git-cliff.org)

## [unreleased]

### 🚜 Refactor

- *(shellexpand)* Use directories_next for getting user HOME - ([18f73de](https://github.com/dablenparty/boxunbox/commit/18f73decf9de8dd9cb456509b98fa6930e476429))

### 📚 Documentation

- *(shellexpand)* Document envvar syntax on expand function - ([c05816d](https://github.com/dablenparty/boxunbox/commit/c05816d411325cb7694ff53958cb3274094b37f1))

### Build

- Optimize release builds for binary size - ([4cfea66](https://github.com/dablenparty/boxunbox/commit/4cfea66c9967d813753b500dd81817b5ee4c540b))


## [0.1.16](https://github.com/dablenparty/boxunbox/compare/v0.1.15..v0.1.16) - 2025-03-21

### ⛰️  Features

- *(shellexpand)* Implement tilde (~) expansion - ([ebe113e](https://github.com/dablenparty/boxunbox/commit/ebe113e46337015f62443ef930f1b6a6bc4b576d))
- *(shellexpand)* Implement envvar expansion - ([64d5e9e](https://github.com/dablenparty/boxunbox/commit/64d5e9e43a7e26fda7c8c332b4fc77ea0460efc2))
- [**breaking**] Replace shellexpand with custom_shellexpand - ([84df8b6](https://github.com/dablenparty/boxunbox/commit/84df8b62dc4dc13c695a7600519a2fa3ed322d14))

### 🚜 Refactor

- *(shellexpand)* Add print statements, clean up clippy warnings - ([a368a18](https://github.com/dablenparty/boxunbox/commit/a368a18d1df3936efa816c9f8faf794c5c384ba1))
- *(shellexpand)* Stub expand structure - ([40e50c8](https://github.com/dablenparty/boxunbox/commit/40e50c8e7b93b17313cd211b11e9460ef3650de6))
- *(shellexpand)* Extract sub-crate - ([cbcba95](https://github.com/dablenparty/boxunbox/commit/cbcba958196ca6c6e3d5039a56373e54bb716a9b))

### 📚 Documentation

- *(shellexpand)* Document expand - ([d16322f](https://github.com/dablenparty/boxunbox/commit/d16322f95c4971b9523f64e4f8e68efec03e61e3))
- *(shellexpand)* Document envvar regex - ([51a69c1](https://github.com/dablenparty/boxunbox/commit/51a69c1e8c7c63cdcf677187cedc9a376154342c))

### 🧪 Testing

- *(shellexpand)* Add more fallback tests - ([0ebf551](https://github.com/dablenparty/boxunbox/commit/0ebf5513dbc1418abcd6604083203e8be4e9a6d0))
- *(shellexpand)* Add more tests - ([273f164](https://github.com/dablenparty/boxunbox/commit/273f164b817c18a53570b3564b3b6e0f1395f69d))
- *(shellexpand)* Begin work on custom shellexpand - ([cca1214](https://github.com/dablenparty/boxunbox/commit/cca1214ed7bc645b595847a986f44296f5d0e0af))

### ⚙️ Miscellaneous Tasks

- Bump version (v0.1.16) - ([135dc85](https://github.com/dablenparty/boxunbox/commit/135dc8525aadfe67b072f6ff6caaf27604ed312e))

### Build

- *(Cargo.toml)* Bump ron to 0.9.0 - ([a1ef4e4](https://github.com/dablenparty/boxunbox/commit/a1ef4e4891c082084ebf8b69161e26e50c22b7ff))
- Add regex dependency - ([b4b350d](https://github.com/dablenparty/boxunbox/commit/b4b350d49ed4dbe07ac1742ddb21da28ae916efc))


## [0.1.15](https://github.com/dablenparty/boxunbox/compare/v0.1.14..v0.1.15) - 2025-03-16

### 🐛 Bug Fixes

- [**breaking**] Disable rollback - ([ce54899](https://github.com/dablenparty/boxunbox/commit/ce54899a89fcb74dc2cf522b6d2dbef8ab2ee1ad))


## [0.1.14](https://github.com/dablenparty/boxunbox/compare/v0.1.13..v0.1.14) - 2025-03-12

### 🐛 Bug Fixes

- *(plan)* Does not create target dir if root should be linked - ([0ad6d22](https://github.com/dablenparty/boxunbox/commit/0ad6d2208d3225b31f54605d4bf1d582c3bd9c0f))

### ⚙️ Miscellaneous Tasks

- Bump version (v0.1.14) - ([f0c0e3b](https://github.com/dablenparty/boxunbox/commit/f0c0e3b9c87c553a3c447ead411272bdba401dd6))


## [0.1.13](https://github.com/dablenparty/boxunbox/compare/v0.1.12..v0.1.13) - 2025-03-12

### 🐛 Bug Fixes

- [**breaking**] Create target dir by default - ([42443da](https://github.com/dablenparty/boxunbox/commit/42443da8542824a9f61597d1b7407de4614b3b5a))

### 📚 Documentation

- *(README)* Update README.md with example config - ([2f29985](https://github.com/dablenparty/boxunbox/commit/2f299855ede8ffef62764331601bb61a64381a8d))

### ⚙️ Miscellaneous Tasks

- Bump version (v0.1.13) - ([7ee138d](https://github.com/dablenparty/boxunbox/commit/7ee138d37518b9c1b6f796e442ee5af043eba6c4))

### Build

- *(Cargo.toml)* Set readme key - ([a7e4e35](https://github.com/dablenparty/boxunbox/commit/a7e4e353a1729824347a837373360a520efbe2ae))


## [0.1.12](https://github.com/dablenparty/boxunbox/compare/v0.1.11..v0.1.12) - 2025-03-04

### ⛰️  Features

- *(cli)* Add --force flag - ([9b86be7](https://github.com/dablenparty/boxunbox/commit/9b86be78cdc7a6d164979bfeab953b3d763dc018))

### ⚙️ Miscellaneous Tasks

- Bump version (v0.1.12) - ([10d4ee2](https://github.com/dablenparty/boxunbox/commit/10d4ee2f9b7d1d264267700b017179ca887221b1))


## [0.1.11](https://github.com/dablenparty/boxunbox/compare/v0.1.10..v0.1.11) - 2025-03-01

### 🐛 Bug Fixes

- *(plan)* Check for existing target when symlinking package root - ([d66699d](https://github.com/dablenparty/boxunbox/commit/d66699d48f0beca8cc82a29d44b87e3b3aed95ea))
- [**breaking**] Rc file is no longer required - ([5992752](https://github.com/dablenparty/boxunbox/commit/59927527d3926c7286f9ede7ca7b9df2dc16ecc2))

### 🚜 Refactor

- *(cli)* Derive Clone on CLI - ([5db9be6](https://github.com/dablenparty/boxunbox/commit/5db9be6a8cb009964a73797a9d16ccd1d1836f4e))
- ParseError::FileNotFound -> ParseError::ConfigNotFound - ([fdc65b3](https://github.com/dablenparty/boxunbox/commit/fdc65b39e1e77a856d1f2324f767d12d5692d522))

### 🧪 Testing

- *(rc)* Update demo rc files - ([587fe63](https://github.com/dablenparty/boxunbox/commit/587fe63ce3072149518845ade6e5e9f8fa802475))

### ⚙️ Miscellaneous Tasks

- Bump version (v0.1.11) - ([6ee711c](https://github.com/dablenparty/boxunbox/commit/6ee711cc8ca69794897508c502bcafb6cfb54a8e))


## [0.1.10](https://github.com/dablenparty/boxunbox/compare/v0.1.9..v0.1.10) - 2025-02-25

### 🐛 Bug Fixes

- [**breaking**] Ignores all rc files again - ([bfe738d](https://github.com/dablenparty/boxunbox/commit/bfe738d5d1e64894d2cb9488f46ffc0ea660edf3))

### ⚙️ Miscellaneous Tasks

- Bump version (v0.1.10) - ([04e0f80](https://github.com/dablenparty/boxunbox/commit/04e0f8019696772a9329e6cae1de59236bc3fee9))


## [0.1.9](https://github.com/dablenparty/boxunbox/compare/v0.1.7..v0.1.9) - 2025-02-25

### ⛰️  Features

- *(cli)* Add --no-create-dirs flag - ([1e71a0a](https://github.com/dablenparty/boxunbox/commit/1e71a0a262bf4b7f79066057648f881ba4e327aa))
- Support os-specific rc files - ([37bb121](https://github.com/dablenparty/boxunbox/commit/37bb1215d0e08426d00eeb3fab06ef74bc5094ae))

### 🐛 Bug Fixes

- *(plan)* Properly loads nested configs for their root packages - ([f92818c](https://github.com/dablenparty/boxunbox/commit/f92818cd14338f2f41f523383dad92c3d2f05ba7))

### ⚙️ Miscellaneous Tasks

- *(demo)* Create demo .unboxrc.macos.ron - ([d711288](https://github.com/dablenparty/boxunbox/commit/d71128823859afd532880848effa35ef73781d69))
- *(semver)* Bump version (v0.1.8) - ([c98335f](https://github.com/dablenparty/boxunbox/commit/c98335fd1cf22e522cd961c47bf174a4c8f51b4d))
- Bump version (v0.1.9) - ([c36f57c](https://github.com/dablenparty/boxunbox/commit/c36f57c16e1d43a0fd183e86ac18a4c7583d5fb9))

### Build

- Add const_format dep - ([0b766b3](https://github.com/dablenparty/boxunbox/commit/0b766b349100c7ff95a99dd4baea8114debeed2d))
- Switch to Rust 2024 Edition - ([d095860](https://github.com/dablenparty/boxunbox/commit/d0958604e83c0ff0fdd9acd436d61594b2d473fa))


## [0.1.2](https://github.com/dablenparty/boxunbox/compare/v0.1.1..v0.1.2) - 2025-02-04

### UnboxPlan

- :check_plan now validates file permissions - ([f445435](https://github.com/dablenparty/boxunbox/commit/f4454355f1441a2cb575af7ed9c3f9f99e0e57a3))


<!-- generated by git-cliff -->
