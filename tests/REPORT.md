# Dağıtım uyumluluğu raporu (2026-10-09)

Bu rapor x86_64 Docker konteynerlerinde normal `test` kullanıcısıyla yürütülen
kurulum/temel davranış testlerini kapsar. Konteyner hazırlığı root ile, uygulama
kurulumu yalnızca konteyner içindeki `sudo` üzerinden yapıldı. Ana bilgisayarın
home dizini veya Docker socket’i konteynerlere bağlanmadı. `latest`/rolling
etiketleri için kullanılan digest’ler aşağıdadır.
İmaj digest’i tabanı sabitler; Fedora/Arch/openSUSE paket depoları ayrıca
değişebildiğinden yeniden koşuda fastfetch sürümü tekrar kaydedilmelidir.

| Dağıtım / imaj | İmaj digest’i (`sha256:`) | fastfetch yokken | Önceden kurulu iken | Test edilen fastfetch |
| --- | --- | --- | --- | --- |
| Ubuntu 24.04 (`ubuntu:24.04`) | `534baea6a22c03a63003dbc8dbe78fe34bc0d7e595d9a9dc9834884ff530eb55` | Geçti | Geçti | 2.53.0 |
| Debian 12 (`debian:12`) | `2c037a04925515fdd6ea85ea14a682d0e79931f5e9f5d07b6dbfc6ba12f9e858` | Geçti | Geçti | 2.53.0 |
| Debian 13 (`debian:13`) | `913f6706df59a68922d1dd08f78c2476560a8d367897200a6005b00e5f67c2d5` | Geçti | Geçti | 2.53.0 |
| Fedora 43 (`fedora:43`) | `a651ddf48ea28a06ed4e1e6519f51c9f47e7a5a138722ade87369b8fbb7e5b42` | Geçti | Geçti | 2.68.1 |
| Arch Linux rolling, `VERSION_ID=20261004.0.606936` (`archlinux:latest`) | `4e77cf2ea5f410e6f8be5abf93ccf17ce2436e87a138c167208356711a405dbd` | Geçti | Geçti | 2.69.0 |
| openSUSE Tumbleweed, `VERSION_ID=20261007` (`opensuse/tumbleweed:latest`) | `1086f83593526439d3e07673c9639b2111e384a26b8b63b27524d507dd86ad33` | Geçti | Geçti | 2.69.0 |

Son koşu: **12/12 senaryo, 336/336 kontrol geçti**. Kurucu SHA-256: `bfdb4c5d5825b83efdb55626ae13e3330b360a1bca555eb5792b84ae70cbd6a6`.
Her senaryoda dağıtım algılama, fastfetch başlangıç durumu, gerekli/isteğe bağlı
bağımlılıklar, kurulum dosyaları, `--version`, `--help`, `--preview`, fastfetch
yapılandırmasının çalışması, tema/preset kalıcılığı, iki arşivle yedekleme,
boşluklu dosya adıyla geri yükleme, hatalı seçim, yeniden kurulum, kaldırma ve
ilgisiz kullanıcı dosyasının korunması denetlendi. ImageMagick 6/7 ile küçük
görsel dönüştürme, terminal sinyali ve bilinmeyen terminalin metne düşmesi de
kontrol edildi. Ham sonuçlar [docker-results.json](docker-results.json) içindedir.

Tüm altı preset fastfetch `--show-errors` ile ayrıca kontrol edildi. İşlem hatalarının
maskelenmemesi için her kontrol `set -e` ile çalışır. Başlangıçta olmayan `cmp`
(test aracı) SHA-256 karşılaştırmasıyla değiştirildi; Arch ASCII beklentisi
dağıtıma ait `arch.txt` olarak düzeltildi. Uygulama hataları atlanmadı.

## Eski kodda yeniden üretilen hatalar

