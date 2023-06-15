{
  description = "Deduplicated space counter";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
      let 
        systems = [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ]; 
      in {
        devShells = nixpkgs.lib.genAttrs systems (system: 
        let 
          pkgs = nixpkgs.legacyPackages.${system}; in
        {
          default =
          pkgs.mkShell {
            buildInputs = [
              pkgs.cargo
              pkgs.rustc
              pkgs.rust-analyzer
              pkgs.vscode-extensions.vadimcn.vscode-lldb
              (pkgs.writeScriptBin "dedup-count" "./target/debug/dedup-count $@")
            ];
            VSCODE_CODELLDB = "${pkgs.vscode-extensions.vadimcn.vscode-lldb}";
          };
        });

        packages = nixpkgs.lib.genAttrs systems (system: 
        let 
          pkgs = nixpkgs.legacyPackages.${system}; in
        {
          default = nixpkgs.legacyPackages.${system}.rustPlatform.buildRustPackage {
            pname = "dedup-count";
            version = "0.0.1";
            src = ./.;

            nativeBuildInputs = [
              pkgs.installShellFiles
              pkgs.makeWrapper
            ];

            buildInputs = [
            ];

            # cargoSha256 = "";
            cargoSha256 = "sha256-4gVfAu/YmH1mMqgOLvdQqpeDMcTLDf7MtdAf6qC+JjE=";
            meta = with pkgs.lib; {
              homepage = "https://github.com/WhiteBlackGoose/dedup-count";
              description = "Count amount of space, that can be saved via file-based deduplication";
              platforms = platforms.all;
              maintainers = with maintainers; [ WhiteBlackGoose ];
              license = licenses.gpl3Plus;
              mainProgram = "dedup-count";
            };
          };
      });
    };
}
