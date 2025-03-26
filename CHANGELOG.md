## [0.1.20](https://github.com/dablenparty/boxunbox/compare/v0.1.19..v0.1.20) - 2025-03-26

### 🐛 Bug Fixes

- *(main)* Remove unused variable - ([54b11db](https://github.com/dablenparty/boxunbox/commit/54b11db00b5d61ac30f4bc30a25b0a490bac9154))
- *(plan)* UnboxPlan shows different info when performing box - ([72c47ef](https://github.com/dablenparty/boxunbox/commit/72c47ef19a3c5ba11a5f0cb6be917bb52cd3d069))

### 🚜 Refactor

- *(errors)* Remove execessive info from TargetAlreadyExists - ([2d0d7e7](https://github.com/dablenparty/boxunbox/commit/2d0d7e7e9eae9ee9a0dbe0eeeb9f1bb3d8bcc61d))
- *(package)* Add perform_box to config struct - ([5692570](https://github.com/dablenparty/boxunbox/commit/56925700d7f4fd41547d0f6a835d8f012a52c352))
- *(plan)* Add error type for failing to verify existence - ([fbe86c4](https://github.com/dablenparty/boxunbox/commit/fbe86c46b1ddcd40824e6ad54d507d541c0c83be))
- *(plan)* Extract UnboxPlan::execute - ([2c055fb](https://github.com/dablenparty/boxunbox/commit/2c055fb61644bf13706347dbc0ea509d3a165056))
- *(plan)* Rename execute() -> unbox() - ([3056b4d](https://github.com/dablenparty/boxunbox/commit/3056b4d4bc76f8758b16becd88528f69b2947ee8))

### 🎨 Styling

- *(errors)* Alphabetized error enum - ([76e4191](https://github.com/dablenparty/boxunbox/commit/76e4191020d2a92a172b5fc9d5e40e75ef725d0f))

### ⚙️ Miscellaneous Tasks

- Bump version (v0.1.20) - ([ed1b91c](https://github.com/dablenparty/boxunbox/commit/ed1b91cc495abb27a034b9adf16fad2258eb2fd6))

## New Contributors ❤️

* @dablenparty made their first contribution

## [0.1.19](https://github.com/dablenparty/boxunbox/compare/v0.1.18..v0.1.19) - 2025-03-26

### 🐛 Bug Fixes

- *(cli)* --color flag is back - ([eaf879d](https://github.com/dablenparty/boxunbox/commit/eaf879d24b4ad4ea6d6d50accc564e2e8e5b9c0d))
- *(plan)* No more double printing plan - ([776ec7f](https://github.com/dablenparty/boxunbox/commit/776ec7f61010174cf87b331a28514ac32a7e54cd))
- Verify package is a dir - ([6bacc30](https://github.com/dablenparty/boxunbox/commit/6bacc30dc096c32a0601789728bb2afbbcf9d625))

### 🚜 Refactor

- *(plan)* Add check for empty plan - ([6aaed3f](https://github.com/dablenparty/boxunbox/commit/6aaed3ff94e61ca84c64b41bbd3c76b7fddfe048))

### 📚 Documentation

- *(CHANGELOG)* Update CHANGELOG.md - ([40d2fd4](https://github.com/dablenparty/boxunbox/commit/40d2fd45cc8af0549995846e4b2601167e9686f5))
- Add comment - ([a5cc859](https://github.com/dablenparty/boxunbox/commit/a5cc8594275086b7f5d71edcf1e5c0771c168c16))

### 🎨 Styling

- Update imports - ([dc7462b](https://github.com/dablenparty/boxunbox/commit/dc7462bbef98408859d41c01e9cf7865fb0c0916))

### ⚙️ Miscellaneous Tasks

- Bump version (v0.1.19) - ([824bbae](https://github.com/dablenparty/boxunbox/commit/824bbaed5488d9ed24eada4ff9f90f2a136f36d1))


## [0.1.18](https://github.com/dablenparty/boxunbox/compare/v0.1.17..v0.1.18) - 2025-03-23

### ⛰️  Features

- *(cli)* Add short -d flag for --dry-run - ([2017172](https://github.com/dablenparty/boxunbox/commit/20171720382cbc735a9c1367f818b8acf586ac9f))

### 🚜 Refactor

- *(cli)* Rename color -> color_override - ([4718cd2](https://github.com/dablenparty/boxunbox/commit/4718cd209e1e08b11a3fdab381c2621c63a7c710))

### 📚 Documentation

- *(CHANGELOG)* Remove massive header - ([4e956bd](https://github.com/dablenparty/boxunbox/commit/4e956bd1f2aa7f0877d98d2665040ad74c01dff9))
- *(CHANGELOG)* Update CHANGELOG.md - ([72721b2](https://github.com/dablenparty/boxunbox/commit/72721b2902cec76feef9532a477637e26b0d0d49))
- *(cli)* Repeated documentation - ([fbf4adb](https://github.com/dablenparty/boxunbox/commit/fbf4adb3c1617e1a59562de4e5b95c36202d3a32))

### ⚙️ Miscellaneous Tasks

- Bump version (v0.1.18) - ([1215e5b](https://github.com/dablenparty/boxunbox/commit/1215e5be494f4f07e8996e8571411d722a16ab53))


## [0.1.17](https://github.com/dablenparty/boxunbox/compare/v0.1.16..v0.1.17) - 2025-03-23

### ⛰️  Features

- *(cli)* Add --color flag - ([60ddfee](https://github.com/dablenparty/boxunbox/commit/60ddfee884b45ce304da8dc76ca165631458dd92))
- *(plan)* Pretty output for UnboxPlan - ([eb9e974](https://github.com/dablenparty/boxunbox/commit/eb9e97479c23edb6ca49f38fbdcb88aaac2e0821))
- Create CHANGELOG.md with git-cliff - ([9998bd2](https://github.com/dablenparty/boxunbox/commit/9998bd23e4df6f74c87c55e2fa025afaab8738bb))

### 🐛 Bug Fixes

- *(plan)* Output now reflects config options - ([e41d676](https://github.com/dablenparty/boxunbox/commit/e41d676b6b843201a12fc3fd9365a6291d2b8643))
- *(plan)* Remove completed todo - ([d42f6e9](https://github.com/dablenparty/boxunbox/commit/d42f6e9677d8f638ba50d2e8cc2da0d95570de00))

### 🚜 Refactor

- *(package)* Add output when saving config - ([e554189](https://github.com/dablenparty/boxunbox/commit/e5541893e7604feb61ef62ec85b066a2860b90cf))
- *(package)* Remove unused field - ([550de51](https://github.com/dablenparty/boxunbox/commit/550de51c1d72376a948acffbc298059d9482be35))
- *(shellexpand)* Use directories_next for getting user HOME - ([18f73de](https://github.com/dablenparty/boxunbox/commit/18f73decf9de8dd9cb456509b98fa6930e476429))

### 📚 Documentation

- *(shellexpand)* Document envvar syntax on expand function - ([c05816d](https://github.com/dablenparty/boxunbox/commit/c05816d411325cb7694ff53958cb3274094b37f1))

### 🧪 Testing

- *(demo)* Update unbox plan - ([aab3739](https://github.com/dablenparty/boxunbox/commit/aab3739c2d8ccb5fc85514690892061ebea50230))

### ⚙️ Miscellaneous Tasks

- Bump version (v0.1.17) - ([7ced0b5](https://github.com/dablenparty/boxunbox/commit/7ced0b595f563458eb85f58fa78100aafee0d572))

### Build

- Add colored dependency - ([cf98a64](https://github.com/dablenparty/boxunbox/commit/cf98a64e5d0ee96826cfa183124b0e42276f9706))
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