- Debian 12’de `wget` kurulu değilken fastfetch indirme yolu `wget: command not found` ile durdu. Son kod `curl` veya `wget` varlığını denetliyor, sabit 2.53.0 paketini SHA-256 ile doğruluyor.
- openSUSE Tumbleweed kimliği `opensuse-tumbleweed` için eski algılama `unknown` döndürdü; fastfetch yokken kurulum durdu. Fastfetch önceden kurulu olduğunda aynı eski kurulum devam etti. Son kod `suse` ailesini tanıyor.
- Türetilmiş `ID=derived` ve `ID_LIKE=debian/arch/fedora/suse` değerleriyle izole algılama kontrolü ilgili dört aileyi döndürdü; bu, türetilmiş dağıtım sürümlerinin destek testi değildir.
- Eski yedekleme ikinci arşive `./backups/` ve önceki arşivi, arşivleme sırasında kendi yeni dosya adını da dahil etti. Eski geri yükleme `a b.tar.gz` adını iki seçim olarak gösterdi; `oops` girdisi `integer expression expected` üretti.
- Sahte, oturum dışı Kitty süreci/komutu ile eski terminal algılama `kitty` döndürdü; son kod `none` döndürdü.
- İsteğe bağlı paket kurulumu başarısız olduğunda eski kod hem uyarı hem de “Optional dependencies installed” yazdı; son kod yalnızca uyarı veriyor.
- openSUSE imajında `perl` olmadığından eski kaldırma akışı yarıda kaldı; son kod yalnızca `sed` kullanıyor.
- İlk kurulumda `fastfetch.jsonc` dosyası ancak uygulama başlatıldığında oluşuyordu; son kurulum bunu hemen oluşturuyor.
- İlk düzeltilmiş openSUSE turu, imajda `find` olmadığından yedek seçimini tamamlayamadı. Bash globuyla değiştirildikten sonra iki senaryo tekrar geçti.

Eski commit `8f26432` ve son kodun aynı imajdaki karşılaştırması:
[baseline-results.json](baseline-results.json). Debian eksik indirme aracı
kurulum dökümü: [baseline-debian.json](baseline-debian.json).

### Test sırasında bulunan ek uygulama hataları

- Minimal openSUSE’de `awk` yoktu; ayar yazma ve yeniden kurulum/restore kontrolleri kaldı. Eksik gawk kurulur, yazma hatası mevcut dosyayı korur; aynı kontroller tekrar geçti.
- Debian mawk `\s` desenini desteklemiyordu; logo alanları güncellenmedi. POSIX boşluk sınıfı ile düzeltildi.
- Eksik logo source alanı içteki padding bloğuna eklenerek JSONC bozuluyordu. Alanlar logo bloğunun başında yazılır; aynı runtime logo testi altı imajda tekrar geçti.
- Geçersiz `keyShow` alanları ve boş logo türü fastfetch hataları üretti. Şablonlar düzeltildi; eski JSONC yeniden kurulumda önce birebir yedeklenerek yalnızca geçersiz alanlardan arındırılır.
- Kitty’nin ham protokolü terminal yanıtı bekleyerek takıldı; `kitty-direct` + boyutlar ile çözüldü. Konsole’de Kitty protokolü görüntüyü bozdu veya desteklenmedi; iTerm protokolü gerçek görüntüyü doğru çizdi.
- Fedora SSH/SCP başlangıcında fastfetch çıktısı kanalı bozdu. Bash/Zsh/Fish yalnızca interaktif ve TTY çıktılı shell’de başlatılır; SCP ve sessiz shell testleri tekrar geçti.
- Konsole ANSI paletine bağlı fastfetch renkleri Nord ile eşleşmedi. RGB tema renkleriyle gerçek çıktı doğrulandı; `echo=` tema yazım hataları düzeltildi.

