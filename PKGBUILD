# Maintainer: FabioLolix
# Maintainer: éclairevoyant
# Contributor: ThatOneCalculator <kainoa at t1c dot dev>

pkgname=alany
pkgver=1
pkgrel=1
pkgdesc="nothing there"
arch=(x86_64)
license=(BSD)
conflicts=("alacritty" "alacritty-git")


options=(!docs !debug)

prepare() {
    echo lmao
}

build() {
    cd $srcdir/
}

package() {
    cd $srcdir/..
	mkdir -p $pkgdir/usr/bin
	mkdir -p $pkgdir/usr/share/applications
	cp target/release/alacritty $pkgdir/usr/bin
	cp extra/linux/Alacritty.desktop $pkgdir/usr/share/applications
}
