# Terminal menüsü ve görsel oranı düzeltmesi

## Karar

Mevcut Bash yapısı korunuyor. Üç menü giriş noktası tek bir ekran çizicisini
kullanıyor. Tam TUI altyapısı/dil değişikliği bu hatalar için gerekli görünmedi:
kurulum, komutlar, presetler ve ayar biçimi aynı kaldı; yeni zorunlu uygulama
bağımlılığı eklenmedi. Daha kapsamlı bir arayüz geçişi ayrı bir kapsam ve dağıtım
matrisinin yeniden doğrulanmasını gerektirir.

## Yeniden üretilen nedenler

- Tam preset `20×18` hücreye zorlanıyordu. Kaynak PNG oranı ve terminalin hücre
  piksel ölçüsü hesaba katılmadığından Pikachu dikey uzuyordu.
- Menü bütün seçenekleri ve görseli art arda basıyordu. 80×24'te başlık/seçim
  ekran dışında kaldı; her seçimde `clear` ve yeniden taşma görüntüyü sıçrattı.
- `printf %-28s` Unicode emoji metinlerini ekran hücresine göre hizalamıyordu.
- İlk düzenleme taslağındaki kaynak seviyesinde EXIT trap'i sessiz kütüphane
  yüklemesini bozdu. Bu taslak kabul edilmedi; trap'ler menü ömrüyle sınırlandı.

## Uygulama

- Görsel ölçüsü mümkünse salt okunur `TIOCGWINSZ` piksel verisiyle hesaplanır.
  Python mevcut değilse Kitty/timeout ölçüsü, o da yoksa geleneksel 1:2 hücre
  varsayımı kullanılır. Bunlar isteğe bağlı yollar; zorunlu Python bağımlılığı yok.
- Görselin boyutları mevcut ImageMagick, file veya isteğe bağlı standart Python
  okuyucusuyla alınır. Açık, sınırlı iki boyut hem JSONC hem runtime wrapper'a
  yazılır; yalnız genişlik kullanılan deney gerçek Kitty'de zaman aşımına girdi.
- Hücreye yuvarlama ve çok uç oranlar nedeniyle kusursuz piksel eşitliği her
  fontta garanti edilmez. Mümkün olduğunda yüzde 5 içinde kalan en geniş alan
  seçilir; piksel verisi olmayan terminallerde oran yaklaşık hesaplanır.
- Menü alternate screen, sabit satır/sütun koordinatları ve senkron çizim kullanır.
  Liste kendi görünür aralığında gezinir. Dar pencerede tek sütun; 16 satırdan
  kısa pencerede önizleme gizlenir. 20×8 altı yalnız yeniden boyutlandırma/geri
  mesajıdır; görünmeyen seçenek Enter ile uygulanmaz.
- Kitty görseli ayrılmış dikdörtgene doğrudan aktarılır. Grafik protokolü Bash
  değişkenine veya `head` borusuna alınmaz. Diğer terminaller metin önizlemesi
  kullanır. Tema önizlemesi geçersiz/sessiz fastfetch yapılandırması yerine
  sınırlı renk örnekleri gösterir.
- Sağ sütun mutlak hücre konumuna sabitlenir. Metin kırpma emoji/birleşen
  karakterlerde muhafazakârdır; tam bir Unicode grapheme motoru iddiası yoktur.
- Resize tuşa basmadan algılanır. Q/Esc/EOF ve sinyallerde cursor, stty durumu,
  alternate screen ve önceki trap'ler geri bırakılır. Kütüphane yüklemek sessizdir.
