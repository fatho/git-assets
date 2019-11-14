{ stdenv, lib, rustPlatform, ... }:
let
    repoPath = toString ./..;
    sources = map (s: repoPath + s) [
        "/src"
        "/src/hash.rs"
        "/src/store.rs"
        "/src/main.rs"
        "/Cargo.toml"
        "/Cargo.lock"
    ];
    isSource = path: type: lib.lists.elem (toString path) sources;
in
    rustPlatform.buildRustPackage rec {
        name = "git-assets-${version}";
        version = "0.2.0";

        src = builtins.filterSource isSource ./..;
        cargoSha256 = "1c4xyxl89b5r2r3l1fx9nnl6vjgf0v5kgnbnj29dg8isz2ka6hh3";

        meta = with stdenv.lib; {
            description = "A git utility for dealing with large binary assets.";
            homepage = https://github.com/fatho/git-assets;
            license = licenses.gpl3;
            maintainers = [ "Fabian Thorand" ];
            platforms = platforms.all;
        };
    }