use crate::paths::Paths;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogoGroup {
    DistroAscii,
    AnimePopAscii,
    CustomAscii,
    PresetImage,
    Custom,
}

impl LogoGroup {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DistroAscii => "Distro ASCII",
            Self::AnimePopAscii => "Custom ASCII · Pop & Anime",
            Self::CustomAscii => "Custom ASCII",
            Self::PresetImage => "PNG/Image · Presets",
            Self::Custom => "PNG/Image · Custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogoKind {
    Ascii,
    PresetImage,
    CustomImage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoItem {
    pub id: String,
    pub name: String,
    pub group: LogoGroup,
    pub kind: LogoKind,
    pub art: String,
    pub file_path: Option<String>,
}

impl LogoItem {
    pub fn is_image(&self) -> bool {
        matches!(self.kind, LogoKind::PresetImage | LogoKind::CustomImage)
    }

    /// Image file on disk, if this is an image logo whose file exists.
    pub fn existing_image(&self) -> Option<&str> {
        self.file_path
            .as_deref()
            .filter(|p| self.is_image() && std::path::Path::new(p).is_file())
    }
}

/// Built-in catalogue using auto-detected asset/config locations.
pub fn get_builtin_logos() -> Vec<LogoItem> {
    builtin_logos(&Paths::detect())
}

pub fn builtin_logos(paths: &Paths) -> Vec<LogoItem> {
    let mut logos = vec![
        // ==========================================
        // 1. DISTRO ASCII LOGOS
        // ==========================================
        LogoItem {
            id: "arch".into(),
            name: "Arch Linux 📐".into(),
            group: LogoGroup::DistroAscii,
            kind: LogoKind::Ascii,
            art: r#"       /\
      /  \
     /\   \
    /      \
   /   ,,   \
  /   |  |  -\
 /_-''    ''-_\"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "tux".into(),
            name: "Tux Linux 🐧".into(),
            group: LogoGroup::DistroAscii,
            kind: LogoKind::Ascii,
            art: r#"   .---.
  /     \
  \.@-@./
  /`\_/`\
 //  _  \\
| \     )|_
/`\_`>  <_/ \
\__/'---'\__/"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "apple".into(),
            name: "Apple macOS 🍎".into(),
            group: LogoGroup::DistroAscii,
            kind: LogoKind::Ascii,
            art: r#"        .:'
    __ :'__
 .'`__`-'__``.
:__________.-'
:_________:
 :_________`-;
  `.__.-.__.'"#
                .into(),
            file_path: None,
        },
        // ==========================================
        // 2. POP CULTURE & ANIME ASCII LOGOS
        // ==========================================
        LogoItem {
            id: "pikachu-ascii".into(),
            name: "Pikachu ⚡".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#"    \.-./
   =`.-.=`
    /\__/\
   / O O  \
  =\_ Y _/="
   `'---'`"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "cat".into(),
            name: "Cat 🐱".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#" /\_/\
( o.o )
 > ^ <"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "neko".into(),
            name: "Neko Sleep 🎌".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#" |\_/|
=( °w° )=
  )   (  //
 (__ __)//"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "bunny".into(),
            name: "Bunny 🐰".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#" () ()
>(. .)<
 (") (")"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "heart".into(),
            name: "Heart 💖".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#"  **   **
 ***** *****
*************
 ***********
   *******
     ***
      *"#
            .into(),
            file_path: None,
        },
        LogoItem {
            id: "sakura".into(),
            name: "Sakura Blossom 🌸".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#"   _ _
  (_) (_)
   (_) (_)
    (_) (_)
     (_) (_)
      (_) (_)"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "naruto".into(),
            name: "Naruto Uzumaki 🍜".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#"  .-''''-.
 / -   -  \
|  @   @   |
 \   ^    /
  '-...-'"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "pokeball".into(),
            name: "Pokeball ⚾".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#"   .---.
  /  _  \
 |  (_)  |
  \ ___ /
   '---'"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "spider".into(),
            name: "Spider-Man 🕷️".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#"   / _ \
 \_\(_)/_/
  _// \\_
   /   \"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "dragon".into(),
            name: "Dragon 🐉".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#"       __====-_  _-====___
 _--^^^#####//      \\#####^^^--_
<#####*{{{{{{{  ||  }}}}}}}*#####>
     /        ||        \
    /        //\\        \"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "creeper".into(),
            name: "Creeper 💚".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#"   [■■■■■]
   [■█■█■]
   [■■█■■]
   [■███■]
   [■█■█■]"#
                .into(),
            file_path: None,
        },
        LogoItem {
            id: "star".into(),
            name: "Star ⭐".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#"      *
   *  |  *
 - - -O- - -
   *  |  *
      *"#
            .into(),
            file_path: None,
        },
        LogoItem {
            id: "rocket".into(),
            name: "Rocket 🚀".into(),
            group: LogoGroup::AnimePopAscii,
            kind: LogoKind::Ascii,
            art: r#"    /\
   |  |
  / == \
 |  ()  |
/|      |\
 |  ||  |
    /\
   /  \"#
                .into(),
            file_path: None,
        },
    ];

