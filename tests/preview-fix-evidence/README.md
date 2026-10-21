# Logo boyutu ve önizleme düzeltmesi — 22 Ekim 2026

Kurucu SHA-256: `9cd244ec1183341ee0e62b7f8406aa38f4fdce675999d2e286f9f2244f5b24fa`.
Sürüm 1.0.0. Host uygulaması/ayarları değiştirilmedi.

## Neden ve düzeltme

- Önceki Medium 20, Large 30 sütunla sınırlıydı. Yeni Small 20×14,
  Medium 36×22, Large 52×32 üst sınırları kullanır; bunlar sabit dikdörtgene
  esnetme değildir. Kaynak/piksel oranına göre sığdırılır; terminal genişliğinin
  üçte biri ve yüksekliğinin dört satır eksiği aşılmaz. Dar pencerede farklı
  seçenekler aynı güvenli boyuta düşebilir. Artık tüm presetler aynı tercihi kullanır.
- Önce ekran temizleniyor, ardından pahalı görsel dönüşümü yapılıyordu.
  Önceki görüntü kayboluyordu. Şimdi bütün frame, grafik baytları dahil özel
  `mktemp` dosyasında hazırlanır, tamamlanınca synchronized-update içinde
  tek `cat` ile aktarılır. Grafik veri Bash değişkenine alınmaz. Menü çıkışında
  yalnız kendisine ait frame dosyası temizlenir.

## Kanıt

| Kontrol | Sonuç |
| --- | --- |
| Yavaş önizleme / eski kod | [Yeniden üretildi](before.log): READY_0 yenisi hazır olmadan kayboluyor |
| Aynı kontrol / yeni kod | [14/14 regresyon geçti](regression.log): yavaş hazırlık sırasında eski frame kalır, sonra yenisi görünür; boyutlar, oran, resize, sinyal ve dosya temizliği dahil |
| Ubuntu 24.04 ve Arch | [4 senaryo / 112 kontrol geçti](docker.json); digest, sürüm ve kurucu hash'i raporda |
| Bash + üretilen dosyalar, ShellCheck | [Hata seviyesi temiz](shellcheck.log), mevcut uyarılar kayıtlı |
| Gerçek grafik | [Native Kitty 0.49.2, Arch/Hyprland Wayland](native.json), fastfetch 2.69.0 |
| Ekran kareleri | [12 örnek](temporal.json): Eren-2 → Goku-1 geçişinde renkli fixture alanı her örnekte dolu; boş örnek yok |

12 framebuffer örneği yaklaşık 90 ms aralıklarla alındı; bu örneklem, aradaki
her anın veya bütün terminallerin hatasız olduğunu kanıtlamaz. Daha güçlü
bekleme davranışı, 600 ms gecikmeli PTY testiyle aynı eski/yeni kodda denetlendi.
Karedeki doygun piksel eşiği yalnız bilinen renkli fixture içindir; genel görüntü
desteği testi değildir.

[Large Pikachu](screenshots/logo-large.png) gerçek kurulu wrapper ile
`--structure Title:OS` kullanılarak alındı; ağ alanları gösterilmedi, demo modu
kullanılmadı. [0](screenshots/frame-00.png), [5](screenshots/frame-05.png),
[11](screenshots/frame-11.png) gerçek QEMU framebuffer kareleridir; piksel
rötuşu, üretilmiş ekran, Xvfb yok. Kullanıcının özel ekranı kopyalanmadı.

## Tekrar

[Önceki Docker hazırlığı](../ui-polish-evidence/README.md) ardından:

```bash
docker exec --user 1000:1000 -e HOME=/tmp tfs-ui-check python3 /src/tests/ui_regression.py
python3 tests/check_shell.py
python3 tests/docker_matrix.py --image ubuntu:24.04 --image archlinux:latest --scenario missing --scenario preinstalled --output /tmp/tfs-preview-matrix.json
```

Mevcut izole VM'de bilinen renkli görsel önizlemesi açıkken, pencereye uygun ROI ile:

```bash
python3 tests/vm_preview_capture.py --monitor /var/tmp/tfs-preview-vm/state/monitor.sock --output /tmp/tfs-preview-samples --region 715 160 475 330
```

VM başlatma: [VM_SETUP.md](../VM_SETUP.md). Bu koşu önceden hazırlanmış test
konuğu overlay'ini yeniden kullandı; temiz yeni dağıtım kurulumu değildir.
Ham kareler `/var/tmp/tfs-preview-vm/recorded` altında saklandı.
Diğer terminallerde yeni grafik regresyonu ve kullanıcının CachyOS/Niri ortamında
kişisel doğrulama bekliyor. Docker sonucu masaüstü doğrulaması sayılmadı.

Bağımsız [kod incelemesi](review.json) kabul edildi; önemli açık bulgu yok.
