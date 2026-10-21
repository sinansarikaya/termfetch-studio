# UI doğrulaması — 22 Ekim 2026 (Europe/Oslo)

Kurucu SHA-256: `357cff724a10c756a7f8b858a53d84fa42af46860dfe2d052eead51889a6223b`.
Sürüm 1.0.0, yerel branch. Host kurulumu, shell dosyaları, fontlar ve kişisel
ayarlar değiştirilmedi. Kullanıcının Niri ekranları repoya kopyalanmadı.

## Sonuçlar

| Kontrol | Sonuç / kanıt |
| --- | --- |
| 80×24 eski taşma | Yeniden üretildi; [baseline-pty.json](baseline-pty.json), `screenshots/before-images.png` |
| Pikachu oranı | Eski/yeni gerçek Kitty ekranı; [aspect-measurement.json](aspect-measurement.json) |
| 80×24, 60×16, 40×12; seçimin görünürlüğü | Docker PTY'de geçti; [regression.log](regression.log) |
| Resize, sağ/sol sütun, Enter/Q/EOF, terminal/trap temizliği | Docker PTY'de geçti; aynı regresyon kaydı |
| Kaynak yükleme sessizliği, gerçek wrapper ve 6 preset JSONC | Geçti; aynı regresyon kaydı |
| Ubuntu 24.04 + Arch, fastfetch eksik/kurulu | 4/4 senaryo, 112 kontrol; [docker-smoke.json](docker-smoke.json) |
| Bash/Zsh/Fish başlangıcı | Docker PTY/sessiz başlangıç geçti; [startup.json](startup.json) |
| Kurucu + üretilen Bash sözdizimi, ShellCheck | Hata seviyesi temiz; uyarılar kayıtlı [shellcheck.log](shellcheck.log) |
| Arch/Hyprland/Wayland gerçek Kitty 0.49.2 | Normal test kullanıcısı, geçici home, gerçek uygulama kurulumu; [native-session.json](native-session.json) |

Bu koşu tüm dağıtım matrisini yeniden çalıştırmadı. Önceki uyumluluk raporunun
12 senaryosu önceki kurucu SHA'sına aittir. Yeni kod için gerçek grafik kanıtı
Arch/Hyprland/Kitty'dendir; kullanıcının CachyOS/Niri oturumuna kurulmadı.

## Gerçek ekranlar

- `before-logo.png` / `after-logo.png`: aynı kaynak Pikachu; çıktı Title/OS ile
  sınırlandırıldı, ağ bilgisi yok. Bu iki oran karşılaştırması üretim
  kütüphane/wrapper'ını çağıran izole UI harness'inden alındı.
- `before-images.png`, `before-themes.png`: önceki kütüphanedeki taşma/hizalama.
- `main-80x24.png`, `themes-80x24.png`, `themes-right.png`: normal kullanıcıyla
  kurulmuş gerçek uygulama; iki sütun, seçim ve renk önizlemesi.
- `images-80x24.png`, `images-next.png`: aynı uygulamada gerçek görsellerin
  değiştirilmesi; başlık ve seçim görünür, görsel ayrılmış alanda.
- `images-40x12.png`, `images-scroll-40x12.png`: aynı açık uygulama küçültüldü;
  önizleme gizli, listede seçili öğe görünür.
- `images-resized-back.png`: pencere büyütüldüğünde seçili öğe korunur ve görsel
  önizleme geri gelir.

Ekranlar QEMU `screendump` ile gerçek framebuffer'dan alındı. Xvfb, tarayıcı
emülasyonu veya üretilmiş ekran kullanılmadı. Uygulama demo modu kapalıydı;
konuk `tfs`, test için kullanılan ayrı home. Karelerde kişisel IP/SSID yok;
pikseller değiştirilmedi. Dosya hash'leri `screenshots/SHA256SUMS` içindedir.

## Tekrar çalıştırma

Mevcut Docker daemon ile, normal test kullanıcısı olan bir konteyner hazırlayın:

```bash
docker run -d --name tfs-ui-check -v "$PWD:/src:ro" ubuntu:24.04 sleep 3600
docker exec tfs-ui-check bash -c 'apt-get update -qq && apt-get install -y python3 python3-pyte imagemagick gawk'
docker exec --user 1000:1000 -e HOME=/tmp tfs-ui-check python3 /src/tests/ui_regression.py
docker rm -f tfs-ui-check
```

Grafik kontrolü için [VM_SETUP.md](../VM_SETUP.md) ve
[vm_control.py](../vm_control.py) kullanılabilir. Bu çalışmanın overlay ve ham
kanıtları `/var/tmp/termfetch-ui-20261021` altında; kaynak VM diski korunmuştur.

Kalan sınırlar: farklı emoji fontları/Zwj kümeleri, piksel ölçüsü vermeyen
terminallerin yaklaşık oranı, uç görsel oranlarının hücreye yuvarlanması,
Konsole/WezTerm/iTerm2/Foot'ta bu yeni UI'nin gerçek grafik regresyonu. Web,
Playwright, SEO/GEO ve mobil kapıları native Bash TUI için N/A'dır.

## Bağımsız inceleme sonrası son doğrulama

Son kurucu SHA-256: `356b78dccea09caba29a44e9e786357ee0cc29b9e58c30e4837eca878e793724`.
Yukarıdaki gerçek ekranlar ve ilk kayıtlar `357cff…` kaynağına aittir.
Sonraki düzeltmeler menü düzenini ve görsel oran algoritmasını değiştirmez:
preset renk noktaları hex yerine geçerli truecolor SGR kullanır; runtime wrapper
her çağrı için `mktemp` dosyası oluşturup başarı/hata çıkışında temizler.
Bu son iki düzeltme için yeni grafik ekranı alındığı iddia edilmez.

- Eski renk hatası: [review-bugs-before.log](review-bugs-before.log).
- Eski paylaşılan dosya hatası, renk düzeltmesinden bağımsız wrapper testi:
  [concurrent-before.log](concurrent-before.log).
- Son kod: **10/10** [regresyon](final-regression.log), üç eşzamanlı çağrı ve
  başarısız fastfetch çıkışı/temizliği dahil.
- Son kod: **4/4 senaryo, 112 kontrol** [Docker raporu](final-docker-smoke.json);
  Ubuntu 24.04 fastfetch 2.53.0, Arch fastfetch 2.70.0; digest'ler raporda.
- Son kod: [Bash/Zsh/Fish başlangıcı](final-startup.json) ve
  [Bash/ShellCheck](final-shellcheck.log); hata yok, mevcut uyarılar kayıtlı.
- Sabit geçici dosyayı arayan eski matris kontrolünün başarısız koşusu
  [saklandı](temp-path-check-failed.json). Yeni kontrol gerçek fastfetch'e verilen
  dosyayı çağrı sırasında kopyalar, gerçek fastfetch'i çalıştırır, logo
  kaynağını/türünü/iki boyutunu ve çıkışta dosya temizliğini denetler.

`source-review/` içindeki bölümler son kurucunun değişmeden bölünmüş halidir;
inceleme aracının 128 KiB okuma sınırı için tutuldu. Önceki incelemelerin araç
sınırları ve bulunan/düzeltilen iki P2 sorunu JSON kayıtlarında görülebilir.

Son bağımsız [inceleme sonucu](source-review/final-review.json) kabul edildi;
açık önemli bulgu yok. Manuel kalan alanlar yukarıda listelenmiştir.
