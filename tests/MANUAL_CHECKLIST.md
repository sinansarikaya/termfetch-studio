# Gerçek terminal kapsamı ve kalan manuel kontroller

Docker sonuçları masaüstü doğrulaması değildir. Aşağıdaki grafik kontroller
izole QEMU/KVM oturumlarında yapıldı; [kanıtlar](screenshots/README.md) ve
[oturum/config kayıtları](vm-evidence/) mevcuttur.

| Dağıtım | Oturum | Terminaller | Kayıtlı temel kontroller |
| --- | --- | --- | --- |
| Ubuntu 24.04.5 | GNOME / Wayland | GNOME Terminal + Kitty | Geçti |
| Debian 13.7 | KDE Plasma / Wayland | Konsole + Kitty; ek xterm | Geçti |
| Fedora 43 | GNOME / Wayland | GNOME Terminal + Kitty | Geçti |
| Arch rolling | Hyprland / Wayland | Kitty | Geçti |
| openSUSE Tumbleweed 20261007 | KDE Plasma / Wayland | Konsole + Kitty | Konuk terminal paketleri tamamlandıktan sonra geçti |
| Ubuntu 24.04.5 | GNOME / ayrı X11 oturumu | GNOME Terminal + Kitty | Geçti |

“Geçti”: menü çizimi, ok/Enter/Q, Dracula→Nord seçimi, yeniden açılan
terminalde Nord/Bash başlangıcı, örnek Nerd Font ikonları ve fastfetch çıktısı.
Kitty/Konsole'de PNG, GNOME Terminal ve Debian xterm'de gerçek ASCII fallback
kontrol edildi. Bu ifade bütün özelliklerin/tüm temaların testi değildir.
openSUSE Kitty başlangıcında eksik `kitty-shell-integration` ve terminfo
paketlerinden kaynaklanan ilk başarısızlık [raporda](REPORT.md) korunmuştur.

## Kullanıcının manuel doğrulaması gereken kalan alanlar

- **Gerçek GPU/donanım ve ekran:** ekran sürücüsü, fiziksel donanım bilgisi,
  ölçekleme/HiDPI, çoklu monitör, gerçek GPU hızlandırması. VM'de virtio görüntü
  aygıtı kullanıldı; Docker'da bunlar doğrulanmadı.
- **Görsel oranları ve biçimleri:** Kitty 20×18 hücre PNG'yi uzatabiliyor.
  Farklı oranlı PNG, JPEG/GIF/BMP, saydamlık, büyük görsel ve animasyon kontrolleri
  manuel bekliyor. Test deseniyle PNG aktarımı doğrulandı.
- **Font kapsamı:** seçilen JetBrainsMono Nerd Font'un örnek modül ikonları
  görüldü; farklı font/sürüm/terminal ayarları ve bütün glifler doğrulanmadı.
- **Konsole menü vurguları:** bazı ANSI vurguları Kitty ile birebir eşleşmiyor.
  Arka plan ve fastfetch RGB renkleri geçti; bütün menü paleti eşitliği bekliyor.
- **Diğer temalar/özelleştirmeler:** Dracula ve Nord dışındaki temaların bütün
  renkleri, özel tema, bütün GUI ayar kombinasyonları ve özel modül düzenleri
  manuel bekliyor. Altı presetin JSONC anahtarları Docker'da doğrulandı.
- **Diğer shell/terminaller:** Zsh/Fish başlangıcı Docker PTY'de geçti; grafik
  oturumları Bash kullandı. WezTerm, iTerm2, Foot, Ghostty ve diğer terminallerin
  gerçek görüntü protokolleri manuel bekliyor.
- **Başka dağıtım/sürüm/mimari:** aarch64 paket yolu statik olarak korunuyor;
  gerçek kurulum testi yok. RHEL/CentOS/Leap ve diğer türevler/sürümler için
  paket bulunabilirliği ve davranış matrisi henüz yok. fastfetch 2.42.0 sınırı
  sentetik sürüm testidir; gerçek tam matris sürümleri raporda listelenir.

Bu kontroller için ana bilgisayara masaüstü/font kurmak gerekmiyor; mevcut
VM snapshot'ları veya ayrı gerçek test oturumu kullanılabilir.

## Kullanıcının günlük kullandığı ortam

Ana bilgisayarda salt okunur tespit (2026-10-09): CachyOS (`ID=cachyos`,
`ID_LIKE=arch`, `BUILD_ID=rolling`; sayısal `VERSION_ID` yok), Hyprland / Wayland, varsayılan shell `/bin/fish`, fastfetch
2.69.0 (x86_64). API komutu kontrol eden TTY olmadan çalıştı (`TERM=dumb`,
`tty: not a tty`). `KITTY_WINDOW_ID=1` ve açık Kitty süreçleri gözlendi; aktif
terminal bu sinyallerle kesin doğrulanamadı. Salt okunur Hyprland sorgusunda
odaklanmış pencere bir Brave tarayıcıydı; o anda odakta terminal yoktu. Kullanıcı mevcut bilgisayarda
TermFetch Studio'yu günlük kullandığını bildirdi. Bu bildirim tüm özelliklerin
test edildiği anlamına gelmez; kurulumuna ve ayarlarına dokunulmadı.
