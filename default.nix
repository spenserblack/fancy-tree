let
  nixpkgs = import <nixpkgs> { };
in
  {
    stdenv ? nixpkgs.stdenv,
    fetchzip ? nixpkgs.fetchzip,
  }: stdenv.mkDerivation {
    pname = "hello";
    version = "2.12.1";

    src = fetchzip {
      url = "https://ftp.gnu.org/gnu/hello/hello-2.12.1.tar.gz";
      sha256 = "1kJjhtlsAkpNB7f6tZEs+dbKd8z7KoNHyDHEJ0tmhnc=";
    };
  }
