# Kalıcı menü oturumu ve canlı renk düzenleyici — 22 Ekim 2026

Kurucu SHA-256: `a9619a2b62c9c12818e5ab1a03e52d7e97fe27a44ca5d2a6be03849413f32268`.
Uygulama 1.0.0; host kurulumu, shell dosyaları, fontları ve ayarları değiştirilmedi.

## Önceki düzeltmede eksik kalanlar

- Her seçimde `ED2` ile bütün ekran yeniden siliniyordu. Frame'i dosyada
  hazırlamak bunu ortadan kaldırmamıştı. Artık yalnız ilk çizim/resize temizler;
  gezinme, liste/önizleme alanlarını günceller.
- Her `select_option` dönüşü alternatif ekranı kapatıyordu. Yeni alt menü
  açılana kadar, alttaki terminal/fastfetch ekranı kısa süreli görünebiliyordu.
  Gerçek uygulama artık baştan sona **tek alternatif ekran oturumu** tutar;
  alt menüler terminale dönmez. Çıkış/sinyalde bir kez geri yüklenir.
- Kitty görseli, eski görüntü ekranda kalırken ayrı kimlikle görünmeden yüklenir;
  hazır olunca yerleştirilir, yalnız önceki menü görselinin kimliği silinir.
  Global grafik silme yok. İlk/resize temizliği yüklemeden önce yapılır.
  Referans: [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/).
- Kitty dışındaki simge önizlemeleri yalnız kendi dikdörtgenini temizler.
  tmux/screen içinde yeni native yol yerine simge fallback kullanılır.

## Sade renk ekranı

Dört satır: **Icons**, **Text**, **Reset to Theme**, **Save & Back**.

- Yukarı/aşağı: alan seç; sağ/sol: renk değiştir. Enter alan üzerinde sonraki
  renge geçer; Reset üzerinde taslağı temaya döndürür.
- Örnek çıktı aynı panelde anında güncellenir; ayrı fastfetch önizleme sayfası,
  mod değiştirme gereği ve bekleme mesajları kaldırıldı.
- Save & Back kaydeder. Q bütün taslağı iptal eder; gezinmek dosyaya yazmaz.
- Theme ve Default her alan için bağımsız çözümlenir; altı presetin JSON'una
  literal `theme` sızmaz. Default terminal varsayılanı `39` olarak üretilir.
- Paneldeki Linux/Terminal/Memory değerleri **renk örnekleridir**, gerçek donanım
  verisi değildir. Tam preset/grup yerleşimini taklit ettiği iddia edilmez.
- 40×12 pencerede de iki renk alanına gezinilebilir ve örnek görünür kalır.

## Kanıtlar

| Kontrol | Sonuç |
| --- | --- |
| Eski kod / okla gezinme ve alt menü geçişi | [İki hata yeniden üretildi](before.log): ED2 ve alternatif tampon kapanıp açılması |
| Yeni kod / PTY siyah kutu | [21/21 geçti](regression.log) |
| Gerçek üretilen uygulama | Aynı testte ana menü → renkler → ana menü → logolar; yalnız 1 giriş/1 çıkış, gezinmede fastfetch çağrısı yok; normal çıkış ve SIGTERM terminali geri yükler |
| Renkler | Taslakta yazmama, kaydetme, iptal, reset, 40×12, altı preset × Theme/özel/Default kombinasyonları geçti |
| Kitty protokolü | Görünmeden yüklemenin kısa frame'den önce olması, yeni yerleşimin eski ID silinmeden önce gelmesi, resize ve scoped cleanup denetlenir |
| Ubuntu 24.04 + Arch | [4 senaryo / 116 kontrol geçti](docker.json): fastfetch eksik/kurulu; altı preset renklerini gerçek fastfetch ile doğrulayan yeni kontrol dahil |
| Bash ve üretilen Bash/ShellCheck | [Hata seviyesi temiz](shellcheck.log); mevcut uyarılar kayıtta |
| Gerçek grafik | [Arch/Hyprland Wayland, Kitty 0.49.2 native, fastfetch 2.69.0](native.json) |
| Zamansal örnekleme | [12/12 karede hem menü metni hem renkli görsel alanı dolu](temporal.json) |

Ara geliştirme koşusunda ilk görselin ekran temizliğiyle silinmesi gerçek
kare testiyle yakalandı: [başarısız kayıt](initial-upload-failure.log). Temizlik
sırası düzeltildi; aynı ROI/eşikler değiştirilmeden son koşu geçti.

Test emülatörü pyte 0.8, bir text run içindeki VS16 karakterinden sonra kalan
etiketi yutabiliyordu. UTF-8 artımlı çözülüp kod noktaları ayrı beslenir; assertion
gevşetilmedi. Kaynakta başlık baytlarının doğru olduğu ayrıca kontrol edildi.

## Gerçek ekranlar

- [Kaydedilmiş Red/Green](screenshots/colors-saved.png).
- [Dosyaya yazmadan canlı Blue/Green taslağı](screenshots/colors-live.png).
- [İlk görsel görünür](screenshots/images-initial.png).
- [Kare 0](screenshots/frame-00.png), [5](screenshots/frame-05.png),
  [11](screenshots/frame-11.png): Eren-1 → Eren-2 geçişi, menü ekranda kalır.

Hepsi normal test kullanıcısının geçici home'unda gerçek kurulu uygulamadan
QEMU framebuffer ile alınmıştır. Renk örnekleri sabittir; demo modu açılmadı.
Xvfb veya üretilmiş ekran kullanılmadı, pikseller rötuşlanmadı; kişisel kullanıcı
fotoğrafları/ağ bilgileri kopyalanmadı. Hash manifesti screenshots/SHA256SUMS.

## Tekrar çalıştırma

Repo kökünde mevcut Docker daemon ile:

```bash
docker run -d --name tfs-ui-check -v "$PWD:/src:ro" ubuntu:24.04 sleep 3600
docker exec tfs-ui-check bash -c 'apt-get update -qq && apt-get install -y python3 python3-pyte imagemagick gawk chafa'
docker exec --user 1000:1000 -e HOME=/tmp tfs-ui-check python3 /src/tests/ui_regression.py
docker rm -f tfs-ui-check
python3 tests/check_shell.py
python3 tests/docker_matrix.py --image ubuntu:24.04 --image archlinux:latest --scenario missing --scenario preinstalled --output /tmp/tfs-live-matrix.json
```

İzole VM'de bilinen renkli görsel listesi açıkken, pencereye uyan ROI ile:

```bash
python3 tests/vm_preview_capture.py --monitor /var/tmp/tfs-live-vm/state/monitor.sock --output /tmp/tfs-live-capture --region 715 160 475 330 --text-region 40 140 500 350
```

[VM hazırlığı](../VM_SETUP.md). Konuk önceden hazırlanmış test overlay'ini
kullanır; yeni temiz dağıtım kurulumu iddia edilmez. Ham kareler
`/var/tmp/tfs-live-vm/release-transition` altında. 12 kare örneklemdir; aradaki
her anı, bütün terminalleri veya kullanıcının CachyOS/Niri oturumunu doğrulamaz.
Konsole/GNOME Terminal/WezTerm/Ghostty/tmux için yeni gerçek grafik regresyonu
manuel doğrulama bekler. Docker sonuçları masaüstü testi sayılmaz.

Bağımsız [son kod incelemesi](review.json) kabul edildi; önemli açık bulgu yok.
