final: prev: {
  btinfo = final.rustPlatform.buildRustPackage {
    pname = "btinfo";
    version = "0.1.3";

    src = ./.;

    cargoLock.lockFile = ./Cargo.lock;

    meta = with final.lib; {
      description = "Bluetooth device info and notification daemon";
      homepage = "https://github.com/findus/btinfo";
      license = licenses.mit;
      maintainers = [ ];
      platforms = [ "x86_64-linux" "aarch64-linux" ];
    };
  };
}
