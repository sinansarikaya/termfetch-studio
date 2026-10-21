# Menü görünümü — 22 Ekim 2026

Kurucu SHA-256: `32b2875a78bb77a7d44a1db842c01f395c107fdafcb29b0de863b52f596f46fa`.
Sürüm 1.0.0; host kurulumu, fontları ve shell dosyaları değiştirilmedi.

## Uygulanan beş öneri

1. Seçim işareti, iki hücrelik ikon alanı ve metin ayrı koordinatlarda çizilir.
2. Büyük başlık çerçevesi kaldırıldı; tek satırlık uygulama başlığı kullanılır.
3. Menü başlığındaki tekrar eden tuş açıklamaları kaldırılır; ortak yardım alttadır.
4. Önizleme >=110 sütunda sağda, daha dar pencerede altta; <16 satırda gizlidir.
   Konumu seçimle değişmez; pencere boyutu değişirse alan yeniden hesaplanır.
5. **Logo & Image Settings → Image Size**: Small/Medium/Large. Varsayılan Medium;
   tercih config.ini `[fastfetch] logo_size` içinde kalır, runtime wrapper ve tüm
   presetlerin görsel boyut hesabında kullanılır. ASCII boyutu değişmez.
   Custom presetin modülleri logo ayarı değişirken korunur.

## Kanıtlar

| Kontrol | Sonuç |
| --- | --- |
| Docker PTY / siyah kutu | [13/13 geçti](regression.log); 120×24, 80×24, 60×16, 40×12, küçük pencere, ikon/metin sütunları, önizleme yerleşimi, klavye, resize, EOF/sinyal ve stty/trap geri yükleme |
| Resize + tek Q ile çıkış tekrarı | [5/5 geçti](resize-stress.log) |
| Logo tercihi | Üç boyutun gerçek wrapper JSONC çıktısı, kalıcılık, custom modüller ve PTY boyut menüsü; aynı regresyon kaydı |
| Ubuntu 24.04 + Arch | [4/4 senaryo, 112 kontrol](docker.json); fastfetch başlangıçta eksik/kurulu, digest ve sürümler JSON içinde |
| Shell başlangıcı | [Bash/Zsh/Fish geçti](startup.json) |
| Bash ve üretilen dosyalar / ShellCheck | [Sözdizimi ve hata seviyesi geçti](shellcheck.log), mevcut uyarılar kayıtlı |
| Gerçek grafik oturum | [Arch/Hyprland Wayland, native Kitty 0.49.2, fastfetch 2.69.0](native.json) |

Önce yakalanan [resize başarısızlığı](resize-failure.log) saklandı. İlk aşamada
Bash'in karakter okuması sırasında giriş modu/sinyal temizliği yarışıyordu.
Son kod sabit giriş modu ve kısa ham okuma grupları kullanır, sinyal temizliğini
okuma tamamlandıktan sonra yapar; aynı timeout ve aynı tek Q beklentisi korunur.
PTY okuyucusu kapanış sırasında çıktıyı tüketmeye devam eder. Emoji hizası düz
Unicode string indeksleri yerine gerçek terminal hücreleriyle doğrulanır.

## Gerçek ekran görüntüleri

QEMU framebuffer ekranları; normal `tfs` kullanıcısının geçici home'una gerçek
kurucuyla kurulan uygulama. Demo modu, Xvfb veya yapay ekran yok. Kişisel
kullanıcı ekranları kopyalanmadı, pikseller düzenlenmedi. Hash'ler screenshots/SHA256SUMS.

- [Tema ve sağ önizleme](screenshots/themes-wide.png): 120×24.
- [Görseller / sağ alan](screenshots/images-wide.png): 120×24.
- [Görseller / alt alan](screenshots/images-bottom.png): aynı pencere 80×24.
- [Küçük pencere](screenshots/images-small.png): 40×12, seçim görünür, önizleme gizli.
- [Small seçimi](screenshots/size-small.png): gerçek menüden kaydedildi.
- [Terminal kapatılıp yeniden açıldıktan sonra Large](screenshots/size-reopened.png).

## Tekrar çalıştırma

Repo kökünde, mevcut Docker daemon ile:

```bash
docker run -d --name tfs-ui-check -v "$PWD:/src:ro" ubuntu:24.04 sleep 3600
docker exec tfs-ui-check bash -c 'apt-get update -qq && apt-get install -y python3 python3-pyte imagemagick gawk'
docker exec --user 1000:1000 -e HOME=/tmp tfs-ui-check python3 /src/tests/ui_regression.py
docker rm -f tfs-ui-check
python3 tests/check_shell.py
python3 tests/docker_matrix.py --image ubuntu:24.04 --image archlinux:latest --scenario missing --scenario preinstalled --output /tmp/tfs-ui-matrix.json
```

VM tekrarı için [VM_SETUP.md](../VM_SETUP.md), [vm_launch.py](../vm_launch.py) ve
[vm_control.py](../vm_control.py). Bu koşunun izole overlay'i
`/var/tmp/tfs-polish-vm/arch.qcow2`; ana sistem home'u veya Docker socket'i konuklara bağlanmadı.

## Manuel doğrulama bekleyenler

Bu koşu bütün dağıtım/masaüstü matrisini yeniden test etmedi. Yeni UI için
GNOME Terminal, Konsole, WezTerm, Foot ve diğer emoji/font kombinasyonlarının
gerçek grafik doğrulaması bekliyor. Kullanıcının CachyOS/Niri oturumuna
kurulum yapılmadı. Uç görsel oranları hücre yuvarlamasına, piksel bilgisi vermeyen
terminaller yaklaşık hücre oranına tabidir. Donanım, font kurulumu ve masaüstü
entegrasyonu Docker sonucundan çıkarılmaz. Tüm alt menülerin bütün işlemlerinin
manuel gezildiği iddia edilmez.

Bağımsız [son inceleme](review.json) kabul edildi; önemli açık bulgu yok.
