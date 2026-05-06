let
  rustOverlay = builtins.fetchGit {
    url = "https://github.com/oxalica/rust-overlay.git";
    ref = "master";
  };
  
  pkgs = import <nixpkgs> {
    overlays = [ (import rustOverlay) ];
  };
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    wayland
    libxkbcommon
    rust-bin.stable."1.93.0".default
    clang
    lld
    pkg-config
  ];

  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
      pkgs.wayland
      pkgs.libxkbcommon
      pkgs.libglvnd
      pkgs.xorg.libX11
      pkgs.xorg.libXi
    ]}:$LD_LIBRARY_PATH
  '';
}
