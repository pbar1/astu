{
  description = "Nix flake to cross-compile Rust projects";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    fenix.url = "github:nix-community/fenix";
    naersk.url = "github:nix-community/naersk";

    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      naersk,
    }:
    let
      # Each of the targets that can be cross-compiled to
      buildTargets = {
        "x86_64-linux" = {
          crossSystemConfig = "x86_64-unknown-linux-musl";
          rustTarget = "x86_64-unknown-linux-musl";
        };

        "aarch64-linux" = {
          crossSystemConfig = "aarch64-unknown-linux-musl";
          rustTarget = "aarch64-unknown-linux-musl";
        };

        "x86_64-darwin" = {
          crossSystemConfig = "x86_64-apple-darwin";
          rustTarget = "x86_64-apple-darwin";
        };

        "aarch64-darwin" = {
          crossSystemConfig = "aarch64-apple-darwin";
          rustTarget = "aarch64-apple-darwin";
        };
      };

      # eachSystem [system] (system: ...)
      #
      # Returns an attrset with a key for every system in the given array, with
      # the key's value being the result of calling the callback with that key.
      eachSystem =
        supportedSystems: callback:
        builtins.foldl' (overall: system: overall // { ${system} = callback system; }) { } supportedSystems;

      # eachCrossSystem [system] (buildSystem: targetSystem: ...)
      #
      # Returns an attrset with a key "$buildSystem.cross-$targetSystem" for
      # every combination of the elements of the array of system strings. The
      # value of the attrs will be the result of calling the callback with each
      # combination.
      #
      # There will also be keys "$system.default", which are aliases of
      # "$system.cross-$system" for every system.
      eachCrossSystem =
        supportedSystems: callback:
        eachSystem supportedSystems (
          buildSystem:
          builtins.foldl' (
            inner: targetSystem:
            inner
            // {
              "cross-${targetSystem}" = callback buildSystem targetSystem;
            }
          ) { default = callback buildSystem buildSystem; } supportedSystems
        );

      mkPkgs =
        buildSystem: targetSystem:
        import nixpkgs (
          {
            system = buildSystem;
          }
          // (
            if targetSystem == null then
              { }
            else
              {
                # The nixpkgs cache doesn't have any packages where cross-compiling has
                # been enabled, even if the target platform is actually the same as the
                # build platform (and therefore it's not really cross-compiling). So we
                # only set up the cross-compiling config if the target platform is
                # different.
                crossSystem.config = buildTargets.${targetSystem}.crossSystemConfig;
              }
          )
        );
    in
    {
      packages = eachCrossSystem (builtins.attrNames buildTargets) (
        buildSystem: targetSystem:
        let
          pkgs = mkPkgs buildSystem null;
          pkgsCross = mkPkgs buildSystem targetSystem;
          rustTarget = buildTargets.${targetSystem}.rustTarget;
          fenixPkgs = fenix.packages.${buildSystem};

          mkToolchain =
            fenixPkgs:
            fenixPkgs.toolchainOf {
              channel = "stable";
              sha256 = "sha256-yMuSb5eQPO/bHv+Bcf/US8LVMbf/G/0MSfiPwBhiPpk=";
            };

          toolchain = fenixPkgs.combine [
            (mkToolchain fenixPkgs).rustc
            (mkToolchain fenixPkgs).cargo
            (mkToolchain fenixPkgs.targets.${rustTarget}).rust-std
          ];

          buildPackageAttrs =
            if builtins.hasAttr "makeBuildPackageAttrs" buildTargets.${targetSystem} then
              buildTargets.${targetSystem}.makeBuildPackageAttrs pkgsCross
            else
              { };

          naersk' = pkgs.callPackage naersk {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        naersk'.buildPackage (
          buildPackageAttrs
          // rec {
            src = ./.;
            strictDeps = true;
            doCheck = false;

            buildInputs =
              with pkgsCross;
              [ ]
              ++ lib.optionals stdenv.isDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
              ];

            CARGO_BUILD_TARGET = rustTarget;
            CARGO_BUILD_RUSTFLAGS = [
              # https://github.com/rust-lang/cargo/issues/4133
              "-C"
              "linker=${TARGET_CC}"
            ];

            TARGET_CC = "${pkgsCross.stdenv.cc}/bin/${pkgsCross.stdenv.cc.targetPrefix}cc";

            OPENSSL_STATIC = "1";
            OPENSSL_LIB_DIR = "${pkgsCross.pkgsStatic.openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${pkgsCross.pkgsStatic.openssl.dev}/include";
          }
        )
      );
    };
}
