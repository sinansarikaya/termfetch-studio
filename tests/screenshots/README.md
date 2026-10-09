# Gerçek grafik ekran kanıtları

PNG'ler izole QEMU/KVM konuklarının gerçek framebuffer'ından `screendump` ile
alındı. QEMU monitor klavye girdileri kullanıldı; Wayland'da X11 otomasyonuna
bağımlı değil. Kullanıcı `tfs`, uygulama demo modu **kapalıydı**. Görsel logo
ImageMagick ile hazırlanmış 160×160 basit TFS test desenidir.

| Dosyalar | Oturum ve kanıt |
| --- | --- |
| `ubuntu-gnome-menu.png`, `ubuntu-gnome-themes.png` | Ubuntu 24.04.5 GNOME 46 Wayland, GNOME Terminal; menü/tema seçimi |
| `ubuntu-kitty-image.png`, `ubuntu-gnome-ascii-fallback.png` | Aynı GNOME oturumunda Kitty PNG ve GNOME Terminal ASCII fallback |
| `debian-konsole-menu.png`, `debian-konsole-persisted.png` | Debian 13.7 Plasma 6.3.6 Wayland; menü ve yeniden açılan Konsole'de Nord |
| `debian-konsole-image.png`, `debian-konsole-reopened.png`, `debian-kitty-image.png` | Konsole iTerm PNG, yeni terminalde kalıcılık, aynı KDE oturumunda Kitty PNG |
| `debian-xterm-ascii-fallback.png` | Aynı KDE Wayland oturumunda xterm ile ASCII fallback; ayrı X11 oturumu değildir |
| `fedora-gnome-menu.png`, `fedora-gnome-ascii-fallback.png`, `fedora-kitty-image.png`, `fedora-kitty-reopened.png` | Fedora 43 GNOME 49 Wayland; menü, GNOME Terminal ASCII, Kitty PNG ve yeniden açma |
| `arch-kitty-menu.png`, `arch-kitty-image.png`, `arch-kitty-reopened.png` | Arch/Hyprland 0.56.2 Wayland; Kitty menüsü, PNG, yeniden açılan terminalde Nord/image ayarı |
| `opensuse-konsole-menu.png`, `opensuse-kitty-menu.png`, `opensuse-konsole-image.png`, `opensuse-konsole-reopened.png`, `opensuse-kitty-image.png`, `opensuse-kitty-reopened.png` | Tumbleweed 20261007 Plasma 6.7.5 Wayland; Konsole ve Kitty karşılaştırması/kalıcılık |
| `ubuntu-x11-gnome-menu.png`, `ubuntu-x11-gnome-reopened.png`, `ubuntu-x11-kitty-image.png` | Ayrı Ubuntu GNOME X11 oturumu; menü, yeni GNOME Terminal ASCII/Nord ve yeni Kitty PNG. `Mutter (X11)` ve loginctl kaydı vardır |

Ağ adresleri/konum satırları düz renk maskeyle kapatıldı. xterm ekranında
arkadaki Konsole ağ satırlarının görünen kısmı da kapatıldı; maske bölgesi bazı
alt çıktı satırlarını da örter. Bunun dışında görüntü üretilmedi/değiştirilmedi.
Bütün dosyalar için [SHA-256 manifesti](SHA256SUMS) mevcuttur.

Ham ekranlar, konuk disk/snapshot'ları ve özel SSH test anahtarı repo dışında
`/var/tmp/termfetch-vm-tests` altında kaldı. Repoya yalnızca maskeli ekranlar ve
kişisel ağ bilgisi içermeyen [oturum/config kanıtları](../vm-evidence/) kondu.
Bu ekranlar yalnızca belirtilen kontrolleri/konukları doğrular; tüm özellikler,
fiziksel GPU, bütün font/görsel biçimleri için [manuel liste](../MANUAL_CHECKLIST.md) geçerlidir.