Dağıtım ailesinin tanınması her sürümde paketin bulunacağı anlamına gelmez.
RHEL, CentOS, Linux Mint, Pop!_OS, Manjaro, EndeavourOS, CachyOS, openSUSE Leap
ve diğer sürümler Docker matrisinde doğrulanmadı. `ID_LIKE` aile fallback’i
mevcut, ancak bu adlar “test edildi” olarak işaretlenmedi. Bilinmeyen kimlik
testinde fastfetch yokken kurulum beklenen şekilde reddedildi; önceden kurulu
2.53.0 ile tamamlandı ([unknown-results.json](unknown-results.json)).

## Paket ve araç değerlendirmesi

Debian/Ubuntu yolunda tutarlı ve tekrar üretilebilir yapılandırma testi için
[upstream fastfetch 2.53.0](https://github.com/fastfetch-cli/fastfetch/releases/tag/2.53.0)
seçildi; `latest` kaldırıldı. `amd64` ve `aarch64` varlıklarının SHA-256 değerleri
kurucuda sabit; diğer mimariler yanlış paket indirmeden duruyor. Bu seçim
Debian 12 ve Ubuntu 24.04'te paket bulunabilirliğine bağlı kalmıyor. Debian 13
[kendi deposunda 2.40.4](https://packages.debian.org/trixie/fastfetch)
sunuyor; RGB biçimi için gereken [2.42.0 alt sınırının](https://github.com/fastfetch-cli/fastfetch/wiki/Color-Format-Specification) altında olduğundan kurucu açık hata ile durur. [Sentetik sürüm kontrolü](version-results.json) bu sınırı doğrular. Ayrıca Debian 13 deposunun gerçek 2.40.4 paketi normal test kullanıcısıyla denenip açık hata ile reddedildi ve mevcut fastfetch değiştirilmedi: [old-fastfetch-results.json](old-fastfetch-results.json). 2.42.0 gerçek ikiliyle tam matris çalıştırılmadı.
Önceden kurulu fastfetch değiştirilmedi. ImageMagick 6'nın `convert`/`identify`
ve 7'nin `magick` komut yolları kodda ayrıştırıldı; konteynerde küçük görsel
dönüştürmesi doğrulandı.

## Kod ve statik inceleme

`install.sh` içindeki `CORELIB` heredoc’unda `color_settings_menu`,
`set_icon_color`, `set_text_color`, `reset_to_theme_colors` gerçekten iki kez
tanımlanmıştı. Bash’te son tanımlar etkin olduğundan ilk dört tanım ve ilk
menünün yardımcı `toggle_theme_colors` işlevi kaldırıldı. Farklı üretilen
`core.sh` ve `fastfetch.sh` dosyalarında `get_theme_ansi_color` adında ayrı
tanımlar var; dosyalar alfabetik sırayla yüklendiğinde `fastfetch.sh` tanımı
etkin. Bu, aynı dosyadaki renk menüsü tekrarından ayrı bir durumdur.
`install_fastfetch_from_source` hiç çağrılmıyordu; ayrıca kaynak kod derlemek
yerine hazır arşiv indiriyordu ve güvenli olmayan `/tmp` temizliği içeriyordu.
Kaldırıldı.

`bash -n` kurucu, üretilen beş Bash dosyası ve Bash başlangıç bloğunda geçti. ShellCheck hata
seviyesinde geçti; uyarılar: SC1090 (3, dinamik kütüphane yükleme), SC2034
(2, mevcut kullanılmayan değişkenler), SC2155 (142, birleşik `local` atamaları
olası dönüş kodunu maskeliyor). Bu uyarılar kapsamlı heredoc yeniden yazımı
yapılmadan ayrı takip edilmelidir.

GitHub'daki [v1.0.0 release](https://github.com/sinansarikaya/termfetch-studio/releases/tag/v1.0.0)
2026-10-09 15:22:09 UTC tarihinde yayımlanmış, taslak veya ön sürüm değil ([API kaydı](release-status.json)). Bu çalışma
`Unreleased` altında tutuldu; uygulama sürümü ve yayımlanmış tag/release değişmedi.

## Tekrar çalıştırma

İmajlar yerelde yoksa önce `docker pull` ile `ubuntu:24.04`, `debian:12`,
`debian:13`, `fedora:43`, `archlinux:latest`, `opensuse/tumbleweed:latest` ve
`koalaman/shellcheck:stable` alın. Koşucu bulduğu digest’i kaydeder. Paket
depolarının değişebilirliği nedeniyle aynı digest tüm paket sürümlerini sabitlemez.

```bash
cd ~/termfetch-studio
python3 tests/check_shell.py
python3 tests/docker_matrix.py
python3 tests/reproduce_baseline.py
python3 tests/unknown_distro.py
python3 tests/fastfetch_version.py
python3 tests/old_fastfetch.py
python3 tests/terminal_detection.py
python3 tests/shell_integration.py
```

Tek imaj/senaryo: `python3 tests/docker_matrix.py --image debian:12 --scenario missing`.
Koşucu `docker-results.json` dosyasını günceller. Docker daemon erişimi,
paket depoları veya GitHub indirmesi engellenirse sonuç “geçti” sayılmamalı;
bu durum uygulama hatasından ayrı değerlendirilmelidir.

## Gerçek grafik VM sonuçları

Mevcut QEMU/KVM ve OVMF kullanıldı; host'a paket/masaüstü kurulmadı. Konuklar
normal `tfs` kullanıcısı ve geçici home ile çalıştı. Temiz cloud/masaüstü
QCOW2 snapshot'ları uygulama overlay'lerinden ayrıldı. Donanım hızlandırmalı
host GPU passthrough kullanılmadı. Klavye/ekran QEMU monitor üzerinden kontrol
edildi; Wayland'da X11 otomasyonunun çalıştığı varsayılmadı.

| Konuk | Gerçek oturum / terminaller | fastfetch | Sonuç |
| --- | --- | --- | --- |
| Ubuntu 24.04.5 | GNOME 46 / Wayland; GNOME Terminal 3.52.0, Kitty 0.32.2 | 2.53.0 | Aşağıdaki kayıtlı kontroller geçti |
| Debian 13.7 | Plasma 6.3.6 / Wayland; Konsole 25.04.2, Kitty 0.41.1; ek xterm 398 | 2.53.0 | Aşağıdaki kayıtlı kontroller geçti |
| Fedora 43 Cloud + workstation packages | GNOME 49.10 / Wayland; GNOME Terminal 3.56.3, Kitty 0.43.1 | 2.68.1 | Aşağıdaki kayıtlı kontroller geçti |
| Arch rolling | Hyprland 0.56.2 / Wayland; Kitty 0.49.2 | 2.69.0 | Aşağıdaki kayıtlı kontroller geçti |
| openSUSE Tumbleweed 20261007 | Plasma 6.7.5 / Wayland; Konsole 26.08.1, Kitty 0.48.2 | 2.69.0 | Aşağıdaki kayıtlı kontroller geçti; Kitty hazırlığı notuna bakın |
| Ubuntu 24.04.5 X11 karşılaştırması | GNOME 46 / X11; GNOME Terminal 3.52.0, Kitty 0.32.2 | 2.53.0 | Ayrı X11 oturumunda kayıtlı kontroller geçti |

Kayıtlı kontroller: gerçek menü çizimi, ok/Enter ile Dracula→Nord tema seçimi,
Q ile geri/çıkış, yeni açılan terminalde Nord ayarı/renklerinin kalması,
örnek Nerd Font modül ikonları, fastfetch çıktısı ve Bash başlangıcı.
Kitty'de gerçek PNG görüntülendi; Konsole'de iTerm protokolüyle görüntülendi.
GNOME Terminal'de Kitty açıkken PNG ayarı ASCII'ye düştü. Debian'da xterm
ile de gerçek ASCII fallback görüldü (bu XWayland istemcisi ayrı X11 oturumu
olarak sayılmadı). Arch/openSUSE'de auto ASCII çıktısı da gözlendi.
X11 karşılaştırmasında `loginctl Type=x11` ve fastfetch `Mutter (X11)` kanıtı var.
Kitty/varsayılan terminal karşılaştırmaları aynı konuk masaüstü oturumunda yapıldı.

Gerçek ekranlar: [screenshots](screenshots/README.md). Demo modu kullanılmadı;
PNG basit test desenidir. Dış/yerel IP satırları görüntülerden maskelendi.
Salt okunur son kurulum/oturum/config kayıtları: [vm-evidence](vm-evidence/).
Cloud imaj SHA-256'ları, hazırlık ve tekrar komutları: [VM_SETUP.md](VM_SETUP.md).

### Ortam hazırlığında çıkan engeller

- Debian cloud kernelinde virtio GPU sürücüsü yoktu; standart `linux-image-amd64` ile grafik oturumu açıldı.
- Arch cloud imajında SSH servisi açılmadı; normal test kullanıcısının konuk konsolundan etkinleştirildi.
- openSUSE minimal KDE pattern'i SDDM/Xorg greeter'ı getirmedi; konuk paketleri ve autologin hazırlandı. Greeter X11 olsa da kullanıcı oturumu Wayland olarak doğrulandı.
- openSUSE minimal Kitty kurulumunda `kitty-shell-integration`/`kitty-terminfo` eksikti. Var olmayan `ENV` dosyası nedeniyle Bash `--posix` kaldı ve `.bashrc` çalışmadı. Konukta eksik terminal paketleri tamamlandıktan sonra varsayılan başlangıç tekrar denendi; bu uygulama hatası olarak sınıflandırılmadı.
- Minimal Arch/openSUSE’de Unicode menü emoji fallback fontu eksikti; konuk Noto Color Emoji paketleri eklendi. Nerd Font modül ikonları bundan ayrı kontrol edildi.
- Oturum odak/kilit ve erken boot sırasında SSH erişimi hataları başarısız hazırlık girişimleri olarak ayrıldı; bu ekranlar başarılı uygulama kanıtı olarak kullanılmadı.

## Kalan sınırlar ve değişen dosyalar

[Manuel kontrol listesi](MANUAL_CHECKLIST.md) kalan kapsamı açıklar. Gerçek
GPU/sürücü, fiziksel donanım, ekran ölçekleme/çoklu monitör, başka fontlar,
tüm glifler, bütün temalar ve görsel dosya biçimleri tam doğrulanmadı.
Konsole’de bazı ANSI menü vurguları Kitty paletiyle birebir eşleşmiyor;
tek tek menü paleti eşitliği doğrulanmış sayılmıyor. Tema arka planı ve
fastfetch RGB renkleri gerçek çıktı ile doğrulandı.
Kitty'deki sabit 20×18 hücre boyutu PNG oranını değiştirebilir; görüntü
aktarımı doğrulandı, kusursuz oran koruma iddia edilmiyor.
Zsh/Fish başlangıcı izole Docker PTY'de geçti; gerçek grafik oturumlarında
Bash kullanıldı. aarch64 ve listelenmeyen dağıtım/fastfetch sürümleri denenmedi.

Üretilen kütüphanede eski `labels` ayarı/toggle yardımcıları kalıyor; menüde
sunulmayan bu alan için davranış desteği iddia edilmiyor. ShellCheck'in
SC1090/SC2034/SC2155 uyarıları ayrıca takip edilmeli; hata seviyesi temiz.

Değişiklikler: `install.sh`, `README.md`, `FONT_SETUP.md`, yeni `CHANGELOG.md`;
`tests/` altında Docker/PTY/regresyon/statik koşucular, VM başlatma/klavye/kanıt
araçları, JSON/log sonuçları, ekranlar ve tekrar/manuel kontrol belgeleri.
Çalışma yerel `fix/distro-compatibility` branch'indedir; push/release/tag işlemi yok.
