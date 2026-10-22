# Grup renkleri ve tek başlangıç çıktısı — 22 Ekim 2026

Kurucu SHA-256: `b52e6510d369f62ef1383c452d4b3f65523c8813140469c363bcd7f0669191fd`. Uygulama sürümü **1.0.0**.

## Yeniden üretim ve düzeltme

Kullanıcının Fish dosyası yalnız okundu: kişisel/işaretsiz eski başlangıç
bloğunda fastfetch ve daha sonra yeni işaretli entegrasyonda bir fastfetch daha
vardı. Kullanıcı dosyaları değiştirilmedi, içerikleri/ekranları repoya alınmadı.
[Önceki kaynakla aynı test](before-startup.log), Bash rc tekrar yüklendiğinde
üç STARTUP_FETCH üreterek başarısız olur. Yeni kaynakla test aynen geçer.

- Kurucu yalnız kendi START/END bloğunu yeniler. Dışarıda doğrudan fastfetch
  çağrısı varsa kişisel bloğu koruyup kendi otomatik çağrısını atlar.
- Yönetilen Bash/Zsh/Fish blokları terminal kimliğini paylaşarak rc tekrarı ve
  Fish/Bash iç içe açılışta bir kez çalışır. Noninteractive başlangıç sessizdir.
- Algılama doğrudan komut satırları içindir; fonksiyon/plugin içindeki koşullu
  çağrıları yorumlayan bir shell parser değildir. Kişisel kodun kendi tekrarını
  otomatik silmez. Mevcut kurulum **kullanıcı yeniden kurana kadar** değişmez.

## Bağımsız incelemede bulunan örnek hatası

İlk inceleme P2 olarak Inherit örneğinin Gaming/Hardware gibi gruplarda seçili
preset yerine sabit tema pozisyonunu kullanmasını buldu. [Aynı test önce
başarısız](before-inherit.log), sonra 25 testin içinde geçti. Kaydedilen renkler
doğruydu; canlı örnek düzeltildi. Şimdi seçili generator bir kere özel mktemp
config kopyasında, grup override olmadan çalıştırılır. Örneğin ikon/metin fallback
renkleri bu gerçek preset modüllerinden okunur ve oturumda önbelleğe alınır.
Gezinmede tekrar config üretimi veya gerçek kullanıcı ayarına yazma yapılmaz;
geçici dizin subshell EXIT/sinyal temizliğindedir. İlk inceleme [kayıtlı](initial-review.json).

## Tek renk ekranı

Ana menü → **Icon & Text Colors**. Advanced Settings'teki tekrar kaldırıldı;
indeksler buna göre düzeltildi. Icons/Text bağımsız; ayrıca OS, Desktop,
Hardware, Network, Time gruplarının ikonu ve metni birlikte renklendirilebilir.
Mevcut modül sırası/boşlukları korunur; altı preset aynı semantik eşlemeyi kullanır.

↑↓ alan, ←→ renk; Enter alanda sonraki renk. Q taslağı iptal eder; Save & Back
kaydeder. Reset to Theme bütün taslağı temaya/grupları Inherit'e döndürür.
Inherit global/preset renklerini korur; Theme grubun tema rengini, Default terminal
ön planını kullanır. 15 ANSI + 16 adlandırılmış RGB rengi yerel palettedir;
dış API/renk servisi yok. RGB için terminal truecolor gerekir. Başlık ve
fastfetch yüzde göstergelerinin iç renkleri ayrıca değişmez.

## Gerçek test sonuçları

| Ortam / test türü | Sonuç | Kanıt |
| --- | --- | --- |
| İzole Ubuntu konteyneri / PTY arayüz | 25/25 geçti | [regression.log](regression.log) |
| Ubuntu 24.04 / Docker, fastfetch eksik ve kurulu | 2 × 30 kontrol geçti; fastfetch 2.53.0 | [docker.json](docker.json) |
| Arch rolling / Docker, fastfetch eksik ve kurulu | 2 × 30 kontrol geçti; fastfetch 2.70.0 | [docker.json](docker.json) |
| Bash/Zsh/Fish / Docker gerçek PTY startup | Sözdizimi, sessiz noninteractive, gerçek fetch geçti | [startup.json](startup.json) |
| Tekrar source, Fish→Bash, Bash→Fish | Tek fetch geçti | [startup.json](startup.json) |
| Eski kişisel Fish bloğu + iki yeniden entegrasyon | İçerik korundu, yalnız 1 yönetilen blok, tek fetch geçti | [startup.json](startup.json) |
| Kurucu + beş üretilen Bash dosyası | Syntax geçti, ShellCheck hata yok | [shellcheck.log](shellcheck.log) |
| Arch / Hyprland Wayland / native Kitty | Canlı beş grup, kaydetme, terminal kapanıp açılması, gerçek fastfetch geçti | [native.json](native.json), aşağıdaki ekranlar |

PTY testleri taslakta dosyaya yazmama, kaydetme/iptal/reset, RGB SGR ve kalıcılık,
altı presetin JSON modül renkleri, gerçek uygulama alt menü/çıkış/sinyal ve önceki
görsel önizleme regresyonlarını kapsar. Docker ek kontrolü altı presetin beş RGB
override ile **gerçek fastfetch tarafından** okunmasını şart koşar; kurulum,
reinstall, yedek/restore ve uninstall kontrolleri korunmuştur.

ShellCheck'te kalan SC1090 (3), SC2034 (1), SC2155 (114) uyarıları mevcut
heredoc/dinamik source ve declaration/assignment yapılarıdır; hata seviyesi temiz.
Renk örnekleri sabit örnek metindir; tam fastfetch çıktısı ayrı doğrulandı.