    // ==========================================
    // 3. HD PRESET IMAGES
    // ==========================================
    let preset_images = [
        ("pikachu.png", "Pikachu (Hi-Res PNG) ⚡"),
        ("eren-1.png", "Eren Yeager (AOT) ⚔️"),
        ("eren-2.png", "Eren Titan Form 🔥"),
        ("goku-1.png", "Son Goku (DBZ) 🥋"),
        ("goku-2.png", "Goku Super Saiyan ⚡"),
        ("goku-3.png", "Goku Ultra Instinct 🌌"),
        ("kratos.png", "Kratos (God of War) 🪓"),
        ("link-(zelda).png", "Link (Zelda) 🛡️"),
        ("luffy-1.png", "Monkey D. Luffy 🍖"),
        ("luffy-2.png", "Luffy Gear 5 🏴‍☠️"),
        ("master-chief.png", "Master Chief (Halo) 🪐"),
        ("pokeball.png", "Pokeball (Hi-Res PNG) ⚾"),
        ("sonic.png", "Sonic The Hedgehog 🦔"),
        ("tanjiro.png", "Tanjiro Kamado (Demon Slayer) 🗡️"),
    ];

    let preset_dir = paths.preset_logos_dir();

    for (file, label) in preset_images {
        let file_path = preset_dir
            .as_ref()
            .map(|dir| dir.join(file).to_string_lossy().into_owned());

        logos.push(LogoItem {
            id: file.trim_end_matches(".png").into(),
            name: label.into(),
            group: LogoGroup::PresetImage,
            kind: LogoKind::PresetImage,
            art: format!("  [Hi-Res PNG: {file}]\n  (asset directory not found)"),
            file_path,
        });
    }

    // ==========================================
    // 4. CUSTOM USER IMAGE
    // ==========================================
    let custom_path = paths.custom_image();
    logos.push(LogoItem {
        id: "custom-user-image".into(),
        name: "Custom Image (custom.png)".into(),
        group: LogoGroup::Custom,
        kind: LogoKind::CustomImage,
        art: format!(
            "  [Custom User Image]\n  Place your image at\n  {}",
            custom_path.display()
        ),
        file_path: Some(custom_path.to_string_lossy().into_owned()),
    });

    let mut files: Vec<_> = std::fs::read_dir(paths.config_dir.join("ascii"))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            matches!(
                p.extension().and_then(|s| s.to_str()),
                Some("txt" | "ascii")
            )
        })
        .collect();
    files.sort();
    for path in files {
        if let Ok(art) = std::fs::read_to_string(&path) {
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            logos.push(LogoItem {
                id: format!("custom-ascii-{name}"),
                name,
                group: LogoGroup::CustomAscii,
                kind: LogoKind::Ascii,
                art,
                file_path: None,
            });
        }
    }
    logos
}
