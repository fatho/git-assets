let
    mozilla = import ./nix/nixpkgs-mozilla-pinned.nix;
    pkgs = import ./nix/nixpkgs-pinned.nix {
        config = {};
        overlays = [
            mozilla
        ];
    };
in
    pkgs.mkShell {
        name = "git-assets-dev";
        buildInputs = [
            pkgs.rustChannels.stable.rust
        ];
    }