### İmajlar

- `ubuntu:24.04`: `ubuntu@sha256:534baea6a22c03a63003dbc8dbe78fe34bc0d7e595d9a9dc9834884ff530eb55`.
- `archlinux:latest`: `archlinux@sha256:4e77cf2ea5f410e6f8be5abf93ccf17ce2436e87a138c167208356711a405dbd`.

Bu Docker koşuları **masaüstü testi değildir**. Önceki dağıtım matrisinin
sonuçları kendi tarihli kanıtlarında kalır; bu takip çalışması yalnız yukarıdaki
iki Docker imajını ve tek grafik ortamını tekrar çalıştırdı.

## Gerçek grafik ekranları

- [Canlı, henüz kaydedilmemiş grup renkleri](screenshots/group-colors-live.png).
- [Kaydedilmiş renk ekranını yeniden açma](screenshots/group-colors-reopened.png).
- [Terminal kapatılıp yeni native Kitty açılınca kalıcılık](screenshots/terminal-reopen-persistence.png).
- [Tekrarlayan renk girişi kaldırılmış Advanced Settings](screenshots/advanced-consolidated.png).
- [Gerçek fastfetch, beş grubun farklı renkleri](screenshots/actual-group-output.png).

QEMU framebuffer'dan alınmış, rötuşlanmamış gerçek ekranlardır; Xvfb yok.
Normal test kullanıcısı, geçici test home'u, native Wayland (`xwayland=false`).
Demo modu kapalı. Arayüzün renk örneği sabit Linux/Memory/Network örnekleridir.
Gerçek çıktı ekranında tam üretilmiş Full config'in ayrı kopyasından publicip,
wifi ve title modülleri çıkarıldı, logo kapatıldı; görünen ağ adresi izole VM'nin
NAT adresidir. Yüzdeler fastfetch'in kendi renklerini korur.
Hazırlanmış test overlay'i kullanıldı, yeni temiz dağıtım kurulumu iddia edilmez.
Hashler: [SHA256SUMS](screenshots/SHA256SUMS).

## Tekrar çalıştırma

Repo kökünde, çalışan mevcut Docker daemon ile:

```bash
docker run -d --name tfs-groups-check -v "$PWD:/src:ro" ubuntu:24.04 sleep 3600
docker exec tfs-groups-check bash -c 'apt-get update -qq && apt-get install -y python3 python3-pyte imagemagick gawk chafa'
docker exec --user 1000:1000 -e HOME=/tmp tfs-groups-check python3 /src/tests/ui_regression.py
docker rm -f tfs-groups-check
python3 tests/check_shell.py
python3 tests/shell_integration.py --output /tmp/tfs-startup.json
python3 tests/docker_matrix.py --image ubuntu:24.04 --image archlinux:latest --scenario missing --scenario preinstalled --output /tmp/tfs-groups.json
```

Gerçek home/socket konteynere bağlanmaz; kurulum normal test kullanıcısıyla yapılır.
Önceki kaynakla startup hatasını tekrarlamak için `git show 010ed25:install.sh`
çıktısını geçici dosyaya yazıp `tests/shell_integration.py --installer DOSYA`
kullanılabilir; yeni test aynı assertion ile eski kaynakta başarısızdır.
Grafik test için [VM hazırlığı](../VM_SETUP.md) ve `tests/vm_control.py` kullanılır;
Wayland'da Kitty remote-control ve Hyprland API ile navigasyon yapıldı.

## Kullanıcı kurulumu ve kalan manuel kontroller

Host kurulumuna/ayarına bu çalışmada dokunulmadı. Yerel branch'teki düzeltmeyi
kurmak için kullanıcı kendi terminalinde:

```bash
cd ~/termfetch-studio
bash ./install.sh
```

Sonra yeni terminal açıp bir kez çıktı geldiğini kontrol et. Tek editörden grup
renklerini seçip kaydet; terminali yeniden aç. Kişisel başlangıç komutlarında
başka tekrar varsa otomatik silinmez; ayrı inceleme gerekir.

CachyOS/Niri günlük kullanıcı ortamı, GNOME Terminal/Konsole/WezTerm ve X11 için
bu takipte yeni gerçek grafik testi yok: manuel doğrulama bekliyor. Bu terminallerde
aynı renkler, ok/Enter/Q, dar pencere, tema ve logo geçişleri, font ikonları,
shell başlangıcı ve terminal yeniden açılınca kalıcılık kontrol edilmeli.
Tüm terminallerde titreşimsizlik veya tüm masaüstü matrisinin doğrulandığı iddia edilmez.

## Değişen dosyalar ve son kabul

- `install.sh`: başlangıç tekilleştirme, tek renk ekranı, grup/palet, preset renkleri ve özel kopyada örnek hesaplama.
- `tests/ui_regression.py`: 25 PTY regresyonu, preset örneği karşılaştırması.
- `tests/shell_integration.py`: normal/nested/re-source/legacy korunumu; isteğe bağlı kaynak ve çıktı yolu.
- `tests/docker_matrix.py`: altı preset için gerçek fastfetch RGB grup kontrolü.
- `README.md`, `CHANGELOG.md`, `TASKS.md` ve bu kanıt dizini.

[Son bağımsız inceleme](review.json) kabul edildi; önemli açık bulgu yok.
Docker / gerçek grafik sınırları yukarıda listelenmiştir. Host kişisel startup
kodundaki gizli/fonksiyon/plugin tekrarlarının otomatik tespiti garanti edilmez.
