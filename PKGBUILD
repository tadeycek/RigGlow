# Maintainer: Tadej <tado.zurko@gmail.com>
pkgname=rigglow-git
pkgver=r0.0000000
pkgrel=1
pkgdesc="A colorful live Linux hardware fetcher"
arch=('x86_64' 'aarch64')
url="https://github.com/tadeycek/RigGlow"
license=('MIT')
depends=('gcc-libs')
makedepends=('cargo' 'git')
provides=('rigglow')
conflicts=('rigglow')
source=("$pkgname::git+$url.git")
sha256sums=('SKIP')

pkgver() {
    cd "$pkgname"
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}

prepare() {
    cd "$pkgname"
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    cd "$pkgname"
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release
}

package() {
    cd "$pkgname"
    install -Dm755 "target/release/rigglow" "$pkgdir/usr/bin/rigglow"
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
}
