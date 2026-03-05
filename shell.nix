{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell rec {
  buildInputs = with pkgs; [
    rustc
    rustfmt
    cargo
    clippy
    gcc
    expat
    fontconfig
    freetype
    freetype.dev
    libGL
    pkg-config
    libx11
    libxcursor
    libxi
    libxrandr
    wayland
    libxkbcommon
  ];

  LD_LIBRARY_PATH = builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;

  shellHook = ''
    echo ""
    echo "Packages loaded: gcc, cargo, rustc, rustfmt, clippy"
  '';

}
