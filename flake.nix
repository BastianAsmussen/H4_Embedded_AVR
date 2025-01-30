{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      pname = "h4-embedded";
      version = "0.1.0";
      src = ./.;

      buildInputs = with pkgs; [
        pkgsCross.avr.avrlibc
        avrdude

        # LSP Support
        clang-tools
      ];

      nativeBuildInputs = with pkgs; [
        bear
      ];
    in {
      devShells.default = pkgs.mkShell {
        inherit buildInputs nativeBuildInputs;
      };

      packages.default = pkgs.pkgsCross.avr.stdenv.mkDerivation {
        inherit buildInputs nativeBuildInputs pname version src;

        buildPhase = ''
          bear -- make
        '';

        installPhase = ''
          mkdir -p $out/{bin,share/doc}

          rm main.{c,o}

          cp main.* $out/bin
          cp compile_commands.json $out/share/doc
        '';
      };
    });
}
