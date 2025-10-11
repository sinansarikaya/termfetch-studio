# Nerd Font Kurulumu - TermFetch Studio

## Neden Nerd Font Gerekli?

TermFetch Studio'nun tüm ikonlarını düzgün görüntüleyebilmek için terminal yazı tipinizin bir **Nerd Font** olması gerekir. Nerd Font'lar özel ikonlar ve semboller içeren yazı tipleridir.

## İkonlar Görünmüyor mu?

Eğer menülerde ve önizlemelerde ikonlar yerine garip karakterler görüyorsanız, terminalinizin yazı tipini bir Nerd Font olarak ayarlamanız gerekiyor.

## Kurulum Adımları

### 1. Nerd Font Yüklü mü Kontrol Et

```bash
fc-list | grep -i "nerd"
```

Eğer bir sonuç görmüyorsanız, Nerd Font yüklenmemiş demektir.

### 2. Arch Linux / CachyOS için Kurulum

```bash
# Pacman ile yükle
sudo pacman -S ttf-jetbrains-mono-nerd

# veya
sudo pacman -S ttf-hack-nerd
```

### 3. Debian/Ubuntu/Mint için Kurulum

```bash
# Manuel kurulum
mkdir -p ~/.local/share/fonts
cd ~/.local/share/fonts
wget https://github.com/ryanoasis/nerd-fonts/releases/download/v3.1.1/JetBrainsMono.zip
unzip JetBrainsMono.zip
rm JetBrainsMono.zip
fc-cache -fv
```

### 4. Terminal Ayarları

Yazı tipi yüklendikten sonra **terminalinizin ayarlarından** yazı tipini değiştirmeniz gerekiyor:

#### GNOME Terminal / Tilix
1. Terminal ayarlarını aç (Preferences)
2. Profil seç
3. "Custom font" seçeneğini işaretle
4. Yazı tipi olarak şunlardan birini seç:
   - `JetBrainsMono Nerd Font`
   - `Hack Nerd Font`
   - `FiraCode Nerd Font`

#### Konsole (KDE)
1. Settings → Edit Current Profile
2. Appearance → Font
3. Select → Nerd Font seç

#### Alacritty
`~/.config/alacritty/alacritty.yml` dosyasını düzenle:

```yaml
font:
  normal:
    family: "JetBrainsMono Nerd Font"
  size: 12.0
```

#### Kitty
`~/.config/kitty/kitty.conf` dosyasını düzenle:

```conf
font_family      JetBrainsMono Nerd Font
bold_font        auto
italic_font      auto
bold_italic_font auto
font_size 12.0
```

### 5. Terminal'i Yeniden Başlat

Yazı tipi ayarlarını yaptıktan sonra terminali **tamamen kapatıp yeniden açın**.

## Test Et

Terminal'i yeniden başlattıktan sonra şu komutu çalıştırın:

```bash
termfetch-studio
```

Artık tüm ikonlar düzgün görünmelidir! 🎉

## Önerilen Nerd Font'lar

- **JetBrainsMono Nerd Font** - Modern, okunabilir, kod için ideal
- **Hack Nerd Font** - Temiz ve net, terminaller için popüler
- **FiraCode Nerd Font** - Ligature desteği ile şık
- **Meslo Nerd Font** - Klasik ve güvenilir

## Sorun mu Yaşıyorsunuz?

1. Font'un gerçekten yüklü olduğundan emin olun: `fc-list | grep -i "nerd"`
2. Terminal ayarlarından font'u değiştirdiğinizden emin olun
3. Terminali **tamamen** kapatıp yeniden açın (yeniden yükle değil!)
4. Hala çalışmıyorsa sistemi yeniden başlatın

## Manuel Kontrol

İkonların düzgün görünüp görünmediğini test etmek için:

```bash
echo -e "\033[1;32m ⚡ 🐧 🎨 󰣇 󰌢 󰏖 󰆍 \033[0m"
```

Bu sembollerin hepsi düzgün görünüyorsa, font kurulumu başarılıdır!
