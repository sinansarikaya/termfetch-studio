# Sabit renk sırası ve Fish kaldırma — 22 Ekim 2026

Kurucu SHA-256: `a3c33c8e5122ab6f72ea748ad94c1b131dd11496d70e7bb87d4be77ddf071704`. Sürüm **1.0.0**.
Host kurulumuna, shell dosyalarına, fontlarına ve ayarlarına dokunulmadı.

## Yeniden üretilen sorunlar

- Renk örneği seçili grubu başa alıp döndürüyordu: [eski kaynakta aynı sıralama
testi başarısız](before-order.log). Dar düzende yalnız üç satır ayrılması da
beş grup için gereksiz kaydırma gerektiriyordu.
- İşaretsiz eski `alias fastfetch="termfetch-studio --preview"` kaldırma sonrası
kalıyordu. Fish alias'ı fonksiyon olarak tutar; kişisel `clear` içinden fastfetch
çağrılınca eksik uygulamaya gider: [gerçek Fish hatası](before-uninstall.log).
Host'ta yalnız bu bilinen alias satırı okundu; içerik/dosya değiştirilmedi.

## Düzeltmeler

- OS → Desktop → Hardware → Network → Time sırası sabit. Yeterli yükseklikte
beş örnek aynı anda görünür. 40×12 gibi fiziksel olarak yetersiz pencerede
sıralı ve kesintisiz üçlü görünüm vardır: Time seçilince Hardware/Network/Time;
Time/OS/Desktop gibi dönme yok. Panelin yeri alanlar arasında gezinirken sabittir.
- Yeni kaldırıcı yalnız tam eşleşen eski Fish alias'ını temizler. Bu temizlikten
önceki config kopyası `config.fish.termfetch-backup.XXXXXX` olarak korunur;
işaretli entegrasyon blokları olağan kaldırma adımında zaten çıkarılmıştır.
Kişisel clear/fonksiyonlar ve ek komut içeren özel alias'lar korunur. Eşleşme
yoksa dosya değiştirilmez; son newline bulunmayan dosya da byte olarak korunur.
- Çocuk Bash kaldırıcı mevcut parent Fish'in belleğindeki fonksiyonu silemez.
Açık Fish için `functions --erase fastfetch` komutunu gösterir. Yeni oturum
kalıcı alias satırı kaldırıldığı için normal fastfetch komutunu bulur.
Autoload edilmiş özel fonksiyon dosyaları ve pluginler otomatik silinmez.

## Sonuçlar / kanıt

| Test türü / ortam | Sonuç | Kanıt |
| --- | --- | --- |
| İzole Ubuntu 24.04 / PTY | 26/26 geçti | [regression.log](regression.log) |
| 120×24, 80×24, 40×16 | Beş grubun tümü aynı sırada; alan 0–6 gezinmesi geçti | Aynı test |
| 40×12 | Time seçimi son üçlüyü sırayla gösterir | Aynı test |
| Ubuntu / gerçek Fish, üretilmiş kaldırıcı | Yeni oturum clear, mevcut oturumun belgelenmiş temizliği, özel alias korunumu geçti | [fish-uninstall.json](fish-uninstall.json) |
| Kurucu + üretilmiş Bash / ShellCheck | Syntax geçti; hata yok, mevcut uyarılar kayıtlı | [shellcheck.log](shellcheck.log) |
| Arch / Hyprland Wayland / Kitty 0.49.2 | Geniş, dar ve çok kısa gerçek grafik kontrolü geçti | [native.json](native.json), ekranlar aşağıda |

Ara tekrarda aynı uygulama testi 150 ms dolunca üçüncü ok tuşunun çizimi henüz
gelmeyen frame'i okuyordu: [kayıt](initial-timing-failure.log). Renk assertion'ı
ve tuş sayısı korunarak, beklenen Red frame'ini en fazla 3 saniye bekleme eklendi;
son tam koşu geçti. Hata atlanmadı veya beklenen renk değiştirilmedi.

Docker sonucu masaüstü doğrulaması değildir. Bu dar takipte dağıtım matrisinin
tamamı tekrar çalıştırılmadı; önceki sonuçlar kendi tarihli raporlarında durur.
Fish testindeki native fastfetch yerine sayaç/çıktı stub'ı, komut yönlendirmesini
izole eder; gerçek fastfetch uyumluluğu bu testten çıkarılmaz.

## Gerçek ekranlar

- [Icons seçili / beş grup](screenshots/colors-top.png).
- [Time seçili / aynı sıra ve aynı konum](screenshots/colors-time.png).
- [Dar pencere / beş grup altta](screenshots/colors-narrow.png).
- [Çok kısa pencere / son üç grup sırayla](screenshots/colors-tiny.png).

Normal test kullanıcısının geçici home'unda, gerçek native Wayland Kitty'den
QEMU framebuffer ile alındı. Demo modu kapalı; renk panelinin değerleri örnek
metinlerdir, donanım ölçümü değildir. Pikseller değiştirilmedi; kullanıcı ekranları
repoya konmadı. [SHA256SUMS](screenshots/SHA256SUMS). Önceden hazırlanmış test VM
kullanıldı; temiz yeni dağıtım kurulumu iddia edilmez. Diğer grafik terminaller,
X11 ve host CachyOS/Niri oturumu bu takipte manuel doğrulama bekler.

## Tekrar çalıştırma

```bash
docker run -d --name tfs-order-check -v "$PWD:/src:ro" ubuntu:24.04 sleep infinity
docker exec tfs-order-check bash -c 'apt-get update -qq && apt-get install -y -qq python3 python3-pyte imagemagick gawk chafa'
docker exec --user 1000:1000 -e HOME=/tmp tfs-order-check python3 /src/tests/ui_regression.py
docker rm -f tfs-order-check
python3 tests/fish_uninstall.py --output /tmp/fish-uninstall.json
python3 tests/check_shell.py
```

Repo salt-okunur bağlanır; gerçek home ve Docker socket bağlanmaz. Eski kaynak
`git show 110ee90:install.sh` ile geçici dosyaya yazılıp Fish testi
`--installer DOSYA` ile aynı şekilde çalıştırılabilir. Grafik için
[VM hazırlığı](../VM_SETUP.md) ve `tests/vm_control.py`; Wayland otomasyonu
Kitty remote control + Hyprland API ile yapıldı, X11 araçları varsayılmadı.

## Mevcut bilgisayarda yapılacaklar

Düzeltmeyi kurmak: `cd ~/termfetch-studio && bash ./install.sh`.
Yeni kurulum güncel kaldırıcıyı da kurar. Mevcut açık Fish'te hata sürüyorsa
`functions --erase fastfetch` çalıştır. Uygulama zaten kaldırıldıysa ve yeniden
kurmayacaksan, `~/.config/fish/config.fish` içindeki tam eski alias satırını
elle kaldır; sadece bellekte silmek sonraki oturum için kalıcı değildir.
Kişisel `clear` fonksiyonunu silmek gerekmez. Özel autoload/plugin tanımlarında
aynı çağrı varsa bunlar kullanıcı tarafından ayrıca kontrol edilmelidir.

Değişen dosyalar: `install.sh`, `tests/ui_regression.py`, yeni
`tests/fish_uninstall.py`, README, CHANGELOG, TASKS ve bu kanıt dizini.

[Bağımsız kod incelemesi](review.json) kabul edildi; önemli açık bulgu yok.
[Son compileall kaydı](compile.json).
