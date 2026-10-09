# İzole grafik VM testlerini tekrar çalıştırma

Ana sisteme paket kurmayın. Mevcut `qemu-system-x86_64`, erişilebilir `/dev/kvm`,
OVMF, `qemu-img`, SSH, `xorriso` ve ekran dönüşümü için ImageMagick gerekir.
Bu koşuda her konuk 4 vCPU / 4 GiB RAM kullandı; diskler 30–40 GiB seyrek
QCOW2 overlay olarak ayrıldı. Konuk ağ bağlantısı QEMU user networking,
SSH ve VNC yalnızca `127.0.0.1` üzerinden yapıldı. Host dizinleri bağlanmadı.
Ham diskler ve anahtarlar repoya eklenmez.

## İmajlar

Aşağıdaki resmî cloud imajları indirildi. Hareketli URL'yi tekrar indirirseniz
SHA-256'yı yeniden kaydedin; aynı URL'nin aynı baytları vereceğini varsaymayın.

| Konuk | Resmî kaynak | İndirilen dosyanın SHA-256'sı |
| --- | --- | --- |
| Ubuntu 24.04 | [noble cloud](https://cloud-images.ubuntu.com/noble/current/noble-server-cloudimg-amd64.img) | `6a81c37564db9b1ee84e141922625e1d7c5b389b99bb3c572e0243607d5bb4d2` |
| Debian 13 | [trixie genericcloud](https://cloud.debian.org/images/cloud/trixie/latest/debian-13-genericcloud-amd64.qcow2) | `b2aca2bee42c7082fd6aba4a5bb1a89612087a405e99896920f3346f7b23b9d1` |
| Fedora 43 | [Cloud 43-1.6](https://dl.fedoraproject.org/pub/fedora/linux/releases/43/Cloud/x86_64/images/Fedora-Cloud-Base-Generic-43-1.6.x86_64.qcow2) | `846574c8a97cd2d8dc1f231062d73107cc85cbbbda56335e264a46e3a6c8ab2f` |
| Arch | [resmî cloud](https://geo.mirror.pkgbuild.com/images/latest/Arch-Linux-x86_64-cloudimg.qcow2) | `360f0fa49db6813bdc8e35bed230a2dc2ae3567b7b5ab74719c0a706e4e34e87` |
| openSUSE | [Tumbleweed Minimal VM](https://download.opensuse.org/tumbleweed/appliances/openSUSE-Tumbleweed-Minimal-VM.x86_64-Cloud.qcow2) | `ba2c573a5894fbd27e6d19916abae222e04daaadad7aa10e8ebface2ebaf0884` |

Cloud-init NoCloud ISO'sunda `tfs` adlı Bash kullanıcısı, yalnızca konukta
NOPASSWD sudo ve teste ait SSH public key tanımlandı. SSH private key ayrı
geçici test dizininde saklandı. Arch/openSUSE konsol hazırlığı gerektiğinde
sadece bu izole kullanıcıya test parolası verildi. Anahtar/parola paylaşmayın.

```yaml
#cloud-config
users:
  - default
  - name: tfs
    groups: [wheel] # Ubuntu/Debian için [sudo]
    shell: /bin/bash
    sudo: ["ALL=(ALL) NOPASSWD:ALL"]
    ssh_authorized_keys:
      - REPLACE_WITH_TEST_PUBLIC_KEY
ssh_pwauth: false
disable_root: true
```

`user-data` ve `meta-data` (`instance-id`, `local-hostname`) dosyalarını ISO'ya
`xorriso -as mkisofs -output seed.iso -volid cidata -joliet -rock user-data meta-data`
ile koyun. Şablondaki anahtar yer tutucusunu gerçek test public key ile değiştirin.

## Masaüstü hazırlığı (yalnızca konukta)

- Ubuntu: `sudo apt-get install ubuntu-desktop-minimal kitty`.
- Debian: `sudo apt-get install kde-plasma-desktop sddm konsole kitty linux-image-amd64`.
  Cloud kernelinde virtio GPU sürücüsü yoktu; standart amd64 kernel ile boot edildi.
- Fedora: `sudo dnf install @workstation-product-environment kitty gnome-terminal`.
- Arch: konsolda `sudo systemctl enable --now sshd`; ardından
  `sudo pacman -Syu hyprland kitty sddm xorg-xwayland mesa ttf-jetbrains-mono-nerd noto-fonts-emoji`.
  Test Hyprland ayarı: `monitor = , preferred, auto, 1` ve `exec-once = kitty`.
- openSUSE: `sudo zypper install -t pattern kde_plasma`; ardından
  `sudo zypper install sddm xorg-x11-server konsole kitty kitty-shell-integration kitty-terminfo google-noto-coloremoji-fonts curl unzip`.
  Minimal imajda KDE pattern'i görüntü yöneticisini veya Xorg greeter'ı getirmedi.
  SDDM seçimi ve `/etc/sysconfig/displaymanager` içindeki yalnızca konuk
  `DISPLAYMANAGER_AUTOLOGIN="tfs"` ayarı gerekiyordu. Oturum `plasmawayland.desktop`.

Ubuntu/Fedora'da GDM test kullanıcısına otomatik giriş; Debian/Arch'ta
`/etc/sddm.conf.d/` altında `[Autologin] User=tfs, Session=...` kullanıldı.
Bunlar ana bilgisayarda uygulanmamalıdır. GNOME/KDE oturum türünü `loginctl`
ile doğrulayın; greeter veya XWayland istemcisi oturumun türünü kanıtlamaz.
Nerd Font v3.4.0 konuk kullanıcı font dizinine kondu, terminal fontu konukta
JetBrainsMono Nerd Font seçildi. Kitty'de `linux_display_server=wayland`
ile yerel Wayland istemcisi kullanıldı.

## Snapshot, uygulama ve ekran kontrolü

Konuk kapalıyken hazırlanan masaüstü diskini temiz snapshot olarak tutun;
uygulama kurulumu için ayrı overlay oluşturun:

```bash
qemu-img create -f qcow2 -F qcow2 -b /absolute/desktop-clean.qcow2 /absolute/app.qcow2
python3 tests/vm_launch.py --disk /absolute/app.qcow2 --seed /absolute/seed.iso \
  --state /absolute/fresh-state --ssh-port 2222 --vnc-display 4
```

`install.sh` ve `preset-logos/` dizinini SSH/SCP ile konuk `tfs` home'una kopyalayın.
Kurucuyu `ssh -tt` ile normal kullanıcı olarak çalıştırın. Font sorusu çıkarsa
`Install Nerd Fonts? (Y/n)` ve son başlatma sorusuna kontrollü `n` yanıtı verildi;
font önceden hazırsa yalnızca son soru gelir. `tests/docker_matrix.py` içindeki
`interactive()` PTY yardımcı işlevi aynı kesin soruları bekleyebilir.

Gerçek grafik terminalini açın ve hazır ekranı kontrol ettikten sonra:

```bash
python3 tests/vm_control.py --monitor /absolute/fresh-state/monitor.sock \
  text:termfetch-studio key:ret wait:2 shot:/absolute/menu.png
python3 tests/vm_control.py --monitor /absolute/fresh-state/monitor.sock \
  key:ret key:down key:ret key:q key:q
```

İkinci komut ana menünün renk seçeneği odaktayken Dracula'dan Nord'a geçer.
Görünen menüyle eşleştirmeden girdileri başarılı saymayın. Tema/config dosyasını
SSH ile okuyun; terminali `exit` ile gerçekten kapatıp tekrar açın, yeni startup
çıktısını ve menüyü kaydedin. PNG logoyu aynı oturumda Kitty ve varsayılan
terminalde karşılaştırın. PNG için basit bir test deseni kullanıldı; demo modu
kapalıydı. QEMU `sendkey`/`screendump` Wayland üzerinde X11 otomasyonuna dayanmaz.

X11 karşılaştırması Ubuntu uygulama diskinden ayrı overlay'de, **konuk** GDM
`WaylandEnable=false` ile yapıldı; yeniden boot sonrası `loginctl Type=x11`
zorunludur. Donanım/gerçek ekran kartı/font çeşitleri ayrıca
[manuel listede](MANUAL_CHECKLIST.md) tutulur.
